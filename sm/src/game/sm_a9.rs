use crate::c_library::printf;
use crate::game::callback_tables::callback_table;
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
use crate::ram_map::ram_ptr;
use crate::ram_map::set_ram_words;
use crate::ram_map::{ram_word, set_ram_word, update_ram_word};
use crate::rtl::abs16;
use crate::rtl::{mov24, MemCpy, ReadRegWord, RomPtr, SnesDivide, Unreachable, WriteReg};
pub use crate::snes::registers::*;
pub use crate::types::Consts_8D;
pub use crate::types::Consts_A6;
pub use crate::types::Consts_A9;
pub use crate::types::Consts_B7;
pub use crate::types::{Consts_86, ExtraEnemyRam7800, Ram7800_Kraid};
pub use crate::types::{
    CoroutineRet, Func_V, LongPtr, OamEnt, Point16U, Rect16U, SnesRegs, VoidP, VramWriteEntry,
};
pub use crate::types::{EnemyBase, EnemyData, EnemyProps};
use ::core::ffi::{c_char, c_long};
use ::core::ptr::null_mut;
crate::game::constants::typed_constants! { Consts_B7;
    addr_kMotherBrain_Misc_TileData = 52736, addr_kDeadMonsters_TileData = 49152,
    addr_kDeadTorizo_TileData = 43008,
}
pub use crate::types::SpawnHardcodedPlmArgs;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MotherBrainInstrExecState {
    pub ip: u16,
    pub timer: u16,
}
pub use crate::types::Enemy_MotherBrain;
enemy_layout! {
    Enemy_Shitroid {
        base: EnemyBase,
        words: [
            shitr_var_A, shitr_var_B, shitr_var_C, shitr_var_D, shitr_var_E, shitr_var_F,
            shitr_parameter_1, shitr_parameter_2,
        ],
        pad1: [u8; 26696],
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
        pad1: [u8; 26696],
        words: [
            dms_var_00, dms_var_01, dms_var_02, dms_var_03, dms_var_04, dms_var_05,
            dms_var_06, dms_var_07, dms_var_08, dms_var_09, dms_var_0A, dms_var_0B,
            dms_var_0C, dms_var_0D, dms_var_0E, dms_var_0F, dms_var_10, dms_var_11,
            dms_var_12, dms_var_13, dms_var_14, dms_var_15, dms_var_16, dms_var_17,
            dms_var_18, dms_var_19, dms_var_1A, dms_var_1B, dms_var_1C, dms_var_1D,
            dms_var_1E, dms_var_1F,
        ],
        pad3: [u8; 4032],
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
        pad1: [u8; 26696],
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
        pad1: [u8; 26696],
        words: [
            sice_var_00, sice_var_01, sice_var_02, sice_var_03, sice_var_04, sice_var_05,
            sice_var_06, sice_var_07, sice_var_08, sice_var_09, sice_var_0A, sice_var_0B,
            sice_var_0C, sice_var_0D, sice_var_0E, sice_var_0F, sice_var_10, sice_var_11,
            sice_var_12, sice_var_13, sice_var_14, sice_var_15, sice_var_16, sice_var_17,
            sice_var_18, sice_var_19, sice_var_1A, sice_var_1B, sice_var_1C, sice_var_1D,
            sice_var_1E, sice_var_1F,
        ],
        pad2: [u8; 1984],
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
        pad1: [u8; 26696],
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
        pad2: [u8; 28744],
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

crate::game::constants::typed_constants! { i32;
    kCoroutineNone = 0,
}
const RomPtr_A9: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0xa9>;
const RomPtr_B7: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0xb7>;
use crate::rtl::rom_ptr_with_bank as RomPtrWithBank;
crate::ram_map::ram_pointer_functions! {
    gOamEnt(v: i32) -> OamEnt = [(0x370i32 + v) as isize];
    gVramWriteEntry(a: i32) -> VramWriteEntry = [(0xd0i32 + a) as isize];
}
crate::game::constants::typed_constants! { i32;
    fnEproj_Init_0x8bde_SkreeDownLeft = 0x868b15, fnMotherBrainsBody_UnpauseHook = 0xa98763,
    fnMotherBrainsBrain_SetupBrainAndNeckToDraw = 0xa987a2, fnMotherBrainsBrain_SetupBrainToDraw = 0xa987d0,
    fnMotherBrainsBody_FirstPhase_DoubleRet = 0xa987e1,
}
crate::game::constants::typed_constants! { u32;
    fnMotherBrainBody_FakeDeath_Descent_0_Pause = 11110429,
}
crate::game::constants::typed_constants! { i32;
    fnMotherBrainBody_FakeDeath_Descent_1 = 0xa98829, fnMotherBrainBody_FakeDeath_Descent_2 = 0xa9884d,
    fnMotherBrainBody_FakeDeath_Descent_3 = 0xa9886c, fnMotherBrainBody_FakeDeath_Descent_4 = 0xa98884,
    fnMotherBrainBody_FakeDeath_Descent_5 = 0xa988b2, fnMotherBrainBody_FakeDeath_Descent_6 = 0xa988d3,
    fnMotherBrainBody_0_Wait = 0xa98949, fnMotherBrainBody_1_ClearBottomLeftTube = 0xa9896e,
    fnMotherBrainBody_2_SpawnTopRightTubeFalling = 0xa98983,
}
crate::game::constants::typed_constants! { u32;
    fnMotherBrainBody_3_ClearCeilingBlock9 = 11110816,
}
crate::game::constants::typed_constants! { i32;
    fnMotherBrainBody_4_SpawnTopLeftTubeFalling = 0xa989b5, fnMotherBrainBody_4_ClearCeilingBlock6 = 0xa989d2,
    fnMotherBrainBody_5_SpawnTubeFallingEnemy1 = 0xa989e7, fnMotherBrainBody_6_ClearBottomRightTube = 0xa989fa,
    fnMotherBrainBody_7_SpawnTubeFallingEnemy2 = 0xa98a0f, fnMotherBrainBody_8_ClearBottomMiddleLeftTube = 0xa98a22,
    fnMotherBrainBody_9_SpawnTopMiddleLeftFalling = 0xa98a37, fnMotherBrainBody_10_ClearCeilingTubeColumn7 = 0xa98a54,
    fnMotherBrainBody_11_SpawnTopMiddleRightFalling = 0xa98a69, fnMotherBrainBody_12_ClearCeilingTubeColumn8 = 0xa98a86,
    fnMotherBrainBody_13_SpawnTubeFallingEnemy3 = 0xa98a9b, fnMotherBrainBody_14_ClearBottomMiddleRightTube = 0xa98aae,
    fnMotherBrainBody_15_SpawnTubeFallingEnemy4 = 0xa98ac3, fnMotherBrainBody_16_ClearBottomMiddleTubes = 0xa98ad6,
}
crate::game::constants::typed_constants! { u32;
    fnnullsub_365 = 11111140, fnMotherBrainsTubesFalling_Main_NonMain = 11111304,
    fnMotherBrainsTubesFalling_WaitToFall = 11111371,
}
crate::game::constants::typed_constants! { i32;
    fnMotherBrainsTubesFalling_Falling = 0xa98bd6, fnMotherBrainBody_FakeDeath_Ascent_0_DrawBG1Row23 = 0xa98c87,
    fnMotherBrainBody_FakeDeath_Ascent_1_DrawBG1Row45 = 0xa98c9e, fnMotherBrainBody_FakeDeath_Ascent_2_DrawBG1Row67 = 0xa98cb5,
    fnMotherBrainBody_FakeDeath_Ascent_3_DrawBG1Row89 = 0xa98ccc, fnMotherBrainBody_FakeDeath_Ascent_4_DrawBG1RowAB = 0xa98ce3,
    fnMotherBrainBody_FakeDeath_Ascent_5_DrawBG1RowCD = 0xa98cfa, fnMotherBrainBody_FakeDeath_Ascent_6_SetupPhase2Gfx = 0xa98d11,
    fnMotherBrainBody_FakeDeath_Ascent_7_SetupPhase2Brain = 0xa98d49, fnMotherBrainBody_FakeDeath_Ascent_8_Pause = 0xa98d79,
    fnMotherBrainBody_FakeDeath_Ascent_9_PrepareRise = 0xa98d8b, fnMotherBrainBody_FakeDeath_Ascent_10_LoadLegTiles = 0xa98db4,
    fnMotherBrainBody_FakeDeath_Ascent_11_ContinuePause = 0xa98dc3, fnMotherBrainBody_FakeDeath_Ascent_12_StartMusic = 0xa98dec,
    fnMotherBrainBody_FakeDeath_Ascent_13_Raise = 0xa98e4d, fnMotherBrainBody_FakeDeath_Ascent_14_WaitForUncouching = 0xa98e95,
    fnMotherBrainBody_FakeDeath_Ascent_15_TransitionFromGrey = 0xa98eaa, fnMotherBrainBody_2ndphase_16_ShakeHeadMenacingly = 0xa98ef5,
    fnMotherBrainBody_2ndphase_17_BringHeadBackUp = 0xa98f14, fnMotherBrainBody_2ndphase_18_FinishStretching = 0xa98f33,
}
crate::game::constants::typed_constants! { u32;
    fnMotherBrain_Instr_Goto = 11115279, fnMotherBrain_Instr_EnableNeckMovementGoto = 11115284,
    fnMotherBrain_Instr_DisableNeckMovement = 11115296, fnMotherBrain_Instr_QueueSfx2 = 11115304,
    fnMotherBrain_Instr_QueueSfx3 = 11115314, fnMotherBrain_Instr_SpawnDroolEproj = 11115324,
    fnMotherBrain_Instr_SpawnPurpleBreath = 11115373, fnMotherBrain_Instr_SetMainShakeTimer50 = 11115383,
    fnMotherBrain_Instr_GotoEitherOr = 11115621, fnMotherBrain_Instr_MaybeGoto = 11115693,
    fnMotherBrain_Instr_MaybeGoto2 = 11115789, fnMotherBrain_Instr_Goto2 = 11115809,
    fnMotherBrain_Instr_QueueShitroidAttackSfx = 11116023, fnMotherBrain_Instr_SpawnBlueRingEproj = 11116073,
    fnMotherBrain_Instr_AimRingsAtShitroid = 11116087, fnMotherBrain_Instr_AimRingsAtSamus = 11116123,
    fnMotherBrain_Instr_IncrShitroidAttackCtr = 11116195, fnMotherBrain_Instr_SetShitroidAttackCtr0 = 11116213,
    fnMotherBrain_Instr_SpawnBombEproj = 11116221, fnMotherBrain_Instr_SpawnLaserEproj = 11116358,
    fnMotherBrain_Instr_SpawnRainbowEproj = 11116420, fnMotherBrain_Instr_SetupFxForRainbowBeam = 11116430,
}
crate::game::constants::typed_constants! { i32;
    fnMotherBrain_Phase3_Death_0 = 0xa9aee1, fnMotherBrain_Phase3_Death_1 = 0xa9af12,
    fnMotherBrain_Phase3_Death_2 = 0xa9af21, fnMotherBrain_Phase3_Death_3 = 0xa9af54,
    fnMotherBrain_Phase3_Death_4 = 0xa9af9d, fnMotherBrain_Phase3_Death_5 = 0xa9afb6,
    fnMotherBrain_Phase3_Death_6 = 0xa9b013, fnMotherBrain_Phase3_Death_7 = 0xa9b115,
    fnMotherBrain_Phase3_Death_8 = 0xa9b12d, fnMotherBrain_Phase3_Death_9 = 0xa9b15e,
    fnMotherBrain_Phase3_Death_10 = 0xa9b173, fnMotherBrain_Phase3_Death_11 = 0xa9b189,
    fnMotherBrain_Phase3_Death_12 = 0xa9b1b8, fnMotherBrain_Phase3_Death_13 = 0xa9b1d5,
    fnMotherBrain_Phase3_Death_14_20framedelay = 0xa9b211,
}
crate::game::constants::typed_constants! { u32;
    fnMotherBrain_CorpseRottingFinished = 11121187,
}
crate::game::constants::typed_constants! { i32;
    fnMotherBrain_Phase3_Death_15_LoadEscapeTimerTiles = 0xa9b258, fnMotherBrain_Phase3_Death_16_StartEscape = 0xa9b26d,
    fnMotherBrain_Phase3_Death_17_SpawnTimeBomb = 0xa9b2d1, fnMotherBrain_Phase3_Death_18_TypesZebesText = 0xa9b2e3,
    fnMotherBrain_Phase3_Death_19_EscapeDoorExploding = 0xa9b2f9, fnMotherBrain_Phase3_Death_20_BlowUpEscapeDoor = 0xa9b32a,
    fnMotherBrain_Phase3_Death_21_KeepEarthquakeGoing = 0xa9b33c, fnMotherBrain_Body_Phase2_Thinking = 0xa9b605,
    fnMotherBrain_Body_Phase2_TryAttack = 0xa9b64b, fnMotherBrain_FiringBomb_DecideOnWalking = 0xa9b781,
    fnMotherBrain_FiringBomb_WalkingBackwards = 0xa9b7ac, fnMotherBrain_FiringBomb_Crouch = 0xa9b7c6,
    fnMotherBrain_FiringBomb_Fired = 0xa9b7e8, fnMotherBrain_FiringBomb_Standup = 0xa9b7f8,
    fnMotherBomb_FiringLaser_PositionHead = 0xa9b80e, fnMotherBomb_FiringLaser_PositionHeadSlowlyFire = 0xa9b839,
    fnMotherBomb_FiringLaser_FinishAttack = 0xa9b863, fnMotherBomb_FiringDeathBeam = 0xa9b87d,
    fnMotherBomb_FiringRainbowBeam_0 = 0xa9b8eb, fnMotherBomb_FiringRainbowBeam_1_StartCharge = 0xa9b91a,
    fnMotherBomb_FiringRainbowBeam_2_RetractNeck = 0xa9b92b, fnMotherBomb_FiringRainbowBeam_3_Wait = 0xa9b93f,
    fnMotherBomb_FiringRainbowBeam_4_ExtendNeckDown = 0xa9b951, fnMotherBomb_FiringRainbowBeam_5_StartFiring = 0xa9b975,
    fnMotherBomb_FiringRainbowBeam_6_MoveSamusToWall = 0xa9b9e5, fnMotherBomb_FiringRainbowBeam_7_DelayFrame = 0xa9ba00,
    fnMotherBomb_FiringRainbowBeam_8_StartDrainSamus = 0xa9ba27, fnMotherBomb_FiringRainbowBeam_9_DrainingSamus = 0xa9ba3c,
    fnMotherBomb_FiringRainbowBeam_10_FinishFiringRainbow = 0xa9ba5e, fnMotherBomb_FiringRainbowBeam_11_LetSamusFall = 0xa9bac4,
    fnMotherBomb_FiringRainbowBeam_12_WaitForSamusHitGround = 0xa9bad1, fnMotherBomb_FiringRainbowBeam_13_LowerHead = 0xa9badd,
    fnMotherBomb_FiringRainbowBeam_14_DecideNextAction = 0xa9bb06, fnMotherBrain_Phase2Cut_0 = 0xa9bd45,
    fnMotherBrain_Phase2Cut_1 = 0xa9bd98, fnMotherBrain_Phase2Cut_2 = 0xa9bda9,
    fnMotherBrain_Phase2Cut_3 = 0xa9bdc1, fnMotherBrain_Phase2Cut_4 = 0xa9bdd2,
    fnMotherBrain_Phase2Cut_5 = 0xa9bded, fnnullsub_364 = 0xa9be1a,
    fnMotherBrain_DrainedByShitroid_0 = 0xa9be38, fnMotherBrain_DrainedByShitroid_1 = 0xa9be5d,
    fnMotherBrain_DrainedByShitroid_2 = 0xa9be96, fnMotherBrain_DrainedByShitroid_3 = 0xa9bf0e,
    fnMotherBrain_DrainedByShitroid_4 = 0xa9bf41, fnMotherBrain_DrainedByShitroid_5 = 0xa9bf56,
    fnMotherBrain_DrainedByShitroid_6 = 0xa9bf7d, fnMotherBrain_DrainedByShitroid_7 = 0xa9bf95,
    fnMotherBrain_PainfulWalkForwards = 0xa9bfd0, fnMotherBrain_PainfulWalkingForwards = 0xa9bfe8,
    fnMotherBrain_PainfulWalkBackwards = 0xa9c004, fnMotherBrain_PainfulWalkingBackwards = 0xa9c01c,
    fnMotherBrain_Phase2_Revive_0 = 0xa9c059, fnMotherBrain_Phase2_Revive_1 = 0xa9c066,
    fnMotherBrain_Phase2_Revive_2 = 0xa9c082, fnMotherBrain_Phase2_Revive_3 = 0xa9c08f,
    fnMotherBrain_Phase2_Revive_4 = 0xa9c0ba, fnMotherBrain_Phase2_Revive_5 = 0xa9c0e4,
    fnMotherBrain_Phase2_Revive_6 = 0xa9c0fb, fnMotherBrain_Phase2_Revive_7 = 0xa9c11e,
    fnMotherBrain_Phase2_Revive_8 = 0xa9c147, fnMotherBrain_Phase2_MurderShitroid_1 = 0xa9c15c,
    fnMotherBrain_Phase2_MurderShitroid_2 = 0xa9c182,
}
crate::game::constants::typed_constants! { u32;
    fnnullsub_367 = 11125133,
}
crate::game::constants::typed_constants! { i32;
    fnMotherBrain_Phase2_PrepareForFinalShitroid = 0xa9c18e, fnMotherBrain_Phase2_ExecuteFinalkShitroid = 0xa9c19a,
    fnnullsub_363 = 0xa9c1a6, fnMotherBrain_Phase3_Recover_MakeDistance = 0xa9c1cf,
    fnMotherBrain_Phase3_Recover_SetupForFight = 0xa9c1f0, fnMotherBrain_Phase3_Fighting_Main = 0xa9c209,
    fnMotherBrain_Phase3_Fighting_Cooldown = 0xa9c24e, fnMotherBrain_Phase3_Walk_TryToInchForward = 0xa9c26a,
    fnMotherBrain_Phase3_Walk_RetreatQuickly = 0xa9c2b3, fnMotherBrain_Phase3_Walk_RetreatSlowly = 0xa9c2d2,
    fnMotherBrain_Phase3_Neck_Normal = 0xa9c330,
}
crate::game::constants::typed_constants! { u32;
    fnnullsub_369 = 11125587,
}
crate::game::constants::typed_constants! { i32;
    fnMotherBrain_Phase3_Neck_SetupRecoilRecovery = 0xa9c354, fnMotherBrain_Phase3_Neck_RecoilRecovery = 0xa9c37b,
    fnMotherBrain_Phase3_Neck_SetupHyperBeamRecoil = 0xa9c395, fnMotherBrain_Phase3_Neck_HyperBeamRecoil = 0xa9c3cd,
    fnShitroidInCutscene_DashOntoScreen = 0xa9c7cc, fnShitroidInCutscene_CurveTowardsBrain = 0xa9c7ec,
    fnShitroidInCutscene_GetIntoFace = 0xa9c811, fnShitroidInCutscene_LatchOntoBrain = 0xa9c851,
    fnShitroidInCutscene_SetMotherBrainToStumbleBack = 0xa9c879, fnShitroidInCutscene_ActivateRainbowBeam = 0xa9c889,
    fnShitroidInCutscene_BrainTurnsToCorpse = 0xa9c8e2, fnShitroidInCutscene_StopDraining = 0xa9c915,
    fnShitroidInCutscene_LetGoAndSpawnDust = 0xa9c94b,
}
crate::game::constants::typed_constants! { u32;
    fnShitroidInCutscene_MoveUpToCeiling = 11127129,
}
crate::game::constants::typed_constants! { i32;
    fnShitroidInCutscene_MoveToSamus = 0xa9c9c3, fnShitroidInCutscene_LatchOntoSamus = 0xa9ca66,
    fnShitroidInCutscene_HealSamusToFullHealth = 0xa9ca7a, fnnullsub_368 = 0xa9cabc,
    fnShitroidInCutscene_IdleUntilToNoHealth = 0xa9cabd, fnShitroidInCutscene_ReleaseSamus = 0xa9cb13,
    fnShitroidInCutscene_StareDownMotherBrain = 0xa9cb2d, fnShitroidInCutscene_FlyOffScreen = 0xa9cb56,
    fnShitroidInCutscene_MoveToFinalChargeStart = 0xa9cb7b, fnShitroidInCutscene_InitiateFinalCharge = 0xa9cbb3,
    fnShitroidInCutscene_FinalCharge = 0xa9cbd8, fnShitroidInCutscene_ShitroidFinalBelow = 0xa9cc3e,
    fnShitroidInCutscene_PlaySamusTheme = 0xa9cc60, fnShitroidInCutscene_PrepareSamusHyperbeam = 0xa9cc7f,
    fnnullsub_338 = 0xa9cc98, fnShitroidInCutscene_DeathSequence = 0xa9cc99,
    fnShitroidInCutscene_UnloadShitroid = 0xa9ccc0, fnShitroidInCutscene_LetSamusRainbowMore = 0xa9ccde,
    fnShitroidInCutscene_FinishCutscene = 0xa9ccf0, fnSamusRainbowPaletteFunc_ActivateWhenEnemyLow = 0xa9cd30,
    fnSamusRainbowPaletteFunc_GraduallySlowDown = 0xa9cd4b, fnDeadTorizo_WaitForSamusColl = 0xa9d3ad,
    fnnullsub_361 = 0xa9d3c7, fnDeadTorizo_PreRotDelay = 0xa9d3c8,
    fnDeadTorizo_Rotting = 0xa9d3e6,
}
crate::game::constants::typed_constants! { u32;
    fnDeadTorizo_CorpseRottingFinished = 11130301,
}
crate::game::constants::typed_constants! { i32;
    fnDeadSidehopper_Alive_WaitForActivate = 0xa9d8e2, fnDeadSidehopper_Activated = 0xa9d8f1,
    fnnullsub_362 = 0xa9d90f, fnDeadMonsters_Func_1 = 0xa9d910,
    fnDeadMonsters_Func_2 = 0xa9d91d, fnDeadMonsters_Func_5 = 0xa9da08,
    fnnullsub_268 = 0xa9da63, fnDeadSidehopper_WaitForSamusColl = 0xa9da64,
    fnDeadZoomer_WaitForSamusColl = 0xa9da69, fnDeadSkree_WaitForSamusColl = 0xa9da6e,
    fnDeadRipper_WaitForSamusColl = 0xa9da73, fnDeadSidehopper_PreRotDelay = 0xa9da8f,
    fnDeadZoomer_PreRotDelay = 0xa9da94, fnDeadRipper_PreRotDelay = 0xa9da99,
    fnDeadSkree_PreRotDelay = 0xa9da9e, fnDeadSidehopper_Rotting = 0xa9daba,
    fnDeadZoomer_Rotting = 0xa9dad0, fnDeadRipper_Rotting = 0xa9dae6,
    fnDeadSkree_Rotting = 0xa9dafc,
}
crate::game::constants::typed_constants! { u32;
    fnCorpseRottingRotEntryFinishedHook = 11131912, fnTorizo_CorpseRottingInitFunc = 11132440,
    fnSidehopper_CorpseRottingInitFunc_0 = 11132609, fnSidehopper_CorpseRottingInitFunc_2 = 11132680,
    fnZoomer_CorpseRottingInitFunc_0 = 11132751, fnZoomer_CorpseRottingInitFunc_2 = 11132780,
    fnZoomer_CorpseRottingInitFunc_4 = 11132809, fnRipper_CorpseRottingInitFunc_0 = 11132838,
    fnRipper_CorpseRottingInitFunc_2 = 11132867, fnSkree_CorpseRottingInitFunc_0 = 11132896,
    fnSkree_CorpseRottingInitFunc_2 = 11132953, fnSkree_CorpseRottingInitFunc_4 = 11133010,
    fnMotherBrain_CorpseRottingInitFunc = 11133067, fnTorizo_CorpseRottingMoveFunc = 11133554,
    fnTorizo_CorpseRottingCopyFunc = 11133835, fnSidehopper_CorpseRottingMoveFunc_0 = 11134056,
    fnSidehopper_CorpseRottingCopyFunc_0 = 11134197, fnSidehopper_CorpseRottingMoveFunc_2 = 11134308,
    fnSidehopper_CorpseRottingCopyFunc_2 = 11134454, fnZoomer_CorpseRottingMoveFunc_0 = 11134570,
    fnZoomer_CorpseRottingCopyFunc_0 = 11134649, fnZoomer_CorpseRottingMoveFunc_2 = 11134710,
    fnZoomer_CorpseRottingCopyFunc_2 = 11134789, fnZoomer_CorpseRottingMoveFunc_4 = 11134850,
    fnZoomer_CorpseRottingCopyFunc_4 = 11134929, fnRipper_CorpseRottingMoveFunc_0 = 11134990,
    fnRipper_CorpseRottingCopyFunc_0 = 11135069, fnRipper_CorpseRottingMoveFunc_2 = 11135130,
    fnRipper_CorpseRottingCopyFunc_2 = 11135209, fnSkree_CorpseRottingMoveFunc_0 = 11135270,
    fnSkree_CorpseRottingCopyFunc_0 = 11135323, fnSkree_CorpseRottingMoveFunc_2 = 11135364,
    fnSkree_CorpseRottingCopyFunc_2 = 11135417, fnSkree_CorpseRottingMoveFunc_4 = 11135458,
    fnSkree_CorpseRottingCopyFunc_4 = 11135511, fnMotherBrain_CorpseRottingMoveFunc = 11135552,
    fnMotherBrain_CorpseRottingCopyFunc = 11135755,
}
crate::game::constants::typed_constants! { i32;
    fnShitroid_Func_3 = 0xa9efdf, fnShitroid_Func_4 = 0xa9efe6,
    fnShitroid_Func_5 = 0xa9f02b, fnShitroid_Func_6 = 0xa9f037,
    fnShitroid_Func_7 = 0xa9f049, fnShitroid_Func_8 = 0xa9f06d,
    fnShitroid_Func_9 = 0xa9f094, fnShitroid_Func_10 = 0xa9f0e6,
    fnShitroid_Func_11 = 0xa9f125, fnShitroid_Func_12 = 0xa9f138,
    fnShitroid_Func_13 = 0xa9f180, fnShitroid_Func_14 = 0xa9f1fa,
    fnShitroid_Func_15 = 0xa9f20e, fnShitroid_Func_16 = 0xa9f21b,
    fnShitroid_Func_17 = 0xa9f2a2, fnShitroid_Func_18 = 0xa9f2ae,
    fnShitroid_Func_19 = 0xa9f2c0, fnShitroid_Func_20 = 0xa9f2fb,
    fnShitroid_Func_21 = 0xa9f324, fnShitroid_Func_22 = 0xa9f360,
    fnShitroid_Func_23 = 0xa9f36d, fnShitroid_Func_24 = 0xa9f3a3,
    fnShitroid_Func_25 = 0xa9f3be,
}
crate::game::constants::typed_constants! { u32;
    fnShitroid_GraduallyAccelerateTowards0x8 = 11138143, fnShitroid_GraduallyAccelerateTowards0x10 = 11138150,
}
crate::game::constants::typed_constants! { i32;
    fnShitroid_HandleCutscenePalette = 0xa9f683, fnShitroid_HandleCutscenePalette_LowHealth = 0xa9f68f,
}
crate::ram_map::ram_pointer_functions! {
    gEnemyData(o: i32) -> EnemyData = [0xf78, o as isize];
    Get_DeadEnemy(o: i32) -> Enemy_DeadEnemy = [0xf78, o as isize];
    Get_DeadMonsters(o: i32) -> Enemy_DeadMonsters = [0xf78, o as isize];
    Get_DeadSidehopper(o: i32) -> Enemy_DeadSidehopper = [0xf78, o as isize];
    Get_DeadTorizo(o: i32) -> Enemy_DeadTorizo = [0xf78, o as isize];
    Get_MotherBomb(o: i32) -> Enemy_MotherBomb = [0xf78, o as isize];
    Get_MotherBrain(o: i32) -> Enemy_MotherBrain = [0xf78, o as isize];
    Get_MotherBrainBody(o: i32) -> Enemy_MotherBrainBody = [0xf78, o as isize];
    Get_MotherBrainsTubesFalling(o: i32) -> Enemy_MotherBrainsTubesFalling = [0xf78, o as isize];
    Get_ShitdroidInCutscene(o: i32) -> Enemy_ShitroidInCutscene = [0xf78, o as isize];
    Get_Shitroid(o: i32) -> Enemy_Shitroid = [0xf78, o as isize];
    Get_ShitroidInCutscene(o: i32) -> Enemy_ShitroidInCutscene = [0xf78, o as isize];
}
static g_word_A98B5D: [u16; 5] = [0x10, 0x10, 8, 8, 0x10];
static g_word_A98B67: [u16; 5] = [0x20, 0x20, 0x18, 0x18, 0x20];
static g_word_A98B71: [u16; 5] = [0xf8, 0xf8, 0xf0, 0xf0, 0xf6];
static g_word_A98C61: [u16; 4] = [0xfff8, 2, 0xfffc, 6];
static g_byte_A98F7D: [u8; 2] = [9, 0x12];
static g_word_A98F7F: [u16; 8] = [0x3d, 0x54, 0x20, 0x35, 0x5a, 0x43, 0x67, 0x29];
static g_word_A993BB: [i16; 4] = [0, -1, 0, 1];
static g_word_A993C3: [i16; 4] = [0, 1, -1, 1];
static g_word_A99E0F: [u16; 13] = [
    0x6f, 0x6f, 0x6f, 0x7e, 0x6f, 0x6f, 0x7e, 0x6f, 0x6f, 0x7e, 0x7e, 0x6f, 0x6f,
];
static g_word_A9B393: [u16; 8] = [8, 0x6c, 0x18, 0x80, 9, 0x90, 0x18, 0x74];
static g_byte_A9B546: [u8; 8] = [0, 1, 1, 0, 0, 0, 0, 0];
static g_byte_A9B5A1: [u8; 8] = [2, 1, 1, 0, 0, 0, 0, 0];
static g_word_A9B72C: [i16; 28] = [
    -1, -17, 0, 0, -1, -1, 0, -1, 0, 0, -1, 0, 0, 0, -17, -1, -1, -1, 0, 0, 0, -17, -1, -1, -1, -1,
    -1, -1,
];
static g_word_A9BCA6: [i16; 8] = [-8, 6, -4, 2, 3, -6, 8, 0];
static g_word_A9BCB6: [i16; 8] = [-7, 2, 5, -4, 6, -2, -6, 7];
static g_byte_A9BEEE: [u8; 16] = [2, 0, 2, 0, 6, 0, 6, 0, 8, 0, 8, 0, 10, 0, 10, 0];
static g_word_A9BEFE: [u16; 8] = [0x500, 0x500, 0x200, 0x200, 0xc0, 0xc0, 0x40, 0x40];
static g_word_A9C049: [i16; 8] = [0x10, 0x10, 0x20, 0x20, 0x30, 0x30, 0x40, 0x40];
static g_word_A9C544: u16 = 1u16;
static g_byte_A9F56A: [i8; 16] = [
    0x10, 0xf, 0xe, 0xd, 0xc, 0xb, 0xa, 9, 8, 7, 6, 5, 4, 3, 2, 1,
];
crate::game::address_dispatch! {
    pub unsafe fn CallCorpseRottingMove(ea: u32, k: u16, j: u16, game: &mut Game) => ea, game {
        11133835 => Torizo_CorpseRottingCopyFunc(k, j, game),
        11134197 => Sidehopper_CorpseRottingCopyFunc_0(k, j, game),
        11134454 => Sidehopper_CorpseRottingCopyFunc_2(k, j, game),
        11134649 => Zoomer_CorpseRottingCopyFunc_0(k, j, game),
        11134789 => Zoomer_CorpseRottingCopyFunc_2(k, j, game),
        11134929 => Zoomer_CorpseRottingCopyFunc_4(k, j, game),
        11135069 => Ripper_CorpseRottingCopyFunc_0(k, j, game),
        11135209 => Ripper_CorpseRottingCopyFunc_2(k, j, game),
        11135323 => Skree_CorpseRottingCopyFunc_0(k, j, game),
        11135417 => Skree_CorpseRottingCopyFunc_2(k, j, game),
        11135511 => Skree_CorpseRottingCopyFunc_4(k, j, game),
        11135755 => MotherBrain_CorpseRottingCopyFunc(k, j, game),
        11133554 => Torizo_CorpseRottingMoveFunc(k, j, game),
        11134056 => Sidehopper_CorpseRottingMoveFunc_0(k, j, game),
        11134308 => Sidehopper_CorpseRottingMoveFunc_2(k, j, game),
        11134570 => Zoomer_CorpseRottingMoveFunc_0(k, j, game),
        11134710 => Zoomer_CorpseRottingMoveFunc_2(k, j, game),
        11134850 => Zoomer_CorpseRottingMoveFunc_4(k, j, game),
        11134990 => Ripper_CorpseRottingMoveFunc_0(k, j, game),
        11135130 => Ripper_CorpseRottingMoveFunc_2(k, j, game),
        11135270 => Skree_CorpseRottingMoveFunc_0(k, j, game),
        11135364 => Skree_CorpseRottingMoveFunc_2(k, j, game),
        11135458 => Skree_CorpseRottingMoveFunc_4(k, j, game),
        11135552 => MotherBrain_CorpseRottingMoveFunc(k, j, game),
    }
    pub unsafe fn CallCorpseRottingInit(ea: u32, game: &mut Game) => ea, game {
        11132440 => Torizo_CorpseRottingInitFunc(game),
        11132609 => Sidehopper_CorpseRottingInitFunc_0(game),
        11132680 => Sidehopper_CorpseRottingInitFunc_2(game),
        11132751 => Zoomer_CorpseRottingInitFunc_0(game),
        11132780 => Zoomer_CorpseRottingInitFunc_2(game),
        11132809 => Zoomer_CorpseRottingInitFunc_4(game),
        11132838 => Ripper_CorpseRottingInitFunc_0(game),
        11132867 => Ripper_CorpseRottingInitFunc_2(game),
        11132896 => Skree_CorpseRottingInitFunc_0(game),
        11132953 => Skree_CorpseRottingInitFunc_2(game),
        11133010 => Skree_CorpseRottingInitFunc_4(game),
        11133067 => MotherBrain_CorpseRottingInitFunc(game),
    }
    pub unsafe fn CallCorpseRottingFinish(ea: u32, game: &mut Game) => ea, game {
        11121187 => MotherBrain_CorpseRottingFinished(game),
        11130301 => DeadTorizo_CorpseRottingFinished(game),
        11131912 => CorpseRottingRotEntryFinishedHook(game),
    }
    pub unsafe fn CallMotherBrainFunc(ea: u32, game: &mut Game) => ea, game {
        11110352 => MotherBrainsBrain_SetupBrainToDraw(game),
        11110369 => {
            Unreachable(game);
        }
        11110729 => MotherBrainBody_0_Wait(game),
        11110766 => MotherBrainBody_1_ClearBottomLeftTube(game),
        11110787 => MotherBrainBody_2_SpawnTopRightTubeFalling(game),
        11110816 => MotherBrainBody_3_ClearCeilingBlock9(game),
        11110837 => MotherBrainBody_4_SpawnTopLeftTubeFalling(game),
        11110866 => MotherBrainBody_4_ClearCeilingBlock6(game),
        11110887 => MotherBrainBody_5_SpawnTubeFallingEnemy1(game),
        11110906 => MotherBrainBody_6_ClearBottomRightTube(game),
        11110927 => MotherBrainBody_7_SpawnTubeFallingEnemy2(game),
        11110946 => MotherBrainBody_8_ClearBottomMiddleLeftTube(game),
        11110967 => MotherBrainBody_9_SpawnTopMiddleLeftFalling(game),
        11110996 => MotherBrainBody_10_ClearCeilingTubeColumn7(game),
        11111017 => MotherBrainBody_11_SpawnTopMiddleRightFalling(game),
        11111046 => MotherBrainBody_12_ClearCeilingTubeColumn8(game),
        11111067 => MotherBrainBody_13_SpawnTubeFallingEnemy3(game),
        11111086 => MotherBrainBody_14_ClearBottomMiddleRightTube(game),
        11111107 => MotherBrainBody_15_SpawnTubeFallingEnemy4(game),
        11111126 => MotherBrainBody_16_ClearBottomMiddleTubes(game),
        11110429 => MotherBrainBody_FakeDeath_Descent_0_Pause(game),
        11110441 => MotherBrainBody_FakeDeath_Descent_1(game),
        11110477 => MotherBrainBody_FakeDeath_Descent_2(game),
        11110508 => MotherBrainBody_FakeDeath_Descent_3(game),
        11110532 => MotherBrainBody_FakeDeath_Descent_4(game),
        11110578 => MotherBrainBody_FakeDeath_Descent_5(game),
        11110611 => MotherBrainBody_FakeDeath_Descent_6(game),
        11111559 => MotherBrainBody_FakeDeath_Ascent_0_DrawBG1Row23(game),
        11111582 => MotherBrainBody_FakeDeath_Ascent_1_DrawBG1Row45(game),
        11111605 => MotherBrainBody_FakeDeath_Ascent_2_DrawBG1Row67(game),
        11111628 => MotherBrainBody_FakeDeath_Ascent_3_DrawBG1Row89(game),
        11111651 => MotherBrainBody_FakeDeath_Ascent_4_DrawBG1RowAB(game),
        11111674 => MotherBrainBody_FakeDeath_Ascent_5_DrawBG1RowCD(game),
        11111697 => MotherBrainBody_FakeDeath_Ascent_6_SetupPhase2Gfx(game),
        11111753 => MotherBrainBody_FakeDeath_Ascent_7_SetupPhase2Brain(game),
        11111801 => MotherBrainBody_FakeDeath_Ascent_8_Pause(game),
        11111819 => MotherBrainBody_FakeDeath_Ascent_9_PrepareRise(game),
        11111860 => MotherBrainBody_FakeDeath_Ascent_10_LoadLegTiles(game),
        11111875 => MotherBrainBody_FakeDeath_Ascent_11_ContinuePause(game),
        11111916 => MotherBrainBody_FakeDeath_Ascent_12_StartMusic(game),
        11112013 => MotherBrainBody_FakeDeath_Ascent_13_Raise(game),
        11112085 => MotherBrainBody_FakeDeath_Ascent_14_WaitForUncouching(game),
        11112106 => MotherBrainBody_FakeDeath_Ascent_15_TransitionFromGrey(game),
        11112181 => MotherBrainBody_2ndphase_16_ShakeHeadMenacingly(game),
        11112212 => MotherBrainBody_2ndphase_17_BringHeadBackUp(game),
        11112243 => MotherBrainBody_2ndphase_18_FinishStretching(game),
        11120353 => MotherBrain_Phase3_Death_0(game),
        11120402 => MotherBrain_Phase3_Death_1(game),
        11120417 => MotherBrain_Phase3_Death_2(game),
        11120468 => MotherBrain_Phase3_Death_3(game),
        11120541 => MotherBrain_Phase3_Death_4(game),
        11120566 => MotherBrain_Phase3_Death_5(game),
        11120659 => MotherBrain_Phase3_Death_6(game),
        11120917 => MotherBrain_Phase3_Death_7(game),
        11120941 => MotherBrain_Phase3_Death_8(game),
        11120990 => MotherBrain_Phase3_Death_9(game),
        11121011 => MotherBrain_Phase3_Death_10(game),
        11121033 => MotherBrain_Phase3_Death_11(game),
        11121080 => MotherBrain_Phase3_Death_12(game),
        11121109 => MotherBrain_Phase3_Death_13(game),
        11121169 => MotherBrain_Phase3_Death_14_20framedelay(game),
        11121240 => MotherBrain_Phase3_Death_15_LoadEscapeTimerTiles(game),
        11121261 => MotherBrain_Phase3_Death_16_StartEscape(game),
        11121361 => MotherBrain_Phase3_Death_17_SpawnTimeBomb(game),
        11121379 => MotherBrain_Phase3_Death_18_TypesZebesText(game),
        11121401 => MotherBrain_Phase3_Death_19_EscapeDoorExploding(game),
        11121450 => MotherBrain_Phase3_Death_20_BlowUpEscapeDoor(game),
        11121468 => MotherBrain_Phase3_Death_21_KeepEarthquakeGoing(game),
        11122181 => MotherBrain_Body_Phase2_Thinking(game),
        11122251 => MotherBrain_Body_Phase2_TryAttack(game),
        11122561 => MotherBrain_FiringBomb_DecideOnWalking(game),
        11122604 => MotherBrain_FiringBomb_WalkingBackwards(game),
        11122630 => MotherBrain_FiringBomb_Crouch(game),
        11122664 => MotherBrain_FiringBomb_Fired(game),
        11122680 => MotherBrain_FiringBomb_Standup(game),
        11122702 => MotherBomb_FiringLaser_PositionHead(game),
        11122745 => MotherBomb_FiringLaser_PositionHeadSlowlyFire(game),
        11122787 => MotherBomb_FiringLaser_FinishAttack(game),
        11122813 => MotherBomb_FiringDeathBeam(game),
        11122923 => MotherBomb_FiringRainbowBeam_0(game),
        11122970 => MotherBomb_FiringRainbowBeam_1_StartCharge(game),
        11122987 => MotherBomb_FiringRainbowBeam_2_RetractNeck(game),
        11123007 => MotherBomb_FiringRainbowBeam_3_Wait(game),
        11123025 => MotherBomb_FiringRainbowBeam_4_ExtendNeckDown(game),
        11123061 => MotherBomb_FiringRainbowBeam_5_StartFiring(game),
        11123173 => MotherBomb_FiringRainbowBeam_6_MoveSamusToWall(game),
        11123200 => MotherBomb_FiringRainbowBeam_7_DelayFrame(game),
        11123239 => MotherBomb_FiringRainbowBeam_8_StartDrainSamus(game),
        11123260 => MotherBomb_FiringRainbowBeam_9_DrainingSamus(game),
        11123294 => MotherBomb_FiringRainbowBeam_10_FinishFiringRainbow(game),
        11123396 => MotherBomb_FiringRainbowBeam_11_LetSamusFall(game),
        11123409 => MotherBomb_FiringRainbowBeam_12_WaitForSamusHitGround(game),
        11123421 => MotherBomb_FiringRainbowBeam_13_LowerHead(game),
        11123462 => MotherBomb_FiringRainbowBeam_14_DecideNextAction(game),
        11124037 => MotherBrain_Phase2Cut_0(game),
        11124120 => MotherBrain_Phase2Cut_1(game),
        11124137 => MotherBrain_Phase2Cut_2(game),
        11124161 => MotherBrain_Phase2Cut_3(game),
        11124178 => MotherBrain_Phase2Cut_4(game),
        11124205 => MotherBrain_Phase2Cut_5(game),
        11124250 => (),
        11124280 => MotherBrain_DrainedByShitroid_0(game),
        11124317 => MotherBrain_DrainedByShitroid_1(game),
        11124374 => MotherBrain_DrainedByShitroid_2(game),
        11124494 => MotherBrain_DrainedByShitroid_3(game),
        11124545 => MotherBrain_DrainedByShitroid_4(game),
        11124566 => MotherBrain_DrainedByShitroid_5(game),
        11124605 => MotherBrain_DrainedByShitroid_6(game),
        11124629 => MotherBrain_DrainedByShitroid_7(game),
        11124825 => MotherBrain_Phase2_Revive_0(game),
        11124838 => MotherBrain_Phase2_Revive_1(game),
        11124866 => MotherBrain_Phase2_Revive_2(game),
        11124879 => MotherBrain_Phase2_Revive_3(game),
        11124922 => MotherBrain_Phase2_Revive_4(game),
        11124964 => MotherBrain_Phase2_Revive_5(game),
        11124987 => MotherBrain_Phase2_Revive_6(game),
        11125022 => MotherBrain_Phase2_Revive_7(game),
        11125063 => MotherBrain_Phase2_Revive_8(game),
        11125084 => MotherBrain_Phase2_MurderShitroid_1(game),
        11125122 => MotherBrain_Phase2_MurderShitroid_2(game),
        11125134 => MotherBrain_Phase2_PrepareForFinalShitroid(game),
        11125146 => MotherBrain_Phase2_ExecuteFinalkShitroid(game),
        11125158 => (),
        11125199 => MotherBrain_Phase3_Recover_MakeDistance(game),
        11125232 => MotherBrain_Phase3_Recover_SetupForFight(game),
        11125257 => MotherBrain_Phase3_Fighting_Main(game),
        11125326 => MotherBrain_Phase3_Fighting_Cooldown(game),
        11111140 => (),
        11125133 => (),
        11110306 => MotherBrainsBrain_SetupBrainAndNeckToDraw(game),
    }
}
pub unsafe fn Enemy_GrappleReact_CancelBeam_A9(game: &mut Game) {
    Enemy_SwitchToFrozenAi(game);
}
pub unsafe fn Enemy_NormalFrozenAI_A9(game: &mut Game) {
    NormalEnemyFrozenAI(game);
}
pub unsafe fn MotherBrainsBody_Init(game: &mut Game) {
    let mut i: i32 = 4094i32;
    while i >= 0i32 {
        set_ram_word!(game.tilemap_stuff[(i >> 1i32)] = 824u16);
        i -= 2i32;
    }
    MotherBrain_SetBodyInstrs(addr_kMotherBrain_Ilist_9C13 as u16, game);
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).base.vram_tiles_index = 0u16;
    (*E).base.properties = ((*E).base.properties as i32
        | (kEnemyProps_BlockPlasmaBeam as i32
            | kEnemyProps_Tangible as i32
            | kEnemyProps_Invisible as i32)) as u16;
    (*E).base.palette_index = 0u16;
    WriteColorsToTargetPalette(
        0xa9u8,
        0x162u16,
        (addr_kMotherBrainPalette_4 as i32 + 2i32) as u16,
        0xfu16,
        game,
    );
    WriteColorsToTargetPalette(
        0xa9u8,
        0x1e2u16,
        (addr_kMotherBrainPalette_3 as i32 + 2i32) as u16,
        0xfu16,
        game,
    );
    (*E).mbn_var_00 = 0u16;
    let E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
    (*E1).mbn_var_02 = 0u16;
    (*E).mbn_var_04 = 2u16;
    (*E1).mbn_var_A = ((0xa987d0i32) as u16);
    (*E).mbn_var_A = ((0xa987e1i32) as u16);
    LoadFxEntry(1u16, game);
    let mut v3: u16 = 0u16;
    let mut v4: u16 = 0;
    loop {
        v4 = v3;
        SpawnEprojWithRoomGfx(addr_kEproj_MotherBrainRoomTurrets as u16, v3, game);
        v3 = (v4).wrapping_add(1u16);
        if ((v4).wrapping_add(1u16) >= 0xcu16) {
            break;
        }
    }
}
pub unsafe fn MotherBrainsBrain_Init(game: &mut Game) {
    InitializeEnemyCorpseRotting(0x40u16, addr_stru_A9DE08 as u16, game);
    let E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
    (*E1).base.health = 3000u16;
    MotherBrain_SetBrainUnusedInstrs(addr_kMotherBrain_Ilist_9C13 as u16, game);
    (*E1).base.vram_tiles_index = 0u16;
    (*E1).base.properties = ((*E1).base.properties) | (0x1100u16);
    (*E1).base.palette_index = 512u16;
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).mbn_var_0C = 512u16;
    (*E).mbn_var_0D = 512u16;
    MotherBrain_SetBrainInstrs(addr_stru_A99C21 as u16, game);
    MotherBrain_SetupBrainNormalPal(game);
}
pub unsafe fn MotherBrainsBody_Hurt(game: &mut Game) {
    MotherBrain_Pal_HandleRoomPal(game);
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    if (*E).mbn_var_A == (fnMotherBrainsBody_FirstPhase_DoubleRet as u16) {
        MotherBrainsBody_FirstPhase_DoubleRet(game);
        return;
    }
    CallMotherBrainFunc((((*E).mbn_var_A) as u32) | (0xa90000u32), game);
    MotherBrain_HandlePalette(game);
    MotherBrain_SamusCollDetect(game);
    MotherBrain_Pal_ProcessInvincibility(game);
    if (*Get_MotherBrain(0x40i32, game)).mbn_var_02 != 0 {
        mov24(
            ram_ptr!(game, LongPtr; 0x604),
            fnMotherBrainsBody_UnpauseHook as u32,
        );
    }
}
pub unsafe fn MotherBrainsBody_UnpauseHook(game: &mut Game) -> CoroutineRet {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    if (*E).mbn_var_16 != 0 {
        QueueSfx1_Max6(0x40u16, game);
    }
    if ((*E).base.extra_properties as i32 & 4i32) != 0i32 {
        set_ram_word!(game.enemy_bg2_tilemap_size = 2048u16);
        set_ram_word!(game.nmi_flag_bg2_enemy_vram_transfer = 1u16);
    }
    kCoroutineNone as CoroutineRet
}
pub unsafe fn MotherBrainsBody_Powerbomb(game: &mut Game) {
    NormalEnemyPowerBombAiSkipDeathAnim_CurEnemy(game);
    MotherBrainsBrain_Hurt(game);
}
pub unsafe fn MotherBrainsBrain_Hurt(game: &mut Game) {
    mov24(ram_ptr!(game, LongPtr; 0x178c), 0xa98786u32);
    let v0: *mut EnemyData = gEnemyData(0i32, game);
    if ((*v0.offset(1)).properties as i32 & 0x100i32) != 0i32 {
        CallMotherBrainFunc((((*v0.offset(1)).ai_var_A) as u32) | (0xa90000u32), game);
    }
}
pub unsafe fn MotherBrainsBrain_SetupBrainAndNeckToDraw(game: &mut Game) {
    if ram_word!(game, time_is_frozen_flag) == 0 {
        MotherBrain_HandleNeck(game);
    }
    let E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
    (*E1).base.x_pos = (*E1).mbn_var_2E;
    (*E1).base.y_pos = ((*E1).mbn_var_2F).wrapping_sub(21u16);
    mov24(ram_ptr!(game, LongPtr; 0x178c), 0xa987c9u32);
}
pub unsafe fn MotherBrain_DrawBrainNeck_EnemyGfxDrawHook(game: &mut Game) {
    MotherBrain_DrawBrain(game);
    MotherBrain_DrawNeck(game);
}
pub unsafe fn MotherBrainsBrain_SetupBrainToDraw(game: &mut Game) {
    mov24(ram_ptr!(game, LongPtr; 0x178c), 0xa987ddu32);
}
pub unsafe fn MotherBrainsBrain_GfxDrawHook(game: &mut Game) {
    MotherBrain_DrawBrain(game);
}
pub unsafe fn MotherBrainsBody_FirstPhase_DoubleRet(game: &mut Game) {
    if CheckEventHappened(2u16, game) != 0 {
        let E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
        (*E1).mbn_var_00 = ram_word!(game, earthquake_timer);
        if ram_word!(game, samus_x_pos) as i32 - 236i32 & 0x8000i32 != 0 && (*E1).base.health == 0 {
            let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
            (*E).mbn_var_1D = 1u16;
            (*E).mbn_var_00 = 1u16;
            DisableMinimapAndMarkBossRoomAsExplored(game);
            QueueMusic_Delayed8(6u16, game);
            MotherBrain_SealWall(game);
        }
    }
    MotherBrain_SamusCollDetect(game);
}
pub unsafe fn MotherBrainBody_FakeDeath_Descent_0_Pause(game: &mut Game) {
    let E: *mut Enemy_MotherBrainBody = Get_MotherBrainBody(0i32, game);
    (*E).mbby_var_A = ((0xa98829i32) as u16);
    (*E).mbby_var_F = 64u16;
    MotherBrainBody_FakeDeath_Descent_1(game);
}
pub unsafe fn MotherBrainBody_FakeDeath_Descent_1(game: &mut Game) {
    let E: *mut Enemy_MotherBrainBody = Get_MotherBrainBody(0i32, game);
    (*E).mbby_var_F = (*E).mbby_var_F.wrapping_sub(1);
    if ((*E).mbby_var_F as i32 & 0x8000i32) != 0i32 {
        CallSomeSamusCode(0u16, game);
        *(ram_ptr!(game, u16; 0xcd20)) = *ram_ptr!(game; 0xcd20, 0) as u16;
        (*E).mbby_var_A = ((0xa9884di32) as u16);
        (*E).mbby_var_F = 32u16;
        MotherBrainBody_FakeDeath_Descent_2(game);
    }
}
pub unsafe fn MotherBrainBody_FakeDeath_Descent_2(game: &mut Game) {
    let E: *mut Enemy_MotherBrainBody = Get_MotherBrainBody(0i32, game);
    (*E).mbby_var_F = (*E).mbby_var_F.wrapping_sub(1);
    if ((*E).mbby_var_F as i32 & 0x8000i32) != 0i32 {
        QueueMusic_Delayed8(0u16, game);
        QueueMusic_Delayed8(0xff21u16, game);
        (*E).mbby_var_A = ((0xa9886ci32) as u16);
        (*E).mbby_var_F = 12u16;
        MotherBrainBody_FakeDeath_Descent_3(game);
    }
}
pub unsafe fn MotherBrainBody_FakeDeath_Descent_3(game: &mut Game) {
    let E: *mut Enemy_MotherBrainBody = Get_MotherBrainBody(0i32, game);
    (*E).mbby_var_F = (*E).mbby_var_F.wrapping_sub(1);
    if ((*E).mbby_var_F as i32 & 0x8000i32) != 0i32 {
        CallSomeSamusCode(1u16, game);
        (*E).mbby_var_A = ((0xa98884i32) as u16);
        (*E).mbby_var_F = 8u16;
        MotherBrainBody_FakeDeath_Descent_4(game);
    }
}
pub unsafe fn MotherBrainBody_FakeDeath_Descent_4(game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if ((*E).mbn_var_F as i32 & 0x8000i32) != 0i32 {
        MotherBrain_Pal_BeginScreenFlash(game);
        LoadFxEntry(2u16, game);
        (*Get_MotherBrain(0x40i32, game)).mbn_var_E = ((0xa98949i32) as u16);
        (*E).mbn_var_A = ((0xa988b2i32) as u16);
        (*E).mbn_var_F = 0u16;
        (*E).mbn_var_37 = 0u16;
        SpawnHardcodedPlm(
            SpawnHardcodedPlmArgs::new(0xeu8, 0x2u8, 0xb6b3 as VoidP),
            game,
        );
    }
}
pub unsafe fn MotherBrainBody_FakeDeath_Descent_5(game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if ((*E).mbn_var_F as i32 & 0x8000i32) != 0i32 {
        (*E).mbn_var_F = 8u16;
        let v2: u16 = ((*E).mbn_var_37).wrapping_add(1u16);
        (*E).mbn_var_37 = v2;
        if MotherBrain_FadeToGray_FakeDeath((v2).wrapping_sub(1u16), game) as i32 & 1i32 != 0 {
            (*E).mbn_var_A = ((0xa988d3i32) as u16);
        }
    }
    MotherBrainBody_FakeDeath_Descent_6(game);
}
pub unsafe fn MotherBrainBody_FakeDeath_Descent_6(game: &mut Game) {
    let E: *mut Enemy_MotherBrainBody = Get_MotherBrainBody(0x40i32, game);
    CallMotherBrainFunc((((*E).mbby_var_E) as u32) | (0xa90000u32), game);
    MotherBrain_HandleFakeDeathExplosions(game);
}
pub unsafe fn MotherBrain_HandleFakeDeathExplosions(game: &mut Game) {
    let mut v1: i16 = 0;
    let mut v2: i16 = 0;
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    v1 = (((*E).mbn_var_38) as i16).wrapping_sub(1i16);
    if v1 < 0i16 {
        (*E).mbn_var_38 = 8u16;
        v2 = (((*E).mbn_var_39) as i16).wrapping_sub(1i16);
        if v2 < 0i16 {
            v2 = 7i16;
        }
        (*E).mbn_var_39 = v2 as u16;
        let v3: i32 = ((4u16).wrapping_mul((v2) as u16)) as i32 >> 1i32;
        game.eproj_spawn_pt = Point16U {
            x: crate::rtl::rom_table_word!(game, 0xa98929, v3 as isize),
            y: crate::rtl::rom_table_word!(game, 0xa98929, (v3 + 1i32) as isize),
        };
        let v4: u16 = (if ram_word!(game, random_number) < 0x4000u16 {
            12i32
        } else {
            3i32
        }) as u16;
        SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, v4, game);
        QueueSfx2_Max3(0x24u16, game);
    } else {
        (*E).mbn_var_38 = v1 as u16;
    };
}
pub unsafe fn MotherBrainBody_0_Wait(game: &mut Game) {
    let mut v1: i16 = 0;
    let mut v0: u16 = 0u16;
    v1 = 0i16;
    loop {
        if *((ram_ptr!(game, u16; 0x1997) as *mut u8).offset(v0 as i32 as isize) as *mut u16) == 0 {
            v1 += 1;
        }
        v0 = (v0).wrapping_add(2u16);
        if (((v0) as i16).wrapping_sub(36i16) >= 0i16) {
            break;
        }
    }
    if (v1).wrapping_sub(4i16) >= 0i16 {
        SpawnEnemy(0xa9u8, addr_stru_A98AE5 as u16, game);
        (*Get_MotherBrainBody(0x40i32, game)).mbby_var_E = ((0xa9896ei32) as u16);
    }
}
pub unsafe fn MotherBrainBody_1_ClearBottomLeftTube(game: &mut Game) {
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs::new(0x5u8, 0x9u8, 0xb6c3 as VoidP),
        game,
    );
    let E: *mut Enemy_MotherBrainBody = Get_MotherBrainBody(0x40i32, game);
    (*E).mbby_var_E = ((0xa98983i32) as u16);
    (*E).mbby_var_F = 32u16;
}
pub unsafe fn MotherBrainBody_2_SpawnTopRightTubeFalling(game: &mut Game) {
    let E: *mut Enemy_MotherBrainBody = Get_MotherBrainBody(0x40i32, game);
    (*E).mbby_var_F = (*E).mbby_var_F.wrapping_sub(1);
    if ((*E).mbby_var_F as i32 & 0x8000i32) != 0i32 {
        game.eproj_spawn_pt = Point16U {
            x: 152u16,
            y: 47u16,
        };
        SpawnEprojWithRoomGfx(0xcc5bu16, 0x2fu16, game);
        (*E).mbby_var_E = -30304i32 as u16;
    }
}
pub unsafe fn MotherBrainBody_3_ClearCeilingBlock9(game: &mut Game) {
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs::new(0x9u8, 0x2u8, 0xb6b3 as VoidP),
        game,
    );
    let E: *mut Enemy_MotherBrainBody = Get_MotherBrainBody(0x40i32, game);
    (*E).mbby_var_E = ((0xa989b5i32) as u16);
    (*E).mbby_var_F = 32u16;
}
pub unsafe fn MotherBrainBody_4_SpawnTopLeftTubeFalling(game: &mut Game) {
    let E: *mut Enemy_MotherBrainBody = Get_MotherBrainBody(0x40i32, game);
    (*E).mbby_var_F = (*E).mbby_var_F.wrapping_sub(1);
    if ((*E).mbby_var_F as i32 & 0x8000i32) != 0i32 {
        game.eproj_spawn_pt = Point16U {
            x: 104u16,
            y: 47u16,
        };
        SpawnEprojWithRoomGfx(
            addr_kEproj_MotherBrainTubeFalling_TopLeft as u16,
            0x2fu16,
            game,
        );
        (*E).mbby_var_E = ((0xa989d2i32) as u16);
    }
}
pub unsafe fn MotherBrainBody_4_ClearCeilingBlock6(game: &mut Game) {
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs::new(0x6u8, 0x2u8, 0xb6b3 as VoidP),
        game,
    );
    let E: *mut Enemy_MotherBrainBody = Get_MotherBrainBody(0x40i32, game);
    (*E).mbby_var_E = ((0xa989e7i32) as u16);
    (*E).mbby_var_F = 32u16;
}
pub unsafe fn MotherBrainBody_5_SpawnTubeFallingEnemy1(game: &mut Game) {
    let E: *mut Enemy_MotherBrainBody = Get_MotherBrainBody(0x40i32, game);
    (*E).mbby_var_F = (*E).mbby_var_F.wrapping_sub(1);
    if ((*E).mbby_var_F as i32 & 0x8000i32) != 0i32 {
        SpawnEnemy(0xa9u8, addr_stru_A98AF5 as u16, game);
        (*E).mbby_var_E = ((0xa989fai32) as u16);
    }
}
pub unsafe fn MotherBrainBody_6_ClearBottomRightTube(game: &mut Game) {
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs::new(0xau8, 0x9u8, 0xb6c7 as VoidP),
        game,
    );
    let E: *mut Enemy_MotherBrainBody = Get_MotherBrainBody(0x40i32, game);
    (*E).mbby_var_E = ((0xa98a0fi32) as u16);
    (*E).mbby_var_F = 32u16;
}
pub unsafe fn MotherBrainBody_7_SpawnTubeFallingEnemy2(game: &mut Game) {
    let E: *mut Enemy_MotherBrainBody = Get_MotherBrainBody(0x40i32, game);
    (*E).mbby_var_F = (*E).mbby_var_F.wrapping_sub(1);
    if ((*E).mbby_var_F as i32 & 0x8000i32) != 0i32 {
        SpawnEnemy(0xa9u8, addr_stru_A98B05 as u16, game);
        (*E).mbby_var_E = ((0xa98a22i32) as u16);
    }
}
pub unsafe fn MotherBrainBody_8_ClearBottomMiddleLeftTube(game: &mut Game) {
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs::new(0x6u8, 0xau8, 0xb6bb as VoidP),
        game,
    );
    let E: *mut Enemy_MotherBrainBody = Get_MotherBrainBody(0x40i32, game);
    (*E).mbby_var_E = ((0xa98a37i32) as u16);
    (*E).mbby_var_F = 32u16;
}
pub unsafe fn MotherBrainBody_9_SpawnTopMiddleLeftFalling(game: &mut Game) {
    let E: *mut Enemy_MotherBrainBody = Get_MotherBrainBody(0x40i32, game);
    (*E).mbby_var_F = (*E).mbby_var_F.wrapping_sub(1);
    if ((*E).mbby_var_F as i32 & 0x8000i32) != 0i32 {
        game.eproj_spawn_pt = Point16U {
            x: 120u16,
            y: 59u16,
        };
        SpawnEprojWithRoomGfx(
            addr_kEproj_MotherBrainTubeFalling_TopMiddleLeft as u16,
            0x3bu16,
            game,
        );
        (*E).mbby_var_E = ((0xa98a54i32) as u16);
    }
}
pub unsafe fn MotherBrainBody_10_ClearCeilingTubeColumn7(game: &mut Game) {
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs::new(0x7u8, 0x2u8, 0xb6b7 as VoidP),
        game,
    );
    let E: *mut Enemy_MotherBrainBody = Get_MotherBrainBody(0x40i32, game);
    (*E).mbby_var_E = ((0xa98a69i32) as u16);
    (*E).mbby_var_F = 32u16;
}
pub unsafe fn MotherBrainBody_11_SpawnTopMiddleRightFalling(game: &mut Game) {
    let E: *mut Enemy_MotherBrainBody = Get_MotherBrainBody(0x40i32, game);
    (*E).mbby_var_F = (*E).mbby_var_F.wrapping_sub(1);
    if ((*E).mbby_var_F as i32 & 0x8000i32) != 0i32 {
        game.eproj_spawn_pt = Point16U {
            x: 136u16,
            y: 59u16,
        };
        SpawnEprojWithRoomGfx(
            addr_kEproj_MotherBrainTubeFalling_TopMiddleRight as u16,
            0x3bu16,
            game,
        );
        (*E).mbby_var_E = ((0xa98a86i32) as u16);
    }
}
pub unsafe fn MotherBrainBody_12_ClearCeilingTubeColumn8(game: &mut Game) {
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs::new(0x8u8, 0x2u8, 0xb6b7 as VoidP),
        game,
    );
    let E: *mut Enemy_MotherBrainBody = Get_MotherBrainBody(0x40i32, game);
    (*E).mbby_var_E = ((0xa98a9bi32) as u16);
    (*E).mbby_var_F = 32u16;
}
pub unsafe fn MotherBrainBody_13_SpawnTubeFallingEnemy3(game: &mut Game) {
    let E: *mut Enemy_MotherBrainBody = Get_MotherBrainBody(0x40i32, game);
    (*E).mbby_var_F = (*E).mbby_var_F.wrapping_sub(1);
    if ((*E).mbby_var_F as i32 & 0x8000i32) != 0i32 {
        SpawnEnemy(0xa9u8, ((0x868b15i32) as u16), game);
        (*E).mbby_var_E = ((0xa98aaei32) as u16);
    }
}
pub unsafe fn MotherBrainBody_14_ClearBottomMiddleRightTube(game: &mut Game) {
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs::new(0x9u8, 0xau8, 0xb6bb as VoidP),
        game,
    );
    let E: *mut Enemy_MotherBrainBody = Get_MotherBrainBody(0x40i32, game);
    (*E).mbby_var_E = ((0xa98ac3i32) as u16);
    (*E).mbby_var_F = 2u16;
}
pub unsafe fn MotherBrainBody_15_SpawnTubeFallingEnemy4(game: &mut Game) {
    let E: *mut Enemy_MotherBrainBody = Get_MotherBrainBody(0x40i32, game);
    (*E).mbby_var_F = (*E).mbby_var_F.wrapping_sub(1);
    if ((*E).mbby_var_F as i32 & 0x8000i32) != 0i32 {
        SpawnEnemy(0xa9u8, addr_stru_A98B25 as u16, game);
        (*E).mbby_var_E = ((0xa98ad6i32) as u16);
    }
}
pub unsafe fn MotherBrainBody_16_ClearBottomMiddleTubes(game: &mut Game) {
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs::new(0x7u8, 0x7u8, 0xb6bf as VoidP),
        game,
    );
    (*Get_MotherBrainBody(0x40i32, game)).mbby_var_E = addr_locret_A98AE4 as u16;
}
pub unsafe fn MotherBrainsTubesFalling_Init(game: &mut Game) {
    let E: *mut Enemy_MotherBrainsTubesFalling =
        Get_MotherBrainsTubesFalling(ram_word!(game, cur_enemy_index) as i32, game);
    let v1: i32 = (*E).mbtfg_parameter_1 as i32 >> 1i32;
    (*E).base.x_width = g_word_A98B5D[v1 as usize];
    (*E).base.y_height = g_word_A98B67[v1 as usize];
    (*E).mbtfg_var_B = g_word_A98B71[v1 as usize];
    (*E).mbtfg_var_D = 0u16;
    (*E).mbtfg_var_E = 0u16;
    (*E).mbtfg_var_C = 0u16;
    (*E).mbtfg_var_A = crate::rtl::rom_table_word!(game, 0xa98b7b, v1 as isize);
}
crate::game::address_dispatch! {
    pub unsafe fn CallMotherBrainsTubesFalling(ea: u32, k: u16, game: &mut Game) => ea, game {
        11111304 => MotherBrainsTubesFalling_Main_NonMain(k, game),
        11111371 => MotherBrainsTubesFalling_WaitToFall(k, game),
        11111382 => MotherBrainsTubesFalling_Falling(k, game),
    }
}
pub unsafe fn MotherBrainsTubesFalling_Main(k: u16, game: &mut Game) {
    let E: *mut Enemy_MotherBrainsTubesFalling = Get_MotherBrainsTubesFalling(k as i32, game);
    CallMotherBrainsTubesFalling((((*E).mbtfg_var_A) as u32) | (0xa90000u32), k, game);
}
pub unsafe fn MotherBrainsTubesFalling_Main_NonMain(k: u16, game: &mut Game) {
    let E: *mut Enemy_MotherBrainsTubesFalling = Get_MotherBrainsTubesFalling(k as i32, game);
    let v2: u16 = ((*E).mbtfg_var_C).wrapping_add(6u16);
    (*E).mbtfg_var_C = v2;
    Enemy_IncreaseYpos(k, v2, game);
    if (((*E).base.y_pos) as i16).wrapping_sub(((*E).mbtfg_var_B) as i16) < 0i16 {
        MotherBrainsTubesFalling_HandleSmoke(k, game);
    } else {
        MotherBrainsTubesFalling_Explode(k, game);
    };
}
pub unsafe fn MotherBrainsTubesFalling_HandleSmoke(k: u16, game: &mut Game) {
    let E: *mut Enemy_MotherBrainsTubesFalling = Get_MotherBrainsTubesFalling(k as i32, game);
    (*E).mbtfg_var_D = (*E).mbtfg_var_D.wrapping_sub(1);
    if ((*E).mbtfg_var_D as i32 & 0x8000i32) != 0i32 {
        MotherBrainsTubesFalling_SpawnSmoke(k, game);
    }
}
pub unsafe fn MotherBrainsTubesFalling_Explode(k: u16, game: &mut Game) {
    let E: *mut Enemy_MotherBrainsTubesFalling = Get_MotherBrainsTubesFalling(k as i32, game);
    (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Deleted) as u16);
    game.eproj_spawn_pt = Point16U {
        x: (*E).base.x_pos,
        y: (*E).base.y_pos,
    };
    SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 3u16, game);
    QueueSfx2_Max3(0x24u16, game);
}
pub unsafe fn MotherBrainsTubesFalling_WaitToFall(k: u16, game: &mut Game) {
    let E: *mut Enemy_MotherBrainsTubesFalling = Get_MotherBrainsTubesFalling(k as i32, game);
    (*E).mbtfg_parameter_2 = (*E).mbtfg_parameter_2.wrapping_sub(1);
    if ((*E).mbtfg_parameter_2 as i32 & 0x8000i32) != 0i32 {
        (*E).mbtfg_var_A = ((0xa98bd6i32) as u16);
        MotherBrainsTubesFalling_Falling(k, game);
    }
}
pub unsafe fn MotherBrainsTubesFalling_Falling(k: u16, game: &mut Game) {
    let E: *mut Enemy_MotherBrainsTubesFalling = Get_MotherBrainsTubesFalling(k as i32, game);
    let v2: u16 = ((*E).mbtfg_var_C).wrapping_add(6u16);
    (*E).mbtfg_var_C = v2;
    Enemy_IncreaseYpos(k, v2, game);
    let ypos: u16 = (*E).base.y_pos;
    if ypos as i32 - 244i32 & 0x8000i32 == 0 {
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Invisible) as u16);
    }
    let E1: *mut Enemy_MotherBrainsTubesFalling = Get_MotherBrainsTubesFalling(0x40i32, game);
    (*E1).base.y_pos = (ypos).wrapping_sub(56u16);
    if (*E1).base.y_pos as i32 - 0xc4i32 & 0x8000i32 != 0 {
        MotherBrainsTubesFalling_HandleSmoke(k, game);
    } else {
        MotherBrain_Pal_EndScreenFlash(game);
        EnableEarthquakeAframes(0x19u16, game);
        set_ram_words! { game, 0u16;
            hdma_object_channels_bitmask[0], hdma_object_channels_bitmask[1],
        }
        (*E1).base.y_pos = 196u16;
        let E0: *mut Enemy_MotherBrainsTubesFalling = Get_MotherBrainsTubesFalling(0i32, game);
        (*E0).base.x_pos = 59u16;
        (*E0).base.y_pos = 279u16;
        MotherBrain_SetupNeckForFakeAscent(game);
        (*E0).mbtfg_var_A = ((0xa98c87i32) as u16);
        MotherBrainsTubesFalling_Explode(ram_word!(game, cur_enemy_index), game);
    };
}
pub unsafe fn MotherBrainsTubesFalling_SpawnSmoke(k: u16, game: &mut Game) {
    let E: *mut Enemy_MotherBrainsTubesFalling = Get_MotherBrainsTubesFalling(k as i32, game);
    (*E).mbtfg_var_D = 8u16;
    let v2: u16 = ((*E).mbtfg_var_E as i32 + 1i32 & 3i32) as u16;
    (*E).mbtfg_var_E = v2;
    game.eproj_spawn_pt = Point16U {
        x: (((*E).base.x_pos).wrapping_add(g_word_A98C61[v2 as usize])),
        y: 208u16,
    };
    SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 9u16, game);
}
pub unsafe fn MotherBrainBody_FakeDeath_Ascent_0_DrawBG1Row23(game: &mut Game) {
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs::new(0x2u8, 0x2u8, 0xb67b as VoidP),
        game,
    );
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs::new(0x2u8, 0x3u8, 0xb67f as VoidP),
        game,
    );
    (*Get_MotherBrainBody(0i32, game)).mbby_var_A = ((0xa98c9ei32) as u16);
}
pub unsafe fn MotherBrainBody_FakeDeath_Ascent_1_DrawBG1Row45(game: &mut Game) {
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs::new(0x2u8, 0x4u8, 0xb683 as VoidP),
        game,
    );
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs::new(0x2u8, 0x5u8, 0xb687 as VoidP),
        game,
    );
    (*Get_MotherBrainBody(0i32, game)).mbby_var_A = ((0xa98cb5i32) as u16);
}
pub unsafe fn MotherBrainBody_FakeDeath_Ascent_2_DrawBG1Row67(game: &mut Game) {
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs::new(0x2u8, 0x6u8, 0xb68b as VoidP),
        game,
    );
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs::new(0x2u8, 0x7u8, 0xb68f as VoidP),
        game,
    );
    (*Get_MotherBrainBody(0i32, game)).mbby_var_A = ((0xa98ccci32) as u16);
}
pub unsafe fn MotherBrainBody_FakeDeath_Ascent_3_DrawBG1Row89(game: &mut Game) {
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs::new(0x2u8, 0x8u8, 0xb693 as VoidP),
        game,
    );
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs::new(0x2u8, 0x9u8, 0xb697 as VoidP),
        game,
    );
    (*Get_MotherBrainBody(0i32, game)).mbby_var_A = ((0xa98ce3i32) as u16);
}
pub unsafe fn MotherBrainBody_FakeDeath_Ascent_4_DrawBG1RowAB(game: &mut Game) {
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs::new(0x2u8, 0xau8, 0xb69b as VoidP),
        game,
    );
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs::new(0x2u8, 0xbu8, 0xb69f as VoidP),
        game,
    );
    (*Get_MotherBrainBody(0i32, game)).mbby_var_A = ((0xa98cfai32) as u16);
}
pub unsafe fn MotherBrainBody_FakeDeath_Ascent_5_DrawBG1RowCD(game: &mut Game) {
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs::new(0x2u8, 0xcu8, 0xb6a3 as VoidP),
        game,
    );
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs::new(0x2u8, 0xdu8, 0xb6a7 as VoidP),
        game,
    );
    (*Get_MotherBrainBody(0i32, game)).mbby_var_A = ((0xa98d11i32) as u16);
}
pub unsafe fn MotherBrainBody_FakeDeath_Ascent_6_SetupPhase2Gfx(game: &mut Game) {
    *(ram_ptr!(game, u16; 0x91b)) = 257u16;
    let ref mut fresh8 = *(ram_ptr!(game, u16; 0x59));
    *fresh8 = (*fresh8) & (0xfffcu16);
    WriteColorsToPalette(
        0x142u16,
        0xa9u8,
        (addr_kMotherBrainPalette_1 as i32 + 2i32) as u16,
        0xfu16,
        game,
    );
    WriteColorsToPalette(
        0x162u16,
        0xa9u8,
        (addr_kMotherBrainPalette_0 as i32 + 2i32) as u16,
        0xfu16,
        game,
    );
    (*Get_MotherBrain(0i32, game)).mbn_var_A = ((0xa98d49i32) as u16);
    set_ram_word!(game.nmi_flag_bg2_enemy_vram_transfer = 1u16);
    (*Get_MotherBrain(0x40i32, game)).mbn_var_02 = 1u16;
}
pub unsafe fn MotherBrainBody_FakeDeath_Ascent_7_SetupPhase2Brain(game: &mut Game) {
    set_ram_word!(game.fx_layer_blending_config_a = 52u16);
    let E: *mut Enemy_MotherBrainBody = Get_MotherBrainBody(0x40i32, game);
    (*E).mbby_var_A = ((0xa987a2i32) as u16);
    let E0: *mut Enemy_MotherBrainBody = Get_MotherBrainBody(0i32, game);
    (*E0).base.properties = ((*E0).base.properties as i32 & !(kEnemyProps_Tangible as i32)) as u16;
    (*E).base.properties = ((*E).base.properties as i32 & !(0x400i32)) as u16;
    (*E).base.health = 18000u16;
    (*E0).mbby_var_A = ((0xa98d79i32) as u16);
    (*E0).mbby_var_F = 128u16;
    MotherBrainBody_FakeDeath_Ascent_8_Pause(game);
}
pub unsafe fn MotherBrainBody_FakeDeath_Ascent_8_Pause(game: &mut Game) {
    let E: *mut Enemy_MotherBrainBody = Get_MotherBrainBody(0i32, game);
    (*E).mbby_var_F = (*E).mbby_var_F.wrapping_sub(1);
    if ((*E).mbby_var_F as i32 & 0x8000i32) != 0i32 {
        (*E).mbby_var_A = ((0xa98d8bi32) as u16);
        (*E).mbby_var_F = 32u16;
        MotherBrainBody_FakeDeath_Ascent_9_PrepareRise(game);
    }
}
pub unsafe fn MotherBrainBody_FakeDeath_Ascent_9_PrepareRise(game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if ((*E).mbn_var_F as i32 & 0x8000i32) != 0i32 {
        (*E).mbn_var_09 = MotherBrainRisingHdmaObject(game);
        let E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
        (*E1).base.properties = ((*E1).base.properties) | (0x100u16);
        MotherBrain_SetBrainInstrs(addr_stru_A99C21 as u16, game);
        (*E).mbn_var_A = ((0xa98db4i32) as u16);
        (*E).mbn_var_F = 256u16;
        MotherBrainBody_FakeDeath_Ascent_10_LoadLegTiles(game);
    }
}
pub unsafe fn MotherBrainBody_FakeDeath_Ascent_10_LoadLegTiles(game: &mut Game) {
    if ProcessSpriteTilesTransfers(0xa9u8, addr_stru_A98F8F as u16, game) != 0 {
        (*Get_MotherBrainBody(0i32, game)).mbby_var_A = ((0xa98dc3i32) as u16);
        MotherBrainBody_FakeDeath_Ascent_11_ContinuePause(game);
    }
}
pub unsafe fn MotherBrainBody_FakeDeath_Ascent_11_ContinuePause(game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if ((*E).mbn_var_F as i32 & 0x8000i32) != 0i32 {
        (*E).base.x_pos = 59u16;
        (*E).base.y_pos = 279u16;
        set_ram_word!(game.reg_BG2HOFS = -27i32 as u16);
        set_ram_word!(game.reg_BG2VOFS = -217i32 as u16);
        (*E).mbn_var_04 = 7u16;
        (*E).mbn_var_A = ((0xa98deci32) as u16);
    }
}
pub unsafe fn MotherBrainBody_FakeDeath_Ascent_12_StartMusic(game: &mut Game) {
    MotherBrain_SetBodyInstrs(addr_kMotherBrain_Ilist_9A02 as u16, game);
    let E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
    (*E1).base.instruction_timer = 1u16;
    let E0: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E0).base.properties = ((*E0).base.properties as i32 & !(kEnemyProps_Invisible as i32)) as u16;
    (*E0).base.x_pos = 59u16;
    (*E0).base.y_pos = 279u16;
    set_ram_word!(game.reg_BG2HOFS = -27i32 as u16);
    set_ram_word!(game.reg_BG2VOFS = -217i32 as u16);
    QueueMusic_Delayed8(5u16, game);
    set_ram_word!(game.earthquake_type = 2u16);
    set_ram_word!(game.earthquake_timer = 256u16);
    (*E1).mbn_var_34 = 80u16;
    (*E1).mbn_var_31 = 1u16;
    (*E1).mbn_var_32 = 8u16;
    (*E1).mbn_var_33 = 6u16;
    (*E0).mbn_var_A = ((0xa98e4di32) as u16);
}
pub unsafe fn MotherBrainBody_FakeDeath_Ascent_13_Raise(game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    let mut v1: u16 = 0;
    if !((ram_word!(game, nmi_frame_counter_word) as i32 & 3i32) != 0i32 || {
        MotherBrain_SpawnDustCloudsForAscent(game);
        update_ram_word!(game.reg_BG2VOFS, wrapping_add(2u16));
        v1 = ((*E).base.y_pos).wrapping_sub(2u16);
        (*E).base.y_pos = v1;
        v1 >= 0xbdu16
    }) {
        set_ram_word!(game.enemy_bg2_tilemap_size = 320u16);
        (*E).base.y_pos = 188u16;
        set_ram_words! { game, 0u16;
            earthquake_timer, hdma_object_channels_bitmask[((*E).mbn_var_09 as i32 >> 1i32)],
        }
        MotherBrain_SetBodyInstrs(addr_kMotherBrain_Ilist_99AA as u16, game);
        (*E).mbn_var_A = ((0xa98e95i32) as u16);
        MotherBrainBody_FakeDeath_Ascent_14_WaitForUncouching(game);
    }
}
pub unsafe fn MotherBrainBody_FakeDeath_Ascent_14_WaitForUncouching(game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    if (*E).mbn_var_02 == 0 {
        (*E).mbn_var_37 = 0u16;
        (*E).mbn_var_A = ((0xa98eaai32) as u16);
        (*E).mbn_var_F = 0u16;
    }
}
pub unsafe fn MotherBrainBody_FakeDeath_Ascent_15_TransitionFromGrey(game: &mut Game) {
    let E0: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E0).mbn_var_F = (*E0).mbn_var_F.wrapping_sub(1);
    if ((*E0).mbn_var_F as i32 & 0x8000i32) != 0i32 {
        (*E0).mbn_var_F = 4u16;
        let v3: u16 = ((*E0).mbn_var_37).wrapping_add(1u16);
        (*E0).mbn_var_37 = v3;
        if MotherBrain_FadeFromGray_FakeDeath((v3).wrapping_sub(1u16), game) as i32 & 1i32 != 0 {
            let E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
            (*E1).mbn_var_10 = 1u16;
            (*E0).mbn_var_00 = 2u16;
            (*E1).mbn_var_12 = 1u16;
            (*E1).mbn_var_32 = 6u16;
            (*E1).mbn_var_33 = 6u16;
            (*E1).mbn_var_34 = 1280u16;
            (*E0).mbn_var_A = ((0xa98ef5i32) as u16);
            (*E0).mbn_var_F = 23u16;
        }
    }
}
pub unsafe fn MotherBrainBody_2ndphase_16_ShakeHeadMenacingly(game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if ((*E).mbn_var_F as i32 & 0x8000i32) != 0i32 {
        MotherBrain_SetBrainInstrs(addr_kMotherBrain_Ilist_9B7F as u16, game);
        (*E).mbn_var_A = ((0xa98f14i32) as u16);
        (*Get_MotherBrain(0x40i32, game)).mbn_var_34 = 64u16;
        (*E).mbn_var_F = 256u16;
        MotherBrainBody_2ndphase_17_BringHeadBackUp(game);
    }
}
pub unsafe fn MotherBrainBody_2ndphase_17_BringHeadBackUp(game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if ((*E).mbn_var_F as i32 & 0x8000i32) != 0i32 {
        let E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
        (*E1).mbn_var_32 = 2u16;
        (*E1).mbn_var_33 = 4u16;
        (*E).mbn_var_A = ((0xa98f33i32) as u16);
        (*E).mbn_var_F = 64u16;
        MotherBrainBody_2ndphase_18_FinishStretching(game);
    }
}
pub unsafe fn MotherBrainBody_2ndphase_18_FinishStretching(game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if ((*E).mbn_var_F as i32 & 0x8000i32) != 0i32 {
        (*Get_MotherBrain(0x40i32, game)).mbn_var_14 = 1u16;
        (*E).mbn_var_A = ((0xa9b605i32) as u16);
    }
}
pub unsafe fn MotherBrain_SpawnDustCloudsForAscent(game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if ((*E).mbn_var_F as i32 & 0x8000i32) != 0i32 {
        (*E).mbn_var_F = 7u16;
    }
    game.eproj_spawn_pt = Point16U {
        x: g_word_A98F7F[(*E).mbn_var_F as usize],
        y: 212u16,
    };
    SpawnEprojWithRoomGfx(
        addr_kEproj_DustCloudExplosion as u16,
        g_byte_A98F7D[(((ram_word!(game, random_number)) & (0x100u16)) as i32 >> 8i32) as usize]
            as u16,
        game,
    );
    QueueSfx2_Max3(0x29u16, game);
}
pub unsafe fn MotherBrain_SetupNeckForFakeAscent(game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
    (*E).mbn_var_24 = 2u16;
    (*E).mbn_var_27 = 10u16;
    (*E).mbn_var_2D = 10u16;
    (*E).mbn_var_2A = 20u16;
    (*E).mbn_var_30 = 20u16;
    (*E).mbn_var_20 = 18432u16;
    (*E).mbn_var_21 = 20480u16;
    (*E).mbn_var_34 = 256u16;
}
pub unsafe fn MotherBrain_HandleNeckLower(game: &mut Game) {
    let mbn_var_32: u16 = (*Get_MotherBrain(0x40i32, game)).mbn_var_32;
    if mbn_var_32 != 0 {
        match mbn_var_32 as i32 {
            2 => MotherBrain_HandleNeckLower_2_BobDown(game),
            4 => MotherBrain_HandleNeckLower_4_BobUp(game),
            6 => MotherBrain_HandleNeckLower_6_Lower(game),
            8 => MotherBrain_HandleNeckLower_8_Raise(game),
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
    let mut v1: i16 = 0;
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
    v1 = (((*E).mbn_var_20) as i16).wrapping_sub(((*E).mbn_var_34) as i16);
    if (v1 as u16) < 0x2800u16 {
        (*E).mbn_var_32 = 4u16;
        v1 = 10240i16;
    }
    (*E).mbn_var_20 = v1 as u16;
}
pub unsafe fn MotherBrain_HandleNeckLower_4_BobUp(game: &mut Game) {
    let mut v1: i16 = 0;
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
    if (*E).base.y_pos as i32 - 60i32 & 0x8000i32 != 0 {
        (*E).mbn_var_32 = 2u16;
    } else {
        v1 = (((*E).mbn_var_34) as i16).wrapping_add(((*E).mbn_var_20) as i16);
        if (v1 as u16) >= 0x9000u16 {
            (*E).mbn_var_32 = 2u16;
            v1 = -28672i16;
        }
        (*E).mbn_var_20 = v1 as u16;
    };
}
pub unsafe fn MotherBrain_HandleNeckLower_6_Lower(game: &mut Game) {
    let mut v1: i16 = 0;
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
    v1 = (((*E).mbn_var_20) as i16).wrapping_sub(((*E).mbn_var_34) as i16);
    if (v1 as u16) < 0x3000u16 {
        (*E).mbn_var_32 = 0u16;
        v1 = 12288i16;
    }
    (*E).mbn_var_20 = v1 as u16;
}
pub unsafe fn MotherBrain_HandleNeckLower_8_Raise(game: &mut Game) {
    let mut v1: i16 = 0;
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
    v1 = (((*E).mbn_var_34) as i16).wrapping_add(((*E).mbn_var_20) as i16);
    if (v1 as u16) >= 0x9000u16 {
        (*E).mbn_var_32 = 0u16;
        v1 = -28672i16;
    }
    (*E).mbn_var_20 = v1 as u16;
}
pub unsafe fn MotherBrain_HandleNeckUpper(game: &mut Game) {
    let mut v2: i16 = 0;
    let mut v6: i16 = 0;
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
    let mbn_var_33: u16 = (*E).mbn_var_33;
    if mbn_var_33 != 0 {
        match mbn_var_33 as i32 {
            2 => {
                if (((*E).base.y_pos as i32 + 4i32 - ram_word!(game, samus_y_pos) as i32) as i16)
                    < 0i16
                {
                    v2 = (((*E).mbn_var_21) as i16).wrapping_sub(((*E).mbn_var_34) as i16);
                    if (v2 as u16) < 0x2000u16 {
                        (*E).mbn_var_33 = 4u16;
                        v2 = 0x2000i16;
                    }
                    (*E).mbn_var_21 = v2 as u16;
                } else {
                    (*E).mbn_var_32 = 4u16;
                    (*E).mbn_var_33 = 4u16;
                }
            }
            4 => {
                let r18: u16 = ((*E).mbn_var_20).wrapping_add(2048u16);
                let mut v4: u16 = ((*E).mbn_var_34).wrapping_add((*E).mbn_var_21);
                if v4 >= r18 {
                    (*E).mbn_var_33 = 2u16;
                    v4 = r18;
                }
                (*E).mbn_var_21 = v4;
            }
            6 => {
                v6 = (((*E).mbn_var_21) as i16).wrapping_sub(((*E).mbn_var_34) as i16);
                if (v6 as u16) < 0x2000u16 {
                    (*E).mbn_var_33 = 0u16;
                    v6 = 0x2000i16;
                }
                (*E).mbn_var_21 = v6 as u16;
            }
            8 => {
                let r18_0: u16 = ((*E).mbn_var_20).wrapping_add(2048u16);
                let mut v8: u16 = ((*E).mbn_var_34).wrapping_add((*E).mbn_var_21);
                if v8 >= r18_0 {
                    (*E).mbn_var_33 = 0u16;
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
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).mbn_var_0A = ((*E).base.x_pos).wrapping_sub(80u16);
    (*E).mbn_var_0B = ((*E).base.y_pos).wrapping_add(46u16);
    let E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
    if (*E1).mbn_var_31 != 0 {
        MotherBrain_HandleNeckLower(game);
        MotherBrain_HandleNeckUpper(game);
    }
    let mut r18: u16 = *(&raw mut (*E1).mbn_var_20 as *mut u8).offset(1isize) as u16;
    (*E1).mbn_var_22 = ((*E).mbn_var_0A as i32
        + ComputeSinMult((*E1).mbn_var_24, r18, game) as i32
        + 112i32) as u16;
    (*E1).mbn_var_23 = ((*E).mbn_var_0B as i32 + ComputeCosMult((*E1).mbn_var_24, r18, game) as i32
        - 96i32) as u16;
    (*E1).mbn_var_25 = ((*E).mbn_var_0A as i32
        + ComputeSinMult((*E1).mbn_var_27, r18, game) as i32
        + 112i32) as u16;
    (*E1).mbn_var_26 = ((*E).mbn_var_0B as i32 + ComputeCosMult((*E1).mbn_var_27, r18, game) as i32
        - 96i32) as u16;
    (*E1).mbn_var_28 = ((*E).mbn_var_0A as i32
        + ComputeSinMult((*E1).mbn_var_2A, r18, game) as i32
        + 112i32) as u16;
    (*E1).mbn_var_29 = ((*E).mbn_var_0B as i32 + ComputeCosMult((*E1).mbn_var_2A, r18, game) as i32
        - 96i32) as u16;
    r18 = *(&raw mut (*E1).mbn_var_21 as *mut u8).offset(1isize) as u16;
    (*E1).mbn_var_2B = ((*E1).mbn_var_28).wrapping_add(ComputeSinMult((*E1).mbn_var_2D, r18, game));
    (*E1).mbn_var_2C = ((*E1).mbn_var_29).wrapping_add(ComputeCosMult((*E1).mbn_var_2D, r18, game));
    (*E1).mbn_var_2E = ((*E1).mbn_var_28).wrapping_add(ComputeSinMult((*E1).mbn_var_30, r18, game));
    (*E1).mbn_var_2F = ((*E1).mbn_var_29).wrapping_add(ComputeCosMult((*E1).mbn_var_30, r18, game));
}
pub unsafe fn CallMotherBrainInstr(ea: u32, k: u16, game: &mut Game) -> u16 {
    match ea {
        11115279 => MotherBrain_Instr_Goto(k, game),
        11115284 => MotherBrain_Instr_EnableNeckMovementGoto(k, game),
        11115296 => MotherBrain_Instr_DisableNeckMovement(k, game),
        11115304 => MotherBrain_Instr_QueueSfx2(k, game),
        11115314 => MotherBrain_Instr_QueueSfx3(k, game),
        11115324 => MotherBrain_Instr_SpawnDroolEproj(k, game),
        11115373 => MotherBrain_Instr_SpawnPurpleBreath(k, game),
        11115383 => MotherBrain_Instr_SetMainShakeTimer50(k, game),
        11115621 => MotherBrain_Instr_GotoEitherOr(k, game),
        11115693 => MotherBrain_Instr_MaybeGoto(k, game),
        11115789 => MotherBrain_Instr_MaybeGoto2(k, game),
        11115809 => MotherBrain_Instr_Goto2(k),
        11116023 => MotherBrain_Instr_QueueShitroidAttackSfx(k, game),
        11116073 => MotherBrain_Instr_SpawnBlueRingEproj(k, game),
        11116087 => MotherBrain_Instr_AimRingsAtShitroid(k, game),
        11116123 => MotherBrain_Instr_AimRingsAtSamus(k, game),
        11116195 => MotherBrain_Instr_IncrShitroidAttackCtr(k, game),
        11116213 => MotherBrain_Instr_SetShitroidAttackCtr0(k, game),
        11116221 => MotherBrain_Instr_SpawnBombEproj(k, game),
        11116358 => MotherBrain_Instr_SpawnLaserEproj(k, game),
        11116420 => MotherBrain_Instr_SpawnRainbowEproj(k, game),
        11116430 => MotherBrain_Instr_SetupFxForRainbowBeam(k, game),
        _ => Unreachable(game) as u16,
    }
}
pub unsafe fn MotherBrain_Func_1_DoubleRet(game: &mut Game) -> i32 {
    let mut execute_instruction: bool;
    let mut v5: u16 = 0;
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    let mut pc: u16 = (*E).mbn_var_21;
    if pc as i32 & 0x8000i32 == 0 {
        return -1i32;
    }
    let mut p: *const u8 = RomPtr_A9(pc, game);
    if ram_word!(game, time_is_frozen_flag) != 0 {
        return *(p.offset(2) as *mut u16) as i32;
    }
    v5 = *(p as *mut u16);
    execute_instruction = (v5 as i32 & 0x8000i32 != 0)
        || ({
            if ((v5) as i16).wrapping_sub(((*E).mbn_var_20) as i16) >= 0i16 {
                (*E).mbn_var_20 = (*E).mbn_var_20.wrapping_add(1);
                return *(p.offset(2) as *mut u16) as i32;
            }
            pc = (pc).wrapping_add(4u16);
            false
        });
    loop {
        if execute_instruction {
            pc = CallMotherBrainInstr(((v5) as u32) | (0xa90000u32), (pc).wrapping_add(2u16), game);
            execute_instruction = false;
        } else {
            p = RomPtr_A9(pc, game);
            v5 = *(p as *mut u16);
            if v5 as i32 & 0x8000i32 == 0 {
                break;
            } else {
                execute_instruction = true;
            }
        }
    }
    (*E).mbn_var_20 = 1u16;
    (*E).mbn_var_21 = pc;
    *(p.offset(2) as *mut u16) as i32
}
pub unsafe fn MotherBrain_DrawNeck(game: &mut Game) {
    if ((*Get_MotherBrain(0i32, game)).base.properties as i32 & kEnemyProps_Invisible as i32)
        == 0i32
    {
        let E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
        MotherBrain_DrawNeckSegment((*E).mbn_var_2E, (*E).mbn_var_2F, game);
        MotherBrain_DrawNeckSegment((*E).mbn_var_2B, (*E).mbn_var_2C, game);
        MotherBrain_DrawNeckSegment((*E).mbn_var_28, (*E).mbn_var_29, game);
        MotherBrain_DrawNeckSegment((*E).mbn_var_25, (*E).mbn_var_26, game);
        MotherBrain_DrawNeckSegment((*E).mbn_var_22, (*E).mbn_var_23, game);
    }
}
pub unsafe fn MotherBrain_DrawBrain(game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
    if (*E).mbn_var_14 != 0u16
        && (*E).mbn_var_15 == 0
        && (ram_word!(game, random_number) as i32 & 0x8000i32) == 0i32
    {
        SpawnEprojWithRoomGfx(
            addr_kEproj_MotherBrainPurpleBreathSmall as u16,
            ram_word!(game, random_number),
            game,
        );
    }
    let tt: i32 = MotherBrain_Func_1_DoubleRet(game);
    if tt < 0i32 {
        return;
    }
    let v2: u16 = tt as u16;
    let mut mbn_var_0D: u16 = (*Get_MotherBrain(0i32, game)).mbn_var_0D;
    if (*E).base.flash_timer as i32 & 1i32 != 0 {
        mbn_var_0D = 0u16;
    }
    let r22: u16 = mbn_var_0D;
    let mbn_var_00: u16 = (*E).mbn_var_00;
    let mut flash_timer: u16 = 0;
    if mbn_var_00 != 0 {
        flash_timer = (mbn_var_00).wrapping_sub(1u16);
        (*E).mbn_var_00 = flash_timer;
    } else {
        flash_timer = (*E).base.flash_timer;
        if flash_timer == 0 {
            flash_timer = (*E).base.shake_timer;
        }
    }
    let v6: i32 = (flash_timer as i32 & 6i32) >> 1i32;
    let r18: u16 = ((*E).base.x_pos).wrapping_add((g_word_A993BB[v6 as usize]) as u16);
    if ((r18 as i32 + 32i32 - ram_word!(game, layer1_x_pos) as i32) as i16) >= 0i16 {
        let r20: u16 = ((*E).base.y_pos).wrapping_add((g_word_A993C3[v6 as usize]) as u16);
        MotherBrain_AddSpritemapToOam(v2, r18, r20, r22, game);
    }
}
pub unsafe fn MotherBrain_DrawNeckSegment(x: u16, y: u16, game: &mut Game) {
    let v0: i32 = ((*Get_MotherBrain(0x40i32, game)).base.flash_timer as i32 & 6i32) >> 1i32;
    MotherBrain_AddSpritemapToOam(
        addr_kMotherBrain_Sprmap_A694 as u16,
        (x).wrapping_add((g_word_A993BB[v0 as usize]) as u16),
        (y).wrapping_add((g_word_A993C3[v0 as usize]) as u16),
        (*Get_MotherBrain(0i32, game)).mbn_var_0C,
        game,
    );
}
pub unsafe fn MotherBrain_AddSpritemapToOam(j: u16, r18: u16, r20: u16, r22: u16, game: &mut Game) {
    let mut p: *const u8 = RomPtr_A9(j, game);
    let mut n: i32 = *(p as *mut u16) as i32;
    p = p.offset(2);
    let mut idx: i32 = ram_word!(game, oam_next_ptr) as i32;
    loop {
        let y: i16 =
            (r20 as i32 + *p.offset(2) as i8 as i32 - ram_word!(game, layer1_y_pos) as i32) as i16;
        if y >= 0i16 {
            let oam: *mut OamEnt = gOamEnt(idx, game);
            let x: u16 = (r18 as i32 + *(p as *mut u16) as i32
                - ram_word!(game, layer1_x_pos) as i32) as u16;
            (*oam).xcoord = x as u8;
            (*oam).ycoord = y as u8;
            *(&raw mut (*oam).charnum as *mut u16) = (r22) | (*(p.offset(3) as *mut u16));
            update_ram_word!(game.oam_ext[(idx >> 5i32)], |fresh6| (fresh6 as i32
                | ((x as i32 & 0x100i32) >> 8i32 | (*(p as *mut i16) < 0i16) as i32 * 2i32)
                    << 2i32 * (idx >> 2i32 & 7i32))
                as u16);
            idx = idx + 4i32 & 0x1ffi32;
        }
        p = p.offset(5);
        n -= 1;
        if (n == 0) {
            break;
        }
    }
    set_ram_word!(game.oam_next_ptr = idx as u16);
}
pub unsafe fn MotherBrain_CalculateRainbowBeamHdma(game: &mut Game) {
    MotherBrain_CalcHdma(game);
}
pub unsafe fn MotherBrain_MoveBodyDownScrollLeft(k: u16, a: u16, game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).base.y_pos = ((*E).base.y_pos).wrapping_add(a);
    update_ram_word!(game.reg_BG2VOFS, wrapping_sub(a));
    set_ram_word!(game.reg_BG2HOFS = (k as i32 + 34i32 - (*E).base.x_pos as i32) as u16);
}
pub unsafe fn MotherBrain_MoveBodyDown(a: u16, game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).base.y_pos = ((*E).base.y_pos).wrapping_add(a);
    update_ram_word!(game.reg_BG2VOFS, wrapping_sub(a));
    set_ram_word!(game.reg_BG2HOFS = (34u16).wrapping_sub((*E).base.x_pos));
}
pub unsafe fn MotherBrain_FootstepEffect(game: &mut Game) {
    set_ram_word!(game.earthquake_type = 1u16);
    set_ram_word!(game.earthquake_timer = 4u16);
    if (*Get_MotherBrain(0i32, game)).mbn_var_00 == 3u16 {
        printf(b"Write to rom!\n\0" as *const u8 as *const c_char);
    }
}
pub unsafe fn MotherBrain_Instr_MoveBodyUp10Left4(
    k: u16,
    jp: *const u16,
    game: &mut Game,
) -> *const u16 {
    MotherBrain_MoveBodyDownScrollLeft(4u16, 0xfff6u16, game);
    jp
}
pub unsafe fn MotherBrain_Instr_MoveBodyUp16Left4(
    k: u16,
    jp: *const u16,
    game: &mut Game,
) -> *const u16 {
    MotherBrain_MoveBodyDownScrollLeft(4u16, 0xfff0u16, game);
    jp
}
pub unsafe fn MotherBrain_Instr_MoveBodyUp12Right2(
    k: u16,
    jp: *const u16,
    game: &mut Game,
) -> *const u16 {
    MotherBrain_MoveBodyDownScrollLeft(0xfffeu16, 0xfff4u16, game);
    jp
}
pub unsafe fn MotherBrain_Instr_MoveDown12Left4(
    k: u16,
    jp: *const u16,
    game: &mut Game,
) -> *const u16 {
    MotherBrain_MoveBodyDownScrollLeft(4u16, 0xcu16, game);
    jp
}
pub unsafe fn MotherBrain_Instr_MoveDown16Right2(
    k: u16,
    jp: *const u16,
    game: &mut Game,
) -> *const u16 {
    MotherBrain_MoveBodyDownScrollLeft(0xfffeu16, 0x10u16, game);
    jp
}
pub unsafe fn MotherBrain_Instr_MoveDown10Right2(
    k: u16,
    jp: *const u16,
    game: &mut Game,
) -> *const u16 {
    MotherBrain_MoveBodyDownScrollLeft(0xfffeu16, 0xau16, game);
    jp
}
pub unsafe fn MotherBrain_Instr_MoveUp2Right1(
    k: u16,
    jp: *const u16,
    game: &mut Game,
) -> *const u16 {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).base.x_pos = (*E).base.x_pos.wrapping_add(1);
    MotherBrain_MoveBodyDown(0xfffeu16, game);
    jp
}
pub unsafe fn MotherBrain_Instr_MoveRight2(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).base.x_pos = ((*E).base.x_pos).wrapping_add(2u16);
    MotherBrain_MoveBodyDown(0u16, game);
    jp
}
pub unsafe fn MotherBrain_Instr_MoveUp1(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    MotherBrain_MoveBodyDown(1u16, game);
    jp
}
pub unsafe fn MotherBrain_Instr_MoveUp1Right3_Sfx(
    k: u16,
    jp: *const u16,
    game: &mut Game,
) -> *const u16 {
    MotherBrain_FootstepEffect(game);
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).base.x_pos = ((*E).base.x_pos).wrapping_add(3u16);
    MotherBrain_MoveBodyDown(1u16, game);
    jp
}
pub unsafe fn MotherBrain_Instr_Down2Right15(
    k: u16,
    jp: *const u16,
    game: &mut Game,
) -> *const u16 {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).base.x_pos = ((*E).base.x_pos).wrapping_add(15u16);
    MotherBrain_MoveBodyDown(0xfffeu16, game);
    jp
}
pub unsafe fn MotherBrain_Instr_Down4Right6(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).base.x_pos = ((*E).base.x_pos).wrapping_add(6u16);
    MotherBrain_MoveBodyDown(0xfffcu16, game);
    jp
}
pub unsafe fn MotherBrain_Instr_Up4Left2(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).base.x_pos = ((*E).base.x_pos).wrapping_sub(2u16);
    MotherBrain_MoveBodyDown(4u16, game);
    jp
}
pub unsafe fn MotherBrain_Instr_Up2Left1_Sfx(
    k: u16,
    jp: *const u16,
    game: &mut Game,
) -> *const u16 {
    MotherBrain_FootstepEffect(game);
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).base.x_pos = (*E).base.x_pos.wrapping_sub(1);
    MotherBrain_MoveBodyDown(2u16, game);
    jp
}
pub unsafe fn MotherBrain_Instr_Up2Left1_Sfx2(
    k: u16,
    jp: *const u16,
    game: &mut Game,
) -> *const u16 {
    MotherBrain_FootstepEffect(game);
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).base.x_pos = (*E).base.x_pos.wrapping_sub(1);
    MotherBrain_MoveBodyDown(2u16, game);
    jp
}
pub unsafe fn MotherBrain_Instr_MoveLeft2(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).base.x_pos = ((*E).base.x_pos).wrapping_sub(2u16);
    MotherBrain_MoveBodyDown(0u16, game);
    jp
}
pub unsafe fn MotherBrain_Instr_MoveDown1(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    MotherBrain_MoveBodyDown(0xffffu16, game);
    jp
}
pub unsafe fn MotherBrain_Instr_MoveDown1Left3(
    k: u16,
    jp: *const u16,
    game: &mut Game,
) -> *const u16 {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).base.x_pos = ((*E).base.x_pos).wrapping_sub(3u16);
    MotherBrain_MoveBodyDown(0xffffu16, game);
    jp
}
pub unsafe fn MotherBrain_Instr_MoveUp2Left15_Sfx(
    k: u16,
    jp: *const u16,
    game: &mut Game,
) -> *const u16 {
    MotherBrain_FootstepEffect(game);
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).base.x_pos = ((*E).base.x_pos).wrapping_sub(15u16);
    MotherBrain_MoveBodyDown(2u16, game);
    jp
}
pub unsafe fn MotherBrain_Instr_MoveUp4Left6(
    k: u16,
    jp: *const u16,
    game: &mut Game,
) -> *const u16 {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).base.x_pos = ((*E).base.x_pos).wrapping_sub(6u16);
    MotherBrain_MoveBodyDown(4u16, game);
    jp
}
pub unsafe fn MotherBrain_Instr_MoveDown4Right2(
    k: u16,
    jp: *const u16,
    game: &mut Game,
) -> *const u16 {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).base.x_pos = ((*E).base.x_pos).wrapping_add(2u16);
    MotherBrain_MoveBodyDown(0xfffcu16, game);
    jp
}
pub unsafe fn MotherBrain_Instr_MoveDown2Right1(
    k: u16,
    jp: *const u16,
    game: &mut Game,
) -> *const u16 {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).base.x_pos = (*E).base.x_pos.wrapping_add(1);
    MotherBrain_MoveBodyDown(0xfffeu16, game);
    jp
}
pub unsafe fn MotherBrain_Instr_SetPose_Standing(
    k: u16,
    jp: *const u16,
    game: &mut Game,
) -> *const u16 {
    (*Get_MotherBrain(0i32, game)).mbn_var_02 = 0u16;
    jp
}
pub unsafe fn MotherBrain_Instr_SetPose_Walking(
    k: u16,
    jp: *const u16,
    game: &mut Game,
) -> *const u16 {
    (*Get_MotherBrain(0i32, game)).mbn_var_02 = 1u16;
    jp
}
pub unsafe fn MotherBrain_Instr_SetPose_Crouched(
    k: u16,
    jp: *const u16,
    game: &mut Game,
) -> *const u16 {
    (*Get_MotherBrain(0i32, game)).mbn_var_02 = 3u16;
    jp
}
pub unsafe fn MotherBrain_Instr_SetPose_CrouchedTrans(
    k: u16,
    jp: *const u16,
    game: &mut Game,
) -> *const u16 {
    (*Get_MotherBrain(0i32, game)).mbn_var_02 = 2u16;
    jp
}
pub unsafe fn MotherBrain_Instr_SetPose_DeathBeamMode(
    k: u16,
    jp: *const u16,
    game: &mut Game,
) -> *const u16 {
    (*Get_MotherBrain(0i32, game)).mbn_var_02 = 4u16;
    jp
}
pub unsafe fn MotherBrain_Instr_SetPose_LeaningDown(
    k: u16,
    jp: *const u16,
    game: &mut Game,
) -> *const u16 {
    (*Get_MotherBrain(0i32, game)).mbn_var_02 = 6u16;
    jp
}
pub unsafe fn MotherBrain_Instr_SpawnEprojToOffset(
    k: u16,
    jp: *const u16,
    game: &mut Game,
) -> *const u16 {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    game.eproj_spawn_pt = Point16U {
        x: (((*E).base.x_pos).wrapping_add(*jp.offset(0))),
        y: (((*E).base.y_pos).wrapping_add(*jp.offset(1))),
    };
    SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, *jp.offset(2), game);
    jp.offset(3)
}
pub unsafe fn MotherBrain_Instr_SpawnDeathBeamEproj(
    k: u16,
    jp: *const u16,
    game: &mut Game,
) -> *const u16 {
    QueueSfx2_Max6(0x63u16, game);
    SpawnEprojWithGfx(
        0u16,
        0x40u16,
        addr_kEproj_MotherBrainDeathBeamCharging as u16,
        game,
    );
    jp
}
pub unsafe fn MotherBrain_Instr_IncrBeamAttackPhase(
    k: u16,
    jp: *const u16,
    game: &mut Game,
) -> *const u16 {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).mbn_var_17 = (*E).mbn_var_17.wrapping_add(1);
    jp
}
pub unsafe fn MotherBrain_Instr_Goto(k: u16, game: &mut Game) -> u16 {
    *(RomPtr_A9(k, game) as *mut u16)
}
pub unsafe fn MotherBrain_Instr_EnableNeckMovementGoto(k: u16, game: &mut Game) -> u16 {
    (*Get_MotherBrain(0x40i32, game)).mbn_var_31 = 1u16;
    *(RomPtr_A9(k, game) as *mut u16)
}
pub unsafe fn MotherBrain_Instr_DisableNeckMovement(k: u16, game: &mut Game) -> u16 {
    (*Get_MotherBrain(0x40i32, game)).mbn_var_31 = 0u16;
    k
}
pub unsafe fn MotherBrain_Instr_QueueSfx2(k: u16, game: &mut Game) -> u16 {
    let v2: *const u16 = RomPtr_A9(k, game) as *const u16;
    QueueSfx2_Max6(*v2, game);
    (k).wrapping_add(2u16)
}
pub unsafe fn MotherBrain_Instr_QueueSfx3(k: u16, game: &mut Game) -> u16 {
    let v2: *const u16 = RomPtr_A9(k, game) as *const u16;
    QueueSfx3_Max6(*v2, game);
    (k).wrapping_add(2u16)
}
pub unsafe fn MotherBrain_Instr_SpawnDroolEproj(k: u16, game: &mut Game) -> u16 {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
    if (*E).mbn_var_12 != 0 {
        let mut v2: u16 = ((*E).mbn_var_13).wrapping_add(1u16);
        if (*E).mbn_var_13 as i32 - 5i32 & 0x8000i32 == 0 {
            v2 = 0u16;
        }
        (*E).mbn_var_13 = v2;
        let v3: u16 = (if (*E).mbn_var_34 as i32 - 128i32 & 0x8000i32 != 0 {
            addr_kEproj_MotherBrainDrool as i32
        } else {
            addr_kEproj_MotherBrainDyingDrool as i32
        }) as u16;
        SpawnEprojWithRoomGfx(v3, (*E).mbn_var_13, game);
    }
    k
}
pub unsafe fn MotherBrain_Instr_SpawnPurpleBreath(k: u16, game: &mut Game) -> u16 {
    SpawnEprojWithRoomGfx(addr_kEproj_MotherBrainPurpleBreathBig as u16, 0u16, game);
    k
}
pub unsafe fn MotherBrain_Instr_SetMainShakeTimer50(k: u16, game: &mut Game) -> u16 {
    (*Get_MotherBrain(0x40i32, game)).mbn_var_00 = 50u16;
    k
}
pub unsafe fn MotherBrain_Instr_GotoEitherOr(k: u16, game: &mut Game) -> u16 {
    let result: u16 = addr_stru_A99C5F as u16;
    if (ram_word!(game, random_number) as i32 & 0xfffi32) >= 0xfe0i32 {
        return addr_stru_A99C47 as u16;
    }
    result
}
pub unsafe fn MotherBrain_Instr_MaybeGoto(k: u16, game: &mut Game) -> u16 {
    if ram_word!(game, random_number) < 0xf000u16 {
        return addr_stru_A99C9F as u16;
    }
    k
}
pub unsafe fn MotherBrain_Instr_MaybeGoto2(k: u16, game: &mut Game) -> u16 {
    if (ram_word!(game, random_number) as i32 & 0xfffi32) < 0xec0i32 {
        return MotherBrain_Instr_Goto2(k);
    }
    k
}
pub unsafe fn MotherBrain_Instr_QueueShitroidAttackSfx(k: u16, game: &mut Game) -> u16 {
    if (*Get_MotherBrain(0i32, game)).mbn_var_13 != 11u16 {
        QueueSfx2_Max6(g_word_A99E0F[0i32 as usize], game);
    }
    k
}
pub unsafe fn MotherBrain_Instr_Goto2(k: u16) -> u16 {
    addr_stru_A99CD1 as u16
}
pub unsafe fn MotherBrain_Instr_SpawnBlueRingEproj(k: u16, game: &mut Game) -> u16 {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    SpawnEprojWithRoomGfx(
        addr_kEproj_MotherBrainBlueRingLasers as u16,
        (*E).mbn_var_1A,
        game,
    );
    k
}
pub unsafe fn MotherBrain_Instr_AimRingsAtShitroid(k: u16, game: &mut Game) -> u16 {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
    let E1: *mut Enemy_MotherBrain = Get_MotherBrain((*E).mbn_var_0A as i32, game);
    MotherBrain_Instr_AimRings(
        ((*E1).base.x_pos as i32 - (*E).base.x_pos as i32 - 10i32) as u16,
        ((*E1).base.y_pos as i32 - (*E).base.y_pos as i32 - 16i32) as u16,
        game,
    );
    k
}
pub unsafe fn MotherBrain_Instr_AimRingsAtSamus(k: u16, game: &mut Game) -> u16 {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
    MotherBrain_Instr_AimRings(
        (ram_word!(game, samus_x_pos) as i32 - (*E).base.x_pos as i32 - 10i32) as u16,
        (ram_word!(game, samus_y_pos) as i32 - (*E).base.y_pos as i32 - 16i32) as u16,
        game,
    );
    k
}
pub unsafe fn MotherBrain_Instr_AimRings(x: u16, y: u16, game: &mut Game) {
    let mut v0: u16 = -(CalculateAngleFromXY(x, y, game) as i32 + 0x80i32) as u8 as u16;
    let clamp_ring_angle: bool = if v0 as i32 - 16i32 & 0x80i32 == 0 {
        !((v0 as u8) < 0x48u8)
    } else {
        (v0 as i32 + 64i32 & 0x80i32 != 0)
            || ({
                *(&raw mut v0 as *mut u8).offset(0) = 16u8;
                false
            })
    };
    if clamp_ring_angle {
        *(&raw mut v0 as *mut u8).offset(0) = 72u8;
    }
    (*Get_MotherBrain(0i32, game)).mbn_var_1A = v0;
}
pub unsafe fn MotherBrain_Instr_IncrShitroidAttackCtr(k: u16, game: &mut Game) -> u16 {
    let mut v3: i16 = 0;
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    v3 = (((*E).mbn_var_13) as i16).wrapping_add(1i16);
    if (v3 as u16) >= 0xcu16 {
        v3 = 12i16;
    }
    (*E).mbn_var_13 = v3 as u16;
    k
}
pub unsafe fn MotherBrain_Instr_SetShitroidAttackCtr0(k: u16, game: &mut Game) -> u16 {
    (*Get_MotherBrain(0i32, game)).mbn_var_13 = 0u16;
    k
}
pub unsafe fn MotherBrain_Instr_SpawnBombEproj(k: u16, game: &mut Game) -> u16 {
    let v2: *const u16 = RomPtr_A9(k, game) as *const u16;
    SpawnEprojWithRoomGfx(addr_kEproj_MotherBrainBomb as u16, *v2, game);
    (k).wrapping_add(2u16)
}
pub unsafe fn MotherBrain_Instr_SpawnLaserEproj(k: u16, game: &mut Game) -> u16 {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
    (*E).mbn_var_31 = 0u16;
    game.eproj_spawn_r22 = 1u16;
    game.eproj_spawn_pt = Point16U {
        x: (((*E).base.x_pos).wrapping_add(16u16)),
        y: (((*E).base.y_pos).wrapping_add(4u16)),
    };
    SpawnEprojWithRoomGfx(addr_stru_86A17B as u16, 1u16, game);
    k
}
pub unsafe fn MotherBrain_Instr_SpawnRainbowEproj(k: u16, game: &mut Game) -> u16 {
    SpawnEprojWithRoomGfx(
        addr_kEproj_MotherBrainRainbowBeamCharging as u16,
        0u16,
        game,
    );
    k
}
pub unsafe fn MotherBrain_Instr_SetupFxForRainbowBeam(k: u16, game: &mut Game) -> u16 {
    (*Get_MotherBrain(0x40i32, game)).mbn_var_14 = 0u16;
    MotherBrain_SetupBrainPalForLaser(game);
    QueueSfx2_Max6(0x7fu16, game);
    k
}
pub unsafe fn MotherBrain_Phase3_Death_0(game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
    let E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
    (*E1).base.properties = ((*E1).base.properties) | (0x400u16);
    (*E).mbn_var_04 = 0u16;
    if MotherBrain_MakeWalkBackwards(0x28u16, 6u16, game) as i32 & 1i32 != 0 {
        (*E).mbn_var_A = ((0xa9af12i32) as u16);
        (*E).mbn_var_F = 128u16;
        MotherBrain_Phase3_Death_1(game);
    }
}
pub unsafe fn MotherBrain_Phase3_Death_1(game: &mut Game) {
    MotherBrain_GenerateSmokyExplosions(game);
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if ((*E).mbn_var_F as i32 & 0x8000i32) != 0i32 {
        (*E).mbn_var_A = ((0xa9af21i32) as u16);
    }
}
pub unsafe fn MotherBrain_Phase3_Death_2(game: &mut Game) {
    MotherBrain_GenerateSmokyExplosions(game);
    if MotherBrain_MakeWalkForwards(2u16, 0x60u16, game) as i32 & 1i32 != 0 {
        MotherBrain_SetBrainInstrs(addr_kMotherBrain_Ilist_9C39 as u16, game);
        let E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
        (*E1).mbn_var_32 = 6u16;
        (*E1).mbn_var_33 = 6u16;
        (*E1).mbn_var_34 = 1280u16;
        let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
        (*E).mbn_var_A = ((0xa9af54i32) as u16);
        (*E).mbn_var_F = 32u16;
    }
}
pub unsafe fn MotherBrain_Phase3_Death_3(game: &mut Game) {
    MotherBrain_GenerateSmokyExplosions(game);
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if ((*E).mbn_var_F as i32 & 0x8000i32) != 0i32 {
        let E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
        (*E1).mbn_var_32 = 0u16;
        (*E1).mbn_var_33 = 0u16;
        (*E1).mbn_var_12 = 0u16;
        (*E1).mbn_var_14 = 0u16;
        (*E1).mbn_var_10 = 0u16;
        (*E1).mbn_var_11 = 0u16;
        let mut i: i32 = 28i32;
        while i >= 0i32 {
            set_ram_word!(
                game.palette_buffer[((i >> 1i32) + 241i32)] =
                    ram_word!(game, palette_buffer[((i >> 1i32) + 145i32)])
            );
            i -= 2i32;
        }
        MotherBrain_HealthBasedPaletteHandling(game);
        (*E).mbn_var_0D = 3584u16;
        (*E1).mbn_var_E = 0u16;
        (*E1).mbn_var_F = 0u16;
        (*E).mbn_var_A = ((0xa9af9di32) as u16);
        MotherBrain_Phase3_Death_4(game);
    }
}
pub unsafe fn MotherBrain_Phase3_Death_4(game: &mut Game) {
    MotherBrain_GenerateMixedExplosions(game);
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if ((*E).mbn_var_F as i32 & 0x8000i32) != 0i32 {
        (*E).mbn_var_37 = 0u16;
        (*E).mbn_var_A = ((0xa9afb6i32) as u16);
        (*E).mbn_var_F = 0u16;
        MotherBrain_Phase3_Death_5(game);
    }
}
pub unsafe fn MotherBrain_Phase3_Death_5(game: &mut Game) {
    HandleMotherBrainBodyFlickering(game);
    MotherBrain_GenerateMixedExplosions(game);
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if ((*E).mbn_var_F as i32 & 0x8000i32) != 0i32 {
        (*E).mbn_var_F = 16u16;
        let v2: u16 = ((*E).mbn_var_37).wrapping_add(1u16);
        (*E).mbn_var_37 = v2;
        if MotherBrain_FadePalToBlack((v2).wrapping_sub(1u16), game) as i32 & 1i32 != 0 {
            set_ram_word!(game.enemy_bg2_tilemap_size = 710u16);
            let mut i: i32 = 710i32;
            while i >= 0i32 {
                set_ram_word!(game.tilemap_stuff[(i >> 1i32)] = 824u16);
                i -= 2i32;
            }
            set_ram_word!(game.nmi_flag_bg2_enemy_vram_transfer = 1u16);
            (*E).base.properties = ((*E).base.properties as i32 & 0xdeffi32 | 0x100i32) as u16;
            (*E).base.extra_properties = 0u16;
            (*E).mbn_var_A = ((0xa9b013i32) as u16);
            (*E).mbn_var_F = 16u16;
        }
    }
}
pub unsafe fn MotherBrain_Phase3_Death_6(game: &mut Game) {
    MotherBrain_GenerateMixedExplosions(game);
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if ((*E).mbn_var_F as i32 & 0x8000i32) != 0i32 {
        (*E).mbn_var_A = ((0xa9b115i32) as u16);
    }
}
pub unsafe fn MotherBrain_GenerateSmokyExplosions(game: &mut Game) {
    MotherBrain_GenerateExplosions(0x10u16, addr_word_A9B10F as u16, 2u16, game);
}
pub unsafe fn MotherBrain_GenerateMixedExplosions(game: &mut Game) {
    MotherBrain_GenerateExplosions(8u16, addr_word_A9B109 as u16, 4u16, game);
}
pub unsafe fn MotherBrain_GenerateExplosions(a: u16, r22: u16, r24: u16, game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
    (*E).mbn_var_E = (*E).mbn_var_E.wrapping_sub(1);
    let mut v2: bool = ((*E).mbn_var_E as i32 & 0x8000i32) != 0i32;
    if v2 {
        (*E).mbn_var_E = a;
        (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
        v2 = ((*E).mbn_var_F as i32 & 0x8000i32) != 0i32;
        if v2 {
            (*E).mbn_var_F = 6u16;
        }
        let mut v3: u16 = (16u16).wrapping_mul((*E).mbn_var_F);
        let mut v4: u16 = r24;
        let mut v9: u16 = 0;
        loop {
            v9 = v4;
            let v5: i32 = v3 as i32 >> 1i32;
            game.eproj_spawn_pt = Point16U {
                x: crate::rtl::rom_table_word!(game, 0xa9b099, v5 as isize),
                y: crate::rtl::rom_table_word!(game, 0xa9b099, (v5 + 1i32) as isize),
            };
            let v6: *const u16 = RomPtr_A9(r22, game) as *const u16;
            let mut v7: u16 = *v6;
            let Random: u16 = NextRandom(game);
            if Random >= 0x4000u16 {
                v7 = *v6.offset(1);
                if Random >= 0xe000u16 {
                    v7 = *v6.offset(2);
                }
            }
            SpawnEprojWithRoomGfx(addr_kEproj_MotherBrainDeathExplosion as u16, v7, game);
            v3 = (v3).wrapping_add(4u16);
            v4 = (v9).wrapping_sub(1u16);
            if (v9 == 1u16) {
                break;
            }
        }
        QueueSfx3_Max3(0x13u16, game);
    }
}
pub unsafe fn MotherBrain_Phase3_Death_7(game: &mut Game) {
    MotherBrain_SetBrainInstrs(addr_kMotherBrain_Ilist_9C29 as u16, game);
    (*Get_MotherBrain(0x40i32, game)).mbn_var_A = ((0xa987d0i32) as u16);
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).mbn_var_F = 0u16;
    (*E).mbn_var_A = ((0xa9b12di32) as u16);
    MotherBrain_Phase3_Death_8(game);
}
pub unsafe fn MotherBrain_Phase3_Death_8(game: &mut Game) {
    let mut v3: i16 = 0;
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    let mut v1: u16 = ((*E).mbn_var_F).wrapping_add(32u16);
    (*E).mbn_var_F = v1;
    let E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
    v3 =
        (((*E1).base.y_pos) as i16).wrapping_add((*(&raw mut v1 as *mut u8).offset(1isize)) as i16);
    if (v3 as u16) >= 0xc4u16 {
        EnableEarthquakeAframes(2u16, game);
        (*E).mbn_var_A = ((0xa9b15ei32) as u16);
        (*E).mbn_var_F = 256u16;
        v3 = 196i16;
    }
    (*E1).base.y_pos = v3 as u16;
}
pub unsafe fn MotherBrain_Phase3_Death_9(game: &mut Game) {
    if ProcessSpriteTilesTransfers(0xa9u8, addr_stru_A99003 as u16, game) != 0 {
        let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
        (*E).mbn_var_A = ((0xa9b173i32) as u16);
        (*E).mbn_var_F = 32u16;
    }
}
pub unsafe fn MotherBrain_Phase3_Death_10(game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if ((*E).mbn_var_F as i32 & 0x8000i32) != 0i32 {
        (*E).mbn_var_37 = 0u16;
        (*E).mbn_var_A = ((0xa9b189i32) as u16);
        (*E).mbn_var_F = 0u16;
    }
}
pub unsafe fn MotherBrain_Phase3_Death_11(game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if ((*E).mbn_var_F as i32 & 0x8000i32) != 0i32 {
        let v2: u16 = ((*E).mbn_var_37).wrapping_add(1u16);
        (*E).mbn_var_37 = v2;
        if MotherBrain_FadeToGray_RealDeath((v2).wrapping_sub(1u16), game) as i32 & 1i32 != 0 {
            MotherBrain_SetBrainInstrs(addr_kMotherBrain_Ilist_9D25 as u16, game);
            (*E).mbn_var_A = ((0xa9b1b8i32) as u16);
            (*E).mbn_var_F = 256u16;
        } else {
            (*E).mbn_var_F = 16u16;
        }
    }
}
pub unsafe fn MotherBrain_Phase3_Death_12(game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if ((*E).mbn_var_F as i32 & 0x8000i32) != 0i32 {
        (*E).mbn_var_A = ((0xa9b1d5i32) as u16);
        let E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
        (*E1).base.properties = ((*E1).base.properties) | (0x400u16);
        (*E).mbn_var_04 = 0u16;
    }
}
pub unsafe fn MotherBrain_Phase3_Death_13(game: &mut Game) {
    if ProcessCorpseRotting(0x40u16, game) as i32 & 1i32 != 0 {
        let dms_var_53: u16 = (*Get_DeadMonsters(0x40i32, game)).dms_var_53;
        ProcessCorpseRottingVramTransfers(dms_var_53, game);
    } else {
        let mut E1: *mut Enemy_DeadMonsters = Get_DeadMonsters(0x40i32, game);
        (*E1).base.properties = ((*E1).base.properties as i32 & 0xdeffi32 | 0x100i32) as u16;
        (*E1).base.extra_properties = 0u16;
        QueueMusic_Delayed8(0u16, game);
        QueueMusic_Delayed8(0xff24u16, game);
        E1 = Get_DeadMonsters(0i32, game);
        (*E1).dms_var_A = ((0xa9b211i32) as u16);
        (*E1).dms_var_F = 20u16;
        MotherBrain_Phase3_Death_14_20framedelay(game);
    };
}
pub unsafe fn MotherBrain_Phase3_Death_14_20framedelay(game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if ((*E).mbn_var_F as i32 & 0x8000i32) != 0i32 {
        let E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
        (*E1).base.x_pos = 0u16;
        (*E1).base.y_pos = 0u16;
        (*E).mbn_var_A = ((0xa9b258i32) as u16);
    }
}
pub unsafe fn MotherBrain_CorpseRottingFinished(game: &mut Game) {
    let E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
    game.eproj_spawn_pt = Point16U {
        x: ((*E1).base.x_pos as i32 + (ram_word!(game, random_number) as i32 & 0x1fi32) - 16i32)
            as u16,
        y: (((*E1).base.y_pos).wrapping_add(16u16)),
    };
    SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 0xau16, game);
    if (ram_word!(game, random_enemy_counter) as i32 & 7i32) == 0i32 {
        QueueSfx2_Max3(0x10u16, game);
    }
}
pub unsafe fn MotherBrain_Phase3_Death_15_LoadEscapeTimerTiles(game: &mut Game) {
    if ProcessSpriteTilesTransfers(0xa6u8, addr_stru_A6C4CB as u16, game) != 0 {
        (*Get_MotherBrain(0i32, game)).mbn_var_A = ((0xa9b26di32) as u16);
        MotherBrain_Phase3_Death_16_StartEscape(game);
    }
}
pub unsafe fn MotherBrain_Phase3_Death_16_StartEscape(game: &mut Game) {
    if ProcessSpriteTilesTransfers(0xa9u8, addr_stru_A9902F as u16, game) != 0 {
        WriteColorsToPalette(
            0x122u16,
            0xa9u8,
            (addr_kMotherBrainPalette_5 as i32 + 2i32) as u16,
            0xeu16,
            game,
        );
        QueueMusic_Delayed8(7u16, game);
        set_ram_word!(game.earthquake_type = 5u16);
        set_ram_word!(game.earthquake_timer = -1i32 as u16);
        SpawnPalfxObject(addr_kPalfx_FFC9 as u16, game);
        SpawnPalfxObject(addr_kPalfx_FFCD as u16, game);
        SpawnPalfxObject(addr_kPalfx_FFD1 as u16, game);
        SpawnPalfxObject(addr_kPalfx_FFD5 as u16, game);
        (*Get_MotherBrain(0x40i32, game)).mbn_var_02 = 0u16;
        SetupZebesEscapeTypewriter(game);
        let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
        (*E).mbn_var_F = 32u16;
        let mut v1: u16 = ((0xa9b2d1i32) as u16);
        if ram_word!(game, japanese_text_flag) == 0 {
            v1 = ((0xa9b2e3i32) as u16);
        }
        (*E).mbn_var_A = v1;
    }
}
pub unsafe fn MotherBrain_Phase3_Death_17_SpawnTimeBomb(game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if ((*E).mbn_var_F as i32 & 0x8000i32) != 0i32 {
        (*E).mbn_var_A = ((0xa9b2e3i32) as u16);
        SpawnEprojWithRoomGfx(
            addr_kEproj_TimeBombSetJapaneseText as u16,
            ((0xa9b2e3i32) as u16),
            game,
        );
    }
    MotherBrain_Phase3_Death_18_TypesZebesText(game);
}
pub unsafe fn MotherBrain_Phase3_Death_18_TypesZebesText(game: &mut Game) {
    if HandleTypewriterText_Ext(0x2610u16, game) as i32 & 1i32 != 0 {
        let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
        (*E).mbn_var_A = ((0xa9b2f9i32) as u16);
        (*E).mbn_var_F = 32u16;
    }
}
pub unsafe fn MotherBrain_Phase3_Death_19_EscapeDoorExploding(game: &mut Game) {
    MotherBrain_GenerateEscapeDoorExploding(game);
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if ((*E).mbn_var_F as i32 & 0x8000i32) != 0i32 {
        CallSomeSamusCode(0xfu16, game);
        set_ram_word!(game.timer_status = 2u16);
        SetBossBitForCurArea(2u16, game);
        SetEventHappened(0xeu16, game);
        (*E).mbn_var_A = ((0xa9b32ai32) as u16);
        let E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
        (*E1).mbn_var_E = 0u16;
        (*E1).mbn_var_F = 0u16;
    }
}
pub unsafe fn MotherBrain_Phase3_Death_20_BlowUpEscapeDoor(game: &mut Game) {
    MotherBrain_ExplodeEscapeDoor(game);
    (*Get_MotherBrain(0i32, game)).mbn_var_A = ((0xa9b33ci32) as u16);
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs::new(0u8, 0x6u8, 0xb677 as VoidP),
        game,
    );
}
pub unsafe fn MotherBrain_Phase3_Death_21_KeepEarthquakeGoing(game: &mut Game) {
    if ram_word!(game, earthquake_timer) == 0 {
        update_ram_word!(game.earthquake_timer, wrapping_sub(1));
    }
}
pub unsafe fn MotherBrain_GenerateEscapeDoorExploding(game: &mut Game) {
    let mut v1: i16 = 0;
    let mut v2: i16 = 0;
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
    v1 = (((*E).mbn_var_E) as i16).wrapping_sub(1i16);
    (*E).mbn_var_E = v1 as u16;
    if v1 < 0i16 {
        (*E).mbn_var_E = 4u16;
        v2 = (((*E).mbn_var_F) as i16).wrapping_sub(1i16);
        (*E).mbn_var_F = v2 as u16;
        if v2 < 0i16 {
            (*E).mbn_var_F = 3u16;
        }
        let v3: i32 = ((4u16).wrapping_mul((*E).mbn_var_F)) as i32 >> 1i32;
        game.eproj_spawn_pt = Point16U {
            x: g_word_A9B393[v3 as usize],
            y: g_word_A9B393[(v3 + 1i32) as usize],
        };
        let mut v4: u16 = 3u16;
        if NextRandom(game) < 0x4000u16 {
            v4 = 12u16;
        }
        SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, v4, game);
        QueueSfx2_Max3(0x24u16, game);
    }
}
pub unsafe fn MotherBrain_ExplodeEscapeDoor(game: &mut Game) {
    let mut v0: u16 = 0u16;
    let mut v1: u16 = 0;
    loop {
        v1 = v0;
        SpawnEprojWithRoomGfx(addr_kEproj_MotherBrainExplodedEscapeDoor as u16, v0, game);
        v0 = (v1).wrapping_add(1u16);
        if ((v1).wrapping_add(1u16) >= 8u16) {
            break;
        }
    }
}
pub unsafe fn MotherBrain_SamusCollDetect(game: &mut Game) {
    let mut v1: i8 = 0;
    let E0: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    let mut R26: u16 = (*E0).mbn_var_04;
    v1 = ((R26) as i8) & (1i8);
    R26 = (R26 as i32 >> 1i32) as u16;
    if v1 == 0
        || MotherBrain_SamusCollDetectPart(
            addr_word_A9B427 as u16,
            (*E0).base.x_pos,
            (*E0).base.y_pos,
            game,
        ) as i32
            & 1i32
            == 0
    {
        let E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
        v1 = ((R26) as i8) & (1i8);
        R26 = (R26 as i32 >> 1i32) as u16;
        if v1 == 0
            || MotherBrain_SamusCollDetectPart(0xb439u16, (*E).base.x_pos, (*E).base.y_pos, game)
                as i32
                & 1i32
                == 0
        {
            v1 = ((R26) as i8) & (1i8);
            R26 = (R26 as i32 >> 1i32) as u16;
            if v1 != 0
                && MotherBrain_SamusCollDetectPart(
                    0xb44bu16,
                    (*E).mbn_var_25,
                    (*E).mbn_var_26,
                    game,
                ) as i32
                    & 1i32
                    == 0
                && MotherBrain_SamusCollDetectPart(
                    0xb44bu16,
                    (*E).mbn_var_28,
                    (*E).mbn_var_29,
                    game,
                ) as i32
                    & 1i32
                    == 0
            {
                MotherBrain_SamusCollDetectPart(0xb44bu16, (*E).mbn_var_2B, (*E).mbn_var_2C, game);
            }
        }
    }
}
pub unsafe fn MotherBrain_SamusCollDetectPart(k: u16, r18: u16, r20: u16, game: &mut Game) -> u8 {
    let mut v5: i16 = 0;
    let mut p: *const u8 = RomPtr_A9(k, game);
    let mut n: i32 = *(p as *mut u16) as i32;
    p = p.offset(2);
    while n != 0 {
        let mut v3: u16 = 0;
        let mut v4: u16 = 0;
        let mut R24: u16 = 0;
        if ((ram_word!(game, samus_y_pos)) as i16).wrapping_sub((r20) as i16) >= 0i16 {
            R24 = (ram_word!(game, samus_y_pos)).wrapping_sub(r20);
            v3 = *(p.offset(6) as *mut u16);
        } else {
            R24 = (r20).wrapping_sub(ram_word!(game, samus_y_pos));
            v3 = *(p.offset(2) as *mut u16);
        }
        if ((ram_word!(game, samus_y_radius) as i32 + abs16(v3) as i32 - R24 as i32) as i16) >= 0i16
        {
            if ((ram_word!(game, samus_x_pos)) as i16).wrapping_sub((r18) as i16) >= 0i16 {
                R24 = (ram_word!(game, samus_x_pos)).wrapping_sub(r18);
                v4 = *(p.offset(4) as *mut u16);
            } else {
                R24 = (r18).wrapping_sub(ram_word!(game, samus_x_pos));
                v4 = *(p as *mut u16);
            }
            v5 = (ram_word!(game, samus_x_radius) as i32 + abs16(v4) as i32 - R24 as i32) as i16;
            if v5 >= 0i16 {
                if v5 as i32 - 4i32 & 0x8000i32 != 0 {
                    v5 = 4i16;
                }
                set_ram_word!(game.extra_samus_x_displacement = v5 as u16);
                set_ram_word!(game.extra_samus_y_displacement = 4u16);
                set_ram_words! { game, 0u16;
                    extra_samus_x_subdisplacement, extra_samus_y_subdisplacement,
                }
                set_ram_word!(game.samus_invincibility_timer = 96u16);
                set_ram_word!(game.samus_knockback_timer = 5u16);
                set_ram_word!(game.knockback_x_dir = 1u16);
                if ram_word!(game, samus_y_pos) as i32 - 192i32 & 0x8000i32 != 0 {
                    set_ram_word!(game.samus_y_dir = 2u16);
                }
                if (((*Get_MotherBrain(0i32, game)).base.x_pos as i32 + 24i32
                    - ram_word!(game, samus_x_pos) as i32) as i16)
                    < 0i16
                {
                    MotherBrain_HurtSamus(game);
                }
                return 1u8;
            }
        }
        p = p.offset(8);
        n -= 1;
    }
    0u8
}
pub unsafe fn MotherBrainsBody_Shot(game: &mut Game) {
    CreateDudShot(game);
}
pub unsafe fn MotherBrainsBrain_Shot(game: &mut Game) {
    let mut v2: i16 = 0;
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    if (*E).mbn_var_00 != 0 {
        MotherBrain_Phase23_ShotReaction(game);
        if (*E).mbn_var_00 == 1u16 {
            CreateDudShot(game);
        } else {
            NormalEnemyShotAiSkipDeathAnim_CurEnemy(game);
        }
    } else {
        let v1: u16 = ((*((ram_ptr!(game, u16; 0xc18))
            .offset(ram_word!(game, collision_detection_index) as isize)
            as *mut u8)
            .offset(1isize)) as u16)
            & (7u16);
        if g_byte_A9B546[v1 as usize] != 0 {
            update_ram_word!(
                game.plm_room_arguments[39],
                wrapping_add((g_byte_A9B546[v1 as usize]) as u16)
            );
            QueueSfx2_Max6(0x6eu16, game);
            v2 = 13i16;
            let E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
            let flash_timer: u16 = (*E1).base.flash_timer;
            if flash_timer != 0 && flash_timer as i32 & 1i32 != 0 {
                v2 = 14i16;
            }
            (*E1).base.flash_timer = v2 as u16;
            NormalEnemyShotAiSkipDeathAnim_CurEnemy(game);
        }
    };
}
pub unsafe fn MotherBrain_Phase23_ShotReaction(game: &mut Game) {
    let mut v1: i16 = 0;
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    if (*E).mbn_var_00 == 4u16 && MotherBrain_DetermineShotReactionType(game) == 2u16 {
        MotherBrain_Phase3_BeamShotReaction(game);
    } else {
        if MotherBrain_DetermineShotReactionType(game) == 1u16 || {
            v1 = (((*E).mbn_var_07) as i16).wrapping_sub(256i16);
            v1 < 0i16
        } {
            v1 = 0i16;
        }
        (*E).mbn_var_07 = v1 as u16;
    };
}
pub unsafe fn MotherBrain_DetermineShotReactionType(game: &mut Game) -> u16 {
    g_byte_A9B5A1[(*((ram_ptr!(game, u16; 0xc18))
        .offset(ram_word!(game, collision_detection_index) as isize)
        as *mut u8)
        .offset(1isize) as i32
        & 7i32) as usize] as u16
}
pub unsafe fn MotherBrain_Phase3_BeamShotReaction(game: &mut Game) {
    let mut v1: i16 = 0;
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    v1 = (((*E).mbn_var_07) as i16).wrapping_sub(266i16);
    if v1 < 0i16 {
        (*Get_MotherBrain(0x40i32, game)).mbn_var_18 = ((0xa9c395i32) as u16);
        v1 = 0i16;
        (*E).mbn_var_F = 0u16;
    }
    (*E).mbn_var_07 = v1 as u16;
}
pub unsafe fn MotherBrainsBrain_Touch(game: &mut Game) {
    let mut v0: i16 = 0;
    if *ram_ptr!(game; 0xa1f) == 3u8 {
        v0 = 13i16;
        let v1: *mut EnemyData = gEnemyData(0i32, game);
        let flash_timer: u16 = (*v1.offset(1)).flash_timer;
        if flash_timer != 0 && flash_timer as i32 & 1i32 != 0 {
            v0 = 14i16;
        }
        (*v1.offset(1)).flash_timer = v0 as u16;
    }
}
pub unsafe fn MotherBrain_HurtSamus(game: &mut Game) {
    Ridley_Func_98(game);
    set_ram_word!(game.samus_invincibility_timer = 96u16);
    set_ram_word!(game.samus_knockback_timer = 5u16);
    set_ram_word!(
        game.knockback_x_dir = (((ram_word!(game, samus_x_pos)) as i16).wrapping_sub(
            ((*Get_MotherBrain(ram_word!(game, cur_enemy_index) as i32, game))
                .base
                .x_pos) as i16,
        ) >= 0i16) as u16
    );
}
pub unsafe fn MotherBrain_Body_Phase2_Thinking(game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    let E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
    if (*E1).base.health == 0 {
        (*E).mbn_var_A = ((0xa9b8ebi32) as u16);
        MotherBomb_FiringRainbowBeam_0(game);
        return;
    }
    if (*E).mbn_var_02 == 0 {
        let keep_walking: bool;
        if (*E1).base.health as i32 - 4500i32 & 0x8000i32 == 0 {
            keep_walking = (ram_word!(game, random_number) >= 0x1000u16);
        } else if ram_word!(game, random_number) < 0x2000u16 {
            keep_walking = true;
        } else if ram_word!(game, random_number) >= 0xa000u16 {
            keep_walking = false;
        } else {
            (*E).mbn_var_A = ((0xa9b87di32) as u16);
            return;
        }
        if keep_walking {
            MotherBrain_HandleWalking(game)
        } else {
            (*E).mbn_var_A = ((0xa9b64bi32) as u16);
        }
    }
}
pub unsafe fn MotherBrain_Body_Phase2_TryAttack(game: &mut Game) {
    let mut v0: i16 = 0;
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    v0 = (2i16).wrapping_mul(((*E).mbn_var_18) as i16);
    if v0 != 0 {
        if v0 == 2i16 {
            MotherBrain_Phase2_Attack_Cooldown(game);
        } else {
            if v0 != 4i16 {
                Unreachable(game);
                loop {}
            }
            MotherBrain_Phase2_Attack_End(game);
        }
    } else {
        (*E).mbn_parameter_1 = 64u16;
        (*E).mbn_var_18 = (*E).mbn_var_18.wrapping_add(1);
        if MotherBrain_Phase2_DecideAttackStrategy_DoubleRet(game) != 0 {
            return;
        }
        let mut v2: u16 = addr_byte_A9B6DC as u16;
        let E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
        if abs16(((*E1).base.y_pos as i32 + 4i32 - ram_word!(game, samus_y_pos) as i32) as u16)
            < 0x20u16
        {
            v2 = addr_byte_A9B6DF as u16;
        }
        let mut v4: u16 = 0u16;
        let v5: *const u8 = RomPtr_A9(v2, game);
        if (ram_word!(game, random_number) as u8) >= *v5 {
            v4 = 2u16;
            if (ram_word!(game, random_number) as u8) >= *v5.offset(1) {
                v4 = 4u16;
                if (ram_word!(game, random_number) as u8) >= *v5.offset(2) {
                    v4 = 6u16;
                }
            }
        }
        let v6: u16 = crate::rtl::rom_table_word!(game, 0xa9b6d4, (v4 as i32 >> 1i32) as isize);
        if v6 as i32 == addr_kMotherBrain_Ilist_9ECC as i32 {
            if (*E1).mbn_var_05 as i32 - 1i32 & 0x8000i32 != 0 {
                (*E).mbn_var_A = ((0xa9b781i32) as u16);
                MotherBrain_FiringBomb_DecideOnWalking(game);
            }
        } else if v6 as i32 == addr_kMotherBrain_Ilist_9F34 as i32 {
            (*E).mbn_var_A = ((0xa9b80ei32) as u16);
            MotherBomb_FiringLaser_PositionHead(game);
        } else {
            MotherBrain_SetBrainInstrs(v6, game);
        }
    };
}
pub unsafe fn MotherBrain_Phase2_DecideAttackStrategy_DoubleRet(game: &mut Game) -> u8 {
    if g_word_A9B72C[*ram_ptr!(game; 0xa1f) as usize] != 0 {
        if ram_word!(game, random_number) as u8 as i32 - 128i32 & 0x8000i32 != 0 {
            0u8
        } else {
            let E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
            if (*E1).mbn_var_05 as i32 - 1i32 & 0x8000i32 != 0 {
                (*Get_MotherBrain(0i32, game)).mbn_var_A = ((0xa9b781i32) as u16);
                MotherBrain_FiringBomb_DecideOnWalking(game);
                return 1u8;
            }
            0u8
        }
    } else {
        if ram_word!(game, random_number) as u8 as i32 - 128i32 & 0x8000i32 != 0 {
            (*Get_MotherBrain(0i32, game)).mbn_var_A = ((0xa9b80ei32) as u16);
            MotherBomb_FiringLaser_PositionHead(game);
        } else {
            MotherBrain_SetBrainInstrs(addr_kMotherBrain_Ilist_9D3D as u16, game);
        }
        1u8
    }
}
pub unsafe fn MotherBrain_Phase2_Attack_Cooldown(game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    let fresh9 = (*E).mbn_parameter_1;
    (*E).mbn_parameter_1 = (*E).mbn_parameter_1.wrapping_sub(1);
    if fresh9 == 1u16 {
        (*E).mbn_var_18 = (*E).mbn_var_18.wrapping_add(1);
    }
}
pub unsafe fn MotherBrain_Phase2_Attack_End(game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).mbn_var_18 = 0u16;
    (*E).mbn_var_A = ((0xa9b605i32) as u16);
}
pub unsafe fn MotherBrain_FiringBomb_DecideOnWalking(game: &mut Game) {
    if ram_word!(game, random_number) >= 0xff80u16 {
        MotherBrain_FiringBomb_DecideOnCrouching(game);
        return;
    }
    let mut v0: u16 = 64u16;
    if ram_word!(game, random_number) < 0x6000u16 {
        v0 = 96u16;
    }
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    if ((v0) as i16).wrapping_sub(((*E).base.x_pos) as i16) >= 0i16 || {
        (*E).mbn_var_F = v0;
        MotherBrain_MakeWalkBackwards(v0, 6u16, game) as i32 & 1i32 != 0
    } {
        MotherBrain_FiringBomb_DecideOnCrouching(game);
    } else {
        (*E).mbn_var_A = ((0xa9b7aci32) as u16);
    };
}
pub unsafe fn MotherBrain_FiringBomb_WalkingBackwards(game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    if MotherBrain_MakeWalkBackwards((*E).mbn_var_F, 6u16, game) as i32 & 1i32 != 0 {
        MotherBrain_FiringBomb_DecideOnCrouching(game);
    }
}
pub unsafe fn MotherBrain_FiringBomb_DecideOnCrouching(game: &mut Game) {
    if NextRandom(game) >= 0x8000u16 {
        (*Get_MotherBrain(0i32, game)).mbn_var_A = ((0xa9b7c6i32) as u16);
        MotherBrain_FiringBomb_Crouch(game);
    } else {
        MotherBrain_B7CB(game);
    };
}
pub unsafe fn MotherBrain_FiringBomb_Crouch(game: &mut Game) {
    if MotherBrain_MakeHerCrouch(game) as i32 & 1i32 != 0 {
        MotherBrain_B7CB(game);
    }
}
pub unsafe fn MotherBrain_B7CB(game: &mut Game) {
    let mut v0: u16 = addr_kMotherBrain_Ilist_9ECC as u16;
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    if (*E).mbn_var_1F != 0 {
        v0 = addr_kMotherBrain_Ilist_9F00 as u16;
    }
    MotherBrain_SetBrainInstrs(v0, game);
    (*E).mbn_var_A = ((0xa9b7e8i32) as u16);
    (*E).mbn_var_F = 44u16;
}
pub unsafe fn MotherBrain_FiringBomb_Fired(game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if ((*E).mbn_var_F as i32 & 0x8000i32) != 0i32 {
        if MotherBrain_MakeHerStandUp(game) as i32 & 1i32 != 0 {
            MotherBrain_FiringBomb_Finish(game);
        } else {
            (*E).mbn_var_A = ((0xa9b7f8i32) as u16);
            MotherBrain_FiringBomb_Standup(game);
        }
    }
}
pub unsafe fn MotherBrain_FiringBomb_Standup(game: &mut Game) {
    if MotherBrain_MakeHerStandUp(game) as i32 & 1i32 != 0 {
        MotherBrain_FiringBomb_Finish(game);
    }
}
pub unsafe fn MotherBrain_FiringBomb_Finish(game: &mut Game) {
    let mut v0: u16 = ((0xa9b605i32) as u16);
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    if (*E).mbn_var_1F != 0 {
        v0 = ((0xa9c15ci32) as u16);
    }
    (*E).mbn_var_A = v0;
}
pub unsafe fn MotherBomb_FiringLaser_PositionHead(game: &mut Game) {
    let mut v0: i16 = 0;
    v0 = 8i16;
    let E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
    if (((*E1).base.y_pos) as i16).wrapping_sub((ram_word!(game, samus_y_pos)) as i16) < 0i16 {
        v0 = 6i16;
    }
    (*E1).mbn_var_32 = v0 as u16;
    (*E1).mbn_var_33 = v0 as u16;
    (*E1).mbn_var_34 = 512u16;
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).mbn_var_A = ((0xa9b839i32) as u16);
    (*E).mbn_var_F = 4u16;
}
pub unsafe fn MotherBomb_FiringLaser_PositionHeadSlowlyFire(game: &mut Game) {
    let mut v2: i16 = 0;
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if ((*E).mbn_var_F as i32 & 0x8000i32) != 0i32 {
        v2 = 256i16;
        let E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
        if ((*E1).mbn_var_34 as i32 & 0x8000i32) != 0i32 {
            v2 = -256i16;
        }
        (*E1).mbn_var_34 = v2 as u16;
        MotherBrain_SetBrainInstrs(addr_kMotherBrain_Ilist_9F34 as u16, game);
        (*E).mbn_var_A = ((0xa9b863i32) as u16);
        (*E).mbn_var_F = 16u16;
    }
}
pub unsafe fn MotherBomb_FiringLaser_FinishAttack(game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if ((*E).mbn_var_F as i32 & 0x8000i32) != 0i32 {
        let E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
        (*E1).mbn_var_32 = 4u16;
        (*E1).mbn_var_33 = 4u16;
        (*E).mbn_var_A = ((0xa9b605i32) as u16);
        MotherBrain_Body_Phase2_Thinking(game);
    }
}
static off_A9B887: [Option<Func_V>; 4] = callback_table! { unsafe Func_V;
    MotherBomb_FiringDeathBeam_0, MotherBomb_FiringDeathBeam_1, MotherBomb_FiringDeathBeam_2,
    MotherBomb_FiringDeathBeam_3,
};
pub unsafe fn MotherBomb_FiringDeathBeam(game: &mut Game) {
    let v0: i32 = (*Get_MotherBrain(0i32, game)).mbn_var_17 as i32;
    off_A9B887[v0 as usize].expect("non-null function pointer")(game);
}
pub unsafe fn MotherBomb_FiringDeathBeam_0(game: &mut Game) {
    if MotherBrain_MakeWalkBackwards(0x28u16, 8u16, game) as i32 & 1i32 != 0 {
        let E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
        (*E1).mbn_var_32 = 8u16;
        (*E1).mbn_var_33 = 6u16;
        let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
        (*E).mbn_var_17 = (*E).mbn_var_17.wrapping_add(1);
    }
}
pub unsafe fn MotherBomb_FiringDeathBeam_1(game: &mut Game) {
    if (*Get_MotherBrain(0x40i32, game)).mbn_var_05 == 0 {
        MotherBrain_SetBodyInstrs(addr_kMotherBrain_Ilist_9A42 as u16, game);
        let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
        (*E).mbn_var_17 = (*E).mbn_var_17.wrapping_add(1);
    }
}
pub fn MotherBomb_FiringDeathBeam_2(game: &mut Game) {}
pub unsafe fn MotherBomb_FiringDeathBeam_3(game: &mut Game) {
    MotherBrain_SetBrainInstrs(addr_kMotherBrain_Ilist_9C87 as u16, game);
    let E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
    (*E1).mbn_var_32 = 2u16;
    (*E1).mbn_var_33 = 4u16;
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).mbn_var_17 = 0u16;
    (*E).mbn_var_A = ((0xa9b605i32) as u16);
}
pub unsafe fn MotherBomb_FiringRainbowBeam_0(game: &mut Game) {
    MotherBrain_SetBrainInstrs(addr_kMotherBrain_Ilist_9C87 as u16, game);
    let E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
    (*E1).mbn_var_34 = 64u16;
    (*E1).mbn_var_31 = 1u16;
    (*E1).mbn_var_32 = 2u16;
    (*E1).mbn_var_33 = 4u16;
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).mbn_var_A = ((0xa9b91ai32) as u16);
    (*E).mbn_var_F = 256u16;
}
pub unsafe fn MotherBomb_FiringRainbowBeam_1_StartCharge(game: &mut Game) {
    let E: *mut Enemy_MotherBomb = Get_MotherBomb(0i32, game);
    (*E).mbb_var_F = (*E).mbb_var_F.wrapping_sub(1);
    if ((*E).mbb_var_F as i32 & 0x8000i32) != 0i32 {
        MotherBrain_SetBrainInstrs(addr_kMotherBrain_Ilist_9F6C as u16, game);
        (*E).mbb_var_A = ((0xa9b92bi32) as u16);
        MotherBomb_FiringRainbowBeam_2_RetractNeck(game);
    }
}
pub unsafe fn MotherBomb_FiringRainbowBeam_2_RetractNeck(game: &mut Game) {
    if MotherBrain_WalkBackwardsSlowlyAndRetractHead(0x28u16, game) as i32 & 1i32 != 0 {
        let E: *mut Enemy_MotherBomb = Get_MotherBomb(0i32, game);
        (*E).mbb_var_A = ((0xa9b93fi32) as u16);
        (*E).mbb_var_F = 256u16;
        MotherBomb_FiringRainbowBeam_3_Wait(game);
    }
}
pub unsafe fn MotherBomb_FiringRainbowBeam_3_Wait(game: &mut Game) {
    let E: *mut Enemy_MotherBomb = Get_MotherBomb(0i32, game);
    (*E).mbb_var_F = (*E).mbb_var_F.wrapping_sub(1);
    if ((*E).mbb_var_F as i32 & 0x8000i32) != 0i32 {
        QueueSfx2_Max6(0x71u16, game);
        (*E).mbb_var_A = ((0xa9b951i32) as u16);
        MotherBomb_FiringRainbowBeam_4_ExtendNeckDown(game);
    }
}
pub unsafe fn MotherBomb_FiringRainbowBeam_4_ExtendNeckDown(game: &mut Game) {
    set_ram_word!(game.cooldown_timer = 8u16);
    let E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
    (*E1).mbn_var_32 = 6u16;
    (*E1).mbn_var_33 = 6u16;
    (*E1).mbn_var_34 = 1280u16;
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).mbn_var_A = ((0xa9b975i32) as u16);
    (*E).mbn_var_F = 16u16;
    MotherBomb_FiringRainbowBeam_5_StartFiring(game);
}
pub unsafe fn MotherBomb_FiringRainbowBeam_5_StartFiring(game: &mut Game) {
    MotherBrain_AimBeamAndIncrWidth(game);
    if ram_word!(game, power_bomb_flag) == 0 {
        let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
        (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
        if ((*E).mbn_var_F as i32 & 0x8000i32) != 0i32 && ram_word!(game, power_bomb_flag) == 0 {
            set_ram_word!(game.cooldown_timer = 0u16);
            MotherBrain_BodyRainbowBeamPalAnimIndex0(game);
            MotherBrain_SetBrainInstrs(addr_kMotherBrain_Ilist_9C77 as u16, game);
            (*E).mbn_var_33 = 512u16;
            (*E).mbn_var_09 = SpawnMotherBrainRainbowBeamHdma(game);
            let E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
            (*E1).mbn_var_34 = 64u16;
            (*E1).mbn_var_31 = 1u16;
            (*E1).mbn_var_32 = 2u16;
            (*E1).mbn_var_33 = 4u16;
            (*E).mbn_parameter_1 = 0u16;
            (*E).mbn_parameter_2 = 0u16;
            let mut v4: u16 = 5u16;
            if ((ram_word!(game, samus_health)) as i16).wrapping_sub(700i16) < 0i16 {
                v4 = 24u16;
            }
            CallSomeSamusCode(v4, game);
            (*E).mbn_var_15 = 6u16;
            (*E).mbn_var_A = ((0xa9b9e5i32) as u16);
        }
    }
}
pub unsafe fn MotherBomb_FiringRainbowBeam_6_MoveSamusToWall(game: &mut Game) {
    MotherBrain_PlayRainbowBeamSfx(game);
    MotherBrain_HandleRainbowBeamPalette(game);
    MotherBrain_AimBeamAndIncrWidth(game);
    MotherBrain_HandleRainbowBeamExplosions(game);
    if MotherBrain_MoveSamusTowardsWallDueToBeam(game) as i32 & 1i32 != 0 {
        let E: *mut Enemy_MotherBomb = Get_MotherBomb(0i32, game);
        (*E).mbb_var_A = ((0xa9ba00i32) as u16);
        (*E).mbb_var_F = 0u16;
    }
}
pub unsafe fn MotherBomb_FiringRainbowBeam_7_DelayFrame(game: &mut Game) {
    MotherBrain_PlayRainbowBeamSfx(game);
    MotherBrain_HandleRainbowBeamPalette(game);
    MotherBrain_AimBeamAndIncrWidth(game);
    MotherBrain_HandleRainbowBeamExplosions(game);
    MotherBrain_MoveSamusTowardsWallDueToBeam(game);
    let E: *mut Enemy_MotherBomb = Get_MotherBomb(0i32, game);
    (*E).mbb_var_F = (*E).mbb_var_F.wrapping_sub(1);
    if ((*E).mbb_var_F as i32 & 0x8000i32) != 0i32 {
        set_ram_words! { game, 8u16; earthquake_type, earthquake_timer }
        (*E).mbb_var_A = ((0xa9ba27i32) as u16);
    }
}
pub unsafe fn MotherBomb_FiringRainbowBeam_8_StartDrainSamus(game: &mut Game) {
    let E: *mut Enemy_MotherBomb = Get_MotherBomb(0i32, game);
    (*E).mbb_var_A = ((0xa9ba3ci32) as u16);
    (*E).mbb_var_F = 299u16;
    set_ram_word!(game.earthquake_timer = 299u16);
    set_ram_word!(game.earthquake_type = 8u16);
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
    let E: *mut Enemy_MotherBomb = Get_MotherBomb(0i32, game);
    (*E).mbb_var_F = (*E).mbb_var_F.wrapping_sub(1);
    if ((*E).mbb_var_F as i32 & 0x8000i32) != 0i32 {
        (*E).mbb_var_A = ((0xa9ba5ei32) as u16);
    }
}
pub unsafe fn MotherBomb_FiringRainbowBeam_10_FinishFiringRainbow(game: &mut Game) {
    MotherBrain_PlayRainbowBeamSfx(game);
    MotherBrain_HandleRainbowBeamPalette(game);
    MotherBrain_AimBeam(game);
    MotherBrain_HandleRainbowBeamExplosions(game);
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    let v1: u16 = ((*E).mbn_var_33).wrapping_sub(384u16);
    (*E).mbn_var_33 = v1;
    if v1 as i32 - 512i32 & 0x8000i32 != 0 {
        (*E).mbn_var_33 = 512u16;
        (*E).mbn_parameter_1 = -256i32 as u16;
        (*E).mbn_parameter_2 = 0u16;
        set_ram_words! { game, 0u16;
            hdma_object_channels_bitmask[((*E).mbn_var_09 as i32 >> 1i32)], earthquake_timer,
        }
        MotherBrain_SetBrainInstrs(addr_kMotherBrain_Ilist_9C87 as u16, game);
        MotherBrain_SetupBrainNormalPal(game);
        MotherBrain_WriteDefaultPalette(game);
        QueueSfx1_Max6(2u16, game);
        (*E).mbn_var_16 = 0u16;
        CallSomeSamusCode(1u16, game);
        set_ram_word!(game.cooldown_timer = 8u16);
        (*E).mbn_var_A = ((0xa9bac4i32) as u16);
    }
}
pub unsafe fn MotherBomb_FiringRainbowBeam_11_LetSamusFall(game: &mut Game) {
    SomeMotherBrainScripts(0u16, game);
    (*Get_MotherBomb(0i32, game)).mbb_var_A = ((0xa9bad1i32) as u16);
    MotherBomb_FiringRainbowBeam_12_WaitForSamusHitGround(game);
}
pub unsafe fn MotherBomb_FiringRainbowBeam_12_WaitForSamusHitGround(game: &mut Game) {
    if MotherBrain_MoveSamusForFallingAfterBeam(game) as i32 & 1i32 != 0 {
        (*Get_MotherBomb(0i32, game)).mbb_var_A = ((0xa9baddi32) as u16);
    }
}
pub unsafe fn MotherBomb_FiringRainbowBeam_13_LowerHead(game: &mut Game) {
    let E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
    (*E1).mbn_var_34 = 64u16;
    (*E1).mbn_var_31 = 1u16;
    (*E1).mbn_var_32 = 2u16;
    (*E1).mbn_var_33 = 4u16;
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).mbn_var_A = ((0xa9bb06i32) as u16);
    (*E).mbn_var_F = 128u16;
}
pub unsafe fn MotherBomb_FiringRainbowBeam_14_DecideNextAction(game: &mut Game) {
    let E: *mut Enemy_MotherBomb = Get_MotherBomb(0i32, game);
    (*E).mbb_var_F = (*E).mbb_var_F.wrapping_sub(1);
    if ((*E).mbb_var_F as i32 & 0x8000i32) != 0i32 {
        if ram_word!(game, samus_health) as i32 - 400i32 & 0x8000i32 != 0 {
            MotherBrain_MakeWalkForwards(0xau16, ((*E).base.x_pos).wrapping_add(16u16), game);
            (*E).mbb_var_A = ((0xa9bd45i32) as u16);
        } else {
            (*E).mbb_var_A = ((0xa9b8ebi32) as u16);
        }
    }
}
pub unsafe fn MotherBrain_PlayRainbowBeamSfx(game: &mut Game) {
    let mut mbn_var_15: i16 = 0;
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    mbn_var_15 = (*E).mbn_var_15 as i16;
    if mbn_var_15 >= 0i16 {
        (*E).mbn_var_15 = ((mbn_var_15) as u16).wrapping_sub(1u16);
        QueueSfx1_Max6(0x40u16, game);
        (*E).mbn_var_16 = 1u16;
    }
}
pub unsafe fn MotherBrain_WalkBackwardsSlowlyAndRetractHead(a: u16, game: &mut Game) -> u8 {
    let rv: u8 = MotherBrain_MakeWalkBackwards(a, 0xau16, game);
    if rv != 0 {
        MotherBrain_RetractHead(game);
    }
    rv
}
pub unsafe fn MotherBrain_RetractHead(game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
    (*E).mbn_var_34 = 80u16;
    (*E).mbn_var_31 = 1u16;
    (*E).mbn_var_32 = 8u16;
    (*E).mbn_var_33 = 6u16;
}
pub unsafe fn MotherBrain_AimBeamAndIncrWidth(game: &mut Game) {
    let mut v1: i16 = 0;
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    v1 = (((*E).mbn_var_33) as i16).wrapping_add(384i16);
    if (*E).mbn_var_33 as i32 - 2688i32 & 0x8000i32 == 0 {
        v1 = 3072i16;
    }
    (*E).mbn_var_33 = v1 as u16;
    MotherBrain_AimBeam(game);
}
pub unsafe fn MotherBrain_AimBeam(game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
    (*Get_MotherBrain(0i32, game)).mbn_var_31 = -(CalculateAngleFromXY(
        (ram_word!(game, samus_x_pos) as i32 - (*E).base.x_pos as i32 - 16i32) as u16,
        (ram_word!(game, samus_y_pos) as i32 - (*E).base.y_pos as i32 - 4i32) as u16,
        game,
    ) as i32
        + 0x80i32) as u8 as u16;
}
pub unsafe fn MotherBrain_CalculateRainbowBeamHdma_(game: &mut Game) {
    MotherBrain_CalculateRainbowBeamHdma(game);
}
pub unsafe fn MotherBrain_MoveSamusTowardsWallDueToBeam(game: &mut Game) -> u8 {
    let v0: u8 = MotherBrain_MoveSamusHorizTowardsWall(0x1000u16, game);
    if v0 == 0 {
        let v1: u16 = Math_MultByCos(0x1000u16, (*Get_MotherBrain(0i32, game)).mbn_var_31);
        MotherBrain_MoveSamusVerticallyTowardsCeilingFloor(v1, game);
    }
    v0
}
pub unsafe fn MotherBrain_MoveSamusTowardsMiddleOfWall(game: &mut Game) {
    let mut v0: u16 = 64u16;
    if (124i16).wrapping_sub((ram_word!(game, samus_y_pos)) as i16) < 0i16 {
        v0 = -64i32 as u16;
    }
    MotherBrain_MoveSamusVerticallyTowardsCeilingFloor(v0, game);
}
pub unsafe fn MotherBrain_MoveSamusForFallingAfterBeam(game: &mut Game) -> u8 {
    let mut v1: i16 = 0;
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    v1 = (((*E).mbn_parameter_1) as i16).wrapping_add(2i16);
    if v1 >= 0i16 {
        v1 = 0i16;
    }
    (*E).mbn_parameter_1 = v1 as u16;
    MotherBrain_MoveSamusHorizTowardsWall(v1 as u16, game);
    let v2: u16 = ((*E).mbn_parameter_2).wrapping_add(24u16);
    (*E).mbn_parameter_2 = v2;
    (MotherBrain_MoveSamusVerticallyTowardsCeilingFloor(v2, game)) & (1u8)
}
pub unsafe fn MotherBrain_MoveSamusVerticallyTowardsCeilingFloor(
    mut a: u16,
    game: &mut Game,
) -> u8 {
    let carry: i32 = *(ram_ptr!(game, u16; 0xafc) as *mut u8).offset(1isize) as i32
        + *(&raw mut a as *mut u8).offset(0) as i32;
    *(ram_ptr!(game, u16; 0xafc) as *mut u8).offset(1isize) = carry as u8;
    *(ram_ptr!(game, u16; 0xb16) as *mut u8).offset(1isize) = carry as u8;
    let v4: u16 = (ram_word!(game, samus_y_pos) as i32
        + (a as i32 >> 8i32) as i8 as i32
        + (carry >> 8i32)) as u16;
    let mut v6: u16 = 0;
    if v4 as i32 - 48i32 & 0x8000i32 != 0 {
        v6 = 48u16;
    } else {
        if v4 as i32 - 192i32 & 0x8000i32 != 0 {
            let ref mut fresh2 = *(ram_ptr!(game, u16; 0xafa));
            *fresh2 = v4;
            set_ram_word!(game.samus_prev_y_pos = *fresh2);
            return 0u8;
        }
        v6 = 192u16;
    }
    let ref mut fresh3 = *(ram_ptr!(game, u16; 0xafa));
    *fresh3 = v6;
    set_ram_word!(game.samus_prev_y_pos = *fresh3);
    set_ram_words! { game, 0u16; samus_y_subpos, samus_prev_y_subpos }
    1u8
}
pub unsafe fn MotherBrain_MoveSamusHorizTowardsWall(mut a: u16, game: &mut Game) -> u8 {
    let carry: i32 = *(ram_ptr!(game, u16; 0xaf8) as *mut u8).offset(1isize) as i32
        + *(&raw mut a as *mut u8).offset(0) as i32;
    *(ram_ptr!(game, u16; 0xaf8) as *mut u8).offset(1isize) = carry as u8;
    *(ram_ptr!(game, u16; 0xb12) as *mut u8).offset(1isize) = carry as u8;
    let v4: u16 = (ram_word!(game, samus_x_pos) as i32
        + (a as i32 >> 8i32) as i8 as i32
        + (carry >> 8i32)) as u16;
    if v4 as i32 - 235i32 & 0x8000i32 != 0 {
        set_ram_word!(game.samus_x_pos = v4);
        set_ram_word!(game.samus_prev_x_pos = v4);
        0u8
    } else {
        set_ram_words! { game, 235u16; samus_x_pos, samus_prev_x_pos }
        set_ram_words! { game, 0u16; samus_x_subpos, samus_prev_x_subpos }
        1u8
    }
}
pub unsafe fn MotherBrain_HandleRainbowBeamExplosions(game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).mbn_parameter_2 = (*E).mbn_parameter_2.wrapping_sub(1);
    if ((*E).mbn_parameter_2 as i32 & 0x8000i32) != 0i32 {
        (*E).mbn_parameter_2 = 8u16;
        (*E).mbn_parameter_1 = (*E).mbn_parameter_1.wrapping_add(1);
        let v2: i32 = (*E).mbn_parameter_1 as i32 & 7i32;
        game.eproj_spawn_pt = Point16U {
            x: g_word_A9BCA6[v2 as usize] as u16,
            y: g_word_A9BCB6[v2 as usize] as u16,
        };
        SpawnEprojWithRoomGfx(
            addr_kEproj_MotherBrainRainbowBeamExplosion as u16,
            0u16,
            game,
        );
        QueueSfx2_Max6(0x24u16, game);
    }
}
pub unsafe fn MotherBrain_BodyRainbowBeamPalAnimIndex0(game: &mut Game) {
    (*Get_MotherBrain(0x40i32, game)).mbn_var_01 = 0u16;
}
pub unsafe fn MotherBrain_WriteDefaultPalette(game: &mut Game) {
    WriteColorsToPalette(
        0x82u16,
        0xa9u8,
        (addr_kMotherBrainsBrain_Palette as i32 + 2i32) as u16,
        0xfu16,
        game,
    );
    WriteColorsToPalette(
        0x122u16,
        0xa9u8,
        (addr_kMotherBrainsBrain_Palette as i32 + 2i32) as u16,
        0xfu16,
        game,
    );
    WriteColorsToPalette(
        0x162u16,
        0xa9u8,
        (addr_kMotherBrainPalette_0 as i32 + 2i32) as u16,
        0xfu16,
        game,
    );
}
pub unsafe fn MotherBrain_WritePhase2DeathPalette(game: &mut Game) {
    MotherBrain_WritePalette(crate::rtl::rom_table_word!(game, 0xade434, 6), game);
}
pub unsafe fn MotherBrain_HandleRainbowBeamPalette(game: &mut Game) {
    if ((*Get_MotherBrain(0i32, game)).base.frame_counter as i32 & 2i32) != 0i32 {
        let mut mbn_var_01: u16 = (*Get_MotherBrain(0x40i32, game)).mbn_var_01;
        let mut v1: u16 = 0;
        loop {
            v1 = mbn_var_01;
            mbn_var_01 =
                crate::rtl::rom_table_word!(game, 0xade434, (mbn_var_01 as i32 >> 1i32) as isize);
            if (mbn_var_01 != 0) {
                break;
            }
        }
        let v2: u16 = (v1).wrapping_add(2u16);
        (*Get_MotherBrain(0x40i32, game)).mbn_var_01 = v2;
        MotherBrain_WritePalette(
            *((game.g_rom.offset(crate::rtl::lorom_offset(0xade434)) as *mut u16 as *mut u8)
                .offset(v2 as i32 as isize)
                .offset(-(2i32 as isize)) as *mut u16),
            game,
        );
    }
}
pub unsafe fn MotherBrain_WritePalette(j: u16, game: &mut Game) {
    WriteColorsToPalette(0x82u16, 0xadu8, j, 15u16, game);
    WriteColorsToPalette(0x122u16, 0xadu8, j, 15u16, game);
    WriteColorsToPalette(0x162u16, 0xadu8, (j).wrapping_add(30u16), 15u16, game);
}
pub unsafe fn MotherBrain_Phase2Cut_0(game: &mut Game) {
    let v0: u16 = SuitDamageDivision(0x50u16, game);
    if ((4i32 * v0 as i32 + 20i32 - ram_word!(game, samus_health) as i32) as i16) >= 0i16 {
        (*Get_MotherBrain(0i32, game)).mbn_var_A = ((0xa9bd98i32) as u16);
    } else if (ram_word!(game, random_number) as i32 & 0xfffi32) < 0xfa0i32 {
        if ((*Get_MotherBrain(0i32, game)).base.frame_counter as i32 & 0x1fi32) == 0i32 {
            MotherBrain_MaybeStandupOrLeanDown(game);
        }
    } else {
        let mut v1: u16 = 0;
        if ((SuitDamageDivision(0xa0u16, game) as i32 + 20i32
            - ram_word!(game, samus_health) as i32) as i16)
            >= 0i16
            || {
                v1 = addr_kMotherBrain_Ilist_9ECC as u16;
                (ram_word!(game, random_number) as i32 & 0xfffi32) < 0xff0i32
            }
        {
            v1 = addr_kMotherBrain_Ilist_9D7F as u16;
        }
        MotherBrain_SetBrainInstrs(v1, game);
    };
}
pub unsafe fn MotherBrain_Phase2Cut_1(game: &mut Game) {
    if MotherBrain_MakeHerStandUp(game) as i32 & 1i32 != 0 {
        let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
        (*E).mbn_var_A = ((0xa9bda9i32) as u16);
        (*E).mbn_var_F = 16u16;
        MotherBrain_Phase2Cut_2(game);
    }
}
pub unsafe fn MotherBrain_Phase2Cut_2(game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if ((*E).mbn_var_F as i32 & 0x8000i32) != 0i32 {
        MotherBrain_SetBrainInstrs(addr_kMotherBrain_Ilist_9B7F as u16, game);
        (*E).mbn_var_A = ((0xa9bdc1i32) as u16);
        (*E).mbn_var_F = 256u16;
    }
}
pub unsafe fn MotherBrain_Phase2Cut_3(game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if ((*E).mbn_var_F as i32 & 0x8000i32) != 0i32 {
        MotherBrain_SetBrainInstrs(addr_kMotherBrain_Ilist_9F6C as u16, game);
        (*E).mbn_var_A = ((0xa9bdd2i32) as u16);
        MotherBrain_Phase2Cut_4(game);
    }
}
pub unsafe fn MotherBrain_Phase2Cut_4(game: &mut Game) {
    if ProcessSpriteTilesTransfers(0xa9u8, addr_stru_A98FE5 as u16, game) != 0 {
        MotherBrain_RetractHead(game);
        MotherBrain_SpawnShitroidInCutscene(game);
        let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
        (*E).mbn_var_A = ((0xa9bdedi32) as u16);
        (*E).mbn_var_F = 256u16;
    }
}
pub unsafe fn MotherBrain_Phase2Cut_5(game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if ((*E).mbn_var_F as i32 & 0x8000i32) != 0i32 {
        MotherBrain_BodyRainbowBeamPalAnimIndex0(game);
        MotherBrain_SetBrainInstrs(addr_kMotherBrain_Ilist_9C77 as u16, game);
        let E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
        (*E1).mbn_var_32 = 6u16;
        (*E1).mbn_var_33 = 6u16;
        (*E1).mbn_var_34 = 1280u16;
        QueueSfx2_Max6(0x71u16, game);
        (*E).mbn_var_A = ((0xa9be1ai32) as u16);
    }
}
pub unsafe fn MotherBrain_SpawnShitroidInCutscene(game: &mut Game) {
    (*Get_MotherBrain(0x40i32, game)).mbn_var_0A =
        SpawnEnemy(0xa9u8, addr_stru_A9BE28 as u16, game);
}
pub unsafe fn MotherBrain_DrainedByShitroid_0(game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).mbn_var_00 = 3u16;
    let E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
    (*E1).mbn_var_32 = 8u16;
    (*E1).mbn_var_33 = 8u16;
    (*E1).mbn_var_34 = 1792u16;
    (*E).mbn_var_A = ((0xa9be5di32) as u16);
    (*E).mbn_var_F = 48u16;
    MotherBrain_DrainedByShitroid_1(game);
}
pub unsafe fn MotherBrain_DrainedByShitroid_1(game: &mut Game) {
    MotherBrain_HandleRainbowBeamPalette(game);
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if ((*E).mbn_var_F as i32 & 0x8000i32) != 0i32 {
        (*E).mbn_var_A = ((0xa9be96i32) as u16);
        let E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
        (*E1).mbn_var_08 = ((0xa9bfd0i32) as u16);
        (*E1).mbn_var_06 = 0u16;
        (*E1).mbn_var_07 = 2u16;
        (*E1).mbn_var_32 = 2u16;
        (*E1).mbn_var_33 = 4u16;
    }
}
crate::game::address_dispatch! {
    pub unsafe fn CallMotherBrainPainfulWalk(ea: u32, game: &mut Game) => ea, game {
        11124688 => MotherBrain_PainfulWalkForwards(game),
        11124712 => MotherBrain_PainfulWalkingForwards(game),
        11124740 => MotherBrain_PainfulWalkBackwards(game),
        11124764 => MotherBrain_PainfulWalkingBackwards(game),
    }
}
pub unsafe fn MotherBrain_DrainedByShitroid_2(game: &mut Game) {
    let E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
    if (*E1).mbn_var_00 == 0 {
        (*E1).mbn_var_00 = 50u16;
    }
    MotherBrain_HandleRainbowBeamPalette(game);
    CallMotherBrainPainfulWalk((((*E1).mbn_var_08) as u32) | (0xa90000u32), game);
    let v2: u16 = (2u16).wrapping_mul((*E1).mbn_var_06);
    (*E1).mbn_var_07 = g_byte_A9BEEE[v2 as usize] as u16;
    (*E1).mbn_var_34 = g_word_A9BEFE[(v2 as i32 >> 1i32) as usize];
    if (*E1).mbn_var_06 == 6u16 {
        let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
        (*E).mbn_var_16 = 0u16;
        (*E1).mbn_var_10 = 0u16;
        MotherBrain_WritePhase2DeathPalette(game);
        QueueSfx1_Max6(2u16, game);
        (*E).mbn_var_A = ((0xa9bf0ei32) as u16);
    }
}
pub unsafe fn MotherBrain_DrainedByShitroid_3(game: &mut Game) {
    let E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
    CallMotherBrainPainfulWalk((((*E1).mbn_var_08) as u32) | (0xa90000u32), game);
    if ((*E1).mbn_var_06 as i32 - 8i32 & 0x8000i32 == 0) {
        (*E1).mbn_var_34 = 64u16;
        (*E1).mbn_var_32 = 8u16;
        (*E1).mbn_var_33 = 8u16;
        MotherBrain_SetBrainInstrs(addr_kMotherBrain_Ilist_9C39 as u16, game);
        (*Get_MotherBrain(0i32, game)).mbn_var_A = ((0xa9bf41i32) as u16);
        MotherBrain_DrainedByShitroid_4(game);
    }
}
pub unsafe fn MotherBrain_DrainedByShitroid_4(game: &mut Game) {
    if MotherBrain_MakeWalkBackwards(0x28u16, 0u16, game) as i32 & 1i32 != 0 {
        (*Get_MotherBrain(0i32, game)).mbn_var_A = ((0xa9bf56i32) as u16);
        (*Get_MotherBrain(0x40i32, game)).mbn_var_33 = 0u16;
        MotherBrain_DrainedByShitroid_5(game);
    }
}
pub unsafe fn MotherBrain_DrainedByShitroid_5(game: &mut Game) {
    let E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
    if *(&raw mut (*E1).mbn_var_32 as *mut u32) == 0 {
        (*E1).mbn_var_12 = ((*E1).mbn_var_33) | ((*E1).mbn_var_32);
        let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
        if (*E).mbn_var_02 == 0 {
            MotherBrain_SetBodyInstrs(addr_kMotherBrain_Ilist_9A26 as u16, game);
            (*E).mbn_var_A = ((0xa9bf7di32) as u16);
            (*E).mbn_var_F = 64u16;
        }
    }
}
pub unsafe fn MotherBrain_DrainedByShitroid_6(game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if ((*E).mbn_var_F as i32 & 0x8000i32) != 0i32 {
        (*E).mbn_var_37 = 0u16;
        (*E).mbn_var_A = ((0xa9bf95i32) as u16);
        (*E).mbn_var_F = 16u16;
        MotherBrain_DrainedByShitroid_7(game);
    }
}
pub unsafe fn MotherBrain_DrainedByShitroid_7(game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    let v2: bool = ((*E).mbn_var_F as i32 & 0x8000i32) != 0i32;
    let mut v3: u16 = 0;
    if v2 && {
        (*E).mbn_var_F = 16u16;
        v3 = ((*E).mbn_var_37).wrapping_add(1u16);
        (*E).mbn_var_37 = v3;
        MotherBrain_FadeToGray_Drained((v3).wrapping_sub(1u16), game) as i32 & 1i32 != 0
    } {
        let E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
        (*E1).base.health = -29536i32 as u16;
        (*E).mbn_var_1F = 1u16;
        (*E1).mbn_var_14 = 0u16;
        (*E).mbn_var_00 = 2u16;
        (*E).mbn_var_A = ((0xa9c059i32) as u16);
    }
}
pub unsafe fn MotherBrain_PainfulWalkForwards(game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
    if MotherBrain_MakeWalkForwards((*E).mbn_var_07, 0x48u16, game) as i32 & 1i32 != 0 {
        (*E).mbn_var_08 = ((0xa9bfe8i32) as u16);
        MotherBrain_SetPainfulWalkingTimer(game);
    }
}
pub unsafe fn MotherBrain_PainfulWalkingForwards(game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
    let v1: u16 = ((*E).mbn_var_09).wrapping_sub(1u16);
    (*E).mbn_var_09 = v1;
    if v1 == 0 {
        (*E).mbn_var_06 = (*E).mbn_var_06.wrapping_add(1);
        (*E).mbn_var_08 = ((0xa9c004i32) as u16);
    }
}
pub unsafe fn MotherBrain_PainfulWalkBackwards(game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
    if MotherBrain_MakeWalkBackwards(0x28u16, (*E).mbn_var_07, game) as i32 & 1i32 != 0 {
        (*E).mbn_var_08 = ((0xa9c01ci32) as u16);
        MotherBrain_SetPainfulWalkingTimer(game);
    }
}
pub unsafe fn MotherBrain_PainfulWalkingBackwards(game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
    let v1: u16 = ((*E).mbn_var_09).wrapping_sub(1u16);
    (*E).mbn_var_09 = v1;
    if v1 == 0 {
        (*E).mbn_var_06 = (*E).mbn_var_06.wrapping_add(1);
        (*E).mbn_var_08 = ((0xa9bfd0i32) as u16);
    }
}
pub unsafe fn MotherBrain_SetPainfulWalkingTimer(game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
    (*E).mbn_var_09 = *((&raw const g_word_A9C049 as *const i16).offset((*E).mbn_var_06 as isize)
        as *mut u8)
        .offset(0) as u16;
}
pub unsafe fn MotherBrain_Phase2_Revive_0(game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).mbn_var_A = ((0xa9c066i32) as u16);
    (*E).mbn_var_F = 768u16;
}
pub unsafe fn MotherBrain_Phase2_Revive_1(game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if ((*E).mbn_var_F as i32 & 0x8000i32) != 0i32 {
        let E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
        (*E1).mbn_var_14 = 1u16;
        (*E1).mbn_var_12 = 1u16;
        (*E).mbn_var_A = ((0xa9c082i32) as u16);
        (*E).mbn_var_F = 224u16;
        MotherBrain_Phase2_Revive_2(game);
    }
}
pub unsafe fn MotherBrain_Phase2_Revive_2(game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).mbn_var_A = ((0xa9c08fi32) as u16);
    (*E).mbn_var_37 = 0u16;
    MotherBrain_Phase2_Revive_3(game);
}
pub unsafe fn MotherBrain_Phase2_Revive_3(game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    let v1: bool = ((*E).mbn_var_F as i32 & 0x8000i32) != 0i32;
    let mut v2: u16 = 0;
    if v1 && {
        (*E).mbn_var_F = 16u16;
        v2 = ((*E).mbn_var_37).wrapping_add(1u16);
        (*E).mbn_var_37 = v2;
        MotherBrain_FadeFromGray_Drained((v2).wrapping_sub(1u16), game) as i32 & 1i32 != 0
    } {
        (*E).mbn_var_A = ((0xa9c0bai32) as u16);
        (*Get_MotherBrain(0x40i32, game)).mbn_var_10 = 1u16;
        MotherBrain_SetupBrainNormalPal(game);
        MotherBrain_Phase2_Revive_4(game);
    }
}
pub unsafe fn MotherBrain_Phase2_Revive_4(game: &mut Game) {
    if MotherBrain_MakeHerStandUp(game) as i32 & 1i32 != 0 {
        let E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
        (*E1).mbn_var_32 = 6u16;
        (*E1).mbn_var_33 = 6u16;
        (*E1).mbn_var_34 = 1280u16;
        (*E1).mbn_var_31 = 1u16;
        let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
        (*E).mbn_var_A = ((0xa9c0e4i32) as u16);
        (*E).mbn_var_F = 16u16;
        MotherBrain_Phase2_Revive_5(game);
    }
}
pub unsafe fn MotherBrain_Phase2_Revive_5(game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if ((*E).mbn_var_F as i32 & 0x8000i32) != 0i32 {
        MotherBrain_SetBrainInstrs(addr_kMotherBrain_Ilist_9BB3 as u16, game);
        (*E).mbn_var_A = ((0xa9c0fbi32) as u16);
        (*E).mbn_var_F = 128u16;
        MotherBrain_Phase2_Revive_6(game);
    }
}
pub unsafe fn MotherBrain_Phase2_Revive_6(game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    let v1: bool = ((*E).mbn_var_F as i32 & 0x8000i32) != 0i32;
    if v1 && MotherBrain_MakeWalkForwards(4u16, 0x50u16, game) as i32 & 1i32 != 0 {
        (*E).mbn_var_A = ((0xa9c11ei32) as u16);
        (*E).mbn_var_1F = 2u16;
        (*Get_MotherBrain(0x40i32, game)).mbn_var_11 = 1u16;
    }
}
pub unsafe fn MotherBrain_Phase2_Revive_7(game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).mbn_var_13 = 0u16;
    let E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
    (*E1).mbn_var_31 = 1u16;
    (*E1).mbn_var_32 = 2u16;
    (*E1).mbn_var_33 = 4u16;
    (*E1).mbn_var_34 = 64u16;
    (*E).mbn_var_A = ((0xa9c147i32) as u16);
    MotherBrain_Phase2_Revive_8(game);
}
pub unsafe fn MotherBrain_Phase2_Revive_8(game: &mut Game) {
    if MotherBrain_MakeHerStandUp(game) as i32 & 1i32 != 0 {
        (*Get_MotherBrain(0i32, game)).mbn_var_A = ((0xa9c15ci32) as u16);
        MotherBrain_MakeWalkForwards(0xau16, 0x50u16, game);
        MotherBrain_Phase2_MurderShitroid_1(game);
    }
}
pub unsafe fn MotherBrain_Phase2_MurderShitroid_1(game: &mut Game) {
    MotherBrain_MaybeStandupOrLeanDown(game);
    if (ram_word!(game, random_number) as i32 & 0x8000i32) != 0i32 {
        let mut v0: u16 = addr_kMotherBrain_Ilist_9DBB as u16;
        if (*Get_MotherBrain(0x40i32, game)).mbn_var_0A != 0 {
            v0 = addr_kMotherBrain_Ilist_9DB1 as u16;
        }
        MotherBrain_SetBrainInstrs(v0, game);
        let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
        (*E).mbn_var_A = ((0xa9c182i32) as u16);
        (*E).mbn_var_F = 64u16;
    }
}
pub unsafe fn MotherBrain_Phase2_MurderShitroid_2(game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if ((*E).mbn_var_F as i32 & 0x8000i32) != 0i32 {
        (*E).mbn_var_A = ((0xa9c15ci32) as u16);
    }
}
pub unsafe fn MotherBrain_Phase2_PrepareForFinalShitroid(game: &mut Game) {
    MotherBrain_MakeHerStandUp(game);
    MotherBrain_MakeWalkBackwards(0x40u16, 4u16, game);
}
pub unsafe fn MotherBrain_Phase2_ExecuteFinalkShitroid(game: &mut Game) {
    MotherBrain_SetBrainInstrs(addr_kMotherBrain_Ilist_9DB1 as u16, game);
    (*Get_MotherBrain(0i32, game)).mbn_var_A = ((0xa9c1a6i32) as u16);
}
pub unsafe fn MotherBrain_MaybeStandupOrLeanDown(game: &mut Game) {
    let mbn_var_02: u16 = (*Get_MotherBrain(0i32, game)).mbn_var_02;
    if mbn_var_02 != 0 {
        if mbn_var_02 == 6u16 && (ram_word!(game, random_number) as u8) >= 0xc0u8 {
            MotherBrain_MakeHerStandUp(game);
        }
    } else if (ram_word!(game, random_number) as u8) >= 0xc0u8 {
        MotherBrain_MakeHerLeanDown(game);
    }
}
pub unsafe fn MotherBrain_Phase3_Recover_MakeDistance(game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).mbn_var_00 = 4u16;
    (*E).mbn_var_A = ((0xa9c1f0i32) as u16);
    (*E).mbn_var_F = 32u16;
    MotherBrain_MakeWalkBackwards(((*E).base.x_pos).wrapping_sub(14u16), 2u16, game);
}
pub unsafe fn MotherBrain_Phase3_Recover_SetupForFight(game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if ((*E).mbn_var_F as i32 & 0x8000i32) != 0i32 {
        (*E).mbn_var_A = ((0xa9c209i32) as u16);
        let E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
        (*E1).mbn_var_18 = ((0xa9c330i32) as u16);
        (*E1).mbn_var_1A = ((0xa9c26ai32) as u16);
        MotherBrain_Phase3_Fighting_Main(game);
    }
}
pub unsafe fn MotherBrain_Phase3_Fighting_Main(game: &mut Game) {
    if (*Get_MotherBrain(0x40i32, game)).base.health != 0 {
        MotherBrain_Phase3_NeckHandler(game);
        MotherBrain_Phase3_WalkHandler(game);
        let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
        if (*E).mbn_var_02 == 0
            && (*E).mbn_var_06 == 0
            && (ram_word!(game, random_number) as i32 & 0x8000i32) != 0i32
        {
            let mut v1: u16 = addr_kMotherBrain_Ilist_9F00 as u16;
            if (ram_word!(game, random_number) as u8) >= 0x80u8 {
                v1 = addr_kMotherBrain_Ilist_9DBB as u16;
            }
            MotherBrain_SetBrainInstrs(v1, game);
            (*E).mbn_var_A = ((0xa9c24ei32) as u16);
            (*E).mbn_var_F = 64u16;
        }
    } else {
        (*Get_MotherBrain(0i32, game)).mbn_var_A = ((0xa9aee1i32) as u16);
    };
}
pub unsafe fn MotherBrain_Phase3_Fighting_Cooldown(game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).mbn_var_F = (*E).mbn_var_F.wrapping_sub(1);
    if ((*E).mbn_var_F as i32 & 0x8000i32) != 0i32 {
        (*E).mbn_var_A = ((0xa9c209i32) as u16);
    }
}
crate::game::address_dispatch! {
    pub unsafe fn CallMotherBrainWallkFunc1A(ea: u32, game: &mut Game) => ea, game {
        11125354 => MotherBrain_Phase3_Walk_TryToInchForward(game),
        11125427 => MotherBrain_Phase3_Walk_RetreatQuickly(game),
        11125458 => MotherBrain_Phase3_Walk_RetreatSlowly(game),
    }
}
pub unsafe fn MotherBrain_Phase3_WalkHandler(game: &mut Game) {
    if (*Get_MotherBrain(0i32, game)).mbn_var_02 == 0 {
        CallMotherBrainWallkFunc1A(
            (((*Get_MotherBrain(0x40i32, game)).mbn_var_1A) as u32) | (0xa90000u32),
            game,
        );
    }
}
pub unsafe fn MotherBrain_Phase3_Walk_TryToInchForward(game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    let mbn_var_07: u16 = (*E).mbn_var_07;
    if mbn_var_07 != 0 {
        let v3: u16 = (mbn_var_07).wrapping_add(32u16);
        (*E).mbn_var_07 = v3;
        if v3 >= 0x100u16 {
            let v4: u16 = ((*E).base.x_pos).wrapping_add(1u16);
            let E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
            (*E1).mbn_var_1B = v4;
            if MotherBrain_MakeWalkForwards(
                ((ram_word!(game, random_number) as i32 & 2i32) + 4i32) as u16,
                (*E1).mbn_var_1B,
                game,
            ) as i32
                & 1i32
                != 0
            {
                (*E).mbn_var_07 = 128u16;
            }
        }
    } else {
        let v6: u16 = ((*E).base.x_pos).wrapping_sub(14u16);
        let E1_0: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
        (*E1_0).mbn_var_1B = v6;
        (*E1_0).mbn_var_1A = ((0xa9c2b3i32) as u16);
        MotherBrain_Phase3_Walk_RetreatQuickly(game);
    };
}
pub unsafe fn MotherBrain_Phase3_Walk_RetreatQuickly(game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
    if MotherBrain_MakeWalkBackwards((*E).mbn_var_1B, 2u16, game) as i32 & 1i32 != 0 {
        (*E).mbn_var_1B = ((*Get_MotherBrain(0i32, game)).base.x_pos).wrapping_sub(14u16);
        (*E).mbn_var_1A = ((0xa9c2d2i32) as u16);
    }
}
pub unsafe fn MotherBrain_Phase3_Walk_RetreatSlowly(game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
    if MotherBrain_MakeWalkBackwards((*E).mbn_var_1B, 4u16, game) as i32 & 1i32 != 0 {
        MotherBrain_SetToTryToInchForward(0x40u16, game);
    }
}
pub unsafe fn MotherBrain_SetToTryToInchForward(a: u16, game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).mbn_var_07 = a;
    let E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
    (*E1).mbn_var_1A = ((0xa9c26ai32) as u16);
    (*E1).mbn_var_1B = ((*E).base.x_pos).wrapping_add(1u16);
}
crate::game::address_dispatch! {
    pub unsafe fn CallMotherBrainNeckFunc(ea: u32, game: &mut Game) => ea, game {
        11125587 => (),
        11125552 => MotherBrain_Phase3_Neck_Normal(game),
        11125588 => MotherBrain_Phase3_Neck_SetupRecoilRecovery(game),
        11125627 => MotherBrain_Phase3_Neck_RecoilRecovery(game),
        11125653 => MotherBrain_Phase3_Neck_SetupHyperBeamRecoil(game),
        11125709 => MotherBrain_Phase3_Neck_HyperBeamRecoil(game),
    }
}
pub unsafe fn MotherBrain_Phase3_NeckHandler(game: &mut Game) {
    CallMotherBrainNeckFunc(
        (((*Get_MotherBrain(0x40i32, game)).mbn_var_18) as u32) | (0xa90000u32),
        game,
    );
}
pub unsafe fn MotherBrain_Phase3_Neck_Normal(game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
    (*E).mbn_var_32 = 1u16;
    (*E).mbn_var_34 = 128u16;
    (*E).mbn_var_32 = 2u16;
    (*E).mbn_var_33 = 4u16;
    (*E).mbn_var_18 = addr_locret_A9C353 as u16;
}
pub unsafe fn MotherBrain_Phase3_Neck_SetupRecoilRecovery(game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
    (*E).mbn_var_31 = 1u16;
    (*E).mbn_var_34 = 1280u16;
    (*E).mbn_var_32 = 6u16;
    (*E).mbn_var_33 = 6u16;
    (*E).mbn_var_18 = ((0xa9c37bi32) as u16);
    (*E).mbn_var_19 = 16u16;
    MotherBrain_Phase3_Neck_RecoilRecovery(game);
}
pub unsafe fn MotherBrain_Phase3_Neck_RecoilRecovery(game: &mut Game) {
    let mut v1: i16 = 0;
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
    v1 = (((*E).mbn_var_19) as i16).wrapping_sub(1i16);
    if v1 < 0i16 {
        MotherBrain_SetBrainInstrs(addr_kMotherBrain_Ilist_9DBB as u16, game);
        (*E).mbn_var_18 = ((0xa9c330i32) as u16);
    } else {
        (*E).mbn_var_19 = v1 as u16;
    };
}
pub unsafe fn MotherBrain_Phase3_Neck_SetupHyperBeamRecoil(game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
    (*E).mbn_var_31 = 1u16;
    (*Get_MotherBrain(0i32, game)).mbn_var_06 = 1u16;
    MotherBrain_SetBrainInstrs(addr_kMotherBrain_Ilist_9BE7 as u16, game);
    (*E).mbn_var_00 = 50u16;
    (*E).mbn_var_34 = 2304u16;
    (*E).mbn_var_32 = 8u16;
    (*E).mbn_var_33 = 8u16;
    (*E).mbn_var_18 = ((0xa9c3cdi32) as u16);
    (*E).mbn_var_19 = 11u16;
    MotherBrain_Phase3_Neck_HyperBeamRecoil(game);
}
pub unsafe fn MotherBrain_Phase3_Neck_HyperBeamRecoil(game: &mut Game) {
    let mut v2: i16 = 0;
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
    v2 = (((*E).mbn_var_19) as i16).wrapping_sub(1i16);
    if v2 < 0i16 {
        (*E).mbn_var_34 = 128u16;
        (*Get_MotherBrain(0i32, game)).mbn_var_06 = 0u16;
        (*E).mbn_var_18 = ((0xa9c354i32) as u16);
    } else {
        (*E).mbn_var_19 = v2 as u16;
    };
}
pub unsafe fn MoveEnemyWithVelocity(game: &mut Game) {
    let v0: *mut EnemyData = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
    let mut ai_var_B: u16 = (*v0).ai_var_B;
    let mut carry: i32 = *(&raw mut (*v0).x_subpos as *mut u8).offset(1isize) as i32
        + *(&raw mut ai_var_B as *mut u8).offset(0) as i32;
    *(&raw mut (*v0).x_subpos as *mut u8).offset(1isize) = carry as u8;
    (*v0).x_pos =
        ((*v0).x_pos as i32 + ((ai_var_B as i32 >> 8i32) as i8 as i32 + (carry >> 8i32))) as u16;
    let mut ai_var_C: u16 = (*v0).ai_var_C;
    carry = *(&raw mut (*v0).y_subpos as *mut u8).offset(1isize) as i32
        + *(&raw mut ai_var_C as *mut u8).offset(0) as i32;
    *(&raw mut (*v0).y_subpos as *mut u8).offset(1isize) = carry as u8;
    (*v0).y_pos =
        ((*v0).y_pos as i32 + ((ai_var_C as i32 >> 8i32) as i8 as i32 + (carry >> 8i32))) as u16;
}
pub unsafe fn MotherBrain_SetBodyInstrs(a: u16, game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).base.current_instruction = a;
    (*E).base.instruction_timer = 1u16;
    (*E).base.timer = 0u16;
}
pub unsafe fn MotherBrain_SetBrainUnusedInstrs(a: u16, game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
    (*E).base.current_instruction = a;
    (*E).base.instruction_timer = 1u16;
    (*E).base.timer = 0u16;
}
pub unsafe fn MotherBrain_SetBrainInstrs(a: u16, game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).mbn_var_21 = a;
    (*E).mbn_var_20 = 1u16;
}
pub unsafe fn Enemy_SetInstrList(k: u16, a: u16, game: &mut Game) {
    let v2: *mut EnemyData = gEnemyData(k as i32, game);
    (*v2).current_instruction = a;
    (*v2).instruction_timer = 1u16;
    (*v2).timer = 0u16;
}
pub unsafe fn ComputeSinMult(a: u16, r18: u16, game: &mut Game) -> u16 {
    sub_A9C46C(r18, a, game)
}
pub unsafe fn ComputeCosMult(a: u16, r18: u16, game: &mut Game) -> u16 {
    sub_A9C46C((r18).wrapping_add(64u16), a, game)
}
pub unsafe fn sub_A9C46C(a: u16, j: u16, game: &mut Game) -> u16 {
    let mut v2: u16 = kSinCosTable8bit_Sext[((a as i32 & 0xffi32) + 64i32) as usize] as u16;
    crate::rtl::register_writes! { game;
        byte(M7A as u16, v2 as u8); byte(M7A as u16, *(&raw mut v2 as *mut u8).offset(1isize));
        byte(M7B as u16, j as u8);
    }
    ReadRegWord(MPYM as u16, game)
}
pub unsafe fn Enemy_IncreaseYpos(k: u16, mut a: u16, game: &mut Game) {
    let v2: *mut EnemyData = gEnemyData(k as i32, game);
    let carry: i32 = *(&raw mut (*v2).y_subpos as *mut u8).offset(1isize) as i32
        + *(&raw mut a as *mut u8).offset(0) as i32;
    *(&raw mut (*v2).y_subpos as *mut u8).offset(1isize) = carry as u8;
    (*v2).y_pos = ((*v2).y_pos as i32 + ((a as i32 >> 8i32) as i8 as i32 + (carry >> 8i32))) as u16;
}
pub unsafe fn Samus_DecrementAmmoDueToRainbowBeam(game: &mut Game) {
    if (ram_word!(game, random_enemy_counter) as i32 & 3i32) == 0i32
        && ram_word!(game, samus_missiles) != 0u16
    {
        let mut v0: u16 = (ram_word!(game, samus_missiles)).wrapping_sub(g_word_A9C544);
        if ram_word!(game, samus_missiles) as i32 - g_word_A9C544 as i32 - 1i32 & 0x8000i32 != 0 {
            if ram_word!(game, hud_item_index) == 1u16 {
                set_ram_word!(game.hud_item_index = 0u16);
            }
            v0 = 0u16;
            set_ram_word!(game.samus_auto_cancel_hud_item_index = 0u16);
        }
        set_ram_word!(game.samus_missiles = v0);
    }
    if (ram_word!(game, random_enemy_counter) as i32 & 3i32) == 0i32
        && ram_word!(game, samus_super_missiles) != 0u16
    {
        let mut v1: u16 = (ram_word!(game, samus_super_missiles)).wrapping_sub(g_word_A9C544);
        if ram_word!(game, samus_super_missiles) as i32 - g_word_A9C544 as i32 - 1i32 & 0x8000i32
            != 0
        {
            if ram_word!(game, hud_item_index) == 2u16 {
                set_ram_word!(game.hud_item_index = 0u16);
            }
            v1 = 0u16;
            set_ram_word!(game.samus_auto_cancel_hud_item_index = 0u16);
        }
        set_ram_word!(game.samus_super_missiles = v1);
    }
    if ram_word!(game, samus_power_bombs) != 0 {
        let mut v2: u16 = (ram_word!(game, samus_power_bombs)).wrapping_sub(g_word_A9C544);
        if ram_word!(game, samus_power_bombs) as i32 - g_word_A9C544 as i32 - 1i32 & 0x8000i32 != 0
        {
            if ram_word!(game, hud_item_index) == 3u16 {
                set_ram_word!(game.hud_item_index = 0u16);
            }
            v2 = 0u16;
            set_ram_word!(game.samus_auto_cancel_hud_item_index = 0u16);
        }
        set_ram_word!(game.samus_power_bombs = v2);
    }
}
pub unsafe fn Samus_PlayGainingLosingHealthSfx(game: &mut Game) {
    if ram_word!(game, samus_health) as i32 - 81i32 & 0x8000i32 == 0
        && (ram_word!(game, random_enemy_counter) as i32 & 7i32) == 0i32
    {
        QueueSfx3_Max3(0x2du16, game);
    }
}
pub unsafe fn Samus_DamageDueToShitroid(game: &mut Game) {
    let mut v0: i16 = 0;
    v0 = -4i16;
    if ram_word!(game, equipped_items) as i32 & 1i32 != 0 {
        v0 = -3i16;
    }
    let mut v1: u16 = (ram_word!(game, samus_health) as i32
        + (ram_word!(game, equipped_items) as i32 & 1i32)
        + v0 as i32) as u16;
    if v1 as i32 - 2i32 & 0x8000i32 != 0 {
        v1 = 1u16;
    }
    set_ram_word!(game.samus_health = v1);
}
pub unsafe fn Samus_DamageDueToRainbowBeam(game: &mut Game) {
    let mut v0: u16 = (ram_word!(game, samus_health) as i32
        + (ram_word!(game, equipped_items) as i32 & 1i32)
        - 2i32) as u16;
    if v0 as i32 - 1i32 & 0x8000i32 != 0 {
        v0 = 0u16;
    }
    set_ram_word!(game.samus_health = v0);
    Samus_PlayGainingLosingHealthSfx(game);
}
pub unsafe fn Samus_HealDueToShitroid(game: &mut Game) -> u8 {
    if ((ram_word!(game, samus_health) as i32 + 1i32 - ram_word!(game, samus_max_health) as i32)
        as i16)
        < 0i16
    {
        update_ram_word!(game.samus_health, wrapping_add(1));
        Samus_PlayGainingLosingHealthSfx(game);
        1u8
    } else {
        set_ram_word!(game.samus_health = ram_word!(game, samus_max_health));
        Samus_PlayGainingLosingHealthSfx(game);
        0u8
    }
}
pub unsafe fn ProcessSpriteTilesTransfers(db: u8, k: u16, game: &mut Game) -> u8 {
    let mut v7: *mut VramWriteEntry = null_mut::<VramWriteEntry>();
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    let mut src: u16 = (*E).mbn_var_22;
    if src == 0 {
        src = k;
    }
    let v4: u16 = ram_word!(game, vram_write_queue_tail);
    let v5: *const u8 = RomPtrWithBank(db, src, game);
    let mut v6: u16 = *(v5 as *mut u16);
    if v6 != 0 {
        v7 = gVramWriteEntry(ram_word!(game, vram_write_queue_tail) as i32, game);
        (*v7).size = v6;
        *((&raw mut (*v7).src.addr as *mut u8).offset(1) as *mut VoidP) =
            *(v5.offset(3) as *mut u16);
        (*v7).src.addr = *(v5.offset(2) as *mut u16);
        (*v7).vram_dst = *(v5.offset(5) as *mut u16);
        set_ram_word!(game.vram_write_queue_tail = (v4).wrapping_add(7u16));
        (*E).mbn_var_22 = (src).wrapping_add(7u16);
        v6 = *(RomPtrWithBank(db, (*E).mbn_var_22, game) as *mut u16);
        if v6 != 0 {
            return 0u8;
        }
    }
    (*E).mbn_var_22 = v6;
    1u8
}
pub unsafe fn MotherBrain_MakeWalkForwards(j: u16, a: u16, game: &mut Game) -> u8 {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    if ((a) as i16).wrapping_sub(((*E).base.x_pos) as i16) >= 0i16 {
        if (*E).mbn_var_02 != 0 {
            return 0u8;
        }
        if (*E).base.x_pos as i32 - 128i32 & 0x8000i32 != 0 {
            MotherBrain_SetBodyInstrs(
                crate::rtl::rom_table_word!(game, 0xa9c61e, (j as i32 >> 1i32) as isize),
                game,
            );
            return 0u8;
        }
    }
    1u8
}
pub unsafe fn MotherBrain_MakeWalkBackwards(a: u16, j: u16, game: &mut Game) -> u8 {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    if ((a) as i16).wrapping_sub(((*E).base.x_pos) as i16) < 0i16 {
        if (*E).mbn_var_02 != 0 {
            return 0u8;
        }
        if (*E).base.x_pos as i32 - 48i32 & 0x8000i32 == 0 {
            MotherBrain_SetBodyInstrs(
                crate::rtl::rom_table_word!(game, 0xa9c664, (j as i32 >> 1i32) as isize),
                game,
            );
            return 0u8;
        }
    }
    1u8
}
pub unsafe fn MotherBrain_MakeHerStandUp(game: &mut Game) -> u8 {
    let mbn_var_02: u16 = (*Get_MotherBrain(0i32, game)).mbn_var_02;
    if mbn_var_02 != 0 {
        let mut v1: u16 = addr_kMotherBrain_Ilist_99C6 as u16;
        if mbn_var_02 != 3u16 {
            if mbn_var_02 != 6u16 {
                return 0u8;
            }
            v1 = addr_kMotherBrain_Ilist_99E2 as u16;
        }
        MotherBrain_SetBodyInstrs(v1, game);
        return 0u8;
    }
    1u8
}
pub unsafe fn MotherBrain_MakeHerCrouch(game: &mut Game) -> u8 {
    let mbn_var_02: u16 = (*Get_MotherBrain(0i32, game)).mbn_var_02;
    if mbn_var_02 != 0 {
        if mbn_var_02 == 3u16 {
            return 1u8;
        }
    } else {
        MotherBrain_SetBodyInstrs(addr_kMotherBrain_Ilist_9A0A as u16, game);
    }
    0u8
}
pub unsafe fn MotherBrain_MakeHerLeanDown(game: &mut Game) -> u8 {
    let mbn_var_02: u16 = (*Get_MotherBrain(0i32, game)).mbn_var_02;
    if mbn_var_02 != 0 {
        if mbn_var_02 == 6u16 {
            return 1u8;
        }
    } else {
        MotherBrain_SetBodyInstrs(addr_kMotherBrain_Ilist_99F2 as u16, game);
    }
    0u8
}
pub unsafe fn MotherBrain_HandleWalking(game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    if (*E).mbn_var_02 == 0 {
        let mut v2: u16 = 0;

        let mbn_var_07: u16 = (*E).mbn_var_07;
        let can_pause_walk: bool = if mbn_var_07 != 0 {
            v2 = 0;
            v2 = (mbn_var_07).wrapping_add(6u16);
            (*E).mbn_var_07 = v2;
            !(v2 >= 0x100u16)
                && ({
                    if (*E).base.x_pos as i32 - 48i32 & 0x8000i32 == 0 {
                        return;
                    }
                    true
                })
        } else {
            (*E).mbn_var_07 = 1u16;
            if (*E).base.x_pos as i32 - 48i32 & 0x8000i32 == 0 {
                MotherBrain_SetBodyInstrs(addr_kMotherBrain_Ilist_98C6 as u16, game);
                return;
            }
            true
        };
        if can_pause_walk
            && (ram_word!(game, random_number) as i32 & 0xfffi32) - 4032i32 & 0x8000i32 != 0
        {
            return;
        }
        (*E).mbn_var_07 = 128u16;
        if (*E).base.x_pos as i32 - 128i32 & 0x8000i32 != 0 {
            MotherBrain_SetBodyInstrs(addr_kMotherBrain_Ilist_97A4 as u16, game);
        }
    }
}
pub unsafe fn ShitroidInCutscene_Init(game: &mut Game) {
    let E: *mut Enemy_ShitroidInCutscene =
        Get_ShitdroidInCutscene(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.properties = ((*E).base.properties as i32
        | (kEnemyProps_DisableSamusColl as i32 | kEnemyProps_BlockPlasmaBeam as i32))
        as u16;
    (*E).base.palette_index = 3584u16;
    (*E).base.current_instruction = addr_kShitroid_Ilist_CFA2 as u16;
    (*E).base.instruction_timer = 1u16;
    (*E).sice_var_04 = 1u16;
    (*E).base.timer = 0u16;
    (*E).sice_var_E = 10u16;
    (*E).base.vram_tiles_index = 160u16;
    (*E).base.x_pos = 320u16;
    (*E).base.y_pos = 96u16;
    (*E).sice_var_B = 0u16;
    (*E).sice_var_C = 0u16;
    (*E).sice_var_09 = 0u16;
    (*E).sice_var_A = ((0xa9c7cci32) as u16);
    (*E).sice_var_F = 248u16;
    (*E).sice_var_0F = ((0xa9f683i32) as u16);
    WriteColorsToPalette(
        0x1e2u16,
        0xa9u8,
        (addr_kMotherBrainPalette_2 as i32 + 2i32) as u16,
        0xfu16,
        game,
    );
}
crate::game::address_dispatch! {
    pub unsafe fn CallShitroidCutsceneFuncA(ea: u32, k: u16, game: &mut Game) => ea, game {
        11125133 => (),
        11126732 => ShitroidInCutscene_DashOntoScreen(k, game),
        11126764 => ShitroidInCutscene_CurveTowardsBrain(k, game),
        11126801 => ShitroidInCutscene_GetIntoFace(k, game),
        11126865 => ShitroidInCutscene_LatchOntoBrain(k, game),
        11126905 => ShitroidInCutscene_SetMotherBrainToStumbleBack(k, game),
        11126921 => ShitroidInCutscene_ActivateRainbowBeam(k, game),
        11127010 => ShitroidInCutscene_BrainTurnsToCorpse(k, game),
        11127061 => ShitroidInCutscene_StopDraining(k, game),
        11127115 => ShitroidInCutscene_LetGoAndSpawnDust(k, game),
        11127129 => ShitroidInCutscene_MoveUpToCeiling(k, game),
        11127235 => ShitroidInCutscene_MoveToSamus(k, game),
        11127398 => ShitroidInCutscene_LatchOntoSamus(k, game),
        11127418 => ShitroidInCutscene_HealSamusToFullHealth(k, game),
        11127485 => ShitroidInCutscene_IdleUntilToNoHealth(k, game),
        11127571 => ShitroidInCutscene_ReleaseSamus(k, game),
        11127597 => ShitroidInCutscene_StareDownMotherBrain(k, game),
        11127638 => ShitroidInCutscene_FlyOffScreen(k, game),
        11127675 => ShitroidInCutscene_MoveToFinalChargeStart(k, game),
        11127731 => ShitroidInCutscene_InitiateFinalCharge(k, game),
        11127768 => ShitroidInCutscene_FinalCharge(k, game),
        11127870 => ShitroidInCutscene_ShitroidFinalBelow(k, game),
        11127904 => ShitroidInCutscene_PlaySamusTheme(k, game),
        11127935 => ShitroidInCutscene_PrepareSamusHyperbeam(k, game),
        11127961 => ShitroidInCutscene_DeathSequence(k, game),
        11128000 => ShitroidInCutscene_UnloadShitroid(k, game),
        11128030 => ShitroidInCutscene_LetSamusRainbowMore(k, game),
        11128048 => ShitroidInCutscene_FinishCutscene(k, game),
    }
    pub unsafe fn CallShitroidCutsceneFunc0F(ea: u32, game: &mut Game) => ea, game {
        11127484 => (),
        11127960 => (),
        11138691 => Shitroid_HandleCutscenePalette(game),
        11138703 => Shitroid_HandleCutscenePalette_LowHealth(game),
    }
}
pub unsafe fn ShitroidInCutscene_Main(game: &mut Game) {
    let E: *mut Enemy_ShitroidInCutscene =
        Get_ShitdroidInCutscene(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.shake_timer = 0u16;
    CallShitroidCutsceneFuncA(
        (((*E).sice_var_A) as u32) | (0xa90000u32),
        ram_word!(game, cur_enemy_index),
        game,
    );
    MoveEnemyWithVelocity(game);
    ShitroidInCutscene_Flashing(ram_word!(game, cur_enemy_index), game);
    ShitroidInCutscene_HandleHealthBasedPalette(ram_word!(game, cur_enemy_index), game);
    let r18: u16 =
        (*Get_ShitdroidInCutscene(ram_word!(game, cur_enemy_index) as i32, game)).sice_var_0F;
    CallShitroidCutsceneFunc0F(((r18) as u32) | (0xa90000u32), game);
}
pub unsafe fn ShitroidInCutscene_Flashing(k: u16, game: &mut Game) {
    let mut v1: i16 = 0;
    v1 = 3584i16;
    let E: *mut Enemy_ShitroidInCutscene = Get_ShitdroidInCutscene(k as i32, game);
    let sice_var_06: u16 = (*E).sice_var_06;
    if sice_var_06 != 0 {
        let v4: u16 = (sice_var_06).wrapping_sub(1u16);
        (*E).sice_var_06 = v4;
        if (v4 as i32 & 2i32) != 0i32 {
            v1 = 0i16;
        }
    }
    (*E).base.palette_index = v1 as u16;
}
pub unsafe fn ShitroidInCutscene_HandleCry(game: &mut Game) {
    let E: *mut Enemy_ShitroidInCutscene = Get_ShitdroidInCutscene(0i32, game);
    if (*E).sice_var_14 != 0 {
        (*E).sice_var_14 = 0u16;
        QueueSfx2_Max6(0x72u16, game);
    }
}
pub unsafe fn ShitroidInCutscene_DashOntoScreen(k: u16, game: &mut Game) {
    let E: *mut Enemy_ShitroidInCutscene = Get_ShitdroidInCutscene(k as i32, game);
    (*E).sice_var_F = (*E).sice_var_F.wrapping_sub(1);
    if ((*E).sice_var_F as i32 & 0x8000i32) != 0i32 {
        (*E).sice_var_0A = -10240i32 as u16;
        (*E).sice_var_0B = 2560u16;
        (*E).sice_var_A = ((0xa9c7eci32) as u16);
        (*E).sice_var_F = 10u16;
        ShitroidInCutscene_CurveTowardsBrain(k, game);
    }
}
pub unsafe fn ShitroidInCutscene_CurveTowardsBrain(k: u16, game: &mut Game) {
    ShitroidInCutscene_UpdateSpeedAndAngle(k, -384i32 as u16, -20480i32 as u16, 2560u16, game);
    let E: *mut Enemy_ShitroidInCutscene = Get_ShitroidInCutscene(k as i32, game);
    (*E).sice_var_F = (*E).sice_var_F.wrapping_sub(1);
    if ((*E).sice_var_F as i32 & 0x8000i32) != 0i32 {
        (*E).sice_var_A = ((0xa9c811i32) as u16);
        (*E).sice_var_F = 9u16;
    }
}
pub unsafe fn ShitroidInCutscene_GetIntoFace(k: u16, game: &mut Game) {
    let mut v3: bool = false;
    ShitroidInCutscene_UpdateSpeedAndAngle(k, -1536i32 as u16, -32256i32 as u16, 3584u16, game);
    let mut E: *mut Enemy_ShitroidInCutscene = Get_ShitroidInCutscene(0x40i32, game);
    let rect: Rect16U = Rect16U {
        x: (*E).base.x_pos,
        y: (*E).base.y_pos,
        w: 4u16,
        h: 4u16,
    };
    if Shitroid_Func_2(k, rect, game) == 0 || {
        E = Get_ShitroidInCutscene(k as i32, game);
        v3 = (((*E).sice_var_F) as i16).wrapping_sub(1i16) < 0i16;
        (*E).sice_var_F = (*E).sice_var_F.wrapping_sub(1);
        v3 as i32 != 0
    } {
        (*Get_ShitroidInCutscene(k as i32, game)).sice_var_A = ((0xa9c851i32) as u16);
        SomeMotherBrainScripts(1u16, game);
    }
}
pub unsafe fn ShitroidInCutscene_LatchOntoBrain(k: u16, game: &mut Game) {
    let E: *mut Enemy_ShitroidInCutscene = Get_ShitroidInCutscene(0x40i32, game);
    let rect: Rect16U = Rect16U {
        x: (*E).base.x_pos,
        y: (((*E).base.y_pos).wrapping_sub(24u16)),
        w: 8u16,
        h: 8u16,
    };
    Shitroid_Func_GraduallyAccelerateTowards0x400(k, 0u16, rect.x, rect.y, game);
    if Shitroid_Func_2(k, rect, game) == 0 {
        (*Get_ShitroidInCutscene(k as i32, game)).sice_var_A = ((0xa9c879i32) as u16);
    }
}
pub unsafe fn ShitroidInCutscene_SetMotherBrainToStumbleBack(k: u16, game: &mut Game) {
    let E: *mut Enemy_ShitroidInCutscene = Get_ShitroidInCutscene(0i32, game);
    MotherBrain_MakeWalkBackwards(((*E).base.x_pos).wrapping_sub(1u16), 2u16, game);
    (*Get_ShitroidInCutscene(k as i32, game)).sice_var_A = ((0xa9c889i32) as u16);
    ShitroidInCutscene_ActivateRainbowBeam(k, game);
}
pub unsafe fn ShitroidInCutscene_ActivateRainbowBeam(k: u16, game: &mut Game) {
    let E1: *mut Enemy_ShitroidInCutscene = Get_ShitdroidInCutscene(0x40i32, game);
    if Shitroid_AccelerateTowardsPoint(
        k,
        0x200u16,
        (*E1).base.x_pos,
        ((*E1).base.y_pos).wrapping_sub(24u16),
        game,
    ) as i32
        & 1i32
        != 0
    {
        let E: *mut Enemy_ShitroidInCutscene = Get_ShitdroidInCutscene(k as i32, game);
        (*E).sice_var_B = 0u16;
        (*E).sice_var_C = 0u16;
        (*E).base.x_pos = (*E1).base.x_pos;
        (*E).base.y_pos = ((*E1).base.y_pos).wrapping_sub(24u16);
        Enemy_SetInstrList(k, addr_kShitroid_Ilist_CFB8 as u16, game);
        (*E).sice_var_A = ((0xa9c8e2i32) as u16);
        (*E).sice_var_E = 1u16;
        let E0: *mut Enemy_ShitroidInCutscene = Get_ShitdroidInCutscene(0i32, game);
        (*E0).sice_var_A = ((0xa9be38i32) as u16);
        QueueSfx1_Max6(0x40u16, game);
        (*E0).sice_var_16 = 1u16;
    }
}
pub unsafe fn ShitroidInCutscene_BrainTurnsToCorpse(k: u16, game: &mut Game) {
    let E: *mut Enemy_ShitroidInCutscene = Get_ShitdroidInCutscene(k as i32, game);
    let E1: *mut Enemy_ShitroidInCutscene = Get_ShitdroidInCutscene(0x40i32, game);
    let v2: i32 = ((*E).base.frame_counter as i32 & 6i32) >> 1i32;
    (*E).base.x_pos = ((*E1).base.x_pos).wrapping_add((g_word_A993BB[v2 as usize]) as u16);
    (*E).base.y_pos = ((*E1).base.y_pos as i32 + g_word_A993C3[v2 as usize] as i32 - 24i32) as u16;
    if (*Get_ShitdroidInCutscene(0i32, game)).sice_var_1F != 0 {
        (*E).sice_var_A = ((0xa9c915i32) as u16);
        (*E).sice_var_F = 64u16;
    }
}
pub unsafe fn ShitroidInCutscene_StopDraining(k: u16, game: &mut Game) {
    let E: *mut Enemy_ShitroidInCutscene = Get_ShitroidInCutscene(k as i32, game);
    let E1: *mut Enemy_ShitroidInCutscene = Get_ShitroidInCutscene(0x40i32, game);
    (*E).base.x_pos = (*E1).base.x_pos;
    (*E).base.y_pos = ((*E1).base.y_pos).wrapping_sub(24u16);
    (*E).sice_var_F = (*E).sice_var_F.wrapping_sub(1);
    if ((*E).sice_var_F as i32 & 0x8000i32) != 0i32 {
        Enemy_SetInstrList(k, addr_kShitroid_Ilist_CFA2 as u16, game);
        (*E).sice_var_E = 10u16;
        (*E).sice_var_A = ((0xa9c94bi32) as u16);
        (*E).sice_var_F = 32u16;
        (*E).sice_var_B = 0u16;
        (*E).sice_var_C = 0u16;
    }
}
pub unsafe fn ShitroidInCutscene_LetGoAndSpawnDust(k: u16, game: &mut Game) {
    let E: *mut Enemy_ShitroidInCutscene = Get_ShitdroidInCutscene(k as i32, game);
    (*E).sice_var_F = (*E).sice_var_F.wrapping_sub(1);
    if ((*E).sice_var_F as i32 & 0x8000i32) != 0i32 {
        ShitroidInCutscene_SpawnThreeDustClouds(game);
        (*E).sice_var_A = addr_loc_A9C959 as u16;
    }
    ShitroidInCutscene_MoveUpToCeiling(k, game);
}
pub unsafe fn ShitroidInCutscene_MoveUpToCeiling(k: u16, game: &mut Game) {
    let rect: Rect16U = Rect16U {
        x: (*Get_ShitdroidInCutscene(0x40i32, game)).base.x_pos,
        y: 0u16,
        w: 4u16,
        h: 4u16,
    };
    Shitroid_Func_GraduallyAccelerateTowards0x400(k, 0u16, rect.x, rect.y, game);
    if Shitroid_Func_2(k, rect, game) == 0 {
        SomeMotherBrainScripts(4u16, game);
        let E: *mut Enemy_ShitroidInCutscene = Get_ShitdroidInCutscene(k as i32, game);
        (*E).sice_var_A = ((0xa9c9c3i32) as u16);
        (*E).sice_var_0E = addr_stru_A9CA24 as u16;
    }
}
pub unsafe fn ShitroidInCutscene_SpawnOneDustCloudAt(xd: u16, yd: u16, game: &mut Game) {
    let E: *mut Enemy_ShitroidInCutscene = Get_ShitroidInCutscene(0x40i32, game);
    game.eproj_spawn_pt = Point16U {
        x: (((*E).base.x_pos).wrapping_add(xd)),
        y: (((*E).base.y_pos).wrapping_add(yd)),
    };
    SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 9u16, game);
}
pub unsafe fn ShitroidInCutscene_SpawnThreeDustClouds(game: &mut Game) {
    ShitroidInCutscene_SpawnOneDustCloudAt(-16i32 as u16, -8i32 as u16, game);
    ShitroidInCutscene_SpawnOneDustCloudAt(0u16, -16i32 as u16, game);
    ShitroidInCutscene_SpawnOneDustCloudAt(16u16, -8i32 as u16, game);
}
crate::game::address_dispatch! {
    pub unsafe fn CallShitroidMoveFunc(ea: u32, k: u16, j: u16, r18: u16, r20: u16, game: &mut Game) => ea, game {
        11138143 => Shitroid_GraduallyAccelerateTowards0x8(k, j, r18, r20, game),
        11138150 => Shitroid_GraduallyAccelerateTowards0x10(k, j, r18, r20, game),
    }
}
pub unsafe fn ShitroidInCutscene_MoveToSamus(k: u16, game: &mut Game) {
    let mut v4: i16 = 0;
    let E: *mut Enemy_ShitroidInCutscene = Get_ShitdroidInCutscene(k as i32, game);
    (*E).sice_var_04 = 0u16;
    (*E).sice_var_09 = 1u16;
    if (ram_word!(game, random_number) as i32 & 0xfffi32) >= 0xfa0i32 {
        QueueSfx2_Max6(0x52u16, game);
    }
    let sice_var_0E: u16 = (*E).sice_var_0E;
    let v3: *const u16 = RomPtr_A9(sice_var_0E, game) as *const u16;
    let rect: Rect16U = Rect16U {
        x: *v3.offset(0),
        y: *v3.offset(1),
        w: 4u16,
        h: 4u16,
    };
    CallShitroidMoveFunc(
        ((*v3.offset(3)) as u32) | (0xa90000u32),
        k,
        *v3.offset(2),
        rect.x,
        rect.y,
        game,
    );
    if Shitroid_Func_2(k, rect, game) == 0 {
        v4 = *(RomPtr_A9(sice_var_0E, game) as *mut u16).offset(4) as i16;
        if v4 < 0i16 {
            (*E).sice_var_A = v4 as u16;
        } else {
            (*E).sice_var_0E = ((*E).sice_var_0E).wrapping_add(8u16);
        }
    }
}
pub unsafe fn ShitroidInCutscene_LatchOntoSamus(k: u16, game: &mut Game) {
    Shitroid_Func_GraduallyAccelerateTowards0x400(
        k,
        0u16,
        ram_word!(game, samus_x_pos),
        (ram_word!(game, samus_y_pos)).wrapping_sub(20u16),
        game,
    );
}
pub unsafe fn ShitroidInCutscene_HealSamusToFullHealth(k: u16, game: &mut Game) {
    let E: *mut Enemy_ShitroidInCutscene = Get_ShitdroidInCutscene(k as i32, game);
    (*E).sice_var_04 = 0u16;
    ShitroidInCutscene_HandleCry(game);
    let v2: i32 = ((*E).base.frame_counter as i32 & 6i32) >> 1i32;
    (*E).base.x_pos =
        (ram_word!(game, samus_x_pos)).wrapping_add((g_word_A993BB[v2 as usize]) as u16);
    (*E).base.y_pos =
        (ram_word!(game, samus_y_pos) as i32 + g_word_A993C3[v2 as usize] as i32 - 20i32) as u16;
    if Samus_HealDueToShitroid(game) as i32 & 1i32 == 0 {
        set_ram_word!(game.samus_reserve_health = ram_word!(game, samus_max_reserve_health));
        (*E).sice_var_A = ((0xa9cabdi32) as u16);
        (*E).sice_var_0F = ((0xa9cabci32) as u16);
    }
}
pub unsafe fn ShitroidInCutscene_IdleUntilToNoHealth(k: u16, game: &mut Game) {
    ShitroidInCutscene_HandleCry(game);
    let E: *mut Enemy_ShitroidInCutscene = Get_ShitdroidInCutscene(k as i32, game);
    let sice_var_06: u16 = (*E).sice_var_06;
    if sice_var_06 != 0 {
        let v3: i32 = (sice_var_06 as i32 & 6i32) >> 1i32;
        (*E).base.x_pos =
            (ram_word!(game, samus_x_pos) as i32 + 2i32 * g_word_A993BB[v3 as usize] as i32) as u16;
        (*E).base.y_pos = (ram_word!(game, samus_y_pos) as i32
            + 2i32 * g_word_A993C3[v3 as usize] as i32
            - 20i32) as u16;
    }
    if (*E).base.health == 0 {
        (*E).base.health = 320u16;
        (*E).sice_var_A = ((0xa9cb13i32) as u16);
        (*E).sice_var_E = 10u16;
        (*E).sice_var_D = 0u16;
        (*E).sice_var_0F = ((0xa9f68fi32) as u16);
        (*E).sice_var_04 = 1u16;
        (*E).sice_var_09 = 0u16;
    }
}
pub unsafe fn ShitroidInCutscene_ReleaseSamus(k: u16, game: &mut Game) {
    QueueSfx2_Max6(0x72u16, game);
    (*Get_ShitdroidInCutscene(0x40i32, game)).sice_var_0B = 1u16;
    (*Get_ShitdroidInCutscene(k as i32, game)).sice_var_A = ((0xa9cb2di32) as u16);
    (*Get_ShitdroidInCutscene(0i32, game)).sice_var_A = ((0xa9c18ei32) as u16);
    ShitroidInCutscene_StareDownMotherBrain(k, game);
}
pub unsafe fn ShitroidInCutscene_StareDownMotherBrain(k: u16, game: &mut Game) {
    let rect: Rect16U = Rect16U {
        x: ((ram_word!(game, samus_x_pos)).wrapping_sub(4u16)),
        y: 96u16,
        w: 4u16,
        h: 4u16,
    };
    Shitroid_GraduallyAccelerateTowards0x10(k, 0u16, rect.x, rect.y, game);
    if Shitroid_Func_2(k, rect, game) == 0 {
        (*Get_ShitroidInCutscene(k as i32, game)).sice_var_A = ((0xa9cb56i32) as u16);
    }
}
pub unsafe fn ShitroidInCutscene_FlyOffScreen(k: u16, game: &mut Game) {
    let rect: Rect16U = Rect16U {
        x: 272u16,
        y: 64u16,
        w: 4u16,
        h: 4u16,
    };
    Shitroid_GraduallyAccelerateTowards0x10(k, 0u16, rect.x, rect.y, game);
    if Shitroid_Func_2(k, rect, game) == 0 {
        (*Get_ShitroidInCutscene(k as i32, game)).sice_var_A = ((0xa9cb7bi32) as u16);
    }
}
pub unsafe fn ShitroidInCutscene_MoveToFinalChargeStart(k: u16, game: &mut Game) {
    let rect: Rect16U = Rect16U {
        x: 305u16,
        y: 160u16,
        w: 4u16,
        h: 4u16,
    };
    Shitroid_GraduallyAccelerateTowards0x10(k, 0u16, rect.x, rect.y, game);
    if Shitroid_Func_2(k, rect, game) == 0 {
        let E: *mut Enemy_ShitroidInCutscene = Get_ShitdroidInCutscene(k as i32, game);
        (*E).base.health = 79u16;
        (*Get_ShitdroidInCutscene(0x40i32, game)).sice_var_0B = 0u16;
        (*Get_ShitdroidInCutscene(0i32, game)).sice_var_A = ((0xa9c19ai32) as u16);
        (*E).sice_var_A = ((0xa9cbb3i32) as u16);
    }
}
pub unsafe fn ShitroidInCutscene_InitiateFinalCharge(k: u16, game: &mut Game) {
    let rect: Rect16U = Rect16U {
        x: 290u16,
        y: 128u16,
        w: 4u16,
        h: 4u16,
    };
    Shitroid_Func_GraduallyAccelerateTowards0x400(k, 0xau16, rect.x, rect.y, game);
    if Shitroid_Func_2(k, rect, game) == 0 {
        (*Get_ShitroidInCutscene(k as i32, game)).sice_var_A = ((0xa9cbd8i32) as u16);
    }
}
pub unsafe fn ShitroidInCutscene_FinalCharge(k: u16, game: &mut Game) {
    let E1: *mut Enemy_ShitroidInCutscene = Get_ShitdroidInCutscene(0x40i32, game);
    Shitroid_Func_GraduallyAccelerateTowards0x400(
        k,
        0xcu16,
        (*E1).base.x_pos,
        ((*E1).base.y_pos).wrapping_sub(32u16),
        game,
    );
    let E: *mut Enemy_ShitroidInCutscene = Get_ShitdroidInCutscene(k as i32, game);
    if (*E).base.health == 0 {
        (*E).sice_var_0F = ((0xa9cc98i32) as u16);
        (*E).base.vram_tiles_index = 4256u16;
        QueueSfx3_Max6(0x19u16, game);
        TurnOffLightsForShitroidDeath(game);
        Enemy_SetInstrList(k, addr_kShitroid_Ilist_CFCE as u16, game);
        (*E).sice_var_B = 0u16;
        (*E).sice_var_C = 0u16;
        (*Get_ShitdroidInCutscene(0i32, game)).sice_var_A = addr_locret_A9C18D as u16;
        QueueMusic_Delayed8(0u16, game);
        (*E).sice_var_A = ((0xa9cc3ei32) as u16);
        (*E).sice_var_F = 16u16;
        (*E).sice_var_10 = (*E).base.x_pos;
        (*E).sice_var_11 = (*E).base.y_pos;
        ShitroidInCutscene_ShitroidFinalBelow(k, game);
    }
}
pub unsafe fn ShitroidInCutscene_ShitroidFinalBelow(k: u16, game: &mut Game) {
    ShitroidInCutscene_Shake(k, game);
    let E: *mut Enemy_ShitroidInCutscene = Get_ShitdroidInCutscene(k as i32, game);
    (*E).sice_var_F = (*E).sice_var_F.wrapping_sub(1);
    if ((*E).sice_var_F as i32 & 0x8000i32) != 0i32 {
        (*E).base.x_pos = (*E).sice_var_10;
        (*E).base.y_pos = (*E).sice_var_11;
        (*E).sice_var_A = ((0xa9cc60i32) as u16);
        (*E).sice_var_F = 56u16;
        ShitroidInCutscene_PlaySamusTheme(k, game);
    }
}
pub unsafe fn ShitroidInCutscene_PlaySamusTheme(k: u16, game: &mut Game) {
    let E: *mut Enemy_ShitroidInCutscene = Get_ShitroidInCutscene(k as i32, game);
    (*E).sice_var_F = (*E).sice_var_F.wrapping_sub(1);
    if ((*E).sice_var_F as i32 & 0x8000i32) != 0i32 {
        QueueMusic_Delayed8(0xff48u16, game);
        QueueMusic_Delayed8(5u16, game);
        (*E).sice_var_A = ((0xa9cc7fi32) as u16);
        (*E).sice_var_F = 12u16;
        ShitroidInCutscene_PrepareSamusHyperbeam(k, game);
    }
}
pub unsafe fn ShitroidInCutscene_PrepareSamusHyperbeam(k: u16, game: &mut Game) {
    let E: *mut Enemy_ShitroidInCutscene = Get_ShitdroidInCutscene(k as i32, game);
    (*E).sice_var_F = (*E).sice_var_F.wrapping_sub(1);
    if ((*E).sice_var_F as i32 & 0x8000i32) != 0i32 {
        CallSomeSamusCode(0x19u16, game);
        (*Get_ShitdroidInCutscene(0x40i32, game)).sice_var_16 = ((0xa9cd30i32) as u16);
        (*E).sice_var_A = ((0xa9cc99i32) as u16);
    }
}
pub unsafe fn ShitroidInCutscene_DeathSequence(k: u16, game: &mut Game) {
    HandleSamusRainbowPaletteAnimation(k, game);
    ShitroidInCutscene_AccelerateDownwards(k, game);
    if ShitroidInCutscene_FadeShitroidToBlack(k, game) as i32 & 1i32 != 0 {
        let E: *mut Enemy_ShitroidInCutscene = Get_ShitroidInCutscene(k as i32, game);
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Invisible) as u16);
        (*E).sice_var_A = ((0xa9ccc0i32) as u16);
        (*E).sice_var_F = 128u16;
    } else {
        ShitroidInCutscene_HandleShitroidDeathExplosions(k, game);
        ShitroidInCutscene_HandleEnemyBlinking(k, game);
    };
}
pub unsafe fn ShitroidInCutscene_UnloadShitroid(k: u16, game: &mut Game) {
    HandleSamusRainbowPaletteAnimation(k, game);
    let E: *mut Enemy_ShitroidInCutscene = Get_ShitroidInCutscene(k as i32, game);
    (*E).sice_var_F = (*E).sice_var_F.wrapping_sub(1);
    let v2: bool = ((*E).sice_var_F as i32 & 0x8000i32) != 0i32;
    if v2 && ProcessSpriteTilesTransfers(0xa9u8, addr_stru_A98FC7 as u16, game) != 0u8 {
        (*E).sice_var_A = ((0xa9ccdei32) as u16);
        (*E).sice_var_F = 176u16;
        ShitroidInCutscene_LetSamusRainbowMore(k, game);
    }
}
pub unsafe fn ShitroidInCutscene_LetSamusRainbowMore(k: u16, game: &mut Game) {
    let E: *mut Enemy_ShitroidInCutscene = Get_ShitdroidInCutscene(k as i32, game);
    (*E).sice_var_F = (*E).sice_var_F.wrapping_sub(1);
    if ((*E).sice_var_F as i32 & 0x8000i32) != 0i32 {
        (*E).sice_var_A = ((0xa9ccf0i32) as u16);
        (*Get_ShitdroidInCutscene(0i32, game)).sice_var_37 = 0u16;
        ShitroidInCutscene_FinishCutscene(k, game);
    }
}
pub unsafe fn ShitroidInCutscene_FinishCutscene(k: u16, game: &mut Game) {
    let E: *mut Enemy_ShitroidInCutscene = Get_ShitdroidInCutscene(0i32, game);
    let v2: u16 = ((*E).sice_var_37).wrapping_add(1u16);
    (*E).sice_var_37 = v2;
    if MotherBrain_Phase3_TurnLightsBackOn((v2).wrapping_sub(1u16), game) as i32 & 1i32 != 0 {
        (*E).sice_var_A = ((0xa9c1cfi32) as u16);
        CallSomeSamusCode(0x17u16, game);
        SomeMotherBrainScripts(3u16, game);
        let fresh12 = &raw mut (*Get_ShitdroidInCutscene(k as i32, game)).base.properties;
        fresh12.write_unaligned((fresh12.read_unaligned()) | ((kEnemyProps_Deleted) as u16));
        (*Get_ShitdroidInCutscene(0x40i32, game)).sice_var_0A = 0u16;
    }
}
crate::game::address_dispatch! {
    pub unsafe fn CallSamusRainbowPaletteFunc(ea: u32, k: u16, game: &mut Game) => ea, game {
        11128112 => SamusRainbowPaletteFunc_ActivateWhenEnemyLow(k, game),
        11128139 => SamusRainbowPaletteFunc_GraduallySlowDown(game),
    }
}
pub unsafe fn HandleSamusRainbowPaletteAnimation(k: u16, game: &mut Game) {
    let r18: u16 = (*Get_ShitdroidInCutscene(0x40i32, game)).sice_var_16;
    CallSamusRainbowPaletteFunc(((r18) as u32) | (0xa90000u32), k, game);
}
pub unsafe fn SamusRainbowPaletteFunc_ActivateWhenEnemyLow(k: u16, game: &mut Game) {
    if (((*Get_ShitdroidInCutscene(k as i32, game)).base.y_pos as i32 + 16i32
        - ram_word!(game, samus_y_pos) as i32) as i16)
        >= 0i16
    {
        CallSomeSamusCode(0x16u16, game);
        (*Get_ShitdroidInCutscene(0x40i32, game)).sice_var_16 = ((0xa9cd4bi32) as u16);
    }
}
pub unsafe fn SamusRainbowPaletteFunc_GraduallySlowDown(game: &mut Game) {
    let E: *mut Enemy_ShitroidInCutscene = Get_ShitdroidInCutscene(0x40i32, game);
    let sice_var_1D: u16 = (*E).sice_var_1D;
    (*E).sice_var_1D = (sice_var_1D).wrapping_add(768u16);
    if sice_var_1D >= 0xfd00u16 {
        let mut v2: u16 = (ram_word!(game, special_samus_palette_frame)).wrapping_add(1u16);
        if ram_word!(game, special_samus_palette_frame) as i32 - 9i32 & 0x8000i32 == 0 {
            v2 = 10u16;
        }
        set_ram_word!(game.special_samus_palette_frame = v2);
    }
}
pub unsafe fn ShitroidInCutscene_FadeShitroidToBlack(k: u16, game: &mut Game) -> u8 {
    let mut v2: i16 = 0;
    let E: *mut Enemy_ShitroidInCutscene = Get_ShitdroidInCutscene(k as i32, game);
    if (*E).base.y_pos as i32 - 128i32 & 0x8000i32 != 0 {
        return 0u8;
    }
    v2 = (((*E).sice_var_0C) as i16).wrapping_sub(1i16);
    if v2 >= 0i16 {
        (*E).sice_var_0C = v2 as u16;
        return 0u8;
    }
    (*E).sice_var_0C = 8u16;
    let v4: u16 = ((*E).sice_var_0D).wrapping_add(1u16);
    if (*E).sice_var_0D as i32 - 6i32 & 0x8000i32 == 0 {
        return 1u8;
    }
    (*E).sice_var_0D = v4;
    WriteColorsToPalette(
        0x1e2u16,
        0xadu8,
        crate::rtl::rom_table_word!(game, 0xade8e2, v4 as isize),
        0xeu16,
        game,
    );
    0u8
}
pub unsafe fn ShitroidInCutscene_HandleShitroidDeathExplosions(k: u16, game: &mut Game) {
    let mut v2: i16 = 0;
    let E: *mut Enemy_ShitroidInCutscene = Get_ShitdroidInCutscene(k as i32, game);
    v2 = (((*E).sice_var_08) as i16).wrapping_sub(1i16);
    if v2 < 0i16 {
        (*E).sice_var_08 = 4u16;
        let mut v3: u16 = ((*E).sice_var_07).wrapping_add(1u16);
        if (*E).sice_var_07 as i32 - 9i32 & 0x8000i32 == 0 {
            v3 = 0u16;
        }
        (*E).sice_var_07 = v3;
        let v4: i32 = ((4u16).wrapping_mul(v3)) as i32 >> 1i32;
        game.eproj_spawn_pt = Point16U {
            x: (((*E).base.x_pos).wrapping_add(crate::rtl::rom_table_word!(
                game,
                0xa9cdfc,
                v4 as isize
            ))),
            y: (((*E).base.y_pos).wrapping_add(crate::rtl::rom_table_word!(
                game,
                0xa9cdfc,
                (v4 + 1i32) as isize
            ))),
        };
        SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 3u16, game);
        QueueSfx3_Max3(0x13u16, game);
    } else {
        (*E).sice_var_08 = v2 as u16;
    };
}
pub unsafe fn ShitroidInCutscene_HandleEnemyBlinking(k: u16, game: &mut Game) {
    let E: *mut Enemy_ShitroidInCutscene = Get_ShitroidInCutscene(k as i32, game);
    if ((*E).base.frame_counter as i32 & 1i32) != 0i32 {
        (*E).base.properties =
            ((*E).base.properties as i32 & !(kEnemyProps_Invisible as i32)) as u16;
    } else {
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Invisible) as u16);
    };
}
pub unsafe fn ShitroidInCutscene_AccelerateDownwards(k: u16, game: &mut Game) {
    let mut v2: i16 = 0;
    let E: *mut Enemy_ShitroidInCutscene = Get_ShitroidInCutscene(k as i32, game);
    v2 = ((abs16((*E).sice_var_B)) as i16).wrapping_sub(32i16);
    if v2 < 0i16 {
        v2 = 0i16;
    }
    (*E).sice_var_B = (if (*E).sice_var_B as i32 & 0x8000i32 != 0 {
        -(v2 as i32)
    } else {
        v2 as i32
    }) as u16;
    (*E).sice_var_C = ((*E).sice_var_C).wrapping_add(2u16);
}
pub unsafe fn ShitroidInCutscene_HandleHealthBasedPalette(k: u16, game: &mut Game) {
    let E: *mut Enemy_ShitroidInCutscene = Get_ShitdroidInCutscene(k as i32, game);
    if (*E).sice_var_09 != 0 {
        let health: u16 = (*E).base.health;
        if health as i32 - 2560i32 & 0x8000i32 != 0 {
            let mut v3: u16 = 2u16;
            if health as i32 - 2240i32 & 0x8000i32 != 0 {
                v3 = 4u16;
                if health as i32 - 1920i32 & 0x8000i32 != 0 {
                    v3 = 6u16;
                    if health as i32 - 1600i32 & 0x8000i32 != 0 {
                        v3 = 8u16;
                        if health as i32 - 1280i32 & 0x8000i32 != 0 {
                            v3 = 10u16;
                            if health as i32 - 960i32 & 0x8000i32 != 0 {
                                v3 = 12u16;
                                if health as i32 - 640i32 & 0x8000i32 != 0 {
                                    v3 = 14u16;
                                }
                            }
                        }
                    }
                }
            }
            let v5: u16 = v3;
            let v4: u16 = crate::rtl::rom_table_word!(game, 0xade7e2, (v3 as i32 >> 1i32) as isize);
            WriteColorsToPalette(0x1e2u16, 0xadu8, v4, 4u16, game);
            WriteColorsToPalette(0x1f4u16, 0xadu8, (v4).wrapping_add(8u16), 5u16, game);
            WriteColorsToPalette(
                0x1eau16,
                0xadu8,
                crate::rtl::rom_table_word!(game, 0xade882, (v5 as i32 >> 1i32) as isize),
                5u16,
                game,
            );
        }
    }
}
pub unsafe fn ShitroidInCutscene_Shake(k: u16, game: &mut Game) {
    let E: *mut Enemy_ShitroidInCutscene = Get_ShitdroidInCutscene(k as i32, game);
    (*E).sice_var_C = (*E).sice_var_C.wrapping_sub(1);
    let v2: i32 = ((*E).base.frame_counter as i32 & 6i32) >> 1i32;
    (*E).base.x_pos = ((*E).sice_var_10).wrapping_add((g_word_A993BB[v2 as usize]) as u16);
    (*E).base.y_pos = ((*E).sice_var_11).wrapping_add((g_word_A993C3[v2 as usize]) as u16);
}
pub unsafe fn ShitroidInCutscene_Touch(game: &mut Game) {
    let E: *mut Enemy_ShitroidInCutscene =
        Get_ShitroidInCutscene(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).sice_var_A as i32 == 0xa9ca66i32 & 0xffffi32
        && Shitroid_AccelerateTowardsPoint(
            ram_word!(game, cur_enemy_index),
            0x10u16,
            ram_word!(game, samus_x_pos),
            (ram_word!(game, samus_y_pos)).wrapping_sub(20u16),
            game,
        ) as i32
            & 1i32
            != 0
    {
        (*E).sice_var_B = 0u16;
        (*E).sice_var_C = 0u16;
        (*E).sice_var_A = ((0xa9ca7ai32) as u16);
    }
}
pub unsafe fn ShitroidInCutscene_UpdateSpeedAndAngle(
    k: u16,
    mut r18: u16,
    r20: u16,
    r22: u16,
    game: &mut Game,
) {
    let E: *mut Enemy_ShitroidInCutscene = Get_ShitdroidInCutscene(k as i32, game);
    if r22 != (*E).sice_var_0B {
        if ((r22) as i16).wrapping_sub(((*E).sice_var_0B) as i16) >= 0i16 {
            let mut v3: u16 = ((*E).sice_var_0B).wrapping_add(32u16);
            if v3 as i32 - r22 as i32 & 0x8000i32 == 0 {
                v3 = r22;
            }
            (*E).sice_var_0B = v3;
        } else {
            let mut v2: u16 = ((*E).sice_var_0B).wrapping_sub(32u16);
            if v2 as i32 - r22 as i32 & 0x8000i32 != 0 {
                v2 = r22;
            }
            (*E).sice_var_0B = v2;
        }
    }
    let mut v4: u16 = 0;

    let use_target_angle: bool = if (r18 as i32 & 0x8000i32) != 0i32 {
        v4 = ((*E).sice_var_0A).wrapping_add(r18);
        (v4 as i32 - r20 as i32 & 0x8000i32 != 0)
    } else {
        v4 = ((*E).sice_var_0A).wrapping_add(r18);
        (v4 as i32 - r20 as i32 & 0x8000i32 == 0)
    };
    if use_target_angle {
        v4 = r20;
    }
    (*E).sice_var_0A = v4;
    r18 = *(&raw mut v4 as *mut u8).offset(1isize) as u16;
    (*E).sice_var_B = Math_MultBySin((*E).sice_var_0B, r18);
    (*E).sice_var_C = Math_MultByCos((*E).sice_var_0B, r18);
}
pub unsafe fn Shitroid_Instr_1(k: u16, jp: *const u16) -> *const u16 {
    addr_kShitroid_Ilist_CFA2 as i32 as usize as *const u16
}
pub unsafe fn Shitroid_Instr_2(k: u16, jp: *const u16) -> *const u16 {
    addr_kShitroid_Ilist_CFB8 as i32 as usize as *const u16
}
pub unsafe fn MotherBrain_Pal_ProcessInvincibility(game: &mut Game) {
    if (*Get_MotherBrain(0i32, game)).mbn_var_00 == 4u16 {
        let flash_timer: u16 = (*Get_MotherBrain(0x40i32, game)).base.flash_timer;
        if flash_timer as i32 >> 1i32 != 0 && flash_timer as i32 & 1i32 == 0 {
            let mut i: i32 = 28i32;
            while i >= 0i32 {
                let v2: i32 = i >> 1i32;
                let v3: u16 = ram_word!(game, palette_buffer[(v2 + 129i32)]);
                set_ram_word!(game.palette_buffer[(v2 + 145i32)] = v3);
                set_ram_word!(game.palette_buffer[(v2 + 177i32)] = v3);
                set_ram_word!(game.palette_buffer[(v2 + 65i32)] = v3);
                i -= 2i32;
            }
        }
    }
}
pub unsafe fn MotherBrain_Pal_BeginScreenFlash(game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).mbn_var_0E = addr_stru_A9D046 as u16;
    (*E).mbn_var_0F = 1u16;
}
pub unsafe fn MotherBrain_Pal_EndScreenFlash(game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    (*E).mbn_var_0E = 0u16;
    (*E).mbn_var_0F = 0u16;
    MotherBrain_Pal_WriteRoomPal(addr_word_A9D082 as u16, game);
}
pub unsafe fn MotherBrain_Pal_HandleRoomPal(game: &mut Game) {
    let j: u16 = HandleMotherBrainInstructionList(
        ((ram_ptr!(game, ExtraEnemyRam7800; 0x7800) as *mut u8)
            .offset_from(&raw mut game.g_ram as *mut u8) as c_long
            + 28 as c_long) as u16,
        game,
    );
    if j != 0 {
        MotherBrain_Pal_WriteRoomPal(j, game);
    }
}
pub unsafe fn MotherBrain_Pal_WriteRoomPal(j: u16, game: &mut Game) {
    WriteColorsToPalette(0x68u16, 0xa9u8, j, 12u16, game);
    WriteColorsToPalette(0xa6u16, 0xa9u8, (j).wrapping_add(24u16), 12u16, game);
    WriteColorsToPalette(0xe6u16, 0xa9u8, (j).wrapping_add(24u16), 12u16, game);
}
pub unsafe fn HandleMotherBrainInstructionList(a: u16, game: &mut Game) -> u16 {
    let mut execute_instruction: bool;
    let st: *mut MotherBrainInstrExecState = ram_ptr!(game, MotherBrainInstrExecState; a as isize);
    if ((*st).ip as i32 & 0x8000i32) == 0i32 {
        return 0u16;
    }
    let mut v2: u16 = (*st).ip;
    let mut v3: *const u16 = RomPtr_A9(v2, game) as *mut u16;
    execute_instruction = (*v3.offset(0) as i32 & 0x8000i32 != 0)
        || ({
            if (*st).timer != *v3.offset(0) {
                (*st).timer = (*st).timer.wrapping_add(1);
                return *v3.offset(1);
            }
            v2 = (v2).wrapping_add(4u16);
            false
        });
    loop {
        if execute_instruction {
            v2 = CallMotherBrainInstr(
                ((*v3.offset(0)) as u32) | (0xa90000u32),
                (v2).wrapping_add(2u16),
                game,
            );
            execute_instruction = false;
        } else {
            v3 = RomPtr_A9(v2, game) as *mut u16;
            if *v3.offset(0) as i32 & 0x8000i32 == 0 {
                break;
            } else {
                execute_instruction = true;
            }
        }
    }
    if *v3.offset(0) == 0 {
        (*st).ip = 0u16;
        (*st).timer = 0u16;
        0u16
    } else {
        (*st).timer = 1u16;
        (*st).ip = v2;
        *v3.offset(1)
    }
}
pub unsafe fn MotherBrain_HandlePalette(game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
    if (*E).mbn_var_10 != 0 {
        MotherBrain_HandleBrainPal(game);
    }
    if (*E).mbn_var_11 != 0 {
        MotherBrain_HealthBasedPaletteHandling(game);
    }
}
pub unsafe fn MotherBrain_SetupBrainNormalPal(game: &mut Game) {
    (*Get_MotherBrain(0i32, game)).mbn_var_E = 10u16;
}
pub unsafe fn MotherBrain_SetupBrainPalForLaser(game: &mut Game) {
    (*Get_MotherBrain(0i32, game)).mbn_var_E = 514u16;
}
pub unsafe fn MotherBrain_HandleBrainPal(game: &mut Game) {
    let mut mbn_var_D_high: i8 = 0;
    let mut mbn_var_D: i8 = 0;
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    if (*E).mbn_var_00 != 1u16 {
        mbn_var_D_high = *(&raw mut (*E).mbn_var_D as *mut u8).offset(1isize) as i8;
        if mbn_var_D_high != 0 {
            *(&raw mut (*E).mbn_var_D as *mut u8).offset(1isize) =
                ((mbn_var_D_high) as u8).wrapping_sub(1u8);
        } else {
            let r18: u16 = crate::rtl::rom_table_word!(
                game,
                0xa9d260,
                (*(&raw mut (*E).mbn_var_E as *mut u8).offset(1isize) as i32 >> 1i32) as isize,
            );
            *(&raw mut (*E).mbn_var_D as *mut u8).offset(1isize) = (*E).mbn_var_E as u8;
            mbn_var_D = (*E).mbn_var_D as i8;
            if mbn_var_D != 0i8
                || (*Get_MotherBrain(0x40i32, game)).mbn_var_A as i32 == 0xa987a2i32 & 0xffffi32
            {
                let v4: u8 = (mbn_var_D as i32 + 1i32 & 7i32) as u8;
                *(&raw mut (*E).mbn_var_D as *mut u8).offset(0) = v4;
                let mut v5: u16 = 290u16;
                if (*E).mbn_var_0D != 512u16 {
                    v5 = 482u16;
                }
                WriteColorsToPalette(
                    v5,
                    0xa9u8,
                    (r18 as i32 + 8i32 * v4 as i32) as u16,
                    3u16,
                    game,
                );
            }
        }
    }
}
pub unsafe fn WriteColorsToPalette(mut k: u16, db: u8, mut j: u16, a: u16, game: &mut Game) {
    let mut n: i32 = a as i32;
    loop {
        set_ram_word!(
            game.palette_buffer[(k as i32 >> 1i32)] = *(RomPtrWithBank(db, j, game) as *mut u16)
        );
        k = (k).wrapping_add(2u16);
        j = (j).wrapping_add(2u16);
        n -= 1;
        if (n == 0) {
            break;
        }
    }
}
pub unsafe fn WriteColorsToTargetPalette(db: u8, mut k: u16, mut j: u16, a: u16, game: &mut Game) {
    let mut n: i32 = a as i32;
    loop {
        set_ram_word!(
            game.target_palettes[(k as i32 >> 1i32)] = *(RomPtrWithBank(db, j, game) as *mut u16)
        );
        k = (k).wrapping_add(2u16);
        j = (j).wrapping_add(2u16);
        n -= 1;
        if (n == 0) {
            break;
        }
    }
}
pub unsafe fn DeadTorizo_Init(game: &mut Game) {
    let mut i: i32 = 4094i32;
    while i >= 0i32 {
        set_ram_word!(game.tilemap_stuff[(i >> 1i32)] = 0u16);
        i -= 2i32;
    }
    let E: *mut Enemy_DeadTorizo = Get_DeadTorizo(0i32, game);
    (*E).dto_var_A = ((0xa9d3adi32) as u16);
    (*E).base.properties = ((*E).base.properties) | (0xa000u16);
    (*E).base.current_instruction = addr_kDeadTorizo_Ilist_D6DC as u16;
    (*E).base.instruction_timer = 1u16;
    (*E).base.timer = 0u16;
    (*E).base.palette_index = 512u16;
    (*E).dto_var_B = 0u16;
    (*E).dto_var_C = 8u16;
    (*E).dto_var_04 = 0u16;
    (*E).dto_var_02 = 15u16;
    (*E).dto_var_03 = 0u16;
    InitializeEnemyCorpseRotting(0u16, addr_stru_A9DD58 as u16, game);
}
crate::game::address_dispatch! {
    pub unsafe fn CallDeadTorizoFuncA(ea: u32, game: &mut Game) => ea, game {
        11129773 => {
            DeadTorizo_WaitForSamusColl(ram_word!(game, cur_enemy_index), game);
        }
        11129830 => DeadTorizo_Rotting(game),
        11129800 => DeadTorizo_PreRotDelay(game),
        11129799 => (),
    }
}
pub unsafe fn DeadTorizo_Main(game: &mut Game) {
    let E: *mut Enemy_DeadTorizo = Get_DeadTorizo(0i32, game);
    if ((*E).base.properties as i32 & kEnemyProps_Tangible as i32) == 0i32
        && DeadTorizo_Func_0(game) as i32 & 1i32 != 0
    {
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
        (*E).dto_var_A = ((0xa9d3e6i32) as u16);
    }
    mov24(ram_ptr!(game, LongPtr; 0x178c), 0xa9d39au32);
    CallDeadTorizoFuncA((((*E).dto_var_A) as u32) | (0xa90000u32), game);
    DeadTorizo_Func_1(game);
}
pub unsafe fn DeadTorizo_MainGfxHook(game: &mut Game) {
    MotherBrain_AddSpritemapToOam(
        addr_kDeadTorizo_Sprmap_D761 as u16,
        296u16,
        187u16,
        0u16,
        game,
    );
}
pub unsafe fn DeadTorizo_WaitForSamusColl(k: u16, game: &mut Game) {
    if k == ram_word!(game, enemy_index_colliding_dirs[0])
        || k == ram_word!(game, enemy_index_colliding_dirs[1])
        || k == ram_word!(game, enemy_index_colliding_dirs[2])
        || k == ram_word!(game, enemy_index_colliding_dirs[3])
    {
        (*Get_DeadTorizo(0i32, game)).dto_var_A = ((0xa9d3c8i32) as u16);
    }
}
pub unsafe fn DeadTorizo_PreRotDelay(game: &mut Game) {
    let E: *mut Enemy_DeadTorizo = Get_DeadTorizo(0i32, game);
    let v1: u16 = ((*E).dto_var_04).wrapping_add(1u16);
    (*E).dto_var_04 = v1;
    if v1 >= 0x10u16 {
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
        (*E).dto_var_A = ((0xa9d3e6i32) as u16);
        DeadTorizo_Rotting(game);
    }
}
pub unsafe fn DeadTorizo_Rotting(game: &mut Game) {
    let E: *mut Enemy_DeadTorizo = Get_DeadTorizo(0i32, game);
    let v1: u16 = ((*E).dto_var_03).wrapping_add(1u16);
    (*E).dto_var_03 = v1;
    if v1 >= 0xfu16 {
        (*E).dto_var_03 = 0u16;
        let dto_var_02: u16 = (*E).dto_var_02;
        if dto_var_02 != 0 {
            DeadTorizo_CopyLineOfSandHeapTileData(dto_var_02, game);
            (*E).dto_var_02 = (*E).dto_var_02.wrapping_sub(1);
        }
    }
    (*E).dto_var_C = (*E).dto_var_C.wrapping_add(1);
    MoveEnemyWithVelocity(game);
    if ProcessCorpseRotting(0u16, game) as i32 & 1i32 == 0 {
        (*E).dto_var_A = ((0xa9d3c7i32) as u16);
    }
}
pub unsafe fn DeadTorizo_Powerbomb(game: &mut Game) {
    if ((*Get_DeadTorizo(0i32, game)).base.properties as i32 & kEnemyProps_Tangible as i32) == 0i32
    {
        DeadTorizo_Shot(game);
    }
}
pub unsafe fn DeadTorizo_Shot(game: &mut Game) {
    let E: *mut Enemy_DeadTorizo = Get_DeadTorizo(0i32, game);
    (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
    (*E).dto_var_A = ((0xa9d3e6i32) as u16);
}
pub unsafe fn DeadTorizo_Func_0(game: &mut Game) -> u8 {
    let mut v5: i16 = 0;
    let mut v3: u16 = 0;
    let mut v4: u16 = 0;
    let E: *mut Enemy_DeadTorizo = Get_DeadTorizo(0i32, game);
    let r18: u16 = (*E).base.x_pos;
    let r20: u16 = (*E).base.y_pos;
    let mut p: *const u8 = RomPtr_A9(addr_kDeadTorizo_Hitbox_D77C as u16, game);
    let mut R22: i32 = *(p as *mut u16) as i32;
    let mut R24: u16 = 0;
    p = p.offset(2);
    while R22 != 0 {
        if ((ram_word!(game, samus_y_pos)) as i16).wrapping_sub((r20) as i16) >= 0i16 {
            R24 = (ram_word!(game, samus_y_pos)).wrapping_sub(r20);
            v3 = *(p.offset(6) as *mut u16);
        } else {
            R24 = (r20).wrapping_sub(ram_word!(game, samus_y_pos));
            v3 = *(p.offset(2) as *mut u16);
        }
        if ((ram_word!(game, samus_y_radius) as i32 + abs16(v3) as i32 - R24 as i32) as i16) >= 0i16
        {
            if ((ram_word!(game, samus_x_pos)) as i16).wrapping_sub((r18) as i16) >= 0i16 {
                R24 = (ram_word!(game, samus_x_pos)).wrapping_sub(r18);
                v4 = *(p.offset(4) as *mut u16);
            } else {
                R24 = (r18).wrapping_sub(ram_word!(game, samus_x_pos));
                v4 = *(p as *mut u16);
            }
            v5 = (ram_word!(game, samus_x_radius) as i32 + abs16(v4) as i32 - R24 as i32) as i16;
            if v5 >= 0i16 {
                if v5 as i32 - 4i32 & 0x8000i32 != 0 {
                    v5 = 4i16;
                }
                set_ram_word!(game.extra_samus_x_displacement = v5 as u16);
                set_ram_word!(game.extra_samus_y_displacement = 4u16);
                set_ram_words! { game, 0u16;
                    extra_samus_x_subdisplacement, extra_samus_y_subdisplacement,
                }
                return 1u8;
            }
        }
        p = p.offset(8);
        R22 -= 1;
    }
    0u8
}
pub unsafe fn DeadTorizo_Func_1(game: &mut Game) {
    let mut v5: *mut VramWriteEntry = null_mut::<VramWriteEntry>();
    let mut v10: *mut VramWriteEntry = null_mut::<VramWriteEntry>();
    let E: *mut Enemy_DeadTorizo = Get_DeadTorizo(0i32, game);
    let v1: u16 = ((*E).dto_var_00).wrapping_add(1u16);
    (*E).dto_var_00 = v1;
    let mut v3: u16 = ram_word!(game, vram_write_queue_tail);
    if v1 as i32 & 1i32 != 0 {
        let mut v8: u16 = 0u16;
        let mut v9: u16 = crate::rtl::rom_table_word!(game, 0xa9d583, 0);
        loop {
            v10 = gVramWriteEntry(v3 as i32, game);
            (*v10).size = v9;
            let v11: i32 = v8 as i32 >> 1i32;
            *((&raw mut (*v10).src.addr as *mut u8).offset(1) as *mut VoidP) =
                crate::rtl::rom_table_word!(game, 0xa9d583, (v11 + 1i32) as isize);
            (*v10).src.addr = crate::rtl::rom_table_word!(game, 0xa9d583, (v11 + 2i32) as isize);
            (*v10).vram_dst = crate::rtl::rom_table_word!(game, 0xa9d583, (v11 + 3i32) as isize);
            v3 = (v3).wrapping_add(7u16);
            v8 = (v8).wrapping_add(8u16);
            v9 = crate::rtl::rom_table_word!(game, 0xa9d583, (v8 as i32 >> 1i32) as isize);
            if (v9 == 0) {
                break;
            }
        }
    } else {
        let mut v2: u16 = 0u16;
        let mut v4: u16 = crate::rtl::rom_table_word!(game, 0xa9d549, 0);
        loop {
            v5 = gVramWriteEntry(v3 as i32, game);
            (*v5).size = v4;
            let v6: i32 = v2 as i32 >> 1i32;
            *((&raw mut (*v5).src.addr as *mut u8).offset(1) as *mut VoidP) =
                crate::rtl::rom_table_word!(game, 0xa9d549, (v6 + 1i32) as isize);
            (*v5).src.addr = crate::rtl::rom_table_word!(game, 0xa9d549, (v6 + 2i32) as isize);
            (*v5).vram_dst = crate::rtl::rom_table_word!(game, 0xa9d549, (v6 + 3i32) as isize);
            v3 = (v3).wrapping_add(7u16);
            v2 = (v2).wrapping_add(8u16);
            v4 = crate::rtl::rom_table_word!(game, 0xa9d549, (v2 as i32 >> 1i32) as isize);
            if (v4 == 0) {
                break;
            }
        }
    }
    (*Get_DeadEnemy(0i32, game)).dey_var_22 = 0u16;
    set_ram_word!(game.vram_write_queue_tail = v3);
}
pub unsafe fn DeadTorizo_CorpseRottingFinished(game: &mut Game) {
    game.eproj_spawn_pt = Point16U {
        x: ((ram_word!(game, random_number) as i32 & 0x1fi32) + 272i32) as u16,
        y: 188u16,
    };
    SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 0xau16, game);
    if (ram_word!(game, random_enemy_counter) as i32 & 7i32) == 0i32 {
        QueueSfx2_Max6(0x10u16, game);
    }
}
pub unsafe fn DeadTorizo_CopyLineOfSandHeapTileData(a: u16, game: &mut Game) {
    let v1: u16 = crate::rtl::rom_table_word!(game, 0xa9d67c, a as isize);
    let v2: i32 = crate::rtl::rom_table_word!(game, 0xa9d69c, a as isize) as i32 >> 1i32;
    *(ram_ptr!(game, u16; 0x9500, v1 as i32 as isize)) =
        crate::rtl::rom_table_word!(game, 0xb7a800, v2 as isize);
    *(ram_ptr!(game, u16; 0x9510, v1 as i32 as isize)) =
        crate::rtl::rom_table_word!(game, 0xb7a800, (v2 + 8i32) as isize);
    *(ram_ptr!(game, u16; 0x9520, v1 as i32 as isize)) =
        crate::rtl::rom_table_word!(game, 0xb7a800, (v2 + 16i32) as isize);
    *(ram_ptr!(game, u16; 0x9530, v1 as i32 as isize)) =
        crate::rtl::rom_table_word!(game, 0xb7a800, (v2 + 24i32) as isize);
    *(ram_ptr!(game, u16; 0x9540, v1 as i32 as isize)) =
        crate::rtl::rom_table_word!(game, 0xb7a800, (v2 + 32i32) as isize);
    *(ram_ptr!(game, u16; 0x9550, v1 as i32 as isize)) =
        crate::rtl::rom_table_word!(game, 0xb7a800, (v2 + 40i32) as isize);
    *(ram_ptr!(game, u16; 0x9560, v1 as i32 as isize)) =
        crate::rtl::rom_table_word!(game, 0xb7a800, (v2 + 48i32) as isize);
    *(ram_ptr!(game, u16; 0x9570, v1 as i32 as isize)) =
        crate::rtl::rom_table_word!(game, 0xb7a800, (v2 + 56i32) as isize);
    *(ram_ptr!(game, u16; 0x9580, v1 as i32 as isize)) =
        crate::rtl::rom_table_word!(game, 0xb7a800, (v2 + 64i32) as isize);
    *(ram_ptr!(game, u16; 0x9590, v1 as i32 as isize)) =
        crate::rtl::rom_table_word!(game, 0xb7a800, (v2 + 72i32) as isize);
    *(ram_ptr!(game, u16; 0x95a0, v1 as i32 as isize)) =
        crate::rtl::rom_table_word!(game, 0xb7a800, (v2 + 80i32) as isize);
    *(ram_ptr!(game, u16; 0x95b0, v1 as i32 as isize)) =
        crate::rtl::rom_table_word!(game, 0xb7a800, (v2 + 88i32) as isize);
    *(ram_ptr!(game, u16; 0x95c0, v1 as i32 as isize)) =
        crate::rtl::rom_table_word!(game, 0xb7a800, (v2 + 96i32) as isize);
    *(ram_ptr!(game, u16; 0x95d0, v1 as i32 as isize)) =
        crate::rtl::rom_table_word!(game, 0xb7a800, (v2 + 104i32) as isize);
    *(ram_ptr!(game, u16; 0x95e0, v1 as i32 as isize)) =
        crate::rtl::rom_table_word!(game, 0xb7a800, (v2 + 112i32) as isize);
    *(ram_ptr!(game, u16; 0x95f0, v1 as i32 as isize)) =
        crate::rtl::rom_table_word!(game, 0xb7a800, (v2 + 120i32) as isize);
    *(ram_ptr!(game, u16; 0x9600, v1 as i32 as isize)) =
        crate::rtl::rom_table_word!(game, 0xb7a800, (v2 + 128i32) as isize);
    *(ram_ptr!(game, u16; 0x9610, v1 as i32 as isize)) =
        crate::rtl::rom_table_word!(game, 0xb7a800, (v2 + 136i32) as isize);
}
pub unsafe fn DeadSidehopper_Init(game: &mut Game) {
    let dsr_parameter_1: u16 =
        (*Get_DeadSidehopper(ram_word!(game, cur_enemy_index) as i32, game)).dsr_parameter_1;
    if dsr_parameter_1 != 0 {
        if dsr_parameter_1 != 2u16 {
            Unreachable(game);
        }
        DeadSidehopper_Init_1(game);
    } else {
        DeadSidehopper_Init_0(game);
    };
}
pub unsafe fn DeadSidehopper_Init_0(game: &mut Game) {
    let E: *mut Enemy_DeadSidehopper =
        Get_DeadSidehopper(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.properties =
        ((*E).base.properties as i32 & 0x77ffi32 | kEnemyProps_ProcessedOffscreen as i32) as u16;
    if ((*Get_DeadSidehopper(0i32, game)).base.properties as i32 & kEnemyProps_Invisible as i32)
        != 0i32
    {
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Deleted) as u16);
    }
    (*E).dsr_var_08 = 0u16;
    (*E).dsr_var_0A = 96u16;
    (*E).dsr_var_0B = 256u16;
    (*E).base.x_pos = 488u16;
    (*E).base.y_pos = 184u16;
    (*E).dsr_var_A = ((0xa9d8e2i32) as u16);
    (*E).base.palette_index = 512u16;
    (*E).base.y_height = 21u16;
    Enemy_SetInstrList(
        ram_word!(game, cur_enemy_index),
        addr_kDeadMonsters_Ilist_ECE3 as u16,
        game,
    );
    InitializeEnemyCorpseRotting(
        ram_word!(game, cur_enemy_index),
        addr_stru_A9DD68 as u16,
        game,
    );
}
pub unsafe fn DeadSidehopper_Init_1(game: &mut Game) {
    let E: *mut Enemy_DeadSidehopper =
        Get_DeadSidehopper(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).dsr_var_08 = -1i32 as u16;
    (*E).dsr_var_A = ((0xa9da64i32) as u16);
    (*E).base.palette_index = 3584u16;
    Enemy_SetInstrList(
        ram_word!(game, cur_enemy_index),
        addr_kDeadMonsters_Ilist_ECEF as u16,
        game,
    );
    InitializeEnemyCorpseRotting(
        ram_word!(game, cur_enemy_index),
        addr_stru_A9DD78 as u16,
        game,
    );
}
pub unsafe fn DeadZoomer_Init(game: &mut Game) {
    let v1: *mut EnemyData = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
    (*v1).palette_index = 3584u16;
    (*v1).ai_var_A = ((0xa9da69i32) as u16);
    let v2: i32 = (*v1).parameter_1 as i32 >> 1i32;
    Enemy_SetInstrList(
        ram_word!(game, cur_enemy_index),
        crate::rtl::rom_table_word!(game, 0xa9d86a, v2 as isize),
        game,
    );
    InitializeEnemyCorpseRotting(
        ram_word!(game, cur_enemy_index),
        crate::rtl::rom_table_word!(game, 0xa9d870, v2 as isize),
        game,
    );
}
pub unsafe fn DeadRipper_Init(game: &mut Game) {
    let v1: *mut EnemyData = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
    (*v1).palette_index = 3584u16;
    (*v1).ai_var_A = ((0xa9da73i32) as u16);
    let v2: i32 = (*v1).parameter_1 as i32 >> 1i32;
    Enemy_SetInstrList(
        ram_word!(game, cur_enemy_index),
        crate::rtl::rom_table_word!(game, 0xa9d897, v2 as isize),
        game,
    );
    InitializeEnemyCorpseRotting(
        ram_word!(game, cur_enemy_index),
        crate::rtl::rom_table_word!(game, 0xa9d89b, v2 as isize),
        game,
    );
}
pub unsafe fn DeadSkree_Init(game: &mut Game) {
    let v1: *mut EnemyData = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
    (*v1).palette_index = 3584u16;
    (*v1).ai_var_A = ((0xa9da6ei32) as u16);
    let v2: i32 = (*v1).parameter_1 as i32 >> 1i32;
    Enemy_SetInstrList(
        ram_word!(game, cur_enemy_index),
        crate::rtl::rom_table_word!(game, 0xa9d8c0, v2 as isize),
        game,
    );
    InitializeEnemyCorpseRotting(
        ram_word!(game, cur_enemy_index),
        crate::rtl::rom_table_word!(game, 0xa9d8c6, v2 as isize),
        game,
    );
}
pub unsafe fn DeadSidehopper_Powerbomb(game: &mut Game) {
    if (*Get_DeadSidehopper(ram_word!(game, cur_enemy_index) as i32, game)).dsr_var_08 >= 8u16 {
        DeadSidehopper_Shot(game);
    } else {
        DeadSidehopper_Main(game);
    };
}
crate::game::address_dispatch! {
    pub unsafe fn CallDeadSidehopperFunc(ea: u32, k: u16, game: &mut Game) => ea, game {
        11131121 => DeadSidehopper_Activated(k, game),
        11131106 => DeadSidehopper_Alive_WaitForActivate(k, game),
        11131492 => DeadSidehopper_WaitForSamusColl(k, game),
        11131497 => DeadZoomer_WaitForSamusColl(k, game),
        11131502 => DeadSkree_WaitForSamusColl(k, game),
        11131507 => DeadRipper_WaitForSamusColl(k, game),
        11131535 => DeadSidehopper_PreRotDelay(k, game),
        11131540 => DeadZoomer_PreRotDelay(k, game),
        11131550 => DeadSkree_PreRotDelay(k, game),
        11131545 => DeadRipper_PreRotDelay(k, game),
        11131578 => DeadSidehopper_Rotting(k, game),
        11131600 => DeadZoomer_Rotting(k, game),
        11131644 => DeadSkree_Rotting(k, game),
        11131622 => DeadRipper_Rotting(k, game),
        11131152 => DeadMonsters_Func_1(k, game),
        11131165 => DeadMonsters_Func_2(k, game),
        11131400 => DeadMonsters_Func_5(k, game),
        11131151 => (),
        11131491 => (),
    }
}
pub unsafe fn DeadSidehopper_Main(game: &mut Game) {
    let E: *mut Enemy_DeadSidehopper =
        Get_DeadSidehopper(ram_word!(game, cur_enemy_index) as i32, game);
    CallDeadSidehopperFunc(
        (((*E).dsr_var_A) as u32) | (0xa90000u32),
        ram_word!(game, cur_enemy_index),
        game,
    );
}
pub unsafe fn DeadSidehopper_Alive_WaitForActivate(k: u16, game: &mut Game) {
    if ram_word!(game, layer1_x_pos) as i32 - 513i32 & 0x8000i32 != 0 {
        (*Get_DeadSidehopper(k as i32, game)).dsr_var_A = ((0xa9d8f1i32) as u16);
        DeadSidehopper_Activated(k, game);
    }
}
pub unsafe fn DeadSidehopper_Activated(k: u16, game: &mut Game) {
    if DeadMonsters_Func_3(k, game) as i32 & 1i32 != 0 {
        let E: *mut Enemy_DeadSidehopper = Get_DeadSidehopper(k as i32, game);
        (*E).dsr_var_06 = ((*E).dsr_var_06 as i32 + 1i32 & 3i32) as u16;
        Enemy_SetInstrList(k, addr_kDeadMonsters_Ilist_ECAC as u16, game);
        (*E).dsr_var_A = ((0xa9d90fi32) as u16);
    }
}
pub unsafe fn DeadMonsters_Func_1(k: u16, game: &mut Game) {
    let E: *mut Enemy_DeadMonsters = Get_DeadMonsters(k as i32, game);
    (*E).dms_var_A = ((0xa9d91di32) as u16);
    (*E).dms_var_F = 64u16;
}
pub unsafe fn DeadMonsters_Func_2(k: u16, game: &mut Game) {
    let E: *mut Enemy_DeadMonsters = Get_DeadMonsters(k as i32, game);
    (*E).dms_var_F = (*E).dms_var_F.wrapping_sub(1);
    if ((*E).dms_var_F as i32 & 0x8000i32) != 0i32 {
        if (*E).dms_var_08 != 0 {
            (*E).dms_var_A = ((0xa9da08i32) as u16);
        } else {
            (*E).dms_var_A = ((0xa9d8f1i32) as u16);
            Enemy_SetInstrList(k, addr_kDeadMonsters_Ilist_ECE3 as u16, game);
            let v3: i32 = (*E).dms_var_06 as i32;
            (*E).dms_var_0B = crate::rtl::rom_table_word!(game, 0xa9d951, v3 as isize);
            (*E).dms_var_0A = crate::rtl::rom_table_word!(game, 0xa9d959, v3 as isize);
        }
    }
}
pub unsafe fn DeadMonsters_Func_3(k: u16, game: &mut Game) -> u8 {
    let E: *mut Enemy_DeadMonsters = Get_DeadMonsters(k as i32, game);
    DeadMonsters_Func_4(k, (*E).dms_var_0A, game);
    let mut v2: u16 = 32u16;
    if ((*E).dms_var_0B as i32 & 0x8000i32) == 0i32 {
        v2 = 128u16;
    }
    let mut v3: u16 = ((*E).dms_var_0B).wrapping_add(v2);
    (*E).dms_var_0B = v3;
    if (*E).base.x_pos as i32 - 544i32 & 0x8000i32 != 0 {
        let full: i32 = *(&raw mut (*E).base.y_subpos as *mut u8).offset(1isize) as i32
            + *(&raw mut v3 as *mut u8).offset(0) as i32;
        *(&raw mut (*E).base.y_subpos as *mut u8).offset(1isize) = full as u8;
        let v12: u16 =
            ((*E).base.y_pos as i32 + (v3 as i32 >> 8i32) as i8 as i32 + (full >> 8i32)) as u16;
        (*E).base.y_pos = v12;
        ((v12 as i32 - 184i32 & 0x8000i32) == 0i32) as u8
    } else {
        Enemy_MoveDown(k, (v3 as i16 as i32) << 8i32, game)
    }
}
pub unsafe fn DeadMonsters_Func_4(k: u16, mut a: u16, game: &mut Game) {
    let E: *mut Enemy_DeadMonsters = Get_DeadMonsters(k as i32, game);
    if (*E).base.x_pos as i32 - 544i32 & 0x8000i32 != 0 {
        let full: i32 = *(&raw mut (*E).base.x_subpos as *mut u8).offset(1isize) as i32
            + *(&raw mut a as *mut u8).offset(0) as i32;
        *(&raw mut (*E).base.x_subpos as *mut u8).offset(1isize) = full as u8;
        (*E).base.x_pos =
            ((*E).base.x_pos as i32 + ((a as i32 >> 8i32) as i8 as i32 + (full >> 8i32))) as u16;
    } else {
        Enemy_MoveRight_IgnoreSlopes(k, (a as i16 as i32) << 8i32, game);
    };
}
pub unsafe fn DeadMonsters_Func_5(k: u16, game: &mut Game) {
    let E: *mut Enemy_DeadMonsters = Get_DeadMonsters(k as i32, game);
    let v2: u16 = ((*E).dms_var_07).wrapping_add(1u16);
    (*E).dms_var_07 = v2;
    if v2 >= 8u16 {
        (*E).dms_var_07 = 0u16;
        WriteColorsToPalette(
            0x122u16,
            0xa9u8,
            (32i32 * ((*E).dms_var_08 as i32 - 1i32) - 0x1434i32) as u16,
            0xfu16,
            game,
        );
        let v3: u16 = ram_word!(game, cur_enemy_index);
        let v5: u16 = ((*E).dms_var_08).wrapping_add(1u16);
        (*E).dms_var_08 = v5;
        if v5 >= 8u16 {
            Enemy_SetInstrList(v3, addr_kDeadMonsters_Ilist_ECE9 as u16, game);
            (*E).dms_var_A = ((0xa9da64i32) as u16);
            (*E).base.properties = ((*E).base.properties) | (0x8000u16);
            (*E).base.y_height = 12u16;
        }
    }
}
pub unsafe fn DeadSidehopper_WaitForSamusColl(k: u16, game: &mut Game) {
    DeadMonsters_WaitForSamusColl(k, ((0xa9da8fi32) as u16), game);
}
pub unsafe fn DeadZoomer_WaitForSamusColl(k: u16, game: &mut Game) {
    DeadMonsters_WaitForSamusColl(k, ((0xa9da94i32) as u16), game);
}
pub unsafe fn DeadSkree_WaitForSamusColl(k: u16, game: &mut Game) {
    DeadMonsters_WaitForSamusColl(k, ((0xa9da9ei32) as u16), game);
}
pub unsafe fn DeadRipper_WaitForSamusColl(k: u16, game: &mut Game) {
    DeadMonsters_WaitForSamusColl(k, ((0xa9da99i32) as u16), game);
}
pub unsafe fn DeadMonsters_WaitForSamusColl(k: u16, j: u16, game: &mut Game) {
    if k == ram_word!(game, enemy_index_colliding_dirs[0])
        || k == ram_word!(game, enemy_index_colliding_dirs[1])
        || k == ram_word!(game, enemy_index_colliding_dirs[2])
        || k == ram_word!(game, enemy_index_colliding_dirs[3])
    {
        (*Get_DeadMonsters(k as i32, game)).dms_var_A = j;
    }
}
pub unsafe fn DeadSidehopper_PreRotDelay(k: u16, game: &mut Game) {
    DeadMonsters_PreRotDelay_Common(k, ((0xa9dabai32) as u16), game);
}
pub unsafe fn DeadZoomer_PreRotDelay(k: u16, game: &mut Game) {
    DeadMonsters_PreRotDelay_Common(k, ((0xa9dad0i32) as u16), game);
}
pub unsafe fn DeadRipper_PreRotDelay(k: u16, game: &mut Game) {
    DeadMonsters_PreRotDelay_Common(k, ((0xa9dae6i32) as u16), game);
}
pub unsafe fn DeadSkree_PreRotDelay(k: u16, game: &mut Game) {
    DeadMonsters_PreRotDelay_Common(k, ((0xa9dafci32) as u16), game);
}
pub unsafe fn DeadMonsters_PreRotDelay_Common(k: u16, j: u16, game: &mut Game) {
    let E: *mut Enemy_DeadMonsters = Get_DeadMonsters(k as i32, game);
    (*E).dms_var_B = (*E).dms_var_B.wrapping_add(1);
    if (*E).dms_var_B >= 0x10u16 {
        (*E).dms_var_A = j;
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
    }
}
pub unsafe fn DeadSidehopper_Rotting(k: u16, game: &mut Game) {
    let v1: u8 = ProcessCorpseRotting(k, game);
    if v1 as i32 & 1i32 == 0 {
        (*Get_DeadMonsters(ram_word!(game, cur_enemy_index) as i32, game)).dms_var_A =
            ((0xa9da64i32) as u16);
    }
    let dms_var_53: u16 =
        (*Get_DeadMonsters(ram_word!(game, cur_enemy_index) as i32, game)).dms_var_53;
    ProcessCorpseRottingVramTransfers(dms_var_53, game);
}
pub unsafe fn DeadZoomer_Rotting(k: u16, game: &mut Game) {
    let v1: u8 = ProcessCorpseRotting(k, game);
    if v1 as i32 & 1i32 == 0 {
        (*Get_DeadMonsters(ram_word!(game, cur_enemy_index) as i32, game)).dms_var_A =
            ((0xa9da63i32) as u16);
    }
    let dms_var_53: u16 =
        (*Get_DeadMonsters(ram_word!(game, cur_enemy_index) as i32, game)).dms_var_53;
    ProcessCorpseRottingVramTransfers(dms_var_53, game);
}
pub unsafe fn DeadRipper_Rotting(k: u16, game: &mut Game) {
    let v1: u8 = ProcessCorpseRotting(k, game);
    if v1 as i32 & 1i32 == 0 {
        (*Get_DeadMonsters(ram_word!(game, cur_enemy_index) as i32, game)).dms_var_A =
            ((0xa9da63i32) as u16);
    }
    let dms_var_53: u16 =
        (*Get_DeadMonsters(ram_word!(game, cur_enemy_index) as i32, game)).dms_var_53;
    ProcessCorpseRottingVramTransfers(dms_var_53, game);
}
pub unsafe fn DeadSkree_Rotting(k: u16, game: &mut Game) {
    let v1: u8 = ProcessCorpseRotting(k, game);
    if v1 as i32 & 1i32 == 0 {
        (*Get_DeadMonsters(ram_word!(game, cur_enemy_index) as i32, game)).dms_var_A =
            ((0xa9da63i32) as u16);
    }
    let dms_var_53: u16 =
        (*Get_DeadMonsters(ram_word!(game, cur_enemy_index) as i32, game)).dms_var_53;
    ProcessCorpseRottingVramTransfers(dms_var_53, game);
}
pub unsafe fn ProcessCorpseRotting(k: u16, game: &mut Game) -> u8 {
    let mut E: *mut Enemy_DeadMonsters = null_mut::<Enemy_DeadMonsters>();
    let mut r18_0: u16 = 0;
    let mut v11: *mut u16 = null_mut::<u16>();
    let mut v12: u16 = 0;
    let mut v5: *mut i16 = null_mut::<i16>();
    let mut v7: i16 = 0;
    let mut v8: i16 = 0;
    let EK: *mut Enemy_DeadMonsters = Get_DeadMonsters(k as i32, game);
    let mut E0: *mut Enemy_DeadMonsters = Get_DeadMonsters(0i32, game);
    (*E0).dms_var_45 = (*EK).dms_var_57;
    (*E0).dms_var_46 = (*EK).dms_var_58;
    (*E0).dms_var_47 = (*EK).dms_var_59;
    (*E0).dms_var_48 = (*EK).dms_var_5A;
    (*E0).dms_var_49 = (*EK).dms_var_5B;
    (*E0).dms_var_44 = (*EK).dms_var_56;
    (*E0).dms_var_42 = (*EK).dms_var_54;
    (*E0).dms_var_43 = (*EK).dms_var_55;
    let mut dms_var_52: u16 = (*EK).dms_var_52;
    let mut v4: u16 = 0u16;
    let mut v15: u16 = 0;
    let mut v6: *mut u16 = null_mut::<u16>();
    loop {
        v15 = v4;
        v5 = ram_ptr!(game, i16; dms_var_52 as i32 as isize);
        v6 = v5 as *mut u16;
        if *v5 >= 0i16 {
            v7 = *v5.offset(1);
            if v7 != 0 {
                v8 = (v7).wrapping_sub(1i16);
                *v5.offset(1) = v8;
                if (v8 as u16) < 4u16 {
                    E0 = Get_DeadMonsters(0i32, game);
                    let mut r18: u16 = (*E0).dms_var_42;
                    if v15 >= (*E0).dms_var_47 {
                        r18 = (*E0).dms_var_43;
                    }
                    CopyMoveCorpseRottingRotEntry(
                        RomPtr_A9((*E0).dms_var_44, game) as *const u16,
                        *v6,
                        r18,
                        game,
                    );
                }
            } else {
                E = Get_DeadMonsters(0i32, game);
                r18_0 = (*E).dms_var_43;
                v11 = ram_ptr!(game, u16; dms_var_52 as i32 as isize);
                CopyMoveCorpseRottingRotEntry(
                    RomPtr_A9((*E).dms_var_44, game) as *const u16,
                    *v11,
                    r18_0,
                    game,
                );
                v12 = (*v11).wrapping_add(2u16);
                if v12 < (*E).dms_var_46 {
                    *v11 = v12;
                } else {
                    CallCorpseRottingFinish((((*E).dms_var_49) as u32) | (0xa90000u32), game);
                    if (v15 >= (*Get_DeadMonsters(0i32, game)).dms_var_46) {
                        break;
                    }
                    *v11 = -1i32 as u16;
                }
            }
        }
        dms_var_52 = (dms_var_52).wrapping_add(4u16);
        v4 = (v15).wrapping_add(1u16);
        E0 = Get_DeadMonsters(0i32, game);
        if ((v15 as i32 + 1i32 - (*E0).dms_var_45 as i32) as i16) >= 0i16 {
            return ((v15).wrapping_add(1u16) >= (*E0).dms_var_45) as u8;
        }
    }
    0u8
}
pub unsafe fn CopyMoveCorpseRottingRotEntry(r20: *const u16, a: u16, r18: u16, game: &mut Game) {
    let E: *mut Enemy_DeadMonsters = Get_DeadMonsters(0i32, game);
    (*E).dms_var_41 = a;
    let v1: u16 = (((a) & (0xfff8u16)) as i32 >> 2i32) as u16;
    let v2: u16 = (a) & (7u16);
    let v6: u16 = (*r20.offset((v1 as i32 >> 1i32) as isize) as i32 + 2i32 * v2 as i32) as u16;
    if v2 >= 6u16 {
        CallCorpseRottingMove(
            ((r18) as u32) | (0xa90000u32),
            ((*E).dms_var_48).wrapping_add(v6),
            v6,
            game,
        );
    } else {
        CallCorpseRottingMove(((r18) as u32) | (0xa90000u32), v6, v6, game);
    };
}
pub unsafe fn CorpseRottingRotEntryFinishedHook(game: &mut Game) {
    let v0: *mut EnemyData = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
    game.eproj_spawn_pt = Point16U {
        x: ((*v0).x_pos as i32 + (ram_word!(game, random_number) as i32 & 0x1ai32) - 14i32) as u16,
        y: (((*v0).y_pos).wrapping_add(16u16)),
    };
    SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 0xau16, game);
    if (ram_word!(game, random_enemy_counter) as i32 & 7i32) == 0i32 {
        QueueSfx2_Max3(0x10u16, game);
    }
}
pub unsafe fn InitializeCorpseRottingDataTable(mut table: *mut u16, a: u16) {
    let mut v2: i16 = 0;
    v2 = ((a) as i16).wrapping_sub(1i16);
    let mut r18: u16 = 0u16;
    loop {
        *table.offset(0) = v2 as u16;
        *table.offset(1) = r18;
        table = table.offset(2);
        r18 = (r18).wrapping_add(2u16);
        v2 -= 1;
        if (v2 < 0i16) {
            break;
        }
    }
}
pub unsafe fn InitializeEnemyCorpseRotting(k: u16, j: u16, game: &mut Game) {
    let v2: *const u8 = RomPtr_A9(j, game);
    let E: *mut Enemy_DeadMonsters = Get_DeadMonsters(k as i32, game);
    (*E).dms_var_52 = *(v2 as *mut u16);
    (*E).dms_var_53 = *(v2.offset(2) as *mut u16);
    (*E).dms_var_54 = *(v2.offset(4) as *mut u16);
    (*E).dms_var_55 = *(v2.offset(6) as *mut u16);
    (*E).dms_var_5B = *(v2.offset(14) as *mut u16);
    let v4: u16 = *(v2.offset(12) as *mut u16);
    (*E).dms_var_56 = v4;
    (*E).dms_var_5A = (*(RomPtr_A9(v4, game) as *mut u16).offset(1)).wrapping_sub(12u16);
    let v5: *mut u16 = v2 as *mut u16;
    let mut v6: u16 = *v5.offset(4);
    let fresh5 = v6;
    v6 = v6.wrapping_sub(1);
    (*E).dms_var_57 = fresh5;
    (*E).dms_var_58 = v6;
    (*E).dms_var_59 = (v6).wrapping_sub(1u16);
    InitializeCorpseRottingDataTable(ram_ptr!(game, u16; *v5 as i32 as isize), *v5.offset(4));
    CallCorpseRottingInit(((*v5.offset(5)) as u32) | (0xa90000u32), game);
}
pub unsafe fn ProcessCorpseRottingVramTransfers(k: u16, game: &mut Game) {
    let mut p: *const u8 = RomPtr_A9(k, game);
    let mut v1: u16 = ram_word!(game, vram_write_queue_tail);
    let mut v2: u16 = *(p as *mut u16);
    loop {
        let v3: *mut VramWriteEntry = gVramWriteEntry(v1 as i32, game);
        (*v3).size = v2;
        *((&raw mut (*v3).src.addr as *mut u8).offset(1) as *mut VoidP) =
            *(p as *mut u16).offset(1);
        (*v3).src.addr = *(p as *mut u16).offset(2);
        (*v3).vram_dst = *(p as *mut u16).offset(3);
        v1 = (v1).wrapping_add(7u16);
        p = p.offset(8);
        v2 = *(p as *mut u16);
        if (v2 == 0) {
            break;
        }
    }
    (*Get_DeadEnemy(0i32, game)).dey_var_22 = 0u16;
    set_ram_word!(game.vram_write_queue_tail = v1);
}
pub unsafe fn DeadZoomer_Powerbomb(game: &mut Game) {
    if ((*gEnemyData(ram_word!(game, cur_enemy_index) as i32, game)).properties as i32
        & kEnemyProps_Tangible as i32)
        == 0i32
    {
        DeadZoomer_Shot(game);
    }
}
pub unsafe fn DeadZoomer_Shot(game: &mut Game) {
    DeadSidehopper_DD34(((0xa9dad0i32) as u16), game);
}
pub unsafe fn DeadRipper_Powerbomb(game: &mut Game) {
    if ((*gEnemyData(ram_word!(game, cur_enemy_index) as i32, game)).properties as i32
        & kEnemyProps_Tangible as i32)
        == 0i32
    {
        DeadRipper_Shot(game);
    }
}
pub unsafe fn DeadRipper_Shot(game: &mut Game) {
    DeadSidehopper_DD34(((0xa9dae6i32) as u16), game);
}
pub unsafe fn DeadSkree_Powerbomb(game: &mut Game) {
    if ((*gEnemyData(ram_word!(game, cur_enemy_index) as i32, game)).properties as i32
        & kEnemyProps_Tangible as i32)
        == 0i32
    {
        DeadSkree_Shot(game);
    }
}
pub unsafe fn DeadSkree_Shot(game: &mut Game) {
    DeadSidehopper_DD34(((0xa9dafci32) as u16), game);
}
pub unsafe fn DeadSidehopper_Shot(game: &mut Game) {
    let E: *mut Enemy_DeadSidehopper =
        Get_DeadSidehopper(ram_word!(game, cur_enemy_index) as i32, game);
    if !(((*E).base.properties as i32 & kEnemyProps_Tangible as i32) != 0i32
        || (*E).dsr_var_08 < 8u16)
    {
        DeadSidehopper_DD31(game);
    }
}
pub unsafe fn DeadSidehopper_DD31(game: &mut Game) {
    DeadSidehopper_DD34(((0xa9dabai32) as u16), game);
}
pub unsafe fn DeadSidehopper_DD34(a: u16, game: &mut Game) {
    let E: *mut Enemy_DeadSidehopper =
        Get_DeadSidehopper(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).dsr_var_A = a;
    (*E).base.properties = ((*E).base.properties as i32
        | (kEnemyProps_ProcessedOffscreen as i32 | kEnemyProps_Tangible as i32))
        as u16;
}
pub unsafe fn DeadSidehopper_Touch(game: &mut Game) {
    if (*Get_DeadSidehopper(ram_word!(game, cur_enemy_index) as i32, game)).dsr_var_08 < 8u16 {
        NormalEnemyTouchAiSkipDeathAnim_CurEnemy(game);
    } else {
        DeadSidehopper_DD31(game);
    };
}
unsafe fn initialize_corpse_tiles(
    rom_address: u16,
    tile_base: usize,
    segments: &[(usize, usize, i32)],
    game: &mut Game,
) {
    let source = RomPtr_B7(rom_address, game);
    let destination = (&raw mut game.g_ram as *mut u8).add(tile_base);
    for &(destination_offset, source_offset, length) in segments {
        MemCpy(
            destination.add(destination_offset).cast(),
            source.add(source_offset).cast(),
            length,
        );
    }
}

pub unsafe fn Torizo_CorpseRottingInitFunc(game: &mut Game) {
    initialize_corpse_tiles(
        addr_kDeadTorizo_TileData as u16,
        0x2000,
        &[
            (0x60, 0x120, 0xc0),
            (0x1a0, 0x320, 0xc0),
            (0x2c0, 0x500, 0x100),
            (0x400, 0x700, 0x100),
            (0x540, 0x900, 0x100),
            (0x680, 0xb00, 0x100),
            (0x7c0, 0xd00, 0x100),
            (0x900, 0xf00, 0x100),
            (0xa40, 0x1100, 0x100),
            (0xb60, 0x12e0, 0x120),
            (0xc80, 0x14c0, 0x140),
            (0xdc0, 0x16c0, 0x140),
        ],
        game,
    );
}
pub unsafe fn Sidehopper_CorpseRottingInitFunc_0(game: &mut Game) {
    initialize_corpse_tiles(
        addr_kDeadMonsters_TileData as u16,
        0x2000,
        &[
            (0x40, 0x40, 0x60),
            (0xa0, 0x200, 0xa0),
            (0x140, 0x400, 0xa0),
            (0x1e0, 0x600, 0xa0),
            (0x280, 0x800, 0xa0),
        ],
        game,
    );
}
pub unsafe fn Sidehopper_CorpseRottingInitFunc_2(game: &mut Game) {
    initialize_corpse_tiles(
        addr_kDeadMonsters_TileData as u16,
        0x2000,
        &[
            (0x320, 0x120, 0x40),
            (0x3c0, 0x320, 0xa0),
            (0x460, 0x520, 0xa0),
            (0x500, 0x720, 0xa0),
            (0x5a0, 0x920, 0xa0),
        ],
        game,
    );
}
pub unsafe fn Zoomer_CorpseRottingInitFunc_0(game: &mut Game) {
    initialize_corpse_tiles(
        addr_kDeadMonsters_TileData as u16,
        0x2000,
        &[(0x940, 0xa60, 0x60), (0x9a0, 0xc60, 0x60)],
        game,
    );
}
pub unsafe fn Zoomer_CorpseRottingInitFunc_2(game: &mut Game) {
    initialize_corpse_tiles(
        addr_kDeadMonsters_TileData as u16,
        0x2000,
        &[(0xa00, 0xac0, 0x60), (0xa60, 0xcc0, 0x60)],
        game,
    );
}
pub unsafe fn Zoomer_CorpseRottingInitFunc_4(game: &mut Game) {
    initialize_corpse_tiles(
        addr_kDeadMonsters_TileData as u16,
        0x2000,
        &[(0xac0, 0xb20, 0x60), (0xb20, 0xd20, 0x60)],
        game,
    );
}
pub unsafe fn Ripper_CorpseRottingInitFunc_0(game: &mut Game) {
    initialize_corpse_tiles(
        addr_kDeadMonsters_TileData as u16,
        0x2000,
        &[(0xb80, 0xa00, 0x60), (0xbe0, 0xc00, 0x60)],
        game,
    );
}
pub unsafe fn Ripper_CorpseRottingInitFunc_2(game: &mut Game) {
    initialize_corpse_tiles(
        addr_kDeadMonsters_TileData as u16,
        0x2000,
        &[(0xc40, 0xb80, 0x60), (0xca0, 0xd80, 0x60)],
        game,
    );
}
pub unsafe fn Skree_CorpseRottingInitFunc_0(game: &mut Game) {
    initialize_corpse_tiles(
        addr_kDeadMonsters_TileData as u16,
        0x2000,
        &[
            (0x640, 0x2a0, 0x40),
            (0x680, 0x4a0, 0x40),
            (0x6c0, 0x6a0, 0x40),
            (0x700, 0x8a0, 0x40),
        ],
        game,
    );
}
pub unsafe fn Skree_CorpseRottingInitFunc_2(game: &mut Game) {
    initialize_corpse_tiles(
        addr_kDeadMonsters_TileData as u16,
        0x2000,
        &[
            (0x740, 0xe0, 0x40),
            (0x780, 0x2e0, 0x40),
            (0x7c0, 0x4e0, 0x40),
            (0x800, 0x6e0, 0x40),
        ],
        game,
    );
}
pub unsafe fn Skree_CorpseRottingInitFunc_4(game: &mut Game) {
    initialize_corpse_tiles(
        addr_kDeadMonsters_TileData as u16,
        0x2000,
        &[
            (0x840, 0x1c0, 0x40),
            (0x880, 0x3c0, 0x40),
            (0x8c0, 0x5c0, 0x40),
            (0x900, 0x7c0, 0x40),
        ],
        game,
    );
}
pub unsafe fn MotherBrain_CorpseRottingInitFunc(game: &mut Game) {
    initialize_corpse_tiles(
        addr_kMotherBrain_Misc_TileData as u16,
        0x9000,
        &[
            (0x0, 0xc0, 0xc0),
            (0xe0, 0x2c0, 0xc0),
            (0x1c0, 0x4c0, 0xc0),
            (0x2a0, 0x6c0, 0xc0),
            (0x380, 0x8c0, 0xe0),
            (0x460, 0xac0, 0xe0),
        ],
        game,
    );
}
// Each descriptor lists row gates in transfer order. A move copies each row
// before clearing its source; a copy leaves the source intact.
fn transfer_corpse_tiles(
    j: u16,
    k: u16,
    tile_base: usize,
    copy_limit: u16,
    row_gates: &[u16],
    clear_source: bool,
    byte_indices: bool,
    game: &mut Game,
) {
    let destination = if byte_indices {
        j as usize
    } else {
        (j as usize >> 1) * 2
    };
    let source = if byte_indices {
        k as usize
    } else {
        (k as usize >> 1) * 2
    };
    let ram = &raw mut game.g_ram as *mut u8;
    let progress_offset = 0xf78 + core::mem::offset_of!(Enemy_DeadMonsters, dms_var_41);
    // All u16 indices and descriptor offsets fit within g_ram. Raw accesses
    // avoid creating references across potentially overlapping tile/state words.
    unsafe {
        for (row, &minimum) in row_gates.iter().enumerate() {
            let progress = || u16::from_le(ram.add(progress_offset).cast::<u16>().read_unaligned());
            if minimum != 0 && progress() < minimum {
                continue;
            }
            let row_base = tile_base + row * 32;
            // Preserve the original bit-15 subtraction test, including high values.
            if (progress() as i32 - copy_limit as i32) & 0x8000 != 0 {
                for plane in [0, 16] {
                    let value = ram
                        .add(row_base + source + plane)
                        .cast::<u16>()
                        .read_unaligned();
                    ram.add(row_base + destination + plane + 2)
                        .cast::<u16>()
                        .write_unaligned(value);
                }
            }
            if clear_source {
                for plane in [0, 16] {
                    ram.add(row_base + source + plane)
                        .cast::<u16>()
                        .write_unaligned(0);
                }
            }
        }
    }
}

pub fn Torizo_CorpseRottingMoveFunc(j: u16, k: u16, game: &mut Game) {
    transfer_corpse_tiles(
        j,
        k,
        0x2000,
        94,
        &[80, 72, 16, 0, 0, 0, 0, 0, 0, 16],
        true,
        false,
        game,
    );
}
pub fn Torizo_CorpseRottingCopyFunc(j: u16, k: u16, game: &mut Game) {
    transfer_corpse_tiles(
        j,
        k,
        0x2000,
        94,
        &[80, 72, 16, 0, 0, 0, 0, 0, 0, 16],
        false,
        false,
        game,
    );
}
pub fn Sidehopper_CorpseRottingMoveFunc_0(j: u16, k: u16, game: &mut Game) {
    transfer_corpse_tiles(j, k, 0x2000, 38, &[8, 8, 0, 0, 0], true, false, game);
}
pub fn Sidehopper_CorpseRottingCopyFunc_0(j: u16, k: u16, game: &mut Game) {
    transfer_corpse_tiles(j, k, 0x2000, 38, &[8, 8, 0, 0, 0], false, false, game);
}
pub fn Sidehopper_CorpseRottingMoveFunc_2(j: u16, k: u16, game: &mut Game) {
    transfer_corpse_tiles(j, k, 0x2320, 38, &[0, 0, 8, 8, 8], true, false, game);
}
pub fn Sidehopper_CorpseRottingCopyFunc_2(j: u16, k: u16, game: &mut Game) {
    transfer_corpse_tiles(j, k, 0x2320, 38, &[0, 0, 8, 8, 8], false, false, game);
}
pub fn Zoomer_CorpseRottingMoveFunc_0(j: u16, k: u16, game: &mut Game) {
    transfer_corpse_tiles(j, k, 0x2940, 14, &[0, 0, 0], true, false, game);
}
pub fn Zoomer_CorpseRottingCopyFunc_0(j: u16, k: u16, game: &mut Game) {
    transfer_corpse_tiles(j, k, 0x2940, 14, &[0, 0, 0], false, false, game);
}
pub fn Zoomer_CorpseRottingMoveFunc_2(j: u16, k: u16, game: &mut Game) {
    transfer_corpse_tiles(j, k, 0x2a00, 14, &[0, 0, 0], true, false, game);
}
pub fn Zoomer_CorpseRottingCopyFunc_2(j: u16, k: u16, game: &mut Game) {
    transfer_corpse_tiles(j, k, 0x2a00, 14, &[0, 0, 0], false, false, game);
}
pub fn Zoomer_CorpseRottingMoveFunc_4(j: u16, k: u16, game: &mut Game) {
    transfer_corpse_tiles(j, k, 0x2ac0, 14, &[0, 0, 0], true, false, game);
}
pub fn Zoomer_CorpseRottingCopyFunc_4(j: u16, k: u16, game: &mut Game) {
    transfer_corpse_tiles(j, k, 0x2ac0, 14, &[0, 0, 0], false, false, game);
}
pub fn Ripper_CorpseRottingMoveFunc_0(j: u16, k: u16, game: &mut Game) {
    transfer_corpse_tiles(j, k, 0x2b80, 14, &[0, 0, 0], true, false, game);
}
pub fn Ripper_CorpseRottingCopyFunc_0(j: u16, k: u16, game: &mut Game) {
    transfer_corpse_tiles(j, k, 0x2b80, 14, &[0, 0, 0], false, false, game);
}
pub fn Ripper_CorpseRottingMoveFunc_2(j: u16, k: u16, game: &mut Game) {
    transfer_corpse_tiles(j, k, 0x2c40, 14, &[0, 0, 0], true, false, game);
}
pub fn Ripper_CorpseRottingCopyFunc_2(j: u16, k: u16, game: &mut Game) {
    transfer_corpse_tiles(j, k, 0x2c40, 14, &[0, 0, 0], false, false, game);
}
pub fn Skree_CorpseRottingMoveFunc_0(j: u16, k: u16, game: &mut Game) {
    transfer_corpse_tiles(j, k, 0x2640, 30, &[0, 0], true, false, game);
}
pub fn Skree_CorpseRottingCopyFunc_0(j: u16, k: u16, game: &mut Game) {
    transfer_corpse_tiles(j, k, 0x2640, 30, &[0, 0], false, false, game);
}
pub fn Skree_CorpseRottingMoveFunc_2(j: u16, k: u16, game: &mut Game) {
    transfer_corpse_tiles(j, k, 0x2740, 30, &[0, 0], true, false, game);
}
pub fn Skree_CorpseRottingCopyFunc_2(j: u16, k: u16, game: &mut Game) {
    transfer_corpse_tiles(j, k, 0x2740, 30, &[0, 0], false, false, game);
}
pub fn Skree_CorpseRottingMoveFunc_4(j: u16, k: u16, game: &mut Game) {
    transfer_corpse_tiles(j, k, 0x2840, 30, &[0, 0], true, false, game);
}
pub fn Skree_CorpseRottingCopyFunc_4(j: u16, k: u16, game: &mut Game) {
    transfer_corpse_tiles(j, k, 0x2840, 30, &[0, 0], false, false, game);
}
pub fn MotherBrain_CorpseRottingMoveFunc(j: u16, k: u16, game: &mut Game) {
    transfer_corpse_tiles(j, k, 0x9000, 46, &[16, 8, 0, 0, 0, 8, 32], true, true, game);
}
pub fn MotherBrain_CorpseRottingCopyFunc(j: u16, k: u16, game: &mut Game) {
    transfer_corpse_tiles(
        j,
        k,
        0x9000,
        46,
        &[16, 8, 0, 0, 0, 8, 32],
        false,
        true,
        game,
    );
}
pub unsafe fn sub_A9ECD0(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let mut v2: u16 = ((0xa9d910i32) as u16);
    let E: *mut Enemy_DeadMonsters = Get_DeadMonsters(k as i32, game);
    if (*E).dms_var_08 != 0 {
        v2 = ((0xa9da08i32) as u16);
    }
    (*E).dms_var_A = v2;
    jp
}
pub unsafe fn Shitroid_Func_1(k: u16, j: u16, game: &mut Game) -> u8 {
    let E: *mut Enemy_Shitroid = Get_Shitroid(j as i32, game);
    let G: *mut Enemy_Shitroid = Get_Shitroid(k as i32, game);
    let mut r18: u16 = ((*G).base.x_width as i32 + (*E).base.x_width as i32 + 1i32) as u16;
    let v4: u16 = abs16(((*E).base.x_pos).wrapping_sub((*G).base.x_pos));
    let v5: bool = v4 >= r18;
    if v4 < r18 {
        r18 = ((*G).base.y_height as i32 + (*E).base.y_height as i32 + 1i32) as u16;
        return (abs16(((*E).base.y_pos).wrapping_sub((*G).base.y_pos)) >= r18) as u8;
    }
    v5 as u8
}
pub unsafe fn Shitroid_Func_2(k: u16, rect: Rect16U, game: &mut Game) -> u8 {
    let E: *mut Enemy_Shitroid = Get_Shitroid(k as i32, game);
    let mut R26: u16 = ((*E).base.x_width as i32 + rect.w as i32 + 1i32) as u16;
    let v2: u16 = abs16((rect.x).wrapping_sub((*E).base.x_pos));
    let v3: bool = v2 >= R26;
    if v2 < R26 {
        R26 = ((*E).base.y_height as i32 + rect.h as i32 + 1i32) as u16;
        return (abs16((rect.y).wrapping_sub((*E).base.y_pos)) >= R26) as u8;
    }
    v3 as u8
}
pub unsafe fn Shitroid_Init(game: &mut Game) {
    let mut i: i32 = 4094i32;
    while i >= 0i32 {
        set_ram_word!(game.tilemap_stuff[(i >> 1i32)] = 0u16);
        i -= 2i32;
    }
    let E: *mut Enemy_Shitroid = Get_Shitroid(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.properties = ((*E).base.properties as i32
        | (kEnemyProps_DisableSamusColl as i32 | kEnemyProps_BlockPlasmaBeam as i32))
        as u16;
    (*E).base.palette_index = 1024u16;
    (*E).base.current_instruction = addr_kShitroid_Ilist_F90E as u16;
    (*E).base.instruction_timer = 1u16;
    (*E).base.timer = 0u16;
    let mut v2: u16 = ((0xa9efe6i32) as u16);
    if (ram_word!(game, layer1_x_pos) as i32 & 0x8000i32) != 0i32 {
        (*E).base.properties = ((*E).base.properties as i32
            | (kEnemyProps_Tangible as i32 | kEnemyProps_Invisible as i32))
            as u16;
        v2 = ((0xa9efdfi32) as u16);
    }
    (*E).shitr_var_A = v2;
    (*E).shitr_var_B = 0u16;
    (*E).shitr_var_C = 0u16;
    (*E).shitr_var_E = 10u16;
    (*E).shitr_parameter_2 = 0u16;
    WriteColorsToTargetPalette(0xa9u8, 0x120u16, addr_word_A9F8C6 as u16, 0x10u16, game);
    WriteColorsToTargetPalette(
        0xa9u8,
        0x140u16,
        addr_kShitroidInCutscene_Palette as u16,
        0x10u16,
        game,
    );
    WriteColorsToTargetPalette(
        0xa9u8,
        0x1e0u16,
        addr_kDeadSidehopper_Palette_0 as u16,
        0x10u16,
        game,
    );
}
pub unsafe fn Shitroid_Powerbomb(game: &mut Game) {
    if (*Get_Shitroid(ram_word!(game, cur_enemy_index) as i32, game)).shitr_parameter_2 != 0 {
        Shitroid_Func_26(ram_word!(game, cur_enemy_index), game);
    }
    Shitroid_Main(game);
}
crate::game::address_dispatch! {
    pub unsafe fn CallShitroidFunc(ea: u32, k: u16, game: &mut Game) => ea, game {
        11136991 => Shitroid_Func_3(k, game),
        11136998 => Shitroid_Func_4(game),
        11137067 => Shitroid_Func_5(k, game),
        11137079 => Shitroid_Func_6(k, game),
        11137097 => Shitroid_Func_7(k, game),
        11137133 => Shitroid_Func_8(k, game),
        11137172 => Shitroid_Func_9(k, game),
        11137254 => Shitroid_Func_10(k, game),
        11137317 => Shitroid_Func_11(k, game),
        11137336 => Shitroid_Func_12(k, game),
        11137408 => Shitroid_Func_13(k, game),
        11137530 => Shitroid_Func_14(k, game),
        11137550 => Shitroid_Func_15(k, game),
        11137563 => Shitroid_Func_16(k, game),
        11137698 => Shitroid_Func_17(k, game),
        11137710 => Shitroid_Func_18(k, game),
        11137728 => Shitroid_Func_19(k, game),
        11137787 => Shitroid_Func_20(game),
        11137828 => Shitroid_Func_21(game),
        11137888 => Shitroid_Func_22(k, game),
        11137901 => Shitroid_Func_23(game),
        11137955 => Shitroid_Func_24(k, game),
        11137982 => Shitroid_Func_25(k, game),
    }
}
pub unsafe fn Shitroid_Main(game: &mut Game) {
    let E: *mut Enemy_Shitroid = Get_Shitroid(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.health = 0x7fffu16;
    CallShitroidFunc(
        (((*E).shitr_var_A) as u32) | (0xa90000u32),
        ram_word!(game, cur_enemy_index),
        game,
    );
    MoveEnemyWithVelocity(game);
    if ram_word!(game, palette_change_num) == 0 {
        Shitroid_HandleNormalPalette(game);
    }
}
pub unsafe fn Shitroid_Func_3(k: u16, game: &mut Game) {
    let E: *mut Enemy_Shitroid = Get_Shitroid(k as i32, game);
    (*E).shitr_var_B = 0u16;
    (*E).shitr_var_C = 0u16;
}
pub unsafe fn Shitroid_Func_4(game: &mut Game) {
    if ram_word!(game, layer1_x_pos) as i32 - 513i32 & 0x8000i32 != 0 {
        set_ram_word!(game.layer1_x_pos = 512u16);
        *(ram_ptr!(game, u16; 0xcd20)) = *ram_ptr!(game; 0xcd20, 0) as u16;
        *(ram_ptr!(game, u16; 0xcd20, 2)) = *ram_ptr!(game; 0xcd20, 2) as u16;
        SpawnHardcodedPlm(
            SpawnHardcodedPlmArgs::new(0x30u8, 0x3u8, 0xb767 as VoidP),
            game,
        );
        SpawnHardcodedPlm(
            SpawnHardcodedPlmArgs::new(0x1fu8, 0x3u8, 0xb767 as VoidP),
            game,
        );
        let E: *mut Enemy_Shitroid = Get_Shitroid(ram_word!(game, cur_enemy_index) as i32, game);
        (*E).shitr_var_A = ((0xa9f02bi32) as u16);
        (*E).shitr_var_04 = 1u16;
    }
}
pub unsafe fn Shitroid_Func_5(k: u16, game: &mut Game) {
    let E: *mut Enemy_Shitroid = Get_Shitroid(k as i32, game);
    (*E).shitr_var_A = ((0xa9f037i32) as u16);
    (*E).shitr_var_F = 464u16;
    Shitroid_Func_6(k, game);
}
pub unsafe fn Shitroid_Func_6(k: u16, game: &mut Game) {
    let E: *mut Enemy_Shitroid = Get_Shitroid(k as i32, game);
    (*E).shitr_var_F = (*E).shitr_var_F.wrapping_sub(1);
    if ((*E).shitr_var_F as i32 & 0x8000i32) != 0i32 {
        QueueMusic_Delayed8(5u16, game);
        (*E).shitr_var_A = ((0xa9f049i32) as u16);
        Shitroid_Func_7(k, game);
    }
}
pub unsafe fn Shitroid_Func_7(k: u16, game: &mut Game) {
    let rect: Rect16U = Rect16U {
        x: 584u16,
        y: 74u16,
        w: 1u16,
        h: 1u16,
    };
    Shitroid_Func_GraduallyAccelerateTowards0x400(k, 0xfu16, rect.x, rect.y, game);
    if Shitroid_Func_2(k, rect, game) == 0 {
        (*Get_Shitroid(k as i32, game)).shitr_var_A = ((0xa9f06di32) as u16);
    }
}
pub unsafe fn Shitroid_Func_8(k: u16, game: &mut Game) {
    let E: *mut Enemy_Shitroid = Get_Shitroid(k as i32 + 64i32, game);
    Shitroid_Func_GraduallyAccelerateTowards0x400(
        k,
        0xfu16,
        (*E).base.x_pos,
        ((*E).base.y_pos).wrapping_sub(32u16),
        game,
    );
    if Shitroid_Func_1(k, (k).wrapping_add(64u16), game) as i32 & 1i32 == 0 {
        (*Get_Shitroid(k as i32, game)).shitr_var_A = ((0xa9f094i32) as u16);
    }
}
pub unsafe fn Shitroid_Func_9(k: u16, game: &mut Game) {
    let E1: *mut Enemy_Shitroid = Get_Shitroid(k as i32 + 64i32, game);
    if Shitroid_AccelerateTowardsPoint(
        k,
        0x200u16,
        (*E1).base.x_pos,
        ((*E1).base.y_pos).wrapping_sub(32u16),
        game,
    ) as i32
        & 1i32
        != 0
    {
        let E: *mut Enemy_Shitroid = Get_Shitroid(k as i32, game);
        (*E).shitr_var_B = 0u16;
        (*E).shitr_var_C = 0u16;
        (*E).base.x_pos = (*E1).base.x_pos;
        (*E).base.y_pos = ((*E1).base.y_pos).wrapping_sub(32u16);
        (*E).base.current_instruction = addr_kShitroid_Ilist_F924 as u16;
        (*E).base.instruction_timer = 1u16;
        (*E).base.timer = 0u16;
        (*E).shitr_var_A = ((0xa9f0e6i32) as u16);
        (*E).shitr_var_E = 1u16;
        (*E).shitr_parameter_2 = 0u16;
        (*E).shitr_var_F = 320u16;
    }
}
pub unsafe fn Shitroid_Func_10(k: u16, game: &mut Game) {
    let E: *mut Enemy_Shitroid = Get_Shitroid(k as i32, game);
    let v2: i32 = ((*E).base.frame_counter as i32 & 6i32) >> 1i32;
    let E1: *mut Enemy_Shitroid = Get_Shitroid(k as i32 + 64i32, game);
    (*E).base.x_pos = ((*E1).base.x_pos).wrapping_add((g_word_A993BB[v2 as usize]) as u16);
    (*E).base.y_pos = ((*E1).base.y_pos as i32 + g_word_A993C3[v2 as usize] as i32 - 32i32) as u16;
    let fresh13 = (*E).shitr_var_F;
    (*E).shitr_var_F = (*E).shitr_var_F.wrapping_sub(1);
    if fresh13 == 1u16 {
        (*E).shitr_var_A = ((0xa9f125i32) as u16);
        (*E).base.current_instruction = addr_kShitroid_Ilist_F906 as u16;
        (*E).base.instruction_timer = 1u16;
        (*E).base.timer = 0u16;
        (*E).shitr_var_E = 10u16;
    }
}
pub unsafe fn Shitroid_Func_11(k: u16, game: &mut Game) {
    (*Get_Shitroid(0x40i32, game)).shitr_var_08 = 1u16;
    let E: *mut Enemy_Shitroid = Get_Shitroid(k as i32, game);
    (*E).shitr_var_A = ((0xa9f138i32) as u16);
    (*E).shitr_var_F = 192u16;
    Shitroid_Func_12(k, game);
}
pub unsafe fn Shitroid_Func_12(k: u16, game: &mut Game) {
    let E: *mut Enemy_Shitroid = Get_Shitroid(k as i32, game);
    Shitroid_Func_GraduallyAccelerateTowards0x400(k, 0u16, (*E).base.x_pos, 104u16, game);
    (*E).shitr_var_F = (*E).shitr_var_F.wrapping_sub(1);
    if ((*E).shitr_var_F as i32 & 0x8000i32) != 0i32 {
        (*E).shitr_var_A = ((0xa9f180i32) as u16);
        (*E).shitr_parameter_2 = 1u16;
        let ref mut fresh14 = *(ram_ptr!(game, u16; 0xcd20));
        *fresh14 = (*fresh14) | (0x100u16);
        let ref mut fresh15 = *(ram_ptr!(game, u16; 0xcd20, 2));
        *fresh15 = (*fresh15) | (0x100u16);
        SpawnHardcodedPlm(
            SpawnHardcodedPlmArgs::new(0x30u8, 0x3u8, 0xb763 as VoidP),
            game,
        );
        SpawnHardcodedPlm(
            SpawnHardcodedPlmArgs::new(0x1fu8, 0x3u8, 0xb763 as VoidP),
            game,
        );
    }
}
pub unsafe fn Shitroid_Func_13(k: u16, game: &mut Game) {
    let E: *mut Enemy_Shitroid = Get_Shitroid(k as i32, game);
    let v2: u16 = abs16(((*E).base.x_pos).wrapping_sub(ram_word!(game, samus_x_pos)));
    let mut shitro_var_02: u16 = 0;
    if v2 >= 8u16 {
        shitro_var_02 = (*E).shitr_var_02;
        if shitro_var_02 != 0 {
            shitro_var_02 = shitro_var_02.wrapping_sub(1);
            if (shitro_var_02 as i32 & 0x8000i32) != 0i32 {
                shitro_var_02 = 0u16;
            }
        }
    } else {
        shitro_var_02 = ((v2 >= 8u16) as i32 + (*E).shitr_var_02 as i32 + 2i32) as u16;
    }
    (*E).shitr_var_02 = shitro_var_02;
    if shitro_var_02 >= 0x100u16 || ram_word!(game, samus_x_pos) as i32 - 512i32 & 0x8000i32 != 0 {
        (*Get_Shitroid(ram_word!(game, cur_enemy_index) as i32, game)).shitr_var_A =
            ((0xa9f1fai32) as u16);
    } else {
        let shitro_var_01: u16 = (*E).shitr_var_01;
        let mut v5: u16 = 0;
        if shitro_var_01 != 0 {
            (*E).shitr_var_01 = (shitro_var_01).wrapping_sub(1u16);
            v5 = ram_word!(game, samus_y_pos);
        } else {
            v5 = 80u16;
            if (ram_word!(game, random_number) as i32 & 0xfffi32) >= 0xfe0i32 {
                (*E).shitr_var_01 = 32u16;
            }
        }
        Shitroid_Func_GraduallyAccelerateTowards0x400(
            ram_word!(game, cur_enemy_index),
            0xau16,
            ram_word!(game, samus_x_pos),
            v5,
            game,
        );
    };
}
pub unsafe fn Shitroid_Func_14(k: u16, game: &mut Game) {
    Shitroid_Func_GraduallyAccelerateTowards0x400(
        k,
        0xfu16,
        ram_word!(game, samus_x_pos),
        (ram_word!(game, samus_y_pos)).wrapping_sub(32u16),
        game,
    );
}
pub unsafe fn Shitroid_Func_15(k: u16, game: &mut Game) {
    CallSomeSamusCode(0x12u16, game);
    (*Get_Shitroid(0i32, game)).shitr_var_A = ((0xa9f21bi32) as u16);
    Shitroid_Func_16(k, game);
}
pub unsafe fn Shitroid_Func_16(k: u16, game: &mut Game) {
    let E: *mut Enemy_Shitroid = Get_Shitroid(k as i32, game);
    if ram_word!(game, samus_health) < 2u16 {
        set_ram_words! { game, 0u16; samus_x_speed_divisor, bomb_counter }
        (*E).shitr_var_A = ((0xa9f2a2i32) as u16);
        (*E).shitr_var_B = 0u16;
        (*E).shitr_var_C = 0u16;
        Enemy_SetInstrList(k, addr_kShitroid_Ilist_F906 as u16, game);
        (*E).shitr_var_E = 10u16;
        CallSomeSamusCode(0x13u16, game);
        SomeMotherBrainScripts(0u16, game);
        (*E).shitr_var_04 = 0u16;
        QueueMusic_Delayed8(7u16, game);
    } else {
        set_ram_word!(game.cooldown_timer = 8u16);
        set_ram_word!(game.bomb_counter = 5u16);
        set_ram_word!(game.samus_x_speed_divisor = 2u16);
        if ram_word!(game, samus_y_speed) as i32 - 4i32 & 0x8000i32 == 0 {
            set_ram_word!(game.samus_y_speed = 2u16);
        }
        let v2: i32 = ((*E).base.frame_counter as i32 & 6i32) >> 1i32;
        (*E).base.x_pos =
            (ram_word!(game, samus_x_pos)).wrapping_add((g_word_A993BB[v2 as usize]) as u16);
        (*E).base.y_pos = (ram_word!(game, samus_y_pos) as i32 + g_word_A993C3[v2 as usize] as i32
            - 20i32) as u16;
        Samus_DamageDueToShitroid(game);
    };
}
pub unsafe fn Shitroid_Func_17(k: u16, game: &mut Game) {
    let E: *mut Enemy_Shitroid = Get_Shitroid(k as i32, game);
    (*E).shitr_var_A = ((0xa9f2aei32) as u16);
    (*E).shitr_var_F = 120u16;
    Shitroid_Func_18(k, game);
}
pub unsafe fn Shitroid_Func_18(k: u16, game: &mut Game) {
    let E: *mut Enemy_Shitroid = Get_Shitroid(k as i32, game);
    (*E).shitr_var_F = (*E).shitr_var_F.wrapping_sub(1);
    if ((*E).shitr_var_F as i32 & 0x8000i32) != 0i32 {
        (*E).shitr_var_A = ((0xa9f2c0i32) as u16);
        (*E).shitr_var_F = 192u16;
        Shitroid_Func_19(k, game);
    }
}
pub unsafe fn Shitroid_Func_19(k: u16, game: &mut Game) {
    Shitroid_Func_GraduallyAccelerateTowards0x400(
        ram_word!(game, cur_enemy_index),
        0u16,
        ram_word!(game, samus_x_pos),
        104u16,
        game,
    );
    let E: *mut Enemy_Shitroid = Get_Shitroid(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).shitr_var_F = (*E).shitr_var_F.wrapping_sub(1);
    if ((*E).shitr_var_F as i32 & 0x8000i32) != 0i32 {
        QueueSfx2_Max6(0x7du16, game);
        (*E).shitr_var_A = ((0xa9f2fbi32) as u16);
        (*E).shitr_var_F = 88u16;
        (*E).base.current_instruction = addr_kShitroid_Ilist_F924 as u16;
        (*E).base.instruction_timer = 1u16;
        (*E).base.timer = 0u16;
        Shitroid_Func_20(game);
    }
}
pub unsafe fn Shitroid_Func_20(game: &mut Game) {
    Shitroid_Func_GraduallyAccelerateTowards0x400(
        ram_word!(game, cur_enemy_index),
        0u16,
        (ram_word!(game, samus_x_pos)).wrapping_sub(64u16),
        100u16,
        game,
    );
    let E: *mut Enemy_Shitroid = Get_Shitroid(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).shitr_var_F = (*E).shitr_var_F.wrapping_sub(1);
    if ((*E).shitr_var_F as i32 & 0x8000i32) != 0i32 {
        (*E).shitr_var_A = ((0xa9f324i32) as u16);
        (*E).shitr_var_F = 88u16;
        Shitroid_Func_21(game);
    }
}
pub unsafe fn Shitroid_Func_21(game: &mut Game) {
    Shitroid_Func_GraduallyAccelerateTowards0x400(
        ram_word!(game, cur_enemy_index),
        0u16,
        (ram_word!(game, samus_x_pos)).wrapping_add(96u16),
        104u16,
        game,
    );
    let E: *mut Enemy_Shitroid = Get_Shitroid(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).shitr_var_F = (*E).shitr_var_F.wrapping_sub(1);
    if ((*E).shitr_var_F as i32 & 0x8000i32) != 0i32 {
        (*E).shitr_var_A = ((0xa9f3a3i32) as u16);
        (*E).shitr_var_F = 256u16;
        (*E).base.current_instruction = addr_kShitroid_Ilist_F93A as u16;
        (*E).base.instruction_timer = 1u16;
        (*E).base.timer = 0u16;
    }
}
pub unsafe fn Shitroid_Func_22(k: u16, game: &mut Game) {
    QueueSfx2_Max6(0x52u16, game);
    (*Get_Shitroid(k as i32, game)).shitr_var_A = ((0xa9f36di32) as u16);
    Shitroid_Func_23(game);
}
pub unsafe fn Shitroid_Func_23(game: &mut Game) {
    let rect: Rect16U = Rect16U {
        x: -128i32 as u16,
        y: 64u16,
        w: 8u16,
        h: 8u16,
    };
    Shitroid_Func_GraduallyAccelerateTowards0x400(
        ram_word!(game, cur_enemy_index),
        0u16,
        rect.x,
        rect.y,
        game,
    );
    if Shitroid_Func_2(ram_word!(game, cur_enemy_index), rect, game) == 0 {
        let E: *mut Enemy_Shitroid = Get_Shitroid(ram_word!(game, cur_enemy_index) as i32, game);
        (*E).shitr_var_B = 0u16;
        (*E).shitr_var_C = 0u16;
        (*E).base.properties = ((*E).base.properties as i32
            & !(kEnemyProps_DisableSamusColl as i32 | kEnemyProps_Invisible as i32))
            as u16;
        (*E).shitr_var_A = ((0xa9efdfi32) as u16);
    }
}
pub unsafe fn Shitroid_Func_24(k: u16, game: &mut Game) {
    let E: *mut Enemy_Shitroid = Get_Shitroid(k as i32, game);
    (*E).shitr_var_F = (*E).shitr_var_F.wrapping_sub(1);
    if ((*E).shitr_var_F as i32 & 0x8000i32) != 0i32 {
        SomeMotherBrainScripts(2u16, game);
        (*E).shitr_parameter_2 = 1u16;
        (*E).shitr_var_A = ((0xa9f3bei32) as u16);
        Shitroid_Func_25(k, game);
    } else {
        Shitroid_Func_27(k, game);
    };
}
pub unsafe fn Shitroid_Func_25(k: u16, game: &mut Game) {
    if Shitroid_Func_27(k, game) as i32 & 1i32 != 0 {
        Shitroid_F3C4(k, game);
    }
}
pub unsafe fn Shitroid_F3C4(k: u16, game: &mut Game) {
    (*Get_Shitroid(k as i32, game)).shitr_var_A = ((0xa9f360i32) as u16);
}
pub unsafe fn Shitroid_Func_26(k: u16, game: &mut Game) {
    if (*Get_Shitroid(k as i32, game)).shitr_var_A as i32 == 0xa9f3bei32 & 0xffffi32 {
        Shitroid_F3C4(k, game);
    }
}
pub unsafe fn Shitroid_Func_27(k: u16, game: &mut Game) -> u8 {
    let E: *mut Enemy_Shitroid = Get_Shitroid(k as i32, game);
    let v2: u16 = abs16(((*E).base.x_pos).wrapping_sub(ram_word!(game, samus_x_pos)));
    let mut shitro_var_02: u16 = 0;
    if v2 >= 2u16 {
        shitro_var_02 = (*E).shitr_var_02;
        if shitro_var_02 != 0 {
            shitro_var_02 = shitro_var_02.wrapping_sub(1);
            if (shitro_var_02 as i32 & 0x8000i32) != 0i32 {
                shitro_var_02 = 0u16;
            }
        }
    } else {
        shitro_var_02 = ((v2 >= 2u16) as i32 + (*E).shitr_var_02 as i32 + 2i32) as u16;
    }
    (*E).shitr_var_02 = shitro_var_02;
    let shitro_var_01: u16 = (*E).shitr_var_01;
    let mut v5: u16 = 0;
    if shitro_var_01 != 0 {
        (*E).shitr_var_01 = (shitro_var_01).wrapping_sub(1u16);
        v5 = (ram_word!(game, samus_y_pos)).wrapping_sub(18u16);
    } else {
        v5 = 80u16;
        if (ram_word!(game, random_number) as i32 & 0xfffi32) >= 0xfe0i32 {
            (*E).shitr_var_01 = 32u16;
        }
    }
    Shitroid_Func_GraduallyAccelerateTowards0x400(
        ram_word!(game, cur_enemy_index),
        8u16,
        ram_word!(game, samus_x_pos),
        v5,
        game,
    );
    ((*Get_Shitroid(ram_word!(game, cur_enemy_index) as i32, game)).shitr_var_02 >= 0x400u16
        || ram_word!(game, samus_x_pos) as i32 - 128i32 & 0x8000i32 != 0) as u8
}
pub unsafe fn Shitroid_GraduallyAccelerateTowardsPt(
    k: u16,
    j: u16,
    r18: u16,
    r20: u16,
    r26: u16,
    game: &mut Game,
) {
    let mut v3: i16 = 0;
    let mut v5: i16 = 0;
    let mut v6: i16 = 0;
    let mut shitr_var_C: i16 = 0;
    let mut v9: i16 = 0;
    let r24: u16 = g_byte_A9F56A[j as usize] as u16;
    Shitroid_GraduallyAccelerateHoriz(k, r18, r24, r26, game);
    let E: *mut Enemy_Shitroid = Get_Shitroid(k as i32, game);
    v3 = (((*E).base.y_pos) as i16).wrapping_sub((r20) as i16);
    if v3 != 0 {
        if v3 >= 0i16 {
            let mut RegWord: u16 = SnesDivide(v3 as u16, r24 as u8);
            if RegWord == 0 {
                RegWord = 1u16;
            }
            let r22: u16 = RegWord;
            shitr_var_C = (*E).shitr_var_C as i16;
            if shitr_var_C >= 0i16 {
                shitr_var_C = (shitr_var_C as i32 - 8i32 - r22 as i32) as i16;
            }
            v9 = (shitr_var_C).wrapping_sub((r22) as i16);
            if v9 as i32 + 1280i32 & 0x8000i32 != 0 {
                v9 = -1280i16;
            }
            (*E).shitr_var_C = v9 as u16;
        } else {
            let mut v4: u16 = SnesDivide((r20).wrapping_sub((*E).base.y_pos), r24 as u8);
            if v4 == 0 {
                v4 = 1u16;
            }
            let r22_0: u16 = v4;
            v5 = (*E).shitr_var_C as i16;
            if v5 < 0i16 {
                v5 = (v5 as i32 + (r22_0 as i32 + 8i32)) as i16;
            }
            v6 = ((r22_0) as i16).wrapping_add(v5);
            if v6 as i32 - 1280i32 & 0x8000i32 == 0 {
                v6 = 1280i16;
            }
            (*E).shitr_var_C = v6 as u16;
        }
    }
}
pub unsafe fn Shitroid_Func_GraduallyAccelerateTowards0x400(
    k: u16,
    j: u16,
    r18: u16,
    r20: u16,
    game: &mut Game,
) {
    Shitroid_GraduallyAccelerateTowardsPt(k, j, r18, r20, 1024u16, game);
}
pub unsafe fn Shitroid_GraduallyAccelerateTowards0x4(
    k: u16,
    j: u16,
    r18: u16,
    r20: u16,
    game: &mut Game,
) {
    Shitroid_GraduallyAccelerateTowardsPt(k, j, r18, r20, 4u16, game);
}
pub unsafe fn Shitroid_GraduallyAccelerateTowards0x8(
    k: u16,
    j: u16,
    r18: u16,
    r20: u16,
    game: &mut Game,
) {
    Shitroid_GraduallyAccelerateTowardsPt(k, j, r18, r20, 8u16, game);
}
pub unsafe fn Shitroid_GraduallyAccelerateTowards0x10(
    k: u16,
    j: u16,
    r18: u16,
    r20: u16,
    game: &mut Game,
) {
    Shitroid_GraduallyAccelerateTowardsPt(k, j, r18, r20, 16u16, game);
}
pub unsafe fn Shitroid_GraduallyAccelerateHoriz(
    k: u16,
    r18: u16,
    r24: u16,
    r26: u16,
    game: &mut Game,
) {
    let mut v2: i16 = 0;
    let mut v4: i16 = 0;
    let mut v5: i8 = 0;
    let mut v7: i16 = 0;
    let mut shitr_var_B: i16 = 0;
    let mut v11: i16 = 0;
    let E: *mut Enemy_Shitroid = Get_Shitroid(k as i32, game);
    v2 = (((*E).base.x_pos) as i16).wrapping_sub((r18) as i16);
    if v2 != 0 {
        if v2 >= 0i16 {
            let mut RegWord: u16 = SnesDivide(v2 as u16, r24 as u8);
            if RegWord == 0 {
                RegWord = 1u16;
            }
            let r22: u16 = RegWord;
            shitr_var_B = (*E).shitr_var_B as i16;
            if shitr_var_B >= 0i16 {
                let v13: u16 = (*E).shitr_var_B;
                v5 = ((Shitroid_CheckIfOnScreen(k, game)) as i8) & (1i8);
                let mut v10: u16 = v13;
                if v5 != 0 {
                    v10 = (v13).wrapping_sub(r26);
                }
                shitr_var_B = (v10 as i32 - 8i32 - r22 as i32) as i16;
            }
            v11 = (shitr_var_B).wrapping_sub((r22) as i16);
            if v11 as i32 + 2048i32 & 0x8000i32 != 0 {
                v11 = -2048i16;
            }
            (*E).shitr_var_B = v11 as u16;
        } else {
            let mut v3: u16 = SnesDivide((r18).wrapping_sub((*E).base.x_pos), r24 as u8);
            if v3 == 0 {
                v3 = 1u16;
            }
            let r22_0: u16 = v3;
            v4 = (*E).shitr_var_B as i16;
            if v4 < 0i16 {
                let v12: u16 = (*E).shitr_var_B;
                v5 = ((Shitroid_CheckIfOnScreen(k, game)) as i8) & (1i8);
                let mut v6: u16 = v12;
                if v5 != 0 {
                    v6 = (r26).wrapping_add(v12);
                }
                v4 = (r22_0 as i32 + v6 as i32 + 8i32) as i16;
            }
            v7 = ((r22_0) as i16).wrapping_add(v4);
            if v7 as i32 - 2048i32 & 0x8000i32 == 0 {
                v7 = 2048i16;
            }
            (*E).shitr_var_B = v7 as u16;
        }
    }
}
pub unsafe fn Shitroid_CheckIfOnScreen(k: u16, game: &mut Game) -> u8 {
    let mut y_pos: i16 = 0;
    let mut v3: i16 = 0;
    let mut x_pos: i16 = 0;
    let mut v5: i16 = 0;
    let E: *mut Enemy_Shitroid = Get_Shitroid(k as i32, game);
    y_pos = (*E).base.y_pos as i16;
    let result: u8 = 1u8;
    if y_pos >= 0i16 {
        v3 = (y_pos as i32 + 96i32 - ram_word!(game, layer1_y_pos) as i32) as i16;
        if v3 >= 0i16 && v3 as i32 - 416i32 & 0x8000i32 != 0 {
            x_pos = (*E).base.x_pos as i16;
            if x_pos >= 0i16 {
                v5 = (x_pos as i32 + 16i32 - ram_word!(game, layer1_x_pos) as i32) as i16;
                if v5 >= 0i16 && v5 as i32 - 288i32 & 0x8000i32 != 0 {
                    return 0u8;
                }
            }
        }
    }
    result
}
pub unsafe fn Shitroid_AccelerateTowardsPoint(
    k: u16,
    a: u16,
    r18: u16,
    r20: u16,
    game: &mut Game,
) -> u8 {
    let mut R28: u16 = 0u16;
    R28 = (R28).wrapping_add(Shitroid_AccelerateTowardsX(k, r18, a, game));
    R28 = (R28).wrapping_add(Shitroid_AccelerateTowardsY(k, r20, a, game));
    R28 = (R28 as i32 >> 1i32) as u16;
    ((R28) as u8) & (1u8)
}
pub unsafe fn Shitroid_AccelerateTowardsY(k: u16, r20: u16, r22: u16, game: &mut Game) -> u16 {
    let E: *mut Enemy_Shitroid = Get_Shitroid(k as i32, game);
    let mut result: u16 = 0u16;
    let v2: i16 = (((*E).base.y_pos) as i16).wrapping_sub((r20) as i16);
    if v2 == 0 {
        result = result.wrapping_add(1);
    } else if v2 >= 0i16 {
        let mut v7: u16 = ((*E).shitr_var_C).wrapping_sub(r22);
        if v7 as i32 + 1280i32 & 0x8000i32 != 0 {
            v7 = -1280i32 as u16;
        }
        (*E).shitr_var_C = v7;
        if (((*E).base.y_pos as i32 + (v7 as i32 >> 8i32) as i8 as i32 - r20 as i32) as i16) <= 0i16
        {
            (*E).shitr_var_C = 0u16;
            result = result.wrapping_add(1);
        }
    } else {
        let mut v3: u16 = (r22).wrapping_add((*E).shitr_var_C);
        if v3 as i32 - 1280i32 & 0x8000i32 == 0 {
            v3 = 1280u16;
        }
        (*E).shitr_var_C = v3;
        if (((*E).base.y_pos as i32 + (v3 as i32 >> 8i32) as i8 as i32 - r20 as i32) as i16) >= 0i16
        {
            (*E).shitr_var_C = 0u16;
            result = result.wrapping_add(1);
        }
    }
    result
}
pub unsafe fn Shitroid_AccelerateTowardsX(k: u16, r18: u16, r22: u16, game: &mut Game) -> u16 {
    let E: *mut Enemy_Shitroid = Get_Shitroid(k as i32, game);
    let mut result: u16 = 0u16;
    if (((*E).base.x_pos) as i16).wrapping_sub((r18) as i16) >= 0i16 {
        let mut v6: u16 = ((*E).shitr_var_B).wrapping_sub(r22);
        if v6 as i32 + 1280i32 & 0x8000i32 != 0 {
            v6 = -1280i32 as u16;
        }
        (*E).shitr_var_B = v6;
        if (((*E).base.x_pos as i32 + (v6 as i32 >> 8i32) as i8 as i32 - r18 as i32) as i16) <= 0i16
        {
            (*E).shitr_var_B = 0u16;
            result = result.wrapping_add(1);
        }
    } else {
        let mut v2: u16 = (r22).wrapping_add((*E).shitr_var_B);
        if v2 as i32 - 1280i32 & 0x8000i32 == 0 {
            v2 = 1280u16;
        }
        (*E).shitr_var_B = v2;
        if (((*E).base.x_pos as i32 + (v2 as i32 >> 8i32) as i8 as i32 - r18 as i32) as i16) >= 0i16
        {
            (*E).shitr_var_B = 0u16;
            result = result.wrapping_add(1);
        }
    }
    result
}
pub unsafe fn Shitroid_HandleNormalPalette(game: &mut Game) {
    Shitroid_HandleCutscenePalette_Common(addr_word_A9F6D1 as u16, 330u16, game);
}
pub unsafe fn Shitroid_HandleCutscenePalette(game: &mut Game) {
    Shitroid_HandleCutscenePalette_Common(addr_word_A9F6D1 as u16, 490u16, game);
}
pub unsafe fn Shitroid_HandleCutscenePalette_LowHealth(game: &mut Game) {
    Shitroid_HandleCutscenePalette_Common(addr_word_A9F711 as u16, 490u16, game);
}
pub unsafe fn Shitroid_HandleCutscenePalette_Common(r22: u16, r18: u16, game: &mut Game) {
    let mut shitro_var_D_high: i8 = 0;
    let E: *mut Enemy_Shitroid = Get_Shitroid(ram_word!(game, cur_enemy_index) as i32, game);
    shitro_var_D_high = *(&raw mut (*E).shitr_var_D as *mut u8).offset(1isize) as i8;
    if shitro_var_D_high != 0 {
        *(&raw mut (*E).shitr_var_D as *mut u8).offset(1isize) =
            ((shitro_var_D_high) as u8).wrapping_sub(1u8);
    } else {
        *(&raw mut (*E).shitr_var_D as *mut u8).offset(1isize) = (*E).shitr_var_E as u8;
        let v3: u8 = (*(&raw mut (*E).shitr_var_D as *mut u8).offset(0) as i32 + 1i32 & 7i32) as u8;
        *(&raw mut (*E).shitr_var_D as *mut u8).offset(0) = v3;
        let v4: u16 =
            Shitroid_HandleCrySoundEffect(ram_word!(game, cur_enemy_index), v3 as u16, game);
        WriteColorsToPalette(
            r18,
            0xa9u8,
            (r22 as i32 + 8i32 * v4 as i32) as u16,
            4u16,
            game,
        );
    };
}
pub unsafe fn Shitroid_HandleCrySoundEffect(k: u16, a: u16, game: &mut Game) -> u16 {
    if a == 5u16 {
        let E: *mut Enemy_Shitroid = Get_Shitroid(k as i32, game);
        if (*E).shitr_var_04 != 0 {
            let v3: u16 = ((*E).shitr_var_05).wrapping_add(1u16);
            (*E).shitr_var_05 = v3;
            if v3 >= 4u16 {
                (*E).shitr_var_05 = 0u16;
                let mut v4: u16 = 114u16;
                if (*E).shitr_var_E < 0xau16 {
                    v4 = 120u16;
                }
                QueueSfx2_Max6(v4, game);
            }
        }
        return 5u16;
    }
    a
}
pub unsafe fn Shitroid_Touch(game: &mut Game) {
    let E: *mut Enemy_Shitroid = Get_Shitroid(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).shitr_parameter_2 != 0 {
        Shitroid_Func_26(ram_word!(game, cur_enemy_index), game);
        if *ram_ptr!(game; 0xa1f) == 3u8
            && ram_word!(game, samus_x_pos) as i32 - 512i32 & 0x8000i32 == 0
        {
            let r18: u16 = (0x80i32
                - CalculateAngleFromXY(
                    (ram_word!(game, samus_x_pos)).wrapping_sub((*E).base.x_pos),
                    (ram_word!(game, samus_y_pos)).wrapping_sub((*E).base.y_pos),
                    game,
                ) as i32
                + 0x80i32) as u8 as u16;
            (*E).shitr_var_B = ((*E).shitr_var_B).wrapping_add(Math_MultBySin(0x40u16, r18));
            (*E).shitr_var_C = ((*E).shitr_var_C).wrapping_add(Math_MultByCos(0x40u16, r18));
        } else if (*E).shitr_var_A as i32 == 0xa9f1fai32 & 0xffffi32 {
            if Shitroid_AccelerateTowardsPoint(
                ram_word!(game, cur_enemy_index),
                0x200u16,
                ram_word!(game, samus_x_pos),
                (ram_word!(game, samus_y_pos)).wrapping_sub(32u16),
                game,
            ) as i32
                & 1i32
                != 0
            {
                (*E).base.current_instruction = addr_kShitroid_Ilist_F924 as u16;
                (*E).base.instruction_timer = 1u16;
                (*E).base.timer = 0u16;
                (*E).shitr_var_E = 1u16;
                (*E).shitr_parameter_2 = 0u16;
                (*E).shitr_var_B = 0u16;
                (*E).shitr_var_C = 0u16;
                (*E).shitr_var_A = ((0xa9f20ei32) as u16);
            }
        } else if (*E).shitr_var_A as i32 == 0xa9f180i32 & 0xffffi32 {
            (*E).shitr_var_A = ((0xa9f1fai32) as u16);
        }
    }
}
pub unsafe fn Shitroid_Shot(game: &mut Game) {
    let E: *mut Enemy_Shitroid = Get_Shitroid(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).shitr_parameter_2 != 0 {
        Shitroid_Func_26(ram_word!(game, cur_enemy_index), game);
        let r18: u16 = -(CalculateAngleFromXY(
            (ram_word!(game, projectile_x_pos[0])).wrapping_sub((*E).base.x_pos),
            (ram_word!(game, projectile_y_pos[0])).wrapping_sub((*E).base.y_pos),
            game,
        ) as i32) as u8 as u16;
        let mut v1: u16 = (8u16).wrapping_mul(ram_word!(
            game,
            projectile_damage[ram_word!(game, collision_detection_index)]
        ));
        if v1 >= 0xf0u16 {
            v1 = 240u16;
        }
        (*E).shitr_var_B = ((*E).shitr_var_B).wrapping_add(Math_MultBySin(v1, r18));
        (*E).shitr_var_C = ((*E).shitr_var_C).wrapping_add(Math_MultByCos(v1, r18));
    }
}
pub unsafe fn Shitroid_Instr_3(k: u16, jp: *const u16) -> *const u16 {
    addr_kShitroid_Ilist_F90E as i32 as usize as *const u16
}
pub unsafe fn Shitroid_Instr_4(k: u16, jp: *const u16) -> *const u16 {
    addr_kShitroid_Ilist_F924 as i32 as usize as *const u16
}
pub unsafe fn Shitroid_Instr_6(k: u16, jp: *const u16) -> *const u16 {
    addr_kShitroid_Ilist_F93A as i32 as usize as *const u16
}
pub unsafe fn Shitroid_Instr_5(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    if (ram_word!(game, random_number) as i32 & 0x8000i32) == 0i32 {
        return *jp.offset(0) as usize as *const u16;
    }
    QueueSfx2_Max6(0x52u16, game);
    jp.offset(1)
}
