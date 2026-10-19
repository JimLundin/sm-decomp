use crate::cpu_infra::Call;
pub use crate::game::constants::bank_86::*;
pub use crate::game::constants::bank_b4::*;
pub use crate::game::constants::enemy_properties::*;
use crate::game::enemy_layouts::enemy_layout;
use crate::game::sm_80::{
    Multiply16x16, NextRandom, QueueSfx1_Max6, QueueSfx2_Max3, QueueSfx2_Max6, QueueSfx3_Max6,
};
use crate::game::sm_86::SpawnEprojWithGfx;
use crate::game::sm_90::{CallSomeSamusCode, ProjectileReflection, ResetProjectileData};
use crate::game::sm_91::Samus_DealDamage;
use crate::game::sm_a0::{
    Abs16, CalculateAngleOfSamusFromEnemy, CalculateBlockContainingPixelPos, CallEnemyPreInstr,
    CheckIfEnemyIsOnScreen, CheckIfEnemyTouchesSamus, ConvertAngleToXy, CosineMult8bit,
    CosineMult8bitFull, DetermineDirectionOfSamusFromEnemy, EnemyDeathAnimation, EnemyFunc_B691,
    EnemyFunc_BBBF, EnemyFunc_BC76, EnemyFunc_BF8A, EnemyFunc_C8AD, EnemyGrappleDeath,
    Enemy_ItemDrop_Metroid, Enemy_MoveDown, Enemy_MoveRight_IgnoreSlopes,
    Enemy_MoveRight_ProcessSlopes, Enemy_SwitchToFrozenAi, GetSamusEnemyDelta_X,
    GetSamusEnemyDelta_Y, IsSamusWithinEnemy_X, IsSamusWithinEnemy_Y, NormalEnemyFrozenAI,
    NormalEnemyPowerBombAi, NormalEnemyShotAi, NormalEnemyShotAiSkipDeathAnim_CurEnemy,
    NormalEnemyTouchAi, SignExtend8, SineMult8bit, SineMult8bitFull, SwitchEnemyAiToMainAi,
};
use crate::game::sm_b4::CreateSpriteAtPos;
pub use crate::game::waver::{Enemy_Waver, Waver_Func_1, Waver_Init, Waver_Instr_1, Waver_Main};
use crate::game_state::Game;
use crate::ram_map::{ram_word, set_ram_word};
use crate::rtl::abs16;
use crate::rtl::{AddToHiLo, SetHiLo};
use crate::rtl::{RomPtr, SnesDivide, Unreachable};
extern "C" {
    fn printf(__format: *const ::core::ffi::c_char, ...) -> ::core::ffi::c_int;

}
pub use crate::types::{
    __int16_t, __int32_t, __int8_t, __uint16_t, __uint32_t, __uint8_t, int16, int16_t, int32,
    int32_t, int8, int8_t, uint16, uint16_t, uint32, uint32_t, uint8, uint8_t, EnemyBase,
    EnemyData, EnemyProps, EnemySpawnData, Func_V, Func_Y_V, LongPtr, VoidP,
};
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct MaridiaSnailData2 {
    pub field_0: VoidP,
    pub field_2: uint16,
    pub field_4: VoidP,
    pub field_6: uint16,
}
pub use crate::types::Consts_A3;
pub use crate::types::{Consts_86, EnemyDef};
pub const addr_kMetroid_Ilist_EA25: Consts_A3 = 59941;
pub const addr_kMetroid_Ilist_E9CF: Consts_A3 = 59855;
pub const addr_locret_A3E08A: Consts_A3 = 57482;
pub const addr_kWreckedShipOrangeZoomer_Ilist_E01F: Consts_A3 = 57375;
pub const addr_kWreckedShipOrangeZoomer_Ilist_E003: Consts_A3 = 57347;
pub const addr_kWreckedShipOrangeZoomer_Ilist_DFE7: Consts_A3 = 57319;
pub const addr_kWreckedShipOrangeZoomer_Ilist_DFCB: Consts_A3 = 57291;
pub const addr_stru_A3CD3A: Consts_A3 = 52538;
pub const addr_stru_A3CD32: Consts_A3 = 52530;
pub const addr_stru_A3CD2A: Consts_A3 = 52522;
pub const addr_stru_A3CD22: Consts_A3 = 52514;
pub const addr_stru_A3CD1A: Consts_A3 = 52506;
pub const addr_stru_A3CD12: Consts_A3 = 52498;
pub const addr_stru_A3CD0A: Consts_A3 = 52490;
pub const addr_stru_A3CD02: Consts_A3 = 52482;
pub const addr_stru_A3CCFA: Consts_A3 = 52474;
pub const addr_stru_A3CCF2: Consts_A3 = 52466;
pub const addr_stru_A3CCEA: Consts_A3 = 52458;
pub const addr_stru_A3CCE2: Consts_A3 = 52450;
pub const addr_kSkree_Ilist_C65E: Consts_A3 = 50782;
pub const addr_kBang_Ilist_B75E: Consts_A3 = 46942;
pub const addr_kMaridiaRefillCandy_Ilist_B3C1: Consts_A3 = 46017;
pub const addr_kMochtroid_Ilist_A759: Consts_A3 = 42841;
pub const addr_kMochtroid_Ilist_A745: Consts_A3 = 42821;
pub const addr_kPlatformThatFallsWithSamus_Sprmap_A015: Consts_A3 = 40981;
pub const addr_kPlatformThatFallsWithSamus_Sprmap_A009: Consts_A3 = 40969;
pub const addr_kPlatformThatFallsWithSamus_Ilist_9C55: Consts_A3 = 40021;
pub const addr_kPlatformThatFallsWithSamus_Ilist_9C3F: Consts_A3 = 39999;
pub const addr_kPlatformThatFallsWithSamus_Ilist_9C29: Consts_A3 = 39977;
pub const addr_kPlatformThatFallsWithSamus_Ilist_9C13: Consts_A3 = 39955;
pub const addr_kPlatformThatFallsWithSamus_Ilist_9BFD: Consts_A3 = 39933;
pub const addr_kPlatformThatFallsWithSamus_Ilist_9BE7: Consts_A3 = 39911;
pub const addr_kPlatformThatFallsWithSamus_Ilist_9BD1: Consts_A3 = 39889;
pub const addr_kPlatformThatFallsWithSamus_Ilist_9BBB: Consts_A3 = 39867;
pub const addr_kElevator_Ilist_94D6: Consts_A3 = 38102;
pub const addr_kMaridiaFish_Ilist_9072: Consts_A3 = 36978;
pub const addr_kMaridiaFish_Ilist_9060: Consts_A3 = 36960;
pub const addr_kMaridiaFish_Ilist_903C: Consts_A3 = 36924;
pub const addr_kMaridiaFish_Ilist_902A: Consts_A3 = 36906;
pub const addr_kFireflea_Ilist_8C2F: Consts_A3 = 35887;
pub const addr_kMetalee_Ilist_8910: Consts_A3 = 35088;
pub const addr_kWaver_Ilist_86A7: Consts_A3 = 34471;
pub const addr_kSpritemap_Nothing_A3: Consts_A3 = 32845;
pub use crate::types::Consts_B4;
pub use crate::types::Point32;
enemy_layout! {
    Enemy_Bang {
        base: EnemyBase,
        words: [
            bang_var_A, bang_var_B, bang_var_C, bang_var_D, bang_var_E, bang_var_F,
            bang_parameter_1, bang_parameter_2,
        ],
        pad1: [uint8; 26696],
        words: [
            bang_var_00, bang_var_01, bang_var_02, bang_var_03, bang_var_04, bang_var_05,
            bang_var_06, bang_var_07, bang_var_08, bang_var_09, bang_var_0A, bang_var_0B,
            bang_var_0C, bang_var_0D, bang_var_0E, bang_var_0F, bang_var_10, bang_var_11,
            bang_var_12, bang_var_13, bang_var_14, bang_var_15, bang_var_16, bang_var_17,
            bang_var_18, bang_var_19, bang_var_1A, bang_var_1B, bang_var_1C, bang_var_1D,
            bang_var_1E, bang_var_1F,
        ],
        pad2: [uint8; 1984],
        words: [
            bang_var_20, bang_var_21, bang_var_22, bang_var_23, bang_var_24, bang_var_25,
            bang_var_26, bang_var_27, bang_var_28, bang_var_29, bang_var_2A, bang_var_2B,
            bang_var_2C, bang_var_2D, bang_var_2E, bang_var_2F, bang_var_30, bang_var_31,
            bang_var_32, bang_var_33, bang_var_34, bang_var_35, bang_var_36, bang_var_37,
            bang_var_38, bang_var_39, bang_var_3A, bang_var_3B, bang_var_3C, bang_var_3D,
            bang_var_3E, bang_var_3F,
        ],
    }
}

enemy_layout! {
    Enemy_MaridiaFish {
        base: EnemyBase,
        words: [
            mfh_var_A, mfh_var_B, mfh_var_C, mfh_var_D, mfh_var_E, mfh_var_F,
            mfh_parameter_1, mfh_parameter_2,
        ],
        pad1: [uint8; 26696],
        words: [
            mfh_var_00, mfh_var_01, mfh_var_02, mfh_var_03, mfh_var_04, mfh_var_05,
            mfh_var_06, mfh_var_07, mfh_var_08, mfh_var_09, mfh_var_0A, mfh_var_0B,
            mfh_var_0C, mfh_var_0D, mfh_var_0E, mfh_var_0F, mfh_var_10, mfh_var_11,
            mfh_var_12, mfh_var_13, mfh_var_14, mfh_var_15, mfh_var_16, mfh_var_17,
            mfh_var_18, mfh_var_19, mfh_var_1A, mfh_var_1B, mfh_var_1C, mfh_var_1D,
            mfh_var_1E, mfh_var_1F,
        ],
    }
}

enemy_layout! {
    Enemy_MaridiaRefillCandy {
        base: EnemyBase,
        words: [
            mrcy_var_A, mrcy_var_B, mrcy_var_C, mrcy_var_D, mrcy_var_E, mrcy_var_F,
            mrcy_parameter_1, mrcy_parameter_2,
        ],
        pad1: [uint8; 26696],
        words: [
            mrcy_var_00, mrcy_var_01, mrcy_var_02, mrcy_var_03, mrcy_var_04, mrcy_var_05,
            mrcy_var_06, mrcy_var_07, mrcy_var_08, mrcy_var_09, mrcy_var_0A, mrcy_var_0B,
            mrcy_var_0C, mrcy_var_0D, mrcy_var_0E, mrcy_var_0F, mrcy_var_10, mrcy_var_11,
            mrcy_var_12, mrcy_var_13, mrcy_var_14, mrcy_var_15, mrcy_var_16, mrcy_var_17,
            mrcy_var_18, mrcy_var_19, mrcy_var_1A, mrcy_var_1B, mrcy_var_1C, mrcy_var_1D,
            mrcy_var_1E, mrcy_var_1F,
        ],
    }
}

enemy_layout! {
    Enemy_MaridiaSnail {
        base: EnemyBase,
        words: [
            msl_var_A, msl_var_B, msl_var_C, msl_var_D, msl_var_E, msl_var_F,
            msl_parameter_1, msl_parameter_2,
        ],
        pad1: [uint8; 26696],
        words: [
            msl_var_00, msl_var_01, msl_var_02, msl_var_03, msl_var_04, msl_var_05,
            msl_var_06, msl_var_07, msl_var_08, msl_var_09, msl_var_0A, msl_var_0B,
            msl_var_0C, msl_var_0D, msl_var_0E, msl_var_0F, msl_var_10, msl_var_11,
            msl_var_12, msl_var_13, msl_var_14, msl_var_15, msl_var_16, msl_var_17,
            msl_var_18, msl_var_19, msl_var_1A, msl_var_1B, msl_var_1C, msl_var_1D,
            msl_var_1E, msl_var_1F,
        ],
        pad2: [uint8; 1984],
        words: [
            msl_var_20, msl_var_21, msl_var_22, msl_var_23, msl_var_24, msl_var_25,
            msl_var_26, msl_var_27, msl_var_28, msl_var_29, msl_var_2A, msl_var_2B,
            msl_var_2C, msl_var_2D, msl_var_2E, msl_var_2F, msl_var_30, msl_var_31,
            msl_var_32, msl_var_33, msl_var_34, msl_var_35, msl_var_36, msl_var_37,
            msl_var_38, msl_var_39, msl_var_3A, msl_var_3B, msl_var_3C, msl_var_3D,
            msl_var_3E, msl_var_3F,
        ],
    }
}

enemy_layout! {
    Enemy_Metalee {
        base: EnemyBase,
        words: [
            metalee_var_A, metalee_var_B, metalee_var_C, metalee_var_D, metalee_var_E,
            metalee_var_F, metalee_parameter_1, metalee_parameter_2,
        ],
    }
}

enemy_layout! {
    Enemy_PlatformThatFallsWithSamus {
        base: EnemyBase,
        words: [
            ptfwss_var_A, ptfwss_var_B, ptfwss_var_C, ptfwss_var_D, ptfwss_var_E,
            ptfwss_var_F, ptfwss_parameter_1, ptfwss_parameter_2,
        ],
        pad1: [uint8; 26696],
        words: [
            ptfwss_var_00, ptfwss_var_01, ptfwss_var_02, ptfwss_var_03, ptfwss_var_04,
            ptfwss_var_05, ptfwss_var_06, ptfwss_var_07, ptfwss_var_08, ptfwss_var_09,
            ptfwss_var_0A, ptfwss_var_0B, ptfwss_var_0C, ptfwss_var_0D, ptfwss_var_0E,
            ptfwss_var_0F, ptfwss_var_10, ptfwss_var_11, ptfwss_var_12, ptfwss_var_13,
            ptfwss_var_14, ptfwss_var_15, ptfwss_var_16, ptfwss_var_17, ptfwss_var_18,
            ptfwss_var_19, ptfwss_var_1A, ptfwss_var_1B, ptfwss_var_1C, ptfwss_var_1D,
            ptfwss_var_1E, ptfwss_var_1F,
        ],
    }
}

enemy_layout! {
    Enemy_Reflec {
        base: EnemyBase,
        words: [
            reflec_var_A, reflec_var_B, reflec_var_C, reflec_var_D, reflec_var_E,
            reflec_var_F, reflec_parameter_1, reflec_parameter_2,
        ],
    }
}

enemy_layout! {
    Enemy_Sidehopper {
        base: EnemyBase,
        words: [
            sideh_var_A, sideh_var_B, sideh_var_C, sideh_var_D, sideh_var_E, sideh_var_F,
            sideh_parameter_1, sideh_parameter_2,
        ],
        pad1: [uint8; 26696],
        words: [
            sideh_var_00, sideh_var_01, sideh_var_02, sideh_var_03, sideh_var_04,
            sideh_var_05, sideh_var_06, sideh_var_07, sideh_var_08, sideh_var_09,
            sideh_var_0A, sideh_var_0B, sideh_var_0C, sideh_var_0D, sideh_var_0E,
            sideh_var_0F, sideh_var_10, sideh_var_11, sideh_var_12, sideh_var_13,
            sideh_var_14, sideh_var_15, sideh_var_16, sideh_var_17, sideh_var_18,
            sideh_var_19, sideh_var_1A, sideh_var_1B, sideh_var_1C, sideh_var_1D,
            sideh_var_1E, sideh_var_1F,
        ],
    }
}

enemy_layout! {
    Enemy_Skree {
        base: EnemyBase,
        words: [
            skree_var_A, skree_var_B, skree_var_C, skree_var_D, skree_var_E, skree_var_F,
            skree_parameter_1, skree_parameter_2,
        ],
        pad1: [uint8; 26696],
        words: [
            skree_var_00, skree_var_01, skree_var_02, skree_var_03, skree_var_04,
            skree_var_05, skree_var_06, skree_var_07, skree_var_08, skree_var_09,
            skree_var_0A, skree_var_0B, skree_var_0C, skree_var_0D, skree_var_0E,
            skree_var_0F, skree_var_10, skree_var_11, skree_var_12, skree_var_13,
            skree_var_14, skree_var_15, skree_var_16, skree_var_17, skree_var_18,
            skree_var_19, skree_var_1A, skree_var_1B, skree_var_1C, skree_var_1D,
            skree_var_1E, skree_var_1F,
        ],
    }
}

enemy_layout! {
    Enemy_WreckedShipOrangeZoomer {
        base: EnemyBase,
        words: [
            wsozr_var_A, wsozr_var_B, wsozr_var_C, wsozr_var_D, wsozr_var_E, wsozr_var_F,
            wsozr_parameter_1, wsozr_parameter_2,
        ],
        pad1: [uint8; 26696],
        words: [
            wsozr_var_00, wsozr_var_01, wsozr_var_02, wsozr_var_03, wsozr_var_04,
            wsozr_var_05, wsozr_var_06, wsozr_var_07, wsozr_var_08, wsozr_var_09,
            wsozr_var_0A, wsozr_var_0B, wsozr_var_0C, wsozr_var_0D, wsozr_var_0E,
            wsozr_var_0F, wsozr_var_10, wsozr_var_11, wsozr_var_12, wsozr_var_13,
            wsozr_var_14, wsozr_var_15, wsozr_var_16, wsozr_var_17, wsozr_var_18,
            wsozr_var_19, wsozr_var_1A, wsozr_var_1B, wsozr_var_1C, wsozr_var_1D,
            wsozr_var_1E, wsozr_var_1F,
        ],
    }
}

enemy_layout! {
    Enemy_StoneZoomer {
        base: EnemyBase,
        words: [
            szr_var_A, szr_var_B, szr_var_C, szr_var_D, szr_var_E, szr_var_F,
            szr_parameter_1, szr_parameter_2,
        ],
    }
}

enemy_layout! {
    Enemy_Elevator {
        base: EnemyBase,
        words: [
            elevat_var_A, elevat_var_B, elevat_var_C, elevat_var_D, elevat_var_E,
            elevat_var_F, elevat_parameter_1, elevat_parameter_2,
        ],
    }
}

enemy_layout! {
    Enemy_FireZoomer {
        base: EnemyBase,
        words: [
            fzr_var_A, fzr_var_B, fzr_var_C, fzr_var_D, fzr_var_E, fzr_var_F,
            fzr_parameter_1, fzr_parameter_2,
        ],
        pad1: [uint8; 26696],
        words: [
            fzr_var_00, fzr_var_01, fzr_var_02, fzr_var_03, fzr_var_04, fzr_var_05,
            fzr_var_06, fzr_var_07, fzr_var_08, fzr_var_09, fzr_var_0A, fzr_var_0B,
            fzr_var_0C, fzr_var_0D, fzr_var_0E, fzr_var_0F, fzr_var_10, fzr_var_11,
            fzr_var_12, fzr_var_13, fzr_var_14, fzr_var_15, fzr_var_16, fzr_var_17,
            fzr_var_18, fzr_var_19, fzr_var_1A, fzr_var_1B, fzr_var_1C, fzr_var_1D,
            fzr_var_1E, fzr_var_1F,
        ],
    }
}

enemy_layout! {
    Enemy_FireFlea {
        base: EnemyBase,
        words: [
            ffa_var_A, ffa_var_B, ffa_var_C, ffa_var_D, ffa_var_E, ffa_var_F,
            ffa_parameter_1, ffa_parameter_2,
        ],
        pad1: [uint8; 26696],
        words: [
            ffa_var_00, ffa_var_01, ffa_var_02, ffa_var_03, ffa_var_04, ffa_var_05,
            ffa_var_06, ffa_var_07, ffa_var_08, ffa_var_09, ffa_var_0A, ffa_var_0B,
            ffa_var_0C, ffa_var_0D, ffa_var_0E, ffa_var_0F, ffa_var_10, ffa_var_11,
            ffa_var_12, ffa_var_13, ffa_var_14, ffa_var_15, ffa_var_16, ffa_var_17,
            ffa_var_18, ffa_var_19, ffa_var_1A, ffa_var_1B, ffa_var_1C, ffa_var_1D,
            ffa_var_1E, ffa_var_1F,
        ],
    }
}

enemy_layout! {
    Enemy_Metroid {
        base: EnemyBase,
        words: [
            metroid_var_A, metroid_var_B, metroid_var_C, metroid_var_D, metroid_var_E,
            metroid_var_F, metroid_parameter_1, metroid_parameter_2,
        ],
        pad1: [uint8; 26696],
        words: [
            metroid_var_00, metroid_var_01, metroid_var_02, metroid_var_03, metroid_var_04,
            metroid_var_05, metroid_var_06, metroid_var_07, metroid_var_08, metroid_var_09,
            metroid_var_0A, metroid_var_0B, metroid_var_0C, metroid_var_0D, metroid_var_0E,
            metroid_var_0F, metroid_var_10, metroid_var_11, metroid_var_12, metroid_var_13,
            metroid_var_14, metroid_var_15, metroid_var_16, metroid_var_17, metroid_var_18,
            metroid_var_19, metroid_var_1A, metroid_var_1B, metroid_var_1C, metroid_var_1D,
            metroid_var_1E, metroid_var_1F,
        ],
    }
}

enemy_layout! {
    Enemy_Mochtroid {
        base: EnemyBase,
        words: [
            mochtr_var_A, mochtr_var_B, mochtr_var_C, mochtr_var_D, mochtr_var_E,
            mochtr_var_F, mochtr_parameter_1, mochtr_parameter_2,
        ],
        pad1: [uint8; 26696],
        words: [
            mochtr_var_00, mochtr_var_01, mochtr_var_02, mochtr_var_03, mochtr_var_04,
            mochtr_var_05, mochtr_var_06, mochtr_var_07, mochtr_var_08, mochtr_var_09,
            mochtr_var_0A, mochtr_var_0B, mochtr_var_0C, mochtr_var_0D, mochtr_var_0E,
            mochtr_var_0F, mochtr_var_10, mochtr_var_11, mochtr_var_12, mochtr_var_13,
            mochtr_var_14, mochtr_var_15, mochtr_var_16, mochtr_var_17, mochtr_var_18,
            mochtr_var_19, mochtr_var_1A, mochtr_var_1B, mochtr_var_1C, mochtr_var_1D,
            mochtr_var_1E, mochtr_var_1F,
        ],
        pad2: [uint8; 1984],
        words: [
            mochtr_var_20, mochtr_var_21, mochtr_var_22, mochtr_var_23, mochtr_var_24,
            mochtr_var_25, mochtr_var_26, mochtr_var_27, mochtr_var_28, mochtr_var_29,
            mochtr_var_2A, mochtr_var_2B, mochtr_var_2C, mochtr_var_2D, mochtr_var_2E,
            mochtr_var_2F, mochtr_var_30, mochtr_var_31, mochtr_var_32, mochtr_var_33,
            mochtr_var_34, mochtr_var_35, mochtr_var_36, mochtr_var_37, mochtr_var_38,
            mochtr_var_39, mochtr_var_3A, mochtr_var_3B, mochtr_var_3C, mochtr_var_3D,
            mochtr_var_3E, mochtr_var_3F,
        ],
    }
}

enemy_layout! {
    Enemy_Roach {
        base: EnemyBase,
        words: [
            roach_var_A, roach_var_B, roach_var_C, roach_var_D, roach_var_E, roach_var_F,
            roach_parameter_1, roach_parameter_2,
        ],
        pad1: [uint8; 26696],
        words: [
            roach_var_00, roach_var_01, roach_var_02, roach_var_03, roach_var_04,
            roach_var_05, roach_var_06, roach_var_07, roach_var_08, roach_var_09,
            roach_var_0A, roach_var_0B, roach_var_0C, roach_var_0D, roach_var_0E,
            roach_var_0F, roach_var_10, roach_var_11, roach_var_12, roach_var_13,
            roach_var_14, roach_var_15, roach_var_16, roach_var_17, roach_var_18,
            roach_var_19, roach_var_1A, roach_var_1B, roach_var_1C, roach_var_1D,
            roach_var_1E, roach_var_1F,
        ],
        pad2: [uint8; 1984],
        words: [
            roach_var_20, roach_var_21, roach_var_22, roach_var_23, roach_var_24,
            roach_var_25, roach_var_26, roach_var_27, roach_var_28, roach_var_29,
            roach_var_2A, roach_var_2B, roach_var_2C, roach_var_2D, roach_var_2E,
            roach_var_2F, roach_var_30, roach_var_31, roach_var_32, roach_var_33,
            roach_var_34, roach_var_35, roach_var_36, roach_var_37, roach_var_38,
            roach_var_39, roach_var_3A, roach_var_3B, roach_var_3C, roach_var_3D,
            roach_var_3E, roach_var_3F,
        ],
    }
}

