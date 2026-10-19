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
use crate::ram_map::ram_ptr;
use crate::ram_map::{ram_word, set_ram_word, update_ram_word};
use crate::rtl::abs16;
use crate::rtl::{AddToHiLo, SetHiLo};
use crate::rtl::{RomPtr, SnesDivide, Unreachable};
use ::core::ffi::c_char;
extern "C" {
    fn printf(__format: *const c_char, ...) -> i32;

}
pub use crate::types::{
    EnemyBase, EnemyData, EnemyProps, EnemySpawnData, Func_V, Func_Y_V, LongPtr, VoidP,
};
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct MaridiaSnailData2 {
    pub field_0: VoidP,
    pub field_2: u16,
    pub field_4: VoidP,
    pub field_6: u16,
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
        pad1: [u8; 26696],
        words: [
            bang_var_00, bang_var_01, bang_var_02, bang_var_03, bang_var_04, bang_var_05,
            bang_var_06, bang_var_07, bang_var_08, bang_var_09, bang_var_0A, bang_var_0B,
            bang_var_0C, bang_var_0D, bang_var_0E, bang_var_0F, bang_var_10, bang_var_11,
            bang_var_12, bang_var_13, bang_var_14, bang_var_15, bang_var_16, bang_var_17,
            bang_var_18, bang_var_19, bang_var_1A, bang_var_1B, bang_var_1C, bang_var_1D,
            bang_var_1E, bang_var_1F,
        ],
        pad2: [u8; 1984],
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
        pad1: [u8; 26696],
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
        pad1: [u8; 26696],
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
        pad1: [u8; 26696],
        words: [
            msl_var_00, msl_var_01, msl_var_02, msl_var_03, msl_var_04, msl_var_05,
            msl_var_06, msl_var_07, msl_var_08, msl_var_09, msl_var_0A, msl_var_0B,
            msl_var_0C, msl_var_0D, msl_var_0E, msl_var_0F, msl_var_10, msl_var_11,
            msl_var_12, msl_var_13, msl_var_14, msl_var_15, msl_var_16, msl_var_17,
            msl_var_18, msl_var_19, msl_var_1A, msl_var_1B, msl_var_1C, msl_var_1D,
            msl_var_1E, msl_var_1F,
        ],
        pad2: [u8; 1984],
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
        pad1: [u8; 26696],
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
        pad1: [u8; 26696],
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
        pad1: [u8; 26696],
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
        pad1: [u8; 26696],
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
        pad1: [u8; 26696],
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
        pad1: [u8; 26696],
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
        pad1: [u8; 26696],
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
        pad1: [u8; 26696],
        words: [
            mochtr_var_00, mochtr_var_01, mochtr_var_02, mochtr_var_03, mochtr_var_04,
            mochtr_var_05, mochtr_var_06, mochtr_var_07, mochtr_var_08, mochtr_var_09,
            mochtr_var_0A, mochtr_var_0B, mochtr_var_0C, mochtr_var_0D, mochtr_var_0E,
            mochtr_var_0F, mochtr_var_10, mochtr_var_11, mochtr_var_12, mochtr_var_13,
            mochtr_var_14, mochtr_var_15, mochtr_var_16, mochtr_var_17, mochtr_var_18,
            mochtr_var_19, mochtr_var_1A, mochtr_var_1B, mochtr_var_1C, mochtr_var_1D,
            mochtr_var_1E, mochtr_var_1F,
        ],
        pad2: [u8; 1984],
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
        pad1: [u8; 26696],
        words: [
            roach_var_00, roach_var_01, roach_var_02, roach_var_03, roach_var_04,
            roach_var_05, roach_var_06, roach_var_07, roach_var_08, roach_var_09,
            roach_var_0A, roach_var_0B, roach_var_0C, roach_var_0D, roach_var_0E,
            roach_var_0F, roach_var_10, roach_var_11, roach_var_12, roach_var_13,
            roach_var_14, roach_var_15, roach_var_16, roach_var_17, roach_var_18,
            roach_var_19, roach_var_1A, roach_var_1B, roach_var_1C, roach_var_1D,
            roach_var_1E, roach_var_1F,
        ],
        pad2: [u8; 1984],
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

const RomPtr_A3: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0xa3>;
#[inline]
unsafe fn get_EnemyDef_A2(a: u16, game: &mut Game) -> *mut EnemyDef {
    RomPtr((0xa00000u32) | ((a) as u32), game) as *mut EnemyDef
}
pub const fnMetalee_Func_1: i32 = 0xa38987;
pub const fnMetalee_Func_3: i32 = 0xa389d4;
pub const fnMetalee_Func_4: i32 = 0xa389f3;
pub const fnMetalee_Func_5: i32 = 0xa38a5c;
pub const fnMaridiaFish_Func_1: i32 = 0xa39132;
pub const fnMaridiaFish_Func_2: i32 = 0xa391ab;
pub const fnMaridiaFish_Func_3: i32 = 0xa39224;
pub const fnMaridiaFish_Func_4: i32 = 0xa39256;
pub const fnRoach_Func_9: i32 = 0xa3a2d7;
pub const fnRoach_Func_10: u32 = 10724097;
pub const fnRoach_Func_11: u32 = 10724107;
pub const fnRoach_Func_12: u32 = 10724117;
pub const fnRoach_Func_13: u32 = 10724133;
pub const fnRoach_Func_14: u32 = 10724155;
pub const fnRoach_Func_15: u32 = 10724171;
pub const fnRoach_Func_16: u32 = 10724224;
pub const fnRoach_Func_19: i32 = 0xa3a407;
pub const fnRoach_Func_20: i32 = 0xa3a40e;
pub const fnRoach_Func_21: i32 = 0xa3a440;
pub const fnRoach_Func_22: i32 = 0xa3a447;
pub const fnRoach_Func_23: i32 = 0xa3a44e;
pub const fnRoach_Func_24: i32 = 0xa3a462;
pub const fnRoach_Func_25: i32 = 0xa3a476;
pub const fnRoach_Func_26: i32 = 0xa3a4b6;
pub const fnRoach_Func_27: i32 = 0xa3a4f0;
pub const fnSidehopper_Func_4: i32 = 0xa3abd6;
pub const fnSidehopper_Func_5: u32 = 10726374;
pub const fnSidehopper_Func_6: u32 = 10726419;
pub const fnSidehopper_Func_7: i32 = 0xa3ac40;
pub const fnSidehopper_Func_8: i32 = 0xa3ac56;
pub const fnSidehopper_Func_9: i32 = 0xa3ac6c;
pub const fnSidehopper_Func_10: i32 = 0xa3ac8f;
pub const fnSidehopper_Func_11: i32 = 0xa3aca8;
pub const fnSidehopper_Func_12: i32 = 0xa3accb;
pub const fnSidehopper_Func_13: i32 = 0xa3ace4;
pub const fnSidehopper_Func_14: i32 = 0xa3ad0e;
pub const fnSidehopper_Func_15: i32 = 0xa3ad20;
pub const fnSidehopper_Func_16: i32 = 0xa3ad32;
pub const fnSidehopper_Func_17: i32 = 0xa3ad44;
pub const fnSidehopper_Func_18: i32 = 0xa3ad56;
pub const fnMaridiaRefillCandy_Func_1: i32 = 0xa3b482;
pub const fnMaridiaRefillCandy_Func_2: i32 = 0xa3b4a8;
pub const fnMaridiaRefillCandy_Func_3: i32 = 0xa3b4d6;
pub const fnBang_Func_6: i32 = 0xa3bca5;
pub const fnBang_Func_7: i32 = 0xa3bcc1;
pub const fnBang_Func_8: i32 = 0xa3bcc5;
pub const fnBang_Func_10: i32 = 0xa3bd1c;
pub const fnBang_Func_11: i32 = 0xa3bd2c;
pub const fnSkree_Func_1: i32 = 0xa3c6d5;
pub const fnSkree_Func_2: i32 = 0xa3c6f7;
pub const fnSkree_Func_3: i32 = 0xa3c716;
pub const fnSkree_Func_4: i32 = 0xa3c77f;
pub const fnnullsub_215: i32 = 0xa3cf5f;
pub const fnMaridiaSnail_Func_15: i32 = 0xa3d1b3;
pub const fnReflec_Func_1: i32 = 0xa3db0c;
pub const fnnullsub_304: i32 = 0xa3e6c1;
pub const fnFireZoomer_Func_1: i32 = 0xa3e6c8;
pub const fnFireZoomer_Func_2: i32 = 0xa3e785;
crate::ram_map::ram_pointer_functions! {
    gEnemyData(o: i32) -> EnemyData = [0xf78, o as isize];
    gEnemySpawnData(o: i32) -> EnemySpawnData = [0x7000, o as isize];
    Get_Bang(o: i32) -> Enemy_Bang = [0xf78, o as isize];
    Get_Elevator(o: i32) -> Enemy_Elevator = [0xf78, o as isize];
    Get_FireFlea(o: i32) -> Enemy_FireFlea = [0xf78, o as isize];
    Get_FireZoomer(o: i32) -> Enemy_FireZoomer = [0xf78, o as isize];
    Get_MaridiaFish(o: i32) -> Enemy_MaridiaFish = [0xf78, o as isize];
    Get_MaridiaRefillCandy(o: i32) -> Enemy_MaridiaRefillCandy = [0xf78, o as isize];
    Get_MaridiaSnail(o: i32) -> Enemy_MaridiaSnail = [0xf78, o as isize];
    Get_Metalee(o: i32) -> Enemy_Metalee = [0xf78, o as isize];
    Get_Metroid(o: i32) -> Enemy_Metroid = [0xf78, o as isize];
    Get_Mochtroid(o: i32) -> Enemy_Mochtroid = [0xf78, o as isize];
    Get_PlatformThatFallsWithSamus(o: i32) -> Enemy_PlatformThatFallsWithSamus = [0xf78, o as isize];
    Get_Reflec(o: i32) -> Enemy_Reflec = [0xf78, o as isize];
    Get_Roach(o: i32) -> Enemy_Roach = [0xf78, o as isize];
    Get_Sidehopper(o: i32) -> Enemy_Sidehopper = [0xf78, o as isize];
    Get_Skree(o: i32) -> Enemy_Skree = [0xf78, o as isize];
    Get_StoneZoomer(o: i32) -> Enemy_StoneZoomer = [0xf78, o as isize];
    Get_WreckedShipOrangeZoomer(o: i32) -> Enemy_WreckedShipOrangeZoomer = [0xf78, o as isize];
}
static g_word_A3A76D: [i16; 4] = [2, 0, -2, 0];
static g_word_A3A775: [i16; 4] = [0, -2, 0, 2];
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
pub unsafe fn Metalee_Instr_1(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*Get_Metalee(ram_word!(game, cur_enemy_index) as i32, game)).metalee_var_E = 1u16;
    jp
}
pub unsafe fn Metalee_Init(game: &mut Game) {
    let E: *mut Enemy_Metalee = Get_Metalee(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).metalee_var_C = 0u16;
    (*E).metalee_var_D = 0u16;
    (*E).metalee_var_E = 0u16;
    (*E).base.current_instruction = addr_kMetalee_Ilist_8910 as u16;
    (*E).metalee_var_B = ((0xa38987i32) as u16);
}
pub unsafe fn CallMetaleeFunc(ea: u32, game: &mut Game) {
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
    let E: *mut Enemy_Metalee = Get_Metalee(ram_word!(game, cur_enemy_index) as i32, game);
    CallMetaleeFunc((((*E).metalee_var_B) as u32) | (0xa30000u32), game);
}
pub unsafe fn Metalee_Func_1(game: &mut Game) {
    let E: *mut Enemy_Metalee = Get_Metalee(ram_word!(game, cur_enemy_index) as i32, game);
    if abs16(((*E).base.x_pos).wrapping_sub(ram_word!(game, samus_x_pos))) < 0x48u16 {
        Metalee_Func_2(ram_word!(game, cur_enemy_index), game);
        (*E).metalee_var_C = (*E).metalee_var_C.wrapping_add(1);
        Metalee_Func_6(game);
        (*E).metalee_var_B = ((0xa389d4i32) as u16);
    }
}
pub unsafe fn Metalee_Func_2(k: u16, game: &mut Game) {
    let E: *mut Enemy_Metalee = Get_Metalee(k as i32, game);
    let div: u16 = SnesDivide(
        (ram_word!(game, samus_y_pos)).wrapping_sub((*E).base.y_pos),
        24u8,
    ) as u8 as u16;
    (*E).metalee_var_F = (div).wrapping_add(4u16);
}
pub unsafe fn Metalee_Func_3(k: u16, game: &mut Game) {
    let E: *mut Enemy_Metalee = Get_Metalee(k as i32, game);
    if (*E).metalee_var_E != 0 {
        (*E).metalee_var_E = 0u16;
        (*E).metalee_var_C = (*E).metalee_var_C.wrapping_add(1);
        Metalee_Func_6(game);
        (*E).metalee_var_B = ((0xa389f3i32) as u16);
        QueueSfx2_Max6(0x5bu16, game);
    }
}
pub unsafe fn Metalee_Func_4(game: &mut Game) {
    let E: *mut Enemy_Metalee = Get_Metalee(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).metalee_var_A = 21u16;
    let v1: u16 = ((*E).base.properties) | (3u16);
    (*E).base.properties = v1;
    if EnemyFunc_BF8A(
        ram_word!(game, cur_enemy_index),
        v1,
        ((*E).metalee_var_F as i16 as i32) << 16i32,
        game,
    ) as i32
        & 1i32
        != 0
    {
        (*E).base.instruction_timer = 1u16;
        (*E).base.timer = 0u16;
        (*E).metalee_var_B = ((0xa38a5ci32) as u16);
        QueueSfx2_Max6(0x5cu16, game);
    } else {
        (*E).base.y_pos = ((*E).base.y_pos).wrapping_add((*E).metalee_var_F);
        (*E).base.x_pos = ((*E).base.x_pos as i32
            + if (((*E).base.x_pos) as i16).wrapping_sub((ram_word!(game, samus_x_pos)) as i16)
                >= 0i16
            {
                -2i32
            } else {
                2i32
            }) as u16;
    };
}
pub unsafe fn Metalee_Func_5(game: &mut Game) {
    let E: *mut Enemy_Metalee = Get_Metalee(ram_word!(game, cur_enemy_index) as i32, game);
    let fresh8 = (*E).metalee_var_A;
    (*E).metalee_var_A = (*E).metalee_var_A.wrapping_sub(1);
    if fresh8 == 1u16 {
        (*gEnemySpawnData(ram_word!(game, cur_enemy_index) as i32, game)).vram_tiles_index =
            ((*E).base.vram_tiles_index) | ((*E).base.palette_index);
        (*E).base.palette_index = 2560u16;
        (*E).base.vram_tiles_index = 0u16;
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Deleted) as u16);
    } else {
        if (*E).metalee_var_A == 8u16 {
            let v2: u16 = ram_word!(game, cur_enemy_index);
            let mut parameter = SpawnEprojWithGfx(8, v2, addr_stru_868BFA as u16, game);
            parameter = SpawnEprojWithGfx(parameter, v2, addr_stru_868C08 as u16, game);
            parameter = SpawnEprojWithGfx(parameter, v2, addr_stru_868C16 as u16, game);
            SpawnEprojWithGfx(parameter, v2, addr_stru_868C24 as u16, game);
        }
        (*E).base.y_pos = (*E).base.y_pos.wrapping_add(1);
    };
}
pub unsafe fn Metalee_Func_6(game: &mut Game) {
    let E: *mut Enemy_Metalee = Get_Metalee(ram_word!(game, cur_enemy_index) as i32, game);
    let metalee_var_C: u16 = (*E).metalee_var_C;
    if metalee_var_C != (*E).metalee_var_D {
        (*E).metalee_var_D = metalee_var_C;
        (*E).base.current_instruction =
            crate::rtl::rom_table_word!(game, 0xa3894e, metalee_var_C as isize);
        (*E).base.instruction_timer = 1u16;
        (*E).base.timer = 0u16;
    }
}
pub unsafe fn Metalee_Shot(game: &mut Game) {
    let E: *mut Enemy_Metalee = Get_Metalee(ram_word!(game, cur_enemy_index) as i32, game);
    let varE2A: u16 = (*E).base.vram_tiles_index;
    let varE2C: u16 = (*E).base.palette_index;
    Enemy_NormalShotAI_A3(game);
    if (*Get_Metalee(ram_word!(game, cur_enemy_index) as i32, game))
        .base
        .health
        == 0
    {
        (*E).base.vram_tiles_index = varE2A;
        (*E).base.palette_index = varE2C;
        let v2: u16 = ram_word!(game, cur_enemy_index);
        SpawnEprojWithGfx(
            (*E).metalee_var_A,
            ram_word!(game, cur_enemy_index),
            addr_stru_868BFA as u16,
            game,
        );
        SpawnEprojWithGfx(0u16, v2, addr_stru_868C08 as u16, game);
        SpawnEprojWithGfx(0u16, v2, addr_stru_868C16 as u16, game);
        SpawnEprojWithGfx(0u16, v2, addr_stru_868C24 as u16, game);
        (*E).base.vram_tiles_index = 0u16;
        (*E).base.palette_index = 0u16;
    }
}
pub unsafe fn Fireflea_Init(game: &mut Game) {
    let E: *mut Enemy_FireFlea = Get_FireFlea(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.current_instruction = addr_kFireflea_Ilist_8C2F as u16;
    if ((*E).ffa_parameter_1 as i32 & 2i32) != 0i32 {
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
pub unsafe fn Fireflea_Func_1(k: u16, game: &mut Game) {
    let E: *mut Enemy_FireFlea = Get_FireFlea(k as i32, game);
    (*E).ffa_var_E = (*E).base.x_pos;
    (*E).ffa_var_F = (*E).base.y_pos;
}
pub unsafe fn Fireflea_Func_2(k: u16, game: &mut Game) {
    let E: *mut Enemy_FireFlea = Get_FireFlea(k as i32, game);
    (*E).ffa_var_D =
        ((*(&raw mut (*E).ffa_parameter_1 as *mut u8).offset(1isize) as i32) << 8i32) as u16;
}
pub unsafe fn Fireflea_Func_3(k: u16, game: &mut Game) {
    let E: *mut Enemy_FireFlea = Get_FireFlea(k as i32, game);
    let mut v2: u16 =
        (8u16).wrapping_mul((*(&raw mut (*E).ffa_parameter_2 as *mut u8).offset(0)) as u16);
    if ((*E).ffa_parameter_1 as i32 & 1i32) == 0i32 {
        v2 = (v2).wrapping_add(4u16);
    }
    (*E).ffa_var_02 = v2;
    let v3: i32 = v2 as i32 >> 1i32;
    (*E).ffa_var_B = crate::rtl::rom_table_word!(game, 0xa28187, v3 as isize);
    (*E).ffa_var_A = crate::rtl::rom_table_word!(game, 0xa28187, (v3 + 1i32) as isize);
}
pub unsafe fn Fireflea_Func_4(k: u16, game: &mut Game) {
    let E: *mut Enemy_FireFlea = Get_FireFlea(k as i32, game);
    (*E).ffa_var_C = *((game.g_rom.offset(crate::rtl::lorom_offset(0xa38d1d)) as *mut u16)
        .offset(*(&raw mut (*E).ffa_parameter_2 as *mut u8).offset(1isize) as isize)
        as *mut u8)
        .offset(0) as u16;
}
pub unsafe fn Fireflea_Func_5(k: u16, game: &mut Game) {
    let E: *mut Enemy_FireFlea = Get_FireFlea(k as i32, game);
    (*E).base.x_pos =
        ((*E).ffa_var_E).wrapping_add(CosineMult8bit((*E).ffa_var_D, (*E).ffa_var_C, game));
    (*E).base.y_pos =
        ((*E).ffa_var_F).wrapping_add(SineMult8bit((*E).ffa_var_D, (*E).ffa_var_C, game));
}
pub unsafe fn Fireflea_Func_6(k: u16, game: &mut Game) {
    let E: *mut Enemy_FireFlea = Get_FireFlea(k as i32, game);
    (*E).ffa_var_00 = ((*E).base.y_pos).wrapping_sub((*E).ffa_var_C);
    (*E).ffa_var_01 = ((*E).ffa_var_C).wrapping_add((*E).base.y_pos);
}
pub unsafe fn Fireflea_Main(game: &mut Game) {
    let E: *mut Enemy_FireFlea = Get_FireFlea(ram_word!(game, cur_enemy_index) as i32, game);
    if ((*E).ffa_parameter_1 as i32 & 2i32) != 0i32 {
        (*E).base.x_pos = ((*E).ffa_var_E).wrapping_add(CosineMult8bit(
            *(&raw mut (*E).ffa_var_D as *mut u8).offset(1isize) as u16,
            (*E).ffa_var_C,
            game,
        ));
        (*E).base.y_pos = ((*E).ffa_var_F).wrapping_add(SineMult8bit(
            *(&raw mut (*E).ffa_var_D as *mut u8).offset(1isize) as u16,
            (*E).ffa_var_C,
            game,
        ));
        (*E).ffa_var_D = ((*E).ffa_var_D)
            .wrapping_add(*((&raw mut (*E).ffa_var_A as *mut u8).offset(1) as *mut u16));
    } else {
        let v2: i32 = (*E).ffa_var_02 as i32 >> 1i32;
        AddToHiLo(
            &raw mut (*E).base.y_pos,
            &raw mut (*E).base.y_subpos,
            (crate::rtl::rom_table_word!(game, 0xa28187, v2 as isize) as u32) << 16i32
                | crate::rtl::rom_table_word!(game, 0xa28187, (v2 + 1i32) as isize) as u32,
        );
        if (((*E).base.y_pos) as i16).wrapping_sub(((*E).ffa_var_00) as i16) < 0i16
            || (((*E).base.y_pos) as i16).wrapping_sub(((*E).ffa_var_01) as i16) >= 0i16
        {
            (*E).ffa_var_02 = ((*E).ffa_var_02) ^ (4u16);
        }
    };
}
pub unsafe fn Fireflea_Touch(k: u16, game: &mut Game) {
    let v1: u16 = 0u16;
    Enemy_NormalTouchAI_A3(game);
    EnemyDeathAnimation(k, v1, game);
    if ram_word!(game, fireflea_darkness_level) as i32 - 12i32 & 0x8000i32 != 0 {
        update_ram_word!(game.fireflea_darkness_level, wrapping_add(2u16));
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
    if (*Get_FireFlea(ram_word!(game, cur_enemy_index) as i32, game))
        .base
        .health
        == 0
        && ram_word!(game, fireflea_darkness_level) as i32 - 12i32 & 0x8000i32 != 0
    {
        update_ram_word!(game.fireflea_darkness_level, wrapping_add(2u16));
    }
}
pub unsafe fn MaridiaFish_Instr_3(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*Get_MaridiaFish(ram_word!(game, cur_enemy_index) as i32, game))
        .base
        .layer = 6u16;
    jp
}
pub unsafe fn MaridiaFish_Instr_1(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*Get_MaridiaFish(ram_word!(game, cur_enemy_index) as i32, game))
        .base
        .layer = 2u16;
    jp
}
pub unsafe fn MaridiaFish_Instr_2(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*Get_MaridiaFish(ram_word!(game, cur_enemy_index) as i32, game)).mfh_var_01 = 1u16;
    jp
}
pub unsafe fn MaridiaFish_Init(game: &mut Game) {
    let E: *mut Enemy_MaridiaFish = Get_MaridiaFish(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.current_instruction = addr_kMaridiaFish_Ilist_902A as u16;
    (*E).mfh_var_A = ((0xa39132i32) as u16);
    if *(&raw mut (*E).mfh_parameter_1 as *mut u8).offset(1isize) == 0 {
        (*E).base.current_instruction = addr_kMaridiaFish_Ilist_9060 as u16;
        (*E).mfh_var_A = ((0xa391abi32) as u16);
    }
    let v1: i32 = 8i32 * *(&raw mut (*E).mfh_parameter_1 as *mut u8).offset(0) as i32 >> 1i32;
    (*E).mfh_var_C = crate::rtl::rom_table_word!(game, 0xa28187, v1 as isize);
    (*E).mfh_var_B = crate::rtl::rom_table_word!(game, 0xa28187, (v1 + 1i32) as isize);
    (*E).mfh_var_E = crate::rtl::rom_table_word!(game, 0xa28187, (v1 + 2i32) as isize);
    (*E).mfh_var_D = crate::rtl::rom_table_word!(game, 0xa28187, (v1 + 3i32) as isize);
    (*E).mfh_var_00 = *(&raw mut (*E).mfh_parameter_2 as *mut u8).offset(0) as u16;
    (*E).mfh_var_02 = *(&raw mut (*E).mfh_parameter_2 as *mut u8).offset(1isize) as u16;
    (*E).mfh_var_F = 0u16;
    (*E).mfh_var_01 = 0u16;
    (*E).mfh_var_03 = SineMult8bit((*E).mfh_var_F, (*E).mfh_var_00, game);
}
pub unsafe fn MaridiaFish_Func_1(game: &mut Game) {
    let E: *mut Enemy_MaridiaFish = Get_MaridiaFish(ram_word!(game, cur_enemy_index) as i32, game);
    if Enemy_MoveRight_IgnoreSlopes(
        ram_word!(game, cur_enemy_index),
        (((*E).mfh_var_E as u32) << 16i32 | (*E).mfh_var_D as u32) as i32,
        game,
    ) != 0
    {
        (*E).mfh_var_A = ((0xa39224i32) as u16);
        (*E).base.instruction_timer = 1u16;
        (*E).base.timer = 0u16;
        (*E).base.current_instruction = addr_kMaridiaFish_Ilist_903C as u16;
    } else {
        let v2: u16 = SineMult8bit((*E).mfh_var_F, (*E).mfh_var_00, game);
        (*E).mfh_var_04 = v2;
        if Enemy_MoveDown(
            ram_word!(game, cur_enemy_index),
            ((((v2) as i16).wrapping_sub(((*E).mfh_var_03) as i16)) as i32) << 16i32,
            game,
        ) != 0
        {
            (*E).mfh_var_A = ((0xa39224i32) as u16);
            (*E).base.instruction_timer = 1u16;
            (*E).base.timer = 0u16;
            (*E).base.current_instruction = addr_kMaridiaFish_Ilist_903C as u16;
        } else {
            (*E).mfh_var_F = ((*(&raw mut (*E).mfh_var_02 as *mut u8).offset(0))
                .wrapping_add(((*E).mfh_var_F) as u8)) as u16;
        }
    }
    (*E).mfh_var_03 = (*E).mfh_var_04;
}
pub unsafe fn MaridiaFish_Func_2(game: &mut Game) {
    let E: *mut Enemy_MaridiaFish = Get_MaridiaFish(ram_word!(game, cur_enemy_index) as i32, game);
    if Enemy_MoveRight_IgnoreSlopes(
        ram_word!(game, cur_enemy_index),
        (((*E).mfh_var_C as u32) << 16i32 | (*E).mfh_var_B as u32) as i32,
        game,
    ) != 0
    {
        (*E).mfh_var_A = ((0xa39256i32) as u16);
        (*E).base.instruction_timer = 1u16;
        (*E).base.timer = 0u16;
        (*E).base.current_instruction = addr_kMaridiaFish_Ilist_9072 as u16;
    } else {
        let v2: u16 = SineMult8bit((*E).mfh_var_F, (*E).mfh_var_00, game);
        (*E).mfh_var_04 = v2;
        if Enemy_MoveDown(
            ram_word!(game, cur_enemy_index),
            ((((v2) as i16).wrapping_sub(((*E).mfh_var_03) as i16)) as i32) << 16i32,
            game,
        ) != 0
        {
            (*E).mfh_var_A = ((0xa39256i32) as u16);
            (*E).base.instruction_timer = 1u16;
            (*E).base.timer = 0u16;
            (*E).base.current_instruction = addr_kMaridiaFish_Ilist_9072 as u16;
        } else {
            (*E).mfh_var_F = ((*(&raw mut (*E).mfh_var_02 as *mut u8).offset(0))
                .wrapping_add(((*E).mfh_var_F) as u8)) as u16;
        }
    }
    (*E).mfh_var_03 = (*E).mfh_var_04;
}
pub unsafe fn MaridiaFish_Func_3(game: &mut Game) {
    let E: *mut Enemy_MaridiaFish = Get_MaridiaFish(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).mfh_var_01 != 0 {
        (*E).mfh_var_01 = 0u16;
        (*E).mfh_var_A = ((0xa391abi32) as u16);
        (*E).mfh_var_02 = -((*E).mfh_var_02 as i32) as u16;
        (*E).base.instruction_timer = 1u16;
        (*E).base.timer = 0u16;
        (*E).base.current_instruction = addr_kMaridiaFish_Ilist_9060 as u16;
    }
}
pub unsafe fn MaridiaFish_Func_4(game: &mut Game) {
    let E: *mut Enemy_MaridiaFish = Get_MaridiaFish(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).mfh_var_01 != 0 {
        (*E).mfh_var_01 = 0u16;
        (*E).mfh_var_A = ((0xa39132i32) as u16);
        (*E).mfh_var_02 = -((*E).mfh_var_02 as i32) as u16;
        (*E).base.instruction_timer = 1u16;
        (*E).base.timer = 0u16;
        (*E).base.current_instruction = addr_kMaridiaFish_Ilist_902A as u16;
    }
}
pub unsafe fn CallMaridiaFishFunc(ea: u32, game: &mut Game) {
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
    let E: *mut Enemy_MaridiaFish = Get_MaridiaFish(ram_word!(game, cur_enemy_index) as i32, game);
    CallMaridiaFishFunc((((*E).mfh_var_A) as u32) | (0xa30000u32), game);
}
pub unsafe fn Elevator_Init(game: &mut Game) {
    let E: *mut Enemy_Elevator = Get_Elevator(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.spritemap_pointer = addr_kSpritemap_Nothing_A3 as u16;
    (*E).base.instruction_timer = 1u16;
    (*E).base.timer = 0u16;
    (*E).base.current_instruction = addr_kElevator_Ilist_94D6 as u16;
    (*E).elevat_parameter_1 = ((*E).elevat_parameter_1).wrapping_mul(2u16);
    (*E).elevat_var_A = (*E).base.y_pos;
    if ram_word!(game, elevator_status) != 2u16 {
        let ref mut fresh5 = *(ram_ptr!(game, u16; 0xe18));
        *fresh5 = 0u16;
        set_ram_word!(game.elevator_flags = *fresh5);
    }
    if (ram_word!(game, elevator_status) as u32) << 16i32 | ram_word!(game, elevator_flags) as u32
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
        && (ram_word!(game, elevator_status) as u32) << 16i32
            | ram_word!(game, elevator_flags) as u32
            != 0
    {
        off_A39540[ram_word!(game, elevator_status) as usize].expect("non-null function pointer")(
            game,
        );
    }
}
pub unsafe fn Elevator_Func_1(game: &mut Game) {
    if (crate::rtl::rom_table_word!(
        game,
        0xa394e2,
        ((*Get_Elevator(ram_word!(game, cur_enemy_index) as i32, game)).elevat_parameter_1 as i32
            >> 1i32) as isize,
    ) as i32
        & ram_word!(game, joypad1_newkeys) as i32)
        != 0i32
    {
        QueueSfx3_Max6(0xbu16, game);
        QueueSfx1_Max6(0x32u16, game);
        CallSomeSamusCode(7u16, game);
        ResetProjectileData(game);
        Elevator_Func_4(game);
        update_ram_word!(game.elevator_status, wrapping_add(1));
    } else {
        set_ram_word!(game.elevator_flags = 0u16);
    };
}
pub unsafe fn Elevator_Func_2(game: &mut Game) {
    let E: *mut Enemy_Elevator = Get_Elevator(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).elevat_parameter_1 != 0 {
        set_ram_word!(game.elevator_direction = 0x8000u16);
        AddToHiLo(
            &raw mut (*E).base.y_pos,
            &raw mut (*E).base.y_subpos,
            -0x18000i32 as u32,
        );
    } else {
        set_ram_word!(game.elevator_direction = 0u16);
        AddToHiLo(
            &raw mut (*E).base.y_pos,
            &raw mut (*E).base.y_subpos,
            0x18000u32,
        );
    }
    Elevator_Func_4(game);
}
pub unsafe fn Elevator_Func_3(game: &mut Game) {
    update_ram_word!(game.elevator_status, wrapping_add(1));
    Elevator_Func3b(game);
}
pub unsafe fn Elevator_Func3b(game: &mut Game) {
    let E: *mut Enemy_Elevator = Get_Elevator(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).elevat_parameter_1 != 0 {
        AddToHiLo(
            &raw mut (*E).base.y_pos,
            &raw mut (*E).base.y_subpos,
            0x18000u32,
        );
        if (*E).base.y_pos < (*E).elevat_var_A {
            Elevator_Func_4(game);
            return;
        }
    } else {
        AddToHiLo(
            &raw mut (*E).base.y_pos,
            &raw mut (*E).base.y_subpos,
            -0x18000i32 as u32,
        );
        if (*E).base.y_pos >= (*E).elevat_var_A {
            Elevator_Func_4(game);
            return;
        }
    }
    set_ram_word!(game.elevator_status = 0u16);
    set_ram_word!(game.elevator_flags = 0u16);
    QueueSfx3_Max6(0x25u16, game);
    (*E).base.y_pos = (*E).elevat_var_A;
    CallSomeSamusCode(0xbu16, game);
    Elevator_Func_4(game);
}
pub unsafe fn Elevator_Func_4(game: &mut Game) {
    let E: *mut Enemy_Elevator = Get_Elevator(ram_word!(game, cur_enemy_index) as i32, game);
    set_ram_word!(game.samus_y_pos = ((*E).base.y_pos).wrapping_sub(26u16));
    set_ram_word!(game.samus_y_subpos = 0u16);
    set_ram_word!(game.samus_x_pos = (*E).base.x_pos);
    set_ram_word!(game.samus_y_speed = 0u16);
    set_ram_word!(game.samus_y_subspeed = 0u16);
}
pub unsafe fn Crab_Init(game: &mut Game) {
    let v1: *mut EnemyData = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
    (*v1).parameter_2 = 8u16;
    (*v1).current_instruction = crate::rtl::rom_table_word!(
        game,
        0xa396db,
        ((*v1).current_instruction as i32 & 3i32) as isize
    );
    StoneZoomer_E67A(ram_word!(game, cur_enemy_index), game);
}
pub fn Crab_Func_1() {}
pub unsafe fn Slug_Init(game: &mut Game) {
    let v1: *mut EnemyData = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
    (*v1).parameter_2 = 10u16;
    (*v1).current_instruction = crate::rtl::rom_table_word!(
        game,
        0xa3992b,
        ((*v1).current_instruction as i32 & 3i32) as isize
    );
    StoneZoomer_E67A(ram_word!(game, cur_enemy_index), game);
}
pub fn Slug_Func_1() {}
pub unsafe fn PlatformThatFallsWithSamus_Instr_3(
    k: u16,
    jp: *const u16,
    game: &mut Game,
) -> *const u16 {
    (*Get_PlatformThatFallsWithSamus(ram_word!(game, cur_enemy_index) as i32, game))
        .ptfwss_var_02 = 0u16;
    jp
}
pub unsafe fn PlatformThatFallsWithSamus_Instr_4(
    k: u16,
    jp: *const u16,
    game: &mut Game,
) -> *const u16 {
    (*Get_PlatformThatFallsWithSamus(ram_word!(game, cur_enemy_index) as i32, game))
        .ptfwss_var_02 = 1u16;
    jp
}
pub unsafe fn PlatformThatFallsWithSamus_Instr_1(
    k: u16,
    jp: *const u16,
    game: &mut Game,
) -> *const u16 {
    (*Get_PlatformThatFallsWithSamus(ram_word!(game, cur_enemy_index) as i32, game))
        .ptfwss_var_02 = 0u16;
    jp
}
pub unsafe fn PlatformThatFallsWithSamus_Instr_2(
    k: u16,
    jp: *const u16,
    game: &mut Game,
) -> *const u16 {
    (*Get_PlatformThatFallsWithSamus(ram_word!(game, cur_enemy_index) as i32, game))
        .ptfwss_var_02 = 1u16;
    jp
}
pub unsafe fn PlatformThatFallsWithSamus_Init(game: &mut Game) {
    let E: *mut Enemy_PlatformThatFallsWithSamus =
        Get_PlatformThatFallsWithSamus(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).ptfwss_var_1F = -1i32 as u16;
    let ptfwss_parameter_1: u16 = (*E).ptfwss_parameter_1;
    (*E).ptfwss_var_02 = ptfwss_parameter_1;
    if ptfwss_parameter_1 != 0 {
        PlatformThatFalls_Init(
            ram_word!(game, cur_enemy_index),
            addr_kPlatformThatFallsWithSamus_Ilist_9BFD as u16,
            game,
        );
    } else {
        PlatformThatFalls_Init(
            ram_word!(game, cur_enemy_index),
            addr_kPlatformThatFallsWithSamus_Ilist_9BE7 as u16,
            game,
        );
    };
}
pub unsafe fn FastMovingSlowSinkingPlatform_Init(game: &mut Game) {
    let E: *mut Enemy_PlatformThatFallsWithSamus =
        Get_PlatformThatFallsWithSamus(ram_word!(game, cur_enemy_index) as i32, game);
    let ptfwss_parameter_1: u16 = (*E).ptfwss_parameter_1;
    (*E).ptfwss_var_02 = ptfwss_parameter_1;
    if ptfwss_parameter_1 != 0 {
        PlatformThatFalls_Init(
            ram_word!(game, cur_enemy_index),
            addr_kPlatformThatFallsWithSamus_Ilist_9C55 as u16,
            game,
        );
    } else {
        PlatformThatFalls_Init(
            ram_word!(game, cur_enemy_index),
            addr_kPlatformThatFallsWithSamus_Ilist_9C3F as u16,
            game,
        );
    };
}
pub unsafe fn PlatformThatFalls_Init(k: u16, j: u16, game: &mut Game) {
    let E: *mut Enemy_PlatformThatFallsWithSamus = Get_PlatformThatFallsWithSamus(k as i32, game);
    (*E).base.current_instruction = j;
    let v3: i32 = 8i32 * *(&raw mut (*E).ptfwss_parameter_2 as *mut u8).offset(0) as i32 >> 1i32;
    (*E).ptfwss_var_C = crate::rtl::rom_table_word!(game, 0xa28187, v3 as isize);
    (*E).ptfwss_var_B = crate::rtl::rom_table_word!(game, 0xa28187, (v3 + 1i32) as isize);
    (*E).ptfwss_var_E = crate::rtl::rom_table_word!(game, 0xa28187, (v3 + 2i32) as isize);
    (*E).ptfwss_var_D = crate::rtl::rom_table_word!(game, 0xa28187, (v3 + 3i32) as isize);
    (*E).ptfwss_var_00 = 0u16;
    (*E).ptfwss_var_03 = 0u16;
    (*E).ptfwss_var_04 = 0u16;
    (*E).ptfwss_var_A = ((*E).base.y_pos).wrapping_add(1u16);
    (*E).ptfwss_var_F = 0u16;
    (*E).ptfwss_var_05 = *(&raw mut (*E).ptfwss_parameter_2 as *mut u8).offset(1isize) as u16;
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
    let E: *mut Enemy_PlatformThatFallsWithSamus =
        Get_PlatformThatFallsWithSamus(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).ptfwss_var_00 = 0u16;
    if CheckIfEnemyTouchesSamus(ram_word!(game, cur_enemy_index), game) != 0 {
        (*E).ptfwss_var_00 = 1u16;
    }
    off_A39C97[(*E).ptfwss_var_02 as usize].expect("non-null function pointer")(game);
    let v3: i32 = (*Get_PlatformThatFallsWithSamus(ram_word!(game, cur_enemy_index) as i32, game))
        .ptfwss_var_00 as i32;
    off_A39C9B[v3 as usize].expect("non-null function pointer")(game);
    if (*E).ptfwss_var_00 != (*E).ptfwss_var_06 {
        (*E).ptfwss_var_F = 0u16;
    }
    (*E).ptfwss_var_06 = (*E).ptfwss_var_00;
}
pub unsafe fn PlatformThatFallsWithSamus_Func_1(game: &mut Game) {
    let E: *mut Enemy_PlatformThatFallsWithSamus =
        Get_PlatformThatFallsWithSamus(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).ptfwss_var_01 = (*E).base.x_pos;
    if Enemy_MoveRight_IgnoreSlopes(
        ram_word!(game, cur_enemy_index),
        (((*E).ptfwss_var_E as u32) << 16i32 | (*E).ptfwss_var_D as u32) as i32,
        game,
    ) != 0
    {
        (*E).ptfwss_var_02 = 1u16;
        PlatformThatFallsWithSamus_Func_8(game);
    }
}
pub unsafe fn PlatformThatFallsWithSamus_Func_2(game: &mut Game) {
    let E: *mut Enemy_PlatformThatFallsWithSamus =
        Get_PlatformThatFallsWithSamus(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).ptfwss_var_01 = (*E).base.x_pos;
    if Enemy_MoveRight_IgnoreSlopes(
        ram_word!(game, cur_enemy_index),
        (((*E).ptfwss_var_C as u32) << 16i32 | (*E).ptfwss_var_B as u32) as i32,
        game,
    ) != 0
    {
        (*E).ptfwss_var_02 = 0u16;
        PlatformThatFallsWithSamus_Func_7(game);
    }
}
pub unsafe fn PlatformThatFallsWithSamus_Func_3(game: &mut Game) {
    let mut ptfwss_var_05: u16 = 0;
    let mut v3: i32 = 0;
    let current_block: u64;
    let E: *mut Enemy_PlatformThatFallsWithSamus =
        Get_PlatformThatFallsWithSamus(ram_word!(game, cur_enemy_index) as i32, game);
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
        v3 = 8i32 * ptfwss_var_05 as i32 >> 1i32;
        if Enemy_MoveDown(
            ram_word!(game, cur_enemy_index),
            ((crate::rtl::rom_table_word!(game, 0xa2838f, (v3 + 3i32) as isize) as u32) << 16i32
                | crate::rtl::rom_table_word!(game, 0xa2838f, (v3 + 2i32) as isize) as u32)
                as i32,
            game,
        ) != 0
        {
            current_block = 2432778910466551290;
        } else {
            current_block = 3640593987805443782;
        }
    }
    if current_block == 2432778910466551290 {
        (*E).ptfwss_var_F = 0u16;
        PlatformThatFallsWithSamus_Func_10(game);
    };
}
pub unsafe fn PlatformThatFallsWithSamus_Func_4(game: &mut Game) {
    let E: *mut Enemy_PlatformThatFallsWithSamus =
        Get_PlatformThatFallsWithSamus(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).ptfwss_var_F = (*E).ptfwss_var_F.wrapping_add(1);
    if (((*E).ptfwss_var_F) as i16).wrapping_sub(((*E).ptfwss_var_05) as i16) >= 0i16 {
        (*E).ptfwss_var_F = (*E).ptfwss_var_05;
    }
    update_ram_word!(game.extra_samus_x_displacement, |fresh13| (fresh13 as i32
        + ((*E).base.x_pos as i32 - (*E).ptfwss_var_01 as i32))
        as u16);
    (*E).ptfwss_var_01 = (*E).base.y_pos;
    let v2: i32 = 8i32 * (*E).ptfwss_var_F as i32 >> 1i32;
    if Enemy_MoveDown(
        ram_word!(game, cur_enemy_index),
        ((crate::rtl::rom_table_word!(game, 0xa2838f, (v2 + 1i32) as isize) as u32) << 16i32
            | crate::rtl::rom_table_word!(game, 0xa2838f, v2 as isize) as u32) as i32,
        game,
    ) != 0
    {
        (*E).ptfwss_var_F = 0u16;
        PlatformThatFallsWithSamus_Func_10(game);
    }
    update_ram_word!(game.extra_samus_y_displacement, |fresh14| (fresh14 as i32
        + ((*E).base.y_pos as i32 - (*E).ptfwss_var_01 as i32))
        as u16);
}
pub unsafe fn PlatformThatFallsWithSamus_Func_5(game: &mut Game) {
    let E: *mut Enemy_PlatformThatFallsWithSamus =
        Get_PlatformThatFallsWithSamus(ram_word!(game, cur_enemy_index) as i32, game);
    let mut v0: u16 = addr_kPlatformThatFallsWithSamus_Ilist_9C13 as u16;
    if ((*E).ptfwss_var_1F as i32 & 0x8000i32) != 0i32 {
        v0 = addr_kPlatformThatFallsWithSamus_Ilist_9BBB as u16;
    }
    (*E).base.current_instruction = v0;
    (*E).base.instruction_timer = 1u16;
    (*E).base.timer = 0u16;
}
pub unsafe fn PlatformThatFallsWithSamus_Func_6(game: &mut Game) {
    let E: *mut Enemy_PlatformThatFallsWithSamus =
        Get_PlatformThatFallsWithSamus(ram_word!(game, cur_enemy_index) as i32, game);
    let mut v0: u16 = addr_kPlatformThatFallsWithSamus_Ilist_9C29 as u16;
    if ((*E).ptfwss_var_1F as i32 & 0x8000i32) != 0i32 {
        v0 = addr_kPlatformThatFallsWithSamus_Ilist_9BD1 as u16;
    }
    (*E).base.current_instruction = v0;
    (*E).base.instruction_timer = 1u16;
    (*E).base.timer = 0u16;
}
pub unsafe fn PlatformThatFallsWithSamus_Func_7(game: &mut Game) {
    let E: *mut Enemy_PlatformThatFallsWithSamus =
        Get_PlatformThatFallsWithSamus(ram_word!(game, cur_enemy_index) as i32, game);
    let mut v0: u16 = addr_kPlatformThatFallsWithSamus_Ilist_9C3F as u16;
    if ((*E).ptfwss_var_1F as i32 & 0x8000i32) != 0i32 {
        v0 = addr_kPlatformThatFallsWithSamus_Ilist_9BE7 as u16;
    }
    (*E).base.current_instruction = v0;
    (*E).base.instruction_timer = 1u16;
    (*E).base.timer = 0u16;
}
pub unsafe fn PlatformThatFallsWithSamus_Func_8(game: &mut Game) {
    let E: *mut Enemy_PlatformThatFallsWithSamus =
        Get_PlatformThatFallsWithSamus(ram_word!(game, cur_enemy_index) as i32, game);
    let mut v0: u16 = addr_kPlatformThatFallsWithSamus_Ilist_9C55 as u16;
    if ((*E).ptfwss_var_1F as i32 & 0x8000i32) != 0i32 {
        v0 = addr_kPlatformThatFallsWithSamus_Ilist_9BFD as u16;
    }
    (*E).base.current_instruction = v0;
    (*E).base.instruction_timer = 1u16;
    (*E).base.timer = 0u16;
}
pub unsafe fn PlatformThatFallsWithSamus_Func_9(game: &mut Game) {
    let E: *mut Enemy_PlatformThatFallsWithSamus =
        Get_PlatformThatFallsWithSamus(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).ptfwss_var_03 == 0 {
        (*E).ptfwss_var_03 = 1u16;
        if (*E).ptfwss_var_02 != 0 {
            PlatformThatFallsWithSamus_Func_6(game);
        } else {
            PlatformThatFallsWithSamus_Func_5(game);
        }
    }
    (*E).ptfwss_var_04 = 0u16;
}
pub unsafe fn PlatformThatFallsWithSamus_Func_10(game: &mut Game) {
    let E: *mut Enemy_PlatformThatFallsWithSamus =
        Get_PlatformThatFallsWithSamus(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).ptfwss_var_04 == 0 {
        (*E).ptfwss_var_04 = 1u16;
        if (*E).ptfwss_var_02 != 0 {
            PlatformThatFallsWithSamus_Func_8(game);
        } else {
            PlatformThatFallsWithSamus_Func_7(game);
        }
    }
    (*E).ptfwss_var_03 = 0u16;
}
pub unsafe fn FastMovingSlowSinkingPlatform_Shot(game: &mut Game) {
    Enemy_NormalShotAI_A3(game);
    let E: *mut Enemy_PlatformThatFallsWithSamus =
        Get_PlatformThatFallsWithSamus(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).base.frozen_timer != 0 {
        if (*E).ptfwss_var_02 != 0 {
            (*E).base.spritemap_pointer = addr_kPlatformThatFallsWithSamus_Sprmap_A015 as u16;
        } else {
            (*E).base.spritemap_pointer = addr_kPlatformThatFallsWithSamus_Sprmap_A009 as u16;
        }
    }
}
pub unsafe fn Roach_Func_1(game: &mut Game) {
    let E: *mut Enemy_Roach = Get_Roach(ram_word!(game, cur_enemy_index) as i32, game);
    let roach_var_24: u16 = (*E).roach_var_24;
    if roach_var_24 != (*E).roach_var_25 {
        (*E).roach_var_25 = roach_var_24;
        (*E).base.current_instruction = roach_var_24;
        (*E).base.instruction_timer = 1u16;
        (*E).base.timer = 0u16;
    }
}
pub unsafe fn Roach_Init(game: &mut Game) {
    let E: *mut Enemy_Roach = Get_Roach(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).roach_var_25 = 0u16;
    (*E).roach_var_24 = 0u16;
    Roach_Func_2(ram_word!(game, cur_enemy_index), game);
    Roach_Func_4(ram_word!(game, cur_enemy_index), game);
    Roach_Func_5(ram_word!(game, cur_enemy_index), game);
    Roach_Func_6(ram_word!(game, cur_enemy_index), game);
    Roach_Func_7(ram_word!(game, cur_enemy_index), game);
    Roach_Func_8(ram_word!(game, cur_enemy_index), game);
    (*E).roach_var_24 =
        crate::rtl::rom_table_word!(game, 0xa3a111, ((*E).roach_var_20 as i32 >> 1i32) as isize);
    Roach_Func_1(game);
    (*E).roach_var_B = ((0xa3a2d7i32) as u16);
}
pub unsafe fn Roach_Func_2(k: u16, game: &mut Game) {
    let E: *mut Enemy_Roach = Get_Roach(k as i32, game);
    let pt: Point32 = ConvertAngleToXy(
        *(&raw mut (*E).roach_parameter_1 as *mut u8).offset(1isize) as u16,
        *(&raw mut (*E).roach_parameter_1 as *mut u8).offset(0) as u16,
        game,
    );
    SetHiLo(&raw mut (*E).roach_var_01, &raw mut (*E).roach_var_00, pt.x);
    SetHiLo(&raw mut (*E).roach_var_03, &raw mut (*E).roach_var_02, pt.y);
}
pub unsafe fn Roach_Func_3(k: u16, game: &mut Game) {
    let E: *mut Enemy_Roach = Get_Roach(k as i32, game);
    SetHiLo(
        &raw mut (*E).roach_var_01,
        &raw mut (*E).roach_var_00,
        CosineMult8bitFull(
            *(&raw mut (*E).roach_parameter_1 as *mut u8).offset(1isize) as u16,
            *(&raw mut (*E).roach_parameter_1 as *mut u8).offset(0) as u16,
            game,
        ),
    );
    SetHiLo(
        &raw mut (*E).roach_var_03,
        &raw mut (*E).roach_var_02,
        SineMult8bitFull(
            *(&raw mut (*E).roach_parameter_1 as *mut u8).offset(1isize) as u16,
            *(&raw mut (*E).roach_parameter_1 as *mut u8).offset(0) as u16,
            game,
        ),
    );
}
pub unsafe fn Roach_Func_4(k: u16, game: &mut Game) {
    let E: *mut Enemy_Roach = Get_Roach(k as i32, game);
    SetHiLo(
        &raw mut (*E).roach_var_05,
        &raw mut (*E).roach_var_04,
        CosineMult8bitFull(
            ((*(&raw mut (*E).roach_parameter_1 as *mut u8).offset(1isize)).wrapping_sub(32u8))
                as u16,
            *(&raw mut (*E).roach_parameter_1 as *mut u8).offset(0) as u16,
            game,
        ),
    );
    SetHiLo(
        &raw mut (*E).roach_var_07,
        &raw mut (*E).roach_var_06,
        SineMult8bitFull(
            ((*(&raw mut (*E).roach_parameter_1 as *mut u8).offset(1isize)).wrapping_sub(32u8))
                as u16,
            *(&raw mut (*E).roach_parameter_1 as *mut u8).offset(0) as u16,
            game,
        ),
    );
}
pub unsafe fn Roach_Func_5(k: u16, game: &mut Game) {
    let E: *mut Enemy_Roach = Get_Roach(k as i32, game);
    SetHiLo(
        &raw mut (*E).roach_var_09,
        &raw mut (*E).roach_var_08,
        CosineMult8bitFull(
            ((*(&raw mut (*E).roach_parameter_1 as *mut u8).offset(1isize)).wrapping_add(32u8))
                as u16,
            *(&raw mut (*E).roach_parameter_1 as *mut u8).offset(0) as u16,
            game,
        ),
    );
    SetHiLo(
        &raw mut (*E).roach_var_0B,
        &raw mut (*E).roach_var_0A,
        SineMult8bitFull(
            ((*(&raw mut (*E).roach_parameter_1 as *mut u8).offset(1isize)).wrapping_add(32u8))
                as u16,
            *(&raw mut (*E).roach_parameter_1 as *mut u8).offset(0) as u16,
            game,
        ),
    );
}
pub unsafe fn Roach_Func_6(k: u16, game: &mut Game) {
    let E: *mut Enemy_Roach = Get_Roach(k as i32, game);
    (*E).roach_var_20 = (2i32
        * (((*(&raw mut (*E).roach_parameter_1 as *mut u8).offset(1isize)).wrapping_sub(48u8))
            as i32
            >> 5i32)) as u16;
}
pub unsafe fn Roach_Func_7(k: u16, game: &mut Game) {
    let E: *mut Enemy_Roach = Get_Roach(k as i32, game);
    (*E).roach_var_21 = (2i32
        * (((*(&raw mut (*E).roach_parameter_1 as *mut u8).offset(1isize)).wrapping_sub(80u8))
            as i32
            >> 5i32)) as u16;
}
pub unsafe fn Roach_Func_8(k: u16, game: &mut Game) {
    let E: *mut Enemy_Roach = Get_Roach(k as i32, game);
    (*E).roach_var_22 = (2i32
        * ((*(&raw mut (*E).roach_parameter_1 as *mut u8).offset(1isize) as i32 - 48i32 + 32i32)
            as u8 as i32
            >> 5i32)) as u16;
}
pub unsafe fn CallRoachFunc(ea: u32, game: &mut Game) {
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
    let E: *mut Enemy_Roach = Get_Roach(ram_word!(game, cur_enemy_index) as i32, game);
    CallRoachFunc((((*E).roach_var_B) as u32) | (0xa30000u32), game);
}
pub unsafe fn Roach_Func_9(game: &mut Game) {
    let E: *mut Enemy_Roach = Get_Roach(ram_word!(game, cur_enemy_index) as i32, game);
    if IsSamusWithinEnemy_X(
        ram_word!(game, cur_enemy_index),
        *(&raw mut (*E).roach_parameter_2 as *mut u8).offset(0) as u16,
        game,
    ) != 0
        && IsSamusWithinEnemy_Y(
            ram_word!(game, cur_enemy_index),
            *(&raw mut (*E).roach_parameter_2 as *mut u8).offset(0) as u16,
            game,
        ) != 0
    {
        (*E).roach_var_B = crate::rtl::rom_table_word!(
            game,
            0xa3a121,
            *(&raw mut (*E).roach_parameter_2 as *mut u8).offset(1isize) as isize
        );
    }
}
pub unsafe fn Roach_Func_10(game: &mut Game) {
    (*Get_Roach(ram_word!(game, cur_enemy_index) as i32, game)).roach_var_B =
        ((0xa3a407i32) as u16);
}
pub unsafe fn Roach_Func_11(game: &mut Game) {
    (*Get_Roach(ram_word!(game, cur_enemy_index) as i32, game)).roach_var_B =
        ((0xa3a40ei32) as u16);
}
pub unsafe fn Roach_Func_12(game: &mut Game) {
    set_ram_word!(game.random_number = 11u16);
    (*Get_Roach(ram_word!(game, cur_enemy_index) as i32, game)).roach_var_B =
        ((0xa3a4f0i32) as u16);
}
pub unsafe fn Roach_Func_13(game: &mut Game) {
    let E: *mut Enemy_Roach = Get_Roach(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).roach_var_A = 512u16;
    set_ram_word!(game.random_number = 11u16);
    (*E).roach_var_B = ((0xa3a4b6i32) as u16);
}
pub unsafe fn Roach_Func_14(game: &mut Game) {
    let E: *mut Enemy_Roach = Get_Roach(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).roach_var_F = 32u16;
    (*E).roach_var_B = ((0xa3a44ei32) as u16);
}
pub unsafe fn Roach_Func_15(game: &mut Game) {
    let v1: u8 = (64u8).wrapping_sub(
        (CalculateAngleOfSamusFromEnemy(ram_word!(game, cur_enemy_index), game)) as u8,
    );
    let E: *mut Enemy_Roach = Get_Roach(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).roach_var_C = v1 as u16;
    Roach_Func_18(ram_word!(game, cur_enemy_index), game);
    Roach_Func_17(ram_word!(game, cur_enemy_index), game);
    (*E).roach_var_24 =
        crate::rtl::rom_table_word!(game, 0xa3a111, ((*E).roach_var_23 as i32 >> 1i32) as isize);
    Roach_Func_1(game);
    (*E).roach_var_B = ((0xa3a440i32) as u16);
}
pub unsafe fn Roach_Func_16(game: &mut Game) {
    let v1: u16 = (64i32
        - CalculateAngleOfSamusFromEnemy(ram_word!(game, cur_enemy_index), game) as i32
        + 0x80i32) as u8 as u16;
    let E: *mut Enemy_Roach = Get_Roach(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).roach_var_C = v1;
    Roach_Func_18(ram_word!(game, cur_enemy_index), game);
    Roach_Func_17(ram_word!(game, cur_enemy_index), game);
    (*E).roach_var_24 =
        crate::rtl::rom_table_word!(game, 0xa3a111, ((*E).roach_var_23 as i32 >> 1i32) as isize);
    Roach_Func_1(game);
    (*E).roach_var_B = ((0xa3a447i32) as u16);
}
pub unsafe fn Roach_Func_17(k: u16, game: &mut Game) {
    let E: *mut Enemy_Roach = Get_Roach(k as i32, game);
    (*E).roach_var_23 =
        (2i32 * (((((*E).roach_var_C) as u8).wrapping_sub(48u8)) as i32 >> 5i32)) as u16;
}
pub unsafe fn Roach_Func_18(k: u16, game: &mut Game) {
    let E: *mut Enemy_Roach = Get_Roach(k as i32, game);
    SetHiLo(
        &raw mut (*E).roach_var_0D,
        &raw mut (*E).roach_var_0C,
        CosineMult8bitFull(
            (*E).roach_var_C,
            *(&raw mut (*E).roach_parameter_1 as *mut u8).offset(0) as u16,
            game,
        ),
    );
    SetHiLo(
        &raw mut (*E).roach_var_0F,
        &raw mut (*E).roach_var_0E,
        SineMult8bitFull(
            (*E).roach_var_C,
            *(&raw mut (*E).roach_parameter_1 as *mut u8).offset(0) as u16,
            game,
        ),
    );
}
pub unsafe fn Roach_Func_19(game: &mut Game) {
    Roach_Func_29(ram_word!(game, cur_enemy_index), game);
}
pub unsafe fn Roach_Func_20(game: &mut Game) {
    let E: *mut Enemy_Roach = Get_Roach(ram_word!(game, cur_enemy_index) as i32, game);
    if ((*E).base.frame_counter as i32 & 0x10i32) != 0i32 {
        (*E).roach_var_24 = crate::rtl::rom_table_word!(
            game,
            0xa3a111,
            ((*E).roach_var_22 as i32 >> 1i32) as isize
        );
        Roach_Func_1(game);
        Roach_Func_31(ram_word!(game, cur_enemy_index), game);
    } else {
        (*E).roach_var_24 = crate::rtl::rom_table_word!(
            game,
            0xa3a111,
            ((*E).roach_var_21 as i32 >> 1i32) as isize
        );
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
    let E: *mut Enemy_Roach = Get_Roach(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).roach_var_F = (*E).roach_var_F.wrapping_sub(1);
    if ((*E).roach_var_F as i32 & 0x8000i32) != 0i32 {
        (*E).roach_var_B = ((0xa3a2d7i32) as u16);
    } else {
        Roach_Func_29(ram_word!(game, cur_enemy_index), game);
    };
}
pub unsafe fn Roach_Func_24(game: &mut Game) {
    let E: *mut Enemy_Roach = Get_Roach(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).roach_var_F = (*E).roach_var_F.wrapping_sub(1);
    if ((*E).roach_var_F as i32 & 0x8000i32) != 0i32 {
        (*E).roach_var_B = ((0xa3a4f0i32) as u16);
    } else {
        Roach_Func_33(ram_word!(game, cur_enemy_index), game);
    };
}
pub unsafe fn Roach_Func_25(game: &mut Game) {
    let E: *mut Enemy_Roach = Get_Roach(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).roach_var_A = (*E).roach_var_A.wrapping_sub(1);
    let mut v2: bool = ((*E).roach_var_A as i32 & 0x8000i32) != 0i32;
    if !v2 {
        (*E).roach_var_F = (*E).roach_var_F.wrapping_sub(1);
        v2 = ((*E).roach_var_F as i32 & 0x8000i32) != 0i32;
        if !v2 {
            let v3: u16 = Abs16(((*E).base.x_pos).wrapping_sub(ram_word!(game, samus_x_pos)));
            if v3 as i32 - 96i32 & 0x8000i32 == 0 {
                let v4: u16 = Abs16(((*E).base.y_pos).wrapping_sub(ram_word!(game, samus_y_pos)));
                if v4 as i32 - 96i32 & 0x8000i32 == 0 {
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
    let v1: u8 = ((NextRandom(game)) as u8).wrapping_sub(64u8);
    let E: *mut Enemy_Roach = Get_Roach(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).roach_var_C = ((*E).roach_var_C).wrapping_add((v1) as u16);
    Roach_Func_18(ram_word!(game, cur_enemy_index), game);
    Roach_Func_17(ram_word!(game, cur_enemy_index), game);
    (*E).roach_var_24 =
        crate::rtl::rom_table_word!(game, 0xa3a111, ((*E).roach_var_23 as i32 >> 1i32) as isize);
    Roach_Func_1(game);
    (*E).roach_var_F = 32u16;
    (*E).roach_var_B = ((0xa3a476i32) as u16);
}
pub unsafe fn Roach_Func_27(game: &mut Game) {
    let v1: u8 = ((NextRandom(game)) as u8).wrapping_sub(64u8);
    let E: *mut Enemy_Roach = Get_Roach(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).roach_var_C = ((*E).roach_var_C).wrapping_add((v1) as u16);
    Roach_Func_18(ram_word!(game, cur_enemy_index), game);
    Roach_Func_17(ram_word!(game, cur_enemy_index), game);
    (*E).roach_var_24 =
        crate::rtl::rom_table_word!(game, 0xa3a111, ((*E).roach_var_23 as i32 >> 1i32) as isize);
    Roach_Func_1(game);
    (*E).roach_var_F = 32u16;
    (*E).roach_var_B = ((0xa3a462i32) as u16);
}
pub unsafe fn Roach_Func_28(game: &mut Game) {
    let E: *mut Enemy_Roach = Get_Roach(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).roach_var_0D = -((*E).roach_var_0D as i32) as u16;
    (*E).roach_var_0C = -((*E).roach_var_0C as i32) as u16;
    (*E).roach_var_0F = -((*E).roach_var_0F as i32) as u16;
    (*E).roach_var_0E = -((*E).roach_var_0E as i32) as u16;
    let v1: u16 = ((*E).roach_var_23 as i32 + 4i32 & 7i32) as u16;
    (*E).roach_var_23 = v1;
    (*E).roach_var_24 = crate::rtl::rom_table_word!(game, 0xa3a111, (v1 as i32 >> 1i32) as isize);
    Roach_Func_1(game);
}
pub unsafe fn Roach_Func_29(k: u16, game: &mut Game) {
    let E: *mut Enemy_Roach = Get_Roach(k as i32, game);
    let varE20: u16 = *(&raw mut (*E).roach_parameter_1 as *mut u8).offset(1isize) as u16;
    EnemyFunc_B691(
        varE20,
        Point32 {
            x: ((*E).roach_var_01 as u32) << 16i32 | (*E).roach_var_00 as u32,
            y: ((*E).roach_var_03 as u32) << 16i32 | (*E).roach_var_02 as u32,
        },
        game,
    );
}
pub unsafe fn Roach_Func_30(k: u16, game: &mut Game) {
    let E: *mut Enemy_Roach = Get_Roach(k as i32, game);
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        ((*E).roach_var_05 as u32) << 16i32 | (*E).roach_var_04 as u32,
    );
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        ((*E).roach_var_07 as u32) << 16i32 | (*E).roach_var_06 as u32,
    );
}
pub unsafe fn Roach_Func_31(k: u16, game: &mut Game) {
    let E: *mut Enemy_Roach = Get_Roach(k as i32, game);
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        ((*E).roach_var_09 as u32) << 16i32 | (*E).roach_var_08 as u32,
    );
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        ((*E).roach_var_0B as u32) << 16i32 | (*E).roach_var_0A as u32,
    );
}
pub unsafe fn Roach_Func_32(k: u16, game: &mut Game) {
    let E: *mut Enemy_Roach = Get_Roach(k as i32, game);
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        ((*E).roach_var_0D as u32) << 16i32 | (*E).roach_var_0C as u32,
    );
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        ((*E).roach_var_0F as u32) << 16i32 | (*E).roach_var_0E as u32,
    );
}
pub unsafe fn Roach_Func_33(k: u16, game: &mut Game) {
    let E: *mut Enemy_Roach = Get_Roach(k as i32, game);
    if Enemy_MoveRight_IgnoreSlopes(
        k,
        (((*E).roach_var_0D as u32) << 16i32 | (*E).roach_var_0C as u32) as i32,
        game,
    ) != 0
    {
        (*E).roach_var_B = ((0xa3a2d7i32) as u16);
    } else if Enemy_MoveDown(
        k,
        (((*E).roach_var_0F as u32) << 16i32 | (*E).roach_var_0E as u32) as i32,
        game,
    ) != 0
    {
        (*E).roach_var_B = ((0xa3a2d7i32) as u16);
    }
}
pub unsafe fn Mochtroid_Init(game: &mut Game) {
    let E: *mut Enemy_Mochtroid = Get_Mochtroid(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.layer = 2u16;
    Mochtroid_Func_4(
        ram_word!(game, cur_enemy_index),
        addr_kMochtroid_Ilist_A745 as u16,
        game,
    );
    (*E).mochtr_var_F = 0u16;
}
static off_A3A7A4: [Option<Func_V>; 3] = unsafe {
    [
        Some(Mochtroid_Func_1 as unsafe fn(&mut Game) -> ()),
        Some(Mochtroid_Func_3 as unsafe fn(&mut Game) -> ()),
        Some(Mochtroid_Func_2 as unsafe fn(&mut Game) -> ()),
    ]
};
pub unsafe fn Mochtroid_Main(game: &mut Game) {
    let v0: u16 = (2u16)
        .wrapping_mul((*Get_Mochtroid(ram_word!(game, cur_enemy_index) as i32, game)).mochtr_var_F);
    (*Get_Mochtroid(ram_word!(game, cur_enemy_index) as i32, game)).mochtr_var_F = 0u16;
    off_A3A7A4[(v0 as i32 >> 1i32) as usize].expect("non-null function pointer")(game);
}
pub unsafe fn Mochtroid_Func_1(game: &mut Game) {
    let mut v5: i16 = 0;
    let mut v10: i16 = 0;
    let E: *mut Enemy_Mochtroid = Get_Mochtroid(ram_word!(game, cur_enemy_index) as i32, game);
    let mut t: i32 = ((((((*E).base.y_pos) as i16)
        .wrapping_sub((ram_word!(game, samus_y_pos)) as i16)) as i32
        >> 2i32) as i16 as i32)
        << 8i32;
    AddToHiLo(
        &raw mut (*E).mochtr_var_D,
        &raw mut (*E).mochtr_var_C,
        -t as u32,
    );
    let current_block_4: u64;
    if ((*E).mochtr_var_D as i16) < 0i16 {
        if (*E).mochtr_var_D < 0xfffdu16 {
            v5 = -3i16;
            current_block_4 = 13796420526896809451;
        } else {
            current_block_4 = 8515828400728868193;
        }
    } else if (*E).mochtr_var_D >= 3u16 {
        v5 = 3i16;
        current_block_4 = 13796420526896809451;
    } else {
        current_block_4 = 8515828400728868193;
    }
    if current_block_4 == 13796420526896809451 {
        (*E).mochtr_var_D = v5 as u16;
        (*E).mochtr_var_C = 0u16;
    }
    if Enemy_MoveDown(
        ram_word!(game, cur_enemy_index),
        (((*E).mochtr_var_D as u32) << 16i32 | (*E).mochtr_var_C as u32) as i32,
        game,
    ) != 0
    {
        (*E).mochtr_var_C = 0u16;
        (*E).mochtr_var_D = 0u16;
    }
    t = ((((((*E).base.x_pos) as i16).wrapping_sub((ram_word!(game, samus_x_pos)) as i16)) as i32
        >> 2i32) as i16 as i32)
        << 8i32;
    AddToHiLo(
        &raw mut (*E).mochtr_var_B,
        &raw mut (*E).mochtr_var_A,
        -t as u32,
    );
    let current_block_15: u64;
    if ((*E).mochtr_var_B as i16) < 0i16 {
        if (*E).mochtr_var_B < 0xfffdu16 {
            v10 = -3i16;
            current_block_15 = 10272573711837802396;
        } else {
            current_block_15 = 4808432441040389987;
        }
    } else if (*E).mochtr_var_B >= 3u16 {
        v10 = 3i16;
        current_block_15 = 10272573711837802396;
    } else {
        current_block_15 = 4808432441040389987;
    }
    if current_block_15 == 10272573711837802396 {
        (*E).mochtr_var_B = v10 as u16;
        (*E).mochtr_var_A = 0u16;
    }
    if Enemy_MoveRight_IgnoreSlopes(
        ram_word!(game, cur_enemy_index),
        (((*E).mochtr_var_B as u32) << 16i32 | (*E).mochtr_var_A as u32) as i32,
        game,
    ) != 0
    {
        (*E).mochtr_var_A = 0u16;
        (*E).mochtr_var_B = 0u16;
    }
    Mochtroid_Func_4(
        ram_word!(game, cur_enemy_index),
        addr_kMochtroid_Ilist_A745 as u16,
        game,
    );
}
pub unsafe fn Mochtroid_Func_2(game: &mut Game) {
    let E: *mut Enemy_Mochtroid = Get_Mochtroid(ram_word!(game, cur_enemy_index) as i32, game);
    let v2: i32 = ((*E).mochtr_var_E as i32 & 6i32) >> 1i32;
    (*E).base.x_pos = ((*E).base.x_pos).wrapping_add((g_word_A3A76D[v2 as usize]) as u16);
    (*E).base.y_pos = ((*E).base.y_pos).wrapping_add((g_word_A3A775[v2 as usize]) as u16);
    (*E).mochtr_var_A = 0u16;
    (*E).mochtr_var_B = 0u16;
    (*E).mochtr_var_C = 0u16;
    (*E).mochtr_var_D = 0u16;
    let fresh12 = (*E).mochtr_var_E;
    (*E).mochtr_var_E = (*E).mochtr_var_E.wrapping_sub(1);
    if fresh12 == 1u16 {
        (*E).mochtr_var_F = 0u16;
    }
    Mochtroid_Func_4(
        ram_word!(game, cur_enemy_index),
        addr_kMochtroid_Ilist_A745 as u16,
        game,
    );
}
pub unsafe fn Mochtroid_Func_3(game: &mut Game) {
    let E: *mut Enemy_Mochtroid = Get_Mochtroid(ram_word!(game, cur_enemy_index) as i32, game);
    let x_pos: u16 = (*E).base.x_pos;
    if x_pos == ram_word!(game, samus_x_pos) {
        (*E).mochtr_var_A = 0u16;
        (*E).mochtr_var_B = 0u16;
    } else if ((x_pos) as i16).wrapping_sub((ram_word!(game, samus_x_pos)) as i16) >= 0i16 {
        (*E).mochtr_var_A = 0u16;
        (*E).mochtr_var_B = -1i32 as u16;
    } else {
        (*E).mochtr_var_A = 0u16;
        (*E).mochtr_var_B = 1u16;
    }
    Enemy_MoveRight_IgnoreSlopes(
        ram_word!(game, cur_enemy_index),
        (((*E).mochtr_var_B as u32) << 16i32 | (*E).mochtr_var_A as u32) as i32,
        game,
    );
    let y_pos: u16 = (*E).base.y_pos;
    if y_pos == ram_word!(game, samus_y_pos) {
        (*E).mochtr_var_C = 0u16;
        (*E).mochtr_var_D = 0u16;
    } else if ((y_pos) as i16).wrapping_sub((ram_word!(game, samus_y_pos)) as i16) >= 0i16 {
        (*E).mochtr_var_C = 0u16;
        (*E).mochtr_var_D = -1i32 as u16;
    } else {
        (*E).mochtr_var_C = 0u16;
        (*E).mochtr_var_D = 1u16;
    }
    Enemy_MoveDown(
        ram_word!(game, cur_enemy_index),
        (((*E).mochtr_var_D as u32) << 16i32 | (*E).mochtr_var_C as u32) as i32,
        game,
    );
}
pub unsafe fn Mochtroid_Func_4(k: u16, a: u16, game: &mut Game) {
    let E: *mut Enemy_Mochtroid = Get_Mochtroid(k as i32, game);
    if a != (*E).mochtr_var_01 {
        (*E).mochtr_var_01 = a;
        (*E).base.current_instruction = a;
        (*E).base.instruction_timer = 1u16;
        (*E).base.timer = 0u16;
    }
}
pub unsafe fn Mochtroid_Touch(game: &mut Game) {
    let current_block: u64;
    let E: *mut Enemy_Mochtroid = Get_Mochtroid(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).mochtr_var_F = 1u16;
    Mochtroid_Func_4(
        ram_word!(game, cur_enemy_index),
        addr_kMochtroid_Ilist_A759 as u16,
        game,
    );
    (*E).mochtr_var_20 = (*E).mochtr_var_20.wrapping_add(1);
    if ram_word!(game, samus_contact_damage_index) != 0 {
        current_block = 17285009320546006530;
    } else {
        if (ram_word!(game, random_enemy_counter) as i32 & 7i32) == 7i32
            && ram_word!(game, samus_health) as i32 - 30i32 & 0x8000i32 == 0
        {
            QueueSfx3_Max6(0x2du16, game);
        }
        if (*E).mochtr_var_20 as i32 - 80i32 & 0x8000i32 == 0 {
            (*E).mochtr_var_20 = 0u16;
            current_block = 17285009320546006530;
        } else {
            current_block = 13513818773234778473;
        }
    }
    if current_block == 17285009320546006530 {
        Enemy_NormalTouchAI_A3(game);
        set_ram_word!(game.samus_invincibility_timer = 0u16);
        set_ram_word!(game.samus_knockback_timer = 0u16);
    };
}
pub unsafe fn Mochtroid_Shot(game: &mut Game) {
    Enemy_NormalShotAI_A3(game);
}
pub unsafe fn Sidehopper_Func_1(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    QueueSfx2_Max3(*jp, game);
    jp.offset(1)
}
pub unsafe fn Sidehopper_Instr_1(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*Get_Sidehopper(ram_word!(game, cur_enemy_index) as i32, game)).sideh_var_04 = 1u16;
    jp
}
pub unsafe fn Sidehopper_Init(game: &mut Game) {
    set_ram_word!(game.random_number = 37u16);
    NextRandom(game);
    let E: *mut Enemy_Sidehopper = Get_Sidehopper(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).sideh_var_03 = 0u16;
    (*E).sideh_var_04 = 0u16;
    (*E).sideh_var_05 = 0u16;
    let v2: u16 = (2u16).wrapping_mul((*get_EnemyDef_A2((*E).base.enemy_ptr, game)).field_2A);
    (*E).sideh_var_06 = v2;
    let mut v4: u16 = 0;
    if (*E).sideh_parameter_1 != 0 {
        v4 = crate::rtl::rom_table_word!(
            game,
            0xa3aaca,
            ((*E).sideh_var_06 as i32 >> 1i32) as isize
        );
    } else {
        v4 = crate::rtl::rom_table_word!(
            game,
            0xa3aac2,
            ((*E).sideh_var_06 as i32 >> 1i32) as isize
        );
    }
    (*E).sideh_var_00 = v4;
    Sidehopper_Func_3(game);
    if (*get_EnemyDef_A2((*E).base.enemy_ptr, game)).field_2A != 0 {
        (*E).sideh_var_05 = 2u16;
    }
    let v6: i32 = (*E).sideh_var_05 as i32 >> 1i32;
    (*E).sideh_var_01 = Sidehopper_Func_2(
        crate::rtl::rom_table_word!(game, 0xa3aaee, v6 as isize),
        crate::rtl::rom_table_word!(game, 0xa3aae6, v6 as isize),
        game,
    );
    let v8: i32 = (*E).sideh_var_05 as i32 >> 1i32;
    (*E).sideh_var_02 = Sidehopper_Func_2(
        crate::rtl::rom_table_word!(game, 0xa3aafa, v8 as isize),
        crate::rtl::rom_table_word!(game, 0xa3aaf2, v8 as isize),
        game,
    );
    (*E).sideh_var_B = ((0xa3abd6i32) as u16);
}
pub unsafe fn Sidehopper_Func_2(r22: u16, r24: u16, game: &mut Game) -> u16 {
    let mut r18: u16 = 0u16;
    let mut r20: u16 = 0u16;
    loop {
        r18 = (r18).wrapping_add(r24);
        r20 = (r20).wrapping_add(
            *((game.g_rom.offset(crate::rtl::lorom_offset(0xa2838f)) as *mut u16 as *mut u8)
                .offset((8i32 * r18 as i32) as isize)
                .offset(1) as *mut u16),
        );
        if (r20 as i32 - r22 as i32 & 0x8000i32 == 0) {
            break;
        }
    }
    r18
}
pub unsafe fn Sidehopper_Func_3(game: &mut Game) {
    let E: *mut Enemy_Sidehopper = Get_Sidehopper(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.current_instruction = (*E).sideh_var_00;
    (*E).base.instruction_timer = 1u16;
    (*E).base.timer = 0u16;
}
pub unsafe fn Sidehopper_Main(game: &mut Game) {
    let E: *mut Enemy_Sidehopper = Get_Sidehopper(ram_word!(game, cur_enemy_index) as i32, game);
    CallSidehopperFunc((((*E).sideh_var_B) as u32) | (0xa30000u32), game);
}
pub unsafe fn Sidehopper_Func_4(k: u16, game: &mut Game) {
    let v1: u16 =
        crate::rtl::rom_table_word!(game, 0xa3aae2, (NextRandom(game) as i32 & 1i32) as isize);
    (*Get_Sidehopper(k as i32, game)).sideh_var_B = v1;
}
pub unsafe fn Sidehopper_Func_5(game: &mut Game) {
    let E: *mut Enemy_Sidehopper = Get_Sidehopper(ram_word!(game, cur_enemy_index) as i32, game);
    let v1: i32 = (*E).sideh_var_05 as i32 >> 1i32;
    (*E).sideh_var_E = crate::rtl::rom_table_word!(game, 0xa3aae6, v1 as isize);
    (*E).sideh_var_D = crate::rtl::rom_table_word!(game, 0xa3aaea, v1 as isize);
    (*E).sideh_var_C = (*E).sideh_var_01;
    (*E).sideh_var_B = ((0xa3ac40i32) as u16);
    if (*E).sideh_parameter_1 != 0 {
        (*E).sideh_var_B = ((0xa3ac56i32) as u16);
    }
}
pub unsafe fn Sidehopper_Func_6(game: &mut Game) {
    let E: *mut Enemy_Sidehopper = Get_Sidehopper(ram_word!(game, cur_enemy_index) as i32, game);
    let v1: i32 = (*E).sideh_var_05 as i32 >> 1i32;
    (*E).sideh_var_E = crate::rtl::rom_table_word!(game, 0xa3aaf2, v1 as isize);
    (*E).sideh_var_D = crate::rtl::rom_table_word!(game, 0xa3aaf6, v1 as isize);
    (*E).sideh_var_C = (*E).sideh_var_02;
    (*E).sideh_var_B = ((0xa3ac40i32) as u16);
    if (*E).sideh_parameter_1 != 0 {
        (*E).sideh_var_B = ((0xa3ac56i32) as u16);
    }
}
pub unsafe fn Sidehopper_Func_7(game: &mut Game) {
    let E: *mut Enemy_Sidehopper = Get_Sidehopper(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).sideh_var_B = ((0xa3ac6ci32) as u16);
    if (GetSamusEnemyDelta_X(ram_word!(game, cur_enemy_index), game) as i32 & 0x8000i32) == 0i32 {
        (*E).sideh_var_B = ((0xa3ac8fi32) as u16);
    }
}
pub unsafe fn Sidehopper_Func_8(game: &mut Game) {
    let E: *mut Enemy_Sidehopper = Get_Sidehopper(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).sideh_var_B = ((0xa3aca8i32) as u16);
    if (GetSamusEnemyDelta_X(ram_word!(game, cur_enemy_index), game) as i32 & 0x8000i32) == 0i32 {
        (*E).sideh_var_B = ((0xa3accbi32) as u16);
    }
}
pub unsafe fn Sidehopper_Func_9(game: &mut Game) {
    let E: *mut Enemy_Sidehopper = Get_Sidehopper(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).sideh_var_D = -((*E).sideh_var_D as i32) as u16;
    (*E).sideh_var_00 =
        crate::rtl::rom_table_word!(game, 0xa3aad2, ((*E).sideh_var_06 as i32 >> 1i32) as isize);
    Sidehopper_Func_3(game);
    (*E).sideh_var_B = ((0xa3ad0ei32) as u16);
}
pub unsafe fn Sidehopper_Func_10(game: &mut Game) {
    let E: *mut Enemy_Sidehopper = Get_Sidehopper(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).sideh_var_00 =
        crate::rtl::rom_table_word!(game, 0xa3aad2, ((*E).sideh_var_06 as i32 >> 1i32) as isize);
    Sidehopper_Func_3(game);
    (*E).sideh_var_B = ((0xa3ad20i32) as u16);
}
pub unsafe fn Sidehopper_Func_11(game: &mut Game) {
    let E: *mut Enemy_Sidehopper = Get_Sidehopper(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).sideh_var_D = -((*E).sideh_var_D as i32) as u16;
    (*E).sideh_var_00 =
        crate::rtl::rom_table_word!(game, 0xa3aada, ((*E).sideh_var_06 as i32 >> 1i32) as isize);
    Sidehopper_Func_3(game);
    (*E).sideh_var_B = ((0xa3ad32i32) as u16);
}
pub unsafe fn Sidehopper_Func_12(game: &mut Game) {
    let E: *mut Enemy_Sidehopper = Get_Sidehopper(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).sideh_var_00 =
        crate::rtl::rom_table_word!(game, 0xa3aada, ((*E).sideh_var_06 as i32 >> 1i32) as isize);
    Sidehopper_Func_3(game);
    (*E).sideh_var_B = ((0xa3ad44i32) as u16);
}
pub unsafe fn Sidehopper_Func_13(game: &mut Game) {
    let E: *mut Enemy_Sidehopper = Get_Sidehopper(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).sideh_var_00 =
        crate::rtl::rom_table_word!(game, 0xa3aac2, ((*E).sideh_var_06 as i32 >> 1i32) as isize);
    if (*E).sideh_parameter_1 != 0 {
        (*E).sideh_var_00 = crate::rtl::rom_table_word!(
            game,
            0xa3aaca,
            ((*E).sideh_var_06 as i32 >> 1i32) as isize
        );
    }
    Sidehopper_Func_3(game);
    (*E).sideh_var_B = ((0xa3ad56i32) as u16);
}
pub unsafe fn Sidehopper_Func_14(game: &mut Game) {
    if (*Get_Sidehopper(ram_word!(game, cur_enemy_index) as i32, game)).sideh_var_03 != 0 {
        Sidehopper_Func_20(game);
    } else {
        Sidehopper_Func_19(game);
    };
}
pub unsafe fn Sidehopper_Func_15(game: &mut Game) {
    if (*Get_Sidehopper(ram_word!(game, cur_enemy_index) as i32, game)).sideh_var_03 != 0 {
        Sidehopper_Func_20(game);
    } else {
        Sidehopper_Func_19(game);
    };
}
pub unsafe fn Sidehopper_Func_16(game: &mut Game) {
    if (*Get_Sidehopper(ram_word!(game, cur_enemy_index) as i32, game)).sideh_var_03 != 0 {
        Sidehopper_Func_22(game);
    } else {
        Sidehopper_Func_21(game);
    };
}
pub unsafe fn Sidehopper_Func_17(game: &mut Game) {
    if (*Get_Sidehopper(ram_word!(game, cur_enemy_index) as i32, game)).sideh_var_03 != 0 {
        Sidehopper_Func_22(game);
    } else {
        Sidehopper_Func_21(game);
    };
}
pub unsafe fn Sidehopper_Func_18(game: &mut Game) {
    let E: *mut Enemy_Sidehopper = Get_Sidehopper(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).sideh_var_04 != 0 {
        (*E).sideh_var_04 = 0u16;
        (*E).sideh_var_B = ((0xa3abd6i32) as u16);
    }
}
unsafe fn CallSidehopperFunc(ea: u32, game: &mut Game) {
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
    let E: *mut Enemy_Sidehopper = Get_Sidehopper(ram_word!(game, cur_enemy_index) as i32, game);
    let v2: i32 = 8i32 * (*E).sideh_var_C as i32 >> 1i32;
    if Enemy_MoveDown(
        ram_word!(game, cur_enemy_index),
        ((crate::rtl::rom_table_word!(game, 0xa2838f, (v2 + 3i32) as isize) as u32) << 16i32
            | crate::rtl::rom_table_word!(game, 0xa2838f, (v2 + 2i32) as isize) as u32)
            as i32,
        game,
    ) != 0
    {
        (*E).sideh_var_D = -((*E).sideh_var_D as i32) as u16;
        (*E).sideh_var_03 = 1u16;
    } else if Enemy_MoveRight_IgnoreSlopes(
        ram_word!(game, cur_enemy_index),
        ((*E).sideh_var_D as i16 as i32) << 16i32,
        game,
    ) != 0
    {
        (*E).sideh_var_D = -((*E).sideh_var_D as i32) as u16;
        (*E).sideh_var_03 = 1u16;
    } else {
        let v3: i16 = (((*E).sideh_var_C) as i16).wrapping_sub(((*E).sideh_var_E) as i16);
        (*E).sideh_var_C = v3 as u16;
        if v3 < 0i16 {
            (*E).sideh_var_03 = 1u16;
            (*E).sideh_var_C = 0u16;
        }
    };
}
pub unsafe fn Sidehopper_Func_20(game: &mut Game) {
    let E: *mut Enemy_Sidehopper = Get_Sidehopper(ram_word!(game, cur_enemy_index) as i32, game);
    let v2: i32 = 8i32 * (*E).sideh_var_C as i32 >> 1i32;
    if Enemy_MoveDown(
        ram_word!(game, cur_enemy_index),
        ((crate::rtl::rom_table_word!(game, 0xa2838f, (v2 + 1i32) as isize) as u32) << 16i32
            | crate::rtl::rom_table_word!(game, 0xa2838f, v2 as isize) as u32) as i32,
        game,
    ) != 0
    {
        (*E).sideh_var_03 = 0u16;
        (*E).sideh_var_B = ((0xa3ace4i32) as u16);
    } else {
        if Enemy_MoveRight_IgnoreSlopes(
            ram_word!(game, cur_enemy_index),
            ((*E).sideh_var_D as i16 as i32) << 16i32,
            game,
        ) != 0
        {
            (*E).sideh_var_D = -((*E).sideh_var_D as i32) as u16;
        }
        let mut v3: i16 = (((*E).sideh_var_E) as i16).wrapping_add(((*E).sideh_var_C) as i16);
        if v3 as i32 - 64i32 & 0x8000i32 == 0 {
            v3 = 64i16;
        }
        (*E).sideh_var_C = v3 as u16;
    };
}
pub unsafe fn Sidehopper_Func_21(game: &mut Game) {
    let E: *mut Enemy_Sidehopper = Get_Sidehopper(ram_word!(game, cur_enemy_index) as i32, game);
    let v2: i32 = 8i32 * (*E).sideh_var_C as i32 >> 1i32;
    if Enemy_MoveDown(
        ram_word!(game, cur_enemy_index),
        ((crate::rtl::rom_table_word!(game, 0xa2838f, (v2 + 1i32) as isize) as u32) << 16i32
            | crate::rtl::rom_table_word!(game, 0xa2838f, v2 as isize) as u32) as i32,
        game,
    ) != 0
    {
        (*E).sideh_var_D = -((*E).sideh_var_D as i32) as u16;
        (*E).sideh_var_03 = 1u16;
    } else if Enemy_MoveRight_IgnoreSlopes(
        ram_word!(game, cur_enemy_index),
        ((*E).sideh_var_D as i16 as i32) << 16i32,
        game,
    ) != 0
    {
        (*E).sideh_var_D = -((*E).sideh_var_D as i32) as u16;
        (*E).sideh_var_03 = 1u16;
    } else {
        let v3: i16 = (((*E).sideh_var_C) as i16).wrapping_sub(((*E).sideh_var_E) as i16);
        (*E).sideh_var_C = v3 as u16;
        if v3 < 0i16 {
            (*E).sideh_var_03 = 1u16;
            (*E).sideh_var_C = 0u16;
        }
    };
}
pub unsafe fn Sidehopper_Func_22(game: &mut Game) {
    let mut v3: i16 = 0;
    let E: *mut Enemy_Sidehopper = Get_Sidehopper(ram_word!(game, cur_enemy_index) as i32, game);
    let v2: i32 = 8i32 * (*E).sideh_var_C as i32 >> 1i32;
    if Enemy_MoveDown(
        ram_word!(game, cur_enemy_index),
        ((crate::rtl::rom_table_word!(game, 0xa2838f, (v2 + 3i32) as isize) as u32) << 16i32
            | crate::rtl::rom_table_word!(game, 0xa2838f, (v2 + 2i32) as isize) as u32)
            as i32,
        game,
    ) != 0
    {
        (*E).sideh_var_03 = 0u16;
        (*E).sideh_var_B = ((0xa3ace4i32) as u16);
    } else {
        if Enemy_MoveRight_IgnoreSlopes(
            ram_word!(game, cur_enemy_index),
            ((*E).sideh_var_D as i16 as i32) << 16i32,
            game,
        ) != 0
        {
            (*E).sideh_var_D = -((*E).sideh_var_D as i32) as u16;
        }
        v3 = (((*E).sideh_var_E) as i16).wrapping_add(((*E).sideh_var_C) as i16);
        if v3 as i32 - 64i32 & 0x8000i32 == 0 {
            v3 = 64i16;
        }
        (*E).sideh_var_C = v3 as u16;
    };
}
pub unsafe fn MaridiaRefillCandy_Instr_1(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*Get_MaridiaRefillCandy(ram_word!(game, cur_enemy_index) as i32, game)).mrcy_var_00 = 4u16;
    jp
}
pub unsafe fn MaridiaRefillCandy_Instr_2(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*Get_MaridiaRefillCandy(ram_word!(game, cur_enemy_index) as i32, game)).mrcy_var_00 = 8u16;
    jp
}
pub unsafe fn MaridiaRefillCandy_Instr_3(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*Get_MaridiaRefillCandy(ram_word!(game, cur_enemy_index) as i32, game)).mrcy_var_00 = 12u16;
    jp
}
pub unsafe fn MaridiaRefillCandy_Init(game: &mut Game) {
    let E: *mut Enemy_MaridiaRefillCandy =
        Get_MaridiaRefillCandy(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).mrcy_parameter_1 = ((0xa3b482i32) as u16);
    (*E).mrcy_var_D = 0u16;
    (*E).mrcy_var_E = 0u16;
    (*E).mrcy_var_00 = 0u16;
    (*E).base.current_instruction = addr_kMaridiaRefillCandy_Ilist_B3C1 as u16;
    (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Invisible) as u16);
    (*E).mrcy_var_B = (*E).base.x_pos;
    (*E).mrcy_var_C = (*E).base.y_pos;
}
pub unsafe fn CallMaridiaRefillCandyFunc(ea: u32, k: u16, game: &mut Game) {
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
        Get_MaridiaRefillCandy(ram_word!(game, cur_enemy_index) as i32, game);
    CallMaridiaRefillCandyFunc(
        (((*E).mrcy_parameter_1) as u32) | (0xa30000u32),
        ram_word!(game, cur_enemy_index),
        game,
    );
}
pub unsafe fn MaridiaRefillCandy_Func_1(game: &mut Game) {
    let mut v1: i16 = 0;
    if IsSamusWithinEnemy_X(ram_word!(game, cur_enemy_index), 0x80u16, game) != 0 {
        v1 = 1i16;
        if (GetSamusEnemyDelta_X(ram_word!(game, cur_enemy_index), game) as i32 & 0x8000i32) == 0i32
        {
            v1 = 3i16;
        }
        let E: *mut Enemy_MaridiaRefillCandy =
            Get_MaridiaRefillCandy(ram_word!(game, cur_enemy_index) as i32, game);
        (*E).mrcy_var_D = v1 as u16;
        MaridiaRefillCandy_Func_4(game);
        (*E).mrcy_parameter_1 = ((0xa3b4a8i32) as u16);
    }
}
pub unsafe fn MaridiaRefillCandy_Func_2(k: u16, game: &mut Game) {
    let E: *mut Enemy_MaridiaRefillCandy = Get_MaridiaRefillCandy(k as i32, game);
    (*E).base.properties = ((*E).base.properties as i32 & !(kEnemyProps_Invisible as i32)) as u16;
    if (GetSamusEnemyDelta_Y(k, game) as i32 & 0x8000i32) != 0i32 {
        AddToHiLo(
            &raw mut (*E).base.y_pos,
            &raw mut (*E).base.y_subpos,
            -0x8000i32 as u32,
        );
    } else {
        (*E).mrcy_var_D = (*E).mrcy_var_D.wrapping_sub(1);
        MaridiaRefillCandy_Func_4(game);
        (*E).mrcy_var_F = 0u16;
        (*E).mrcy_parameter_1 = ((0xa3b4d6i32) as u16);
    };
}
pub unsafe fn MaridiaRefillCandy_Func_3(k: u16, game: &mut Game) {
    let E: *mut Enemy_MaridiaRefillCandy = Get_MaridiaRefillCandy(k as i32, game);
    let v2: i32 = (*E).mrcy_var_00 as i32 >> 1i32;
    let v: i32 = ((crate::rtl::rom_table_word!(game, 0xa3b415, v2 as isize) as u32) << 16i32
        | crate::rtl::rom_table_word!(game, 0xa3b415, (v2 + 1i32) as isize) as u32)
        as i32;
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        (if (*E).mrcy_var_D != 0u16 { v } else { -v }) as u32,
    );
    if CheckIfEnemyIsOnScreen(game) == 0 {
        MaridiaRefillCandy_Func_4(game);
        return;
    }
    (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Invisible) as u16);
    (*E).base.x_pos = (*E).mrcy_var_B;
    (*E).base.y_pos = (*E).mrcy_var_C;
    (*E).mrcy_var_D = 0u16;
    MaridiaRefillCandy_Func_4(game);
    (*E).mrcy_parameter_1 = ((0xa3b482i32) as u16);
}
pub unsafe fn MaridiaRefillCandy_Func_4(game: &mut Game) {
    let E: *mut Enemy_MaridiaRefillCandy =
        Get_MaridiaRefillCandy(ram_word!(game, cur_enemy_index) as i32, game);
    let mrcy_var_D: u16 = (*E).mrcy_var_D;
    if mrcy_var_D != (*E).mrcy_var_E {
        (*E).mrcy_var_E = mrcy_var_D;
        (*E).base.current_instruction =
            crate::rtl::rom_table_word!(game, 0xa3b40d, mrcy_var_D as isize);
        (*E).base.instruction_timer = 1u16;
        (*E).base.timer = 0u16;
    }
}
pub fn MaridiaRefillCandy_Func_5() {}
pub fn MaridiaRefillCandy_Func_6() {}
pub unsafe fn NorfairSlowFireball_Init(game: &mut Game) {
    let v1: *mut EnemyData = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
    (*v1).parameter_2 = 6u16;
    (*v1).current_instruction =
        crate::rtl::rom_table_word!(game, 0xa3b667, ((*v1).properties as i32 & 3i32) as isize);
    StoneZoomer_E67A(ram_word!(game, cur_enemy_index), game);
}
pub unsafe fn NorfairSlowFireball_Func_1(game: &mut Game) {
    let current_instruction: u16 =
        (*gEnemyData(ram_word!(game, cur_enemy_index) as i32, game)).current_instruction;
    if current_instruction != 0 {
        Unreachable(game);
    }
}
pub unsafe fn Bang_Instr_1(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    QueueSfx2_Max6(0x56u16, game);
    jp
}
pub unsafe fn Bang_Instr_2(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*Get_Bang(ram_word!(game, cur_enemy_index) as i32, game)).bang_var_22 = 1u16;
    jp
}
pub unsafe fn Bang_Init(game: &mut Game) {
    let E: *mut Enemy_Bang = Get_Bang(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).bang_var_B = (*E).base.palette_index;
    (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_BlockPlasmaBeam) as u16);
    (*E).bang_var_F = ((0xa3bca5i32) as u16);
    (*E).bang_var_00 = 16u16;
    (*E).bang_var_01 = 0u16;
    (*E).bang_var_02 = 0u16;
    (*E).bang_var_20 = 0u16;
    (*E).bang_var_21 = 0u16;
    (*E).bang_var_22 = 0u16;
    (*E).bang_var_0B = crate::rtl::rom_table_word!(
        game,
        0xa3ba84,
        *(&raw mut (*E).bang_parameter_2 as *mut u8).offset(0) as isize
    );
    let v1: i32 = 2i32 * *(&raw mut (*E).bang_parameter_2 as *mut u8).offset(1isize) as i32;
    let v2: u16 = crate::rtl::rom_table_word!(game, 0xa3bc6a, v1 as isize);
    (*E).bang_var_0C = v2;
    (*E).bang_var_0D = v2;
    (*E).bang_var_0E = crate::rtl::rom_table_word!(game, 0xa3bc6a, (v1 + 1i32) as isize);
    if (*E).base.current_instruction == 0 {
        (*E).bang_var_F = ((0xa3bcc1i32) as u16);
    }
    (*E).base.current_instruction = addr_kBang_Ilist_B75E as u16;
}
pub unsafe fn Bang_Main(game: &mut Game) {
    let E: *mut Enemy_Bang = Get_Bang(ram_word!(game, cur_enemy_index) as i32, game);
    Call((((*E).bang_parameter_1) as u32) | (0xa30000u32), game);
}
pub unsafe fn Bang_Func_1(game: &mut Game) {
    let v0: u16 = ((*Get_Bang(ram_word!(game, cur_enemy_index) as i32 + 64i32, game)).bang_var_20)
        .wrapping_add(10u16);
    (*Get_Bang(ram_word!(game, cur_enemy_index) as i32, game)).bang_var_20 = v0;
    Bang_Func_18(game);
    let E: *mut Enemy_Bang = Get_Bang(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
}
pub unsafe fn Bang_Func_2(game: &mut Game) {
    let v0: u16 = ((*Get_Bang(ram_word!(game, cur_enemy_index) as i32 + 1984i32, game))
        .bang_var_00)
        .wrapping_add(20u16);
    let E: *mut Enemy_Bang = Get_Bang(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).bang_var_20 = v0;
    Bang_Func_18(game);
    (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
}
pub unsafe fn Bang_Func_3(game: &mut Game) {
    let mut bang_var_B: i16 = 0;
    Bang_Func_5(game);
    let E0: *mut Enemy_Bang = Get_Bang(ram_word!(game, cur_enemy_index) as i32, game);
    let x_pos: u16 = (*E0).base.x_pos;
    let v3: i32 = ram_word!(game, cur_enemy_index) as i32 >> 1i32;
    set_ram_word!(game.enemy_drawing_queue[(v3 + 91i32)] = x_pos);
    let E1: *mut Enemy_Bang = Get_Bang(ram_word!(game, cur_enemy_index) as i32 + 64i32, game);
    (*E1).base.x_pos = x_pos;
    let y_pos: u16 = (*E0).base.y_pos;
    set_ram_word!(game.enemy_drawing_queue[(v3 + 93i32)] = y_pos);
    (*E1).base.y_pos = y_pos;
    bang_var_B = (*E0).bang_var_B as i16;
    if ((*E0).bang_var_20 as i32 & 1i32) != 0i32 {
        bang_var_B = 3072i16;
    }
    (*E0).base.palette_index = bang_var_B as u16;
    Bang_Func_18(game);
    if (*E0).bang_var_22 != 0 {
        (*E0).bang_var_22 = 0u16;
        if (*E0).bang_var_20 == 9u16 {
            (*E0).base.invincibility_timer = 16u16;
            (*E0).base.properties = ((*E0).base.properties) | ((kEnemyProps_Tangible) as u16);
            let v7: u16 = DetermineDirectionOfSamusFromEnemy(game);
            let v8: u16 = Bang_Func_4(v7, game);
            EnemyDeathAnimation(ram_word!(game, cur_enemy_index), v8, game);
            let v9: u16 = ram_word!(game, cur_enemy_index);
            let E: *mut Enemy_Bang =
                Get_Bang(ram_word!(game, cur_enemy_index) as i32 + 64i32, game);
            (*E).base.properties = ((*E).base.properties) | (0x200u16);
            update_ram_word!(
                game.enemy_drawing_queue[((v9 as i32 >> 1i32) + 97i32)],
                |fresh1| (fresh1) | (0x200u16)
            );
        } else {
            (*E0).bang_var_20 = (*E0).bang_var_20.wrapping_add(1);
            Bang_Func_18(game);
        }
    }
}
pub unsafe fn Bang_Func_4(a: u16, game: &mut Game) -> u16 {
    let mut i: i32 = 0;
    if ram_word!(game, projectile_counter) as i32 - 5i32 & 0x8000i32 == 0 {
        return 1u16;
    }
    i = 0i32;
    while ram_word!(game, projectile_damage[(i >> 1i32)]) != 0 {
        i += 2i32;
    }
    let E: *mut Enemy_Bang = Get_Bang(ram_word!(game, cur_enemy_index) as i32, game);
    let v3: i32 = i >> 1i32;
    set_ram_word!(game.projectile_x_pos[v3] = (*E).base.x_pos);
    set_ram_word!(game.projectile_y_pos[v3] = (*E).base.y_pos);
    set_ram_word!(game.projectile_dir[v3] = a);
    set_ram_word!(
        game.projectile_type[v3] =
            (ram_word!(game, equipped_beams) as i32 & 0xfi32 | 0x10i32) as u16
    );
    update_ram_word!(game.projectile_counter, wrapping_add(1));
    ProjectileReflection(i as u16, game);
    set_ram_word!(game.projectile_damage[v3] = (*E).bang_var_E);
    QueueSfx1_Max6(
        crate::rtl::rom_table_word!(
            game,
            0xa3bc4a,
            (ram_word!(game, projectile_type[v3]) as i32 & 0xfi32) as isize
        ),
        game,
    );
    0u16
}
pub unsafe fn Bang_Func_5(game: &mut Game) {
    let E: *mut Enemy_Bang = Get_Bang(ram_word!(game, cur_enemy_index) as i32, game);
    CallEnemyPreInstr((((*E).bang_var_F) as u32) | (0xa30000u32), game);
}
pub unsafe fn Bang_Func_6(k: u16, game: &mut Game) {
    let E: *mut Enemy_Bang = Get_Bang(ram_word!(game, cur_enemy_index) as i32, game);
    let v1: u16 = ((*E).bang_var_00).wrapping_sub(1u16);
    (*E).bang_var_00 = v1;
    if v1 == 0 {
        (*E).bang_var_00 = 16u16;
        (*E).bang_var_F = ((0xa3bcc5i32) as u16);
    }
}
pub fn Bang_Func_7(k: u16) {}
pub unsafe fn Bang_Func_8(k: u16, game: &mut Game) {
    let v1: u8 = ((CalculateAngleOfSamusFromEnemy(ram_word!(game, cur_enemy_index), game)) as u8)
        .wrapping_sub(64u8);
    let E: *mut Enemy_Bang = Get_Bang(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).bang_var_01 = v1 as u16;
    (*E).bang_var_F = ((0xa3bd1ci32) as u16);
    (*E).bang_var_07 = 0u16;
    (*E).bang_var_08 = 0u16;
    (*E).bang_var_09 = 0u16;
    (*E).bang_var_0A = 0u16;
}
pub unsafe fn Bang_Func_9(game: &mut Game) {
    let v1: u8 = ((CalculateAngleOfSamusFromEnemy(ram_word!(game, cur_enemy_index), game)) as u8)
        .wrapping_sub(64u8);
    let E: *mut Enemy_Bang = Get_Bang(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).bang_var_02 = v1 as u16;
    let v3: u16 = SignExtend8(((v1) as u16).wrapping_sub((*E).bang_var_01));
    let v4: u16 = Abs16(v3);
    if v4 as i32 - 48i32 & 0x8000i32 == 0 {
        (*E).bang_var_F = ((0xa3bd2ci32) as u16);
    }
}
pub unsafe fn Bang_Func_10(k: u16, game: &mut Game) {
    Bang_Func_14(game);
    Bang_Func_15(game);
    Bang_Func_12(game);
    Bang_Func_9(game);
}
pub unsafe fn Bang_Func_11(k: u16, game: &mut Game) {
    Bang_Func_14(game);
    Bang_Func_15(game);
    Bang_Func_13(game);
    let E: *mut Enemy_Bang = Get_Bang(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).bang_var_0A != 0u16 || ((*E).bang_var_09 as i16) <= 0i16 {
        (*E).bang_var_F = ((0xa3bca5i32) as u16);
    }
}
pub unsafe fn Bang_Func_12(game: &mut Game) {
    let E: *mut Enemy_Bang = Get_Bang(ram_word!(game, cur_enemy_index) as i32, game);
    let v1: i16 = (((*E).bang_var_0C) as i16).wrapping_sub(1i16);
    (*E).bang_var_0C = v1 as u16;
    if v1 < 0i16 {
        (*E).bang_var_0C = (*E).bang_var_0D;
        if (((*E).bang_var_0A) as i16).wrapping_sub(((*E).bang_var_0B) as i16) < 0i16 {
            (*E).bang_var_09 = ((*E).bang_var_09).wrapping_add(22u16);
            (*E).bang_var_0A = ((*E).bang_var_0A).wrapping_add((*E).bang_var_09);
        }
    }
}
pub unsafe fn Bang_Func_13(game: &mut Game) {
    let E: *mut Enemy_Bang = Get_Bang(ram_word!(game, cur_enemy_index) as i32, game);
    let v1: i16 = (((*E).bang_var_0C) as i16).wrapping_sub(1i16);
    (*E).bang_var_0C = v1 as u16;
    if v1 < 0i16 {
        (*E).bang_var_0C = (*E).bang_var_0E;
        (*E).bang_var_09 = ((*E).bang_var_09).wrapping_sub(22u16);
        (*E).bang_var_0A = ((*E).bang_var_0A).wrapping_sub((*E).bang_var_09);
    }
}
pub unsafe fn Bang_Func_14(game: &mut Game) {
    let E: *mut Enemy_Bang = Get_Bang(ram_word!(game, cur_enemy_index) as i32, game);
    let r18: u16 = crate::rtl::rom_table_word!(
        game,
        0xa0b1c3,
        ((((*E).bang_var_01) as u8).wrapping_add(64u8)) as isize
    );
    if (r18 as i32 & 0x8000i32) != 0i32 {
        (*E).bang_var_07 = 1u16;
    }
    let mut t: u32 = ((Abs16(r18) as i32 >> 8i32) * (*E).bang_var_0A as i32) as u32;
    if (*E).bang_var_07 != 0 {
        t = -(t as i32) as u32;
    }
    t = t.wrapping_add((*E).base.x_subpos as u32);
    (*E).base.x_subpos = t as u16;
    (*E).base.x_pos = (t >> 16i32) as u16;
}
pub unsafe fn Bang_Func_15(game: &mut Game) {
    let E: *mut Enemy_Bang = Get_Bang(ram_word!(game, cur_enemy_index) as i32, game);
    let r18: u16 = crate::rtl::rom_table_word!(game, 0xa0b1c3, (*E).bang_var_01 as u8 as isize);
    if (r18 as i32 & 0x8000i32) != 0i32 {
        (*E).bang_var_08 = 1u16;
    }
    let mut t: u32 = ((Abs16(r18) as i32 >> 8i32) * (*E).bang_var_0A as i32) as u32;
    if (*E).bang_var_08 != 0 {
        t = -(t as i32) as u32;
    }
    t = t.wrapping_add((*E).base.y_subpos as u32);
    (*E).base.y_subpos = t as u16;
    (*E).base.y_pos = (t >> 16i32) as u16;
}
pub unsafe fn Bang_Func_18(game: &mut Game) {
    let E: *mut Enemy_Bang = Get_Bang(ram_word!(game, cur_enemy_index) as i32, game);
    let bang_var_20: u16 = (*E).bang_var_20;
    if bang_var_20 != (*E).bang_var_21 {
        (*E).bang_var_21 = bang_var_20;
        (*E).base.current_instruction =
            crate::rtl::rom_table_word!(game, 0xa3b722, bang_var_20 as isize);
        (*E).base.instruction_timer = 1u16;
        (*E).base.timer = 0u16;
    }
}
pub unsafe fn Bang_Shot(game: &mut Game) {
    let E: *mut Enemy_Bang = Get_Bang(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).bang_var_F != 0xbcc1u16 {
        (*E).bang_var_01 = crate::rtl::rom_table_word!(
            game,
            0xa3ba94,
            (ram_word!(
                game,
                projectile_dir[ram_word!(game, collision_detection_index)]
            ) as i32
                & 0xfi32) as isize,
        );
        (*E).bang_var_F = ((0xa3bd2ci32) as u16);
        (*E).bang_var_07 = 0u16;
        (*E).bang_var_08 = 0u16;
        (*E).bang_var_09 = 256u16;
        (*E).bang_var_0A = 1536u16;
    }
    if (*E).bang_var_20 != 9u16 {
        (*E).bang_var_20 = (*E).bang_var_20.wrapping_add(1);
        Bang_Func_18(game);
        let v1: i32 = ram_word!(game, collision_detection_index) as i32;
        (*E).bang_var_E = ((*E).bang_var_E).wrapping_add(ram_word!(game, projectile_damage[v1]));
        update_ram_word!(game.projectile_dir[v1], |fresh2| (fresh2) | (0x10u16));
        if (*E).bang_var_20 == 9u16 {
            (*E).bang_var_D = 1u16;
        }
    }
}
pub unsafe fn Skree_Instr_1(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*Get_Skree(ram_word!(game, cur_enemy_index) as i32, game)).skree_var_E = 1u16;
    jp
}
pub unsafe fn Skree_Init(game: &mut Game) {
    let E: *mut Enemy_Skree = Get_Skree(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).skree_var_C = 0u16;
    (*E).skree_var_D = 0u16;
    (*E).skree_var_E = 0u16;
    (*E).base.current_instruction = addr_kSkree_Ilist_C65E as u16;
    (*E).skree_var_B = ((0xa3c6d5i32) as u16);
}
pub unsafe fn CallSkreeFunc(ea: u32, game: &mut Game) {
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
    let E: *mut Enemy_Skree = Get_Skree(ram_word!(game, cur_enemy_index) as i32, game);
    CallSkreeFunc((((*E).skree_var_B) as u32) | (0xa30000u32), game);
}
pub unsafe fn Skree_Func_1(game: &mut Game) {
    let E: *mut Enemy_Skree = Get_Skree(ram_word!(game, cur_enemy_index) as i32, game);
    if abs16(((*E).base.x_pos).wrapping_sub(ram_word!(game, samus_x_pos))) < 0x30u16 {
        (*E).skree_var_C = (*E).skree_var_C.wrapping_add(1);
        Skree_Func_5(game);
        (*E).skree_var_B = ((0xa3c6f7i32) as u16);
    }
}
pub unsafe fn Skree_Func_2(k: u16, game: &mut Game) {
    let E: *mut Enemy_Skree = Get_Skree(k as i32, game);
    if (*E).skree_var_E != 0 {
        (*E).skree_var_E = 0u16;
        (*E).skree_var_C = (*E).skree_var_C.wrapping_add(1);
        Skree_Func_5(game);
        (*E).skree_var_B = ((0xa3c716i32) as u16);
        QueueSfx2_Max6(0x5bu16, game);
    }
}
pub unsafe fn Skree_Func_3(game: &mut Game) {
    let E: *mut Enemy_Skree = Get_Skree(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).skree_var_A = 21u16;
    let v1: u16 = ((*E).base.properties) | (3u16);
    (*E).base.properties = v1;
    if EnemyFunc_BF8A(
        ram_word!(game, cur_enemy_index),
        v1,
        (6i16 as i32) << 16i32,
        game,
    ) as i32
        & 1i32
        != 0
    {
        (*E).base.instruction_timer = 1u16;
        (*E).base.timer = 0u16;
        (*E).skree_var_B = ((0xa3c77fi32) as u16);
        QueueSfx2_Max6(0x5cu16, game);
    } else {
        (*E).base.y_pos = ((*E).base.y_pos).wrapping_add(6u16);
        (*E).base.x_pos = ((*E).base.x_pos as i32
            + if (((*E).base.x_pos) as i16).wrapping_sub((ram_word!(game, samus_x_pos)) as i16)
                >= 0i16
            {
                -1i32
            } else {
                1i32
            }) as u16;
    };
}
pub unsafe fn Skree_Func_4(game: &mut Game) {
    let E: *mut Enemy_Skree = Get_Skree(ram_word!(game, cur_enemy_index) as i32, game);
    let fresh18 = (*E).skree_var_A;
    (*E).skree_var_A = (*E).skree_var_A.wrapping_sub(1);
    if fresh18 == 1u16 {
        (*gEnemySpawnData(ram_word!(game, cur_enemy_index) as i32, game)).vram_tiles_index =
            ((*E).base.vram_tiles_index) | ((*E).base.palette_index);
        (*E).base.palette_index = 2560u16;
        (*E).base.vram_tiles_index = 0u16;
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Deleted) as u16);
    } else {
        if (*E).skree_var_A == 8u16 {
            let v2: u16 = ram_word!(game, cur_enemy_index);
            let mut parameter = SpawnEprojWithGfx(
                8u16,
                ram_word!(game, cur_enemy_index),
                addr_stru_868BC2 as u16,
                game,
            );
            parameter = SpawnEprojWithGfx(parameter, v2, addr_stru_868BD0 as u16, game);
            parameter = SpawnEprojWithGfx(parameter, v2, addr_stru_868BDE as u16, game);
            SpawnEprojWithGfx(parameter, v2, addr_stru_868BEC as u16, game);
        }
        (*E).base.y_pos = (*E).base.y_pos.wrapping_add(1);
    };
}
pub unsafe fn Skree_Func_5(game: &mut Game) {
    let E: *mut Enemy_Skree = Get_Skree(ram_word!(game, cur_enemy_index) as i32, game);
    let skree_var_C: u16 = (*E).skree_var_C;
    if skree_var_C != (*E).skree_var_D {
        (*E).skree_var_D = skree_var_C;
        (*E).base.current_instruction =
            crate::rtl::rom_table_word!(game, 0xa3c69c, skree_var_C as isize);
        (*E).base.instruction_timer = 1u16;
        (*E).base.timer = 0u16;
    }
}
pub unsafe fn Skree_Shot(game: &mut Game) {
    Enemy_NormalShotAI_SkipSomeParts_A3(game);
    let E: *mut Enemy_Skree = Get_Skree(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).base.health == 0 {
        let v1: u16 = ram_word!(game, cur_enemy_index);
        let mut parameter = SpawnEprojWithGfx(
            (*E).skree_var_A,
            ram_word!(game, cur_enemy_index),
            addr_stru_868BC2 as u16,
            game,
        );
        parameter = SpawnEprojWithGfx(parameter, v1, addr_stru_868BD0 as u16, game);
        parameter = SpawnEprojWithGfx(parameter, v1, addr_stru_868BDE as u16, game);
        SpawnEprojWithGfx(parameter, v1, addr_stru_868BEC as u16, game);
        let mut v5: u16 = 2u16;
        if (ram_word!(
            game,
            projectile_type[ram_word!(game, collision_detection_index)]
        ) as i32
            & 0xf00i32)
            != 512i32
        {
            v5 = 0u16;
        }
        EnemyDeathAnimation(
            (2u16).wrapping_mul(ram_word!(game, collision_detection_index)),
            v5,
            game,
        );
    }
}
pub unsafe fn MaridiaSnail_Instr_1(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*Get_MaridiaSnail(k as i32, game)).msl_var_F = *jp;
    jp.offset(1)
}
pub unsafe fn MaridiaSnail_Instr_2(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*Get_MaridiaSnail(k as i32, game)).msl_var_D = *jp;
    jp.offset(1)
}
pub unsafe fn MaridiaSnail_Instr_4(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E: *mut Enemy_MaridiaSnail = Get_MaridiaSnail(k as i32, game);
    (*E).msl_var_07 = *jp.offset(0);
    (*E).msl_var_C = *((&raw mut (*(game.g_rom.offset(crate::rtl::lorom_offset(0xa3cd42))
        as *mut MaridiaSnailData2)
        .offset(0))
    .field_6 as *mut u8)
        .offset((8i32 * (*E).msl_var_07 as i32) as isize) as *mut u16);
    jp.offset(1)
}
pub unsafe fn MaridiaSnail_Instr_3(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E: *mut Enemy_MaridiaSnail = Get_MaridiaSnail(k as i32, game);
    (*E).base.x_pos = ((*E).base.x_pos).wrapping_add(*jp.offset(0));
    (*E).base.y_pos = ((*E).base.y_pos).wrapping_add(*jp.offset(1));
    jp.offset(2)
}
pub unsafe fn MaridiaSnail_Instr_5(k: u16, mut jp: *const u16, game: &mut Game) -> *const u16 {
    if (*Get_MaridiaSnail(k as i32, game)).msl_var_08 == 2u16
        || (NextRandom(game) as i32 & 1i32) != 0i32
    {
        jp = jp.offset(-(3i32 as isize));
    }
    jp
}
pub unsafe fn MaridiaSnail_Func_1(k: u16, game: &mut Game) {
    MaridiaSnail_Func_17(k, game);
}
pub unsafe fn MaridiaSnail_Init(game: &mut Game) {
    let E: *mut Enemy_MaridiaSnail =
        Get_MaridiaSnail(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).msl_var_F = ((0xa3cf5fi32) as u16);
    (*E).base.spritemap_pointer = addr_kSpritemap_Nothing_A3 as u16;
    (*E).base.instruction_timer = 1u16;
    let current_instruction: u16 = (*E).base.current_instruction;
    (*E).base.current_instruction = (*(game.g_rom.offset(crate::rtl::lorom_offset(0xa3cd42))
        as *mut MaridiaSnailData2)
        .offset(current_instruction as isize))
    .field_0;
    (*E).base.properties = ((*E).base.properties)
        | ((*(game.g_rom.offset(crate::rtl::lorom_offset(0xa3cd42)) as *mut MaridiaSnailData2)
            .offset(current_instruction as isize))
        .field_2);
    (*E).msl_var_D = (*(game.g_rom.offset(crate::rtl::lorom_offset(0xa3cd42))
        as *mut MaridiaSnailData2)
        .offset(current_instruction as isize))
    .field_4;
    (*E).msl_var_C = (*(game.g_rom.offset(crate::rtl::lorom_offset(0xa3cd42))
        as *mut MaridiaSnailData2)
        .offset(current_instruction as isize))
    .field_6;
    (*E).msl_var_08 = 0u16;
    (*E).msl_var_06 = (*E).msl_parameter_1;
    MaridiaSnail_Func_2(
        ram_word!(game, cur_enemy_index),
        (current_instruction).wrapping_mul(8u16),
        game,
    );
}
pub unsafe fn MaridiaSnail_Func_2(k: u16, j: u16, game: &mut Game) {
    let v2: i32 = j as i32 >> 1i32;
    let r18: u16 = crate::rtl::rom_table_word!(game, 0xa3cd82, v2 as isize);
    let r20: u16 = crate::rtl::rom_table_word!(game, 0xa3cd82, (v2 + 1i32) as isize);
    let r22: u16 = crate::rtl::rom_table_word!(game, 0xa3cd82, (v2 + 2i32) as isize);
    let r24: u16 = crate::rtl::rom_table_word!(game, 0xa3cd82, (v2 + 3i32) as isize);
    let E: *mut Enemy_MaridiaSnail = Get_MaridiaSnail(k as i32, game);
    (*E).msl_var_A = (r20 as i32
        + (r18 as i32
            ^ crate::rtl::rom_table_word!(game, 0xa3cca2, (*E).msl_parameter_1 as isize) as i32))
        as u16;
    (*E).msl_var_B = (r24 as i32
        + (r22 as i32
            ^ crate::rtl::rom_table_word!(game, 0xa3cca2, (*E).msl_parameter_1 as isize) as i32))
        as u16;
}
pub unsafe fn MaridiaSnail_Func_3(k: u16, game: &mut Game) {
    let E: *mut Enemy_MaridiaSnail = Get_MaridiaSnail(k as i32, game);
    (*E).msl_var_F = crate::rtl::rom_table_word!(game, 0xa3cdd2, (*E).msl_var_07 as isize);
}
pub unsafe fn MaridiaSnail_Main(game: &mut Game) {
    MaridiaSnail_Func_4(ram_word!(game, cur_enemy_index), game);
    MaridiaSnail_Func_5(ram_word!(game, cur_enemy_index), game);
    MaridiaSnail_Func_6(ram_word!(game, cur_enemy_index), game);
    let E: *mut Enemy_MaridiaSnail =
        Get_MaridiaSnail(ram_word!(game, cur_enemy_index) as i32, game);
    CallEnemyPreInstr((((*E).msl_var_F) as u32) | (0xa30000u32), game);
}
pub unsafe fn MaridiaSnail_Func_4(k: u16, game: &mut Game) {
    let msl_var_08: u16 = (*Get_MaridiaSnail(k as i32, game)).msl_var_08;
    if msl_var_08 != 3u16
        && msl_var_08 != 4u16
        && msl_var_08 != 5u16
        && ram_word!(game, earthquake_timer) == 30u16
        && ram_word!(game, earthquake_type) == 20u16
    {
        MaridiaSnail_Func_14(k, game);
    }
}
pub unsafe fn MaridiaSnail_Func_5(k: u16, game: &mut Game) {
    let current_block: u64;
    let E: *mut Enemy_MaridiaSnail = Get_MaridiaSnail(k as i32, game);
    let msl_var_08: u16 = (*E).msl_var_08;
    if msl_var_08 != 1u16 && msl_var_08 != 3u16 && msl_var_08 != 4u16 && msl_var_08 != 5u16 {
        if (*E).base.y_pos as i32 - ram_word!(game, samus_y_pos) as i32 + 96i32 & 0x8000i32 == 0 {
            if (((*E).base.x_pos) as i16).wrapping_sub((ram_word!(game, samus_x_pos)) as i16) < 0i16
            {
                if *ram_ptr!(game; 0xa1e) != 4u8 {
                    current_block = 3898722140564953443;
                } else {
                    current_block = 6873731126896040597;
                }
            } else if *ram_ptr!(game; 0xa1e) != 8u8 {
                current_block = 3898722140564953443;
            } else {
                current_block = 6873731126896040597;
            }
            if current_block == 3898722140564953443 {
            } else {
                if (*E).msl_var_08 == 2u16 {
                    return;
                }
                if (*E).msl_var_D as i32 != 0xa3cf5fi32 & 0xffffi32 {
                    (*E).base.current_instruction = (*E).msl_var_D;
                    (*E).base.spritemap_pointer = addr_kSpritemap_Nothing_A3 as u16;
                    (*E).base.instruction_timer = 1u16;
                    (*E).base.timer = 0u16;
                    (*E).msl_var_08 = 2u16;
                    return;
                }
            }
        }
        (*E).msl_var_08 = 0u16;
    }
}
pub unsafe fn MaridiaSnail_Func_6(k: u16, game: &mut Game) {
    let current_block: u64;
    let E: *mut Enemy_MaridiaSnail = Get_MaridiaSnail(k as i32, game);
    if ((*E).msl_var_F as i32 != 0xa3d1b3i32 & 0xffffi32) {
        if (*E).msl_var_08 == 1u16 {
            if ram_word!(game, samus_has_momentum_flag) != 0u16
                || (*E).msl_var_F as i32 != 0xa3cf5fi32 & 0xffffi32
            {
                current_block = 2304351028894610342;
            } else {
                current_block = 12573672619892793823;
            }
        } else if ram_word!(game, samus_has_momentum_flag) == 0 {
            let msl_var_08: u16 = (*E).msl_var_08;
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
unsafe fn Shift8AddMagn(a: i16, s: i32) -> u32 {
    let mut r: i32 = (a as i32) << 8i32;
    r = r + if r < 0i32 { -(s << 16i32) } else { s << 16i32 };
    r as u32
}
pub unsafe fn MaridiaSnail_Func_7(k: u16, game: &mut Game) {
    let E: *mut Enemy_MaridiaSnail = Get_MaridiaSnail(k as i32, game);
    if (*E).msl_var_07 >= 4u16 {
        if EnemyFunc_BC76(k, Shift8AddMagn((*E).msl_var_B as i16, 7i32) as i32, game) != 0 {
            return;
        }
    } else if EnemyFunc_BBBF(k, Shift8AddMagn((*E).msl_var_A as i16, 7i32) as i32, game) != 0 {
        return;
    }
    MaridiaSnail_Func_14(k, game);
}
pub unsafe fn MaridiaSnail_Func_9(k: u16, game: &mut Game) {
    let v1: u16 = (if (*Get_MaridiaSnail(k as i32, game)).msl_var_05 != 0u16 {
        addr_stru_A3CD22 as i32
    } else {
        addr_stru_A3CCE2 as i32
    }) as u16;
    MaridiaSnail_D07E(k, RomPtr_A3(v1, game), game);
}
pub unsafe fn MaridiaSnail_CFB7(k: u16, game: &mut Game) {
    MaridiaSnail_D002(k, RomPtr_A3(addr_stru_A3CCEA as u16, game), game);
}
pub unsafe fn MaridiaSnail_CFBD(k: u16, game: &mut Game) {
    let v1: u16 = (if (*Get_MaridiaSnail(k as i32, game)).msl_var_05 != 0u16 {
        addr_stru_A3CD2A as i32
    } else {
        addr_stru_A3CCF2 as i32
    }) as u16;
    MaridiaSnail_D07E(k, RomPtr_A3(v1, game), game);
}
pub unsafe fn MaridiaSnail_CFCE(k: u16, game: &mut Game) {
    MaridiaSnail_D002(k, RomPtr_A3(addr_stru_A3CCFA as u16, game), game);
}
pub unsafe fn MaridiaSnail_CFD4(k: u16, game: &mut Game) {
    let v1: u16 = (if (*Get_MaridiaSnail(k as i32, game)).msl_var_05 != 0u16 {
        addr_stru_A3CD32 as i32
    } else {
        addr_stru_A3CD02 as i32
    }) as u16;
    MaridiaSnail_D07E(k, RomPtr_A3(v1, game), game);
}
pub unsafe fn MaridiaSnail_CFE5(k: u16, game: &mut Game) {
    MaridiaSnail_D002(k, RomPtr_A3(addr_stru_A3CD0A as u16, game), game);
}
pub unsafe fn MaridiaSnail_CFEB(k: u16, game: &mut Game) {
    let mut v1: u16 = 0;
    if (*Get_MaridiaSnail(k as i32, game)).msl_var_05 != 0 {
        v1 = addr_stru_A3CD3A as u16;
    } else {
        v1 = addr_stru_A3CD12 as u16;
    }
    MaridiaSnail_D07E(k, RomPtr_A3(v1, game), game);
}
pub unsafe fn MaridiaSnail_CFFC(k: u16, game: &mut Game) {
    MaridiaSnail_D002(k, RomPtr_A3(addr_stru_A3CD1A as u16, game), game);
}
pub unsafe fn MaridiaSnail_D002(k: u16, j: *const u8, game: &mut Game) {
    MaridiaSnail_Func_10(k, j, game);
    let E: *mut Enemy_MaridiaSnail = Get_MaridiaSnail(k as i32, game);
    let rv: u8 =
        Enemy_MoveRight_ProcessSlopes(k, Shift8AddMagn((*E).msl_var_A as i16, 1i32) as i32, game);
    MaridiaSnail_Func_11(k, j, game);
    if rv != 0 {
        (*E).msl_var_E = 0u16;
        let carry: u8 = EnemyFunc_C8AD(k, game);
        MaridiaSnail_Func_12(k, carry as u16, game);
        if Enemy_MoveDown(k, ((*E).msl_var_B as i16 as i32) << 8i32, game) != 0 {
            (*E).msl_var_A = -((*E).msl_var_A as i32) as u16;
            MaridiaSnail_Func_13(k, *(j.offset(6) as *mut u16), game);
        }
    } else {
        let v7: u16 = ((*E).msl_var_E).wrapping_add(1u16);
        (*E).msl_var_E = v7;
        if v7 as i32 - 4i32 & 0x8000i32 != 0 {
            (*E).msl_var_B = -((*E).msl_var_B as i32) as u16;
            MaridiaSnail_Func_13(k, *(j.offset(4) as *mut u16), game);
        } else {
            MaridiaSnail_Func_14(k, game);
        }
    };
}
pub unsafe fn MaridiaSnail_D07E(k: u16, j: *const u8, game: &mut Game) {
    MaridiaSnail_Func_10(k, j, game);
    let E: *mut Enemy_MaridiaSnail = Get_MaridiaSnail(k as i32, game);
    let rv: u8 = Enemy_MoveDown(k, Shift8AddMagn((*E).msl_var_B as i16, 1i32) as i32, game);
    MaridiaSnail_Func_11(k, j, game);
    if rv != 0 {
        (*E).msl_var_E = 0u16;
        if Enemy_MoveRight_ProcessSlopes(k, ((*E).msl_var_A as i16 as i32) << 8i32, game) != 0 {
            (*E).msl_var_B = -((*E).msl_var_B as i32) as u16;
            MaridiaSnail_Func_13(k, *(j.offset(6) as *mut u16), game);
        } else {
            let carry: u8 = EnemyFunc_C8AD(k, game);
            MaridiaSnail_Func_12(k, carry as u16, game);
        }
    } else {
        let v7: u16 = ((*E).msl_var_E).wrapping_add(1u16);
        (*E).msl_var_E = v7;
        if v7 as i32 - 4i32 & 0x8000i32 != 0 {
            (*E).msl_var_A = -((*E).msl_var_A as i32) as u16;
            MaridiaSnail_Func_13(k, *(j.offset(4) as *mut u16), game);
        } else {
            MaridiaSnail_Func_14(k, game);
        }
    };
}
pub unsafe fn MaridiaSnail_Func_10(k: u16, j: *const u8, game: &mut Game) {
    let E: *mut Enemy_MaridiaSnail = Get_MaridiaSnail(k as i32, game);
    (*E).base.x_pos = ((*E).base.x_pos).wrapping_add(*(j as *mut u16));
    (*E).base.y_pos = ((*E).base.y_pos).wrapping_add(*(j.offset(2) as *mut u16));
}
pub unsafe fn MaridiaSnail_Func_11(k: u16, j: *const u8, game: &mut Game) {
    let E: *mut Enemy_MaridiaSnail = Get_MaridiaSnail(k as i32, game);
    (*E).base.x_pos = ((*E).base.x_pos).wrapping_sub(*(j as *mut u16));
    (*E).base.y_pos = ((*E).base.y_pos).wrapping_sub(*(j.offset(2) as *mut u16));
}
pub unsafe fn MaridiaSnail_Func_12(k: u16, carry: u16, game: &mut Game) {
    let E: *mut Enemy_MaridiaSnail = Get_MaridiaSnail(k as i32, game);
    if carry == 0 {
        if ((*E).msl_var_04).wrapping_add(1u16) >= 0x10u16 {
            (*E).msl_var_05 = 0u16;
        } else {
            (*E).msl_var_04 = (*E).msl_var_04.wrapping_add(1);
        }
    } else {
        (*E).msl_var_05 = 1u16;
        (*E).msl_var_04 = 0u16;
    };
}
pub unsafe fn MaridiaSnail_Func_13(k: u16, a: u16, game: &mut Game) {
    let E: *mut Enemy_MaridiaSnail = Get_MaridiaSnail(k as i32, game);
    (*E).base.current_instruction = a;
    (*E).base.instruction_timer = 1u16;
    (*E).msl_var_05 = 1u16;
    (*E).msl_var_04 = 0u16;
}
pub unsafe fn MaridiaSnail_Func_14(k: u16, game: &mut Game) {
    let E: *mut Enemy_MaridiaSnail = Get_MaridiaSnail(k as i32, game);
    if (*E).msl_var_08 != 3u16 {
        (*E).msl_var_08 = 3u16;
        (*E).msl_var_F = ((0xa3d1b3i32) as u16);
        let v2: i32 = 2i32 * (*E).msl_var_C as i32;
        (*E).base.current_instruction = crate::rtl::rom_table_word!(game, 0xa3d1ab, v2 as isize);
        (*E).msl_var_D = crate::rtl::rom_table_word!(game, 0xa3d1ab, (v2 + 1i32) as isize);
        (*E).base.instruction_timer = 1u16;
        (*E).base.timer = 0u16;
        (*E).msl_var_02 = 0u16;
        (*E).msl_var_03 = 0u16;
        (*E).msl_var_00 = 0u16;
        (*E).msl_var_01 = 0u16;
    }
}
pub unsafe fn MaridiaSnail_Func_15(k: u16, game: &mut Game) {
    let E: *mut Enemy_MaridiaSnail = Get_MaridiaSnail(k as i32, game);
    if (*E).msl_var_08 != 3u16 {
        if Enemy_MoveRight_IgnoreSlopes(
            k,
            (((*E).msl_var_03 as u32) << 16i32 | (*E).msl_var_02 as u32) as i32,
            game,
        ) != 0
        {
            (*E).msl_var_02 = -((*E).msl_var_02 as i32) as u16;
            (*E).msl_var_03 = -((*E).msl_var_03 as i32) as u16;
            (*E).msl_var_20 = 1u16;
            QueueSfx2_Max3(0x70u16, game);
        } else {
            let mut delta: u32 = 0;
            if ((*E).msl_var_03 as i32 & 0x8000i32) != 0i32 {
                delta = 4096u32;
            } else {
                delta = -4096i32 as u32;
            }
            let t: u32 =
                (((*E).msl_var_03 as u32) << 16i32 | (*E).msl_var_02 as u32).wrapping_add(delta);
            (*E).msl_var_02 = t as u16;
            if t >> 16i32 != 0 {
                (*E).msl_var_03 = (t >> 16i32) as u16;
            }
        }
    }
    if Enemy_MoveDown(
        k,
        (((*E).msl_var_01 as u32) << 16i32 | (*E).msl_var_00 as u32) as i32,
        game,
    ) != 0
    {
        let msl_var_01: i16 = (*E).msl_var_01 as i16;
        if msl_var_01 >= 0i16 && msl_var_01 as i32 - 3i32 & 0x8000i32 != 0 {
            (*E).msl_var_03 = 0u16;
            (*E).msl_var_02 = 0u16;
            (*E).msl_var_01 = 0u16;
            (*E).msl_var_00 = 0u16;
            (*E).msl_var_E = 0u16;
            (*E).msl_var_04 = 0u16;
            (*E).msl_var_05 = 1u16;
            if (*E).msl_var_08 == 3u16 {
                (*E).msl_var_08 = 0u16;
            } else {
                (*E).msl_var_08 = 1u16;
                (*E).msl_parameter_1 = 8u16;
                MaridiaSnail_Func_18(k, game);
                MaridiaSnail_Func_16(k, game);
            }
            (*E).base.properties =
                ((*E).msl_var_C as i32 | (*E).base.properties as i32 & !(3i32)) as u16;
            StoneZoomer_E67A(k, game);
            (*E).msl_var_F = ((0xa3cf5fi32) as u16);
            (*E).base.current_instruction = (*E).msl_var_D;
            (*E).base.instruction_timer = 1u16;
            (*E).base.timer = 0u16;
        } else {
            (*E).msl_var_00 = -((*E).msl_var_00 as i32) as u16;
            (*E).msl_var_01 = -((*E).msl_var_01 as i32) as u16;
            (*E).msl_var_20 = 0u16;
        }
    } else {
        let t_0: u32 =
            (((*E).msl_var_01 as u32) << 16i32 | (*E).msl_var_00 as u32).wrapping_add(0x2000u32);
        (*E).msl_var_00 = t_0 as u16;
        if (t_0 >> 16i32).wrapping_sub(4u32) & 0x8000u32 != 0 {
            (*E).msl_var_01 = (t_0 >> 16i32) as u16;
        }
    };
}
pub unsafe fn MaridiaSnail_Func_16(k: u16, game: &mut Game) {
    let E: *mut Enemy_MaridiaSnail = Get_MaridiaSnail(k as i32, game);
    let v2: i32 = ((4u16).wrapping_mul((*E).msl_var_C)) as i32 >> 1i32;
    (*E).base.current_instruction = crate::rtl::rom_table_word!(game, 0xa3d30d, v2 as isize);
    (*E).msl_var_D = crate::rtl::rom_table_word!(game, 0xa3d30d, (v2 + 1i32) as isize);
}
pub unsafe fn MaridiaSnail_Func_17(k: u16, game: &mut Game) -> u8 {
    let E: *mut Enemy_MaridiaSnail = Get_MaridiaSnail(k as i32, game);
    if (*E).msl_var_05 != 0 {
        return 0u8;
    }
    if (*E).msl_var_07 >= 4u16 {
        return (MaridiaSnail_Func_18(k, game)) & (1u8);
    }
    if ((*E).msl_var_07 as i32 & 1i32) != 0i32 {
        ((if (*E).base.y_pos >= ram_word!(game, samus_y_pos) {
            MaridiaSnail_Func_19(k, game) as i32
        } else {
            0i32
        }) as u8)
    } else {
        ((if (*E).base.y_pos < ram_word!(game, samus_y_pos) {
            MaridiaSnail_Func_19(k, game) as i32
        } else {
            0i32
        }) as u8)
    }
}
pub unsafe fn MaridiaSnail_Func_18(k: u16, game: &mut Game) -> u8 {
    let E: *mut Enemy_MaridiaSnail = Get_MaridiaSnail(k as i32, game);
    if (*E).msl_var_C != 0 {
        ((if (*E).base.x_pos >= ram_word!(game, samus_x_pos) {
            MaridiaSnail_Func_19(k, game) as i32
        } else {
            0i32
        }) as u8)
    } else {
        ((if (*E).base.x_pos < ram_word!(game, samus_x_pos) {
            MaridiaSnail_Func_19(k, game) as i32
        } else {
            0i32
        }) as u8)
    }
}
pub unsafe fn MaridiaSnail_Func_19(k: u16, game: &mut Game) -> u8 {
    let E: *mut Enemy_MaridiaSnail = Get_MaridiaSnail(k as i32, game);
    if (*E).msl_var_08 == 2u16 || (*E).msl_var_F == 0xcf5fu16 {
        return 0u8;
    }
    let v2: u16 = crate::rtl::rom_table_word!(game, 0xa3cdc2, (*E).msl_var_07 as isize);
    (*E).msl_var_07 = v2;
    (*E).base.current_instruction =
        *((&raw mut (*(game.g_rom.offset(crate::rtl::lorom_offset(0xa3cd42))
            as *mut MaridiaSnailData2)
            .offset(0))
        .field_0 as *mut u8)
            .offset((8i32 * v2 as i32) as isize) as *mut VoidP);
    (*E).base.properties = (*((&raw mut (*(game.g_rom.offset(crate::rtl::lorom_offset(0xa3cd42))
        as *mut MaridiaSnailData2)
        .offset(0))
    .field_2 as *mut u8)
        .offset((8i32 * v2 as i32) as isize) as *mut u16) as i32
        | (*E).base.properties as i32 & !(3i32)) as u16;
    (*E).msl_var_D = *((&raw mut (*(game.g_rom.offset(crate::rtl::lorom_offset(0xa3cd42))
        as *mut MaridiaSnailData2)
        .offset(0))
    .field_4 as *mut u8)
        .offset((8i32 * v2 as i32) as isize) as *mut VoidP);
    (*E).msl_var_C = *((&raw mut (*(game.g_rom.offset(crate::rtl::lorom_offset(0xa3cd42))
        as *mut MaridiaSnailData2)
        .offset(0))
    .field_6 as *mut u8)
        .offset((8i32 * v2 as i32) as isize) as *mut u16);
    MaridiaSnail_Func_2(k, (8u16).wrapping_mul(v2), game);
    MaridiaSnail_Func_3(k, game);
    (*E).msl_var_05 = 1u16;
    (*E).msl_var_04 = 0u16;
    1u8
}
pub unsafe fn MaridiaSnail_Touch(game: &mut Game) {
    let E: *mut Enemy_MaridiaSnail =
        Get_MaridiaSnail(ram_word!(game, cur_enemy_index) as i32, game);
    if ((*E).msl_var_08 != 1u16
        || (*E).msl_var_F == 0xcf5fu16
        || MaridiaSnail_Func_20(ram_word!(game, cur_enemy_index), game) as i32 & 1i32 == 0)
        && ((*E).msl_var_F == 0xd1b3u16 || ram_word!(game, samus_has_momentum_flag) != 0u16)
    {
        MaridiaSnail_Func_22(ram_word!(game, cur_enemy_index), game);
        if (*E).msl_var_F == 0xd1b3u16 {
            QueueSfx2_Max3(0x70u16, game);
        }
    } else if (*E).msl_var_F != 0xcf5fu16 && (*E).msl_var_08 != 4u16 && (*E).msl_var_08 != 3u16 {
        Enemy_NormalTouchAI_A3(game);
        (*E).msl_parameter_1 = (*E).msl_var_06;
        if (*E).msl_var_08 != 0 {
            MaridiaSnail_Func_19(ram_word!(game, cur_enemy_index), game);
        }
        (*E).msl_var_08 = 0u16;
    }
}
pub unsafe fn MaridiaSnail_Func_20(k: u16, game: &mut Game) -> u8 {
    let r20: u16 = ((*Get_MaridiaSnail(k as i32, game)).msl_var_C) & (1u16);
    if (((ram_word!(game, joypad1_lastkeys)) & (0x300u16)) as i32 >> 8i32) == 1i32 {
        if r20 != 0 {
            return 0u8;
        }
    } else if r20 == 0 {
        return 0u8;
    }
    1u8
}
pub unsafe fn MaridiaSnail_Func_21(k: u16, game: &mut Game) -> u8 {
    if ((*Get_MaridiaSnail(k as i32, game)).msl_var_A as i32 & 0x8000i32) != 0i32 {
        return (*ram_ptr!(game; 0xa1e) == 8u8) as u8;
    }
    (*ram_ptr!(game; 0xa1e) == 4u8) as u8
}
pub unsafe fn MaridiaSnail_Shot(game: &mut Game) {
    let v0: u16 = (ram_word!(
        game,
        projectile_type[ram_word!(game, collision_detection_index)]
    )) & (0xff00u16);
    if v0 == 768u16 || v0 == 1280u16 {
        Enemy_NormalShotAI_A3(game);
    } else {
        let msl_var_08: u16 =
            (*Get_MaridiaSnail(ram_word!(game, cur_enemy_index) as i32, game)).msl_var_08;
        if msl_var_08 != 3u16 && msl_var_08 != 4u16 {
            MaridiaSnail_Func_23(ram_word!(game, cur_enemy_index), game);
        }
        QueueSfx2_Max3(0x70u16, game);
    };
}
pub unsafe fn MaridiaSnail_Func_22(k: u16, game: &mut Game) {
    let E: *mut Enemy_MaridiaSnail = Get_MaridiaSnail(k as i32, game);
    (*E).msl_var_08 = 4u16;
    (*E).msl_var_F = ((0xa3d1b3i32) as u16);
    let v2: i32 = ((4u16).wrapping_mul((*E).msl_var_C)) as i32 >> 1i32;
    (*E).base.current_instruction = crate::rtl::rom_table_word!(game, 0xa3d50f, v2 as isize);
    (*E).msl_var_D = crate::rtl::rom_table_word!(game, 0xa3d50f, (v2 + 1i32) as isize);
    (*E).base.instruction_timer = 1u16;
    (*E).base.timer = 0u16;
    (*E).msl_var_02 = ram_word!(game, absolute_moved_last_frame_x_fract);
    let mut v3: u16 = ram_word!(game, absolute_moved_last_frame_x);
    (*E).msl_var_03 = ram_word!(game, absolute_moved_last_frame_x);
    if v3 >= 0x10u16 {
        v3 = 15u16;
    }
    let v4: i32 = ((4u16).wrapping_mul(v3)) as i32 >> 1i32;
    (*E).msl_var_00 = crate::rtl::rom_table_word!(game, 0xa3d517, v4 as isize);
    (*E).msl_var_01 = crate::rtl::rom_table_word!(game, 0xa3d517, (v4 + 1i32) as isize);
    if (*ram_ptr!(game; 0xa1e) as i32 & 4i32) != 0i32 {
        (*E).msl_var_02 = -((*E).msl_var_02 as i32) as u16;
        (*E).msl_var_03 = -((*E).msl_var_03 as i32) as u16;
    }
}
pub unsafe fn MaridiaSnail_Func_23(k: u16, game: &mut Game) {
    let E: *mut Enemy_MaridiaSnail = Get_MaridiaSnail(k as i32, game);
    (*E).msl_var_08 = 5u16;
    (*E).msl_var_F = ((0xa3d1b3i32) as u16);
    let v2: i32 = ((4u16).wrapping_mul((*E).msl_var_C)) as i32 >> 1i32;
    (*E).base.current_instruction = crate::rtl::rom_table_word!(game, 0xa3d5a4, v2 as isize);
    (*E).msl_var_D = crate::rtl::rom_table_word!(game, 0xa3d5a4, (v2 + 1i32) as isize);
    (*E).base.instruction_timer = 1u16;
    (*E).base.timer = 0u16;
    (*E).msl_var_01 = -1i32 as u16;
    (*E).msl_var_03 = (if *ram_ptr!(game; 0xa1e) == 4u8 {
        -1i32
    } else {
        1i32
    }) as u16;
}
pub unsafe fn Reflec_Func_1(game: &mut Game) {
    if ram_word!(game, door_transition_flag_enemies) == 0 && {
        update_ram_word!(
            game.variables_for_enemy_graphics_drawn_hook[2],
            wrapping_sub(1),
            reread
        ) == 0
    } {
        set_ram_word!(game.variables_for_enemy_graphics_drawn_hook[2] = 16u16);
        let mut v0: u16 = ram_word!(game, variables_for_enemy_graphics_drawn_hook[0]);
        let mut v1: u16 =
            (8u16).wrapping_mul(ram_word!(game, variables_for_enemy_graphics_drawn_hook[1]));
        let mut n: i32 = 4i32;
        loop {
            *(ram_ptr!(game, u16; 0xc000)).offset(((v0 as i32 >> 1i32) + 137i32) as isize) =
                crate::rtl::rom_table_word!(game, 0xa3dabc, (v1 as i32 >> 1i32) as isize);
            v1 = (v1).wrapping_add(2u16);
            v0 = (v0).wrapping_add(2u16);
            n -= 1;
            if (n == 0) {
                break;
            }
        }
        set_ram_word!(
            game.variables_for_enemy_graphics_drawn_hook[1] =
                (*((ram_ptr!(game, u16; 0x1794)).offset(1) as *mut u8).offset(0) as i32 + 1i32
                    & 7i32) as u16
        );
    }
}
pub unsafe fn Reflec_Instr_1(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*Get_Reflec(k as i32, game)).reflec_parameter_2 = *jp.offset(0);
    jp.offset(1)
}
pub unsafe fn Reflec_Init(game: &mut Game) {
    let E: *mut Enemy_Reflec = Get_Reflec(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_BlockPlasmaBeam) as u16);
    (*E).base.current_instruction =
        crate::rtl::rom_table_word!(game, 0xa3dc0b, (*E).reflec_parameter_1 as isize);
    (*(ram_ptr!(game, LongPtr; 0x178c))).addr = ((0xa3db0ci32) as u16);
    *(&raw mut (*(ram_ptr!(game, LongPtr; 0x178c))).bank as *mut u16) = 163u16;
    set_ram_word!(
        game.variables_for_enemy_graphics_drawn_hook[0] =
            ((16i32 * (*E).base.palette_index as i32 & 0xff00i32) >> 8i32) as u16
    );
    set_ram_word!(game.variables_for_enemy_graphics_drawn_hook[2] = 16u16);
}
pub unsafe fn Reflec_Shot(game: &mut Game) {
    let v0: u16 = (2u16).wrapping_mul(ram_word!(game, collision_detection_index));
    let EK: *mut Enemy_Reflec = Get_Reflec(ram_word!(game, cur_enemy_index) as i32, game);
    (*EK).base.invincibility_timer = 10u16;
    let v2: u16 = (32i32 * (*EK).reflec_parameter_2 as i32
        + 2i32 * (ram_word!(game, projectile_dir[(v0 as i32 >> 1i32)]) as i32 & 0xfi32))
        as u16;
    let varE32: u16 = v2;
    let v3: i32 = v2 as i32 >> 1i32;
    if crate::rtl::rom_table_word!(game, 0xa3dcae, v3 as isize) == 0x8000u16 {
        update_ram_word!(
            game.projectile_dir[(v0 as i32 >> 1i32)],
            |fresh16| (fresh16) | (0x10u16)
        );
        printf(b"Possible bug. What is X?\n\0" as *const u8 as *const c_char);
        let ET: *mut Enemy_Reflec = Get_Reflec(v2 as i32, game);
        if (*ET).base.health != 0 {
            Enemy_NormalShotAI_SkipSomeParts_A3(game);
            if (*ET).base.health == 0 {
                (*ET).base.current_instruction =
                    crate::rtl::rom_table_word!(game, 0xa3dca6, (*ET).reflec_parameter_2 as isize);
                (*ET).base.instruction_timer = 1u16;
                (*ET).base.timer = 0u16;
            }
        }
    } else {
        let mut v4: i16 = crate::rtl::rom_table_word!(game, 0xa3dcae, v3 as isize) as i16;
        if v4 < 0i16 {
            v4 = -(crate::rtl::rom_table_word!(game, 0xa3dcae, (varE32 as i32 >> 1i32) as isize)
                as i32) as i16;
        }
        let v5: i32 = v0 as i32 >> 1i32;
        set_ram_word!(game.projectile_dir[v5] = v4 as u16);
        update_ram_word!(
            game.projectile_type[v5],
            |fresh17| (fresh17 as i32 & !(0x8000i32)) as u16
        );
        ProjectileReflection(v0, game);
        QueueSfx2_Max6(0x57u16, game);
    };
}
pub unsafe fn WreckedShipOrangeZoomer_Func_1(
    k: u16,
    jp: *const u16,
    game: &mut Game,
) -> *const u16 {
    (*Get_WreckedShipOrangeZoomer(k as i32, game)).wsozr_var_F = *jp.offset(0);
    jp.offset(1)
}
pub unsafe fn WreckedShipOrangeZoomer_Init(game: &mut Game) {
    let E: *mut Enemy_WreckedShipOrangeZoomer =
        Get_WreckedShipOrangeZoomer(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.current_instruction = crate::rtl::rom_table_word!(
        game,
        0xa3e03b,
        ((*E).base.current_instruction as i32 & 3i32) as isize
    );
    (*E).wsozr_var_F = addr_locret_A3E08A as u16;
    let v1: u16 = crate::rtl::rom_table_word!(game, 0xa3e5f0, (*E).wsozr_parameter_1 as isize);
    (*E).wsozr_var_A = v1;
    (*E).wsozr_var_B = v1;
    if ((*E).base.properties as i32 & 3i32) != 0i32 {
        if ((*E).base.properties as i32 & 3i32) == 2i32 {
            (*E).wsozr_var_B = -((*E).wsozr_var_B as i32) as u16;
        }
    } else {
        (*E).wsozr_var_A = -((*E).wsozr_var_A as i32) as u16;
    };
}
pub unsafe fn WreckedShipOrangeZoomer_Main(game: &mut Game) {
    let E: *mut Enemy_WreckedShipOrangeZoomer =
        Get_WreckedShipOrangeZoomer(ram_word!(game, cur_enemy_index) as i32, game);
    CallEnemyPreInstr((((*E).wsozr_var_F) as u32) | (0xa30000u32), game);
}
pub unsafe fn WreckedShipOrangeZoomer_Func_2(k: u16, game: &mut Game) {
    let mut v7: i16 = 0;
    let mut v9: u16 = 0;
    let mut v12: u16 = 0;
    let E: *mut Enemy_WreckedShipOrangeZoomer = Get_WreckedShipOrangeZoomer(k as i32, game);
    if ram_word!(game, earthquake_timer) == 30u16 && ram_word!(game, earthquake_type) == 20u16 {
        (*E).wsozr_var_03 = (*E).wsozr_var_F;
        (*E).wsozr_var_F = ((0xa3e785i32) as u16);
    }
    if Enemy_MoveRight_ProcessSlopes(k, Shift8AddMagn((*E).wsozr_var_A as i16, 1i32) as i32, game)
        != 0
    {
        (*E).wsozr_var_04 = 0u16;
        EnemyFunc_C8AD(k, game);
        if Enemy_MoveDown(k, ((*E).wsozr_var_B as i16 as i32) << 8i32, game) == 0 {
            let current_block_8: u64;
            if ((ram_word!(game, samus_y_pos)) as i16).wrapping_sub(((*E).base.y_pos) as i16)
                >= 0i16
            {
                v7 = (*E).wsozr_var_B as i16;
                if v7 < 0i16 {
                    current_block_8 = 9790114338316996362;
                } else {
                    current_block_8 = 11050875288958768710;
                }
            } else {
                v7 = (*E).wsozr_var_B as i16;
                if v7 >= 0i16 {
                    current_block_8 = 9790114338316996362;
                } else {
                    current_block_8 = 11050875288958768710;
                }
            }
            if current_block_8 == 9790114338316996362 {
                v7 = -(v7 as i32) as i16;
            }
            (*E).wsozr_var_B = v7 as u16;
            return;
        }
        (*E).wsozr_var_A = -((*E).wsozr_var_A as i32) as u16;
        v9 = addr_kWreckedShipOrangeZoomer_Ilist_E01F as u16;
        if ((*E).wsozr_var_B as i32 & 0x8000i32) != 0i32 {
            v9 = addr_kWreckedShipOrangeZoomer_Ilist_E003 as u16;
        }
        (*E).base.current_instruction = v9;
        (*E).base.instruction_timer = 1u16;
    } else {
        let v11: u16 = ((*E).wsozr_var_04).wrapping_add(1u16);
        (*E).wsozr_var_04 = v11;
        if v11 as i32 - 4i32 & 0x8000i32 != 0 {
            (*E).wsozr_var_B = -((*E).wsozr_var_B as i32) as u16;
            v12 = addr_kWreckedShipOrangeZoomer_Ilist_E01F as u16;
            if ((*E).wsozr_var_B as i32 & 0x8000i32) != 0i32 {
                v12 = addr_kWreckedShipOrangeZoomer_Ilist_E003 as u16;
            }
            (*E).base.current_instruction = v12;
            (*E).base.instruction_timer = 1u16;
        } else {
            (*E).wsozr_var_03 = (*E).wsozr_var_F;
            (*E).wsozr_var_F = ((0xa3e785i32) as u16);
        }
    };
}
pub unsafe fn sub_A3E168(k: u16, game: &mut Game) {
    let mut v7: i16 = 0;
    let mut v9: u16 = 0;
    let mut v12: u16 = 0;
    let E: *mut Enemy_WreckedShipOrangeZoomer = Get_WreckedShipOrangeZoomer(k as i32, game);
    if ram_word!(game, earthquake_timer) == 30u16 && ram_word!(game, earthquake_type) == 20u16 {
        (*E).wsozr_var_03 = (*E).wsozr_var_F;
        (*E).wsozr_var_F = ((0xa3e785i32) as u16);
    }
    if Enemy_MoveDown(k, Shift8AddMagn((*E).wsozr_var_B as i16, 1i32) as i32, game) != 0 {
        (*E).wsozr_var_04 = 0u16;
        if Enemy_MoveRight_ProcessSlopes(k, ((*E).wsozr_var_A as i16 as i32) << 8i32, game) == 0 {
            EnemyFunc_C8AD(k, game);
            if ((ram_word!(game, samus_x_pos)) as i16).wrapping_sub(((*E).base.x_pos) as i16)
                >= 0i16
            {
                v7 = (*E).wsozr_var_A as i16;
                if v7 < 0i16 {
                    v7 = -(v7 as i32) as i16;
                }
            } else {
                v7 = (*E).wsozr_var_A as i16;
                if v7 >= 0i16 {
                    v7 = -(v7 as i32) as i16;
                }
            }
            (*E).wsozr_var_A = v7 as u16;
            return;
        }
        (*E).wsozr_var_B = -((*E).wsozr_var_B as i32) as u16;
        v9 = addr_kWreckedShipOrangeZoomer_Ilist_DFE7 as u16;
        if ((*E).wsozr_var_A as i32 & 0x8000i32) != 0i32 {
            v9 = addr_kWreckedShipOrangeZoomer_Ilist_DFCB as u16;
        }
        (*E).base.current_instruction = v9;
        (*E).base.instruction_timer = 1u16;
    } else {
        let v11: u16 = ((*E).wsozr_var_04).wrapping_add(1u16);
        (*E).wsozr_var_04 = v11;
        if v11 as i32 - 4i32 & 0x8000i32 != 0 {
            (*E).wsozr_var_A = -((*E).wsozr_var_A as i32) as u16;
            v12 = addr_kWreckedShipOrangeZoomer_Ilist_DFE7 as u16;
            if ((*E).wsozr_var_A as i32 & 0x8000i32) != 0i32 {
                v12 = addr_kWreckedShipOrangeZoomer_Ilist_DFCB as u16;
            }
            (*E).base.current_instruction = v12;
            (*E).base.instruction_timer = 1u16;
        } else {
            (*E).wsozr_var_03 = (*E).wsozr_var_F;
            (*E).wsozr_var_F = ((0xa3e785i32) as u16);
        }
    };
}
pub unsafe fn BigEyeBugs_Init(game: &mut Game) {
    let v1: *mut EnemyData = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
    (*v1).current_instruction = crate::rtl::rom_table_word!(
        game,
        0xa3e2cc,
        ((*v1).current_instruction as i32 & 3i32) as isize
    );
    StoneZoomer_E67A(ram_word!(game, cur_enemy_index), game);
}
pub unsafe fn FireZoomer_Init(game: &mut Game) {
    let E: *mut Enemy_FireZoomer = Get_FireZoomer(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.current_instruction = crate::rtl::rom_table_word!(
        game,
        0xa3e2cc,
        ((*E).base.current_instruction as i32 & 3i32) as isize
    );
    StoneZoomer_E67A(ram_word!(game, cur_enemy_index), game);
}
pub unsafe fn Zoomer_Instr_SetPreinstr(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*gEnemyData(k as i32, game)).ai_preinstr = *jp.offset(0);
    jp.offset(1)
}
pub unsafe fn StoneZoomer_Init(game: &mut Game) {
    let E: *mut Enemy_StoneZoomer = Get_StoneZoomer(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.current_instruction = crate::rtl::rom_table_word!(
        game,
        0xa3e2cc,
        ((*E).base.current_instruction as i32 & 3i32) as isize
    );
    StoneZoomer_E67A(ram_word!(game, cur_enemy_index), game);
}
pub unsafe fn StoneZoomer_E67A(k: u16, game: &mut Game) {
    let E: *mut Enemy_StoneZoomer = Get_StoneZoomer(k as i32, game);
    (*E).base.spritemap_pointer = addr_kSpritemap_Nothing_A3 as u16;
    (*E).base.instruction_timer = 1u16;
    (*E).szr_var_F = ((0xa3e6c1i32) as u16);
    if (*E).szr_parameter_1 != 255u16 {
        let v2: u16 = crate::rtl::rom_table_word!(game, 0xa3e5f0, (*E).szr_parameter_1 as isize);
        (*E).szr_var_A = v2;
        (*E).szr_var_B = v2;
    }
    if ((*E).base.properties as i32 & 3i32) != 0i32 {
        if ((*E).base.properties as i32 & 3i32) == 2i32 {
            (*E).szr_var_B = -((*E).szr_var_B as i32) as u16;
        }
    } else {
        (*E).szr_var_A = -((*E).szr_var_A as i32) as u16;
    };
}
pub unsafe fn StoneZoomer_Main(game: &mut Game) {
    let E: *mut Enemy_StoneZoomer = Get_StoneZoomer(ram_word!(game, cur_enemy_index) as i32, game);
    CallEnemyPreInstr((((*E).szr_var_F) as u32) | (0xa30000u32), game);
}
pub unsafe fn FireZoomer_Func_1(k: u16, game: &mut Game) {
    let E: *mut Enemy_FireZoomer = Get_FireZoomer(k as i32, game);
    if ram_word!(game, earthquake_timer) == 30u16 && ram_word!(game, earthquake_type) == 20u16 {
        (*E).fzr_var_03 = (*E).fzr_var_F;
        (*E).fzr_var_F = ((0xa3e785i32) as u16);
    }
    if Enemy_MoveRight_IgnoreSlopes(k, Shift8AddMagn((*E).fzr_var_A as i16, 1i32) as i32, game) != 0
    {
        (*E).fzr_var_04 = 0u16;
        EnemyFunc_C8AD(k, game);
        if Enemy_MoveDown(k, ((*E).fzr_var_B as i16 as i32) << 8i32, game) != 0 {
            (*E).fzr_var_A = -((*E).fzr_var_A as i32) as u16;
            let fzr_parameter_2: u16 = (*E).fzr_parameter_2;
            let mut v7: u16 = 0;
            if ((*E).fzr_var_B as i32 & 0x8000i32) == 0i32 {
                v7 = crate::rtl::rom_table_word!(
                    game,
                    0xa3e63c,
                    (fzr_parameter_2 as i32 >> 1i32) as isize
                );
            } else {
                v7 = crate::rtl::rom_table_word!(
                    game,
                    0xa3e630,
                    (fzr_parameter_2 as i32 >> 1i32) as isize
                );
            }
            (*E).base.current_instruction = v7;
            (*E).base.instruction_timer = 1u16;
        }
    } else {
        let v8: u16 = ((*E).fzr_var_04).wrapping_add(1u16);
        (*E).fzr_var_04 = v8;
        if v8 as i32 - 4i32 & 0x8000i32 != 0 {
            (*E).fzr_var_B = -((*E).fzr_var_B as i32) as u16;
            let v9: u16 = (*E).fzr_parameter_2;
            let mut v10: u16 = 0;
            if ((*E).fzr_var_B as i32 & 0x8000i32) == 0i32 {
                v10 = crate::rtl::rom_table_word!(game, 0xa3e63c, (v9 as i32 >> 1i32) as isize);
            } else {
                v10 = crate::rtl::rom_table_word!(game, 0xa3e630, (v9 as i32 >> 1i32) as isize);
            }
            (*E).base.current_instruction = v10;
            (*E).base.instruction_timer = 1u16;
        } else {
            (*E).fzr_var_03 = (*E).fzr_var_F;
            (*E).fzr_var_F = ((0xa3e785i32) as u16);
        }
    };
}
pub unsafe fn FireZoomer_Func_2(k: u16, game: &mut Game) {
    let E: *mut Enemy_FireZoomer = Get_FireZoomer(k as i32, game);
    if Enemy_MoveDown(
        k,
        (((*E).fzr_var_02 as u32) << 16i32 | (*E).fzr_var_01 as u32) as i32,
        game,
    ) != 0
    {
        if (*E).fzr_parameter_1 == 255u16 {
            (*E).fzr_var_A = 128u16;
            (*E).fzr_var_B = 128u16;
        }
        (*E).fzr_var_01 = 0u16;
        (*E).fzr_var_02 = 0u16;
        (*E).fzr_var_04 = 0u16;
        (*E).fzr_var_F = (*E).fzr_var_03;
    } else {
        if (*E).fzr_var_02 as i32 - 4i32 & 0x8000i32 != 0 {
            AddToHiLo(
                &raw mut (*E).fzr_var_02,
                &raw mut (*E).fzr_var_01,
                0x8000u32,
            );
        }
        if (*E).fzr_var_01 == 0 && (*E).fzr_var_02 == 0 {
            (*E).fzr_var_F = ((0xa3e6c8i32) as u16);
        }
    };
}
pub unsafe fn FireZoomer_Func_3(k: u16, game: &mut Game) {
    let E: *mut Enemy_FireZoomer = Get_FireZoomer(k as i32, game);
    if ram_word!(game, earthquake_timer) == 30u16 && ram_word!(game, earthquake_type) == 20u16 {
        (*E).fzr_var_03 = (*E).fzr_var_F;
        (*E).fzr_var_F = ((0xa3e785i32) as u16);
    }
    if Enemy_MoveDown(k, Shift8AddMagn((*E).fzr_var_B as i16, 1i32) as i32, game) != 0 {
        (*E).fzr_var_04 = 0u16;
        let amt: i32 = FireZoomer_E8A5(k, game);
        if Enemy_MoveRight_IgnoreSlopes(k, amt, game) != 0 {
            (*E).fzr_var_B = -((*E).fzr_var_B as i32) as u16;
            let fzr_parameter_2: u16 = (*E).fzr_parameter_2;
            let mut v6: u16 = 0;
            if ((*E).fzr_var_A as i32 & 0x8000i32) == 0i32 {
                v6 = crate::rtl::rom_table_word!(
                    game,
                    0xa3e654,
                    (fzr_parameter_2 as i32 >> 1i32) as isize
                );
            } else {
                v6 = crate::rtl::rom_table_word!(
                    game,
                    0xa3e648,
                    (fzr_parameter_2 as i32 >> 1i32) as isize
                );
            }
            (*E).base.current_instruction = v6;
            (*E).base.instruction_timer = 1u16;
        } else {
            EnemyFunc_C8AD(k, game);
        }
    } else {
        let v7: u16 = ((*E).fzr_var_04).wrapping_add(1u16);
        (*E).fzr_var_04 = v7;
        if v7 as i32 - 4i32 & 0x8000i32 != 0 {
            (*E).fzr_var_A = -((*E).fzr_var_A as i32) as u16;
            let v8: u16 = (*E).fzr_parameter_2;
            let mut v9: u16 = 0;
            if ((*E).fzr_var_A as i32 & 0x8000i32) == 0i32 {
                v9 = crate::rtl::rom_table_word!(game, 0xa3e654, (v8 as i32 >> 1i32) as isize);
            } else {
                v9 = crate::rtl::rom_table_word!(game, 0xa3e648, (v8 as i32 >> 1i32) as isize);
            }
            (*E).base.current_instruction = v9;
            (*E).base.instruction_timer = 1u16;
        } else {
            (*E).fzr_var_03 = (*E).fzr_var_F;
            (*E).fzr_var_F = ((0xa3e785i32) as u16);
        }
    };
}
pub unsafe fn FireZoomer_E8A5(k: u16, game: &mut Game) -> i32 {
    let E: *mut Enemy_FireZoomer = Get_FireZoomer(ram_word!(game, cur_enemy_index) as i32, game);
    let xpos: u16 = (*E).base.x_pos;
    let ypos: u16 = (if (*E).fzr_var_B as i32 & 0x8000i32 != 0 {
        (*E).base.y_pos as i32 - (*E).base.y_height as i32
    } else {
        (*E).base.y_pos as i32 + (*E).base.y_height as i32 - 1i32
    }) as u16;
    CalculateBlockContainingPixelPos(xpos, ypos, game);
    if (ram_word!(game, level_data[ram_word!(game, cur_block_index)]) as i32 & 0xf000i32) == 4096i32
        && (*ram_ptr!(game; 0x16402, ram_word!(game, cur_block_index) as isize) as i32 & 0x1fi32)
            >= 5i32
    {
        let v1: u16 = crate::rtl::rom_table_word!(
            game,
            0xa3e931,
            ((4i32
                * (*ram_ptr!(game; 0x16402, ram_word!(game, cur_block_index) as isize) as i32
                    & 0x1fi32)) as u16 as i32
                >> 1i32) as isize,
        );
        if ((*E).fzr_var_A as i16) >= 0i16 {
            Multiply16x16((*E).fzr_var_A, v1, game) as i32
        } else {
            -(Multiply16x16(-((*E).fzr_var_A as i32) as u16, v1, game) as i32)
        }
    } else {
        ((*E).fzr_var_A as i16 as i32) << 8i32
    }
}
pub unsafe fn Metroid_Init(game: &mut Game) {
    let E: *mut Enemy_Metroid = Get_Metroid(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.current_instruction = addr_kMetroid_Ilist_E9CF as u16;
    (*E).metroid_var_00 = CreateSpriteAtPos(
        (*E).base.x_pos,
        (*E).base.y_pos,
        50u16,
        ((*E).base.vram_tiles_index) | ((*E).base.palette_index),
        game,
    );
    (*E).metroid_var_01 = CreateSpriteAtPos(
        (*E).base.x_pos,
        (*E).base.y_pos,
        52u16,
        ((*E).base.vram_tiles_index) | ((*E).base.palette_index),
        game,
    );
    (*E).metroid_var_02 = 0u16;
}
pub unsafe fn Metroid_Instr_2(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    QueueSfx2_Max6(0x50u16, game);
    jp
}
pub unsafe fn Metroid_Instr_1(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let v2: i32 = NextRandom(game) as i32 & 7i32;
    QueueSfx2_Max6(
        crate::rtl::rom_table_word!(game, 0xa3ead6, v2 as isize),
        game,
    );
    jp
}
pub unsafe fn Metroid_Frozen(game: &mut Game) {
    Enemy_NormalFrozenAI_A3(game);
    let E: *mut Enemy_Metroid = Get_Metroid(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).metroid_var_E != 0 {
        (*E).metroid_var_E = (*E).metroid_var_E.wrapping_sub(1);
        (*E).base.flash_timer = 2u16;
    }
    let v1: i32 = (*E).metroid_var_00 as i32 >> 1i32;
    set_ram_word!(game.sprite_palettes[v1] = 3072u16);
    set_ram_word!(game.sprite_disable_flag[v1] = 1u16);
    set_ram_word!(game.sprite_instr_list_ptrs[v1] = addr_kSpriteObject_Ilist_C3BA as u16);
    let v2: i32 = (*E).metroid_var_01 as i32 >> 1i32;
    set_ram_word!(game.sprite_palettes[v2] = 3072u16);
    set_ram_word!(game.sprite_disable_flag[v2] = 1u16);
    set_ram_word!(game.sprite_instr_list_ptrs[v2] = addr_kSpriteObject_Ilist_C4B6 as u16);
}
pub unsafe fn Metroid_Hurt(game: &mut Game) {
    let E: *mut Enemy_Metroid = Get_Metroid(ram_word!(game, cur_enemy_index) as i32, game);
    if ((*E).base.flash_timer as i32 & 2i32) != 0i32 {
        let r18: u16 = (*E).base.palette_index;
        let v1: i32 = (*E).metroid_var_00 as i32 >> 1i32;
        set_ram_word!(
            game.sprite_palettes[v1] =
                (r18 as i32 | ram_word!(game, sprite_palettes[v1]) as i32 & 0xf1ffi32) as u16
        );
        let v2: i32 = (*E).metroid_var_01 as i32 >> 1i32;
        set_ram_word!(
            game.sprite_palettes[v2] =
                (r18 as i32 | ram_word!(game, sprite_palettes[v2]) as i32 & 0xf1ffi32) as u16
        );
    } else {
        let v3: i32 = (*E).metroid_var_00 as i32 >> 1i32;
        update_ram_word!(game.sprite_palettes[v3], |fresh10| (fresh10) & (0xf1ffu16));
        let metroid_var_01: u16 = (*E).metroid_var_01;
        update_ram_word!(
            game.sprite_palettes[(metroid_var_01 as i32 >> 1i32)],
            |fresh11| (fresh11) & (0xf1ffu16)
        );
    };
}
static off_A3EC09: [Option<Func_Y_V>; 4] = unsafe {
    [
        Some(Metroid_Func_1 as unsafe fn(u16, &mut Game) -> ()),
        Some(Metroid_Func_2 as unsafe fn(u16, &mut Game) -> ()),
        Some(Metroid_Func_3 as unsafe fn(u16, &mut Game) -> ()),
        Some(Metroid_Func_4 as unsafe fn(u16, &mut Game) -> ()),
    ]
};
pub unsafe fn Metroid_Main(game: &mut Game) {
    let E: *mut Enemy_Metroid = Get_Metroid(ram_word!(game, cur_enemy_index) as i32, game);
    off_A3EC09[(*E).metroid_var_F as usize].expect("non-null function pointer")(
        (ram_word!(game, samus_y_pos)).wrapping_sub(8u16),
        game,
    );
    let r24: u16 = ((*E).base.vram_tiles_index) | ((*E).base.palette_index);
    let v2: i32 = (*E).metroid_var_00 as i32 >> 1i32;
    set_ram_word!(game.sprite_x_pos[v2] = (*E).base.x_pos);
    set_ram_word!(game.sprite_y_pos[v2] = (*E).base.y_pos);
    set_ram_word!(game.sprite_palettes[v2] = r24);
    set_ram_word!(game.sprite_disable_flag[v2] = 0u16);
    let v4: i32 = (*E).metroid_var_01 as i32 >> 1i32;
    set_ram_word!(game.sprite_x_pos[v4] = (*E).base.x_pos);
    set_ram_word!(game.sprite_y_pos[v4] = (*E).base.y_pos);
    set_ram_word!(game.sprite_palettes[v4] = r24);
    set_ram_word!(game.sprite_disable_flag[v4] = 0u16);
}
pub unsafe fn Metroid_Func_1(varE32: u16, game: &mut Game) {
    let mut current_block: u64;
    let mut v3: i16 = 0;
    let mut v4: i16 = 0;
    let mut v8: i16 = 0;
    let mut v9: i16 = 0;
    let E: *mut Enemy_Metroid = Get_Metroid(ram_word!(game, cur_enemy_index) as i32, game);
    let mut t: i32 = ((((((*E).base.y_pos) as i16).wrapping_sub((varE32) as i16)) as i32 >> 2i32)
        as i16 as i32)
        << 8i32;
    let mut r18: u16 = t as u16;
    let mut r20: u16 = (t >> 16i32) as u16;
    let metroid_var_C: u16 = (*E).metroid_var_C;
    let mut v2: bool = metroid_var_C < r18;
    (*E).metroid_var_C = (metroid_var_C).wrapping_sub(r18);
    v3 = ((*E).metroid_var_D as i32 - (v2 as i32 + r20 as i32)) as i16;
    (*E).metroid_var_D = v3 as u16;
    if v3 < 0i16 {
        if (v3 as u16) >= 0xfffdu16 {
            current_block = 3810685344454692134;
        } else {
            v4 = -3i16;
            current_block = 2868539653012386629;
        }
    } else if (v3 as u16) < 3u16 {
        current_block = 3810685344454692134;
    } else {
        v4 = 3i16;
        current_block = 2868539653012386629;
    }
    if current_block == 2868539653012386629 {
        (*E).metroid_var_D = v4 as u16;
        (*E).metroid_var_C = 0u16;
    }
    if Enemy_MoveDown(
        ram_word!(game, cur_enemy_index),
        (((*E).metroid_var_D as u32) << 16i32 | (*E).metroid_var_C as u32) as i32,
        game,
    ) != 0
    {
        (*E).metroid_var_C = 0u16;
        (*E).metroid_var_D = 0u16;
    }
    t = ((((((*E).base.x_pos) as i16).wrapping_sub((ram_word!(game, samus_x_pos)) as i16)) as i32
        >> 2i32) as i16 as i32)
        << 8i32;
    r18 = t as u16;
    r20 = (t >> 16i32) as u16;
    let metroid_var_A: u16 = (*E).metroid_var_A;
    v2 = metroid_var_A < r18;
    (*E).metroid_var_A = (metroid_var_A).wrapping_sub(r18);
    v8 = ((*E).metroid_var_B as i32 - (v2 as i32 + r20 as i32)) as i16;
    (*E).metroid_var_B = v8 as u16;
    if v8 < 0i16 {
        if (v8 as u16) >= 0xfffdu16 {
            current_block = 8759308886461481507;
        } else {
            v9 = -3i16;
            current_block = 17478428563724192186;
        }
    } else if (v8 as u16) < 3u16 {
        current_block = 8759308886461481507;
    } else {
        v9 = 3i16;
        current_block = 17478428563724192186;
    }
    if current_block == 17478428563724192186 {
        (*E).metroid_var_B = v9 as u16;
        (*E).metroid_var_A = 0u16;
    }
    if Enemy_MoveRight_IgnoreSlopes(
        ram_word!(game, cur_enemy_index),
        (((*E).metroid_var_B as u32) << 16i32 | (*E).metroid_var_A as u32) as i32,
        game,
    ) != 0
    {
        (*E).metroid_var_A = 0u16;
        (*E).metroid_var_B = 0u16;
    }
}
pub unsafe fn Metroid_Func_2(varE32: u16, game: &mut Game) {
    let mut current_block: u64;
    let mut v1: i16 = 0;
    let mut v5: i16 = 0;
    let mut v2: u16 = 0;
    let mut v6: u16 = 0;
    let mut r18: u16 = 0u16;
    let mut r20: u16 = 0u16;
    let E: *mut Enemy_Metroid = Get_Metroid(ram_word!(game, cur_enemy_index) as i32, game);
    *(&raw mut v1 as *mut u8).offset(0) =
        (((varE32).wrapping_sub((*E).base.y_pos)) as i32 >> 8i32) as u8;
    *(&raw mut v1 as *mut u8).offset(1isize) =
        ((varE32) as u8).wrapping_sub(*(&raw mut (*E).base.y_pos as *mut u8).offset(0));
    r20 = ((((v1) as u16) & (0xff00u16)) as i32 >> 3i32) as u16;
    if (r20 as i32 & 0x1000i32) != 0i32 {
        r20 = (r20) | (0xe000u16);
    }
    if (r20 as i32 & 0x8000i32) != 0i32 {
        if r20 >= 0xfffdu16 {
            current_block = 12681381661298955615;
        } else {
            v2 = -3i32 as u16;
            current_block = 2868539653012386629;
        }
    } else if r20 < 3u16 {
        current_block = 12681381661298955615;
    } else {
        v2 = 3u16;
        current_block = 2868539653012386629;
    }
    if current_block == 2868539653012386629 {
        r20 = v2;
        r18 = 0u16;
    }
    (*E).metroid_var_C = r18;
    (*E).metroid_var_D = r20;
    if Enemy_MoveDown(
        ram_word!(game, cur_enemy_index),
        ((r20 as u32) << 16i32 | r18 as u32) as i32,
        game,
    ) != 0
    {
        (*E).metroid_var_C = 0u16;
        (*E).metroid_var_D = 0u16;
    }
    r18 = 0u16;
    r20 = 0u16;
    *(&raw mut v5 as *mut u8).offset(0) =
        (((ram_word!(game, samus_x_pos)).wrapping_sub((*E).base.x_pos)) as i32 >> 8i32) as u8;
    *(&raw mut v5 as *mut u8).offset(1isize) = ((ram_word!(game, samus_x_pos)) as u8)
        .wrapping_sub(*(&raw mut (*E).base.x_pos as *mut u8).offset(0));
    r20 = ((((v5) as u16) & (0xff00u16)) as i32 >> 3i32) as u16;
    if (r20 as i32 & 0x1000i32) != 0i32 {
        r20 = (r20) | (0xe000u16);
    }
    if (r20 as i32 & 0x8000i32) != 0i32 {
        if r20 >= 0xfffdu16 {
            current_block = 8114601772576486457;
        } else {
            v6 = -3i32 as u16;
            current_block = 18386322304582297246;
        }
    } else if r20 < 3u16 {
        current_block = 8114601772576486457;
    } else {
        v6 = 3u16;
        current_block = 18386322304582297246;
    }
    if current_block == 18386322304582297246 {
        r20 = v6;
        r18 = 0u16;
    }
    (*E).metroid_var_A = r18;
    (*E).metroid_var_B = r20;
    if Enemy_MoveRight_IgnoreSlopes(
        ram_word!(game, cur_enemy_index),
        ((r20 as u32) << 16i32 | r18 as u32) as i32,
        game,
    ) != 0
    {
        (*E).metroid_var_A = 0u16;
        (*E).metroid_var_B = 0u16;
    }
}
pub unsafe fn Metroid_Func_3(varE32: u16, game: &mut Game) {
    let E: *mut Enemy_Metroid = Get_Metroid(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.x_pos = ram_word!(game, samus_x_pos);
    (*E).base.y_pos = varE32;
    (*E).metroid_var_A = 0u16;
    (*E).metroid_var_B = 0u16;
    (*E).metroid_var_C = 0u16;
    (*E).metroid_var_D = 0u16;
}
pub unsafe fn Metroid_Func_4(varE32: u16, game: &mut Game) {
    let E: *mut Enemy_Metroid = Get_Metroid(ram_word!(game, cur_enemy_index) as i32, game);
    let v1: i32 = (*E).metroid_var_E as i32 & 3i32;
    (*E).base.x_pos =
        ((*E).base.x_pos).wrapping_add(crate::rtl::rom_table_word!(game, 0xa3ea3f, v1 as isize));
    (*E).base.y_pos = ((*E).base.y_pos).wrapping_add(crate::rtl::rom_table_word!(
        game,
        0xa3ea3f,
        (v1 + 4i32) as isize
    ));
    (*E).metroid_var_A = 0u16;
    (*E).metroid_var_B = 0u16;
    (*E).metroid_var_C = 0u16;
    (*E).metroid_var_D = 0u16;
    let fresh9 = (*E).metroid_var_E;
    (*E).metroid_var_E = (*E).metroid_var_E.wrapping_sub(1);
    if fresh9 == 1u16 {
        (*E).metroid_var_F = 0u16;
        (*E).base.current_instruction = addr_kMetroid_Ilist_E9CF as u16;
        (*E).base.instruction_timer = 1u16;
    }
}
pub unsafe fn Metroid_Touch(game: &mut Game) {
    let E: *mut Enemy_Metroid = Get_Metroid(ram_word!(game, cur_enemy_index) as i32, game);
    let varE32: u16 = (ram_word!(game, samus_y_pos)).wrapping_sub(8u16);
    if ram_word!(game, samus_contact_damage_index) != 0 {
        if (*E).metroid_var_F != 2u16 {
            (*E).metroid_var_A = 0u16;
            (*E).metroid_var_C = 0u16;
            let v2: i16 =
                (((*E).base.x_pos) as i16).wrapping_sub((ram_word!(game, samus_x_pos)) as i16);
            (*E).metroid_var_B = (if v2 < 0i16 { -256i32 } else { 0i32 }) as u16;
            *((&raw mut (*E).metroid_var_A as *mut u8).offset(1) as *mut u16) =
                ((v2 as i32) << 6i32) as u16;
            let v4: i16 = (((*E).base.y_pos) as i16).wrapping_sub((varE32) as i16);
            (*E).metroid_var_D = (if v4 < 0i16 { -256i32 } else { 0i32 }) as u16;
            *((&raw mut (*E).metroid_var_C as *mut u8).offset(1) as *mut u16) =
                ((v4 as i32) << 6i32) as u16;
            (*E).metroid_var_F = 0u16;
            (*E).base.current_instruction = addr_kMetroid_Ilist_E9CF as u16;
            (*E).base.instruction_timer = 1u16;
        }
    } else {
        let v5: u16 = ram_word!(game, cur_enemy_index);
        if (*E).metroid_var_F != 3u16 {
            if (ram_word!(game, random_enemy_counter) as i32 & 7i32) == 7i32
                && ram_word!(game, samus_health) as i32 - 30i32 & 0x8000i32 == 0
            {
                QueueSfx3_Max6(0x2du16, game);
            }
            Metroid_Func_5(v5, game);
        }
        if (*E).metroid_var_F < 2u16 {
            let mut v7: u16 = 1u16;
            if abs16(((*E).base.x_pos).wrapping_sub(ram_word!(game, samus_x_pos))) < 8u16
                && abs16(((*E).base.y_pos).wrapping_sub(varE32)) < 8u16
            {
                v7 = 2u16;
                CallSomeSamusCode(0x12u16, game);
            }
            (*E).metroid_var_F = v7;
            if v7 == 2u16 {
                (*E).base.current_instruction = addr_kMetroid_Ilist_EA25 as u16;
                (*E).base.instruction_timer = 1u16;
            }
        }
    };
}
pub unsafe fn Metroid_Func_5(k: u16, game: &mut Game) {
    let mut v1: u16 = 0;
    let varE32: u16 = (ram_word!(game, samus_y_pos)).wrapping_sub(8u16);
    if (ram_word!(game, equipped_items) as i32 & 0x20i32) != 0i32 {
        v1 = 12288u16;
    } else if ram_word!(game, equipped_items) as i32 & 1i32 != 0 {
        v1 = 24576u16;
    } else {
        v1 = -16384i32 as u16;
    }
    let r18: u16 = v1;
    let E: *mut Enemy_Metroid = Get_Metroid(k as i32, game);
    let metroid_var_02: u16 = (*E).metroid_var_02;
    let v4: bool = metroid_var_02 < r18;
    (*E).metroid_var_02 = (metroid_var_02).wrapping_sub(r18);
    if v4 {
        Samus_DealDamage(1u16, game);
    }
}
pub unsafe fn Metroid_Shot(game: &mut Game) {
    let v0: u16 = (2u16).wrapping_mul(ram_word!(game, collision_detection_index));
    let E: *mut Enemy_Metroid = Get_Metroid(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).base.frozen_timer != 0 {
        let v3: u16 = (ram_word!(game, projectile_type[(v0 as i32 >> 1i32)])) & (0xf00u16);
        if v3 == 256u16 || v3 == 512u16 {
            set_ram_word!(game.special_death_item_drop_x_origin_pos = (*E).base.x_pos);
            set_ram_word!(game.special_death_item_drop_y_origin_pos = (*E).base.y_pos);
            Enemy_NormalShotAI_SkipSomeParts_A3(game);
            if (*E).base.health == 0 {
                (*E).metroid_var_B = 0u16;
                EnemyDeathAnimation(ram_word!(game, cur_enemy_index), 4u16, game);
                CallSomeSamusCode(0x13u16, game);
                *(ram_ptr!(game, u16; 0xef78))
                    .offset(((*E).metroid_var_00 as i32 >> 1i32) as isize) = 0u16;
                let metroid_var_01: u16 = (*E).metroid_var_01;
                *(ram_ptr!(game, u16; 0xef78)).offset((metroid_var_01 as i32 >> 1i32) as isize) =
                    0u16;
                Enemy_ItemDrop_Metroid(metroid_var_01, game);
            }
        }
    } else if (*E).metroid_var_F == 2u16 {
        if (ram_word!(game, projectile_type[(v0 as i32 >> 1i32)]) as i32 & 0xf00i32) == 1280i32 {
            (*E).metroid_var_E = 4u16;
            (*E).metroid_var_F = 3u16;
            (*E).base.current_instruction = addr_kMetroid_Ilist_E9CF as u16;
            (*E).base.instruction_timer = 1u16;
            CallSomeSamusCode(0x13u16, game);
        }
    } else {
        (*E).metroid_var_A = 0u16;
        (*E).metroid_var_C = 0u16;
        let v7: i16 =
            (((*E).base.x_pos) as i16).wrapping_sub((ram_word!(game, projectile_x_pos[0])) as i16);
        (*E).metroid_var_B = (if v7 < 0i16 { -256i32 } else { 0i32 }) as u16;
        *((&raw mut (*E).metroid_var_A as *mut u8).offset(1) as *mut u16) =
            (32u16).wrapping_mul((v7) as u16);
        let v9: i16 =
            (((*E).base.y_pos) as i16).wrapping_sub((ram_word!(game, projectile_y_pos[0])) as i16);
        (*E).metroid_var_D = (if v9 < 0i16 { -256i32 } else { 0i32 }) as u16;
        *((&raw mut (*E).metroid_var_C as *mut u8).offset(1) as *mut u16) =
            (32u16).wrapping_mul((v9) as u16);
        (*E).metroid_var_F = 0u16;
        (*E).base.current_instruction = addr_kMetroid_Ilist_E9CF as u16;
        (*E).base.instruction_timer = 1u16;
        let v10: i32 = ram_word!(game, collision_detection_index) as i32;
        if (ram_word!(game, projectile_type[v10]) as i32 & 2i32) != 0i32 {
            QueueSfx3_Max6(0xau16, game);
            let r18: u16 = ram_word!(game, projectile_damage[v10]);
            (*E).base.flash_timer = 4u16;
            let metroid_parameter_2: u16 = (*E).metroid_parameter_2;
            let mut v13: bool = metroid_parameter_2 < r18;
            let v12: u16 = (metroid_parameter_2).wrapping_sub(r18);
            v13 = !v13;
            if v12 == 0 || !v13 {
                (*E).metroid_parameter_2 = 0u16;
                (*E).base.frozen_timer = 400u16;
                (*E).base.ai_handler_bits = ((*E).base.ai_handler_bits) | (4u16);
                return;
            }
            (*E).metroid_parameter_2 = v12;
        }
        QueueSfx2_Max6(0x5au16, game);
    };
}
pub unsafe fn Metroid_Powerbomb(k: u16, game: &mut Game) {
    NormalEnemyPowerBombAi(game);
    if (*Get_Metroid(k as i32, game)).base.health == 0 {
        CallSomeSamusCode(0x13u16, game);
        *(ram_ptr!(game, u16; 0xef78)).offset(
            ((*Get_Metroid(ram_word!(game, cur_enemy_index) as i32, game)).metroid_var_00 as i32
                >> 1i32) as isize,
        ) = 0u16;
        *(ram_ptr!(game, u16; 0xef78)).offset(
            ((*Get_Metroid(ram_word!(game, cur_enemy_index) as i32, game)).metroid_var_01 as i32
                >> 1i32) as isize,
        ) = 0u16;
    }
}
