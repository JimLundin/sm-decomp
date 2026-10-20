use crate::game::callback_tables::callback_table;
pub use crate::game::constants::bank_86::*;
pub use crate::game::constants::bank_8d::*;
pub use crate::game::constants::buttons::*;
pub use crate::game::constants::enemy_properties::*;
pub use crate::game::constants::game_states::*;
pub use crate::game::constants::loading_game_states::*;
pub use crate::game::constants::samus_poses::*;
use crate::game::enemy_instruction;
use crate::game::enemy_layouts::enemy_layout;
use crate::game::sm_80::{CheckEventHappened, NextRandom, QueueSfx2_Max6, QueueSfx3_Max6};
use crate::game::sm_81::SaveToSram;
use crate::game::sm_85::DisplayMessageBox_Poll;
use crate::game::sm_86::{
    Math_MultByCos, Math_MultBySin, SpawnEprojWithGfx, SpawnEprojWithRoomGfx,
};
use crate::game::sm_8d::SpawnPalfxObject;
use crate::game::sm_90::CallSomeSamusCode;
use crate::game::sm_91::{
    MakeSamusFaceForward, Samus_RestoreHealth, Samus_RestoreMissiles, Samus_RestorePowerBombs,
    Samus_RestoreSuperMissiles,
};
use crate::game::sm_94::kSinCosTable8bit_Sext;
use crate::game::sm_a0::{
    CalculateAngleFromXY, CalculateAngleOfSamusFromEnemy, CallEnemyPreInstr,
    CheckIfEnemyIsOnScreen, CheckIfEnemyTouchesSamus, CompareDistToSamus_X, ConvertAngleToXy,
    DeleteEnemyAndConnectedEnemies, EnemyFunc_AC67, EnemyFunc_BC76, EnemyGrappleDeath,
    Enemy_MoveDown, Enemy_MoveRight_IgnoreSlopes, Enemy_MoveRight_SlopesAsWalls,
    Enemy_SwitchToFrozenAi, GetSamusEnemyDelta_X, IsSamusWithinEnemy_X, IsSamusWithinEnemy_Y,
    NormalEnemyFrozenAI, NormalEnemyPowerBombAi, NormalEnemyPowerBombAiSkipDeathAnim_CurEnemy,
    NormalEnemyShotAi, NormalEnemyShotAiSkipDeathAnim_CurEnemy, NormalEnemyTouchAi,
    NormalEnemyTouchAiSkipDeathAnim_CurEnemy, RinkasDeathAnimation, SamusLatchesOnWithGrapple,
};
use crate::game::sm_a9::MoveEnemyWithVelocity;
use crate::game_state::Game;
use crate::ram_map::ram_ptr;
use crate::ram_map::set_ram_words;
use crate::ram_map::{ram_word, set_ram_word, update_ram_word};
use crate::rtl::abs16;
use crate::rtl::{AddToHiLo, SetHiLo};
use crate::rtl::{Mult8x8, RomPtr, Unreachable};
use crate::set_fields;
use ::core::ptr::null_mut;

pub use crate::types::Buttons;
pub use crate::types::Consts_86;
pub use crate::types::Consts_8D;
pub use crate::types::Consts_A2;
pub use crate::types::GameState;
pub use crate::types::LoadingGameState;
pub use crate::types::{
    EnemyBase, EnemyData, Func_V, LongPtr, Point16U, SamusPose, VoidP, VramWriteEntry,
};
pub use crate::types::{EnemyProps, EnemySpawnData};
crate::game::constants::typed_constants! { Consts_A2;
    addr_kRisingFallingPlatform_Ilist_EDE7 = 60903, addr_kTimedShutter_Ilist_E9D4 = 59860,
    addr_kTimedShutter_Ilist_E9AA = 59818, addr_kTimedShutter_Ilist_E9A4 = 59812,
    addr_kTimedShutter_Ilist_E99E = 59806, addr_kTimedShutter_Ilist_E998 = 59800,
    addr_kLavaSeahorse_Ilist_E5A1 = 58785, addr_kLavaSeahorse_Ilist_E59B = 58779,
    addr_kRipper_Ilist_E48B = 58507, addr_kRipper_Ilist_E477 = 58487,
    addr_kGripper_Sprmap_E44B = 58443, addr_kGripper_Sprmap_E43F = 58431,
    addr_kRipper_Ilist_E2F4 = 58100, addr_kRipper_Ilist_E2E0 = 58080,
    addr_kGripper_Ilist_E1AF = 57775, addr_kGripper_Ilist_E19B = 57755,
    addr_kHirisingSlowfalling_Ilist_D840 = 55360, addr_kHirisingSlowfalling_Ilist_D834 = 55348,
    addr_kHirisingSlowfalling_Ilist_D82C = 55340, addr_kMaridiaLargeSnail_Ilist_CA4B = 51787,
    addr_kLowerNorfairRio_Ilist_C6B0 = 50864, addr_kLowerNorfairRio_Ilist_C686 = 50822,
    addr_kLowerNorfairRio_Ilist_C674 = 50804, addr_kLowerNorfairRio_Ilist_C662 = 50786,
    addr_kLowerNorfairRio_Ilist_C65A = 50778, addr_kLowerNorfairRio_Ilist_C630 = 50736,
    addr_kLowerNorfairRio_Ilist_C61A = 50714, addr_kNorfairRio_Ilist_C1A3 = 49571,
    addr_kNorfairRio_Ilist_C18F = 49551, addr_kNorfairRio_Ilist_C179 = 49529,
    addr_kNorfairRio_Ilist_C145 = 49477, addr_kNorfairRio_Ilist_C12F = 49455,
    addr_kNorfairRio_Ilist_C107 = 49415, addr_kNorfairRio_Ilist_C0F1 = 49393,
    addr_kNorfairLavajumpingEnemy_Ilist_BE62 = 48738, addr_kNorfairLavajumpingEnemy_Ilist_BE42 = 48706,
    addr_kNorfairLavajumpingEnemy_Ilist_BE3C = 48700, addr_kRio_Ilist_BBA3 = 48035,
    addr_kRio_Ilist_BB97 = 48023, addr_kRio_Ilist_BB7F = 47999,
    addr_stru_A2BB53 = 47955, addr_kRio_Ilist_BB4B = 47947,
    addr_kRinka_Ilist_BA0C = 47628, addr_kRinka_Ilist_B9E0 = 47584,
    addr_kNorfairErraticFireball_Ilist_B51A = 46362, addr_kNorfairErraticFireball_Ilist_B2DC = 45788,
    addr_kFlies_Ilist_B013 = 45075, addr_kGunshipTop_Ilist_A61C = 42524,
    addr_kGunshipTop_Ilist_A616 = 42518, addr_kGunshipTop_Ilist_A60E = 42510,
    addr_kGunshipTop_Ilist_A5EE = 42478, addr_kGunshipTop_Ilist_A5BE = 42430,
    addr_kMaridiaSpikeyShell_Ilist_A3BD = 41917, addr_kMaridiaSpikeyShell_Ilist_A3AB = 41899,
    addr_kSpikeShootingPlant_Ilist_9F00 = 40704, addr_kSpikeShootingPlant_Ilist_9EDA = 40666,
    addr_kSpikeShootingPlant_Ilist_9EB0 = 40624, addr_kSpikeShootingPlant_Ilist_9E8A = 40586,
    addr_kThinHoppingBlobs_Ilist_9A01 = 39425, addr_kThinHoppingBlobs_Ilist_99FB = 39419,
    addr_kThinHoppingBlobs_Ilist_99F5 = 39413, addr_kThinHoppingBlobs_Ilist_99EF = 39407,
    addr_kThinHoppingBlobs_Ilist_99E9 = 39401, addr_kThinHoppingBlobs_Ilist_99D5 = 39381,
    addr_kThinHoppingBlobs_Ilist_99C1 = 39361, addr_kThinHoppingBlobs_Ilist_99AD = 39341,
    addr_locret_A28E09 = 36361, addr_kMaridiaBeybladeTurtle_Ilist_8D40 = 36160,
    addr_kMaridiaBeybladeTurtle_Ilist_8D28 = 36136, addr_kMaridiaBeybladeTurtle_Ilist_8D14 = 36116,
    addr_kMaridiaBeybladeTurtle_Ilist_8D00 = 36096, addr_kMaridiaBeybladeTurtle_Ilist_8C72 = 35954,
    addr_kMaridiaBeybladeTurtle_Ilist_8C62 = 35938, addr_kMaridiaBeybladeTurtle_Ilist_8C4A = 35914,
    addr_kMaridiaBeybladeTurtle_Ilist_8C44 = 35908, addr_kMaridiaBeybladeTurtle_Ilist_8C30 = 35888,
    addr_kMaridiaBeybladeTurtle_Ilist_8C1C = 35868, addr_kMaridiaBeybladeTurtle_Ilist_8C02 = 35842,
    addr_kMaridiaBeybladeTurtle_Ilist_8BD2 = 35794, addr_kMaridiaBeybladeTurtle_Ilist_8B80 = 35712,
    addr_kMiniCrocomire_Ilist_896E = 35182, addr_kMiniCrocomire_Ilist_8958 = 35160,
    addr_kMiniCrocomire_Ilist_8948 = 35144, addr_kMiniCrocomire_Ilist_8932 = 35122,
    addr_kBouncingGoofball_Ilist_86BF = 34495, addr_kBouncingGoofball_Ilist_86A7 = 34471,
    addr_kSpritemap_Nothing_A2 = 32845,
}
pub use crate::types::Point32;
enemy_layout! {
    Enemy_BouncingGoofball {
        base: EnemyBase,
        words: [
            bgl_var_A, bgl_var_B, bgl_var_C, bgl_var_D, bgl_var_E, bgl_var_F,
            bgl_parameter_1, bgl_parameter_2,
        ],
        pad1: [u8; 26696],
        words: [
            bgl_var_00, bgl_var_01, bgl_var_02, bgl_var_03, bgl_var_04, bgl_var_05,
            bgl_var_06, bgl_var_07, bgl_var_08, bgl_var_09, bgl_var_0A, bgl_var_0B,
            bgl_var_0C, bgl_var_0D, bgl_var_0E, bgl_var_0F, bgl_var_10, bgl_var_11,
            bgl_var_12, bgl_var_13, bgl_var_14, bgl_var_15, bgl_var_16, bgl_var_17,
            bgl_var_18, bgl_var_19, bgl_var_1A, bgl_var_1B, bgl_var_1C, bgl_var_1D,
            bgl_var_1E, bgl_var_1F,
        ],
    }
}

enemy_layout! {
    Enemy_Rio {
        base: EnemyBase,
        words: [
            rio_var_A, rio_var_B, rio_var_C, rio_var_D, rio_var_E, rio_var_F,
            rio_parameter_1, rio_parameter_2,
        ],
    }
}

enemy_layout! {
    Enemy_Flies {
        base: EnemyBase,
        words: [
            flies_var_A, flies_var_B, flies_var_C, flies_var_D, flies_var_E, flies_var_F,
            flies_parameter_1, flies_parameter_2,
        ],
    }
}

enemy_layout! {
    Enemy_LavaSeahorse {
        base: EnemyBase,
        words: [
            lse_var_A, lse_var_B, lse_var_C, lse_var_D, lse_var_E, lse_var_F,
            lse_parameter_1, lse_parameter_2,
        ],
        pad1: [u8; 26696],
        words: [
            lse_var_00, lse_var_01, lse_var_02, lse_var_03, lse_var_04, lse_var_05,
            lse_var_06, lse_var_07, lse_var_08, lse_var_09, lse_var_0A, lse_var_0B,
            lse_var_0C, lse_var_0D, lse_var_0E, lse_var_0F, lse_var_10, lse_var_11,
            lse_var_12, lse_var_13, lse_var_14, lse_var_15, lse_var_16, lse_var_17,
            lse_var_18, lse_var_19, lse_var_1A, lse_var_1B, lse_var_1C, lse_var_1D,
            lse_var_1E, lse_var_1F,
        ],
    }
}

enemy_layout! {
    Enemy_NorfairRio {
        base: EnemyBase,
        words: [
            nro_var_A, nro_var_B, nro_var_C, nro_var_D, nro_var_E, nro_var_F,
            nro_parameter_1, nro_parameter_2,
        ],
        pad1: [u8; 26696],
        words: [
            nro_var_00, nro_var_01, nro_var_02, nro_var_03, nro_var_04, nro_var_05,
            nro_var_06, nro_var_07, nro_var_08, nro_var_09, nro_var_0A, nro_var_0B,
            nro_var_0C, nro_var_0D, nro_var_0E, nro_var_0F, nro_var_10, nro_var_11,
            nro_var_12, nro_var_13, nro_var_14, nro_var_15, nro_var_16, nro_var_17,
            nro_var_18, nro_var_19, nro_var_1A, nro_var_1B, nro_var_1C, nro_var_1D,
            nro_var_1E, nro_var_1F,
        ],
    }
}

enemy_layout! {
    Enemy_MaridiaBeybladeTurtle {
        base: EnemyBase,
        words: [
            mbte_var_A, mbte_var_B, mbte_var_C, mbte_var_D, mbte_var_E, mbte_var_F,
            mbte_parameter_1, mbte_parameter_2,
        ],
    }
}

enemy_layout! {
    Enemy_MaridiaTurtle {
        base: EnemyBase,
        words: [
            mte_var_A, mte_var_B, mte_var_C, mte_var_D, mte_var_E, mte_var_F,
            mte_parameter_1, mte_parameter_2,
        ],
        pad1: [u8; 26696],
        words: [
            mte_var_00, mte_var_01, mte_var_02, mte_var_03, mte_var_04, mte_var_05,
            mte_var_06, mte_var_07, mte_var_08, mte_var_09, mte_var_0A, mte_var_0B,
            mte_var_0C, mte_var_0D, mte_var_0E, mte_var_0F, mte_var_10, mte_var_11,
            mte_var_12, mte_var_13, mte_var_14, mte_var_15, mte_var_16, mte_var_17,
            mte_var_18, mte_var_19, mte_var_1A, mte_var_1B, mte_var_1C, mte_var_1D,
            mte_var_1E, mte_var_1F,
        ],
    }
}

enemy_layout! {
    Enemy_MaridiaLargeSnail {
        base: EnemyBase,
        words: [
            mlsl_var_A, mlsl_var_B, mlsl_var_C, mlsl_var_D, mlsl_var_E, mlsl_var_F,
            mlsl_parameter_1, mlsl_parameter_2,
        ],
        pad1: [u8; 26696],
        words: [
            mlsl_var_00, mlsl_var_01, mlsl_var_02, mlsl_var_03, mlsl_var_04, mlsl_var_05,
            mlsl_var_06, mlsl_var_07, mlsl_var_08, mlsl_var_09, mlsl_var_0A, mlsl_var_0B,
            mlsl_var_0C, mlsl_var_0D, mlsl_var_0E, mlsl_var_0F, mlsl_var_10, mlsl_var_11,
            mlsl_var_12, mlsl_var_13, mlsl_var_14, mlsl_var_15, mlsl_var_16, mlsl_var_17,
            mlsl_var_18, mlsl_var_19, mlsl_var_1A, mlsl_var_1B, mlsl_var_1C, mlsl_var_1D,
            mlsl_var_1E, mlsl_var_1F,
        ],
    }
}

enemy_layout! {
    Enemy_MaridiaSpikeyShell {
        base: EnemyBase,
        words: [
            mssl_var_A, mssl_var_B, mssl_var_C, mssl_var_D, mssl_var_E, mssl_var_F,
            mssl_parameter_1, mssl_parameter_2,
        ],
        pad1: [u8; 26696],
        words: [
            mssl_var_00, mssl_var_01, mssl_var_02, mssl_var_03, mssl_var_04, mssl_var_05,
            mssl_var_06, mssl_var_07, mssl_var_08, mssl_var_09, mssl_var_0A, mssl_var_0B,
            mssl_var_0C, mssl_var_0D, mssl_var_0E, mssl_var_0F, mssl_var_10, mssl_var_11,
            mssl_var_12, mssl_var_13, mssl_var_14, mssl_var_15, mssl_var_16, mssl_var_17,
            mssl_var_18, mssl_var_19, mssl_var_1A, mssl_var_1B, mssl_var_1C, mssl_var_1D,
            mssl_var_1E, mssl_var_1F,
        ],
    }
}

enemy_layout! {
    Enemy_MiniCrocomire {
        base: EnemyBase,
        words: [
            mce_var_A, mce_var_B, mce_var_C, mce_var_D, mce_var_E, mce_var_F,
            mce_parameter_1, mce_parameter_2,
        ],
    }
}

enemy_layout! {
    Enemy_NorfairLavajumpingEnemy {
        base: EnemyBase,
        words: [
            nley_var_A, nley_var_B, nley_var_C, nley_var_D, nley_var_E, nley_var_F,
            nley_parameter_1, nley_parameter_2,
        ],
        pad1: [u8; 26696],
        words: [
            nley_var_00, nley_var_01, nley_var_02, nley_var_03, nley_var_04, nley_var_05,
            nley_var_06, nley_var_07, nley_var_08, nley_var_09, nley_var_0A, nley_var_0B,
            nley_var_0C, nley_var_0D, nley_var_0E, nley_var_0F, nley_var_10, nley_var_11,
            nley_var_12, nley_var_13, nley_var_14, nley_var_15, nley_var_16, nley_var_17,
            nley_var_18, nley_var_19, nley_var_1A, nley_var_1B, nley_var_1C, nley_var_1D,
            nley_var_1E, nley_var_1F,
        ],
    }
}

enemy_layout! {
    Enemy_Rinka {
        base: EnemyBase,
        words: [
            rinka_var_A, rinka_var_B, rinka_var_C, rinka_var_D, rinka_var_E, rinka_var_F,
            rinka_parameter_1, rinka_parameter_2,
        ],
        pad1: [u8; 26696],
        words: [
            rinka_var_00, rinka_var_01, rinka_var_02, rinka_var_03, rinka_var_04,
            rinka_var_05, rinka_var_06, rinka_var_07, rinka_var_08, rinka_var_09,
            rinka_var_0A, rinka_var_0B, rinka_var_0C, rinka_var_0D, rinka_var_0E,
            rinka_var_0F, rinka_var_10, rinka_var_11, rinka_var_12, rinka_var_13,
            rinka_var_14, rinka_var_15, rinka_var_16, rinka_var_17, rinka_var_18,
            rinka_var_19, rinka_var_1A, rinka_var_1B, rinka_var_1C, rinka_var_1D,
            rinka_var_1E, rinka_var_1F,
        ],
    }
}

enemy_layout! {
    Enemy_SpikeShootingPlant {
        base: EnemyBase,
        words: [
            sspt_var_A, sspt_var_B, sspt_var_C, sspt_var_D, sspt_var_E, sspt_var_F,
            sspt_parameter_1, sspt_parameter_2,
        ],
        pad1: [u8; 26696],
        words: [
            sspt_var_00, sspt_var_01, sspt_var_02, sspt_var_03, sspt_var_04, sspt_var_05,
            sspt_var_06, sspt_var_07, sspt_var_08, sspt_var_09, sspt_var_0A, sspt_var_0B,
            sspt_var_0C, sspt_var_0D, sspt_var_0E, sspt_var_0F, sspt_var_10, sspt_var_11,
            sspt_var_12, sspt_var_13, sspt_var_14, sspt_var_15, sspt_var_16, sspt_var_17,
            sspt_var_18, sspt_var_19, sspt_var_1A, sspt_var_1B, sspt_var_1C, sspt_var_1D,
            sspt_var_1E, sspt_var_1F,
        ],
    }
}

enemy_layout! {
    Enemy_Gripper {
        base: EnemyBase,
        words: [
            gripper_var_A, gripper_var_B, gripper_var_C, gripper_var_D, gripper_var_E,
            gripper_var_F, gripper_parameter_1, gripper_parameter_2,
        ],
    }
}

enemy_layout! {
    Enemy_GunshipBottom {
        base: EnemyBase,
        words: [
            gbm_var_A, gbm_var_B, gbm_var_C, gbm_var_D, gbm_var_E, gbm_var_F,
            gbm_parameter_1, gbm_parameter_2,
        ],
    }
}

enemy_layout! {
    Enemy_GunshipTop {
        base: EnemyBase,
        words: [
            gtp_var_A, gtp_var_B, gtp_var_C, gtp_var_D, gtp_var_E, gtp_var_F,
            gtp_parameter_1, gtp_parameter_2,
        ],
        pad1: [u8; 26696],
        words: [
            gtp_var_00, gtp_var_01, gtp_var_02, gtp_var_03, gtp_var_04, gtp_var_05,
            gtp_var_06, gtp_var_07, gtp_var_08, gtp_var_09, gtp_var_0A, gtp_var_0B,
            gtp_var_0C, gtp_var_0D, gtp_var_0E, gtp_var_0F, gtp_var_10, gtp_var_11,
            gtp_var_12, gtp_var_13, gtp_var_14, gtp_var_15, gtp_var_16, gtp_var_17,
            gtp_var_18, gtp_var_19, gtp_var_1A, gtp_var_1B, gtp_var_1C, gtp_var_1D,
            gtp_var_1E, gtp_var_1F,
        ],
    }
}

enemy_layout! {
    Enemy_HirisingSlowfalling {
        base: EnemyBase,
        words: [
            hsg_var_A, hsg_var_B, hsg_var_C, hsg_var_D, hsg_var_E, hsg_var_F,
            hsg_parameter_1, hsg_parameter_2,
        ],
        pad1: [u8; 26696],
        words: [
            hsg_var_00, hsg_var_01, hsg_var_02, hsg_var_03, hsg_var_04, hsg_var_05,
            hsg_var_06, hsg_var_07, hsg_var_08, hsg_var_09, hsg_var_0A, hsg_var_0B,
            hsg_var_0C, hsg_var_0D, hsg_var_0E, hsg_var_0F, hsg_var_10, hsg_var_11,
            hsg_var_12, hsg_var_13, hsg_var_14, hsg_var_15, hsg_var_16, hsg_var_17,
            hsg_var_18, hsg_var_19, hsg_var_1A, hsg_var_1B, hsg_var_1C, hsg_var_1D,
            hsg_var_1E, hsg_var_1F,
        ],
    }
}

enemy_layout! {
    Enemy_RisingFallingPlatform {
        base: EnemyBase,
        words: [
            rfpm_var_A, rfpm_var_B, rfpm_var_C, rfpm_var_D, rfpm_var_E, rfpm_var_F,
            rfpm_parameter_1, rfpm_parameter_2,
        ],
        pad1: [u8; 26696],
        words: [
            rfpm_var_00, rfpm_var_01, rfpm_var_02, rfpm_var_03, rfpm_var_04, rfpm_var_05,
            rfpm_var_06, rfpm_var_07, rfpm_var_08, rfpm_var_09, rfpm_var_0A, rfpm_var_0B,
            rfpm_var_0C, rfpm_var_0D, rfpm_var_0E, rfpm_var_0F, rfpm_var_10, rfpm_var_11,
            rfpm_var_12, rfpm_var_13, rfpm_var_14, rfpm_var_15, rfpm_var_16, rfpm_var_17,
            rfpm_var_18, rfpm_var_19, rfpm_var_1A, rfpm_var_1B, rfpm_var_1C, rfpm_var_1D,
            rfpm_var_1E, rfpm_var_1F,
        ],
        pad2: [u8; 1984],
        words: [
            rfpm_var_20, rfpm_var_21, rfpm_var_22, rfpm_var_23, rfpm_var_24, rfpm_var_25,
            rfpm_var_26, rfpm_var_27, rfpm_var_28, rfpm_var_29, rfpm_var_2A, rfpm_var_2B,
            rfpm_var_2C, rfpm_var_2D, rfpm_var_2E, rfpm_var_2F, rfpm_var_30, rfpm_var_31,
            rfpm_var_32, rfpm_var_33, rfpm_var_34, rfpm_var_35, rfpm_var_36, rfpm_var_37,
            rfpm_var_38, rfpm_var_39, rfpm_var_3A, rfpm_var_3B, rfpm_var_3C, rfpm_var_3D,
            rfpm_var_3E, rfpm_var_3F,
        ],
    }
}

enemy_layout! {
    Enemy_HorizontalShootableShutter {
        base: EnemyBase,
        words: [
            hssr_var_A, hssr_var_B, hssr_var_C, hssr_var_D, hssr_var_E, hssr_var_F,
            hssr_parameter_1, hssr_parameter_2,
        ],
    }
}

enemy_layout! {
    Enemy_JetPowerRipper {
        base: EnemyBase,
        words: [
            jprr_var_A, jprr_var_B, jprr_var_C, jprr_var_D, jprr_var_E, jprr_var_F,
            jprr_parameter_1, jprr_parameter_2,
        ],
    }
}

enemy_layout! {
    Enemy_LavaquakeRocks {
        base: EnemyBase,
        words: [
            lrs_var_A, lrs_var_B, lrs_var_C, lrs_var_D, lrs_var_E, lrs_var_F,
            lrs_parameter_1, lrs_parameter_2,
        ],
        pad1: [u8; 26696],
        words: [
            lrs_var_00, lrs_var_01, lrs_var_02, lrs_var_03, lrs_var_04, lrs_var_05,
            lrs_var_06, lrs_var_07, lrs_var_08, lrs_var_09, lrs_var_0A, lrs_var_0B,
            lrs_var_0C, lrs_var_0D, lrs_var_0E, lrs_var_0F, lrs_var_10, lrs_var_11,
            lrs_var_12, lrs_var_13, lrs_var_14, lrs_var_15, lrs_var_16, lrs_var_17,
            lrs_var_18, lrs_var_19, lrs_var_1A, lrs_var_1B, lrs_var_1C, lrs_var_1D,
            lrs_var_1E, lrs_var_1F,
        ],
    }
}

enemy_layout! {
    Enemy_LowerNorfairRio {
        base: EnemyBase,
        words: [
            lnro_var_A, lnro_var_B, lnro_var_C, lnro_var_D, lnro_var_E, lnro_var_F,
            lnro_parameter_1, lnro_parameter_2,
        ],
    }
}

enemy_layout! {
    Enemy_MiniMaridiaTurtle {
        base: EnemyBase,
        words: [
            mmte_var_A, mmte_var_B, mmte_var_C, mmte_var_D, mmte_var_E, mmte_var_F,
            mmte_parameter_1, mmte_parameter_2,
        ],
        pad1: [u8; 26696],
        words: [
            mmte_var_00, mmte_var_01, mmte_var_02, mmte_var_03, mmte_var_04, mmte_var_05,
            mmte_var_06, mmte_var_07, mmte_var_08, mmte_var_09, mmte_var_0A, mmte_var_0B,
            mmte_var_0C, mmte_var_0D, mmte_var_0E, mmte_var_0F, mmte_var_10, mmte_var_11,
            mmte_var_12, mmte_var_13, mmte_var_14, mmte_var_15, mmte_var_16, mmte_var_17,
            mmte_var_18, mmte_var_19, mmte_var_1A, mmte_var_1B, mmte_var_1C, mmte_var_1D,
            mmte_var_1E, mmte_var_1F,
        ],
    }
}

enemy_layout! {
    Enemy_MiniMaridiaBeybladeTurtle {
        base: EnemyBase,
        words: [
            mmbte_var_A, mmbte_var_B, mmbte_var_C, mmbte_var_D, mmbte_var_E, mmbte_var_F,
            mmbte_parameter_1, mmbte_parameter_2,
        ],
    }
}

enemy_layout! {
    Enemy_NorfairErraticFireball {
        base: EnemyBase,
        words: [
            nefl_var_A, nefl_var_B, nefl_var_C, nefl_var_D, nefl_var_E, nefl_var_F,
            nefl_parameter_1, nefl_parameter_2,
        ],
    }
}

pub use crate::types::Ram8000_Default;
enemy_layout! {
    Enemy_Ripper {
        base: EnemyBase,
        words: [
            ripper_var_A, ripper_var_B, ripper_var_C, ripper_var_D, ripper_var_E,
            ripper_var_F, ripper_parameter_1, ripper_parameter_2,
        ],
    }
}

enemy_layout! {
    Enemy_ThinHoppingBlobs {
        base: EnemyBase,
        words: [
            thbs_var_A, thbs_var_B, thbs_var_C, thbs_var_D, thbs_var_E, thbs_var_F,
            thbs_parameter_1, thbs_parameter_2,
        ],
        pad1: [u8; 26696],
        words: [
            thbs_var_00, thbs_var_01, thbs_var_02, thbs_var_03, thbs_var_04, thbs_var_05,
            thbs_var_06, thbs_var_07, thbs_var_08, thbs_var_09, thbs_var_0A, thbs_var_0B,
            thbs_var_0C, thbs_var_0D, thbs_var_0E, thbs_var_0F, thbs_var_10, thbs_var_11,
            thbs_var_12, thbs_var_13, thbs_var_14, thbs_var_15, thbs_var_16, thbs_var_17,
            thbs_var_18, thbs_var_19, thbs_var_1A, thbs_var_1B, thbs_var_1C, thbs_var_1D,
            thbs_var_1E, thbs_var_1F,
        ],
    }
}

enemy_layout! {
    Enemy_TimedShutter {
        base: EnemyBase,
        words: [
            tsr_var_A, tsr_var_B, tsr_var_C, tsr_var_D, tsr_var_E, tsr_var_F,
            tsr_parameter_1, tsr_parameter_2,
        ],
        pad1: [u8; 26696],
        words: [
            tsr_var_00, tsr_var_01, tsr_var_02, tsr_var_03, tsr_var_04, tsr_var_05,
            tsr_var_06, tsr_var_07, tsr_var_08, tsr_var_09, tsr_var_0A, tsr_var_0B,
            tsr_var_0C, tsr_var_0D, tsr_var_0E, tsr_var_0F, tsr_var_10, tsr_var_11,
            tsr_var_12, tsr_var_13, tsr_var_14, tsr_var_15, tsr_var_16, tsr_var_17,
            tsr_var_18, tsr_var_19, tsr_var_1A, tsr_var_1B, tsr_var_1C, tsr_var_1D,
            tsr_var_1E, tsr_var_1F,
        ],
        pad3: [u8; 4032],
        words: [
            tsr_var_40, tsr_var_41, tsr_var_42, tsr_var_43, tsr_var_44, tsr_var_45,
            tsr_var_46, tsr_var_47, tsr_var_48, tsr_var_49, tsr_var_4A, tsr_var_4B,
            tsr_var_4C, tsr_var_4D, tsr_var_4E, tsr_var_4F, tsr_var_50, tsr_var_51,
            tsr_var_52, tsr_var_53, tsr_var_54, tsr_var_55, tsr_var_56, tsr_var_57,
            tsr_var_58, tsr_var_59, tsr_var_5A, tsr_var_5B, tsr_var_5C, tsr_var_5D,
            tsr_var_5E, tsr_var_5F,
        ],
    }
}

