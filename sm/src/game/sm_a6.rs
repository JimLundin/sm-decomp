pub use crate::game::constants::bank_86::*;
pub use crate::game::constants::bank_a6::*;
pub use crate::game::constants::bank_a9::*;
pub use crate::game::constants::enemy_properties::*;
use crate::game::enemy_layouts::enemy_layout;
use crate::game::sm_80::{
    CheckEventHappened, ClearEventHappened, NextRandom, QueueMode7Transfers, QueueMusic_Delayed8,
    QueueSfx1_Max6, QueueSfx2_Max3, QueueSfx2_Max6, QueueSfx3_Max3, QueueSfx3_Max6,
    SetBossBitForCurArea, SetEventHappened,
};
use crate::game::sm_81::DrawSpritemap;
use crate::game::sm_86::{
    Math_MultByCos, Math_MultBySin, SpawnEprojWithGfx, SpawnEprojWithRoomGfx,
};
use crate::game::sm_8b::CalcCeresSteamPos_Mode7;
use crate::game::sm_90::{
    CallSomeSamusCode, DisableMinimapAndMarkBossRoomAsExplored, Samus_SetPushedOutOfCeresRidley,
};
use crate::game::sm_91::Samus_DealDamage;
use crate::game::sm_94::kSinCosTable8bit_Sext;
use crate::game::sm_a0::{
    CalculateAngleFromXY, CheckIfEnemyIsOnScreen, CosineMult8bit, EnemyDeathAnimation,
    Enemy_ItemDrop_MiniKraid, Enemy_ItemDrop_Ridley, Enemy_MoveDown, Enemy_MoveRight_IgnoreSlopes,
    Enemy_SwitchToFrozenAi, GetSamusEnemyDelta_X, GetSamusEnemyDelta_Y, NormalEnemyFrozenAI,
    NormalEnemyPowerBombAiSkipDeathAnim_CurEnemy, NormalEnemyShotAi,
    NormalEnemyShotAiSkipDeathAnim_CurEnemy, NormalEnemyTouchAi,
    NormalEnemyTouchAiSkipDeathAnim_CurEnemy, SineMult8bit, SpawnEnemy, SuitDamageDivision,
    SwitchEnemyAiToMainAi,
};
use crate::game::sm_a9::{
    ComputeCosMult, ComputeSinMult, MoveEnemyWithVelocity, Shitroid_Func_2, WriteColorsToPalette,
    WriteColorsToTargetPalette,
};
use crate::game::sm_b4::CreateSpriteAtPos;
use crate::game_state::Game;
use crate::rtl::abs16;
use crate::rtl::AddToHiLo;
use crate::rtl::{mov24, Mult8x8, RomPtr, SnesDivide, Unreachable};
extern "C" {
    fn printf(__format: *const ::core::ffi::c_char, ...) -> ::core::ffi::c_int;

}
pub use crate::types::Consts_A6;
pub use crate::types::Consts_A9;
pub use crate::types::{
    __int16_t, __int32_t, __int8_t, __uint16_t, __uint32_t, __uint8_t, int16, int16_t, int32,
    int32_t, int8, int8_t, uint16, uint16_t, uint32, uint32_t, uint8, uint8_t, uintptr_t,
    EnemyBase, EnemyData, EnemyProps, EnemySpawnData, Func_V, LongPtr, OamEnt, PairU16, Point16U,
    Rect16U, VoidP, VramWriteEntry,
};
pub use crate::types::{Consts_86, ExtraEnemyRam7800, ExtraEnemyRam8000, Ram7800_Kraid};
enemy_layout! {
    Enemy_Ridley {
        base: EnemyBase,
        words: [
            ridley_var_A, ridley_var_B, ridley_var_C, ridley_var_D, ridley_var_E,
            ridley_var_F, ridley_parameter_1, ridley_parameter_2,
        ],
        pad1: [uint8; 26696],
        words: [
            ridley_var_00, ridley_var_01, ridley_var_02, ridley_var_03, ridley_var_04,
            ridley_var_05, ridley_var_06, ridley_var_07, ridley_var_08, ridley_var_09,
            ridley_var_0A, ridley_var_0B, ridley_var_0C, ridley_var_0D, ridley_var_0E,
            ridley_var_0F, ridley_var_10, ridley_var_11, ridley_var_12, ridley_var_13,
            ridley_var_14, ridley_var_15, ridley_var_16, ridley_var_17, ridley_var_18,
            ridley_var_19, ridley_var_1A, ridley_var_1B, ridley_var_1C, ridley_var_1D,
            ridley_var_1E, ridley_var_1F,
        ],
        pad2: [uint8; 1984],
        words: [
            ridley_var_20, ridley_var_21, ridley_var_22, ridley_var_23, ridley_var_24,
            ridley_var_25, ridley_var_26, ridley_var_27, ridley_var_28, ridley_var_29,
            ridley_var_2A, ridley_var_2B, ridley_var_2C, ridley_var_2D, ridley_var_2E,
            ridley_var_2F, ridley_var_30, ridley_var_31, ridley_var_32, ridley_var_33,
            ridley_var_34, ridley_var_35, ridley_var_36, ridley_var_37, ridley_var_38,
            ridley_var_39, ridley_var_3A, ridley_var_3B, ridley_var_3C, ridley_var_3D,
            ridley_var_3E, ridley_var_3F,
        ],
        pad3: [uint8; 1984],
        words: [
            ridley_var_40, ridley_var_41, ridley_var_42, ridley_var_43, ridley_var_44,
            ridley_var_45, ridley_var_46, ridley_var_47, ridley_var_48, ridley_var_49,
            ridley_var_4A, ridley_var_4B, ridley_var_4C, ridley_var_4D, ridley_var_4E,
            ridley_var_4F, ridley_var_50, ridley_var_51, ridley_var_52, ridley_var_53,
            ridley_var_54, ridley_var_55, ridley_var_56, ridley_var_57, ridley_var_58,
            ridley_var_59, ridley_var_5A, ridley_var_5B, ridley_var_5C, ridley_var_5D,
            ridley_var_5E, ridley_var_5F,
        ],
    }
}

enemy_layout! {
    Enemy_CeresDoor {
        base: EnemyBase,
        words: [
            cdr_var_A, cdr_var_B, cdr_var_C, cdr_var_D, cdr_var_E, cdr_var_F,
            cdr_parameter_1, cdr_parameter_2,
        ],
    }
}

enemy_layout! {
    Enemy_CeresSteam {
        base: EnemyBase,
        words: [
            csm_var_A, csm_var_B, csm_var_C, csm_var_D, csm_var_E, csm_var_F,
            csm_parameter_1, csm_parameter_2,
        ],
    }
}

enemy_layout! {
    Enemy_FakeKraid {
        base: EnemyBase,
        words: [
            fkd_var_A, fkd_var_B, fkd_var_C, fkd_var_D, fkd_var_E, fkd_var_F,
            fkd_parameter_1, fkd_parameter_2,
        ],
        pad1: [uint8; 26696],
        words: [
            fkd_var_00, fkd_var_01, fkd_var_02, fkd_var_03, fkd_var_04, fkd_var_05,
            fkd_var_06, fkd_var_07, fkd_var_08, fkd_var_09, fkd_var_0A, fkd_var_0B,
            fkd_var_0C, fkd_var_0D, fkd_var_0E, fkd_var_0F, fkd_var_10, fkd_var_11,
            fkd_var_12, fkd_var_13, fkd_var_14, fkd_var_15, fkd_var_16, fkd_var_17,
            fkd_var_18, fkd_var_19, fkd_var_1A, fkd_var_1B, fkd_var_1C, fkd_var_1D,
            fkd_var_1E, fkd_var_1F,
        ],
    }
}

enemy_layout! {
    Enemy_FireGeyser {
        base: EnemyBase,
        words: [
            fgr_var_A, fgr_var_B, fgr_var_C, fgr_var_D, fgr_var_E, fgr_var_F,
            fgr_parameter_1, fgr_parameter_2,
        ],
    }
}

enemy_layout! {
    Enemy_NuclearWaffle {
        base: EnemyBase,
        words: [
            nwe_var_A, nwe_var_B, nwe_var_C, nwe_var_D, nwe_var_E, nwe_var_F,
            nwe_parameter_1, nwe_parameter_2,
        ],
        pad1: [uint8; 26696],
        words: [
            nwe_var_00, nwe_var_01, nwe_var_02, nwe_var_03, nwe_var_04, nwe_var_05,
            nwe_var_06, nwe_var_07, nwe_var_08, nwe_var_09, nwe_var_0A, nwe_var_0B,
            nwe_var_0C, nwe_var_0D, nwe_var_0E, nwe_var_0F, nwe_var_10, nwe_var_11,
            nwe_var_12, nwe_var_13, nwe_var_14, nwe_var_15, nwe_var_16, nwe_var_17,
            nwe_var_18, nwe_var_19, nwe_var_1A, nwe_var_1B, nwe_var_1C, nwe_var_1D,
            nwe_var_1E, nwe_var_1F,
        ],
        pad2: [uint8; 1984],
        words: [
            nwe_var_20, nwe_var_21, nwe_var_22, nwe_var_23, nwe_var_24, nwe_var_25,
            nwe_var_26, nwe_var_27, nwe_var_28, nwe_var_29, nwe_var_2A, nwe_var_2B,
            nwe_var_2C, nwe_var_2D, nwe_var_2E, nwe_var_2F, nwe_var_30, nwe_var_31,
            nwe_var_32, nwe_var_33, nwe_var_34, nwe_var_35, nwe_var_36, nwe_var_37,
            nwe_var_38, nwe_var_39, nwe_var_3A, nwe_var_3B, nwe_var_3C, nwe_var_3D,
            nwe_var_3E, nwe_var_3F,
        ],
    }
}

enemy_layout! {
    Enemy_SpikeyPlatform {
        base: EnemyBase,
        words: [
            spm_var_A, spm_var_B, spm_var_C, spm_var_D, spm_var_E, spm_var_F,
            spm_parameter_1, spm_parameter_2,
        ],
        pad1: [uint8; 26696],
        words: [
            spm_var_00, spm_var_01, spm_var_02, spm_var_03, spm_var_04, spm_var_05,
            spm_var_06, spm_var_07, spm_var_08, spm_var_09, spm_var_0A, spm_var_0B,
            spm_var_0C, spm_var_0D, spm_var_0E, spm_var_0F, spm_var_10, spm_var_11,
            spm_var_12, spm_var_13, spm_var_14, spm_var_15, spm_var_16, spm_var_17,
            spm_var_18, spm_var_19, spm_var_1A, spm_var_1B, spm_var_1C, spm_var_1D,
            spm_var_1E, spm_var_1F,
        ],
    }
}

pub use crate::types::Enemy_MotherBrain;
enemy_layout! {
    Enemy_Boulder {
        base: EnemyBase,
        words: [
            boulder_var_A, boulder_var_B, boulder_var_C, boulder_var_D, boulder_var_E,
            boulder_var_F, boulder_parameter_1, boulder_parameter_2,
        ],
        pad1: [uint8; 26696],
        words: [
            boulder_var_00, boulder_var_01, boulder_var_02, boulder_var_03, boulder_var_04,
            boulder_var_05, boulder_var_06, boulder_var_07, boulder_var_08, boulder_var_09,
            boulder_var_0A, boulder_var_0B, boulder_var_0C, boulder_var_0D, boulder_var_0E,
            boulder_var_0F, boulder_var_10, boulder_var_11, boulder_var_12, boulder_var_13,
            boulder_var_14, boulder_var_15, boulder_var_16, boulder_var_17, boulder_var_18,
            boulder_var_19, boulder_var_1A, boulder_var_1B, boulder_var_1C, boulder_var_1D,
            boulder_var_1E, boulder_var_1F,
        ],
    }
}