#[inline]
unsafe fn RomPtr_A3(addr: uint16_t, game: &mut Game) -> *const uint8 {
    RomPtr((0xa30000u32) | ((addr) as u32), game)
}
#[inline]
unsafe fn get_EnemyDef_A2(a: uint16, game: &mut Game) -> *mut EnemyDef {
    RomPtr((0xa00000u32) | ((a) as u32), game) as *mut EnemyDef
}
pub const fnMetalee_Func_1: ::core::ffi::c_int = 0xa38987;
pub const fnMetalee_Func_3: ::core::ffi::c_int = 0xa389d4;
pub const fnMetalee_Func_4: ::core::ffi::c_int = 0xa389f3;
pub const fnMetalee_Func_5: ::core::ffi::c_int = 0xa38a5c;
pub const fnMaridiaFish_Func_1: ::core::ffi::c_int = 0xa39132;
pub const fnMaridiaFish_Func_2: ::core::ffi::c_int = 0xa391ab;
pub const fnMaridiaFish_Func_3: ::core::ffi::c_int = 0xa39224;
pub const fnMaridiaFish_Func_4: ::core::ffi::c_int = 0xa39256;
pub const fnRoach_Func_9: ::core::ffi::c_int = 0xa3a2d7;
pub const fnRoach_Func_10: uint32 = 10724097;
pub const fnRoach_Func_11: uint32 = 10724107;
pub const fnRoach_Func_12: uint32 = 10724117;
pub const fnRoach_Func_13: uint32 = 10724133;
pub const fnRoach_Func_14: uint32 = 10724155;
pub const fnRoach_Func_15: uint32 = 10724171;
pub const fnRoach_Func_16: uint32 = 10724224;
pub const fnRoach_Func_19: ::core::ffi::c_int = 0xa3a407;
pub const fnRoach_Func_20: ::core::ffi::c_int = 0xa3a40e;
pub const fnRoach_Func_21: ::core::ffi::c_int = 0xa3a440;
pub const fnRoach_Func_22: ::core::ffi::c_int = 0xa3a447;
pub const fnRoach_Func_23: ::core::ffi::c_int = 0xa3a44e;
pub const fnRoach_Func_24: ::core::ffi::c_int = 0xa3a462;
pub const fnRoach_Func_25: ::core::ffi::c_int = 0xa3a476;
pub const fnRoach_Func_26: ::core::ffi::c_int = 0xa3a4b6;
pub const fnRoach_Func_27: ::core::ffi::c_int = 0xa3a4f0;
pub const fnSidehopper_Func_4: ::core::ffi::c_int = 0xa3abd6;
pub const fnSidehopper_Func_5: uint32 = 10726374;
pub const fnSidehopper_Func_6: uint32 = 10726419;
pub const fnSidehopper_Func_7: ::core::ffi::c_int = 0xa3ac40;
pub const fnSidehopper_Func_8: ::core::ffi::c_int = 0xa3ac56;
pub const fnSidehopper_Func_9: ::core::ffi::c_int = 0xa3ac6c;
pub const fnSidehopper_Func_10: ::core::ffi::c_int = 0xa3ac8f;
pub const fnSidehopper_Func_11: ::core::ffi::c_int = 0xa3aca8;
pub const fnSidehopper_Func_12: ::core::ffi::c_int = 0xa3accb;
pub const fnSidehopper_Func_13: ::core::ffi::c_int = 0xa3ace4;
pub const fnSidehopper_Func_14: ::core::ffi::c_int = 0xa3ad0e;
pub const fnSidehopper_Func_15: ::core::ffi::c_int = 0xa3ad20;
pub const fnSidehopper_Func_16: ::core::ffi::c_int = 0xa3ad32;
pub const fnSidehopper_Func_17: ::core::ffi::c_int = 0xa3ad44;
pub const fnSidehopper_Func_18: ::core::ffi::c_int = 0xa3ad56;
pub const fnMaridiaRefillCandy_Func_1: ::core::ffi::c_int = 0xa3b482;
pub const fnMaridiaRefillCandy_Func_2: ::core::ffi::c_int = 0xa3b4a8;
pub const fnMaridiaRefillCandy_Func_3: ::core::ffi::c_int = 0xa3b4d6;
pub const fnBang_Func_6: ::core::ffi::c_int = 0xa3bca5;
pub const fnBang_Func_7: ::core::ffi::c_int = 0xa3bcc1;
pub const fnBang_Func_8: ::core::ffi::c_int = 0xa3bcc5;
pub const fnBang_Func_10: ::core::ffi::c_int = 0xa3bd1c;
pub const fnBang_Func_11: ::core::ffi::c_int = 0xa3bd2c;
pub const fnSkree_Func_1: ::core::ffi::c_int = 0xa3c6d5;
pub const fnSkree_Func_2: ::core::ffi::c_int = 0xa3c6f7;
pub const fnSkree_Func_3: ::core::ffi::c_int = 0xa3c716;
pub const fnSkree_Func_4: ::core::ffi::c_int = 0xa3c77f;
pub const fnnullsub_215: ::core::ffi::c_int = 0xa3cf5f;
pub const fnMaridiaSnail_Func_15: ::core::ffi::c_int = 0xa3d1b3;
pub const fnReflec_Func_1: ::core::ffi::c_int = 0xa3db0c;
pub const fnnullsub_304: ::core::ffi::c_int = 0xa3e6c1;
pub const fnFireZoomer_Func_1: ::core::ffi::c_int = 0xa3e6c8;
pub const fnFireZoomer_Func_2: ::core::ffi::c_int = 0xa3e785;
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
unsafe fn Get_Bang(o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_Bang {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_Bang
}
#[inline]
unsafe fn Get_Elevator(o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_Elevator {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_Elevator
}
#[inline]
unsafe fn Get_FireFlea(o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_FireFlea {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_FireFlea
}
#[inline]
unsafe fn Get_FireZoomer(o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_FireZoomer {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_FireZoomer
}
#[inline]
unsafe fn Get_MaridiaFish(o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_MaridiaFish {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_MaridiaFish
}
#[inline]
unsafe fn Get_MaridiaRefillCandy(
    o: ::core::ffi::c_int,
    game: &mut Game,
) -> *mut Enemy_MaridiaRefillCandy {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_MaridiaRefillCandy
}
#[inline]
unsafe fn Get_MaridiaSnail(o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_MaridiaSnail {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_MaridiaSnail
}
#[inline]
unsafe fn Get_Metalee(o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_Metalee {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_Metalee
}
#[inline]
unsafe fn Get_Metroid(o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_Metroid {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_Metroid
}
#[inline]
unsafe fn Get_Mochtroid(o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_Mochtroid {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_Mochtroid
}
#[inline]
unsafe fn Get_PlatformThatFallsWithSamus(
    o: ::core::ffi::c_int,
    game: &mut Game,
) -> *mut Enemy_PlatformThatFallsWithSamus {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_PlatformThatFallsWithSamus
}
#[inline]
unsafe fn Get_Reflec(o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_Reflec {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_Reflec
}
#[inline]
unsafe fn Get_Roach(o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_Roach {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_Roach
}
#[inline]
unsafe fn Get_Sidehopper(o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_Sidehopper {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_Sidehopper
}
#[inline]
unsafe fn Get_Skree(o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_Skree {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_Skree
}
#[inline]
unsafe fn Get_StoneZoomer(o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_StoneZoomer {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_StoneZoomer
}
#[inline]
unsafe fn Get_WreckedShipOrangeZoomer(
    o: ::core::ffi::c_int,
    game: &mut Game,
) -> *mut Enemy_WreckedShipOrangeZoomer {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_WreckedShipOrangeZoomer
}
static g_word_A3A76D: [int16; 4] = [2, 0, -(2), 0];
static g_word_A3A775: [int16; 4] = [0, -(2), 0, 2];
pub unsafe fn Enemy_GrappleReact_NoInteract_A3(game: &mut Game) {
    SwitchEnemyAiToMainAi(game);
}
pub unsafe fn Enemy_GrappleReact_KillEnemy_A3(game: &mut Game) {
    EnemyGrappleDeath(game);
}
pub unsafe fn Enemy_GrappleReact_CancelBeam_A3(game: &mut Game) {
    Enemy_SwitchToFrozenAi(game);
}
pub unsafe fn Enemy_NormalTouchAI_A3(game: &mut Game) {
    NormalEnemyTouchAi(game);
}
pub unsafe fn Enemy_NormalShotAI_A3(game: &mut Game) {
    NormalEnemyShotAi(game);
}
pub unsafe fn Enemy_NormalShotAI_SkipSomeParts_A3(game: &mut Game) {
    NormalEnemyShotAiSkipDeathAnim_CurEnemy(game);
}
pub unsafe fn Enemy_NormalPowerBombAI_A3(game: &mut Game) {
    NormalEnemyPowerBombAi(game);
}
pub unsafe fn Enemy_NormalFrozenAI_A3(game: &mut Game) {
    NormalEnemyFrozenAI(game);
}
pub unsafe fn Metalee_Instr_1(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    (*Get_Metalee(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game)).metalee_var_E =
        1 as uint16;
    jp
}
pub unsafe fn Metalee_Init(game: &mut Game) {
    let E: *mut Enemy_Metalee =
        Get_Metalee(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).metalee_var_C = 0 as uint16;
    (*E).metalee_var_D = 0 as uint16;
    (*E).metalee_var_E = 0 as uint16;
    (*E).base.current_instruction = addr_kMetalee_Ilist_8910 as uint16;
    (*E).metalee_var_B = ((0xa38987i32) as u16);
}
pub unsafe fn CallMetaleeFunc(ea: uint32, game: &mut Game) {
    match ea {
        10717575 => Metalee_Func_1(game),
        10717652 => {
            Metalee_Func_3(ram_word!(game, cur_enemy_index), game);
        }
        10717683 => Metalee_Func_4(game),
        10717788 => Metalee_Func_5(game),
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn Metalee_Main(game: &mut Game) {
    let E: *mut Enemy_Metalee =
        Get_Metalee(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    CallMetaleeFunc((((*E).metalee_var_B) as u32) | (0xa30000u32), game);
}
pub unsafe fn Metalee_Func_1(game: &mut Game) {
    let E: *mut Enemy_Metalee =
        Get_Metalee(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    if abs16(((*E).base.x_pos).wrapping_sub(ram_word!(game, samus_x_pos))) < 0x48u16 {
        Metalee_Func_2(ram_word!(game, cur_enemy_index), game);
        (*E).metalee_var_C = (*E).metalee_var_C.wrapping_add(1);
        Metalee_Func_6(game);
        (*E).metalee_var_B = ((0xa389d4i32) as u16);
    }
}
pub unsafe fn Metalee_Func_2(k: uint16, game: &mut Game) {
    let E: *mut Enemy_Metalee = Get_Metalee(k as ::core::ffi::c_int, game);
    let div: uint16 = SnesDivide(
        (ram_word!(game, samus_y_pos)).wrapping_sub((*E).base.y_pos),
        24 as uint8,
    ) as uint8 as uint16;
    (*E).metalee_var_F = (div).wrapping_add(4u16);
}
pub unsafe fn Metalee_Func_3(k: uint16, game: &mut Game) {
    let E: *mut Enemy_Metalee = Get_Metalee(k as ::core::ffi::c_int, game);
    if (*E).metalee_var_E != 0 {
        (*E).metalee_var_E = 0 as uint16;
        (*E).metalee_var_C = (*E).metalee_var_C.wrapping_add(1);
        Metalee_Func_6(game);
        (*E).metalee_var_B = ((0xa389f3i32) as u16);
        QueueSfx2_Max6(0x5b as uint16, game);
    }
}
pub unsafe fn Metalee_Func_4(game: &mut Game) {
    let E: *mut Enemy_Metalee =
        Get_Metalee(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).metalee_var_A = 21 as uint16;
    let v1: uint16 = ((*E).base.properties) | (3u16);
    (*E).base.properties = v1;
    if EnemyFunc_BF8A(
        ram_word!(game, cur_enemy_index),
        v1,
        ((*E).metalee_var_F as int16 as int32) << 16 as ::core::ffi::c_int,
        game,
    ) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
        (*E).metalee_var_B = ((0xa38a5ci32) as u16);
        QueueSfx2_Max6(0x5c as uint16, game);
    } else {
        (*E).base.y_pos = ((*E).base.y_pos).wrapping_add((*E).metalee_var_F);
        (*E).base.x_pos = ((*E).base.x_pos as ::core::ffi::c_int
            + if (((*E).base.x_pos) as i16).wrapping_sub((ram_word!(game, samus_x_pos)) as i16)
                >= 0i16
            {
                -(2 as ::core::ffi::c_int)
            } else {
                2 as ::core::ffi::c_int
            }) as uint16;
    };
}
pub unsafe fn Metalee_Func_5(game: &mut Game) {
    let E: *mut Enemy_Metalee =
        Get_Metalee(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let fresh8 = (*E).metalee_var_A;
    (*E).metalee_var_A = (*E).metalee_var_A.wrapping_sub(1);
    if fresh8 == 1u16 {
        (*gEnemySpawnData(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game))
            .vram_tiles_index = ((*E).base.vram_tiles_index) | ((*E).base.palette_index);
        (*E).base.palette_index = 2560 as uint16;
        (*E).base.vram_tiles_index = 0 as uint16;
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Deleted) as u16);
    } else {
        if (*E).metalee_var_A == 8u16 {
            let v2: uint16 = ram_word!(game, cur_enemy_index);
            let mut parameter = SpawnEprojWithGfx(8, v2, addr_stru_868BFA as uint16, game);
            parameter = SpawnEprojWithGfx(parameter, v2, addr_stru_868C08 as uint16, game);
            parameter = SpawnEprojWithGfx(parameter, v2, addr_stru_868C16 as uint16, game);
            SpawnEprojWithGfx(parameter, v2, addr_stru_868C24 as uint16, game);
        }
        (*E).base.y_pos = (*E).base.y_pos.wrapping_add(1);
    };
}
pub unsafe fn Metalee_Func_6(game: &mut Game) {
    let E: *mut Enemy_Metalee =
        Get_Metalee(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let metalee_var_C: uint16 = (*E).metalee_var_C;
    if metalee_var_C != (*E).metalee_var_D {
        (*E).metalee_var_D = metalee_var_C;
        (*E).base.current_instruction = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3894e))
            as *const uint8 as *mut uint16)
            .offset(metalee_var_C as isize);
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
    }
}
pub unsafe fn Metalee_Shot(game: &mut Game) {
    let E: *mut Enemy_Metalee =
        Get_Metalee(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let varE2A: uint16 = (*E).base.vram_tiles_index;
    let varE2C: uint16 = (*E).base.palette_index;
    Enemy_NormalShotAI_A3(game);
    if (*Get_Metalee(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game))
        .base
        .health
        == 0
    {
        (*E).base.vram_tiles_index = varE2A;
        (*E).base.palette_index = varE2C;
        let v2: uint16 = ram_word!(game, cur_enemy_index);
        SpawnEprojWithGfx(
            (*E).metalee_var_A,
            ram_word!(game, cur_enemy_index),
            addr_stru_868BFA as uint16,
            game,
        );
        SpawnEprojWithGfx(0 as uint16, v2, addr_stru_868C08 as uint16, game);
        SpawnEprojWithGfx(0 as uint16, v2, addr_stru_868C16 as uint16, game);
        SpawnEprojWithGfx(0 as uint16, v2, addr_stru_868C24 as uint16, game);
        (*E).base.vram_tiles_index = 0 as uint16;
        (*E).base.palette_index = 0 as uint16;
    }
}
pub unsafe fn Fireflea_Init(game: &mut Game) {
    let E: *mut Enemy_FireFlea =
        Get_FireFlea(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).base.current_instruction = addr_kFireflea_Ilist_8C2F as uint16;
    if ((*E).ffa_parameter_1 as ::core::ffi::c_int & 2 as ::core::ffi::c_int) != 0i32 {
        Fireflea_Func_1(ram_word!(game, cur_enemy_index), game);
        Fireflea_Func_2(ram_word!(game, cur_enemy_index), game);
        Fireflea_Func_3(ram_word!(game, cur_enemy_index), game);
        Fireflea_Func_4(ram_word!(game, cur_enemy_index), game);
        Fireflea_Func_5(ram_word!(game, cur_enemy_index), game);
    } else {
        Fireflea_Func_3(ram_word!(game, cur_enemy_index), game);
        Fireflea_Func_4(ram_word!(game, cur_enemy_index), game);
        Fireflea_Func_6(ram_word!(game, cur_enemy_index), game);
    };
}
pub unsafe fn Fireflea_Func_1(k: uint16, game: &mut Game) {
    let E: *mut Enemy_FireFlea = Get_FireFlea(k as ::core::ffi::c_int, game);
    (*E).ffa_var_E = (*E).base.x_pos;
    (*E).ffa_var_F = (*E).base.y_pos;
}
pub unsafe fn Fireflea_Func_2(k: uint16, game: &mut Game) {
    let E: *mut Enemy_FireFlea = Get_FireFlea(k as ::core::ffi::c_int, game);
    (*E).ffa_var_D = ((*(&raw mut (*E).ffa_parameter_1 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>())
            .wrapping_div(::core::mem::size_of::<uint8>())
            .wrapping_sub(1_usize) as isize,
    ) as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Fireflea_Func_3(k: uint16, game: &mut Game) {
    let E: *mut Enemy_FireFlea = Get_FireFlea(k as ::core::ffi::c_int, game);
    let mut v2: uint16 =
        (8u16).wrapping_mul((*(&raw mut (*E).ffa_parameter_2 as *mut uint8).offset(0)) as u16);
    if ((*E).ffa_parameter_1 as ::core::ffi::c_int & 1 as ::core::ffi::c_int) == 0i32 {
        v2 = (v2).wrapping_add(4u16);
    }
    (*E).ffa_var_02 = v2;
    let v3: ::core::ffi::c_int = v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    (*E).ffa_var_B = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8
        as *mut uint16)
        .offset(v3 as isize);
    (*E).ffa_var_A = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8
        as *mut uint16)
        .offset((v3 + 1 as ::core::ffi::c_int) as isize);
}
pub unsafe fn Fireflea_Func_4(k: uint16, game: &mut Game) {
    let E: *mut Enemy_FireFlea = Get_FireFlea(k as ::core::ffi::c_int, game);
    (*E).ffa_var_C = *((game.g_rom.offset(crate::rtl::lorom_offset(0xa38d1d)) as *const uint8
        as *mut uint16)
        .offset(
            *(&raw mut (*E).ffa_parameter_2 as *mut uint8).offset(
                (::core::mem::size_of::<uint16>())
                    .wrapping_div(::core::mem::size_of::<uint8>())
                    .wrapping_sub(1_usize) as isize,
            ) as isize,
        ) as *mut uint16 as *mut uint8)
        .offset(0) as uint16;
}
pub unsafe fn Fireflea_Func_5(k: uint16, game: &mut Game) {
    let E: *mut Enemy_FireFlea = Get_FireFlea(k as ::core::ffi::c_int, game);
    (*E).base.x_pos =
        ((*E).ffa_var_E).wrapping_add(CosineMult8bit((*E).ffa_var_D, (*E).ffa_var_C, game));
    (*E).base.y_pos =
        ((*E).ffa_var_F).wrapping_add(SineMult8bit((*E).ffa_var_D, (*E).ffa_var_C, game));
}
pub unsafe fn Fireflea_Func_6(k: uint16, game: &mut Game) {
    let E: *mut Enemy_FireFlea = Get_FireFlea(k as ::core::ffi::c_int, game);
    (*E).ffa_var_00 = ((*E).base.y_pos).wrapping_sub((*E).ffa_var_C);
    (*E).ffa_var_01 = ((*E).ffa_var_C).wrapping_add((*E).base.y_pos);
}
pub unsafe fn Fireflea_Main(game: &mut Game) {
    let E: *mut Enemy_FireFlea =
        Get_FireFlea(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    if ((*E).ffa_parameter_1 as ::core::ffi::c_int & 2 as ::core::ffi::c_int) != 0i32 {
        (*E).base.x_pos = ((*E).ffa_var_E).wrapping_add(CosineMult8bit(
            *(&raw mut (*E).ffa_var_D as *mut uint8).offset(
                (::core::mem::size_of::<uint16>())
                    .wrapping_div(::core::mem::size_of::<uint8>())
                    .wrapping_sub(1_usize) as isize,
            ) as uint16,
            (*E).ffa_var_C,
            game,
        ));
        (*E).base.y_pos = ((*E).ffa_var_F).wrapping_add(SineMult8bit(
            *(&raw mut (*E).ffa_var_D as *mut uint8).offset(
                (::core::mem::size_of::<uint16>())
                    .wrapping_div(::core::mem::size_of::<uint8>())
                    .wrapping_sub(1_usize) as isize,
            ) as uint16,
            (*E).ffa_var_C,
            game,
        ));
        (*E).ffa_var_D = ((*E).ffa_var_D)
            .wrapping_add(*((&raw mut (*E).ffa_var_A as *mut uint8).offset(1) as *mut uint16));
    } else {
        let v2: ::core::ffi::c_int =
            (*E).ffa_var_02 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        AddToHiLo(
            &raw mut (*E).base.y_pos,
            &raw mut (*E).base.y_subpos,
            (*(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8 as *mut uint16)
                .offset(v2 as isize) as uint32)
                << 16 as ::core::ffi::c_int
                | *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8
                    as *mut uint16)
                    .offset((v2 + 1 as ::core::ffi::c_int) as isize) as uint32,
        );
        if (((*E).base.y_pos) as i16).wrapping_sub(((*E).ffa_var_00) as i16) < 0i16
            || (((*E).base.y_pos) as i16).wrapping_sub(((*E).ffa_var_01) as i16) >= 0i16
        {
            (*E).ffa_var_02 = ((*E).ffa_var_02) ^ (4u16);
        }
    };
}
pub unsafe fn Fireflea_Touch(k: uint16, game: &mut Game) {
    let v1: uint16 = 0 as uint16;
    Enemy_NormalTouchAI_A3(game);
    EnemyDeathAnimation(k, v1, game);
    if ram_word!(game, fireflea_darkness_level) as ::core::ffi::c_int - 12 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let ref mut fresh7 = *((&raw mut game.g_ram as *mut uint8).offset(0x177e) as *mut uint16);
        *fresh7 = (*fresh7).wrapping_add(2u16);
    }
}
pub unsafe fn Fireflea_Powerbomb(game: &mut Game) {
    Enemy_NormalPowerBombAI_A3(game);
    Fireflea_Common(game);
}
pub unsafe fn Fireflea_Shot(game: &mut Game) {
    Enemy_NormalShotAI_A3(game);
    Fireflea_Common(game);
}
pub unsafe fn Fireflea_Common(game: &mut Game) {
    if (*Get_FireFlea(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game))
        .base
        .health
        == 0
        && ram_word!(game, fireflea_darkness_level) as ::core::ffi::c_int - 12 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
    {
        let ref mut fresh6 = *((&raw mut game.g_ram as *mut uint8).offset(0x177e) as *mut uint16);
        *fresh6 = (*fresh6).wrapping_add(2u16);
    }
}
pub unsafe fn MaridiaFish_Instr_3(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    (*Get_MaridiaFish(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game))
        .base
        .layer = 6 as uint16;
    jp
}
pub unsafe fn MaridiaFish_Instr_1(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    (*Get_MaridiaFish(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game))
        .base
        .layer = 2 as uint16;
    jp
}
pub unsafe fn MaridiaFish_Instr_2(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    (*Get_MaridiaFish(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game)).mfh_var_01 =
        1 as uint16;
    jp
}
pub unsafe fn MaridiaFish_Init(game: &mut Game) {
    let E: *mut Enemy_MaridiaFish =
        Get_MaridiaFish(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).base.current_instruction = addr_kMaridiaFish_Ilist_902A as uint16;
    (*E).mfh_var_A = ((0xa39132i32) as u16);
    if *(&raw mut (*E).mfh_parameter_1 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>())
            .wrapping_div(::core::mem::size_of::<uint8>())
            .wrapping_sub(1_usize) as isize,
    ) == 0
    {
        (*E).base.current_instruction = addr_kMaridiaFish_Ilist_9060 as uint16;
        (*E).mfh_var_A = ((0xa391abi32) as u16);
    }
    let v1: ::core::ffi::c_int = 8 as ::core::ffi::c_int
        * *(&raw mut (*E).mfh_parameter_1 as *mut uint8).offset(0) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    (*E).mfh_var_C = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8
        as *mut uint16)
        .offset(v1 as isize);
    (*E).mfh_var_B = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8
        as *mut uint16)
        .offset((v1 + 1 as ::core::ffi::c_int) as isize);
    (*E).mfh_var_E = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8
        as *mut uint16)
        .offset((v1 + 2 as ::core::ffi::c_int) as isize);
    (*E).mfh_var_D = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8
        as *mut uint16)
        .offset((v1 + 3 as ::core::ffi::c_int) as isize);
    (*E).mfh_var_00 = *(&raw mut (*E).mfh_parameter_2 as *mut uint8).offset(0) as uint16;
    (*E).mfh_var_02 = *(&raw mut (*E).mfh_parameter_2 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>())
            .wrapping_div(::core::mem::size_of::<uint8>())
            .wrapping_sub(1_usize) as isize,
    ) as uint16;
    (*E).mfh_var_F = 0 as uint16;
    (*E).mfh_var_01 = 0 as uint16;
    (*E).mfh_var_03 = SineMult8bit((*E).mfh_var_F, (*E).mfh_var_00, game);
}
pub unsafe fn MaridiaFish_Func_1(game: &mut Game) {
    let E: *mut Enemy_MaridiaFish =
        Get_MaridiaFish(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    if Enemy_MoveRight_IgnoreSlopes(
        ram_word!(game, cur_enemy_index),
        (((*E).mfh_var_E as uint32) << 16 as ::core::ffi::c_int | (*E).mfh_var_D as uint32)
            as int32,
        game,
    ) != 0
    {
        (*E).mfh_var_A = ((0xa39224i32) as u16);
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
        (*E).base.current_instruction = addr_kMaridiaFish_Ilist_903C as uint16;
    } else {
        let v2: uint16 = SineMult8bit((*E).mfh_var_F, (*E).mfh_var_00, game);
        (*E).mfh_var_04 = v2;
        if Enemy_MoveDown(
            ram_word!(game, cur_enemy_index),
            ((((v2) as i16).wrapping_sub(((*E).mfh_var_03) as i16)) as int32)
                << 16 as ::core::ffi::c_int,
            game,
        ) != 0
        {
            (*E).mfh_var_A = ((0xa39224i32) as u16);
            (*E).base.instruction_timer = 1 as uint16;
            (*E).base.timer = 0 as uint16;
            (*E).base.current_instruction = addr_kMaridiaFish_Ilist_903C as uint16;
        } else {
            (*E).mfh_var_F = ((*(&raw mut (*E).mfh_var_02 as *mut uint8).offset(0))
                .wrapping_add(((*E).mfh_var_F) as u8)) as uint16;
        }
    }
    (*E).mfh_var_03 = (*E).mfh_var_04;
}
pub unsafe fn MaridiaFish_Func_2(game: &mut Game) {
    let E: *mut Enemy_MaridiaFish =
        Get_MaridiaFish(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    if Enemy_MoveRight_IgnoreSlopes(
        ram_word!(game, cur_enemy_index),
        (((*E).mfh_var_C as uint32) << 16 as ::core::ffi::c_int | (*E).mfh_var_B as uint32)
            as int32,
        game,
    ) != 0
    {
        (*E).mfh_var_A = ((0xa39256i32) as u16);
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
        (*E).base.current_instruction = addr_kMaridiaFish_Ilist_9072 as uint16;
    } else {
        let v2: uint16 = SineMult8bit((*E).mfh_var_F, (*E).mfh_var_00, game);
        (*E).mfh_var_04 = v2;
        if Enemy_MoveDown(
            ram_word!(game, cur_enemy_index),
            ((((v2) as i16).wrapping_sub(((*E).mfh_var_03) as i16)) as int32)
                << 16 as ::core::ffi::c_int,
            game,
        ) != 0
        {
            (*E).mfh_var_A = ((0xa39256i32) as u16);
            (*E).base.instruction_timer = 1 as uint16;
            (*E).base.timer = 0 as uint16;
            (*E).base.current_instruction = addr_kMaridiaFish_Ilist_9072 as uint16;
        } else {
            (*E).mfh_var_F = ((*(&raw mut (*E).mfh_var_02 as *mut uint8).offset(0))
                .wrapping_add(((*E).mfh_var_F) as u8)) as uint16;
        }
    }
    (*E).mfh_var_03 = (*E).mfh_var_04;
}
pub unsafe fn MaridiaFish_Func_3(game: &mut Game) {
    let E: *mut Enemy_MaridiaFish =
        Get_MaridiaFish(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    if (*E).mfh_var_01 != 0 {
        (*E).mfh_var_01 = 0 as uint16;
        (*E).mfh_var_A = ((0xa391abi32) as u16);
        (*E).mfh_var_02 = -((*E).mfh_var_02 as ::core::ffi::c_int) as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
        (*E).base.current_instruction = addr_kMaridiaFish_Ilist_9060 as uint16;
    }
}
pub unsafe fn MaridiaFish_Func_4(game: &mut Game) {
    let E: *mut Enemy_MaridiaFish =
        Get_MaridiaFish(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    if (*E).mfh_var_01 != 0 {
        (*E).mfh_var_01 = 0 as uint16;
        (*E).mfh_var_A = ((0xa39132i32) as u16);
        (*E).mfh_var_02 = -((*E).mfh_var_02 as ::core::ffi::c_int) as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
        (*E).base.current_instruction = addr_kMaridiaFish_Ilist_902A as uint16;
    }
}
pub unsafe fn CallMaridiaFishFunc(ea: uint32, game: &mut Game) {
    match ea {
        10719538 => MaridiaFish_Func_1(game),
        10719659 => MaridiaFish_Func_2(game),
        10719780 => MaridiaFish_Func_3(game),
        10719830 => MaridiaFish_Func_4(game),
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn MaridiaFish_Main(game: &mut Game) {
    let E: *mut Enemy_MaridiaFish =
        Get_MaridiaFish(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    CallMaridiaFishFunc((((*E).mfh_var_A) as u32) | (0xa30000u32), game);
}
pub unsafe fn Elevator_Init(game: &mut Game) {
    let E: *mut Enemy_Elevator =
        Get_Elevator(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).base.spritemap_pointer = addr_kSpritemap_Nothing_A3 as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
    (*E).base.current_instruction = addr_kElevator_Ilist_94D6 as uint16;
    (*E).elevat_parameter_1 = ((*E).elevat_parameter_1).wrapping_mul(2u16);
    (*E).elevat_var_A = (*E).base.y_pos;
    if ram_word!(game, elevator_status) != 2u16 {
        let ref mut fresh5 = *((&raw mut game.g_ram as *mut uint8).offset(0xe18) as *mut uint16);
        *fresh5 = 0 as uint16;
        set_ram_word!(game.elevator_flags = *fresh5);
    }
    if (ram_word!(game, elevator_status) as uint32) << 16 as ::core::ffi::c_int
        | ram_word!(game, elevator_flags) as uint32
        != 0
    {
        (*E).base.y_pos = (*E).elevat_parameter_2;
        Elevator_Func_4(game);
    }
}
static off_A39540: [Option<Func_V>; 4] = unsafe {
    [
        Some(Elevator_Func_1 as unsafe fn(&mut Game) -> ()),
        Some(Elevator_Func_2 as unsafe fn(&mut Game) -> ()),
        Some(Elevator_Func_3 as unsafe fn(&mut Game) -> ()),
        Some(Elevator_Func3b as unsafe fn(&mut Game) -> ()),
    ]
};
pub unsafe fn Elevator_Frozen(game: &mut Game) {
    if ram_word!(game, door_transition_flag_elevator_zebetites) == 0
        && (ram_word!(game, elevator_status) as uint32) << 16 as ::core::ffi::c_int
            | ram_word!(game, elevator_flags) as uint32
            != 0
    {
        off_A39540[ram_word!(game, elevator_status) as usize].expect("non-null function pointer")(
            game,
        );
    }
}
pub unsafe fn Elevator_Func_1(game: &mut Game) {
    if (*(game.g_rom.offset(crate::rtl::lorom_offset(0xa394e2)) as *const uint8 as *mut uint16)
        .offset(
            ((*Get_Elevator(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game))
                .elevat_parameter_1 as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int) as isize,
        ) as ::core::ffi::c_int
        & ram_word!(game, joypad1_newkeys) as ::core::ffi::c_int)
        != 0i32
    {
        QueueSfx3_Max6(0xb as uint16, game);
        QueueSfx1_Max6(0x32 as uint16, game);
        CallSomeSamusCode(7 as uint16, game);
        ResetProjectileData(game);
        Elevator_Func_4(game);
        let ref mut fresh4 = *((&raw mut game.g_ram as *mut uint8).offset(0xe18) as *mut uint16);
        *fresh4 = (*fresh4).wrapping_add(1);
    } else {
        set_ram_word!(game.elevator_flags = 0 as uint16);
    };
}
pub unsafe fn Elevator_Func_2(game: &mut Game) {
    let E: *mut Enemy_Elevator =
        Get_Elevator(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    if (*E).elevat_parameter_1 != 0 {
        set_ram_word!(game.elevator_direction = 0x8000 as uint16);
        AddToHiLo(
            &raw mut (*E).base.y_pos,
            &raw mut (*E).base.y_subpos,
            -(0x18000 as ::core::ffi::c_int) as uint32,
        );
    } else {
        set_ram_word!(game.elevator_direction = 0 as uint16);
        AddToHiLo(
            &raw mut (*E).base.y_pos,
            &raw mut (*E).base.y_subpos,
            0x18000 as uint32,
        );
    }
    Elevator_Func_4(game);
}
pub unsafe fn Elevator_Func_3(game: &mut Game) {
    let ref mut fresh3 = *((&raw mut game.g_ram as *mut uint8).offset(0xe18) as *mut uint16);
    *fresh3 = (*fresh3).wrapping_add(1);
    Elevator_Func3b(game);
}
pub unsafe fn Elevator_Func3b(game: &mut Game) {
    let E: *mut Enemy_Elevator =
        Get_Elevator(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    if (*E).elevat_parameter_1 != 0 {
        AddToHiLo(
            &raw mut (*E).base.y_pos,
            &raw mut (*E).base.y_subpos,
            0x18000 as uint32,
        );
        if (*E).base.y_pos < (*E).elevat_var_A {
            Elevator_Func_4(game);
            return;
        }
    } else {
        AddToHiLo(
            &raw mut (*E).base.y_pos,
            &raw mut (*E).base.y_subpos,
            -(0x18000 as ::core::ffi::c_int) as uint32,
        );
        if (*E).base.y_pos >= (*E).elevat_var_A {
            Elevator_Func_4(game);
            return;
        }
    }
    set_ram_word!(game.elevator_status = 0 as uint16);
    set_ram_word!(game.elevator_flags = 0 as uint16);
    QueueSfx3_Max6(0x25 as uint16, game);
    (*E).base.y_pos = (*E).elevat_var_A;
    CallSomeSamusCode(0xb as uint16, game);
    Elevator_Func_4(game);
}
pub unsafe fn Elevator_Func_4(game: &mut Game) {
    let E: *mut Enemy_Elevator =
        Get_Elevator(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    set_ram_word!(game.samus_y_pos = ((*E).base.y_pos).wrapping_sub(26u16));
    set_ram_word!(game.samus_y_subpos = 0 as uint16);
    set_ram_word!(game.samus_x_pos = (*E).base.x_pos);
    set_ram_word!(game.samus_y_speed = 0 as uint16);
    set_ram_word!(game.samus_y_subspeed = 0 as uint16);
}
pub unsafe fn Crab_Init(game: &mut Game) {
    let v1: *mut EnemyData =
        gEnemyData(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*v1).parameter_2 = 8 as uint16;
    (*v1).current_instruction = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa396db))
        as *const uint8 as *mut uint16)
        .offset(
            ((*v1).current_instruction as ::core::ffi::c_int & 3 as ::core::ffi::c_int) as isize,
        );
    StoneZoomer_E67A(ram_word!(game, cur_enemy_index), game);
}
pub fn Crab_Func_1() {}
pub unsafe fn Slug_Init(game: &mut Game) {
    let v1: *mut EnemyData =
        gEnemyData(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*v1).parameter_2 = 10 as uint16;
    (*v1).current_instruction = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3992b))
        as *const uint8 as *mut uint16)
        .offset(
            ((*v1).current_instruction as ::core::ffi::c_int & 3 as ::core::ffi::c_int) as isize,
        );
    StoneZoomer_E67A(ram_word!(game, cur_enemy_index), game);
}
pub fn Slug_Func_1() {}
pub unsafe fn PlatformThatFallsWithSamus_Instr_3(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_PlatformThatFallsWithSamus(
        ram_word!(game, cur_enemy_index) as ::core::ffi::c_int,
        game,
    ))
    .ptfwss_var_02 = 0 as uint16;
    jp
}
pub unsafe fn PlatformThatFallsWithSamus_Instr_4(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_PlatformThatFallsWithSamus(
        ram_word!(game, cur_enemy_index) as ::core::ffi::c_int,
        game,
    ))
    .ptfwss_var_02 = 1 as uint16;
    jp
}
pub unsafe fn PlatformThatFallsWithSamus_Instr_1(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_PlatformThatFallsWithSamus(
        ram_word!(game, cur_enemy_index) as ::core::ffi::c_int,
        game,
    ))
    .ptfwss_var_02 = 0 as uint16;
    jp
}
pub unsafe fn PlatformThatFallsWithSamus_Instr_2(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_PlatformThatFallsWithSamus(
        ram_word!(game, cur_enemy_index) as ::core::ffi::c_int,
        game,
    ))
    .ptfwss_var_02 = 1 as uint16;
    jp
}
pub unsafe fn PlatformThatFallsWithSamus_Init(game: &mut Game) {
    let E: *mut Enemy_PlatformThatFallsWithSamus = Get_PlatformThatFallsWithSamus(
        ram_word!(game, cur_enemy_index) as ::core::ffi::c_int,
        game,
    );
    (*E).ptfwss_var_1F = -(1 as ::core::ffi::c_int) as uint16;
    let ptfwss_parameter_1: uint16 = (*E).ptfwss_parameter_1;
    (*E).ptfwss_var_02 = ptfwss_parameter_1;
    if ptfwss_parameter_1 != 0 {
        PlatformThatFalls_Init(
            ram_word!(game, cur_enemy_index),
            addr_kPlatformThatFallsWithSamus_Ilist_9BFD as uint16,
            game,
        );
    } else {
        PlatformThatFalls_Init(
            ram_word!(game, cur_enemy_index),
            addr_kPlatformThatFallsWithSamus_Ilist_9BE7 as uint16,
            game,
        );
    };
}
pub unsafe fn FastMovingSlowSinkingPlatform_Init(game: &mut Game) {
    let E: *mut Enemy_PlatformThatFallsWithSamus = Get_PlatformThatFallsWithSamus(
        ram_word!(game, cur_enemy_index) as ::core::ffi::c_int,
        game,
    );
    let ptfwss_parameter_1: uint16 = (*E).ptfwss_parameter_1;
    (*E).ptfwss_var_02 = ptfwss_parameter_1;
    if ptfwss_parameter_1 != 0 {
        PlatformThatFalls_Init(
            ram_word!(game, cur_enemy_index),
            addr_kPlatformThatFallsWithSamus_Ilist_9C55 as uint16,
            game,
        );
    } else {
        PlatformThatFalls_Init(
            ram_word!(game, cur_enemy_index),
            addr_kPlatformThatFallsWithSamus_Ilist_9C3F as uint16,
            game,
        );
    };
}
pub unsafe fn PlatformThatFalls_Init(k: uint16, j: uint16, game: &mut Game) {
    let E: *mut Enemy_PlatformThatFallsWithSamus =
        Get_PlatformThatFallsWithSamus(k as ::core::ffi::c_int, game);
    (*E).base.current_instruction = j;
    let v3: ::core::ffi::c_int = 8 as ::core::ffi::c_int
        * *(&raw mut (*E).ptfwss_parameter_2 as *mut uint8).offset(0) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    (*E).ptfwss_var_C = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8
        as *mut uint16)
        .offset(v3 as isize);
    (*E).ptfwss_var_B = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8
        as *mut uint16)
        .offset((v3 + 1 as ::core::ffi::c_int) as isize);
    (*E).ptfwss_var_E = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8
        as *mut uint16)
        .offset((v3 + 2 as ::core::ffi::c_int) as isize);
    (*E).ptfwss_var_D = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8
        as *mut uint16)
        .offset((v3 + 3 as ::core::ffi::c_int) as isize);
    (*E).ptfwss_var_00 = 0 as uint16;
    (*E).ptfwss_var_03 = 0 as uint16;
    (*E).ptfwss_var_04 = 0 as uint16;
    (*E).ptfwss_var_A = ((*E).base.y_pos).wrapping_add(1u16);
    (*E).ptfwss_var_F = 0 as uint16;
    (*E).ptfwss_var_05 = *(&raw mut (*E).ptfwss_parameter_2 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>())
            .wrapping_div(::core::mem::size_of::<uint8>())
            .wrapping_sub(1_usize) as isize,
    ) as uint16;
}
static off_A39C97: [Option<Func_V>; 2] = unsafe {
    [
        Some(PlatformThatFallsWithSamus_Func_1 as unsafe fn(&mut Game) -> ()),
        Some(PlatformThatFallsWithSamus_Func_2 as unsafe fn(&mut Game) -> ()),
    ]
};
static off_A39C9B: [Option<Func_V>; 2] = unsafe {
    [
        Some(PlatformThatFallsWithSamus_Func_3 as unsafe fn(&mut Game) -> ()),
        Some(PlatformThatFallsWithSamus_Func_4 as unsafe fn(&mut Game) -> ()),
    ]
};
pub unsafe fn PlatformThatFallsWithSamus_Main(game: &mut Game) {
    let E: *mut Enemy_PlatformThatFallsWithSamus = Get_PlatformThatFallsWithSamus(
        ram_word!(game, cur_enemy_index) as ::core::ffi::c_int,
        game,
    );
    (*E).ptfwss_var_00 = 0 as uint16;
    if CheckIfEnemyTouchesSamus(ram_word!(game, cur_enemy_index), game) != 0 {
        (*E).ptfwss_var_00 = 1 as uint16;
    }
    off_A39C97[(*E).ptfwss_var_02 as usize].expect("non-null function pointer")(game);
    let v3: ::core::ffi::c_int = (*Get_PlatformThatFallsWithSamus(
        ram_word!(game, cur_enemy_index) as ::core::ffi::c_int,
        game,
    ))
    .ptfwss_var_00 as ::core::ffi::c_int;
    off_A39C9B[v3 as usize].expect("non-null function pointer")(game);
    if (*E).ptfwss_var_00 != (*E).ptfwss_var_06 {
        (*E).ptfwss_var_F = 0 as uint16;
    }
    (*E).ptfwss_var_06 = (*E).ptfwss_var_00;
}
pub unsafe fn PlatformThatFallsWithSamus_Func_1(game: &mut Game) {
    let E: *mut Enemy_PlatformThatFallsWithSamus = Get_PlatformThatFallsWithSamus(
        ram_word!(game, cur_enemy_index) as ::core::ffi::c_int,
        game,
    );
    (*E).ptfwss_var_01 = (*E).base.x_pos;
    if Enemy_MoveRight_IgnoreSlopes(
        ram_word!(game, cur_enemy_index),
        (((*E).ptfwss_var_E as uint32) << 16 as ::core::ffi::c_int | (*E).ptfwss_var_D as uint32)
            as int32,
        game,
    ) != 0
    {
        (*E).ptfwss_var_02 = 1 as uint16;
        PlatformThatFallsWithSamus_Func_8(game);
    }
}
pub unsafe fn PlatformThatFallsWithSamus_Func_2(game: &mut Game) {
    let E: *mut Enemy_PlatformThatFallsWithSamus = Get_PlatformThatFallsWithSamus(
        ram_word!(game, cur_enemy_index) as ::core::ffi::c_int,
        game,
    );
    (*E).ptfwss_var_01 = (*E).base.x_pos;
    if Enemy_MoveRight_IgnoreSlopes(
        ram_word!(game, cur_enemy_index),
        (((*E).ptfwss_var_C as uint32) << 16 as ::core::ffi::c_int | (*E).ptfwss_var_B as uint32)
            as int32,
        game,
    ) != 0
    {
        (*E).ptfwss_var_02 = 0 as uint16;
        PlatformThatFallsWithSamus_Func_7(game);
    }
}
pub unsafe fn PlatformThatFallsWithSamus_Func_3(game: &mut Game) {
    let mut ptfwss_var_05: uint16 = 0;
    let mut v3: ::core::ffi::c_int = 0;
    let current_block: u64;
    let E: *mut Enemy_PlatformThatFallsWithSamus = Get_PlatformThatFallsWithSamus(
        ram_word!(game, cur_enemy_index) as ::core::ffi::c_int,
        game,
    );
    if (((*E).base.y_pos) as i16).wrapping_sub(((*E).ptfwss_var_A) as i16) < 0i16 {
        current_block = 2432778910466551290;
    } else {
        PlatformThatFallsWithSamus_Func_9(game);
        ptfwss_var_05 = 0;
        (*E).ptfwss_var_F = (*E).ptfwss_var_F.wrapping_add(1);
        ptfwss_var_05 = (*E).ptfwss_var_F;
        if ((ptfwss_var_05) as i16).wrapping_sub(((*E).ptfwss_var_05) as i16) >= 0i16 {
            ptfwss_var_05 = (*E).ptfwss_var_05;
            (*E).ptfwss_var_F = ptfwss_var_05;
        }
        v3 = 0;
        v3 = 8 as ::core::ffi::c_int * ptfwss_var_05 as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int;
        if Enemy_MoveDown(
            ram_word!(game, cur_enemy_index),
            ((*(game.g_rom.offset(crate::rtl::lorom_offset(0xa2838f)) as *const uint8
                as *mut uint16)
                .offset((v3 + 3 as ::core::ffi::c_int) as isize) as uint32)
                << 16 as ::core::ffi::c_int
                | *(game.g_rom.offset(crate::rtl::lorom_offset(0xa2838f)) as *const uint8
                    as *mut uint16)
                    .offset((v3 + 2 as ::core::ffi::c_int) as isize) as uint32)
                as int32,
            game,
        ) != 0
        {
            current_block = 2432778910466551290;
        } else {
            current_block = 3640593987805443782;
        }
    }
    if current_block == 2432778910466551290 {
        (*E).ptfwss_var_F = 0 as uint16;
        PlatformThatFallsWithSamus_Func_10(game);
    };
}
pub unsafe fn PlatformThatFallsWithSamus_Func_4(game: &mut Game) {
    let E: *mut Enemy_PlatformThatFallsWithSamus = Get_PlatformThatFallsWithSamus(
        ram_word!(game, cur_enemy_index) as ::core::ffi::c_int,
        game,
    );
    (*E).ptfwss_var_F = (*E).ptfwss_var_F.wrapping_add(1);
    if (((*E).ptfwss_var_F) as i16).wrapping_sub(((*E).ptfwss_var_05) as i16) >= 0i16 {
        (*E).ptfwss_var_F = (*E).ptfwss_var_05;
    }
    let ref mut fresh13 = *((&raw mut game.g_ram as *mut uint8).offset(0xb58) as *mut uint16);
    *fresh13 = (*fresh13 as ::core::ffi::c_int
        + ((*E).base.x_pos as ::core::ffi::c_int - (*E).ptfwss_var_01 as ::core::ffi::c_int))
        as uint16;
    (*E).ptfwss_var_01 = (*E).base.y_pos;
    let v2: ::core::ffi::c_int = 8 as ::core::ffi::c_int * (*E).ptfwss_var_F as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    if Enemy_MoveDown(
        ram_word!(game, cur_enemy_index),
        ((*(game.g_rom.offset(crate::rtl::lorom_offset(0xa2838f)) as *const uint8 as *mut uint16)
            .offset((v2 + 1 as ::core::ffi::c_int) as isize) as uint32)
            << 16 as ::core::ffi::c_int
            | *(game.g_rom.offset(crate::rtl::lorom_offset(0xa2838f)) as *const uint8
                as *mut uint16)
                .offset(v2 as isize) as uint32) as int32,
        game,
    ) != 0
    {
        (*E).ptfwss_var_F = 0 as uint16;
        PlatformThatFallsWithSamus_Func_10(game);
    }
    let ref mut fresh14 = *((&raw mut game.g_ram as *mut uint8).offset(0xb5c) as *mut uint16);
    *fresh14 = (*fresh14 as ::core::ffi::c_int
        + ((*E).base.y_pos as ::core::ffi::c_int - (*E).ptfwss_var_01 as ::core::ffi::c_int))
        as uint16;
}
pub unsafe fn PlatformThatFallsWithSamus_Func_5(game: &mut Game) {
    let E: *mut Enemy_PlatformThatFallsWithSamus = Get_PlatformThatFallsWithSamus(
        ram_word!(game, cur_enemy_index) as ::core::ffi::c_int,
        game,
    );
    let mut v0: uint16 = addr_kPlatformThatFallsWithSamus_Ilist_9C13 as uint16;
    if ((*E).ptfwss_var_1F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        v0 = addr_kPlatformThatFallsWithSamus_Ilist_9BBB as uint16;
    }
    (*E).base.current_instruction = v0;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
}
pub unsafe fn PlatformThatFallsWithSamus_Func_6(game: &mut Game) {
    let E: *mut Enemy_PlatformThatFallsWithSamus = Get_PlatformThatFallsWithSamus(
        ram_word!(game, cur_enemy_index) as ::core::ffi::c_int,
        game,
    );
    let mut v0: uint16 = addr_kPlatformThatFallsWithSamus_Ilist_9C29 as uint16;
    if ((*E).ptfwss_var_1F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        v0 = addr_kPlatformThatFallsWithSamus_Ilist_9BD1 as uint16;
    }
    (*E).base.current_instruction = v0;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
}
pub unsafe fn PlatformThatFallsWithSamus_Func_7(game: &mut Game) {
    let E: *mut Enemy_PlatformThatFallsWithSamus = Get_PlatformThatFallsWithSamus(
        ram_word!(game, cur_enemy_index) as ::core::ffi::c_int,
        game,
    );
    let mut v0: uint16 = addr_kPlatformThatFallsWithSamus_Ilist_9C3F as uint16;
    if ((*E).ptfwss_var_1F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        v0 = addr_kPlatformThatFallsWithSamus_Ilist_9BE7 as uint16;
    }
    (*E).base.current_instruction = v0;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
}
pub unsafe fn PlatformThatFallsWithSamus_Func_8(game: &mut Game) {
    let E: *mut Enemy_PlatformThatFallsWithSamus = Get_PlatformThatFallsWithSamus(
        ram_word!(game, cur_enemy_index) as ::core::ffi::c_int,
        game,
    );
    let mut v0: uint16 = addr_kPlatformThatFallsWithSamus_Ilist_9C55 as uint16;
    if ((*E).ptfwss_var_1F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        v0 = addr_kPlatformThatFallsWithSamus_Ilist_9BFD as uint16;
    }
    (*E).base.current_instruction = v0;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
}
pub unsafe fn PlatformThatFallsWithSamus_Func_9(game: &mut Game) {
    let E: *mut Enemy_PlatformThatFallsWithSamus = Get_PlatformThatFallsWithSamus(
        ram_word!(game, cur_enemy_index) as ::core::ffi::c_int,
        game,
    );
    if (*E).ptfwss_var_03 == 0 {
        (*E).ptfwss_var_03 = 1 as uint16;
        if (*E).ptfwss_var_02 != 0 {
            PlatformThatFallsWithSamus_Func_6(game);
        } else {
            PlatformThatFallsWithSamus_Func_5(game);
        }
    }
    (*E).ptfwss_var_04 = 0 as uint16;
}
pub unsafe fn PlatformThatFallsWithSamus_Func_10(game: &mut Game) {
    let E: *mut Enemy_PlatformThatFallsWithSamus = Get_PlatformThatFallsWithSamus(
        ram_word!(game, cur_enemy_index) as ::core::ffi::c_int,
        game,
    );
    if (*E).ptfwss_var_04 == 0 {
        (*E).ptfwss_var_04 = 1 as uint16;
        if (*E).ptfwss_var_02 != 0 {
            PlatformThatFallsWithSamus_Func_8(game);
        } else {
            PlatformThatFallsWithSamus_Func_7(game);
        }
    }
    (*E).ptfwss_var_03 = 0 as uint16;
}
pub unsafe fn FastMovingSlowSinkingPlatform_Shot(game: &mut Game) {
    Enemy_NormalShotAI_A3(game);
    let E: *mut Enemy_PlatformThatFallsWithSamus = Get_PlatformThatFallsWithSamus(
        ram_word!(game, cur_enemy_index) as ::core::ffi::c_int,
        game,
    );
    if (*E).base.frozen_timer != 0 {
        if (*E).ptfwss_var_02 != 0 {
            (*E).base.spritemap_pointer = addr_kPlatformThatFallsWithSamus_Sprmap_A015 as uint16;
        } else {
            (*E).base.spritemap_pointer = addr_kPlatformThatFallsWithSamus_Sprmap_A009 as uint16;
        }
    }
}
pub unsafe fn Roach_Func_1(game: &mut Game) {
    let E: *mut Enemy_Roach =
        Get_Roach(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let roach_var_24: uint16 = (*E).roach_var_24;
    if roach_var_24 != (*E).roach_var_25 {
        (*E).roach_var_25 = roach_var_24;
        (*E).base.current_instruction = roach_var_24;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
    }
}
pub unsafe fn Roach_Init(game: &mut Game) {
    let E: *mut Enemy_Roach =
        Get_Roach(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).roach_var_25 = 0 as uint16;
    (*E).roach_var_24 = 0 as uint16;
    Roach_Func_2(ram_word!(game, cur_enemy_index), game);
    Roach_Func_4(ram_word!(game, cur_enemy_index), game);
    Roach_Func_5(ram_word!(game, cur_enemy_index), game);
    Roach_Func_6(ram_word!(game, cur_enemy_index), game);
    Roach_Func_7(ram_word!(game, cur_enemy_index), game);
    Roach_Func_8(ram_word!(game, cur_enemy_index), game);
    (*E).roach_var_24 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3a111)) as *const uint8
        as *mut uint16)
        .offset(((*E).roach_var_20 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    Roach_Func_1(game);
    (*E).roach_var_B = ((0xa3a2d7i32) as u16);
}
pub unsafe fn Roach_Func_2(k: uint16, game: &mut Game) {
    let E: *mut Enemy_Roach = Get_Roach(k as ::core::ffi::c_int, game);
    let pt: Point32 = ConvertAngleToXy(
        *(&raw mut (*E).roach_parameter_1 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1_usize) as isize,
        ) as uint16,
        *(&raw mut (*E).roach_parameter_1 as *mut uint8).offset(0) as uint16,
        game,
    );
    SetHiLo(&raw mut (*E).roach_var_01, &raw mut (*E).roach_var_00, pt.x);
    SetHiLo(&raw mut (*E).roach_var_03, &raw mut (*E).roach_var_02, pt.y);
}
pub unsafe fn Roach_Func_3(k: uint16, game: &mut Game) {
    let E: *mut Enemy_Roach = Get_Roach(k as ::core::ffi::c_int, game);
    SetHiLo(
        &raw mut (*E).roach_var_01,
        &raw mut (*E).roach_var_00,
        CosineMult8bitFull(
            *(&raw mut (*E).roach_parameter_1 as *mut uint8).offset(
                (::core::mem::size_of::<uint16>())
                    .wrapping_div(::core::mem::size_of::<uint8>())
                    .wrapping_sub(1_usize) as isize,
            ) as uint16,
            *(&raw mut (*E).roach_parameter_1 as *mut uint8).offset(0) as uint16,
            game,
        ),
    );
    SetHiLo(
        &raw mut (*E).roach_var_03,
        &raw mut (*E).roach_var_02,
        SineMult8bitFull(
            *(&raw mut (*E).roach_parameter_1 as *mut uint8).offset(
                (::core::mem::size_of::<uint16>())
                    .wrapping_div(::core::mem::size_of::<uint8>())
                    .wrapping_sub(1_usize) as isize,
            ) as uint16,
            *(&raw mut (*E).roach_parameter_1 as *mut uint8).offset(0) as uint16,
            game,
        ),
    );
}
pub unsafe fn Roach_Func_4(k: uint16, game: &mut Game) {
    let E: *mut Enemy_Roach = Get_Roach(k as ::core::ffi::c_int, game);
    SetHiLo(
        &raw mut (*E).roach_var_05,
        &raw mut (*E).roach_var_04,
        CosineMult8bitFull(
            ((*(&raw mut (*E).roach_parameter_1 as *mut uint8).offset(
                (::core::mem::size_of::<uint16>())
                    .wrapping_div(::core::mem::size_of::<uint8>())
                    .wrapping_sub(1_usize) as isize,
            ))
            .wrapping_sub(32u8)) as uint16,
            *(&raw mut (*E).roach_parameter_1 as *mut uint8).offset(0) as uint16,
            game,
        ),
    );
    SetHiLo(
        &raw mut (*E).roach_var_07,
        &raw mut (*E).roach_var_06,
        SineMult8bitFull(
            ((*(&raw mut (*E).roach_parameter_1 as *mut uint8).offset(
                (::core::mem::size_of::<uint16>())
                    .wrapping_div(::core::mem::size_of::<uint8>())
                    .wrapping_sub(1_usize) as isize,
            ))
            .wrapping_sub(32u8)) as uint16,
            *(&raw mut (*E).roach_parameter_1 as *mut uint8).offset(0) as uint16,
            game,
        ),
    );
}
pub unsafe fn Roach_Func_5(k: uint16, game: &mut Game) {
    let E: *mut Enemy_Roach = Get_Roach(k as ::core::ffi::c_int, game);
    SetHiLo(
        &raw mut (*E).roach_var_09,
        &raw mut (*E).roach_var_08,
        CosineMult8bitFull(
            ((*(&raw mut (*E).roach_parameter_1 as *mut uint8).offset(
                (::core::mem::size_of::<uint16>())
                    .wrapping_div(::core::mem::size_of::<uint8>())
                    .wrapping_sub(1_usize) as isize,
            ))
            .wrapping_add(32u8)) as uint16,
            *(&raw mut (*E).roach_parameter_1 as *mut uint8).offset(0) as uint16,
            game,
        ),
    );
    SetHiLo(
        &raw mut (*E).roach_var_0B,
        &raw mut (*E).roach_var_0A,
        SineMult8bitFull(
            ((*(&raw mut (*E).roach_parameter_1 as *mut uint8).offset(
                (::core::mem::size_of::<uint16>())
                    .wrapping_div(::core::mem::size_of::<uint8>())
                    .wrapping_sub(1_usize) as isize,
            ))
            .wrapping_add(32u8)) as uint16,
            *(&raw mut (*E).roach_parameter_1 as *mut uint8).offset(0) as uint16,
            game,
        ),
    );
}
pub unsafe fn Roach_Func_6(k: uint16, game: &mut Game) {
    let E: *mut Enemy_Roach = Get_Roach(k as ::core::ffi::c_int, game);
    (*E).roach_var_20 = (2 as ::core::ffi::c_int
        * (((*(&raw mut (*E).roach_parameter_1 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1_usize) as isize,
        ))
        .wrapping_sub(48u8)) as ::core::ffi::c_int
            >> 5 as ::core::ffi::c_int)) as uint16;
}
pub unsafe fn Roach_Func_7(k: uint16, game: &mut Game) {
    let E: *mut Enemy_Roach = Get_Roach(k as ::core::ffi::c_int, game);
    (*E).roach_var_21 = (2 as ::core::ffi::c_int
        * (((*(&raw mut (*E).roach_parameter_1 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1_usize) as isize,
        ))
        .wrapping_sub(80u8)) as ::core::ffi::c_int
            >> 5 as ::core::ffi::c_int)) as uint16;
}
pub unsafe fn Roach_Func_8(k: uint16, game: &mut Game) {
    let E: *mut Enemy_Roach = Get_Roach(k as ::core::ffi::c_int, game);
    (*E).roach_var_22 = (2 as ::core::ffi::c_int
        * ((*(&raw mut (*E).roach_parameter_1 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1_usize) as isize,
        ) as ::core::ffi::c_int
            - 48 as ::core::ffi::c_int
            + 32 as ::core::ffi::c_int) as uint8 as ::core::ffi::c_int
            >> 5 as ::core::ffi::c_int)) as uint16;
}
pub unsafe fn CallRoachFunc(ea: uint32, game: &mut Game) {
    match ea {
        10724055 => Roach_Func_9(game),
        10724097 => Roach_Func_10(game),
        10724107 => Roach_Func_11(game),
        10724117 => Roach_Func_12(game),
        10724133 => Roach_Func_13(game),
        10724155 => Roach_Func_14(game),
        10724171 => Roach_Func_15(game),
        10724224 => Roach_Func_16(game),
        10724359 => Roach_Func_19(game),
        10724366 => Roach_Func_20(game),
        10724416 => Roach_Func_21(game),
        10724423 => Roach_Func_22(game),
        10724430 => Roach_Func_23(game),
        10724450 => Roach_Func_24(game),
        10724470 => Roach_Func_25(game),
        10724534 => Roach_Func_26(game),
        10724592 => Roach_Func_27(game),
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn Roach_Main(game: &mut Game) {
    let E: *mut Enemy_Roach =
        Get_Roach(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    CallRoachFunc((((*E).roach_var_B) as u32) | (0xa30000u32), game);
}
pub unsafe fn Roach_Func_9(game: &mut Game) {
    let E: *mut Enemy_Roach =
        Get_Roach(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    if IsSamusWithinEnemy_X(
        ram_word!(game, cur_enemy_index),
        *(&raw mut (*E).roach_parameter_2 as *mut uint8).offset(0) as uint16,
        game,
    ) != 0
        && IsSamusWithinEnemy_Y(
            ram_word!(game, cur_enemy_index),
            *(&raw mut (*E).roach_parameter_2 as *mut uint8).offset(0) as uint16,
            game,
        ) != 0
    {
        (*E).roach_var_B = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3a121)) as *const uint8
            as *mut uint16)
            .offset(
                *(&raw mut (*E).roach_parameter_2 as *mut uint8).offset(
                    (::core::mem::size_of::<uint16>())
                        .wrapping_div(::core::mem::size_of::<uint8>())
                        .wrapping_sub(1_usize) as isize,
                ) as isize,
            );
    }
}
pub unsafe fn Roach_Func_10(game: &mut Game) {
    (*Get_Roach(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game)).roach_var_B =
        ((0xa3a407i32) as u16);
}
pub unsafe fn Roach_Func_11(game: &mut Game) {
    (*Get_Roach(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game)).roach_var_B =
        ((0xa3a40ei32) as u16);
}
pub unsafe fn Roach_Func_12(game: &mut Game) {
    set_ram_word!(game.random_number = 11 as uint16);
    (*Get_Roach(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game)).roach_var_B =
        ((0xa3a4f0i32) as u16);
}
pub unsafe fn Roach_Func_13(game: &mut Game) {
    let E: *mut Enemy_Roach =
        Get_Roach(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).roach_var_A = 512 as uint16;
    set_ram_word!(game.random_number = 11 as uint16);
    (*E).roach_var_B = ((0xa3a4b6i32) as u16);
}
pub unsafe fn Roach_Func_14(game: &mut Game) {
    let E: *mut Enemy_Roach =
        Get_Roach(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).roach_var_F = 32 as uint16;
    (*E).roach_var_B = ((0xa3a44ei32) as u16);
}
pub unsafe fn Roach_Func_15(game: &mut Game) {
    let v1: uint8 = (64u8).wrapping_sub(
        (CalculateAngleOfSamusFromEnemy(ram_word!(game, cur_enemy_index), game)) as u8,
    );
    let E: *mut Enemy_Roach =
        Get_Roach(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).roach_var_C = v1 as uint16;
    Roach_Func_18(ram_word!(game, cur_enemy_index), game);
    Roach_Func_17(ram_word!(game, cur_enemy_index), game);
    (*E).roach_var_24 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3a111)) as *const uint8
        as *mut uint16)
        .offset(((*E).roach_var_23 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    Roach_Func_1(game);
    (*E).roach_var_B = ((0xa3a440i32) as u16);
}
pub unsafe fn Roach_Func_16(game: &mut Game) {
    let v1: uint16 = (64 as ::core::ffi::c_int
        - CalculateAngleOfSamusFromEnemy(ram_word!(game, cur_enemy_index), game)
            as ::core::ffi::c_int
        + 0x80 as ::core::ffi::c_int) as uint8 as uint16;
    let E: *mut Enemy_Roach =
        Get_Roach(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).roach_var_C = v1;
    Roach_Func_18(ram_word!(game, cur_enemy_index), game);
    Roach_Func_17(ram_word!(game, cur_enemy_index), game);
    (*E).roach_var_24 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3a111)) as *const uint8
        as *mut uint16)
        .offset(((*E).roach_var_23 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    Roach_Func_1(game);
    (*E).roach_var_B = ((0xa3a447i32) as u16);
}
pub unsafe fn Roach_Func_17(k: uint16, game: &mut Game) {
    let E: *mut Enemy_Roach = Get_Roach(k as ::core::ffi::c_int, game);
    (*E).roach_var_23 = (2 as ::core::ffi::c_int
        * (((((*E).roach_var_C) as u8).wrapping_sub(48u8)) as ::core::ffi::c_int
            >> 5 as ::core::ffi::c_int)) as uint16;
}
pub unsafe fn Roach_Func_18(k: uint16, game: &mut Game) {
    let E: *mut Enemy_Roach = Get_Roach(k as ::core::ffi::c_int, game);
    SetHiLo(
        &raw mut (*E).roach_var_0D,
        &raw mut (*E).roach_var_0C,
        CosineMult8bitFull(
            (*E).roach_var_C,
            *(&raw mut (*E).roach_parameter_1 as *mut uint8).offset(0) as uint16,
            game,
        ),
    );
    SetHiLo(
        &raw mut (*E).roach_var_0F,
        &raw mut (*E).roach_var_0E,
        SineMult8bitFull(
            (*E).roach_var_C,
            *(&raw mut (*E).roach_parameter_1 as *mut uint8).offset(0) as uint16,
            game,
        ),
    );
}
pub unsafe fn Roach_Func_19(game: &mut Game) {
    Roach_Func_29(ram_word!(game, cur_enemy_index), game);
}
pub unsafe fn Roach_Func_20(game: &mut Game) {
    let E: *mut Enemy_Roach =
        Get_Roach(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    if ((*E).base.frame_counter as ::core::ffi::c_int & 0x10 as ::core::ffi::c_int) != 0i32 {
        (*E).roach_var_24 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3a111)) as *const uint8
            as *mut uint16)
            .offset(((*E).roach_var_22 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        Roach_Func_1(game);
        Roach_Func_31(ram_word!(game, cur_enemy_index), game);
    } else {
        (*E).roach_var_24 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3a111)) as *const uint8
            as *mut uint16)
            .offset(((*E).roach_var_21 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        Roach_Func_1(game);
        Roach_Func_30(ram_word!(game, cur_enemy_index), game);
    };
}
pub unsafe fn Roach_Func_21(game: &mut Game) {
    Roach_Func_32(ram_word!(game, cur_enemy_index), game);
}
pub unsafe fn Roach_Func_22(game: &mut Game) {
    Roach_Func_32(ram_word!(game, cur_enemy_index), game);
}
pub unsafe fn Roach_Func_23(game: &mut Game) {
    let E: *mut Enemy_Roach =
        Get_Roach(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).roach_var_F = (*E).roach_var_F.wrapping_sub(1);
    if ((*E).roach_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        (*E).roach_var_B = ((0xa3a2d7i32) as u16);
    } else {
        Roach_Func_29(ram_word!(game, cur_enemy_index), game);
    };
}
pub unsafe fn Roach_Func_24(game: &mut Game) {
    let E: *mut Enemy_Roach =
        Get_Roach(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).roach_var_F = (*E).roach_var_F.wrapping_sub(1);
    if ((*E).roach_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        (*E).roach_var_B = ((0xa3a4f0i32) as u16);
    } else {
        Roach_Func_33(ram_word!(game, cur_enemy_index), game);
    };
}
pub unsafe fn Roach_Func_25(game: &mut Game) {
    let E: *mut Enemy_Roach =
        Get_Roach(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).roach_var_A = (*E).roach_var_A.wrapping_sub(1);
    let mut v2: bool =
        ((*E).roach_var_A as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32;
    if !v2 {
        (*E).roach_var_F = (*E).roach_var_F.wrapping_sub(1);
        v2 = ((*E).roach_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32;
        if !v2 {
            let v3: uint16 = Abs16(((*E).base.x_pos).wrapping_sub(ram_word!(game, samus_x_pos)));
            if v3 as ::core::ffi::c_int - 96 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                == 0
            {
                let v4: uint16 =
                    Abs16(((*E).base.y_pos).wrapping_sub(ram_word!(game, samus_y_pos)));
                if v4 as ::core::ffi::c_int - 96 as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    == 0
                {
                    Roach_Func_28(game);
                }
            }
        } else {
            (*E).roach_var_B = ((0xa3a4b6i32) as u16);
            return;
        }
    }
    Roach_Func_32(ram_word!(game, cur_enemy_index), game);
}
pub unsafe fn Roach_Func_26(game: &mut Game) {
    let v1: uint8 = ((NextRandom(game)) as u8).wrapping_sub(64u8);
    let E: *mut Enemy_Roach =
        Get_Roach(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).roach_var_C = ((*E).roach_var_C).wrapping_add((v1) as u16);
    Roach_Func_18(ram_word!(game, cur_enemy_index), game);
    Roach_Func_17(ram_word!(game, cur_enemy_index), game);
    (*E).roach_var_24 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3a111)) as *const uint8
        as *mut uint16)
        .offset(((*E).roach_var_23 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    Roach_Func_1(game);
    (*E).roach_var_F = 32 as uint16;
    (*E).roach_var_B = ((0xa3a476i32) as u16);
}
pub unsafe fn Roach_Func_27(game: &mut Game) {
    let v1: uint8 = ((NextRandom(game)) as u8).wrapping_sub(64u8);
    let E: *mut Enemy_Roach =
        Get_Roach(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).roach_var_C = ((*E).roach_var_C).wrapping_add((v1) as u16);
    Roach_Func_18(ram_word!(game, cur_enemy_index), game);
    Roach_Func_17(ram_word!(game, cur_enemy_index), game);
    (*E).roach_var_24 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3a111)) as *const uint8
        as *mut uint16)
        .offset(((*E).roach_var_23 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    Roach_Func_1(game);
    (*E).roach_var_F = 32 as uint16;
    (*E).roach_var_B = ((0xa3a462i32) as u16);
}
pub unsafe fn Roach_Func_28(game: &mut Game) {
    let E: *mut Enemy_Roach =
        Get_Roach(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).roach_var_0D = -((*E).roach_var_0D as ::core::ffi::c_int) as uint16;
    (*E).roach_var_0C = -((*E).roach_var_0C as ::core::ffi::c_int) as uint16;
    (*E).roach_var_0F = -((*E).roach_var_0F as ::core::ffi::c_int) as uint16;
    (*E).roach_var_0E = -((*E).roach_var_0E as ::core::ffi::c_int) as uint16;
    let v1: uint16 = ((*E).roach_var_23 as ::core::ffi::c_int + 4 as ::core::ffi::c_int
        & 7 as ::core::ffi::c_int) as uint16;
    (*E).roach_var_23 = v1;
    (*E).roach_var_24 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3a111)) as *const uint8
        as *mut uint16)
        .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    Roach_Func_1(game);
}
pub unsafe fn Roach_Func_29(k: uint16, game: &mut Game) {
    let E: *mut Enemy_Roach = Get_Roach(k as ::core::ffi::c_int, game);
    let varE20: uint16 = *(&raw mut (*E).roach_parameter_1 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>())
            .wrapping_div(::core::mem::size_of::<uint8>())
            .wrapping_sub(1_usize) as isize,
    ) as uint16;
    EnemyFunc_B691(
        varE20,
        Point32 {
            x: ((*E).roach_var_01 as uint32) << 16 as ::core::ffi::c_int
                | (*E).roach_var_00 as uint32,
            y: ((*E).roach_var_03 as uint32) << 16 as ::core::ffi::c_int
                | (*E).roach_var_02 as uint32,
        },
        game,
    );
}
pub unsafe fn Roach_Func_30(k: uint16, game: &mut Game) {
    let E: *mut Enemy_Roach = Get_Roach(k as ::core::ffi::c_int, game);
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        ((*E).roach_var_05 as uint32) << 16 as ::core::ffi::c_int | (*E).roach_var_04 as uint32,
    );
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        ((*E).roach_var_07 as uint32) << 16 as ::core::ffi::c_int | (*E).roach_var_06 as uint32,
    );
}
pub unsafe fn Roach_Func_31(k: uint16, game: &mut Game) {
    let E: *mut Enemy_Roach = Get_Roach(k as ::core::ffi::c_int, game);
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        ((*E).roach_var_09 as uint32) << 16 as ::core::ffi::c_int | (*E).roach_var_08 as uint32,
    );
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        ((*E).roach_var_0B as uint32) << 16 as ::core::ffi::c_int | (*E).roach_var_0A as uint32,
    );
}
pub unsafe fn Roach_Func_32(k: uint16, game: &mut Game) {
    let E: *mut Enemy_Roach = Get_Roach(k as ::core::ffi::c_int, game);
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        ((*E).roach_var_0D as uint32) << 16 as ::core::ffi::c_int | (*E).roach_var_0C as uint32,
    );
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        ((*E).roach_var_0F as uint32) << 16 as ::core::ffi::c_int | (*E).roach_var_0E as uint32,
    );
}
pub unsafe fn Roach_Func_33(k: uint16, game: &mut Game) {
    let E: *mut Enemy_Roach = Get_Roach(k as ::core::ffi::c_int, game);
    if Enemy_MoveRight_IgnoreSlopes(
        k,
        (((*E).roach_var_0D as uint32) << 16 as ::core::ffi::c_int | (*E).roach_var_0C as uint32)
            as int32,
        game,
    ) != 0
    {
        (*E).roach_var_B = ((0xa3a2d7i32) as u16);
    } else if Enemy_MoveDown(
        k,
        (((*E).roach_var_0F as uint32) << 16 as ::core::ffi::c_int | (*E).roach_var_0E as uint32)
            as int32,
        game,
    ) != 0
    {
        (*E).roach_var_B = ((0xa3a2d7i32) as u16);
    }
}
pub unsafe fn Mochtroid_Init(game: &mut Game) {
    let E: *mut Enemy_Mochtroid =
        Get_Mochtroid(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).base.layer = 2 as uint16;
    Mochtroid_Func_4(
        ram_word!(game, cur_enemy_index),
        addr_kMochtroid_Ilist_A745 as uint16,
        game,
    );
    (*E).mochtr_var_F = 0 as uint16;
}
static off_A3A7A4: [Option<Func_V>; 3] = unsafe {
    [
        Some(Mochtroid_Func_1 as unsafe fn(&mut Game) -> ()),
        Some(Mochtroid_Func_3 as unsafe fn(&mut Game) -> ()),
        Some(Mochtroid_Func_2 as unsafe fn(&mut Game) -> ()),
    ]
};
pub unsafe fn Mochtroid_Main(game: &mut Game) {
    let v0: uint16 = (2u16).wrapping_mul(
        (*Get_Mochtroid(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game)).mochtr_var_F,
    );
    (*Get_Mochtroid(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game)).mochtr_var_F =
        0 as uint16;
    off_A3A7A4[(v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize]
        .expect("non-null function pointer")(game);
}
pub unsafe fn Mochtroid_Func_1(game: &mut Game) {
    let mut v5: int16 = 0;
    let mut v10: int16 = 0;
    let E: *mut Enemy_Mochtroid =
        Get_Mochtroid(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let mut t: int32 = ((((((*E).base.y_pos) as i16)
        .wrapping_sub((ram_word!(game, samus_y_pos)) as i16))
        as ::core::ffi::c_int
        >> 2 as ::core::ffi::c_int) as int16 as int32)
        << 8 as ::core::ffi::c_int;
    AddToHiLo(
        &raw mut (*E).mochtr_var_D,
        &raw mut (*E).mochtr_var_C,
        -t as uint32,
    );
    let current_block_4: u64;
    if ((*E).mochtr_var_D as int16) < 0i16 {
        if (*E).mochtr_var_D < 0xfffdu16 {
            v5 = -(3 as ::core::ffi::c_int) as int16;
            current_block_4 = 13796420526896809451;
        } else {
            current_block_4 = 8515828400728868193;
        }
    } else if (*E).mochtr_var_D >= 3u16 {
        v5 = 3 as int16;
        current_block_4 = 13796420526896809451;
    } else {
        current_block_4 = 8515828400728868193;
    }
    if current_block_4 == 13796420526896809451 {
        (*E).mochtr_var_D = v5 as uint16;
        (*E).mochtr_var_C = 0 as uint16;
    }
    if Enemy_MoveDown(
        ram_word!(game, cur_enemy_index),
        (((*E).mochtr_var_D as uint32) << 16 as ::core::ffi::c_int | (*E).mochtr_var_C as uint32)
            as int32,
        game,
    ) != 0
    {
        (*E).mochtr_var_C = 0 as uint16;
        (*E).mochtr_var_D = 0 as uint16;
    }
    t = ((((((*E).base.x_pos) as i16).wrapping_sub((ram_word!(game, samus_x_pos)) as i16))
        as ::core::ffi::c_int
        >> 2 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int;
    AddToHiLo(
        &raw mut (*E).mochtr_var_B,
        &raw mut (*E).mochtr_var_A,
        -t as uint32,
    );
    let current_block_15: u64;
    if ((*E).mochtr_var_B as int16) < 0i16 {
        if (*E).mochtr_var_B < 0xfffdu16 {
            v10 = -(3 as ::core::ffi::c_int) as int16;
            current_block_15 = 10272573711837802396;
        } else {
            current_block_15 = 4808432441040389987;
        }
    } else if (*E).mochtr_var_B >= 3u16 {
        v10 = 3 as int16;
        current_block_15 = 10272573711837802396;
    } else {
        current_block_15 = 4808432441040389987;
    }
    if current_block_15 == 10272573711837802396 {
        (*E).mochtr_var_B = v10 as uint16;
        (*E).mochtr_var_A = 0 as uint16;
    }
    if Enemy_MoveRight_IgnoreSlopes(
        ram_word!(game, cur_enemy_index),
        (((*E).mochtr_var_B as uint32) << 16 as ::core::ffi::c_int | (*E).mochtr_var_A as uint32)
            as int32,
        game,
    ) != 0
    {
        (*E).mochtr_var_A = 0 as uint16;
        (*E).mochtr_var_B = 0 as uint16;
    }
    Mochtroid_Func_4(
        ram_word!(game, cur_enemy_index),
        addr_kMochtroid_Ilist_A745 as uint16,
        game,
    );
}
pub unsafe fn Mochtroid_Func_2(game: &mut Game) {
    let E: *mut Enemy_Mochtroid =
        Get_Mochtroid(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let v2: ::core::ffi::c_int = ((*E).mochtr_var_E as ::core::ffi::c_int
        & 6 as ::core::ffi::c_int)
        >> 1 as ::core::ffi::c_int;
    (*E).base.x_pos = ((*E).base.x_pos).wrapping_add((g_word_A3A76D[v2 as usize]) as u16);
    (*E).base.y_pos = ((*E).base.y_pos).wrapping_add((g_word_A3A775[v2 as usize]) as u16);
    (*E).mochtr_var_A = 0 as uint16;
    (*E).mochtr_var_B = 0 as uint16;
    (*E).mochtr_var_C = 0 as uint16;
    (*E).mochtr_var_D = 0 as uint16;
    let fresh12 = (*E).mochtr_var_E;
    (*E).mochtr_var_E = (*E).mochtr_var_E.wrapping_sub(1);
    if fresh12 == 1u16 {
        (*E).mochtr_var_F = 0 as uint16;
    }
    Mochtroid_Func_4(
        ram_word!(game, cur_enemy_index),
        addr_kMochtroid_Ilist_A745 as uint16,
        game,
    );
}
pub unsafe fn Mochtroid_Func_3(game: &mut Game) {
    let E: *mut Enemy_Mochtroid =
        Get_Mochtroid(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let x_pos: uint16 = (*E).base.x_pos;
    if x_pos == ram_word!(game, samus_x_pos) {
        (*E).mochtr_var_A = 0 as uint16;
        (*E).mochtr_var_B = 0 as uint16;
    } else if ((x_pos) as i16).wrapping_sub((ram_word!(game, samus_x_pos)) as i16) >= 0i16 {
        (*E).mochtr_var_A = 0 as uint16;
        (*E).mochtr_var_B = -(1 as ::core::ffi::c_int) as uint16;
    } else {
        (*E).mochtr_var_A = 0 as uint16;
        (*E).mochtr_var_B = 1 as uint16;
    }
    Enemy_MoveRight_IgnoreSlopes(
        ram_word!(game, cur_enemy_index),
        (((*E).mochtr_var_B as uint32) << 16 as ::core::ffi::c_int | (*E).mochtr_var_A as uint32)
            as int32,
        game,
    );
    let y_pos: uint16 = (*E).base.y_pos;
    if y_pos == ram_word!(game, samus_y_pos) {
        (*E).mochtr_var_C = 0 as uint16;
        (*E).mochtr_var_D = 0 as uint16;
    } else if ((y_pos) as i16).wrapping_sub((ram_word!(game, samus_y_pos)) as i16) >= 0i16 {
        (*E).mochtr_var_C = 0 as uint16;
        (*E).mochtr_var_D = -(1 as ::core::ffi::c_int) as uint16;
    } else {
        (*E).mochtr_var_C = 0 as uint16;
        (*E).mochtr_var_D = 1 as uint16;
    }
    Enemy_MoveDown(
        ram_word!(game, cur_enemy_index),
        (((*E).mochtr_var_D as uint32) << 16 as ::core::ffi::c_int | (*E).mochtr_var_C as uint32)
            as int32,
        game,
    );
}
pub unsafe fn Mochtroid_Func_4(k: uint16, a: uint16, game: &mut Game) {
    let E: *mut Enemy_Mochtroid = Get_Mochtroid(k as ::core::ffi::c_int, game);
    if a != (*E).mochtr_var_01 {
        (*E).mochtr_var_01 = a;
        (*E).base.current_instruction = a;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
    }
}
pub unsafe fn Mochtroid_Touch(game: &mut Game) {
    let current_block: u64;
    let E: *mut Enemy_Mochtroid =
        Get_Mochtroid(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).mochtr_var_F = 1 as uint16;
    Mochtroid_Func_4(
        ram_word!(game, cur_enemy_index),
        addr_kMochtroid_Ilist_A759 as uint16,
        game,
    );
    (*E).mochtr_var_20 = (*E).mochtr_var_20.wrapping_add(1);
    if ram_word!(game, samus_contact_damage_index) != 0 {
        current_block = 17285009320546006530;
    } else {
        if (ram_word!(game, random_enemy_counter) as ::core::ffi::c_int & 7 as ::core::ffi::c_int)
            == 7i32
            && ram_word!(game, samus_health) as ::core::ffi::c_int - 30 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0
        {
            QueueSfx3_Max6(0x2d as uint16, game);
        }
        if (*E).mochtr_var_20 as ::core::ffi::c_int - 80 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            (*E).mochtr_var_20 = 0 as uint16;
            current_block = 17285009320546006530;
        } else {
            current_block = 13513818773234778473;
        }
    }
    if current_block == 17285009320546006530 {
        Enemy_NormalTouchAI_A3(game);
        set_ram_word!(game.samus_invincibility_timer = 0 as uint16);
        set_ram_word!(game.samus_knockback_timer = 0 as uint16);
    };
}
pub unsafe fn Mochtroid_Shot(game: &mut Game) {
    Enemy_NormalShotAI_A3(game);
}
pub unsafe fn Sidehopper_Func_1(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    QueueSfx2_Max3(*jp, game);
    jp.offset(1)
}
pub unsafe fn Sidehopper_Instr_1(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    (*Get_Sidehopper(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game)).sideh_var_04 =
        1 as uint16;
    jp
}
pub unsafe fn Sidehopper_Init(game: &mut Game) {
    set_ram_word!(game.random_number = 37 as uint16);
    NextRandom(game);
    let E: *mut Enemy_Sidehopper =
        Get_Sidehopper(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).sideh_var_03 = 0 as uint16;
    (*E).sideh_var_04 = 0 as uint16;
    (*E).sideh_var_05 = 0 as uint16;
    let v2: uint16 = (2u16).wrapping_mul((*get_EnemyDef_A2((*E).base.enemy_ptr, game)).field_2A);
    (*E).sideh_var_06 = v2;
    let mut v4: uint16 = 0;
    if (*E).sideh_parameter_1 != 0 {
        v4 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3aaca)) as *const uint8
            as *mut uint16)
            .offset(((*E).sideh_var_06 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    } else {
        v4 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3aac2)) as *const uint8
            as *mut uint16)
            .offset(((*E).sideh_var_06 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    }
    (*E).sideh_var_00 = v4;
    Sidehopper_Func_3(game);
    if (*get_EnemyDef_A2((*E).base.enemy_ptr, game)).field_2A != 0 {
        (*E).sideh_var_05 = 2 as uint16;
    }
    let v6: ::core::ffi::c_int = (*E).sideh_var_05 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    (*E).sideh_var_01 = Sidehopper_Func_2(
        *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3aaee)) as *const uint8 as *mut uint16)
            .offset(v6 as isize),
        *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3aae6)) as *const uint8 as *mut uint16)
            .offset(v6 as isize),
        game,
    );
    let v8: ::core::ffi::c_int = (*E).sideh_var_05 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    (*E).sideh_var_02 = Sidehopper_Func_2(
        *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3aafa)) as *const uint8 as *mut uint16)
            .offset(v8 as isize),
        *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3aaf2)) as *const uint8 as *mut uint16)
            .offset(v8 as isize),
        game,
    );
    (*E).sideh_var_B = ((0xa3abd6i32) as u16);
}
pub unsafe fn Sidehopper_Func_2(r22: uint16, r24: uint16, game: &mut Game) -> uint16 {
    let mut r18: uint16 = 0 as uint16;
    let mut r20: uint16 = 0 as uint16;
    loop {
        r18 = (r18).wrapping_add(r24);
        r20 = (r20).wrapping_add(
            *((game.g_rom.offset(crate::rtl::lorom_offset(0xa2838f)) as *const uint8 as *mut uint16
                as *mut uint8)
                .offset((8 as ::core::ffi::c_int * r18 as ::core::ffi::c_int) as isize)
                .offset(1) as *mut uint16),
        );
        if (r20 as ::core::ffi::c_int - r22 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            == 0)
        {
            break;
        }
    }
    r18
}
pub unsafe fn Sidehopper_Func_3(game: &mut Game) {
    let E: *mut Enemy_Sidehopper =
        Get_Sidehopper(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).base.current_instruction = (*E).sideh_var_00;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
}
pub unsafe fn Sidehopper_Main(game: &mut Game) {
    let E: *mut Enemy_Sidehopper =
        Get_Sidehopper(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    CallSidehopperFunc((((*E).sideh_var_B) as u32) | (0xa30000u32), game);
}
pub unsafe fn Sidehopper_Func_4(k: uint16, game: &mut Game) {
    let v1: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3aae2)) as *const uint8
        as *mut uint16)
        .offset((NextRandom(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int) as isize);
    (*Get_Sidehopper(k as ::core::ffi::c_int, game)).sideh_var_B = v1;
}
pub unsafe fn Sidehopper_Func_5(game: &mut Game) {
    let E: *mut Enemy_Sidehopper =
        Get_Sidehopper(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let v1: ::core::ffi::c_int = (*E).sideh_var_05 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    (*E).sideh_var_E = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3aae6)) as *const uint8
        as *mut uint16)
        .offset(v1 as isize);
    (*E).sideh_var_D = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3aaea)) as *const uint8
        as *mut uint16)
        .offset(v1 as isize);
    (*E).sideh_var_C = (*E).sideh_var_01;
    (*E).sideh_var_B = ((0xa3ac40i32) as u16);
    if (*E).sideh_parameter_1 != 0 {
        (*E).sideh_var_B = ((0xa3ac56i32) as u16);
    }
}
pub unsafe fn Sidehopper_Func_6(game: &mut Game) {
    let E: *mut Enemy_Sidehopper =
        Get_Sidehopper(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let v1: ::core::ffi::c_int = (*E).sideh_var_05 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    (*E).sideh_var_E = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3aaf2)) as *const uint8
        as *mut uint16)
        .offset(v1 as isize);
    (*E).sideh_var_D = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3aaf6)) as *const uint8
        as *mut uint16)
        .offset(v1 as isize);
    (*E).sideh_var_C = (*E).sideh_var_02;
    (*E).sideh_var_B = ((0xa3ac40i32) as u16);
    if (*E).sideh_parameter_1 != 0 {
        (*E).sideh_var_B = ((0xa3ac56i32) as u16);
    }
}
pub unsafe fn Sidehopper_Func_7(game: &mut Game) {
    let E: *mut Enemy_Sidehopper =
        Get_Sidehopper(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).sideh_var_B = ((0xa3ac6ci32) as u16);
    if (GetSamusEnemyDelta_X(ram_word!(game, cur_enemy_index), game) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int)
        == 0i32
    {
        (*E).sideh_var_B = ((0xa3ac8fi32) as u16);
    }
}
pub unsafe fn Sidehopper_Func_8(game: &mut Game) {
    let E: *mut Enemy_Sidehopper =
        Get_Sidehopper(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).sideh_var_B = ((0xa3aca8i32) as u16);
    if (GetSamusEnemyDelta_X(ram_word!(game, cur_enemy_index), game) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int)
        == 0i32
    {
        (*E).sideh_var_B = ((0xa3accbi32) as u16);
    }
}
pub unsafe fn Sidehopper_Func_9(game: &mut Game) {
    let E: *mut Enemy_Sidehopper =
        Get_Sidehopper(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).sideh_var_D = -((*E).sideh_var_D as ::core::ffi::c_int) as uint16;
    (*E).sideh_var_00 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3aad2)) as *const uint8
        as *mut uint16)
        .offset(((*E).sideh_var_06 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    Sidehopper_Func_3(game);
    (*E).sideh_var_B = ((0xa3ad0ei32) as u16);
}
pub unsafe fn Sidehopper_Func_10(game: &mut Game) {
    let E: *mut Enemy_Sidehopper =
        Get_Sidehopper(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).sideh_var_00 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3aad2)) as *const uint8
        as *mut uint16)
        .offset(((*E).sideh_var_06 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    Sidehopper_Func_3(game);
    (*E).sideh_var_B = ((0xa3ad20i32) as u16);
}
pub unsafe fn Sidehopper_Func_11(game: &mut Game) {
    let E: *mut Enemy_Sidehopper =
        Get_Sidehopper(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).sideh_var_D = -((*E).sideh_var_D as ::core::ffi::c_int) as uint16;
    (*E).sideh_var_00 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3aada)) as *const uint8
        as *mut uint16)
        .offset(((*E).sideh_var_06 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    Sidehopper_Func_3(game);
    (*E).sideh_var_B = ((0xa3ad32i32) as u16);
}
pub unsafe fn Sidehopper_Func_12(game: &mut Game) {
    let E: *mut Enemy_Sidehopper =
        Get_Sidehopper(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).sideh_var_00 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3aada)) as *const uint8
        as *mut uint16)
        .offset(((*E).sideh_var_06 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    Sidehopper_Func_3(game);
    (*E).sideh_var_B = ((0xa3ad44i32) as u16);
}
pub unsafe fn Sidehopper_Func_13(game: &mut Game) {
    let E: *mut Enemy_Sidehopper =
        Get_Sidehopper(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).sideh_var_00 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3aac2)) as *const uint8
        as *mut uint16)
        .offset(((*E).sideh_var_06 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    if (*E).sideh_parameter_1 != 0 {
        (*E).sideh_var_00 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3aaca)) as *const uint8
            as *mut uint16)
            .offset(((*E).sideh_var_06 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    }
    Sidehopper_Func_3(game);
    (*E).sideh_var_B = ((0xa3ad56i32) as u16);
}
pub unsafe fn Sidehopper_Func_14(game: &mut Game) {
    if (*Get_Sidehopper(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game)).sideh_var_03
        != 0
    {
        Sidehopper_Func_20(game);
    } else {
        Sidehopper_Func_19(game);
    };
}
pub unsafe fn Sidehopper_Func_15(game: &mut Game) {
    if (*Get_Sidehopper(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game)).sideh_var_03
        != 0
    {
        Sidehopper_Func_20(game);
    } else {
        Sidehopper_Func_19(game);
    };
}
pub unsafe fn Sidehopper_Func_16(game: &mut Game) {
    if (*Get_Sidehopper(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game)).sideh_var_03
        != 0
    {
        Sidehopper_Func_22(game);
    } else {
        Sidehopper_Func_21(game);
    };
}
pub unsafe fn Sidehopper_Func_17(game: &mut Game) {
    if (*Get_Sidehopper(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game)).sideh_var_03
        != 0
    {
        Sidehopper_Func_22(game);
    } else {
        Sidehopper_Func_21(game);
    };
}
pub unsafe fn Sidehopper_Func_18(game: &mut Game) {
    let E: *mut Enemy_Sidehopper =
        Get_Sidehopper(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    if (*E).sideh_var_04 != 0 {
        (*E).sideh_var_04 = 0 as uint16;
        (*E).sideh_var_B = ((0xa3abd6i32) as u16);
    }
}
unsafe fn CallSidehopperFunc(ea: uint32, game: &mut Game) {
    match ea {
        10726358 => {
            Sidehopper_Func_4(ram_word!(game, cur_enemy_index), game);
        }
        10726374 => Sidehopper_Func_5(game),
        10726419 => Sidehopper_Func_6(game),
        10726464 => Sidehopper_Func_7(game),
        10726486 => Sidehopper_Func_8(game),
        10726508 => Sidehopper_Func_9(game),
        10726543 => Sidehopper_Func_10(game),
        10726568 => Sidehopper_Func_11(game),
        10726603 => Sidehopper_Func_12(game),
        10726628 => Sidehopper_Func_13(game),
        10726670 => Sidehopper_Func_14(game),
        10726688 => Sidehopper_Func_15(game),
        10726706 => Sidehopper_Func_16(game),
        10726724 => Sidehopper_Func_17(game),
        10726742 => Sidehopper_Func_18(game),
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn Sidehopper_Func_19(game: &mut Game) {
    let E: *mut Enemy_Sidehopper =
        Get_Sidehopper(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let v2: ::core::ffi::c_int =
        8 as ::core::ffi::c_int * (*E).sideh_var_C as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if Enemy_MoveDown(
        ram_word!(game, cur_enemy_index),
        ((*(game.g_rom.offset(crate::rtl::lorom_offset(0xa2838f)) as *const uint8 as *mut uint16)
            .offset((v2 + 3 as ::core::ffi::c_int) as isize) as uint32)
            << 16 as ::core::ffi::c_int
            | *(game.g_rom.offset(crate::rtl::lorom_offset(0xa2838f)) as *const uint8
                as *mut uint16)
                .offset((v2 + 2 as ::core::ffi::c_int) as isize) as uint32) as int32,
        game,
    ) != 0
    {
        (*E).sideh_var_D = -((*E).sideh_var_D as ::core::ffi::c_int) as uint16;
        (*E).sideh_var_03 = 1 as uint16;
    } else if Enemy_MoveRight_IgnoreSlopes(
        ram_word!(game, cur_enemy_index),
        ((*E).sideh_var_D as int16 as int32) << 16 as ::core::ffi::c_int,
        game,
    ) != 0
    {
        (*E).sideh_var_D = -((*E).sideh_var_D as ::core::ffi::c_int) as uint16;
        (*E).sideh_var_03 = 1 as uint16;
    } else {
        let v3: int16 = (((*E).sideh_var_C) as i16).wrapping_sub(((*E).sideh_var_E) as i16);
        (*E).sideh_var_C = v3 as uint16;
        if v3 < 0i16 {
            (*E).sideh_var_03 = 1 as uint16;
            (*E).sideh_var_C = 0 as uint16;
        }
    };
}
pub unsafe fn Sidehopper_Func_20(game: &mut Game) {
    let E: *mut Enemy_Sidehopper =
        Get_Sidehopper(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let v2: ::core::ffi::c_int =
        8 as ::core::ffi::c_int * (*E).sideh_var_C as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if Enemy_MoveDown(
        ram_word!(game, cur_enemy_index),
        ((*(game.g_rom.offset(crate::rtl::lorom_offset(0xa2838f)) as *const uint8 as *mut uint16)
            .offset((v2 + 1 as ::core::ffi::c_int) as isize) as uint32)
            << 16 as ::core::ffi::c_int
            | *(game.g_rom.offset(crate::rtl::lorom_offset(0xa2838f)) as *const uint8
                as *mut uint16)
                .offset(v2 as isize) as uint32) as int32,
        game,
    ) != 0
    {
        (*E).sideh_var_03 = 0 as uint16;
        (*E).sideh_var_B = ((0xa3ace4i32) as u16);
    } else {
        if Enemy_MoveRight_IgnoreSlopes(
            ram_word!(game, cur_enemy_index),
            ((*E).sideh_var_D as int16 as int32) << 16 as ::core::ffi::c_int,
            game,
        ) != 0
        {
            (*E).sideh_var_D = -((*E).sideh_var_D as ::core::ffi::c_int) as uint16;
        }
        let mut v3: int16 = (((*E).sideh_var_E) as i16).wrapping_add(((*E).sideh_var_C) as i16);
        if v3 as ::core::ffi::c_int - 64 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
            v3 = 64 as int16;
        }
        (*E).sideh_var_C = v3 as uint16;
    };
}
pub unsafe fn Sidehopper_Func_21(game: &mut Game) {
    let E: *mut Enemy_Sidehopper =
        Get_Sidehopper(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let v2: ::core::ffi::c_int =
        8 as ::core::ffi::c_int * (*E).sideh_var_C as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if Enemy_MoveDown(
        ram_word!(game, cur_enemy_index),
        ((*(game.g_rom.offset(crate::rtl::lorom_offset(0xa2838f)) as *const uint8 as *mut uint16)
            .offset((v2 + 1 as ::core::ffi::c_int) as isize) as uint32)
            << 16 as ::core::ffi::c_int
            | *(game.g_rom.offset(crate::rtl::lorom_offset(0xa2838f)) as *const uint8
                as *mut uint16)
                .offset(v2 as isize) as uint32) as int32,
        game,
    ) != 0
    {
        (*E).sideh_var_D = -((*E).sideh_var_D as ::core::ffi::c_int) as uint16;
        (*E).sideh_var_03 = 1 as uint16;
    } else if Enemy_MoveRight_IgnoreSlopes(
        ram_word!(game, cur_enemy_index),
        ((*E).sideh_var_D as int16 as int32) << 16 as ::core::ffi::c_int,
        game,
    ) != 0
    {
        (*E).sideh_var_D = -((*E).sideh_var_D as ::core::ffi::c_int) as uint16;
        (*E).sideh_var_03 = 1 as uint16;
    } else {
        let v3: int16 = (((*E).sideh_var_C) as i16).wrapping_sub(((*E).sideh_var_E) as i16);
        (*E).sideh_var_C = v3 as uint16;
        if v3 < 0i16 {
            (*E).sideh_var_03 = 1 as uint16;
            (*E).sideh_var_C = 0 as uint16;
        }
    };
}
pub unsafe fn Sidehopper_Func_22(game: &mut Game) {
    let mut v3: int16 = 0;
    let E: *mut Enemy_Sidehopper =
        Get_Sidehopper(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let v2: ::core::ffi::c_int =
        8 as ::core::ffi::c_int * (*E).sideh_var_C as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if Enemy_MoveDown(
        ram_word!(game, cur_enemy_index),
        ((*(game.g_rom.offset(crate::rtl::lorom_offset(0xa2838f)) as *const uint8 as *mut uint16)
            .offset((v2 + 3 as ::core::ffi::c_int) as isize) as uint32)
            << 16 as ::core::ffi::c_int
            | *(game.g_rom.offset(crate::rtl::lorom_offset(0xa2838f)) as *const uint8
                as *mut uint16)
                .offset((v2 + 2 as ::core::ffi::c_int) as isize) as uint32) as int32,
        game,
    ) != 0
    {
        (*E).sideh_var_03 = 0 as uint16;
        (*E).sideh_var_B = ((0xa3ace4i32) as u16);
    } else {
        if Enemy_MoveRight_IgnoreSlopes(
            ram_word!(game, cur_enemy_index),
            ((*E).sideh_var_D as int16 as int32) << 16 as ::core::ffi::c_int,
            game,
        ) != 0
        {
            (*E).sideh_var_D = -((*E).sideh_var_D as ::core::ffi::c_int) as uint16;
        }
        v3 = (((*E).sideh_var_E) as i16).wrapping_add(((*E).sideh_var_C) as i16);
        if v3 as ::core::ffi::c_int - 64 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
            v3 = 64 as int16;
        }
        (*E).sideh_var_C = v3 as uint16;
    };
}
pub unsafe fn MaridiaRefillCandy_Instr_1(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_MaridiaRefillCandy(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game))
        .mrcy_var_00 = 4 as uint16;
    jp
}
pub unsafe fn MaridiaRefillCandy_Instr_2(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_MaridiaRefillCandy(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game))
        .mrcy_var_00 = 8 as uint16;
    jp
}
pub unsafe fn MaridiaRefillCandy_Instr_3(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_MaridiaRefillCandy(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game))
        .mrcy_var_00 = 12 as uint16;
    jp
}
pub unsafe fn MaridiaRefillCandy_Init(game: &mut Game) {
    let E: *mut Enemy_MaridiaRefillCandy =
        Get_MaridiaRefillCandy(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).mrcy_parameter_1 = ((0xa3b482i32) as u16);
    (*E).mrcy_var_D = 0 as uint16;
    (*E).mrcy_var_E = 0 as uint16;
    (*E).mrcy_var_00 = 0 as uint16;
    (*E).base.current_instruction = addr_kMaridiaRefillCandy_Ilist_B3C1 as uint16;
    (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Invisible) as u16);
    (*E).mrcy_var_B = (*E).base.x_pos;
    (*E).mrcy_var_C = (*E).base.y_pos;
}
pub unsafe fn CallMaridiaRefillCandyFunc(ea: uint32, k: uint16, game: &mut Game) {
    match ea {
        10728578 => MaridiaRefillCandy_Func_1(game),
        10728616 => MaridiaRefillCandy_Func_2(k, game),
        10728662 => MaridiaRefillCandy_Func_3(k, game),
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn MaridiaRefillCandy_Main(game: &mut Game) {
    let E: *mut Enemy_MaridiaRefillCandy =
        Get_MaridiaRefillCandy(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    CallMaridiaRefillCandyFunc(
        (((*E).mrcy_parameter_1) as u32) | (0xa30000u32),
        ram_word!(game, cur_enemy_index),
        game,
    );
}
pub unsafe fn MaridiaRefillCandy_Func_1(game: &mut Game) {
    let mut v1: int16 = 0;
    if IsSamusWithinEnemy_X(ram_word!(game, cur_enemy_index), 0x80 as uint16, game) != 0 {
        v1 = 1 as int16;
        if (GetSamusEnemyDelta_X(ram_word!(game, cur_enemy_index), game) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int)
            == 0i32
        {
            v1 = 3 as int16;
        }
        let E: *mut Enemy_MaridiaRefillCandy =
            Get_MaridiaRefillCandy(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
        (*E).mrcy_var_D = v1 as uint16;
        MaridiaRefillCandy_Func_4(game);
        (*E).mrcy_parameter_1 = ((0xa3b4a8i32) as u16);
    }
}
pub unsafe fn MaridiaRefillCandy_Func_2(k: uint16, game: &mut Game) {
    let E: *mut Enemy_MaridiaRefillCandy = Get_MaridiaRefillCandy(k as ::core::ffi::c_int, game);
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        & !(kEnemyProps_Invisible as ::core::ffi::c_int)) as uint16;
    if (GetSamusEnemyDelta_Y(k, game) as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32
    {
        AddToHiLo(
            &raw mut (*E).base.y_pos,
            &raw mut (*E).base.y_subpos,
            -(0x8000 as ::core::ffi::c_int) as uint32,
        );
    } else {
        (*E).mrcy_var_D = (*E).mrcy_var_D.wrapping_sub(1);
        MaridiaRefillCandy_Func_4(game);
        (*E).mrcy_var_F = 0 as uint16;
        (*E).mrcy_parameter_1 = ((0xa3b4d6i32) as u16);
    };
}
pub unsafe fn MaridiaRefillCandy_Func_3(k: uint16, game: &mut Game) {
    let E: *mut Enemy_MaridiaRefillCandy = Get_MaridiaRefillCandy(k as ::core::ffi::c_int, game);
    let v2: ::core::ffi::c_int = (*E).mrcy_var_00 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let v: int32 = ((*(game.g_rom.offset(crate::rtl::lorom_offset(0xa3b415)) as *const uint8
        as *mut uint16)
        .offset(v2 as isize) as uint32)
        << 16 as ::core::ffi::c_int
        | *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3b415)) as *const uint8 as *mut uint16)
            .offset((v2 + 1 as ::core::ffi::c_int) as isize) as uint32) as int32;
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        (if (*E).mrcy_var_D != 0u16 { v } else { -v }) as uint32,
    );
    if CheckIfEnemyIsOnScreen(game) == 0 {
        MaridiaRefillCandy_Func_4(game);
        return;
    }
    (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Invisible) as u16);
    (*E).base.x_pos = (*E).mrcy_var_B;
    (*E).base.y_pos = (*E).mrcy_var_C;
    (*E).mrcy_var_D = 0 as uint16;
    MaridiaRefillCandy_Func_4(game);
    (*E).mrcy_parameter_1 = ((0xa3b482i32) as u16);
}
pub unsafe fn MaridiaRefillCandy_Func_4(game: &mut Game) {
    let E: *mut Enemy_MaridiaRefillCandy =
        Get_MaridiaRefillCandy(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let mrcy_var_D: uint16 = (*E).mrcy_var_D;
    if mrcy_var_D != (*E).mrcy_var_E {
        (*E).mrcy_var_E = mrcy_var_D;
        (*E).base.current_instruction = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3b40d))
            as *const uint8 as *mut uint16)
            .offset(mrcy_var_D as isize);
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
    }
}
pub fn MaridiaRefillCandy_Func_5() {}
pub fn MaridiaRefillCandy_Func_6() {}
pub unsafe fn NorfairSlowFireball_Init(game: &mut Game) {
    let v1: *mut EnemyData =
        gEnemyData(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*v1).parameter_2 = 6 as uint16;
    (*v1).current_instruction = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3b667))
        as *const uint8 as *mut uint16)
        .offset(((*v1).properties as ::core::ffi::c_int & 3 as ::core::ffi::c_int) as isize);
    StoneZoomer_E67A(ram_word!(game, cur_enemy_index), game);
}
pub unsafe fn NorfairSlowFireball_Func_1(game: &mut Game) {
    let current_instruction: uint16 =
        (*gEnemyData(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game))
            .current_instruction;
    if current_instruction != 0 {
        Unreachable(game);
    }
}
pub unsafe fn Bang_Instr_1(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    QueueSfx2_Max6(0x56 as uint16, game);
    jp
}
pub unsafe fn Bang_Instr_2(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    (*Get_Bang(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game)).bang_var_22 =
        1 as uint16;
    jp
}
pub unsafe fn Bang_Init(game: &mut Game) {
    let E: *mut Enemy_Bang = Get_Bang(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).bang_var_B = (*E).base.palette_index;
    (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_BlockPlasmaBeam) as u16);
    (*E).bang_var_F = ((0xa3bca5i32) as u16);
    (*E).bang_var_00 = 16 as uint16;
    (*E).bang_var_01 = 0 as uint16;
    (*E).bang_var_02 = 0 as uint16;
    (*E).bang_var_20 = 0 as uint16;
    (*E).bang_var_21 = 0 as uint16;
    (*E).bang_var_22 = 0 as uint16;
    (*E).bang_var_0B = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3ba84)) as *const uint8
        as *mut uint16)
        .offset(*(&raw mut (*E).bang_parameter_2 as *mut uint8).offset(0) as isize);
    let v1: ::core::ffi::c_int = 2 as ::core::ffi::c_int
        * *(&raw mut (*E).bang_parameter_2 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1_usize) as isize,
        ) as ::core::ffi::c_int;
    let v2: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3bc6a)) as *const uint8
        as *mut uint16)
        .offset(v1 as isize);
    (*E).bang_var_0C = v2;
    (*E).bang_var_0D = v2;
    (*E).bang_var_0E = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3bc6a)) as *const uint8
        as *mut uint16)
        .offset((v1 + 1 as ::core::ffi::c_int) as isize);
    if (*E).base.current_instruction == 0 {
        (*E).bang_var_F = ((0xa3bcc1i32) as u16);
    }
    (*E).base.current_instruction = addr_kBang_Ilist_B75E as uint16;
}
pub unsafe fn Bang_Main(game: &mut Game) {
    let E: *mut Enemy_Bang = Get_Bang(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    Call((((*E).bang_parameter_1) as u32) | (0xa30000u32), game);
}
pub unsafe fn Bang_Func_1(game: &mut Game) {
    let v0: uint16 = ((*Get_Bang(
        ram_word!(game, cur_enemy_index) as ::core::ffi::c_int + 64 as ::core::ffi::c_int,
        game,
    ))
    .bang_var_20)
        .wrapping_add(10u16);
    (*Get_Bang(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game)).bang_var_20 = v0;
    Bang_Func_18(game);
    let E: *mut Enemy_Bang = Get_Bang(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
}
pub unsafe fn Bang_Func_2(game: &mut Game) {
    let v0: uint16 = ((*Get_Bang(
        ram_word!(game, cur_enemy_index) as ::core::ffi::c_int + 1984 as ::core::ffi::c_int,
        game,
    ))
    .bang_var_00)
        .wrapping_add(20u16);
    let E: *mut Enemy_Bang = Get_Bang(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).bang_var_20 = v0;
    Bang_Func_18(game);
    (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
}
pub unsafe fn Bang_Func_3(game: &mut Game) {
    let mut bang_var_B: int16 = 0;
    Bang_Func_5(game);
    let E0: *mut Enemy_Bang =
        Get_Bang(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let x_pos: uint16 = (*E0).base.x_pos;
    let v3: ::core::ffi::c_int =
        ram_word!(game, cur_enemy_index) as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    set_ram_word!(game.enemy_drawing_queue[(v3 + 91i32)] = x_pos);
    let E1: *mut Enemy_Bang = Get_Bang(
        ram_word!(game, cur_enemy_index) as ::core::ffi::c_int + 64 as ::core::ffi::c_int,
        game,
    );
    (*E1).base.x_pos = x_pos;
    let y_pos: uint16 = (*E0).base.y_pos;
    set_ram_word!(game.enemy_drawing_queue[(v3 + 93i32)] = y_pos);
    (*E1).base.y_pos = y_pos;
    bang_var_B = (*E0).bang_var_B as int16;
    if ((*E0).bang_var_20 as ::core::ffi::c_int & 1 as ::core::ffi::c_int) != 0i32 {
        bang_var_B = 3072 as int16;
    }
    (*E0).base.palette_index = bang_var_B as uint16;
    Bang_Func_18(game);
    if (*E0).bang_var_22 != 0 {
        (*E0).bang_var_22 = 0 as uint16;
        if (*E0).bang_var_20 == 9u16 {
            (*E0).base.invincibility_timer = 16 as uint16;
            (*E0).base.properties = ((*E0).base.properties) | ((kEnemyProps_Tangible) as u16);
            let v7: uint16 = DetermineDirectionOfSamusFromEnemy(game);
            let v8: uint16 = Bang_Func_4(v7, game);
            EnemyDeathAnimation(ram_word!(game, cur_enemy_index), v8, game);
            let v9: uint16 = ram_word!(game, cur_enemy_index);
            let E: *mut Enemy_Bang = Get_Bang(
                ram_word!(game, cur_enemy_index) as ::core::ffi::c_int + 64 as ::core::ffi::c_int,
                game,
            );
            (*E).base.properties = ((*E).base.properties) | (0x200u16);
            let ref mut fresh1 =
                *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16).offset(
                    ((v9 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
                        + 97 as ::core::ffi::c_int) as isize,
                );
            *fresh1 = (*fresh1) | (0x200u16);
        } else {
            (*E0).bang_var_20 = (*E0).bang_var_20.wrapping_add(1);
            Bang_Func_18(game);
        }
    }
}
pub unsafe fn Bang_Func_4(a: uint16, game: &mut Game) -> uint16 {
    let mut i: ::core::ffi::c_int = 0;
    if ram_word!(game, projectile_counter) as ::core::ffi::c_int - 5 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        return 1 as uint16;
    }
    i = 0 as ::core::ffi::c_int;
    while ram_word!(game, projectile_damage[(i >> 1i32)]) != 0 {
        i += 2 as ::core::ffi::c_int;
    }
    let E: *mut Enemy_Bang = Get_Bang(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let v3: ::core::ffi::c_int = i >> 1 as ::core::ffi::c_int;
    set_ram_word!(game.projectile_x_pos[v3] = (*E).base.x_pos);
    set_ram_word!(game.projectile_y_pos[v3] = (*E).base.y_pos);
    set_ram_word!(game.projectile_dir[v3] = a);
    set_ram_word!(
        game.projectile_type[v3] = (ram_word!(game, equipped_beams) as ::core::ffi::c_int
            & 0xf as ::core::ffi::c_int
            | 0x10 as ::core::ffi::c_int) as uint16
    );
    let ref mut fresh0 = *((&raw mut game.g_ram as *mut uint8).offset(0xcce) as *mut uint16);
    *fresh0 = (*fresh0).wrapping_add(1);
    ProjectileReflection(i as uint16, game);
    set_ram_word!(game.projectile_damage[v3] = (*E).bang_var_E);
    QueueSfx1_Max6(
        *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3bc4a)) as *const uint8 as *mut uint16)
            .offset(
                (ram_word!(game, projectile_type[v3]) as ::core::ffi::c_int
                    & 0xf as ::core::ffi::c_int) as isize,
            ),
        game,
    );
    0 as uint16
}
pub unsafe fn Bang_Func_5(game: &mut Game) {
    let E: *mut Enemy_Bang = Get_Bang(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    CallEnemyPreInstr((((*E).bang_var_F) as u32) | (0xa30000u32), game);
}
pub unsafe fn Bang_Func_6(k: uint16, game: &mut Game) {
    let E: *mut Enemy_Bang = Get_Bang(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let v1: uint16 = ((*E).bang_var_00).wrapping_sub(1u16);
    (*E).bang_var_00 = v1;
    if v1 == 0 {
        (*E).bang_var_00 = 16 as uint16;
        (*E).bang_var_F = ((0xa3bcc5i32) as u16);
    }
}
pub fn Bang_Func_7(k: uint16) {}
pub unsafe fn Bang_Func_8(k: uint16, game: &mut Game) {
    let v1: uint8 = ((CalculateAngleOfSamusFromEnemy(ram_word!(game, cur_enemy_index), game))
        as u8)
        .wrapping_sub(64u8);
    let E: *mut Enemy_Bang = Get_Bang(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).bang_var_01 = v1 as uint16;
    (*E).bang_var_F = ((0xa3bd1ci32) as u16);
    (*E).bang_var_07 = 0 as uint16;
    (*E).bang_var_08 = 0 as uint16;
    (*E).bang_var_09 = 0 as uint16;
    (*E).bang_var_0A = 0 as uint16;
}
pub unsafe fn Bang_Func_9(game: &mut Game) {
    let v1: uint8 = ((CalculateAngleOfSamusFromEnemy(ram_word!(game, cur_enemy_index), game))
        as u8)
        .wrapping_sub(64u8);
    let E: *mut Enemy_Bang = Get_Bang(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).bang_var_02 = v1 as uint16;
    let v3: uint16 = SignExtend8(((v1) as u16).wrapping_sub((*E).bang_var_01));
    let v4: uint16 = Abs16(v3);
    if v4 as ::core::ffi::c_int - 48 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
        (*E).bang_var_F = ((0xa3bd2ci32) as u16);
    }
}
pub unsafe fn Bang_Func_10(k: uint16, game: &mut Game) {
    Bang_Func_14(game);
    Bang_Func_15(game);
    Bang_Func_12(game);
    Bang_Func_9(game);
}
pub unsafe fn Bang_Func_11(k: uint16, game: &mut Game) {
    Bang_Func_14(game);
    Bang_Func_15(game);
    Bang_Func_13(game);
    let E: *mut Enemy_Bang = Get_Bang(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    if (*E).bang_var_0A != 0u16 || ((*E).bang_var_09 as int16) <= 0i16 {
        (*E).bang_var_F = ((0xa3bca5i32) as u16);
    }
}
pub unsafe fn Bang_Func_12(game: &mut Game) {
    let E: *mut Enemy_Bang = Get_Bang(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let v1: int16 = (((*E).bang_var_0C) as i16).wrapping_sub(1i16);
    (*E).bang_var_0C = v1 as uint16;
    if v1 < 0i16 {
        (*E).bang_var_0C = (*E).bang_var_0D;
        if (((*E).bang_var_0A) as i16).wrapping_sub(((*E).bang_var_0B) as i16) < 0i16 {
            (*E).bang_var_09 = ((*E).bang_var_09).wrapping_add(22u16);
            (*E).bang_var_0A = ((*E).bang_var_0A).wrapping_add((*E).bang_var_09);
        }
    }
}
pub unsafe fn Bang_Func_13(game: &mut Game) {
    let E: *mut Enemy_Bang = Get_Bang(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let v1: int16 = (((*E).bang_var_0C) as i16).wrapping_sub(1i16);
    (*E).bang_var_0C = v1 as uint16;
    if v1 < 0i16 {
        (*E).bang_var_0C = (*E).bang_var_0E;
        (*E).bang_var_09 = ((*E).bang_var_09).wrapping_sub(22u16);
        (*E).bang_var_0A = ((*E).bang_var_0A).wrapping_sub((*E).bang_var_09);
    }
}
pub unsafe fn Bang_Func_14(game: &mut Game) {
    let E: *mut Enemy_Bang = Get_Bang(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let r18: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa0b1c3)) as *const uint8
        as *mut uint16)
        .offset(((((*E).bang_var_01) as u8).wrapping_add(64u8)) as isize);
    if (r18 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        (*E).bang_var_07 = 1 as uint16;
    }
    let mut t: uint32 = ((Abs16(r18) as ::core::ffi::c_int >> 8 as ::core::ffi::c_int)
        * (*E).bang_var_0A as ::core::ffi::c_int) as uint32;
    if (*E).bang_var_07 != 0 {
        t = -(t as int32) as uint32;
    }
    t = t.wrapping_add((*E).base.x_subpos as uint32);
    (*E).base.x_subpos = t as uint16;
    (*E).base.x_pos = (t >> 16 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Bang_Func_15(game: &mut Game) {
    let E: *mut Enemy_Bang = Get_Bang(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let r18: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa0b1c3)) as *const uint8
        as *mut uint16)
        .offset((*E).bang_var_01 as uint8 as isize);
    if (r18 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        (*E).bang_var_08 = 1 as uint16;
    }
    let mut t: uint32 = ((Abs16(r18) as ::core::ffi::c_int >> 8 as ::core::ffi::c_int)
        * (*E).bang_var_0A as ::core::ffi::c_int) as uint32;
    if (*E).bang_var_08 != 0 {
        t = -(t as int32) as uint32;
    }
    t = t.wrapping_add((*E).base.y_subpos as uint32);
    (*E).base.y_subpos = t as uint16;
    (*E).base.y_pos = (t >> 16 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Bang_Func_18(game: &mut Game) {
    let E: *mut Enemy_Bang = Get_Bang(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let bang_var_20: uint16 = (*E).bang_var_20;
    if bang_var_20 != (*E).bang_var_21 {
        (*E).bang_var_21 = bang_var_20;
        (*E).base.current_instruction = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3b722))
            as *const uint8 as *mut uint16)
            .offset(bang_var_20 as isize);
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
    }
}
pub unsafe fn Bang_Shot(game: &mut Game) {
    let E: *mut Enemy_Bang = Get_Bang(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    if (*E).bang_var_F != 0xbcc1u16 {
        (*E).bang_var_01 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3ba94)) as *const uint8
            as *mut uint16)
            .offset(
                (ram_word!(
                    game,
                    projectile_dir[ram_word!(game, collision_detection_index)]
                ) as ::core::ffi::c_int
                    & 0xf as ::core::ffi::c_int) as isize,
            );
        (*E).bang_var_F = ((0xa3bd2ci32) as u16);
        (*E).bang_var_07 = 0 as uint16;
        (*E).bang_var_08 = 0 as uint16;
        (*E).bang_var_09 = 256 as uint16;
        (*E).bang_var_0A = 1536 as uint16;
    }
    if (*E).bang_var_20 != 9u16 {
        (*E).bang_var_20 = (*E).bang_var_20.wrapping_add(1);
        Bang_Func_18(game);
        let v1: ::core::ffi::c_int =
            ram_word!(game, collision_detection_index) as ::core::ffi::c_int;
        (*E).bang_var_E = ((*E).bang_var_E).wrapping_add(ram_word!(game, projectile_damage[v1]));
        let ref mut fresh2 =
            *((&raw mut game.g_ram as *mut uint8).offset(0xc04) as *mut uint16).offset(v1 as isize);
        *fresh2 = (*fresh2) | (0x10u16);
        if (*E).bang_var_20 == 9u16 {
            (*E).bang_var_D = 1 as uint16;
        }
    }
}
pub unsafe fn Skree_Instr_1(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    (*Get_Skree(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game)).skree_var_E =
        1 as uint16;
    jp
}
pub unsafe fn Skree_Init(game: &mut Game) {
    let E: *mut Enemy_Skree =
        Get_Skree(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).skree_var_C = 0 as uint16;
    (*E).skree_var_D = 0 as uint16;
    (*E).skree_var_E = 0 as uint16;
    (*E).base.current_instruction = addr_kSkree_Ilist_C65E as uint16;
    (*E).skree_var_B = ((0xa3c6d5i32) as u16);
}
pub unsafe fn CallSkreeFunc(ea: uint32, game: &mut Game) {
    match ea {
        10733269 => Skree_Func_1(game),
        10733303 => {
            Skree_Func_2(ram_word!(game, cur_enemy_index), game);
        }
        10733334 => Skree_Func_3(game),
        10733439 => Skree_Func_4(game),
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn Skree_Main(game: &mut Game) {
    let E: *mut Enemy_Skree =
        Get_Skree(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    CallSkreeFunc((((*E).skree_var_B) as u32) | (0xa30000u32), game);
}
pub unsafe fn Skree_Func_1(game: &mut Game) {
    let E: *mut Enemy_Skree =
        Get_Skree(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    if abs16(((*E).base.x_pos).wrapping_sub(ram_word!(game, samus_x_pos))) < 0x30u16 {
        (*E).skree_var_C = (*E).skree_var_C.wrapping_add(1);
        Skree_Func_5(game);
        (*E).skree_var_B = ((0xa3c6f7i32) as u16);
    }
}
pub unsafe fn Skree_Func_2(k: uint16, game: &mut Game) {
    let E: *mut Enemy_Skree = Get_Skree(k as ::core::ffi::c_int, game);
    if (*E).skree_var_E != 0 {
        (*E).skree_var_E = 0 as uint16;
        (*E).skree_var_C = (*E).skree_var_C.wrapping_add(1);
        Skree_Func_5(game);
        (*E).skree_var_B = ((0xa3c716i32) as u16);
        QueueSfx2_Max6(0x5b as uint16, game);
    }
}
pub unsafe fn Skree_Func_3(game: &mut Game) {
    let E: *mut Enemy_Skree =
        Get_Skree(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).skree_var_A = 21 as uint16;
    let v1: uint16 = ((*E).base.properties) | (3u16);
    (*E).base.properties = v1;
    if EnemyFunc_BF8A(
        ram_word!(game, cur_enemy_index),
        v1,
        (6 as int16 as int32) << 16 as ::core::ffi::c_int,
        game,
    ) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
        (*E).skree_var_B = ((0xa3c77fi32) as u16);
        QueueSfx2_Max6(0x5c as uint16, game);
    } else {
        (*E).base.y_pos = ((*E).base.y_pos).wrapping_add(6u16);
        (*E).base.x_pos = ((*E).base.x_pos as ::core::ffi::c_int
            + if (((*E).base.x_pos) as i16).wrapping_sub((ram_word!(game, samus_x_pos)) as i16)
                >= 0i16
            {
                -(1 as ::core::ffi::c_int)
            } else {
                1 as ::core::ffi::c_int
            }) as uint16;
    };
}
pub unsafe fn Skree_Func_4(game: &mut Game) {
    let E: *mut Enemy_Skree =
        Get_Skree(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let fresh18 = (*E).skree_var_A;
    (*E).skree_var_A = (*E).skree_var_A.wrapping_sub(1);
    if fresh18 == 1u16 {
        (*gEnemySpawnData(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game))
            .vram_tiles_index = ((*E).base.vram_tiles_index) | ((*E).base.palette_index);
        (*E).base.palette_index = 2560 as uint16;
        (*E).base.vram_tiles_index = 0 as uint16;
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Deleted) as u16);
    } else {
        if (*E).skree_var_A == 8u16 {
            let v2: uint16 = ram_word!(game, cur_enemy_index);
            let mut parameter = SpawnEprojWithGfx(
                8 as uint16,
                ram_word!(game, cur_enemy_index),
                addr_stru_868BC2 as uint16,
                game,
            );
            parameter = SpawnEprojWithGfx(parameter, v2, addr_stru_868BD0 as uint16, game);
            parameter = SpawnEprojWithGfx(parameter, v2, addr_stru_868BDE as uint16, game);
            SpawnEprojWithGfx(parameter, v2, addr_stru_868BEC as uint16, game);
        }
        (*E).base.y_pos = (*E).base.y_pos.wrapping_add(1);
    };
}
pub unsafe fn Skree_Func_5(game: &mut Game) {
    let E: *mut Enemy_Skree =
        Get_Skree(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let skree_var_C: uint16 = (*E).skree_var_C;
    if skree_var_C != (*E).skree_var_D {
        (*E).skree_var_D = skree_var_C;
        (*E).base.current_instruction = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3c69c))
            as *const uint8 as *mut uint16)
            .offset(skree_var_C as isize);
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
    }
}
pub unsafe fn Skree_Shot(game: &mut Game) {
    Enemy_NormalShotAI_SkipSomeParts_A3(game);
    let E: *mut Enemy_Skree =
        Get_Skree(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    if (*E).base.health == 0 {
        let v1: uint16 = ram_word!(game, cur_enemy_index);
        let mut parameter = SpawnEprojWithGfx(
            (*E).skree_var_A,
            ram_word!(game, cur_enemy_index),
            addr_stru_868BC2 as uint16,
            game,
        );
        parameter = SpawnEprojWithGfx(parameter, v1, addr_stru_868BD0 as uint16, game);
        parameter = SpawnEprojWithGfx(parameter, v1, addr_stru_868BDE as uint16, game);
        SpawnEprojWithGfx(parameter, v1, addr_stru_868BEC as uint16, game);
        let mut v5: uint16 = 2 as uint16;
        if (ram_word!(
            game,
            projectile_type[ram_word!(game, collision_detection_index)]
        ) as ::core::ffi::c_int
            & 0xf00 as ::core::ffi::c_int)
            != 512i32
        {
            v5 = 0 as uint16;
        }
        EnemyDeathAnimation(
            (2u16).wrapping_mul(ram_word!(game, collision_detection_index)),
            v5,
            game,
        );
    }
}
pub unsafe fn MaridiaSnail_Instr_1(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    (*Get_MaridiaSnail(k as ::core::ffi::c_int, game)).msl_var_F = *jp;
    jp.offset(1)
}
pub unsafe fn MaridiaSnail_Instr_2(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    (*Get_MaridiaSnail(k as ::core::ffi::c_int, game)).msl_var_D = *jp;
    jp.offset(1)
}
pub unsafe fn MaridiaSnail_Instr_4(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let E: *mut Enemy_MaridiaSnail = Get_MaridiaSnail(k as ::core::ffi::c_int, game);
    (*E).msl_var_07 = *jp.offset(0);
    (*E).msl_var_C = *((&raw mut (*(game.g_rom.offset(crate::rtl::lorom_offset(0xa3cd42))
        as *const uint8 as *mut MaridiaSnailData2)
        .offset(0))
    .field_6 as *mut uint8)
        .offset((8 as ::core::ffi::c_int * (*E).msl_var_07 as ::core::ffi::c_int) as isize)
        as *mut uint16);
    jp.offset(1)
}
pub unsafe fn MaridiaSnail_Instr_3(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let E: *mut Enemy_MaridiaSnail = Get_MaridiaSnail(k as ::core::ffi::c_int, game);
    (*E).base.x_pos = ((*E).base.x_pos).wrapping_add(*jp.offset(0));
    (*E).base.y_pos = ((*E).base.y_pos).wrapping_add(*jp.offset(1));
    jp.offset(2)
}
pub unsafe fn MaridiaSnail_Instr_5(
    k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    if (*Get_MaridiaSnail(k as ::core::ffi::c_int, game)).msl_var_08 == 2u16
        || (NextRandom(game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int) != 0i32
    {
        jp = jp.offset(-(3 as ::core::ffi::c_int as isize));
    }
    jp
}
pub unsafe fn MaridiaSnail_Func_1(k: uint16, game: &mut Game) {
    MaridiaSnail_Func_17(k, game);
}
pub unsafe fn MaridiaSnail_Init(game: &mut Game) {
    let E: *mut Enemy_MaridiaSnail =
        Get_MaridiaSnail(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).msl_var_F = ((0xa3cf5fi32) as u16);
    (*E).base.spritemap_pointer = addr_kSpritemap_Nothing_A3 as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    let current_instruction: uint16 = (*E).base.current_instruction;
    (*E).base.current_instruction = (*(game.g_rom.offset(crate::rtl::lorom_offset(0xa3cd42))
        as *const uint8 as *mut MaridiaSnailData2)
        .offset(current_instruction as isize))
    .field_0;
    (*E).base.properties = ((*E).base.properties)
        | ((*(game.g_rom.offset(crate::rtl::lorom_offset(0xa3cd42)) as *const uint8
            as *mut MaridiaSnailData2)
            .offset(current_instruction as isize))
        .field_2);
    (*E).msl_var_D = (*(game.g_rom.offset(crate::rtl::lorom_offset(0xa3cd42)) as *const uint8
        as *mut MaridiaSnailData2)
        .offset(current_instruction as isize))
    .field_4;
    (*E).msl_var_C = (*(game.g_rom.offset(crate::rtl::lorom_offset(0xa3cd42)) as *const uint8
        as *mut MaridiaSnailData2)
        .offset(current_instruction as isize))
    .field_6;
    (*E).msl_var_08 = 0 as uint16;
    (*E).msl_var_06 = (*E).msl_parameter_1;
    MaridiaSnail_Func_2(
        ram_word!(game, cur_enemy_index),
        (current_instruction).wrapping_mul(8u16),
        game,
    );
}
pub unsafe fn MaridiaSnail_Func_2(k: uint16, j: uint16, game: &mut Game) {
    let v2: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let r18: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3cd82)) as *const uint8
        as *mut uint16)
        .offset(v2 as isize);
    let r20: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3cd82)) as *const uint8
        as *mut uint16)
        .offset((v2 + 1 as ::core::ffi::c_int) as isize);
    let r22: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3cd82)) as *const uint8
        as *mut uint16)
        .offset((v2 + 2 as ::core::ffi::c_int) as isize);
    let r24: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3cd82)) as *const uint8
        as *mut uint16)
        .offset((v2 + 3 as ::core::ffi::c_int) as isize);
    let E: *mut Enemy_MaridiaSnail = Get_MaridiaSnail(k as ::core::ffi::c_int, game);
    (*E).msl_var_A = (r20 as ::core::ffi::c_int
        + (r18 as ::core::ffi::c_int
            ^ *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3cca2)) as *const uint8
                as *mut uint16)
                .offset((*E).msl_parameter_1 as isize) as ::core::ffi::c_int))
        as uint16;
    (*E).msl_var_B = (r24 as ::core::ffi::c_int
        + (r22 as ::core::ffi::c_int
            ^ *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3cca2)) as *const uint8
                as *mut uint16)
                .offset((*E).msl_parameter_1 as isize) as ::core::ffi::c_int))
        as uint16;
}
pub unsafe fn MaridiaSnail_Func_3(k: uint16, game: &mut Game) {
    let E: *mut Enemy_MaridiaSnail = Get_MaridiaSnail(k as ::core::ffi::c_int, game);
    (*E).msl_var_F = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3cdd2)) as *const uint8
        as *mut uint16)
        .offset((*E).msl_var_07 as isize);
}
pub unsafe fn MaridiaSnail_Main(game: &mut Game) {
    MaridiaSnail_Func_4(ram_word!(game, cur_enemy_index), game);
    MaridiaSnail_Func_5(ram_word!(game, cur_enemy_index), game);
    MaridiaSnail_Func_6(ram_word!(game, cur_enemy_index), game);
    let E: *mut Enemy_MaridiaSnail =
        Get_MaridiaSnail(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    CallEnemyPreInstr((((*E).msl_var_F) as u32) | (0xa30000u32), game);
}
pub unsafe fn MaridiaSnail_Func_4(k: uint16, game: &mut Game) {
    let msl_var_08: uint16 = (*Get_MaridiaSnail(k as ::core::ffi::c_int, game)).msl_var_08;
    if msl_var_08 != 3u16
        && msl_var_08 != 4u16
        && msl_var_08 != 5u16
        && ram_word!(game, earthquake_timer) == 30u16
        && ram_word!(game, earthquake_type) == 20u16
    {
        MaridiaSnail_Func_14(k, game);
    }
}
pub unsafe fn MaridiaSnail_Func_5(k: uint16, game: &mut Game) {
    let current_block: u64;
    let E: *mut Enemy_MaridiaSnail = Get_MaridiaSnail(k as ::core::ffi::c_int, game);
    let msl_var_08: uint16 = (*E).msl_var_08;
    if msl_var_08 != 1u16 && msl_var_08 != 3u16 && msl_var_08 != 4u16 && msl_var_08 != 5u16 {
        if (*E).base.y_pos as ::core::ffi::c_int
            - ram_word!(game, samus_y_pos) as ::core::ffi::c_int
            + 96 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            if (((*E).base.x_pos) as i16).wrapping_sub((ram_word!(game, samus_x_pos)) as i16) < 0i16
            {
                if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e) != 4u8 {
                    current_block = 3898722140564953443;
                } else {
                    current_block = 6873731126896040597;
                }
            } else if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e) != 8u8 {
                current_block = 3898722140564953443;
            } else {
                current_block = 6873731126896040597;
            }
            if current_block == 3898722140564953443 {
            } else {
                if (*E).msl_var_08 == 2u16 {
                    return;
                }
                if (*E).msl_var_D as ::core::ffi::c_int
                    != 0xa3cf5f as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
                {
                    (*E).base.current_instruction = (*E).msl_var_D;
                    (*E).base.spritemap_pointer = addr_kSpritemap_Nothing_A3 as uint16;
                    (*E).base.instruction_timer = 1 as uint16;
                    (*E).base.timer = 0 as uint16;
                    (*E).msl_var_08 = 2 as uint16;
                    return;
                }
            }
        }
        (*E).msl_var_08 = 0 as uint16;
    }
}
pub unsafe fn MaridiaSnail_Func_6(k: uint16, game: &mut Game) {
    let current_block: u64;
    let E: *mut Enemy_MaridiaSnail = Get_MaridiaSnail(k as ::core::ffi::c_int, game);
    if ((*E).msl_var_F as ::core::ffi::c_int
        != 0xa3d1b3 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int)
    {
        if (*E).msl_var_08 == 1u16 {
            if ram_word!(game, samus_has_momentum_flag) != 0u16
                || (*E).msl_var_F as ::core::ffi::c_int
                    != 0xa3cf5f as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
            {
                current_block = 2304351028894610342;
            } else {
                current_block = 12573672619892793823;
            }
        } else if ram_word!(game, samus_has_momentum_flag) == 0 {
            let msl_var_08: uint16 = (*E).msl_var_08;
            if msl_var_08 == 2u16 || msl_var_08 != 3u16 && msl_var_08 != 5u16 {
                current_block = 12573672619892793823;
            } else {
                current_block = 2304351028894610342;
            }
        } else {
            current_block = 2304351028894610342;
        }
        if current_block == 2304351028894610342 {
        } else {
            (*E).base.properties = ((*E).base.properties) | (0x8000u16);
            return;
        }
    }
    (*E).base.properties = ((*E).base.properties) & (0x7fffu16);
}
unsafe fn Shift8AddMagn(a: int16, s: ::core::ffi::c_int) -> uint32 {
    let mut r: int32 = (a as int32) << 8 as ::core::ffi::c_int;
    r = r as ::core::ffi::c_int
        + if r < 0i32 {
            -(s << 16 as ::core::ffi::c_int)
        } else {
            s << 16 as ::core::ffi::c_int
        };
    r as uint32
}
pub unsafe fn MaridiaSnail_Func_7(k: uint16, game: &mut Game) {
    let E: *mut Enemy_MaridiaSnail = Get_MaridiaSnail(k as ::core::ffi::c_int, game);
    if (*E).msl_var_07 >= 4u16 {
        if EnemyFunc_BC76(
            k,
            Shift8AddMagn((*E).msl_var_B as int16, 7 as ::core::ffi::c_int) as int32,
            game,
        ) != 0
        {
            return;
        }
    } else if EnemyFunc_BBBF(
        k,
        Shift8AddMagn((*E).msl_var_A as int16, 7 as ::core::ffi::c_int) as int32,
        game,
    ) != 0
    {
        return;
    }
    MaridiaSnail_Func_14(k, game);
}
pub unsafe fn MaridiaSnail_Func_9(k: uint16, game: &mut Game) {
    let v1: uint16 = (if (*Get_MaridiaSnail(k as ::core::ffi::c_int, game)).msl_var_05 != 0u16 {
        addr_stru_A3CD22 as ::core::ffi::c_int
    } else {
        addr_stru_A3CCE2 as ::core::ffi::c_int
    }) as uint16;
    MaridiaSnail_D07E(k, RomPtr_A3(v1, game), game);
}
pub unsafe fn MaridiaSnail_CFB7(k: uint16, game: &mut Game) {
    MaridiaSnail_D002(k, RomPtr_A3(addr_stru_A3CCEA as uint16_t, game), game);
}
pub unsafe fn MaridiaSnail_CFBD(k: uint16, game: &mut Game) {
    let v1: uint16 = (if (*Get_MaridiaSnail(k as ::core::ffi::c_int, game)).msl_var_05 != 0u16 {
        addr_stru_A3CD2A as ::core::ffi::c_int
    } else {
        addr_stru_A3CCF2 as ::core::ffi::c_int
    }) as uint16;
    MaridiaSnail_D07E(k, RomPtr_A3(v1, game), game);
}
pub unsafe fn MaridiaSnail_CFCE(k: uint16, game: &mut Game) {
    MaridiaSnail_D002(k, RomPtr_A3(addr_stru_A3CCFA as uint16_t, game), game);
}
pub unsafe fn MaridiaSnail_CFD4(k: uint16, game: &mut Game) {
    let v1: uint16 = (if (*Get_MaridiaSnail(k as ::core::ffi::c_int, game)).msl_var_05 != 0u16 {
        addr_stru_A3CD32 as ::core::ffi::c_int
    } else {
        addr_stru_A3CD02 as ::core::ffi::c_int
    }) as uint16;
    MaridiaSnail_D07E(k, RomPtr_A3(v1, game), game);
}
pub unsafe fn MaridiaSnail_CFE5(k: uint16, game: &mut Game) {
    MaridiaSnail_D002(k, RomPtr_A3(addr_stru_A3CD0A as uint16_t, game), game);
}
pub unsafe fn MaridiaSnail_CFEB(k: uint16, game: &mut Game) {
    let mut v1: uint16 = 0;
    if (*Get_MaridiaSnail(k as ::core::ffi::c_int, game)).msl_var_05 != 0 {
        v1 = addr_stru_A3CD3A as uint16;
    } else {
        v1 = addr_stru_A3CD12 as uint16;
    }
    MaridiaSnail_D07E(k, RomPtr_A3(v1, game), game);
}
pub unsafe fn MaridiaSnail_CFFC(k: uint16, game: &mut Game) {
    MaridiaSnail_D002(k, RomPtr_A3(addr_stru_A3CD1A as uint16_t, game), game);
}
pub unsafe fn MaridiaSnail_D002(k: uint16, j: *const uint8, game: &mut Game) {
    MaridiaSnail_Func_10(k, j, game);
    let E: *mut Enemy_MaridiaSnail = Get_MaridiaSnail(k as ::core::ffi::c_int, game);
    let rv: uint8 = Enemy_MoveRight_ProcessSlopes(
        k,
        Shift8AddMagn((*E).msl_var_A as int16, 1 as ::core::ffi::c_int) as int32,
        game,
    );
    MaridiaSnail_Func_11(k, j, game);
    if rv != 0 {
        (*E).msl_var_E = 0 as uint16;
        let carry: uint8 = EnemyFunc_C8AD(k, game);
        MaridiaSnail_Func_12(k, carry as uint16, game);
        if Enemy_MoveDown(
            k,
            ((*E).msl_var_B as int16 as int32) << 8 as ::core::ffi::c_int,
            game,
        ) != 0
        {
            (*E).msl_var_A = -((*E).msl_var_A as ::core::ffi::c_int) as uint16;
            MaridiaSnail_Func_13(k, *(j.offset(6) as *mut uint16), game);
        }
    } else {
        let v7: uint16 = ((*E).msl_var_E).wrapping_add(1u16);
        (*E).msl_var_E = v7;
        if v7 as ::core::ffi::c_int - 4 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
            (*E).msl_var_B = -((*E).msl_var_B as ::core::ffi::c_int) as uint16;
            MaridiaSnail_Func_13(k, *(j.offset(4) as *mut uint16), game);
        } else {
            MaridiaSnail_Func_14(k, game);
        }
    };
}
pub unsafe fn MaridiaSnail_D07E(k: uint16, j: *const uint8, game: &mut Game) {
    MaridiaSnail_Func_10(k, j, game);
    let E: *mut Enemy_MaridiaSnail = Get_MaridiaSnail(k as ::core::ffi::c_int, game);
    let rv: uint8 = Enemy_MoveDown(
        k,
        Shift8AddMagn((*E).msl_var_B as int16, 1 as ::core::ffi::c_int) as int32,
        game,
    );
    MaridiaSnail_Func_11(k, j, game);
    if rv != 0 {
        (*E).msl_var_E = 0 as uint16;
        if Enemy_MoveRight_ProcessSlopes(
            k,
            ((*E).msl_var_A as int16 as int32) << 8 as ::core::ffi::c_int,
            game,
        ) != 0
        {
            (*E).msl_var_B = -((*E).msl_var_B as ::core::ffi::c_int) as uint16;
            MaridiaSnail_Func_13(k, *(j.offset(6) as *mut uint16), game);
        } else {
            let carry: uint8 = EnemyFunc_C8AD(k, game);
            MaridiaSnail_Func_12(k, carry as uint16, game);
        }
    } else {
        let v7: uint16 = ((*E).msl_var_E).wrapping_add(1u16);
        (*E).msl_var_E = v7;
        if v7 as ::core::ffi::c_int - 4 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
            (*E).msl_var_A = -((*E).msl_var_A as ::core::ffi::c_int) as uint16;
            MaridiaSnail_Func_13(k, *(j.offset(4) as *mut uint16), game);
        } else {
            MaridiaSnail_Func_14(k, game);
        }
    };
}
pub unsafe fn MaridiaSnail_Func_10(k: uint16, j: *const uint8, game: &mut Game) {
    let E: *mut Enemy_MaridiaSnail = Get_MaridiaSnail(k as ::core::ffi::c_int, game);
    (*E).base.x_pos = ((*E).base.x_pos).wrapping_add(*(j as *mut uint16));
    (*E).base.y_pos = ((*E).base.y_pos).wrapping_add(*(j.offset(2) as *mut uint16));
}
pub unsafe fn MaridiaSnail_Func_11(k: uint16, j: *const uint8, game: &mut Game) {
    let E: *mut Enemy_MaridiaSnail = Get_MaridiaSnail(k as ::core::ffi::c_int, game);
    (*E).base.x_pos = ((*E).base.x_pos).wrapping_sub(*(j as *mut uint16));
    (*E).base.y_pos = ((*E).base.y_pos).wrapping_sub(*(j.offset(2) as *mut uint16));
}
pub unsafe fn MaridiaSnail_Func_12(k: uint16, carry: uint16, game: &mut Game) {
    let E: *mut Enemy_MaridiaSnail = Get_MaridiaSnail(k as ::core::ffi::c_int, game);
    if carry == 0 {
        if ((*E).msl_var_04).wrapping_add(1u16) >= 0x10u16 {
            (*E).msl_var_05 = 0 as uint16;
        } else {
            (*E).msl_var_04 = (*E).msl_var_04.wrapping_add(1);
        }
    } else {
        (*E).msl_var_05 = 1 as uint16;
        (*E).msl_var_04 = 0 as uint16;
    };
}
pub unsafe fn MaridiaSnail_Func_13(k: uint16, a: uint16, game: &mut Game) {
    let E: *mut Enemy_MaridiaSnail = Get_MaridiaSnail(k as ::core::ffi::c_int, game);
    (*E).base.current_instruction = a;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).msl_var_05 = 1 as uint16;
    (*E).msl_var_04 = 0 as uint16;
}
pub unsafe fn MaridiaSnail_Func_14(k: uint16, game: &mut Game) {
    let E: *mut Enemy_MaridiaSnail = Get_MaridiaSnail(k as ::core::ffi::c_int, game);
    if (*E).msl_var_08 != 3u16 {
        (*E).msl_var_08 = 3 as uint16;
        (*E).msl_var_F = ((0xa3d1b3i32) as u16);
        let v2: ::core::ffi::c_int = 2 as ::core::ffi::c_int * (*E).msl_var_C as ::core::ffi::c_int;
        (*E).base.current_instruction = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3d1ab))
            as *const uint8 as *mut uint16)
            .offset(v2 as isize);
        (*E).msl_var_D = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3d1ab)) as *const uint8
            as *mut uint16)
            .offset((v2 + 1 as ::core::ffi::c_int) as isize);
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
        (*E).msl_var_02 = 0 as uint16;
        (*E).msl_var_03 = 0 as uint16;
        (*E).msl_var_00 = 0 as uint16;
        (*E).msl_var_01 = 0 as uint16;
    }
}
pub unsafe fn MaridiaSnail_Func_15(k: uint16, game: &mut Game) {
    let E: *mut Enemy_MaridiaSnail = Get_MaridiaSnail(k as ::core::ffi::c_int, game);
    if (*E).msl_var_08 != 3u16 {
        if Enemy_MoveRight_IgnoreSlopes(
            k,
            (((*E).msl_var_03 as uint32) << 16 as ::core::ffi::c_int | (*E).msl_var_02 as uint32)
                as int32,
            game,
        ) != 0
        {
            (*E).msl_var_02 = -((*E).msl_var_02 as ::core::ffi::c_int) as uint16;
            (*E).msl_var_03 = -((*E).msl_var_03 as ::core::ffi::c_int) as uint16;
            (*E).msl_var_20 = 1 as uint16;
            QueueSfx2_Max3(0x70 as uint16, game);
        } else {
            let mut delta: uint32 = 0;
            if ((*E).msl_var_03 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
                delta = 4096 as uint32;
            } else {
                delta = -(4096 as ::core::ffi::c_int) as uint32;
            }
            let t: uint32 = (((*E).msl_var_03 as uint32) << 16 as ::core::ffi::c_int
                | (*E).msl_var_02 as uint32)
                .wrapping_add(delta);
            (*E).msl_var_02 = t as uint16;
            if t >> 16 as ::core::ffi::c_int != 0 {
                (*E).msl_var_03 = (t >> 16 as ::core::ffi::c_int) as uint16;
            }
        }
    }
    if Enemy_MoveDown(
        k,
        (((*E).msl_var_01 as uint32) << 16 as ::core::ffi::c_int | (*E).msl_var_00 as uint32)
            as int32,
        game,
    ) != 0
    {
        let msl_var_01: int16 = (*E).msl_var_01 as int16;
        if msl_var_01 >= 0i16
            && msl_var_01 as ::core::ffi::c_int - 3 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
        {
            (*E).msl_var_03 = 0 as uint16;
            (*E).msl_var_02 = 0 as uint16;
            (*E).msl_var_01 = 0 as uint16;
            (*E).msl_var_00 = 0 as uint16;
            (*E).msl_var_E = 0 as uint16;
            (*E).msl_var_04 = 0 as uint16;
            (*E).msl_var_05 = 1 as uint16;
            if (*E).msl_var_08 == 3u16 {
                (*E).msl_var_08 = 0 as uint16;
            } else {
                (*E).msl_var_08 = 1 as uint16;
                (*E).msl_parameter_1 = 8 as uint16;
                MaridiaSnail_Func_18(k, game);
                MaridiaSnail_Func_16(k, game);
            }
            (*E).base.properties = ((*E).msl_var_C as ::core::ffi::c_int
                | (*E).base.properties as ::core::ffi::c_int & !(3 as ::core::ffi::c_int))
                as uint16;
            StoneZoomer_E67A(k, game);
            (*E).msl_var_F = ((0xa3cf5fi32) as u16);
            (*E).base.current_instruction = (*E).msl_var_D;
            (*E).base.instruction_timer = 1 as uint16;
            (*E).base.timer = 0 as uint16;
        } else {
            (*E).msl_var_00 = -((*E).msl_var_00 as ::core::ffi::c_int) as uint16;
            (*E).msl_var_01 = -((*E).msl_var_01 as ::core::ffi::c_int) as uint16;
            (*E).msl_var_20 = 0 as uint16;
        }
    } else {
        let t_0: uint32 = (((*E).msl_var_01 as uint32) << 16 as ::core::ffi::c_int
            | (*E).msl_var_00 as uint32)
            .wrapping_add(0x2000 as uint32);
        (*E).msl_var_00 = t_0 as uint16;
        if (t_0 >> 16 as ::core::ffi::c_int).wrapping_sub(4 as uint32) & 0x8000 as uint32 != 0 {
            (*E).msl_var_01 = (t_0 >> 16 as ::core::ffi::c_int) as uint16;
        }
    };
}
pub unsafe fn MaridiaSnail_Func_16(k: uint16, game: &mut Game) {
    let E: *mut Enemy_MaridiaSnail = Get_MaridiaSnail(k as ::core::ffi::c_int, game);
    let v2: ::core::ffi::c_int =
        ((4u16).wrapping_mul((*E).msl_var_C)) as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    (*E).base.current_instruction = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3d30d))
        as *const uint8 as *mut uint16)
        .offset(v2 as isize);
    (*E).msl_var_D = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3d30d)) as *const uint8
        as *mut uint16)
        .offset((v2 + 1 as ::core::ffi::c_int) as isize);
}
pub unsafe fn MaridiaSnail_Func_17(k: uint16, game: &mut Game) -> uint8 {
    let E: *mut Enemy_MaridiaSnail = Get_MaridiaSnail(k as ::core::ffi::c_int, game);
    if (*E).msl_var_05 != 0 {
        return 0 as uint8;
    }
    if (*E).msl_var_07 >= 4u16 {
        return (MaridiaSnail_Func_18(k, game)) & (1u8);
    }
    if ((*E).msl_var_07 as ::core::ffi::c_int & 1 as ::core::ffi::c_int) != 0i32 {
        ((if (*E).base.y_pos >= ram_word!(game, samus_y_pos) {
            MaridiaSnail_Func_19(k, game) as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) as uint8)
    } else {
        ((if (*E).base.y_pos < ram_word!(game, samus_y_pos) {
            MaridiaSnail_Func_19(k, game) as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) as uint8)
    }
}
pub unsafe fn MaridiaSnail_Func_18(k: uint16, game: &mut Game) -> uint8 {
    let E: *mut Enemy_MaridiaSnail = Get_MaridiaSnail(k as ::core::ffi::c_int, game);
    if (*E).msl_var_C != 0 {
        ((if (*E).base.x_pos >= ram_word!(game, samus_x_pos) {
            MaridiaSnail_Func_19(k, game) as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) as uint8)
    } else {
        ((if (*E).base.x_pos < ram_word!(game, samus_x_pos) {
            MaridiaSnail_Func_19(k, game) as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) as uint8)
    }
}
pub unsafe fn MaridiaSnail_Func_19(k: uint16, game: &mut Game) -> uint8 {
    let E: *mut Enemy_MaridiaSnail = Get_MaridiaSnail(k as ::core::ffi::c_int, game);
    if (*E).msl_var_08 == 2u16 || (*E).msl_var_F == 0xcf5fu16 {
        return 0 as uint8;
    }
    let v2: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3cdc2)) as *const uint8
        as *mut uint16)
        .offset((*E).msl_var_07 as isize);
    (*E).msl_var_07 = v2;
    (*E).base.current_instruction =
        *((&raw mut (*(game.g_rom.offset(crate::rtl::lorom_offset(0xa3cd42)) as *const uint8
            as *mut MaridiaSnailData2)
            .offset(0))
        .field_0 as *mut uint8)
            .offset((8 as ::core::ffi::c_int * v2 as ::core::ffi::c_int) as isize)
            as *mut VoidP);
    (*E).base.properties = (*((&raw mut (*(game.g_rom.offset(crate::rtl::lorom_offset(0xa3cd42))
        as *const uint8 as *mut MaridiaSnailData2)
        .offset(0))
    .field_2 as *mut uint8)
        .offset((8 as ::core::ffi::c_int * v2 as ::core::ffi::c_int) as isize)
        as *mut uint16) as ::core::ffi::c_int
        | (*E).base.properties as ::core::ffi::c_int & !(3 as ::core::ffi::c_int))
        as uint16;
    (*E).msl_var_D = *((&raw mut (*(game.g_rom.offset(crate::rtl::lorom_offset(0xa3cd42))
        as *const uint8 as *mut MaridiaSnailData2)
        .offset(0))
    .field_4 as *mut uint8)
        .offset((8 as ::core::ffi::c_int * v2 as ::core::ffi::c_int) as isize)
        as *mut VoidP);
    (*E).msl_var_C = *((&raw mut (*(game.g_rom.offset(crate::rtl::lorom_offset(0xa3cd42))
        as *const uint8 as *mut MaridiaSnailData2)
        .offset(0))
    .field_6 as *mut uint8)
        .offset((8 as ::core::ffi::c_int * v2 as ::core::ffi::c_int) as isize)
        as *mut uint16);
    MaridiaSnail_Func_2(k, (8u16).wrapping_mul(v2), game);
    MaridiaSnail_Func_3(k, game);
    (*E).msl_var_05 = 1 as uint16;
    (*E).msl_var_04 = 0 as uint16;
    1 as uint8
}
pub unsafe fn MaridiaSnail_Touch(game: &mut Game) {
    let E: *mut Enemy_MaridiaSnail =
        Get_MaridiaSnail(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    if ((*E).msl_var_08 != 1u16
        || (*E).msl_var_F == 0xcf5fu16
        || MaridiaSnail_Func_20(ram_word!(game, cur_enemy_index), game) as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            == 0)
        && ((*E).msl_var_F == 0xd1b3u16 || ram_word!(game, samus_has_momentum_flag) != 0u16)
    {
        MaridiaSnail_Func_22(ram_word!(game, cur_enemy_index), game);
        if (*E).msl_var_F == 0xd1b3u16 {
            QueueSfx2_Max3(0x70 as uint16, game);
        }
    } else if (*E).msl_var_F != 0xcf5fu16 && (*E).msl_var_08 != 4u16 && (*E).msl_var_08 != 3u16 {
        Enemy_NormalTouchAI_A3(game);
        (*E).msl_parameter_1 = (*E).msl_var_06;
        if (*E).msl_var_08 != 0 {
            MaridiaSnail_Func_19(ram_word!(game, cur_enemy_index), game);
        }
        (*E).msl_var_08 = 0 as uint16;
    }
}
pub unsafe fn MaridiaSnail_Func_20(k: uint16, game: &mut Game) -> uint8 {
    let r20: uint16 = ((*Get_MaridiaSnail(k as ::core::ffi::c_int, game)).msl_var_C) & (1u16);
    if (((ram_word!(game, joypad1_lastkeys)) & (0x300u16)) as ::core::ffi::c_int
        >> 8 as ::core::ffi::c_int)
        == 1i32
    {
        if r20 != 0 {
            return 0 as uint8;
        }
    } else if r20 == 0 {
        return 0 as uint8;
    }
    1 as uint8
}
pub unsafe fn MaridiaSnail_Func_21(k: uint16, game: &mut Game) -> uint8 {
    if ((*Get_MaridiaSnail(k as ::core::ffi::c_int, game)).msl_var_A as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int)
        != 0i32
    {
        return (*(&raw mut game.g_ram as *mut uint8).offset(0xa1e) == 8u8) as uint8;
    }
    (*(&raw mut game.g_ram as *mut uint8).offset(0xa1e) == 4u8) as uint8
}
pub unsafe fn MaridiaSnail_Shot(game: &mut Game) {
    let v0: uint16 = (ram_word!(
        game,
        projectile_type[ram_word!(game, collision_detection_index)]
    )) & (0xff00u16);
    if v0 == 768u16 || v0 == 1280u16 {
        Enemy_NormalShotAI_A3(game);
    } else {
        let msl_var_08: uint16 =
            (*Get_MaridiaSnail(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game))
                .msl_var_08;
        if msl_var_08 != 3u16 && msl_var_08 != 4u16 {
            MaridiaSnail_Func_23(ram_word!(game, cur_enemy_index), game);
        }
        QueueSfx2_Max3(0x70 as uint16, game);
    };
}
pub unsafe fn MaridiaSnail_Func_22(k: uint16, game: &mut Game) {
    let E: *mut Enemy_MaridiaSnail = Get_MaridiaSnail(k as ::core::ffi::c_int, game);
    (*E).msl_var_08 = 4 as uint16;
    (*E).msl_var_F = ((0xa3d1b3i32) as u16);
    let v2: ::core::ffi::c_int =
        ((4u16).wrapping_mul((*E).msl_var_C)) as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    (*E).base.current_instruction = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3d50f))
        as *const uint8 as *mut uint16)
        .offset(v2 as isize);
    (*E).msl_var_D = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3d50f)) as *const uint8
        as *mut uint16)
        .offset((v2 + 1 as ::core::ffi::c_int) as isize);
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
    (*E).msl_var_02 = ram_word!(game, absolute_moved_last_frame_x_fract);
    let mut v3: uint16 = ram_word!(game, absolute_moved_last_frame_x);
    (*E).msl_var_03 = ram_word!(game, absolute_moved_last_frame_x);
    if v3 >= 0x10u16 {
        v3 = 15 as uint16;
    }
    let v4: ::core::ffi::c_int =
        ((4u16).wrapping_mul(v3)) as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    (*E).msl_var_00 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3d517)) as *const uint8
        as *mut uint16)
        .offset(v4 as isize);
    (*E).msl_var_01 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3d517)) as *const uint8
        as *mut uint16)
        .offset((v4 + 1 as ::core::ffi::c_int) as isize);
    if (*(&raw mut game.g_ram as *mut uint8).offset(0xa1e) as ::core::ffi::c_int
        & 4 as ::core::ffi::c_int)
        != 0i32
    {
        (*E).msl_var_02 = -((*E).msl_var_02 as ::core::ffi::c_int) as uint16;
        (*E).msl_var_03 = -((*E).msl_var_03 as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn MaridiaSnail_Func_23(k: uint16, game: &mut Game) {
    let E: *mut Enemy_MaridiaSnail = Get_MaridiaSnail(k as ::core::ffi::c_int, game);
    (*E).msl_var_08 = 5 as uint16;
    (*E).msl_var_F = ((0xa3d1b3i32) as u16);
    let v2: ::core::ffi::c_int =
        ((4u16).wrapping_mul((*E).msl_var_C)) as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    (*E).base.current_instruction = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3d5a4))
        as *const uint8 as *mut uint16)
        .offset(v2 as isize);
    (*E).msl_var_D = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3d5a4)) as *const uint8
        as *mut uint16)
        .offset((v2 + 1 as ::core::ffi::c_int) as isize);
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
    (*E).msl_var_01 = -(1 as ::core::ffi::c_int) as uint16;
    (*E).msl_var_03 = (if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e) == 4u8 {
        -(1 as ::core::ffi::c_int)
    } else {
        1 as ::core::ffi::c_int
    }) as uint16;
}
pub unsafe fn Reflec_Func_1(game: &mut Game) {
    if ram_word!(game, door_transition_flag_enemies) == 0 && {
        let ref mut fresh15 =
            *((&raw mut game.g_ram as *mut uint8).offset(0x1794) as *mut uint16).offset(2);
        *fresh15 = (*fresh15).wrapping_sub(1);
        *fresh15 == 0
    } {
        set_ram_word!(game.variables_for_enemy_graphics_drawn_hook[2] = 16 as uint16);
        let mut v0: uint16 = ram_word!(game, variables_for_enemy_graphics_drawn_hook[0]);
        let mut v1: uint16 =
            (8u16).wrapping_mul(ram_word!(game, variables_for_enemy_graphics_drawn_hook[1]));
        let mut n: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
        loop {
            *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(
                ((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) + 137 as ::core::ffi::c_int)
                    as isize,
            ) = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3dabc)) as *const uint8
                as *mut uint16)
                .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
            v1 = (v1).wrapping_add(2u16);
            v0 = (v0).wrapping_add(2u16);
            n -= 1;
            if (n == 0) {
                break;
            }
        }
        set_ram_word!(
            game.variables_for_enemy_graphics_drawn_hook[1] =
                (*(((&raw mut game.g_ram as *mut uint8).offset(0x1794) as *mut uint16).offset(1)
                    as *mut uint16 as *mut uint8)
                    .offset(0) as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    & 7 as ::core::ffi::c_int) as uint16
        );
    }
}
pub unsafe fn Reflec_Instr_1(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    (*Get_Reflec(k as ::core::ffi::c_int, game)).reflec_parameter_2 = *jp.offset(0);
    jp.offset(1)
}
pub unsafe fn Reflec_Init(game: &mut Game) {
    let E: *mut Enemy_Reflec =
        Get_Reflec(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_BlockPlasmaBeam) as u16);
    (*E).base.current_instruction = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3dc0b))
        as *const uint8 as *mut uint16)
        .offset((*E).reflec_parameter_1 as isize);
    (*((&raw mut game.g_ram as *mut uint8).offset(0x178c) as *mut LongPtr)).addr =
        ((0xa3db0ci32) as u16);
    *(&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x178c) as *mut LongPtr)).bank
        as *mut uint16) = 163 as uint16;
    set_ram_word!(
        game.variables_for_enemy_graphics_drawn_hook[0] =
            ((16 as ::core::ffi::c_int * (*E).base.palette_index as ::core::ffi::c_int
                & 0xff00 as ::core::ffi::c_int)
                >> 8 as ::core::ffi::c_int) as uint16
    );
    set_ram_word!(game.variables_for_enemy_graphics_drawn_hook[2] = 16 as uint16);
}
pub unsafe fn Reflec_Shot(game: &mut Game) {
    let v0: uint16 = (2u16).wrapping_mul(ram_word!(game, collision_detection_index));
    let EK: *mut Enemy_Reflec =
        Get_Reflec(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*EK).base.invincibility_timer = 10 as uint16;
    let v2: uint16 = (32 as ::core::ffi::c_int * (*EK).reflec_parameter_2 as ::core::ffi::c_int
        + 2 as ::core::ffi::c_int
            * (ram_word!(game, projectile_dir[(v0 as i32 >> 1i32)]) as ::core::ffi::c_int
                & 0xf as ::core::ffi::c_int)) as uint16;
    let varE32: uint16 = v2;
    let v3: ::core::ffi::c_int = v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3dcae)) as *const uint8 as *mut uint16)
        .offset(v3 as isize)
        == 0x8000u16
    {
        let ref mut fresh16 = *((&raw mut game.g_ram as *mut uint8).offset(0xc04) as *mut uint16)
            .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        *fresh16 = (*fresh16) | (0x10u16);
        printf(b"Possible bug. What is X?\n\0" as *const u8 as *const ::core::ffi::c_char);
        let ET: *mut Enemy_Reflec = Get_Reflec(v2 as ::core::ffi::c_int, game);
        if (*ET).base.health != 0 {
            Enemy_NormalShotAI_SkipSomeParts_A3(game);
            if (*ET).base.health == 0 {
                (*ET).base.current_instruction =
                    *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3dca6)) as *const uint8
                        as *mut uint16)
                        .offset((*ET).reflec_parameter_2 as isize);
                (*ET).base.instruction_timer = 1 as uint16;
                (*ET).base.timer = 0 as uint16;
            }
        }
    } else {
        let mut v4: int16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3dcae)) as *const uint8
            as *mut uint16)
            .offset(v3 as isize) as int16;
        if v4 < 0i16 {
            v4 = -(*(game.g_rom.offset(crate::rtl::lorom_offset(0xa3dcae)) as *const uint8
                as *mut uint16)
                .offset((varE32 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int) as int16;
        }
        let v5: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        set_ram_word!(game.projectile_dir[v5] = v4 as uint16);
        let ref mut fresh17 =
            *((&raw mut game.g_ram as *mut uint8).offset(0xc18) as *mut uint16).offset(v5 as isize);
        *fresh17 = (*fresh17 as ::core::ffi::c_int & !(0x8000 as ::core::ffi::c_int)) as uint16;
        ProjectileReflection(v0, game);
        QueueSfx2_Max6(0x57 as uint16, game);
    };
}
pub unsafe fn WreckedShipOrangeZoomer_Func_1(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_WreckedShipOrangeZoomer(k as ::core::ffi::c_int, game)).wsozr_var_F = *jp.offset(0);
    jp.offset(1)
}
pub unsafe fn WreckedShipOrangeZoomer_Init(game: &mut Game) {
    let E: *mut Enemy_WreckedShipOrangeZoomer =
        Get_WreckedShipOrangeZoomer(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).base.current_instruction = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3e03b))
        as *const uint8 as *mut uint16)
        .offset(
            ((*E).base.current_instruction as ::core::ffi::c_int & 3 as ::core::ffi::c_int)
                as isize,
        );
    (*E).wsozr_var_F = addr_locret_A3E08A as uint16;
    let v1: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3e5f0)) as *const uint8
        as *mut uint16)
        .offset((*E).wsozr_parameter_1 as isize);
    (*E).wsozr_var_A = v1;
    (*E).wsozr_var_B = v1;
    if ((*E).base.properties as ::core::ffi::c_int & 3 as ::core::ffi::c_int) != 0i32 {
        if ((*E).base.properties as ::core::ffi::c_int & 3 as ::core::ffi::c_int) == 2i32 {
            (*E).wsozr_var_B = -((*E).wsozr_var_B as ::core::ffi::c_int) as uint16;
        }
    } else {
        (*E).wsozr_var_A = -((*E).wsozr_var_A as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn WreckedShipOrangeZoomer_Main(game: &mut Game) {
    let E: *mut Enemy_WreckedShipOrangeZoomer =
        Get_WreckedShipOrangeZoomer(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    CallEnemyPreInstr((((*E).wsozr_var_F) as u32) | (0xa30000u32), game);
}
pub unsafe fn WreckedShipOrangeZoomer_Func_2(k: uint16, game: &mut Game) {
    let mut v7: int16 = 0;
    let mut v9: uint16 = 0;
    let mut v12: uint16 = 0;
    let E: *mut Enemy_WreckedShipOrangeZoomer =
        Get_WreckedShipOrangeZoomer(k as ::core::ffi::c_int, game);
    if ram_word!(game, earthquake_timer) == 30u16 && ram_word!(game, earthquake_type) == 20u16 {
        (*E).wsozr_var_03 = (*E).wsozr_var_F;
        (*E).wsozr_var_F = ((0xa3e785i32) as u16);
    }
    if Enemy_MoveRight_ProcessSlopes(
        k,
        Shift8AddMagn((*E).wsozr_var_A as int16, 1 as ::core::ffi::c_int) as int32,
        game,
    ) != 0
    {
        (*E).wsozr_var_04 = 0 as uint16;
        EnemyFunc_C8AD(k, game);
        if Enemy_MoveDown(
            k,
            ((*E).wsozr_var_B as int16 as int32) << 8 as ::core::ffi::c_int,
            game,
        ) == 0
        {
            let current_block_8: u64;
            if ((ram_word!(game, samus_y_pos)) as i16).wrapping_sub(((*E).base.y_pos) as i16)
                >= 0i16
            {
                v7 = (*E).wsozr_var_B as int16;
                if v7 < 0i16 {
                    current_block_8 = 9790114338316996362;
                } else {
                    current_block_8 = 11050875288958768710;
                }
            } else {
                v7 = (*E).wsozr_var_B as int16;
                if v7 >= 0i16 {
                    current_block_8 = 9790114338316996362;
                } else {
                    current_block_8 = 11050875288958768710;
                }
            }
            if current_block_8 == 9790114338316996362 {
                v7 = -(v7 as ::core::ffi::c_int) as int16;
            }
            (*E).wsozr_var_B = v7 as uint16;
            return;
        }
        (*E).wsozr_var_A = -((*E).wsozr_var_A as ::core::ffi::c_int) as uint16;
        v9 = addr_kWreckedShipOrangeZoomer_Ilist_E01F as uint16;
        if ((*E).wsozr_var_B as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
            v9 = addr_kWreckedShipOrangeZoomer_Ilist_E003 as uint16;
        }
        (*E).base.current_instruction = v9;
        (*E).base.instruction_timer = 1 as uint16;
    } else {
        let v11: uint16 = ((*E).wsozr_var_04).wrapping_add(1u16);
        (*E).wsozr_var_04 = v11;
        if v11 as ::core::ffi::c_int - 4 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
            (*E).wsozr_var_B = -((*E).wsozr_var_B as ::core::ffi::c_int) as uint16;
            v12 = addr_kWreckedShipOrangeZoomer_Ilist_E01F as uint16;
            if ((*E).wsozr_var_B as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
                v12 = addr_kWreckedShipOrangeZoomer_Ilist_E003 as uint16;
            }
            (*E).base.current_instruction = v12;
            (*E).base.instruction_timer = 1 as uint16;
        } else {
            (*E).wsozr_var_03 = (*E).wsozr_var_F;
            (*E).wsozr_var_F = ((0xa3e785i32) as u16);
        }
    };
}
pub unsafe fn sub_A3E168(k: uint16, game: &mut Game) {
    let mut v7: int16 = 0;
    let mut v9: uint16 = 0;
    let mut v12: uint16 = 0;
    let E: *mut Enemy_WreckedShipOrangeZoomer =
        Get_WreckedShipOrangeZoomer(k as ::core::ffi::c_int, game);
    if ram_word!(game, earthquake_timer) == 30u16 && ram_word!(game, earthquake_type) == 20u16 {
        (*E).wsozr_var_03 = (*E).wsozr_var_F;
        (*E).wsozr_var_F = ((0xa3e785i32) as u16);
    }
    if Enemy_MoveDown(
        k,
        Shift8AddMagn((*E).wsozr_var_B as int16, 1 as ::core::ffi::c_int) as int32,
        game,
    ) != 0
    {
        (*E).wsozr_var_04 = 0 as uint16;
        if Enemy_MoveRight_ProcessSlopes(
            k,
            ((*E).wsozr_var_A as int16 as int32) << 8 as ::core::ffi::c_int,
            game,
        ) == 0
        {
            EnemyFunc_C8AD(k, game);
            if ((ram_word!(game, samus_x_pos)) as i16).wrapping_sub(((*E).base.x_pos) as i16)
                >= 0i16
            {
                v7 = (*E).wsozr_var_A as int16;
                if v7 < 0i16 {
                    v7 = -(v7 as ::core::ffi::c_int) as int16;
                }
            } else {
                v7 = (*E).wsozr_var_A as int16;
                if v7 >= 0i16 {
                    v7 = -(v7 as ::core::ffi::c_int) as int16;
                }
            }
            (*E).wsozr_var_A = v7 as uint16;
            return;
        }
        (*E).wsozr_var_B = -((*E).wsozr_var_B as ::core::ffi::c_int) as uint16;
        v9 = addr_kWreckedShipOrangeZoomer_Ilist_DFE7 as uint16;
        if ((*E).wsozr_var_A as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
            v9 = addr_kWreckedShipOrangeZoomer_Ilist_DFCB as uint16;
        }
        (*E).base.current_instruction = v9;
        (*E).base.instruction_timer = 1 as uint16;
    } else {
        let v11: uint16 = ((*E).wsozr_var_04).wrapping_add(1u16);
        (*E).wsozr_var_04 = v11;
        if v11 as ::core::ffi::c_int - 4 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
            (*E).wsozr_var_A = -((*E).wsozr_var_A as ::core::ffi::c_int) as uint16;
            v12 = addr_kWreckedShipOrangeZoomer_Ilist_DFE7 as uint16;
            if ((*E).wsozr_var_A as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
                v12 = addr_kWreckedShipOrangeZoomer_Ilist_DFCB as uint16;
            }
            (*E).base.current_instruction = v12;
            (*E).base.instruction_timer = 1 as uint16;
        } else {
            (*E).wsozr_var_03 = (*E).wsozr_var_F;
            (*E).wsozr_var_F = ((0xa3e785i32) as u16);
        }
    };
}
pub unsafe fn BigEyeBugs_Init(game: &mut Game) {
    let v1: *mut EnemyData =
        gEnemyData(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*v1).current_instruction = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3e2cc))
        as *const uint8 as *mut uint16)
        .offset(
            ((*v1).current_instruction as ::core::ffi::c_int & 3 as ::core::ffi::c_int) as isize,
        );
    StoneZoomer_E67A(ram_word!(game, cur_enemy_index), game);
}
pub unsafe fn FireZoomer_Init(game: &mut Game) {
    let E: *mut Enemy_FireZoomer =
        Get_FireZoomer(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).base.current_instruction = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3e2cc))
        as *const uint8 as *mut uint16)
        .offset(
            ((*E).base.current_instruction as ::core::ffi::c_int & 3 as ::core::ffi::c_int)
                as isize,
        );
    StoneZoomer_E67A(ram_word!(game, cur_enemy_index), game);
}
pub unsafe fn Zoomer_Instr_SetPreinstr(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*gEnemyData(k as ::core::ffi::c_int, game)).ai_preinstr = *jp.offset(0);
    jp.offset(1)
}
pub unsafe fn StoneZoomer_Init(game: &mut Game) {
    let E: *mut Enemy_StoneZoomer =
        Get_StoneZoomer(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).base.current_instruction = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3e2cc))
        as *const uint8 as *mut uint16)
        .offset(
            ((*E).base.current_instruction as ::core::ffi::c_int & 3 as ::core::ffi::c_int)
                as isize,
        );
    StoneZoomer_E67A(ram_word!(game, cur_enemy_index), game);
}
pub unsafe fn StoneZoomer_E67A(k: uint16, game: &mut Game) {
    let E: *mut Enemy_StoneZoomer = Get_StoneZoomer(k as ::core::ffi::c_int, game);
    (*E).base.spritemap_pointer = addr_kSpritemap_Nothing_A3 as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).szr_var_F = ((0xa3e6c1i32) as u16);
    if (*E).szr_parameter_1 != 255u16 {
        let v2: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3e5f0)) as *const uint8
            as *mut uint16)
            .offset((*E).szr_parameter_1 as isize);
        (*E).szr_var_A = v2;
        (*E).szr_var_B = v2;
    }
    if ((*E).base.properties as ::core::ffi::c_int & 3 as ::core::ffi::c_int) != 0i32 {
        if ((*E).base.properties as ::core::ffi::c_int & 3 as ::core::ffi::c_int) == 2i32 {
            (*E).szr_var_B = -((*E).szr_var_B as ::core::ffi::c_int) as uint16;
        }
    } else {
        (*E).szr_var_A = -((*E).szr_var_A as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn StoneZoomer_Main(game: &mut Game) {
    let E: *mut Enemy_StoneZoomer =
        Get_StoneZoomer(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    CallEnemyPreInstr((((*E).szr_var_F) as u32) | (0xa30000u32), game);
}
pub unsafe fn FireZoomer_Func_1(k: uint16, game: &mut Game) {
    let E: *mut Enemy_FireZoomer = Get_FireZoomer(k as ::core::ffi::c_int, game);
    if ram_word!(game, earthquake_timer) == 30u16 && ram_word!(game, earthquake_type) == 20u16 {
        (*E).fzr_var_03 = (*E).fzr_var_F;
        (*E).fzr_var_F = ((0xa3e785i32) as u16);
    }
    if Enemy_MoveRight_IgnoreSlopes(
        k,
        Shift8AddMagn((*E).fzr_var_A as int16, 1 as ::core::ffi::c_int) as int32,
        game,
    ) != 0
    {
        (*E).fzr_var_04 = 0 as uint16;
        EnemyFunc_C8AD(k, game);
        if Enemy_MoveDown(
            k,
            ((*E).fzr_var_B as int16 as int32) << 8 as ::core::ffi::c_int,
            game,
        ) != 0
        {
            (*E).fzr_var_A = -((*E).fzr_var_A as ::core::ffi::c_int) as uint16;
            let fzr_parameter_2: uint16 = (*E).fzr_parameter_2;
            let mut v7: uint16 = 0;
            if ((*E).fzr_var_B as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) == 0i32 {
                v7 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3e63c)) as *const uint8
                    as *mut uint16)
                    .offset(
                        (fzr_parameter_2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize,
                    );
            } else {
                v7 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3e630)) as *const uint8
                    as *mut uint16)
                    .offset(
                        (fzr_parameter_2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize,
                    );
            }
            (*E).base.current_instruction = v7;
            (*E).base.instruction_timer = 1 as uint16;
        }
    } else {
        let v8: uint16 = ((*E).fzr_var_04).wrapping_add(1u16);
        (*E).fzr_var_04 = v8;
        if v8 as ::core::ffi::c_int - 4 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
            (*E).fzr_var_B = -((*E).fzr_var_B as ::core::ffi::c_int) as uint16;
            let v9: uint16 = (*E).fzr_parameter_2;
            let mut v10: uint16 = 0;
            if ((*E).fzr_var_B as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) == 0i32 {
                v10 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3e63c)) as *const uint8
                    as *mut uint16)
                    .offset((v9 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
            } else {
                v10 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3e630)) as *const uint8
                    as *mut uint16)
                    .offset((v9 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
            }
            (*E).base.current_instruction = v10;
            (*E).base.instruction_timer = 1 as uint16;
        } else {
            (*E).fzr_var_03 = (*E).fzr_var_F;
            (*E).fzr_var_F = ((0xa3e785i32) as u16);
        }
    };
}
pub unsafe fn FireZoomer_Func_2(k: uint16, game: &mut Game) {
    let E: *mut Enemy_FireZoomer = Get_FireZoomer(k as ::core::ffi::c_int, game);
    if Enemy_MoveDown(
        k,
        (((*E).fzr_var_02 as uint32) << 16 as ::core::ffi::c_int | (*E).fzr_var_01 as uint32)
            as int32,
        game,
    ) != 0
    {
        if (*E).fzr_parameter_1 == 255u16 {
            (*E).fzr_var_A = 128 as uint16;
            (*E).fzr_var_B = 128 as uint16;
        }
        (*E).fzr_var_01 = 0 as uint16;
        (*E).fzr_var_02 = 0 as uint16;
        (*E).fzr_var_04 = 0 as uint16;
        (*E).fzr_var_F = (*E).fzr_var_03;
    } else {
        if (*E).fzr_var_02 as ::core::ffi::c_int - 4 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            AddToHiLo(
                &raw mut (*E).fzr_var_02,
                &raw mut (*E).fzr_var_01,
                0x8000 as uint32,
            );
        }
        if (*E).fzr_var_01 == 0 && (*E).fzr_var_02 == 0 {
            (*E).fzr_var_F = ((0xa3e6c8i32) as u16);
        }
    };
}
pub unsafe fn FireZoomer_Func_3(k: uint16, game: &mut Game) {
    let E: *mut Enemy_FireZoomer = Get_FireZoomer(k as ::core::ffi::c_int, game);
    if ram_word!(game, earthquake_timer) == 30u16 && ram_word!(game, earthquake_type) == 20u16 {
        (*E).fzr_var_03 = (*E).fzr_var_F;
        (*E).fzr_var_F = ((0xa3e785i32) as u16);
    }
    if Enemy_MoveDown(
        k,
        Shift8AddMagn((*E).fzr_var_B as int16, 1 as ::core::ffi::c_int) as int32,
        game,
    ) != 0
    {
        (*E).fzr_var_04 = 0 as uint16;
        let amt: int32 = FireZoomer_E8A5(k, game);
        if Enemy_MoveRight_IgnoreSlopes(k, amt, game) != 0 {
            (*E).fzr_var_B = -((*E).fzr_var_B as ::core::ffi::c_int) as uint16;
            let fzr_parameter_2: uint16 = (*E).fzr_parameter_2;
            let mut v6: uint16 = 0;
            if ((*E).fzr_var_A as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) == 0i32 {
                v6 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3e654)) as *const uint8
                    as *mut uint16)
                    .offset(
                        (fzr_parameter_2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize,
                    );
            } else {
                v6 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3e648)) as *const uint8
                    as *mut uint16)
                    .offset(
                        (fzr_parameter_2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize,
                    );
            }
            (*E).base.current_instruction = v6;
            (*E).base.instruction_timer = 1 as uint16;
        } else {
            EnemyFunc_C8AD(k, game);
        }
    } else {
        let v7: uint16 = ((*E).fzr_var_04).wrapping_add(1u16);
        (*E).fzr_var_04 = v7;
        if v7 as ::core::ffi::c_int - 4 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
            (*E).fzr_var_A = -((*E).fzr_var_A as ::core::ffi::c_int) as uint16;
            let v8: uint16 = (*E).fzr_parameter_2;
            let mut v9: uint16 = 0;
            if ((*E).fzr_var_A as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) == 0i32 {
                v9 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3e654)) as *const uint8
                    as *mut uint16)
                    .offset((v8 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
            } else {
                v9 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3e648)) as *const uint8
                    as *mut uint16)
                    .offset((v8 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
            }
            (*E).base.current_instruction = v9;
            (*E).base.instruction_timer = 1 as uint16;
        } else {
            (*E).fzr_var_03 = (*E).fzr_var_F;
            (*E).fzr_var_F = ((0xa3e785i32) as u16);
        }
    };
}
pub unsafe fn FireZoomer_E8A5(k: uint16, game: &mut Game) -> int32 {
    let E: *mut Enemy_FireZoomer =
        Get_FireZoomer(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let xpos: uint16 = (*E).base.x_pos;
    let ypos: uint16 = (if (*E).fzr_var_B as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0
    {
        (*E).base.y_pos as ::core::ffi::c_int - (*E).base.y_height as ::core::ffi::c_int
    } else {
        (*E).base.y_pos as ::core::ffi::c_int + (*E).base.y_height as ::core::ffi::c_int
            - 1 as ::core::ffi::c_int
    }) as uint16;
    CalculateBlockContainingPixelPos(xpos, ypos, game);
    if (ram_word!(game, level_data[ram_word!(game, cur_block_index)]) as ::core::ffi::c_int
        & 0xf000 as ::core::ffi::c_int)
        == 4096i32
        && (*(&raw mut game.g_ram as *mut uint8)
            .offset(0x16402)
            .offset(ram_word!(game, cur_block_index) as isize) as ::core::ffi::c_int
            & 0x1f as ::core::ffi::c_int)
            >= 5i32
    {
        let v1: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3e931)) as *const uint8
            as *mut uint16)
            .offset(
                ((4 as ::core::ffi::c_int
                    * (*(&raw mut game.g_ram as *mut uint8)
                        .offset(0x16402)
                        .offset(ram_word!(game, cur_block_index) as isize)
                        as ::core::ffi::c_int
                        & 0x1f as ::core::ffi::c_int)) as uint16
                    as ::core::ffi::c_int
                    >> 1 as ::core::ffi::c_int) as isize,
            );
        if ((*E).fzr_var_A as int16) >= 0i16 {
            Multiply16x16((*E).fzr_var_A, v1, game) as int32
        } else {
            -(Multiply16x16(-((*E).fzr_var_A as ::core::ffi::c_int) as uint16, v1, game) as int32)
        }
    } else {
        ((*E).fzr_var_A as int16 as int32) << 8 as ::core::ffi::c_int
    }
}
pub unsafe fn Metroid_Init(game: &mut Game) {
    let E: *mut Enemy_Metroid =
        Get_Metroid(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).base.current_instruction = addr_kMetroid_Ilist_E9CF as uint16;
    (*E).metroid_var_00 = CreateSpriteAtPos(
        (*E).base.x_pos,
        (*E).base.y_pos,
        50 as uint16,
        ((*E).base.vram_tiles_index) | ((*E).base.palette_index),
        game,
    );
    (*E).metroid_var_01 = CreateSpriteAtPos(
        (*E).base.x_pos,
        (*E).base.y_pos,
        52 as uint16,
        ((*E).base.vram_tiles_index) | ((*E).base.palette_index),
        game,
    );
    (*E).metroid_var_02 = 0 as uint16;
}
pub unsafe fn Metroid_Instr_2(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    QueueSfx2_Max6(0x50 as uint16, game);
    jp
}
pub unsafe fn Metroid_Instr_1(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let v2: ::core::ffi::c_int = NextRandom(game) as ::core::ffi::c_int & 7 as ::core::ffi::c_int;
    QueueSfx2_Max6(
        *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3ead6)) as *const uint8 as *mut uint16)
            .offset(v2 as isize),
        game,
    );
    jp
}
pub unsafe fn Metroid_Frozen(game: &mut Game) {
    Enemy_NormalFrozenAI_A3(game);
    let E: *mut Enemy_Metroid =
        Get_Metroid(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    if (*E).metroid_var_E != 0 {
        (*E).metroid_var_E = (*E).metroid_var_E.wrapping_sub(1);
        (*E).base.flash_timer = 2 as uint16;
    }
    let v1: ::core::ffi::c_int =
        (*E).metroid_var_00 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    set_ram_word!(game.sprite_palettes[v1] = 3072 as uint16);
    set_ram_word!(game.sprite_disable_flag[v1] = 1 as uint16);
    set_ram_word!(game.sprite_instr_list_ptrs[v1] = addr_kSpriteObject_Ilist_C3BA as uint16);
    let v2: ::core::ffi::c_int =
        (*E).metroid_var_01 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    set_ram_word!(game.sprite_palettes[v2] = 3072 as uint16);
    set_ram_word!(game.sprite_disable_flag[v2] = 1 as uint16);
    set_ram_word!(game.sprite_instr_list_ptrs[v2] = addr_kSpriteObject_Ilist_C4B6 as uint16);
}
pub unsafe fn Metroid_Hurt(game: &mut Game) {
    let E: *mut Enemy_Metroid =
        Get_Metroid(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    if ((*E).base.flash_timer as ::core::ffi::c_int & 2 as ::core::ffi::c_int) != 0i32 {
        let r18: uint16 = (*E).base.palette_index;
        let v1: ::core::ffi::c_int =
            (*E).metroid_var_00 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        set_ram_word!(
            game.sprite_palettes[v1] = (r18 as ::core::ffi::c_int
                | ram_word!(game, sprite_palettes[v1]) as ::core::ffi::c_int
                    & 0xf1ff as ::core::ffi::c_int)
                as uint16
        );
        let v2: ::core::ffi::c_int =
            (*E).metroid_var_01 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        set_ram_word!(
            game.sprite_palettes[v2] = (r18 as ::core::ffi::c_int
                | ram_word!(game, sprite_palettes[v2]) as ::core::ffi::c_int
                    & 0xf1ff as ::core::ffi::c_int)
                as uint16
        );
    } else {
        let v3: ::core::ffi::c_int =
            (*E).metroid_var_00 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let ref mut fresh10 = *((&raw mut game.g_ram as *mut uint8).offset(0xf078) as *mut uint16)
            .offset(v3 as isize);
        *fresh10 = (*fresh10) & (0xf1ffu16);
        let metroid_var_01: uint16 = (*E).metroid_var_01;
        let ref mut fresh11 = *((&raw mut game.g_ram as *mut uint8).offset(0xf078) as *mut uint16)
            .offset((metroid_var_01 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        *fresh11 = (*fresh11) & (0xf1ffu16);
    };
}
static off_A3EC09: [Option<Func_Y_V>; 4] = unsafe {
    [
        Some(Metroid_Func_1 as unsafe fn(uint16, &mut Game) -> ()),
        Some(Metroid_Func_2 as unsafe fn(uint16, &mut Game) -> ()),
        Some(Metroid_Func_3 as unsafe fn(uint16, &mut Game) -> ()),
        Some(Metroid_Func_4 as unsafe fn(uint16, &mut Game) -> ()),
    ]
};
pub unsafe fn Metroid_Main(game: &mut Game) {
    let E: *mut Enemy_Metroid =
        Get_Metroid(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    off_A3EC09[(*E).metroid_var_F as usize].expect("non-null function pointer")(
        (ram_word!(game, samus_y_pos)).wrapping_sub(8u16),
        game,
    );
    let r24: uint16 = ((*E).base.vram_tiles_index) | ((*E).base.palette_index);
    let v2: ::core::ffi::c_int =
        (*E).metroid_var_00 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    set_ram_word!(game.sprite_x_pos[v2] = (*E).base.x_pos);
    set_ram_word!(game.sprite_y_pos[v2] = (*E).base.y_pos);
    set_ram_word!(game.sprite_palettes[v2] = r24);
    set_ram_word!(game.sprite_disable_flag[v2] = 0 as uint16);
    let v4: ::core::ffi::c_int =
        (*E).metroid_var_01 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    set_ram_word!(game.sprite_x_pos[v4] = (*E).base.x_pos);
    set_ram_word!(game.sprite_y_pos[v4] = (*E).base.y_pos);
    set_ram_word!(game.sprite_palettes[v4] = r24);
    set_ram_word!(game.sprite_disable_flag[v4] = 0 as uint16);
}
pub unsafe fn Metroid_Func_1(varE32: uint16, game: &mut Game) {
    let mut current_block: u64;
    let mut v3: int16 = 0;
    let mut v4: int16 = 0;
    let mut v8: int16 = 0;
    let mut v9: int16 = 0;
    let E: *mut Enemy_Metroid =
        Get_Metroid(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let mut t: int32 = ((((((*E).base.y_pos) as i16).wrapping_sub((varE32) as i16))
        as ::core::ffi::c_int
        >> 2 as ::core::ffi::c_int) as int16 as int32)
        << 8 as ::core::ffi::c_int;
    let mut r18: uint16 = t as uint16;
    let mut r20: uint16 = (t >> 16 as ::core::ffi::c_int) as uint16;
    let metroid_var_C: uint16 = (*E).metroid_var_C;
    let mut v2: bool = metroid_var_C < r18;
    (*E).metroid_var_C = (metroid_var_C).wrapping_sub(r18);
    v3 = ((*E).metroid_var_D as ::core::ffi::c_int
        - (v2 as ::core::ffi::c_int + r20 as ::core::ffi::c_int)) as int16;
    (*E).metroid_var_D = v3 as uint16;
    if v3 < 0i16 {
        if (v3 as uint16) >= 0xfffdu16 {
            current_block = 3810685344454692134;
        } else {
            v4 = -(3 as ::core::ffi::c_int) as int16;
            current_block = 2868539653012386629;
        }
    } else if (v3 as uint16) < 3u16 {
        current_block = 3810685344454692134;
    } else {
        v4 = 3 as int16;
        current_block = 2868539653012386629;
    }
    if current_block == 2868539653012386629 {
        (*E).metroid_var_D = v4 as uint16;
        (*E).metroid_var_C = 0 as uint16;
    }
    if Enemy_MoveDown(
        ram_word!(game, cur_enemy_index),
        (((*E).metroid_var_D as uint32) << 16 as ::core::ffi::c_int | (*E).metroid_var_C as uint32)
            as int32,
        game,
    ) != 0
    {
        (*E).metroid_var_C = 0 as uint16;
        (*E).metroid_var_D = 0 as uint16;
    }
    t = ((((((*E).base.x_pos) as i16).wrapping_sub((ram_word!(game, samus_x_pos)) as i16))
        as ::core::ffi::c_int
        >> 2 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int;
    r18 = t as uint16;
    r20 = (t >> 16 as ::core::ffi::c_int) as uint16;
    let metroid_var_A: uint16 = (*E).metroid_var_A;
    v2 = metroid_var_A < r18;
    (*E).metroid_var_A = (metroid_var_A).wrapping_sub(r18);
    v8 = ((*E).metroid_var_B as ::core::ffi::c_int
        - (v2 as ::core::ffi::c_int + r20 as ::core::ffi::c_int)) as int16;
    (*E).metroid_var_B = v8 as uint16;
    if v8 < 0i16 {
        if (v8 as uint16) >= 0xfffdu16 {
            current_block = 8759308886461481507;
        } else {
            v9 = -(3 as ::core::ffi::c_int) as int16;
            current_block = 17478428563724192186;
        }
    } else if (v8 as uint16) < 3u16 {
        current_block = 8759308886461481507;
    } else {
        v9 = 3 as int16;
        current_block = 17478428563724192186;
    }
    if current_block == 17478428563724192186 {
        (*E).metroid_var_B = v9 as uint16;
        (*E).metroid_var_A = 0 as uint16;
    }
    if Enemy_MoveRight_IgnoreSlopes(
        ram_word!(game, cur_enemy_index),
        (((*E).metroid_var_B as uint32) << 16 as ::core::ffi::c_int | (*E).metroid_var_A as uint32)
            as int32,
        game,
    ) != 0
    {
        (*E).metroid_var_A = 0 as uint16;
        (*E).metroid_var_B = 0 as uint16;
    }
}
pub unsafe fn Metroid_Func_2(varE32: uint16, game: &mut Game) {
    let mut current_block: u64;
    let mut v1: int16 = 0;
    let mut v5: int16 = 0;
    let mut v2: uint16 = 0;
    let mut v6: uint16 = 0;
    let mut r18: uint16 = 0 as uint16;
    let mut r20: uint16 = 0 as uint16;
    let E: *mut Enemy_Metroid =
        Get_Metroid(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    *(&raw mut v1 as *mut uint8).offset(0) = (((varE32).wrapping_sub((*E).base.y_pos))
        as ::core::ffi::c_int
        >> 8 as ::core::ffi::c_int) as uint8;
    *(&raw mut v1 as *mut uint8).offset(
        (::core::mem::size_of::<int16>())
            .wrapping_div(::core::mem::size_of::<uint8>())
            .wrapping_sub(1_usize) as isize,
    ) = ((varE32) as u8).wrapping_sub(*(&raw mut (*E).base.y_pos as *mut uint8).offset(0));
    r20 =
        ((((v1) as u16) & (0xff00u16)) as ::core::ffi::c_int >> 3 as ::core::ffi::c_int) as uint16;
    if (r20 as ::core::ffi::c_int & 0x1000 as ::core::ffi::c_int) != 0i32 {
        r20 = (r20) | (0xe000u16);
    }
    if (r20 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        if r20 >= 0xfffdu16 {
            current_block = 12681381661298955615;
        } else {
            v2 = -(3 as ::core::ffi::c_int) as uint16;
            current_block = 2868539653012386629;
        }
    } else if r20 < 3u16 {
        current_block = 12681381661298955615;
    } else {
        v2 = 3 as uint16;
        current_block = 2868539653012386629;
    }
    if current_block == 2868539653012386629 {
        r20 = v2;
        r18 = 0 as uint16;
    }
    (*E).metroid_var_C = r18;
    (*E).metroid_var_D = r20;
    if Enemy_MoveDown(
        ram_word!(game, cur_enemy_index),
        ((r20 as uint32) << 16 as ::core::ffi::c_int | r18 as uint32) as int32,
        game,
    ) != 0
    {
        (*E).metroid_var_C = 0 as uint16;
        (*E).metroid_var_D = 0 as uint16;
    }
    r18 = 0 as uint16;
    r20 = 0 as uint16;
    *(&raw mut v5 as *mut uint8).offset(0) =
        (((ram_word!(game, samus_x_pos)).wrapping_sub((*E).base.x_pos)) as ::core::ffi::c_int
            >> 8 as ::core::ffi::c_int) as uint8;
    *(&raw mut v5 as *mut uint8).offset(
        (::core::mem::size_of::<int16>())
            .wrapping_div(::core::mem::size_of::<uint8>())
            .wrapping_sub(1_usize) as isize,
    ) = ((ram_word!(game, samus_x_pos)) as u8)
        .wrapping_sub(*(&raw mut (*E).base.x_pos as *mut uint8).offset(0));
    r20 =
        ((((v5) as u16) & (0xff00u16)) as ::core::ffi::c_int >> 3 as ::core::ffi::c_int) as uint16;
    if (r20 as ::core::ffi::c_int & 0x1000 as ::core::ffi::c_int) != 0i32 {
        r20 = (r20) | (0xe000u16);
    }
    if (r20 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        if r20 >= 0xfffdu16 {
            current_block = 8114601772576486457;
        } else {
            v6 = -(3 as ::core::ffi::c_int) as uint16;
            current_block = 18386322304582297246;
        }
    } else if r20 < 3u16 {
        current_block = 8114601772576486457;
    } else {
        v6 = 3 as uint16;
        current_block = 18386322304582297246;
    }
    if current_block == 18386322304582297246 {
        r20 = v6;
        r18 = 0 as uint16;
    }
    (*E).metroid_var_A = r18;
    (*E).metroid_var_B = r20;
    if Enemy_MoveRight_IgnoreSlopes(
        ram_word!(game, cur_enemy_index),
        ((r20 as uint32) << 16 as ::core::ffi::c_int | r18 as uint32) as int32,
        game,
    ) != 0
    {
        (*E).metroid_var_A = 0 as uint16;
        (*E).metroid_var_B = 0 as uint16;
    }
}
pub unsafe fn Metroid_Func_3(varE32: uint16, game: &mut Game) {
    let E: *mut Enemy_Metroid =
        Get_Metroid(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).base.x_pos = ram_word!(game, samus_x_pos);
    (*E).base.y_pos = varE32;
    (*E).metroid_var_A = 0 as uint16;
    (*E).metroid_var_B = 0 as uint16;
    (*E).metroid_var_C = 0 as uint16;
    (*E).metroid_var_D = 0 as uint16;
}
pub unsafe fn Metroid_Func_4(varE32: uint16, game: &mut Game) {
    let E: *mut Enemy_Metroid =
        Get_Metroid(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let v1: ::core::ffi::c_int = (*E).metroid_var_E as ::core::ffi::c_int & 3 as ::core::ffi::c_int;
    (*E).base.x_pos = ((*E).base.x_pos).wrapping_add(
        *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3ea3f)) as *const uint8 as *mut uint16)
            .offset(v1 as isize),
    );
    (*E).base.y_pos = ((*E).base.y_pos).wrapping_add(
        *(game.g_rom.offset(crate::rtl::lorom_offset(0xa3ea3f)) as *const uint8 as *mut uint16)
            .offset((v1 + 4 as ::core::ffi::c_int) as isize),
    );
    (*E).metroid_var_A = 0 as uint16;
    (*E).metroid_var_B = 0 as uint16;
    (*E).metroid_var_C = 0 as uint16;
    (*E).metroid_var_D = 0 as uint16;
    let fresh9 = (*E).metroid_var_E;
    (*E).metroid_var_E = (*E).metroid_var_E.wrapping_sub(1);
    if fresh9 == 1u16 {
        (*E).metroid_var_F = 0 as uint16;
        (*E).base.current_instruction = addr_kMetroid_Ilist_E9CF as uint16;
        (*E).base.instruction_timer = 1 as uint16;
    }
}
pub unsafe fn Metroid_Touch(game: &mut Game) {
    let E: *mut Enemy_Metroid =
        Get_Metroid(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let varE32: uint16 = (ram_word!(game, samus_y_pos)).wrapping_sub(8u16);
    if ram_word!(game, samus_contact_damage_index) != 0 {
        if (*E).metroid_var_F != 2u16 {
            (*E).metroid_var_A = 0 as uint16;
            (*E).metroid_var_C = 0 as uint16;
            let v2: int16 =
                (((*E).base.x_pos) as i16).wrapping_sub((ram_word!(game, samus_x_pos)) as i16);
            (*E).metroid_var_B = (if v2 < 0i16 {
                -(256 as ::core::ffi::c_int)
            } else {
                0 as ::core::ffi::c_int
            }) as uint16;
            *((&raw mut (*E).metroid_var_A as *mut uint8).offset(1) as *mut uint16) =
                ((v2 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int) as uint16;
            let v4: int16 = (((*E).base.y_pos) as i16).wrapping_sub((varE32) as i16);
            (*E).metroid_var_D = (if v4 < 0i16 {
                -(256 as ::core::ffi::c_int)
            } else {
                0 as ::core::ffi::c_int
            }) as uint16;
            *((&raw mut (*E).metroid_var_C as *mut uint8).offset(1) as *mut uint16) =
                ((v4 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int) as uint16;
            (*E).metroid_var_F = 0 as uint16;
            (*E).base.current_instruction = addr_kMetroid_Ilist_E9CF as uint16;
            (*E).base.instruction_timer = 1 as uint16;
        }
    } else {
        let v5: uint16 = ram_word!(game, cur_enemy_index);
        if (*E).metroid_var_F != 3u16 {
            if (ram_word!(game, random_enemy_counter) as ::core::ffi::c_int
                & 7 as ::core::ffi::c_int)
                == 7i32
                && ram_word!(game, samus_health) as ::core::ffi::c_int - 30 as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    == 0
            {
                QueueSfx3_Max6(0x2d as uint16, game);
            }
            Metroid_Func_5(v5, game);
        }
        if (*E).metroid_var_F < 2u16 {
            let mut v7: uint16 = 1 as uint16;
            if abs16(((*E).base.x_pos).wrapping_sub(ram_word!(game, samus_x_pos))) < 8u16
                && abs16(((*E).base.y_pos).wrapping_sub(varE32)) < 8u16
            {
                v7 = 2 as uint16;
                CallSomeSamusCode(0x12 as uint16, game);
            }
            (*E).metroid_var_F = v7;
            if v7 == 2u16 {
                (*E).base.current_instruction = addr_kMetroid_Ilist_EA25 as uint16;
                (*E).base.instruction_timer = 1 as uint16;
            }
        }
    };
}
pub unsafe fn Metroid_Func_5(k: uint16, game: &mut Game) {
    let mut v1: uint16 = 0;
    let varE32: uint16 = (ram_word!(game, samus_y_pos)).wrapping_sub(8u16);
    if (ram_word!(game, equipped_items) as ::core::ffi::c_int & 0x20 as ::core::ffi::c_int) != 0i32
    {
        v1 = 12288 as uint16;
    } else if ram_word!(game, equipped_items) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        v1 = 24576 as uint16;
    } else {
        v1 = -(16384 as ::core::ffi::c_int) as uint16;
    }
    let r18: uint16 = v1;
    let E: *mut Enemy_Metroid = Get_Metroid(k as ::core::ffi::c_int, game);
    let metroid_var_02: uint16 = (*E).metroid_var_02;
    let v4: bool = metroid_var_02 < r18;
    (*E).metroid_var_02 = (metroid_var_02).wrapping_sub(r18);
    if v4 {
        Samus_DealDamage(1 as uint16, game);
    }
}
pub unsafe fn Metroid_Shot(game: &mut Game) {
    let v0: uint16 = (2u16).wrapping_mul(ram_word!(game, collision_detection_index));
    let E: *mut Enemy_Metroid =
        Get_Metroid(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    if (*E).base.frozen_timer != 0 {
        let v3: uint16 = (ram_word!(game, projectile_type[(v0 as i32 >> 1i32)])) & (0xf00u16);
        if v3 == 256u16 || v3 == 512u16 {
            set_ram_word!(game.special_death_item_drop_x_origin_pos = (*E).base.x_pos);
            set_ram_word!(game.special_death_item_drop_y_origin_pos = (*E).base.y_pos);
            Enemy_NormalShotAI_SkipSomeParts_A3(game);
            if (*E).base.health == 0 {
                (*E).metroid_var_B = 0 as uint16;
                EnemyDeathAnimation(ram_word!(game, cur_enemy_index), 4 as uint16, game);
                CallSomeSamusCode(0x13 as uint16, game);
                *((&raw mut game.g_ram as *mut uint8).offset(0xef78) as *mut uint16).offset(
                    ((*E).metroid_var_00 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize,
                ) = 0 as uint16;
                let metroid_var_01: uint16 = (*E).metroid_var_01;
                *((&raw mut game.g_ram as *mut uint8).offset(0xef78) as *mut uint16).offset(
                    (metroid_var_01 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize,
                ) = 0 as uint16;
                Enemy_ItemDrop_Metroid(metroid_var_01, game);
            }
        }
    } else if (*E).metroid_var_F == 2u16 {
        if (ram_word!(game, projectile_type[(v0 as i32 >> 1i32)]) as ::core::ffi::c_int
            & 0xf00 as ::core::ffi::c_int)
            == 1280i32
        {
            (*E).metroid_var_E = 4 as uint16;
            (*E).metroid_var_F = 3 as uint16;
            (*E).base.current_instruction = addr_kMetroid_Ilist_E9CF as uint16;
            (*E).base.instruction_timer = 1 as uint16;
            CallSomeSamusCode(0x13 as uint16, game);
        }
    } else {
        (*E).metroid_var_A = 0 as uint16;
        (*E).metroid_var_C = 0 as uint16;
        let v7: int16 =
            (((*E).base.x_pos) as i16).wrapping_sub((ram_word!(game, projectile_x_pos[0])) as i16);
        (*E).metroid_var_B = (if v7 < 0i16 {
            -(256 as ::core::ffi::c_int)
        } else {
            0 as ::core::ffi::c_int
        }) as uint16;
        *((&raw mut (*E).metroid_var_A as *mut uint8).offset(1) as *mut uint16) =
            (32u16).wrapping_mul((v7) as u16);
        let v9: int16 =
            (((*E).base.y_pos) as i16).wrapping_sub((ram_word!(game, projectile_y_pos[0])) as i16);
        (*E).metroid_var_D = (if v9 < 0i16 {
            -(256 as ::core::ffi::c_int)
        } else {
            0 as ::core::ffi::c_int
        }) as uint16;
        *((&raw mut (*E).metroid_var_C as *mut uint8).offset(1) as *mut uint16) =
            (32u16).wrapping_mul((v9) as u16);
        (*E).metroid_var_F = 0 as uint16;
        (*E).base.current_instruction = addr_kMetroid_Ilist_E9CF as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        let v10: ::core::ffi::c_int =
            ram_word!(game, collision_detection_index) as ::core::ffi::c_int;
        if (ram_word!(game, projectile_type[v10]) as ::core::ffi::c_int & 2 as ::core::ffi::c_int)
            != 0i32
        {
            QueueSfx3_Max6(0xa as uint16, game);
            let r18: uint16 = ram_word!(game, projectile_damage[v10]);
            (*E).base.flash_timer = 4 as uint16;
            let metroid_parameter_2: uint16 = (*E).metroid_parameter_2;
            let mut v13: bool = metroid_parameter_2 < r18;
            let v12: uint16 = (metroid_parameter_2).wrapping_sub(r18);
            v13 = !v13;
            if v12 == 0 || !v13 {
                (*E).metroid_parameter_2 = 0 as uint16;
                (*E).base.frozen_timer = 400 as uint16;
                (*E).base.ai_handler_bits = ((*E).base.ai_handler_bits) | (4u16);
                return;
            }
            (*E).metroid_parameter_2 = v12;
        }
        QueueSfx2_Max6(0x5a as uint16, game);
    };
}
pub unsafe fn Metroid_Powerbomb(k: uint16, game: &mut Game) {
    NormalEnemyPowerBombAi(game);
    if (*Get_Metroid(k as ::core::ffi::c_int, game)).base.health == 0 {
        CallSomeSamusCode(0x13 as uint16, game);
        *((&raw mut game.g_ram as *mut uint8).offset(0xef78) as *mut uint16).offset(
            ((*Get_Metroid(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game))
                .metroid_var_00 as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int) as isize,
        ) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xef78) as *mut uint16).offset(
            ((*Get_Metroid(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game))
                .metroid_var_01 as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int) as isize,
        ) = 0 as uint16;
    }
}