const RomPtr_A2: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0xa2>;
crate::ram_map::ram_pointer_functions! {
    gVramWriteEntry(a: i32) -> VramWriteEntry = [(0xd0i32 + a) as isize];
}
crate::game::constants::typed_constants! { i32;
    fnSamus_FrameHandlerAlfa_Func11 = 0x90e695, fnSamus_FrameHandlerBeta_Func17 = 0x90e725,
    fnMiniCrocomire_PreInstr5 = 0xa28a43, fnMiniCrocomire_PreInstr6 = 0xa28a5c,
    fnnullsub_175 = 0xa28a75,
}
crate::game::constants::typed_constants! { i32;
    fnSpikeShootingPlant_2 = 0xa29fba, fnSpikeShootingPlant_3 = 0xa29fec,
    fnnullsub_182 = 0xa2a01b, fnnullsub_187 = 0xa2a7d7,
    fnGunshipTop_3 = 0xa2a80c, fnGunshipTop_4 = 0xa2a8d0,
    fnGunshipTop_5 = 0xa2a942, fnGunshipTop_6 = 0xa2a950,
    fnGunshipTop_7 = 0xa2a987, fnGunshipTop_8 = 0xa2a9bd,
    fnGunshipTop_9 = 0xa2aa4f, fnGunshipTop_10 = 0xa2aa5d,
    fnGunshipTop_11 = 0xa2aa94, fnGunshipTop_12 = 0xa2aaa2,
    fnGunshipTop_13 = 0xa2ab1f, fnGunshipTop_14 = 0xa2ab60,
    fnGunshipTop_15 = 0xa2ab6e, fnGunshipTop_16 = 0xa2aba5,
    fnGunshipTop_17 = 0xa2abc7, fnGunshipTop_18 = 0xa2ac1b,
    fnGunshipTop_19 = 0xa2acd7, fnGunshipTop_20 = 0xa2ad0e,
    fnGunshipTop_21 = 0xa2ad2d, fnFlies_4 = 0xa2b14e,
    fnFlies_5 = 0xa2b17c, fnFlies_6 = 0xa2b1aa,
    fnFlies_7 = 0xa2b1d2,
    fnNorfairLavajumpingEnemy_Func_1 = 0xa2bedc,
    fnNorfairLavajumpingEnemy_Func_2 = 0xa2bf1a, fnNorfairLavajumpingEnemy_Func_3 = 0xa2bf3e,
    fnNorfairLavajumpingEnemy_Func_4 = 0xa2bf7c, fnNorfairLavajumpingEnemy_Func_5 = 0xa2bfbc,
    fnNorfairRio_Func_1 = 0xa2c281, fnNorfairRio_Func_2 = 0xa2c2e7,
    fnNorfairRio_Func_3 = 0xa2c33f, fnNorfairRio_Func_4 = 0xa2c361,
    fnNorfairRio_Func_5 = 0xa2c3b1, fnNorfairRio_Func_6 = 0xa2c406,
    fnLowerNorfairRio_Func_1 = 0xa2c72e, fnLowerNorfairRio_Func_2 = 0xa2c771,
    fnLowerNorfairRio_Func_3 = 0xa2c7bb, fnLowerNorfairRio_Func_4 = 0xa2c7d6,
    fnLowerNorfairRio_Func_5 = 0xa2c82d, fnLowerNorfairRio_Func_6 = 0xa2c888,
    fnMaridiaLargeSnail_Func_7 = 0xa2cf66,
    fnMaridiaLargeSnail_Func_8 = 0xa2cfa9,
    fnLavaSeahorse_Func_1 = 0xa2e654,
    fnLavaSeahorse_Func_2 = 0xa2e6ad, fnLavaSeahorse_Func_3 = 0xa2e6f1,
    fnLavaSeahorse_Func_4 = 0xa2e734, fnLavaSeahorse_Func_5 = 0xa2e749,
    fnnullsub_196 = 0xa2e781,
}
crate::ram_map::ram_pointer_functions! {
    gEnemyData(o: i32) -> EnemyData = [0xf78, o as isize];
    gEnemySpawnData(o: i32) -> EnemySpawnData = [0x7000, o as isize];
    gRam8000_Default(o: i32) -> Ram8000_Default = [0x8000, o as isize];
    Get_Flies(o: i32) -> Enemy_Flies = [0xf78, o as isize];
    Get_Gripper(o: i32) -> Enemy_Gripper = [0xf78, o as isize];
    Get_GunshipBottom(o: i32) -> Enemy_GunshipBottom = [0xf78, o as isize];
    Get_GunshipTop(o: i32) -> Enemy_GunshipTop = [0xf78, o as isize];
    Get_HirisingSlowfalling(o: i32) -> Enemy_HirisingSlowfalling = [0xf78, o as isize];
    Get_HorizontalShootableShutter(o: i32) -> Enemy_HorizontalShootableShutter = [0xf78, o as isize];
    Get_JetPowerRipper(o: i32) -> Enemy_JetPowerRipper = [0xf78, o as isize];
    Get_LavaSeahorse(o: i32) -> Enemy_LavaSeahorse = [0xf78, o as isize];
    Get_LavaquakeRocks(o: i32) -> Enemy_LavaquakeRocks = [0xf78, o as isize];
    Get_LowerNorfairRio(o: i32) -> Enemy_LowerNorfairRio = [0xf78, o as isize];
    Get_MaridiaBeybladeTurtle(o: i32) -> Enemy_MaridiaBeybladeTurtle = [0xf78, o as isize];
    Get_MaridiaLargeSnail(o: i32) -> Enemy_MaridiaLargeSnail = [0xf78, o as isize];
    Get_MaridiaSpikeyShell(o: i32) -> Enemy_MaridiaSpikeyShell = [0xf78, o as isize];
    Get_MaridiaTurtle(o: i32) -> Enemy_MaridiaTurtle = [0xf78, o as isize];
    Get_MiniMaridiaBeybladeTurtle(o: i32) -> Enemy_MiniMaridiaBeybladeTurtle = [0xf78, o as isize];
    Get_MiniMaridiaTurtle(o: i32) -> Enemy_MiniMaridiaTurtle = [0xf78, o as isize];
    Get_NorfairErraticFireball(o: i32) -> Enemy_NorfairErraticFireball = [0xf78, o as isize];
    Get_NorfairLavajumpingEnemy(o: i32) -> Enemy_NorfairLavajumpingEnemy = [0xf78, o as isize];
    Get_NorfairRio(o: i32) -> Enemy_NorfairRio = [0xf78, o as isize];
    Get_Rinka(o: i32) -> Enemy_Rinka = [0xf78, o as isize];
    Get_Rio(o: i32) -> Enemy_Rio = [0xf78, o as isize];
    Get_Ripper(o: i32) -> Enemy_Ripper = [0xf78, o as isize];
    Get_RisingFallingPlatform(o: i32) -> Enemy_RisingFallingPlatform = [0xf78, o as isize];
    Get_SpikeShootingPlant(o: i32) -> Enemy_SpikeShootingPlant = [0xf78, o as isize];
    Get_ThinHoppingBlobs(o: i32) -> Enemy_ThinHoppingBlobs = [0xf78, o as isize];
    Get_TimedShutter(o: i32) -> Enemy_TimedShutter = [0xf78, o as isize];
}
static g_word_A28D50: u16 = 0x30u16;
static g_word_A28D52: u16 = 1u16;
static g_word_A28D5E: u16 = 3u16;
static g_word_A28D60: u16 = 0x1e8u16;
static g_word_A28D62: u16 = 7u16;
static g_word_A28D64: u16 = 0x1eu16;
static g_word_A28D66: u16 = 4u16;
static g_word_A28D68: u16 = 0xfffdu16;
static g_word_A28D6A: u16 = 3u16;
static g_word_A28E80: [i16; 48] = [
    -16, -16, -16, -16, -15, -15, -15, -15, -15, -14, -13, -13, -12, -11, -10, -9, -8, -7, -6, -5,
    -4, -4, 0, 0, -16, -16, -16, -15, -15, -15, -14, -13, -12, -11, -10, -9, -8, -7, -6, -5, -4,
    -3, -3, -2, 0, 0, 0, 0,
];
crate::forward_calls! { pub unsafe fn (game: &mut Game);
    Enemy_GrappleReact_CancelBeam_A2 => Enemy_SwitchToFrozenAi(game);
    Enemy_GrappleReact_KillEnemy_A2 => EnemyGrappleDeath(game);
    Enemy_GrappleReact_SamusLatchesOn_A2 => SamusLatchesOnWithGrapple(game);
    Enemy_NormalFrozenAI_A2 => NormalEnemyFrozenAI(game);
    Enemy_NormalPowerBombAI_A2 => NormalEnemyPowerBombAi(game);
    Enemy_NormalTouchAI_A2 => NormalEnemyTouchAi(game);
    Enemy_NormalShotAI_A2 => NormalEnemyShotAi(game);
}
pub fn BouncingGoofball_Init(game: &mut Game) {
    let index = game.cur_enemy_index();
    game.enemy_instructions(index)
        .set_spritemap_pointer(addr_kSpritemap_Nothing_A2 as u16);
    BouncingGoofball_Func2(game);
    GoofballState::new(game, index).set_state(1);
    let parameter = GoofballState::new(game, index).parameter_low();
    let scale = game.read_rom_word(0xa286df, usize::from(parameter) * 2);
    GoofballState::new(game, index).set_scale(scale);
    let parameter = GoofballState::new(game, index).parameter_high();
    let distance = game.read_rom_word(0xa286ef, usize::from(parameter) * 2);
    GoofballState::new(game, index).set_distance(distance);
    GoofballState::new(game, index).set_accumulated(0);
    GoofballState::new(game, index).set_phase(0);
    GoofballState::new(game, index).set_animation_trigger(0);
}
pub fn BouncingGoofball_Func1(k: u16, game: &mut Game) {
    loop {
        let phase = GoofballState::new(game, k).phase();
        let product = goofball_movement_product(k, phase, game);
        GoofballState::new(game, k).set_product(product);
        let accumulated = GoofballState::new(game, k)
            .accumulated()
            .wrapping_add(product);
        GoofballState::new(game, k).set_accumulated(accumulated);
        let phase = GoofballState::new(game, k).phase().wrapping_add(1);
        GoofballState::new(game, k).set_phase(phase);
        if (accumulated.wrapping_sub(GoofballState::new(game, k).distance()) as i16) >= 0 {
            break;
        }
    }
    let phase = GoofballState::new(game, k).phase();
    GoofballState::new(game, k).set_limit(phase);
    GoofballState::new(game, k).set_active(1);
    GoofballState::new(game, k).set_animation_ready(1);
}
pub fn BouncingGoofball_Main(game: &mut Game) {
    let index = game.cur_enemy_index();
    if GoofballState::new(game, index).initialized() == 0 {
        BouncingGoofball_Func1(game.cur_enemy_index(), game);
        GoofballState::new(game, index).set_initialized(1);
        return;
    }
    let scale = GoofballState::new(game, index).scale();
    GoofballState::new(game, index).set_saved_scale(scale);
    if BouncingGoofball_SamusCloseX(game) != 0 {
        GoofballState::new(game, index).set_active(0);
        GoofballState::new(game, index).set_animation_ready(0);
        if GoofballState::new(game, index).animation_trigger() == 0 {
            GoofballState::new(game, index).set_animation_trigger(1);
            BouncingGoofball_Func3(game);
        }
    } else if GoofballState::new(game, index).active() != 0 {
        if GoofballState::new(game, index).animation_ready() == 0 {
            GoofballState::new(game, index).set_animation_ready(1);
            BouncingGoofball_Func2(game);
        }
        return;
    }
    let scale = GoofballState::new(game, index).saved_scale();
    GoofballState::new(game, index).set_scale(scale);
    match GoofballState::new(game, index).state() {
        0 => BouncingGoofball_State0(game),
        1 => BouncingGoofball_State1(game),
        _ => panic!("invalid Goofball state"),
    }
}
struct GoofballState<'a> {
    bytes: &'a mut [u8],
}
impl<'a> GoofballState<'a> {
    fn accumulated(&self) -> u16 {
        self.read(core::mem::offset_of!(Enemy_BouncingGoofball, bgl_var_D))
    }
    fn distance(&self) -> u16 {
        self.read(core::mem::offset_of!(Enemy_BouncingGoofball, bgl_var_C))
    }
    fn set_limit(&mut self, value: u16) {
        self.write(
            core::mem::offset_of!(Enemy_BouncingGoofball, bgl_var_00),
            value,
        );
    }
    fn initialized(&self) -> u16 {
        self.read(core::mem::offset_of!(Enemy_BouncingGoofball, bgl_var_05))
    }
    fn set_initialized(&mut self, value: u16) {
        self.write(
            core::mem::offset_of!(Enemy_BouncingGoofball, bgl_var_05),
            value,
        );
    }
    fn saved_scale(&self) -> u16 {
        self.read(core::mem::offset_of!(Enemy_BouncingGoofball, bgl_var_03))
    }
    fn set_saved_scale(&mut self, value: u16) {
        self.write(
            core::mem::offset_of!(Enemy_BouncingGoofball, bgl_var_03),
            value,
        );
    }
    fn animation_ready(&self) -> u16 {
        self.read(core::mem::offset_of!(Enemy_BouncingGoofball, bgl_var_04))
    }
    fn set_animation_ready(&mut self, value: u16) {
        self.write(
            core::mem::offset_of!(Enemy_BouncingGoofball, bgl_var_04),
            value,
        );
    }
    fn active(&self) -> u16 {
        self.read(core::mem::offset_of!(Enemy_BouncingGoofball, bgl_var_02))
    }
    fn animation_trigger(&self) -> u16 {
        self.read(core::mem::offset_of!(Enemy_BouncingGoofball, bgl_var_F))
    }
    fn state(&self) -> u16 {
        self.read(core::mem::offset_of!(Enemy_BouncingGoofball, bgl_var_01))
    }
    fn parameter_low(&self) -> u8 {
        self.bytes[core::mem::offset_of!(Enemy_BouncingGoofball, bgl_parameter_1)]
    }
    fn parameter_high(&self) -> u8 {
        self.bytes[core::mem::offset_of!(Enemy_BouncingGoofball, bgl_parameter_1) + 1]
    }
    fn set_scale(&mut self, value: u16) {
        self.write(
            core::mem::offset_of!(Enemy_BouncingGoofball, bgl_var_A),
            value,
        );
    }
    fn set_distance(&mut self, value: u16) {
        self.write(
            core::mem::offset_of!(Enemy_BouncingGoofball, bgl_var_C),
            value,
        );
    }
    fn set_accumulated(&mut self, value: u16) {
        self.write(
            core::mem::offset_of!(Enemy_BouncingGoofball, bgl_var_D),
            value,
        );
    }
    fn new(game: &'a mut Game, index: u16) -> Self {
        Self {
            bytes: &mut game.g_ram[0xf78 + usize::from(index)..],
        }
    }
    fn read(&self, offset: usize) -> u16 {
        u16::from_le_bytes([self.bytes[offset], self.bytes[offset + 1]])
    }
    fn write(&mut self, offset: usize, value: u16) {
        self.bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    }
    fn phase(&self) -> u16 {
        self.read(core::mem::offset_of!(Enemy_BouncingGoofball, bgl_var_E))
    }
    fn set_phase(&mut self, value: u16) {
        self.write(
            core::mem::offset_of!(Enemy_BouncingGoofball, bgl_var_E),
            value,
        );
    }
    fn scale(&self) -> u16 {
        self.read(core::mem::offset_of!(Enemy_BouncingGoofball, bgl_var_A))
    }
    fn set_product(&mut self, value: u16) {
        self.write(
            core::mem::offset_of!(Enemy_BouncingGoofball, bgl_var_B),
            value,
        );
    }
    fn limit(&self) -> u16 {
        self.read(core::mem::offset_of!(Enemy_BouncingGoofball, bgl_var_00))
    }
    fn set_state(&mut self, value: u16) {
        self.write(
            core::mem::offset_of!(Enemy_BouncingGoofball, bgl_var_01),
            value,
        );
    }
    fn set_active(&mut self, value: u16) {
        self.write(
            core::mem::offset_of!(Enemy_BouncingGoofball, bgl_var_02),
            value,
        );
    }
    fn set_animation_trigger(&mut self, value: u16) {
        self.write(
            core::mem::offset_of!(Enemy_BouncingGoofball, bgl_var_F),
            value,
        );
    }
    fn y_pos(&self) -> u16 {
        self.read(
            core::mem::offset_of!(Enemy_BouncingGoofball, base)
                + core::mem::offset_of!(EnemyBase, y_pos),
        )
    }
    fn set_y_pos(&mut self, value: u16) {
        self.write(
            core::mem::offset_of!(Enemy_BouncingGoofball, base)
                + core::mem::offset_of!(EnemyBase, y_pos),
            value,
        );
    }
}
fn goofball_movement_product(index: u16, phase: u16, game: &mut Game) -> u16 {
    // The ROM byte is read even when the signed gate replaces it with FF.
    let sample = game.read_rom_byte(0xa28701, usize::from(phase));
    let sample = if (phase.wrapping_sub(23) as i16) >= 0 {
        255
    } else {
        sample
    };
    u16::from(sample) * u16::from(GoofballState::new(game, index).scale() as u8)
}
pub fn BouncingGoofball_State0(game: &mut Game) {
    let index = game.cur_enemy_index();
    let phase = GoofballState::new(game, index).phase().wrapping_add(1);
    GoofballState::new(game, index).set_phase(phase);
    let phase = GoofballState::new(game, index).phase();
    let product = goofball_movement_product(index, phase, game);
    GoofballState::new(game, index).set_product(product);
    let y = GoofballState::new(game, index)
        .y_pos()
        .wrapping_add(product >> 8);
    GoofballState::new(game, index).set_y_pos(y);
    if (GoofballState::new(game, index)
        .phase()
        .wrapping_sub(GoofballState::new(game, index).limit()) as i16)
        >= 0
    {
        GoofballState::new(game, index).set_state(1);
        GoofballState::new(game, index).set_active(1);
        GoofballState::new(game, index).set_animation_trigger(0);
    }
}
pub fn BouncingGoofball_State1(game: &mut Game) {
    let index = game.cur_enemy_index();
    let phase = GoofballState::new(game, index).phase();
    let product = goofball_movement_product(index, phase, game);
    GoofballState::new(game, index).set_product(product);
    let y = GoofballState::new(game, index)
        .y_pos()
        .wrapping_sub(product >> 8);
    GoofballState::new(game, index).set_y_pos(y);
    let phase = GoofballState::new(game, index).phase().wrapping_sub(1);
    GoofballState::new(game, index).set_phase(phase);
    if GoofballState::new(game, index).phase() & 0x8000 != 0 {
        GoofballState::new(game, index).set_state(0);
    }
}
pub fn BouncingGoofball_SamusCloseX(game: &mut Game) -> u16 {
    let slot = game.cur_enemy_index();
    let index = game.cur_enemy_index();
    let offset =
        0xf78 + usize::from(slot) + core::mem::offset_of!(Enemy_BouncingGoofball, bgl_parameter_2);
    let radius = u16::from_le_bytes([game.g_ram[offset], game.g_ram[offset + 1]]);
    IsSamusWithinEnemy_X(index, radius, game)
}
pub fn BouncingGoofball_Func2(game: &mut Game) {
    let index = game.cur_enemy_index();
    set_goofball_animation(index, addr_kBouncingGoofball_Ilist_86A7 as u16, game);
}
pub fn BouncingGoofball_Func3(game: &mut Game) {
    let index = game.cur_enemy_index();
    set_goofball_animation(index, addr_kBouncingGoofball_Ilist_86BF as u16, game);
}
fn set_goofball_animation(index: u16, instruction: u16, game: &mut Game) {
    game.enemy_instructions(index)
        .set_current_instruction(instruction);
    game.enemy_instructions(index).set_instruction_timer(1);
    game.enemy_instructions(index).set_timer(0);
}
pub fn BouncingGoofball_Instr_88C5(_k: u16, jp: *const u16) -> *const u16 {
    jp
}
pub unsafe fn BouncingGoofball_Instr_88C6(_k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let index = game.cur_enemy_index();
    let offset =
        0xf78 + usize::from(index) + core::mem::offset_of!(Enemy_BouncingGoofball, bgl_var_02);
    game.g_ram[offset..offset + 2].copy_from_slice(&0u16.to_le_bytes());
    QueueSfx2_Max6(0xe, game);
    jp
}
pub unsafe fn MiniCrocomire_Instr_897E(_k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let parameter = *jp;
    let index = game.cur_enemy_index();
    SpawnEprojWithGfx(parameter, index, addr_Eproj_DBF2 as u16, game);
    jp.add(1)
}
pub fn MiniCrocomire_Instr_8990(_k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let index = game.cur_enemy_index();
    set_mini_crocomire_preinstruction(index, 0x8a43, 0, game);
    jp
}
pub fn MiniCrocomire_Instr_899D(_k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let index = game.cur_enemy_index();
    set_mini_crocomire_preinstruction(index, 0x8a5c, 1, game);
    jp
}
fn set_mini_crocomire_preinstruction(index: u16, preinstruction: u16, state: u16, game: &mut Game) {
    let base = 0xf78 + usize::from(index);
    let field = base + core::mem::offset_of!(Enemy_MiniCrocomire, mce_var_F);
    game.g_ram[field..field + 2].copy_from_slice(&preinstruction.to_le_bytes());
    let field = base + core::mem::offset_of!(Enemy_MiniCrocomire, mce_var_E);
    game.g_ram[field..field + 2].copy_from_slice(&state.to_le_bytes());
}

struct MiniCrocomireState<'a> {
    bytes: &'a mut [u8],
}
impl<'a> MiniCrocomireState<'a> {
    fn new(game: &'a mut Game, index: u16) -> Self {
        Self {
            bytes: &mut game.g_ram[0xf78 + usize::from(index)..],
        }
    }
    fn read(&self, offset: usize) -> u16 {
        u16::from_le_bytes([self.bytes[offset], self.bytes[offset + 1]])
    }
    fn write(&mut self, offset: usize, value: u16) {
        self.bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    }
    fn parameter_1(&self) -> u16 {
        self.read(core::mem::offset_of!(Enemy_MiniCrocomire, mce_parameter_1))
    }
    fn parameter_2(&self) -> u16 {
        self.read(core::mem::offset_of!(Enemy_MiniCrocomire, mce_parameter_2))
    }
    fn var_a(&self) -> u16 {
        self.read(core::mem::offset_of!(Enemy_MiniCrocomire, mce_var_A))
    }
    fn var_b(&self) -> u16 {
        self.read(core::mem::offset_of!(Enemy_MiniCrocomire, mce_var_B))
    }
    fn var_c(&self) -> u16 {
        self.read(core::mem::offset_of!(Enemy_MiniCrocomire, mce_var_C))
    }
    fn var_d(&self) -> u16 {
        self.read(core::mem::offset_of!(Enemy_MiniCrocomire, mce_var_D))
    }
    fn state(&self) -> u16 {
        self.read(core::mem::offset_of!(Enemy_MiniCrocomire, mce_var_E))
    }
    fn frame_counter_low(&self) -> u8 {
        self.bytes[core::mem::offset_of!(Enemy_MiniCrocomire, base)
            + core::mem::offset_of!(EnemyBase, frame_counter)]
    }
    fn set_var_b(&mut self, value: u16) {
        self.write(core::mem::offset_of!(Enemy_MiniCrocomire, mce_var_B), value);
    }
    fn set_var_a(&mut self, value: u16) {
        self.write(core::mem::offset_of!(Enemy_MiniCrocomire, mce_var_A), value);
    }
    fn set_var_d(&mut self, value: u16) {
        self.write(core::mem::offset_of!(Enemy_MiniCrocomire, mce_var_D), value);
    }
    fn set_var_c(&mut self, value: u16) {
        self.write(core::mem::offset_of!(Enemy_MiniCrocomire, mce_var_C), value);
    }
    fn preinstruction(&self) -> u16 {
        self.read(core::mem::offset_of!(Enemy_MiniCrocomire, mce_var_F))
    }
    fn set_preinstruction(&mut self, value: u16) {
        self.write(core::mem::offset_of!(Enemy_MiniCrocomire, mce_var_F), value);
    }
    fn set_state(&mut self, value: u16) {
        self.write(core::mem::offset_of!(Enemy_MiniCrocomire, mce_var_E), value);
    }
}
pub fn MiniCrocomire_Init(game: &mut Game) {
    let index = game.cur_enemy_index();
    game.enemy_instructions(index)
        .set_spritemap_pointer(addr_kSpritemap_Nothing_A2 as u16);
    // Parameter indexing is wider than a SNES word; do not wrap this offset.
    let table_offset = usize::from(MiniCrocomireState::new(game, index).parameter_2()) * 8;
    let value = game.read_rom_word(0xa28187, table_offset);
    MiniCrocomireState::new(game, index).set_var_b(value);
    let value = game.read_rom_word(0xa28187, table_offset + 2);
    MiniCrocomireState::new(game, index).set_var_a(value);
    let value = game.read_rom_word(0xa28187, table_offset + 4);
    MiniCrocomireState::new(game, index).set_var_d(value);
    let value = game.read_rom_word(0xa28187, table_offset + 6);
    MiniCrocomireState::new(game, index).set_var_c(value);
    MiniCrocomire_Func1(game);
    MiniCrocomireState::new(game, index).set_preinstruction(0x8a43);
    let parameter = MiniCrocomireState::new(game, index).parameter_1();
    MiniCrocomireState::new(game, index).set_state(parameter);
    if parameter != 0 {
        MiniCrocomire_Func3(game);
        MiniCrocomireState::new(game, index).set_preinstruction(0x8a5c);
    }
}
pub unsafe fn MiniCrocomire_Main(game: &mut Game) {
    let index = game.cur_enemy_index();
    let preinstruction = MiniCrocomireState::new(game, index).preinstruction();
    CallEnemyPreInstr(u32::from(preinstruction) | 0xa20000, game);
}
pub fn MiniCrocomire_Func1(game: &mut Game) {
    let index = game.cur_enemy_index();
    set_mini_crocomire_animation(index, addr_kMiniCrocomire_Ilist_8932 as u16, game);
}
pub fn MiniCrocomire_Func2(game: &mut Game) {
    let index = game.cur_enemy_index();
    set_mini_crocomire_animation(index, addr_kMiniCrocomire_Ilist_8948 as u16, game);
}
pub fn MiniCrocomire_Func3(game: &mut Game) {
    let index = game.cur_enemy_index();
    set_mini_crocomire_animation(index, addr_kMiniCrocomire_Ilist_8958 as u16, game);
}
pub fn MiniCrocomire_Func4(game: &mut Game) {
    let index = game.cur_enemy_index();
    set_mini_crocomire_animation(index, addr_kMiniCrocomire_Ilist_896E as u16, game);
}
fn set_mini_crocomire_animation(index: u16, instruction: u16, game: &mut Game) {
    game.enemy_instructions(index).set_instruction_timer(1);
    game.enemy_instructions(index).set_timer(0);
    game.enemy_instructions(index)
        .set_current_instruction(instruction);
}
pub unsafe fn MiniCrocomire_PreInstr5(_k: u16, game: &mut Game) {
    let index = game.cur_enemy_index();
    let high = MiniCrocomireState::new(game, index).var_d();
    let low = MiniCrocomireState::new(game, index).var_c();
    MiniCrocomire_Func7((u32::from(high) << 16 | u32::from(low)) as i32, game);
    if MiniCrocomire_Func9(game) != 0 {
        MiniCrocomire_Func2(game);
    }
}
pub unsafe fn MiniCrocomire_PreInstr6(_k: u16, game: &mut Game) {
    let index = game.cur_enemy_index();
    let high = MiniCrocomireState::new(game, index).var_b();
    let low = MiniCrocomireState::new(game, index).var_a();
    MiniCrocomire_Func7((u32::from(high) << 16 | u32::from(low)) as i32, game);
    if MiniCrocomire_Func9(game) != 0 {
        MiniCrocomire_Func4(game);
    }
}
pub unsafe fn MiniCrocomire_Func7(amt: i32, game: &mut Game) {
    let index = game.cur_enemy_index();
    if Enemy_MoveRight_IgnoreSlopes(index, amt, game) == 0 {
        let index = game.cur_enemy_index();
        if EnemyFunc_BC76(index, 2 << 16, game) != 0 {
            return;
        }
    }
    MiniCrocomire_Func8(game);
}
pub fn MiniCrocomire_Func8(game: &mut Game) {
    MiniCrocomire_Func1(game);
    let index = game.cur_enemy_index();
    if MiniCrocomireState::new(game, index).state() != 1 {
        MiniCrocomire_Func3(game);
    }
}
pub unsafe fn MiniCrocomire_Func9(game: &mut Game) -> u8 {
    NextRandom(game);
    let index = game.cur_enemy_index();
    let frame = MiniCrocomireState::new(game, index).frame_counter_low();
    if frame.wrapping_add(game.random_number() as u8) < 2 {
        MiniCrocomireState::new(game, index).set_preinstruction(0x8a75);
        return 1;
    }
    0
}
pub fn MaridiaBeybladeTurtle_Init(game: &mut Game) {
    let index = game.cur_enemy_index();
    let properties = game.enemy_instructions(index).properties();
    game.enemy_instructions(index)
        .set_properties(properties | kEnemyProps_DisableSamusColl as u16);
    game.enemy_instructions(index)
        .set_spritemap_pointer(addr_kSpritemap_Nothing_A2 as u16);
    game.enemy_instructions(index).set_instruction_timer(1);
    game.enemy_instructions(index).set_timer(0);
    game.enemy_coordinates(index).set_y_height(0);
    game.enemy_instructions(index)
        .set_current_instruction(addr_kMaridiaBeybladeTurtle_Ilist_8C44 as u16);
    let base = 0xf78 + usize::from(index);
    let field = base + core::mem::offset_of!(Enemy_MaridiaBeybladeTurtle, mbte_var_A);
    game.g_ram[field..field + 2].copy_from_slice(&0x8dd8u16.to_le_bytes());
    let field = base + core::mem::offset_of!(Enemy_MaridiaBeybladeTurtle, mbte_var_F);
    game.g_ram[field..field + 2].copy_from_slice(&g_word_A28D52.to_le_bytes());
}
pub fn MiniMaridiaBeybladeTurtle_Init(game: &mut Game) {
    let index = game.cur_enemy_index();
    let base = 0xf78 + usize::from(index);
    let x = game.enemy_coordinates(index).x_pos();
    let field = base + core::mem::offset_of!(Enemy_MiniMaridiaBeybladeTurtle, mmbte_var_C);
    game.g_ram[field..field + 2].copy_from_slice(&x.to_le_bytes());
    let y = game.enemy_coordinates(index).y_pos();
    let radius = game.enemy_coordinates(index).y_height();
    let field = base + core::mem::offset_of!(Enemy_MiniMaridiaBeybladeTurtle, mmbte_var_D);
    game.g_ram[field..field + 2].copy_from_slice(&y.wrapping_sub(radius).to_le_bytes());
    let field = base + core::mem::offset_of!(Enemy_MiniMaridiaBeybladeTurtle, mmbte_var_A);
    game.g_ram[field..field + 2].copy_from_slice(&0x9142u16.to_le_bytes());
    game.enemy_instructions(index).set_instruction_timer(1);
    game.enemy_instructions(index).set_timer(0);
    let field = base + core::mem::offset_of!(Enemy_MiniMaridiaBeybladeTurtle, mmbte_parameter_1);
    let parameter = u16::from_le_bytes([game.g_ram[field], game.g_ram[field + 1]]);
    let field = base + core::mem::offset_of!(Enemy_MiniMaridiaBeybladeTurtle, mmbte_var_E);
    game.g_ram[field..field + 2].copy_from_slice(&parameter.to_le_bytes());
    let instruction = if (parameter as i16) >= 0 {
        addr_kMaridiaBeybladeTurtle_Ilist_8C72 as u16
    } else {
        addr_kMaridiaBeybladeTurtle_Ilist_8B80 as u16
    };
    game.enemy_instructions(index)
        .set_current_instruction(instruction);
}
crate::game::address_dispatch! {
    pub unsafe fn CallMaridiaBeybladeTurtleFunc(ea: u32, k: u16, game: &mut Game) => ea;
        call (k, game); default { Unreachable(game); };
    {
        0xa28dd8 => MaridiaBeybladeTurtle_Func1; 0xa28e0a => MaridiaBeybladeTurtle_Func2;
        0xa28ee0 => MaridiaBeybladeTurtle_Func3; 0xa28f3f => MaridiaBeybladeTurtle_Func5;
        0xa28f8d => MaridiaBeybladeTurtle_Func7; 0xa28feb => MaridiaBeybladeTurtle_Func8;
        0xa29083 => MaridiaBeybladeTurtle_Func9; 0xa290cc => MaridiaBeybladeTurtle_Func10;
        0xa290e1 => MaridiaBeybladeTurtle_Func11; 0xa28e09 => [{}];
        0xa29142 => MiniMaridiaBeybladeTurtle_Func1; 0xa2916e => MiniMaridiaBeybladeTurtle_Func2;
        0xa29198 => MiniMaridiaBeybladeTurtle_Func3; 0xa291f8 => MiniMaridiaBeybladeTurtle_Func4;
        0xa29239 => MiniMaridiaBeybladeTurtle_Func6;
        0xa2925e => [MiniMaridiaBeybladeTurtle_Func7(game)];
    }
}