enemy_layout! {
    Enemy_CeresRidley {
        base: EnemyBase,
        words: [
            cry_var_A, cry_var_B, cry_var_C, cry_var_D, cry_var_E, cry_var_F,
            cry_parameter_1, cry_parameter_2,
        ],
        pad1: [uint8; 26696],
        words: [
            cry_var_00, cry_var_01, cry_var_02, cry_var_03, cry_var_04, cry_var_05,
            cry_var_06, cry_var_07, cry_var_08, cry_var_09, cry_var_0A, cry_var_0B,
            cry_var_0C, cry_var_0D, cry_var_0E, cry_var_0F, cry_var_10, cry_var_11,
            cry_var_12, cry_var_13, cry_var_14, cry_var_15, cry_var_16, cry_var_17,
            cry_var_18, cry_var_19, cry_var_1A, cry_var_1B, cry_var_1C, cry_var_1D,
            cry_var_1E, cry_var_1F,
        ],
        pad2: [uint8; 1984],
        words: [
            cry_var_20, cry_var_21, cry_var_22, cry_var_23, cry_var_24, cry_var_25,
            cry_var_26, cry_var_27, cry_var_28, cry_var_29, cry_var_2A, cry_var_2B,
            cry_var_2C, cry_var_2D, cry_var_2E, cry_var_2F, cry_var_30, cry_var_31,
            cry_var_32, cry_var_33, cry_var_34, cry_var_35, cry_var_36, cry_var_37,
            cry_var_38, cry_var_39, cry_var_3A, cry_var_3B, cry_var_3C, cry_var_3D,
            cry_var_3E, cry_var_3F,
        ],
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct BabyMetroidExecState {
    pub ip: uint16,
    pub timer: uint16,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Ram8800_Default {
    pub var_40: uint16,
    pub var_41: uint16,
    pub var_42: uint16,
    pub var_43: uint16,
    pub var_44: uint16,
    pub var_45: uint16,
    pub var_46: uint16,
    pub var_47: uint16,
    pub var_48: uint16,
    pub var_49: uint16,
    pub var_4A: uint16,
    pub var_4B: uint16,
    pub var_4C: uint16,
    pub var_4D: uint16,
    pub var_4E: uint16,
    pub var_4F: uint16,
    pub var_50: uint16,
    pub var_51: uint16,
    pub var_52: uint16,
    pub var_53: uint16,
    pub var_54: uint16,
    pub var_55: uint16,
    pub var_56: uint16,
    pub var_57: uint16,
    pub var_58: uint16,
    pub var_59: uint16,
    pub var_5A: uint16,
    pub var_5B: uint16,
    pub var_5C: uint16,
    pub var_5D: uint16,
    pub var_5E: uint16,
    pub var_5F: uint16,
}
enemy_layout! {
    Enemy_RidleysExplosion {
        base: EnemyBase,
        words: [
            ren_var_A, ren_var_B, ren_var_C, ren_var_D, ren_var_E, ren_var_F,
            ren_parameter_1, ren_parameter_2,
        ],
    }
}

enemy_layout! {
    Enemy_Zebetites {
        base: EnemyBase,
        words: [
            zebet_var_A, zebet_var_B, zebet_var_C, zebet_var_D, zebet_var_E, zebet_var_F,
            zebet_parameter_1, zebet_parameter_2,
        ],
    }
}

pub const true_0: ::core::ffi::c_int = 1;
pub const false_0: ::core::ffi::c_int = 0;
#[inline]
unsafe fn RomPtr_A0(addr: uint16_t, game: &mut Game) -> *const uint8 {
    RomPtr((0xa00000u32) | ((addr) as u32), game)
}
#[inline]
unsafe fn RomPtr_A6(addr: uint16_t, game: &mut Game) -> *const uint8 {
    RomPtr((0xa60000u32) | ((addr) as u32), game)
}
#[inline]
unsafe fn gOamEnt(v: ::core::ffi::c_int, game: &mut Game) -> *mut OamEnt {
    (&raw mut game.g_ram as *mut uint8).offset((0x370 as ::core::ffi::c_int + v) as isize)
        as *mut uint8 as *mut OamEnt
}
#[inline]
unsafe fn gVramWriteEntry(a: ::core::ffi::c_int, game: &mut Game) -> *mut VramWriteEntry {
    (&raw mut game.g_ram as *mut uint8).offset((0xd0 as ::core::ffi::c_int + a) as isize)
        as *mut uint8 as *mut VramWriteEntry
}
pub const fnBoulder_Func_1: ::core::ffi::c_int = 0xa6879a;
pub const fnBoulder_Func_2: ::core::ffi::c_int = 0xa687ed;
pub const fnBoulder_Func_3: ::core::ffi::c_int = 0xa68832;
pub const fnBoulder_Func_4: ::core::ffi::c_int = 0xa6888b;
pub const fnBoulder_Func_5: ::core::ffi::c_int = 0xa68942;
pub const fnBoulder_Func_6: ::core::ffi::c_int = 0xa689fc;
pub const fnSpikeyPlatform_Func_1: ::core::ffi::c_int = 0xa68bb4;
pub const fnSpikeyPlatform_Func_2: ::core::ffi::c_int = 0xa68bdc;
pub const fnSpikeyPlatform_Func_3: ::core::ffi::c_int = 0xa68c4a;
pub const fnSpikeyPlatform_Func_4: ::core::ffi::c_int = 0xa68c5d;
pub const fnFireGeyser_Func_1: ::core::ffi::c_int = 0xa6902f;
pub const fnFireGeyser_Func_2: ::core::ffi::c_int = 0xa69062;
pub const fnNuclearWaffle_Func_1: ::core::ffi::c_int = 0xa69615;
pub const fnNuclearWaffle_Func_2: ::core::ffi::c_int = 0xa69682;
pub const fnCeresRidley_Func_2: ::core::ffi::c_int = 0xa6a354;
pub const fnCeresRidley_Func_3: ::core::ffi::c_int = 0xa6a35b;
pub const fnCeresRidley_A377: ::core::ffi::c_int = 0xa6a377;
pub const fnCeresRidley_Func_4: ::core::ffi::c_int = 0xa6a389;
pub const fnCeresRidley_Func_5: ::core::ffi::c_int = 0xa6a3df;
pub const fnCeresRidley_Func_6: ::core::ffi::c_int = 0xa6a455;
pub const fnCeresRidley_Func_7: ::core::ffi::c_int = 0xa6a478;
pub const fnCeresRidley_Func_9: ::core::ffi::c_int = 0xa6a6af;
pub const fnCeresRidley_Func_10: ::core::ffi::c_int = 0xa6a6c8;
pub const fnCeresRidley_Func_11: ::core::ffi::c_int = 0xa6a6e8;
pub const fnCeresRidley_Func_13: uint32 = 10921858;
pub const fnCeresRidley_A7F9: uint32 = 10921977;
pub const fnCeresRidley_Func_14: uint32 = 10922044;
pub const fnCeresRidley_Func_15: ::core::ffi::c_int = 0xa6a84e;
pub const fnCeresRidley_Func_16: ::core::ffi::c_int = 0xa6a88d;
pub const fnCeresRidley_Func_17: ::core::ffi::c_int = 0xa6a8a4;
pub const fnCeresRidley_Func_18: ::core::ffi::c_int = 0xa6a8d4;
pub const fnCeresRidley_Func_19: ::core::ffi::c_int = 0xa6a8f8;
pub const fnCeresRidley_Func_20: ::core::ffi::c_int = 0xa6a923;
pub const fnCeresRidley_Func_21: ::core::ffi::c_int = 0xa6a947;
pub const fnCeresRidley_Func_22: ::core::ffi::c_int = 0xa6a971;
pub const fnCeresRidley_Func_23: ::core::ffi::c_int = 0xa6a9a0;
pub const fnCeresRidley_Func_24: ::core::ffi::c_int = 0xa6aa11;
pub const fnnullsub_233: ::core::ffi::c_int = 0xa6aa4f;
pub const fnCeresRidley_Func_26: ::core::ffi::c_int = 0xa6aa50;
pub const fnRidley_Func_3: ::core::ffi::c_int = 0xa6b2f3;
pub const fnRidley_Func_3b: ::core::ffi::c_int = 0xa6b321;
pub const fnRidley_Func_5: ::core::ffi::c_int = 0xa6b3ec;
pub const fnRidley_Func_6: ::core::ffi::c_int = 0xa6b3f8;
pub const fnRidley_Func_8: uint32 = 10925121;
pub const fnRidley_B455: uint32 = 10925141;
pub const fnRidley_Func_9: ::core::ffi::c_int = 0xa6b493;
pub const fnRidley_Func_10: ::core::ffi::c_int = 0xa6b4d1;
pub const fnRidley_Func_11: ::core::ffi::c_int = 0xa6b516;
pub const fnRidley_Func_12: ::core::ffi::c_int = 0xa6b554;
pub const fnRidley_Func_13: ::core::ffi::c_int = 0xa6b594;
pub const fnRidley_Func_14: uint32 = 10925508;
pub const fnRidley_Func_15: ::core::ffi::c_int = 0xa6b5e5;
pub const fnRidley_Func_16: ::core::ffi::c_int = 0xa6b613;
pub const fnRidley_Func_19: ::core::ffi::c_int = 0xa6b6a7;
pub const fnRidley_Func_20: ::core::ffi::c_int = 0xa6b6dd;
pub const fnRidley_Func_21: ::core::ffi::c_int = 0xa6b70e;
pub const fnRidley_Func_22: ::core::ffi::c_int = 0xa6b7b9;
pub const fnRidley_Func_33: ::core::ffi::c_int = 0xa6bab7;
pub const fnRidley_Func_33b: ::core::ffi::c_int = 0xa6bb8f;
pub const fnRidley_Func_34: ::core::ffi::c_int = 0xa6bbc4;
pub const fnRidley_Func_35: ::core::ffi::c_int = 0xa6bbf1;
pub const fnRidley_Func_36: ::core::ffi::c_int = 0xa6bc2e;
pub const fnRidley_Func_43: ::core::ffi::c_int = 0xa6bd4e;
pub const fnRidley_Func_44: ::core::ffi::c_int = 0xa6bd9a;
pub const fnRidley_Func_45: ::core::ffi::c_int = 0xa6bdbc;
pub const fnRidley_Func_46: ::core::ffi::c_int = 0xa6bdf2;
pub const fnRidley_Func_47: ::core::ffi::c_int = 0xa6be03;
pub const fnRidley_Func_50: ::core::ffi::c_int = 0xa6be9c;
pub const fnRidley_Func_51: ::core::ffi::c_int = 0xa6beb3;
pub const fnRidley_Func_52: ::core::ffi::c_int = 0xa6beca;
pub const fnRidley_Func_53: ::core::ffi::c_int = 0xa6bedc;
pub const fnnullsub_348: uint32 = 10927897;
pub const fnBabyMetroid_Instr_2: uint32 = 10928073;
pub const fnBabyMetroid_Instr_3: uint32 = 10928097;
pub const fnBabyMetroid_Instr_1: uint32 = 10928114;
pub const fnBabyMetroid_Instr_4: uint32 = 10928120;
pub const fnRidley_Func_54: ::core::ffi::c_int = 0xa6c04e;
pub const fnRidley_Func_63: ::core::ffi::c_int = 0xa6c538;
pub const fnRidley_C53E: ::core::ffi::c_int = 0xa6c53e;
pub const fnRidley_C551: ::core::ffi::c_int = 0xa6c551;
pub const fnRidley_Func_64: ::core::ffi::c_int = 0xa6c588;
pub const fnRidley_Func_65: ::core::ffi::c_int = 0xa6c5a8;
pub const fnRidley_Func_66: ::core::ffi::c_int = 0xa6c5c8;
pub const fnRidley_Func_67: ::core::ffi::c_int = 0xa6c5da;
pub const fnnullsub_349: uint32 = 10929664;
pub const fnnullsub_350: uint32 = 10940404;
pub const fnCeresSteam_Func_1: uint32 = 10940441;
pub const fnCeresDoor_Func_2: uint32 = 10942315;
pub const fnCeresDoor_Func_3: uint32 = 10942320;
pub const fnCeresDoor_Func_4: uint32 = 10942373;
pub const fnCeresDoor_Func_5: uint32 = 10942397;
pub const fnCeresDoor_Func_6: ::core::ffi::c_int = 0xa6f7dc;
pub const fnCeresDoor_Func_7: ::core::ffi::c_int = 0xa6f850;
pub const fnZebetites_Func_1: ::core::ffi::c_int = 0xa6fc41;
pub const fnZebetites_Func_2: ::core::ffi::c_int = 0xa6fc5b;
pub const fnZebetites_Func_3: ::core::ffi::c_int = 0xa6fc67;
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
unsafe fn gRam8800_Default(o: ::core::ffi::c_int, game: &mut Game) -> *mut Ram8800_Default {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0x8800)
        .offset(o as isize) as *mut Ram8800_Default
}
#[inline]
unsafe fn Get_Boulder(o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_Boulder {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_Boulder
}
#[inline]
unsafe fn Get_CeresDoor(o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_CeresDoor {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_CeresDoor
}
#[inline]
unsafe fn Get_CeresRidley(o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_CeresRidley {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_CeresRidley
}
#[inline]
unsafe fn Get_CeresSteam(o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_CeresSteam {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_CeresSteam
}
#[inline]
unsafe fn Get_FakeKraid(o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_FakeKraid {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_FakeKraid
}
#[inline]
unsafe fn Get_FireGeyser(o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_FireGeyser {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_FireGeyser
}
#[inline]
unsafe fn Get_MotherBrain(o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_MotherBrain {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_MotherBrain
}
#[inline]
unsafe fn Get_NuclearWaffle(o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_NuclearWaffle {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_NuclearWaffle
}
#[inline]
unsafe fn Get_Ridley(o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_Ridley {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_Ridley
}
#[inline]
unsafe fn Get_RidleysExplosion(
    o: ::core::ffi::c_int,
    game: &mut Game,
) -> *mut Enemy_RidleysExplosion {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_RidleysExplosion
}
#[inline]
unsafe fn Get_SpikeyPlatform(o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_SpikeyPlatform {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_SpikeyPlatform
}
#[inline]
unsafe fn Get_Zebetites(o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_Zebetites {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_Zebetites
}
static g_word_A686F1: [uint16; 2] = [0x1000, 0x1800];
static g_word_A68DBB: uint16 = 5 as uint16;
static g_word_A68DBD: uint16 = 0xa as uint16;
static g_word_A68DBF: uint16 = 0xf as uint16;
static g_word_A68DC1: uint16 = 0x14 as uint16;
static g_word_A68DC3: uint16 = 0x19 as uint16;
static g_word_A68DC5: uint16 = 0x1e as uint16;
static g_word_A68DC7: uint16 = 0x23 as uint16;
static g_word_A68DC9: uint16 = 0x28 as uint16;
static g_word_A68DCB: uint16 = 0x2d as uint16;
static g_word_A68DCD: uint16 = 0x32 as uint16;
static g_word_A68DCF: uint16 = 0x37 as uint16;
static g_word_A68DD1: uint16 = 0x3c as uint16;
static g_word_A68DD3: uint16 = 0x41 as uint16;
static g_word_A68DD5: uint16 = 0x46 as uint16;
static g_word_A68DD7: uint16 = 0x4b as uint16;
static g_word_A68DD9: uint16 = 0x50 as uint16;
static g_word_A68DDB: uint16 = 0x55 as uint16;
static g_word_A68DDD: uint16 = 0x5a as uint16;
static g_word_A68DDF: uint16 = 0x5f as uint16;
static g_word_A68DE1: uint16 = 0x64 as uint16;
static g_word_A68DE3: uint16 = 0x69 as uint16;
static g_word_A68DE5: uint16 = 0x6e as uint16;
static g_word_A68DE7: uint16 = 0x18 as uint16;
static g_word_A68DE9: uint16 = 0x18 as uint16;
static g_word_A68DEB: uint16 = 0x18 as uint16;
static g_word_A68DED: uint16 = 0x18 as uint16;
static g_word_A68DEF: uint16 = 0x18 as uint16;
static g_word_A68DF1: uint16 = 0x18 as uint16;
static g_word_A68DF3: uint16 = 0x18 as uint16;
static g_word_A68DF5: uint16 = 0x18 as uint16;
static g_word_A68DF7: uint16 = 0x18 as uint16;
static g_word_A68DF9: uint16 = 0x18 as uint16;
static g_word_A68DFB: uint16 = 0x18 as uint16;
static g_word_A68DFD: uint16 = 0x18 as uint16;
static g_word_A68DFF: uint16 = 0x18 as uint16;
static g_word_A68E01: uint16 = 0x18 as uint16;
static g_word_A68E03: uint16 = 0x18 as uint16;
static g_word_A68E05: uint16 = 0x18 as uint16;
static g_word_A68E07: uint16 = 0x18 as uint16;
static g_word_A68E09: uint16 = 0x18 as uint16;
static g_word_A68E0B: uint16 = 0x14 as uint16;
static g_word_A68E0D: uint16 = 0x10 as uint16;
static g_word_A68E0F: uint16 = 0xc as uint16;
static g_word_A68E11: uint16 = 8 as uint16;
static g_word_A695F6: [uint16; 4] = [0x190, 0xf0, 0xf0, 0x190];
static g_word_A695FE: [uint16; 4] = [0xffe8, 0xfff4, 0x18, 0xc];
static g_word_A69606: [uint16; 4] = [0x180, 0x100, 0x100, 0x180];
static g_word_A69A48: [uint16; 8] = [0xfe00, 0xfb00, 0xfc00, 0xfb00, 0x200, 0xfb00, 0x400, 0xfb00];
static g_word_A6B288: uint16 = 8 as uint16;
static g_word_A6BBEB: [uint16; 3] = [0x40, 0, 0xd0];
static g_word_A6BC62: [uint16; 3] = [0xb0, 0, 0x50];
static g_word_A6C66E: [int16; 20] = [
    -(24),
    -(24),
    -(20),
    20,
    16,
    -(30),
    30,
    -(3),
    14,
    -(13),
    -(2),
    18,
    -(2),
    -(32),
    -(31),
    8,
    -(4),
    -(10),
    19,
    19,
];
static g_word_A6CF54: [int16; 3] = [0x20, 0, -(0x20)];
static g_byte_A6D61F: [uint8; 16] = [
    0x10, 0xf, 0xe, 0xd, 0xc, 0xb, 0xa, 9, 8, 7, 6, 5, 4, 3, 2, 1,
];
static g_byte_A6D712: [uint8; 16] = [
    0x10, 0xf, 0xe, 0xd, 0xc, 0xb, 0xa, 9, 8, 7, 6, 5, 4, 3, 2, 1,
];
static g_word_A6D9ED: [uint16; 8] = [0xc, 0xe, 0x10, 0x12, 0x1c, 0x20, 0x28, 0x30];
pub unsafe fn Enemy_GrappleReact_NoInteract_A6(game: &mut Game) {
    SwitchEnemyAiToMainAi(game);
}
pub unsafe fn Enemy_GrappleReact_CancelBeam_A6(game: &mut Game) {
    Enemy_SwitchToFrozenAi(game);
}
pub unsafe fn Enemy_NormalTouchAI_A6(game: &mut Game) {
    NormalEnemyTouchAi(game);
}
pub unsafe fn Enemy_NormalShotAI_A6(game: &mut Game) {
    NormalEnemyShotAi(game);
}
pub unsafe fn Enemy_NormalFrozenAI_A6(game: &mut Game) {
    NormalEnemyFrozenAI(game);
}
pub unsafe fn Boulder_Init(game: &mut Game) {
    let mut boulder_parameter_2_high: int16 = 0;
    let E: *mut Enemy_Boulder = Get_Boulder(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).boulder_var_C = 0 as uint16;
    (*E).boulder_var_B = 0 as uint16;
    (*E).boulder_var_D = 2 as uint16;
    (*E).boulder_var_A = ((0xa6879ai32) as u16);
    (*E).boulder_var_04 = (*E).base.x_pos;
    let y_pos: uint16 = (*E).base.y_pos;
    (*E).boulder_var_05 = y_pos;
    (*E).boulder_var_03 = y_pos;
    (*E).boulder_var_02 = (*E).base.y_subpos;
    boulder_parameter_2_high = *(&raw mut (*E).boulder_parameter_2 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>())
            .wrapping_div(::core::mem::size_of::<uint8>())
            .wrapping_sub(1_usize) as isize,
    ) as int16;
    if *(&raw mut (*E).boulder_parameter_2 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>())
            .wrapping_div(::core::mem::size_of::<uint8>())
            .wrapping_sub(1_usize) as isize,
    ) == 0
    {
        boulder_parameter_2_high = 1 as int16;
        (*E).boulder_var_07 = 1 as uint16;
    }
    (*E).base.y_pos = ((*E).boulder_var_05).wrapping_sub((boulder_parameter_2_high) as u16);
    (*E).boulder_var_00 = -(*(&raw mut (*E).boulder_parameter_2 as *mut uint8).offset(0)
        as ::core::ffi::c_int) as uint16;
    (*E).boulder_var_06 =
        *(&raw mut (*E).base.current_instruction as *mut uint8).offset(0) as uint16;
    (*E).base.current_instruction = addr_kBoulder_Ilist_86A7 as uint16;
    let boulder_parameter_1_high: uint16 = *(&raw mut (*E).boulder_parameter_1 as *mut uint8)
        .offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1_usize) as isize,
        ) as uint16;
    (*E).boulder_var_E = boulder_parameter_1_high;
    if boulder_parameter_1_high == 0 {
        (*E).boulder_var_00 = -((*E).boulder_var_00 as ::core::ffi::c_int) as uint16;
        (*E).base.current_instruction = addr_kBoulder_Ilist_86CB as uint16;
    }
    (*E).boulder_var_01 = 2 as uint16;
    if *(&raw mut (*E).boulder_parameter_1 as *mut uint8).offset(0) != 0 {
        (*E).boulder_var_01 = 5 as uint16;
    }
}
pub unsafe fn CallBoulderFunc(ea: uint32, game: &mut Game) {
    match ea {
        10913690 => {
            Boulder_Func_1(game);
        }
        10913773 => {
            Boulder_Func_2(game);
        }
        10913842 => {
            Boulder_Func_3(game);
        }
        10913931 => {
            Boulder_Func_4(game);
        }
        10914114 => {
            Boulder_Func_5(game);
        }
        10914300 => {
            Boulder_Func_6();
        }
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn Boulder_Main(game: &mut Game) {
    let E: *mut Enemy_Boulder = Get_Boulder(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    CallBoulderFunc((((*E).boulder_var_A) as u32) | (0xa60000u32), game);
}
pub unsafe fn Boulder_Func_1(game: &mut Game) {
    let mut SamusEnemyDelta_Y: int16 = 0;
    let mut v3: int16 = 0;
    let mut SamusEnemyDelta_X: int16 = 0;
    SamusEnemyDelta_Y = GetSamusEnemyDelta_Y(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    ) as int16;
    if SamusEnemyDelta_Y >= 0i16 {
        let E: *mut Enemy_Boulder = Get_Boulder(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        );
        if (SamusEnemyDelta_Y).wrapping_sub(((*E).boulder_var_06) as i16) < 0i16 {
            if (*E).boulder_var_E != 0 {
                SamusEnemyDelta_X = GetSamusEnemyDelta_X(
                    *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
                    game,
                ) as int16;
                if SamusEnemyDelta_X < 0i16
                    && (SamusEnemyDelta_X).wrapping_sub(((*E).boulder_var_00) as i16) >= 0i16
                {
                    (*E).boulder_var_A = ((0xa687edi32) as u16);
                    if (*E).boulder_var_07 != 0 {
                        (*E).boulder_var_A = ((0xa68942i32) as u16);
                    }
                }
            } else {
                v3 = GetSamusEnemyDelta_X(
                    *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
                    game,
                ) as int16;
                if v3 >= 0i16 && (v3).wrapping_sub(((*E).boulder_var_00) as i16) < 0i16 {
                    (*E).boulder_var_A = ((0xa687edi32) as u16);
                    if (*E).boulder_var_07 != 0 {
                        (*E).boulder_var_A = ((0xa68942i32) as u16);
                    }
                }
            }
        }
    }
}
pub unsafe fn Boulder_Func_2(game: &mut Game) {
    let E: *mut Enemy_Boulder = Get_Boulder(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    Boulder_Func_9(
        (8u16).wrapping_mul(
            (*(&raw mut (*E).boulder_var_C as *mut uint8).offset(
                (::core::mem::size_of::<uint16>())
                    .wrapping_div(::core::mem::size_of::<uint8>())
                    .wrapping_sub(1_usize) as isize,
            )) as u16,
        ),
        game,
    );
    if (((*E).base.y_pos) as i16).wrapping_sub(((*E).boulder_var_05) as i16) < 0i16 {
        let v1: uint16 = ((*E).boulder_var_C).wrapping_add(256u16);
        (*E).boulder_var_C = v1;
        if v1 as ::core::ffi::c_int - 20480 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            == 0
        {
            (*E).boulder_var_C = 20480 as uint16;
        }
    } else {
        (*E).base.y_pos = (*E).boulder_var_05;
        (*E).boulder_var_A = ((0xa68832i32) as u16);
        (*E).boulder_var_C = 0x2000 as uint16;
    };
}
pub unsafe fn Boulder_Func_3(game: &mut Game) {
    let mut v1: int16 = 0;
    let E: *mut Enemy_Boulder = Get_Boulder(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    Boulder_Func_9(
        (8 as ::core::ffi::c_int
            * *(&raw mut (*E).boulder_var_C as *mut uint8).offset(
                (::core::mem::size_of::<uint16>())
                    .wrapping_div(::core::mem::size_of::<uint8>())
                    .wrapping_sub(1_usize) as isize,
            ) as ::core::ffi::c_int
            + 4 as ::core::ffi::c_int) as uint16,
        game,
    );
    v1 = (((*E).boulder_var_C) as i16).wrapping_sub(256i16);
    (*E).boulder_var_C = v1 as uint16;
    if v1 >= 0i16 {
        let mut v2: uint16 = (8u16).wrapping_mul(
            (*(&raw mut (*E).boulder_var_B as *mut uint8).offset(
                (::core::mem::size_of::<uint16>())
                    .wrapping_div(::core::mem::size_of::<uint8>())
                    .wrapping_sub(1_usize) as isize,
            )) as u16,
        );
        if (*E).boulder_var_E != 0 {
            v2 = (v2).wrapping_add(4u16);
        }
        Boulder_Func_7(v2, game);
        let v3: uint16 = ((*E).boulder_var_B).wrapping_add(32u16);
        (*E).boulder_var_B = v3;
        if v3 as ::core::ffi::c_int - 20480 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            == 0
        {
            (*E).boulder_var_B = 20480 as uint16;
        }
    } else {
        (*E).boulder_var_C = 0 as uint16;
        (*E).boulder_var_A = ((0xa6888bi32) as u16);
    };
}
pub unsafe fn Boulder_Func_4(game: &mut Game) {
    let E: *mut Enemy_Boulder = Get_Boulder(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let v1: ::core::ffi::c_int = 8 as ::core::ffi::c_int
        * *(&raw mut (*E).boulder_var_C as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1_usize) as isize,
        ) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    if Enemy_MoveDown(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        *(game.g_rom.offset(crate::rtl::lorom_offset(0xa0cbc7)) as *const uint8 as *mut uint32)
            .offset((v1 >> 1 as ::core::ffi::c_int) as isize) as int32,
        game,
    ) != 0
    {
        QueueSfx2_Max6(0x42 as uint16, game);
        if (*E).boulder_var_E == 2u16 {
            (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Deleted) as u16);
            game.eproj_spawn_pt = Point16U {
                x: (*E).base.x_pos,
                y: (*E).base.y_pos,
            };
            SpawnEprojWithRoomGfx(
                addr_kEproj_DustCloudExplosion as uint16,
                0x11 as uint16,
                game,
            );
            QueueSfx2_Max6(0x43 as uint16, game);
        } else {
            (*E).boulder_var_A = ((0xa68832i32) as u16);
            (*E).boulder_var_C = g_word_A686F1
                [((*E).boulder_var_D as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as usize];
            (*E).boulder_var_D = (*E).boulder_var_D.wrapping_sub(1);
            if ((*E).boulder_var_D as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
                (*E).boulder_var_03 = (*E).base.y_pos;
                (*E).boulder_var_02 = (*E).base.y_subpos;
                (*E).boulder_var_A = ((0xa68942i32) as u16);
            }
        }
    } else {
        (*E).boulder_var_C = ((*E).boulder_var_C).wrapping_add(256u16);
        let mut v3: uint16 = (8u16).wrapping_mul(
            (*(&raw mut (*E).boulder_var_B as *mut uint8).offset(
                (::core::mem::size_of::<uint16>())
                    .wrapping_div(::core::mem::size_of::<uint8>())
                    .wrapping_sub(1_usize) as isize,
            )) as u16,
        );
        if (*E).boulder_var_E != 0 {
            v3 = (v3).wrapping_add(4u16);
        }
        Boulder_Func_7(v3, game);
        let v4: uint16 = ((*E).boulder_var_B).wrapping_add(32u16);
        (*E).boulder_var_B = v4;
        if v4 as ::core::ffi::c_int - 20480 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            == 0
        {
            (*E).boulder_var_B = 20480 as uint16;
        }
    };
}
pub unsafe fn Boulder_Func_5(game: &mut Game) {
    let E: *mut Enemy_Boulder = Get_Boulder(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut v2: uint16 = (8u16).wrapping_mul(
        (*(&raw mut (*E).boulder_var_B as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1_usize) as isize,
        )) as u16,
    );
    let v3: ::core::ffi::c_int = v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    Enemy_MoveDown(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0xa0cbc7)) as *const uint8 as *mut uint32)
            .offset((v3 >> 1 as ::core::ffi::c_int) as isize))
        .wrapping_add(
            (((*E).boulder_var_01 as ::core::ffi::c_int) << 16 as ::core::ffi::c_int) as uint32,
        ) as int32,
        game,
    );
    (*E).base.y_pos = ((*E).base.y_pos).wrapping_sub((*E).boulder_var_01);
    if (*E).boulder_var_E != 0 {
        v2 = (v2).wrapping_add(4u16);
    }
    let v4: ::core::ffi::c_int = v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if Enemy_MoveRight_IgnoreSlopes(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        *(game.g_rom.offset(crate::rtl::lorom_offset(0xa0cbc7)) as *const uint8 as *mut uint32)
            .offset((v4 >> 1 as ::core::ffi::c_int) as isize) as int32,
        game,
    ) != 0
    {
        (*E).base.properties = ((*E).base.properties) | (0x300u16);
        (*E).boulder_var_A = ((0xa689fci32) as u16);
        QueueSfx2_Max6(0x42 as uint16, game);
        game.eproj_spawn_pt = Point16U {
            x: (*E).base.x_pos,
            y: (*E).base.y_pos,
        };
        SpawnEprojWithRoomGfx(
            addr_kEproj_DustCloudExplosion as uint16,
            0x11 as uint16,
            game,
        );
        QueueSfx2_Max6(0x43 as uint16, game);
    } else {
        let v5: uint16 = ((*E).boulder_var_B).wrapping_add(64u16);
        (*E).boulder_var_B = v5;
        if v5 as ::core::ffi::c_int - 0x4000 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            == 0
        {
            (*E).boulder_var_B = 0x4000 as uint16;
        }
        if (*E).base.y_pos == (*E).boulder_var_03 && (*E).base.y_subpos == (*E).boulder_var_02 {
            (*E).boulder_var_01 = 0 as uint16;
        }
    }
    (*E).boulder_var_03 = (*E).base.y_pos;
    (*E).boulder_var_02 = (*E).base.y_subpos;
}
pub fn Boulder_Func_6() {}
pub unsafe fn Boulder_Func_7(j: uint16, game: &mut Game) {
    let E: *mut Enemy_Boulder = Get_Boulder(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let v3: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0xa2838f)) as *const uint8 as *mut uint16)
            .offset((v3 + 1 as ::core::ffi::c_int) as isize) as uint32)
            << 16 as ::core::ffi::c_int
            | *(game.g_rom.offset(crate::rtl::lorom_offset(0xa2838f)) as *const uint8
                as *mut uint16)
                .offset(v3 as isize) as uint32,
    );
}
pub unsafe fn Boulder_Func_8(j: uint16, game: &mut Game) {
    let E: *mut Enemy_Boulder = Get_Boulder(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let v3: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8 as *mut uint16)
            .offset((v3 + 1 as ::core::ffi::c_int) as isize) as uint32)
            << 16 as ::core::ffi::c_int
            | *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8
                as *mut uint16)
                .offset(v3 as isize) as uint32,
    );
}
pub unsafe fn Boulder_Func_9(j: uint16, game: &mut Game) {
    let E: *mut Enemy_Boulder = Get_Boulder(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let v3: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0xa2838f)) as *const uint8 as *mut uint16)
            .offset((v3 + 1 as ::core::ffi::c_int) as isize) as uint32)
            << 16 as ::core::ffi::c_int
            | *(game.g_rom.offset(crate::rtl::lorom_offset(0xa2838f)) as *const uint8
                as *mut uint16)
                .offset(v3 as isize) as uint32,
    );
}
pub unsafe fn SpikeyPlatform_Init(game: &mut Game) {
    let E: *mut Enemy_SpikeyPlatform = Get_SpikeyPlatform(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.current_instruction = addr_kSpikeyPlatform_Ilist_8B29 as uint16;
    (*E).spm_var_A = ((0xa68bb4i32) as u16);
    let v1: uint16 =
        (8u16).wrapping_mul((*(&raw mut (*E).spm_parameter_1 as *mut uint8).offset(0)) as u16);
    (*E).spm_var_02 = v1;
    let v2: ::core::ffi::c_int = v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    (*E).spm_var_E = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8
        as *mut uint16)
        .offset((v2 + 1 as ::core::ffi::c_int) as isize);
    (*E).spm_var_F = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8
        as *mut uint16)
        .offset(v2 as isize);
    (*E).spm_var_D = ((*E).base.y_pos).wrapping_add(
        (*(&raw mut (*E).spm_parameter_2 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1_usize) as isize,
        )) as u16,
    );
    (*E).spm_var_C = (*E).base.y_pos;
    let spm_parameter_2_low: uint16 =
        *(&raw mut (*E).spm_parameter_2 as *mut uint8).offset(0) as uint16;
    (*E).spm_var_00 = spm_parameter_2_low;
    (*E).spm_var_B = spm_parameter_2_low;
}
pub unsafe fn SpikeyPlatform2ndEnemy_Init(game: &mut Game) {
    let v0: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
        as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    let v1: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*v1).x_pos = *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
        .offset((v0 + 91 as ::core::ffi::c_int) as isize);
    (*v1).y_pos = (*((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
        .offset((v0 + 93 as ::core::ffi::c_int) as isize))
    .wrapping_add(12u16);
}
pub unsafe fn SpikeyPlatform2ndEnemy_Main(game: &mut Game) {
    let v0: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
        as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    let v1: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*v1).x_pos = *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
        .offset((v0 + 91 as ::core::ffi::c_int) as isize);
    (*v1).y_pos = (*((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
        .offset((v0 + 93 as ::core::ffi::c_int) as isize))
    .wrapping_add(12u16);
}
pub unsafe fn CallSpikeyPlatformFunc(ea: uint32, k: uint16, game: &mut Game) {
    match ea {
        10914740 => {
            SpikeyPlatform_Func_1(game);
        }
        10914780 => {
            SpikeyPlatform_Func_2(game);
        }
        10914890 => {
            SpikeyPlatform_Func_3(k, game);
        }
        10914909 => {
            SpikeyPlatform_Func_4(game);
        }
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn SpikeyPlatform_Main(game: &mut Game) {
    let E: *mut Enemy_SpikeyPlatform = Get_SpikeyPlatform(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    CallSpikeyPlatformFunc(
        (((*E).spm_var_A) as u32) | (0xa60000u32),
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
}
pub unsafe fn SpikeyPlatform_Func_1(game: &mut Game) {
    let E: *mut Enemy_SpikeyPlatform = Get_SpikeyPlatform(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).spm_var_B = (*E).spm_var_B.wrapping_sub(1);
    if (*E).spm_var_B == 0 {
        (*E).spm_var_B = (*E).spm_var_00;
        (*E).spm_var_02 =
            (8u16).wrapping_mul((*(&raw mut (*E).spm_parameter_1 as *mut uint8).offset(0)) as u16);
        (*E).spm_var_A = ((0xa68bdci32) as u16);
    }
}
pub unsafe fn SpikeyPlatform_Func_2(game: &mut Game) {
    let E: *mut Enemy_SpikeyPlatform = Get_SpikeyPlatform(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).spm_var_01 = (*E).base.y_pos;
    let v1: ::core::ffi::c_int = (*E).spm_var_02 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8 as *mut uint16)
            .offset(v1 as isize) as uint32)
            << 16 as ::core::ffi::c_int
            | *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8
                as *mut uint16)
                .offset((v1 + 1 as ::core::ffi::c_int) as isize) as uint32,
    );
    if (((*E).base.y_pos) as i16).wrapping_sub(((*E).spm_var_D) as i16) >= 0i16 {
        (*E).spm_var_03 = 64 as uint16;
        (*E).spm_var_A = ((0xa68c4ai32) as u16);
        (*E).base.y_pos = (*E).spm_var_D;
        QueueSfx2_Max6(0x1b as uint16, game);
    }
    if SpikeyPlatform_Func_5(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    ) != 0
    {
        let ref mut fresh17 = *((&raw mut game.g_ram as *mut uint8).offset(0xb5c) as *mut uint16);
        *fresh17 = (*fresh17 as ::core::ffi::c_int
            + ((*E).base.y_pos as ::core::ffi::c_int - (*E).spm_var_01 as ::core::ffi::c_int))
            as uint16;
    }
    let mut v4: int16 = (((*E).spm_var_02) as i16).wrapping_add(8i16);
    if (*E).spm_var_02 as ::core::ffi::c_int - 504 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        v4 = 512 as int16;
    }
    (*E).spm_var_02 = v4 as uint16;
}
pub unsafe fn SpikeyPlatform_Func_3(k: uint16, game: &mut Game) {
    let E: *mut Enemy_SpikeyPlatform = Get_SpikeyPlatform(k as ::core::ffi::c_int, game);
    let v2: uint16 = ((*E).spm_var_03).wrapping_sub(1u16);
    (*E).spm_var_03 = v2;
    if v2 == 0 {
        (*E).spm_var_A = ((0xa68c5di32) as u16);
    }
}
pub unsafe fn SpikeyPlatform_Func_4(game: &mut Game) {
    let E: *mut Enemy_SpikeyPlatform = Get_SpikeyPlatform(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).spm_var_01 = (*E).base.y_pos;
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        -(0x8000 as ::core::ffi::c_int) as uint32,
    );
    if (((*E).base.y_pos) as i16).wrapping_sub(((*E).spm_var_C) as i16) < 0i16 {
        (*E).spm_var_A = ((0xa68bb4i32) as u16);
        (*E).base.y_pos = (*E).spm_var_C;
    }
    if SpikeyPlatform_Func_5(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    ) != 0
    {
        let ref mut fresh18 = *((&raw mut game.g_ram as *mut uint8).offset(0xb5c) as *mut uint16);
        *fresh18 = (*fresh18 as ::core::ffi::c_int
            + ((*E).base.y_pos as ::core::ffi::c_int - (*E).spm_var_01 as ::core::ffi::c_int))
            as uint16;
    }
}
pub unsafe fn SpikeyPlatform_Func_5(k: uint16, game: &mut Game) -> uint16 {
    let E: *mut Enemy_SpikeyPlatform = Get_SpikeyPlatform(k as ::core::ffi::c_int, game);
    let v2: uint16 = abs16(
        (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16))
            .wrapping_sub((*E).base.x_pos),
    );
    let mut v3: bool = v2 < *((&raw mut game.g_ram as *mut uint8).offset(0xafe) as *mut uint16);
    let v4: uint16 =
        (v2).wrapping_sub(*((&raw mut game.g_ram as *mut uint8).offset(0xafe) as *mut uint16));
    if !v3 && v4 >= (*E).base.x_width {
        return 0 as uint16;
    }
    if ((*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16) as ::core::ffi::c_int
        + 5 as ::core::ffi::c_int
        - (*E).base.y_pos as ::core::ffi::c_int) as int16)
        < 0i16
    {
        let v6: uint16 = ((*E).base.y_pos as ::core::ffi::c_int
            - (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
                as ::core::ffi::c_int
                + 5 as ::core::ffi::c_int)) as uint16;
        v3 = v6 < *((&raw mut game.g_ram as *mut uint8).offset(0xb00) as *mut uint16);
        let v7: uint16 =
            (v6).wrapping_sub(*((&raw mut game.g_ram as *mut uint8).offset(0xb00) as *mut uint16));
        if v3 || v7 == (*E).base.y_height || v7 < (*E).base.y_height {
            return -(1 as ::core::ffi::c_int) as uint16;
        }
    }
    0 as uint16
}
pub unsafe fn FireGeyser_Instr_1(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    QueueSfx2_Max6(0x61 as uint16, game);
    jp
}
pub unsafe fn FireGeyser_Instr_2(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let v2: uint16 = ((*Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .fgr_var_D)
        .wrapping_sub(g_word_A68DBB);
    let E: *mut Enemy_FireGeyser = Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int,
        game,
    );
    (*E).base.y_pos = v2;
    (*E).base.y_height = g_word_A68DE7;
    (*E).base.x_width = 8 as uint16;
    jp
}
pub unsafe fn FireGeyser_Instr_3(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let v2: uint16 = ((*Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .fgr_var_D)
        .wrapping_sub(g_word_A68DBD);
    let E: *mut Enemy_FireGeyser = Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int,
        game,
    );
    (*E).base.y_pos = v2;
    (*E).base.y_height = g_word_A68DE9;
    jp
}
pub unsafe fn FireGeyser_Instr_4(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let v2: uint16 = ((*Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .fgr_var_D)
        .wrapping_sub(g_word_A68DBF);
    let E: *mut Enemy_FireGeyser = Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int,
        game,
    );
    (*E).base.y_pos = v2;
    (*E).base.y_height = g_word_A68DEB;
    jp
}
pub unsafe fn FireGeyser_Instr_5(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let v2: uint16 = ((*Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .fgr_var_D)
        .wrapping_sub(g_word_A68DC1);
    let E: *mut Enemy_FireGeyser = Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int,
        game,
    );
    (*E).base.y_pos = v2;
    (*E).base.y_height = g_word_A68DED;
    jp
}
pub unsafe fn FireGeyser_Instr_6(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let v2: uint16 = ((*Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .fgr_var_D)
        .wrapping_sub(g_word_A68DC3);
    let E: *mut Enemy_FireGeyser = Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int,
        game,
    );
    (*E).base.y_pos = v2;
    (*E).base.y_height = g_word_A68DEF;
    jp
}
pub unsafe fn FireGeyser_Instr_7(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let v2: uint16 = ((*Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .fgr_var_D)
        .wrapping_sub(g_word_A68DC5);
    let E: *mut Enemy_FireGeyser = Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int,
        game,
    );
    (*E).base.y_pos = v2;
    (*E).base.y_height = g_word_A68DF1;
    jp
}
pub unsafe fn FireGeyser_Instr_8(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let v2: uint16 = ((*Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .fgr_var_D)
        .wrapping_sub(g_word_A68DC7);
    let E: *mut Enemy_FireGeyser = Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int,
        game,
    );
    (*E).base.y_pos = v2;
    (*E).base.y_height = g_word_A68DF3;
    jp
}
pub unsafe fn FireGeyser_Instr_9(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let v2: uint16 = ((*Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .fgr_var_D)
        .wrapping_sub(g_word_A68DC9);
    let E: *mut Enemy_FireGeyser = Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int,
        game,
    );
    (*E).base.y_pos = v2;
    (*E).base.y_height = g_word_A68DF5;
    jp
}
pub unsafe fn FireGeyser_Instr_10(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let v2: uint16 = ((*Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .fgr_var_D)
        .wrapping_sub(g_word_A68DCB);
    let E: *mut Enemy_FireGeyser = Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int,
        game,
    );
    (*E).base.y_pos = v2;
    (*E).base.y_height = g_word_A68DF7;
    jp
}
pub unsafe fn FireGeyser_Instr_11(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let v2: uint16 = ((*Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .fgr_var_D)
        .wrapping_sub(g_word_A68DCD);
    let E: *mut Enemy_FireGeyser = Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int,
        game,
    );
    (*E).base.y_pos = v2;
    (*E).base.y_height = g_word_A68DF9;
    jp
}
pub unsafe fn FireGeyser_Instr_12(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let v2: uint16 = ((*Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .fgr_var_D)
        .wrapping_sub(g_word_A68DCF);
    let E: *mut Enemy_FireGeyser = Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int,
        game,
    );
    (*E).base.y_pos = v2;
    (*E).base.y_height = g_word_A68DFB;
    jp
}
pub unsafe fn FireGeyser_Instr_13(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let v2: uint16 = ((*Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .fgr_var_D)
        .wrapping_sub(g_word_A68DD1);
    let E: *mut Enemy_FireGeyser = Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int,
        game,
    );
    (*E).base.y_pos = v2;
    (*E).base.y_height = g_word_A68DFD;
    jp
}
pub unsafe fn FireGeyser_Instr_14(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let v2: uint16 = ((*Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .fgr_var_D)
        .wrapping_sub(g_word_A68DD3);
    let E: *mut Enemy_FireGeyser = Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int,
        game,
    );
    (*E).base.y_pos = v2;
    (*E).base.y_height = g_word_A68DFF;
    jp
}
pub unsafe fn FireGeyser_Instr_15(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let v2: uint16 = ((*Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .fgr_var_D)
        .wrapping_sub(g_word_A68DD5);
    let E: *mut Enemy_FireGeyser = Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int,
        game,
    );
    (*E).base.y_pos = v2;
    (*E).base.y_height = g_word_A68E01;
    jp
}
pub unsafe fn FireGeyser_Instr_16(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let v2: uint16 = ((*Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .fgr_var_D)
        .wrapping_sub(g_word_A68DD7);
    let E: *mut Enemy_FireGeyser = Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int,
        game,
    );
    (*E).base.y_pos = v2;
    (*E).base.y_height = g_word_A68E03;
    jp
}
pub unsafe fn FireGeyser_Instr_17(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let v2: uint16 = ((*Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .fgr_var_D)
        .wrapping_sub(g_word_A68DD9);
    let E: *mut Enemy_FireGeyser = Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int,
        game,
    );
    (*E).base.y_pos = v2;
    (*E).base.y_height = g_word_A68E05;
    jp
}
pub unsafe fn FireGeyser_Instr_18(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let v2: uint16 = ((*Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .fgr_var_D)
        .wrapping_sub(g_word_A68DDB);
    let E: *mut Enemy_FireGeyser = Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int,
        game,
    );
    (*E).base.y_pos = v2;
    (*E).base.y_height = g_word_A68E07;
    jp
}
pub unsafe fn FireGeyser_Instr_19(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let v2: uint16 = ((*Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .fgr_var_D)
        .wrapping_sub(g_word_A68DDD);
    let E: *mut Enemy_FireGeyser = Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int,
        game,
    );
    (*E).base.y_pos = v2;
    (*E).base.y_height = g_word_A68E09;
    jp
}
pub unsafe fn FireGeyser_Instr_20(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let v2: uint16 = ((*Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .fgr_var_D)
        .wrapping_sub(g_word_A68DDF);
    let E: *mut Enemy_FireGeyser = Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int,
        game,
    );
    (*E).base.y_pos = v2;
    (*E).base.y_height = g_word_A68E0B;
    jp
}
pub unsafe fn FireGeyser_Instr_21(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let v2: uint16 = ((*Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .fgr_var_D)
        .wrapping_sub(g_word_A68DE1);
    let E: *mut Enemy_FireGeyser = Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int,
        game,
    );
    (*E).base.y_pos = v2;
    (*E).base.y_height = g_word_A68E0D;
    jp
}
pub unsafe fn FireGeyser_Instr_22(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let v2: uint16 = ((*Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .fgr_var_D)
        .wrapping_sub(g_word_A68DE3);
    let E: *mut Enemy_FireGeyser = Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int,
        game,
    );
    (*E).base.y_pos = v2;
    (*E).base.y_height = g_word_A68E0F;
    jp
}
pub unsafe fn FireGeyser_Instr_23(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let v2: uint16 = ((*Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .fgr_var_D)
        .wrapping_sub(g_word_A68DE5);
    let E: *mut Enemy_FireGeyser = Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int,
        game,
    );
    (*E).base.y_pos = v2;
    (*E).base.y_height = g_word_A68E11;
    jp
}
pub unsafe fn FireGeyser_Instr_24(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let E: *mut Enemy_FireGeyser = Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).fgr_var_C = 1 as uint16;
    let E1: *mut Enemy_FireGeyser = Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int,
        game,
    );
    (*E1).base.x_width = 0 as uint16;
    (*E1).base.y_height = 0 as uint16;
    (*E).base.y_pos = (*E).fgr_var_D;
    (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Invisible) as u16);
    (*E1).base.properties = ((*E1).base.properties) | (0x400u16);
    jp
}
pub unsafe fn FireGeyser_Init(game: &mut Game) {
    let E: *mut Enemy_FireGeyser = Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.current_instruction = addr_kFireGeyser_Ilist_8DA9 as uint16;
    if (*E).fgr_parameter_2 == 0 {
        (*E).base.current_instruction = addr_kFireGeyser_Ilist_8D1B as uint16;
        (*E).fgr_var_A = ((0xa6902fi32) as u16);
        (*E).fgr_var_D = (*E).base.y_pos;
        (*E).base.x_width = 0 as uint16;
    }
}
pub unsafe fn CallFireGeyserFunc(ea: uint32, game: &mut Game) {
    match ea {
        10915887 => {
            FireGeyser_Func_1(game);
        }
        10915938 => {
            FireGeyser_Func_2(game);
        }
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn FireGeyser_Main(game: &mut Game) {
    let E: *mut Enemy_FireGeyser = Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).fgr_parameter_2 == 0 {
        CallFireGeyserFunc((((*E).fgr_var_A) as u32) | (0xa60000u32), game);
    }
}
pub unsafe fn FireGeyser_Func_1(game: &mut Game) {
    let E: *mut Enemy_FireGeyser = Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).fgr_var_B = (*E).fgr_var_B.wrapping_sub(1);
    if ((*E).fgr_var_B as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        (*E).fgr_var_A = ((0xa69062i32) as u16);
        (*E).fgr_var_C = 0 as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
        (*E).base.current_instruction = addr_kFireGeyser_Ilist_8D1B as uint16;
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            & !(kEnemyProps_Invisible as ::core::ffi::c_int))
            as uint16;
        let E1: *mut Enemy_FireGeyser = Get_FireGeyser(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int
                + 64 as ::core::ffi::c_int,
            game,
        );
        (*E1).base.properties = ((*E1).base.properties as ::core::ffi::c_int
            & !(0x400 as ::core::ffi::c_int)) as uint16;
    }
}
pub unsafe fn FireGeyser_Func_2(game: &mut Game) {
    let E: *mut Enemy_FireGeyser = Get_FireGeyser(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).fgr_var_C != 0 {
        (*E).fgr_var_B = (*E).fgr_parameter_1;
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Invisible) as u16);
        (*E).fgr_var_A = ((0xa6902fi32) as u16);
    }
}
pub unsafe fn NuclearWaffle_Init(game: &mut Game) {
    let E: *mut Enemy_NuclearWaffle = Get_NuclearWaffle(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.current_instruction = addr_kNuclearWaffle_Ilist_9490 as uint16;
    (*E).nwe_var_C = *(&raw mut (*E).nwe_parameter_1 as *mut uint8).offset(0) as uint16;
    (*E).nwe_var_D = *(&raw mut (*E).nwe_parameter_1 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>())
            .wrapping_div(::core::mem::size_of::<uint8>())
            .wrapping_sub(1_usize) as isize,
    ) as uint16;
    (*E).nwe_var_E = *(&raw mut (*E).nwe_parameter_2 as *mut uint8).offset(0) as uint16;
    let nwe_parameter_2_high: uint16 = *(&raw mut (*E).nwe_parameter_2 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>())
            .wrapping_div(::core::mem::size_of::<uint8>())
            .wrapping_sub(1_usize) as isize,
    ) as uint16;
    (*E).nwe_var_F = nwe_parameter_2_high;
    (*E).nwe_var_B = nwe_parameter_2_high;
    (*E).nwe_var_A = ((0xa69615i32) as u16);
    let v2: ::core::ffi::c_int =
        ((4u16).wrapping_mul((*E).nwe_var_E)) as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let v3: uint16 = g_word_A695F6[v2 as usize];
    (*E).nwe_var_23 = v3;
    (*E).nwe_var_21 = v3;
    (*E).nwe_var_2E = g_word_A695F6[(v2 + 1 as ::core::ffi::c_int) as usize];
    (*E).nwe_var_2B = g_word_A695FE[v2 as usize];
    (*E).nwe_var_2C = g_word_A695FE[(v2 + 1 as ::core::ffi::c_int) as usize];
    (*E).nwe_var_30 = g_word_A69606[v2 as usize];
    (*E).nwe_var_2F = g_word_A69606[(v2 + 1 as ::core::ffi::c_int) as usize];
    let mut v4: uint16 = (8u16).wrapping_mul((*E).nwe_var_C);
    if (*E).nwe_var_E == 0 {
        v4 = (v4).wrapping_add(4u16);
    }
    let v5: ::core::ffi::c_int = v4 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    (*E).nwe_var_25 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8
        as *mut uint16)
        .offset(v5 as isize);
    (*E).nwe_var_24 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8
        as *mut uint16)
        .offset((v5 + 1 as ::core::ffi::c_int) as isize);
    (*E).nwe_var_26 = (*E).base.x_pos;
    (*E).nwe_var_27 = (*E).base.y_pos;
    let v6: uint16 =
        ((*E).nwe_var_26).wrapping_add(CosineMult8bit((*E).nwe_var_23, (*E).nwe_var_D, game));
    (*E).nwe_var_28 = v6;
    (*E).base.x_pos = v6;
    let v7: uint16 =
        ((*E).nwe_var_27).wrapping_add(SineMult8bit((*E).nwe_var_23, (*E).nwe_var_D, game));
    (*E).nwe_var_29 = v7;
    (*E).base.y_pos = v7;
    let mut v8: uint16 = 8 as uint16;
    (*E).nwe_var_2A = 8 as uint16;
    loop {
        SpawnEprojWithGfx(
            v8,
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            addr_kEproj_NuclearWaffleBody as uint16,
            game,
        );
        v8 = ((*E).nwe_var_2A).wrapping_sub(2u16);
        (*E).nwe_var_2A = v8;
        if !(v8 != 0) {
            break;
        }
    }
    (*E).nwe_var_2A = 6 as uint16;
    let mut v15: uint16 = 0;
    loop {
        let v13: uint16 = ((*E).base.vram_tiles_index) | ((*E).base.palette_index);
        (*E).nwe_var_34 = v13;
        let r18: uint16 =
            CreateSpriteAtPos((*E).base.x_pos, (*E).base.y_pos, 43 as uint16, v13, game);
        (*Get_NuclearWaffle(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int
                + (*E).nwe_var_2A as ::core::ffi::c_int,
            game,
        ))
        .nwe_var_10 = r18;
        v15 = ((*E).nwe_var_2A).wrapping_sub(2u16);
        (*E).nwe_var_2A = v15;
        if !(v15 != 0) {
            break;
        }
    }
}
pub unsafe fn CallNuclearWaffleFunc(ea: uint32, game: &mut Game) {
    match ea {
        10917397 => {
            NuclearWaffle_Func_1(game);
        }
        10917506 => {
            NuclearWaffle_Func_2(game);
        }
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn NuclearWaffle_Main(game: &mut Game) {
    let E: *mut Enemy_NuclearWaffle = Get_NuclearWaffle(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    CallNuclearWaffleFunc((((*E).nwe_var_A) as u32) | (0xa60000u32), game);
}
pub unsafe fn NuclearWaffle_Func_1(game: &mut Game) {
    let E: *mut Enemy_NuclearWaffle = Get_NuclearWaffle(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).nwe_var_B = (*E).nwe_var_B.wrapping_sub(1);
    if ((*E).nwe_var_B as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        (*E).nwe_var_B = (*E).nwe_var_F;
        (*E).nwe_var_21 = (*E).nwe_var_23;
        (*E).nwe_var_A = ((0xa69682i32) as u16);
        (*E).nwe_var_33 = 0 as uint16;
        (*E).nwe_var_08 = 0 as uint16;
        (*E).nwe_var_09 = 0 as uint16;
        (*E).nwe_var_0A = 0 as uint16;
        (*E).nwe_var_0B = 0 as uint16;
        (*E).nwe_var_0C = 0 as uint16;
        (*E).nwe_var_0D = 0 as uint16;
        (*E).nwe_var_0E = 0 as uint16;
        (*E).nwe_var_0F = 0 as uint16;
        (*E).nwe_var_18 = 0 as uint16;
        (*E).nwe_var_19 = 0 as uint16;
        (*E).nwe_var_1A = 0 as uint16;
        (*E).nwe_var_1B = 0 as uint16;
        (*E).nwe_var_1C = 0 as uint16;
        (*E).nwe_var_1D = 0 as uint16;
        (*E).nwe_var_1E = 0 as uint16;
        (*E).nwe_var_1F = 0 as uint16;
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_ProcessedOffscreen) as u16);
    }
}
pub unsafe fn NuclearWaffle_Func_2(game: &mut Game) {
    let E: *mut Enemy_NuclearWaffle = Get_NuclearWaffle(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let varE32: uint16 = (*E).nwe_var_D;
    let pair: PairU16 = NuclearWaffle_Func_6((*E).nwe_var_21, game);
    let v8: uint16 = pair.j;
    if pair.j != (*E).nwe_var_33 {
        CreateSpriteAtPos(
            (*E).base.x_pos,
            (*E).base.y_pos,
            46 as uint16,
            (*E).nwe_var_34,
            game,
        );
        CreateSpriteAtPos(
            (*E).base.x_pos,
            (*E).base.y_pos,
            (pair.k).wrapping_add(44u16),
            (*E).nwe_var_34,
            game,
        );
        NuclearWaffle_Func_7(pair.j, game);
    }
    (*E).nwe_var_33 = v8;
    let mut tmp: uint16 = 0;
    let v4: uint16 = NuclearWaffle_Func_5((*E).nwe_var_21, &raw mut tmp, game);
    (*E).base.x_pos = ((*E).nwe_var_26).wrapping_add(CosineMult8bit(v4, (*E).nwe_var_D, game));
    let v5: uint16 = NuclearWaffle_Func_5((*E).nwe_var_21, &raw mut tmp, game);
    (*E).base.y_pos = ((*E).nwe_var_27).wrapping_add(SineMult8bit(v5, (*E).nwe_var_D, game));
    NuclearWaffle_Func_3(varE32, game);
    NuclearWaffle_Func_4(varE32, game);
    AddToHiLo(
        &raw mut (*E).nwe_var_21,
        &raw mut (*E).nwe_var_20,
        ((*E).nwe_var_25 as uint32) << 16 as ::core::ffi::c_int | (*E).nwe_var_24 as uint32,
    );
}
pub unsafe fn NuclearWaffle_Func_3(varE32: uint16, game: &mut Game) {
    let E: *mut Enemy_NuclearWaffle = Get_NuclearWaffle(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut v12: uint16 = 0;
    (*E).nwe_var_2A = 8 as uint16;
    let mut R36: uint16 = ((*E).nwe_var_2C).wrapping_add((*E).nwe_var_21);
    let mut r28: uint16 = 0;
    loop {
        R36 = (R36).wrapping_sub((*E).nwe_var_2B);
        let ET: *mut Enemy_NuclearWaffle = Get_NuclearWaffle(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int
                + (*E).nwe_var_2A as ::core::ffi::c_int,
            game,
        );
        let nwe_var_00: uint16 = (*ET).nwe_var_00;
        let pair: PairU16 = NuclearWaffle_Func_6(R36, game);
        let v13: uint16 = pair.j;
        if pair.j != (*ET).nwe_var_08 {
            let v5: ::core::ffi::c_int =
                nwe_var_00 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            CreateSpriteAtPos(
                *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16)
                    .offset(v5 as isize),
                *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16)
                    .offset(v5 as isize),
                46 as uint16,
                (*E).nwe_var_34,
                game,
            );
            CreateSpriteAtPos(
                *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16)
                    .offset(v5 as isize),
                *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16)
                    .offset(v5 as isize),
                (pair.k).wrapping_add(44u16),
                (*E).nwe_var_34,
                game,
            );
            NuclearWaffle_Func_7(pair.j, game);
        }
        (*ET).nwe_var_08 = v13;
        let v7: uint16 = NuclearWaffle_Func_5(R36, &raw mut r28, game);
        let v10: ::core::ffi::c_int = nwe_var_00 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v10 as isize) =
            ((*E).nwe_var_26).wrapping_add(CosineMult8bit(v7, varE32, game));
        let v11: uint16 = NuclearWaffle_Func_5(R36, &raw mut r28, game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v10 as isize) =
            ((*E).nwe_var_27).wrapping_add(SineMult8bit(v11, varE32, game));
        v12 = ((*E).nwe_var_2A).wrapping_sub(2u16);
        (*E).nwe_var_2A = v12;
        if !(v12 != 0) {
            break;
        }
    }
    if r28 != 0 {
        (*E).nwe_var_A = ((0xa69615i32) as u16);
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            & !(kEnemyProps_ProcessedOffscreen as ::core::ffi::c_int))
            as uint16;
    }
}
pub unsafe fn NuclearWaffle_Func_4(varE32: uint16, game: &mut Game) {
    let E: *mut Enemy_NuclearWaffle = Get_NuclearWaffle(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).nwe_var_2A = 6 as uint16;
    let mut R36: uint16 = (*E).nwe_var_21;
    let mut v12: uint16 = 0;
    let mut r28: uint16 = 0 as uint16;
    loop {
        R36 = (R36).wrapping_sub((*E).nwe_var_2B);
        let ET: *mut Enemy_NuclearWaffle = Get_NuclearWaffle(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int
                + (*E).nwe_var_2A as ::core::ffi::c_int,
            game,
        );
        let nwe_var_10: uint16 = (*ET).nwe_var_10;
        let pair: PairU16 = NuclearWaffle_Func_6(R36, game);
        let v13: uint16 = pair.j;
        if pair.j != (*ET).nwe_var_18 {
            let r38: uint16 = (*E).nwe_var_34;
            let v5: ::core::ffi::c_int =
                nwe_var_10 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            CreateSpriteAtPos(
                *((&raw mut game.g_ram as *mut uint8).offset(0xf0f8) as *mut uint16)
                    .offset(v5 as isize),
                *((&raw mut game.g_ram as *mut uint8).offset(0xf1f8) as *mut uint16)
                    .offset(v5 as isize),
                46 as uint16,
                r38,
                game,
            );
            CreateSpriteAtPos(
                *((&raw mut game.g_ram as *mut uint8).offset(0xf0f8) as *mut uint16)
                    .offset(v5 as isize),
                *((&raw mut game.g_ram as *mut uint8).offset(0xf1f8) as *mut uint16)
                    .offset(v5 as isize),
                (pair.k).wrapping_add(44u16),
                r38,
                game,
            );
            NuclearWaffle_Func_7(pair.j, game);
        }
        (*ET).nwe_var_18 = v13;
        let v6: uint16 = NuclearWaffle_Func_5(R36, &raw mut r28, game);
        let v7: uint16 = CosineMult8bit(v6, varE32, game);
        let R32: uint16 = ((*E).nwe_var_26).wrapping_add(v7);
        let v9: uint16 = NuclearWaffle_Func_5(R36, &raw mut r28, game);
        let R34: uint16 = ((*E).nwe_var_27).wrapping_add(SineMult8bit(v9, varE32, game));
        *((&raw mut game.g_ram as *mut uint8).offset(0xf0f8) as *mut uint16)
            .offset((nwe_var_10 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = R32;
        *((&raw mut game.g_ram as *mut uint8).offset(0xf1f8) as *mut uint16)
            .offset((nwe_var_10 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = R34;
        v12 = ((*E).nwe_var_2A).wrapping_sub(2u16);
        (*E).nwe_var_2A = v12;
        if !(v12 != 0) {
            break;
        }
    }
}
pub unsafe fn NuclearWaffle_Func_5(a: uint16, r28_out: *mut uint16, game: &mut Game) -> uint16 {
    *r28_out = 0 as uint16;
    let E: *mut Enemy_NuclearWaffle = Get_NuclearWaffle(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).nwe_var_E == 0 {
        if ((a) as i16).wrapping_sub(((*E).nwe_var_2E) as i16) >= 0i16 {
            if ((a) as i16).wrapping_sub(((*E).nwe_var_23) as i16) < 0i16 {
                return a;
            }
            return (*E).nwe_var_23;
        }
    } else if !(((a) as i16).wrapping_sub(((*E).nwe_var_2E) as i16) >= 0i16) {
        if ((a) as i16).wrapping_sub(((*E).nwe_var_23) as i16) < 0i16 {
            return (*E).nwe_var_23;
        }
        return a;
    }
    *r28_out = (*r28_out).wrapping_add(1);
    (*E).nwe_var_2E
}
unsafe fn NuclearWaffle_Func_6(a: uint16, game: &mut Game) -> PairU16 {
    let E: *mut Enemy_NuclearWaffle = Get_NuclearWaffle(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).nwe_var_E != 0 {
        if ((a) as i16).wrapping_sub(((*E).nwe_var_2F) as i16) >= 0i16 {
            PairU16 {
                k: 1 as uint16,
                j: 2 as uint16,
            }
        } else if ((a) as i16).wrapping_sub(((*E).nwe_var_30) as i16) >= 0i16 {
            PairU16 {
                k: 0 as uint16,
                j: 1 as uint16,
            }
        } else {
            PairU16 {
                k: 0 as uint16,
                j: 0 as uint16,
            }
        }
    } else if ((a) as i16).wrapping_sub(((*E).nwe_var_2F) as i16) < 0i16 {
        PairU16 {
            k: 0 as uint16,
            j: 2 as uint16,
        }
    } else if ((a) as i16).wrapping_sub(((*E).nwe_var_30) as i16) < 0i16 {
        PairU16 {
            k: 1 as uint16,
            j: 1 as uint16,
        }
    } else {
        PairU16 {
            k: 0 as uint16,
            j: 0 as uint16,
        }
    }
}
pub unsafe fn NuclearWaffle_Func_7(r30: uint16, game: &mut Game) {
    if r30 != 2u16 {
        QueueSfx2_Max6(0x5e as uint16, game);
    }
}
pub unsafe fn FakeKraid_Init(game: &mut Game) {
    let mut v0: uint16 = ((*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
        as ::core::ffi::c_int
        & 3 as ::core::ffi::c_int)
        + 2 as ::core::ffi::c_int) as uint16;
    let E: *mut Enemy_FakeKraid = Get_FakeKraid(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).fkd_var_D = v0;
    (*E).fkd_var_E = v0;
    v0 = (v0).wrapping_add(64u16);
    (*E).fkd_var_03 = v0;
    v0 = (v0).wrapping_add(32u16);
    (*E).fkd_var_04 = v0;
    (*E).fkd_var_05 = (v0).wrapping_sub(48u16);
    (*E).fkd_var_07 = 0 as uint16;
    (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_DisableSamusColl) as u16);
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
    (*E).fkd_var_B = -(4 as ::core::ffi::c_int) as uint16;
    (*E).fkd_var_C = -(4 as ::core::ffi::c_int) as uint16;
    let mut v2: uint16 = addr_stru_A699AE as uint16;
    if (((*E).base.x_pos) as i16)
        .wrapping_sub((*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)) as i16)
        < 0i16
    {
        (*E).fkd_var_B = 4 as uint16;
        (*E).fkd_var_C = 4 as uint16;
        v2 = addr_stru_A699FC as uint16;
    }
    (*E).base.current_instruction = v2;
}
pub unsafe fn FakeKraid_Main(game: &mut Game) {
    let E: *mut Enemy_FakeKraid = Get_FakeKraid(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let fkd_var_07: uint16 = (*E).fkd_var_07;
    let mut v3: uint16 = (fkd_var_07).wrapping_add(2u16);
    if fkd_var_07 as ::core::ffi::c_int - 4 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        == 0
    {
        v3 = 0 as uint16;
    }
    (*E).fkd_var_07 = v3;
    FakeKraid_Func_1(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        fkd_var_07,
        game,
    );
}
pub unsafe fn FakeKraid_Func_1(k: uint16, j: uint16, game: &mut Game) {
    let ET: *mut Enemy_FakeKraid =
        Get_FakeKraid(k as ::core::ffi::c_int + j as ::core::ffi::c_int, game);
    let fkd_var_03: uint16 = (*ET).fkd_var_03;
    if fkd_var_03 != 0 {
        (*ET).fkd_var_03 = (fkd_var_03).wrapping_sub(1u16);
    } else {
        (*ET).fkd_var_03 = ((*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
            as ::core::ffi::c_int
            & 0x3f as ::core::ffi::c_int)
            + 16 as ::core::ffi::c_int) as uint16;
        let E: *mut Enemy_FakeKraid = Get_FakeKraid(k as ::core::ffi::c_int, game);
        (*E).fkd_var_06 = j;
        let mut v6: uint16 = addr_kEproj_MiniKraidSpikesLeft as uint16;
        let fkd_var_C: int16 = (*E).fkd_var_C as int16;
        if fkd_var_C >= 0i16 {
            v6 = addr_kEproj_MiniKraidSpikesRight as uint16;
        }
        SpawnEprojWithGfx(fkd_var_C as uint16, k, v6, game);
        if CheckIfEnemyIsOnScreen(game) == 0 {
            QueueSfx2_Max6(0x3f as uint16, game);
        }
    };
}
pub unsafe fn FakeKraid_Instr_2(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let current_block: u64;
    let E: *mut Enemy_FakeKraid = Get_FakeKraid(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).fkd_var_E != 0 {
        (*E).fkd_var_E = (*E).fkd_var_E.wrapping_sub(1);
    }
    let fresh1 = (*E).fkd_var_D;
    (*E).fkd_var_D = (*E).fkd_var_D.wrapping_sub(1);
    if fresh1 == 1u16 {
        (*E).fkd_var_D = ((*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
            as ::core::ffi::c_int
            & 3 as ::core::ffi::c_int)
            + 7 as ::core::ffi::c_int) as uint16;
        current_block = 6873731126896040597;
    } else if Enemy_MoveRight_IgnoreSlopes(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        ((*E).fkd_var_B as int16 as int32) << 16 as ::core::ffi::c_int,
        game,
    ) == 0
    {
        current_block = 7288802866200656202;
    } else {
        current_block = 6873731126896040597;
    }
    match current_block {
        6873731126896040597 => {
            (*E).fkd_var_B = -((*E).fkd_var_B as ::core::ffi::c_int) as uint16;
        }
        _ => {}
    }
    (*E).fkd_var_C = -(4 as ::core::ffi::c_int) as uint16;
    if (((*E).base.x_pos) as i16)
        .wrapping_sub((*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)) as i16)
        < 0i16
    {
        (*E).fkd_var_C = 4 as uint16;
    }
    jp
}
pub unsafe fn FakeKraid_Instr_1(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let E: *mut Enemy_FakeKraid = Get_FakeKraid(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).fkd_var_E != 0 {
        if ((*E).fkd_var_C as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
            if ((*Get_FakeKraid(
                *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                    as ::core::ffi::c_int,
                game,
            ))
            .fkd_var_B as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int)
                == 0i32
            {
                return addr_stru_A699C6 as ::core::ffi::c_int as uintptr_t as *const uint16;
            }
            addr_stru_A699AE as ::core::ffi::c_int as uintptr_t as *const uint16
        } else {
            if ((*E).fkd_var_B as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
                return addr_stru_A69A14 as ::core::ffi::c_int as uintptr_t as *const uint16;
            }
            addr_stru_A699FC as ::core::ffi::c_int as uintptr_t as *const uint16
        }
    } else {
        (*E).fkd_var_E = ((*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
            as ::core::ffi::c_int
            & 3 as ::core::ffi::c_int)
            + 3 as ::core::ffi::c_int) as uint16;
        if ((*E).fkd_var_C as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
            return addr_kFakeKraid_Ilist_99DC as ::core::ffi::c_int as uintptr_t as *const uint16;
        }
        addr_kFakeKraid_Ilist_9A2A as ::core::ffi::c_int as uintptr_t as *const uint16
    }
}
pub unsafe fn FakeKraid_Instr_3(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    if CheckIfEnemyIsOnScreen(game) == 0u16 {
        QueueSfx2_Max6(0x16 as uint16, game);
    }
    jp
}
pub unsafe fn FakeKraid_Instr_4(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    FakeKraid_InstrHelper_45(k, 0 as uint16, 0xfffc as uint16, game);
    jp
}
pub unsafe fn FakeKraid_InstrHelper_45(k: uint16, j: uint16, a: uint16, game: &mut Game) {
    let ka: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16);
    let E: *mut Enemy_FakeKraid = Get_FakeKraid(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).fkd_var_02 = a;
    let v5: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    (*E).fkd_var_00 = g_word_A69A48[v5 as usize];
    let v6: uint16 = g_word_A69A48[(v5 + 1 as ::core::ffi::c_int) as usize];
    (*E).fkd_var_01 = v6;
    SpawnEprojWithGfx(
        v6,
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        addr_kEproj_MiniKraidSpit as uint16,
        game,
    );
    (*E).fkd_var_00 = g_word_A69A48[(v5 + 2 as ::core::ffi::c_int) as usize];
    let v8: uint16 = g_word_A69A48[(v5 + 3 as ::core::ffi::c_int) as usize];
    (*E).fkd_var_01 = v8;
    SpawnEprojWithGfx(v8, ka, addr_kEproj_MiniKraidSpit as uint16, game);
}
pub unsafe fn FakeKraid_Instr_5(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    FakeKraid_InstrHelper_45(k, 8 as uint16, 4 as uint16, game);
    jp
}
pub unsafe fn sub_A69C0B(game: &mut Game) {
    let v0: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0xf434) as *mut uint16) = (*v0).x_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0xf436) as *mut uint16) = (*v0).y_pos;
    NormalEnemyPowerBombAiSkipDeathAnim_CurEnemy(game);
    FakeKraid_9C50(game);
}
pub unsafe fn FakeKraid_Touch(game: &mut Game) {
    let E: *mut Enemy_FakeKraid = Get_FakeKraid(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0xf434) as *mut uint16) = (*E).base.x_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0xf436) as *mut uint16) = (*E).base.y_pos;
    NormalEnemyTouchAiSkipDeathAnim_CurEnemy(game);
    FakeKraid_9C50(game);
}
pub unsafe fn FakeKraid_Shot(game: &mut Game) {
    let E: *mut Enemy_FakeKraid = Get_FakeKraid(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0xf434) as *mut uint16) = (*E).base.x_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0xf436) as *mut uint16) = (*E).base.y_pos;
    NormalEnemyShotAiSkipDeathAnim_CurEnemy(game);
    FakeKraid_9C50(game);
}
pub unsafe fn FakeKraid_9C50(game: &mut Game) {
    if (*Get_FakeKraid(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .base
    .health
        == 0
    {
        // The death animation remains in the ROM spawn parameter for all four drops.
        let death_animation = 3;
        EnemyDeathAnimation(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            death_animation,
            game,
        );
        Enemy_ItemDrop_MiniKraid(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            death_animation,
            game,
        );
    }
}
pub unsafe fn CeresRidley_Init(game: &mut Game) {
    let mut v4: int16 = 0;
    let mut v8: int16 = 0;
    let E: *mut Enemy_CeresRidley = Get_CeresRidley(0 as ::core::ffi::c_int, game);
    if (*(&raw mut game.g_ram as *mut uint8)
        .offset(0xd828)
        .offset(*((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) as isize)
        as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int)
        != 0i32
    {
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            | (kEnemyProps_Tangible as ::core::ffi::c_int
                | kEnemyProps_Deleted as ::core::ffi::c_int
                | kEnemyProps_Invisible as ::core::ffi::c_int))
            as uint16;
    } else {
        let mut i: ::core::ffi::c_int = 4094 as ::core::ffi::c_int;
        while i >= 0i32 {
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((i >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
            i -= 2 as ::core::ffi::c_int;
        }
        DisableMinimapAndMarkBossRoomAsExplored(game);
        (*E).cry_parameter_1 = 0 as uint16;
        (*E).cry_parameter_2 = 0 as uint16;
        Ridley_Func_99(addr_kRidley_Ilist_E538 as uint16, game);
        (*E).base.palette_index = 3584 as uint16;
        (*E).cry_var_0C = 3584 as uint16;
        (*E).base.extra_properties = ((*E).base.extra_properties) | (4u16);
        (*E).cry_var_01 = 0 as uint16;
        (*E).cry_var_0D = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x183e) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1840) as *mut uint16) = 0 as uint16;
        Ridley_Func_92(game);
        Ridley_Func_117(game);
        if *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) == 2u16 {
            (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
                | (kEnemyProps_BlockPlasmaBeam as ::core::ffi::c_int
                    | kEnemyProps_Tangible as ::core::ffi::c_int))
                as uint16;
            (*E).base.x_pos = 96 as uint16;
            (*E).base.y_pos = 394 as uint16;
            (*E).cry_var_A = ((0xa6a35bi32) as u16);
            (*E).cry_var_B = 0 as uint16;
            (*E).cry_var_C = 0 as uint16;
            (*E).base.layer = 5 as uint16;
            (*E).cry_var_01 = 0 as uint16;
            (*E).cry_var_02 = 1 as uint16;
            (*E).cry_var_20 = 64 as uint16;
            (*E).cry_var_21 = 416 as uint16;
            (*E).cry_var_22 = 64 as uint16;
            (*E).cry_var_23 = 224 as uint16;
            (*E).cry_var_10 = 2 as uint16;
            (*E).cry_var_1C = 120 as uint16;
            WriteColorsToTargetPalette(
                0xa6 as uint8,
                0x140 as uint16,
                addr_word_A6E1CF as uint16,
                0x20 as uint16,
                game,
            );
            let mut v3: uint16 = 0 as uint16;
            v4 = 15 as int16;
            loop {
                let v5: ::core::ffi::c_int = v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
                *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16)
                    .offset((v5 + 113 as ::core::ffi::c_int) as isize) = 0 as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16)
                    .offset((v5 + 241 as ::core::ffi::c_int) as isize) = 0 as uint16;
                v3 = (v3).wrapping_add(2u16);
                v4 -= 1;
                if !(v4 != 0) {
                    break;
                }
            }
        } else {
            (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
                | (kEnemyProps_BlockPlasmaBeam as ::core::ffi::c_int
                    | kEnemyProps_Tangible as ::core::ffi::c_int))
                as uint16;
            (*E).base.x_pos = 186 as uint16;
            (*E).base.y_pos = 169 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x93f) as *mut uint16) = 0 as uint16;
            (*E).cry_var_10 = 0 as uint16;
            (*E).cry_var_02 = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(1) =
                1 as uint16;
            (*E).cry_var_1C = 15 as uint16;
            (*E).cry_var_A = ((0xa6a35bi32) as u16);
            (*E).cry_var_B = 0 as uint16;
            (*E).cry_var_C = 0 as uint16;
            (*E).cry_var_20 = -(32 as ::core::ffi::c_int) as uint16;
            (*E).cry_var_21 = 176 as uint16;
            (*E).cry_var_22 = 40 as uint16;
            (*E).cry_var_23 = 224 as uint16;
            WriteColorsToTargetPalette(
                0xa6 as uint8,
                0x140 as uint16,
                addr_word_A6E16F as uint16,
                0x20 as uint16,
                game,
            );
            let mut v7: uint16 = 482 as uint16;
            v8 = 15 as int16;
            loop {
                *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16)
                    .offset((v7 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
                    0 as uint16;
                v7 = (v7).wrapping_add(2u16);
                v8 -= 1;
                if !(v8 != 0) {
                    break;
                }
            }
            (*E).cry_var_03 = addr_kBabyMetroid_Ilist_BF31 as uint16;
            (*E).cry_var_04 = 1 as uint16;
            (*gRam8800_Default(0 as ::core::ffi::c_int, game)).var_40 = ((0xa6be9ci32) as u16);
            (*E).cry_var_07 = 5 as uint16;
            QueueMusic_Delayed8(0 as uint16, game);
        }
    };
}
pub unsafe fn CallRidleyFunc(ea: uint32, game: &mut Game) {
    match ea {
        10920788 => {
            CeresRidley_Func_2(game);
        }
        10920795 => {
            CeresRidley_Func_3(game);
        }
        10920823 => {
            CeresRidley_A377(game);
        }
        10920841 => {
            CeresRidley_Func_4(game);
        }
        10920927 => {
            CeresRidley_Func_5(game);
        }
        10921045 => {
            CeresRidley_Func_6(game);
        }
        10921080 => {
            CeresRidley_Func_7(game);
        }
        10921647 => {
            CeresRidley_Func_9(game);
        }
        10921672 => {
            CeresRidley_Func_10(game);
        }
        10921704 => {
            CeresRidley_Func_11(game);
        }
        10921858 => {
            CeresRidley_Func_13(game);
        }
        10921977 => {
            CeresRidley_A7F9(game);
        }
        10922044 => {
            CeresRidley_Func_14(game);
        }
        10922062 => {
            CeresRidley_Func_15(game);
        }
        10922125 => {
            CeresRidley_Func_16(game);
        }
        10922148 => {
            CeresRidley_Func_17(game);
        }
        10922196 => {
            CeresRidley_Func_18(game);
        }
        10922232 => {
            CeresRidley_Func_19(game);
        }
        10922275 => {
            CeresRidley_Func_20(game);
        }
        10922311 => {
            CeresRidley_Func_21(game);
        }
        10922353 => {
            CeresRidley_Func_22(game);
        }
        10922400 => {
            CeresRidley_Func_23(game);
        }
        10922513 => {
            CeresRidley_Func_24(game);
        }
        10922575 => (),
        10922576 => {
            CeresRidley_Func_26(game);
        }
        10924787 => {
            Ridley_Func_3(game);
        }
        10924833 => {
            Ridley_Func_3b(game);
        }
        10925036 => {
            Ridley_Func_5(game);
        }
        10925048 => {
            Ridley_Func_6(game);
        }
        10925121 => {
            Ridley_Func_8(game);
        }
        10925141 => {
            Ridley_B455(game);
        }
        10925203 => {
            Ridley_Func_9(game);
        }
        10925265 => {
            Ridley_Func_10(game);
        }
        10925334 => {
            Ridley_Func_11(game);
        }
        10925396 => {
            Ridley_Func_12(game);
        }
        10925460 => {
            Ridley_Func_13(game);
        }
        10925508 => {
            Ridley_Func_14(game);
        }
        10925541 => {
            Ridley_Func_15(game);
        }
        10925587 => {
            Ridley_Func_16(game);
        }
        10925735 => {
            Ridley_Func_19(game);
        }
        10925789 => {
            Ridley_Func_20(game);
        }
        10925838 => {
            Ridley_Func_21(game);
        }
        10926009 => {
            Ridley_Func_22(game);
        }
        10926775 => {
            Ridley_Func_33(game);
        }
        10926991 => {
            Ridley_Func_33b(game);
        }
        10927044 => {
            Ridley_Func_34(game);
        }
        10927089 => {
            Ridley_Func_35(game);
        }
        10927150 => {
            Ridley_Func_36(game);
        }
        10927438 => {
            Ridley_Func_43(game);
        }
        10927514 => {
            Ridley_Func_44(game);
        }
        10927548 => {
            Ridley_Func_45(game);
        }
        10927602 => {
            Ridley_Func_46(game);
        }
        10927619 => {
            Ridley_Func_47(game);
        }
        10928206 => {
            Ridley_Func_54(game);
        }
        10929464 => {
            Ridley_Func_63(game);
        }
        10929470 => {
            Ridley_C53E(game);
        }
        10929489 => {
            Ridley_C551(game);
        }
        10929544 => {
            Ridley_Func_64(game);
        }
        10929576 => {
            Ridley_Func_65(game);
        }
        10929608 => {
            Ridley_Func_66(game);
        }
        10929626 => {
            Ridley_Func_67(game);
        }
        10929664 => (),
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn CeresRidley_Main(game: &mut Game) {
    let E: *mut Enemy_CeresRidley = Get_CeresRidley(0 as ::core::ffi::c_int, game);
    (*E).base.health = 0x7fff as uint16;
    CallRidleyFunc((((*E).cry_var_A) as u32) | (0xa60000u32), game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x93f) as *mut uint16) == 0 {
        if (*Get_CeresRidley(0 as ::core::ffi::c_int, game)).cry_var_02 != 0 {
            Ridley_Func_102(game);
            Ridley_Func_112(game);
            Ridley_Func_115(game);
            CeresRidley_Func_1(game);
            Ridley_Func_70(game);
            Ridley_Func_101(game);
            Ridley_Func_118(game);
            sub_A6DB2A(game);
            Ridley_Func_120(game);
        }
        Ridley_A2DC(game);
    }
}
pub unsafe fn CeresRidley_Func_1(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16) >= 0xff00u16 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(15) =
            ((*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
                as ::core::ffi::c_int
                & 0xf as ::core::ffi::c_int)
                + 8 as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn CeresRidley_Hurt(game: &mut Game) {
    Ridley_Func_102(game);
    sub_A6DB2A(game);
    Ridley_Func_120(game);
    Ridley_A2DC(game);
}
pub unsafe fn Ridley_A2DC(game: &mut Game) {
    Ridley_Func_49(game);
    if (*Get_CeresRidley(0 as ::core::ffi::c_int, game)).cry_var_02 != 0 {
        mov24(
            (&raw mut game.g_ram as *mut uint8).offset(0x178c) as *mut LongPtr,
            0xa6a2f2 as uint32,
        );
    } else {
        Ridley_A2F2(game);
    };
}
pub unsafe fn Ridley_A2F2(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x93f) as *mut uint16) == 0 {
        DrawBabyMetroid_0(game);
    }
    let E: *mut Enemy_CeresRidley = Get_CeresRidley(0x40 as ::core::ffi::c_int, game);
    if (*E).cry_var_B != 0 {
        static g_word_A6A321: [int16; 4] = [0, -(1024), -(4), -(1)];
        DrawSpritemap(
            0xa6 as uint8,
            addr_kCeresRidley_Sprmap_A329 as uint16,
            ((*E).base.x_pos).wrapping_add(
                (g_word_A6A321[(*((&raw mut game.g_ram as *mut uint8).offset(0x1840) as *mut uint16)
                    as ::core::ffi::c_int
                    & 3 as ::core::ffi::c_int) as usize]) as u16,
            ),
            (*E).base.y_pos,
            1024 as uint16,
            game,
        );
    }
}
pub unsafe fn CeresRidley_Func_2(game: &mut Game) {
    let E: *mut Enemy_CeresRidley = Get_CeresRidley(0 as ::core::ffi::c_int, game);
    (*E).cry_var_B = 0 as uint16;
    (*E).cry_var_C = 0 as uint16;
}
pub unsafe fn CeresRidley_Func_3(game: &mut Game) {
    let mut v1: int16 = 0;
    if !(*((&raw mut game.g_ram as *mut uint8).offset(0x797) as *mut uint16) != 0) {
        let E: *mut Enemy_CeresRidley = Get_CeresRidley(0 as ::core::ffi::c_int, game);
        (*E).cry_var_A = ((0xa6a377i32) as u16);
        v1 = 512 as int16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) == 2u16 {
            v1 = 170 as int16;
        }
        (*E).cry_var_F = v1 as uint16;
        CeresRidley_A377(game);
    }
}
pub unsafe fn CeresRidley_A377(game: &mut Game) {
    let E: *mut Enemy_CeresRidley = Get_CeresRidley(0 as ::core::ffi::c_int, game);
    (*E).cry_var_F = (*E).cry_var_F.wrapping_sub(1);
    if ((*E).cry_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        (*E).cry_var_A = ((0xa6a389i32) as u16);
        (*E).cry_var_E = 0 as uint16;
        (*E).cry_var_F = 0 as uint16;
    }
}
pub unsafe fn CeresRidley_Func_4(game: &mut Game) {
    let mut v2: int16 = 0;
    let E: *mut Enemy_CeresRidley = Get_CeresRidley(0 as ::core::ffi::c_int, game);
    if ((*E).cry_var_E as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) == 0i32 {
        let cry_var_E: uint16 = (*E).cry_var_E;
        (*E).cry_var_F = (*E).cry_var_F.wrapping_add(1);
        if (*E).cry_var_F != 0 {
            (*E).cry_var_F = 0 as uint16;
            v2 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa6e269)) as *const uint8
                as *mut uint8)
                .offset(cry_var_E as isize) as int16;
            if v2 == 255i16 {
                (*E).cry_var_E = 0 as uint16;
                (*E).cry_var_A = ((0xa6a3dfi32) as u16);
                (*E).cry_var_02 = 1 as uint16;
            } else {
                (*E).cry_var_E = (cry_var_E).wrapping_add(1u16);
                let v3: ::core::ffi::c_int = ((6u16).wrapping_mul((v2) as u16))
                    as ::core::ffi::c_int
                    >> 1 as ::core::ffi::c_int;
                *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(252) =
                    *(game.g_rom.offset(crate::rtl::lorom_offset(0xa6e2aa)) as *const uint8
                        as *mut uint16)
                        .offset(v3 as isize);
                *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(253) =
                    *(game.g_rom.offset(crate::rtl::lorom_offset(0xa6e2aa)) as *const uint8
                        as *mut uint16)
                        .offset((v3 + 1 as ::core::ffi::c_int) as isize);
                *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(254) =
                    *(game.g_rom.offset(crate::rtl::lorom_offset(0xa6e2aa)) as *const uint8
                        as *mut uint16)
                        .offset((v3 + 2 as ::core::ffi::c_int) as isize);
            }
        }
    }
}
pub unsafe fn CeresRidley_Func_5(game: &mut Game) {
    let mut v2: int16 = 0;
    let mut v5: int16 = 0;
    let E: *mut Enemy_CeresRidley = Get_CeresRidley(0 as ::core::ffi::c_int, game);
    (*E).cry_var_F = (*E).cry_var_F.wrapping_add(1);
    if (*E).cry_var_F >= 2u16 {
        (*E).cry_var_F = 0 as uint16;
        let mut cry_var_E: uint16 = (*E).cry_var_E;
        let mut r18: uint16 = 290 as uint16;
        let mut r20: uint16 = 482 as uint16;
        v2 = 11 as int16;
        loop {
            v5 = v2;
            let v3: uint16 =
                *(game.g_rom.offset(crate::rtl::lorom_offset(0xa6e30a)) as *const uint8
                    as *mut uint16)
                    .offset((cry_var_E as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
            *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                .offset((r18 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = v3;
            *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                .offset((r20 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = v3;
            cry_var_E = (cry_var_E).wrapping_add(2u16);
            r18 = (r18).wrapping_add(2u16);
            r20 = (r20).wrapping_add(2u16);
            v2 = (v5).wrapping_sub(1i16);
            if !(v5 != 1i16) {
                break;
            }
        }
        if cry_var_E >= 0x160u16 {
            if *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) == 2u16 {
                (*E).base.layer = 2 as uint16;
            }
            (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
                & !(kEnemyProps_Tangible as ::core::ffi::c_int))
                as uint16;
            (*E).cry_var_E = 0 as uint16;
            (*E).cry_var_A = ((0xa6a455i32) as u16);
            (*E).cry_var_F = 4 as uint16;
            QueueMusic_Delayed8(5 as uint16, game);
        } else {
            (*E).cry_var_E = cry_var_E;
        }
    }
}
pub unsafe fn CeresRidley_Func_6(game: &mut Game) {
    let E: *mut Enemy_CeresRidley = Get_CeresRidley(0 as ::core::ffi::c_int, game);
    (*E).cry_var_F = (*E).cry_var_F.wrapping_sub(1);
    if ((*E).cry_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        Ridley_Func_99(addr_kRidley_Ilist_E690 as uint16, game);
        (*E).cry_var_F = 0 as uint16;
        (*E).cry_var_A = ((0xa6a478i32) as u16);
        if *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) != 2u16 {
            (*E).cry_var_F = 252 as uint16;
        }
    }
}
pub unsafe fn CeresRidley_Func_7(game: &mut Game) {
    let E: *mut Enemy_CeresRidley = Get_CeresRidley(0 as ::core::ffi::c_int, game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) == 2u16 {
        (*E).cry_var_F = (*E).cry_var_F.wrapping_sub(1);
        let v1: bool =
            ((*E).cry_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32;
        if !v1 {
            return;
        }
        (*E).cry_var_F = 2 as uint16;
        let v2: uint16 = ((*E).cry_var_E).wrapping_add(1u16);
        (*E).cry_var_E = v2;
        if CeresRidley_Func_8((v2).wrapping_sub(1u16), game) as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            == 0
        {
            return;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x197a) as *mut uint16) = 440 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x197c) as *mut uint16) =
            -(96 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1980) as *mut uint16) = 32 as uint16;
    } else {
        (*E).cry_var_F = (*E).cry_var_F.wrapping_sub(1);
        let v1_0: bool =
            ((*E).cry_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32;
        if !v1_0 {
            return;
        }
    }
    (*E).cry_var_E = 0 as uint16;
    Ridley_Func_99(addr_kRidley_Ilist_E91D as uint16, game);
    (*E).cry_var_08 = 8 as uint16;
    (*E).cry_var_09 = 8 as uint16;
    Ridley_Func_95(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(0) = 1 as uint16;
    (*E).cry_var_A = ((0xa6a354i32) as u16);
}
pub unsafe fn CeresRidley_Func_8(a: uint16, game: &mut Game) -> uint8 {
    let v1: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa6a4eb)) as *const uint8
        as *mut uint16)
        .offset(a as isize);
    if v1 == 0 {
        return 1 as uint8;
    }
    WriteColorsToPalette(0xe2 as uint16, 0xa6 as uint8, v1, 0xe as uint16, game);
    0 as uint8
}
pub unsafe fn CeresRidley_Func_9(game: &mut Game) {
    let E: *mut Enemy_CeresRidley = Get_CeresRidley(0 as ::core::ffi::c_int, game);
    (*E).cry_var_C = ((*E).cry_var_C).wrapping_sub(16u16);
    if (*E).base.y_pos as ::core::ffi::c_int - 112 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        (*E).cry_var_A = ((0xa6a6c8i32) as u16);
        CeresRidley_Func_10(game);
    }
}
pub unsafe fn CeresRidley_Func_10(game: &mut Game) {
    let E: *mut Enemy_CeresRidley = Get_CeresRidley(0 as ::core::ffi::c_int, game);
    (*E).cry_var_C = ((*E).cry_var_C).wrapping_add(20u16);
    if (*E).base.y_pos as ::core::ffi::c_int - 80 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        (*E).cry_var_A = ((0xa6a6e8i32) as u16);
        (*E).cry_var_01 = 1 as uint16;
    }
}
pub unsafe fn CeresRidley_Func_11(game: &mut Game) {
    let mut v1: uint16 = 0;
    let E: *mut Enemy_CeresRidley = Get_CeresRidley(0 as ::core::ffi::c_int, game);
    if (*E).cry_var_0D >= 0x64u16 {
        (*E).cry_var_01 = 0 as uint16;
        (*E).cry_var_A = ((0xa6bd9ai32) as u16);
        Ridley_Func_44(game);
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0x9c2) as *mut uint16)
        as ::core::ffi::c_int
        - 30 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        (*E).cry_var_01 = 0 as uint16;
        (*E).cry_var_A = ((0xa6a971i32) as u16);
        CeresRidley_Func_22(game);
    } else if CeresRidley_Func_12(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int == 0 || {
        v1 = ((*E).cry_var_00).wrapping_add(1u16);
        (*E).cry_var_00 = v1;
        v1 >= 0x7cu16
    } {
        (*E).cry_var_A = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa6a743)) as *const uint8
            as *mut uint16)
            .offset(
                (*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
                    as ::core::ffi::c_int
                    & 0xf as ::core::ffi::c_int) as isize,
            );
        (*E).cry_var_00 = 0 as uint16;
    }
}
pub unsafe fn CeresRidley_Func_12(game: &mut Game) -> uint8 {
    let rect: Rect16U = Rect16U {
        x: 192 as uint16,
        y: 100 as uint16,
        w: 8 as uint16,
        h: 8 as uint16,
    };
    Ridley_Func_106(0 as uint16, 0 as uint16, rect.x, rect.y, game);
    Shitroid_Func_2(0 as uint16, rect, game)
}
pub unsafe fn CeresRidley_Func_13(game: &mut Game) {
    let E: *mut Enemy_CeresRidley = Get_CeresRidley(0 as ::core::ffi::c_int, game);
    let mut v1: uint16 = abs16((*E).cry_var_C);
    if v1 < 0x80u16 {
        v1 = 128 as uint16;
    }
    (*E).cry_var_C = (if (*E).cry_var_C as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        -(v1 as ::core::ffi::c_int)
    } else {
        v1 as ::core::ffi::c_int
    }) as uint16;
    Ridley_Func_106(
        0 as uint16,
        0 as uint16,
        (*E).base.x_pos,
        88 as uint16,
        game,
    );
    if (*E).base.y_pos as ::core::ffi::c_int - 80 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let v3: uint16 = ((*E).cry_var_00).wrapping_add(1u16);
        (*E).cry_var_00 = v3;
        if v3 >= 0x30u16 {
            (*E).cry_var_A = ((0xa6a88di32) as u16);
        }
    } else if (*E).base.y_pos as ::core::ffi::c_int - 128 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        (*E).cry_var_17 = (*E).base.x_pos;
        (*E).cry_var_18 = (*E).base.y_pos;
        Ridley_Func_99(addr_kRidley_Ilist_E73A as uint16, game);
        (*E).cry_var_A = addr_loc_A6A7F9 as uint16;
        (*E).cry_var_00 = 224 as uint16;
        CeresRidley_A7F9(game);
    }
}
pub unsafe fn CeresRidley_A7F9(game: &mut Game) {
    let r22: uint16 = (if *((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
        as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        -(*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16) as ::core::ffi::c_int
            & 7 as ::core::ffi::c_int)
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16) as ::core::ffi::c_int
            & 7 as ::core::ffi::c_int
    }) as uint16;
    let E: *mut Enemy_CeresRidley = Get_CeresRidley(0 as ::core::ffi::c_int, game);
    let r18: uint16 = ((*E).cry_var_17).wrapping_add(r22);
    let r20: uint16 = ((*E).cry_var_18).wrapping_add(r22);
    Ridley_Func_106(0 as uint16, 0 as uint16, r18, r20, game);
    let v4: int16 = (((*E).cry_var_00) as i16).wrapping_sub(1i16);
    (*E).cry_var_00 = v4 as uint16;
    if v4 < 0i16 {
        (*E).cry_var_00 = 0 as uint16;
        (*E).cry_var_A = ((0xa6a6e8i32) as u16);
    }
}
pub unsafe fn CeresRidley_Func_14(game: &mut Game) {
    Ridley_Func_99(addr_kRidley_Ilist_E548 as uint16, game);
    let E: *mut Enemy_CeresRidley = Get_CeresRidley(0 as ::core::ffi::c_int, game);
    (*E).cry_var_A = ((0xa6a84ei32) as u16);
    (*E).cry_var_F = 64 as uint16;
    CeresRidley_Func_15(game);
}
pub unsafe fn CeresRidley_Func_15(game: &mut Game) {
    let mut v2: bool = false;
    let E: *mut Enemy_CeresRidley = Get_CeresRidley(0 as ::core::ffi::c_int, game);
    let r18: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16);
    let mut v0: uint16 =
        (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)).wrapping_sub(68u16);
    if *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16) as ::core::ffi::c_int
        - 132 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        v0 = 64 as uint16;
    }
    let r20: uint16 = v0;
    Ridley_Func_106(0 as uint16, 0xd as uint16, r18, r20, game);
    let rect: Rect16U = Rect16U {
        x: r18,
        y: r20,
        w: 2 as uint16,
        h: 2 as uint16,
    };
    if Shitroid_Func_2(0 as uint16, rect, game) == 0 || {
        v2 = (((*E).cry_var_F) as i16).wrapping_sub(1i16) < 0i16;
        (*E).cry_var_F = (*E).cry_var_F.wrapping_sub(1);
        v2 as ::core::ffi::c_int != 0
    } {
        (*E).cry_var_00 = 0 as uint16;
        (*E).cry_var_A = ((0xa6a6e8i32) as u16);
    }
}
pub unsafe fn CeresRidley_Func_16(game: &mut Game) {
    let E: *mut Enemy_CeresRidley = Get_CeresRidley(0 as ::core::ffi::c_int, game);
    (*E).cry_var_A = ((0xa6a8a4i32) as u16);
    (*E).cry_var_F = 10 as uint16;
    (*E).cry_var_0A = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(1) = 0 as uint16;
    CeresRidley_Func_17(game);
}
pub unsafe fn CeresRidley_Func_17(game: &mut Game) {
    Ridley_Func_106(0 as uint16, 1 as uint16, 192 as uint16, 80 as uint16, game);
    let E: *mut Enemy_CeresRidley = Get_CeresRidley(0 as ::core::ffi::c_int, game);
    if (*E).base.y_pos as ::core::ffi::c_int - 96 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        (*E).cry_var_A = ((0xa6a8d4i32) as u16);
        (*E).cry_var_F = 10 as uint16;
        (*E).cry_var_0A = 0 as uint16;
    }
}
pub unsafe fn CeresRidley_Func_18(game: &mut Game) {
    Ridley_Func_111(
        -(32 as ::core::ffi::c_int) as uint16,
        -(1024 as ::core::ffi::c_int) as uint16,
        768 as uint16,
        game,
    );
    let E: *mut Enemy_CeresRidley = Get_CeresRidley(0 as ::core::ffi::c_int, game);
    (*E).cry_var_F = (*E).cry_var_F.wrapping_sub(1);
    if ((*E).cry_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        (*E).cry_var_A = ((0xa6a8f8i32) as u16);
        (*E).cry_var_F = 36 as uint16;
    }
}
pub unsafe fn CeresRidley_Func_19(game: &mut Game) {
    Ridley_Func_111(
        -(512 as ::core::ffi::c_int) as uint16,
        -(16384 as ::core::ffi::c_int) as uint16,
        768 as uint16,
        game,
    );
    let E: *mut Enemy_CeresRidley = Get_CeresRidley(0 as ::core::ffi::c_int, game);
    (*E).cry_var_F = (*E).cry_var_F.wrapping_sub(1);
    if ((*E).cry_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        (*E).cry_var_A = ((0xa6a923i32) as u16);
        (*E).cry_var_F = 28 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(2) =
            1 as uint16;
    }
}
pub unsafe fn CeresRidley_Func_20(game: &mut Game) {
    Ridley_Func_111(
        -(512 as ::core::ffi::c_int) as uint16,
        -(30720 as ::core::ffi::c_int) as uint16,
        768 as uint16,
        game,
    );
    let E: *mut Enemy_CeresRidley = Get_CeresRidley(0 as ::core::ffi::c_int, game);
    (*E).cry_var_F = (*E).cry_var_F.wrapping_sub(1);
    if ((*E).cry_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        (*E).cry_var_A = ((0xa6a947i32) as u16);
        (*E).cry_var_F = 1 as uint16;
    }
}
pub unsafe fn CeresRidley_Func_21(game: &mut Game) {
    Ridley_Func_111(
        -(768 as ::core::ffi::c_int) as uint16,
        -(30720 as ::core::ffi::c_int) as uint16,
        768 as uint16,
        game,
    );
    let E: *mut Enemy_CeresRidley = Get_CeresRidley(0 as ::core::ffi::c_int, game);
    (*E).cry_var_F = (*E).cry_var_F.wrapping_sub(1);
    if ((*E).cry_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        (*E).cry_var_A = ((0xa6a6e8i32) as u16);
        (*E).cry_var_00 = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(1) =
            1 as uint16;
    }
}
pub unsafe fn CeresRidley_Func_22(game: &mut Game) {
    let E: *mut Enemy_CeresRidley = Get_CeresRidley(0 as ::core::ffi::c_int, game);
    (*E).cry_var_20 = -(192 as ::core::ffi::c_int) as uint16;
    Ridley_Func_106(
        0 as uint16,
        1 as uint16,
        192 as uint16,
        -(128 as ::core::ffi::c_int) as uint16,
        game,
    );
    if (*E).base.y_pos as ::core::ffi::c_int + 128 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        CeresRidley_A994(game);
    }
}
pub unsafe fn CeresRidley_A994(game: &mut Game) {
    let E: *mut Enemy_CeresRidley = Get_CeresRidley(0 as ::core::ffi::c_int, game);
    (*E).cry_var_A = ((0xa6a9a0i32) as u16);
    (*E).cry_var_F = 64 as uint16;
    CeresRidley_Func_23(game);
}
pub unsafe fn CeresRidley_Func_23(game: &mut Game) {
    let E: *mut Enemy_CeresRidley = Get_CeresRidley(0 as ::core::ffi::c_int, game);
    (*E).cry_var_F = (*E).cry_var_F.wrapping_sub(1);
    if ((*E).cry_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        CeresRidley_Func_25(game);
        (*E).cry_var_B = 0 as uint16;
        (*E).cry_var_C = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(0) =
            0 as uint16;
        (*E).cry_var_A = ((0xa6aa11i32) as u16);
        WriteColorsToPalette(
            0xa2 as uint16,
            0xa6 as uint8,
            addr_word_A6A9E3 as uint16,
            0xf as uint16,
            game,
        );
        WriteColorsToPalette(
            0x42 as uint16,
            0xa6 as uint8,
            addr_word_A6AA01 as uint16,
            8 as uint16,
            game,
        );
        WriteColorsToPalette(
            0x1e2 as uint16,
            0xa6 as uint8,
            addr_word_A6AA01 as uint16,
            8 as uint16,
            game,
        );
    }
}
pub unsafe fn CeresRidley_Func_24(game: &mut Game) {
    (*Get_CeresRidley(0 as ::core::ffi::c_int, game)).cry_var_A = ((0xa6aa4fi32) as u16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x93f) as *mut uint16) = 1 as uint16;
    CeresRidley_Func_27(game);
}
pub unsafe fn CeresRidley_Func_25(game: &mut Game) {
    SpawnEnemy(0xa6 as uint8, addr_stru_A6AA2F as uint16, game);
    SpawnEnemy(0xa6 as uint8, addr_stru_A6AA3F as uint16, game);
}
pub unsafe fn CeresRidley_Func_26(game: &mut Game) {
    Ridley_Func_61(game);
}
pub unsafe fn CeresRidley_Func_27(game: &mut Game) {
    *(&raw mut game.g_ram as *mut uint8).offset(0x56) = 7 as uint8;
    *((&raw mut game.g_ram as *mut uint8).offset(0x783) as *mut uint16) = 1 as uint16;
    *(&raw mut game.g_ram as *mut uint8).offset(0x5f) = 0x80 as uint8;
    *((&raw mut game.g_ram as *mut uint8).offset(0x78) as *mut uint16) = 256 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x7a) as *mut uint16) = 256 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x7c) as *mut uint16) = 256 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x7e) as *mut uint16) = 256 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x80) as *mut uint16) = 64 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x82) as *mut uint16) = 64 as uint16;
    let E: *mut Enemy_CeresRidley = Get_CeresRidley(0 as ::core::ffi::c_int, game);
    (*E).cry_var_32 = 0 as uint16;
    (*E).cry_var_33 = 0 as uint16;
    (*E).cry_var_0A = 1 as uint16;
    (*E).cry_var_34 = -(128 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb1) as *mut uint16) =
        -(128 as ::core::ffi::c_int) as uint16;
    (*E).cry_var_35 = 32 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb3) as *mut uint16) = 32 as uint16;
    (*E).cry_var_30 = 2048 as uint16;
    (*E).cry_var_31 = 1024 as uint16;
}
pub unsafe fn HandleCeresRidleyGetawayCutscene(game: &mut Game) {
    if (*Get_CeresRidley(0 as ::core::ffi::c_int, game)).cry_var_32 == 0 {
        CeresRidley_Func_28(game);
    }
}
pub unsafe fn CeresRidley_Func_28(game: &mut Game) {
    let E: *mut Enemy_CeresRidley = Get_CeresRidley(0 as ::core::ffi::c_int, game);
    let cry_var_33: uint16 = (*E).cry_var_33;
    (*E).cry_var_33 = (cry_var_33).wrapping_add(2u16);
    if cry_var_33 == 0 {
        QueueSfx2_Max6(0x4e as uint16, game);
    }
    if cry_var_33 == 208u16 {
        Samus_SetPushedOutOfCeresRidley(game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x183e) as *mut uint16) = 35 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1840) as *mut uint16) = 64 as uint16;
    }
    let v2: ::core::ffi::c_int = cry_var_33 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *(game.g_rom.offset(crate::rtl::lorom_offset(0xa6ae4d)) as *const uint8 as *mut uint16)
        .offset(v2 as isize)
        == 0xffffu16
    {
        (*E).cry_var_32 = (*E).cry_var_32.wrapping_add(1);
        *(&raw mut game.g_ram as *mut uint8).offset(0x56) = 9 as uint8;
        *(&raw mut game.g_ram as *mut uint8).offset(0x5f) = 0 as uint8;
        *((&raw mut game.g_ram as *mut uint8).offset(0x78) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x7a) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x7c) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x7e) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x80) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x82) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb1) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb3) as *mut uint16) = 0 as uint16;
        (*E).cry_var_A = ((0xa6c04ei32) as u16);
        (*E).cry_var_F = 0 as uint16;
    } else {
        (*E).cry_var_30 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa6ae4d)) as *const uint8
            as *mut uint16)
            .offset(v2 as isize);
        let v3: uint16 = (*(game.g_rom.offset(crate::rtl::lorom_offset(0xa6af2f)) as *const uint8
            as *mut uint16)
            .offset(v2 as isize))
        .wrapping_add((*E).cry_var_35);
        (*E).cry_var_35 = v3;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb3) as *mut uint16) = v3;
        let v4: uint16 = ((*E).cry_var_34).wrapping_sub(
            *(game.g_rom.offset(crate::rtl::lorom_offset(0xa6b00f)) as *const uint8 as *mut uint16)
                .offset(v2 as isize),
        );
        (*E).cry_var_34 = v4;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb1) as *mut uint16) = v4;
        CeresRidley_Func_35((*E).cry_var_30, game);
        (*E).cry_var_0A = ((*E).cry_var_0A).wrapping_add(48u16);
        CeresRidley_Func_29(game);
        CeresRidley_Func_33(game);
        CeresRidley_Func_34(game);
    };
}
unsafe fn CeresMult(a: uint16, b: uint16) -> uint16 {
    let r: uint16 = (abs16(a) as ::core::ffi::c_int * b as ::core::ffi::c_int
        >> 8 as ::core::ffi::c_int) as uint16;
    ((if a as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        -(r as ::core::ffi::c_int)
    } else {
        r as ::core::ffi::c_int
    }) as uint16)
}
pub unsafe fn CeresRidley_Func_29(game: &mut Game) {
    let E: *mut Enemy_CeresRidley = Get_CeresRidley(0 as ::core::ffi::c_int, game);
    let r18: uint16 = *(&raw mut (*E).cry_var_0A as *mut uint8).offset(
        (::core::mem::size_of::<uint16>())
            .wrapping_div(::core::mem::size_of::<uint8>())
            .wrapping_sub(1_usize) as isize,
    ) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x78) as *mut uint16) = CeresMult(
        CeresRidley_AC30((r18).wrapping_add(64u16), 0x100 as int16) as uint16,
        (*E).cry_var_30,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x7a) as *mut uint16) = CeresMult(
        CeresRidley_AC30(r18, 0x100 as int16) as uint16,
        (*E).cry_var_30,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x7c) as *mut uint16) = CeresMult(
        -(CeresRidley_AC30(r18, 0x100 as int16) as ::core::ffi::c_int) as uint16,
        (*E).cry_var_30,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x7e) as *mut uint16) = CeresMult(
        CeresRidley_AC30((r18).wrapping_add(64u16), 0x100 as int16) as uint16,
        (*E).cry_var_30,
    );
}
pub unsafe fn CeresRidley_AC30(a: uint16, r20: int16) -> int16 {
    (kSinCosTable8bit_Sext[((a as ::core::ffi::c_int & 0xff as ::core::ffi::c_int)
        + 64 as ::core::ffi::c_int) as usize] as ::core::ffi::c_int
        * r20 as ::core::ffi::c_int
        >> 8 as ::core::ffi::c_int) as int16
}
pub unsafe fn CeresRidley_Func_33(game: &mut Game) {
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x5b6) as *mut uint16) as ::core::ffi::c_int
        & 3 as ::core::ffi::c_int)
        == 0i32
    {
        let E: *mut Enemy_CeresRidley = Get_CeresRidley(0 as ::core::ffi::c_int, game);
        let v1: uint16 = ((*E).cry_var_36 as ::core::ffi::c_int + 1 as ::core::ffi::c_int
            & 3 as ::core::ffi::c_int) as uint16;
        (*E).cry_var_36 = v1;
        QueueMode7Transfers(
            0xa6 as uint8,
            *(game.g_rom.offset(crate::rtl::lorom_offset(0xa6acda)) as *const uint8 as *mut uint16)
                .offset(v1 as isize),
            game,
        );
    }
}
pub unsafe fn CeresRidley_Func_34(game: &mut Game) {
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x5b6) as *mut uint16) as ::core::ffi::c_int
        & 7 as ::core::ffi::c_int)
        == 0i32
    {
        let E: *mut Enemy_CeresRidley = Get_CeresRidley(0 as ::core::ffi::c_int, game);
        let v1: uint16 = ((*E).cry_var_37 as ::core::ffi::c_int + 1 as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int) as uint16;
        (*E).cry_var_37 = v1;
        QueueMode7Transfers(
            0xa6 as uint8,
            *(game.g_rom.offset(crate::rtl::lorom_offset(0xa6ad45)) as *const uint8 as *mut uint16)
                .offset(v1 as isize),
            game,
        );
    }
}
pub unsafe fn CeresRidley_Func_35(mut a: uint16, game: &mut Game) {
    WriteColorsToPalette(
        0xa2 as uint16,
        0xa6 as uint8,
        (32 as ::core::ffi::c_int
            * *(&raw mut a as *mut uint8).offset(
                (::core::mem::size_of::<uint16>())
                    .wrapping_div(::core::mem::size_of::<uint8>())
                    .wrapping_sub(1_usize) as isize,
            ) as ::core::ffi::c_int
            - 0x4ef9 as ::core::ffi::c_int) as uint16,
        0xf as uint16,
        game,
    );
}
pub unsafe fn Ridley_Main(game: &mut Game) {
    let mut v1: int16 = 0;
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    v1 = (((*E).ridley_var_24) as i16).wrapping_sub(4i16);
    if v1 < 0i16 {
        v1 = 0 as int16;
    }
    (*E).ridley_var_24 = v1 as uint16;
    Ridley_Func_39(game);
    Ridley_Func_42(game);
    CallRidleyFunc((((*E).ridley_var_A) as u32) | (0xa60000u32), game);
    if (*E).ridley_var_02 != 0 {
        Ridley_Func_102(game);
        Ridley_Func_112(game);
        Ridley_Func_115(game);
        Ridley_Func_70(game);
        Ridley_Func_129(game);
        sub_A6DB2A(game);
        Ridley_Func_120(game);
        Ridley_Func_118(game);
        if (*E).ridley_var_1B != 0 {
            Ridley_Func_30(game);
        }
    }
    Ridley_Func_100(game);
}
pub unsafe fn Ridley_Func_1(game: &mut Game) {
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    if (*E).ridley_var_1B as ::core::ffi::c_int | (*E).ridley_var_1E as ::core::ffi::c_int == 0 {
        if Ridley_Func_26(4 as uint16, 4 as uint16, game) as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            != 0
        {
            Ridley_Func_37(game);
        }
    }
}
pub unsafe fn Ridley_Func_2(game: &mut Game) {
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    (*E).ridley_var_24 = 0 as uint16;
    (*E).base.frame_counter = 1 as uint16;
    Ridley_Hurt(game);
}
pub unsafe fn Ridley_Hurt(game: &mut Game) {
    let current_block: u64;
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    if ((*E).base.frame_counter as ::core::ffi::c_int & 1 as ::core::ffi::c_int) == 0i32 {
        Ridley_Func_39(game);
        Ridley_Func_42(game);
        CallRidleyFunc((((*E).ridley_var_A) as u32) | (0xa60000u32), game);
        if (*Get_Ridley(0 as ::core::ffi::c_int, game)).ridley_var_02 == 0 {
            current_block = 18350095613861854843;
        } else {
            Ridley_Func_112(game);
            Ridley_Func_115(game);
            Ridley_Func_70(game);
            current_block = 13513818773234778473;
        }
    } else {
        current_block = 13513818773234778473;
    }
    match current_block {
        13513818773234778473 => {
            Ridley_Func_129(game);
            Ridley_Func_102(game);
            sub_A6DB2A(game);
            Ridley_Func_120(game);
            Ridley_Func_100(game);
            Ridley_Func_118(game);
            let mut v2: uint16 = ((*E).ridley_var_24).wrapping_add(1u16);
            if ((v2) as i16).wrapping_sub((g_word_A6B288) as i16) >= 0i16 {
                v2 = g_word_A6B288;
            }
            (*E).ridley_var_24 = v2;
            if !((((*E).ridley_var_24) as i16).wrapping_sub((g_word_A6B288) as i16) >= 0i16) {
                return;
            }
        }
        _ => {}
    }
    if (*Get_Ridley(0 as ::core::ffi::c_int, game)).ridley_var_1B != 0 {
        Ridley_Func_30(game);
    }
}
pub unsafe fn Ridley_Func_3(game: &mut Game) {
    let rect: Rect16U = Rect16U {
        x: 64 as uint16,
        y: 256 as uint16,
        w: 8 as uint16,
        h: 8 as uint16,
    };
    Ridley_Func_104_0(0 as uint16, 14 as uint16, rect.x, rect.y, game);
    if Shitroid_Func_2(0 as uint16, rect, game) == 0 {
        let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
        (*E).ridley_var_01 = 1 as uint16;
        (*E).ridley_var_A = ((0xa6b321i32) as u16);
        Ridley_Func_3b(game);
    }
}
pub unsafe fn Ridley_Func_3b(game: &mut Game) {
    let r18: uint16 = Ridley_Func_4(game);
    let v0: ::core::ffi::c_int = 2 as ::core::ffi::c_int
        * (NextRandom(game) as ::core::ffi::c_int & 7 as ::core::ffi::c_int);
    let v1: uint16 = *((RomPtr_A6 as unsafe fn(uint16_t, &mut Game) -> *const uint8)(r18, game)
        .offset(v0 as isize) as *const uint8 as *mut uint16);
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    (*E).ridley_var_A = v1;
    CallRidleyFunc((((*E).ridley_var_A) as u32) | (0xa60000u32), game);
}
pub unsafe fn Ridley_Func_4(game: &mut Game) -> uint16 {
    let mut r18: uint16 = 0;
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa1f) == 3u8 {
        r18 = addr_off_A6B3CC as uint16;
    } else {
        let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
        let health: uint16 = (*E).base.health;
        if health != 0 {
            if health as ::core::ffi::c_int - 14400 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
            {
                r18 = addr_off_A6B38C as uint16;
            } else if Ridley_Func_41(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
                r18 = addr_off_A6B3BC as uint16;
            } else if (Ridley_Func_40_Carry(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int)
                != 0i32
            {
                r18 = addr_off_A6B3AC as uint16;
            } else {
                r18 = (if (*E).base.health as ::core::ffi::c_int - 9000 as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    != 0
                {
                    addr_off_A6B39C as ::core::ffi::c_int
                } else {
                    addr_off_A6B38C as ::core::ffi::c_int
                }) as uint16;
            }
        } else {
            r18 = addr_off_A6B3DC as uint16;
            (*E).ridley_var_25 = (*E).ridley_var_25.wrapping_add(1);
        }
    }
    r18
}
pub unsafe fn Ridley_Func_5(game: &mut Game) {
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    (*E).ridley_var_A = ((0xa6b3f8i32) as u16);
    (*E).ridley_var_F = 128 as uint16;
    Ridley_Func_6(game);
}
pub unsafe fn Ridley_Func_6(game: &mut Game) {
    let mut v2: uint16 = 0;
    let mut rect: Rect16U = Rect16U {
        x: 0,
        y: 0,
        w: 0,
        h: 0,
    };
    let current_block: u64;
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    (*E).ridley_var_F = (*E).ridley_var_F.wrapping_sub(1);
    if ((*E).ridley_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        current_block = 10433200308384073363;
    } else {
        v2 = 0;
        v2 = 192 as uint16;
        if (*E).ridley_var_10 != 0 {
            v2 = 96 as uint16;
        }
        rect = Rect16U {
            x: v2,
            y: 256 as uint16,
            w: 8 as uint16,
            h: 8 as uint16,
        };
        Ridley_Func_104_0(0 as uint16, Ridley_Func_7(game), rect.x, rect.y, game);
        if Shitroid_Func_2(0 as uint16, rect, game) == 0 {
            current_block = 10433200308384073363;
        } else {
            current_block = 11006700562992250127;
        }
    }
    match current_block {
        10433200308384073363 => {
            (*E).ridley_var_A = ((0xa6b321i32) as u16);
        }
        _ => {}
    };
}
pub unsafe fn Ridley_Func_7(game: &mut Game) -> uint16 {
    static g_word_A6B439: [uint16; 4] = [4, 8, 0xa, 0xc];
    g_word_A6B439[(*Get_Ridley(0 as ::core::ffi::c_int, game)).ridley_var_12 as usize]
}
pub unsafe fn Ridley_Func_8(game: &mut Game) {
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    (*E).ridley_var_A = addr_loc_A6B455 as uint16;
    (*E).ridley_var_00 = 10 as uint16;
    (*E).ridley_var_0A = 0 as uint16;
    Ridley_B455(game);
}
pub unsafe fn Ridley_B455(game: &mut Game) {
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    let mut v1: uint16 = 192 as uint16;
    if (*E).ridley_var_10 != 0 {
        v1 = 64 as uint16;
    }
    let rect: Rect16U = Rect16U {
        x: v1,
        y: 128 as uint16,
        w: 8 as uint16,
        h: 8 as uint16,
    };
    Ridley_Func_104_0(0 as uint16, 1 as uint16, rect.x, rect.y, game);
    if Shitroid_Func_2(0 as uint16, rect, game) == 0 {
        (*E).ridley_var_A = ((0xa6b493i32) as u16);
        (*E).ridley_var_00 = 32 as uint16;
        (*E).ridley_var_0A = 0 as uint16;
    }
}
pub unsafe fn Ridley_Func_9(game: &mut Game) {
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    if (*E).ridley_var_10 != 0 {
        Ridley_Func_111(32 as uint16, 512 as uint16, 1152 as uint16, game);
    } else {
        Ridley_Func_111(
            -(32 as ::core::ffi::c_int) as uint16,
            -(512 as ::core::ffi::c_int) as uint16,
            1152 as uint16,
            game,
        );
    }
    let ridley_var_00: uint16 = (*E).ridley_var_00;
    if ridley_var_00 != 0 {
        (*E).ridley_var_00 = (ridley_var_00).wrapping_sub(1u16);
    } else {
        (*E).ridley_var_A = ((0xa6b4d1i32) as u16);
        (*E).ridley_var_00 = 20 as uint16;
    };
}
pub unsafe fn Ridley_Func_10(game: &mut Game) {
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    if (*E).ridley_var_10 != 0 {
        Ridley_Func_111(320 as uint16, 0x4000 as uint16, 1280 as uint16, game);
    } else {
        Ridley_Func_111(
            -(320 as ::core::ffi::c_int) as uint16,
            -(16384 as ::core::ffi::c_int) as uint16,
            1280 as uint16,
            game,
        );
    }
    let ridley_var_00: uint16 = (*E).ridley_var_00;
    if ridley_var_00 != 0 {
        (*E).ridley_var_00 = (ridley_var_00).wrapping_sub(1u16);
    } else {
        (*E).ridley_var_A = ((0xa6b516i32) as u16);
        (*E).ridley_var_00 = 16 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(2) =
            1 as uint16;
    };
}
pub unsafe fn Ridley_Func_11(game: &mut Game) {
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    if (*E).ridley_var_10 != 0 {
        Ridley_Func_111(512 as uint16, 30720 as uint16, 768 as uint16, game);
    } else {
        Ridley_Func_111(
            -(512 as ::core::ffi::c_int) as uint16,
            -(30720 as ::core::ffi::c_int) as uint16,
            768 as uint16,
            game,
        );
    }
    let ridley_var_00: uint16 = (*E).ridley_var_00;
    if ridley_var_00 != 0 {
        (*E).ridley_var_00 = (ridley_var_00).wrapping_sub(1u16);
    } else {
        (*E).ridley_var_A = ((0xa6b554i32) as u16);
        (*E).ridley_var_00 = 32 as uint16;
    };
}
pub unsafe fn Ridley_Func_12(game: &mut Game) {
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    if (*E).ridley_var_10 != 0 {
        Ridley_Func_111(1024 as uint16, 30720 as uint16, 768 as uint16, game);
    } else {
        Ridley_Func_111(
            -(1024 as ::core::ffi::c_int) as uint16,
            -(30720 as ::core::ffi::c_int) as uint16,
            768 as uint16,
            game,
        );
    }
    let ridley_var_00: uint16 = (*E).ridley_var_00;
    if ridley_var_00 != 0 {
        (*E).ridley_var_00 = (ridley_var_00).wrapping_sub(1u16);
    } else {
        (*E).ridley_var_A = ((0xa6b594i32) as u16);
        (*E).ridley_var_00 = 32 as uint16;
        Ridley_Func_114(game);
    };
}
pub unsafe fn Ridley_Func_13(game: &mut Game) {
    Ridley_Func_111(0 as uint16, 0x8000 as uint16, 448 as uint16, game);
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    let ridley_var_00: uint16 = (*E).ridley_var_00;
    if ridley_var_00 != 0 {
        (*E).ridley_var_00 = (ridley_var_00).wrapping_sub(1u16);
    } else {
        (*E).ridley_var_A = (if Ridley_Func_40_Carry(game) != 0u8 {
            0xa6bab7 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
        } else {
            0xa6b321 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
        }) as uint16;
    };
}
pub unsafe fn Ridley_Func_14(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(15) = 11 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(9) = 384 as uint16;
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    (*E).ridley_var_A = ((0xa6b5e5i32) as u16);
    (*E).ridley_var_F = ((*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
        as ::core::ffi::c_int
        & 0x1f as ::core::ffi::c_int)
        + 32 as ::core::ffi::c_int) as uint16;
    Ridley_Func_15(game);
}
pub unsafe fn Ridley_Func_15(game: &mut Game) {
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    if Ridley_Func_17(
        *(game.g_rom.offset(crate::rtl::lorom_offset(0xa6b60d)) as *const uint8 as *mut uint16)
            .offset((*E).ridley_var_10 as isize),
        game,
    ) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        Ridley_Func_18(game);
    } else {
        (*E).ridley_var_F = (*E).ridley_var_F.wrapping_sub(1);
        if ((*E).ridley_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
            (*E).ridley_var_A = ((0xa6b613i32) as u16);
            (*E).ridley_var_F = 128 as uint16;
            Ridley_Func_114(game);
        }
    };
}
pub unsafe fn Ridley_Func_16(game: &mut Game) {
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    if Ridley_Func_17(
        *(game.g_rom.offset(crate::rtl::lorom_offset(0xa6b63b)) as *const uint8 as *mut uint16)
            .offset((*E).ridley_var_10 as isize),
        game,
    ) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        Ridley_Func_18(game);
    } else {
        (*E).ridley_var_F = (*E).ridley_var_F.wrapping_sub(1);
        if ((*E).ridley_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
            (*E).ridley_var_A = ((0xa6b5e5i32) as u16);
            (*E).ridley_var_F = 128 as uint16;
            Ridley_Func_114(game);
        }
    };
}
pub unsafe fn Ridley_Func_17(r18: uint16, game: &mut Game) -> uint8 {
    let mut v0: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16);
    if *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16) as ::core::ffi::c_int
        - 352 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        v0 = 352 as uint16;
    }
    Ridley_Func_104_0(0 as uint16, Ridley_Func_7(game), r18, v0, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(2) = 1 as uint16;
    if *(&raw mut game.g_ram as *mut uint8).offset(0xa1f) != 3u8 {
        return 1 as uint8;
    }
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16) as uint8) >= 0x80u8 {
        let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
        if (*E).ridley_var_0F == 0 && (*E).ridley_var_10 != 1u16 {
            Ridley_Func_99(addr_kRidley_Ilist_E73A as uint16, game);
        }
    }
    0 as uint8
}
pub unsafe fn Ridley_Func_18(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(9) = 240 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(15) = 16 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(0) = 1 as uint16;
    (*Get_Ridley(0 as ::core::ffi::c_int, game)).ridley_var_A = ((0xa6b6a7i32) as u16);
}
pub unsafe fn Ridley_Func_19(game: &mut Game) {
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    if (*E).base.y_pos as ::core::ffi::c_int - 288 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        Ridley_Func_114(game);
        (*E).ridley_var_A = ((0xa6b6ddi32) as u16);
        (*E).ridley_var_F = 32 as uint16;
        Ridley_Func_20(game);
    } else {
        Ridley_Func_104_0(
            0 as uint16,
            0 as uint16,
            *(game.g_rom.offset(crate::rtl::lorom_offset(0xa6b6c8)) as *const uint8 as *mut uint16)
                .offset((*E).ridley_var_10 as isize),
            288 as uint16,
            game,
        );
    };
}
pub unsafe fn Ridley_Func_20(game: &mut Game) {
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    Ridley_Func_104_0(
        0 as uint16,
        0 as uint16,
        (*E).base.x_pos,
        288 as uint16,
        game,
    );
    (*E).ridley_var_F = (*E).ridley_var_F.wrapping_sub(1);
    if ((*E).ridley_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        Ridley_Func_71(game);
        Ridley_Func_29(game);
        (*E).ridley_var_A = ((0xa6b70ei32) as u16);
        (*E).ridley_var_F = ((*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
            as ::core::ffi::c_int
            & 0x3f as ::core::ffi::c_int)
            + 128 as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn Ridley_Func_21(game: &mut Game) {
    let mut v1: int16 = 0;
    if Ridley_Func_25(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        Ridley_Func_24(game);
        Ridley_Func_27(game);
    } else {
        let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
        v1 = ((*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(6))
            as i16)
            .wrapping_add(((*E).ridley_var_C) as i16);
        if v1 as ::core::ffi::c_int - 1536 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0
        {
            v1 = 1536 as int16;
        }
        (*E).ridley_var_C = v1 as uint16;
        if Ridley_Func_23(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
            game.eproj_spawn_pt = Point16U {
                x: *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(82),
                y: ((*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                    .offset(83))
                .wrapping_add(12u16)),
            };
            SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as uint16, 9 as uint16, game);
            QueueSfx2_Max6(0x76 as uint16, game);
            *((&raw mut game.g_ram as *mut uint8).offset(0x183e) as *mut uint16) = 13 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1840) as *mut uint16) = 4 as uint16;
            Ridley_Func_28(game);
            Ridley_Func_29(game);
            Ridley_Func_93(0x2026 as uint16, addr_word_A6D37C as uint16, game);
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(20) =
                3072 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(30) =
                3072 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(40) =
                3072 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(50) =
                3072 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(60) =
                3072 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(70) =
                3072 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(80) =
                3072 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(0) =
                4 as uint16;
            let mut v2: uint16 = ((*E).ridley_var_26).wrapping_add(1u16);
            if (*E).ridley_var_26 as ::core::ffi::c_int - 1 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0
            {
                if (*E).ridley_var_10 != 1u16 {
                    Ridley_Func_99(addr_kRidley_Ilist_E73A as uint16, game);
                }
                v2 = 0 as uint16;
            }
            (*E).ridley_var_26 = v2;
            (*E).ridley_var_A = ((0xa6b7b9i32) as u16);
        }
    };
}
pub unsafe fn Ridley_Func_22(game: &mut Game) {
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    let mut v1: bool = false;
    if Ridley_Func_41(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int == 0 || {
        v1 = (((*E).ridley_var_F) as i16).wrapping_sub(1i16) < 0i16;
        (*E).ridley_var_F = (*E).ridley_var_F.wrapping_sub(1);
        v1 as ::core::ffi::c_int != 0
    } {
        Ridley_Func_24(game);
        (*E).ridley_var_A = ((0xa6b321i32) as u16);
    } else {
        let ridley_var_C: uint16 = (*E).ridley_var_C;
        v1 = ((*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(7))
            as i16)
            .wrapping_add((ridley_var_C) as i16)
            < 0i16;
        (*E).ridley_var_C = (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset(7))
        .wrapping_add(ridley_var_C);
        if !v1 {
            (*E).ridley_var_C = 0 as uint16;
            (*E).ridley_var_A = ((0xa6b70ei32) as u16);
        }
    };
}
pub unsafe fn Ridley_Func_23(game: &mut Game) -> uint8 {
    let mut v0: uint8 = (Ridley_Func_103(
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(82),
        (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(83))
            .wrapping_add(16u16),
        game,
    )) & (1u8);
    if v0 == 0 {
        v0 = (Ridley_Func_103(
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(72),
            (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(73))
                .wrapping_add(18u16),
            game,
        )) & (1u8);
        if v0 == 0 {
            v0 = (Ridley_Func_103(
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(62),
                (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(63))
                    .wrapping_add(18u16),
                game,
            )) & (1u8);
            if v0 == 0 {
                v0 = (Ridley_Func_103(
                    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(52),
                    (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                        .offset(53))
                    .wrapping_add(18u16),
                    game,
                )) & (1u8);
                if v0 == 0 {
                    return (Ridley_Func_103(
                        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                            .offset(42),
                        (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                            .offset(43))
                        .wrapping_add(18u16),
                        game,
                    )) & (1u8);
                }
            }
        }
    }
    v0
}
pub unsafe fn Ridley_Func_24(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(0) = 1 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(10) = 1 as uint16;
}
pub unsafe fn Ridley_Func_25(game: &mut Game) -> uint8 {
    if (Ridley_Func_40_Carry(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int) != 0i32 {
        (Ridley_Func_26(4 as uint16, 4 as uint16, game)) & (1u8)
    } else {
        0 as uint8
    }
}
pub unsafe fn Ridley_Func_26(k: uint16, j: uint16, game: &mut Game) -> uint8 {
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    let rect: Rect16U = Rect16U {
        x: (((*E).base.x_pos).wrapping_add(
            *(game.g_rom.offset(crate::rtl::lorom_offset(0xa6b9d5)) as *const uint8 as *mut uint16)
                .offset((*E).ridley_var_10 as isize),
        )),
        y: (((*E).base.y_pos).wrapping_add(
            *(game.g_rom.offset(crate::rtl::lorom_offset(0xa6b9db)) as *const uint8 as *mut uint16)
                .offset(
                    ((*E).ridley_var_1D as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize,
                ),
        )),
        w: k,
        h: j,
    };
    Ridley_Func_124(rect, game)
}
pub unsafe fn Ridley_Func_27(game: &mut Game) {
    let mut ridley_var_C: int16 = 0;
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    ridley_var_C = (*E).ridley_var_C as int16;
    if ridley_var_C >= 0i16 {
        ridley_var_C = -(ridley_var_C as ::core::ffi::c_int) as int16;
    }
    if ridley_var_C as ::core::ffi::c_int + 512 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        == 0
    {
        ridley_var_C = -(512 as ::core::ffi::c_int) as int16;
    }
    (*E).ridley_var_C = ridley_var_C as uint16;
    Ridley_Func_24(game);
    (*E).ridley_var_A = ((0xa6bb8fi32) as u16);
    Ridley_Func_33b(game);
}
pub unsafe fn Ridley_Func_28(game: &mut Game) {
    let mut x_pos: uint16 = 0;
    let mut v4: uint16 = 0;
    let current_block: u64;
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    if (*E).ridley_var_B == 0 {
        (*E).ridley_var_B =
            (if (*E).base.x_pos as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int != 0 {
                -(0xc0 as ::core::ffi::c_int)
            } else {
                0xc0 as ::core::ffi::c_int
            }) as uint16;
    }
    if (((*E).ridley_var_B as ::core::ffi::c_int
        ^ (*E).base.x_pos as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
                as ::core::ffi::c_int)
        & 0x8000 as ::core::ffi::c_int)
        == 0i32
    {
        x_pos = 0;
        x_pos = (*E).base.x_pos;
        if ((x_pos) as i16).wrapping_sub(((*E).ridley_var_22) as i16) >= 0i16 {
            if ((x_pos) as i16).wrapping_sub(((*E).ridley_var_23) as i16) < 0i16 {
                if *((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16) < 0x555u16 {
                    return;
                }
                current_block = 7587493754414084492;
            } else {
                current_block = 12558520430340354144;
            }
        } else {
            current_block = 3419095937434320988;
        }
    } else {
        v4 = 0;
        v4 = (*E).base.x_pos;
        if ((v4) as i16).wrapping_sub(((*E).ridley_var_22) as i16) < 0i16 {
            current_block = 3419095937434320988;
        } else if ((v4) as i16).wrapping_sub(((*E).ridley_var_23) as i16) >= 0i16 {
            current_block = 12558520430340354144;
        } else if *((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16) < 0x555u16 {
            current_block = 7587493754414084492;
        } else {
            return;
        }
    }
    match current_block {
        3419095937434320988 => {
            if ((*E).ridley_var_B as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) == 0i32 {
                return;
            }
        }
        12558520430340354144 => {
            if ((*E).ridley_var_B as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
                return;
            }
        }
        _ => {}
    }
    (*E).ridley_var_B = -((*E).ridley_var_B as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Ridley_Func_29(game: &mut Game) {
    let v0: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
        as ::core::ffi::c_int
        & 3 as ::core::ffi::c_int;
    let r18: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa6b965)) as *const uint8
        as *mut uint16)
        .offset(v0 as isize);
    let r20: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa6b96d)) as *const uint8
        as *mut uint16)
        .offset(v0 as isize);
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    let v2: uint16 = (2 as ::core::ffi::c_int
        * ((*E).ridley_var_12 as ::core::ffi::c_int + 2 as ::core::ffi::c_int))
        as uint16;
    let v3: ::core::ffi::c_int = v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(7) =
        *(game.g_rom.offset(crate::rtl::lorom_offset(0xa6b94d)) as *const uint8 as *mut uint16)
            .offset(v3 as isize);
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(6) =
        *(game.g_rom.offset(crate::rtl::lorom_offset(0xa6b959)) as *const uint8 as *mut uint16)
            .offset(v3 as isize);
    (*E).ridley_var_C = *((RomPtr_A6 as unsafe fn(uint16_t, &mut Game) -> *const uint8)(r20, game)
        .offset(v2 as isize) as *const uint8 as *mut uint16);
    let v4: *const uint8 = RomPtr_A6(r18, game);
    let t: uint16 = *(v4.offset(v2 as isize) as *const uint8 as *mut uint16);
    (*E).ridley_var_B =
        (if (*E).ridley_var_B as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
            -(t as ::core::ffi::c_int)
        } else {
            t as ::core::ffi::c_int
        }) as uint16;
}
pub unsafe fn Ridley_Func_30(game: &mut Game) {
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    let ridley_var_14: uint16 = (*E).ridley_var_14;
    let mut v3: uint16 = 0;
    let mut v6: uint16 = 0;
    if ridley_var_14 != 0 {
        let v2: int16 = ((abs16(ridley_var_14)) as i16).wrapping_sub(4i16);
        if v2 >= 0i16 {
            v3 = (if ridley_var_14 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
                -(v2 as ::core::ffi::c_int)
            } else {
                v2 as ::core::ffi::c_int
            }) as uint16;
        } else {
            v3 = 0 as uint16;
        }
        (*E).ridley_var_14 = v3;
    }
    let ridley_var_15: uint16 = (*E).ridley_var_15;
    if ridley_var_15 != 0 {
        let v5: int16 = ((abs16(ridley_var_15)) as i16).wrapping_sub(4i16);
        if v5 >= 0i16 {
            v6 = (if ridley_var_15 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
                -(v5 as ::core::ffi::c_int)
            } else {
                v5 as ::core::ffi::c_int
            }) as uint16;
        } else {
            v6 = 0 as uint16;
        }
        (*E).ridley_var_15 = v6;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) = ((*E).ridley_var_14
        as ::core::ffi::c_int
        + (*E).base.x_pos as ::core::ffi::c_int
        + *(game.g_rom.offset(crate::rtl::lorom_offset(0xa6b9d5)) as *const uint8 as *mut uint16)
            .offset((*E).ridley_var_10 as isize) as ::core::ffi::c_int)
        as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16) = ((*E).ridley_var_15
        as ::core::ffi::c_int
        + (*E).base.y_pos as ::core::ffi::c_int
        + *(game.g_rom.offset(crate::rtl::lorom_offset(0xa6b9db)) as *const uint8 as *mut uint16)
            .offset(((*E).ridley_var_1D as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int)
        as uint16;
}
pub unsafe fn Ridley_Func_31(game: &mut Game) {
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    (*E).ridley_var_14 =
        (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) as ::core::ffi::c_int
            - ((*E).base.x_pos as ::core::ffi::c_int
                + *(game.g_rom.offset(crate::rtl::lorom_offset(0xa6b9d5)) as *const uint8
                    as *mut uint16)
                    .offset((*E).ridley_var_10 as isize) as ::core::ffi::c_int)) as uint16;
    (*E).ridley_var_15 = (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
        as ::core::ffi::c_int
        - ((*E).base.y_pos as ::core::ffi::c_int
            + *(game.g_rom.offset(crate::rtl::lorom_offset(0xa6b9db)) as *const uint8
                as *mut uint16)
                .offset(
                    ((*E).ridley_var_1D as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize,
                ) as ::core::ffi::c_int)) as uint16;
}
pub unsafe fn Ridley_Func_32(game: &mut Game) {
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    if (*E).ridley_var_25 as ::core::ffi::c_int - 10 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0xcee) as *mut uint16) != 0 {
            (*E).ridley_var_A = ((0xa6bd4ei32) as u16);
            Ridley_Func_43(game);
        } else {
            (*E).ridley_var_A = ((0xa6b3eci32) as u16);
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(2) =
                1 as uint16;
        }
    } else {
        Ridley_Func_126(game);
        (*E).ridley_var_A = ((0xa6c53ei32) as u16);
        Ridley_C53E(game);
    };
}
pub unsafe fn Ridley_Func_33(game: &mut Game) {
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    let mut v1: uint16 = 0;
    let mut v2: int16 = 0;
    let mut r18: uint16 = 0;
    let mut r20: uint16 = 0;
    if (Ridley_Func_40_Carry(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int) != 0i32
        && (*E).ridley_var_1F == 0
        && {
            r18 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa6bb48)) as *const uint8
                as *mut uint16)
                .offset((*E).ridley_var_10 as isize);
            r20 = ((*E).base.x_pos)
                .wrapping_sub(*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16));
            ((r18 as ::core::ffi::c_int ^ r20 as ::core::ffi::c_int) & 0x8000 as ::core::ffi::c_int)
                == 0i32
                || {
                    v1 = abs16(r20);
                    v1 as ::core::ffi::c_int - 32 as ::core::ffi::c_int
                        & 0x8000 as ::core::ffi::c_int
                        != 0
                }
        }
        && (((*E).base.y_pos as ::core::ffi::c_int + 35 as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
                as ::core::ffi::c_int) as int16)
            < 0i16
    {
        v2 = -(16 as ::core::ffi::c_int) as int16;
        if (*E).ridley_var_10 != 0 {
            v2 = 16 as int16;
        }
        Ridley_Func_104_0(
            0 as uint16,
            *(game.g_rom.offset(crate::rtl::lorom_offset(0xa6bb4e)) as *const uint8 as *mut uint16)
                .offset((*E).ridley_var_12 as isize),
            (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16))
                .wrapping_add((v2) as u16),
            (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16))
                .wrapping_sub(4u16),
            game,
        );
        let rect: Rect16U = Rect16U {
            x: (((*E).base.x_pos).wrapping_add(
                *(game.g_rom.offset(crate::rtl::lorom_offset(0xa6b9d5)) as *const uint8
                    as *mut uint16)
                    .offset((*E).ridley_var_10 as isize),
            )),
            y: (((*E).base.y_pos).wrapping_add(35u16)),
            w: 8 as uint16,
            h: 12 as uint16,
        };
        if Ridley_Func_124(rect, game) != 0 {
            (*E).ridley_var_C = -((*E).ridley_var_C as ::core::ffi::c_int) as uint16;
            if (*E).base.health != 0 {
                if *((&raw mut game.g_ram as *mut uint8).offset(0xcee) as *mut uint16) != 0 {
                    Ridley_Func_37(game);
                    (*E).ridley_var_A = ((0xa6bd4ei32) as u16);
                    Ridley_Func_43(game);
                } else {
                    Ridley_Func_33b(game);
                }
            } else {
                if (*E).ridley_var_1B == 0 {
                    Ridley_Func_37(game);
                }
                Ridley_Func_126(game);
                (*E).ridley_var_A = ((0xa6c538i32) as u16);
                Ridley_Func_63(game);
            }
        }
    } else {
        Ridley_Func_32(game);
    };
}
pub unsafe fn Ridley_Func_33b(game: &mut Game) {
    let mut v1: int16 = 0;
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    (*E).ridley_var_17 = g_word_A6BBEB[(*E).ridley_var_10 as usize];
    v1 = (((*E).base.y_pos) as i16).wrapping_sub(64i16);
    if (*E).base.y_pos as ::core::ffi::c_int - 320 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        v1 = 256 as int16;
    }
    (*E).ridley_var_18 = v1 as uint16;
    if (*E).ridley_var_1B == 0 {
        Ridley_Func_37(game);
    }
    (*E).ridley_var_A = ((0xa6bbc4i32) as u16);
    (*E).ridley_var_F = 32 as uint16;
    Ridley_Func_34(game);
}
pub unsafe fn Ridley_Func_34(game: &mut Game) {
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    Ridley_Func_104_0(
        0 as uint16,
        0 as uint16,
        (*E).ridley_var_17,
        (*E).ridley_var_18,
        game,
    );
    (*E).ridley_var_F = (*E).ridley_var_F.wrapping_sub(1);
    if ((*E).ridley_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        (*E).ridley_var_A = ((0xa6bbf1i32) as u16);
        (*E).ridley_var_F = 32 as uint16;
    }
}
pub unsafe fn Ridley_Func_35(game: &mut Game) {
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    (*E).ridley_var_F = (*E).ridley_var_F.wrapping_sub(1);
    if ((*E).ridley_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(15) =
            8 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(9) =
            240 as uint16;
        Ridley_Func_38(game);
        (*E).ridley_var_A = ((0xa6bc2ei32) as u16);
        (*E).ridley_var_F = 64 as uint16;
    } else {
        Ridley_Func_104_0(
            0 as uint16,
            0 as uint16,
            (*E).ridley_var_17,
            256 as uint16,
            game,
        );
    };
}
pub unsafe fn Ridley_Func_36(game: &mut Game) {
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    (*E).ridley_var_F = (*E).ridley_var_F.wrapping_sub(1);
    if ((*E).ridley_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(15) =
            16 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(9) =
            240 as uint16;
        (*E).ridley_var_A = ((0xa6b321i32) as u16);
    } else {
        Ridley_Func_104_0(
            0 as uint16,
            0 as uint16,
            g_word_A6BC62[(*E).ridley_var_10 as usize],
            224 as uint16,
            game,
        );
    };
}
pub unsafe fn Ridley_Func_37(game: &mut Game) {
    Ridley_Func_31(game);
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
    (*E).ridley_var_1B = 1 as uint16;
    CallSomeSamusCode(0 as uint16, game);
    Ridley_Func_119(1 as uint8, game);
}
pub unsafe fn Ridley_Func_38(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(2) = 1 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(0) = 1 as uint16;
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    if ((*E).ridley_var_01 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) == 0i32 {
        (*E).ridley_var_1E = (if Ridley_Func_40_Sign(game) != 0u8 {
            6 as ::core::ffi::c_int
        } else {
            10 as ::core::ffi::c_int
        }) as uint16;
    }
    (*E).ridley_var_1B = 0 as uint16;
    CallSomeSamusCode(1 as uint16, game);
    Ridley_Func_119(0 as uint8, game);
}
pub unsafe fn Ridley_Func_39(game: &mut Game) {
    let mut ridley_var_01: int16 = 0;
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    ridley_var_01 = (*E).ridley_var_01 as int16;
    if ridley_var_01 >= 0i16 {
        if ridley_var_01 != 0 {
            if Ridley_Func_121(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
                (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
                return;
            }
            (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
                & !(kEnemyProps_Tangible as ::core::ffi::c_int))
                as uint16;
        }
        let ridley_var_1E: uint16 = (*E).ridley_var_1E;
        if ridley_var_1E != 0 {
            let v3: uint16 = (ridley_var_1E).wrapping_sub(1u16);
            (*E).ridley_var_1E = v3;
            if v3 == 0
                && ((*E).ridley_var_01 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) == 0i32
            {
                (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
                    & !(kEnemyProps_Tangible as ::core::ffi::c_int))
                    as uint16;
            }
        }
    }
}
static byte_A6BD04: [uint8; 28] = [
    0x80, 0x80, 0x80, 0, 0xff, 0x80, 0x80, 0xff, 0xff, 0xff, 0x80, 0, 0, 0x80, 0x80, 0x80, 0x80,
    0xff, 0xff, 0xff, 0x80, 0x80, 0, 0x80, 0x80, 0, 0, 0x80,
];
unsafe fn Ridley_Func_40_Carry(game: &mut Game) -> uint8 {
    ((byte_A6BD04[*(&raw mut game.g_ram as *mut uint8).offset(0xa1f) as usize]
        as ::core::ffi::c_int
        & 0x80 as ::core::ffi::c_int)
        != 0i32) as uint8
}
unsafe fn Ridley_Func_40_Sign(game: &mut Game) -> uint8 {
    ((byte_A6BD04[*(&raw mut game.g_ram as *mut uint8).offset(0xa1f) as usize]
        as ::core::ffi::c_int
        & 0x40 as ::core::ffi::c_int)
        != 0i32) as uint8
}
pub unsafe fn Ridley_Func_41(game: &mut Game) -> uint8 {
    ((*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16) as ::core::ffi::c_int
        - 352 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int)
        == 0i32) as uint8
}
pub unsafe fn Ridley_Func_42(game: &mut Game) {
    let mut ridley_var_01: int16 = 0;
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    ridley_var_01 = (*E).ridley_var_01 as int16;
    if ridley_var_01 > 0i16
        && ridley_var_01 != 2i16
        && *((&raw mut game.g_ram as *mut uint8).offset(0xcee) as *mut uint16) != 0u16
        && (*E).ridley_var_1B == 0
    {
        Ridley_Func_24(game);
        (*E).ridley_var_A = ((0xa6bab7i32) as u16);
    }
}
pub unsafe fn Ridley_Func_43(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xcee) as *mut uint16) != 0 {
        (*Get_Ridley(0 as ::core::ffi::c_int, game)).ridley_var_01 = 2 as uint16;
        let r18: uint16 = (if *((&raw mut game.g_ram as *mut uint8).offset(0xce2) as *mut uint16)
            as ::core::ffi::c_int
            - 128 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            192 as ::core::ffi::c_int
        } else {
            80 as ::core::ffi::c_int
        }) as uint16;
        let r20: uint16 = (if *((&raw mut game.g_ram as *mut uint8).offset(0xce4) as *mut uint16)
            as ::core::ffi::c_int
            - 256 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            384 as ::core::ffi::c_int
        } else {
            192 as ::core::ffi::c_int
        }) as uint16;
        Ridley_Func_104_0(0 as uint16, Ridley_Func_7(game), r18, r20, game);
    } else {
        let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
        (*E).ridley_var_01 = 1 as uint16;
        (*E).ridley_var_A = (if (*E).ridley_var_1B != 0u16 {
            0xa6bb8f as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
        } else {
            0xa6b321 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
        }) as uint16;
    };
}
pub unsafe fn Ridley_Func_44(game: &mut Game) {
    Ridley_Func_106(0 as uint16, 1 as uint16, 192 as uint16, 128 as uint16, game);
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    if (*E).base.x_pos as ::core::ffi::c_int - 192 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        (*E).ridley_var_A = ((0xa6bdbci32) as u16);
    }
}
pub unsafe fn Ridley_Func_45(game: &mut Game) {
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    (*E).ridley_var_20 = -(192 as ::core::ffi::c_int) as uint16;
    Ridley_Func_106(
        0 as uint16,
        1 as uint16,
        192 as uint16,
        -(128 as ::core::ffi::c_int) as uint16,
        game,
    );
    if (*E).base.y_pos as ::core::ffi::c_int - 32 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        (*E).ridley_var_40 = ((0xa6becai32) as u16);
        (*E).ridley_var_A = ((0xa6bdf2i32) as u16);
        (*E).ridley_var_F = 21 as uint16;
    }
}
pub unsafe fn Ridley_Func_46(game: &mut Game) {
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    (*E).ridley_var_F = (*E).ridley_var_F.wrapping_sub(1);
    if ((*E).ridley_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        Ridley_Func_99(addr_kRidley_Ilist_E658 as uint16, game);
        (*E).ridley_var_A = ((0xa6be03i32) as u16);
        Ridley_Func_47(game);
    }
}
pub unsafe fn Ridley_Func_47(game: &mut Game) {
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    Ridley_Func_104_0(
        0 as uint16,
        12 as uint16,
        ((*E).ridley_var_42).wrapping_sub(10u16),
        ((*E).ridley_var_44).wrapping_sub(56u16),
        game,
    );
    let rect: Rect16U = Rect16U {
        x: (((*E).base.x_pos).wrapping_add(14u16)),
        y: (((*E).base.y_pos).wrapping_add(66u16)),
        w: 4 as uint16,
        h: 4 as uint16,
    };
    if Ridley_Func_48(rect, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        (*E).ridley_var_1B = 1 as uint16;
        Ridley_Func_119(1 as uint8, game);
        (*E).ridley_var_C = -(512 as ::core::ffi::c_int) as uint16;
        (*E).ridley_var_40 = ((0xa6beb3i32) as u16);
        (*E).ridley_var_A = ((0xa6a971i32) as u16);
    }
}
pub unsafe fn Ridley_Func_48(rect: Rect16U, game: &mut Game) -> uint8 {
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    let v1: uint16 = abs16(((*E).ridley_var_42).wrapping_sub(rect.x));
    let mut v2: bool = v1 < 4u16;
    let v3: uint16 = (v1).wrapping_sub(4u16);
    let result: uint8 = 0 as uint8;
    if v2 || v3 < rect.w {
        let v4: uint16 = abs16(((*E).ridley_var_44).wrapping_sub(rect.y));
        v2 = v4 < 4u16;
        let v5: uint16 = (v4).wrapping_sub(4u16);
        if v2 || v5 < rect.h {
            return 1 as uint8;
        }
    }
    result
}
pub unsafe fn CallRidleyFunc_var40(ea: uint32, game: &mut Game) {
    match ea {
        10927772 => {
            Ridley_Func_50(game);
        }
        10927795 => {
            Ridley_Func_51(game);
        }
        10927818 => {
            Ridley_Func_52(game);
        }
        10927836 => {
            Ridley_Func_53(game);
        }
        10927897 => (),
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn Ridley_Func_49(game: &mut Game) {
    CallRidleyFunc_var40(
        (((*Get_Ridley(0 as ::core::ffi::c_int, game)).ridley_var_40) as u32) | (0xa60000u32),
        game,
    );
}
pub unsafe fn Ridley_Func_50(game: &mut Game) {
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    (*E).ridley_var_42 = ((*E).base.x_pos).wrapping_sub(16u16);
    (*E).ridley_var_44 = ((*E).base.y_pos).wrapping_add(22u16);
}
pub unsafe fn Ridley_Func_51(game: &mut Game) {
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    (*E).ridley_var_42 = ((*E).base.x_pos).wrapping_add(14u16);
    (*E).ridley_var_44 = ((*E).base.y_pos).wrapping_add(66u16);
}
pub unsafe fn Ridley_Func_52(game: &mut Game) {
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    (*E).ridley_var_45 = 0 as uint16;
    (*E).ridley_var_46 = 0 as uint16;
    (*E).ridley_var_40 = ((0xa6bedci32) as u16);
    Ridley_Func_53(game);
}
pub unsafe fn Ridley_Func_53(game: &mut Game) {
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    let v1: uint16 = ((*E).ridley_var_46).wrapping_add(8u16);
    (*E).ridley_var_46 = v1;
    let t: ::core::ffi::c_int = *(&raw mut (*E).ridley_var_43 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>())
            .wrapping_div(::core::mem::size_of::<uint8>())
            .wrapping_sub(1_usize) as isize,
    ) as ::core::ffi::c_int
        + (v1 as ::core::ffi::c_int & 0xff as ::core::ffi::c_int);
    *(&raw mut (*E).ridley_var_43 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>())
            .wrapping_div(::core::mem::size_of::<uint8>())
            .wrapping_sub(1_usize) as isize,
    ) = t as uint8;
    (*E).ridley_var_44 = ((*E).ridley_var_44 as ::core::ffi::c_int
        + ((v1 as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as int8 as ::core::ffi::c_int
            + (t >> 8 as ::core::ffi::c_int))) as uint16;
    if (*E).ridley_var_44 as ::core::ffi::c_int - 192 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        (*E).ridley_var_44 = 192 as uint16;
        (*E).ridley_var_40 = addr_locret_A6BF19 as uint16;
    }
}
pub unsafe fn DrawBabyMetroid_0(game: &mut Game) {
    let v0: ::core::ffi::c_int = BabyMetroid_DBCB_DoubleRetEx(
        (((&raw mut game.g_ram as *mut uint8).offset(0x7800) as *mut ExtraEnemyRam7800
            as *mut uint8)
            .offset_from(&raw mut game.g_ram as *mut uint8) as ::core::ffi::c_long
            + 6 as ::core::ffi::c_long) as uint16,
        game,
    );
    if v0 < 0i32 {
        return;
    }
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    sub_A6DC13(
        v0 as uint16,
        (*E).ridley_var_42,
        (*E).ridley_var_44,
        0 as uint16,
        game,
    );
}
pub unsafe fn BabyMetroid_Instr_2(k: uint16, game: &mut Game) -> uint16 {
    if (*Get_Ridley(0 as ::core::ffi::c_int, game)).ridley_var_46 == 0
        && (*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
            as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int)
            != 0i32
    {
        return BabyMetroid_Goto(k, game);
    }
    QueueSfx3_Max6(0x24 as uint16, game);
    (k).wrapping_add(2u16)
}
pub unsafe fn BabyMetroid_Instr_3(k: uint16, game: &mut Game) -> uint16 {
    let v1: uint16 = *(RomPtr_A6(k, game) as *mut uint16);
    WriteColorsToPalette(0x162 as uint16, 0xa6 as uint8, v1, 0xf as uint16, game);
    (k).wrapping_add(2u16)
}
pub unsafe fn BabyMetroid_Instr_1(k: uint16, game: &mut Game) -> uint16 {
    if (*Get_Ridley(0 as ::core::ffi::c_int, game)).ridley_var_46 != 0 {
        BabyMetroid_Goto(k, game)
    } else {
        (k).wrapping_add(2u16)
    }
}
pub unsafe fn BabyMetroid_Goto(k: uint16, game: &mut Game) -> uint16 {
    *(RomPtr_A6(k, game) as *mut uint16)
}
pub unsafe fn Ridley_Func_54(game: &mut Game) {
    let ridley_var_F: uint16 = (*Get_Ridley(0 as ::core::ffi::c_int, game)).ridley_var_F;
    if ridley_var_F != 0 {
        match ridley_var_F as ::core::ffi::c_int {
            2 => {
                Ridley_C08E(game);
            }
            4 => {
                Ridley_C09F(game);
            }
            6 => {
                Ridley_Func_56(game);
            }
            8 => {
                Ridley_Func_57(game);
            }
            10 => {
                Ridley_C104(game);
            }
            12 => {
                Ridley_Func_58(game);
            }
            _ => {
                Unreachable(game);
                loop {}
            }
        }
    } else {
        Ridley_Func_55(game);
    };
}
pub unsafe fn Ridley_Func_55(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(97) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(1);
    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(99) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(3);
    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(81) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(17);
    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(83) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(19);
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    (*E).ridley_var_E = -(15157 as ::core::ffi::c_int) as uint16;
    (*E).ridley_var_F = (*E).ridley_var_F.wrapping_add(1);
    (*E).ridley_var_F = (*E).ridley_var_F.wrapping_add(1);
    Ridley_C08E(game);
}
pub unsafe fn Ridley_C08E(game: &mut Game) {
    if ProcessEscapeTimerTileTransfers(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
        (*E).ridley_var_E = -(15106 as ::core::ffi::c_int) as uint16;
        (*E).ridley_var_F = (*E).ridley_var_F.wrapping_add(1);
        (*E).ridley_var_F = (*E).ridley_var_F.wrapping_add(1);
        Ridley_C09F(game);
    }
}
pub unsafe fn Ridley_C09F(game: &mut Game) {
    if ProcessEscapeTimerTileTransfers(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
        (*E).ridley_var_F = (*E).ridley_var_F.wrapping_add(1);
        (*E).ridley_var_F = (*E).ridley_var_F.wrapping_add(1);
        Ridley_Func_59(game);
        (*E).ridley_var_E = 128 as uint16;
        QueueMusic_Delayed8(7 as uint16, game);
    }
}
pub unsafe fn Ridley_Func_56(game: &mut Game) {
    Ridley_Func_61(game);
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    let fresh16 = (*E).ridley_var_E;
    (*E).ridley_var_E = (*E).ridley_var_E.wrapping_sub(1);
    if fresh16 == 1u16 {
        (*E).ridley_var_3B = -(15280 as ::core::ffi::c_int) as uint16;
        (*E).ridley_var_3C = 0 as uint16;
        (*E).ridley_var_3D = 0 as uint16;
        (*E).ridley_var_3E = 0 as uint16;
        (*E).ridley_var_3F = 0 as uint16;
        (*E).ridley_var_E = 32 as uint16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x9e2) as *mut uint16) == 0 {
            (*E).ridley_var_F = (*E).ridley_var_F.wrapping_add(1);
            (*E).ridley_var_F = (*E).ridley_var_F.wrapping_add(1);
        }
        (*E).ridley_var_F = (*E).ridley_var_F.wrapping_add(1);
        (*E).ridley_var_F = (*E).ridley_var_F.wrapping_add(1);
    }
}
pub unsafe fn Ridley_Func_57(game: &mut Game) {
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    let fresh15 = (*E).ridley_var_E;
    (*E).ridley_var_E = (*E).ridley_var_E.wrapping_sub(1);
    if fresh15 == 1u16 {
        (*E).ridley_var_F = (*E).ridley_var_F.wrapping_add(1);
        (*E).ridley_var_F = (*E).ridley_var_F.wrapping_add(1);
        Ridley_Func_62(game);
    }
    Ridley_C104(game);
}
pub unsafe fn Ridley_C104(game: &mut Game) {
    Ridley_Func_61(game);
    if HandleTypewriterText_Ext(0x3582 as uint16, game) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
        (*E).ridley_var_F = (*E).ridley_var_F.wrapping_add(1);
        (*E).ridley_var_F = (*E).ridley_var_F.wrapping_add(1);
    }
}
pub unsafe fn Ridley_Func_58(game: &mut Game) {
    Ridley_Func_61(game);
    CeresRidley_Func_2(game);
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    (*E).ridley_var_F = 0 as uint16;
    (*E).ridley_var_A = ((0xa6aa50i32) as u16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x943) as *mut uint16) = 1 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x93f) as *mut uint16) = 2 as uint16;
    SetBossBitForCurArea(1 as uint16, game);
}
pub unsafe fn Ridley_Func_59(game: &mut Game) {
    let mut v2: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    let v0: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16);
    let v1: *const uint8 = RomPtr_A6(addr_byte_A6C15D as uint16_t, game);
    v2 = gVramWriteEntry(
        *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*v2).size = *(v1 as *mut uint16);
    *((&raw mut (*v2).src.addr as *mut uint8).offset(1) as *mut VoidP) =
        *(v1.offset(3) as *mut uint16);
    (*v2).src.addr = *(v1.offset(2) as *mut uint16);
    (*v2).vram_dst = *(v1.offset(5) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) = (v0).wrapping_add(7u16);
}
pub unsafe fn Ridley_Func_60(game: &mut Game) {
    let mut v1: int16 = 0;
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    v1 = (((*E).ridley_var_39) as i16).wrapping_add(1i16);
    if (*E).ridley_var_39 as ::core::ffi::c_int - 31 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        v1 = -(31 as ::core::ffi::c_int) as int16;
    }
    (*E).ridley_var_39 = v1 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(11) =
        (abs16((*E).ridley_var_39)) & (0x1fu16);
}
pub unsafe fn Ridley_Func_61(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x797) as *mut uint16) as ::core::ffi::c_int
        | *((&raw mut game.g_ram as *mut uint8).offset(0xc400) as *mut uint16) as ::core::ffi::c_int
        == 0
        && (*((&raw mut game.g_ram as *mut uint8).offset(0x5b6) as *mut uint16)
            as ::core::ffi::c_int
            & 3 as ::core::ffi::c_int)
            == 0i32
    {
        let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
        let mut v1: uint16 = ((*E).ridley_var_38).wrapping_add(1u16);
        if v1 >= 0x10u16 {
            v1 = 0 as uint16;
        }
        (*E).ridley_var_38 = v1;
        let v2: ::core::ffi::c_int = (*E).ridley_var_38 as ::core::ffi::c_int
            + 2 as ::core::ffi::c_int * v1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(97) =
            *(game.g_rom.offset(crate::rtl::lorom_offset(0xa6c1df)) as *const uint8 as *mut uint16)
                .offset(v2 as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(98) =
            *(game.g_rom.offset(crate::rtl::lorom_offset(0xa6c1df)) as *const uint8 as *mut uint16)
                .offset((v2 + 1 as ::core::ffi::c_int) as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(99) =
            *(game.g_rom.offset(crate::rtl::lorom_offset(0xa6c1df)) as *const uint8 as *mut uint16)
                .offset((v2 + 2 as ::core::ffi::c_int) as isize);
    }
}
pub unsafe fn SetupZebesEscapeTypewriter(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(157) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(125);
    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(158) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(126);
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    (*E).mbn_var_3B = addr_loc_A9C49C as uint16;
    (*E).mbn_var_3C = 0 as uint16;
    (*E).mbn_var_3D = 0 as uint16;
    (*E).mbn_var_3E = 0 as uint16;
    (*E).mbn_var_3F = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb5) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb7) as *mut uint16) = 0 as uint16;
}
pub unsafe fn ProcessEscapeTimerTileTransfers(game: &mut Game) -> uint8 {
    let mut v4: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    let v0: *mut EnemyData = gEnemyData(0 as ::core::ffi::c_int, game);
    let ai_var_E: uint16 = (*v0).ai_var_E;
    let v2: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16);
    let v3: *const uint8 = RomPtr_A6(ai_var_E, game);
    let result: uint8 = 1 as uint8;
    if *(v3 as *mut uint16) != 0 {
        v4 = gVramWriteEntry(
            *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        );
        (*v4).size = *(v3 as *mut uint16);
        *((&raw mut (*v4).src.addr as *mut uint8).offset(1) as *mut VoidP) =
            *(v3.offset(3) as *mut uint16);
        (*v4).src.addr = *(v3.offset(2) as *mut uint16);
        (*v4).vram_dst = *(v3.offset(5) as *mut uint16);
        *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) =
            (v2).wrapping_add(7u16);
        (*v0).ai_var_E = (ai_var_E).wrapping_add(7u16);
        if *(RomPtr_A6((*v0).ai_var_E, game) as *mut uint16) != 0 {
            return 0 as uint8;
        }
    }
    result
}
pub unsafe fn HandleTypewriterText_Ext(a: uint16, game: &mut Game) -> uint8 {
    let r18: uint16 = a;
    let mut v4: int16 = 0;
    let mut v12: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    let mbn_var_3D: uint16 = (*E).mbn_var_3D;
    if mbn_var_3D != 0 {
        (*E).mbn_var_3D = (mbn_var_3D).wrapping_sub(1u16);
        0 as uint8
    } else {
        (*E).mbn_var_3D = (*E).mbn_var_3E;
        let mut i: ::core::ffi::c_int = 0;
        let mut v7: ::core::ffi::c_int = 0;
        i = (*E).mbn_var_3B as ::core::ffi::c_int;
        loop {
            loop {
                v4 = *(RomPtr_A6(i as uint16_t, game) as *mut uint16) as int16;
                if v4 == 0 {
                    return 1 as uint8;
                }
                if v4 != 1i16 {
                    break;
                }
                let v5: uint16 = (i + 2 as ::core::ffi::c_int) as uint16;
                let v6: uint16 = *(RomPtr_A6(v5, game) as *mut uint16);
                (*E).mbn_var_3E = v6;
                i = v5 as ::core::ffi::c_int + 2 as ::core::ffi::c_int;
            }
            if v4 != 13i16 {
                break;
            }
            v7 = i + 2 as ::core::ffi::c_int;
            let v8: uint16 = *(RomPtr_A6(v7 as uint16_t, game) as *mut uint16);
            (*E).mbn_var_3C = v8;
            i = v7 + 2 as ::core::ffi::c_int;
        }
        v4 = v4 as uint8 as int16;
        if (v4 as uint8) == 32u8 {
            (*E).mbn_var_3C = (*E).mbn_var_3C.wrapping_add(1);
            (*E).mbn_var_3B = (i + 1 as ::core::ffi::c_int) as uint16;
            0 as uint8
        } else {
            if (v4 as uint8) == 33u8 {
                v4 = 91 as int16;
            }
            (*E).mbn_var_3B = (i + 1 as ::core::ffi::c_int) as uint16;
            let v11: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16);
            v12 = gVramWriteEntry(
                *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16)
                    as ::core::ffi::c_int,
                game,
            );
            (*v12).size = 2 as uint16;
            *((&raw mut (*v12).src.addr as *mut uint8).offset(1) as *mut VoidP) = 32256 as VoidP;
            (*E).mbn_var_3A = (r18 as ::core::ffi::c_int + v4 as ::core::ffi::c_int
                - 65 as ::core::ffi::c_int) as uint16;
            (*v12).src.addr = (((&raw mut game.g_ram as *mut uint8).offset(0x8000)
                as *mut ExtraEnemyRam8000 as *mut uint8)
                .offset_from(&raw mut game.g_ram as *mut uint8)
                as ::core::ffi::c_long
                + 52 as ::core::ffi::c_long) as VoidP;
            let mbn_var_3C: uint16 = (*E).mbn_var_3C;
            (*v12).vram_dst = mbn_var_3C;
            (*E).mbn_var_3C = (mbn_var_3C).wrapping_add(1u16);
            *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) =
                (v11).wrapping_add(7u16);
            let v14: uint16 = ((*E).mbn_var_3F).wrapping_add(1u16);
            (*E).mbn_var_3F = v14;
            if v14 as ::core::ffi::c_int - 2 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                == 0
            {
                (*E).mbn_var_3F = 0 as uint16;
                if *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) == 6u16 {
                    QueueSfx2_Max3(0x45 as uint16, game);
                } else {
                    QueueSfx3_Max3(0xd as uint16, game);
                }
            }
            0 as uint8
        }
    }
}
pub unsafe fn Ridley_Func_62(game: &mut Game) {
    let mut i: ::core::ffi::c_int = 0;
    let mut v3: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    let mut v0: uint16 = addr_stru_A6C3B8 as uint16;
    i = *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) as ::core::ffi::c_int;
    loop {
        let v2: *const uint8 = RomPtr_A6(v0, game);
        if *(v2 as *mut uint16) == 0 {
            break;
        }
        v3 = gVramWriteEntry(i, game);
        (*v3).size = *(v2 as *mut uint16);
        *((&raw mut (*v3).src.addr as *mut uint8).offset(1) as *mut VoidP) =
            *(v2.offset(3) as *mut uint16);
        (*v3).src.addr = *(v2.offset(2) as *mut uint16);
        (*v3).vram_dst = *(v2.offset(5) as *mut uint16);
        v0 = (v0).wrapping_add(7u16);
        i += 7 as ::core::ffi::c_int;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) = i as uint16;
}
pub unsafe fn Ridley_Func_63(game: &mut Game) {
    if Ridley_Func_68(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int == 0 {
        Ridley_C53E(game);
    }
}
pub unsafe fn Ridley_C53E(game: &mut Game) {
    Ridley_Func_99(addr_kRidley_Ilist_E6C8 as uint16, game);
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    (*E).ridley_var_A = ((0xa6c551i32) as u16);
    (*E).ridley_var_F = 32 as uint16;
}
pub unsafe fn Ridley_C551(game: &mut Game) {
    Ridley_Func_68(game);
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    (*E).ridley_var_F = (*E).ridley_var_F.wrapping_sub(1);
    if ((*E).ridley_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        (*E).ridley_var_B = 0 as uint16;
        (*E).ridley_var_C = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x197a) as *mut uint16) = 528 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x197c) as *mut uint16) = 64 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1980) as *mut uint16) = 1 as uint16;
        (*E).ridley_var_27 = 0 as uint16;
        (*E).ridley_var_28 = 0 as uint16;
        (*E).ridley_var_A = ((0xa6c588i32) as u16);
        (*E).ridley_var_F = 160 as uint16;
        Ridley_Func_64(game);
    }
}
pub unsafe fn Ridley_Func_64(game: &mut Game) {
    Ridley_Func_69(game);
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    (*E).ridley_var_F = (*E).ridley_var_F.wrapping_sub(1);
    if ((*E).ridley_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        if (*E).ridley_var_1B != 0 {
            Ridley_Func_38(game);
        }
        (*E).ridley_var_A = ((0xa6c5a8i32) as u16);
        (*E).ridley_var_B = 0 as uint16;
        (*E).ridley_var_C = 0 as uint16;
        RidleysExplosion_Func_2(game);
    }
}
pub unsafe fn Ridley_Func_65(game: &mut Game) {
    Ridley_Func_69(game);
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    (*E).ridley_var_02 = 0 as uint16;
    (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Invisible) as u16);
    (*E).ridley_var_A = ((0xa6c5c8i32) as u16);
    (*E).ridley_var_F = 32 as uint16;
}
pub unsafe fn Ridley_Func_66(game: &mut Game) {
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    (*E).ridley_var_F = (*E).ridley_var_F.wrapping_sub(1);
    if ((*E).ridley_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        (*E).ridley_var_A = ((0xa6c5dai32) as u16);
        (*E).ridley_var_F = 256 as uint16;
    }
}
pub unsafe fn Ridley_Func_67(game: &mut Game) {
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    (*E).ridley_var_F = (*E).ridley_var_F.wrapping_sub(1);
    if ((*E).ridley_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        let k: uint16 = 0 as uint16;
        printf(b"Warning: X undefined\n\0" as *const u8 as *const ::core::ffi::c_char);
        SetBossBitForCurArea(1 as uint16, game);
        Enemy_ItemDrop_Ridley(k, game);
        QueueMusic_Delayed8(3 as uint16, game);
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Deleted) as u16);
        (*E).ridley_var_A = addr_locret_A6C600 as uint16;
    }
}
pub unsafe fn Ridley_Func_68(game: &mut Game) -> uint8 {
    let rect: Rect16U = Rect16U {
        x: 128 as uint16,
        y: 328 as uint16,
        w: 4 as uint16,
        h: 4 as uint16,
    };
    Ridley_Func_104(
        0 as uint16,
        0 as uint16,
        0x10 as uint16,
        rect.x,
        rect.y,
        game,
    );
    Shitroid_Func_2(0 as uint16, rect, game)
}
pub unsafe fn Ridley_Func_69(game: &mut Game) {
    let mut v1: int16 = 0;
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    v1 = (((*E).ridley_var_27) as i16).wrapping_sub(1i16);
    if v1 < 0i16 {
        (*E).ridley_var_27 = 4 as uint16;
        let mut v2: uint16 = ((*E).ridley_var_28).wrapping_add(1u16);
        if (*E).ridley_var_28 as ::core::ffi::c_int - 9 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            v2 = 0 as uint16;
        }
        (*E).ridley_var_28 = v2;
        let v3: ::core::ffi::c_int =
            ((4u16).wrapping_mul(v2)) as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        game.eproj_spawn_pt = Point16U {
            x: (((*E).base.x_pos).wrapping_add((g_word_A6C66E[v3 as usize]) as u16)),
            y: (((*E).base.y_pos)
                .wrapping_add((g_word_A6C66E[(v3 + 1 as ::core::ffi::c_int) as usize]) as u16)),
        };
        SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as uint16, 3 as uint16, game);
        QueueSfx2_Max3(0x24 as uint16, game);
    } else {
        (*E).ridley_var_27 = v1 as uint16;
    };
}
pub unsafe fn RidleysExplosion_Init(game: &mut Game) {
    let E: *mut Enemy_Ridley = Get_Ridley(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
    (*E).base.vram_tiles_index = 0 as uint16;
    (*E).base.palette_index = 3584 as uint16;
    let ridley_parameter_1: uint16 = (*E).ridley_parameter_1;
    (*E).ridley_var_F = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa6c6ce)) as *const uint8
        as *mut uint16)
        .offset((ridley_parameter_1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    let Random: uint16 = NextRandom(game);
    (*E).ridley_var_B = (if Random as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        -(Random as ::core::ffi::c_int & 0x130 as ::core::ffi::c_int)
    } else {
        Random as ::core::ffi::c_int & 0x130 as ::core::ffi::c_int
    }) as uint16;
    (*E).ridley_var_C = 0 as uint16;
    if ridley_parameter_1 != 0 {
        match ridley_parameter_1 as ::core::ffi::c_int {
            2 => {
                (*E).base.x_pos =
                    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(32);
                (*E).base.y_pos =
                    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(33);
                (*E).base.current_instruction = addr_kRidleysExplosion_Ilist_CA47 as uint16;
            }
            4 => {
                (*E).base.x_pos =
                    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(42);
                (*E).base.y_pos =
                    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(43);
                (*E).base.current_instruction = addr_kRidleysExplosion_Ilist_CA4D as uint16;
            }
            6 => {
                (*E).base.x_pos =
                    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(52);
                (*E).base.y_pos =
                    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(53);
                (*E).base.current_instruction = addr_kRidleysExplosion_Ilist_CA4D as uint16;
            }
            8 => {
                (*E).base.x_pos =
                    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(62);
                (*E).base.y_pos =
                    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(63);
                (*E).base.current_instruction = addr_kRidleysExplosion_Ilist_CA53 as uint16;
            }
            10 => {
                (*E).base.x_pos =
                    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(72);
                (*E).base.y_pos =
                    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(73);
                (*E).base.current_instruction = addr_kRidleysExplosion_Ilist_CA53 as uint16;
            }
            12 => {
                (*E).base.x_pos =
                    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(82);
                (*E).base.y_pos =
                    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(83);
                (*E).base.current_instruction = *(game
                    .g_rom
                    .offset(crate::rtl::lorom_offset(0xa6c7ba))
                    as *const uint8
                    as *mut uint16)
                    .offset(
                        ((*(((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                            .offset(71) as *mut uint16 as *mut uint8)
                            .offset(0) as ::core::ffi::c_int
                            + *(((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                                .offset(81) as *mut uint16
                                as *mut uint8)
                                .offset(0) as ::core::ffi::c_int
                            + 8 as ::core::ffi::c_int
                            & 0xf0 as ::core::ffi::c_int) as uint8
                            as ::core::ffi::c_int
                            >> 4 as ::core::ffi::c_int) as isize,
                    );
            }
            14 => {
                let mut v10: uint16 = 0 as uint16;
                let E0: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
                if (*E0).ridley_var_10 != 0 {
                    v10 = 2 as uint16;
                }
                let v13: ::core::ffi::c_int = v10 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
                let v14: uint16 = ((*E0).base.x_pos).wrapping_add(
                    *(game.g_rom.offset(crate::rtl::lorom_offset(0xa6c804)) as *const uint8
                        as *mut uint16)
                        .offset(v13 as isize),
                );
                (*E).base.x_pos = v14;
                (*E).base.y_pos = (*E0).base.y_pos;
                (*E).base.current_instruction =
                    *(game.g_rom.offset(crate::rtl::lorom_offset(0xa6c808)) as *const uint8
                        as *mut uint16)
                        .offset(v13 as isize);
            }
            16 => {
                let mut v16: uint16 = 0 as uint16;
                let E0_0: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
                if (*E0_0).ridley_var_10 != 0 {
                    v16 = 2 as uint16;
                }
                let v19: ::core::ffi::c_int = v16 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
                let v20: uint16 = ((*E0_0).base.x_pos).wrapping_add(
                    *(game.g_rom.offset(crate::rtl::lorom_offset(0xa6c836)) as *const uint8
                        as *mut uint16)
                        .offset(v19 as isize),
                );
                (*E).base.x_pos = v20;
                (*E).base.y_pos = ((*E0_0).base.y_pos).wrapping_add(22u16);
                (*E).base.current_instruction =
                    *(game.g_rom.offset(crate::rtl::lorom_offset(0xa6c83a)) as *const uint8
                        as *mut uint16)
                        .offset(v19 as isize);
            }
            18 => {
                let mut v22: uint16 = 0 as uint16;
                let E0_1: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
                if (*E0_1).ridley_var_10 != 0 {
                    v22 = 2 as uint16;
                }
                let v25: ::core::ffi::c_int = v22 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
                let v26: uint16 = ((*E0_1).base.x_pos).wrapping_add(
                    *(game.g_rom.offset(crate::rtl::lorom_offset(0xa6c868)) as *const uint8
                        as *mut uint16)
                        .offset(v25 as isize),
                );
                (*E).base.x_pos = v26;
                (*E).base.y_pos = ((*E0_1).base.y_pos).wrapping_sub(24u16);
                (*E).base.current_instruction =
                    *(game.g_rom.offset(crate::rtl::lorom_offset(0xa6c86c)) as *const uint8
                        as *mut uint16)
                        .offset(v25 as isize);
            }
            20 => {
                let mut v28: uint16 = 0 as uint16;
                let E0_2: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
                if (*E0_2).ridley_var_10 != 0 {
                    v28 = 2 as uint16;
                }
                let v31: ::core::ffi::c_int = v28 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
                let v32: uint16 = ((*E0_2).base.x_pos).wrapping_add(
                    *(game.g_rom.offset(crate::rtl::lorom_offset(0xa6c89a)) as *const uint8
                        as *mut uint16)
                        .offset(v31 as isize),
                );
                (*E).base.x_pos = v32;
                (*E).base.y_pos = (*E0_2).base.y_pos;
                (*E).base.current_instruction =
                    *(game.g_rom.offset(crate::rtl::lorom_offset(0xa6c89e)) as *const uint8
                        as *mut uint16)
                        .offset(v31 as isize);
            }
            22 => {
                let mut v34: uint16 = 0 as uint16;
                let E0_3: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
                if (*E0_3).ridley_var_10 != 0 {
                    v34 = 2 as uint16;
                }
                let v37: ::core::ffi::c_int = v34 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
                let v38: uint16 = ((*E0_3).base.x_pos).wrapping_add(
                    *(game.g_rom.offset(crate::rtl::lorom_offset(0xa6c8cc)) as *const uint8
                        as *mut uint16)
                        .offset(v37 as isize),
                );
                (*E).base.x_pos = v38;
                (*E).base.y_pos = ((*E0_3).base.y_pos).wrapping_add(7u16);
                (*E).base.current_instruction =
                    *(game.g_rom.offset(crate::rtl::lorom_offset(0xa6c8d0)) as *const uint8
                        as *mut uint16)
                        .offset(v37 as isize);
            }
            _ => {
                Unreachable(game);
                loop {}
            }
        }
    } else {
        (*E).base.x_pos =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(22);
        (*E).base.y_pos =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(23);
        (*E).base.current_instruction = addr_kRidleysExplosion_Ilist_CA47 as uint16;
    };
}
pub unsafe fn RidleysExplosion_Main(game: &mut Game) {
    let mut v2: int16 = 0;
    RidleysExplosion_Func_1(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
    let E: *mut Enemy_RidleysExplosion = Get_RidleysExplosion(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    v2 = ((abs16((*E).ren_var_B)) as i16).wrapping_sub(4i16);
    if v2 < 0i16 {
        v2 = 0 as int16;
    }
    (*E).ren_var_B = (if (*E).ren_var_B as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        -(v2 as ::core::ffi::c_int)
    } else {
        v2 as ::core::ffi::c_int
    }) as uint16;
    (*E).ren_var_C = ((*E).ren_var_C).wrapping_add(4u16);
    MoveEnemyWithVelocity(game);
    (*E).ren_var_F = (*E).ren_var_F.wrapping_sub(1);
    if ((*E).ren_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        EnemyDeathAnimation(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            0 as uint16,
            game,
        );
    }
}
pub unsafe fn RidleysExplosion_Func_1(k: uint16, game: &mut Game) {
    let E: *mut Enemy_RidleysExplosion = Get_RidleysExplosion(k as ::core::ffi::c_int, game);
    let v2: uint16 = ((*E).base.frame_counter) & (1u16);
    let mut v3: uint16 = 0;
    (*E).base.frame_counter = v2;
    if v2 != 0 {
        v3 = ((*E).base.properties) | ((kEnemyProps_Invisible) as u16);
    } else {
        v3 = ((*E).base.properties as ::core::ffi::c_int
            & !(kEnemyProps_Invisible as ::core::ffi::c_int)) as uint16;
    }
    (*E).base.properties = v3;
}
pub unsafe fn RidleysExplosion_Func_2(game: &mut Game) {
    SpawnEnemy(0xa6 as uint8, addr_stru_A6C9E7 as uint16, game);
    SpawnEnemy(0xa6 as uint8, addr_stru_A6C9D7 as uint16, game);
    SpawnEnemy(0xa6 as uint8, addr_stru_A6C9C7 as uint16, game);
    SpawnEnemy(0xa6 as uint8, addr_stru_A6C9B7 as uint16, game);
    SpawnEnemy(0xa6 as uint8, addr_stru_A6C9A7 as uint16, game);
    SpawnEnemy(0xa6 as uint8, addr_stru_A6C997 as uint16, game);
    SpawnEnemy(0xa6 as uint8, addr_stru_A6C987 as uint16, game);
    SpawnEnemy(0xa6 as uint8, addr_stru_A6C9F7 as uint16, game);
    SpawnEnemy(0xa6 as uint8, addr_stru_A6CA07 as uint16, game);
    SpawnEnemy(0xa6 as uint8, addr_stru_A6CA27 as uint16, game);
    SpawnEnemy(0xa6 as uint8, addr_stru_A6CA17 as uint16, game);
    SpawnEnemy(0xa6 as uint8, addr_stru_A6CA37 as uint16, game);
}
pub fn locret_A6CB20(game: &mut Game) {}
static off_A6CB21: [Option<Func_V>; 9] = unsafe {
    [
        Some(locret_A6CB20 as unsafe fn(&mut Game) -> ()),
        Some(Ridley_CBC0 as unsafe fn(&mut Game) -> ()),
        Some(Ridley_Func_71 as unsafe fn(&mut Game) -> ()),
        Some(Ridley_Func_72 as unsafe fn(&mut Game) -> ()),
        Some(Ridley_CBC7 as unsafe fn(&mut Game) -> ()),
        Some(Ridley_CBCE as unsafe fn(&mut Game) -> ()),
        Some(Ridley_Func_73 as unsafe fn(&mut Game) -> ()),
        Some(Ridley_Func_74 as unsafe fn(&mut Game) -> ()),
        Some(Ridley_CBD5 as unsafe fn(&mut Game) -> ()),
    ]
};
pub unsafe fn Ridley_Func_70(game: &mut Game) {
    Ridley_Func_91(game);
    off_A6CB21
        [*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(0) as usize]
        .expect("non-null function pointer")(game);
    Ridley_Func_86(game);
    Ridley_Func_87(game);
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    if (*E).ridley_var_1B as ::core::ffi::c_int
        | (*((&raw mut game.g_ram as *mut uint8).offset(0x18a8) as *mut uint16)
            as ::core::ffi::c_int
            | (*E).base.properties as ::core::ffi::c_int
                & kEnemyProps_Tangible as ::core::ffi::c_int) as uint16
            as ::core::ffi::c_int
        == 0
    {
        Ridley_Func_127(game);
    }
}
pub unsafe fn Ridley_Func_71(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(10) = 8 as uint16;
    Ridley_Func_75(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(0) = 3 as uint16;
}
pub unsafe fn Ridley_Func_72(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(10) = 8 as uint16;
    Ridley_Func_75(game);
}
pub unsafe fn Ridley_Func_73(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(10) = 3 as uint16;
    Ridley_Func_75(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(0) = 6 as uint16;
}
pub unsafe fn Ridley_Func_74(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(10) = 2 as uint16;
    Ridley_Func_75(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(0) = 7 as uint16;
}
pub unsafe fn Ridley_Func_75(game: &mut Game) {
    if (*Get_Ridley(0 as ::core::ffi::c_int, game)).ridley_var_10 != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(13) =
            0x4000 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(14) =
            -(1 as ::core::ffi::c_int) as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(14) =
            0x4000 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(13) =
            -(1 as ::core::ffi::c_int) as uint16;
    }
    Ridley_CBC7(game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(76)
        as ::core::ffi::c_int
        | (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(66)
            as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(56)
                as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(46)
                as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(36)
                as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(26)
                as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(16)
                as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
        == 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(0) =
            4 as uint16;
    }
}
pub unsafe fn Ridley_CBC0(game: &mut Game) {
    Ridley_Func_79(game);
    Ridley_CBDC(game);
}
pub unsafe fn Ridley_CBC7(game: &mut Game) {
    Ridley_Func_83(game);
    Ridley_CBDC(game);
}
pub unsafe fn Ridley_CBCE(game: &mut Game) {
    Ridley_Func_84(game);
    Ridley_CBDC(game);
}
pub unsafe fn Ridley_CBD5(game: &mut Game) {
    Ridley_Func_81(game);
    Ridley_CBDC(game);
}
pub unsafe fn Ridley_CBDC(game: &mut Game) {
    let mut v0: uint16 = 7 as uint16;
    let mut v1: uint16 = 0 as uint16;
    loop {
        Ridley_Func_88(v1, game);
        v1 = (v1).wrapping_add(20u16);
        let fresh11 = v0;
        v0 = v0.wrapping_sub(1);
        if !(fresh11 != 1u16) {
            break;
        }
    }
}
pub unsafe fn Ridley_Func_77(game: &mut Game) {
    let v0: ::core::ffi::c_int =
        (*Get_Ridley(0 as ::core::ffi::c_int, game)).ridley_var_10 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(11) =
        *(game.g_rom.offset(crate::rtl::lorom_offset(0xa6cc12)) as *const uint8 as *mut uint16)
            .offset(v0 as isize);
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(12) =
        *(game.g_rom.offset(crate::rtl::lorom_offset(0xa6cc18)) as *const uint8 as *mut uint16)
            .offset(v0 as isize);
}
pub unsafe fn Ridley_Func_78(game: &mut Game) -> uint8 {
    ((*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(76)
        as ::core::ffi::c_int
        & (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(66)
            as ::core::ffi::c_int
            & *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(56)
                as ::core::ffi::c_int
            & *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(46)
                as ::core::ffi::c_int
            & *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(36)
                as ::core::ffi::c_int
            & *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(26)
                as ::core::ffi::c_int
            & *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(16)
                as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int)
        != 0i32) as uint8
}
pub unsafe fn Ridley_Func_79(game: &mut Game) {
    Ridley_Func_77(game);
    if Ridley_Func_78(game) != 0 {
        if Ridley_Func_80_DoubleRet(game) != 0 {
            return;
        }
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(76)
        as ::core::ffi::c_int
        | (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(66)
            as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(56)
                as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(46)
                as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(36)
                as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(26)
                as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(16)
                as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
        == 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(16) =
            0x8000 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(13) =
            -(1 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(14) =
            -(1 as ::core::ffi::c_int) as uint16;
        let mut v0: uint16 = 2 as uint16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) < 0x70u16 {
            v0 = 1 as uint16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(10) = v0;
    }
}
pub unsafe fn Ridley_Func_80_DoubleRet(game: &mut Game) -> uint8 {
    let mut current_block: u64;
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    let mut retval: bool = false_0 != 0;
    let mut v1: uint16 = 0;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(2) != 0u16
        && (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(13)
            as ::core::ffi::c_int
            & *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(14)
                as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int)
            != 0i32
    {
        current_block = 5875655901067838705;
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(1) == 0 {
        current_block = 15129621311889290514;
    } else if (*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16) as uint8)
        >= 0xf0u8
    {
        current_block = 5875655901067838705;
    } else if abs16(
        (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16))
            .wrapping_sub((*E).base.x_pos),
    ) < 0x80u16
    {
        v1 = 0 as uint16;
        current_block = 993568416893248401;
    } else {
        current_block = 15129621311889290514;
    }
    match current_block {
        5875655901067838705 => {
            v1 = (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(2))
                .wrapping_sub(1u16);
            current_block = 993568416893248401;
        }
        _ => {}
    }
    match current_block {
        993568416893248401 => {
            Ridley_Func_89(v1, game);
            retval = true_0 != 0;
        }
        _ => {}
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(2) = 0 as uint16;
    retval as uint8
}
pub unsafe fn Ridley_Func_81(game: &mut Game) {
    Ridley_Func_77(game);
    if Ridley_Func_78(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        if Ridley_Func_82_DoubleRet(game) {
            return;
        }
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(76)
        as ::core::ffi::c_int
        | (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(66)
            as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(56)
                as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(46)
                as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(36)
                as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(26)
                as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(16)
                as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
        == 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(16) =
            0x8000 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(13) =
            -(1 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(14) =
            -(1 as ::core::ffi::c_int) as uint16;
        let mut v0: uint16 = 2 as uint16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) < 0x70u16 {
            v0 = 1 as uint16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(10) = v0;
    }
}
pub unsafe fn Ridley_Func_82_DoubleRet(game: &mut Game) -> bool {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(2) != 0 {
        if (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(13)
            as ::core::ffi::c_int
            & *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(14)
                as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int)
            != 0i32
        {
            Ridley_Func_89(1 as uint16, game);
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(2) =
                0 as uint16;
            return true_0 != 0;
        }
    }
    false_0 != 0
}
pub unsafe fn Ridley_Func_83(game: &mut Game) {
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    Ridley_Func_77(game);
    if Ridley_Func_78(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
        && ((*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16) as uint8)
            >= 0xf0u8
            || abs16(
                (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16))
                    .wrapping_sub((*E).base.x_pos),
            ) < 0x80u16)
        && (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(13)
            as ::core::ffi::c_int
            & *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(14)
                as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int)
            != 0i32
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(13) =
            16128 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(10) =
            8 as uint16;
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(76)
        as ::core::ffi::c_int
        | (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(66)
            as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(56)
                as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(46)
                as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(36)
                as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(26)
                as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(16)
                as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
        == 0
    {
        if ((*Get_Ridley(0 as ::core::ffi::c_int, game)).ridley_var_C as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int)
            == 0i32
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(0) =
                5 as uint16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(13) =
            -(1 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(14) =
            -(1 as ::core::ffi::c_int) as uint16;
        let mut v1: uint16 =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(8);
        if *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(8) == 0 || {
            v1 = (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(8))
                .wrapping_sub(1u16);
            let ref mut fresh14 =
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(8);
            *fresh14 = v1;
            *fresh14 != 0u16
        } {
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(10) = v1;
        } else {
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(16) =
                0x8000 as uint16;
            Ridley_Func_85(game);
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(10) =
                8 as uint16;
        }
    }
}
pub unsafe fn Ridley_Func_84(game: &mut Game) {
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    Ridley_Func_77(game);
    if Ridley_Func_78(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0
        && ((*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16) as uint8)
            >= 0xf0u8
            || abs16(
                (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16))
                    .wrapping_sub((*E).base.x_pos),
            ) < 0x80u16)
        && (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(13)
            as ::core::ffi::c_int
            & *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(14)
                as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int)
            != 0i32
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(13) =
            16128 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(10) =
            8 as uint16;
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(76)
        as ::core::ffi::c_int
        | (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(66)
            as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(56)
                as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(46)
                as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(36)
                as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(26)
                as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(16)
                as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
        == 0
    {
        if ((*E).ridley_var_C as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) == 0i32 {
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(0) =
                6 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(20) =
                2560 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(30) =
                2560 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(40) =
                2560 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(50) =
                2560 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(60) =
                2560 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(70) =
                2560 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(80) =
                2560 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(21) =
                0x4000 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(31) =
                0x4000 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(41) =
                0x4000 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(51) =
                0x4000 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(61) =
                0x4000 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(71) =
                0x4000 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(81) =
                0x4000 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(16) =
                0x8000 as uint16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(13) =
            -(1 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(14) =
            -(1 as ::core::ffi::c_int) as uint16;
        let mut v1: uint16 =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(8);
        if *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(8) != 0 {
            v1 = (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(8))
                .wrapping_sub(1u16);
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(8) = v1;
            if v1 == 0 {
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(16) =
                    0x8000 as uint16;
                Ridley_Func_85(game);
                v1 = 8 as uint16;
            }
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(10) = v1;
    }
}
pub unsafe fn Ridley_Func_85(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(21) =
        ((*(((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(21)
            as *mut uint16 as *mut uint8)
            .offset(0)) as u16)
            | (0x4000u16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(31) =
        ((*(((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(31)
            as *mut uint16 as *mut uint8)
            .offset(0)) as u16)
            | (0x4000u16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(41) =
        ((*(((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(41)
            as *mut uint16 as *mut uint8)
            .offset(0)) as u16)
            | (0x4000u16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(51) =
        ((*(((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(51)
            as *mut uint16 as *mut uint8)
            .offset(0)) as u16)
            | (0x4000u16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(61) =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(21)
            as ::core::ffi::c_int
            & 0x4ff as ::core::ffi::c_int
            | 0x4000 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(71) =
        ((*(((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(71)
            as *mut uint16 as *mut uint8)
            .offset(0)) as u16)
            | (0x4000u16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(81) =
        ((*(((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(81)
            as *mut uint16 as *mut uint8)
            .offset(0)) as u16)
            | (0x4000u16);
}
pub unsafe fn Ridley_Func_86(game: &mut Game) {
    let mut v1: int16 = 0;
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(23) =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(25)
            as ::core::ffi::c_int
            + (*E).base.y_pos as ::core::ffi::c_int
            + 16 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(33) =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(35))
            .wrapping_add(
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(23),
            );
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(43) =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(45)
            as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(35)
                as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(23)
                as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(53) =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(55))
            .wrapping_add(
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(43),
            );
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(63) =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(65)
            as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(55)
                as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(43)
                as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(73) =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(75))
            .wrapping_add(
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(63),
            );
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(83) =
        (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(85)
            as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(75)
                as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(63)
                as ::core::ffi::c_int) as uint16;
    v1 = (((*E).ridley_var_10) as i16).wrapping_sub(1i16);
    if (*E).ridley_var_10 != 1u16 {
        v1 = ((*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(24))
            as i16)
            .wrapping_add(g_word_A6CF54[(*E).ridley_var_10 as usize]);
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(22) =
        ((*E).base.x_pos).wrapping_add((v1) as u16);
    if (*E).ridley_var_10 == 1u16 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(32) =
            (*E).base.x_pos;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(42) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(32);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(52) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(32);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(62) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(32);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(72) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(32);
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(82) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(32);
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(32) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(34))
                .wrapping_add(
                    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(22),
                );
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(42) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(44)
                as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(34)
                    as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(22)
                    as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(52) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(54))
                .wrapping_add(
                    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(42),
                );
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(62) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(64)
                as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(54)
                    as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(42)
                    as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(72) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(74))
                .wrapping_add(
                    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(62),
                );
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(82) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(84)
                as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(74)
                    as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(62)
                    as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn Ridley_Func_87(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(30) != 0 {
        if ((*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(30))
            as i16)
            .wrapping_sub(
                (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(29))
                    as i16,
            )
            < 0i16
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(30) =
                0 as uint16;
        }
        let mut v0: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset(9))
        .wrapping_add(
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(29),
        );
        if *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(9)
            as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(29)
                as ::core::ffi::c_int
            - 6144 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            v0 = 6144 as uint16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(29) = v0;
    } else if (2048i16).wrapping_sub(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(29)) as i16,
    ) < 0i16
    {
        let ref mut fresh5 =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(29);
        *fresh5 = (*fresh5).wrapping_sub(128u16);
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(40) != 0 {
        if ((*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(40))
            as i16)
            .wrapping_sub(
                (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(39))
                    as i16,
            )
            < 0i16
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(40) =
                0 as uint16;
        }
        let mut v1: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset(9))
        .wrapping_add(
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(39),
        );
        if *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(9)
            as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(39)
                as ::core::ffi::c_int
            - 6144 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            v1 = 6144 as uint16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(39) = v1;
    } else if (2048i16).wrapping_sub(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(39)) as i16,
    ) < 0i16
    {
        let ref mut fresh6 =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(39);
        *fresh6 = (*fresh6).wrapping_sub(128u16);
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(50) != 0 {
        if ((*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(50))
            as i16)
            .wrapping_sub(
                (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(49))
                    as i16,
            )
            < 0i16
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(50) =
                0 as uint16;
        }
        let mut v2: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset(9))
        .wrapping_add(
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(49),
        );
        if *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(9)
            as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(49)
                as ::core::ffi::c_int
            - 5632 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            v2 = 5632 as uint16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(49) = v2;
    } else if (2048i16).wrapping_sub(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(49)) as i16,
    ) < 0i16
    {
        let ref mut fresh7 =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(49);
        *fresh7 = (*fresh7).wrapping_sub(128u16);
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(60) != 0 {
        if ((*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(60))
            as i16)
            .wrapping_sub(
                (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(59))
                    as i16,
            )
            < 0i16
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(60) =
                0 as uint16;
        }
        let mut v3: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset(9))
        .wrapping_add(
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(59),
        );
        if *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(9)
            as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(59)
                as ::core::ffi::c_int
            - 5632 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            v3 = 5632 as uint16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(59) = v3;
    } else if (2048i16).wrapping_sub(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(59)) as i16,
    ) < 0i16
    {
        let ref mut fresh8 =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(59);
        *fresh8 = (*fresh8).wrapping_sub(128u16);
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(70) != 0 {
        if ((*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(70))
            as i16)
            .wrapping_sub(
                (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(69))
                    as i16,
            )
            < 0i16
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(70) =
                0 as uint16;
        }
        let mut v4: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset(9))
        .wrapping_add(
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(69),
        );
        if *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(9)
            as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(69)
                as ::core::ffi::c_int
            - 4608 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            v4 = 4608 as uint16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(69) = v4;
    } else if (2048i16).wrapping_sub(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(69)) as i16,
    ) < 0i16
    {
        let ref mut fresh9 =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(69);
        *fresh9 = (*fresh9).wrapping_sub(128u16);
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(80) != 0 {
        if ((*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(80))
            as i16)
            .wrapping_sub(
                (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(79))
                    as i16,
            )
            < 0i16
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(80) =
                0 as uint16;
        }
        let mut v5: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset(9))
        .wrapping_add(
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(79),
        );
        if *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(9)
            as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(79)
                as ::core::ffi::c_int
            - 1280 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            v5 = 1280 as uint16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(79) = v5;
    } else if (1280i16).wrapping_sub(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(79)) as i16,
    ) < 0i16
    {
        let ref mut fresh10 =
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(79);
        *fresh10 = (*fresh10).wrapping_sub(128u16);
    }
}
pub unsafe fn Ridley_Func_88(k: uint16, game: &mut Game) {
    let mut current_block: u64;
    let mut v6: int16 = 0;
    let mut v2: uint16 = 0;
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
        .offset((v1 + 16 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int)
        != 0i32
    {
        let v4: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let v5: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v4 + 17 as ::core::ffi::c_int) as isize);
        if v5 < *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(15) {
            let ref mut fresh12 =
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(
                    ((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
                        + 17 as ::core::ffi::c_int) as isize,
                );
            *fresh12 = (*fresh12).wrapping_add(
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(10),
            );
            return;
        }
        if v5 != 0xffffu16 {
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v4 + 17 as ::core::ffi::c_int) as isize) =
                -(1 as ::core::ffi::c_int) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v4 + 26 as ::core::ffi::c_int) as isize) = 0x8000 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v4 + 28 as ::core::ffi::c_int) as isize) =
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                    .offset((v4 + 18 as ::core::ffi::c_int) as isize);
        }
        if (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v4 + 18 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int)
            == 0i32
        {
            if (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(14)
                as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int)
                != 0i32
            {
                v2 = (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                    .offset(10))
                .wrapping_add(
                    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                        .offset((v4 + 21 as ::core::ffi::c_int) as isize),
                );
                if ((v2) as i16).wrapping_sub(
                    (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                        .offset(12)) as i16,
                ) >= 0i16
                {
                    current_block = 4838848238792072683;
                } else {
                    current_block = 4216153537599944117;
                }
            } else {
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                    .offset((v4 + 20 as ::core::ffi::c_int) as isize) = 3072 as uint16;
                v2 = (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                    .offset(10))
                .wrapping_add(
                    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                        .offset((v4 + 21 as ::core::ffi::c_int) as isize),
                );
                if ((v2) as i16).wrapping_sub(
                    (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                        .offset(14)) as i16,
                ) >= 0i16
                {
                    if k == 0
                        || *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                            .offset((v4 + 6 as ::core::ffi::c_int) as isize)
                            == 0
                    {
                        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                            .offset((v4 + 21 as ::core::ffi::c_int) as isize) =
                            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                                .offset(14);
                        current_block = 5689001924483802034;
                    } else {
                        current_block = 4838848238792072683;
                    }
                } else {
                    current_block = 4216153537599944117;
                }
            }
            match current_block {
                5689001924483802034 => {}
                4216153537599944117 => {}
                _ => {
                    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                        .offset((v4 + 18 as ::core::ffi::c_int) as isize) = 0x8000 as uint16;
                    v2 = *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                        .offset(12);
                    current_block = 4216153537599944117;
                }
            }
        } else {
            if (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(13)
                as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int)
                != 0i32
            {
                v6 = (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                    .offset((v4 + 21 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_int
                    - *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                        .offset(10) as ::core::ffi::c_int
                    - 1 as ::core::ffi::c_int) as int16;
                if (v6).wrapping_sub(
                    (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                        .offset(11)) as i16,
                ) < 0i16
                {
                    current_block = 9260896848985358503;
                } else {
                    current_block = 2873832966593178012;
                }
            } else {
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                    .offset((v4 + 20 as ::core::ffi::c_int) as isize) = 3072 as uint16;
                v6 = (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                    .offset((v4 + 21 as ::core::ffi::c_int) as isize)
                    as ::core::ffi::c_int
                    - *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                        .offset(10) as ::core::ffi::c_int
                    - 1 as ::core::ffi::c_int) as int16;
                if (v6).wrapping_sub(
                    (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                        .offset(13)) as i16,
                ) < 0i16
                {
                    if k == 0
                        || *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                            .offset((v4 + 6 as ::core::ffi::c_int) as isize)
                            == 0
                    {
                        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                            .offset((v4 + 21 as ::core::ffi::c_int) as isize) =
                            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                                .offset(13);
                        current_block = 5689001924483802034;
                    } else {
                        current_block = 9260896848985358503;
                    }
                } else {
                    current_block = 2873832966593178012;
                }
            }
            match current_block {
                5689001924483802034 => {}
                _ => {
                    match current_block {
                        2873832966593178012 => {
                            v2 = ((v6) as u16).wrapping_add(1u16);
                        }
                        _ => {
                            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                                .offset((v4 + 18 as ::core::ffi::c_int) as isize) = 0 as uint16;
                            v2 = *((&raw mut game.g_ram as *mut uint8).offset(0x2000)
                                as *mut uint16)
                                .offset(11);
                        }
                    }
                    current_block = 4216153537599944117;
                }
            }
        }
        match current_block {
            5689001924483802034 => {
                let v3: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                    .offset((v3 + 16 as ::core::ffi::c_int) as isize) = 0 as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                    .offset((v3 + 17 as ::core::ffi::c_int) as isize) = 0 as uint16;
                let ref mut fresh13 = *((&raw mut game.g_ram as *mut uint8).offset(0x2000)
                    as *mut uint16)
                    .offset((v3 + 18 as ::core::ffi::c_int) as isize);
                *fresh13 = (*fresh13) ^ (0x8000u16);
                return;
            }
            _ => {
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                    .offset((v4 + 21 as ::core::ffi::c_int) as isize) = v2;
            }
        }
    } else {
        v2 = *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v1 + 21 as ::core::ffi::c_int) as isize);
    }
    let mut r18: uint16 = v2 as uint8 as uint16;
    if k != 0 {
        r18 = ((*(((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(
            ((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) + 11 as ::core::ffi::c_int)
                as isize,
        ) as *mut uint16 as *mut uint8)
            .offset(0))
        .wrapping_add((r18) as u8)) as uint16;
    }
    let v7: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
        .offset((v7 + 24 as ::core::ffi::c_int) as isize) = ComputeSinMult(
        *(((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v7 + 19 as ::core::ffi::c_int) as isize) as *mut uint16
            as *mut uint8)
            .offset(
                (::core::mem::size_of::<uint16>())
                    .wrapping_div(::core::mem::size_of::<uint8>())
                    .wrapping_sub(1_usize) as isize,
            ) as uint16,
        r18,
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
        .offset((v7 + 25 as ::core::ffi::c_int) as isize) = ComputeCosMult(
        *(((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v7 + 19 as ::core::ffi::c_int) as isize) as *mut uint16
            as *mut uint8)
            .offset(
                (::core::mem::size_of::<uint16>())
                    .wrapping_div(::core::mem::size_of::<uint8>())
                    .wrapping_sub(1_usize) as isize,
            ) as uint16,
        r18,
        game,
    );
}
pub unsafe fn Ridley_Func_89(a: uint16, game: &mut Game) {
    let mut v1: uint16 = 0;
    let mut r18: uint16 = 0;
    let mut r20: uint16 = 0;
    let mut v2: int8 = 0;
    v2 = a as int8;
    *(&raw mut v1 as *mut uint8).offset(0) = 0 as uint8;
    *(&raw mut v1 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>())
            .wrapping_div(::core::mem::size_of::<uint8>())
            .wrapping_sub(1_usize) as isize,
    ) = v2 as uint8;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(5) = v1;
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    if (*E).ridley_var_10 != 1u16 && (*E).ridley_var_01 != 0u16 {
        let mut pt: Point16U = Point16U { x: 0, y: 0 };
        if Ridley_Func_90(&raw mut pt, game) != 0 {
            r18 = (pt.x).wrapping_sub(
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(22),
            );
            r20 = (pt.y).wrapping_sub(
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(23),
            );
        } else {
            r18 = (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16))
                .wrapping_sub(
                    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(22),
                );
            r20 = (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
                as ::core::ffi::c_int
                + 24 as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(23)
                    as ::core::ffi::c_int) as uint16;
        }
        r18 = -(CalculateAngleFromXY(r18, r20, game) as ::core::ffi::c_int
            + 0x80 as ::core::ffi::c_int) as uint8 as uint16;
        if (*E).ridley_var_10 != 0 {
            let mut v7: uint16 = r18;
            if r18 >= 0x18u16 && r18 < 0xe8u16 {
                v7 = 24 as uint16;
            }
            let v8: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset(5) as ::core::ffi::c_int
                + v7 as ::core::ffi::c_int
                + 0x4000 as ::core::ffi::c_int) as uint16;
            if v8 >= *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(21)
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(14) =
                    v8;
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(10) =
                    8 as uint16;
            }
        } else {
            let mut v5: uint16 = r18;
            if r18 >= 0x18u16 && r18 < 0xe8u16 {
                v5 = 232 as uint16;
            }
            let v6: uint16 = (v5 as ::core::ffi::c_int + 16128 as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(5)
                    as ::core::ffi::c_int) as uint16;
            if v6 < *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(21)
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(13) =
                    v6;
                *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(10) =
                    8 as uint16;
            }
        }
    }
}
pub unsafe fn Ridley_Func_90(out: *mut Point16U, game: &mut Game) -> uint8 {
    let rect: Rect16U = Rect16U {
        x: *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(82),
        y: *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(83),
        w: 64 as uint16,
        h: 64 as uint16,
    };
    if *((&raw mut game.g_ram as *mut uint8).offset(0xcce) as *mut uint16) == 0 {
        return 0 as uint8;
    }
    let mut v1: uint16 = 0 as uint16;
    loop {
        let v2: ::core::ffi::c_int = v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        if (*(((&raw mut game.g_ram as *mut uint8).offset(0xc18) as *mut uint16).offset(v2 as isize)
            as *mut uint16 as *mut uint8)
            .offset(
                (::core::mem::size_of::<uint16>())
                    .wrapping_div(::core::mem::size_of::<uint8>())
                    .wrapping_sub(1_usize) as isize,
            ) as ::core::ffi::c_int
            & 0xf as ::core::ffi::c_int)
            == 1i32
            || (*(((&raw mut game.g_ram as *mut uint8).offset(0xc18) as *mut uint16)
                .offset(v2 as isize) as *mut uint16 as *mut uint8)
                .offset(
                    (::core::mem::size_of::<uint16>())
                        .wrapping_div(::core::mem::size_of::<uint8>())
                        .wrapping_sub(1_usize) as isize,
                ) as ::core::ffi::c_int
                & 0xf as ::core::ffi::c_int)
                == 2i32
        {
            let v3: uint16 = abs16(
                (*((&raw mut game.g_ram as *mut uint8).offset(0xb64) as *mut uint16)
                    .offset(v2 as isize))
                .wrapping_sub(rect.x),
            );
            let mut v4: bool = v3
                < *((&raw mut game.g_ram as *mut uint8).offset(0xbb4) as *mut uint16)
                    .offset(v2 as isize);
            let v5: uint16 = (v3).wrapping_sub(
                *((&raw mut game.g_ram as *mut uint8).offset(0xbb4) as *mut uint16)
                    .offset(v2 as isize),
            );
            if v4 || v5 < rect.w {
                let v6: uint16 = abs16(
                    (*((&raw mut game.g_ram as *mut uint8).offset(0xb78) as *mut uint16)
                        .offset(v2 as isize))
                    .wrapping_sub(rect.y),
                );
                v4 = v6
                    < *((&raw mut game.g_ram as *mut uint8).offset(0xbc8) as *mut uint16)
                        .offset(v2 as isize);
                let v7: uint16 = (v6).wrapping_sub(
                    *((&raw mut game.g_ram as *mut uint8).offset(0xbc8) as *mut uint16)
                        .offset(v2 as isize),
                );
                if v4 || v7 < rect.h {
                    break;
                }
            }
        }
        v1 = (v1).wrapping_add(2u16);
        if ((v1) as i16).wrapping_sub(10i16) >= 0i16 {
            return 0 as uint8;
        }
    }
    let v8: ::core::ffi::c_int = v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    (*out).x =
        *((&raw mut game.g_ram as *mut uint8).offset(0xb64) as *mut uint16).offset(v8 as isize);
    (*out).y =
        *((&raw mut game.g_ram as *mut uint8).offset(0xb78) as *mut uint16).offset(v8 as isize);
    1 as uint8
}
pub unsafe fn Ridley_Func_91(game: &mut Game) {
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(10)
        as ::core::ffi::c_int
        - 8 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        let v1: uint16 = ((*E).ridley_var_0E).wrapping_add(1u16);
        (*E).ridley_var_0E = v1;
        if v1 as ::core::ffi::c_int - 16 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
            return;
        }
        if (*E).ridley_var_0F == 0 {
            QueueSfx3_Max6(0x21 as uint16, game);
        }
    }
    (*E).ridley_var_0E = 0 as uint16;
}
pub unsafe fn Ridley_Func_92(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(10) = 1 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(11) =
        16368 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(12) =
        16448 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(13) =
        -(1 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(14) =
        -(1 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(9) = 240 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(15) = 16 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(17) = 17 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(27) = 17 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(37) = 17 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(47) = 17 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(57) = 17 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(67) = 17 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(77) = 17 as uint16;
    Ridley_Func_93(0x2024 as uint16, addr_word_A6D36E as uint16, game);
    Ridley_Func_93(0x2026 as uint16, addr_word_A6D37C as uint16, game);
    Ridley_Func_93(0x202a as uint16, addr_word_A6D38A as uint16, game);
    Ridley_Func_93(0x202c as uint16, addr_word_A6D398 as uint16, game);
    Ridley_Func_93(0x202e as uint16, addr_word_A6D3A6 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(26) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(36) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(46) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(56) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(66) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(76) = 0 as uint16;
}
pub unsafe fn Ridley_Func_93(j: uint16, k: uint16, game: &mut Game) {
    let mut dst: *mut uint16 =
        (&raw mut game.g_ram as *mut uint8).offset(j as ::core::ffi::c_int as isize) as *mut uint16;
    let mut src: *const uint16 = RomPtr_A6(k, game) as *const uint16;
    let mut v3: ::core::ffi::c_int = 7 as ::core::ffi::c_int;
    loop {
        *dst = *src;
        dst = dst.offset(10);
        src = src.offset(1);
        v3 -= 1;
        if !(v3 != 0) {
            break;
        }
    }
}
pub unsafe fn Ridley_Func_94(game: &mut Game) {
    Ridley_Func_D3DC(0 as uint16, game);
}
pub unsafe fn Ridley_Func_95(game: &mut Game) {
    Ridley_Func_D3DC(0x8000 as uint16, game);
}
pub unsafe fn Ridley_Func_D3DC(a: uint16, game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(16) = a;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(26) = a;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(36) = a;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(46) = a;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(56) = a;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(66) = a;
    *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(76) = a;
}
pub unsafe fn Ridley_Func_96(game: &mut Game) {
    let mut v0: int16 = 0;
    Ridley_Func_93(0x2026 as uint16, addr_word_A6D37C as uint16, game);
    v0 = 7 as int16;
    let mut v1: uint16 = 0 as uint16;
    loop {
        let v2: ::core::ffi::c_int = v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v2 + 21 as ::core::ffi::c_int) as isize) = (0x8000u16).wrapping_sub(
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v2 + 21 as ::core::ffi::c_int) as isize),
        );
        let ref mut fresh3 = *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v2 + 18 as ::core::ffi::c_int) as isize);
        *fresh3 = (*fresh3) | (0x8000u16);
        v1 = (v1).wrapping_add(20u16);
        v0 -= 1;
        if !(v0 != 0) {
            break;
        }
    }
}
pub unsafe fn Ridley_Func_97(game: &mut Game) -> uint8 {
    ((*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(76)
        as ::core::ffi::c_int
        & (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(66)
            as ::core::ffi::c_int
            & *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(56)
                as ::core::ffi::c_int
            & *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(46)
                as ::core::ffi::c_int
            & *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(36)
                as ::core::ffi::c_int
            & *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(26)
                as ::core::ffi::c_int
            & *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(16)
                as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int)
        != 0i32) as uint8
}
pub unsafe fn Ridley_Func_98(game: &mut Game) {
    let enemy_ptr: uint16 = (*Get_Ridley(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .base
    .enemy_ptr;
    let v1: *const uint8 = RomPtr_A0(enemy_ptr, game);
    Samus_DealDamage(
        SuitDamageDivision(*(v1.offset(6) as *mut uint16), game),
        game,
    );
}
pub unsafe fn Ridley_Func_99(a: uint16, game: &mut Game) {
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    (*E).base.current_instruction = a;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
}
pub unsafe fn Ridley_Func_100(game: &mut Game) {
    let mut v0: int16 = 0;
    v0 = 0 as int16;
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    let health: uint16 = (*E).base.health;
    if health < 0x2328u16 {
        v0 = 1 as int16;
        if health < 0x1518u16 {
            v0 = 2 as int16;
            if health < 0x708u16 {
                v0 = 3 as int16;
            }
        }
    }
    (*E).ridley_var_12 = v0 as uint16;
    if (v0).wrapping_sub(1i16) >= 0i16 {
        Ridley_D495(((v0) as u16).wrapping_sub(1u16), game);
    }
}
pub unsafe fn Ridley_D495(r18: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xc400) as *mut uint16) == 0 {
        WriteColorsToPalette(
            0x1e2 as uint16,
            0xa6 as uint8,
            (28 as ::core::ffi::c_int * r18 as ::core::ffi::c_int - 7062 as ::core::ffi::c_int)
                as uint16,
            0xe as uint16,
            game,
        );
    }
}
pub unsafe fn Ridley_Func_101(game: &mut Game) {
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    if (*E).ridley_var_01 != 0u16 && (*E).ridley_var_0D >= 0x32u16 {
        Ridley_D495(
            (if (*E).ridley_var_0D >= 0x46u16 {
                2 as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }) as uint16,
            game,
        );
    }
}
pub unsafe fn Ridley_Func_102(game: &mut Game) {
    let mut v0: int16 = 0;
    v0 = 3584 as int16;
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    if (*E).base.flash_timer >= 2u16
        && (*((&raw mut game.g_ram as *mut uint8).offset(0xe44) as *mut uint16)
            as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            & 2 as ::core::ffi::c_int)
            != 0i32
    {
        v0 = 0 as int16;
    }
    (*E).ridley_var_0C = v0 as uint16;
}
pub unsafe fn Ridley_Func_103(k: uint16, j: uint16, game: &mut Game) -> uint8 {
    let prod: uint16 = Mult8x8(
        (j as ::core::ffi::c_int >> 4 as ::core::ffi::c_int) as uint8,
        *((&raw mut game.g_ram as *mut uint8).offset(0x7a5) as *mut uint16) as uint8,
    );
    ((*((&raw mut game.g_ram as *mut uint8).offset(0x10002) as *mut uint16).offset(
        (prod as ::core::ffi::c_int + (k as ::core::ffi::c_int >> 4 as ::core::ffi::c_int))
            as isize,
    ) as ::core::ffi::c_int
        & 0xf000 as ::core::ffi::c_int)
        != 0i32) as uint8
}
pub unsafe fn Ridley_Func_104_0(k: uint16, j: uint16, r18: uint16, r20: uint16, game: &mut Game) {
    Ridley_Func_104(k, j, 0 as uint16, r18, r20, game);
}
pub unsafe fn Ridley_Func_104(
    k: uint16,
    j: uint16,
    a: uint16,
    r18: uint16,
    r20: uint16,
    game: &mut Game,
) {
    let mut v4: int16 = 0;
    let mut v6: int16 = 0;
    let mut v8: int16 = 0;
    let mut v9: int16 = 0;
    let mut v11: int16 = 0;
    let mut ridley_var_C: int16 = 0;
    let mut v17: int16 = 0;
    let R26: uint16 = a;
    let r24: uint16 = g_byte_A6D61F[j as usize] as uint16;
    Ridley_Func_105(k, r18, r24, a, game);
    let E: *mut Enemy_Ridley = Get_Ridley(k as ::core::ffi::c_int, game);
    v4 = (((*E).base.y_pos) as i16).wrapping_sub((r20) as i16);
    if v4 != 0 {
        if v4 >= 0i16 {
            let mut RegWord: uint16 = SnesDivide(v4 as uint16, r24 as uint8);
            if RegWord == 0 {
                RegWord = 1 as uint16;
            }
            let r22: uint16 = RegWord;
            ridley_var_C = (*E).ridley_var_C as int16;
            let v14: bool = 0u8 != 0u8;
            if ridley_var_C >= 0i16 {
                let mut v15: uint16 = ((ridley_var_C) as u16).wrapping_sub(R26);
                v15 = (v15).wrapping_sub(8u16);
                let v16: uint16 = r22;
                ridley_var_C = ((v15) as i16).wrapping_sub((v16) as i16);
            }
            v17 = (ridley_var_C).wrapping_sub((r22) as i16);
            if v17 as ::core::ffi::c_int + 1280 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != 0
            {
                v17 = -(1280 as ::core::ffi::c_int) as int16;
            }
            (*E).ridley_var_C = v17 as uint16;
        } else {
            let mut v5: uint16 = SnesDivide((r20).wrapping_sub((*E).base.y_pos), r24 as uint8);
            if v5 == 0 {
                v5 = 1 as uint16;
            }
            let r22_0: uint16 = v5;
            v6 = (*E).ridley_var_C as int16;
            let v7: bool = 0u8 != 0u8;
            if v6 < 0i16 {
                v8 = ((R26) as i16).wrapping_add(v6);
                v8 = (v8).wrapping_add(8i16);
                v9 = v8;
                v6 = ((r22_0) as i16).wrapping_add(v9);
            }
            v11 = ((r22_0) as i16).wrapping_add(v6);
            if v11 as ::core::ffi::c_int - 1280 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                == 0
            {
                v11 = 1280 as int16;
            }
            (*E).ridley_var_C = v11 as uint16;
        }
    }
}
pub unsafe fn Ridley_Func_105(k: uint16, r18: uint16, r24: uint16, r26: uint16, game: &mut Game) {
    let mut v2: int16 = 0;
    let mut v4: int16 = 0;
    let mut v6: int16 = 0;
    let mut v7: int16 = 0;
    let mut v9: int16 = 0;
    let mut ridley_var_B: int16 = 0;
    let mut v15: int16 = 0;
    let E: *mut Enemy_Ridley = Get_Ridley(k as ::core::ffi::c_int, game);
    v2 = (((*E).base.x_pos) as i16).wrapping_sub((r18) as i16);
    if v2 != 0 {
        if v2 >= 0i16 {
            let mut RegWord: uint16 = SnesDivide(v2 as uint16, r24 as uint8);
            if RegWord == 0 {
                RegWord = 1 as uint16;
            }
            let r22: uint16 = RegWord;
            ridley_var_B = (*E).ridley_var_B as int16;
            let v12: bool = 0u8 != 0u8;
            if ridley_var_B >= 0i16 {
                let mut v13: uint16 = ((ridley_var_B) as u16).wrapping_sub(r26);
                v13 = (v13).wrapping_sub(8u16);
                let v14: uint16 = r22;
                ridley_var_B = ((v13) as i16).wrapping_sub((v14) as i16);
            }
            v15 = (ridley_var_B).wrapping_sub((r22) as i16);
            if v15 as ::core::ffi::c_int + 1280 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != 0
            {
                v15 = -(1280 as ::core::ffi::c_int) as int16;
            }
            (*E).ridley_var_B = v15 as uint16;
        } else {
            let mut v3: uint16 = SnesDivide((r18).wrapping_sub((*E).base.x_pos), r24 as uint8);
            if v3 == 0 {
                v3 = 1 as uint16;
            }
            let r22_0: uint16 = v3;
            v4 = (*E).ridley_var_B as int16;
            if v4 < 0i16 {
                v6 = ((r26) as i16).wrapping_add(v4);
                v6 = (v6).wrapping_add(8i16);
                v7 = v6;
                v4 = ((r22_0) as i16).wrapping_add(v7);
            }
            v9 = ((r22_0) as i16).wrapping_add(v4);
            if v9 as ::core::ffi::c_int - 1280 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                == 0
            {
                v9 = 1280 as int16;
            }
            (*E).ridley_var_B = v9 as uint16;
        }
    }
}
pub unsafe fn Ridley_Func_106(k: uint16, j: uint16, r18: uint16, r20: uint16, game: &mut Game) {
    let mut v3: int16 = 0;
    let mut v5: int16 = 0;
    let mut v6: int16 = 0;
    let mut ridley_var_C: int16 = 0;
    let mut v9: int16 = 0;
    let r24: uint16 = g_byte_A6D712[j as usize] as uint16;
    Ridley_Func_107(k, r18, r24, game);
    let E: *mut Enemy_Ridley = Get_Ridley(k as ::core::ffi::c_int, game);
    v3 = (((*E).base.y_pos) as i16).wrapping_sub((r20) as i16);
    if v3 != 0 {
        if v3 >= 0i16 {
            let mut RegWord: uint16 = SnesDivide(v3 as uint16, r24 as uint8);
            if RegWord == 0 {
                RegWord = 1 as uint16;
            }
            let r22: uint16 = RegWord;
            ridley_var_C = (*E).ridley_var_C as int16;
            if ridley_var_C >= 0i16 {
                ridley_var_C = (ridley_var_C as ::core::ffi::c_int
                    - r22 as ::core::ffi::c_int
                    - r22 as ::core::ffi::c_int) as int16;
            }
            v9 = (ridley_var_C).wrapping_sub((r22) as i16);
            if v9 as ::core::ffi::c_int + 1280 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != 0
            {
                v9 = -(1280 as ::core::ffi::c_int) as int16;
            }
            (*E).ridley_var_C = v9 as uint16;
        } else {
            let mut v4: uint16 = SnesDivide((r20).wrapping_sub((*E).base.y_pos), r24 as uint8);
            if v4 == 0 {
                v4 = 1 as uint16;
            }
            let r22_0: uint16 = v4;
            v5 = (*E).ridley_var_C as int16;
            if v5 < 0i16 {
                v5 = (v5 as ::core::ffi::c_int
                    + (r22_0 as ::core::ffi::c_int + r22_0 as ::core::ffi::c_int))
                    as int16;
            }
            v6 = ((r22_0) as i16).wrapping_add(v5);
            if v6 as ::core::ffi::c_int - 1280 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                == 0
            {
                v6 = 1280 as int16;
            }
            (*E).ridley_var_C = v6 as uint16;
        }
    }
}
pub unsafe fn Ridley_Func_107(k: uint16, r18: uint16, r24: uint16, game: &mut Game) {
    let E: *mut Enemy_Ridley = Get_Ridley(k as ::core::ffi::c_int, game);
    let v2: int16 = (((*E).base.x_pos) as i16).wrapping_sub((r18) as i16);
    if v2 == 0 {
        return;
    }
    if v2 >= 0i16 {
        let mut RegWord: uint16 = SnesDivide(v2 as uint16, r24 as uint8);
        RegWord = (if RegWord != 0u16 {
            RegWord as ::core::ffi::c_int
        } else {
            1 as ::core::ffi::c_int
        }) as uint16;
        let r22: uint16 = RegWord;
        let mut s: int16 = (*E).ridley_var_B as int16;
        if s >= 0i16 {
            s = (s as ::core::ffi::c_int - 2 as ::core::ffi::c_int * RegWord as ::core::ffi::c_int)
                as int16;
        }
        s = (s).wrapping_sub((RegWord) as i16);
        (*E).ridley_var_B = (if s as ::core::ffi::c_int + 1280 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            -(1280 as ::core::ffi::c_int)
        } else {
            s as ::core::ffi::c_int
        }) as uint16;
    } else {
        let v3: uint16 = SnesDivide((r18).wrapping_sub((*E).base.x_pos), r24 as uint8);
        let r22_0: uint16 = (if v3 != 0u16 {
            v3 as ::core::ffi::c_int
        } else {
            1 as ::core::ffi::c_int
        }) as uint16;
        let mut s_0: int16 = (*E).ridley_var_B as int16;
        if s_0 < 0i16 {
            s_0 = (s_0 as ::core::ffi::c_int
                + 2 as ::core::ffi::c_int * r22_0 as ::core::ffi::c_int) as int16;
        }
        s_0 = (s_0).wrapping_add((r22_0) as i16);
        (*E).ridley_var_B = (if s_0 as ::core::ffi::c_int - 1280 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            1280 as ::core::ffi::c_int
        } else {
            s_0 as ::core::ffi::c_int
        }) as uint16;
    };
}
pub unsafe fn Ridley_Func_111(mut r18: uint16, r20: uint16, r22: uint16, game: &mut Game) {
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    if r22 != (*E).ridley_var_0B {
        if ((r22) as i16).wrapping_sub(((*E).ridley_var_0B) as i16) >= 0i16 {
            let mut v2: uint16 = ((*E).ridley_var_0B).wrapping_add(32u16);
            if v2 as ::core::ffi::c_int - r22 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                == 0
            {
                v2 = r22;
            }
            (*E).ridley_var_0B = v2;
        } else {
            let mut v1: uint16 = ((*E).ridley_var_0B).wrapping_sub(32u16);
            if v1 as ::core::ffi::c_int - r22 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != 0
            {
                v1 = r22;
            }
            (*E).ridley_var_0B = v1;
        }
    }
    let mut v3: uint16 = 0;
    let current_block_13: u64;
    if (r18 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        v3 = ((*E).ridley_var_0A).wrapping_add(r18);
        if v3 as ::core::ffi::c_int - r20 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0
        {
            current_block_13 = 872684080230000114;
        } else {
            current_block_13 = 12039483399334584727;
        }
    } else {
        v3 = ((*E).ridley_var_0A).wrapping_add(r18);
        if v3 as ::core::ffi::c_int - r20 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0
        {
            current_block_13 = 872684080230000114;
        } else {
            current_block_13 = 12039483399334584727;
        }
    }
    match current_block_13 {
        872684080230000114 => {
            v3 = r20;
        }
        _ => {}
    }
    (*E).ridley_var_0A = v3;
    r18 = *(&raw mut v3 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>())
            .wrapping_div(::core::mem::size_of::<uint8>())
            .wrapping_sub(1_usize) as isize,
    ) as uint16;
    (*E).ridley_var_B = Math_MultBySin((*E).ridley_var_0B, r18);
    (*E).ridley_var_C = Math_MultByCos((*E).ridley_var_0B, r18);
}
pub unsafe fn Ridley_Func_112(game: &mut Game) {
    let mut Ridley: *mut Enemy_Ridley = ::core::ptr::null_mut::<Enemy_Ridley>();
    let mut v1: uint16 = 0;
    let mut v2: *mut Enemy_Ridley = ::core::ptr::null_mut::<Enemy_Ridley>();
    let mut ridley_var_23: uint16 = 0;
    let mut ridley_var_21: uint16 = 0;
    Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    (*Ridley).ridley_var_1F = 0 as uint16;
    v1 = *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16);
    v2 = Get_Ridley(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut with_carry: ::core::ffi::c_int = *(&raw mut (*v2).base.x_subpos as *mut uint8).offset(
        (::core::mem::size_of::<uint16>())
            .wrapping_div(::core::mem::size_of::<uint8>())
            .wrapping_sub(1_usize) as isize,
    ) as ::core::ffi::c_int
        + *(&raw mut (*v2).ridley_var_B as *mut uint8).offset(0) as ::core::ffi::c_int;
    *(&raw mut (*v2).base.x_subpos as *mut uint8).offset(
        (::core::mem::size_of::<uint16>())
            .wrapping_div(::core::mem::size_of::<uint8>())
            .wrapping_sub(1_usize) as isize,
    ) = with_carry as uint8;
    ridley_var_23 = ((*v2).base.x_pos as ::core::ffi::c_int
        + ((*v2).ridley_var_B as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as int8
            as ::core::ffi::c_int
        + (with_carry >> 8 as ::core::ffi::c_int)) as uint16;
    if ((ridley_var_23) as i16).wrapping_sub(((*Ridley).ridley_var_22) as i16) >= 0i16 {
        if ((ridley_var_23) as i16).wrapping_sub(((*Ridley).ridley_var_23) as i16) >= 0i16 {
            (*v2).ridley_var_B = 0 as uint16;
            (*Ridley).ridley_var_1F = 2 as uint16;
            ridley_var_23 = (*Ridley).ridley_var_23;
        }
        (*v2).base.x_pos = ridley_var_23;
    } else {
        Ridley_Func_113(v1, game);
        (*v2).base.x_pos = (*Ridley).ridley_var_22;
        (*v2).ridley_var_B = 0 as uint16;
        (*Ridley).ridley_var_1F = 1 as uint16;
    }
    with_carry = *(&raw mut (*v2).base.y_subpos as *mut uint8).offset(
        (::core::mem::size_of::<uint16>())
            .wrapping_div(::core::mem::size_of::<uint8>())
            .wrapping_sub(1_usize) as isize,
    ) as ::core::ffi::c_int
        + *(&raw mut (*v2).ridley_var_C as *mut uint8).offset(0) as ::core::ffi::c_int;
    *(&raw mut (*v2).base.y_subpos as *mut uint8).offset(
        (::core::mem::size_of::<uint16>())
            .wrapping_div(::core::mem::size_of::<uint8>())
            .wrapping_sub(1_usize) as isize,
    ) = with_carry as uint8;
    ridley_var_21 = ((*v2).base.y_pos as ::core::ffi::c_int
        + ((*v2).ridley_var_C as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as int8
            as ::core::ffi::c_int
        + (with_carry >> 8 as ::core::ffi::c_int)) as uint16;
    if ((ridley_var_21) as i16).wrapping_sub(((*Ridley).ridley_var_20) as i16) >= 0i16 {
        if ((ridley_var_21) as i16).wrapping_sub(((*Ridley).ridley_var_21) as i16) >= 0i16 {
            (*v2).ridley_var_C = 0 as uint16;
            (*Ridley).ridley_var_1F = 8 as uint16;
            ridley_var_21 = (*Ridley).ridley_var_21;
        }
        (*v2).base.y_pos = ridley_var_21;
    } else {
        (*v2).base.y_pos = (*Ridley).ridley_var_20;
        (*v2).ridley_var_C = 0 as uint16;
        (*Ridley).ridley_var_1F = 4 as uint16;
    };
}
pub unsafe fn Ridley_Func_113(k: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) != 2u16 {
        let E: *mut Enemy_Ridley = Get_Ridley(k as ::core::ffi::c_int, game);
        let r18: uint16 = abs16((*E).ridley_var_B);
        let mut v2: uint16 = abs16((*E).ridley_var_C);
        if v2 < r18 {
            v2 = r18;
        }
        if v2 >= 0x280u16 {
            let mut v3: uint16 = 0;
            if *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) == 2u16 {
                v3 = 24 as uint16;
            } else {
                v3 = 33 as uint16;
            }
            *((&raw mut game.g_ram as *mut uint8).offset(0x183e) as *mut uint16) = v3;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1840) as *mut uint16) = 12 as uint16;
        }
    }
}
pub unsafe fn Ridley_Func_114(game: &mut Game) {
    let current_block: u64;
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    let ridley_var_10: uint16 = (*E).ridley_var_10;
    let mut v2: uint16 = 0;
    if ridley_var_10 == 0 {
        if *((&raw mut (*E).base.enemy_ptr as *mut uint8).offset(1) as *mut int16) < 0i16 {
            return;
        }
        v2 = addr_kRidley_Ilist_E6F0 as uint16;
        current_block = 16875472102381009524;
    } else if ridley_var_10 != 1u16
        && *((&raw mut (*E).base.enemy_ptr as *mut uint8).offset(1) as *mut int16) < 0i16
    {
        v2 = addr_kRidley_Ilist_E706 as uint16;
        current_block = 16875472102381009524;
    } else {
        current_block = 14523784380283086299;
    }
    match current_block {
        16875472102381009524 => {
            (*E).base.current_instruction = v2;
            (*E).base.instruction_timer = 2 as uint16;
            (*E).base.timer = 0 as uint16;
        }
        _ => {}
    };
}
pub unsafe fn Ridley_Func_115(game: &mut Game) {
    let mut v1: int16 = 0;
    Ridley_Func_116(game);
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    v1 = (((*E).ridley_var_09) as i16).wrapping_sub(((*E).ridley_var_08) as i16);
    (*E).ridley_var_09 = v1 as uint16;
    if v1 < 0i16 {
        (*E).ridley_var_09 = 32 as uint16;
        let mut v2: uint16 = ((*E).ridley_var_07).wrapping_add(1u16);
        if v2 >= 0xau16 {
            v2 = 0 as uint16;
        }
        (*E).ridley_var_07 = v2;
    }
}
pub unsafe fn Ridley_Func_116(game: &mut Game) {
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    let r18: uint16 = abs16((*E).ridley_var_B);
    let v1: uint16 = (r18).wrapping_add(abs16((*E).ridley_var_C));
    if v1 != 0 {
        let mut v2: uint16 = (v1).wrapping_sub(r18);
        if v2 as ::core::ffi::c_int - r18 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0
        {
            v2 = r18;
        }
        let mut v5: ::core::ffi::c_int = (4 as ::core::ffi::c_int * v2 as ::core::ffi::c_int
            & 0xf00 as ::core::ffi::c_int)
            >> 8 as ::core::ffi::c_int;
        if v5 >= 7i32 {
            v5 = 7 as ::core::ffi::c_int;
        }
        let mut v6: uint16 = g_word_A6D9ED[v5 as usize];
        if ((*E).ridley_var_C as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) == 0i32 {
            v6 = (v6 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
        }
        (*E).ridley_var_08 = v6;
    } else {
        (*E).ridley_var_08 = 0 as uint16;
    };
}
pub unsafe fn Ridley_Func_117(game: &mut Game) {
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    (*E).ridley_var_05 = addr_word_A6DA71 as uint16;
    (*E).ridley_var_06 = 1 as uint16;
}
pub unsafe fn Ridley_Func_118(game: &mut Game) {
    let mut v7: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    let v1: uint16 = ((*E).ridley_var_06).wrapping_sub(1u16);
    (*E).ridley_var_06 = v1;
    if v1 == 0 {
        let mut i: ::core::ffi::c_int = 0;
        let mut v3: uint16 = 0;
        let mut v4: *mut uint16 = ::core::ptr::null_mut::<uint16>();
        i = (*E).ridley_var_05 as ::core::ffi::c_int;
        loop {
            v3 = i as uint16;
            v4 = RomPtr_A6(i as uint16_t, game) as *mut uint16;
            i = *v4 as ::core::ffi::c_int;
            if (*v4 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) == 0i32 {
                break;
            }
            (*E).ridley_var_05 = i as uint16;
        }
        (*E).ridley_var_06 = i as uint16;
        let v6: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16);
        v7 = gVramWriteEntry(
            *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        );
        *((&raw mut (*v7).src.addr as *mut uint8).offset(1) as *mut VoidP) =
            -(20480 as ::core::ffi::c_int) as VoidP;
        *((&raw mut (*v7.offset(1)).src.addr as *mut uint8).offset(1) as *mut VoidP) =
            -(20480 as ::core::ffi::c_int) as VoidP;
        (*v7).src.addr = *v4.offset(1);
        (*v7.offset(1)).src.addr = *v4.offset(2);
        (*v7).vram_dst = 29216 as uint16;
        (*v7.offset(1)).vram_dst = 29472 as uint16;
        (*v7).size = 64 as uint16;
        (*v7.offset(1)).size = 64 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) =
            (v6).wrapping_add(14u16);
        (*gVramWriteEntry(v6 as ::core::ffi::c_int + 14 as ::core::ffi::c_int, game)).size =
            0 as uint16;
        (*E).ridley_var_05 = (v3).wrapping_add(6u16);
    }
}
pub unsafe fn Ridley_Func_119(carry: uint8, game: &mut Game) {
    let mut v3: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    let mut v1: uint16 = addr_off_A6DAD0 as uint16;
    if carry != 0 {
        v1 = addr_off_A6DAD4 as uint16;
    }
    let v2: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16);
    v3 = gVramWriteEntry(
        *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    *((&raw mut (*v3).src.addr as *mut uint8).offset(1) as *mut VoidP) =
        -(20480 as ::core::ffi::c_int) as VoidP;
    *((&raw mut (*v3.offset(1)).src.addr as *mut uint8).offset(1) as *mut VoidP) =
        -(20480 as ::core::ffi::c_int) as VoidP;
    let v4: *const uint8 = RomPtr_A6(v1, game);
    (*v3).src.addr = *(v4 as *mut uint16);
    (*v3.offset(1)).src.addr = *(v4.offset(2) as *mut uint16);
    (*v3).vram_dst = 31424 as uint16;
    (*v3.offset(1)).vram_dst = 31680 as uint16;
    (*v3).size = 128 as uint16;
    (*v3.offset(1)).size = 128 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) = (v2).wrapping_add(14u16);
    (*gVramWriteEntry(v2 as ::core::ffi::c_int + 14 as ::core::ffi::c_int, game)).size =
        0 as uint16;
}
pub unsafe fn Ridley_Func_120(game: &mut Game) {
    let mut ridley_var_10: int16 = 0;
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    ridley_var_10 = (*E).ridley_var_10 as int16;
    if ridley_var_10 != 0 {
        if ridley_var_10 == 1i16 {
            return;
        }
        ridley_var_10 = 10 as int16;
    }
    let v2: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa6db02)) as *const uint8
        as *mut uint16)
        .offset(
            ((*E).ridley_var_07 as ::core::ffi::c_int + ridley_var_10 as ::core::ffi::c_int)
                as isize,
        );
    sub_A6DC13(
        v2,
        (*E).base.x_pos,
        (*E).base.y_pos,
        (*E).ridley_var_0C,
        game,
    );
}
pub unsafe fn sub_A6DB2A(game: &mut Game) {
    if ((*gEnemyData(0 as ::core::ffi::c_int, game)).properties as ::core::ffi::c_int
        & kEnemyProps_Invisible as ::core::ffi::c_int)
        == 0i32
    {
        sub_A6DBC2(
            *(game.g_rom.offset(crate::rtl::lorom_offset(0xa6dcba)) as *const uint8 as *mut uint16)
                .offset(
                    ((*(((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                        .offset(71) as *mut uint16 as *mut uint8)
                        .offset(0) as ::core::ffi::c_int
                        + *(((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                            .offset(81) as *mut uint16 as *mut uint8)
                            .offset(0) as ::core::ffi::c_int
                        + 8 as ::core::ffi::c_int
                        & 0xf0 as ::core::ffi::c_int) as uint8
                        as ::core::ffi::c_int
                        >> 4 as ::core::ffi::c_int) as isize,
                ),
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(82),
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(83),
            game,
        );
        sub_A6DBC2(
            0xdc9e as uint16,
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(72),
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(73),
            game,
        );
        sub_A6DBC2(
            0xdc9e as uint16,
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(62),
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(63),
            game,
        );
        sub_A6DBC2(
            0xdc97 as uint16,
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(52),
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(53),
            game,
        );
        sub_A6DBC2(
            0xdc97 as uint16,
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(42),
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(43),
            game,
        );
        sub_A6DBC2(
            0xdc90 as uint16,
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(32),
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(33),
            game,
        );
        sub_A6DBC2(
            0xdc90 as uint16,
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(22),
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(23),
            game,
        );
    }
}
pub unsafe fn sub_A6DBC2(j: uint16, x: uint16, y: uint16, game: &mut Game) {
    sub_A6DC13(
        j,
        x,
        y,
        (*Get_Ridley(0 as ::core::ffi::c_int, game)).ridley_var_0C,
        game,
    );
}
pub unsafe fn CallBabyMetroidInstr(ea: uint32, k: uint16, game: &mut Game) -> uint16 {
    match ea {
        10928073 => BabyMetroid_Instr_2(k, game),
        10928097 => BabyMetroid_Instr_3(k, game),
        10928114 => BabyMetroid_Instr_1(k, game),
        10928120 => BabyMetroid_Goto(k, game),
        _ => Unreachable(game) as uint16,
    }
}
unsafe fn BabyMetroid_DBCB_DoubleRetEx(a: uint16, game: &mut Game) -> ::core::ffi::c_int {
    let mut current_block: u64;
    let st: *mut BabyMetroidExecState = (&raw mut game.g_ram as *mut uint8).offset(a as isize)
        as *mut uint8 as *mut BabyMetroidExecState;
    if ((*st).ip as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) == 0i32 {
        return -(1 as ::core::ffi::c_int);
    }
    let mut v2: uint16 = (*st).ip;
    let mut v3: *const uint16 = RomPtr_A6(v2, game) as *mut uint16;
    if *v3.offset(0) as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        current_block = 14076424727262896545;
    } else {
        if (*st).timer != *v3.offset(0) {
            (*st).timer = (*st).timer.wrapping_add(1);
            return *v3.offset(1) as ::core::ffi::c_int;
        }
        v2 = (v2).wrapping_add(4u16);
        current_block = 6937071982253665452;
    }
    loop {
        match current_block {
            14076424727262896545 => {
                v2 = CallBabyMetroidInstr(
                    ((*v3.offset(0)) as u32) | (0xa60000u32),
                    (v2).wrapping_add(2u16),
                    game,
                );
                current_block = 6937071982253665452;
            }
            _ => {
                v3 = RomPtr_A6(v2, game) as *mut uint16;
                if *v3.offset(0) as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
                    break;
                } else {
                    current_block = 14076424727262896545;
                }
            }
        }
    }
    (*st).timer = 1 as uint16;
    (*st).ip = v2;
    *v3.offset(1) as ::core::ffi::c_int
}
pub unsafe fn sub_A6DC13(j: uint16, r18: uint16, r20: uint16, r22: uint16, game: &mut Game) {
    let mut p: *const uint8 = RomPtr_A6(j, game);
    let mut n: ::core::ffi::c_int = *(p as *mut uint16) as ::core::ffi::c_int;
    p = p.offset(2);
    let mut idx: ::core::ffi::c_int =
        *((&raw mut game.g_ram as *mut uint8).offset(0x590) as *mut uint16) as ::core::ffi::c_int;
    loop {
        let v9: int16 = (r20 as ::core::ffi::c_int + *p.offset(2) as int8 as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16)
                as ::core::ffi::c_int) as int16;
        if v9 >= 0i16
            && v9 as ::core::ffi::c_int - 224 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != 0
        {
            let x: uint16 = (r18 as ::core::ffi::c_int + *(p as *mut uint16) as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16)
                    as ::core::ffi::c_int) as uint16;
            let v11: *mut OamEnt = gOamEnt(idx, game);
            (*v11).xcoord = x as uint8;
            (*v11).ycoord = v9 as uint8;
            *(&raw mut (*v11).charnum as *mut uint16) = (r22) | (*(p.offset(3) as *mut uint16));
            let ref mut fresh4 = *((&raw mut game.g_ram as *mut uint8).offset(0x570)
                as *mut uint16)
                .offset((idx >> 5 as ::core::ffi::c_int) as isize);
            *fresh4 = (*fresh4 as ::core::ffi::c_int
                | ((x as ::core::ffi::c_int & 0x100 as ::core::ffi::c_int)
                    >> 8 as ::core::ffi::c_int
                    | (*(p as *mut int16) < 0i16) as ::core::ffi::c_int * 2 as ::core::ffi::c_int)
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
pub unsafe fn Ridley_Func_121(game: &mut Game) -> uint8 {
    let mut y_pos: int16 = 0;
    let mut v2: int16 = 0;
    let mut x_pos: int16 = 0;
    let mut v4: int16 = 0;
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    y_pos = (*E).base.y_pos as int16;
    let result: uint8 = 1 as uint8;
    if y_pos >= 0i16 {
        v2 = (y_pos as ::core::ffi::c_int + 32 as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16)
                as ::core::ffi::c_int) as int16;
        if v2 >= 0i16 {
            if v2 as ::core::ffi::c_int - 288 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != 0
            {
                x_pos = (*E).base.x_pos as int16;
                if x_pos >= 0i16 {
                    v4 = (x_pos as ::core::ffi::c_int + 32 as ::core::ffi::c_int
                        - *((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16)
                            as ::core::ffi::c_int) as int16;
                    if v4 >= 0i16 {
                        if v4 as ::core::ffi::c_int - 320 as ::core::ffi::c_int
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
    result
}
pub unsafe fn Ridley_Func_122(rect: Rect16U, game: &mut Game) -> uint16 {
    let mut v2: int16 = 0;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xcce) as *mut uint16) == 0 {
        return 0xffff as uint16;
    }
    let mut result: uint16 = 0 as uint16;
    loop {
        let v1: ::core::ffi::c_int = result as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        v2 = *((&raw mut game.g_ram as *mut uint8).offset(0xc18) as *mut uint16).offset(v1 as isize)
            as int16;
        if v2 < 0i16 {
            if (*(&raw mut v2 as *mut uint8).offset(
                (::core::mem::size_of::<int16>())
                    .wrapping_div(::core::mem::size_of::<uint8>())
                    .wrapping_sub(1_usize) as isize,
            ) as ::core::ffi::c_int
                & 0xf as ::core::ffi::c_int)
                - 3 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
            {
                let v3: uint16 = abs16(
                    (*((&raw mut game.g_ram as *mut uint8).offset(0xb64) as *mut uint16)
                        .offset(v1 as isize))
                    .wrapping_sub(rect.x),
                );
                let mut v4: bool = v3
                    < *((&raw mut game.g_ram as *mut uint8).offset(0xbb4) as *mut uint16)
                        .offset(v1 as isize);
                let v5: uint16 = (v3).wrapping_sub(
                    *((&raw mut game.g_ram as *mut uint8).offset(0xbb4) as *mut uint16)
                        .offset(v1 as isize),
                );
                if v4 || v5 < rect.w {
                    let v6: uint16 = abs16(
                        (*((&raw mut game.g_ram as *mut uint8).offset(0xb78) as *mut uint16)
                            .offset(v1 as isize))
                        .wrapping_sub(rect.y),
                    );
                    v4 = v6
                        < *((&raw mut game.g_ram as *mut uint8).offset(0xbc8) as *mut uint16)
                            .offset(v1 as isize);
                    let v7: uint16 = (v6).wrapping_sub(
                        *((&raw mut game.g_ram as *mut uint8).offset(0xbc8) as *mut uint16)
                            .offset(v1 as isize),
                    );
                    if v4 || v7 < rect.h {
                        break;
                    }
                }
            }
        }
        result = (result).wrapping_add(2u16);
        if ((result) as i16).wrapping_sub(10i16) >= 0i16 {
            return 0xffff as uint16;
        }
    }
    let v8: ::core::ffi::c_int = result as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb64) as *mut uint16).offset(v8 as isize) =
        rect.x;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb78) as *mut uint16).offset(v8 as isize) =
        rect.y;
    let ref mut fresh2 =
        *((&raw mut game.g_ram as *mut uint8).offset(0xc04) as *mut uint16).offset(v8 as isize);
    *fresh2 = (*fresh2) | (0x10u16);
    result
}
pub unsafe fn Ridley_Func_123(j: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v2: uint16 = 0;
    if (*((&raw mut game.g_ram as *mut uint8).offset(0xc04) as *mut uint16).offset(v1 as isize)
        as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int)
        == 7i32
    {
        v2 = 1 as uint16;
    } else if (*((&raw mut game.g_ram as *mut uint8).offset(0xc04) as *mut uint16)
        .offset(v1 as isize) as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int)
        == 2i32
    {
        v2 = 8 as uint16;
    } else {
        v2 = 5 as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0xc04) as *mut uint16).offset(v1 as isize) = v2;
}
pub unsafe fn Ridley_Func_124(rect: Rect16U, game: &mut Game) -> uint8 {
    let v0: uint16 = abs16(
        (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)).wrapping_sub(rect.x),
    );
    let mut v1: bool = v0 < *((&raw mut game.g_ram as *mut uint8).offset(0xafe) as *mut uint16);
    let v2: uint16 =
        (v0).wrapping_sub(*((&raw mut game.g_ram as *mut uint8).offset(0xafe) as *mut uint16));
    if v1 || v2 < rect.w {
        let v3: uint16 = abs16(
            (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16))
                .wrapping_sub(rect.y),
        );
        v1 = v3 < *((&raw mut game.g_ram as *mut uint8).offset(0xb00) as *mut uint16);
        let v4: uint16 =
            (v3).wrapping_sub(*((&raw mut game.g_ram as *mut uint8).offset(0xb00) as *mut uint16));
        if v1 || v4 < rect.h {
            return 1 as uint8;
        }
    }
    0 as uint8
}
pub unsafe fn sub_A6DF59(game: &mut Game) {
    NormalEnemyTouchAiSkipDeathAnim_CurEnemy(game);
}
pub unsafe fn sub_A6DF60(game: &mut Game) {
    Ridley_Func_125(game);
}
pub unsafe fn Ridley_Func_125(game: &mut Game) {
    Ridley_Func_98(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x18a8) as *mut uint16) = 96 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x18aa) as *mut uint16) = 5 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xa54) as *mut uint16) =
        (((*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)) as i16)
            .wrapping_sub(
                ((*Get_Ridley(
                    *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                        as ::core::ffi::c_int,
                    game,
                ))
                .base
                .x_pos) as i16,
            )
            >= 0i16) as uint16;
}
pub unsafe fn Ridley_Shot(game: &mut Game) {
    let mut v0: int16 = 0;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) == 2u16 {
        NormalEnemyShotAiSkipDeathAnim_CurEnemy(game);
    } else {
        v0 = 13 as int16;
        let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
        let flash_timer: uint16 = (*E).base.flash_timer;
        if flash_timer != 0 {
            if flash_timer as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
                v0 = 14 as int16;
            }
        }
        (*E).base.flash_timer = v0 as uint16;
        (*E).ridley_var_0D = (*E).ridley_var_0D.wrapping_add(1);
    };
}
pub unsafe fn Ridley_Powerbomb(game: &mut Game) {
    NormalEnemyPowerBombAiSkipDeathAnim_CurEnemy(game);
}
pub unsafe fn Ridley_Func_126(game: &mut Game) {
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    if (*E).base.health == 0
        && ((*E).ridley_var_01 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) == 0i32
    {
        (*E).ridley_var_01 = -(1 as ::core::ffi::c_int) as uint16;
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
        (*E).ridley_var_A = ((0xa6c538i32) as u16);
    }
}
pub unsafe fn Ridley_Func_127(game: &mut Game) {
    let rect: Rect16U = Rect16U {
        x: *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(82),
        y: *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(83),
        w: 14 as uint16,
        h: 14 as uint16,
    };
    if Ridley_Func_124(rect, game) != 0 {
        let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
        Samus_DealDamage(SuitDamageDivision((*E).ridley_var_1C, game), game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x18a8) as *mut uint16) = 96 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x18aa) as *mut uint16) = 5 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xa54) as *mut uint16) =
            (((*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)) as i16)
                .wrapping_sub(
                    (*((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                        .offset(82)) as i16,
                )
                >= 0i16) as uint16;
    }
}
pub unsafe fn Ridley_Func_128(game: &mut Game) {
    let E: *mut Enemy_Ridley = Get_Ridley(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut r18: uint16 = -(CalculateAngleFromXY(
        (*((&raw mut game.g_ram as *mut uint8).offset(0xb64) as *mut uint16).offset(0))
            .wrapping_sub((*E).base.x_pos),
        (*((&raw mut game.g_ram as *mut uint8).offset(0xb78) as *mut uint16).offset(0))
            .wrapping_sub((*E).base.y_pos),
        game,
    ) as ::core::ffi::c_int) as uint8 as uint16;
    let mut v1: uint16 = (4u16).wrapping_mul(
        *((&raw mut game.g_ram as *mut uint8).offset(0xc2c) as *mut uint16)
            .offset(*((&raw mut game.g_ram as *mut uint8).offset(0x18a6) as *mut uint16) as isize),
    );
    if v1 >= 0x300u16 {
        v1 = 768 as uint16;
    }
    let a: uint16 = v1;
    r18 = Math_MultBySin(v1, r18);
    if (((*E).ridley_var_B as ::core::ffi::c_int ^ r18 as ::core::ffi::c_int)
        & 0x8000 as ::core::ffi::c_int)
        != 0i32
    {
        (*E).ridley_var_B = ((*E).ridley_var_B).wrapping_add(r18);
    }
    r18 = Math_MultByCos(a, r18);
    if (((*E).ridley_var_C as ::core::ffi::c_int ^ r18 as ::core::ffi::c_int)
        & 0x8000 as ::core::ffi::c_int)
        != 0i32
    {
        (*E).ridley_var_C = ((*E).ridley_var_C).wrapping_add(r18);
    }
}
pub unsafe fn Ridley_Func_129(game: &mut Game) {
    if ((*Get_Ridley(0 as ::core::ffi::c_int, game)).base.properties as ::core::ffi::c_int
        & kEnemyProps_Tangible as ::core::ffi::c_int)
        == 0i32
    {
        let mut v0: uint16 = Ridley_Func_122(
            Rect16U {
                x: *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(82),
                y: *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(83),
                w: 14 as uint16,
                h: 14 as uint16,
            },
            game,
        );
        if v0 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
            v0 = Ridley_Func_122(
                Rect16U {
                    x: *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                        .offset(72),
                    y: *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                        .offset(73),
                    w: 10 as uint16,
                    h: 10 as uint16,
                },
                game,
            );
            if v0 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
                return;
            }
        }
        let v1: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        game.eproj_spawn_pt = Point16U {
            x: *((&raw mut game.g_ram as *mut uint8).offset(0xb64) as *mut uint16)
                .offset(v1 as isize),
            y: *((&raw mut game.g_ram as *mut uint8).offset(0xb78) as *mut uint16)
                .offset(v1 as isize),
        };
        let mut v2: uint16 = 12 as uint16;
        if (*(((&raw mut game.g_ram as *mut uint8).offset(0xc18) as *mut uint16).offset(v1 as isize)
            as *mut uint16 as *mut uint8)
            .offset(
                (::core::mem::size_of::<uint16>())
                    .wrapping_div(::core::mem::size_of::<uint8>())
                    .wrapping_sub(1_usize) as isize,
            ) as ::core::ffi::c_int
            & 0xf as ::core::ffi::c_int)
            == 1i32
        {
            QueueSfx1_Max6(0x3d as uint16, game);
            v2 = 6 as uint16;
        }
        SpawnEprojWithRoomGfx(0xe509 as uint16, v2, game);
    }
}
pub unsafe fn Ridley_Instr_5(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    (*Get_Ridley(0 as ::core::ffi::c_int, game)).ridley_var_0F = 89 as uint16;
    QueueSfx2_Max6(0x59 as uint16, game);
    jp
}
pub unsafe fn Ridley_Instr_6(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    (*Get_Ridley(0 as ::core::ffi::c_int, game)).ridley_var_0F = 0 as uint16;
    jp
}
pub unsafe fn Ridley_Instr_10(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) == 2u16
        || *((&raw mut game.g_ram as *mut uint8).offset(0x9c2) as *mut uint16) as ::core::ffi::c_int
            - 30 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
    {
        return jp.offset(1);
    }
    (*Get_Ridley(0 as ::core::ffi::c_int, game)).ridley_var_00 = 8 as uint16;
    *jp.offset(0) as uintptr_t as *const uint16
}
pub unsafe fn Ridley_Instr_4(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    if (*Get_Ridley(0 as ::core::ffi::c_int, game)).ridley_var_1B != 0 {
        *jp.offset(0) as uintptr_t as *const uint16
    } else {
        *jp.offset(1) as uintptr_t as *const uint16
    }
}
pub unsafe fn Ridley_Instr_3(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    if (*Get_Ridley(0 as ::core::ffi::c_int, game)).ridley_var_1B != 0 {
        jp.offset(1)
    } else {
        *jp.offset(0) as uintptr_t as *const uint16
    }
}
pub unsafe fn Ridley_Instr_2(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0xb10) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xb14) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16);
    (*Get_Ridley(0 as ::core::ffi::c_int, game)).ridley_var_1D = *jp.offset(0);
    jp.offset(1)
}
pub unsafe fn Ridley_Instr_1(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    if (*Get_Ridley(0 as ::core::ffi::c_int, game)).ridley_var_10 != 0 {
        *jp.offset(0) as uintptr_t as *const uint16
    } else {
        jp.offset(1)
    }
}
pub unsafe fn Ridley_Instr_14(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    (*E).base.x_pos = ((*E).base.x_pos).wrapping_add(*jp.offset(0));
    (*E).base.y_pos = ((*E).base.y_pos).wrapping_add(*jp.offset(1));
    jp.offset(2)
}
pub unsafe fn Ridley_Instr_9(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    (*Get_Ridley(0 as ::core::ffi::c_int, game)).ridley_var_10 = 0 as uint16;
    Ridley_Func_96(game);
    jp
}
pub unsafe fn Ridley_Instr_7(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    (*Get_Ridley(0 as ::core::ffi::c_int, game)).ridley_var_10 = 1 as uint16;
    jp
}
pub unsafe fn Ridley_Instr_8(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    (*Get_Ridley(0 as ::core::ffi::c_int, game)).ridley_var_10 = 2 as uint16;
    Ridley_Func_96(game);
    jp
}
pub unsafe fn Ridley_Func_131(k: uint16, game: &mut Game) {
    Ridley_Func_132(k, 0 as uint16, game);
    Ridley_Func_132(k, 1 as uint16, game);
    Ridley_Func_132(k, 2 as uint16, game);
    Ridley_Func_132(k, 3 as uint16, game);
}
pub unsafe fn Ridley_Func_132(k: uint16, a: uint16, game: &mut Game) {
    (*Get_Ridley(0 as ::core::ffi::c_int, game)).ridley_parameter_1 = a;
    SpawnEprojWithGfx(a, k, addr_stru_869634 as uint16, game);
}
pub unsafe fn Ridley_Instr_11(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let mut v3: uint16 = 0;
    let current_block: u64;
    let mut r18: uint16 = 0;
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    if (*E).ridley_var_10 != 0 {
        let mut v4: uint16 = -(CalculateAngleFromXY(
            (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
                as ::core::ffi::c_int
                - ((*E).base.x_pos as ::core::ffi::c_int + 25 as ::core::ffi::c_int))
                as uint16,
            (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
                as ::core::ffi::c_int
                - ((*E).base.y_pos as ::core::ffi::c_int - 43 as ::core::ffi::c_int))
                as uint16,
            game,
        ) as ::core::ffi::c_int
            + 0x80 as ::core::ffi::c_int) as uint8 as uint16;
        if v4 < 0x50u16 {
            if v4 >= 0x15u16 {
                current_block = 14379314559172793947;
            } else {
                current_block = 11006700562992250127;
            }
        } else if v4 < 0xc0u16 {
            v4 = 80 as uint16;
            current_block = 14379314559172793947;
        } else {
            current_block = 11006700562992250127;
        }
        match current_block {
            11006700562992250127 => {
                v4 = 21 as uint16;
            }
            _ => {}
        }
        r18 = v4;
    } else {
        v3 = 0;
        v3 = -(CalculateAngleFromXY(
            (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
                as ::core::ffi::c_int
                - ((*E).base.x_pos as ::core::ffi::c_int - 25 as ::core::ffi::c_int))
                as uint16,
            (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
                as ::core::ffi::c_int
                - ((*E).base.y_pos as ::core::ffi::c_int - 43 as ::core::ffi::c_int))
                as uint16,
            game,
        ) as ::core::ffi::c_int
            + 0x80 as ::core::ffi::c_int) as uint8 as uint16;
        if v3 >= 0xb0u16 {
            if v3 < 0xebu16 {
                current_block = 16663838568255336676;
            } else {
                current_block = 11874048045265102773;
            }
        } else if v3 < 0x40u16 {
            current_block = 11874048045265102773;
        } else {
            v3 = 176 as uint16;
            current_block = 16663838568255336676;
        }
        match current_block {
            11874048045265102773 => {
                v3 = 235 as uint16;
            }
            _ => {}
        }
        r18 = v3;
    }
    (*E).ridley_var_19 = Math_MultBySin(0x500 as uint16, r18);
    (*E).ridley_var_1A = Math_MultByCos(0x500 as uint16, r18);
    jp
}
pub unsafe fn Ridley_Instr_12(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    Ridley_E90C(k, jp, 0 as uint16, game)
}
pub unsafe fn Ridley_Instr_13(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    Ridley_E90C(k, jp, 0xe as uint16, game)
}
pub unsafe fn Ridley_E90C(
    k: uint16,
    jp: *const uint16,
    a: uint16,
    game: &mut Game,
) -> *const uint16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1995) as *mut uint16) = a;
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    SpawnEprojWithRoomGfx(0x9642 as uint16, (*E).ridley_var_10, game);
    jp
}
pub unsafe fn Ridley_Instr_15(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    (*E).ridley_var_A = ((0xa6a6afi32) as u16);
    (*E).ridley_var_C = -(352 as ::core::ffi::c_int) as uint16;
    jp
}
pub unsafe fn Ridley_Instr_16(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let E: *mut Enemy_Ridley = Get_Ridley(0 as ::core::ffi::c_int, game);
    (*E).ridley_var_A = ((0xa6b2f3i32) as u16);
    (*E).ridley_var_C = -(352 as ::core::ffi::c_int) as uint16;
    jp
}
pub unsafe fn CeresSteam_Init(game: &mut Game) {
    let E: *mut Enemy_CeresSteam = Get_CeresSteam(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.vram_tiles_index = 0 as uint16;
    (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_DisableSamusColl) as u16);
    (*E).base.extra_properties = ((*E).base.extra_properties) | (4u16);
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
    (*E).base.palette_index = 2560 as uint16;
    (*E).csm_var_D = ((NextRandom(game) as ::core::ffi::c_int & 0x1f as ::core::ffi::c_int)
        + 1 as ::core::ffi::c_int) as uint16;
    let v1: ::core::ffi::c_int = (*E).csm_parameter_1 as ::core::ffi::c_int;
    (*E).base.current_instruction = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa6eff5))
        as *const uint8 as *mut uint16)
        .offset(v1 as isize);
    (*E).csm_var_A = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa6f001)) as *const uint8
        as *mut uint16)
        .offset(v1 as isize);
}
pub unsafe fn CallCeresSteamFunc(ea: uint32, k: uint16, game: &mut Game) {
    match ea {
        10940404 => (),
        10940441 => {
            CeresSteam_Func_1(k, game);
        }
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn CeresSteam_Main(game: &mut Game) {
    let E: *mut Enemy_CeresSteam = Get_CeresSteam(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.health = 0x7fff as uint16;
    CallCeresSteamFunc(
        (((*E).csm_var_A) as u32) | (0xa60000u32),
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
}
pub unsafe fn CeresSteam_Func_1(k: uint16, game: &mut Game) {
    let E: *mut Enemy_CeresSteam = Get_CeresSteam(k as ::core::ffi::c_int, game);
    let pt: Point16U = CalcCeresSteamPos_Mode7(
        Point16U {
            x: (*E).base.x_pos,
            y: (*E).base.y_pos,
        },
        game,
    );
    let v3: *mut EnemySpawnData = gEnemySpawnData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*v3).xpos2 = (pt.x).wrapping_sub((*E).base.x_pos);
    (*v3).ypos2 = (pt.y).wrapping_sub((*E).base.y_pos);
}
pub unsafe fn CeresSteam_Touch(game: &mut Game) {
    (*Get_CeresSteam(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .base
    .health = 0x7fff as uint16;
    NormalEnemyTouchAi(game);
}
pub unsafe fn CeresSteam_Instr_1(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let E: *mut Enemy_CeresSteam = Get_CeresSteam(k as ::core::ffi::c_int, game);
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        | (kEnemyProps_Tangible as ::core::ffi::c_int
            | kEnemyProps_Invisible as ::core::ffi::c_int)) as uint16;
    jp
}
pub unsafe fn CeresSteam_Instr_2(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let E: *mut Enemy_CeresSteam = Get_CeresSteam(k as ::core::ffi::c_int, game);
    let fresh0 = (*E).csm_var_D;
    (*E).csm_var_D = (*E).csm_var_D.wrapping_sub(1);
    if fresh0 != 1u16 {
        return *jp.offset(0) as uintptr_t as *const uint16;
    }
    CeresSteam_Instr_3(k, RomPtr_A6(*jp.offset(1), game) as *mut uint16, game)
}
pub unsafe fn CeresSteam_Instr_3(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let E: *mut Enemy_CeresSteam = Get_CeresSteam(k as ::core::ffi::c_int, game);
    (*E).base.properties = ((*E).base.properties) & (0xfaffu16);
    jp
}
pub unsafe fn CeresDoor_Instr_6(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let E: *mut Enemy_CeresDoor = Get_CeresDoor(k as ::core::ffi::c_int, game);
    if abs16(
        ((*E).base.x_pos)
            .wrapping_sub(*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)),
    ) >= 0x30u16
        || abs16(
            ((*E).base.y_pos)
                .wrapping_sub(*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)),
        ) >= 0x30u16
    {
        *jp as uintptr_t as *const uint16
    } else {
        jp.offset(1)
    }
}
pub unsafe fn CeresDoor_Instr_4(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    if *((&raw mut game.g_ram as *mut uint8)
        .offset(0xd828)
        .offset(*((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) as isize)
        as *mut uint8 as *mut uint16) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        jp.offset(1)
    } else {
        *jp as uintptr_t as *const uint16
    }
}
pub unsafe fn CeresDoor_Instr_8(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x93f) as *mut uint16) != 0 {
        jp.offset(1)
    } else {
        *jp as uintptr_t as *const uint16
    }
}
pub unsafe fn CeresDoor_Func_6b(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x93f) as *mut uint16) != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x93f) as *mut uint16) = 0x8000 as uint16;
    }
}
pub unsafe fn CeresSteam_Instr_4(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let E: *mut Enemy_CeresSteam = Get_CeresSteam(k as ::core::ffi::c_int, game);
    (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
    jp
}
pub unsafe fn CeresDoor_Instr_1(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let E: *mut Enemy_CeresDoor = Get_CeresDoor(k as ::core::ffi::c_int, game);
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        & !(kEnemyProps_Tangible as ::core::ffi::c_int)) as uint16;
    jp
}
pub unsafe fn CeresDoor_Instr_3(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    (*Get_CeresDoor(k as ::core::ffi::c_int, game)).cdr_var_B = 1 as uint16;
    jp
}
pub unsafe fn CeresSteam_Instr_5(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let E: *mut Enemy_CeresSteam = Get_CeresSteam(k as ::core::ffi::c_int, game);
    (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Invisible) as u16);
    jp
}
pub unsafe fn CeresDoor_Instr_5(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    (*Get_CeresDoor(k as ::core::ffi::c_int, game)).cdr_var_B = 0 as uint16;
    CeresDoor_Instr_2(k, jp, game)
}
pub unsafe fn CeresDoor_Instr_2(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let E: *mut Enemy_CeresDoor = Get_CeresDoor(k as ::core::ffi::c_int, game);
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        & !(kEnemyProps_Invisible as ::core::ffi::c_int)) as uint16;
    jp
}
pub unsafe fn CeresDoor_Instr_7(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    QueueSfx3_Max6(0x2c as uint16, game);
    jp
}
pub unsafe fn CeresDoor_Init(game: &mut Game) {
    let E: *mut Enemy_CeresDoor = Get_CeresDoor(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.spritemap_pointer = addr_kCeresDoor_Sprmap_FAC7 as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
    (*E).base.vram_tiles_index = 0 as uint16;
    (*E).base.palette_index = 1024 as uint16;
    let v2: ::core::ffi::c_int = (*E).cdr_parameter_1 as ::core::ffi::c_int;
    (*E).cdr_var_A = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa6f72b)) as *const uint8
        as *mut uint16)
        .offset(v2 as isize);
    (*E).base.current_instruction = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa6f52c))
        as *const uint8 as *mut uint16)
        .offset(v2 as isize);
    (*E).cdr_var_B = 0 as uint16;
    CeresDoor_Func_1(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
    let mut v3: uint16 = 0;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x93f) as *mut uint16) != 0 {
        v3 = (addr_word_A6F50C as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    } else {
        if (*E).cdr_parameter_1 == 3u16 {
            WriteColorsToTargetPalette(
                0xa6 as uint8,
                0x142 as uint16,
                (addr_kCeresDoor_Palette as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16,
                0xf as uint16,
                game,
            );
            return;
        }
        v3 = (addr_kCeresDoor_Palette as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as uint16;
    }
    (*Get_CeresDoor(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .base
    .palette_index = 3584 as uint16;
    WriteColorsToTargetPalette(0xa6 as uint8, 0x1e2 as uint16, v3, 0xf as uint16, game);
}
pub unsafe fn CeresDoor_Func_1(k: uint16, game: &mut Game) {
    let mut v2: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    if (*Get_CeresDoor(k as ::core::ffi::c_int, game)).cdr_parameter_1 == 2u16 {
        let v1: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16);
        v2 = gVramWriteEntry(
            *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        );
        (*v2).size = 1024 as uint16;
        *((&raw mut (*v2).src.addr as *mut uint8).offset(1) as *mut VoidP) =
            -(20480 as ::core::ffi::c_int) as VoidP;
        (*v2).src.addr = -(15360 as ::core::ffi::c_int) as VoidP;
        (*v2).vram_dst = 28672 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) =
            (v1).wrapping_add(7u16);
    }
}
pub unsafe fn CallCeresDoor(ea: uint32, game: &mut Game) {
    let k: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16);
    match ea {
        10942315 => {
            CeresDoor_Func_2(game);
        }
        10942320 => {
            CeresDoor_Func_3(game);
        }
        10942373 => {
            CeresDoor_Func_4(game);
        }
        10942397 => {
            CeresDoor_Func_5(k, game);
        }
        10942428 => {
            CeresDoor_Func_6(k, game);
        }
        10942544 => {
            CeresDoor_Func_7(game);
        }
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn CeresDoor_Main(game: &mut Game) {
    let E: *mut Enemy_CeresDoor = Get_CeresDoor(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    CallCeresDoor((((*E).cdr_var_A) as u32) | (0xa60000u32), game);
}
pub unsafe fn CeresDoor_Func_2(game: &mut Game) {
    CeresDoor_F773(0x14 as uint16, game);
}
pub unsafe fn CeresDoor_Func_3(game: &mut Game) {
    CeresDoor_F773(0x1d as uint16, game);
}
pub unsafe fn CeresDoor_F773(j: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x93f) as *mut uint16) >= 2u16
        && *((&raw mut game.g_ram as *mut uint8).offset(0x1840) as *mut uint16) == 0
    {
        if (*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
            as ::core::ffi::c_int
            & 0xfff as ::core::ffi::c_int)
            < 0x80i32
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1840) as *mut uint16) = 4 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x183e) as *mut uint16) =
                (j).wrapping_add(6u16);
        } else {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1840) as *mut uint16) = 2 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x183e) as *mut uint16) = j;
        }
    }
}
pub unsafe fn CeresDoor_Func_4(game: &mut Game) {
    CeresSteam_Instr_5(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        ::core::ptr::null::<uint16>(),
        game,
    );
    if *((&raw mut game.g_ram as *mut uint8).offset(0x93f) as *mut uint16) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        (*Get_CeresDoor(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        ))
        .base
        .palette_index = 3584 as uint16;
        CeresDoor_Instr_2(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            ::core::ptr::null::<uint16>(),
            game,
        );
    }
}
pub unsafe fn CeresDoor_Func_5(k: uint16, game: &mut Game) {
    CeresDoor_Func_7(game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x93f) as *mut uint16) >= 2u16 {
        let E: *mut Enemy_CeresDoor = Get_CeresDoor(k as ::core::ffi::c_int, game);
        (*E).cdr_var_A = ((0xa6f7dci32) as u16);
        (*E).cdr_var_D = 48 as uint16;
        let E0: *mut Enemy_CeresDoor = Get_CeresDoor(0 as ::core::ffi::c_int, game);
        (*E0).cdr_var_E = 0 as uint16;
        (*E0).cdr_var_F = 0 as uint16;
    }
}
pub unsafe fn CeresDoor_Func_6(k: uint16, game: &mut Game) {
    let E: *mut Enemy_CeresDoor = Get_CeresDoor(k as ::core::ffi::c_int, game);
    (*E).cdr_var_D = (*E).cdr_var_D.wrapping_sub(1);
    let mut v3: bool =
        ((*E).cdr_var_D as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32;
    if v3 {
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Invisible) as u16);
        (*E).cdr_var_A = ((0xa6f850i32) as u16);
        CeresDoor_Func_6b(game);
    } else {
        let E0: *mut Enemy_CeresDoor = Get_CeresDoor(0 as ::core::ffi::c_int, game);
        (*E0).cdr_var_E = (*E0).cdr_var_E.wrapping_sub(1);
        v3 = ((*E0).cdr_var_E as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32;
        if v3 {
            (*E0).cdr_var_E = 4 as uint16;
            (*E0).cdr_var_F = (*E0).cdr_var_F.wrapping_sub(1);
            v3 = ((*E0).cdr_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32;
            if v3 {
                (*E0).cdr_var_F = 3 as uint16;
            }
            let v5: ::core::ffi::c_int = ((4u16).wrapping_mul((*E0).cdr_var_F))
                as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int;
            game.eproj_spawn_pt = Point16U {
                x: (((*E).base.x_pos).wrapping_add(
                    *(game.g_rom.offset(crate::rtl::lorom_offset(0xa6f840)) as *const uint8
                        as *mut uint16)
                        .offset(v5 as isize),
                )),
                y: (((*E).base.y_pos).wrapping_add(
                    *(game.g_rom.offset(crate::rtl::lorom_offset(0xa6f840)) as *const uint8
                        as *mut uint16)
                        .offset((v5 + 1 as ::core::ffi::c_int) as isize),
                )),
            };
            let mut v6: uint16 = 3 as uint16;
            if NextRandom(game) < 0x4000u16 {
                v6 = 12 as uint16;
            }
            SpawnEprojWithRoomGfx(0xe509 as uint16, v6, game);
            QueueSfx2_Max6(0x25 as uint16, game);
        }
    };
}
pub unsafe fn CeresDoor_Func_7(game: &mut Game) {
    CeresDoor_Func_8(game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0xc400) as *mut uint16) == 0 {
        WriteColorsToPalette(
            0x52 as uint16,
            0xa6 as uint8,
            (2 as ::core::ffi::c_int
                * (*((&raw mut game.g_ram as *mut uint8).offset(0x5b6) as *mut uint16)
                    as ::core::ffi::c_int
                    & 0x38 as ::core::ffi::c_int)
                - 0x78f as ::core::ffi::c_int) as uint16,
            6 as uint16,
            game,
        );
    }
}
pub unsafe fn CeresDoor_Func_8(game: &mut Game) {
    QueueMode7Transfers(
        0xa6 as uint8,
        *(game.g_rom.offset(crate::rtl::lorom_offset(0xa6f900)) as *const uint8 as *mut uint16)
            .offset(
                ((*((&raw mut game.g_ram as *mut uint8).offset(0x5b6) as *mut uint16)
                    as ::core::ffi::c_int
                    & 2 as ::core::ffi::c_int)
                    >> 1 as ::core::ffi::c_int) as isize,
            ),
        game,
    );
}
pub unsafe fn Zebetites_Init(game: &mut Game) {
    let E: *mut Enemy_Zebetites = Get_Zebetites(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.properties = ((*E).base.properties) | (0xa000u16);
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
    (*E).base.palette_index = 1024 as uint16;
    (*E).base.vram_tiles_index = 128 as uint16;
    (*E).zebet_var_C = 0 as uint16;
    let mut v1: uint16 = ((0xa6fc41i32) as u16);
    if (*E).zebet_parameter_1 != 0 {
        v1 = ((0xa6fc5bi32) as u16);
    }
    (*E).zebet_var_A = v1;
    let mut r18: uint16 = 0 as uint16;
    r18 = ((r18 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
        | CheckEventHappened(5 as uint16, game) as ::core::ffi::c_int) as uint16;
    r18 = ((r18 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
        | CheckEventHappened(4 as uint16, game) as ::core::ffi::c_int) as uint16;
    r18 = ((r18 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
        | CheckEventHappened(3 as uint16, game) as ::core::ffi::c_int) as uint16;
    let v2: uint16 = r18;
    (*E).zebet_var_D = r18;
    if v2 as ::core::ffi::c_int - 4 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        let v3: ::core::ffi::c_int = v2 as ::core::ffi::c_int;
        (*E).zebet_var_F = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa6fc03)) as *const uint8
            as *mut uint16)
            .offset(v3 as isize);
        (*E).base.y_height =
            *(game.g_rom.offset(crate::rtl::lorom_offset(0xa6fc0b)) as *const uint8 as *mut uint16)
                .offset(v3 as isize);
        (*E).base.current_instruction = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa6fc13))
            as *const uint8 as *mut uint16)
            .offset(v3 as isize);
        (*E).base.x_pos = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa6fc1b)) as *const uint8
            as *mut uint16)
            .offset(v3 as isize);
        let mut v4: uint16 = 0;
        if (*E).zebet_parameter_1 != 0 {
            v4 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa6fc2b)) as *const uint8
                as *mut uint16)
                .offset(v3 as isize);
        } else {
            v4 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa6fc23)) as *const uint8
                as *mut uint16)
                .offset(v3 as isize);
        }
        (*E).base.y_pos = v4;
    } else {
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Deleted) as u16);
    };
}
pub unsafe fn CallZebetitesFunc(ea: uint32, k: uint16, game: &mut Game) {
    match ea {
        10943553 => {
            Zebetites_Func_1(k, game);
        }
        10943579 => {
            Zebetites_Func_2(k, game);
        }
        10943591 => {
            Zebetites_Func_3(game);
        }
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn Zebetites_Main(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1840) as *mut uint16) == 0 {
        (*Get_Zebetites(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        ))
        .base
        .shake_timer = 0 as uint16;
    }
    let E: *mut Enemy_Zebetites = Get_Zebetites(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    CallZebetitesFunc(
        (((*E).zebet_var_A) as u32) | (0xa60000u32),
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
}
pub unsafe fn Zebetites_Func_1(k: uint16, game: &mut Game) {
    let E: *mut Enemy_Zebetites = Get_Zebetites(k as ::core::ffi::c_int, game);
    if ((*E).zebet_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        let new_k: uint16 = Zebetites_Func_9(game);
        (*Get_Zebetites(new_k as ::core::ffi::c_int, game)).zebet_parameter_2 = k;
        (*Get_Zebetites(k as ::core::ffi::c_int, game)).zebet_parameter_2 = new_k;
    }
    (*Get_Zebetites(k as ::core::ffi::c_int, game)).zebet_var_A = ((0xa6fc5bi32) as u16);
    Zebetites_Func_2(k, game);
}
pub unsafe fn Zebetites_Func_2(k: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x795) as *mut uint16) == 0 {
        (*Get_Zebetites(k as ::core::ffi::c_int, game)).zebet_var_A = ((0xa6fc67i32) as u16);
        Zebetites_Func_3(game);
    }
}
pub unsafe fn Zebetites_Func_3(game: &mut Game) {
    let mut v3: int16 = 0;
    Zebetites_Func_4(game);
    Zebetites_Func_5(game);
    let E: *mut Enemy_Zebetites = Get_Zebetites(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let health: uint16 = (*E).base.health;
    if health != 0 {
        v3 = ((health) as i16).wrapping_add(1i16);
        if v3 as ::core::ffi::c_int - 1000 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0
        {
            v3 = 1000 as int16;
        }
        (*E).base.health = v3 as uint16;
    } else if (*E).zebet_parameter_1 != 0u16 || {
        Zebetites_Func_6(game);
        (*E).zebet_var_D as ::core::ffi::c_int - 4 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
    } {
        EnemyDeathAnimation(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            0 as uint16,
            game,
        );
    } else {
        EnemyDeathAnimation(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            0 as uint16,
            game,
        );
        Zebetites_Func_8(game);
    };
}
pub unsafe fn Zebetites_Func_6(game: &mut Game) {
    let mut v1: int16 = 0;
    let E: *mut Enemy_Zebetites = Get_Zebetites(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    v1 = (((*E).zebet_var_D) as i16).wrapping_add(1i16);
    (*E).zebet_var_D = v1 as uint16;
    let r18: uint16 = v1 as uint16;
    Zebetites_Func_7(
        3 as uint16,
        (r18 as ::core::ffi::c_int >> 0 as ::core::ffi::c_int & 1 as ::core::ffi::c_int) as uint8,
        game,
    );
    Zebetites_Func_7(
        4 as uint16,
        (r18 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int & 1 as ::core::ffi::c_int) as uint8,
        game,
    );
    Zebetites_Func_7(
        5 as uint16,
        (r18 as ::core::ffi::c_int >> 2 as ::core::ffi::c_int & 1 as ::core::ffi::c_int) as uint8,
        game,
    );
}
pub unsafe fn Zebetites_Func_7(j: uint16, a: uint8, game: &mut Game) {
    if a != 0 {
        SetEventHappened(j, game);
    } else {
        ClearEventHappened(j, game);
    };
}
pub unsafe fn Zebetites_Func_8(game: &mut Game) {
    SpawnEnemy(0xa6 as uint8, addr_stru_A6FCE1 as uint16, game);
}
pub unsafe fn Zebetites_Func_9(game: &mut Game) -> uint16 {
    SpawnEnemy(0xa6 as uint8, addr_stru_A6FCF9 as uint16, game)
}
pub unsafe fn Zebetites_Func_5(game: &mut Game) {
    let mut v0: uint16 = 0 as uint16;
    let E: *mut Enemy_Zebetites = Get_Zebetites(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let health: uint16 = (*E).base.health;
    if health as ::core::ffi::c_int - 800 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0
    {
        v0 = 2 as uint16;
        if health as ::core::ffi::c_int - 600 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            != 0
        {
            v0 = 4 as uint16;
            if health as ::core::ffi::c_int - 400 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
            {
                v0 = 6 as uint16;
                if health as ::core::ffi::c_int - 200 as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    != 0
                {
                    v0 = 8 as uint16;
                }
            }
        }
    }
    let v3: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v4: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa6fd4a)) as *const uint8
        as *mut uint16)
        .offset(v3 as isize);
    if ((*E).zebet_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        v4 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa6fd54)) as *const uint8
            as *mut uint16)
            .offset(v3 as isize);
    }
    (*E).base.current_instruction = v4;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
}
pub unsafe fn Zebetites_Func_4(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xc400) as *mut uint16) == 0
        && (*Get_Zebetites(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        ))
        .zebet_parameter_1
            == 0
    {
        let E: *mut Enemy_Zebetites = Get_Zebetites(0 as ::core::ffi::c_int, game);
        let v1: uint16 = ((*E).zebet_var_C as ::core::ffi::c_int + 1 as ::core::ffi::c_int
            & 7 as ::core::ffi::c_int) as uint16;
        (*E).zebet_var_C = v1;
        WriteColorsToPalette(
            0x158 as uint16,
            0xa6 as uint8,
            (4 as ::core::ffi::c_int * v1 as ::core::ffi::c_int - 0x279 as ::core::ffi::c_int)
                as uint16,
            2 as uint16,
            game,
        );
    }
}
pub unsafe fn Zebetites_Touch(game: &mut Game) {
    NormalEnemyTouchAiSkipDeathAnim_CurEnemy(game);
}
pub unsafe fn Zebetites_Shot(game: &mut Game) {
    QueueSfx3_Max6(9 as uint16, game);
    NormalEnemyShotAiSkipDeathAnim_CurEnemy(game);
    let E: *mut Enemy_Zebetites = Get_Zebetites(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let G: *mut Enemy_Zebetites = Get_Zebetites((*E).zebet_parameter_2 as ::core::ffi::c_int, game);
    (*G).base.health = (*E).base.health;
    (*G).base.flash_timer = (*E).base.flash_timer;
}