pub unsafe fn MaridiaBeybladeTurtle_Main(game: &mut Game) {
    let index = game.cur_enemy_index();
    let preinstruction_offset =
        0xf78 + usize::from(index) + core::mem::offset_of!(Enemy_MaridiaBeybladeTurtle, mbte_var_A);
    let preinstruction = u16::from_le_bytes([
        game.g_ram[preinstruction_offset],
        game.g_ram[preinstruction_offset + 1],
    ]);
    let live_index = game.cur_enemy_index();
    CallMaridiaBeybladeTurtleFunc(0xa20000 | u32::from(preinstruction), live_index, game);
}
pub fn MaridiaBeybladeTurtle_Func1(k: u16, game: &mut Game) {
    let base = 0xf78 + usize::from(k);
    let palette_field = core::mem::offset_of!(Enemy_MaridiaBeybladeTurtle, base)
        + core::mem::offset_of!(EnemyBase, palette_index);
    let field = base + palette_field;
    let palette = [game.g_ram[field], game.g_ram[field + 1]];
    // Child offsets remain wide even when k is near the end of a word.
    for child in [64usize, 128, 192, 256] {
        let field = base + child + palette_field;
        game.g_ram[field..field + 2].copy_from_slice(&palette);
    }
    let tiles_field = core::mem::offset_of!(Enemy_MaridiaBeybladeTurtle, base)
        + core::mem::offset_of!(EnemyBase, vram_tiles_index);
    let field = base + tiles_field;
    let tiles = [game.g_ram[field], game.g_ram[field + 1]];
    for child in [64usize, 128, 192, 256] {
        let field = base + child + tiles_field;
        game.g_ram[field..field + 2].copy_from_slice(&tiles);
    }
    for child in [64usize, 128, 192, 256] {
        let field = base + child + core::mem::offset_of!(Enemy_MaridiaBeybladeTurtle, mbte_var_B);
        game.g_ram[field..field + 2].copy_from_slice(&k.to_le_bytes());
    }
    let field = base + core::mem::offset_of!(Enemy_MaridiaBeybladeTurtle, mbte_var_A);
    game.g_ram[field..field + 2].copy_from_slice(&0x8e0au16.to_le_bytes());
}
pub fn MaridiaBeybladeTurtle_Func2(k: u16, game: &mut Game) {
    let base = 0xf78 + usize::from(k);
    let field = base + core::mem::offset_of!(Enemy_MaridiaBeybladeTurtle, mbte_var_F);
    let active = u16::from_le_bytes([game.g_ram[field], game.g_ram[field + 1]]);
    if active != 0 {
        game.enemy_coordinates(k).set_y_height(0);
        let delta = game
            .enemy_coordinates(k)
            .x_pos()
            .wrapping_sub(game.samus_x_pos());
        let magnitude = if delta & 0x8000 != 0 {
            delta.wrapping_neg()
        } else {
            delta
        };
        if magnitude < 24 {
            let table_index = if delta & 0x8000 != 0 {
                magnitude + 24
            } else {
                magnitude
            };
            let index = game.cur_enemy_index();
            let radius = g_word_A28E80[usize::from(table_index)].wrapping_neg() as u16;
            game.enemy_coordinates(k).set_y_height(radius);
            let properties = game.enemy_instructions(k).properties();
            game.enemy_instructions(k)
                .set_properties(properties | 0x8000);
            if CheckIfEnemyTouchesSamus(index, game) != 0 {
                let top = game
                    .enemy_coordinates(k)
                    .y_pos()
                    .wrapping_sub(game.enemy_coordinates(k).y_height());
                let samus_radius = game.samus_y_radius();
                let samus_y = game.samus_y_pos();
                if (samus_radius.wrapping_add(samus_y).wrapping_sub(top) as i16) >= 0 {
                    let displacement = game.extra_samus_y_displacement();
                    let samus_radius = game.samus_y_radius();
                    let samus_y = game.samus_y_pos();
                    game.set_extra_samus_y_displacement(
                        displacement
                            .wrapping_add(top.wrapping_sub(samus_radius.wrapping_add(samus_y))),
                    );
                }
            }
        }
    } else {
        let field = base + core::mem::offset_of!(Enemy_MaridiaBeybladeTurtle, mbte_var_A);
        game.g_ram[field..field + 2].copy_from_slice(&0x8ee0u16.to_le_bytes());
        let properties = game.enemy_instructions(k).properties();
        game.enemy_instructions(k)
            .set_properties(properties & !(kEnemyProps_Tangible as u16));
    }
}
pub unsafe fn MaridiaBeybladeTurtle_Func3(k: u16, game: &mut Game) {
    MaridiaBeybladeTurtle_Func4(game);
    if (*ram_ptr!(game; 0x5b5) & 1) == 0 {
        if CheckIfEnemyTouchesSamus(k, game) != 0 {
            update_ram_word!(game.extra_samus_x_displacement, wrapping_sub(1));
        }
        let E = Get_MaridiaBeybladeTurtle(k as i32, game);
        (*E).base.y_pos = (*E).base.y_pos.wrapping_sub(1);
        (*E).base.y_height = 16u16;
        if ((*E).base.y_pos & 1) != 0 {
            (*E).base.x_pos = (*E).base.x_pos.wrapping_sub(1);
        } else {
            (*E).base.x_pos = (*E).base.x_pos.wrapping_add(1);
        }
        if Enemy_MoveRight_IgnoreSlopes(k, (1i16 as i32) << 16i32, game) == 0 {
            enemy_instruction! {[(*E).base]; instruction = addr_kMaridiaBeybladeTurtle_Ilist_8C4A as u16, timer = 1u16}
            (*E).mbte_var_A = addr_locret_A28E09 as u16;
        }
    }
}
pub unsafe fn MaridiaBeybladeTurtle_Func5(k: u16, game: &mut Game) {
    let mut v1: u16 = addr_kMaridiaBeybladeTurtle_Ilist_8C1C as u16;
    let E = Get_MaridiaBeybladeTurtle(k as i32, game);
    if (((*E).base.x_pos) as i16).wrapping_sub((ram_word!(game, samus_x_pos)) as i16) < 0i16 {
        v1 = addr_kMaridiaBeybladeTurtle_Ilist_8D00 as u16;
    }
    enemy_instruction! {[(*E).base]; instruction = v1, timer = 1u16}
    (*E).mbte_var_A = addr_locret_A28E09 as u16;
}
pub unsafe fn MaridiaBeybladeTurtle_Func6(k: u16, game: &mut Game) {
    if CheckIfEnemyTouchesSamus(k, game) != 0 {
        let E = Get_MaridiaTurtle(k as i32, game);
        (*E).mte_var_A = ((0xa29083i32) as u16);
        AddToHiLo(
            ram_ptr!(game, u16; 0xb58),
            ram_ptr!(game, u16; 0xb56),
            -(((*E).mte_var_E as i32) << 16i32 | (*E).mte_var_03 as i32) as u32,
        );
        if ram_word!(game, extra_samus_x_displacement) as i32 + 16i32 & 0x8000i32 != 0 {
            set_ram_word!(game.extra_samus_x_displacement = -16i32 as u16);
        }
    }
}
pub unsafe fn MaridiaBeybladeTurtle_Func7(k: u16, game: &mut Game) {
    MaridiaBeybladeTurtle_Func4(game);
    if Enemy_MoveDown(k, (-1i16 as i32) << 16i32, game) == 0 {
        if CheckIfEnemyTouchesSamus(k, game) != 0 {
            update_ram_word!(game.extra_samus_y_displacement, wrapping_sub(1));
        }
        let E = Get_MaridiaTurtle(k as i32, game);
        let v2: u16 = ((*E).mte_var_00).wrapping_sub(1u16);
        (*E).mte_var_00 = v2;
        if v2 == 0 {
            let mut v3: u16 = 0u16;
            if (((*E).base.x_pos) as i16).wrapping_sub((ram_word!(game, samus_x_pos)) as i16) < 0i16
            {
                v3 = 4u16;
            }
            (*E).mte_var_01 = *((game.g_rom.offset(crate::rtl::lorom_offset(0xa28d56)) as *mut u16
                as *mut u8)
                .offset(v3 as i32 as isize) as *mut u16);
            (*E).mte_var_02 = *((game.g_rom.offset(crate::rtl::lorom_offset(0xa28d58)) as *mut u16
                as *mut u8)
                .offset(v3 as i32 as isize) as *mut u16);
            set_fields! {[(*E)], 0u16; mte_var_E, mte_var_03}
            (*E).mte_var_A = ((0xa28febi32) as u16);
        }
    }
}
pub unsafe fn MaridiaBeybladeTurtle_Func8(k: u16, game: &mut Game) {
    MaridiaBeybladeTurtle_Func4(game);
    let E = Get_MaridiaTurtle(k as i32, game);
    if Enemy_MoveRight_IgnoreSlopes(
        k,
        (((*E).mte_var_E as u32) << 16i32 | (*E).mte_var_03 as u32) as i32,
        game,
    ) != 0
    {
        SetHiLo(
            &raw mut (*E).mte_var_E,
            &raw mut (*E).mte_var_03,
            -(((*E).mte_var_E as i32) << 16i32 | (*E).mte_var_03 as i32) as u32,
        );
        SetHiLo(
            &raw mut (*E).mte_var_02,
            &raw mut (*E).mte_var_01,
            -(((*E).mte_var_02 as i32) << 16i32 | (*E).mte_var_01 as i32) as u32,
        );
        set_ram_word!(game.earthquake_type = 0u16);
        set_ram_word!(game.earthquake_timer = 16u16);
        QueueSfx2_Max6(0x1bu16, game);
    } else {
        MaridiaBeybladeTurtle_Func6(k, game);
        AddToHiLo(
            &raw mut (*E).mte_var_E,
            &raw mut (*E).mte_var_03,
            ((*E).mte_var_02 as u32) << 16i32 | (*E).mte_var_01 as u32,
        );
        if ((abs16((*E).mte_var_E)) as i16).wrapping_sub((g_word_A28D5E) as i16) >= 0i16 {
            (*E).mte_var_E = (if ((*E).mte_var_E & 0x8000) != 0 {
                -(g_word_A28D5E as i32)
            } else {
                g_word_A28D5E as i32
            }) as u16;
        }
    };
}
pub unsafe fn MaridiaBeybladeTurtle_Func9(k: u16, game: &mut Game) {
    let E = Get_MaridiaTurtle(k as i32, game);
    let mut v2: u16 = 0;
    MaridiaBeybladeTurtle_Func4(game);
    if (((*E).base.y_pos) as i16).wrapping_sub((g_word_A28D60) as i16) < 0i16 {
        (*E).mte_var_00 = g_word_A28D64;
        v2 = ((0xa290cci32) as u16);
    } else {
        if CheckIfEnemyTouchesSamus(k, game) != 0 {
            (*E).base.y_pos = ((*E).base.y_pos).wrapping_sub(g_word_A28D62);
            update_ram_word!(game.extra_samus_y_displacement, wrapping_sub(g_word_A28D62));
            return;
        }
        v2 = ((0xa290e1i32) as u16);
    }
    (*E).mte_var_A = v2;
    set_fields! {[(*E)], 0u16; mte_var_07, mte_var_04}
}
pub unsafe fn MaridiaBeybladeTurtle_Func10(k: u16, game: &mut Game) {
    MaridiaBeybladeTurtle_Func4(game);
    let E = Get_MaridiaTurtle(k as i32, game);
    let v2: u16 = ((*E).mte_var_00).wrapping_sub(1u16);
    (*E).mte_var_00 = v2;
    if v2 == 0 {
        (*E).mte_var_A = ((0xa290e1i32) as u16);
    }
}
pub unsafe fn MaridiaBeybladeTurtle_Func11(k: u16, game: &mut Game) {
    MaridiaBeybladeTurtle_Func4(game);
    let E = Get_MaridiaTurtle(k as i32, game);
    if (((*Get_MaridiaTurtle(0i32, game)).mte_var_04) as i16).wrapping_sub((g_word_A28D66) as i16)
        < 0i16
    {
        AddToHiLo(
            &raw mut (*E).mte_var_04,
            &raw mut (*E).mte_var_07,
            0x2000u32,
        );
    }
    if Enemy_MoveDown(k, ((*E).mte_var_04 as i16 as i32) << 16i32, game) != 0 {
        enemy_instruction! {[(*E).base]; instruction = (if ((*E).mte_var_E & 0x8000) == 0 {
            addr_kMaridiaBeybladeTurtle_Ilist_8D28 as i32
        } else {
            addr_kMaridiaBeybladeTurtle_Ilist_8C4A as i32
        }) as u16, timer = 1u16}
        (*E).mte_var_A = addr_locret_A28E09 as u16;
    }
}
pub unsafe fn MiniMaridiaBeybladeTurtle_Main(game: &mut Game) {
    let index = game.cur_enemy_index();
    let base = 0xf78 + usize::from(index);
    let parent_offset = base + core::mem::offset_of!(Enemy_MiniMaridiaTurtle, mmte_var_B);
    let parent = u16::from_le_bytes([game.g_ram[parent_offset], game.g_ram[parent_offset + 1]]);
    let parent_trigger_offset =
        0xf78 + usize::from(parent) + core::mem::offset_of!(Enemy_MiniMaridiaTurtle, mmte_var_06);
    game.g_ram[parent_trigger_offset..parent_trigger_offset + 2]
        .copy_from_slice(&0u16.to_le_bytes());
    let preinstruction_offset = base + core::mem::offset_of!(Enemy_MiniMaridiaTurtle, mmte_var_A);
    let preinstruction = u16::from_le_bytes([
        game.g_ram[preinstruction_offset],
        game.g_ram[preinstruction_offset + 1],
    ]);
    let live_index = game.cur_enemy_index();
    CallMaridiaBeybladeTurtleFunc(0xa20000 | u32::from(preinstruction), live_index, game);
}
pub unsafe fn MiniMaridiaBeybladeTurtle_Func1(k: u16, game: &mut Game) {
    if CheckIfEnemyTouchesSamus(k, game) != 0 {
        let E = Get_MiniMaridiaTurtle(k as i32, game);
        (*E).mmte_var_A = ((0xa2916ei32) as u16);
        (*E).mmte_var_05 = 4u16;
        let mut v2: u16 = addr_kMaridiaBeybladeTurtle_Ilist_8C30 as u16;
        if ((*E).mmte_var_E & 0x8000) == 0 {
            v2 = addr_kMaridiaBeybladeTurtle_Ilist_8D14 as u16;
        }
        enemy_instruction! {[(*E).base]; instruction = v2, timer = 1u16}
    }
}
pub unsafe fn MiniMaridiaBeybladeTurtle_Func2(k: u16, game: &mut Game) {
    let E = Get_MiniMaridiaTurtle(k as i32, game);
    if CheckIfEnemyTouchesSamus(k, game) != 0 {
        (*E).mmte_var_05 = 4u16;
    } else {
        let v2: u16 = ((*E).mmte_var_05).wrapping_sub(1u16);
        (*E).mmte_var_05 = v2;
        if v2 == 0 {
            (*E).mmte_var_A = ((0xa29198i32) as u16);
            (*E).mmte_var_00 = 60u16;
        }
    };
}
pub unsafe fn MiniMaridiaBeybladeTurtle_Func3(k: u16, game: &mut Game) {
    let E = Get_MiniMaridiaTurtle(k as i32, game);
    if CheckIfEnemyTouchesSamus(k, game) != 0 {
        (*E).mmte_var_A = ((0xa291f8i32) as u16);
        enemy_instruction! {[(*E).base]; instruction = addr_kMaridiaBeybladeTurtle_Ilist_8BD2 as u16, timer = 1u16}
        (*E).mmte_var_04 = 1u16;
        let mut v2: u16 = g_word_A28D68;
        if (*ram_ptr!(game; 0xa1e) & 0xf) == 8 {
            v2 = g_word_A28D6A;
        }
        (*E).mmte_var_E = v2;
    } else {
        let v4: u16 = ((*E).mmte_var_00).wrapping_sub(1u16);
        (*E).mmte_var_00 = v4;
        if v4 == 0 {
            let mut v5: u16 = addr_kMaridiaBeybladeTurtle_Ilist_8B80 as u16;
            if ((*E).mmte_var_E & 0x8000) == 0 {
                v5 = addr_kMaridiaBeybladeTurtle_Ilist_8C72 as u16;
            }
            enemy_instruction! {[(*E).base]; instruction = v5, timer = 1u16}
            (*E).mmte_var_A = ((0xa29142i32) as u16);
        }
    };
}
pub unsafe fn MiniMaridiaBeybladeTurtle_Func4(k: u16, game: &mut Game) {
    let E = Get_MiniMaridiaTurtle(k as i32, game);
    if Enemy_MoveRight_IgnoreSlopes(
        k,
        (((*E).mmte_var_E as u32) << 16i32 | 0u16 as u32) as i32,
        game,
    ) != 0
    {
        (*E).mmte_var_E = -((*E).mmte_var_E as i32) as u16;
    } else {
        Enemy_MoveDown(k, ((*E).mmte_var_04 as i16 as i32) << 16i32, game);
    };
}
pub unsafe fn MiniMaridiaBeybladeTurtle_Func5(k: u16, game: &mut Game) {
    let mut v1: u16 = addr_kMaridiaBeybladeTurtle_Ilist_8B80 as u16;
    let E = Get_MiniMaridiaBeybladeTurtle(k as i32, game);
    if ((*E).mmbte_var_E & 0x8000) == 0 {
        v1 = addr_kMaridiaBeybladeTurtle_Ilist_8C72 as u16;
    }
    enemy_instruction! {[(*E).base]; instruction = v1, timer = 1u16}
    (*E).mmbte_var_A = ((0xa29142i32) as u16);
}
pub unsafe fn MiniMaridiaBeybladeTurtle_Func6(k: u16, game: &mut Game) {
    if CheckIfEnemyTouchesSamus(k, game) != 0 {
        let mut v1: u16 = addr_kMaridiaBeybladeTurtle_Ilist_8B80 as u16;
        let E = Get_MiniMaridiaBeybladeTurtle(k as i32, game);
        if ((*E).mmbte_var_E & 0x8000) == 0 {
            v1 = addr_kMaridiaBeybladeTurtle_Ilist_8C72 as u16;
        }
        enemy_instruction! {[(*E).base]; instruction = v1, timer = 1u16}
        (*E).mmbte_var_A = ((0xa29142i32) as u16);
    } else {
        MiniMaridiaBeybladeTurtle_Func4(k, game);
    };
}
pub unsafe fn MiniMaridiaBeybladeTurtle_Func7(game: &mut Game) {
    let E = Get_MiniMaridiaTurtle(ram_word!(game, cur_enemy_index) as i32, game);
    (*Get_MiniMaridiaTurtle((*E).mmte_var_B as i32, game)).mmte_var_06 = (*E).base.y_height;
    if CheckIfEnemyTouchesSamus(ram_word!(game, cur_enemy_index), game) == 0 {
        (*Get_MiniMaridiaTurtle(ram_word!(game, cur_enemy_index) as i32, game)).mmte_var_A =
            ((0xa29142i32) as u16);
    }
}
pub unsafe fn MaridiaBeybladeTurtle_Touch(game: &mut Game) {
    let E = Get_MiniMaridiaTurtle(ram_word!(game, cur_enemy_index) as i32, game);
    if ((*E).base.properties & 0x8000) == 0 {
        Enemy_NormalTouchAI_A2(game);
        (*E).mmte_var_A = ((0xa290e1i32) as u16);
        (*E).mmte_var_04 = 2u16;
    }
}
pub unsafe fn MiniMaridiaBeybladeTurtle_Touch(game: &mut Game) {
    let mut v2: i16 = 0;
    let E = Get_MiniMaridiaBeybladeTurtle(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).mmbte_var_A != 0x925eu16 {
        if ((*E).mmbte_var_E & 0x8000) != 0 {
            (*E).base.current_instruction = addr_kMaridiaBeybladeTurtle_Ilist_8C72 as u16;
            v2 = 1i16;
        } else {
            (*E).base.current_instruction = addr_kMaridiaBeybladeTurtle_Ilist_8B80 as u16;
            v2 = -1i16;
        }
        (*E).mmbte_var_E = v2 as u16;
        (*E).base.instruction_timer = 1u16;
        let mut v3: u16 = 0;
        if (((*E).base.x_pos) as i16).wrapping_sub((ram_word!(game, samus_x_pos)) as i16) >= 0i16 {
            v3 = ((*E).base.x_width as i32
                + ram_word!(game, samus_x_radius) as i32
                + ram_word!(game, samus_x_pos) as i32) as u16;
        } else {
            v3 = (((((ram_word!(game, samus_x_pos)) as u32)
                .wrapping_sub((ram_word!(game, samus_x_radius)) as u32))
                << 16i32
                | ram_word!(game, samus_x_pos) as u32)
                .wrapping_sub(
                    ((*E).base.x_width as u32) << 16i32 | ram_word!(game, samus_x_radius) as u32,
                )
                >> 16i32) as u16;
        }
        (*E).base.x_pos = v3;
        (*E).mmbte_var_A = ((0xa29142i32) as u16);
        Enemy_MoveRight_IgnoreSlopes(
            ram_word!(game, cur_enemy_index),
            (((*E).mmbte_var_E as u32) << 16i32 | 0u16 as u32) as i32,
            game,
        );
        MaridiaBeybladeTurtle_Func21(game);
    }
}
pub unsafe fn MaridiaBeybladeTurtle_Func21(game: &mut Game) {
    let mbte_var_B: u16 =
        (*Get_MaridiaBeybladeTurtle(ram_word!(game, cur_enemy_index) as i32, game)).mbte_var_B;
    let E = Get_MaridiaBeybladeTurtle(mbte_var_B as i32, game);
    if (*E).mbte_var_F != 0 {
        (*E).mbte_var_F = (*E).mbte_var_F.wrapping_sub(1);
    }
}
pub unsafe fn MiniMaridiaBeybladeTurtle_Shot(game: &mut Game) {
    NormalEnemyShotAi(game);
    MaridiaBeybladeTurtle_Func21(game);
}
pub unsafe fn MaridiaBeybladeTurtle_Func4(game: &mut Game) {
    let E = Get_MaridiaBeybladeTurtle(0i32, game);
    let r18: u16 = ((*E).base.x_pos as i32 - (*E).base.x_width as i32 - 8i32) as u16;
    let r20: u16 = ((*E).base.x_width as i32 + (*E).base.x_pos as i32 + 8i32) as u16;
    let r22: u16 = ((*E).base.y_pos as i32 - (*E).base.y_height as i32 + 4i32) as u16;
    let r24: u16 = ((*E).base.y_height as i32 + (*E).base.y_pos as i32 - 4i32) as u16;
    if ram_word!(game, samus_x_pos) as i32
        - ram_word!(game, samus_x_radius) as i32
        - 1i32
        - r20 as i32
        & 0x8000i32
        != 0
        && ram_word!(game, samus_x_radius) as i32 + ram_word!(game, samus_x_pos) as i32 - r18 as i32
            & 0x8000i32
            == 0
        && ram_word!(game, samus_y_pos) as i32 - ram_word!(game, samus_y_radius) as i32 + 1i32
            - r24 as i32
            & 0x8000i32
            != 0
        && ram_word!(game, samus_y_radius) as i32 + ram_word!(game, samus_y_pos) as i32 - r22 as i32
            & 0x8000i32
            == 0
        && ram_word!(game, samus_invincibility_timer) == 0
    {
        NormalEnemyTouchAi(game);
    }
}
pub unsafe fn MaridiaBeybladeTurtle_Instr_9381(
    k: u16,
    jp: *const u16,
    game: &mut Game,
) -> *const u16 {
    let mut R48: u16 = 0u16;
    if CheckIfEnemyTouchesSamus(ram_word!(game, cur_enemy_index), game) != 0 {
        update_ram_word!(
            game.extra_samus_x_displacement,
            wrapping_add(
                (*Get_MaridiaBeybladeTurtle(ram_word!(game, cur_enemy_index) as i32, game))
                    .mbte_var_E
            )
        );
        R48 = 1u16;
    }
    let E = Get_MaridiaBeybladeTurtle(ram_word!(game, cur_enemy_index) as i32, game);
    crate::update_place! { previous R50: u16 = (*E).base.y_pos => (*E).mbte_var_D; }
    Enemy_MoveRight_IgnoreSlopes(
        ram_word!(game, cur_enemy_index),
        ((*E).mbte_var_E as i16 as i32) << 16i32,
        game,
    );
    if (*Get_MaridiaBeybladeTurtle((*E).mbte_var_B as i32, game)).mbte_var_A == 0x8e0au16 {
        let x_pos: u16 = (*Get_MaridiaBeybladeTurtle((*E).mbte_var_B as i32, game))
            .base
            .x_pos;
        let xd: u16 = (x_pos).wrapping_sub((*E).base.x_pos);
        let mut v7: u16 = abs16(xd);
        let mut v8: u16 = 0;
        if v7 as i32 - 24i32 & 0x8000i32 != 0 {
            if xd & 0x8000 != 0 {
                v7 = (v7).wrapping_add(24u16);
            }
            v8 = g_word_A28E80[v7 as usize] as u16;
        } else {
            v8 = 1u16;
        }
        Enemy_MoveDown(
            ram_word!(game, cur_enemy_index),
            (v8 as i16 as i32) << 16i32,
            game,
        );
        if R48 != 0 {
            update_ram_word!(game.extra_samus_y_displacement, |fresh1| (fresh1 as i32
                + ((*Get_MaridiaBeybladeTurtle(ram_word!(game, cur_enemy_index) as i32, game))
                    .base
                    .y_pos as i32
                    - R50 as i32))
                as u16);
        }
    }
    jp
}
pub unsafe fn MaridiaBeybladeTurtle_Instr_9412(
    k: u16,
    jp: *const u16,
    game: &mut Game,
) -> *const u16 {
    let mut v4: i16 = 0;
    let E = Get_MaridiaBeybladeTurtle(ram_word!(game, cur_enemy_index) as i32, game);
    let xd: u16 = ((*E).mbte_var_C).wrapping_sub((*E).base.x_pos);
    if ((abs16(xd)) as i16).wrapping_sub((g_word_A28D50) as i16) >= 0i16 {
        if xd & 0x8000 == 0 {
            v4 = 1i16;
        } else {
            v4 = -1i16;
        }
        (*E).mbte_var_E = v4 as u16;
    }
    if ((*Get_MaridiaBeybladeTurtle(ram_word!(game, cur_enemy_index) as i32, game)).mbte_var_E
        & 0x8000)
        != 0
    {
        return addr_kMaridiaBeybladeTurtle_Ilist_8B80 as i32 as usize as *const u16;
    }
    addr_kMaridiaBeybladeTurtle_Ilist_8C72 as i32 as usize as *const u16
}
pub unsafe fn MaridiaBeybladeTurtle_Instr_9447(
    k: u16,
    jp: *const u16,
    game: &mut Game,
) -> *const u16 {
    (*Get_MaridiaBeybladeTurtle(ram_word!(game, cur_enemy_index) as i32, game)).mbte_var_A =
        ((0xa28f3fi32) as u16);
    jp
}
pub unsafe fn MaridiaBeybladeTurtle_Instr_9451(
    k: u16,
    jp: *const u16,
    game: &mut Game,
) -> *const u16 {
    let E = Get_MaridiaTurtle(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).mte_var_A = ((0xa28f8di32) as u16);
    (*E).mte_var_E = -1i32 as u16;
    (*E).mte_var_00 = 16u16;
    addr_kMaridiaBeybladeTurtle_Ilist_8C02 as i32 as usize as *const u16
}
pub unsafe fn MaridiaBeybladeTurtle_Instr_946B(
    k: u16,
    jp: *const u16,
    game: &mut Game,
) -> *const u16 {
    let E = Get_MaridiaTurtle(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).mte_var_A = ((0xa28f8di32) as u16);
    (*E).mte_var_E = 1u16;
    (*E).mte_var_00 = 16u16;
    addr_kMaridiaBeybladeTurtle_Ilist_8C02 as i32 as usize as *const u16
}
pub unsafe fn MaridiaBeybladeTurtle_Instr_9485(
    k: u16,
    jp: *const u16,
    game: &mut Game,
) -> *const u16 {
    if CheckIfEnemyTouchesSamus(ram_word!(game, cur_enemy_index), game) != 0 {
        if ((*Get_MaridiaBeybladeTurtle(ram_word!(game, cur_enemy_index) as i32, game)).mbte_var_E
            & 0x8000)
            == 0
        {
            return addr_kMaridiaBeybladeTurtle_Ilist_8D40 as i32 as usize as *const u16;
        } else {
            return addr_kMaridiaBeybladeTurtle_Ilist_8C62 as i32 as usize as *const u16;
        }
    }
    jp
}
pub unsafe fn MaridiaBeybladeTurtle_Instr_94A1(
    k: u16,
    jp: *const u16,
    game: &mut Game,
) -> *const u16 {
    let E = Get_MaridiaBeybladeTurtle(ram_word!(game, cur_enemy_index) as i32, game);
    if CheckIfEnemyTouchesSamus(ram_word!(game, cur_enemy_index), game) != 0 {
        (*E).mbte_var_A = ((0xa2925ei32) as u16);
    } else {
        (*E).mbte_var_A = ((0xa29142i32) as u16);
    }
    if ((*E).mbte_var_E & 0x8000) == 0 {
        return addr_kMaridiaBeybladeTurtle_Ilist_8C72 as i32 as usize as *const u16;
    }
    addr_kMaridiaBeybladeTurtle_Ilist_8B80 as i32 as usize as *const u16
}
pub unsafe fn MaridiaBeybladeTurtle_Instr_94C7(
    k: u16,
    jp: *const u16,
    game: &mut Game,
) -> *const u16 {
    let E = Get_MaridiaBeybladeTurtle(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).mbte_var_A = ((0xa29239i32) as u16);
    jp
}
pub unsafe fn MaridiaBeybladeTurtle_Instr_94D1(
    k: u16,
    jp: *const u16,
    game: &mut Game,
) -> *const u16 {
    QueueSfx2_Max6(0x3au16, game);
    jp
}
pub unsafe fn ThinHoppingBlobs_Init(game: &mut Game) {
    let E = Get_ThinHoppingBlobs(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.spritemap_pointer = addr_kSpritemap_Nothing_A2 as u16;
    (*E).thbs_var_A = 0u16;
    sub_A29A6C(addr_kThinHoppingBlobs_Ilist_99AD as u16, game);
    (*E).thbs_var_00 = 0u16;
    (*E).thbs_var_D = ((0xa29b65i32) as u16);
    (*E).thbs_var_C = (*E).thbs_parameter_1;
    (*E).thbs_var_04 = 0u16;
}
pub unsafe fn sub_A29A6C(a: u16, game: &mut Game) {
    let v1 = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
    enemy_instruction! {[(*v1)]; instruction = a, timer = 1u16}
    (*v1).timer = 0u16;
}
crate::game::address_dispatch! {
    pub unsafe fn CallThinHoppingBlobs(ea: u32, game: &mut Game) => ea;
        call (game); default { Unreachable(game); };
    {
        10656613 => [{
                    ThinHoppingBlobs_Func6(ram_word!(game, cur_enemy_index), game);
                }];
        10656641 => ThinHoppingBlobs_Func7;
    }
}
pub unsafe fn ThinHoppingBlobs_Main(game: &mut Game) {
    let E = Get_ThinHoppingBlobs(ram_word!(game, cur_enemy_index) as i32, game);
    CallThinHoppingBlobs((((*E).thbs_var_D) as u32) | (0xa20000u32), game);
}
pub unsafe fn ThinHoppingBlobs_Func1(k: u16, game: &mut Game) {
    let E = Get_ThinHoppingBlobs(k as i32, game);
    if (*E).thbs_var_00 as i32 - 3i32 & 0x8000i32 != 0 {
        TwinHoppingBlobs_Func2(game);
    }
    ThinHoppingBlobs_Func3(game);
    ThinHoppingBlobs_Func5(game);
}
pub unsafe fn TwinHoppingBlobs_Func2(game: &mut Game) {
    let E = Get_ThinHoppingBlobs(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).thbs_var_00 = IsSamusWithinEnemy_X(
        ram_word!(game, cur_enemy_index),
        (*E).thbs_parameter_2,
        game,
    );
}
pub unsafe fn ThinHoppingBlobs_Func3(game: &mut Game) {
    let E = Get_ThinHoppingBlobs(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).thbs_var_02 = 1u16;
    if (GetSamusEnemyDelta_X(ram_word!(game, cur_enemy_index), game) & 0x8000) == 0 {
        (*E).thbs_var_02 = 0u16;
    }
    if (*E).thbs_var_04 != 0 {
        (*E).thbs_var_02 = (*E).thbs_var_05;
    }
    (*E).thbs_var_04 = 0u16;
    let mut R28: u16 = ThinHoppingBlobs_Func4(game);
    let mut thbs_var_00: u16 = (*E).thbs_var_00;
    if thbs_var_00 as i32 - 3i32 & 0x8000i32 != 0 {
        if thbs_var_00 == 0 {
            R28 = (R28) & (1u16);
        }
        thbs_var_00 = R28;
        if R28 as i32 - 2i32 & 0x8000i32 == 0 {
            thbs_var_00 = 2u16;
        }
    }
    let v3: u16 = (8u16).wrapping_mul(thbs_var_00);
    (*E).thbs_var_F = v3;
    (*E).thbs_var_E = *((game.g_rom.offset(crate::rtl::lorom_offset(0xa29a0d)) as *mut u16
        as *mut u8)
        .offset(v3 as i32 as isize) as *mut u16);
}
pub unsafe fn ThinHoppingBlobs_Func4(game: &mut Game) -> u16 {
    NextRandom(game);
    ((*Get_ThinHoppingBlobs(ram_word!(game, cur_enemy_index) as i32, game))
        .base
        .frame_counter as i32
        + ram_word!(game, random_number) as i32
        & 7i32) as u16
}
pub unsafe fn ThinHoppingBlobs_Func5(game: &mut Game) {
    let E = Get_ThinHoppingBlobs(ram_word!(game, cur_enemy_index) as i32, game);
    let mut thbs_var_F: u16 = (*E).thbs_var_F;
    let mut r22: u16 = 0u16;
    let mut r24: u16 = 0u16;
    loop {
        r22 = (r22).wrapping_add(
            *((game.g_rom.offset(crate::rtl::lorom_offset(0xa29a09)) as *mut u16 as *mut u8)
                .offset(thbs_var_F as i32 as isize) as *mut u16),
        );
        r24 = (r24).wrapping_add(
            *((game.g_rom.offset(crate::rtl::lorom_offset(0xa2838f)) as *mut u16 as *mut u8)
                .offset((8i32 * ((r22 as i32 & 0xff00i32) >> 8i32)) as isize)
                .offset(1) as *mut u16),
        );
        thbs_var_F = (*E).thbs_var_F;
        let mut v2: u16 = u16::swap_bytes(
            *((game.g_rom.offset(crate::rtl::lorom_offset(0xa29a07)) as *mut u16 as *mut u8)
                .offset(thbs_var_F as i32 as isize) as *mut u16),
        );
        if (v2 as i32 - r24 as i32 & 0x8000i32 != 0) {
            break;
        }
    }
    (*E).thbs_var_B = r22;
    (*E).thbs_var_03 = 0u16;
    let v3: u16 = (r22 >> 1);
    (*E).thbs_var_07 = (r22 >> 1);
    (*E).thbs_var_06 = ((*E).thbs_var_07 as i32 + (v3 as i32 >> 1i32)) as u16;
}
pub unsafe fn ThinHoppingBlobs_Func6(k: u16, game: &mut Game) {
    let E = Get_ThinHoppingBlobs(k as i32, game);
    (*E).thbs_var_C = (*E).thbs_var_C.wrapping_sub(1);
    if ((*E).thbs_var_C & 0x8000) != 0 {
        (*E).thbs_var_D = ((0xa29b81i32) as u16);
        (*E).thbs_var_C = (*E).thbs_parameter_1;
        (*E).thbs_var_01 = 1u16;
        ThinHoppingBlobs_Func1(k, game);
    }
}
crate::game::address_dispatch! {
    pub unsafe fn CallThinHoppingBlobsVarE(ea: u32, game: &mut Game) => ea;
        call (game); default { Unreachable(game); };
    {
        10657035 => ThinHoppingBlobs_Func11; 10657067 => ThinHoppingBlobs_Func12;
        10657099 => ThinHoppingBlobs_Func13; 10657131 => ThinHoppingBlobs_Func14;
        10657176 => ThinHoppingBlobs_Func15; 10657229 => ThinHoppingBlobs_Func16;
    }
}
pub unsafe fn ThinHoppingBlobs_Func7(game: &mut Game) {
    let E = Get_ThinHoppingBlobs(ram_word!(game, cur_enemy_index) as i32, game);
    CallThinHoppingBlobsVarE((((*E).thbs_var_E) as u32) | (0xa20000u32), game);
}
pub unsafe fn ThinHoppingBlobs_Func8(game: &mut Game) {
    let E = Get_ThinHoppingBlobs(ram_word!(game, cur_enemy_index) as i32, game);
    let mut thbs_var_B: u16 = (*E).thbs_var_B;
    if thbs_var_B as i32 - 0x4000i32 & 0x8000i32 == 0 {
        thbs_var_B = 0x4000u16;
    }
    let mut v6: u16 = (8i32 * (thbs_var_B as i32 >> 8i32)) as u16;
    if (*E).thbs_var_03 == 0 {
        v6 = (v6).wrapping_add(4u16);
    }
    if Enemy_MoveDown(
        ram_word!(game, cur_enemy_index),
        *(game.g_rom.offset(crate::rtl::lorom_offset(0xa0cbc7)) as *mut u32)
            .offset((v6 >> 2) as isize) as i32,
        game,
    ) != 0
    {
        if (*E).thbs_var_03 != 0 {
            (*E).thbs_var_D = ((0xa29b65i32) as u16);
        } else {
            (*E).thbs_var_04 = 1u16;
            (*E).thbs_var_05 = ((*E).thbs_var_02) ^ (1u16);
            (*E).thbs_var_00 = 4u16;
            (*E).thbs_var_E = ((0xa29d98i32) as u16);
        }
        (*E).thbs_var_01 = 0u16;
    } else {
        let thbs_var_F: u16 = (*E).thbs_var_F;
        if (*E).thbs_var_03 != 0 {
            ThinHoppingBlobs_Func10(game);
            (*E).thbs_var_B = ((*E).thbs_var_B).wrapping_add(
                *((game.g_rom.offset(crate::rtl::lorom_offset(0xa29a0b)) as *mut u16 as *mut u8)
                    .offset(thbs_var_F as i32 as isize) as *mut u16),
            );
        } else {
            ThinHoppingBlobs_Func9(game);
            (*E).thbs_var_B = ((*E).thbs_var_B).wrapping_sub(
                *((game.g_rom.offset(crate::rtl::lorom_offset(0xa29a0b)) as *mut u16 as *mut u8)
                    .offset(thbs_var_F as i32 as isize) as *mut u16),
            );
        }
        if (*E).thbs_var_B & 0x8000 != 0 {
            (*E).thbs_var_03 = 1u16;
            (*E).thbs_var_B = 0u16;
        }
        let mut R20: u16 =
            ((*((game.g_rom.offset(crate::rtl::lorom_offset(0xa29a09)) as *mut u16 as *mut u8)
                .offset((*E).thbs_var_F as i32 as isize) as *mut u16) as i32
                & 0xff00i32)
                >> 8i32) as u16;
        if (*E).thbs_var_02 != 0 {
            R20 = -(R20 as i32) as u16;
        }
        if Enemy_MoveRight_IgnoreSlopes(
            ram_word!(game, cur_enemy_index),
            (R20 as i16 as i32) << 16i32,
            game,
        ) != 0
        {
            (*E).thbs_var_04 = 1u16;
            (*E).thbs_var_05 = ((*E).thbs_var_02) ^ (1u16);
            (*E).thbs_var_01 = 0u16;
            (*E).thbs_var_00 = 4u16;
            (*E).thbs_var_E = ((0xa29d98i32) as u16);
        }
    };
}
pub unsafe fn ThinHoppingBlobs_Func9(game: &mut Game) {
    let E = Get_ThinHoppingBlobs(ram_word!(game, cur_enemy_index) as i32, game);
    let mut v2: u16 = 0;
    if (*E).thbs_var_02 != 0 {
        let thbs_var_B: u16 = (*E).thbs_var_B;
        if ((thbs_var_B) as i16).wrapping_sub(((*E).thbs_var_06) as i16) >= 0i16 {
            v2 = addr_kThinHoppingBlobs_Ilist_9A01 as u16;
        } else if ((thbs_var_B) as i16).wrapping_sub(((*E).thbs_var_07) as i16) >= 0i16 {
            v2 = addr_kThinHoppingBlobs_Ilist_99FB as u16;
        } else {
            v2 = addr_kThinHoppingBlobs_Ilist_99F5 as u16;
        }
    } else {
        let v1: u16 = (*E).thbs_var_B;
        if ((v1) as i16).wrapping_sub(((*E).thbs_var_06) as i16) >= 0i16 {
            v2 = addr_kThinHoppingBlobs_Ilist_99E9 as u16;
        } else if ((v1) as i16).wrapping_sub(((*E).thbs_var_07) as i16) >= 0i16 {
            v2 = addr_kThinHoppingBlobs_Ilist_99EF as u16;
        } else {
            v2 = addr_kThinHoppingBlobs_Ilist_99F5 as u16;
        }
    }
    sub_A29A6C(v2, game);
}
pub unsafe fn ThinHoppingBlobs_Func10(game: &mut Game) {
    let E = Get_ThinHoppingBlobs(ram_word!(game, cur_enemy_index) as i32, game);
    let mut v2: u16 = 0;
    if (*E).thbs_var_02 != 0 {
        let thbs_var_B: u16 = (*E).thbs_var_B;
        if ((thbs_var_B) as i16).wrapping_sub(((*E).thbs_var_07) as i16) < 0i16 {
            v2 = addr_kThinHoppingBlobs_Ilist_99F5 as u16;
        } else if ((thbs_var_B) as i16).wrapping_sub(((*E).thbs_var_06) as i16) < 0i16 {
            v2 = addr_kThinHoppingBlobs_Ilist_99EF as u16;
        } else {
            v2 = addr_kThinHoppingBlobs_Ilist_99E9 as u16;
        }
    } else {
        let v1: u16 = (*E).thbs_var_B;
        if ((v1) as i16).wrapping_sub(((*E).thbs_var_07) as i16) < 0i16 {
            v2 = addr_kThinHoppingBlobs_Ilist_99F5 as u16;
        } else if ((v1) as i16).wrapping_sub(((*E).thbs_var_06) as i16) < 0i16 {
            v2 = addr_kThinHoppingBlobs_Ilist_99FB as u16;
        } else {
            v2 = addr_kThinHoppingBlobs_Ilist_9A01 as u16;
        }
    }
    sub_A29A6C(v2, game);
}
pub unsafe fn ThinHoppingBlobs_Func11(game: &mut Game) {
    ThinHoppingBlobs_Func8(game);
    let E = Get_ThinHoppingBlobs(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).thbs_var_04 != 0u16 || (*E).thbs_var_01 == 0 {
        (*E).thbs_var_01 = 0u16;
        sub_A29A6C(addr_kThinHoppingBlobs_Ilist_99D5 as u16, game);
    }
}
pub unsafe fn ThinHoppingBlobs_Func12(game: &mut Game) {
    ThinHoppingBlobs_Func8(game);
    let E = Get_ThinHoppingBlobs(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).thbs_var_04 != 0u16 || (*E).thbs_var_01 == 0 {
        (*E).thbs_var_01 = 0u16;
        sub_A29A6C(addr_kThinHoppingBlobs_Ilist_99C1 as u16, game);
    }
}
pub unsafe fn ThinHoppingBlobs_Func13(game: &mut Game) {
    ThinHoppingBlobs_Func8(game);
    let E = Get_ThinHoppingBlobs(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).thbs_var_04 != 0u16 || (*E).thbs_var_01 == 0 {
        (*E).thbs_var_01 = 0u16;
        sub_A29A6C(addr_kThinHoppingBlobs_Ilist_99AD as u16, game);
    }
}
pub unsafe fn ThinHoppingBlobs_Func14(game: &mut Game) {
    ThinHoppingBlobs_Func8(game);
    let E = Get_ThinHoppingBlobs(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).thbs_var_04 == 0 {
        if (*E).thbs_var_01 != 0 {
            return;
        }
        (*E).thbs_var_00 = 0u16;
        (*E).thbs_var_D = ((0xa29b65i32) as u16);
    }
    (*E).thbs_var_01 = 0u16;
    sub_A29A6C(addr_kThinHoppingBlobs_Ilist_99D5 as u16, game);
}
pub unsafe fn ThinHoppingBlobs_Func15(game: &mut Game) {
    let E = Get_ThinHoppingBlobs(ram_word!(game, cur_enemy_index) as i32, game);
    let thbs_var_F: u16 = (*E).thbs_var_F;
    let R20: u16 = ((*((game.g_rom.offset(crate::rtl::lorom_offset(0xa29a0b)) as *mut u16
        as *mut u8)
        .offset(thbs_var_F as i32 as isize) as *mut u16) as i32
        & 0xff00i32)
        >> 8i32) as u16;
    let R18: u16 = ((*(game.g_rom.offset(crate::rtl::lorom_offset(0xa29a0b)) as *mut u16 as *mut u8)
        .offset(thbs_var_F as i32 as isize) as i32)
        << 8i32) as u16;
    if Enemy_MoveDown(
        ram_word!(game, cur_enemy_index),
        ((R20 as u32) << 16i32 | R18 as u32) as i32,
        game,
    ) != 0
    {
        let R28: u16 = ThinHoppingBlobs_Func4(game);
        (*E).thbs_var_00 = ((R28 as i32 & 1i32) + 5i32) as u16;
        (*E).thbs_var_D = ((0xa29b65i32) as u16);
    }
}
pub unsafe fn ThinHoppingBlobs_Func16(game: &mut Game) {
    ThinHoppingBlobs_Func8(game);
    let E = Get_ThinHoppingBlobs(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).thbs_var_01 == 0 {
        (*E).thbs_var_01 = 0u16;
        (*E).thbs_var_00 = 3u16;
        (*E).thbs_var_D = ((0xa29b65i32) as u16);
        sub_A29A6C(addr_kThinHoppingBlobs_Ilist_99D5 as u16, game);
    }
}
pub unsafe fn SpikeShootingPlant_Instr_9F2A(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    QueueSfx2_Max6(0x34u16, game);
    jp
}
pub unsafe fn SpikeShootingPlant_Init(game: &mut Game) {
    let E = Get_SpikeShootingPlant(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.spritemap_pointer = addr_kSpritemap_Nothing_A2 as u16;
    if *(&raw mut (*E).sspt_parameter_1 as *mut u8).offset(1isize) != 0 {
        SpikeShootingPlant_5(game);
    } else {
        SpikeShootingPlant_7(game);
    }
    let sspt_parameter_1_low: u16 = *(&raw mut (*E).sspt_parameter_1 as *mut u8).offset(0) as u16;
    (*E).sspt_var_E = sspt_parameter_1_low;
    (*E).sspt_var_F = crate::rtl::rom_table_word!(game, 0xa29f42, sspt_parameter_1_low as isize);
    let v2: i32 = *(&raw mut (*E).sspt_parameter_2 as *mut u8).offset(0) as i32;
    (*E).sspt_var_01 =
        (crate::rtl::rom_table_word!(game, 0xa29f36, v2 as isize)).wrapping_add((*E).base.x_pos);
    (*E).sspt_var_00 =
        ((*E).base.x_pos).wrapping_sub(crate::rtl::rom_table_word!(game, 0xa29f36, v2 as isize));
    let v3: i32 = 8i32 * *(&raw mut (*E).sspt_parameter_2 as *mut u8).offset(1isize) as i32 >> 1i32;
    (*E).sspt_var_B = crate::rtl::rom_table_word!(game, 0xa28187, v3 as isize);
    (*E).sspt_var_A = crate::rtl::rom_table_word!(game, 0xa28187, (v3 + 1i32) as isize);
    (*E).sspt_var_D = crate::rtl::rom_table_word!(game, 0xa28187, (v3 + 2i32) as isize);
    (*E).sspt_var_C = crate::rtl::rom_table_word!(game, 0xa28187, (v3 + 3i32) as isize);
}
pub unsafe fn SpikeShootingPlant_Main(game: &mut Game) {
    let E = Get_SpikeShootingPlant(ram_word!(game, cur_enemy_index) as i32, game);
    CallEnemyPreInstr((((*E).sspt_var_F) as u32) | (0xa20000u32), game);
}
pub unsafe fn SpikeShootingPlant_2(k: u16, game: &mut Game) {
    let E = Get_SpikeShootingPlant(ram_word!(game, cur_enemy_index) as i32, game);
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        ((*E).sspt_var_D as u32) << 16i32 | (*E).sspt_var_C as u32,
    );
    if (((*E).base.x_pos) as i16).wrapping_sub(((*E).sspt_var_00) as i16) < 0i16 {
        (*E).sspt_var_F = ((0xa29feci32) as u16);
        (*E).sspt_var_E = 1u16;
    }
    SpikeShootingPlant_4(game);
}
pub unsafe fn SpikeShootingPlant_3(k: u16, game: &mut Game) {
    let E = Get_SpikeShootingPlant(ram_word!(game, cur_enemy_index) as i32, game);
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        ((*E).sspt_var_B as u32) << 16i32 | (*E).sspt_var_A as u32,
    );
    if (((*E).base.x_pos) as i16).wrapping_sub(((*E).sspt_var_01) as i16) >= 0i16 {
        (*E).sspt_var_F = ((0xa29fbai32) as u16);
        (*E).sspt_var_E = 0u16;
    }
    SpikeShootingPlant_4(game);
}
pub unsafe fn SpikeShootingPlant_4(game: &mut Game) {
    NextRandom(game);
    let E = Get_SpikeShootingPlant(ram_word!(game, cur_enemy_index) as i32, game);
    if ((*(&raw mut (*E).base.frame_counter as *mut u8).offset(0))
        .wrapping_add((ram_word!(game, random_number)) as u8)) as i32
        - 3i32
        & 0x8000i32
        != 0
    {
        (*E).sspt_var_F = ((0xa2a01bi32) as u16);
        if *(&raw mut (*E).sspt_parameter_1 as *mut u8).offset(1isize) != 0 {
            SpikeShootingPlant_6(game);
        } else {
            SpikeShootingPlant_8(game);
        }
    }
}
pub unsafe fn SpikeShootingPlant_5(game: &mut Game) {
    let E = Get_SpikeShootingPlant(ram_word!(game, cur_enemy_index) as i32, game);
    enemy_instruction! {[(*E).base]; instruction = addr_kSpikeShootingPlant_Ilist_9E8A as u16, timer = 1u16}
    (*E).base.timer = 0u16;
}
pub unsafe fn SpikeShootingPlant_6(game: &mut Game) {
    let E = Get_SpikeShootingPlant(ram_word!(game, cur_enemy_index) as i32, game);
    enemy_instruction! {[(*E).base]; instruction = addr_kSpikeShootingPlant_Ilist_9EB0 as u16, timer = 1u16}
    (*E).base.timer = 0u16;
}
pub unsafe fn SpikeShootingPlant_7(game: &mut Game) {
    let E = Get_SpikeShootingPlant(ram_word!(game, cur_enemy_index) as i32, game);
    enemy_instruction! {[(*E).base]; instruction = addr_kSpikeShootingPlant_Ilist_9EDA as u16, timer = 1u16}
    (*E).base.timer = 0u16;
}
pub unsafe fn SpikeShootingPlant_8(game: &mut Game) {
    let E = Get_SpikeShootingPlant(ram_word!(game, cur_enemy_index) as i32, game);
    enemy_instruction! {[(*E).base]; instruction = addr_kSpikeShootingPlant_Ilist_9F00 as u16, timer = 1u16}
    (*E).base.timer = 0u16;
}
pub unsafe fn SpikeShootingPlant_Instr_A095(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E = Get_SpikeShootingPlant(k as i32, game);
    (*E).sspt_var_F = ((0xa29fbai32) as u16);
    if (*E).sspt_var_E != 0 {
        (*E).sspt_var_F = ((0xa29feci32) as u16);
    }
    jp
}
pub unsafe fn SpikeShootingPlant_Instr_A0A7(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    SpawnEprojWithGfx(
        *jp,
        ram_word!(game, cur_enemy_index),
        addr_kEproj_SpikeShootingPlantSpikes as u16,
        game,
    );
    jp.offset(1)
}
static off_A2A3D3: [Option<Func_V>; 7] = callback_table! { unsafe Func_V;
    MaridiaSpikeyShell_1, MaridiaSpikeyShell_2, MaridiaSpikeyShell_3, MaridiaSpikeyShell_4,
    MaridiaSpikeyShell_5, MaridiaSpikeyShell_6, MaridiaSpikeyShell_7,
};
pub unsafe fn MaridiaSpikeyShell_Init(game: &mut Game) {
    let E = Get_MaridiaSpikeyShell(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.spritemap_pointer = addr_kSpritemap_Nothing_A2 as u16;
    let mssl_parameter_1_low: u16 = *(&raw mut (*E).mssl_parameter_1 as *mut u8).offset(0) as u16;
    (*E).mssl_var_E = mssl_parameter_1_low;
    off_A2A3D3[(mssl_parameter_1_low as i32 & 1i32) as usize].expect("non-null function pointer")(
        game,
    );
    let v3: i32 = 4i32 * *(&raw mut (*E).mssl_parameter_2 as *mut u8).offset(0) as i32;
    (*E).mssl_var_B = crate::rtl::rom_table_word!(game, 0xa28187, v3 as isize);
    (*E).mssl_var_A = crate::rtl::rom_table_word!(game, 0xa28187, (v3 + 1i32) as isize);
    (*E).mssl_var_D = crate::rtl::rom_table_word!(game, 0xa28187, (v3 + 2i32) as isize);
    (*E).mssl_var_C = crate::rtl::rom_table_word!(game, 0xa28187, (v3 + 3i32) as isize);
    let v4: i32 = *(&raw mut (*E).mssl_parameter_2 as *mut u8).offset(1isize) as i32;
    (*E).mssl_var_02 =
        (crate::rtl::rom_table_word!(game, 0xa2a3dd, v4 as isize)).wrapping_add((*E).base.x_pos);
    (*E).mssl_var_01 =
        ((*E).base.x_pos).wrapping_sub(crate::rtl::rom_table_word!(game, 0xa2a3dd, v4 as isize));
    (*E).mssl_var_00 = crate::rtl::rom_table_word!(
        game,
        0xa2a3ed,
        *(&raw mut (*E).mssl_parameter_1 as *mut u8).offset(1isize) as isize
    );
    (*E).mssl_var_F = 0u16;
    if (*E).mssl_var_E == 2u16 {
        (*E).mssl_var_F = 16u16;
        (*E).base.y_pos = ((*E).base.y_pos).wrapping_add(16u16);
    }
}
pub unsafe fn MaridiaSpikeyShell_Main(game: &mut Game) {
    let e = Get_MaridiaSpikeyShell(ram_word!(game, cur_enemy_index) as i32, game);
    off_A2A3D3[((*e).mssl_var_E as i16 as i32 + 2i32) as usize].expect("non-null function pointer")(
        game,
    );
}
pub unsafe fn MaridiaSpikeyShell_1(game: &mut Game) {
    let E = Get_MaridiaSpikeyShell(ram_word!(game, cur_enemy_index) as i32, game);
    enemy_instruction! {[(*E).base]; instruction = addr_kMaridiaSpikeyShell_Ilist_A3AB as u16, timer = 1u16}
    (*E).base.timer = 0u16;
}
pub unsafe fn MaridiaSpikeyShell_2(game: &mut Game) {
    let E = Get_MaridiaSpikeyShell(ram_word!(game, cur_enemy_index) as i32, game);
    enemy_instruction! {[(*E).base]; instruction = addr_kMaridiaSpikeyShell_Ilist_A3BD as u16, timer = 1u16}
    (*E).base.timer = 0u16;
}
pub unsafe fn MaridiaSpikeyShell_3(game: &mut Game) {
    let E = Get_MaridiaSpikeyShell(ram_word!(game, cur_enemy_index) as i32, game);
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        ((*E).mssl_var_D as u32) << 16i32 | (*E).mssl_var_C as u32,
    );
    if (((*E).base.x_pos) as i16).wrapping_sub(((*E).mssl_var_01) as i16) < 0i16 {
        (*E).mssl_var_E = (*E).mssl_var_E.wrapping_sub(1);
    }
    MaridiaSpikeyShell_8(ram_word!(game, cur_enemy_index), game);
}
pub unsafe fn MaridiaSpikeyShell_4(game: &mut Game) {
    let E = Get_MaridiaSpikeyShell(ram_word!(game, cur_enemy_index) as i32, game);
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        ((*E).mssl_var_B as u32) << 16i32 | (*E).mssl_var_A as u32,
    );
    if (((*E).base.x_pos) as i16).wrapping_sub(((*E).mssl_var_02) as i16) >= 0i16 {
        (*E).mssl_var_E = 0u16;
    }
    MaridiaSpikeyShell_8(ram_word!(game, cur_enemy_index), game);
}
pub unsafe fn MaridiaSpikeyShell_5(game: &mut Game) {
    let E = Get_MaridiaSpikeyShell(ram_word!(game, cur_enemy_index) as i32, game);
    let v1: u16 = ((*E).mssl_var_00).wrapping_sub(1u16);
    (*E).mssl_var_00 = v1;
    if v1 == 0 {
        (*E).mssl_var_E = 4u16;
    }
}
pub unsafe fn MaridiaSpikeyShell_6(game: &mut Game) {
    let E = Get_MaridiaSpikeyShell(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.y_pos = (*E).base.y_pos.wrapping_add(1);
    (*E).mssl_var_F = (*E).mssl_var_F.wrapping_add(1);
    if (*E).mssl_var_F as i32 - 16i32 & 0x8000i32 == 0 {
        (*E).mssl_var_E = 2u16;
        (*E).mssl_var_00 = crate::rtl::rom_table_word!(
            game,
            0xa2a3ed,
            *(&raw mut (*E).mssl_parameter_1 as *mut u8).offset(1isize) as isize
        );
    }
}
pub unsafe fn MaridiaSpikeyShell_7(game: &mut Game) {
    let E = Get_MaridiaSpikeyShell(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.y_pos = (*E).base.y_pos.wrapping_sub(1);
    crate::update_place! { previous fresh29 = (*E).mssl_var_F => (*E).mssl_var_F.wrapping_sub(1); }
    if fresh29 == 1u16 {
        (*E).mssl_var_E =
            ((*(ram_ptr!(game, u16; 0x5e5) as *mut u8).offset(1isize)) as u16) & (1u16);
    }
}
pub unsafe fn MaridiaSpikeyShell_8(k: u16, game: &mut Game) {
    NextRandom(game);
    let E = Get_MaridiaSpikeyShell(k as i32, game);
    if ((*(&raw mut (*E).base.frame_counter as *mut u8).offset(0))
        .wrapping_add((ram_word!(game, random_number)) as u8)) as i32
        - 6i32
        & 0x8000i32
        != 0
    {
        (*E).mssl_var_E = 3u16;
    }
}
pub unsafe fn MaridiaSpikeyShell_Instr_A56D(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*Get_MaridiaSpikeyShell(k as i32, game)).mssl_var_E = 0u16;
    jp
}
pub unsafe fn MaridiaSpikeyShell_Instr_A571(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*Get_MaridiaSpikeyShell(k as i32, game)).mssl_var_E = 1u16;
    jp
}
pub unsafe fn MaridiaSpikeyShell_Shot(game: &mut Game) {
    let E = Get_MaridiaSpikeyShell(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).mssl_var_E as i32 - 1i32 & 0x8000i32 != 0 {
        Enemy_NormalShotAI_A2(game);
    }
}
pub unsafe fn GunshipTop_Init(game: &mut Game) {
    let E = Get_GunshipTop(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.properties = ((*E).base.properties as i32
        | (kEnemyProps_DisableSamusColl as i32 | kEnemyProps_Tangible as i32))
        as u16;
    (*E).base.instruction_timer = 1u16;
    (*E).base.timer = 0u16;
    (*E).base.current_instruction = addr_kGunshipTop_Ilist_A616 as u16;
    (*E).base.palette_index = 3584u16;
    if ram_word!(game, game_state) != 40u16 || ram_word!(game, demo_set) != 0u16 {
        if ram_word!(game, loading_game_state) as i32 == kLoadingGameState_22_EscapingCeres as i32 {
            (*E).base.y_pos = (ram_word!(game, samus_y_pos)).wrapping_sub(17u16);
            (*E).gtp_var_F = ((0xa2a80ci32) as u16);
        } else {
            let v3: u16 = ((*E).base.y_pos).wrapping_sub(25u16);
            (*E).base.y_pos = v3;
            (*E).gtp_var_E = v3;
            (*E).gtp_var_F = ((0xa2a9bdi32) as u16);
        }
    } else {
        set_ram_word!(game.samus_y_pos = 1138u16);
        let v5: u16 = ((*E).base.y_pos).wrapping_sub(25u16);
        (*E).base.y_pos = v5;
        (*E).gtp_var_E = v5;
        (*E).gtp_var_F = ((0xa2ab60i32) as u16);
        (*Get_GunshipTop(0i32, game)).gtp_var_A = 144u16;
    }
    SpawnPalfxObject(addr_stru_8DE1C0 as u16, game);
    (*Get_GunshipTop(ram_word!(game, cur_enemy_index) as i32, game)).gtp_var_D = 1u16;
    (*Get_GunshipTop(0i32, game)).gtp_var_C = 0u16;
}
pub unsafe fn GunshipBottom_Init(game: &mut Game) {
    let E = Get_GunshipBottom(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.properties = ((*E).base.properties as i32
        | (kEnemyProps_DisableSamusColl as i32 | kEnemyProps_Tangible as i32))
        as u16;
    (*E).base.instruction_timer = 1u16;
    (*E).base.timer = 0u16;
    if (*E).gbm_parameter_2 != 0 {
        (*E).base.current_instruction = addr_kGunshipTop_Ilist_A60E as u16;
    } else {
        (*E).base.current_instruction = addr_kGunshipTop_Ilist_A61C as u16;
    }
    let v2: i32 = ram_word!(game, cur_enemy_index) as i32 >> 1i32;
    (*E).base.vram_tiles_index = ram_word!(game, enemy_drawing_queue[(v2 + 106i32)]);
    (*E).base.palette_index = 3584u16;
    if (*E).gbm_parameter_2 != 0 {
        (*E).base.y_pos = (ram_word!(game, enemy_drawing_queue[(v2 + 61i32)])).wrapping_sub(1u16);
        if ram_word!(game, game_state) as i32 == kPose_28_FaceL_Crouch as i32
            && ram_word!(game, demo_set) == 0
        {
            enemy_instruction! {[(*E).base]; timer = 1u16, instruction = addr_kGunshipTop_Ilist_A5BE as u16}
        }
    } else if ram_word!(game, loading_game_state) as i32 == kGameState_34_CeresGoesBoom as i32 {
        (*E).base.y_pos = (ram_word!(game, samus_y_pos)).wrapping_add(23u16);
    } else {
        (*E).base.y_pos = ((*E).base.y_pos).wrapping_add(15u16);
        (*E).gbm_var_D = 71u16;
    }
    (*E).gbm_var_F = ((0xa2a7d7i32) as u16);
}
pub unsafe fn GunshipTop_Main(game: &mut Game) {
    let E1 = Get_GunshipTop(ram_word!(game, cur_enemy_index) as i32 + 64i32, game);
    let v2: bool = (*E1).gtp_var_D == 1u16;
    (*E1).gtp_var_D = (*E1).gtp_var_D.wrapping_sub(1);
    let v3: bool = ((*E1).gtp_var_D & 0x8000) != 0;
    if v2 as i32 != 0 || v3 as i32 != 0 {
        QueueSfx2_Max6(0x4du16, game);
        (*E1).gtp_var_D = 70u16;
    }
    let E0 = Get_GunshipTop(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E0).gtp_var_F as i32 + 0x56bei32 & 0x8000i32 == 0
        && (*E0).gtp_var_F as i32 + 0x53e5i32 & 0x8000i32 != 0
    {
        GunshipTop_1(ram_word!(game, cur_enemy_index), game);
    }
    CallEnemyPreInstr((((*E0).gtp_var_F) as u32) | (0xa20000u32), game);
}
pub unsafe fn GunshipTop_1(k: u16, game: &mut Game) {
    let E0 = Get_GunshipTop(k as i32, game);
    let v3: bool = (*E0).gtp_var_D == 1u16;
    (*E0).gtp_var_D = (*E0).gtp_var_D.wrapping_sub(1);
    let v4: bool = ((*E0).gtp_var_D & 0x8000) != 0;
    if v3 as i32 != 0 || v4 as i32 != 0 {
        let v5: u16 = (2u16).wrapping_mul((*E0).gtp_var_C);
        (*E0).gtp_var_D = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa2a7cf)) as *mut u8)
            .offset(v5 as isize) as u16;
        let v6: u16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa2a7cf)) as *mut u8)
            .offset((v5 as i32 + 1i32) as isize) as i8 as u16;
        (*E0).base.y_pos = ((*E0).base.y_pos).wrapping_add(v6);
        let E1 = Get_GunshipTop(k as i32 + 64i32, game);
        (*E1).base.y_pos = ((*E1).base.y_pos).wrapping_add(v6);
        let E2 = Get_GunshipTop(k as i32 + 128i32, game);
        (*E2).base.y_pos = ((*E2).base.y_pos).wrapping_add(v6);
        (*E0).gtp_var_C = ((*E0).gtp_var_C as i32 + 1i32 & 3i32) as u16;
    }
}
pub unsafe fn GunshipTop_2(k: u16, game: &mut Game) {
    update_ram_word!(game.samus_y_pos, wrapping_sub(8u16));
    let E2 = Get_GunshipTop(k as i32 + 128i32, game);
    (*E2).base.y_pos = ((*E2).base.y_pos).wrapping_sub(8u16);
    let E1 = Get_GunshipTop(k as i32 + 64i32, game);
    (*E1).base.y_pos = ((*E1).base.y_pos).wrapping_sub(8u16);
    let E0 = Get_GunshipTop(k as i32, game);
    let v4: u16 = ((*E0).base.y_pos).wrapping_sub(8u16);
    (*E0).base.y_pos = v4;
    if v4 as i32 - 128i32 & 0x8000i32 != 0 {
        (*E0).gtp_var_F = ((0xa2a80ci32) as u16);
    }
}
pub unsafe fn GunshipTop_3(k: u16, game: &mut Game) {
    let E = Get_GunshipTop(k as i32, game);
    if (*E).base.y_pos as i32 - 768i32 & 0x8000i32 != 0 {
        AddToHiLo(
            ram_ptr!(game, u16; 0xafa),
            ram_ptr!(game, u16; 0xafc),
            0x48000u32,
        );
        let v3 = Get_GunshipTop(k as i32 + 128i32, game);
        AddToHiLo(
            &raw mut (*v3).base.y_pos,
            &raw mut (*v3).base.y_subpos,
            0x48000u32,
        );
        let v5 = Get_GunshipTop(k as i32 + 64i32, game);
        AddToHiLo(
            &raw mut (*v5).base.y_pos,
            &raw mut (*v5).base.y_subpos,
            0x48000u32,
        );
        AddToHiLo(
            &raw mut (*E).base.y_pos,
            &raw mut (*E).base.y_subpos,
            0x48000u32,
        );
    } else {
        AddToHiLo(
            ram_ptr!(game, u16; 0xafa),
            ram_ptr!(game, u16; 0xafc),
            0x28000u32,
        );
        let v8 = Get_GunshipTop(k as i32 + 128i32, game);
        AddToHiLo(
            &raw mut (*v8).base.y_pos,
            &raw mut (*v8).base.y_subpos,
            0x28000u32,
        );
        let v10 = Get_GunshipTop(k as i32 + 64i32, game);
        AddToHiLo(
            &raw mut (*v10).base.y_pos,
            &raw mut (*v10).base.y_subpos,
            0x28000u32,
        );
        AddToHiLo(
            &raw mut (*E).base.y_pos,
            &raw mut (*E).base.y_subpos,
            0x28000u32,
        );
        if (*E).base.y_pos as i32 - 1119i32 & 0x8000i32 == 0 {
            (*E).base.y_pos = 1119u16;
            (*v10).base.y_pos = 1159u16;
            (*v8).base.y_pos = ((*E).base.y_pos).wrapping_sub(1u16);
            (*E).gtp_var_F = ((0xa2a8d0i32) as u16);
            (*E).gtp_var_E = 0u16;
        }
    };
}
pub unsafe fn GunshipTop_4(k: u16, game: &mut Game) {
    let GunshipTop = Get_GunshipTop(k as i32, game);
    let mut v2: i32 = (*GunshipTop).gtp_var_E as i32;
    update_ram_word!(
        game.samus_y_pos,
        wrapping_add(crate::rtl::rom_table_word!(game, 0xa2a622, v2 as isize))
    );
    let v3 = Get_GunshipTop(k as i32 + 128i32, game);
    (*v3).base.y_pos =
        ((*v3).base.y_pos).wrapping_add(crate::rtl::rom_table_word!(game, 0xa2a622, v2 as isize));
    let v4 = Get_GunshipTop(k as i32 + 64i32, game);
    (*v4).base.y_pos =
        ((*v4).base.y_pos).wrapping_add(crate::rtl::rom_table_word!(game, 0xa2a622, v2 as isize));
    (*GunshipTop).base.y_pos = ((*GunshipTop).base.y_pos)
        .wrapping_add(crate::rtl::rom_table_word!(game, 0xa2a622, v2 as isize));
    let mut v5: u16 = ((*GunshipTop).gtp_var_E).wrapping_add(1u16);
    (*GunshipTop).gtp_var_E = v5;
    if v5 as i32 - 17i32 & 0x8000i32 == 0 {
        (*GunshipTop).gtp_var_F = ((0xa2a942i32) as u16);
        (*GunshipTop).gtp_var_E = (*GunshipTop).base.y_pos;
        (*GunshipTop).gtp_var_D = 1u16;
        let v6 = Get_GunshipTop(0i32, game);
        (*v6).gtp_var_C = 0u16;
        set_ram_word!(game.samus_x_pos = ((*GunshipTop).base.x_pos).wrapping_add(1u16));
        set_ram_word!(game.samus_prev_x_pos = ram_word!(game, samus_x_pos));
        let v7 = Get_GunshipTop(k as i32 + 128i32, game);
        enemy_instruction! {[(*v7).base]; timer = 1u16, instruction = addr_kGunshipTop_Ilist_A5BE as u16}
        (*v6).gtp_var_A = 144u16;
        QueueSfx3_Max6(0x14u16, game);
    }
}
pub unsafe fn GunshipTop_5(k: u16, game: &mut Game) {
    let E = Get_GunshipTop(0i32, game);
    let v2: bool = (*E).gtp_var_A == 1u16;
    (*E).gtp_var_A = (*E).gtp_var_A.wrapping_sub(1);
    let v3: bool = ((*E).gtp_var_A & 0x8000) != 0;
    if v2 as i32 != 0 || v3 as i32 != 0 {
        (*Get_GunshipTop(k as i32, game)).gtp_var_F = ((0xa2a950i32) as u16);
    }
}
pub unsafe fn GunshipTop_6(k: u16, game: &mut Game) {
    let E = Get_GunshipTop(k as i32, game);
    let r18: u16 = ((*E).gtp_var_E).wrapping_sub(30u16);
    if update_ram_word!(game.samus_y_pos, wrapping_sub(1), reread) as i32 - r18 as i32 & 0x8000i32
        != 0
    {
        (*E).gtp_var_F = ((0xa2a987i32) as u16);
        let E2 = Get_GunshipTop(k as i32 + 128i32, game);
        enemy_instruction! {[(*E2).base]; timer = 1u16, instruction = addr_kGunshipTop_Ilist_A5EE as u16}
        (*Get_GunshipTop(0i32, game)).gtp_var_A = 144u16;
        QueueSfx3_Max6(0x15u16, game);
    }
}
pub unsafe fn GunshipTop_7(k: u16, game: &mut Game) {
    let E = Get_GunshipTop(0i32, game);
    let v2: bool = (*E).gtp_var_A == 1u16;
    (*E).gtp_var_A = (*E).gtp_var_A.wrapping_sub(1);
    let v3: bool = ((*E).gtp_var_A & 0x8000) != 0;
    if v2 as i32 != 0 || v3 as i32 != 0 {
        (*Get_GunshipTop(k as i32, game)).gtp_var_F = ((0xa2a9bdi32) as u16);
        set_ram_word!(game.frame_handler_alfa = ((0x90e695i32) as u16));
        set_ram_word!(game.frame_handler_beta = ((0x90e725i32) as u16));
        set_ram_word!(game.loading_game_state = kLoadingGameState_5_Main as u16);
        crate::update_place! { reference fresh11 = *(ram_ptr!(game, u16; 0xd8f8)) => (*fresh11) | (1u16); }
        set_ram_word!(game.load_station_index = 0u16);
        SaveToSram(ram_word!(game, selected_save_slot), game);
    }
}
pub unsafe fn GunshipTop_8(k: u16, game: &mut Game) {
    if ram_word!(game, game_state) == 8u16
        && ram_word!(game, frame_handler_alfa) as i32 == 0x90e695i32 & 0xffffi32
    {
        let E = Get_GunshipTop(k as i32, game);
        if (((*E).base.x_pos as i32 - 8i32 - ram_word!(game, samus_x_pos) as i32) as i16) < 0i16
            && (((*E).base.x_pos as i32 + 8i32 - ram_word!(game, samus_x_pos) as i32) as i16)
                >= 0i16
            && (((*E).base.y_pos as i32 - 64i32 - ram_word!(game, samus_y_pos) as i32) as i16)
                < 0i16
            && (((*E).base.y_pos) as i16).wrapping_sub((ram_word!(game, samus_y_pos)) as i16)
                >= 0i16
            && *ram_ptr!(game; 0xa1f) == 0
            && (ram_word!(game, joypad1_newkeys) as i32 & kButton_Down as i32) != 0i32
        {
            (*E).gtp_var_F = ((0xa2aa4fi32) as u16);
            if ram_word!(game, samus_x_pos) != 1152u16 {
                set_ram_word!(game.samus_x_pos = (*E).base.x_pos);
                set_ram_word!(game.samus_prev_x_pos = ram_word!(game, samus_x_pos));
            }
            MakeSamusFaceForward(game);
            CallSomeSamusCode(0x1au16, game);
            set_ram_word!(game.elevator_status = 0u16);
            let v3: u16 = ((*E).base.y_pos).wrapping_sub(1u16);
            let E2 = Get_GunshipTop(k as i32 + 128i32, game);
            (*E2).base.y_pos = v3;
            enemy_instruction! {[(*E2).base]; timer = 1u16, instruction = addr_kGunshipTop_Ilist_A5BE as u16}
            (*Get_GunshipTop(0i32, game)).gtp_var_A = 144u16;
            QueueSfx3_Max6(0x14u16, game);
        }
    }
}
pub unsafe fn GunshipTop_9(k: u16, game: &mut Game) {
    let E = Get_GunshipTop(0i32, game);
    let v2: bool = (*E).gtp_var_A == 1u16;
    (*E).gtp_var_A = (*E).gtp_var_A.wrapping_sub(1);
    let v3: bool = ((*E).gtp_var_A & 0x8000) != 0;
    if v2 as i32 != 0 || v3 as i32 != 0 {
        (*Get_GunshipTop(k as i32, game)).gtp_var_F = ((0xa2aa5di32) as u16);
    }
}
pub unsafe fn GunshipTop_10(k: u16, game: &mut Game) {
    let E = Get_GunshipTop(k as i32, game);
    let r18: u16 = ((*E).gtp_var_E).wrapping_add(18u16);
    update_ram_word!(game.samus_y_pos, wrapping_add(2u16));
    if ram_word!(game, samus_y_pos) as i32 - r18 as i32 & 0x8000i32 == 0 {
        (*E).gtp_var_F = ((0xa2aa94i32) as u16);
        let E2 = Get_GunshipTop(k as i32 + 128i32, game);
        enemy_instruction! {[(*E2).base]; timer = 1u16, instruction = addr_kGunshipTop_Ilist_A5EE as u16}
        (*Get_GunshipTop(0i32, game)).gtp_var_A = 144u16;
        QueueSfx3_Max6(0x15u16, game);
    }
}
pub unsafe fn GunshipTop_11(k: u16, game: &mut Game) {
    let E = Get_GunshipTop(0i32, game);
    let v2: bool = (*E).gtp_var_A == 1u16;
    (*E).gtp_var_A = (*E).gtp_var_A.wrapping_sub(1);
    let v3: bool = ((*E).gtp_var_A & 0x8000) != 0;
    if v2 as i32 != 0 || v3 as i32 != 0 {
        (*Get_GunshipTop(k as i32, game)).gtp_var_F = ((0xa2aaa2i32) as u16);
    }
}
pub unsafe fn GunshipTop_12(k: u16, game: &mut Game) {
    if CheckEventHappened(0xeu16, game) != 0 {
        (*Get_GunshipTop(k as i32, game)).gtp_var_F = ((0xa2abc7i32) as u16);
        let E = Get_GunshipTop(k as i32 + 64i32, game);
        set_fields! {[(*E)], 0u16; gtp_var_F, gtp_var_E}
        set_ram_words! { game, 0u16; substate, suit_pickup_light_beam_pos }
        *(ram_ptr!(game, u16; 0xdf0)) = 0u16;
        *(ram_ptr!(game, u16; 0xdf2)) = 0u16;
        CallSomeSamusCode(0xau16, game);
    } else {
        Samus_RestoreHealth(2u16, game);
        Samus_RestoreMissiles(2u16, game);
        Samus_RestoreSuperMissiles(2u16, game);
        Samus_RestorePowerBombs(2u16, game);
        if ((ram_word!(game, samus_reserve_health)) as i16)
            .wrapping_sub((ram_word!(game, samus_max_reserve_health)) as i16)
            >= 0i16
            && ((ram_word!(game, samus_health)) as i16)
                .wrapping_sub((ram_word!(game, samus_max_health)) as i16)
                >= 0i16
            && ((ram_word!(game, samus_missiles)) as i16)
                .wrapping_sub((ram_word!(game, samus_max_missiles)) as i16)
                >= 0i16
            && ((ram_word!(game, samus_super_missiles)) as i16)
                .wrapping_sub((ram_word!(game, samus_max_super_missiles)) as i16)
                >= 0i16
            && ((ram_word!(game, samus_power_bombs)) as i16)
                .wrapping_sub((ram_word!(game, samus_max_power_bombs)) as i16)
                >= 0i16
        {
            (*Get_GunshipTop(k as i32, game)).gtp_var_F = ((0xa2ab1fi32) as u16);
        }
    };
}
pub unsafe fn GunshipTop_13(k: u16, game: &mut Game) {
    let rv: i32 = DisplayMessageBox_Poll(0x1cu16, game);
    if rv < 0i32 {
        return;
    }
    if rv != 2i32 {
        crate::update_place! { reference fresh3 = *(ram_ptr!(game, u16; 0xd8f8)) => (*fresh3) | (1u16); }
        set_ram_word!(game.load_station_index = 0u16);
        SaveToSram(ram_word!(game, selected_save_slot), game);
    }
    (*Get_GunshipTop(k as i32, game)).gtp_var_F = ((0xa2ab60i32) as u16);
    let E = Get_GunshipTop(k as i32 + 128i32, game);
    enemy_instruction! {[(*E).base]; timer = 1u16, instruction = addr_kGunshipTop_Ilist_A5BE as u16}
    (*Get_GunshipTop(0i32, game)).gtp_var_A = 144u16;
    QueueSfx3_Max6(0x14u16, game);
}
pub unsafe fn GunshipTop_14(k: u16, game: &mut Game) {
    let E = Get_GunshipTop(0i32, game);
    let v2: bool = (*E).gtp_var_A == 1u16;
    (*E).gtp_var_A = (*E).gtp_var_A.wrapping_sub(1);
    let v3: bool = ((*E).gtp_var_A & 0x8000) != 0;
    if v2 as i32 != 0 || v3 as i32 != 0 {
        (*Get_GunshipTop(k as i32, game)).gtp_var_F = ((0xa2ab6ei32) as u16);
    }
}
pub unsafe fn GunshipTop_15(k: u16, game: &mut Game) {
    let E = Get_GunshipTop(k as i32, game);
    let r18: u16 = ((*E).gtp_var_E).wrapping_sub(30u16);
    update_ram_word!(game.samus_y_pos, wrapping_sub(2u16));
    if ram_word!(game, samus_y_pos) as i32 - r18 as i32 & 0x8000i32 != 0 {
        (*E).gtp_var_F = ((0xa2aba5i32) as u16);
        let E2 = Get_GunshipTop(k as i32 + 128i32, game);
        enemy_instruction! {[(*E2).base]; timer = 1u16, instruction = addr_kGunshipTop_Ilist_A5EE as u16}
        (*Get_GunshipTop(0i32, game)).gtp_var_A = 144u16;
        QueueSfx3_Max6(0x15u16, game);
    }
}
pub unsafe fn GunshipTop_16(k: u16, game: &mut Game) {
    let E = Get_GunshipTop(0i32, game);
    let v2: bool = (*E).gtp_var_A == 1u16;
    (*E).gtp_var_A = (*E).gtp_var_A.wrapping_sub(1);
    let v3: bool = ((*E).gtp_var_A & 0x8000) != 0;
    if v2 as i32 != 0 || v3 as i32 != 0 {
        (*Get_GunshipTop(k as i32, game)).gtp_var_F = ((0xa2a9bdi32) as u16);
        if ram_word!(game, game_state) as i32 - kGameState_40_TransitionToDemo as i32 & 0x8000i32
            != 0
        {
            set_ram_word!(game.frame_handler_alfa = ((0x90e695i32) as u16));
            set_ram_word!(game.frame_handler_beta = ((0x90e725i32) as u16));
        }
    }
}
pub unsafe fn GunshipTop_17(k: u16, game: &mut Game) {
    let v1: u16 = ram_word!(game, substate);
    let mut v2: u16 = ram_word!(game, vram_write_queue_tail);
    (*gVramWriteEntry(ram_word!(game, vram_write_queue_tail) as i32, game)).size = 1024u16;
    v2 = (v2).wrapping_add(2u16);
    let v3: i32 = v1 as i32 >> 1i32;
    (*gVramWriteEntry(v2 as i32, game)).size =
        crate::rtl::rom_table_word!(game, 0xa2ac07, v3 as isize);
    v2 = (v2).wrapping_add(2u16);
    crate::update_place! { previous fresh5 = v2 => v2.wrapping_add(1); }
    *(&raw mut (*(gVramWriteEntry as unsafe fn(i32, &mut Game) -> *mut VramWriteEntry)(
        fresh5 as i32,
        game,
    ))
    .size as *mut u8)
        .offset(0) = -108i32 as u8;
    (*gVramWriteEntry(v2 as i32, game)).size =
        crate::rtl::rom_table_word!(game, 0xa2ac11, v3 as isize);
    set_ram_word!(game.vram_write_queue_tail = (v2).wrapping_add(2u16));
    update_ram_word!(game.substate, wrapping_add(2u16));
    if ram_word!(game, substate) as i32 - 10i32 & 0x8000i32 == 0 {
        (*Get_GunshipTop(k as i32, game)).gtp_var_F = ((0xa2ac1bi32) as u16);
        set_ram_word!(game.substate = 0u16);
    }
}
pub unsafe fn GunshipTop_18(k: u16, game: &mut Game) {
    let E1 = Get_GunshipTop(k as i32 + 64i32, game);
    let gtp_var_E: u16 = (*E1).gtp_var_E;
    if gtp_var_E as i32 - 64i32 & 0x8000i32 != 0 {
        let mut v3: u16 = 0;
        if (gtp_var_E & 1) != 0 {
            v3 = (ram_word!(game, samus_y_pos)).wrapping_add(1u16);
        } else {
            v3 = (ram_word!(game, samus_y_pos)).wrapping_sub(1u16);
        }
        set_ram_word!(game.samus_y_pos = v3);
        set_ram_word!(game.samus_prev_y_pos = v3);
    } else {
        let mut v4: u16 = 0;
        if (gtp_var_E & 1) != 0 {
            v4 = (ram_word!(game, samus_y_pos)).wrapping_add(2u16);
        } else {
            v4 = (ram_word!(game, samus_y_pos)).wrapping_sub(2u16);
        }
        set_ram_word!(game.samus_y_pos = v4);
        set_ram_word!(game.samus_prev_y_pos = v4);
    }
    let mut v5: i16 = ((ram_word!(game, samus_y_pos)) as i16).wrapping_sub(17i16);
    let E0 = Get_GunshipTop(k as i32, game);
    (*E0).base.y_pos = (ram_word!(game, samus_y_pos)).wrapping_sub(17u16);
    (*Get_GunshipTop(k as i32 + 128i32, game)).base.y_pos = ((v5) as u16).wrapping_sub(1u16);
    (*E1).base.y_pos = (ram_word!(game, samus_y_pos)).wrapping_add(23u16);
    let v7: u16 = ((*E1).gtp_var_E).wrapping_add(1u16);
    (*E1).gtp_var_E = v7;
    if v7 as i32 - 128i32 & 0x8000i32 != 0 {
        if v7 == 64u16 {
            SpawnEprojWithRoomGfx(addr_stru_86A379 as u16, 0u16, game);
            SpawnEprojWithRoomGfx(addr_stru_86A379 as u16, 2u16, game);
            SpawnEprojWithRoomGfx(addr_stru_86A379 as u16, 4u16, game);
            SpawnEprojWithRoomGfx(addr_stru_86A379 as u16, 6u16, game);
            SpawnEprojWithRoomGfx(addr_stru_86A379 as u16, 8u16, game);
            SpawnEprojWithRoomGfx(addr_stru_86A379 as u16, 0xau16, game);
        }
    } else {
        (*E0).gtp_var_F = ((0xa2acd7i32) as u16);
        (*Get_GunshipTop(0i32, game)).gtp_var_A = 0u16;
    };
}
pub unsafe fn GunshipTop_19(k: u16, game: &mut Game) {
    update_ram_word!(game.samus_y_pos, wrapping_sub(2u16));
    let mut v1: i16 = ((ram_word!(game, samus_y_pos)) as i16).wrapping_sub(17i16);
    let E0 = Get_GunshipTop(k as i32, game);
    (*E0).base.y_pos = (ram_word!(game, samus_y_pos)).wrapping_sub(17u16);
    (*Get_GunshipTop(k as i32 + 128i32, game)).base.y_pos = ((v1) as u16).wrapping_sub(1u16);
    let E1 = Get_GunshipTop(k as i32 + 64i32, game);
    (*E1).base.y_pos = (ram_word!(game, samus_y_pos)).wrapping_add(23u16);
    if (*E0).base.y_pos as i32 - 896i32 & 0x8000i32 != 0 {
        (*E0).gtp_var_F = ((0xa2ad0ei32) as u16);
        (*E1).gtp_var_F = 512u16;
    }
}
pub unsafe fn GunshipTop_20(k: u16, game: &mut Game) {
    GunshipTop_21(k, game);
    let E = Get_GunshipTop(k as i32, game);
    if (*E).base.y_pos as i32 - 256i32 & 0x8000i32 != 0 {
        (*E).gtp_var_F = ((0xa2ad2di32) as u16);
        set_ram_word!(game.game_state = 38u16);
        set_ram_words! { game, 0u16; screen_fade_delay, screen_fade_counter }
    }
}
pub unsafe fn GunshipTop_21(k: u16, game: &mut Game) {
    let E = Get_GunshipTop(k as i32 + 64i32, game);
    let v3: u16 = ((*E).gtp_var_F).wrapping_add(64u16);
    (*E).gtp_var_F = v3;
    if (v3 as i32 & 0xff00i32) - 2560i32 & 0x8000i32 == 0 {
        (*E).gtp_var_F = 2304u16;
    }
    let v6: u32 = ((ram_word!(game, samus_y_pos) as u32) << 16i32
        | ram_word!(game, samus_y_subpos) as u32)
        .wrapping_sub(
            (((*E).gtp_var_F >> 8) as u32) << 16i32
                | (((*E).gtp_var_F as i32) << 8i32) as u16 as u32,
        );
    set_ram_word!(game.samus_y_subpos = v6 as u16);
    set_ram_word!(game.samus_y_pos = (v6 >> 16i32) as u16);
    let v5: u16 = (ram_word!(game, samus_y_pos)).wrapping_sub(17u16);
    (*Get_GunshipTop(k as i32, game)).base.y_pos = v5;
    (*Get_GunshipTop(k as i32 + 128i32, game)).base.y_pos = (v5).wrapping_sub(1u16);
    (*E).base.y_pos = (ram_word!(game, samus_y_pos)).wrapping_add(23u16);
}
pub unsafe fn Flies_Init(game: &mut Game) {
    let E = Get_Flies(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).flies_var_E = 0u16;
    (*E).flies_var_F = ((0xa2b14ei32) as u16);
    (*E).base.current_instruction = addr_kFlies_Ilist_B013 as u16;
    (*E).base.spritemap_pointer = addr_kSpritemap_Nothing_A2 as u16;
    (*E).base.instruction_timer = 1u16;
    (*E).base.spritemap_pointer = addr_kSpritemap_Nothing_A2 as u16;
}
pub unsafe fn CarryAdd16(rv: *mut u16, v: u16) -> bool {
    let i: i32 = *rv as i32 + v as i32;
    *rv = i as u16;
    i >> 16i32 != 0
}
pub unsafe fn Flies_1(k: u16, game: &mut Game) {
    let E = Get_Flies(k as i32, game);
    let v2: i32 = (*E).flies_var_E as i32 >> 1i32;
    let mut carry: bool = CarryAdd16(
        &raw mut (*E).base.x_subpos,
        ((*((&raw const kSinCosTable8bit_Sext as *const i16).offset((v2 + 64i32) as isize)
            as *mut u8)
            .offset(0) as i32)
            << 8i32) as u16,
    );
    (*E).base.x_pos = ((*E).base.x_pos as i32
        + (*((&raw const kSinCosTable8bit_Sext as *const i16).offset((v2 + 64i32) as isize)
            as *mut u8)
            .offset(1isize) as i8 as i32
            + carry as i32)) as u16;
    carry = CarryAdd16(
        &raw mut (*E).base.y_subpos,
        ((*((&raw const kSinCosTable8bit_Sext as *const i16).offset(v2 as isize) as *mut u8)
            .offset(0) as i32)
            << 8i32) as u16,
    );
    (*E).base.y_pos = ((*E).base.y_pos as i32
        + (*((&raw const kSinCosTable8bit_Sext as *const i16).offset(v2 as isize) as *mut u8)
            .offset(1isize) as i8 as i32
            + carry as i32)) as u16;
}
pub unsafe fn Flies_2(k: u16, game: &mut Game) -> u16 {
    let E = Get_Flies(k as i32, game);
    let mut carry: bool = CarryAdd16(
        &raw mut (*E).base.x_subpos,
        ((*(&raw mut (*E).flies_var_B as *mut u8).offset(0) as i32) << 8i32) as u16,
    );
    (*E).base.x_pos = ((*E).base.x_pos as i32
        + (*(&raw mut (*E).flies_var_B as *mut u8).offset(1isize) as i8 as i32 + carry as i32))
        as u16;
    carry = CarryAdd16(
        &raw mut (*E).base.y_subpos,
        ((*(&raw mut (*E).flies_var_C as *mut u8).offset(0) as i32) << 8i32) as u16,
    );
    let result: u16 = ((*E).base.y_pos as i32
        + *(&raw mut (*E).flies_var_C as *mut u8).offset(1isize) as i8 as i32
        + carry as i32) as u16;
    (*E).base.y_pos = result;
    result
}
pub unsafe fn Flies_Main(game: &mut Game) {
    NextRandom(game);
    let E = Get_Flies(ram_word!(game, cur_enemy_index) as i32, game);
    CallEnemyPreInstr((((*E).flies_var_F) as u32) | (0xa20000u32), game);
}
pub unsafe fn Flies_3(k: u16, game: &mut Game) {
    let v1: i32 = CalculateAngleOfSamusFromEnemy(k, game) as i32;
    let E = Get_Flies(k as i32, game);
    (*E).flies_var_B = (2u16).wrapping_mul((kSinCosTable8bit_Sext[(v1 + 64i32) as usize]) as u16);
    (*E).flies_var_C = (4u16).wrapping_mul((kSinCosTable8bit_Sext[v1 as usize]) as u16);
    (*E).flies_var_D = ram_word!(game, samus_y_pos);
    (*E).flies_var_F = ((0xa2b1aai32) as u16);
}
pub unsafe fn Flies_4(k: u16, game: &mut Game) {
    let E = Get_Flies(k as i32, game);
    let flies_var_A: u16 = (*E).flies_var_A;
    if flies_var_A != 0 {
        (*E).flies_var_A = (flies_var_A).wrapping_sub(1u16);
    } else if CompareDistToSamus_X(k, 0x70u16, game) & 1 == 0 {
        Flies_3(k, game);
        return;
    }
    Flies_1(k, game);
    let v3: bool = ((*E).flies_var_E as i32 + 32i32 & 0x1ffi32) == 0i32;
    (*E).flies_var_E = ((*E).flies_var_E as i32 + 32i32 & 0x1ffi32) as u16;
    if v3 {
        (*E).flies_var_F = ((0xa2b17ci32) as u16);
    }
}
pub unsafe fn Flies_5(k: u16, game: &mut Game) {
    let E = Get_Flies(k as i32, game);
    let flies_var_A: u16 = (*E).flies_var_A;
    if flies_var_A != 0 {
        (*E).flies_var_A = (flies_var_A).wrapping_sub(1u16);
    } else if CompareDistToSamus_X(k, 0x70u16, game) & 1 == 0 {
        Flies_3(k, game);
        return;
    }
    Flies_1(k, game);
    let v3: bool = ((*E).flies_var_E as i32 - 32i32 & 0x1ffi32) == 0i32;
    (*E).flies_var_E = ((*E).flies_var_E as i32 - 32i32 & 0x1ffi32) as u16;
    if v3 {
        (*E).flies_var_F = ((0xa2b14ei32) as u16);
    }
}
pub unsafe fn Flies_6(k: u16, game: &mut Game) {
    let v1: u16 = Flies_2(k, game);
    let E = Get_Flies(k as i32, game);
    (*E).flies_var_A = (*E).flies_var_A.wrapping_add(1);
    let reverse_direction: bool = if ((*E).flies_var_C & 0x8000) == 0 {
        if v1 < (*E).flies_var_D {
            return;
        }
        true
    } else {
        (v1 < (*E).flies_var_D)
    };
    if reverse_direction {
        (*E).flies_var_C = -((*E).flies_var_C as i32) as u16;
        (*E).flies_var_F = ((0xa2b1d2i32) as u16);
    };
}
pub unsafe fn Flies_7(k: u16, game: &mut Game) {
    Flies_2(k, game);
    let E = Get_Flies(k as i32, game);
    (*E).flies_var_A = (*E).flies_var_A.wrapping_sub(1);
    if ((*E).flies_var_A & 0x8000) != 0 {
        (*E).flies_var_A = 24u16;
        (*E).flies_var_F = ((0xa2b14ei32) as u16);
    }
}
pub unsafe fn NorfairErraticFireball_Init(game: &mut Game) {
    let E = Get_NorfairErraticFireball(ram_word!(game, cur_enemy_index) as i32, game);
    let pt: Point32 = ConvertAngleToXy(
        (*E).nefl_parameter_1,
        *(&raw mut (*E).nefl_parameter_2 as *mut u8).offset(0) as u16,
        game,
    );
    (*E).nefl_var_C = (pt.x >> 16i32) as u16;
    (*E).nefl_var_D = pt.x as u16;
    (*E).nefl_var_E = (pt.y >> 16i32) as u16;
    (*E).nefl_var_F = pt.y as u16;
    (*E).base.current_instruction = addr_kNorfairErraticFireball_Ilist_B2DC as u16;
}
pub unsafe fn NorfairErraticFireball_Main(game: &mut Game) {
    let E = Get_NorfairErraticFireball(ram_word!(game, cur_enemy_index) as i32, game);
    let pt: Point32 = ConvertAngleToXy(
        (*E).nefl_parameter_1,
        *(&raw mut (*E).nefl_parameter_2 as *mut u8).offset(0) as u16,
        game,
    );
    (*E).nefl_var_C = (pt.x >> 16i32) as u16;
    (*E).nefl_var_D = pt.x as u16;
    (*E).nefl_var_E = (pt.y >> 16i32) as u16;
    (*E).nefl_var_F = pt.y as u16;
    if ((*E).nefl_parameter_1 as i32 + 64i32 & 0x80i32) == 0i32 {
        (*E).nefl_var_C = !((*E).nefl_var_C as i32) as u16;
        (*E).nefl_var_D = -((*E).nefl_var_D as i32) as u16;
    }
    if Enemy_MoveRight_IgnoreSlopes(
        ram_word!(game, cur_enemy_index),
        (((*E).nefl_var_C as u32) << 16i32 | (*E).nefl_var_D as u32) as i32,
        game,
    ) != 0
    {
        (*E).nefl_parameter_1 = ((*E).nefl_parameter_1) ^ (0x40u16);
    } else {
        if ((*E).nefl_parameter_1 as i32 + 128i32 & 0x80i32) == 0i32 {
            (*E).nefl_var_E = !((*E).nefl_var_E as i32) as u16;
            (*E).nefl_var_F = -((*E).nefl_var_F as i32) as u16;
        }
        if Enemy_MoveDown(
            ram_word!(game, cur_enemy_index),
            (((*E).nefl_var_E as u32) << 16i32 | (*E).nefl_var_F as u32) as i32,
            game,
        ) != 0
        {
            (*E).nefl_parameter_1 = ((*E).nefl_parameter_1) ^ (0xc0u16);
        }
    };
}
pub unsafe fn LavaquakeRocks_Init(game: &mut Game) {
    let E = Get_LavaquakeRocks(ram_word!(game, cur_enemy_index) as i32, game);
    enemy_instruction! {[(*E).base]; instruction = addr_kNorfairErraticFireball_Ilist_B51A as u16, timer = 1u16}
    (*E).base.timer = 0u16;
    (*E).lrs_var_A = ((0xa2b596i32) as u16);
    set_ram_word!(game.random_number = 17u16);
}
crate::game::address_dispatch! {
    pub unsafe fn CallLavaQuakeRocksFunc(ea: u32, game: &mut Game) => ea;
        call (game); default { Unreachable(game); };
    {
        10663318 => LavaquakeRocks_1; 10663346 => LavaquakeRocks_2; 10663402 => LavaquakeRocks_3;
    }
}
pub unsafe fn LavaquakeRocks_Main(game: &mut Game) {
    let E = Get_LavaquakeRocks(ram_word!(game, cur_enemy_index) as i32, game);
    CallLavaQuakeRocksFunc((((*E).lrs_var_A) as u32) | (0xa20000u32), game);
}
pub unsafe fn LavaquakeRocks_1(game: &mut Game) {
    if IsSamusWithinEnemy_X(ram_word!(game, cur_enemy_index), 0x40u16, game) != 0
        && IsSamusWithinEnemy_Y(ram_word!(game, cur_enemy_index), 0x40u16, game) != 0
    {
        (*Get_LavaquakeRocks(ram_word!(game, cur_enemy_index) as i32, game)).lrs_var_A =
            ((0xa2b5b2i32) as u16);
    }
}
pub unsafe fn LavaquakeRocks_2(game: &mut Game) {
    set_ram_word!(
        game.cinematic_var9 = crate::rtl::rom_table_word!(
            game,
            0xa2b550,
            ((NextRandom(game) as i32 & 0x1ei32) >> 1i32) as isize,
        )
    );
    let v1: i32 = (NextRandom(game) as i32 & 0x1ei32) >> 1i32;
    SpawnEprojWithGfx(
        crate::rtl::rom_table_word!(game, 0xa2b530, v1 as isize),
        ram_word!(game, cur_enemy_index),
        addr_stru_86BD5A as u16,
        game,
    );
    let E = Get_LavaquakeRocks(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).lrs_var_A = ((0xa2b5eai32) as u16);
    (*E).lrs_var_B = crate::rtl::rom_table_word!(
        game,
        0xa2b520,
        (((NextRandom(game)) & (0xeu16)) >> 1) as isize
    );
}
pub unsafe fn LavaquakeRocks_3(game: &mut Game) {
    let E = Get_LavaquakeRocks(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).lrs_var_B = (*E).lrs_var_B.wrapping_sub(1);
    if ((*E).lrs_var_B & 0x8000) != 0 {
        (*E).lrs_var_A = ((0xa2b596i32) as u16);
    }
}
pub unsafe fn Rinka_Init(game: &mut Game) {
    let E = Get_Rinka(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).rinka_parameter_1 != 0 {
        Rinka_1(ram_word!(game, cur_enemy_index), game);
        (*E).base.properties = ((*E).base.properties as i32
            & !(kEnemyProps_RespawnIfKilled as i32
                | kEnemyProps_DisableSamusColl as i32
                | kEnemyProps_ProcessedOffscreen as i32
                | kEnemyProps_Tangible as i32)
            | 0x2c00i32) as u16;
    } else {
        (*E).base.properties = ((*E).base.properties as i32
            & !(kEnemyProps_RespawnIfKilled as i32
                | kEnemyProps_DisableSamusColl as i32
                | kEnemyProps_ProcessedOffscreen as i32
                | kEnemyProps_Tangible as i32)
            | 0x6400i32) as u16;
    }
    (*E).base.palette_index = 1024u16;
    Rinka_Init3(ram_word!(game, cur_enemy_index), game);
}
pub unsafe fn Rinka_Init2(k: u16, game: &mut Game) {
    let E = Get_Rinka(k as i32, game);
    if (*E).rinka_parameter_1 != 0 {
        Rinka_1(k, game);
    }
    let mut v2 = gEnemySpawnData(k as i32, game);
    (*E).base.x_pos = (*v2).x_pos;
    (*E).base.y_pos = (*v2).y_pos;
    Rinka_Init3(k, game);
}
pub unsafe fn Rinka_Init3(k: u16, game: &mut Game) {
    let E = Get_Rinka(k as i32, game);
    (*E).rinka_var_A = ((0xa2b852i32) as u16);
    (*E).rinka_var_F = 26u16;
    set_fields! {[(*E)], 0u16; rinka_var_B, rinka_var_C}
    if (*E).rinka_parameter_1 != 0 {
        if (*Get_Rinka(0i32, game)).rinka_var_1D != 0 {
            (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Deleted) as u16);
        } else {
            enemy_instruction! {[(*E).base]; instruction = addr_kRinka_Ilist_BA0C as u16, timer = 1u16}
            (*E).base.timer = 0u16;
        }
    } else {
        enemy_instruction! {[(*E).base]; instruction = addr_kRinka_Ilist_B9E0 as u16, timer = 1u16}
        (*E).base.timer = 0u16;
    };
}
pub unsafe fn Rinka_1(k: u16, game: &mut Game) {
    let mut v2: u16 = 0;
    let ES = gEnemySpawnData(k as i32, game);
    if Rinka_10((*ES).x_pos, (*ES).y_pos, game) as i32 != 0 || {
        v2 = Rinka_2(k, game);
        (*gRam8000_Default(v2 as i32, game).offset(31)).var_3F & 1 != 0
    } {
        let mut v3: u16 = 0u16;
        loop {
            let v4: i32 = v3 as i32 >> 1i32;
            let x: u16 = crate::rtl::rom_table_word!(game, 0xa2b75b, v4 as isize);
            let y: u16 = crate::rtl::rom_table_word!(game, 0xa2b75b, (v4 + 1i32) as isize);
            if !Rinka_10(x, y, game)
                && (*gRam8000_Default(
                    crate::rtl::rom_table_word!(game, 0xa2b75b, (v4 + 2i32) as isize) as i32,
                    game,
                )
                .offset(31))
                .var_3F
                    & 1
                    == 0
            {
                let E = Get_Rinka(k as i32, game);
                (*ES).x_pos = x;
                (*E).base.x_pos = x;
                (*ES).y_pos = y;
                (*E).base.y_pos = y;
                let v9: u16 = crate::rtl::rom_table_word!(
                    game,
                    0xa2b75b,
                    ((v3 as i32 >> 1i32) + 2i32) as isize
                );
                (*gRam8000_Default(v9 as i32, game).offset(31)).var_3F = -1i32 as u16;
                (*E).rinka_var_D = v9;
                return;
            }
            v3 = (v3).wrapping_add(6u16);
            if (((v3) as i16).wrapping_sub(66i16) >= 0i16) {
                break;
            }
        }
        let mut v10: u16 = 0u16;
        while (*gRam8000_Default(
            crate::rtl::rom_table_word!(game, 0xa2b75b, ((v10 as i32 >> 1i32) + 2i32) as isize)
                as i32,
            game,
        )
        .offset(31))
        .var_3F
            & 1
            != 0
        {
            v10 = (v10).wrapping_add(6u16);
            if ((v10) as i16).wrapping_sub(66i16) >= 0i16 {
                return;
            }
        }
        let v11: i32 = v10 as i32 >> 1i32;
        let v12: u16 = crate::rtl::rom_table_word!(game, 0xa2b75b, v11 as isize);
        (*ES).x_pos = v12;
        let E_0 = Get_Rinka(k as i32, game);
        (*E_0).base.x_pos = v12;
        let v15: u16 = crate::rtl::rom_table_word!(game, 0xa2b75b, (v11 + 1i32) as isize);
        (*ES).y_pos = v15;
        (*E_0).base.y_pos = v15;
        let v16: u16 = crate::rtl::rom_table_word!(game, 0xa2b75b, (v11 + 2i32) as isize);
        (*E_0).rinka_var_D = v16;
        (*gRam8000_Default(v16 as i32, game).offset(31)).var_3F = -1i32 as u16;
    } else {
        (*Get_Rinka(k as i32, game)).rinka_var_D = v2;
        (*gRam8000_Default(v2 as i32, game).offset(31)).var_3F = -1i32 as u16;
    };
}
pub unsafe fn Rinka_2(k: u16, game: &mut Game) -> u16 {
    let mut v1: u16 = 0u16;
    loop {
        let v2: i32 = v1 as i32 >> 1i32;
        let mut v3 = gEnemySpawnData(k as i32, game);
        if crate::rtl::rom_table_word!(game, 0xa2b75b, v2 as isize) == (*v3).x_pos
            && crate::rtl::rom_table_word!(game, 0xa2b75b, (v2 + 1i32) as isize) == (*v3).y_pos
        {
            break;
        }
        v1 = (v1).wrapping_add(6u16);
        if v1 as i32 - 66i32 & 0x8000i32 == 0 {
            v1 = 0u16;
            return crate::rtl::rom_table_word!(
                game,
                0xa2b75b,
                ((v1 as i32 >> 1i32) + 2i32) as isize
            );
        }
    }
    crate::rtl::rom_table_word!(game, 0xa2b75b, ((v1 as i32 >> 1i32) + 2i32) as isize)
}
crate::game::address_dispatch! {
    pub unsafe fn CallRinkaFunc(ea: u32, k: u16, game: &mut Game) => ea;
        call (k, game); default { Unreachable(game); };
    {
        10663903 => Rinka_3; 10664004 => Rinka_4; 10664018 => Rinka_5; 10664027 => Rinka_B85B;
    }
}
pub unsafe fn Rinka_Main(game: &mut Game) {
    let E = Get_Rinka(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).rinka_parameter_1 != 0u16 && (*Get_Rinka(0i32, game)).rinka_var_1D != 0u16 {
        Rinka_6(ram_word!(game, cur_enemy_index), game);
        Rinka_8(ram_word!(game, cur_enemy_index), game);
        let mut v2: u16 = 0u16;
        RinkasDeathAnimation(v2, game);
    } else {
        CallRinkaFunc(
            (((*E).rinka_var_A) as u32) | (0xa20000u32),
            ram_word!(game, cur_enemy_index),
            game,
        );
    };
}
pub unsafe fn Rinka_3(k: u16, game: &mut Game) {
    let E = Get_Rinka(k as i32, game);
    (*E).rinka_var_F = (*E).rinka_var_F.wrapping_sub(1);
    if ((*E).rinka_var_F & 0x8000) != 0 {
        (*E).rinka_var_A = ((0xa2b85bi32) as u16);
        let mut v3: u16 = 0;
        if (*E).rinka_parameter_1 != 0 {
            v3 = ((*E).base.properties as i32 & !(kEnemyProps_Tangible as i32)) as u16;
        } else {
            v3 = ((*E).base.properties & 0xf3ff | 0x800);
        }
        (*E).base.properties = v3;
        let r18: u16 = -(CalculateAngleFromXY(
            (ram_word!(game, samus_x_pos)).wrapping_sub((*E).base.x_pos),
            (ram_word!(game, samus_y_pos)).wrapping_sub((*E).base.y_pos),
            game,
        ) as i32
            + 0x80i32) as u8 as u16;
        (*E).rinka_var_B = Math_MultBySin(0x120u16, r18);
        (*E).rinka_var_C = Math_MultByCos(0x120u16, r18);
    }
}
pub unsafe fn Rinka_4(k: u16, game: &mut Game) {
    let E = Get_Rinka(k as i32, game);
    (*E).rinka_var_F = (*E).rinka_var_F.wrapping_sub(1);
    if ((*E).rinka_var_F & 0x8000) != 0 {
        (*E).base.health = 10u16;
        Rinka_Init2(k, game);
    }
}
pub unsafe fn Rinka_5(k: u16, game: &mut Game) {
    if Rinka_9(k, game) & 1 != 0 {
        Rinka_B865(k, game);
    }
}
pub unsafe fn Rinka_B85B(k: u16, game: &mut Game) {
    MoveEnemyWithVelocity(game);
    if Rinka_9(k, game) & 1 != 0 {
        Rinka_B865(k, game);
    }
}
pub unsafe fn Rinka_B865(k: u16, game: &mut Game) {
    if (*Get_Rinka(k as i32, game)).rinka_parameter_1 != 0u16 && {
        Rinka_8(k, game);
        (*Get_Rinka(0i32, game)).rinka_var_1D != 0u16
    } {
        Rinka_6(k, game);
        DeleteEnemyAndConnectedEnemies(game);
    } else {
        Rinka_6(k, game);
        Rinka_Init2(k, game);
    };
}
pub unsafe fn Rinka_6(k: u16, game: &mut Game) {
    let E = Get_Rinka(k as i32, game);
    if (*E).rinka_parameter_1 != 0u16
        && ((*E).base.properties as i32 & kEnemyProps_Invisible as i32) == 0i32
    {
        let E0 = Get_Rinka(0i32, game);
        let mut v3: i16 = (((*E0).rinka_var_1E) as i16).wrapping_sub(1i16);
        if v3 < 0i16 {
            v3 = 0i16;
        }
        (*E0).rinka_var_1E = v3 as u16;
    }
}
pub unsafe fn Rinka_7(k: u16, game: &mut Game) {
    let E = Get_Rinka(k as i32, game);
    if ram_word!(game, random_enemy_counter) & 3 == (*E).rinka_parameter_1 {
        (*E).base.properties =
            ((*E).base.properties as i32 & !(kEnemyProps_Tangible as i32)) as u16;
    } else {
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
    };
}
pub unsafe fn Rinka_8(k: u16, game: &mut Game) {
    let E = Get_Rinka(k as i32, game);
    if (*E).rinka_parameter_1 != 0 {
        let rinka_var_D: u16 = (*E).rinka_var_D;
        if rinka_var_D != 0 {
            (*gRam8000_Default(rinka_var_D as i32, game).offset(31)).var_3F = 0u16;
            (*E).rinka_var_D = 0u16;
        }
    }
}
pub unsafe fn Rinka_9(k: u16, game: &mut Game) -> u8 {
    let E = Get_Rinka(k as i32, game);
    let mut y_pos: i16 = (*E).base.y_pos as i16;
    let result: u8 = 1u8;
    if y_pos >= 0i16 {
        let mut v3: i16 = (y_pos as i32 + 16i32 - ram_word!(game, layer1_y_pos) as i32) as i16;
        if v3 >= 0i16 && v3 as i32 - 256i32 & 0x8000i32 != 0 {
            let mut x_pos: i16 = (*E).base.x_pos as i16;
            if x_pos >= 0i16 {
                let mut v5: i16 =
                    (x_pos as i32 + 16i32 - ram_word!(game, layer1_x_pos) as i32) as i16;
                if v5 >= 0i16 && v5 as i32 - 288i32 & 0x8000i32 != 0 {
                    return 0u8;
                }
            }
        }
    }
    result
}
pub unsafe fn Rinka_10(r18: u16, r20: u16, game: &mut Game) -> bool {
    (r20 & 0x8000) != 0
        || ((r20) as i16).wrapping_sub((ram_word!(game, layer1_y_pos)) as i16) < 0i16
        || r20 as i32 - ram_word!(game, layer1_y_pos) as i32 - 224i32 & 0x8000i32 == 0
        || (r20 & 0x8000) != 0
        || ((r18) as i16).wrapping_sub((ram_word!(game, layer1_x_pos)) as i16) < 0i16
        || r18 as i32 - ram_word!(game, layer1_x_pos) as i32 - 256i32 & 0x8000i32 == 0
}
pub unsafe fn Rinka_Frozen(k: u16, game: &mut Game) {
    if Rinka_9(k, game) & 1 != 0 {
        (*Get_Rinka(k as i32, game)).base.frozen_timer = 0u16;
    }
    NormalEnemyFrozenAI(game);
    if (*Get_Rinka(0i32, game)).rinka_var_1D != 0 {
        Rinka_6(k, game);
        Rinka_8(k, game);
        let v1: u16 = 0u16;
        RinkasDeathAnimation(v1, game);
    }
}
pub unsafe fn Rinka_Touch(game: &mut Game) {
    NormalEnemyTouchAiSkipDeathAnim_CurEnemy(game);
    Rinka_B960(ram_word!(game, cur_enemy_index), game);
}
pub unsafe fn Rinka_Shot(game: &mut Game) {
    NormalEnemyShotAiSkipDeathAnim_CurEnemy(game);
    Rinka_B960(ram_word!(game, cur_enemy_index), game);
}
pub unsafe fn Rinka_Powerbomb(k: u16, game: &mut Game) {
    if ((*Get_Rinka(k as i32, game)).base.properties as i32 & kEnemyProps_Invisible as i32) == 0i32
    {
        NormalEnemyPowerBombAiSkipDeathAnim_CurEnemy(game);
        Rinka_B960(k, game);
    }
}
pub unsafe fn Rinka_B960(k: u16, game: &mut Game) {
    let E = Get_Rinka(k as i32, game);
    if (*E).base.health == 0 {
        Rinka_6(k, game);
        Rinka_8(k, game);
        if (*E).rinka_parameter_1 != 0 {
            (*E).base.properties = ((*E).base.properties as i32
                | (kEnemyProps_Tangible as i32 | kEnemyProps_Invisible as i32))
                as u16;
            game.eproj_spawn_pt = Point16U {
                x: (*E).base.x_pos,
                y: (*E).base.y_pos,
            };
            SpawnEprojWithRoomGfx(0xe509u16, 3u16, game);
            (*E).rinka_var_A = ((0xa2b844i32) as u16);
            (*E).rinka_var_F = 1u16;
        } else {
            RinkasDeathAnimation(0u16, game);
        }
    }
}
pub unsafe fn Rinka_Instr_B9A2(k: u16, j: u16, game: &mut Game) -> u16 {
    let E = Get_Rinka(0i32, game);
    if (*E).rinka_var_1E as i32 - 3i32 & 0x8000i32 != 0 {
        (j).wrapping_add(2u16)
    } else {
        *(RomPtr_A2(j, game) as *mut u16)
    }
}
pub unsafe fn Rinka_Instr_B9B3(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E = Get_Rinka(k as i32, game);
    (*E).base.properties = ((*E).base.properties as i32
        | (kEnemyProps_Tangible as i32 | kEnemyProps_Invisible as i32))
        as u16;
    jp
}
pub unsafe fn Rinka_Instr_B9BD(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E = Get_Rinka(k as i32, game);
    (*E).base.properties = ((*E).base.properties as i32
        | (kEnemyProps_ProcessedOffscreen as i32
            | kEnemyProps_Tangible as i32
            | kEnemyProps_Invisible as i32)) as u16;
    jp
}
pub unsafe fn Rinka_Instr_B9C7(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E = Get_Rinka(k as i32, game);
    (*E).base.properties = ((*E).base.properties as i32
        & !(kEnemyProps_Tangible as i32 | kEnemyProps_Invisible as i32))
        as u16;
    (*E).rinka_var_A = ((0xa2b7dfi32) as u16);
    let E0 = Get_Rinka(0i32, game);
    (*E0).rinka_var_1E = (*E0).rinka_var_1E.wrapping_add(1);
    jp
}
pub unsafe fn EnemyInstr_Rio_Instr_1(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*Get_Rio(ram_word!(game, cur_enemy_index) as i32, game)).rio_var_E = 1u16;
    jp
}
pub unsafe fn Rio_Init(game: &mut Game) {
    let E = Get_Rio(ram_word!(game, cur_enemy_index) as i32, game);
    set_fields! {[(*E)], 0u16; rio_var_E, rio_var_F}
    (*E).base.current_instruction = addr_kRio_Ilist_BB4B as u16;
    (*E).rio_var_B = ((0xa2bbedi32) as u16);
}
pub unsafe fn CallRioFunc(ea: u32, game: &mut Game) {
    let k: u16 = ram_word!(game, cur_enemy_index);
    match ea {
        10664941 => Rio_1(k, game),
        10665010 => Rio_2(k, game),
        10665032 => Rio_3(k, game),
        10665143 => Rio_4(k, game),
        10665215 => Rio_5(k, game),
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn Rio_Main(game: &mut Game) {
    NextRandom(game);
    let E = Get_Rio(ram_word!(game, cur_enemy_index) as i32, game);
    CallRioFunc((((*E).rio_var_B) as u32) | (0xa20000u32), game);
}
pub unsafe fn Rio_1(k: u16, game: &mut Game) {
    if CompareDistToSamus_X(k, 0xa0u16, game) & 1 == 0 {
        let E = Get_Rio(k as i32, game);
        (*E).rio_var_C = crate::rtl::rom_table_word!(game, 0xa2bbbb, 0isize);
        (*E).rio_var_D = crate::rtl::rom_table_word!(game, 0xa2bbbf, 0isize);
        if ((ram_word!(game, samus_x_pos)) as i16).wrapping_sub(((*E).base.x_pos) as i16) < 0i16 {
            (*E).rio_var_D = -((*E).rio_var_D as i32) as u16;
        }
        Rio_6(addr_kRio_Ilist_BB7F as u16, game);
        (*E).rio_var_B = ((0xa2bc48i32) as u16);
        if CheckIfEnemyIsOnScreen(game) == 0 {
            QueueSfx2_Max6(0x65u16, game);
        }
    }
}
pub unsafe fn Rio_2(k: u16, game: &mut Game) {
    let E = Get_Rio(k as i32, game);
    if (*E).rio_var_E != 0 {
        (*E).rio_var_E = 0u16;
        Rio_6(addr_stru_A2BB53 as u16, game);
        (*E).rio_var_B = ((0xa2bbedi32) as u16);
    }
}
pub unsafe fn Rio_3(k: u16, game: &mut Game) {
    let E = Get_Rio(k as i32, game);
    if Enemy_MoveRight_IgnoreSlopes(k, ((*E).rio_var_D as i16 as i32) << 8i32, game) != 0 {
        (*E).rio_var_D = -((*E).rio_var_D as i32) as u16;
    } else if (Enemy_MoveDown(k, ((*E).rio_var_C as i16 as i32) << 8i32, game) == 0) {
        let mut v4: i16 = (((*E).rio_var_C) as i16).wrapping_sub(24i16);
        (*E).rio_var_C = v4 as u16;
        if v4 < 0i16 {
            (*E).rio_var_A = (*E).rio_var_D;
            set_fields! {[(*E)], 0u16; rio_var_D, rio_var_C}
            (*E).rio_var_B = ((0xa2bcffi32) as u16);
        } else if (*E).rio_var_E != 0 {
            (*E).rio_var_E = 0u16;
            Rio_6(addr_kRio_Ilist_BB97 as u16, game);
        }
        return;
    }
    (*E).rio_var_C = -((*E).rio_var_C as i32) as u16;
    (*E).rio_var_B = ((0xa2bcb7i32) as u16);
}
pub unsafe fn Rio_4(k: u16, game: &mut Game) {
    let E = Get_Rio(k as i32, game);
    if Enemy_MoveRight_IgnoreSlopes(k, ((*E).rio_var_D as i16 as i32) << 8i32, game) != 0 {
        (*E).rio_var_D = -((*E).rio_var_D as i32) as u16;
    }
    if Enemy_MoveDown(k, ((*E).rio_var_C as i16 as i32) << 8i32, game) != 0 {
        Rio_6(addr_kRio_Ilist_BBA3 as u16, game);
        (*E).rio_var_B = ((0xa2bc32i32) as u16);
    } else {
        (*E).rio_var_C = ((*E).rio_var_C).wrapping_sub(24u16);
    };
}
pub unsafe fn Rio_5(k: u16, game: &mut Game) {
    let E = Get_Rio(k as i32, game);
    if (((*Get_Rio(k as i32, game)).base.y_pos) as i16)
        .wrapping_sub((ram_word!(game, samus_y_pos)) as i16)
        >= 0i16
    {
        (*E).rio_var_D = (*E).rio_var_A;
        (*E).rio_var_C = -1i32 as u16;
        (*E).rio_var_B = ((0xa2bcb7i32) as u16);
    } else {
        let v1: i32 = CalculateAngleOfSamusFromEnemy(k, game) as i32;
        (*E).rio_var_D = kSinCosTable8bit_Sext[(v1 + 64i32) as usize] as u16;
        (*E).rio_var_C = kSinCosTable8bit_Sext[v1 as usize] as u16;
        Enemy_MoveRight_IgnoreSlopes(k, ((*E).rio_var_D as i16 as i32) << 8i32, game);
        Enemy_MoveDown(k, ((*E).rio_var_C as i16 as i32) << 8i32, game);
    };
}
pub unsafe fn Rio_6(a: u16, game: &mut Game) {
    let E = Get_Rio(ram_word!(game, cur_enemy_index) as i32, game);
    if a != (*E).rio_var_F {
        (*E).rio_var_F = a;
        enemy_instruction! {[(*E).base]; instruction = a, timer = 1u16}
        (*E).base.timer = 0u16;
    }
}
pub unsafe fn NorfairLavajumpingEnemy_Instr_BE8E(
    k: u16,
    jp: *const u16,
    game: &mut Game,
) -> *const u16 {
    (*Get_NorfairLavajumpingEnemy(ram_word!(game, cur_enemy_index) as i32, game)).nley_var_00 =
        1u16;
    jp
}
pub unsafe fn NorfairLavajumpingEnemy_Init(game: &mut Game) {
    let E = Get_NorfairLavajumpingEnemy(ram_word!(game, cur_enemy_index) as i32, game);
    set_fields! {[(*E)], 0u16; nley_var_00, nley_var_01}
    if ((*E).nley_parameter_1 & 0x8000) != 0 {
        (*E).base.current_instruction = addr_kNorfairLavajumpingEnemy_Ilist_BE62 as u16;
        (*E).nley_var_F = ((0xa2bedci32) as u16);
    } else {
        (*E).nley_var_D = (*E).base.x_pos;
        (*E).nley_var_E = (*E).base.y_pos;
        (*E).base.current_instruction = addr_kNorfairLavajumpingEnemy_Ilist_BE3C as u16;
        (*E).nley_var_F = ((0xa2bf1ai32) as u16);
    };
}
pub unsafe fn NorfairLavajumpingEnemy_Main(game: &mut Game) {
    NextRandom(game);
    let E = Get_NorfairLavajumpingEnemy(ram_word!(game, cur_enemy_index) as i32, game);
    CallEnemyPreInstr((((*E).nley_var_F) as u32) | (0xa20000u32), game);
}
pub unsafe fn NorfairLavajumpingEnemy_Func_1(k: u16, game: &mut Game) {
    let E = Get_NorfairLavajumpingEnemy(k as i32, game);
    let v1: i32 = k as i32 >> 1i32;
    if ram_word!(game, enemy_drawing_queue[(v1 + 100i32)]) != 0 {
        let v3: u16 = ram_word!(game, enemy_drawing_queue[(v1 + 109i32)]);
        (*E).base.frozen_timer = v3;
        if v3 != 0u16 || (ram_word!(game, enemy_drawing_queue_sizes[(v1 + 2i32)]) & 0x8000) == 0 {
            (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Invisible) as u16);
        } else {
            (*E).base.properties =
                ((*E).base.properties as i32 & !(kEnemyProps_Invisible as i32)) as u16;
            (*E).base.y_pos = ram_word!(game, enemy_drawing_queue[(v1 + 93i32)]);
        }
    } else {
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Deleted) as u16);
    };
}
pub unsafe fn NorfairLavajumpingEnemy_Func_2(k: u16, game: &mut Game) {
    let E = Get_NorfairLavajumpingEnemy(k as i32, game);
    (*E).nley_var_C = crate::rtl::rom_table_word!(
        game,
        0xa2be86,
        ((((*(ram_ptr!(game, u16; 0x5e5) as *mut u8).offset(1isize)) as u16) & (6u16)) >> 1)
            as isize,
    );
    (*E).nley_var_F = ((0xa2bf3ei32) as u16);
    (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_ProcessedOffscreen) as u16);
    QueueSfx2_Max6(0xdu16, game);
}
pub unsafe fn NorfairLavajumpingEnemy_Func_3(k: u16, game: &mut Game) {
    let E = Get_NorfairLavajumpingEnemy(k as i32, game);
    let t: i32 = (*E).base.y_subpos as i32
        + ((*(&raw mut (*E).nley_var_C as *mut u8).offset(0) as i32) << 8i32);
    (*E).base.y_subpos = t as u16;
    (*E).base.y_pos = ((*E).base.y_pos as i32
        + (*(&raw mut (*E).nley_var_C as *mut u8).offset(1isize) as i8 as i32 + (t >> 16i32)))
        as u16;
    let v3: u16 = ((*E).nley_var_C).wrapping_add(56u16);
    (*E).nley_var_C = v3;
    if v3 >= 0xfc00u16 {
        NorfairLavajumpingEnemy_Func_6(addr_kNorfairLavajumpingEnemy_Ilist_BE42 as u16, game);
        (*E).nley_var_F = ((0xa2bf7ci32) as u16);
    }
}
pub unsafe fn NorfairLavajumpingEnemy_Func_4(k: u16, game: &mut Game) {
    let E = Get_NorfairLavajumpingEnemy(k as i32, game);
    let t: i32 = (*E).base.y_subpos as i32
        + ((*(&raw mut (*E).nley_var_C as *mut u8).offset(0) as i32) << 8i32);
    (*E).base.y_subpos = t as u16;
    (*E).base.y_pos = ((*E).base.y_pos as i32
        + (*(&raw mut (*E).nley_var_C as *mut u8).offset(1isize) as i8 as i32 + (t >> 16i32)))
        as u16;
    (*E).nley_var_C = ((*E).nley_var_C).wrapping_add(56u16);
    if (*E).nley_var_00 != 0 {
        (*E).nley_var_00 = 0u16;
        (*E).nley_var_F = ((0xa2bfbci32) as u16);
    }
}
pub unsafe fn NorfairLavajumpingEnemy_Func_5(k: u16, game: &mut Game) {
    let E = Get_NorfairLavajumpingEnemy(k as i32, game);
    let t: i32 = (*E).base.y_subpos as i32
        + ((*(&raw mut (*E).nley_var_C as *mut u8).offset(0) as i32) << 8i32);
    (*E).base.y_subpos = t as u16;
    (*E).base.y_pos = ((*E).base.y_pos as i32
        + *(&raw mut (*E).nley_var_C as *mut u8).offset(1isize) as i8 as i32
        + (t >> 16i32)) as u16;
    if ((*E).base.y_pos & 0xf0) == 240 {
        (*E).base.x_pos = (*E).nley_var_D;
        (*E).base.y_pos = (*E).nley_var_E;
        NorfairLavajumpingEnemy_Func_6(addr_kNorfairLavajumpingEnemy_Ilist_BE3C as u16, game);
        (*E).nley_var_F = ((0xa2bf1ai32) as u16);
        (*E).base.properties =
            ((*E).base.properties as i32 & !(kEnemyProps_ProcessedOffscreen as i32)) as u16;
    } else {
        (*E).nley_var_C = ((*E).nley_var_C).wrapping_add(56u16);
    };
}
pub unsafe fn NorfairLavajumpingEnemy_Func_6(a: u16, game: &mut Game) {
    let E = Get_NorfairLavajumpingEnemy(ram_word!(game, cur_enemy_index) as i32, game);
    if a != (*E).nley_var_01 {
        (*E).nley_var_01 = a;
        enemy_instruction! {[(*E).base]; instruction = a, timer = 1u16}
        (*E).base.timer = 0u16;
    }
}
pub unsafe fn NorfairRio_Instr_C1C9(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*Get_NorfairRio(ram_word!(game, cur_enemy_index) as i32, game)).nro_var_01 = 1u16;
    jp
}
pub unsafe fn NorfairRio_Instr_C1D4(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*Get_NorfairRio(ram_word!(game, cur_enemy_index) as i32, game)).nro_var_02 = 8u16;
    jp
}
pub unsafe fn NorfairRio_Instr_C1DF(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*Get_NorfairRio(ram_word!(game, cur_enemy_index) as i32, game)).nro_var_02 = 8u16;
    jp
}
pub unsafe fn NorfairRio_Instr_C1EA(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*Get_NorfairRio(ram_word!(game, cur_enemy_index) as i32, game)).nro_var_02 = 12u16;
    jp
}
pub unsafe fn NorfairRio_Instr_C1F5(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*Get_NorfairRio(ram_word!(game, cur_enemy_index) as i32, game)).nro_var_02 = -12i32 as u16;
    jp
}
pub unsafe fn NorfairRio_Instr_C200(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*Get_NorfairRio(ram_word!(game, cur_enemy_index) as i32, game)).nro_var_02 = 4u16;
    jp
}
pub unsafe fn NorfairRio_Instr_C20B(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*Get_NorfairRio(ram_word!(game, cur_enemy_index) as i32, game)).nro_var_02 = 0u16;
    jp
}
pub unsafe fn NorfairRio_Instr_C216(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*Get_NorfairRio(ram_word!(game, cur_enemy_index) as i32, game)).nro_var_02 = -4i32 as u16;
    jp
}
pub unsafe fn NorfairRio_Instr_C221(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*Get_NorfairRio(ram_word!(game, cur_enemy_index) as i32, game)).nro_var_02 = -12i32 as u16;
    jp
}
pub unsafe fn NorfairRio_Instr_C22C(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*Get_NorfairRio(ram_word!(game, cur_enemy_index) as i32, game)).nro_var_02 = -16i32 as u16;
    jp
}
pub unsafe fn NorfairRio_Instr_C237(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*Get_NorfairRio(ram_word!(game, cur_enemy_index) as i32, game)).nro_var_02 = 12u16;
    jp
}
pub unsafe fn NorfairRio_Init(game: &mut Game) {
    let E = Get_NorfairRio(ram_word!(game, cur_enemy_index) as i32, game);
    set_fields! {[(*E)], 0u16; nro_var_01, nro_var_02}
    if ((*E).nro_parameter_1 & 0x8000) != 0 {
        (*E).nro_var_00 = addr_kNorfairRio_Ilist_C18F as u16;
        (*E).base.current_instruction = addr_kNorfairRio_Ilist_C18F as u16;
        (*E).nro_var_F = ((0xa2c281i32) as u16);
    } else {
        (*E).nro_var_00 = addr_kNorfairRio_Ilist_C0F1 as u16;
        (*E).base.current_instruction = addr_kNorfairRio_Ilist_C0F1 as u16;
        (*E).nro_var_F = ((0xa2c2e7i32) as u16);
    };
}
pub unsafe fn NorfairRio_Main(game: &mut Game) {
    NextRandom(game);
    let E = Get_NorfairRio(ram_word!(game, cur_enemy_index) as i32, game);
    CallEnemyPreInstr((((*E).nro_var_F) as u32) | (0xa20000u32), game);
}
pub unsafe fn NorfairRio_Func_1(k: u16, game: &mut Game) {
    let E = Get_NorfairRio(k as i32, game);
    let v1: i32 = k as i32 >> 1i32;
    if ram_word!(game, enemy_drawing_queue[(v1 + 100i32)]) != 0 {
        let v3: u16 = ram_word!(game, enemy_drawing_queue[(v1 + 109i32)]);
        (*E).base.frozen_timer = v3;
        if v3 != 0u16 || {
            (*E).base.properties =
                ((*E).base.properties as i32 & !(kEnemyProps_Invisible as i32)) as u16;
            (*gEnemySpawnData(k as i32, game).offset(31)).some_flag as i32
                == addr_kNorfairRio_Ilist_C0F1 as i32
        } {
            (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Invisible) as u16);
        } else {
            let mut r18: u16 = addr_kNorfairRio_Ilist_C18F as u16;
            let mut v6 = gEnemySpawnData(k as i32, game);
            if ((*v6.offset(31)).field_4 & 0x8000) != 0 {
                r18 = addr_kNorfairRio_Ilist_C1A3 as u16;
            }
            NorfairRio_Func_7(r18, game);
            (*E).base.properties =
                ((*E).base.properties as i32 & !(kEnemyProps_Invisible as i32)) as u16;
            (*E).base.x_pos = ram_word!(game, enemy_drawing_queue[(v1 + 91i32)]);
            (*E).base.y_pos = ((*v6.offset(31)).field_4)
                .wrapping_add(ram_word!(game, enemy_drawing_queue[(v1 + 93i32)]));
        }
    } else {
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Deleted) as u16);
    };
}
pub unsafe fn NorfairRio_Func_2(k: u16, game: &mut Game) {
    let E = Get_NorfairRio(k as i32, game);
    if (ram_word!(game, random_number) & 0x101) != 0
        && CompareDistToSamus_X(k, 0xc0u16, game) & 1 == 0
    {
        (*E).nro_var_A = crate::rtl::rom_table_word!(
            game,
            0xa2c1c1,
            ((ram_word!(game, random_number) as i32 >> 1i32 & 2i32) as u16 >> 1) as isize,
        );
        (*E).nro_var_B = crate::rtl::rom_table_word!(game, 0xa2c1c5, 0isize);
        if ((ram_word!(game, samus_x_pos)) as i16).wrapping_sub(((*E).base.x_pos) as i16) < 0i16 {
            (*E).nro_var_B = -((*E).nro_var_B as i32) as u16;
        }
        NorfairRio_Func_7(addr_kNorfairRio_Ilist_C107 as u16, game);
        (*E).nro_var_F = ((0xa2c33fi32) as u16);
    } else if (*E).nro_var_01 != 0 {
        (*E).nro_var_01 = 0u16;
        NorfairRio_Func_7(addr_kNorfairRio_Ilist_C0F1 as u16, game);
    }
}
pub unsafe fn NorfairRio_Func_3(k: u16, game: &mut Game) {
    let E = Get_NorfairRio(k as i32, game);
    if (*E).nro_var_01 != 0 {
        (*E).nro_var_01 = 0u16;
        NorfairRio_Func_7(addr_kNorfairRio_Ilist_C12F as u16, game);
        (*E).nro_var_F = ((0xa2c361i32) as u16);
        QueueSfx2_Max6(0x65u16, game);
    }
}
pub unsafe fn NorfairRio_Func_4(k: u16, game: &mut Game) {
    let E = Get_NorfairRio(k as i32, game);
    if Enemy_MoveRight_IgnoreSlopes(k, ((*E).nro_var_B as i16 as i32) << 8i32, game) != 0 {
        (*E).nro_var_B = -((*E).nro_var_B as i32) as u16;
    }
    if Enemy_MoveDown(k, ((*E).nro_var_A as i16 as i32) << 8i32, game) != 0u8 || {
        let mut v4: i16 = (((*E).nro_var_A) as i16).wrapping_sub(32i16);
        (*E).nro_var_A = v4 as u16;
        v4 < 0i16
    } {
        (*E).nro_var_A = -1i32 as u16;
        NorfairRio_Func_7(addr_kNorfairRio_Ilist_C145 as u16, game);
        (*E).nro_var_F = ((0xa2c3b1i32) as u16);
    }
}
pub unsafe fn NorfairRio_Func_5(k: u16, game: &mut Game) {
    let E = Get_NorfairRio(k as i32, game);
    if Enemy_MoveRight_IgnoreSlopes(k, ((*E).nro_var_B as i16 as i32) << 8i32, game) != 0 {
        (*E).nro_var_B = -((*E).nro_var_B as i32) as u16;
    }
    if Enemy_MoveDown(k, ((*E).nro_var_A as i16 as i32) << 8i32, game) != 0 {
        (*E).nro_var_F = ((0xa2c406i32) as u16);
    } else {
        (*E).nro_var_A = ((*E).nro_var_A).wrapping_sub(32u16);
        if (*E).nro_var_01 != 0 {
            (*E).nro_var_01 = 0u16;
            NorfairRio_Func_7(addr_kNorfairRio_Ilist_C179 as u16, game);
        }
    };
}
pub unsafe fn NorfairRio_Func_6(k: u16, game: &mut Game) {
    (*Get_NorfairRio(k as i32, game)).nro_var_F = ((0xa2c2e7i32) as u16);
}
pub unsafe fn NorfairRio_Func_7(a: u16, game: &mut Game) {
    let E = Get_NorfairRio(ram_word!(game, cur_enemy_index) as i32, game);
    if a != (*E).nro_var_00 {
        (*E).nro_var_00 = a;
        enemy_instruction! {[(*E).base]; instruction = a, timer = 1u16}
        (*E).base.timer = 0u16;
    }
}
pub unsafe fn LowerNorfairRio_Instr_C6D2(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*Get_NorfairRio(ram_word!(game, cur_enemy_index) as i32, game)).nro_var_01 = 1u16;
    jp
}
pub unsafe fn LowerNorfairRio_Instr_C6DD(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*Get_NorfairRio(ram_word!(game, cur_enemy_index) as i32, game)).nro_var_02 = 0u16;
    jp
}
pub unsafe fn LowerNorfairRio_Instr_C6E8(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*Get_NorfairRio(ram_word!(game, cur_enemy_index) as i32, game)).nro_var_02 = 1u16;
    jp
}
pub unsafe fn LowerNorfairRio_Init(game: &mut Game) {
    let E = Get_NorfairRio(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).nro_var_01 = 0u16;
    if ((*E).nro_parameter_1 & 0x8000) == 0 {
        (*E).nro_var_F = ((0xa2c771i32) as u16);
        (*E).nro_var_00 = addr_kLowerNorfairRio_Ilist_C61A as u16;
        (*E).base.current_instruction = addr_kLowerNorfairRio_Ilist_C61A as u16;
    } else {
        (*E).nro_var_F = ((0xa2c72ei32) as u16);
        (*E).nro_var_00 = addr_kLowerNorfairRio_Ilist_C6B0 as u16;
        (*E).base.current_instruction = addr_kLowerNorfairRio_Ilist_C6B0 as u16;
    };
}
pub unsafe fn LowerNorfairRio_Main(game: &mut Game) {
    NextRandom(game);
    let E = Get_LowerNorfairRio(ram_word!(game, cur_enemy_index) as i32, game);
    CallEnemyPreInstr((((*E).lnro_var_F) as u32) | (0xa20000u32), game);
}
pub unsafe fn LowerNorfairRio_Func_1(k: u16, game: &mut Game) {
    let E = Get_LowerNorfairRio(k as i32, game);
    let v1: i32 = k as i32 >> 1i32;
    if ram_word!(game, enemy_drawing_queue[(v1 + 100i32)]) != 0 {
        let v3: u16 = ram_word!(game, enemy_drawing_queue[(v1 + 109i32)]);
        (*E).base.frozen_timer = v3;
        if v3 == 0 && (*gEnemySpawnData(k as i32, game).offset(31)).field_4 != 0u16 {
            (*E).base.properties =
                ((*E).base.properties as i32 & !(kEnemyProps_Invisible as i32)) as u16;
            (*E).base.x_pos = ram_word!(game, enemy_drawing_queue[(v1 + 91i32)]);
            (*E).base.y_pos =
                (ram_word!(game, enemy_drawing_queue[(v1 + 93i32)])).wrapping_add(12u16);
        } else {
            (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Invisible) as u16);
        }
    } else {
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Deleted) as u16);
    };
}
pub unsafe fn LowerNorfairRio_Func_2(k: u16, game: &mut Game) {
    let E = Get_NorfairRio(k as i32, game);
    if (ram_word!(game, random_number) & 0x101) != 0
        && CompareDistToSamus_X(k, 0x70u16, game) & 1 == 0
    {
        (*E).nro_var_C = crate::rtl::rom_table_word!(game, 0xa2c6ca, 0isize);
        (*E).nro_var_D = crate::rtl::rom_table_word!(game, 0xa2c6ce, 0isize);
        if ((ram_word!(game, samus_x_pos)) as i16).wrapping_sub(((*E).base.x_pos) as i16) < 0i16 {
            (*E).nro_var_D = -((*E).nro_var_D as i32) as u16;
        }
        LowerNorfairRio_Func_7(addr_kLowerNorfairRio_Ilist_C630 as u16, game);
        (*E).nro_var_F = ((0xa2c7bbi32) as u16);
    } else {
        (*E).nro_var_01 = 0u16;
        LowerNorfairRio_Func_7(addr_kLowerNorfairRio_Ilist_C61A as u16, game);
    };
}
pub unsafe fn LowerNorfairRio_Func_3(k: u16, game: &mut Game) {
    let E = Get_NorfairRio(k as i32, game);
    if (*E).nro_var_01 != 0 {
        (*E).nro_var_01 = 0u16;
        LowerNorfairRio_Func_7(addr_kLowerNorfairRio_Ilist_C65A as u16, game);
        (*E).nro_var_F = ((0xa2c7d6i32) as u16);
    }
}
pub unsafe fn LowerNorfairRio_Func_4(k: u16, game: &mut Game) {
    let E = Get_LowerNorfairRio(k as i32, game);
    if Enemy_MoveRight_IgnoreSlopes(k, ((*E).lnro_var_D as i16 as i32) << 8i32, game) != 0 {
        (*E).lnro_var_D = -((*E).lnro_var_D as i32) as u16;
    }
    if Enemy_MoveDown(k, ((*E).lnro_var_C as i16 as i32) << 8i32, game) != 0u8 || {
        let mut v4: i16 = (((*E).lnro_var_C) as i16).wrapping_sub(32i16);
        (*E).lnro_var_C = v4 as u16;
        v4 < 0i16
    } {
        (*E).lnro_var_C = -1i32 as u16;
        LowerNorfairRio_Func_7(addr_kLowerNorfairRio_Ilist_C662 as u16, game);
        (*E).lnro_var_F = ((0xa2c82di32) as u16);
        QueueSfx2_Max6(0x64u16, game);
    }
}
pub unsafe fn LowerNorfairRio_Func_5(k: u16, game: &mut Game) {
    let E = Get_NorfairRio(k as i32, game);
    if Enemy_MoveRight_IgnoreSlopes(k, ((*E).nro_var_D as i16 as i32) << 8i32, game) != 0 {
        (*E).nro_var_D = -((*E).nro_var_D as i32) as u16;
    }
    if Enemy_MoveDown(k, ((*E).nro_var_C as i16 as i32) << 8i32, game) != 0 {
        LowerNorfairRio_Func_7(addr_kLowerNorfairRio_Ilist_C686 as u16, game);
        (*E).nro_var_F = ((0xa2c888i32) as u16);
    } else {
        (*E).nro_var_C = ((*E).nro_var_C).wrapping_sub(32u16);
        if (*E).nro_var_01 != 0 {
            (*E).nro_var_01 = 0u16;
            LowerNorfairRio_Func_7(addr_kLowerNorfairRio_Ilist_C674 as u16, game);
        }
    };
}
pub unsafe fn LowerNorfairRio_Func_6(k: u16, game: &mut Game) {
    let E = Get_NorfairRio(k as i32, game);
    if (*E).nro_var_01 != 0 {
        (*E).nro_var_01 = 0u16;
        LowerNorfairRio_Func_7(addr_kLowerNorfairRio_Ilist_C686 as u16, game);
        (*E).nro_var_F = ((0xa2c771i32) as u16);
    }
}
pub unsafe fn LowerNorfairRio_Func_7(a: u16, game: &mut Game) {
    let E = Get_NorfairRio(ram_word!(game, cur_enemy_index) as i32, game);
    if a != (*E).nro_var_00 {
        (*E).nro_var_00 = a;
        enemy_instruction! {[(*E).base]; instruction = a, timer = 1u16}
        (*E).base.timer = 0u16;
    }
}
pub unsafe fn MaridiaLargeSnail_Instr_CB6B(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    QueueSfx2_Max6(0xeu16, game);
    jp
}
pub unsafe fn MaridiaLargeSnail_Instr_CCB3(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*Get_MaridiaLargeSnail(ram_word!(game, cur_enemy_index) as i32, game)).mlsl_var_02 = 1u16;
    jp
}
pub unsafe fn MaridiaLargeSnail_Instr_CCBE(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*Get_MaridiaLargeSnail(ram_word!(game, cur_enemy_index) as i32, game)).mlsl_var_03 = 1u16;
    jp
}
pub unsafe fn MaridiaLargeSnail_Instr_CCC9(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*Get_MaridiaLargeSnail(ram_word!(game, cur_enemy_index) as i32, game)).mlsl_var_03 = 0u16;
    jp
}
pub unsafe fn MaridiaLargeSnail_Init(game: &mut Game) {
    let E = Get_MaridiaLargeSnail(ram_word!(game, cur_enemy_index) as i32, game);
    set_fields! {[(*E)], 0u16; mlsl_var_B, mlsl_var_D, mlsl_var_00, mlsl_var_01, mlsl_var_02, mlsl_var_03, mlsl_var_0A}
    (*E).mlsl_var_E = 3u16;
    (*E).mlsl_var_C = 128u16;
    (*E).base.current_instruction = addr_kMaridiaLargeSnail_Ilist_CA4B as u16;
    (*E).mlsl_var_A = ((0xa2cde6i32) as u16);
    (*E).mlsl_var_F = ((0xa2cf66i32) as u16);
}
crate::game::address_dispatch! {
    pub unsafe fn CallMaridiaLargeSnailFunc(ea: u32, game: &mut Game) => ea;
        call (game); default { Unreachable(game); };
    {
        10669542 => MaridiaLargeSnail_Func_4; 10669611 => MaridiaLargeSnail_Func_5;
        10669888 => MaridiaLargeSnail_Func_6;
    }
}
pub unsafe fn MaridiaLargeSnail_Main(game: &mut Game) {
    MaridiaLargeSnail_Func_1(ram_word!(game, cur_enemy_index), game);
    let E = Get_MaridiaLargeSnail(ram_word!(game, cur_enemy_index) as i32, game);
    CallMaridiaLargeSnailFunc((((*E).mlsl_var_A) as u32) | (0xa20000u32), game);
    MaridiaLargeSnail_Func_2(ram_word!(game, cur_enemy_index), game);
    MaridiaLargeSnail_Func_3(ram_word!(game, cur_enemy_index), game);
}
pub unsafe fn MaridiaLargeSnail_Func_1(k: u16, game: &mut Game) {
    MaridiaLargeSnail_Func_11(game);
    let E = Get_MaridiaLargeSnail(k as i32, game);
    (*E).mlsl_var_06 = (*E).base.x_pos;
    (*E).mlsl_var_08 = (*E).base.y_pos;
}
pub unsafe fn MaridiaLargeSnail_Func_2(k: u16, game: &mut Game) {
    let E = Get_MaridiaLargeSnail(k as i32, game);
    (*E).mlsl_var_0B = 0u16;
    if CheckIfEnemyTouchesSamus(k, game) != 0 {
        set_ram_word!(
            game.extra_samus_y_displacement = ((*E).base.y_pos).wrapping_sub((*E).mlsl_var_08)
        );
    }
    if (*E).mlsl_var_04 != 0 {
        let r18: u16 = ((*E).base.x_pos).wrapping_sub((*E).mlsl_var_06);
        if (r18 & 0x8000) != 0 {
            if (*E).mlsl_var_05 == 0 {
                return;
            }
        } else if (*E).mlsl_var_05 != 0 {
            return;
        }
        update_ram_word!(game.extra_samus_x_displacement, wrapping_add(r18));
    }
}
pub unsafe fn MaridiaLargeSnail_Func_3(k: u16, game: &mut Game) {
    let E = Get_MaridiaLargeSnail(k as i32, game);
    if (*E).mlsl_var_04 != 0 {
        let mut v2: i16 = (((*E).base.x_pos) as i16).wrapping_sub(((*E).mlsl_var_06) as i16);
        if v2 == 0 {
            if (*E).mlsl_var_05 != 0 {
                if (*E).mlsl_var_D == 0 || (ram_word!(game, joypad1_lastkeys) & 0x100) == 0 {
                    return;
                }
            } else if (*E).mlsl_var_D != 0u16 || (ram_word!(game, joypad1_lastkeys) & 0x200) == 0 {
                return;
            }
            (*E).mlsl_var_0B = 1u16;
            (*E).base.x_pos = (*E).mlsl_var_06;
            return;
        }
        if v2 < 0i16 {
            if (ram_word!(game, joypad1_lastkeys) & 0x100) != 0 {
                (*E).mlsl_var_0B = 1u16;
                (*E).base.x_pos = (*E).mlsl_var_06;
            }
        } else if (ram_word!(game, joypad1_lastkeys) & 0x200) != 0 {
            (*E).mlsl_var_0B = 1u16;
            (*E).base.x_pos = (*E).mlsl_var_06;
        }
    }
}
pub unsafe fn MaridiaLargeSnail_Func_4(game: &mut Game) {
    let E = Get_MaridiaLargeSnail(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).mlsl_var_E != 0 {
        CallEnemyPreInstr((((*E).mlsl_var_F) as u32) | (0xa20000u32), game);
    } else {
        (*E).mlsl_var_00 = 0u16;
        if (GetSamusEnemyDelta_X(ram_word!(game, cur_enemy_index), game) & 0x8000) != 0 {
            set_fields! {[(*E)], 1u16; mlsl_var_D, mlsl_var_00}
        }
        MaridiaLargeSnail_Func_10(ram_word!(game, cur_enemy_index), game);
        if IsSamusWithinEnemy_X(ram_word!(game, cur_enemy_index), 0x18u16, game) != 0 {
            (*E).mlsl_var_00 = ((*E).mlsl_var_00) | (2u16);
            MaridiaLargeSnail_Func_10(ram_word!(game, cur_enemy_index), game);
            (*E).mlsl_var_A = ((0xa2ce2bi32) as u16);
        }
    };
}
pub unsafe fn MaridiaLargeSnail_Func_5(game: &mut Game) {
    let E = Get_MaridiaLargeSnail(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).mlsl_var_E != 0 {
        CallEnemyPreInstr((((*E).mlsl_var_F) as u32) | (0xa20000u32), game);
    } else if Enemy_MoveDown(
        ram_word!(game, cur_enemy_index),
        (1i16 as i32) << 16i32,
        game,
    ) == 0
    {
        (*E).mlsl_var_B = 0u16;
        (*E).mlsl_var_F = ((0xa2cf66i32) as u16);
        (*E).mlsl_var_E = 3u16;
    }
    if (*E).mlsl_var_0B == 0 {
        let mut R36: u16 = 0u16;
        (*E).mlsl_var_C = (*E).mlsl_var_C.wrapping_sub(1);
        let v4: bool = ((*E).mlsl_var_C & 0x8000) != 0;
        if v4
            && {
                (*E).mlsl_var_C = 0u16;
                IsSamusWithinEnemy_X(ram_word!(game, cur_enemy_index), 0x20u16, game) != 0u16
            }
            && (*E).mlsl_var_03 != 0u16
            && (*E).mlsl_var_E == 0
        {
            set_fields! {[(*E)], 0u16; mlsl_var_00, mlsl_var_D}
            if (GetSamusEnemyDelta_X(ram_word!(game, cur_enemy_index), game) & 0x8000) != 0 {
                set_fields! {[(*E)], 1u16; mlsl_var_00, mlsl_var_D}
            }
            (*E).mlsl_var_C = 128u16;
            (*E).mlsl_var_00 = ((*E).mlsl_var_00 & 1 | 4);
            MaridiaLargeSnail_Func_10(ram_word!(game, cur_enemy_index), game);
            (*E).mlsl_var_A = ((0xa2cf40i32) as u16);
        } else {
            let mut v5: u16 = 128u16;
            if (*E).mlsl_var_D != 0 {
                v5 = 132u16;
            }
            let v6: i32 = v5 as i32 >> 1i32;
            let r18: u16 = crate::rtl::rom_table_word!(game, 0xa28187, (v6 + 1i32) as isize);
            let mut r20: u16 = crate::rtl::rom_table_word!(game, 0xa28187, v6 as isize);
            if (*E).mlsl_var_04 != 0u16 && (*E).mlsl_var_D == (*E).mlsl_var_05 {
                if (*E).mlsl_var_D != 0 {
                    r20 = (r20).wrapping_sub(16u16);
                } else {
                    r20 = (r20).wrapping_add(16u16);
                }
            }
            if Enemy_MoveRight_SlopesAsWalls(
                ram_word!(game, cur_enemy_index),
                ((r20 as u32) << 16i32 | r18 as u32) as i32,
                game,
            ) != 0
            {
                R36 = R36.wrapping_add(1);
            }
            if (*E).mlsl_var_04 != 0u16 && (*E).mlsl_var_D == (*E).mlsl_var_05 {
                let mut v7: u16 = 0;
                if (*E).mlsl_var_D != 0 {
                    v7 = ((*E).base.x_pos).wrapping_add(16u16);
                } else {
                    v7 = ((*E).base.x_pos).wrapping_sub(16u16);
                }
                (*E).base.x_pos = v7;
            }
            if R36 != 0 {
                (*E).mlsl_var_D = ((*E).mlsl_var_D) ^ (1u16);
                (*E).mlsl_var_00 = ((*E).mlsl_var_00) ^ (4u16);
                MaridiaLargeSnail_Func_10(ram_word!(game, cur_enemy_index), game);
            }
        }
    }
}
pub unsafe fn MaridiaLargeSnail_Func_6(game: &mut Game) {
    let E = Get_MaridiaLargeSnail(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).mlsl_var_02 != 0 {
        (*E).mlsl_var_02 = 0u16;
        (*E).mlsl_var_00 = ((*E).mlsl_var_00).wrapping_sub(2u16);
        MaridiaLargeSnail_Func_10(ram_word!(game, cur_enemy_index), game);
        (*E).mlsl_var_A = ((0xa2ce2bi32) as u16);
    }
}
pub unsafe fn MaridiaLargeSnail_Func_7(k: u16, game: &mut Game) {
    let E = Get_MaridiaLargeSnail(k as i32, game);
    let mut v2: u16 = ((*E).mlsl_var_B).wrapping_add(384u16);
    if v2 as i32 - 0x4000i32 & 0x8000i32 == 0 {
        v2 = 0x4000u16;
    }
    (*E).mlsl_var_B = v2;
    let v5: i32 = 8i32 * (v2 as i32 >> 8i32) >> 1i32;
    if Enemy_MoveDown(
        k,
        *(game.g_rom.offset(crate::rtl::lorom_offset(0xa0cbc7)) as *mut u32)
            .offset((v5 >> 1i32) as isize) as i32,
        game,
    ) != 0
    {
        (*E).mlsl_var_E = (*E).mlsl_var_E.wrapping_sub(1);
        let v6: i16 = (((*E).mlsl_var_B) as i16).wrapping_sub(4096i16);
        if v6 < 0i16 {
            (*E).mlsl_var_E = 0u16;
        }
        (*E).mlsl_var_B = v6 as u16;
        (*E).mlsl_var_F = ((0xa2cfa9i32) as u16);
    }
}
pub unsafe fn MaridiaLargeSnail_Func_8(k: u16, game: &mut Game) {
    let E = Get_MaridiaLargeSnail(k as i32, game);
    let v2: i16 = (((*E).mlsl_var_B) as i16).wrapping_sub(384i16);
    (*E).mlsl_var_B = v2 as u16;
    if v2 >= 0i16 {
        let v3: u16 = ((v2 as i32 & 0x7f00i32) >> 8i32) as u16;
        let v5: i32 = 8i32 * v3 as i32 >> 1i32;
        Enemy_MoveDown(
            k,
            *(game.g_rom.offset(crate::rtl::lorom_offset(0xa0cbc7)) as *mut u32)
                .offset((v5 + 2i32 >> 1i32) as isize) as i32,
            game,
        );
    } else {
        (*E).mlsl_var_B = 0u16;
        (*E).mlsl_var_F = ((0xa2cf66i32) as u16);
    };
}
pub unsafe fn MaridiaLargeSnail_Func_9(game: &mut Game) {
    QueueSfx2_Max6(0x63u16, game);
}
pub unsafe fn MaridiaLargeSnail_Func_10(k: u16, game: &mut Game) {
    let E = Get_MaridiaLargeSnail(k as i32, game);
    let mlsl_var_00: u16 = (*E).mlsl_var_00;
    if mlsl_var_00 != (*E).mlsl_var_01 {
        (*E).mlsl_var_01 = mlsl_var_00;
        enemy_instruction! {[(*E).base]; instruction = crate::rtl::rom_table_word!(game, 0xa2cb77, mlsl_var_00 as isize), timer = 1u16}
        (*E).base.timer = 0u16;
    }
}
pub unsafe fn MaridiaLargeSnail_Func_11(game: &mut Game) {
    let E = Get_MaridiaLargeSnail(ram_word!(game, cur_enemy_index) as i32, game);
    set_fields! {[(*E)], 0u16; mlsl_var_04, mlsl_var_05}
    if IsSamusWithinEnemy_Y(ram_word!(game, cur_enemy_index), 0x20u16, game) != 0u16
        && IsSamusWithinEnemy_X(ram_word!(game, cur_enemy_index), 0x18u16, game) != 0u16
    {
        (*E).mlsl_var_04 = 1u16;
        if (GetSamusEnemyDelta_X(ram_word!(game, cur_enemy_index), game) & 0x8000) != 0 {
            (*E).mlsl_var_05 = 1u16;
        }
    }
}
pub unsafe fn MaridiaLargeSnail_Func_12(game: &mut Game) {
    Enemy_NormalTouchAI_A2(game);
    MaridiaLargeSnail_Touch(game);
}
pub unsafe fn MaridiaLargeSnail_Touch(game: &mut Game) {
    let k: u16 = ram_word!(game, cur_enemy_index);
    if CheckIfEnemyTouchesSamus(k, game) == 0 {
        if ((ram_word!(game, samus_x_pos)) as i16).wrapping_sub(
            ((*Get_MaridiaLargeSnail(ram_word!(game, cur_enemy_index) as i32, game))
                .base
                .x_pos) as i16,
        ) < 0i16
        {
            update_ram_word!(game.extra_samus_x_displacement, wrapping_sub(4u16));
        } else {
            update_ram_word!(game.extra_samus_x_displacement, wrapping_add(4u16));
        }
    }
}
pub unsafe fn MaridiaLargeSnail_Shot(game: &mut Game) {
    NormalEnemyShotAi(game);
    QueueSfx2_Max6(0x57u16, game);
}
unsafe fn HirisingSlowfalling_Func_1(k: u16, game: &mut Game) -> u16 {
    let E = Get_HirisingSlowfalling(k as i32, game);
    Mult8x8((*E).hsg_parameter_1 as u8, (*E).hsg_var_F as u8)
}
pub unsafe fn HirisingSlowfalling_Init(game: &mut Game) {
    HirisingSlowfalling_Func_3(addr_kHirisingSlowfalling_Ilist_D82C as u16, game);
    let E = Get_HirisingSlowfalling(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).hsg_var_A = ((0xa2e035i32) as u16);
    (*E).hsg_var_00 = (*E).base.x_pos;
    (*E).hsg_var_01 = (*E).base.y_pos;
    let v2: i32 = *(&raw mut (*E).hsg_parameter_1 as *mut u8).offset(1isize) as i32;
    (*E).hsg_var_E = crate::rtl::rom_table_word!(game, 0xa2df5e, v2 as isize);
    (*E).hsg_var_F = *(RomPtr_A2(
        crate::rtl::rom_table_word!(game, 0xa2df6a, v2 as isize),
        game,
    ) as *mut u16);
    let r18: u16 = HirisingSlowfalling_Func_1(ram_word!(game, cur_enemy_index), game);
    HirisingSlowfalling_Func_2(ram_word!(game, cur_enemy_index), r18, game);
    (*E).hsg_var_03 = ((*E).hsg_var_01).wrapping_sub(r18);
    (*E).hsg_var_02 = (*E).hsg_var_00;
    (*E).hsg_var_B = (*E).hsg_var_06;
}
pub unsafe fn HirisingSlowfalling_Func_2(k: u16, r18: u16, game: &mut Game) {
    let mut v: u32 = 0u32;
    let mut r24: u16 = 0u16;
    loop {
        r24 = (r24).wrapping_add(512u16);
        let v1: i32 = 4i32 * (r24 as i32 >> 8i32);
        v = v.wrapping_add(
            (crate::rtl::rom_table_word!(game, 0xa2838f, (v1 + 1i32) as isize) as u32) << 16i32
                | crate::rtl::rom_table_word!(game, 0xa2838f, v1 as isize) as u32,
        );
        if ((v >> 16i32).wrapping_sub(r18 as u32) & 0x8000u32 == 0) {
            break;
        }
    }
    (*Get_HirisingSlowfalling(k as i32, game)).hsg_var_06 = r24;
}
pub unsafe fn HirisingSlowfalling_Func_3(a: u16, game: &mut Game) {
    let E = Get_HirisingSlowfalling(ram_word!(game, cur_enemy_index) as i32, game);
    enemy_instruction! {[(*E).base]; instruction = a, timer = 1u16}
    (*E).base.timer = 0u16;
}
crate::game::address_dispatch! {
    pub unsafe fn CallHirisingSlowfalling(ea: u32, game: &mut Game) => ea;
        call (game); default { Unreachable(game); };
    {
        10674229 => HirisingSlowfalling_Func_4; 10674255 => HirisingSlowfalling_Func_5;
        10674282 => HirisingSlowfalling_Func_6; 10674381 => HirisingSlowfalling_Func_7;
    }
}
pub unsafe fn HirisingSlowfalling_Main(game: &mut Game) {
    let E = Get_HirisingSlowfalling(ram_word!(game, cur_enemy_index) as i32, game);
    CallHirisingSlowfalling((((*E).hsg_var_A) as u32) | (0xa20000u32), game);
}
pub unsafe fn HirisingSlowfalling_Func_4(game: &mut Game) {
    if IsSamusWithinEnemy_X(ram_word!(game, cur_enemy_index), 0x50u16, game) != 0 {
        let E = Get_HirisingSlowfalling(ram_word!(game, cur_enemy_index) as i32, game);
        (*E).hsg_var_07 = (*E).hsg_parameter_2;
        (*E).hsg_var_A = ((0xa2e04fi32) as u16);
    }
}
pub unsafe fn HirisingSlowfalling_Func_5(game: &mut Game) {
    let E = Get_HirisingSlowfalling(ram_word!(game, cur_enemy_index) as i32, game);
    let v1: i16 = (((*E).hsg_var_07) as i16).wrapping_sub(1i16);
    (*E).hsg_var_07 = v1 as u16;
    if v1 < 0i16 {
        HirisingSlowfalling_Func_3(addr_kHirisingSlowfalling_Ilist_D834 as u16, game);
        (*E).hsg_var_A = ((0xa2e06ai32) as u16);
    }
}
pub unsafe fn HirisingSlowfalling_Func_6(game: &mut Game) {
    let E = Get_HirisingSlowfalling(ram_word!(game, cur_enemy_index) as i32, game);
    let v2: i32 = 4i32 * *(&raw mut (*E).hsg_var_B as *mut u8).offset(1isize) as i32;
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        (crate::rtl::rom_table_word!(game, 0xa2838f, (v2 + 3i32) as isize) as u32) << 16i32
            | crate::rtl::rom_table_word!(game, 0xa2838f, (v2 + 2i32) as isize) as u32,
    );
    let v5: i16 = (((*E).hsg_var_B) as i16).wrapping_sub(512i16);
    (*E).hsg_var_B = v5 as u16;
    if v5 < 0i16 {
        (*E).hsg_var_04 = (*E).hsg_var_02;
        (*E).base.x_pos = (*E).hsg_var_04;
        (*E).hsg_var_05 = (*E).hsg_var_03;
        (*E).base.y_pos = (*E).hsg_var_05;
        (*E).hsg_var_C = 0u16;
        (*E).hsg_var_D =
            ((*(&raw mut (*E).hsg_parameter_1 as *mut u8).offset(0)) as u16).wrapping_sub(1u16);
        HirisingSlowfalling_Func_3(addr_kHirisingSlowfalling_Ilist_D840 as u16, game);
        (*E).hsg_var_A = ((0xa2e0cdi32) as u16);
    }
}
pub unsafe fn HirisingSlowfalling_Func_7(game: &mut Game) {
    let E = Get_HirisingSlowfalling(ram_word!(game, cur_enemy_index) as i32, game);
    let v1 = RomPtr_A2(
        ((*E).hsg_var_E as i32 + 4i32 * *(&raw mut (*E).hsg_var_C as *mut u8).offset(1isize) as i32)
            as u16,
        game,
    );
    if *(v1 as *mut u16) == 0x8000u16 {
        (*E).hsg_var_05 = ((*E).hsg_var_05).wrapping_add((*E).hsg_var_F);
        (*E).hsg_var_C = 0u16;
        (*E).hsg_var_D = (*E).hsg_var_D.wrapping_sub(1);
        if ((*E).hsg_var_D & 0x8000) != 0 {
            (*E).hsg_var_B = (*E).hsg_var_06;
            (*E).base.x_pos = (*E).hsg_var_00;
            (*E).base.x_subpos = 0u16;
            (*E).base.y_pos = (*E).hsg_var_01;
            (*E).base.y_subpos = 0u16;
            HirisingSlowfalling_Func_3(addr_kHirisingSlowfalling_Ilist_D82C as u16, game);
            (*E).hsg_var_A = ((0xa2e035i32) as u16);
        }
    } else {
        (*E).base.x_pos = (*(v1 as *mut u16)).wrapping_add((*E).hsg_var_04);
        (*E).base.y_pos = (*(v1.offset(2) as *mut u16)).wrapping_add((*E).hsg_var_05);
        (*E).hsg_var_C = ((*E).hsg_var_C).wrapping_add(256u16);
    };
}
pub unsafe fn Gripper_Init(game: &mut Game) {
    let E = Get_Gripper(ram_word!(game, cur_enemy_index) as i32, game);
    let v1: u16 = (8u16)
        .wrapping_mul((*(&raw mut (*E).base.current_instruction as *mut u8).offset(0)) as u16);
    (*E).gripper_var_E = v1;
    let v2: i32 = v1 as i32 >> 1i32;
    let mut v3: u16 = 0;
    if ((*E).base.current_instruction & 0xfeff) != 0 {
        (*E).gripper_var_D = crate::rtl::rom_table_word!(game, 0xa28187, v2 as isize);
        v3 = crate::rtl::rom_table_word!(game, 0xa28187, (v2 + 1i32) as isize);
    } else {
        (*E).gripper_var_D = crate::rtl::rom_table_word!(game, 0xa28187, (v2 + 2i32) as isize);
        v3 = crate::rtl::rom_table_word!(game, 0xa28187, (v2 + 3i32) as isize);
    }
    (*E).gripper_var_C = v3;
    let mut v4: u16 = addr_kGripper_Ilist_E19B as u16;
    if ((*E).gripper_var_D & 0x8000) == 0 {
        v4 = addr_kGripper_Ilist_E1AF as u16;
    }
    (*E).base.current_instruction = v4;
    (*E).gripper_var_A = (*E).gripper_parameter_1;
    (*E).gripper_var_B = (*E).gripper_parameter_2;
}
pub unsafe fn Gripper_Main(game: &mut Game) {
    let E = Get_Gripper(ram_word!(game, cur_enemy_index) as i32, game);
    if Enemy_MoveRight_IgnoreSlopes(
        ram_word!(game, cur_enemy_index),
        (((*E).gripper_var_D as u32) << 16i32 | (*E).gripper_var_C as u32) as i32,
        game,
    ) != 0u8
        || Gripper_Func_1(ram_word!(game, cur_enemy_index), game) & 1 != 0
        || Gripper_Func_2(ram_word!(game, cur_enemy_index), game) & 1 != 0
    {
        let v3: i32 = (*E).gripper_var_E as i32 >> 1i32;
        let mut v4: u16 = 0;
        if ((*E).gripper_var_D & 0x8000) == 0 {
            (*E).gripper_var_D = crate::rtl::rom_table_word!(game, 0xa28187, (v3 + 2i32) as isize);
            (*E).gripper_var_C = crate::rtl::rom_table_word!(game, 0xa28187, (v3 + 3i32) as isize);
            v4 = addr_kGripper_Ilist_E19B as u16;
        } else {
            (*E).gripper_var_D = crate::rtl::rom_table_word!(game, 0xa28187, v3 as isize);
            (*E).gripper_var_C = crate::rtl::rom_table_word!(game, 0xa28187, (v3 + 1i32) as isize);
            v4 = addr_kGripper_Ilist_E1AF as u16;
        }
        enemy_instruction! {[(*E).base]; instruction = v4, timer = 1u16}
        (*E).base.timer = 0u16;
    }
}
pub unsafe fn Gripper_Func_1(k: u16, game: &mut Game) -> u8 {
    let E = Get_Gripper(k as i32, game);
    (((*E).gripper_var_D & 0x8000) != 0
        && (((*E).base.x_pos) as i16).wrapping_sub(((*E).gripper_var_A) as i16) < 0i16) as u8
}
pub unsafe fn Gripper_Func_2(k: u16, game: &mut Game) -> u8 {
    let E = Get_Gripper(k as i32, game);
    (((*E).gripper_var_D & 0x8000) == 0
        && (((*E).base.x_pos) as i16).wrapping_sub(((*E).gripper_var_B) as i16) >= 0i16) as u8
}
pub unsafe fn Gripper_Func_3(game: &mut Game) {
    Enemy_NormalFrozenAI_A2(game);
}
pub unsafe fn Gripper_Func_4(game: &mut Game) {
    NormalEnemyShotAi(game);
    let E = Get_Gripper(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).base.frozen_timer != 0 {
        let mut v1: u16 = addr_kGripper_Sprmap_E43F as u16;
        if ((*E).gripper_var_D & 0x8000) == 0 {
            v1 = addr_kGripper_Sprmap_E44B as u16;
        }
        (*E).base.spritemap_pointer = v1;
    }
}
pub unsafe fn JetPowerRipper_Init(game: &mut Game) {
    let mut v0: u16 = addr_kRipper_Ilist_E2E0 as u16;
    let E = Get_JetPowerRipper(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).jprr_parameter_2 != 0 {
        v0 = addr_kRipper_Ilist_E2F4 as u16;
    }
    (*E).base.current_instruction = v0;
    let v2: u16 = (8u16).wrapping_mul((*E).jprr_parameter_1);
    (*E).jprr_var_E = v2;
    let v3: i32 = v2 as i32 >> 1i32;
    let mut v4: u16 = 0;
    if (*E).jprr_parameter_2 != 0 {
        (*E).jprr_var_D = crate::rtl::rom_table_word!(game, 0xa28187, v3 as isize);
        v4 = crate::rtl::rom_table_word!(game, 0xa28187, (v3 + 1i32) as isize);
    } else {
        (*E).jprr_var_D = crate::rtl::rom_table_word!(game, 0xa28187, (v3 + 2i32) as isize);
        v4 = crate::rtl::rom_table_word!(game, 0xa28187, (v3 + 3i32) as isize);
    }
    (*E).jprr_var_C = v4;
}
pub unsafe fn JetPowerRipper_Main(game: &mut Game) {
    let mut v3: i16 = 0;
    let E = Get_JetPowerRipper(ram_word!(game, cur_enemy_index) as i32, game);
    if Enemy_MoveRight_IgnoreSlopes(
        ram_word!(game, cur_enemy_index),
        (((*E).jprr_var_D as u32) << 16i32 | (*E).jprr_var_C as u32) as i32,
        game,
    ) != 0
    {
        let v2: i32 = (*E).jprr_var_E as i32 >> 1i32;
        if ((*E).jprr_var_D & 0x8000) == 0 {
            (*E).jprr_var_D = crate::rtl::rom_table_word!(game, 0xa28187, (v2 + 2i32) as isize);
            (*E).jprr_var_C = crate::rtl::rom_table_word!(game, 0xa28187, (v2 + 3i32) as isize);
            v3 = -7456i16;
        } else {
            (*E).jprr_var_D = crate::rtl::rom_table_word!(game, 0xa28187, v2 as isize);
            (*E).jprr_var_C = crate::rtl::rom_table_word!(game, 0xa28187, (v2 + 1i32) as isize);
            v3 = -7436i16;
        }
        enemy_instruction! {[(*E).base]; instruction = v3 as u16, timer = 1u16}
        (*E).base.timer = 0u16;
    }
}
pub unsafe fn Ripper_Func_1(game: &mut Game) {
    Enemy_NormalFrozenAI_A2(game);
}
pub unsafe fn JetPowerRipper_Shot(game: &mut Game) {
    NormalEnemyShotAi(game);
    let E = Get_JetPowerRipper(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).base.frozen_timer != 0 {
        let mut v1: i16 = -7105i16;
        if ((*E).jprr_var_D & 0x8000) == 0 {
            v1 = -7093i16;
        }
        (*E).base.spritemap_pointer = v1 as u16;
    }
}
pub unsafe fn Ripper_Init(game: &mut Game) {
    let mut v0: u16 = addr_kRipper_Ilist_E477 as u16;
    let E = Get_Ripper(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).ripper_parameter_2 == 0 {
        v0 = addr_kRipper_Ilist_E48B as u16;
    }
    (*E).base.current_instruction = v0;
    let v2: u16 = (8u16).wrapping_mul((*E).ripper_parameter_1);
    (*E).ripper_var_E = v2;
    let v3: i32 = v2 as i32 >> 1i32;
    let mut v4: u16 = 0;
    if (*E).ripper_parameter_2 != 0 {
        (*E).ripper_var_D = crate::rtl::rom_table_word!(game, 0xa28187, v3 as isize);
        v4 = crate::rtl::rom_table_word!(game, 0xa28187, (v3 + 1i32) as isize);
    } else {
        (*E).ripper_var_D = crate::rtl::rom_table_word!(game, 0xa28187, (v3 + 2i32) as isize);
        v4 = crate::rtl::rom_table_word!(game, 0xa28187, (v3 + 3i32) as isize);
    }
    (*E).ripper_var_C = v4;
}
pub unsafe fn Ripper_Main(game: &mut Game) {
    let mut v3: i16 = 0;
    let E = Get_Ripper(ram_word!(game, cur_enemy_index) as i32, game);
    if Enemy_MoveRight_IgnoreSlopes(
        ram_word!(game, cur_enemy_index),
        (((*E).ripper_var_D as u32) << 16i32 | (*E).ripper_var_C as u32) as i32,
        game,
    ) != 0
    {
        let v2: i32 = (*E).ripper_var_E as i32 >> 1i32;
        if ((*E).ripper_var_D & 0x8000) == 0 {
            (*E).ripper_var_D = crate::rtl::rom_table_word!(game, 0xa28187, (v2 + 2i32) as isize);
            (*E).ripper_var_C = crate::rtl::rom_table_word!(game, 0xa28187, (v2 + 3i32) as isize);
            v3 = -7029i16;
        } else {
            (*E).ripper_var_D = crate::rtl::rom_table_word!(game, 0xa28187, v2 as isize);
            (*E).ripper_var_C = crate::rtl::rom_table_word!(game, 0xa28187, (v2 + 1i32) as isize);
            v3 = -7049i16;
        }
        enemy_instruction! {[(*E).base]; instruction = v3 as u16, timer = 1u16}
        (*E).base.timer = 0u16;
    }
}
pub unsafe fn LavaSeahorse_Instr_E5FB(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*Get_LavaSeahorse(ram_word!(game, cur_enemy_index) as i32, game)).lse_var_02 = 1u16;
    jp
}
pub unsafe fn LavaSeahorse_Init(game: &mut Game) {
    let E = Get_LavaSeahorse(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).lse_var_02 = 0u16;
    if (*E).lse_parameter_1 != 0 {
        set_fields! {[(*E)], 2u16; lse_var_00, lse_var_01}
        (*E).base.current_instruction = addr_kLavaSeahorse_Ilist_E5A1 as u16;
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
        (*E).lse_var_F = ((0xa2e781i32) as u16);
    } else {
        set_fields! {[(*E)], 0u16; lse_var_00, lse_var_01}
        (*E).base.current_instruction = addr_kLavaSeahorse_Ilist_E59B as u16;
        (*E).lse_var_F = ((0xa2e654i32) as u16);
    };
}
pub unsafe fn LavaSeahorse_Main(game: &mut Game) {
    let E = Get_LavaSeahorse(ram_word!(game, cur_enemy_index) as i32, game);
    CallEnemyPreInstr((((*E).lse_var_F) as u32) | (0xa20000u32), game);
}
pub unsafe fn LavaSeahorse_Func_1(k: u16, game: &mut Game) {
    let E = Get_LavaSeahorse(k as i32, game);
    (*E).lse_var_D = (*E).lse_var_D.wrapping_sub(1);
    if ((*E).lse_var_D & 0x8000) != 0 {
        (*E).lse_var_D = 48u16;
        (*E).lse_var_F = ((0xa2e6adi32) as u16);
        let t: u16 = GetSamusEnemyDelta_X(k, game);
        (*E).lse_var_A = ((*E).lse_var_A & 0x7fff | t & 0x8000);
        if ((*E).lse_var_A & 0x8000) != 0 {
            crate::update_place! { reference fresh16 = *(&raw mut (*E).lse_var_00 as *mut u8).offset(0) => (*fresh16 & !(1)); }
            let E1 = Get_LavaSeahorse(k as i32 + 64i32, game);
            crate::update_place! { reference fresh17 = *(&raw mut (*E1).lse_var_00 as *mut u8).offset(0) => (*fresh17 & !(1)); }
        } else {
            crate::update_place! { reference fresh18 = *(&raw mut (*E).lse_var_00 as *mut u8).offset(0) => (*fresh18) | (1u8); }
            let E1_0 = Get_LavaSeahorse(k as i32 + 64i32, game);
            crate::update_place! { reference fresh19 = *(&raw mut (*E1_0).lse_var_00 as *mut u8).offset(0) => (*fresh19) | (1u8); }
        }
        LavaSeahorse_Func_6(game);
        LavaSeahorse_Func_7(game);
    }
}
pub unsafe fn LavaSeahorse_Func_2(k: u16, game: &mut Game) {
    let E = Get_LavaSeahorse(k as i32, game);
    (*E).lse_var_D = (*E).lse_var_D.wrapping_sub(1);
    if ((*E).lse_var_D & 0x8000) != 0 {
        (*E).lse_var_00 = ((*E).lse_var_00).wrapping_add(4u16);
        (*E).lse_var_D = 3u16;
        (*E).lse_var_F = ((0xa2e6f1i32) as u16);
    }
    (*E).base.y_pos = (*E).base.y_pos.wrapping_sub(1);
    let E1 = Get_LavaSeahorse(k as i32 + 64i32, game);
    (*E1).base.y_pos = (*E1).base.y_pos.wrapping_sub(1);
}
pub unsafe fn LavaSeahorse_Func_3(k: u16, game: &mut Game) {
    LavaSeahorse_Func_6(game);
    let E = Get_LavaSeahorse(k as i32, game);
    if (*E).lse_var_02 != 0 {
        (*E).lse_var_02 = 0u16;
        (*E).lse_var_01 = -1i32 as u16;
        SpawnEprojWithGfx(0xffffu16, k, addr_kEproj_LavaSeahorseFireball as u16, game);
        QueueSfx2_Max6(0x61u16, game);
        crate::update_place! { previous fresh20 = (*E).lse_var_D => (*E).lse_var_D.wrapping_sub(1); }
        if fresh20 == 1u16 {
            (*E).lse_var_00 = ((*E).lse_var_00).wrapping_sub(4u16);
            (*E).lse_var_D = 96u16;
            (*E).lse_var_F = ((0xa2e734i32) as u16);
        }
    }
}
pub unsafe fn LavaSeahorse_Func_4(k: u16, game: &mut Game) {
    let E = Get_LavaSeahorse(k as i32, game);
    crate::update_place! { previous fresh21 = (*E).lse_var_D => (*E).lse_var_D.wrapping_sub(1); }
    if fresh21 == 1u16 {
        (*E).lse_var_D = 48u16;
        (*E).lse_var_F = ((0xa2e749i32) as u16);
        LavaSeahorse_Func_6(game);
    }
}
pub unsafe fn LavaSeahorse_Func_5(k: u16, game: &mut Game) {
    let E = Get_LavaSeahorse(k as i32, game);
    (*E).lse_var_D = (*E).lse_var_D.wrapping_sub(1);
    if ((*E).lse_var_D & 0x8000) != 0 {
        (*E).lse_var_D = 128u16;
        (*E).lse_var_F = ((0xa2e654i32) as u16);
    }
    (*E).base.y_pos = (*E).base.y_pos.wrapping_add(1);
    let E1 = Get_LavaSeahorse(k as i32 + 64i32, game);
    (*E1).base.y_pos = (*E1).base.y_pos.wrapping_add(1);
}
pub unsafe fn LavaSeahorse_Func_6(game: &mut Game) {
    let E = Get_LavaSeahorse(ram_word!(game, cur_enemy_index) as i32, game);
    let lse_var_00: u16 = (*E).lse_var_00;
    if lse_var_00 != (*E).lse_var_01 {
        (*E).lse_var_01 = lse_var_00;
        enemy_instruction! {[(*E).base]; instruction = crate::rtl::rom_table_word!(game, 0xa2e5ef, lse_var_00 as isize), timer = 1u16}
        (*E).base.timer = 0u16;
    }
}
pub unsafe fn LavaSeahorse_Func_7(game: &mut Game) {
    let E = Get_LavaSeahorse(ram_word!(game, cur_enemy_index) as i32 + 64i32, game);
    let lse_var_00: u16 = (*E).lse_var_00;
    if lse_var_00 != (*E).lse_var_01 {
        (*E).lse_var_01 = lse_var_00;
        enemy_instruction! {[(*E).base]; instruction = crate::rtl::rom_table_word!(game, 0xa2e5ef, lse_var_00 as isize), timer = 1u16}
        (*E).base.timer = 0u16;
    }
}
pub unsafe fn LavaSeahorse_Touch(game: &mut Game) {
    Enemy_NormalTouchAI_A2(game);
    LavaSeahorse_E7DA(game);
}
pub unsafe fn LavaSeahorse_Shot(game: &mut Game) {
    Enemy_NormalShotAI_A2(game);
    LavaSeahorse_E7DA(game);
}
pub unsafe fn LavaSeahorse_Powerbomb(game: &mut Game) {
    Enemy_NormalPowerBombAI_A2(game);
    LavaSeahorse_E7DA(game);
}
pub unsafe fn LavaSeahorse_E7DA(game: &mut Game) {
    let E = Get_LavaSeahorse(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).base.health != 0 {
        let shake_timer: u16 = (*E).base.shake_timer;
        let E1 = Get_LavaSeahorse(ram_word!(game, cur_enemy_index) as i32 + 64i32, game);
        (*E1).base.shake_timer = shake_timer;
        (*E1).base.invincibility_timer = (*E).base.invincibility_timer;
        (*E1).base.flash_timer = (*E).base.flash_timer;
        (*E1).base.frozen_timer = (*E).base.frozen_timer;
        (*E1).base.ai_handler_bits = (*E).base.ai_handler_bits;
    } else {
        let E1_0 = Get_LavaSeahorse(ram_word!(game, cur_enemy_index) as i32 + 64i32, game);
        (*E1_0).base.properties = ((*E1_0).base.properties) | (0x200u16);
    };
}
pub unsafe fn TimedShutter_Init(game: &mut Game) {
    let E = Get_TimedShutter(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).tsr_var_A = crate::rtl::rom_table_word!(
        game,
        0xa2ea4e,
        ((*E).base.current_instruction as i32 + 2i32 * (*E).base.extra_properties as i32) as isize,
    );
    let y_pos: u16 = (*E).base.y_pos;
    let mut v3: u16 = 0;
    if (*E).base.extra_properties != 0 {
        (*E).tsr_var_B = y_pos;
        let mut v2: u16 = (y_pos).wrapping_sub(8u16);
        (*E).tsr_var_C = v2;
        v2 = (v2).wrapping_sub(8u16);
        (*E).tsr_var_D = v2;
        v3 = (v2).wrapping_sub(8u16);
    } else {
        (*E).tsr_var_B = y_pos;
        let mut v4: u16 = (y_pos).wrapping_add(8u16);
        (*E).tsr_var_C = v4;
        v4 = (v4).wrapping_add(8u16);
        (*E).tsr_var_D = v4;
        v3 = (v4).wrapping_add(8u16);
    }
    (*E).tsr_var_E = v3;
    (*E).base.extra_properties = 0u16;
    (*E).tsr_var_F = 0u16;
    (*E).base.current_instruction = addr_kTimedShutter_Ilist_E998 as u16;
    let v5: i32 = ((4u16)
        .wrapping_mul((*(&raw mut (*E).tsr_parameter_2 as *mut u8).offset(0)) as u16))
        as i32
        >> 1i32;
    (*E).tsr_var_00 = crate::rtl::rom_table_word!(game, 0xa2ea56, v5 as isize);
    (*E).tsr_var_01 = crate::rtl::rom_table_word!(game, 0xa2ea56, (v5 + 1i32) as isize);
}
crate::game::address_dispatch! {
    pub unsafe fn CallTimedShutterFunc(ea: u32, k: u16, game: &mut Game) => ea;
        call (k, game); default { Unreachable(game); };
    {
        10676925 => TimedShutter_Func_1; 10676945 => TimedShutter_Func_2;
        10676967 => TimedShutter_Func_3; 10676989 => TimedShutter_Func_4;
        10677009 => TimedShutter_Func_5; 10677267 => TimedShutter_Func_10;
    }
}
pub unsafe fn TimedShutter_Main(game: &mut Game) {
    let E = Get_TimedShutter(ram_word!(game, cur_enemy_index) as i32, game);
    CallTimedShutterFunc(
        (((*E).tsr_var_A) as u32) | (0xa20000u32),
        ram_word!(game, cur_enemy_index),
        game,
    );
}
pub unsafe fn TimedShutter_Func_1(k: u16, game: &mut Game) {
    let E = Get_TimedShutter(k as i32, game);
    let tsr_parameter_1: u16 = (*E).tsr_parameter_1;
    if tsr_parameter_1 != 0 {
        (*E).tsr_parameter_1 = (tsr_parameter_1).wrapping_sub(1u16);
    } else {
        RisingFallingPlatform_Func_8(game);
        (*E).tsr_var_A = ((0xa2ec13i32) as u16);
    };
}
pub unsafe fn TimedShutter_Func_2(k: u16, game: &mut Game) {
    let E = Get_TimedShutter(k as i32, game);
    if IsSamusWithinEnemy_X(k, (*E).tsr_parameter_1, game) != 0 {
        RisingFallingPlatform_Func_8(game);
        (*E).tsr_var_A = ((0xa2ec13i32) as u16);
    }
}
pub unsafe fn TimedShutter_Func_3(k: u16, game: &mut Game) {
    let E = Get_TimedShutter(k as i32, game);
    if IsSamusWithinEnemy_X(k, (*E).tsr_parameter_1, game) != 0 {
        RisingFallingPlatform_Func_8(game);
        (*E).tsr_var_A = ((0xa2eb11i32) as u16);
    }
}
pub unsafe fn TimedShutter_Func_4(k: u16, game: &mut Game) {
    let E = Get_TimedShutter(k as i32, game);
    let tsr_parameter_1: u16 = (*E).tsr_parameter_1;
    if tsr_parameter_1 != 0 {
        (*E).tsr_parameter_1 = (tsr_parameter_1).wrapping_sub(1u16);
    } else {
        RisingFallingPlatform_Func_8(game);
        (*E).tsr_var_A = ((0xa2eb11i32) as u16);
    };
}
pub fn TimedShutter_Func_Null(game: &mut Game) {}
static off_A2EB1A: [Option<Func_V>; 5] = callback_table! { unsafe Func_V;
    TimedShutter_Func_6, TimedShutter_Func_7, TimedShutter_Func_8, TimedShutter_Func_9,
    TimedShutter_Func_Null,
};
pub unsafe fn TimedShutter_Func_5(k: u16, game: &mut Game) {
    let v1: i32 = (*Get_TimedShutter(k as i32, game)).tsr_var_F as i32;
    off_A2EB1A[v1 as usize].expect("non-null function pointer")(game);
}
pub unsafe fn TimedShutter_Func_6(game: &mut Game) {
    let E = Get_TimedShutter(ram_word!(game, cur_enemy_index) as i32, game);
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        ((*E).tsr_var_00 as u32) << 16i32 | (*E).tsr_var_01 as u32,
    );
    if (((*E).tsr_var_B as i32 + 16i32 - (*E).base.y_pos as i32) as i16) < 0i16 {
        (*E).base.y_pos = ((*E).tsr_var_B).wrapping_add(9u16);
        (*E).tsr_var_F = (*E).tsr_var_F.wrapping_add(1);
        enemy_instruction! {[(*E).base]; timer = 1u16, instruction = addr_kTimedShutter_Ilist_E99E as u16}
        (*E).base.y_height = 16u16;
    }
}
pub unsafe fn TimedShutter_Func_7(game: &mut Game) {
    let E = Get_TimedShutter(ram_word!(game, cur_enemy_index) as i32, game);
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        ((*E).tsr_var_00 as u32) << 16i32 | (*E).tsr_var_01 as u32,
    );
    if (((*E).tsr_var_C as i32 + 16i32 - (*E).base.y_pos as i32) as i16) < 0i16 {
        (*E).base.y_pos = ((*E).tsr_var_C).wrapping_add(9u16);
        (*E).tsr_var_F = (*E).tsr_var_F.wrapping_add(1);
        enemy_instruction! {[(*E).base]; timer = 1u16, instruction = addr_kTimedShutter_Ilist_E9A4 as u16}
        (*E).base.y_height = 24u16;
    }
}
pub unsafe fn TimedShutter_Func_8(game: &mut Game) {
    let E = Get_TimedShutter(ram_word!(game, cur_enemy_index) as i32, game);
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        ((*E).tsr_var_00 as u32) << 16i32 | (*E).tsr_var_01 as u32,
    );
    if (((*E).tsr_var_D as i32 + 16i32 - (*E).base.y_pos as i32) as i16) < 0i16 {
        (*E).base.y_pos = ((*E).tsr_var_D).wrapping_add(9u16);
        (*E).tsr_var_F = (*E).tsr_var_F.wrapping_add(1);
        enemy_instruction! {[(*E).base]; timer = 1u16, instruction = addr_kTimedShutter_Ilist_E9AA as u16}
        (*E).base.y_height = 32u16;
    }
}
pub unsafe fn TimedShutter_Func_9(game: &mut Game) {
    let E = Get_TimedShutter(ram_word!(game, cur_enemy_index) as i32, game);
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        ((*E).tsr_var_00 as u32) << 16i32 | (*E).tsr_var_01 as u32,
    );
    let v3: u16 = ((*E).tsr_var_E).wrapping_add(16u16);
    if ((v3) as i16).wrapping_sub(((*E).base.y_pos) as i16) < 0i16 {
        (*E).base.y_pos = v3;
        (*E).tsr_var_F = (*E).tsr_var_F.wrapping_add(1);
    }
}
pub fn TimedShutter_Func_Null2(game: &mut Game) {}
static off_A2EC3A: [Option<Func_V>; 5] = callback_table! { unsafe Func_V;
    TimedShutter_Func_11, TimedShutter_Func_12, TimedShutter_Func_13, TimedShutter_Func_14,
    TimedShutter_Func_Null2,
};
pub unsafe fn TimedShutter_Func_10(k: u16, game: &mut Game) {
    let E = Get_TimedShutter(k as i32, game);
    (*E).tsr_var_40 = (*E).base.y_pos;
    let v2: u16 = (2u16).wrapping_mul((*E).tsr_var_F);
    off_A2EC3A[(v2 >> 1) as usize].expect("non-null function pointer")(game);
    if CheckIfEnemyTouchesSamus(k, game) != 0 {
        let v4: i16 = (((*E).base.y_pos) as i16).wrapping_sub(((*E).tsr_var_40) as i16);
        if v4 < 0i16 {
            update_ram_word!(game.extra_samus_y_displacement, wrapping_add((v4) as u16));
        }
    }
}
pub unsafe fn TimedShutter_Func_11(game: &mut Game) {
    let E = Get_TimedShutter(ram_word!(game, cur_enemy_index) as i32, game);
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        -(((*E).tsr_var_00 as i32) << 16i32 | (*E).tsr_var_01 as i32) as u32,
    );
    let v3: i16 = (((*E).tsr_var_B) as i16).wrapping_sub(16i16);
    if (v3).wrapping_sub(((*E).base.y_pos) as i16) >= 0i16 {
        (*E).base.y_pos = ((v3) as u16).wrapping_add(7u16);
        (*E).tsr_var_F = (*E).tsr_var_F.wrapping_add(1);
        enemy_instruction! {[(*E).base]; timer = 1u16, instruction = addr_kTimedShutter_Ilist_E99E as u16}
        (*E).base.y_height = 16u16;
    }
}
pub unsafe fn TimedShutter_Func_12(game: &mut Game) {
    let E = Get_TimedShutter(ram_word!(game, cur_enemy_index) as i32, game);
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        -(((*E).tsr_var_00 as i32) << 16i32 | (*E).tsr_var_01 as i32) as u32,
    );
    let v3: i16 = (((*E).tsr_var_C) as i16).wrapping_sub(16i16);
    if (v3).wrapping_sub(((*E).base.y_pos) as i16) >= 0i16 {
        (*E).base.y_pos = ((v3) as u16).wrapping_add(7u16);
        (*E).tsr_var_F = (*E).tsr_var_F.wrapping_add(1);
        enemy_instruction! {[(*E).base]; timer = 1u16, instruction = addr_kTimedShutter_Ilist_E9A4 as u16}
        (*E).base.y_height = 24u16;
    }
}
pub unsafe fn TimedShutter_Func_13(game: &mut Game) {
    let E = Get_TimedShutter(ram_word!(game, cur_enemy_index) as i32, game);
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        -(((*E).tsr_var_00 as i32) << 16i32 | (*E).tsr_var_01 as i32) as u32,
    );
    let v3: i16 = (((*E).tsr_var_D) as i16).wrapping_sub(16i16);
    if (v3).wrapping_sub(((*E).base.y_pos) as i16) >= 0i16 {
        (*E).base.y_pos = ((v3) as u16).wrapping_add(7u16);
        (*E).tsr_var_F = (*E).tsr_var_F.wrapping_add(1);
        enemy_instruction! {[(*E).base]; timer = 1u16, instruction = addr_kTimedShutter_Ilist_E9AA as u16}
        (*E).base.y_height = 32u16;
    }
}
pub unsafe fn TimedShutter_Func_14(game: &mut Game) {
    let E = Get_TimedShutter(ram_word!(game, cur_enemy_index) as i32, game);
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        -(((*E).tsr_var_00 as i32) << 16i32 | (*E).tsr_var_01 as i32) as u32,
    );
    let v3: u16 = ((*E).tsr_var_E).wrapping_sub(16u16);
    if ((v3) as i16).wrapping_sub(((*E).base.y_pos) as i16) >= 0i16 {
        (*E).base.y_pos = v3;
        (*E).tsr_var_F = (*E).tsr_var_F.wrapping_add(1);
    }
}
pub fn TimedShutter_Func_15() {}
pub unsafe fn RisingFallingPlatform_Init(game: &mut Game) {
    RisingFallingPlatform_Func_1(ram_word!(game, cur_enemy_index), game);
    (*Get_RisingFallingPlatform(ram_word!(game, cur_enemy_index) as i32, game))
        .base
        .current_instruction = addr_kRisingFallingPlatform_Ilist_EDE7 as u16;
}
pub unsafe fn ShootableShutter_Init(game: &mut Game) {
    RisingFallingPlatform_Func_1(ram_word!(game, cur_enemy_index), game);
    (*gEnemyData(ram_word!(game, cur_enemy_index) as i32, game)).current_instruction =
        addr_kTimedShutter_Ilist_E9AA as u16;
}
pub unsafe fn RisingFallingPlatform_Func_1(k: u16, game: &mut Game) {
    let E = Get_RisingFallingPlatform(k as i32, game);
    let current_instruction_low: u16 =
        *(&raw mut (*E).base.current_instruction as *mut u8).offset(0) as u16;
    (*E).rfpm_var_00 = current_instruction_low;
    let v3: i32 = 8i32 * current_instruction_low as i32 >> 1i32;
    (*E).rfpm_var_D = crate::rtl::rom_table_word!(game, 0xa28187, v3 as isize);
    (*E).rfpm_var_C = crate::rtl::rom_table_word!(game, 0xa28187, (v3 + 1i32) as isize);
    (*E).rfpm_var_F = crate::rtl::rom_table_word!(game, 0xa28187, (v3 + 2i32) as isize);
    (*E).rfpm_var_E = crate::rtl::rom_table_word!(game, 0xa28187, (v3 + 3i32) as isize);
    let current_instruction_high: u16 =
        *(&raw mut (*E).base.current_instruction as *mut u8).offset(1isize) as u16;
    (*E).rfpm_var_01 = current_instruction_high;
    (*E).rfpm_var_20 = current_instruction_high;
    let extra_properties_low: u16 =
        *(&raw mut (*E).base.extra_properties as *mut u8).offset(0) as u16;
    (*E).rfpm_var_02 = extra_properties_low;
    (*E).rfpm_var_08 = (16u16).wrapping_mul(extra_properties_low);
    let extra_properties_high: u16 =
        *(&raw mut (*E).base.extra_properties as *mut u8).offset(1isize) as u16;
    (*E).rfpm_var_03 = extra_properties_high;
    (*E).rfpm_var_09 = (16u16).wrapping_mul(extra_properties_high);
    let rfpm_parameter_1_low: u16 = *(&raw mut (*E).rfpm_parameter_1 as *mut u8).offset(0) as u16;
    (*E).rfpm_var_04 = rfpm_parameter_1_low;
    (*E).rfpm_var_07 = (2u16).wrapping_mul(rfpm_parameter_1_low);
    (*E).rfpm_var_05 = *(&raw mut (*E).rfpm_parameter_1 as *mut u8).offset(1isize) as u16;
    let rfpm_parameter_2: u16 = (*E).rfpm_parameter_2;
    (*E).rfpm_var_06 = rfpm_parameter_2;
    (*E).rfpm_var_B = rfpm_parameter_2;
    let y_pos: u16 = (*E).base.y_pos;
    (*E).rfpm_var_0F = y_pos;
    (*E).rfpm_var_10 = ((*E).rfpm_var_05).wrapping_add(y_pos);
    if (*E).rfpm_var_01 == 0 {
        let v10: u16 = (*E).base.y_pos;
        (*E).rfpm_var_10 = v10;
        (*E).rfpm_var_0F = (v10).wrapping_sub((*E).rfpm_var_05);
    }
    (*E).rfpm_var_A = ((0xa2ef09i32) as u16);
    (*E).base.extra_properties = 0u16;
    (*E).rfpm_var_0A = 0u16;
}
crate::game::address_dispatch! {
    pub unsafe fn CallRisingFallingPlatformFunc(ea: u32, game: &mut Game) => ea;
        call (game); default { Unreachable(game); };
    {
        10678025 => RisingFallingPlatform_Func_2; 10678056 => RisingFallingPlatform_Func_4;
        10678120 => RisingFallingPlatform_Func_9; 10678228 => RisingFallingPlatform_Func_10;
        10678336 => RisingFallingPlatform_Func_11; 10678386 => RisingFallingPlatform_Func_12;
        10678425 => [RisingFallingPlatform_Func_13()];
        10678820 => HorizontalShootableShutter_Func_2;
        10678851 => HorizontalShootableShutter_Func_4;
        10678898 => HorizontalShootableShutter_Func_8;
        10679012 => HorizontalShootableShutter_Func_9;
        10679180 => HorizontalShootableShutter_Func_12;
        10679216 => HorizontalShootableShutter_Func_13;
        10679252 => [HorizontalShootableShutter_Func_14()];
    }
}
pub unsafe fn RisingFallingPlatform_Main(game: &mut Game) {
    let E = Get_RisingFallingPlatform(ram_word!(game, cur_enemy_index) as i32, game);
    CallRisingFallingPlatformFunc((((*E).rfpm_var_A) as u32) | (0xa20000u32), game);
    if (*E).rfpm_var_A as i32 != 0xa2ef68i32 & 0xffffi32
        && (*E).rfpm_var_A as i32 != 0xa2efd4i32 & 0xffffi32
        && (ram_word!(game, enemy_index_colliding_dirs[3])
            & (ram_word!(game, enemy_index_colliding_dirs[2])
                & ram_word!(game, enemy_index_colliding_dirs[1])
                & ram_word!(game, enemy_index_colliding_dirs[0])))
            != 0xffff
        && ram_word!(game, enemy_index_colliding_dirs[3])
            & (ram_word!(game, enemy_index_colliding_dirs[2])
                & ram_word!(game, enemy_index_colliding_dirs[1])
                & ram_word!(game, enemy_index_colliding_dirs[0]))
            == ram_word!(game, cur_enemy_index)
        && ram_word!(game, samus_contact_damage_index) != 0
    {
        RisingFallingPlatform_Powerbomb(game);
    }
}
static off_A2EDFB: [Option<Func_V>; 5] = callback_table! { unsafe Func_V;
    RisingFallingPlatform_Func_3, RisingFallingPlatform_Func_4, RisingFallingPlatform_Func_5,
    RisingFallingPlatform_Func_6, RisingFallingPlatform_Func_6,
};
pub unsafe fn RisingFallingPlatform_Func_2(game: &mut Game) {
    let v0: i32 = (*Get_RisingFallingPlatform(ram_word!(game, cur_enemy_index) as i32, game))
        .rfpm_var_07 as i32
        >> 1i32;
    off_A2EDFB[v0 as usize].expect("non-null function pointer")(game);
}
pub unsafe fn RisingFallingPlatform_Func_3(game: &mut Game) {
    let E = Get_RisingFallingPlatform(ram_word!(game, cur_enemy_index) as i32, game);
    crate::update_place! { previous fresh15 = (*E).rfpm_var_B => (*E).rfpm_var_B.wrapping_sub(1); }
    if fresh15 == 1u16 {
        (*E).rfpm_var_B = (*E).rfpm_var_06;
        RisingFallingPlatform_Func_7(ram_word!(game, cur_enemy_index), game);
    }
}
pub unsafe fn RisingFallingPlatform_Func_4(game: &mut Game) {
    let E = Get_RisingFallingPlatform(ram_word!(game, cur_enemy_index) as i32, game);
    if IsSamusWithinEnemy_X(ram_word!(game, cur_enemy_index), (*E).rfpm_var_06, game) != 0 {
        RisingFallingPlatform_Func_7(ram_word!(game, cur_enemy_index), game);
    }
}
pub unsafe fn RisingFallingPlatform_Func_5(game: &mut Game) {
    RisingFallingPlatform_Func_7(ram_word!(game, cur_enemy_index), game);
}
pub fn RisingFallingPlatform_Func_6(game: &mut Game) {}
pub unsafe fn RisingFallingPlatform_Func_7(k: u16, game: &mut Game) {
    let E = Get_RisingFallingPlatform(k as i32, game);
    (*E).rfpm_var_A = ((0xa2ef68i32) as u16);
    if (*E).rfpm_var_01 != 0 {
        (*E).rfpm_var_A = ((0xa2efd4i32) as u16);
    }
    RisingFallingPlatform_Func_8(game);
}
pub unsafe fn RisingFallingPlatform_Func_8(game: &mut Game) {
    if CheckIfEnemyIsOnScreen(game) == 0 {
        QueueSfx3_Max6(0xeu16, game);
    }
}
pub unsafe fn RisingFallingPlatform_Func_9(game: &mut Game) {
    let E = Get_RisingFallingPlatform(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).rfpm_var_0E = (*E).base.y_pos;
    (*E).rfpm_var_0A = 0u16;
    if CheckIfEnemyTouchesSamus(ram_word!(game, cur_enemy_index), game) != 0 {
        (*E).rfpm_var_0A = 1u16;
    }
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        ((*E).rfpm_var_F as u32) << 16i32 | (*E).rfpm_var_E as u32,
    );
    if (*E).rfpm_var_0A != 0 {
        set_ram_word!(
            game.extra_samus_y_displacement = ((*E).base.y_pos).wrapping_sub((*E).rfpm_var_0E)
        );
    }
    if (((*E).rfpm_var_0F) as i16).wrapping_sub(((*E).base.y_pos) as i16) >= 0i16 {
        if (*E).rfpm_var_08 == 4080u16 {
            (*E).rfpm_var_A = ((0xa2f099i32) as u16);
        } else {
            (*E).rfpm_var_B = (*E).rfpm_var_08;
            (*E).rfpm_var_A = ((0xa2f040i32) as u16);
        }
    }
}
pub unsafe fn RisingFallingPlatform_Func_10(game: &mut Game) {
    let E = Get_RisingFallingPlatform(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).rfpm_var_0E = (*E).base.y_pos;
    (*E).rfpm_var_0A = 0u16;
    if CheckIfEnemyTouchesSamus(ram_word!(game, cur_enemy_index), game) != 0 {
        (*E).rfpm_var_0A = 1u16;
    }
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        ((*E).rfpm_var_D as u32) << 16i32 | (*E).rfpm_var_C as u32,
    );
    if (*E).rfpm_var_0A != 0 {
        set_ram_word!(
            game.extra_samus_y_displacement = ((*E).base.y_pos).wrapping_sub((*E).rfpm_var_0E)
        );
    }
    if (((*E).base.y_pos) as i16).wrapping_sub(((*E).rfpm_var_10) as i16) >= 0i16 {
        if (*E).rfpm_var_09 == 4080u16 {
            (*E).rfpm_var_A = ((0xa2f099i32) as u16);
        } else {
            (*E).rfpm_var_B = (*E).rfpm_var_09;
            (*E).rfpm_var_A = ((0xa2f072i32) as u16);
        }
    }
}
pub unsafe fn RisingFallingPlatform_Func_11(game: &mut Game) {
    let E = Get_RisingFallingPlatform(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).rfpm_var_B = (*E).rfpm_var_B.wrapping_sub(1);
    if ((*E).rfpm_var_B & 0x8000) != 0 {
        (*E).rfpm_var_A = ((0xa2efd4i32) as u16);
        if (*E).rfpm_var_04 == 1u16 && (*E).rfpm_var_01 != 0u16 {
            (*E).rfpm_var_A = ((0xa2ef28i32) as u16);
        }
        if (*Get_RisingFallingPlatform(ram_word!(game, cur_enemy_index) as i32, game))
            .base
            .enemy_ptr
            != 0xd83fu16
        {
            RisingFallingPlatform_Func_8(game);
        }
    }
}
pub unsafe fn RisingFallingPlatform_Func_12(game: &mut Game) {
    let E = Get_RisingFallingPlatform(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).rfpm_var_B = (*E).rfpm_var_B.wrapping_sub(1);
    if ((*E).rfpm_var_B & 0x8000) != 0 {
        RisingFallingPlatform_Func_8(game);
        (*E).rfpm_var_A = ((0xa2ef68i32) as u16);
        if (*E).rfpm_var_04 == 1u16 && (*E).rfpm_var_01 == 0 {
            (*E).rfpm_var_A = ((0xa2ef28i32) as u16);
        }
    }
}
pub fn RisingFallingPlatform_Func_13() {}
crate::forward_calls! { pub unsafe fn (game: &mut Game);
    RisingFallingPlatform_Touch => RisingFallingPlatform_Powerbomb(game);
    RisingFallingPlatform_Shot => RisingFallingPlatform_Powerbomb(game);
}
pub unsafe fn ShootableShutter_Shot(game: &mut Game) {
    NormalEnemyShotAi(game);
    RisingFallingPlatform_Powerbomb(game);
}
pub unsafe fn RisingFallingPlatform_Powerbomb(game: &mut Game) {
    let E = Get_RisingFallingPlatform(ram_word!(game, cur_enemy_index) as i32, game);
    if ((*E).rfpm_var_07 as i32 - 6i32 & 0x8000i32 == 0) {
        if (*E).rfpm_var_07 != 8u16 {
            if (*E).rfpm_var_0C != 0 {
                return;
            }
            (*E).rfpm_var_0C = 1u16;
        }
        if (*E).rfpm_var_A as i32 != 0xa2ef68i32 & 0xffffi32
            && (*E).rfpm_var_A as i32 != 0xa2efd4i32 & 0xffffi32
        {
            (*E).rfpm_var_A = ((0xa2ef68i32) as u16);
            if (*E).rfpm_var_20 != 0 {
                (*E).rfpm_var_A = ((0xa2efd4i32) as u16);
            }
            (*E).rfpm_var_20 = ((*E).rfpm_var_20) ^ (1u16);
        } else {
            return;
        }
    }
    RisingFallingPlatform_Func_8(game);
}
pub unsafe fn HorizontalShootableShutter_Init(game: &mut Game) {
    HorizontalShootableShutter_Func_1(ram_word!(game, cur_enemy_index), game);
    (*Get_HorizontalShootableShutter(ram_word!(game, cur_enemy_index) as i32, game))
        .base
        .current_instruction = addr_kTimedShutter_Ilist_E9D4 as u16;
}
pub unsafe fn HorizontalShootableShutter_Func_1(k: u16, game: &mut Game) {
    let E = Get_RisingFallingPlatform(k as i32, game);
    let current_instruction_low: u16 =
        *(&raw mut (*E).base.current_instruction as *mut u8).offset(0) as u16;
    (*E).rfpm_var_00 = current_instruction_low;
    let v3: i32 = 8i32 * current_instruction_low as i32 >> 1i32;
    (*E).rfpm_var_D = crate::rtl::rom_table_word!(game, 0xa28187, v3 as isize);
    (*E).rfpm_var_C = crate::rtl::rom_table_word!(game, 0xa28187, (v3 + 1i32) as isize);
    (*E).rfpm_var_F = crate::rtl::rom_table_word!(game, 0xa28187, (v3 + 2i32) as isize);
    (*E).rfpm_var_E = crate::rtl::rom_table_word!(game, 0xa28187, (v3 + 3i32) as isize);
    let current_instruction_high: u16 =
        *(&raw mut (*E).base.current_instruction as *mut u8).offset(1isize) as u16;
    (*E).rfpm_var_01 = current_instruction_high;
    (*E).rfpm_var_20 = (current_instruction_high) ^ (1u16);
    let extra_properties_low: u16 =
        *(&raw mut (*E).base.extra_properties as *mut u8).offset(0) as u16;
    (*E).rfpm_var_02 = extra_properties_low;
    (*E).rfpm_var_08 = (16u16).wrapping_mul(extra_properties_low);
    let extra_properties_high: u16 =
        *(&raw mut (*E).base.extra_properties as *mut u8).offset(1isize) as u16;
    (*E).rfpm_var_03 = extra_properties_high;
    (*E).rfpm_var_09 = (16u16).wrapping_mul(extra_properties_high);
    let rfpm_parameter_1_low: u16 = *(&raw mut (*E).rfpm_parameter_1 as *mut u8).offset(0) as u16;
    (*E).rfpm_var_04 = rfpm_parameter_1_low;
    (*E).rfpm_var_07 = (2u16).wrapping_mul(rfpm_parameter_1_low);
    (*E).rfpm_var_05 = *(&raw mut (*E).rfpm_parameter_1 as *mut u8).offset(1isize) as u16;
    let rfpm_parameter_2: u16 = (*E).rfpm_parameter_2;
    (*E).rfpm_var_06 = rfpm_parameter_2;
    (*E).rfpm_var_B = rfpm_parameter_2;
    let x_pos: u16 = (*E).base.x_pos;
    (*E).rfpm_var_11 = x_pos;
    (*E).rfpm_var_12 = ((*E).rfpm_var_05).wrapping_add(x_pos);
    if (*E).rfpm_var_01 == 0 {
        let v10: u16 = (*E).base.x_pos;
        (*E).rfpm_var_12 = v10;
        (*E).rfpm_var_11 = (v10).wrapping_sub((*E).rfpm_var_05);
    }
    (*E).rfpm_var_A = ((0xa2f224i32) as u16);
    (*E).rfpm_var_15 = ram_word!(game, samus_x_pos);
    (*E).base.extra_properties = 0u16;
    set_fields! {[(*E)], 0u16; rfpm_var_0A, rfpm_var_0B}
}
pub unsafe fn HorizontalShootableShutter_Main(game: &mut Game) {
    let E = Get_RisingFallingPlatform(ram_word!(game, cur_enemy_index) as i32, game);
    CallRisingFallingPlatformFunc((((*E).rfpm_var_A) as u32) | (0xa20000u32), game);
    if (*E).rfpm_var_A as i32 != 0xa2f272i32 & 0xffffi32
        && (*E).rfpm_var_A as i32 != 0xa2f2e4i32 & 0xffffi32
        && (ram_word!(game, enemy_index_colliding_dirs[3])
            & (ram_word!(game, enemy_index_colliding_dirs[2])
                & ram_word!(game, enemy_index_colliding_dirs[1])
                & ram_word!(game, enemy_index_colliding_dirs[0])))
            != 0xffff
        && ram_word!(game, enemy_index_colliding_dirs[3])
            & (ram_word!(game, enemy_index_colliding_dirs[2])
                & ram_word!(game, enemy_index_colliding_dirs[1])
                & ram_word!(game, enemy_index_colliding_dirs[0]))
            == ram_word!(game, cur_enemy_index)
        && ram_word!(game, samus_contact_damage_index) != 0u16
    {
        HorizontalShootableShutter_Powerbomb(game);
    }
    (*E).rfpm_var_15 = ram_word!(game, samus_x_pos);
    (*E).rfpm_var_16 = ram_word!(game, samus_x_subpos);
}
static off_A2F107: [Option<Func_V>; 5] = callback_table! { unsafe Func_V;
    HorizontalShootableShutter_Func_3, HorizontalShootableShutter_Func_4,
    HorizontalShootableShutter_Func_5, HorizontalShootableShutter_Func_6,
    HorizontalShootableShutter_Func_6,
};
pub unsafe fn HorizontalShootableShutter_Func_2(game: &mut Game) {
    let v0: i32 = (*Get_RisingFallingPlatform(ram_word!(game, cur_enemy_index) as i32, game))
        .rfpm_var_07 as i32
        >> 1i32;
    off_A2F107[v0 as usize].expect("non-null function pointer")(game);
}
pub unsafe fn HorizontalShootableShutter_Func_3(game: &mut Game) {
    let E = Get_RisingFallingPlatform(ram_word!(game, cur_enemy_index) as i32, game);
    crate::update_place! { previous fresh14 = (*E).rfpm_var_B => (*E).rfpm_var_B.wrapping_sub(1); }
    if fresh14 == 1u16 {
        (*E).rfpm_var_B = (*E).rfpm_var_06;
        HorizontalShootableShutter_Func_7(ram_word!(game, cur_enemy_index), game);
    }
}
pub unsafe fn HorizontalShootableShutter_Func_4(game: &mut Game) {
    let E = Get_RisingFallingPlatform(ram_word!(game, cur_enemy_index) as i32, game);
    if IsSamusWithinEnemy_X(ram_word!(game, cur_enemy_index), (*E).rfpm_var_06, game) != 0 {
        HorizontalShootableShutter_Func_7(ram_word!(game, cur_enemy_index), game);
    }
}
pub unsafe fn HorizontalShootableShutter_Func_5(game: &mut Game) {
    HorizontalShootableShutter_Func_7(ram_word!(game, cur_enemy_index), game);
}
pub fn HorizontalShootableShutter_Func_6(game: &mut Game) {}
pub unsafe fn HorizontalShootableShutter_Func_7(k: u16, game: &mut Game) {
    let E = Get_RisingFallingPlatform(k as i32, game);
    (*E).rfpm_var_A = ((0xa2f272i32) as u16);
    if (*E).rfpm_var_01 != 0 {
        (*E).rfpm_var_A = ((0xa2f2e4i32) as u16);
    }
}
pub unsafe fn HorizontalShootableShutter_Func_8(game: &mut Game) {
    let E = Get_RisingFallingPlatform(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).rfpm_var_0D = (*E).base.x_pos;
    (*E).rfpm_var_13 = 0u16;
    if EnemyFunc_AC67(ram_word!(game, cur_enemy_index), game) != 0u16
        && ((ram_word!(game, samus_x_pos)) as i16).wrapping_sub(((*E).base.x_pos) as i16) < 0i16
    {
        (*E).rfpm_var_13 = 1u16;
    }
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        ((*E).rfpm_var_F as u32) << 16i32 | (*E).rfpm_var_E as u32,
    );
    if (*E).rfpm_var_13 != 0 {
        set_ram_word!(game.extra_samus_x_subdisplacement = (*E).rfpm_var_E);
        set_ram_word!(game.extra_samus_x_displacement = (*E).rfpm_var_F);
        HorizontalShootableShutter_Func_10(ram_word!(game, cur_enemy_index), game);
    }
    if (((*E).rfpm_var_11) as i16).wrapping_sub(((*E).base.x_pos) as i16) >= 0i16 {
        if (*E).rfpm_var_08 == 4080u16 {
            (*E).rfpm_var_A = ((0xa2f3d4i32) as u16);
        } else {
            (*E).rfpm_var_B = (*E).rfpm_var_08;
            (*E).rfpm_var_A = ((0xa2f38ci32) as u16);
        }
    }
}
pub unsafe fn HorizontalShootableShutter_Func_9(game: &mut Game) {
    let E = Get_RisingFallingPlatform(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).rfpm_var_0D = (*E).base.x_pos;
    (*E).rfpm_var_13 = 0u16;
    if EnemyFunc_AC67(ram_word!(game, cur_enemy_index), game) != 0u16
        && ((ram_word!(game, samus_x_pos)) as i16).wrapping_sub(((*E).base.x_pos) as i16) >= 0i16
    {
        (*E).rfpm_var_13 = 1u16;
    }
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        ((*E).rfpm_var_D as u32) << 16i32 | (*E).rfpm_var_C as u32,
    );
    if (*E).rfpm_var_13 != 0 {
        set_ram_word!(game.extra_samus_x_subdisplacement = (*E).rfpm_var_C);
        set_ram_word!(game.extra_samus_x_displacement = (*E).rfpm_var_D);
        HorizontalShootableShutter_Func_11(ram_word!(game, cur_enemy_index), game);
    }
    if (((*E).base.x_pos) as i16).wrapping_sub(((*E).rfpm_var_12) as i16) >= 0i16 {
        if (*E).rfpm_var_09 == 4080u16 {
            (*E).rfpm_var_A = ((0xa2f3d4i32) as u16);
        } else {
            (*E).rfpm_var_B = (*E).rfpm_var_09;
            (*E).rfpm_var_A = ((0xa2f3b0i32) as u16);
        }
    }
}
pub unsafe fn HorizontalShootableShutter_Func_10(k: u16, game: &mut Game) {
    if (*Get_RisingFallingPlatform(k as i32, game)).rfpm_var_13 != 0
        && (ram_word!(game, joypad1_lastkeys) & 0x100) != 0
    {
        update_ram_word!(game.extra_samus_x_displacement, wrapping_sub(4u16));
        set_ram_word!(game.extra_samus_y_subdisplacement = 0u16);
    }
}
pub unsafe fn HorizontalShootableShutter_Func_11(k: u16, game: &mut Game) {
    if (*Get_RisingFallingPlatform(k as i32, game)).rfpm_var_13 != 0
        && (ram_word!(game, joypad1_lastkeys) & 0x200) != 0
    {
        update_ram_word!(game.extra_samus_x_displacement, wrapping_add(4u16));
        set_ram_word!(game.extra_samus_y_subdisplacement = 0u16);
    }
}
pub unsafe fn HorizontalShootableShutter_Func_12(game: &mut Game) {
    let E = Get_RisingFallingPlatform(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).rfpm_var_B = (*E).rfpm_var_B.wrapping_sub(1);
    if ((*E).rfpm_var_B & 0x8000) != 0 {
        (*E).rfpm_var_A = ((0xa2f2e4i32) as u16);
        if (*E).rfpm_var_04 == 1u16 && (*E).rfpm_var_01 != 0 {
            (*E).rfpm_var_A = ((0xa2f243i32) as u16);
        }
    }
}
pub unsafe fn HorizontalShootableShutter_Func_13(game: &mut Game) {
    let E = Get_RisingFallingPlatform(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).rfpm_var_B = (*E).rfpm_var_B.wrapping_sub(1);
    if ((*E).rfpm_var_B & 0x8000) != 0 {
        (*E).rfpm_var_A = ((0xa2f272i32) as u16);
        if (*E).rfpm_var_04 == 1u16 && (*E).rfpm_var_01 == 0 {
            (*E).rfpm_var_A = ((0xa2f243i32) as u16);
        }
    }
}
pub fn HorizontalShootableShutter_Func_14() {}
pub unsafe fn HorizontalShootableShutter_Touch(game: &mut Game) {
    let E = Get_HorizontalShootableShutter(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).hssr_var_A as i32 == 0xa2f3d4i32 & 0xffffi32 {
        if ((ram_word!(game, samus_x_pos)) as i16).wrapping_sub(((*E).base.x_pos) as i16) >= 0i16 {
            if (ram_word!(game, joypad1_lastkeys) & 0x200) != 0 {
                set_ram_word!(game.extra_samus_x_displacement = 4u16);
                set_ram_word!(game.extra_samus_y_subdisplacement = 0u16);
            }
        } else if (ram_word!(game, joypad1_lastkeys) & 0x100) != 0 {
            set_ram_word!(game.extra_samus_x_displacement = -4i32 as u16);
            set_ram_word!(game.extra_samus_y_subdisplacement = 0u16);
        }
    }
}
pub unsafe fn HorizontalShootableShutter_Shot(game: &mut Game) {
    NormalEnemyShotAiSkipDeathAnim_CurEnemy(game);
    HorizontalShootableShutter_Powerbomb(game);
}
pub unsafe fn HorizontalShootableShutter_Powerbomb(game: &mut Game) {
    let E = Get_RisingFallingPlatform(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).rfpm_var_07 as i32 - 6i32 & 0x8000i32 == 0 {
        if (*E).rfpm_var_07 != 8u16 {
            if (*E).rfpm_var_0C != 0 {
                return;
            }
            (*E).rfpm_var_0C = 1u16;
        }
        if (*E).rfpm_var_A as i32 != 0xa2f272i32 & 0xffffi32
            && (*E).rfpm_var_A as i32 != 0xa2f2e4i32 & 0xffffi32
        {
            (*E).rfpm_var_20 = ((*E).rfpm_var_20) ^ (1u16);
            (*E).rfpm_var_A = ((0xa2f272i32) as u16);
            if (*E).rfpm_var_20 != 0 {
                (*E).rfpm_var_A = ((0xa2f2e4i32) as u16);
            }
        }
    }
}
