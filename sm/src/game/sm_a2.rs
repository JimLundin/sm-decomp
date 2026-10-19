pub use crate::game::constants::bank_86::*;
pub use crate::game::constants::bank_8d::*;
pub use crate::game::constants::buttons::*;
pub use crate::game::constants::enemy_properties::*;
pub use crate::game::constants::game_states::*;
pub use crate::game::constants::loading_game_states::*;
pub use crate::game::constants::samus_poses::*;
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
use crate::rtl::abs16;
use crate::rtl::{AddToHiLo, SetHiLo};
use crate::rtl::{Mult8x8, RomPtr, Unreachable};

pub use crate::types::Buttons;
pub use crate::types::Consts_86;
pub use crate::types::Consts_8D;
pub use crate::types::Consts_A2;
pub use crate::types::GameState;
pub use crate::types::LoadingGameState;
pub use crate::types::{
    __int16_t, __int32_t, __int8_t, __uint16_t, __uint32_t, __uint8_t, int16, int16_t, int32,
    int32_t, int8, int8_t, uint16, uint16_t, uint32, uint32_t, uint8, uint8_t, uintptr_t,
    EnemyBase, EnemyData, Func_V, LongPtr, Point16U, SamusPose, VoidP, VramWriteEntry,
};
pub use crate::types::{EnemyProps, EnemySpawnData};
pub const addr_kRisingFallingPlatform_Ilist_EDE7: Consts_A2 = 60903;
pub const addr_kTimedShutter_Ilist_E9D4: Consts_A2 = 59860;
pub const addr_kTimedShutter_Ilist_E9AA: Consts_A2 = 59818;
pub const addr_kTimedShutter_Ilist_E9A4: Consts_A2 = 59812;
pub const addr_kTimedShutter_Ilist_E99E: Consts_A2 = 59806;
pub const addr_kTimedShutter_Ilist_E998: Consts_A2 = 59800;
pub const addr_kLavaSeahorse_Ilist_E5A1: Consts_A2 = 58785;
pub const addr_kLavaSeahorse_Ilist_E59B: Consts_A2 = 58779;
pub const addr_kRipper_Ilist_E48B: Consts_A2 = 58507;
pub const addr_kRipper_Ilist_E477: Consts_A2 = 58487;
pub const addr_kGripper_Sprmap_E44B: Consts_A2 = 58443;
pub const addr_kGripper_Sprmap_E43F: Consts_A2 = 58431;
pub const addr_kRipper_Ilist_E2F4: Consts_A2 = 58100;
pub const addr_kRipper_Ilist_E2E0: Consts_A2 = 58080;
pub const addr_kGripper_Ilist_E1AF: Consts_A2 = 57775;
pub const addr_kGripper_Ilist_E19B: Consts_A2 = 57755;
pub const addr_kHirisingSlowfalling_Ilist_D840: Consts_A2 = 55360;
pub const addr_kHirisingSlowfalling_Ilist_D834: Consts_A2 = 55348;
pub const addr_kHirisingSlowfalling_Ilist_D82C: Consts_A2 = 55340;
pub const addr_kMaridiaLargeSnail_Ilist_CA4B: Consts_A2 = 51787;
pub const addr_kLowerNorfairRio_Ilist_C6B0: Consts_A2 = 50864;
pub const addr_kLowerNorfairRio_Ilist_C686: Consts_A2 = 50822;
pub const addr_kLowerNorfairRio_Ilist_C674: Consts_A2 = 50804;
pub const addr_kLowerNorfairRio_Ilist_C662: Consts_A2 = 50786;
pub const addr_kLowerNorfairRio_Ilist_C65A: Consts_A2 = 50778;
pub const addr_kLowerNorfairRio_Ilist_C630: Consts_A2 = 50736;
pub const addr_kLowerNorfairRio_Ilist_C61A: Consts_A2 = 50714;
pub const addr_kNorfairRio_Ilist_C1A3: Consts_A2 = 49571;
pub const addr_kNorfairRio_Ilist_C18F: Consts_A2 = 49551;
pub const addr_kNorfairRio_Ilist_C179: Consts_A2 = 49529;
pub const addr_kNorfairRio_Ilist_C145: Consts_A2 = 49477;
pub const addr_kNorfairRio_Ilist_C12F: Consts_A2 = 49455;
pub const addr_kNorfairRio_Ilist_C107: Consts_A2 = 49415;
pub const addr_kNorfairRio_Ilist_C0F1: Consts_A2 = 49393;
pub const addr_kNorfairLavajumpingEnemy_Ilist_BE62: Consts_A2 = 48738;
pub const addr_kNorfairLavajumpingEnemy_Ilist_BE42: Consts_A2 = 48706;
pub const addr_kNorfairLavajumpingEnemy_Ilist_BE3C: Consts_A2 = 48700;
pub const addr_kRio_Ilist_BBA3: Consts_A2 = 48035;
pub const addr_kRio_Ilist_BB97: Consts_A2 = 48023;
pub const addr_kRio_Ilist_BB7F: Consts_A2 = 47999;
pub const addr_stru_A2BB53: Consts_A2 = 47955;
pub const addr_kRio_Ilist_BB4B: Consts_A2 = 47947;
pub const addr_kRinka_Ilist_BA0C: Consts_A2 = 47628;
pub const addr_kRinka_Ilist_B9E0: Consts_A2 = 47584;
pub const addr_kNorfairErraticFireball_Ilist_B51A: Consts_A2 = 46362;
pub const addr_kNorfairErraticFireball_Ilist_B2DC: Consts_A2 = 45788;
pub const addr_kFlies_Ilist_B013: Consts_A2 = 45075;
pub const addr_kGunshipTop_Ilist_A61C: Consts_A2 = 42524;
pub const addr_kGunshipTop_Ilist_A616: Consts_A2 = 42518;
pub const addr_kGunshipTop_Ilist_A60E: Consts_A2 = 42510;
pub const addr_kGunshipTop_Ilist_A5EE: Consts_A2 = 42478;
pub const addr_kGunshipTop_Ilist_A5BE: Consts_A2 = 42430;
pub const addr_kMaridiaSpikeyShell_Ilist_A3BD: Consts_A2 = 41917;
pub const addr_kMaridiaSpikeyShell_Ilist_A3AB: Consts_A2 = 41899;
pub const addr_kSpikeShootingPlant_Ilist_9F00: Consts_A2 = 40704;
pub const addr_kSpikeShootingPlant_Ilist_9EDA: Consts_A2 = 40666;
pub const addr_kSpikeShootingPlant_Ilist_9EB0: Consts_A2 = 40624;
pub const addr_kSpikeShootingPlant_Ilist_9E8A: Consts_A2 = 40586;
pub const addr_kThinHoppingBlobs_Ilist_9A01: Consts_A2 = 39425;
pub const addr_kThinHoppingBlobs_Ilist_99FB: Consts_A2 = 39419;
pub const addr_kThinHoppingBlobs_Ilist_99F5: Consts_A2 = 39413;
pub const addr_kThinHoppingBlobs_Ilist_99EF: Consts_A2 = 39407;
pub const addr_kThinHoppingBlobs_Ilist_99E9: Consts_A2 = 39401;
pub const addr_kThinHoppingBlobs_Ilist_99D5: Consts_A2 = 39381;
pub const addr_kThinHoppingBlobs_Ilist_99C1: Consts_A2 = 39361;
pub const addr_kThinHoppingBlobs_Ilist_99AD: Consts_A2 = 39341;
pub const addr_locret_A28E09: Consts_A2 = 36361;
pub const addr_kMaridiaBeybladeTurtle_Ilist_8D40: Consts_A2 = 36160;
pub const addr_kMaridiaBeybladeTurtle_Ilist_8D28: Consts_A2 = 36136;
pub const addr_kMaridiaBeybladeTurtle_Ilist_8D14: Consts_A2 = 36116;
pub const addr_kMaridiaBeybladeTurtle_Ilist_8D00: Consts_A2 = 36096;
pub const addr_kMaridiaBeybladeTurtle_Ilist_8C72: Consts_A2 = 35954;
pub const addr_kMaridiaBeybladeTurtle_Ilist_8C62: Consts_A2 = 35938;
pub const addr_kMaridiaBeybladeTurtle_Ilist_8C4A: Consts_A2 = 35914;
pub const addr_kMaridiaBeybladeTurtle_Ilist_8C44: Consts_A2 = 35908;
pub const addr_kMaridiaBeybladeTurtle_Ilist_8C30: Consts_A2 = 35888;
pub const addr_kMaridiaBeybladeTurtle_Ilist_8C1C: Consts_A2 = 35868;
pub const addr_kMaridiaBeybladeTurtle_Ilist_8C02: Consts_A2 = 35842;
pub const addr_kMaridiaBeybladeTurtle_Ilist_8BD2: Consts_A2 = 35794;
pub const addr_kMaridiaBeybladeTurtle_Ilist_8B80: Consts_A2 = 35712;
pub const addr_kMiniCrocomire_Ilist_896E: Consts_A2 = 35182;
pub const addr_kMiniCrocomire_Ilist_8958: Consts_A2 = 35160;
pub const addr_kMiniCrocomire_Ilist_8948: Consts_A2 = 35144;
pub const addr_kMiniCrocomire_Ilist_8932: Consts_A2 = 35122;
pub const addr_kBouncingGoofball_Ilist_86BF: Consts_A2 = 34495;
pub const addr_kBouncingGoofball_Ilist_86A7: Consts_A2 = 34471;
pub const addr_kSpritemap_Nothing_A2: Consts_A2 = 32845;
pub use crate::types::Point32;
enemy_layout! {
    Enemy_BouncingGoofball {
        base: EnemyBase,
        words: [
            bgl_var_A, bgl_var_B, bgl_var_C, bgl_var_D, bgl_var_E, bgl_var_F,
            bgl_parameter_1, bgl_parameter_2,
        ],
        pad1: [uint8; 26696],
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
        pad1: [uint8; 26696],
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
        pad1: [uint8; 26696],
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
        pad1: [uint8; 26696],
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
        pad1: [uint8; 26696],
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
        pad1: [uint8; 26696],
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
        pad1: [uint8; 26696],
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
        pad1: [uint8; 26696],
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
        pad1: [uint8; 26696],
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
        pad1: [uint8; 26696],
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
        pad1: [uint8; 26696],
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
        pad1: [uint8; 26696],
        words: [
            rfpm_var_00, rfpm_var_01, rfpm_var_02, rfpm_var_03, rfpm_var_04, rfpm_var_05,
            rfpm_var_06, rfpm_var_07, rfpm_var_08, rfpm_var_09, rfpm_var_0A, rfpm_var_0B,
            rfpm_var_0C, rfpm_var_0D, rfpm_var_0E, rfpm_var_0F, rfpm_var_10, rfpm_var_11,
            rfpm_var_12, rfpm_var_13, rfpm_var_14, rfpm_var_15, rfpm_var_16, rfpm_var_17,
            rfpm_var_18, rfpm_var_19, rfpm_var_1A, rfpm_var_1B, rfpm_var_1C, rfpm_var_1D,
            rfpm_var_1E, rfpm_var_1F,
        ],
        pad2: [uint8; 1984],
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
        pad1: [uint8; 26696],
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
        pad1: [uint8; 26696],
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
        pad1: [uint8; 26696],
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
        pad1: [uint8; 26696],
        words: [
            tsr_var_00, tsr_var_01, tsr_var_02, tsr_var_03, tsr_var_04, tsr_var_05,
            tsr_var_06, tsr_var_07, tsr_var_08, tsr_var_09, tsr_var_0A, tsr_var_0B,
            tsr_var_0C, tsr_var_0D, tsr_var_0E, tsr_var_0F, tsr_var_10, tsr_var_11,
            tsr_var_12, tsr_var_13, tsr_var_14, tsr_var_15, tsr_var_16, tsr_var_17,
            tsr_var_18, tsr_var_19, tsr_var_1A, tsr_var_1B, tsr_var_1C, tsr_var_1D,
            tsr_var_1E, tsr_var_1F,
        ],
        pad3: [uint8; 4032],
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

#[inline]
unsafe fn RomPtr_A2(mut addr: uint16_t, game: &mut Game) -> *const uint8 {
    return RomPtr((0xa20000u32) | ((addr) as u32), game);
}
#[inline]
unsafe fn gVramWriteEntry(mut a: ::core::ffi::c_int, game: &mut Game) -> *mut VramWriteEntry {
    return (&raw mut game.g_ram as *mut uint8).offset((0xd0 as ::core::ffi::c_int + a) as isize)
        as *mut uint8 as *mut VramWriteEntry;
}
pub const fnSamus_FrameHandlerAlfa_Func11: ::core::ffi::c_int = 0x90e695;
pub const fnSamus_FrameHandlerBeta_Func17: ::core::ffi::c_int = 0x90e725;
pub const fnMiniCrocomire_PreInstr5: ::core::ffi::c_int = 0xa28a43;
pub const fnMiniCrocomire_PreInstr6: ::core::ffi::c_int = 0xa28a5c;
pub const fnnullsub_175: ::core::ffi::c_int = 0xa28a75;
pub const fnMaridiaBeybladeTurtle_Func1: ::core::ffi::c_int = 0xa28dd8;
pub const fnnullsub_360: uint32 = 10653193;
pub const fnMaridiaBeybladeTurtle_Func2: ::core::ffi::c_int = 0xa28e0a;
pub const fnMaridiaBeybladeTurtle_Func3: ::core::ffi::c_int = 0xa28ee0;
pub const fnMaridiaBeybladeTurtle_Func5: ::core::ffi::c_int = 0xa28f3f;
pub const fnMaridiaBeybladeTurtle_Func7: ::core::ffi::c_int = 0xa28f8d;
pub const fnMaridiaBeybladeTurtle_Func8: ::core::ffi::c_int = 0xa28feb;
pub const fnMaridiaBeybladeTurtle_Func9: ::core::ffi::c_int = 0xa29083;
pub const fnMaridiaBeybladeTurtle_Func10: ::core::ffi::c_int = 0xa290cc;
pub const fnMaridiaBeybladeTurtle_Func11: ::core::ffi::c_int = 0xa290e1;
pub const fnMiniMaridiaBeybladeTurtle_Func1: ::core::ffi::c_int = 0xa29142;
pub const fnMiniMaridiaBeybladeTurtle_Func2: ::core::ffi::c_int = 0xa2916e;
pub const fnMiniMaridiaBeybladeTurtle_Func3: ::core::ffi::c_int = 0xa29198;
pub const fnMiniMaridiaBeybladeTurtle_Func4: ::core::ffi::c_int = 0xa291f8;
pub const fnMiniMaridiaBeybladeTurtle_Func6: ::core::ffi::c_int = 0xa29239;
pub const fnMiniMaridiaBeybladeTurtle_Func7: ::core::ffi::c_int = 0xa2925e;
pub const fnThinHoppingBlobs_Func6: ::core::ffi::c_int = 0xa29b65;
pub const fnThinHoppingBlobs_Func7: ::core::ffi::c_int = 0xa29b81;
pub const fnThinHoppingBlobs_Func11: uint32 = 10657035;
pub const fnThinHoppingBlobs_Func12: uint32 = 10657067;
pub const fnThinHoppingBlobs_Func13: uint32 = 10657099;
pub const fnThinHoppingBlobs_Func14: uint32 = 10657131;
pub const fnThinHoppingBlobs_Func15: ::core::ffi::c_int = 0xa29d98;
pub const fnThinHoppingBlobs_Func16: uint32 = 10657229;
pub const fnSpikeShootingPlant_2: ::core::ffi::c_int = 0xa29fba;
pub const fnSpikeShootingPlant_3: ::core::ffi::c_int = 0xa29fec;
pub const fnnullsub_182: ::core::ffi::c_int = 0xa2a01b;
pub const fnnullsub_187: ::core::ffi::c_int = 0xa2a7d7;
pub const fnGunshipTop_3: ::core::ffi::c_int = 0xa2a80c;
pub const fnGunshipTop_4: ::core::ffi::c_int = 0xa2a8d0;
pub const fnGunshipTop_5: ::core::ffi::c_int = 0xa2a942;
pub const fnGunshipTop_6: ::core::ffi::c_int = 0xa2a950;
pub const fnGunshipTop_7: ::core::ffi::c_int = 0xa2a987;
pub const fnGunshipTop_8: ::core::ffi::c_int = 0xa2a9bd;
pub const fnGunshipTop_9: ::core::ffi::c_int = 0xa2aa4f;
pub const fnGunshipTop_10: ::core::ffi::c_int = 0xa2aa5d;
pub const fnGunshipTop_11: ::core::ffi::c_int = 0xa2aa94;
pub const fnGunshipTop_12: ::core::ffi::c_int = 0xa2aaa2;
pub const fnGunshipTop_13: ::core::ffi::c_int = 0xa2ab1f;
pub const fnGunshipTop_14: ::core::ffi::c_int = 0xa2ab60;
pub const fnGunshipTop_15: ::core::ffi::c_int = 0xa2ab6e;
pub const fnGunshipTop_16: ::core::ffi::c_int = 0xa2aba5;
pub const fnGunshipTop_17: ::core::ffi::c_int = 0xa2abc7;
pub const fnGunshipTop_18: ::core::ffi::c_int = 0xa2ac1b;
pub const fnGunshipTop_19: ::core::ffi::c_int = 0xa2acd7;
pub const fnGunshipTop_20: ::core::ffi::c_int = 0xa2ad0e;
pub const fnGunshipTop_21: ::core::ffi::c_int = 0xa2ad2d;
pub const fnFlies_4: ::core::ffi::c_int = 0xa2b14e;
pub const fnFlies_5: ::core::ffi::c_int = 0xa2b17c;
pub const fnFlies_6: ::core::ffi::c_int = 0xa2b1aa;
pub const fnFlies_7: ::core::ffi::c_int = 0xa2b1d2;
pub const fnLavaquakeRocks_1: ::core::ffi::c_int = 0xa2b596;
pub const fnLavaquakeRocks_2: ::core::ffi::c_int = 0xa2b5b2;
pub const fnLavaquakeRocks_3: ::core::ffi::c_int = 0xa2b5ea;
pub const fnRinka_3: ::core::ffi::c_int = 0xa2b7df;
pub const fnRinka_4: ::core::ffi::c_int = 0xa2b844;
pub const fnRinka_5: ::core::ffi::c_int = 0xa2b852;
pub const fnRinka_B85B: ::core::ffi::c_int = 0xa2b85b;
pub const fnRio_1: ::core::ffi::c_int = 0xa2bbed;
pub const fnRio_2: ::core::ffi::c_int = 0xa2bc32;
pub const fnRio_3: ::core::ffi::c_int = 0xa2bc48;
pub const fnRio_4: ::core::ffi::c_int = 0xa2bcb7;
pub const fnRio_5: ::core::ffi::c_int = 0xa2bcff;
pub const fnNorfairLavajumpingEnemy_Func_1: ::core::ffi::c_int = 0xa2bedc;
pub const fnNorfairLavajumpingEnemy_Func_2: ::core::ffi::c_int = 0xa2bf1a;
pub const fnNorfairLavajumpingEnemy_Func_3: ::core::ffi::c_int = 0xa2bf3e;
pub const fnNorfairLavajumpingEnemy_Func_4: ::core::ffi::c_int = 0xa2bf7c;
pub const fnNorfairLavajumpingEnemy_Func_5: ::core::ffi::c_int = 0xa2bfbc;
pub const fnNorfairRio_Func_1: ::core::ffi::c_int = 0xa2c281;
pub const fnNorfairRio_Func_2: ::core::ffi::c_int = 0xa2c2e7;
pub const fnNorfairRio_Func_3: ::core::ffi::c_int = 0xa2c33f;
pub const fnNorfairRio_Func_4: ::core::ffi::c_int = 0xa2c361;
pub const fnNorfairRio_Func_5: ::core::ffi::c_int = 0xa2c3b1;
pub const fnNorfairRio_Func_6: ::core::ffi::c_int = 0xa2c406;
pub const fnLowerNorfairRio_Func_1: ::core::ffi::c_int = 0xa2c72e;
pub const fnLowerNorfairRio_Func_2: ::core::ffi::c_int = 0xa2c771;
pub const fnLowerNorfairRio_Func_3: ::core::ffi::c_int = 0xa2c7bb;
pub const fnLowerNorfairRio_Func_4: ::core::ffi::c_int = 0xa2c7d6;
pub const fnLowerNorfairRio_Func_5: ::core::ffi::c_int = 0xa2c82d;
pub const fnLowerNorfairRio_Func_6: ::core::ffi::c_int = 0xa2c888;
pub const fnMaridiaLargeSnail_Func_4: ::core::ffi::c_int = 0xa2cde6;
pub const fnMaridiaLargeSnail_Func_5: ::core::ffi::c_int = 0xa2ce2b;
pub const fnMaridiaLargeSnail_Func_6: ::core::ffi::c_int = 0xa2cf40;
pub const fnMaridiaLargeSnail_Func_7: ::core::ffi::c_int = 0xa2cf66;
pub const fnMaridiaLargeSnail_Func_8: ::core::ffi::c_int = 0xa2cfa9;
pub const fnHirisingSlowfalling_Func_4: ::core::ffi::c_int = 0xa2e035;
pub const fnHirisingSlowfalling_Func_5: ::core::ffi::c_int = 0xa2e04f;
pub const fnHirisingSlowfalling_Func_6: ::core::ffi::c_int = 0xa2e06a;
pub const fnHirisingSlowfalling_Func_7: ::core::ffi::c_int = 0xa2e0cd;
pub const fnLavaSeahorse_Func_1: ::core::ffi::c_int = 0xa2e654;
pub const fnLavaSeahorse_Func_2: ::core::ffi::c_int = 0xa2e6ad;
pub const fnLavaSeahorse_Func_3: ::core::ffi::c_int = 0xa2e6f1;
pub const fnLavaSeahorse_Func_4: ::core::ffi::c_int = 0xa2e734;
pub const fnLavaSeahorse_Func_5: ::core::ffi::c_int = 0xa2e749;
pub const fnnullsub_196: ::core::ffi::c_int = 0xa2e781;
pub const fnTimedShutter_Func_1: uint32 = 10676925;
pub const fnTimedShutter_Func_2: uint32 = 10676945;
pub const fnTimedShutter_Func_3: uint32 = 10676967;
pub const fnTimedShutter_Func_4: uint32 = 10676989;
pub const fnTimedShutter_Func_5: ::core::ffi::c_int = 0xa2eb11;
pub const fnTimedShutter_Func_10: ::core::ffi::c_int = 0xa2ec13;
pub const fnRisingFallingPlatform_Func_2: ::core::ffi::c_int = 0xa2ef09;
pub const fnRisingFallingPlatform_Func_4: ::core::ffi::c_int = 0xa2ef28;
pub const fnRisingFallingPlatform_Func_9: ::core::ffi::c_int = 0xa2ef68;
pub const fnRisingFallingPlatform_Func_10: ::core::ffi::c_int = 0xa2efd4;
pub const fnRisingFallingPlatform_Func_11: ::core::ffi::c_int = 0xa2f040;
pub const fnRisingFallingPlatform_Func_12: ::core::ffi::c_int = 0xa2f072;
pub const fnRisingFallingPlatform_Func_13: ::core::ffi::c_int = 0xa2f099;
pub const fnHorizontalShootableShutter_Func_2: ::core::ffi::c_int = 0xa2f224;
pub const fnHorizontalShootableShutter_Func_4: ::core::ffi::c_int = 0xa2f243;
pub const fnHorizontalShootableShutter_Func_8: ::core::ffi::c_int = 0xa2f272;
pub const fnHorizontalShootableShutter_Func_9: ::core::ffi::c_int = 0xa2f2e4;
pub const fnHorizontalShootableShutter_Func_12: ::core::ffi::c_int = 0xa2f38c;
pub const fnHorizontalShootableShutter_Func_13: ::core::ffi::c_int = 0xa2f3b0;
pub const fnHorizontalShootableShutter_Func_14: ::core::ffi::c_int = 0xa2f3d4;
#[inline]
unsafe fn gEnemyData(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut EnemyData {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut EnemyData;
}
#[inline]
unsafe fn gEnemySpawnData(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut EnemySpawnData {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0x7000)
        .offset(o as isize) as *mut EnemySpawnData;
}
#[inline]
unsafe fn gRam8000_Default(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut Ram8000_Default {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0x8000)
        .offset(o as isize) as *mut Ram8000_Default;
}
#[inline]
unsafe fn Get_Flies(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_Flies {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_Flies;
}
#[inline]
unsafe fn Get_Gripper(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_Gripper {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_Gripper;
}
#[inline]
unsafe fn Get_GunshipBottom(
    mut o: ::core::ffi::c_int,
    game: &mut Game,
) -> *mut Enemy_GunshipBottom {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_GunshipBottom;
}
#[inline]
unsafe fn Get_GunshipTop(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_GunshipTop {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_GunshipTop;
}
#[inline]
unsafe fn Get_HirisingSlowfalling(
    mut o: ::core::ffi::c_int,
    game: &mut Game,
) -> *mut Enemy_HirisingSlowfalling {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_HirisingSlowfalling;
}
#[inline]
unsafe fn Get_HorizontalShootableShutter(
    mut o: ::core::ffi::c_int,
    game: &mut Game,
) -> *mut Enemy_HorizontalShootableShutter {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_HorizontalShootableShutter;
}
#[inline]
unsafe fn Get_JetPowerRipper(
    mut o: ::core::ffi::c_int,
    game: &mut Game,
) -> *mut Enemy_JetPowerRipper {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_JetPowerRipper;
}
#[inline]
unsafe fn Get_LavaSeahorse(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_LavaSeahorse {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_LavaSeahorse;
}
#[inline]
unsafe fn Get_LavaquakeRocks(
    mut o: ::core::ffi::c_int,
    game: &mut Game,
) -> *mut Enemy_LavaquakeRocks {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_LavaquakeRocks;
}
#[inline]
unsafe fn Get_LowerNorfairRio(
    mut o: ::core::ffi::c_int,
    game: &mut Game,
) -> *mut Enemy_LowerNorfairRio {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_LowerNorfairRio;
}
#[inline]
unsafe fn Get_MaridiaBeybladeTurtle(
    mut o: ::core::ffi::c_int,
    game: &mut Game,
) -> *mut Enemy_MaridiaBeybladeTurtle {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_MaridiaBeybladeTurtle;
}
#[inline]
unsafe fn Get_MaridiaLargeSnail(
    mut o: ::core::ffi::c_int,
    game: &mut Game,
) -> *mut Enemy_MaridiaLargeSnail {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_MaridiaLargeSnail;
}
#[inline]
unsafe fn Get_MaridiaSpikeyShell(
    mut o: ::core::ffi::c_int,
    game: &mut Game,
) -> *mut Enemy_MaridiaSpikeyShell {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_MaridiaSpikeyShell;
}
#[inline]
unsafe fn Get_MaridiaTurtle(
    mut o: ::core::ffi::c_int,
    game: &mut Game,
) -> *mut Enemy_MaridiaTurtle {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_MaridiaTurtle;
}
#[inline]
unsafe fn Get_MiniMaridiaBeybladeTurtle(
    mut o: ::core::ffi::c_int,
    game: &mut Game,
) -> *mut Enemy_MiniMaridiaBeybladeTurtle {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_MiniMaridiaBeybladeTurtle;
}
#[inline]
unsafe fn Get_MiniMaridiaTurtle(
    mut o: ::core::ffi::c_int,
    game: &mut Game,
) -> *mut Enemy_MiniMaridiaTurtle {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_MiniMaridiaTurtle;
}
#[inline]
unsafe fn Get_NorfairErraticFireball(
    mut o: ::core::ffi::c_int,
    game: &mut Game,
) -> *mut Enemy_NorfairErraticFireball {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_NorfairErraticFireball;
}
#[inline]
unsafe fn Get_NorfairLavajumpingEnemy(
    mut o: ::core::ffi::c_int,
    game: &mut Game,
) -> *mut Enemy_NorfairLavajumpingEnemy {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_NorfairLavajumpingEnemy;
}
#[inline]
unsafe fn Get_NorfairRio(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_NorfairRio {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_NorfairRio;
}
#[inline]
unsafe fn Get_Rinka(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_Rinka {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_Rinka;
}
#[inline]
unsafe fn Get_Rio(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_Rio {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_Rio;
}
#[inline]
unsafe fn Get_Ripper(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_Ripper {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_Ripper;
}
#[inline]
unsafe fn Get_RisingFallingPlatform(
    mut o: ::core::ffi::c_int,
    game: &mut Game,
) -> *mut Enemy_RisingFallingPlatform {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_RisingFallingPlatform;
}
#[inline]
unsafe fn Get_SpikeShootingPlant(
    mut o: ::core::ffi::c_int,
    game: &mut Game,
) -> *mut Enemy_SpikeShootingPlant {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_SpikeShootingPlant;
}
#[inline]
unsafe fn Get_ThinHoppingBlobs(
    mut o: ::core::ffi::c_int,
    game: &mut Game,
) -> *mut Enemy_ThinHoppingBlobs {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_ThinHoppingBlobs;
}
#[inline]
unsafe fn Get_TimedShutter(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_TimedShutter {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_TimedShutter;
}
static g_word_A28D50: uint16 = 0x30 as uint16;
static g_word_A28D52: uint16 = 1 as uint16;
static g_word_A28D5E: uint16 = 3 as uint16;
static g_word_A28D60: uint16 = 0x1e8 as uint16;
static g_word_A28D62: uint16 = 7 as uint16;
static g_word_A28D64: uint16 = 0x1e as uint16;
static g_word_A28D66: uint16 = 4 as uint16;
static g_word_A28D68: uint16 = 0xfffd as uint16;
static g_word_A28D6A: uint16 = 3 as uint16;
static g_word_A28E80: [int16; 48] = [
    -(16),
    -(16),
    -(16),
    -(16),
    -(15),
    -(15),
    -(15),
    -(15),
    -(15),
    -(14),
    -(13),
    -(13),
    -(12),
    -(11),
    -(10),
    -(9),
    -(8),
    -(7),
    -(6),
    -(5),
    -(4),
    -(4),
    0,
    0,
    -(16),
    -(16),
    -(16),
    -(15),
    -(15),
    -(15),
    -(14),
    -(13),
    -(12),
    -(11),
    -(10),
    -(9),
    -(8),
    -(7),
    -(6),
    -(5),
    -(4),
    -(3),
    -(3),
    -(2),
    0,
    0,
    0,
    0,
];
pub unsafe fn Enemy_GrappleReact_CancelBeam_A2(game: &mut Game) {
    Enemy_SwitchToFrozenAi(game);
}
pub unsafe fn Enemy_GrappleReact_KillEnemy_A2(game: &mut Game) {
    EnemyGrappleDeath(game);
}
pub unsafe fn Enemy_GrappleReact_SamusLatchesOn_A2(game: &mut Game) {
    SamusLatchesOnWithGrapple(game);
}
pub unsafe fn Enemy_NormalFrozenAI_A2(game: &mut Game) {
    NormalEnemyFrozenAI(game);
}
pub unsafe fn Enemy_NormalPowerBombAI_A2(game: &mut Game) {
    NormalEnemyPowerBombAi(game);
}
pub unsafe fn Enemy_NormalTouchAI_A2(game: &mut Game) {
    NormalEnemyTouchAi(game);
}
pub unsafe fn Enemy_NormalShotAI_A2(game: &mut Game) {
    NormalEnemyShotAi(game);
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
pub fn BouncingGoofball_Func1(k: uint16, game: &mut Game) {
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
pub fn BouncingGoofball_SamusCloseX(game: &mut Game) -> uint16 {
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
pub fn BouncingGoofball_Instr_88C5(_k: uint16, jp: *const uint16) -> *const uint16 {
    jp
}
pub unsafe fn BouncingGoofball_Instr_88C6(
    _k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let index = game.cur_enemy_index();
    let offset =
        0xf78 + usize::from(index) + core::mem::offset_of!(Enemy_BouncingGoofball, bgl_var_02);
    game.g_ram[offset..offset + 2].copy_from_slice(&0u16.to_le_bytes());
    QueueSfx2_Max6(0xe, game);
    jp
}
pub unsafe fn MiniCrocomire_Instr_897E(
    _k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let parameter = *jp;
    let index = game.cur_enemy_index();
    SpawnEprojWithGfx(parameter, index, addr_Eproj_DBF2 as u16, game);
    jp.add(1)
}
pub fn MiniCrocomire_Instr_8990(_k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let index = game.cur_enemy_index();
    set_mini_crocomire_preinstruction(index, 0x8a43, 0, game);
    jp
}
pub fn MiniCrocomire_Instr_899D(_k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
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
pub unsafe fn MiniCrocomire_PreInstr5(_k: uint16, game: &mut Game) {
    let index = game.cur_enemy_index();
    let high = MiniCrocomireState::new(game, index).var_d();
    let low = MiniCrocomireState::new(game, index).var_c();
    MiniCrocomire_Func7((u32::from(high) << 16 | u32::from(low)) as i32, game);
    if MiniCrocomire_Func9(game) != 0 {
        MiniCrocomire_Func2(game);
    }
}
pub unsafe fn MiniCrocomire_PreInstr6(_k: uint16, game: &mut Game) {
    let index = game.cur_enemy_index();
    let high = MiniCrocomireState::new(game, index).var_b();
    let low = MiniCrocomireState::new(game, index).var_a();
    MiniCrocomire_Func7((u32::from(high) << 16 | u32::from(low)) as i32, game);
    if MiniCrocomire_Func9(game) != 0 {
        MiniCrocomire_Func4(game);
    }
}
pub unsafe fn MiniCrocomire_Func7(amt: int32, game: &mut Game) {
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
pub unsafe fn MiniCrocomire_Func9(game: &mut Game) -> uint8 {
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
pub unsafe fn CallMaridiaBeybladeTurtleFunc(ea: uint32, k: uint16, game: &mut Game) {
    match ea {
        0xa28dd8 => MaridiaBeybladeTurtle_Func1(k, game),
        0xa28e0a => MaridiaBeybladeTurtle_Func2(k, game),
        0xa28ee0 => MaridiaBeybladeTurtle_Func3(k, game),
        0xa28f3f => MaridiaBeybladeTurtle_Func5(k, game),
        0xa28f8d => MaridiaBeybladeTurtle_Func7(k, game),
        0xa28feb => MaridiaBeybladeTurtle_Func8(k, game),
        0xa29083 => MaridiaBeybladeTurtle_Func9(k, game),
        0xa290cc => MaridiaBeybladeTurtle_Func10(k, game),
        0xa290e1 => MaridiaBeybladeTurtle_Func11(k, game),
        0xa28e09 => {}
        0xa29142 => MiniMaridiaBeybladeTurtle_Func1(k, game),
        0xa2916e => MiniMaridiaBeybladeTurtle_Func2(k, game),
        0xa29198 => MiniMaridiaBeybladeTurtle_Func3(k, game),
        0xa291f8 => MiniMaridiaBeybladeTurtle_Func4(k, game),
        0xa29239 => MiniMaridiaBeybladeTurtle_Func6(k, game),
        0xa2925e => MiniMaridiaBeybladeTurtle_Func7(game),
        _ => {
            Unreachable(game);
        }
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
pub fn MaridiaBeybladeTurtle_Func1(k: uint16, game: &mut Game) {
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
pub fn MaridiaBeybladeTurtle_Func2(k: uint16, game: &mut Game) {
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
pub unsafe fn MaridiaBeybladeTurtle_Func3(mut k: uint16, game: &mut Game) {
    MaridiaBeybladeTurtle_Func4(game);
    if (*(&raw mut game.g_ram as *mut uint8).offset(0x5b5) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int)
        == 0i32
    {
        if CheckIfEnemyTouchesSamus(k, game) != 0 {
            let ref mut fresh23 =
                *((&raw mut game.g_ram as *mut uint8).offset(0xb58) as *mut uint16);
            *fresh23 = (*fresh23).wrapping_sub(1);
        }
        let mut E: *mut Enemy_MaridiaBeybladeTurtle =
            Get_MaridiaBeybladeTurtle(k as ::core::ffi::c_int, game);
        (*E).base.y_pos = (*E).base.y_pos.wrapping_sub(1);
        (*E).base.y_height = 16 as uint16;
        if ((*E).base.y_pos as ::core::ffi::c_int & 1 as ::core::ffi::c_int) != 0i32 {
            (*E).base.x_pos = (*E).base.x_pos.wrapping_sub(1);
        } else {
            (*E).base.x_pos = (*E).base.x_pos.wrapping_add(1);
        }
        if Enemy_MoveRight_IgnoreSlopes(k, (1 as int16 as int32) << 16 as ::core::ffi::c_int, game)
            == 0
        {
            (*E).base.current_instruction = addr_kMaridiaBeybladeTurtle_Ilist_8C4A as uint16;
            (*E).base.instruction_timer = 1 as uint16;
            (*E).mbte_var_A = addr_locret_A28E09 as uint16;
        }
    }
}
pub unsafe fn MaridiaBeybladeTurtle_Func5(mut k: uint16, game: &mut Game) {
    let mut v1: uint16 = addr_kMaridiaBeybladeTurtle_Ilist_8C1C as uint16;
    let mut E: *mut Enemy_MaridiaBeybladeTurtle =
        Get_MaridiaBeybladeTurtle(k as ::core::ffi::c_int, game);
    if (((*E).base.x_pos) as i16)
        .wrapping_sub((*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)) as i16)
        < 0i16
    {
        v1 = addr_kMaridiaBeybladeTurtle_Ilist_8D00 as uint16;
    }
    (*E).base.current_instruction = v1;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).mbte_var_A = addr_locret_A28E09 as uint16;
}
pub unsafe fn MaridiaBeybladeTurtle_Func6(mut k: uint16, game: &mut Game) {
    if CheckIfEnemyTouchesSamus(k, game) != 0 {
        let mut E: *mut Enemy_MaridiaTurtle = Get_MaridiaTurtle(k as ::core::ffi::c_int, game);
        (*E).mte_var_A = ((0xa29083i32) as u16) & (0xffffu16);
        AddToHiLo(
            (&raw mut game.g_ram as *mut uint8).offset(0xb58) as *mut uint16,
            (&raw mut game.g_ram as *mut uint8).offset(0xb56) as *mut uint16,
            -(((*E).mte_var_E as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
                | (*E).mte_var_03 as ::core::ffi::c_int) as uint32,
        );
        if *((&raw mut game.g_ram as *mut uint8).offset(0xb58) as *mut uint16) as ::core::ffi::c_int
            + 16 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xb58) as *mut uint16) =
                -(16 as ::core::ffi::c_int) as uint16;
        }
    }
}
pub unsafe fn MaridiaBeybladeTurtle_Func7(mut k: uint16, game: &mut Game) {
    MaridiaBeybladeTurtle_Func4(game);
    if Enemy_MoveDown(
        k,
        (-(1 as ::core::ffi::c_int) as int16 as int32) << 16 as ::core::ffi::c_int,
        game,
    ) == 0
    {
        if CheckIfEnemyTouchesSamus(k, game) != 0 {
            let ref mut fresh24 =
                *((&raw mut game.g_ram as *mut uint8).offset(0xb5c) as *mut uint16);
            *fresh24 = (*fresh24).wrapping_sub(1);
        }
        let mut E: *mut Enemy_MaridiaTurtle = Get_MaridiaTurtle(k as ::core::ffi::c_int, game);
        let mut v2: uint16 = ((*E).mte_var_00).wrapping_sub(1u16);
        (*E).mte_var_00 = v2;
        if v2 == 0 {
            let mut v3: uint16 = 0 as uint16;
            if (((*E).base.x_pos) as i16).wrapping_sub(
                (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)) as i16,
            ) < 0i16
            {
                v3 = 4 as uint16;
            }
            (*E).mte_var_01 = *((game.g_rom.offset(crate::rtl::lorom_offset(0xa28d56))
                as *const uint8 as *mut uint16 as *mut uint8)
                .offset(v3 as ::core::ffi::c_int as isize)
                as *mut uint16);
            (*E).mte_var_02 = *((game.g_rom.offset(crate::rtl::lorom_offset(0xa28d58))
                as *const uint8 as *mut uint16 as *mut uint8)
                .offset(v3 as ::core::ffi::c_int as isize)
                as *mut uint16);
            (*E).mte_var_E = 0 as uint16;
            (*E).mte_var_03 = 0 as uint16;
            (*E).mte_var_A = ((0xa28febi32) as u16) & (0xffffu16);
        }
    }
}
pub unsafe fn MaridiaBeybladeTurtle_Func8(mut k: uint16, game: &mut Game) {
    MaridiaBeybladeTurtle_Func4(game);
    let mut E: *mut Enemy_MaridiaTurtle = Get_MaridiaTurtle(k as ::core::ffi::c_int, game);
    if Enemy_MoveRight_IgnoreSlopes(
        k,
        (((*E).mte_var_E as uint32) << 16 as ::core::ffi::c_int | (*E).mte_var_03 as uint32)
            as int32,
        game,
    ) != 0
    {
        SetHiLo(
            &raw mut (*E).mte_var_E,
            &raw mut (*E).mte_var_03,
            -(((*E).mte_var_E as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
                | (*E).mte_var_03 as ::core::ffi::c_int) as uint32,
        );
        SetHiLo(
            &raw mut (*E).mte_var_02,
            &raw mut (*E).mte_var_01,
            -(((*E).mte_var_02 as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
                | (*E).mte_var_01 as ::core::ffi::c_int) as uint32,
        );
        *((&raw mut game.g_ram as *mut uint8).offset(0x183e) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1840) as *mut uint16) = 16 as uint16;
        QueueSfx2_Max6(0x1b as uint16, game);
    } else {
        MaridiaBeybladeTurtle_Func6(k, game);
        AddToHiLo(
            &raw mut (*E).mte_var_E,
            &raw mut (*E).mte_var_03,
            ((*E).mte_var_02 as uint32) << 16 as ::core::ffi::c_int | (*E).mte_var_01 as uint32,
        );
        if ((abs16((*E).mte_var_E)) as i16).wrapping_sub((g_word_A28D5E) as i16) >= 0i16 {
            (*E).mte_var_E =
                (if ((*E).mte_var_E as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
                    -(g_word_A28D5E as ::core::ffi::c_int)
                } else {
                    g_word_A28D5E as ::core::ffi::c_int
                }) as uint16;
        }
    };
}
pub unsafe fn MaridiaBeybladeTurtle_Func9(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_MaridiaTurtle = Get_MaridiaTurtle(k as ::core::ffi::c_int, game);
    let mut v2: uint16 = 0;
    MaridiaBeybladeTurtle_Func4(game);
    if (((*E).base.y_pos) as i16).wrapping_sub((g_word_A28D60) as i16) < 0i16 {
        (*E).mte_var_00 = g_word_A28D64;
        v2 = ((0xa290cci32) as u16) & (0xffffu16);
    } else {
        if CheckIfEnemyTouchesSamus(k, game) != 0 {
            (*E).base.y_pos = ((*E).base.y_pos).wrapping_sub(g_word_A28D62);
            let ref mut fresh25 =
                *((&raw mut game.g_ram as *mut uint8).offset(0xb5c) as *mut uint16);
            *fresh25 = (*fresh25).wrapping_sub(g_word_A28D62);
            return;
        }
        v2 = ((0xa290e1i32) as u16) & (0xffffu16);
    }
    (*E).mte_var_A = v2;
    (*E).mte_var_07 = 0 as uint16;
    (*E).mte_var_04 = 0 as uint16;
}
pub unsafe fn MaridiaBeybladeTurtle_Func10(mut k: uint16, game: &mut Game) {
    MaridiaBeybladeTurtle_Func4(game);
    let mut E: *mut Enemy_MaridiaTurtle = Get_MaridiaTurtle(k as ::core::ffi::c_int, game);
    let mut v2: uint16 = ((*E).mte_var_00).wrapping_sub(1u16);
    (*E).mte_var_00 = v2;
    if v2 == 0 {
        (*E).mte_var_A = ((0xa290e1i32) as u16) & (0xffffu16);
    }
}
pub unsafe fn MaridiaBeybladeTurtle_Func11(mut k: uint16, game: &mut Game) {
    MaridiaBeybladeTurtle_Func4(game);
    let mut E: *mut Enemy_MaridiaTurtle = Get_MaridiaTurtle(k as ::core::ffi::c_int, game);
    if (((*Get_MaridiaTurtle(0 as ::core::ffi::c_int, game)).mte_var_04) as i16)
        .wrapping_sub((g_word_A28D66) as i16)
        < 0i16
    {
        AddToHiLo(
            &raw mut (*E).mte_var_04,
            &raw mut (*E).mte_var_07,
            0x2000 as uint32,
        );
    }
    if Enemy_MoveDown(
        k,
        ((*E).mte_var_04 as int16 as int32) << 16 as ::core::ffi::c_int,
        game,
    ) != 0
    {
        (*E).base.current_instruction =
            (if ((*E).mte_var_E as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) == 0i32 {
                addr_kMaridiaBeybladeTurtle_Ilist_8D28 as ::core::ffi::c_int
            } else {
                addr_kMaridiaBeybladeTurtle_Ilist_8C4A as ::core::ffi::c_int
            }) as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).mte_var_A = addr_locret_A28E09 as uint16;
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
pub unsafe fn MiniMaridiaBeybladeTurtle_Func1(mut k: uint16, game: &mut Game) {
    if CheckIfEnemyTouchesSamus(k, game) != 0 {
        let mut E: *mut Enemy_MiniMaridiaTurtle =
            Get_MiniMaridiaTurtle(k as ::core::ffi::c_int, game);
        (*E).mmte_var_A = ((0xa2916ei32) as u16) & (0xffffu16);
        (*E).mmte_var_05 = 4 as uint16;
        let mut v2: uint16 = addr_kMaridiaBeybladeTurtle_Ilist_8C30 as uint16;
        if ((*E).mmte_var_E as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) == 0i32 {
            v2 = addr_kMaridiaBeybladeTurtle_Ilist_8D14 as uint16;
        }
        (*E).base.current_instruction = v2;
        (*E).base.instruction_timer = 1 as uint16;
    }
}
pub unsafe fn MiniMaridiaBeybladeTurtle_Func2(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_MiniMaridiaTurtle = Get_MiniMaridiaTurtle(k as ::core::ffi::c_int, game);
    if CheckIfEnemyTouchesSamus(k, game) != 0 {
        (*E).mmte_var_05 = 4 as uint16;
    } else {
        let mut v2: uint16 = ((*E).mmte_var_05).wrapping_sub(1u16);
        (*E).mmte_var_05 = v2;
        if v2 == 0 {
            (*E).mmte_var_A = ((0xa29198i32) as u16) & (0xffffu16);
            (*E).mmte_var_00 = 60 as uint16;
        }
    };
}
pub unsafe fn MiniMaridiaBeybladeTurtle_Func3(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_MiniMaridiaTurtle = Get_MiniMaridiaTurtle(k as ::core::ffi::c_int, game);
    if CheckIfEnemyTouchesSamus(k, game) != 0 {
        (*E).mmte_var_A = ((0xa291f8i32) as u16) & (0xffffu16);
        (*E).base.current_instruction = addr_kMaridiaBeybladeTurtle_Ilist_8BD2 as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).mmte_var_04 = 1 as uint16;
        let mut v2: uint16 = g_word_A28D68;
        if (*(&raw mut game.g_ram as *mut uint8).offset(0xa1e) as ::core::ffi::c_int
            & 0xf as ::core::ffi::c_int)
            == 8i32
        {
            v2 = g_word_A28D6A;
        }
        (*E).mmte_var_E = v2;
    } else {
        let mut v4: uint16 = ((*E).mmte_var_00).wrapping_sub(1u16);
        (*E).mmte_var_00 = v4;
        if v4 == 0 {
            let mut v5: uint16 = addr_kMaridiaBeybladeTurtle_Ilist_8B80 as uint16;
            if ((*E).mmte_var_E as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) == 0i32 {
                v5 = addr_kMaridiaBeybladeTurtle_Ilist_8C72 as uint16;
            }
            (*E).base.current_instruction = v5;
            (*E).base.instruction_timer = 1 as uint16;
            (*E).mmte_var_A = ((0xa29142i32) as u16) & (0xffffu16);
        }
    };
}
pub unsafe fn MiniMaridiaBeybladeTurtle_Func4(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_MiniMaridiaTurtle = Get_MiniMaridiaTurtle(k as ::core::ffi::c_int, game);
    if Enemy_MoveRight_IgnoreSlopes(
        k,
        (((*E).mmte_var_E as uint32) << 16 as ::core::ffi::c_int | 0 as uint16 as uint32) as int32,
        game,
    ) != 0
    {
        (*E).mmte_var_E = -((*E).mmte_var_E as ::core::ffi::c_int) as uint16;
    } else {
        Enemy_MoveDown(
            k,
            ((*E).mmte_var_04 as int16 as int32) << 16 as ::core::ffi::c_int,
            game,
        );
    };
}
pub unsafe fn MiniMaridiaBeybladeTurtle_Func5(mut k: uint16, game: &mut Game) {
    let mut v1: uint16 = addr_kMaridiaBeybladeTurtle_Ilist_8B80 as uint16;
    let mut E: *mut Enemy_MiniMaridiaBeybladeTurtle =
        Get_MiniMaridiaBeybladeTurtle(k as ::core::ffi::c_int, game);
    if ((*E).mmbte_var_E as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) == 0i32 {
        v1 = addr_kMaridiaBeybladeTurtle_Ilist_8C72 as uint16;
    }
    (*E).base.current_instruction = v1;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).mmbte_var_A = ((0xa29142i32) as u16) & (0xffffu16);
}
pub unsafe fn MiniMaridiaBeybladeTurtle_Func6(mut k: uint16, game: &mut Game) {
    if CheckIfEnemyTouchesSamus(k, game) != 0 {
        let mut v1: uint16 = addr_kMaridiaBeybladeTurtle_Ilist_8B80 as uint16;
        let mut E: *mut Enemy_MiniMaridiaBeybladeTurtle =
            Get_MiniMaridiaBeybladeTurtle(k as ::core::ffi::c_int, game);
        if ((*E).mmbte_var_E as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) == 0i32 {
            v1 = addr_kMaridiaBeybladeTurtle_Ilist_8C72 as uint16;
        }
        (*E).base.current_instruction = v1;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).mmbte_var_A = ((0xa29142i32) as u16) & (0xffffu16);
    } else {
        MiniMaridiaBeybladeTurtle_Func4(k, game);
    };
}
pub unsafe fn MiniMaridiaBeybladeTurtle_Func7(game: &mut Game) {
    let mut E: *mut Enemy_MiniMaridiaTurtle = Get_MiniMaridiaTurtle(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*Get_MiniMaridiaTurtle((*E).mmte_var_B as ::core::ffi::c_int, game)).mmte_var_06 =
        (*E).base.y_height;
    if CheckIfEnemyTouchesSamus(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    ) == 0
    {
        (*Get_MiniMaridiaTurtle(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        ))
        .mmte_var_A = ((0xa29142i32) as u16) & (0xffffu16);
    }
}
pub unsafe fn MaridiaBeybladeTurtle_Touch(game: &mut Game) {
    let mut E: *mut Enemy_MiniMaridiaTurtle = Get_MiniMaridiaTurtle(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if ((*E).base.properties as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) == 0i32 {
        Enemy_NormalTouchAI_A2(game);
        (*E).mmte_var_A = ((0xa290e1i32) as u16) & (0xffffu16);
        (*E).mmte_var_04 = 2 as uint16;
    }
}
pub unsafe fn MiniMaridiaBeybladeTurtle_Touch(game: &mut Game) {
    let mut v2: int16 = 0;
    let mut E: *mut Enemy_MiniMaridiaBeybladeTurtle = Get_MiniMaridiaBeybladeTurtle(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).mmbte_var_A != 0x925eu16 {
        if ((*E).mmbte_var_E as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
            (*E).base.current_instruction = addr_kMaridiaBeybladeTurtle_Ilist_8C72 as uint16;
            v2 = 1 as int16;
        } else {
            (*E).base.current_instruction = addr_kMaridiaBeybladeTurtle_Ilist_8B80 as uint16;
            v2 = -(1 as ::core::ffi::c_int) as int16;
        }
        (*E).mmbte_var_E = v2 as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        let mut v3: uint16 = 0;
        if (((*E).base.x_pos) as i16).wrapping_sub(
            (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)) as i16,
        ) >= 0i16
        {
            v3 = ((*E).base.x_width as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0xafe) as *mut uint16)
                    as ::core::ffi::c_int
                + *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
                    as ::core::ffi::c_int) as uint16;
        } else {
            v3 = (((((*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16))
                as u32)
                .wrapping_sub(
                    (*((&raw mut game.g_ram as *mut uint8).offset(0xafe) as *mut uint16)) as u32,
                ))
                << 16 as ::core::ffi::c_int
                | *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) as uint32)
                .wrapping_sub(
                    ((*E).base.x_width as uint32) << 16 as ::core::ffi::c_int
                        | *((&raw mut game.g_ram as *mut uint8).offset(0xafe) as *mut uint16)
                            as uint32,
                )
                >> 16 as ::core::ffi::c_int) as uint16;
        }
        (*E).base.x_pos = v3;
        (*E).mmbte_var_A = ((0xa29142i32) as u16) & (0xffffu16);
        Enemy_MoveRight_IgnoreSlopes(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            (((*E).mmbte_var_E as uint32) << 16 as ::core::ffi::c_int | 0 as uint16 as uint32)
                as int32,
            game,
        );
        MaridiaBeybladeTurtle_Func21(game);
    }
}
pub unsafe fn MaridiaBeybladeTurtle_Func21(game: &mut Game) {
    let mut mbte_var_B: uint16 = (*Get_MaridiaBeybladeTurtle(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .mbte_var_B;
    let mut E: *mut Enemy_MaridiaBeybladeTurtle =
        Get_MaridiaBeybladeTurtle(mbte_var_B as ::core::ffi::c_int, game);
    if (*E).mbte_var_F != 0 {
        (*E).mbte_var_F = (*E).mbte_var_F.wrapping_sub(1);
    }
}
pub unsafe fn MiniMaridiaBeybladeTurtle_Shot(game: &mut Game) {
    NormalEnemyShotAi(game);
    MaridiaBeybladeTurtle_Func21(game);
}
pub unsafe fn MaridiaBeybladeTurtle_Func4(game: &mut Game) {
    let mut E: *mut Enemy_MaridiaBeybladeTurtle =
        Get_MaridiaBeybladeTurtle(0 as ::core::ffi::c_int, game);
    let mut r18: uint16 = ((*E).base.x_pos as ::core::ffi::c_int
        - (*E).base.x_width as ::core::ffi::c_int
        - 8 as ::core::ffi::c_int) as uint16;
    let mut r20: uint16 = ((*E).base.x_width as ::core::ffi::c_int
        + (*E).base.x_pos as ::core::ffi::c_int
        + 8 as ::core::ffi::c_int) as uint16;
    let mut r22: uint16 = ((*E).base.y_pos as ::core::ffi::c_int
        - (*E).base.y_height as ::core::ffi::c_int
        + 4 as ::core::ffi::c_int) as uint16;
    let mut r24: uint16 = ((*E).base.y_height as ::core::ffi::c_int
        + (*E).base.y_pos as ::core::ffi::c_int
        - 4 as ::core::ffi::c_int) as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0xafe) as *mut uint16) as ::core::ffi::c_int
        - 1 as ::core::ffi::c_int
        - r20 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
        && *((&raw mut game.g_ram as *mut uint8).offset(0xafe) as *mut uint16) as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
                as ::core::ffi::c_int
            - r18 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        && *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16) as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0xb00) as *mut uint16)
                as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            - r24 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        && *((&raw mut game.g_ram as *mut uint8).offset(0xb00) as *mut uint16) as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
                as ::core::ffi::c_int
            - r22 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        && *((&raw mut game.g_ram as *mut uint8).offset(0x18a8) as *mut uint16) == 0
    {
        NormalEnemyTouchAi(game);
    }
}
pub unsafe fn MaridiaBeybladeTurtle_Instr_9381(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut R48: uint16 = 0 as uint16;
    if CheckIfEnemyTouchesSamus(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    ) != 0
    {
        let ref mut fresh0 = *((&raw mut game.g_ram as *mut uint8).offset(0xb58) as *mut uint16);
        *fresh0 = (*fresh0).wrapping_add(
            (*Get_MaridiaBeybladeTurtle(
                *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                    as ::core::ffi::c_int,
                game,
            ))
            .mbte_var_E,
        );
        R48 = 1 as uint16;
    }
    let mut E: *mut Enemy_MaridiaBeybladeTurtle = Get_MaridiaBeybladeTurtle(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut R50: uint16 = (*E).base.y_pos;
    (*E).base.y_pos = (*E).mbte_var_D;
    Enemy_MoveRight_IgnoreSlopes(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        ((*E).mbte_var_E as int16 as int32) << 16 as ::core::ffi::c_int,
        game,
    );
    if (*Get_MaridiaBeybladeTurtle((*E).mbte_var_B as ::core::ffi::c_int, game)).mbte_var_A
        == 0x8e0au16
    {
        let mut x_pos: uint16 =
            (*Get_MaridiaBeybladeTurtle((*E).mbte_var_B as ::core::ffi::c_int, game))
                .base
                .x_pos;
        let mut xd: uint16 = (x_pos).wrapping_sub((*E).base.x_pos);
        let mut v7: uint16 = abs16(xd);
        let mut v8: uint16 = 0;
        if v7 as ::core::ffi::c_int - 24 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
            if xd as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
                v7 = (v7).wrapping_add(24u16);
            }
            v8 = g_word_A28E80[v7 as usize] as uint16;
        } else {
            v8 = 1 as uint16;
        }
        Enemy_MoveDown(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            (v8 as int16 as int32) << 16 as ::core::ffi::c_int,
            game,
        );
        if R48 != 0 {
            let ref mut fresh1 =
                *((&raw mut game.g_ram as *mut uint8).offset(0xb5c) as *mut uint16);
            *fresh1 = (*fresh1 as ::core::ffi::c_int
                + ((*Get_MaridiaBeybladeTurtle(
                    *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                        as ::core::ffi::c_int,
                    game,
                ))
                .base
                .y_pos as ::core::ffi::c_int
                    - R50 as ::core::ffi::c_int)) as uint16;
        }
    }
    return jp;
}
pub unsafe fn MaridiaBeybladeTurtle_Instr_9412(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut v4: int16 = 0;
    let mut E: *mut Enemy_MaridiaBeybladeTurtle = Get_MaridiaBeybladeTurtle(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut xd: uint16 = ((*E).mbte_var_C).wrapping_sub((*E).base.x_pos);
    if ((abs16(xd)) as i16).wrapping_sub((g_word_A28D50) as i16) >= 0i16 {
        if xd as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
            v4 = 1 as int16;
        } else {
            v4 = -(1 as ::core::ffi::c_int) as int16;
        }
        (*E).mbte_var_E = v4 as uint16;
    }
    if ((*Get_MaridiaBeybladeTurtle(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .mbte_var_E as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int)
        != 0i32
    {
        return addr_kMaridiaBeybladeTurtle_Ilist_8B80 as ::core::ffi::c_int as uintptr_t
            as *const uint16;
    }
    return addr_kMaridiaBeybladeTurtle_Ilist_8C72 as ::core::ffi::c_int as uintptr_t
        as *const uint16;
}
pub unsafe fn MaridiaBeybladeTurtle_Instr_9447(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_MaridiaBeybladeTurtle(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .mbte_var_A = ((0xa28f3fi32) as u16) & (0xffffu16);
    return jp;
}
pub unsafe fn MaridiaBeybladeTurtle_Instr_9451(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_MaridiaTurtle = Get_MaridiaTurtle(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).mte_var_A = ((0xa28f8di32) as u16) & (0xffffu16);
    (*E).mte_var_E = -(1 as ::core::ffi::c_int) as uint16;
    (*E).mte_var_00 = 16 as uint16;
    return addr_kMaridiaBeybladeTurtle_Ilist_8C02 as ::core::ffi::c_int as uintptr_t
        as *const uint16;
}
pub unsafe fn MaridiaBeybladeTurtle_Instr_946B(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_MaridiaTurtle = Get_MaridiaTurtle(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).mte_var_A = ((0xa28f8di32) as u16) & (0xffffu16);
    (*E).mte_var_E = 1 as uint16;
    (*E).mte_var_00 = 16 as uint16;
    return addr_kMaridiaBeybladeTurtle_Ilist_8C02 as ::core::ffi::c_int as uintptr_t
        as *const uint16;
}
pub unsafe fn MaridiaBeybladeTurtle_Instr_9485(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    if CheckIfEnemyTouchesSamus(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    ) != 0
    {
        if ((*Get_MaridiaBeybladeTurtle(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        ))
        .mbte_var_E as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int)
            == 0i32
        {
            return addr_kMaridiaBeybladeTurtle_Ilist_8D40 as ::core::ffi::c_int as uintptr_t
                as *const uint16;
        } else {
            return addr_kMaridiaBeybladeTurtle_Ilist_8C62 as ::core::ffi::c_int as uintptr_t
                as *const uint16;
        }
    }
    return jp;
}
pub unsafe fn MaridiaBeybladeTurtle_Instr_94A1(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_MaridiaBeybladeTurtle = Get_MaridiaBeybladeTurtle(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if CheckIfEnemyTouchesSamus(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    ) != 0
    {
        (*E).mbte_var_A = ((0xa2925ei32) as u16) & (0xffffu16);
    } else {
        (*E).mbte_var_A = ((0xa29142i32) as u16) & (0xffffu16);
    }
    if ((*E).mbte_var_E as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) == 0i32 {
        return addr_kMaridiaBeybladeTurtle_Ilist_8C72 as ::core::ffi::c_int as uintptr_t
            as *const uint16;
    }
    return addr_kMaridiaBeybladeTurtle_Ilist_8B80 as ::core::ffi::c_int as uintptr_t
        as *const uint16;
}
pub unsafe fn MaridiaBeybladeTurtle_Instr_94C7(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_MaridiaBeybladeTurtle = Get_MaridiaBeybladeTurtle(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).mbte_var_A = ((0xa29239i32) as u16) & (0xffffu16);
    return jp;
}
pub unsafe fn MaridiaBeybladeTurtle_Instr_94D1(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    QueueSfx2_Max6(0x3a as uint16, game);
    return jp;
}
pub unsafe fn ThinHoppingBlobs_Init(game: &mut Game) {
    let mut E: *mut Enemy_ThinHoppingBlobs = Get_ThinHoppingBlobs(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.spritemap_pointer = addr_kSpritemap_Nothing_A2 as uint16;
    (*E).thbs_var_A = 0 as uint16;
    sub_A29A6C(addr_kThinHoppingBlobs_Ilist_99AD as uint16, game);
    (*E).thbs_var_00 = 0 as uint16;
    (*E).thbs_var_D = ((0xa29b65i32) as u16) & (0xffffu16);
    (*E).thbs_var_C = (*E).thbs_parameter_1;
    (*E).thbs_var_04 = 0 as uint16;
}
pub unsafe fn sub_A29A6C(mut a: uint16, game: &mut Game) {
    let mut v1: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*v1).current_instruction = a;
    (*v1).instruction_timer = 1 as uint16;
    (*v1).timer = 0 as uint16;
}
pub unsafe fn CallThinHoppingBlobs(mut ea: uint32, game: &mut Game) {
    match ea {
        10656613 => {
            ThinHoppingBlobs_Func6(
                *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
                game,
            );
            return;
        }
        10656641 => {
            ThinHoppingBlobs_Func7(game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn ThinHoppingBlobs_Main(game: &mut Game) {
    let mut E: *mut Enemy_ThinHoppingBlobs = Get_ThinHoppingBlobs(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    CallThinHoppingBlobs((((*E).thbs_var_D) as u32) | (0xa20000u32), game);
}
pub unsafe fn ThinHoppingBlobs_Func1(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_ThinHoppingBlobs = Get_ThinHoppingBlobs(k as ::core::ffi::c_int, game);
    if (*E).thbs_var_00 as ::core::ffi::c_int - 3 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        TwinHoppingBlobs_Func2(game);
    }
    ThinHoppingBlobs_Func3(game);
    ThinHoppingBlobs_Func5(game);
}
pub unsafe fn TwinHoppingBlobs_Func2(game: &mut Game) {
    let mut E: *mut Enemy_ThinHoppingBlobs = Get_ThinHoppingBlobs(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).thbs_var_00 = IsSamusWithinEnemy_X(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        (*E).thbs_parameter_2,
        game,
    );
}
pub unsafe fn ThinHoppingBlobs_Func3(game: &mut Game) {
    let mut E: *mut Enemy_ThinHoppingBlobs = Get_ThinHoppingBlobs(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).thbs_var_02 = 1 as uint16;
    if (GetSamusEnemyDelta_X(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    ) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int)
        == 0i32
    {
        (*E).thbs_var_02 = 0 as uint16;
    }
    if (*E).thbs_var_04 != 0 {
        (*E).thbs_var_02 = (*E).thbs_var_05;
    }
    (*E).thbs_var_04 = 0 as uint16;
    let mut R28: uint16 = ThinHoppingBlobs_Func4(game);
    let mut thbs_var_00: uint16 = (*E).thbs_var_00;
    if thbs_var_00 as ::core::ffi::c_int - 3 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0
    {
        if thbs_var_00 == 0 {
            R28 = (R28) & (1u16);
        }
        thbs_var_00 = R28;
        if R28 as ::core::ffi::c_int - 2 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
            thbs_var_00 = 2 as uint16;
        }
    }
    let mut v3: uint16 = (8u16).wrapping_mul(thbs_var_00);
    (*E).thbs_var_F = v3;
    (*E).thbs_var_E = *((game.g_rom.offset(crate::rtl::lorom_offset(0xa29a0d)) as *const uint8
        as *mut uint16 as *mut uint8)
        .offset(v3 as ::core::ffi::c_int as isize) as *mut uint16);
}
pub unsafe fn ThinHoppingBlobs_Func4(game: &mut Game) -> uint16 {
    NextRandom(game);
    return ((*Get_ThinHoppingBlobs(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .base
    .frame_counter as ::core::ffi::c_int
        + *((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16) as ::core::ffi::c_int
        & 7 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn ThinHoppingBlobs_Func5(game: &mut Game) {
    let mut v2: uint16 = 0;
    let mut E: *mut Enemy_ThinHoppingBlobs = Get_ThinHoppingBlobs(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut thbs_var_F: uint16 = (*E).thbs_var_F;
    let mut r22: uint16 = 0 as uint16;
    let mut r24: uint16 = 0 as uint16;
    loop {
        r22 = (r22).wrapping_add(
            *((game.g_rom.offset(crate::rtl::lorom_offset(0xa29a09)) as *const uint8 as *mut uint16
                as *mut uint8)
                .offset(thbs_var_F as ::core::ffi::c_int as isize) as *mut uint16),
        );
        r24 = (r24).wrapping_add(
            *((game.g_rom.offset(crate::rtl::lorom_offset(0xa2838f)) as *const uint8 as *mut uint16
                as *mut uint8)
                .offset(
                    (8 as ::core::ffi::c_int
                        * ((r22 as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int)
                            >> 8 as ::core::ffi::c_int)) as isize,
                )
                .offset(1) as *mut uint16),
        );
        thbs_var_F = (*E).thbs_var_F;
        v2 = u16::swap_bytes(
            *((game.g_rom.offset(crate::rtl::lorom_offset(0xa29a07)) as *const uint8 as *mut uint16
                as *mut uint8)
                .offset(thbs_var_F as ::core::ffi::c_int as isize) as *mut uint16),
        );
        if !(v2 as ::core::ffi::c_int - r24 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            == 0)
        {
            break;
        }
    }
    (*E).thbs_var_B = r22;
    (*E).thbs_var_03 = 0 as uint16;
    let mut v3: uint16 = (r22 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
    (*E).thbs_var_07 = (r22 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
    (*E).thbs_var_06 = ((*E).thbs_var_07 as ::core::ffi::c_int
        + (v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)) as uint16;
}
pub unsafe fn ThinHoppingBlobs_Func6(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_ThinHoppingBlobs = Get_ThinHoppingBlobs(k as ::core::ffi::c_int, game);
    (*E).thbs_var_C = (*E).thbs_var_C.wrapping_sub(1);
    if ((*E).thbs_var_C as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        (*E).thbs_var_D = ((0xa29b81i32) as u16) & (0xffffu16);
        (*E).thbs_var_C = (*E).thbs_parameter_1;
        (*E).thbs_var_01 = 1 as uint16;
        ThinHoppingBlobs_Func1(k, game);
    }
}
pub unsafe fn CallThinHoppingBlobsVarE(mut ea: uint32, game: &mut Game) {
    match ea {
        10657035 => {
            ThinHoppingBlobs_Func11(game);
            return;
        }
        10657067 => {
            ThinHoppingBlobs_Func12(game);
            return;
        }
        10657099 => {
            ThinHoppingBlobs_Func13(game);
            return;
        }
        10657131 => {
            ThinHoppingBlobs_Func14(game);
            return;
        }
        10657176 => {
            ThinHoppingBlobs_Func15(game);
            return;
        }
        10657229 => {
            ThinHoppingBlobs_Func16(game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn ThinHoppingBlobs_Func7(game: &mut Game) {
    let mut E: *mut Enemy_ThinHoppingBlobs = Get_ThinHoppingBlobs(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    CallThinHoppingBlobsVarE((((*E).thbs_var_E) as u32) | (0xa20000u32), game);
}
pub unsafe fn ThinHoppingBlobs_Func8(game: &mut Game) {
    let mut E: *mut Enemy_ThinHoppingBlobs = Get_ThinHoppingBlobs(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut thbs_var_B: uint16 = (*E).thbs_var_B;
    if thbs_var_B as ::core::ffi::c_int - 0x4000 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        thbs_var_B = 0x4000 as uint16;
    }
    let mut v6: uint16 = (8 as ::core::ffi::c_int
        * (thbs_var_B as ::core::ffi::c_int >> 8 as ::core::ffi::c_int))
        as uint16;
    if (*E).thbs_var_03 == 0 {
        v6 = (v6).wrapping_add(4u16);
    }
    if Enemy_MoveDown(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        *(game.g_rom.offset(crate::rtl::lorom_offset(0xa0cbc7)) as *const uint8 as *mut uint32)
            .offset((v6 as ::core::ffi::c_int >> 2 as ::core::ffi::c_int) as isize)
            as int32,
        game,
    ) != 0
    {
        if (*E).thbs_var_03 != 0 {
            (*E).thbs_var_D = ((0xa29b65i32) as u16) & (0xffffu16);
        } else {
            (*E).thbs_var_04 = 1 as uint16;
            (*E).thbs_var_05 = ((*E).thbs_var_02) ^ (1u16);
            (*E).thbs_var_00 = 4 as uint16;
            (*E).thbs_var_E = ((0xa29d98i32) as u16) & (0xffffu16);
        }
        (*E).thbs_var_01 = 0 as uint16;
    } else {
        let mut thbs_var_F: uint16 = (*E).thbs_var_F;
        if (*E).thbs_var_03 != 0 {
            ThinHoppingBlobs_Func10(game);
            (*E).thbs_var_B = ((*E).thbs_var_B).wrapping_add(
                *((game.g_rom.offset(crate::rtl::lorom_offset(0xa29a0b)) as *const uint8
                    as *mut uint16 as *mut uint8)
                    .offset(thbs_var_F as ::core::ffi::c_int as isize)
                    as *mut uint16),
            );
        } else {
            ThinHoppingBlobs_Func9(game);
            (*E).thbs_var_B = ((*E).thbs_var_B).wrapping_sub(
                *((game.g_rom.offset(crate::rtl::lorom_offset(0xa29a0b)) as *const uint8
                    as *mut uint16 as *mut uint8)
                    .offset(thbs_var_F as ::core::ffi::c_int as isize)
                    as *mut uint16),
            );
        }
        if (*E).thbs_var_B as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
            (*E).thbs_var_03 = 1 as uint16;
            (*E).thbs_var_B = 0 as uint16;
        }
        let mut R20: uint16 = ((*((game.g_rom.offset(crate::rtl::lorom_offset(0xa29a09))
            as *const uint8 as *mut uint16 as *mut uint8)
            .offset((*E).thbs_var_F as ::core::ffi::c_int as isize)
            as *mut uint16) as ::core::ffi::c_int
            & 0xff00 as ::core::ffi::c_int)
            >> 8 as ::core::ffi::c_int) as uint16;
        if (*E).thbs_var_02 != 0 {
            R20 = -(R20 as ::core::ffi::c_int) as uint16;
        }
        if Enemy_MoveRight_IgnoreSlopes(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            (R20 as int16 as int32) << 16 as ::core::ffi::c_int,
            game,
        ) != 0
        {
            (*E).thbs_var_04 = 1 as uint16;
            (*E).thbs_var_05 = ((*E).thbs_var_02) ^ (1u16);
            (*E).thbs_var_01 = 0 as uint16;
            (*E).thbs_var_00 = 4 as uint16;
            (*E).thbs_var_E = ((0xa29d98i32) as u16) & (0xffffu16);
        }
    };
}
pub unsafe fn ThinHoppingBlobs_Func9(game: &mut Game) {
    let mut E: *mut Enemy_ThinHoppingBlobs = Get_ThinHoppingBlobs(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut v2: uint16 = 0;
    if (*E).thbs_var_02 != 0 {
        let mut thbs_var_B: uint16 = (*E).thbs_var_B;
        if ((thbs_var_B) as i16).wrapping_sub(((*E).thbs_var_06) as i16) >= 0i16 {
            v2 = addr_kThinHoppingBlobs_Ilist_9A01 as uint16;
        } else if ((thbs_var_B) as i16).wrapping_sub(((*E).thbs_var_07) as i16) >= 0i16 {
            v2 = addr_kThinHoppingBlobs_Ilist_99FB as uint16;
        } else {
            v2 = addr_kThinHoppingBlobs_Ilist_99F5 as uint16;
        }
    } else {
        let mut v1: uint16 = (*E).thbs_var_B;
        if ((v1) as i16).wrapping_sub(((*E).thbs_var_06) as i16) >= 0i16 {
            v2 = addr_kThinHoppingBlobs_Ilist_99E9 as uint16;
        } else if ((v1) as i16).wrapping_sub(((*E).thbs_var_07) as i16) >= 0i16 {
            v2 = addr_kThinHoppingBlobs_Ilist_99EF as uint16;
        } else {
            v2 = addr_kThinHoppingBlobs_Ilist_99F5 as uint16;
        }
    }
    sub_A29A6C(v2, game);
}
pub unsafe fn ThinHoppingBlobs_Func10(game: &mut Game) {
    let mut E: *mut Enemy_ThinHoppingBlobs = Get_ThinHoppingBlobs(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut v2: uint16 = 0;
    if (*E).thbs_var_02 != 0 {
        let mut thbs_var_B: uint16 = (*E).thbs_var_B;
        if ((thbs_var_B) as i16).wrapping_sub(((*E).thbs_var_07) as i16) < 0i16 {
            v2 = addr_kThinHoppingBlobs_Ilist_99F5 as uint16;
        } else if ((thbs_var_B) as i16).wrapping_sub(((*E).thbs_var_06) as i16) < 0i16 {
            v2 = addr_kThinHoppingBlobs_Ilist_99EF as uint16;
        } else {
            v2 = addr_kThinHoppingBlobs_Ilist_99E9 as uint16;
        }
    } else {
        let mut v1: uint16 = (*E).thbs_var_B;
        if ((v1) as i16).wrapping_sub(((*E).thbs_var_07) as i16) < 0i16 {
            v2 = addr_kThinHoppingBlobs_Ilist_99F5 as uint16;
        } else if ((v1) as i16).wrapping_sub(((*E).thbs_var_06) as i16) < 0i16 {
            v2 = addr_kThinHoppingBlobs_Ilist_99FB as uint16;
        } else {
            v2 = addr_kThinHoppingBlobs_Ilist_9A01 as uint16;
        }
    }
    sub_A29A6C(v2, game);
}
pub unsafe fn ThinHoppingBlobs_Func11(game: &mut Game) {
    ThinHoppingBlobs_Func8(game);
    let mut E: *mut Enemy_ThinHoppingBlobs = Get_ThinHoppingBlobs(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).thbs_var_04 != 0u16 || (*E).thbs_var_01 == 0 {
        (*E).thbs_var_01 = 0 as uint16;
        sub_A29A6C(addr_kThinHoppingBlobs_Ilist_99D5 as uint16, game);
    }
}
pub unsafe fn ThinHoppingBlobs_Func12(game: &mut Game) {
    ThinHoppingBlobs_Func8(game);
    let mut E: *mut Enemy_ThinHoppingBlobs = Get_ThinHoppingBlobs(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).thbs_var_04 != 0u16 || (*E).thbs_var_01 == 0 {
        (*E).thbs_var_01 = 0 as uint16;
        sub_A29A6C(addr_kThinHoppingBlobs_Ilist_99C1 as uint16, game);
    }
}
pub unsafe fn ThinHoppingBlobs_Func13(game: &mut Game) {
    ThinHoppingBlobs_Func8(game);
    let mut E: *mut Enemy_ThinHoppingBlobs = Get_ThinHoppingBlobs(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).thbs_var_04 != 0u16 || (*E).thbs_var_01 == 0 {
        (*E).thbs_var_01 = 0 as uint16;
        sub_A29A6C(addr_kThinHoppingBlobs_Ilist_99AD as uint16, game);
    }
}
pub unsafe fn ThinHoppingBlobs_Func14(game: &mut Game) {
    ThinHoppingBlobs_Func8(game);
    let mut E: *mut Enemy_ThinHoppingBlobs = Get_ThinHoppingBlobs(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).thbs_var_04 == 0 {
        if (*E).thbs_var_01 != 0 {
            return;
        }
        (*E).thbs_var_00 = 0 as uint16;
        (*E).thbs_var_D = ((0xa29b65i32) as u16) & (0xffffu16);
    }
    (*E).thbs_var_01 = 0 as uint16;
    sub_A29A6C(addr_kThinHoppingBlobs_Ilist_99D5 as uint16, game);
}
pub unsafe fn ThinHoppingBlobs_Func15(game: &mut Game) {
    let mut E: *mut Enemy_ThinHoppingBlobs = Get_ThinHoppingBlobs(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut thbs_var_F: uint16 = (*E).thbs_var_F;
    let mut R20: uint16 = ((*((game.g_rom.offset(crate::rtl::lorom_offset(0xa29a0b)) as *const uint8
        as *mut uint16 as *mut uint8)
        .offset(thbs_var_F as ::core::ffi::c_int as isize)
        as *mut uint16) as ::core::ffi::c_int
        & 0xff00 as ::core::ffi::c_int)
        >> 8 as ::core::ffi::c_int) as uint16;
    let mut R18: uint16 = ((*(game.g_rom.offset(crate::rtl::lorom_offset(0xa29a0b)) as *const uint8
        as *mut uint16 as *mut uint8)
        .offset(thbs_var_F as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int) as uint16;
    if Enemy_MoveDown(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        ((R20 as uint32) << 16 as ::core::ffi::c_int | R18 as uint32) as int32,
        game,
    ) != 0
    {
        let mut R28: uint16 = ThinHoppingBlobs_Func4(game);
        (*E).thbs_var_00 = ((R28 as ::core::ffi::c_int & 1 as ::core::ffi::c_int)
            + 5 as ::core::ffi::c_int) as uint16;
        (*E).thbs_var_D = ((0xa29b65i32) as u16) & (0xffffu16);
    }
}
pub unsafe fn ThinHoppingBlobs_Func16(game: &mut Game) {
    ThinHoppingBlobs_Func8(game);
    let mut E: *mut Enemy_ThinHoppingBlobs = Get_ThinHoppingBlobs(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).thbs_var_01 == 0 {
        (*E).thbs_var_01 = 0 as uint16;
        (*E).thbs_var_00 = 3 as uint16;
        (*E).thbs_var_D = ((0xa29b65i32) as u16) & (0xffffu16);
        sub_A29A6C(addr_kThinHoppingBlobs_Ilist_99D5 as uint16, game);
    }
}
pub unsafe fn SpikeShootingPlant_Instr_9F2A(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    QueueSfx2_Max6(0x34 as uint16, game);
    return jp;
}
pub unsafe fn SpikeShootingPlant_Init(game: &mut Game) {
    let mut E: *mut Enemy_SpikeShootingPlant = Get_SpikeShootingPlant(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.spritemap_pointer = addr_kSpritemap_Nothing_A2 as uint16;
    if *(&raw mut (*E).sspt_parameter_1 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>())
            .wrapping_div(::core::mem::size_of::<uint8>())
            .wrapping_sub(1 as usize) as isize,
    ) != 0
    {
        SpikeShootingPlant_5(game);
    } else {
        SpikeShootingPlant_7(game);
    }
    let mut sspt_parameter_1_low: uint16 =
        *(&raw mut (*E).sspt_parameter_1 as *mut uint8).offset(0) as uint16;
    (*E).sspt_var_E = sspt_parameter_1_low;
    (*E).sspt_var_F = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa29f42)) as *const uint8
        as *mut uint16)
        .offset(sspt_parameter_1_low as isize);
    let mut v2: ::core::ffi::c_int =
        *(&raw mut (*E).sspt_parameter_2 as *mut uint8).offset(0) as ::core::ffi::c_int;
    (*E).sspt_var_01 = (*(game.g_rom.offset(crate::rtl::lorom_offset(0xa29f36)) as *const uint8
        as *mut uint16)
        .offset(v2 as isize))
    .wrapping_add((*E).base.x_pos);
    (*E).sspt_var_00 = ((*E).base.x_pos).wrapping_sub(
        *(game.g_rom.offset(crate::rtl::lorom_offset(0xa29f36)) as *const uint8 as *mut uint16)
            .offset(v2 as isize),
    );
    let mut v3: ::core::ffi::c_int = 8 as ::core::ffi::c_int
        * *(&raw mut (*E).sspt_parameter_2 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1 as usize) as isize,
        ) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    (*E).sspt_var_B = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8
        as *mut uint16)
        .offset(v3 as isize);
    (*E).sspt_var_A = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8
        as *mut uint16)
        .offset((v3 + 1 as ::core::ffi::c_int) as isize);
    (*E).sspt_var_D = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8
        as *mut uint16)
        .offset((v3 + 2 as ::core::ffi::c_int) as isize);
    (*E).sspt_var_C = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8
        as *mut uint16)
        .offset((v3 + 3 as ::core::ffi::c_int) as isize);
}
pub unsafe fn SpikeShootingPlant_Main(game: &mut Game) {
    let mut E: *mut Enemy_SpikeShootingPlant = Get_SpikeShootingPlant(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    CallEnemyPreInstr((((*E).sspt_var_F) as u32) | (0xa20000u32), game);
}
pub unsafe fn SpikeShootingPlant_2(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_SpikeShootingPlant = Get_SpikeShootingPlant(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        ((*E).sspt_var_D as uint32) << 16 as ::core::ffi::c_int | (*E).sspt_var_C as uint32,
    );
    if (((*E).base.x_pos) as i16).wrapping_sub(((*E).sspt_var_00) as i16) < 0i16 {
        (*E).sspt_var_F = ((0xa29feci32) as u16) & (0xffffu16);
        (*E).sspt_var_E = 1 as uint16;
    }
    SpikeShootingPlant_4(game);
}
pub unsafe fn SpikeShootingPlant_3(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_SpikeShootingPlant = Get_SpikeShootingPlant(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        ((*E).sspt_var_B as uint32) << 16 as ::core::ffi::c_int | (*E).sspt_var_A as uint32,
    );
    if (((*E).base.x_pos) as i16).wrapping_sub(((*E).sspt_var_01) as i16) >= 0i16 {
        (*E).sspt_var_F = ((0xa29fbai32) as u16) & (0xffffu16);
        (*E).sspt_var_E = 0 as uint16;
    }
    SpikeShootingPlant_4(game);
}
pub unsafe fn SpikeShootingPlant_4(game: &mut Game) {
    NextRandom(game);
    let mut E: *mut Enemy_SpikeShootingPlant = Get_SpikeShootingPlant(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if ((*(&raw mut (*E).base.frame_counter as *mut uint8).offset(0))
        .wrapping_add((*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)) as u8))
        as ::core::ffi::c_int
        - 3 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        (*E).sspt_var_F = ((0xa2a01bi32) as u16) & (0xffffu16);
        if *(&raw mut (*E).sspt_parameter_1 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1 as usize) as isize,
        ) != 0
        {
            SpikeShootingPlant_6(game);
        } else {
            SpikeShootingPlant_8(game);
        }
    }
}
pub unsafe fn SpikeShootingPlant_5(game: &mut Game) {
    let mut E: *mut Enemy_SpikeShootingPlant = Get_SpikeShootingPlant(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.current_instruction = addr_kSpikeShootingPlant_Ilist_9E8A as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
}
pub unsafe fn SpikeShootingPlant_6(game: &mut Game) {
    let mut E: *mut Enemy_SpikeShootingPlant = Get_SpikeShootingPlant(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.current_instruction = addr_kSpikeShootingPlant_Ilist_9EB0 as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
}
pub unsafe fn SpikeShootingPlant_7(game: &mut Game) {
    let mut E: *mut Enemy_SpikeShootingPlant = Get_SpikeShootingPlant(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.current_instruction = addr_kSpikeShootingPlant_Ilist_9EDA as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
}
pub unsafe fn SpikeShootingPlant_8(game: &mut Game) {
    let mut E: *mut Enemy_SpikeShootingPlant = Get_SpikeShootingPlant(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.current_instruction = addr_kSpikeShootingPlant_Ilist_9F00 as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
}
pub unsafe fn SpikeShootingPlant_Instr_A095(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_SpikeShootingPlant =
        Get_SpikeShootingPlant(k as ::core::ffi::c_int, game);
    (*E).sspt_var_F = ((0xa29fbai32) as u16) & (0xffffu16);
    if (*E).sspt_var_E != 0 {
        (*E).sspt_var_F = ((0xa29feci32) as u16) & (0xffffu16);
    }
    return jp;
}
pub unsafe fn SpikeShootingPlant_Instr_A0A7(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    SpawnEprojWithGfx(
        *jp,
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        addr_kEproj_SpikeShootingPlantSpikes as uint16,
        game,
    );
    return jp.offset(1);
}
static off_A2A3D3: [Option<Func_V>; 7] = unsafe {
    [
        Some(MaridiaSpikeyShell_1 as unsafe fn(&mut Game) -> ()),
        Some(MaridiaSpikeyShell_2 as unsafe fn(&mut Game) -> ()),
        Some(MaridiaSpikeyShell_3 as unsafe fn(&mut Game) -> ()),
        Some(MaridiaSpikeyShell_4 as unsafe fn(&mut Game) -> ()),
        Some(MaridiaSpikeyShell_5 as unsafe fn(&mut Game) -> ()),
        Some(MaridiaSpikeyShell_6 as unsafe fn(&mut Game) -> ()),
        Some(MaridiaSpikeyShell_7 as unsafe fn(&mut Game) -> ()),
    ]
};
pub unsafe fn MaridiaSpikeyShell_Init(game: &mut Game) {
    let mut E: *mut Enemy_MaridiaSpikeyShell = Get_MaridiaSpikeyShell(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.spritemap_pointer = addr_kSpritemap_Nothing_A2 as uint16;
    let mut mssl_parameter_1_low: uint16 =
        *(&raw mut (*E).mssl_parameter_1 as *mut uint8).offset(0) as uint16;
    (*E).mssl_var_E = mssl_parameter_1_low;
    off_A2A3D3[(mssl_parameter_1_low as ::core::ffi::c_int & 1 as ::core::ffi::c_int) as usize]
        .expect("non-null function pointer")(game);
    let mut v3: ::core::ffi::c_int = 4 as ::core::ffi::c_int
        * *(&raw mut (*E).mssl_parameter_2 as *mut uint8).offset(0) as ::core::ffi::c_int;
    (*E).mssl_var_B = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8
        as *mut uint16)
        .offset(v3 as isize);
    (*E).mssl_var_A = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8
        as *mut uint16)
        .offset((v3 + 1 as ::core::ffi::c_int) as isize);
    (*E).mssl_var_D = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8
        as *mut uint16)
        .offset((v3 + 2 as ::core::ffi::c_int) as isize);
    (*E).mssl_var_C = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8
        as *mut uint16)
        .offset((v3 + 3 as ::core::ffi::c_int) as isize);
    let mut v4: ::core::ffi::c_int = *(&raw mut (*E).mssl_parameter_2 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>())
            .wrapping_div(::core::mem::size_of::<uint8>())
            .wrapping_sub(1 as usize) as isize,
    ) as ::core::ffi::c_int;
    (*E).mssl_var_02 = (*(game.g_rom.offset(crate::rtl::lorom_offset(0xa2a3dd)) as *const uint8
        as *mut uint16)
        .offset(v4 as isize))
    .wrapping_add((*E).base.x_pos);
    (*E).mssl_var_01 = ((*E).base.x_pos).wrapping_sub(
        *(game.g_rom.offset(crate::rtl::lorom_offset(0xa2a3dd)) as *const uint8 as *mut uint16)
            .offset(v4 as isize),
    );
    (*E).mssl_var_00 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa2a3ed)) as *const uint8
        as *mut uint16)
        .offset(
            *(&raw mut (*E).mssl_parameter_1 as *mut uint8).offset(
                (::core::mem::size_of::<uint16>())
                    .wrapping_div(::core::mem::size_of::<uint8>())
                    .wrapping_sub(1 as usize) as isize,
            ) as isize,
        );
    (*E).mssl_var_F = 0 as uint16;
    if (*E).mssl_var_E == 2u16 {
        (*E).mssl_var_F = 16 as uint16;
        (*E).base.y_pos = ((*E).base.y_pos).wrapping_add(16u16);
    }
}
pub unsafe fn MaridiaSpikeyShell_Main(game: &mut Game) {
    let mut e: *mut Enemy_MaridiaSpikeyShell = Get_MaridiaSpikeyShell(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    off_A2A3D3[((*e).mssl_var_E as int16 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as usize]
        .expect("non-null function pointer")(game);
}
pub unsafe fn MaridiaSpikeyShell_1(game: &mut Game) {
    let mut E: *mut Enemy_MaridiaSpikeyShell = Get_MaridiaSpikeyShell(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.current_instruction = addr_kMaridiaSpikeyShell_Ilist_A3AB as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
}
pub unsafe fn MaridiaSpikeyShell_2(game: &mut Game) {
    let mut E: *mut Enemy_MaridiaSpikeyShell = Get_MaridiaSpikeyShell(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.current_instruction = addr_kMaridiaSpikeyShell_Ilist_A3BD as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
}
pub unsafe fn MaridiaSpikeyShell_3(game: &mut Game) {
    let mut E: *mut Enemy_MaridiaSpikeyShell = Get_MaridiaSpikeyShell(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        ((*E).mssl_var_D as uint32) << 16 as ::core::ffi::c_int | (*E).mssl_var_C as uint32,
    );
    if (((*E).base.x_pos) as i16).wrapping_sub(((*E).mssl_var_01) as i16) < 0i16 {
        (*E).mssl_var_E = (*E).mssl_var_E.wrapping_sub(1);
    }
    MaridiaSpikeyShell_8(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
}
pub unsafe fn MaridiaSpikeyShell_4(game: &mut Game) {
    let mut E: *mut Enemy_MaridiaSpikeyShell = Get_MaridiaSpikeyShell(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        ((*E).mssl_var_B as uint32) << 16 as ::core::ffi::c_int | (*E).mssl_var_A as uint32,
    );
    if (((*E).base.x_pos) as i16).wrapping_sub(((*E).mssl_var_02) as i16) >= 0i16 {
        (*E).mssl_var_E = 0 as uint16;
    }
    MaridiaSpikeyShell_8(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
}
pub unsafe fn MaridiaSpikeyShell_5(game: &mut Game) {
    let mut E: *mut Enemy_MaridiaSpikeyShell = Get_MaridiaSpikeyShell(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut v1: uint16 = ((*E).mssl_var_00).wrapping_sub(1u16);
    (*E).mssl_var_00 = v1;
    if v1 == 0 {
        (*E).mssl_var_E = 4 as uint16;
    }
}
pub unsafe fn MaridiaSpikeyShell_6(game: &mut Game) {
    let mut E: *mut Enemy_MaridiaSpikeyShell = Get_MaridiaSpikeyShell(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.y_pos = (*E).base.y_pos.wrapping_add(1);
    (*E).mssl_var_F = (*E).mssl_var_F.wrapping_add(1);
    if (*E).mssl_var_F as ::core::ffi::c_int - 16 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        (*E).mssl_var_E = 2 as uint16;
        (*E).mssl_var_00 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa2a3ed)) as *const uint8
            as *mut uint16)
            .offset(
                *(&raw mut (*E).mssl_parameter_1 as *mut uint8).offset(
                    (::core::mem::size_of::<uint16>())
                        .wrapping_div(::core::mem::size_of::<uint8>())
                        .wrapping_sub(1 as usize) as isize,
                ) as isize,
            );
    }
}
pub unsafe fn MaridiaSpikeyShell_7(game: &mut Game) {
    let mut E: *mut Enemy_MaridiaSpikeyShell = Get_MaridiaSpikeyShell(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.y_pos = (*E).base.y_pos.wrapping_sub(1);
    let fresh29 = (*E).mssl_var_F;
    (*E).mssl_var_F = (*E).mssl_var_F.wrapping_sub(1);
    if fresh29 == 1u16 {
        (*E).mssl_var_E = ((*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16
            as *mut uint8)
            .offset(
                (::core::mem::size_of::<uint16>())
                    .wrapping_div(::core::mem::size_of::<uint8>())
                    .wrapping_sub(1 as usize) as isize,
            )) as u16)
            & (1u16);
    }
}
pub unsafe fn MaridiaSpikeyShell_8(mut k: uint16, game: &mut Game) {
    NextRandom(game);
    let mut E: *mut Enemy_MaridiaSpikeyShell =
        Get_MaridiaSpikeyShell(k as ::core::ffi::c_int, game);
    if ((*(&raw mut (*E).base.frame_counter as *mut uint8).offset(0))
        .wrapping_add((*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)) as u8))
        as ::core::ffi::c_int
        - 6 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        (*E).mssl_var_E = 3 as uint16;
    }
}
pub unsafe fn MaridiaSpikeyShell_Instr_A56D(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_MaridiaSpikeyShell(k as ::core::ffi::c_int, game)).mssl_var_E = 0 as uint16;
    return jp;
}
pub unsafe fn MaridiaSpikeyShell_Instr_A571(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_MaridiaSpikeyShell(k as ::core::ffi::c_int, game)).mssl_var_E = 1 as uint16;
    return jp;
}
pub unsafe fn MaridiaSpikeyShell_Shot(game: &mut Game) {
    let mut E: *mut Enemy_MaridiaSpikeyShell = Get_MaridiaSpikeyShell(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).mssl_var_E as ::core::ffi::c_int - 1 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        Enemy_NormalShotAI_A2(game);
    }
}
pub unsafe fn GunshipTop_Init(game: &mut Game) {
    let mut E: *mut Enemy_GunshipTop = Get_GunshipTop(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        | (kEnemyProps_DisableSamusColl as ::core::ffi::c_int
            | kEnemyProps_Tangible as ::core::ffi::c_int)) as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
    (*E).base.current_instruction = addr_kGunshipTop_Ilist_A616 as uint16;
    (*E).base.palette_index = 3584 as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16) != 40u16
        || *((&raw mut game.g_ram as *mut uint8).offset(0x1f55) as *mut uint16) != 0u16
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0xd914) as *mut uint16)
            as ::core::ffi::c_int
            == kLoadingGameState_22_EscapingCeres as ::core::ffi::c_int
        {
            (*E).base.y_pos = (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16))
                .wrapping_sub(17u16);
            (*E).gtp_var_F = ((0xa2a80ci32) as u16) & (0xffffu16);
        } else {
            let mut v3: uint16 = ((*E).base.y_pos).wrapping_sub(25u16);
            (*E).base.y_pos = v3;
            (*E).gtp_var_E = v3;
            (*E).gtp_var_F = ((0xa2a9bdi32) as u16) & (0xffffu16);
        }
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16) = 1138 as uint16;
        let mut v5: uint16 = ((*E).base.y_pos).wrapping_sub(25u16);
        (*E).base.y_pos = v5;
        (*E).gtp_var_E = v5;
        (*E).gtp_var_F = ((0xa2ab60i32) as u16) & (0xffffu16);
        (*Get_GunshipTop(0 as ::core::ffi::c_int, game)).gtp_var_A = 144 as uint16;
    }
    SpawnPalfxObject(addr_stru_8DE1C0 as uint16, game);
    (*Get_GunshipTop(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .gtp_var_D = 1 as uint16;
    (*Get_GunshipTop(0 as ::core::ffi::c_int, game)).gtp_var_C = 0 as uint16;
}
pub unsafe fn GunshipBottom_Init(game: &mut Game) {
    let mut E: *mut Enemy_GunshipBottom = Get_GunshipBottom(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        | (kEnemyProps_DisableSamusColl as ::core::ffi::c_int
            | kEnemyProps_Tangible as ::core::ffi::c_int)) as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
    if (*E).gbm_parameter_2 != 0 {
        (*E).base.current_instruction = addr_kGunshipTop_Ilist_A60E as uint16;
    } else {
        (*E).base.current_instruction = addr_kGunshipTop_Ilist_A61C as uint16;
    }
    let mut v2: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0xe54)
        as *mut uint16) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    (*E).base.vram_tiles_index = *((&raw mut game.g_ram as *mut uint8).offset(0xe84)
        as *mut uint16)
        .offset((v2 + 106 as ::core::ffi::c_int) as isize);
    (*E).base.palette_index = 3584 as uint16;
    if (*E).gbm_parameter_2 != 0 {
        (*E).base.y_pos = (*((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
            .offset((v2 + 61 as ::core::ffi::c_int) as isize))
        .wrapping_sub(1u16);
        if *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16) as ::core::ffi::c_int
            == kPose_28_FaceL_Crouch as ::core::ffi::c_int
            && *((&raw mut game.g_ram as *mut uint8).offset(0x1f55) as *mut uint16) == 0
        {
            (*E).base.instruction_timer = 1 as uint16;
            (*E).base.current_instruction = addr_kGunshipTop_Ilist_A5BE as uint16;
        }
    } else if *((&raw mut game.g_ram as *mut uint8).offset(0xd914) as *mut uint16)
        as ::core::ffi::c_int
        == kGameState_34_CeresGoesBoom as ::core::ffi::c_int
    {
        (*E).base.y_pos = (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16))
            .wrapping_add(23u16);
    } else {
        (*E).base.y_pos = ((*E).base.y_pos).wrapping_add(15u16);
        (*E).gbm_var_D = 71 as uint16;
    }
    (*E).gbm_var_F = ((0xa2a7d7i32) as u16) & (0xffffu16);
}
pub unsafe fn GunshipTop_Main(game: &mut Game) {
    let mut E1: *mut Enemy_GunshipTop = Get_GunshipTop(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int,
        game,
    );
    let mut v2: bool = (*E1).gtp_var_D == 1u16;
    (*E1).gtp_var_D = (*E1).gtp_var_D.wrapping_sub(1);
    let mut v3: bool =
        ((*E1).gtp_var_D as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32;
    if v2 as ::core::ffi::c_int != 0 || v3 as ::core::ffi::c_int != 0 {
        QueueSfx2_Max6(0x4d as uint16, game);
        (*E1).gtp_var_D = 70 as uint16;
    }
    let mut E0: *mut Enemy_GunshipTop = Get_GunshipTop(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E0).gtp_var_F as ::core::ffi::c_int + 0x56be as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
        && (*E0).gtp_var_F as ::core::ffi::c_int + 0x53e5 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
    {
        GunshipTop_1(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        );
    }
    CallEnemyPreInstr((((*E0).gtp_var_F) as u32) | (0xa20000u32), game);
}
pub unsafe fn GunshipTop_1(mut k: uint16, game: &mut Game) {
    let mut E0: *mut Enemy_GunshipTop = Get_GunshipTop(k as ::core::ffi::c_int, game);
    let mut v3: bool = (*E0).gtp_var_D == 1u16;
    (*E0).gtp_var_D = (*E0).gtp_var_D.wrapping_sub(1);
    let mut v4: bool =
        ((*E0).gtp_var_D as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32;
    if v3 as ::core::ffi::c_int != 0 || v4 as ::core::ffi::c_int != 0 {
        let mut v5: uint16 = (2u16).wrapping_mul((*E0).gtp_var_C);
        (*E0).gtp_var_D = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa2a7cf)) as *const uint8
            as *mut uint8)
            .offset(v5 as isize) as uint16;
        let mut v6: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa2a7cf)) as *const uint8
            as *mut uint8)
            .offset((v5 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize)
            as int8 as uint16;
        (*E0).base.y_pos = ((*E0).base.y_pos).wrapping_add(v6);
        let mut E1: *mut Enemy_GunshipTop =
            Get_GunshipTop(k as ::core::ffi::c_int + 64 as ::core::ffi::c_int, game);
        (*E1).base.y_pos = ((*E1).base.y_pos).wrapping_add(v6);
        let mut E2: *mut Enemy_GunshipTop =
            Get_GunshipTop(k as ::core::ffi::c_int + 128 as ::core::ffi::c_int, game);
        (*E2).base.y_pos = ((*E2).base.y_pos).wrapping_add(v6);
        (*E0).gtp_var_C = ((*E0).gtp_var_C as ::core::ffi::c_int + 1 as ::core::ffi::c_int
            & 3 as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn GunshipTop_2(mut k: uint16, game: &mut Game) {
    let ref mut fresh8 = *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16);
    *fresh8 = (*fresh8).wrapping_sub(8u16);
    let mut E2: *mut Enemy_GunshipTop =
        Get_GunshipTop(k as ::core::ffi::c_int + 128 as ::core::ffi::c_int, game);
    (*E2).base.y_pos = ((*E2).base.y_pos).wrapping_sub(8u16);
    let mut E1: *mut Enemy_GunshipTop =
        Get_GunshipTop(k as ::core::ffi::c_int + 64 as ::core::ffi::c_int, game);
    (*E1).base.y_pos = ((*E1).base.y_pos).wrapping_sub(8u16);
    let mut E0: *mut Enemy_GunshipTop = Get_GunshipTop(k as ::core::ffi::c_int, game);
    let mut v4: uint16 = ((*E0).base.y_pos).wrapping_sub(8u16);
    (*E0).base.y_pos = v4;
    if v4 as ::core::ffi::c_int - 128 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        (*E0).gtp_var_F = ((0xa2a80ci32) as u16) & (0xffffu16);
    }
}
pub unsafe fn GunshipTop_3(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_GunshipTop = Get_GunshipTop(k as ::core::ffi::c_int, game);
    if (*E).base.y_pos as ::core::ffi::c_int - 768 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        AddToHiLo(
            (&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16,
            (&raw mut game.g_ram as *mut uint8).offset(0xafc) as *mut uint16,
            0x48000 as uint32,
        );
        let mut v3: *mut Enemy_GunshipTop =
            Get_GunshipTop(k as ::core::ffi::c_int + 128 as ::core::ffi::c_int, game);
        AddToHiLo(
            &raw mut (*v3).base.y_pos,
            &raw mut (*v3).base.y_subpos,
            0x48000 as uint32,
        );
        let mut v5: *mut Enemy_GunshipTop =
            Get_GunshipTop(k as ::core::ffi::c_int + 64 as ::core::ffi::c_int, game);
        AddToHiLo(
            &raw mut (*v5).base.y_pos,
            &raw mut (*v5).base.y_subpos,
            0x48000 as uint32,
        );
        AddToHiLo(
            &raw mut (*E).base.y_pos,
            &raw mut (*E).base.y_subpos,
            0x48000 as uint32,
        );
    } else {
        AddToHiLo(
            (&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16,
            (&raw mut game.g_ram as *mut uint8).offset(0xafc) as *mut uint16,
            0x28000 as uint32,
        );
        let mut v8: *mut Enemy_GunshipTop =
            Get_GunshipTop(k as ::core::ffi::c_int + 128 as ::core::ffi::c_int, game);
        AddToHiLo(
            &raw mut (*v8).base.y_pos,
            &raw mut (*v8).base.y_subpos,
            0x28000 as uint32,
        );
        let mut v10: *mut Enemy_GunshipTop =
            Get_GunshipTop(k as ::core::ffi::c_int + 64 as ::core::ffi::c_int, game);
        AddToHiLo(
            &raw mut (*v10).base.y_pos,
            &raw mut (*v10).base.y_subpos,
            0x28000 as uint32,
        );
        AddToHiLo(
            &raw mut (*E).base.y_pos,
            &raw mut (*E).base.y_subpos,
            0x28000 as uint32,
        );
        if (*E).base.y_pos as ::core::ffi::c_int - 1119 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            (*E).base.y_pos = 1119 as uint16;
            (*v10).base.y_pos = 1159 as uint16;
            (*v8).base.y_pos = ((*E).base.y_pos).wrapping_sub(1u16);
            (*E).gtp_var_F = ((0xa2a8d0i32) as u16) & (0xffffu16);
            (*E).gtp_var_E = 0 as uint16;
        }
    };
}
pub unsafe fn GunshipTop_4(mut k: uint16, game: &mut Game) {
    let mut v2: ::core::ffi::c_int = 0;
    let mut v5: uint16 = 0;
    let mut GunshipTop: *mut Enemy_GunshipTop = Get_GunshipTop(k as ::core::ffi::c_int, game);
    v2 = (*GunshipTop).gtp_var_E as ::core::ffi::c_int;
    let ref mut fresh9 = *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16);
    *fresh9 = (*fresh9).wrapping_add(
        *(game.g_rom.offset(crate::rtl::lorom_offset(0xa2a622)) as *const uint8 as *mut uint16)
            .offset(v2 as isize),
    );
    let mut v3: *mut Enemy_GunshipTop =
        Get_GunshipTop(k as ::core::ffi::c_int + 128 as ::core::ffi::c_int, game);
    (*v3).base.y_pos = ((*v3).base.y_pos).wrapping_add(
        *(game.g_rom.offset(crate::rtl::lorom_offset(0xa2a622)) as *const uint8 as *mut uint16)
            .offset(v2 as isize),
    );
    let mut v4: *mut Enemy_GunshipTop =
        Get_GunshipTop(k as ::core::ffi::c_int + 64 as ::core::ffi::c_int, game);
    (*v4).base.y_pos = ((*v4).base.y_pos).wrapping_add(
        *(game.g_rom.offset(crate::rtl::lorom_offset(0xa2a622)) as *const uint8 as *mut uint16)
            .offset(v2 as isize),
    );
    (*GunshipTop).base.y_pos = ((*GunshipTop).base.y_pos).wrapping_add(
        *(game.g_rom.offset(crate::rtl::lorom_offset(0xa2a622)) as *const uint8 as *mut uint16)
            .offset(v2 as isize),
    );
    v5 = ((*GunshipTop).gtp_var_E).wrapping_add(1u16);
    (*GunshipTop).gtp_var_E = v5;
    if v5 as ::core::ffi::c_int - 17 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
        (*GunshipTop).gtp_var_F = ((0xa2a942i32) as u16) & (0xffffu16);
        (*GunshipTop).gtp_var_E = (*GunshipTop).base.y_pos;
        (*GunshipTop).gtp_var_D = 1 as uint16;
        let mut v6: *mut Enemy_GunshipTop = Get_GunshipTop(0 as ::core::ffi::c_int, game);
        (*v6).gtp_var_C = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) =
            ((*GunshipTop).base.x_pos).wrapping_add(1u16);
        *((&raw mut game.g_ram as *mut uint8).offset(0xb10) as *mut uint16) =
            *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16);
        let mut v7: *mut Enemy_GunshipTop =
            Get_GunshipTop(k as ::core::ffi::c_int + 128 as ::core::ffi::c_int, game);
        (*v7).base.instruction_timer = 1 as uint16;
        (*v7).base.current_instruction = addr_kGunshipTop_Ilist_A5BE as uint16;
        (*v6).gtp_var_A = 144 as uint16;
        QueueSfx3_Max6(0x14 as uint16, game);
    }
}
pub unsafe fn GunshipTop_5(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_GunshipTop = Get_GunshipTop(0 as ::core::ffi::c_int, game);
    let mut v2: bool = (*E).gtp_var_A == 1u16;
    (*E).gtp_var_A = (*E).gtp_var_A.wrapping_sub(1);
    let mut v3: bool =
        ((*E).gtp_var_A as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32;
    if v2 as ::core::ffi::c_int != 0 || v3 as ::core::ffi::c_int != 0 {
        (*Get_GunshipTop(k as ::core::ffi::c_int, game)).gtp_var_F =
            ((0xa2a950i32) as u16) & (0xffffu16);
    }
}
pub unsafe fn GunshipTop_6(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_GunshipTop = Get_GunshipTop(k as ::core::ffi::c_int, game);
    let mut r18: uint16 = ((*E).gtp_var_E).wrapping_sub(30u16);
    let ref mut fresh10 = *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16);
    *fresh10 = (*fresh10).wrapping_sub(1);
    if *fresh10 as ::core::ffi::c_int - r18 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0
    {
        (*E).gtp_var_F = ((0xa2a987i32) as u16) & (0xffffu16);
        let mut E2: *mut Enemy_GunshipTop =
            Get_GunshipTop(k as ::core::ffi::c_int + 128 as ::core::ffi::c_int, game);
        (*E2).base.instruction_timer = 1 as uint16;
        (*E2).base.current_instruction = addr_kGunshipTop_Ilist_A5EE as uint16;
        (*Get_GunshipTop(0 as ::core::ffi::c_int, game)).gtp_var_A = 144 as uint16;
        QueueSfx3_Max6(0x15 as uint16, game);
    }
}
pub unsafe fn GunshipTop_7(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_GunshipTop = Get_GunshipTop(0 as ::core::ffi::c_int, game);
    let mut v2: bool = (*E).gtp_var_A == 1u16;
    (*E).gtp_var_A = (*E).gtp_var_A.wrapping_sub(1);
    let mut v3: bool =
        ((*E).gtp_var_A as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32;
    if v2 as ::core::ffi::c_int != 0 || v3 as ::core::ffi::c_int != 0 {
        (*Get_GunshipTop(k as ::core::ffi::c_int, game)).gtp_var_F =
            ((0xa2a9bdi32) as u16) & (0xffffu16);
        *((&raw mut game.g_ram as *mut uint8).offset(0xa42) as *mut uint16) =
            ((0x90e695i32) as u16) & (0xffffu16);
        *((&raw mut game.g_ram as *mut uint8).offset(0xa44) as *mut uint16) =
            ((0x90e725i32) as u16) & (0xffffu16);
        *((&raw mut game.g_ram as *mut uint8).offset(0xd914) as *mut uint16) =
            kLoadingGameState_5_Main as uint16;
        let ref mut fresh11 = *((&raw mut game.g_ram as *mut uint8).offset(0xd8f8) as *mut uint16);
        *fresh11 = (*fresh11) | (1u16);
        *((&raw mut game.g_ram as *mut uint8).offset(0x78b) as *mut uint16) = 0 as uint16;
        SaveToSram(
            *((&raw mut game.g_ram as *mut uint8).offset(0x952) as *mut uint16),
            game,
        );
    }
}
pub unsafe fn GunshipTop_8(mut k: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16) == 8u16
        && *((&raw mut game.g_ram as *mut uint8).offset(0xa42) as *mut uint16) as ::core::ffi::c_int
            == 0x90e695 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
    {
        let mut E: *mut Enemy_GunshipTop = Get_GunshipTop(k as ::core::ffi::c_int, game);
        if (((*E).base.x_pos as ::core::ffi::c_int
            - 8 as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
                as ::core::ffi::c_int) as int16)
            < 0i16
            && (((*E).base.x_pos as ::core::ffi::c_int + 8 as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
                    as ::core::ffi::c_int) as int16)
                >= 0i16
            && (((*E).base.y_pos as ::core::ffi::c_int
                - 64 as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
                    as ::core::ffi::c_int) as int16)
                < 0i16
            && (((*E).base.y_pos) as i16).wrapping_sub(
                (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)) as i16,
            ) >= 0i16
            && *(&raw mut game.g_ram as *mut uint8).offset(0xa1f) == 0
            && (*((&raw mut game.g_ram as *mut uint8).offset(0x8f) as *mut uint16)
                as ::core::ffi::c_int
                & kButton_Down as ::core::ffi::c_int)
                != 0i32
        {
            (*E).gtp_var_F = ((0xa2aa4fi32) as u16) & (0xffffu16);
            if *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) != 1152u16 {
                *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) =
                    (*E).base.x_pos;
                *((&raw mut game.g_ram as *mut uint8).offset(0xb10) as *mut uint16) =
                    *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16);
            }
            MakeSamusFaceForward(game);
            CallSomeSamusCode(0x1a as uint16, game);
            *((&raw mut game.g_ram as *mut uint8).offset(0xe18) as *mut uint16) = 0 as uint16;
            let mut v3: uint16 = ((*E).base.y_pos).wrapping_sub(1u16);
            let mut E2: *mut Enemy_GunshipTop =
                Get_GunshipTop(k as ::core::ffi::c_int + 128 as ::core::ffi::c_int, game);
            (*E2).base.y_pos = v3;
            (*E2).base.instruction_timer = 1 as uint16;
            (*E2).base.current_instruction = addr_kGunshipTop_Ilist_A5BE as uint16;
            (*Get_GunshipTop(0 as ::core::ffi::c_int, game)).gtp_var_A = 144 as uint16;
            QueueSfx3_Max6(0x14 as uint16, game);
        }
    }
}
pub unsafe fn GunshipTop_9(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_GunshipTop = Get_GunshipTop(0 as ::core::ffi::c_int, game);
    let mut v2: bool = (*E).gtp_var_A == 1u16;
    (*E).gtp_var_A = (*E).gtp_var_A.wrapping_sub(1);
    let mut v3: bool =
        ((*E).gtp_var_A as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32;
    if v2 as ::core::ffi::c_int != 0 || v3 as ::core::ffi::c_int != 0 {
        (*Get_GunshipTop(k as ::core::ffi::c_int, game)).gtp_var_F =
            ((0xa2aa5di32) as u16) & (0xffffu16);
    }
}
pub unsafe fn GunshipTop_10(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_GunshipTop = Get_GunshipTop(k as ::core::ffi::c_int, game);
    let mut r18: uint16 = ((*E).gtp_var_E).wrapping_add(18u16);
    let ref mut fresh2 = *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16);
    *fresh2 = (*fresh2).wrapping_add(2u16);
    if *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16) as ::core::ffi::c_int
        - r18 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        (*E).gtp_var_F = ((0xa2aa94i32) as u16) & (0xffffu16);
        let mut E2: *mut Enemy_GunshipTop =
            Get_GunshipTop(k as ::core::ffi::c_int + 128 as ::core::ffi::c_int, game);
        (*E2).base.instruction_timer = 1 as uint16;
        (*E2).base.current_instruction = addr_kGunshipTop_Ilist_A5EE as uint16;
        (*Get_GunshipTop(0 as ::core::ffi::c_int, game)).gtp_var_A = 144 as uint16;
        QueueSfx3_Max6(0x15 as uint16, game);
    }
}
pub unsafe fn GunshipTop_11(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_GunshipTop = Get_GunshipTop(0 as ::core::ffi::c_int, game);
    let mut v2: bool = (*E).gtp_var_A == 1u16;
    (*E).gtp_var_A = (*E).gtp_var_A.wrapping_sub(1);
    let mut v3: bool =
        ((*E).gtp_var_A as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32;
    if v2 as ::core::ffi::c_int != 0 || v3 as ::core::ffi::c_int != 0 {
        (*Get_GunshipTop(k as ::core::ffi::c_int, game)).gtp_var_F =
            ((0xa2aaa2i32) as u16) & (0xffffu16);
    }
}
pub unsafe fn GunshipTop_12(mut k: uint16, game: &mut Game) {
    if CheckEventHappened(0xe as uint16, game) != 0 {
        (*Get_GunshipTop(k as ::core::ffi::c_int, game)).gtp_var_F =
            ((0xa2abc7i32) as u16) & (0xffffu16);
        let mut E: *mut Enemy_GunshipTop =
            Get_GunshipTop(k as ::core::ffi::c_int + 64 as ::core::ffi::c_int, game);
        (*E).gtp_var_F = 0 as uint16;
        (*E).gtp_var_E = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xdec) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xdee) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xdf0) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xdf2) as *mut uint16) = 0 as uint16;
        CallSomeSamusCode(0xa as uint16, game);
    } else {
        Samus_RestoreHealth(2 as uint16, game);
        Samus_RestoreMissiles(2 as uint16, game);
        Samus_RestoreSuperMissiles(2 as uint16, game);
        Samus_RestorePowerBombs(2 as uint16, game);
        if ((*((&raw mut game.g_ram as *mut uint8).offset(0x9d6) as *mut uint16)) as i16)
            .wrapping_sub(
                (*((&raw mut game.g_ram as *mut uint8).offset(0x9d4) as *mut uint16)) as i16,
            )
            >= 0i16
            && ((*((&raw mut game.g_ram as *mut uint8).offset(0x9c2) as *mut uint16)) as i16)
                .wrapping_sub(
                    (*((&raw mut game.g_ram as *mut uint8).offset(0x9c4) as *mut uint16)) as i16,
                )
                >= 0i16
            && ((*((&raw mut game.g_ram as *mut uint8).offset(0x9c6) as *mut uint16)) as i16)
                .wrapping_sub(
                    (*((&raw mut game.g_ram as *mut uint8).offset(0x9c8) as *mut uint16)) as i16,
                )
                >= 0i16
            && ((*((&raw mut game.g_ram as *mut uint8).offset(0x9ca) as *mut uint16)) as i16)
                .wrapping_sub(
                    (*((&raw mut game.g_ram as *mut uint8).offset(0x9cc) as *mut uint16)) as i16,
                )
                >= 0i16
            && ((*((&raw mut game.g_ram as *mut uint8).offset(0x9ce) as *mut uint16)) as i16)
                .wrapping_sub(
                    (*((&raw mut game.g_ram as *mut uint8).offset(0x9d0) as *mut uint16)) as i16,
                )
                >= 0i16
        {
            (*Get_GunshipTop(k as ::core::ffi::c_int, game)).gtp_var_F =
                ((0xa2ab1fi32) as u16) & (0xffffu16);
        }
    };
}
pub unsafe fn GunshipTop_13(mut k: uint16, game: &mut Game) {
    let mut rv: ::core::ffi::c_int = DisplayMessageBox_Poll(0x1c as uint16, game);
    if rv < 0i32 {
        return;
    }
    if rv != 2i32 {
        let ref mut fresh3 = *((&raw mut game.g_ram as *mut uint8).offset(0xd8f8) as *mut uint16);
        *fresh3 = (*fresh3) | (1u16);
        *((&raw mut game.g_ram as *mut uint8).offset(0x78b) as *mut uint16) = 0 as uint16;
        SaveToSram(
            *((&raw mut game.g_ram as *mut uint8).offset(0x952) as *mut uint16),
            game,
        );
    }
    (*Get_GunshipTop(k as ::core::ffi::c_int, game)).gtp_var_F =
        ((0xa2ab60i32) as u16) & (0xffffu16);
    let mut E: *mut Enemy_GunshipTop =
        Get_GunshipTop(k as ::core::ffi::c_int + 128 as ::core::ffi::c_int, game);
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.current_instruction = addr_kGunshipTop_Ilist_A5BE as uint16;
    (*Get_GunshipTop(0 as ::core::ffi::c_int, game)).gtp_var_A = 144 as uint16;
    QueueSfx3_Max6(0x14 as uint16, game);
}
pub unsafe fn GunshipTop_14(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_GunshipTop = Get_GunshipTop(0 as ::core::ffi::c_int, game);
    let mut v2: bool = (*E).gtp_var_A == 1u16;
    (*E).gtp_var_A = (*E).gtp_var_A.wrapping_sub(1);
    let mut v3: bool =
        ((*E).gtp_var_A as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32;
    if v2 as ::core::ffi::c_int != 0 || v3 as ::core::ffi::c_int != 0 {
        (*Get_GunshipTop(k as ::core::ffi::c_int, game)).gtp_var_F =
            ((0xa2ab6ei32) as u16) & (0xffffu16);
    }
}
pub unsafe fn GunshipTop_15(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_GunshipTop = Get_GunshipTop(k as ::core::ffi::c_int, game);
    let mut r18: uint16 = ((*E).gtp_var_E).wrapping_sub(30u16);
    let ref mut fresh4 = *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16);
    *fresh4 = (*fresh4).wrapping_sub(2u16);
    if *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16) as ::core::ffi::c_int
        - r18 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        (*E).gtp_var_F = ((0xa2aba5i32) as u16) & (0xffffu16);
        let mut E2: *mut Enemy_GunshipTop =
            Get_GunshipTop(k as ::core::ffi::c_int + 128 as ::core::ffi::c_int, game);
        (*E2).base.instruction_timer = 1 as uint16;
        (*E2).base.current_instruction = addr_kGunshipTop_Ilist_A5EE as uint16;
        (*Get_GunshipTop(0 as ::core::ffi::c_int, game)).gtp_var_A = 144 as uint16;
        QueueSfx3_Max6(0x15 as uint16, game);
    }
}
pub unsafe fn GunshipTop_16(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_GunshipTop = Get_GunshipTop(0 as ::core::ffi::c_int, game);
    let mut v2: bool = (*E).gtp_var_A == 1u16;
    (*E).gtp_var_A = (*E).gtp_var_A.wrapping_sub(1);
    let mut v3: bool =
        ((*E).gtp_var_A as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32;
    if v2 as ::core::ffi::c_int != 0 || v3 as ::core::ffi::c_int != 0 {
        (*Get_GunshipTop(k as ::core::ffi::c_int, game)).gtp_var_F =
            ((0xa2a9bdi32) as u16) & (0xffffu16);
        if *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16) as ::core::ffi::c_int
            - kGameState_40_TransitionToDemo as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xa42) as *mut uint16) =
                ((0x90e695i32) as u16) & (0xffffu16);
            *((&raw mut game.g_ram as *mut uint8).offset(0xa44) as *mut uint16) =
                ((0x90e725i32) as u16) & (0xffffu16);
        }
    }
}
pub unsafe fn GunshipTop_17(mut k: uint16, game: &mut Game) {
    let mut v1: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0xdec) as *mut uint16);
    let mut v2: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16);
    (*gVramWriteEntry(
        *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .size = 1024 as uint16;
    v2 = (v2).wrapping_add(2u16);
    let mut v3: ::core::ffi::c_int = v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    (*gVramWriteEntry(v2 as ::core::ffi::c_int, game)).size =
        *(game.g_rom.offset(crate::rtl::lorom_offset(0xa2ac07)) as *const uint8 as *mut uint16)
            .offset(v3 as isize);
    v2 = (v2).wrapping_add(2u16);
    let fresh5 = v2;
    v2 = v2.wrapping_add(1);
    *(&raw mut (*(gVramWriteEntry
        as unsafe fn(::core::ffi::c_int, &mut Game) -> *mut VramWriteEntry)(
        fresh5 as ::core::ffi::c_int,
        game,
    ))
    .size as *mut uint8)
        .offset(0) = -(108 as ::core::ffi::c_int) as uint8;
    (*gVramWriteEntry(v2 as ::core::ffi::c_int, game)).size =
        *(game.g_rom.offset(crate::rtl::lorom_offset(0xa2ac11)) as *const uint8 as *mut uint16)
            .offset(v3 as isize);
    *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) = (v2).wrapping_add(2u16);
    let ref mut fresh6 = *((&raw mut game.g_ram as *mut uint8).offset(0xdec) as *mut uint16);
    *fresh6 = (*fresh6).wrapping_add(2u16);
    if *((&raw mut game.g_ram as *mut uint8).offset(0xdec) as *mut uint16) as ::core::ffi::c_int
        - 10 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        (*Get_GunshipTop(k as ::core::ffi::c_int, game)).gtp_var_F =
            ((0xa2ac1bi32) as u16) & (0xffffu16);
        *((&raw mut game.g_ram as *mut uint8).offset(0xdec) as *mut uint16) = 0 as uint16;
    }
}
pub unsafe fn GunshipTop_18(mut k: uint16, game: &mut Game) {
    let mut v5: int16 = 0;
    let mut E1: *mut Enemy_GunshipTop =
        Get_GunshipTop(k as ::core::ffi::c_int + 64 as ::core::ffi::c_int, game);
    let mut gtp_var_E: uint16 = (*E1).gtp_var_E;
    if gtp_var_E as ::core::ffi::c_int - 64 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v3: uint16 = 0;
        if (gtp_var_E as ::core::ffi::c_int & 1 as ::core::ffi::c_int) != 0i32 {
            v3 = (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16))
                .wrapping_add(1u16);
        } else {
            v3 = (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16))
                .wrapping_sub(1u16);
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16) = v3;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb14) as *mut uint16) = v3;
    } else {
        let mut v4: uint16 = 0;
        if (gtp_var_E as ::core::ffi::c_int & 1 as ::core::ffi::c_int) != 0i32 {
            v4 = (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16))
                .wrapping_add(2u16);
        } else {
            v4 = (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16))
                .wrapping_sub(2u16);
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16) = v4;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb14) as *mut uint16) = v4;
    }
    v5 = ((*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)) as i16)
        .wrapping_sub(17i16);
    let mut E0: *mut Enemy_GunshipTop = Get_GunshipTop(k as ::core::ffi::c_int, game);
    (*E0).base.y_pos =
        (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)).wrapping_sub(17u16);
    (*Get_GunshipTop(k as ::core::ffi::c_int + 128 as ::core::ffi::c_int, game))
        .base
        .y_pos = ((v5) as u16).wrapping_sub(1u16);
    (*E1).base.y_pos =
        (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)).wrapping_add(23u16);
    let mut v7: uint16 = ((*E1).gtp_var_E).wrapping_add(1u16);
    (*E1).gtp_var_E = v7;
    if v7 as ::core::ffi::c_int - 128 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        if v7 == 64u16 {
            SpawnEprojWithRoomGfx(addr_stru_86A379 as uint16, 0 as uint16, game);
            SpawnEprojWithRoomGfx(addr_stru_86A379 as uint16, 2 as uint16, game);
            SpawnEprojWithRoomGfx(addr_stru_86A379 as uint16, 4 as uint16, game);
            SpawnEprojWithRoomGfx(addr_stru_86A379 as uint16, 6 as uint16, game);
            SpawnEprojWithRoomGfx(addr_stru_86A379 as uint16, 8 as uint16, game);
            SpawnEprojWithRoomGfx(addr_stru_86A379 as uint16, 0xa as uint16, game);
        }
    } else {
        (*E0).gtp_var_F = ((0xa2acd7i32) as u16) & (0xffffu16);
        (*Get_GunshipTop(0 as ::core::ffi::c_int, game)).gtp_var_A = 0 as uint16;
    };
}
pub unsafe fn GunshipTop_19(mut k: uint16, game: &mut Game) {
    let mut v1: int16 = 0;
    let ref mut fresh7 = *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16);
    *fresh7 = (*fresh7).wrapping_sub(2u16);
    v1 = ((*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)) as i16)
        .wrapping_sub(17i16);
    let mut E0: *mut Enemy_GunshipTop = Get_GunshipTop(k as ::core::ffi::c_int, game);
    (*E0).base.y_pos =
        (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)).wrapping_sub(17u16);
    (*Get_GunshipTop(k as ::core::ffi::c_int + 128 as ::core::ffi::c_int, game))
        .base
        .y_pos = ((v1) as u16).wrapping_sub(1u16);
    let mut E1: *mut Enemy_GunshipTop =
        Get_GunshipTop(k as ::core::ffi::c_int + 64 as ::core::ffi::c_int, game);
    (*E1).base.y_pos =
        (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)).wrapping_add(23u16);
    if (*E0).base.y_pos as ::core::ffi::c_int - 896 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        (*E0).gtp_var_F = ((0xa2ad0ei32) as u16) & (0xffffu16);
        (*E1).gtp_var_F = 512 as uint16;
    }
}
pub unsafe fn GunshipTop_20(mut k: uint16, game: &mut Game) {
    GunshipTop_21(k, game);
    let mut E: *mut Enemy_GunshipTop = Get_GunshipTop(k as ::core::ffi::c_int, game);
    if (*E).base.y_pos as ::core::ffi::c_int - 256 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        (*E).gtp_var_F = ((0xa2ad2di32) as u16) & (0xffffu16);
        *((&raw mut game.g_ram as *mut uint8).offset(0x998) as *mut uint16) = 38 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x723) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x725) as *mut uint16) = 0 as uint16;
    }
}
pub unsafe fn GunshipTop_21(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_GunshipTop =
        Get_GunshipTop(k as ::core::ffi::c_int + 64 as ::core::ffi::c_int, game);
    let mut v3: uint16 = ((*E).gtp_var_F).wrapping_add(64u16);
    (*E).gtp_var_F = v3;
    if (v3 as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int) - 2560 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        (*E).gtp_var_F = 2304 as uint16;
    }
    let mut v6: uint32 = ((*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)
        as uint32)
        << 16 as ::core::ffi::c_int
        | *((&raw mut game.g_ram as *mut uint8).offset(0xafc) as *mut uint16) as uint32)
        .wrapping_sub(
            (((*E).gtp_var_F as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as uint32)
                << 16 as ::core::ffi::c_int
                | (((*E).gtp_var_F as ::core::ffi::c_int) << 8 as ::core::ffi::c_int) as uint16
                    as uint32,
        );
    *((&raw mut game.g_ram as *mut uint8).offset(0xafc) as *mut uint16) = v6 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16) =
        (v6 >> 16 as ::core::ffi::c_int) as uint16;
    let mut v5: uint16 =
        (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)).wrapping_sub(17u16);
    (*Get_GunshipTop(k as ::core::ffi::c_int, game)).base.y_pos = v5;
    (*Get_GunshipTop(k as ::core::ffi::c_int + 128 as ::core::ffi::c_int, game))
        .base
        .y_pos = (v5).wrapping_sub(1u16);
    (*E).base.y_pos =
        (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)).wrapping_add(23u16);
}
pub unsafe fn Flies_Init(game: &mut Game) {
    let mut E: *mut Enemy_Flies = Get_Flies(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).flies_var_E = 0 as uint16;
    (*E).flies_var_F = ((0xa2b14ei32) as u16) & (0xffffu16);
    (*E).base.current_instruction = addr_kFlies_Ilist_B013 as uint16;
    (*E).base.spritemap_pointer = addr_kSpritemap_Nothing_A2 as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.spritemap_pointer = addr_kSpritemap_Nothing_A2 as uint16;
}
pub unsafe fn CarryAdd16(mut rv: *mut uint16, mut v: uint16) -> bool {
    let mut i: ::core::ffi::c_int = *rv as ::core::ffi::c_int + v as ::core::ffi::c_int;
    *rv = i as uint16;
    return i >> 16 as ::core::ffi::c_int != 0;
}
pub unsafe fn Flies_1(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Flies = Get_Flies(k as ::core::ffi::c_int, game);
    let mut v2: ::core::ffi::c_int =
        (*E).flies_var_E as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut carry: bool = CarryAdd16(
        &raw mut (*E).base.x_subpos,
        ((*((&raw const kSinCosTable8bit_Sext as *const int16)
            .offset((v2 + 64 as ::core::ffi::c_int) as isize) as *const int16
            as *mut uint8)
            .offset(0) as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint16,
    );
    (*E).base.x_pos = ((*E).base.x_pos as ::core::ffi::c_int
        + (*((&raw const kSinCosTable8bit_Sext as *const int16)
            .offset((v2 + 64 as ::core::ffi::c_int) as isize) as *const int16
            as *mut uint8)
            .offset(
                (::core::mem::size_of::<int16>())
                    .wrapping_div(::core::mem::size_of::<uint8>())
                    .wrapping_sub(1 as usize) as isize,
            ) as int8 as ::core::ffi::c_int
            + carry as ::core::ffi::c_int)) as uint16;
    carry = CarryAdd16(
        &raw mut (*E).base.y_subpos,
        ((*((&raw const kSinCosTable8bit_Sext as *const int16).offset(v2 as isize) as *const int16
            as *mut uint8)
            .offset(0) as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint16,
    );
    (*E).base.y_pos = ((*E).base.y_pos as ::core::ffi::c_int
        + (*((&raw const kSinCosTable8bit_Sext as *const int16).offset(v2 as isize) as *const int16
            as *mut uint8)
            .offset(
                (::core::mem::size_of::<int16>())
                    .wrapping_div(::core::mem::size_of::<uint8>())
                    .wrapping_sub(1 as usize) as isize,
            ) as int8 as ::core::ffi::c_int
            + carry as ::core::ffi::c_int)) as uint16;
}
pub unsafe fn Flies_2(mut k: uint16, game: &mut Game) -> uint16 {
    let mut E: *mut Enemy_Flies = Get_Flies(k as ::core::ffi::c_int, game);
    let mut carry: bool = CarryAdd16(
        &raw mut (*E).base.x_subpos,
        ((*(&raw mut (*E).flies_var_B as *mut uint8).offset(0) as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint16,
    );
    (*E).base.x_pos = ((*E).base.x_pos as ::core::ffi::c_int
        + (*(&raw mut (*E).flies_var_B as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1 as usize) as isize,
        ) as int8 as ::core::ffi::c_int
            + carry as ::core::ffi::c_int)) as uint16;
    carry = CarryAdd16(
        &raw mut (*E).base.y_subpos,
        ((*(&raw mut (*E).flies_var_C as *mut uint8).offset(0) as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint16,
    );
    let mut result: uint16 = ((*E).base.y_pos as ::core::ffi::c_int
        + *(&raw mut (*E).flies_var_C as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1 as usize) as isize,
        ) as int8 as ::core::ffi::c_int
        + carry as ::core::ffi::c_int) as uint16;
    (*E).base.y_pos = result;
    return result;
}
pub unsafe fn Flies_Main(game: &mut Game) {
    NextRandom(game);
    let mut E: *mut Enemy_Flies = Get_Flies(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    CallEnemyPreInstr((((*E).flies_var_F) as u32) | (0xa20000u32), game);
}
pub unsafe fn Flies_3(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int = CalculateAngleOfSamusFromEnemy(k, game) as ::core::ffi::c_int;
    let mut E: *mut Enemy_Flies = Get_Flies(k as ::core::ffi::c_int, game);
    (*E).flies_var_B = (2u16)
        .wrapping_mul((kSinCosTable8bit_Sext[(v1 + 64 as ::core::ffi::c_int) as usize]) as u16);
    (*E).flies_var_C = (4u16).wrapping_mul((kSinCosTable8bit_Sext[v1 as usize]) as u16);
    (*E).flies_var_D = *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16);
    (*E).flies_var_F = ((0xa2b1aai32) as u16) & (0xffffu16);
}
pub unsafe fn Flies_4(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Flies = Get_Flies(k as ::core::ffi::c_int, game);
    let mut flies_var_A: uint16 = (*E).flies_var_A;
    if flies_var_A != 0 {
        (*E).flies_var_A = (flies_var_A).wrapping_sub(1u16);
    } else if CompareDistToSamus_X(k, 0x70 as uint16, game) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        == 0
    {
        Flies_3(k, game);
        return;
    }
    Flies_1(k, game);
    let mut v3: bool = ((*E).flies_var_E as ::core::ffi::c_int + 32 as ::core::ffi::c_int
        & 0x1ff as ::core::ffi::c_int)
        == 0i32;
    (*E).flies_var_E = ((*E).flies_var_E as ::core::ffi::c_int + 32 as ::core::ffi::c_int
        & 0x1ff as ::core::ffi::c_int) as uint16;
    if v3 {
        (*E).flies_var_F = ((0xa2b17ci32) as u16) & (0xffffu16);
    }
}
pub unsafe fn Flies_5(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Flies = Get_Flies(k as ::core::ffi::c_int, game);
    let mut flies_var_A: uint16 = (*E).flies_var_A;
    if flies_var_A != 0 {
        (*E).flies_var_A = (flies_var_A).wrapping_sub(1u16);
    } else if CompareDistToSamus_X(k, 0x70 as uint16, game) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        == 0
    {
        Flies_3(k, game);
        return;
    }
    Flies_1(k, game);
    let mut v3: bool = ((*E).flies_var_E as ::core::ffi::c_int - 32 as ::core::ffi::c_int
        & 0x1ff as ::core::ffi::c_int)
        == 0i32;
    (*E).flies_var_E = ((*E).flies_var_E as ::core::ffi::c_int - 32 as ::core::ffi::c_int
        & 0x1ff as ::core::ffi::c_int) as uint16;
    if v3 {
        (*E).flies_var_F = ((0xa2b14ei32) as u16) & (0xffffu16);
    }
}
pub unsafe fn Flies_6(mut k: uint16, game: &mut Game) {
    let mut current_block: u64;
    let mut v1: uint16 = Flies_2(k, game);
    let mut E: *mut Enemy_Flies = Get_Flies(k as ::core::ffi::c_int, game);
    (*E).flies_var_A = (*E).flies_var_A.wrapping_add(1);
    if ((*E).flies_var_C as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) == 0i32 {
        if v1 < (*E).flies_var_D {
            return;
        }
        current_block = 6833738745606017744;
    } else if v1 < (*E).flies_var_D {
        current_block = 6833738745606017744;
    } else {
        current_block = 10879442775620481940;
    }
    match current_block {
        6833738745606017744 => {
            (*E).flies_var_C = -((*E).flies_var_C as ::core::ffi::c_int) as uint16;
            (*E).flies_var_F = ((0xa2b1d2i32) as u16) & (0xffffu16);
        }
        _ => {}
    };
}
pub unsafe fn Flies_7(mut k: uint16, game: &mut Game) {
    Flies_2(k, game);
    let mut E: *mut Enemy_Flies = Get_Flies(k as ::core::ffi::c_int, game);
    (*E).flies_var_A = (*E).flies_var_A.wrapping_sub(1);
    if ((*E).flies_var_A as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        (*E).flies_var_A = 24 as uint16;
        (*E).flies_var_F = ((0xa2b14ei32) as u16) & (0xffffu16);
    }
}
pub unsafe fn NorfairErraticFireball_Init(game: &mut Game) {
    let mut E: *mut Enemy_NorfairErraticFireball = Get_NorfairErraticFireball(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut pt: Point32 = ConvertAngleToXy(
        (*E).nefl_parameter_1,
        *(&raw mut (*E).nefl_parameter_2 as *mut uint8).offset(0) as uint16,
        game,
    );
    (*E).nefl_var_C = (pt.x >> 16 as ::core::ffi::c_int) as uint16;
    (*E).nefl_var_D = pt.x as uint16;
    (*E).nefl_var_E = (pt.y >> 16 as ::core::ffi::c_int) as uint16;
    (*E).nefl_var_F = pt.y as uint16;
    (*E).base.current_instruction = addr_kNorfairErraticFireball_Ilist_B2DC as uint16;
}
pub unsafe fn NorfairErraticFireball_Main(game: &mut Game) {
    let mut E: *mut Enemy_NorfairErraticFireball = Get_NorfairErraticFireball(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut pt: Point32 = ConvertAngleToXy(
        (*E).nefl_parameter_1,
        *(&raw mut (*E).nefl_parameter_2 as *mut uint8).offset(0) as uint16,
        game,
    );
    (*E).nefl_var_C = (pt.x >> 16 as ::core::ffi::c_int) as uint16;
    (*E).nefl_var_D = pt.x as uint16;
    (*E).nefl_var_E = (pt.y >> 16 as ::core::ffi::c_int) as uint16;
    (*E).nefl_var_F = pt.y as uint16;
    if ((*E).nefl_parameter_1 as ::core::ffi::c_int + 64 as ::core::ffi::c_int
        & 0x80 as ::core::ffi::c_int)
        == 0i32
    {
        (*E).nefl_var_C = !((*E).nefl_var_C as ::core::ffi::c_int) as uint16;
        (*E).nefl_var_D = -((*E).nefl_var_D as ::core::ffi::c_int) as uint16;
    }
    if Enemy_MoveRight_IgnoreSlopes(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        (((*E).nefl_var_C as uint32) << 16 as ::core::ffi::c_int | (*E).nefl_var_D as uint32)
            as int32,
        game,
    ) != 0
    {
        (*E).nefl_parameter_1 = ((*E).nefl_parameter_1) ^ (0x40u16);
    } else {
        if ((*E).nefl_parameter_1 as ::core::ffi::c_int + 128 as ::core::ffi::c_int
            & 0x80 as ::core::ffi::c_int)
            == 0i32
        {
            (*E).nefl_var_E = !((*E).nefl_var_E as ::core::ffi::c_int) as uint16;
            (*E).nefl_var_F = -((*E).nefl_var_F as ::core::ffi::c_int) as uint16;
        }
        if Enemy_MoveDown(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            (((*E).nefl_var_E as uint32) << 16 as ::core::ffi::c_int | (*E).nefl_var_F as uint32)
                as int32,
            game,
        ) != 0
        {
            (*E).nefl_parameter_1 = ((*E).nefl_parameter_1) ^ (0xc0u16);
        }
    };
}
pub unsafe fn LavaquakeRocks_Init(game: &mut Game) {
    let mut E: *mut Enemy_LavaquakeRocks = Get_LavaquakeRocks(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.current_instruction = addr_kNorfairErraticFireball_Ilist_B51A as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
    (*E).lrs_var_A = ((0xa2b596i32) as u16) & (0xffffu16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16) = 17 as uint16;
}
pub unsafe fn CallLavaQuakeRocksFunc(mut ea: uint32, game: &mut Game) {
    match ea {
        10663318 => {
            LavaquakeRocks_1(game);
            return;
        }
        10663346 => {
            LavaquakeRocks_2(game);
            return;
        }
        10663402 => {
            LavaquakeRocks_3(game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn LavaquakeRocks_Main(game: &mut Game) {
    let mut E: *mut Enemy_LavaquakeRocks = Get_LavaquakeRocks(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    CallLavaQuakeRocksFunc((((*E).lrs_var_A) as u32) | (0xa20000u32), game);
}
pub unsafe fn LavaquakeRocks_1(game: &mut Game) {
    if IsSamusWithinEnemy_X(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        0x40 as uint16,
        game,
    ) != 0
    {
        if IsSamusWithinEnemy_Y(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            0x40 as uint16,
            game,
        ) != 0
        {
            (*Get_LavaquakeRocks(
                *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                    as ::core::ffi::c_int,
                game,
            ))
            .lrs_var_A = ((0xa2b5b2i32) as u16) & (0xffffu16);
        }
    }
}
pub unsafe fn LavaquakeRocks_2(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1995) as *mut uint16) =
        *(game.g_rom.offset(crate::rtl::lorom_offset(0xa2b550)) as *const uint8 as *mut uint16)
            .offset(
                ((NextRandom(game) as ::core::ffi::c_int & 0x1e as ::core::ffi::c_int)
                    >> 1 as ::core::ffi::c_int) as isize,
            );
    let mut v1: ::core::ffi::c_int = (NextRandom(game) as ::core::ffi::c_int
        & 0x1e as ::core::ffi::c_int)
        >> 1 as ::core::ffi::c_int;
    SpawnEprojWithGfx(
        *(game.g_rom.offset(crate::rtl::lorom_offset(0xa2b530)) as *const uint8 as *mut uint16)
            .offset(v1 as isize),
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        addr_stru_86BD5A as uint16,
        game,
    );
    let mut E: *mut Enemy_LavaquakeRocks = Get_LavaquakeRocks(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).lrs_var_A = ((0xa2b5eai32) as u16) & (0xffffu16);
    (*E).lrs_var_B = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa2b520)) as *const uint8
        as *mut uint16)
        .offset(
            (((NextRandom(game)) & (0xeu16)) as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
                as isize,
        );
}
pub unsafe fn LavaquakeRocks_3(game: &mut Game) {
    let mut E: *mut Enemy_LavaquakeRocks = Get_LavaquakeRocks(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).lrs_var_B = (*E).lrs_var_B.wrapping_sub(1);
    if ((*E).lrs_var_B as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        (*E).lrs_var_A = ((0xa2b596i32) as u16) & (0xffffu16);
    }
}
pub unsafe fn Rinka_Init(game: &mut Game) {
    let mut E: *mut Enemy_Rinka = Get_Rinka(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).rinka_parameter_1 != 0 {
        Rinka_1(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        );
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            & !(kEnemyProps_RespawnIfKilled as ::core::ffi::c_int
                | kEnemyProps_DisableSamusColl as ::core::ffi::c_int
                | kEnemyProps_ProcessedOffscreen as ::core::ffi::c_int
                | kEnemyProps_Tangible as ::core::ffi::c_int)
            | 0x2c00 as ::core::ffi::c_int) as uint16;
    } else {
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            & !(kEnemyProps_RespawnIfKilled as ::core::ffi::c_int
                | kEnemyProps_DisableSamusColl as ::core::ffi::c_int
                | kEnemyProps_ProcessedOffscreen as ::core::ffi::c_int
                | kEnemyProps_Tangible as ::core::ffi::c_int)
            | 0x6400 as ::core::ffi::c_int) as uint16;
    }
    (*E).base.palette_index = 1024 as uint16;
    Rinka_Init3(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
}
pub unsafe fn Rinka_Init2(mut k: uint16, game: &mut Game) {
    let mut v2: *mut EnemySpawnData = ::core::ptr::null_mut::<EnemySpawnData>();
    let mut E: *mut Enemy_Rinka = Get_Rinka(k as ::core::ffi::c_int, game);
    if (*E).rinka_parameter_1 != 0 {
        Rinka_1(k, game);
    }
    v2 = gEnemySpawnData(k as ::core::ffi::c_int, game);
    (*E).base.x_pos = (*v2).x_pos;
    (*E).base.y_pos = (*v2).y_pos;
    Rinka_Init3(k, game);
}
pub unsafe fn Rinka_Init3(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Rinka = Get_Rinka(k as ::core::ffi::c_int, game);
    (*E).rinka_var_A = ((0xa2b852i32) as u16) & (0xffffu16);
    (*E).rinka_var_F = 26 as uint16;
    (*E).rinka_var_B = 0 as uint16;
    (*E).rinka_var_C = 0 as uint16;
    if (*E).rinka_parameter_1 != 0 {
        if (*Get_Rinka(0 as ::core::ffi::c_int, game)).rinka_var_1D != 0 {
            (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Deleted) as u16);
        } else {
            (*E).base.current_instruction = addr_kRinka_Ilist_BA0C as uint16;
            (*E).base.instruction_timer = 1 as uint16;
            (*E).base.timer = 0 as uint16;
        }
    } else {
        (*E).base.current_instruction = addr_kRinka_Ilist_B9E0 as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
    };
}
pub unsafe fn Rinka_1(mut k: uint16, game: &mut Game) {
    let mut v2: uint16 = 0;
    let mut ES: *mut EnemySpawnData = gEnemySpawnData(k as ::core::ffi::c_int, game);
    if Rinka_10((*ES).x_pos, (*ES).y_pos, game) as ::core::ffi::c_int != 0 || {
        v2 = Rinka_2(k, game);
        (*gRam8000_Default(v2 as ::core::ffi::c_int, game).offset(31)).var_3F as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            != 0
    } {
        let mut v3: uint16 = 0 as uint16;
        loop {
            let mut v4: ::core::ffi::c_int = v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            let mut x: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa2b75b))
                as *const uint8 as *mut uint16)
                .offset(v4 as isize);
            let mut y: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa2b75b))
                as *const uint8 as *mut uint16)
                .offset((v4 + 1 as ::core::ffi::c_int) as isize);
            if !Rinka_10(x, y, game)
                && (*gRam8000_Default(
                    *(game.g_rom.offset(crate::rtl::lorom_offset(0xa2b75b)) as *const uint8
                        as *mut uint16)
                        .offset((v4 + 2 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_int,
                    game,
                )
                .offset(31))
                .var_3F as ::core::ffi::c_int
                    & 1 as ::core::ffi::c_int
                    == 0
            {
                let mut E: *mut Enemy_Rinka = Get_Rinka(k as ::core::ffi::c_int, game);
                (*ES).x_pos = x;
                (*E).base.x_pos = x;
                (*ES).y_pos = y;
                (*E).base.y_pos = y;
                let mut v9: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa2b75b))
                    as *const uint8 as *mut uint16)
                    .offset(
                        ((v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
                            + 2 as ::core::ffi::c_int) as isize,
                    );
                (*gRam8000_Default(v9 as ::core::ffi::c_int, game).offset(31)).var_3F =
                    -(1 as ::core::ffi::c_int) as uint16;
                (*E).rinka_var_D = v9;
                return;
            }
            v3 = (v3).wrapping_add(6u16);
            if !(((v3) as i16).wrapping_sub(66i16) < 0i16) {
                break;
            }
        }
        let mut v10: uint16 = 0 as uint16;
        while (*gRam8000_Default(
            *(game.g_rom.offset(crate::rtl::lorom_offset(0xa2b75b)) as *const uint8 as *mut uint16)
                .offset(
                    ((v10 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
                        + 2 as ::core::ffi::c_int) as isize,
                ) as ::core::ffi::c_int,
            game,
        )
        .offset(31))
        .var_3F as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            != 0
        {
            v10 = (v10).wrapping_add(6u16);
            if ((v10) as i16).wrapping_sub(66i16) >= 0i16 {
                return;
            }
        }
        let mut v11: ::core::ffi::c_int = v10 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v12: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa2b75b))
            as *const uint8 as *mut uint16)
            .offset(v11 as isize);
        (*ES).x_pos = v12;
        let mut E_0: *mut Enemy_Rinka = Get_Rinka(k as ::core::ffi::c_int, game);
        (*E_0).base.x_pos = v12;
        let mut v15: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa2b75b))
            as *const uint8 as *mut uint16)
            .offset((v11 + 1 as ::core::ffi::c_int) as isize);
        (*ES).y_pos = v15;
        (*E_0).base.y_pos = v15;
        let mut v16: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa2b75b))
            as *const uint8 as *mut uint16)
            .offset((v11 + 2 as ::core::ffi::c_int) as isize);
        (*E_0).rinka_var_D = v16;
        (*gRam8000_Default(v16 as ::core::ffi::c_int, game).offset(31)).var_3F =
            -(1 as ::core::ffi::c_int) as uint16;
    } else {
        (*Get_Rinka(k as ::core::ffi::c_int, game)).rinka_var_D = v2;
        (*gRam8000_Default(v2 as ::core::ffi::c_int, game).offset(31)).var_3F =
            -(1 as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn Rinka_2(mut k: uint16, game: &mut Game) -> uint16 {
    let mut v3: *mut EnemySpawnData = ::core::ptr::null_mut::<EnemySpawnData>();
    let mut v1: uint16 = 0 as uint16;
    loop {
        let mut v2: ::core::ffi::c_int = v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        v3 = gEnemySpawnData(k as ::core::ffi::c_int, game);
        if *(game.g_rom.offset(crate::rtl::lorom_offset(0xa2b75b)) as *const uint8 as *mut uint16)
            .offset(v2 as isize)
            == (*v3).x_pos
            && *(game.g_rom.offset(crate::rtl::lorom_offset(0xa2b75b)) as *const uint8
                as *mut uint16)
                .offset((v2 + 1 as ::core::ffi::c_int) as isize)
                == (*v3).y_pos
        {
            break;
        }
        v1 = (v1).wrapping_add(6u16);
        if v1 as ::core::ffi::c_int - 66 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
            v1 = 0 as uint16;
            return *(game.g_rom.offset(crate::rtl::lorom_offset(0xa2b75b)) as *const uint8
                as *mut uint16)
                .offset(
                    ((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
                        + 2 as ::core::ffi::c_int) as isize,
                );
        }
    }
    return *(game.g_rom.offset(crate::rtl::lorom_offset(0xa2b75b)) as *const uint8 as *mut uint16)
        .offset(
            ((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) + 2 as ::core::ffi::c_int)
                as isize,
        );
}
pub unsafe fn CallRinkaFunc(mut ea: uint32, mut k: uint16, game: &mut Game) {
    match ea {
        10663903 => {
            Rinka_3(k, game);
            return;
        }
        10664004 => {
            Rinka_4(k, game);
            return;
        }
        10664018 => {
            Rinka_5(k, game);
            return;
        }
        10664027 => {
            Rinka_B85B(k, game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn Rinka_Main(game: &mut Game) {
    let mut v2: uint16 = 0;
    let mut E: *mut Enemy_Rinka = Get_Rinka(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).rinka_parameter_1 != 0u16
        && (*Get_Rinka(0 as ::core::ffi::c_int, game)).rinka_var_1D != 0u16
    {
        Rinka_6(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        );
        Rinka_8(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        );
        v2 = 0 as uint16;
        RinkasDeathAnimation(v2, game);
    } else {
        CallRinkaFunc(
            (((*E).rinka_var_A) as u32) | (0xa20000u32),
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        );
    };
}
pub unsafe fn Rinka_3(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Rinka = Get_Rinka(k as ::core::ffi::c_int, game);
    (*E).rinka_var_F = (*E).rinka_var_F.wrapping_sub(1);
    if ((*E).rinka_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        (*E).rinka_var_A = ((0xa2b85bi32) as u16) & (0xffffu16);
        let mut v3: uint16 = 0;
        if (*E).rinka_parameter_1 != 0 {
            v3 = ((*E).base.properties as ::core::ffi::c_int
                & !(kEnemyProps_Tangible as ::core::ffi::c_int)) as uint16;
        } else {
            v3 = ((*E).base.properties as ::core::ffi::c_int & 0xf3ff as ::core::ffi::c_int
                | 0x800 as ::core::ffi::c_int) as uint16;
        }
        (*E).base.properties = v3;
        let mut r18: uint16 = -(CalculateAngleFromXY(
            (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16))
                .wrapping_sub((*E).base.x_pos),
            (*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16))
                .wrapping_sub((*E).base.y_pos),
            game,
        ) as ::core::ffi::c_int
            + 0x80 as ::core::ffi::c_int) as uint8 as uint16;
        (*E).rinka_var_B = Math_MultBySin(0x120 as uint16, r18);
        (*E).rinka_var_C = Math_MultByCos(0x120 as uint16, r18);
    }
}
pub unsafe fn Rinka_4(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Rinka = Get_Rinka(k as ::core::ffi::c_int, game);
    (*E).rinka_var_F = (*E).rinka_var_F.wrapping_sub(1);
    if ((*E).rinka_var_F as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        (*E).base.health = 10 as uint16;
        Rinka_Init2(k, game);
    }
}
pub unsafe fn Rinka_5(mut k: uint16, game: &mut Game) {
    if Rinka_9(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        Rinka_B865(k, game);
    }
}
pub unsafe fn Rinka_B85B(mut k: uint16, game: &mut Game) {
    MoveEnemyWithVelocity(game);
    if Rinka_9(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        Rinka_B865(k, game);
    }
}
pub unsafe fn Rinka_B865(mut k: uint16, game: &mut Game) {
    if (*Get_Rinka(k as ::core::ffi::c_int, game)).rinka_parameter_1 != 0u16 && {
        Rinka_8(k, game);
        (*Get_Rinka(0 as ::core::ffi::c_int, game)).rinka_var_1D != 0u16
    } {
        Rinka_6(k, game);
        DeleteEnemyAndConnectedEnemies(game);
    } else {
        Rinka_6(k, game);
        Rinka_Init2(k, game);
    };
}
pub unsafe fn Rinka_6(mut k: uint16, game: &mut Game) {
    let mut v3: int16 = 0;
    let mut E: *mut Enemy_Rinka = Get_Rinka(k as ::core::ffi::c_int, game);
    if (*E).rinka_parameter_1 != 0u16
        && ((*E).base.properties as ::core::ffi::c_int
            & kEnemyProps_Invisible as ::core::ffi::c_int)
            == 0i32
    {
        let mut E0: *mut Enemy_Rinka = Get_Rinka(0 as ::core::ffi::c_int, game);
        v3 = (((*E0).rinka_var_1E) as i16).wrapping_sub(1i16);
        if v3 < 0i16 {
            v3 = 0 as int16;
        }
        (*E0).rinka_var_1E = v3 as uint16;
    }
}
pub unsafe fn Rinka_7(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Rinka = Get_Rinka(k as ::core::ffi::c_int, game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0xe44) as *mut uint16) as ::core::ffi::c_int
        & 3 as ::core::ffi::c_int
        == (*E).rinka_parameter_1 as ::core::ffi::c_int
    {
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            & !(kEnemyProps_Tangible as ::core::ffi::c_int))
            as uint16;
    } else {
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
    };
}
pub unsafe fn Rinka_8(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Rinka = Get_Rinka(k as ::core::ffi::c_int, game);
    if (*E).rinka_parameter_1 != 0 {
        let mut rinka_var_D: uint16 = (*E).rinka_var_D;
        if rinka_var_D != 0 {
            (*gRam8000_Default(rinka_var_D as ::core::ffi::c_int, game).offset(31)).var_3F =
                0 as uint16;
            (*E).rinka_var_D = 0 as uint16;
        }
    }
}
pub unsafe fn Rinka_9(mut k: uint16, game: &mut Game) -> uint8 {
    let mut y_pos: int16 = 0;
    let mut v3: int16 = 0;
    let mut x_pos: int16 = 0;
    let mut v5: int16 = 0;
    let mut E: *mut Enemy_Rinka = Get_Rinka(k as ::core::ffi::c_int, game);
    y_pos = (*E).base.y_pos as int16;
    let mut result: uint8 = 1 as uint8;
    if y_pos >= 0i16 {
        v3 = (y_pos as ::core::ffi::c_int + 16 as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16)
                as ::core::ffi::c_int) as int16;
        if v3 >= 0i16 {
            if v3 as ::core::ffi::c_int - 256 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != 0
            {
                x_pos = (*E).base.x_pos as int16;
                if x_pos >= 0i16 {
                    v5 = (x_pos as ::core::ffi::c_int + 16 as ::core::ffi::c_int
                        - *((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16)
                            as ::core::ffi::c_int) as int16;
                    if v5 >= 0i16 {
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
pub unsafe fn Rinka_10(mut r18: uint16, mut r20: uint16, game: &mut Game) -> bool {
    return (r20 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32
        || ((r20) as i16).wrapping_sub(
            (*((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16)) as i16,
        ) < 0i16
        || r20 as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16)
                as ::core::ffi::c_int
            - 224 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        || (r20 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32
        || ((r18) as i16).wrapping_sub(
            (*((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16)) as i16,
        ) < 0i16
        || r18 as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16)
                as ::core::ffi::c_int
            - 256 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0;
}
pub unsafe fn Rinka_Frozen(mut k: uint16, game: &mut Game) {
    if Rinka_9(k, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        (*Get_Rinka(k as ::core::ffi::c_int, game))
            .base
            .frozen_timer = 0 as uint16;
    }
    NormalEnemyFrozenAI(game);
    if (*Get_Rinka(0 as ::core::ffi::c_int, game)).rinka_var_1D != 0 {
        Rinka_6(k, game);
        Rinka_8(k, game);
        let mut v1: uint16 = 0 as uint16;
        RinkasDeathAnimation(v1, game);
    }
}
pub unsafe fn Rinka_Touch(game: &mut Game) {
    NormalEnemyTouchAiSkipDeathAnim_CurEnemy(game);
    Rinka_B960(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
}
pub unsafe fn Rinka_Shot(game: &mut Game) {
    NormalEnemyShotAiSkipDeathAnim_CurEnemy(game);
    Rinka_B960(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
}
pub unsafe fn Rinka_Powerbomb(mut k: uint16, game: &mut Game) {
    if ((*Get_Rinka(k as ::core::ffi::c_int, game)).base.properties as ::core::ffi::c_int
        & kEnemyProps_Invisible as ::core::ffi::c_int)
        == 0i32
    {
        NormalEnemyPowerBombAiSkipDeathAnim_CurEnemy(game);
        Rinka_B960(k, game);
    }
}
pub unsafe fn Rinka_B960(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Rinka = Get_Rinka(k as ::core::ffi::c_int, game);
    if (*E).base.health == 0 {
        Rinka_6(k, game);
        Rinka_8(k, game);
        if (*E).rinka_parameter_1 != 0 {
            (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
                | (kEnemyProps_Tangible as ::core::ffi::c_int
                    | kEnemyProps_Invisible as ::core::ffi::c_int))
                as uint16;
            game.eproj_spawn_pt = Point16U {
                x: (*E).base.x_pos,
                y: (*E).base.y_pos,
            };
            SpawnEprojWithRoomGfx(0xe509 as uint16, 3 as uint16, game);
            (*E).rinka_var_A = ((0xa2b844i32) as u16) & (0xffffu16);
            (*E).rinka_var_F = 1 as uint16;
        } else {
            RinkasDeathAnimation(0 as uint16, game);
        }
    }
}
pub unsafe fn Rinka_Instr_B9A2(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    let mut E: *mut Enemy_Rinka = Get_Rinka(0 as ::core::ffi::c_int, game);
    if (*E).rinka_var_1E as ::core::ffi::c_int - 3 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        return (j).wrapping_add(2u16);
    } else {
        return *(RomPtr_A2(j, game) as *mut uint16);
    };
}
pub unsafe fn Rinka_Instr_B9B3(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_Rinka = Get_Rinka(k as ::core::ffi::c_int, game);
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        | (kEnemyProps_Tangible as ::core::ffi::c_int
            | kEnemyProps_Invisible as ::core::ffi::c_int)) as uint16;
    return jp;
}
pub unsafe fn Rinka_Instr_B9BD(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_Rinka = Get_Rinka(k as ::core::ffi::c_int, game);
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        | (kEnemyProps_ProcessedOffscreen as ::core::ffi::c_int
            | kEnemyProps_Tangible as ::core::ffi::c_int
            | kEnemyProps_Invisible as ::core::ffi::c_int)) as uint16;
    return jp;
}
pub unsafe fn Rinka_Instr_B9C7(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_Rinka = Get_Rinka(k as ::core::ffi::c_int, game);
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        & !(kEnemyProps_Tangible as ::core::ffi::c_int
            | kEnemyProps_Invisible as ::core::ffi::c_int)) as uint16;
    (*E).rinka_var_A = ((0xa2b7dfi32) as u16) & (0xffffu16);
    let mut E0: *mut Enemy_Rinka = Get_Rinka(0 as ::core::ffi::c_int, game);
    (*E0).rinka_var_1E = (*E0).rinka_var_1E.wrapping_add(1);
    return jp;
}
pub unsafe fn EnemyInstr_Rio_Instr_1(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_Rio(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .rio_var_E = 1 as uint16;
    return jp;
}
pub unsafe fn Rio_Init(game: &mut Game) {
    let mut E: *mut Enemy_Rio = Get_Rio(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).rio_var_E = 0 as uint16;
    (*E).rio_var_F = 0 as uint16;
    (*E).base.current_instruction = addr_kRio_Ilist_BB4B as uint16;
    (*E).rio_var_B = ((0xa2bbedi32) as u16) & (0xffffu16);
}
pub unsafe fn CallRioFunc(mut ea: uint32, game: &mut Game) {
    let mut k: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16);
    match ea {
        10664941 => {
            Rio_1(k, game);
            return;
        }
        10665010 => {
            Rio_2(k, game);
            return;
        }
        10665032 => {
            Rio_3(k, game);
            return;
        }
        10665143 => {
            Rio_4(k, game);
            return;
        }
        10665215 => {
            Rio_5(k, game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn Rio_Main(game: &mut Game) {
    NextRandom(game);
    let mut E: *mut Enemy_Rio = Get_Rio(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    CallRioFunc((((*E).rio_var_B) as u32) | (0xa20000u32), game);
}
pub unsafe fn Rio_1(mut k: uint16, game: &mut Game) {
    if CompareDistToSamus_X(k, 0xa0 as uint16, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int
        == 0
    {
        let mut E: *mut Enemy_Rio = Get_Rio(k as ::core::ffi::c_int, game);
        (*E).rio_var_C =
            *(game.g_rom.offset(crate::rtl::lorom_offset(0xa2bbbb)) as *const uint8 as *mut uint16);
        (*E).rio_var_D =
            *(game.g_rom.offset(crate::rtl::lorom_offset(0xa2bbbf)) as *const uint8 as *mut uint16);
        if ((*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)) as i16)
            .wrapping_sub(((*E).base.x_pos) as i16)
            < 0i16
        {
            (*E).rio_var_D = -((*E).rio_var_D as ::core::ffi::c_int) as uint16;
        }
        Rio_6(addr_kRio_Ilist_BB7F as uint16, game);
        (*E).rio_var_B = ((0xa2bc48i32) as u16) & (0xffffu16);
        if CheckIfEnemyIsOnScreen(game) == 0 {
            QueueSfx2_Max6(0x65 as uint16, game);
        }
    }
}
pub unsafe fn Rio_2(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Rio = Get_Rio(k as ::core::ffi::c_int, game);
    if (*E).rio_var_E != 0 {
        (*E).rio_var_E = 0 as uint16;
        Rio_6(addr_stru_A2BB53 as uint16, game);
        (*E).rio_var_B = ((0xa2bbedi32) as u16) & (0xffffu16);
    }
}
pub unsafe fn Rio_3(mut k: uint16, game: &mut Game) {
    let mut v4: int16 = 0;
    let mut E: *mut Enemy_Rio = Get_Rio(k as ::core::ffi::c_int, game);
    if Enemy_MoveRight_IgnoreSlopes(
        k,
        ((*E).rio_var_D as int16 as int32) << 8 as ::core::ffi::c_int,
        game,
    ) != 0
    {
        (*E).rio_var_D = -((*E).rio_var_D as ::core::ffi::c_int) as uint16;
    } else if !(Enemy_MoveDown(
        k,
        ((*E).rio_var_C as int16 as int32) << 8 as ::core::ffi::c_int,
        game,
    ) != 0)
    {
        v4 = (((*E).rio_var_C) as i16).wrapping_sub(24i16);
        (*E).rio_var_C = v4 as uint16;
        if v4 < 0i16 {
            (*E).rio_var_A = (*E).rio_var_D;
            (*E).rio_var_D = 0 as uint16;
            (*E).rio_var_C = 0 as uint16;
            (*E).rio_var_B = ((0xa2bcffi32) as u16) & (0xffffu16);
        } else if (*E).rio_var_E != 0 {
            (*E).rio_var_E = 0 as uint16;
            Rio_6(addr_kRio_Ilist_BB97 as uint16, game);
        }
        return;
    }
    (*E).rio_var_C = -((*E).rio_var_C as ::core::ffi::c_int) as uint16;
    (*E).rio_var_B = ((0xa2bcb7i32) as u16) & (0xffffu16);
}
pub unsafe fn Rio_4(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Rio = Get_Rio(k as ::core::ffi::c_int, game);
    if Enemy_MoveRight_IgnoreSlopes(
        k,
        ((*E).rio_var_D as int16 as int32) << 8 as ::core::ffi::c_int,
        game,
    ) != 0
    {
        (*E).rio_var_D = -((*E).rio_var_D as ::core::ffi::c_int) as uint16;
    }
    if Enemy_MoveDown(
        k,
        ((*E).rio_var_C as int16 as int32) << 8 as ::core::ffi::c_int,
        game,
    ) != 0
    {
        Rio_6(addr_kRio_Ilist_BBA3 as uint16, game);
        (*E).rio_var_B = ((0xa2bc32i32) as u16) & (0xffffu16);
    } else {
        (*E).rio_var_C = ((*E).rio_var_C).wrapping_sub(24u16);
    };
}
pub unsafe fn Rio_5(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Rio = Get_Rio(k as ::core::ffi::c_int, game);
    if (((*Get_Rio(k as ::core::ffi::c_int, game)).base.y_pos) as i16)
        .wrapping_sub((*((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16)) as i16)
        >= 0i16
    {
        (*E).rio_var_D = (*E).rio_var_A;
        (*E).rio_var_C = -(1 as ::core::ffi::c_int) as uint16;
        (*E).rio_var_B = ((0xa2bcb7i32) as u16) & (0xffffu16);
    } else {
        let mut v1: ::core::ffi::c_int =
            CalculateAngleOfSamusFromEnemy(k, game) as ::core::ffi::c_int;
        (*E).rio_var_D = kSinCosTable8bit_Sext[(v1 + 64 as ::core::ffi::c_int) as usize] as uint16;
        (*E).rio_var_C = kSinCosTable8bit_Sext[v1 as usize] as uint16;
        Enemy_MoveRight_IgnoreSlopes(
            k,
            ((*E).rio_var_D as int16 as int32) << 8 as ::core::ffi::c_int,
            game,
        );
        Enemy_MoveDown(
            k,
            ((*E).rio_var_C as int16 as int32) << 8 as ::core::ffi::c_int,
            game,
        );
    };
}
pub unsafe fn Rio_6(mut a: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Rio = Get_Rio(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if a != (*E).rio_var_F {
        (*E).rio_var_F = a;
        (*E).base.current_instruction = a;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
    }
}
pub unsafe fn NorfairLavajumpingEnemy_Instr_BE8E(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_NorfairLavajumpingEnemy(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .nley_var_00 = 1 as uint16;
    return jp;
}
pub unsafe fn NorfairLavajumpingEnemy_Init(game: &mut Game) {
    let mut E: *mut Enemy_NorfairLavajumpingEnemy = Get_NorfairLavajumpingEnemy(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).nley_var_00 = 0 as uint16;
    (*E).nley_var_01 = 0 as uint16;
    if ((*E).nley_parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        (*E).base.current_instruction = addr_kNorfairLavajumpingEnemy_Ilist_BE62 as uint16;
        (*E).nley_var_F = ((0xa2bedci32) as u16) & (0xffffu16);
    } else {
        (*E).nley_var_D = (*E).base.x_pos;
        (*E).nley_var_E = (*E).base.y_pos;
        (*E).base.current_instruction = addr_kNorfairLavajumpingEnemy_Ilist_BE3C as uint16;
        (*E).nley_var_F = ((0xa2bf1ai32) as u16) & (0xffffu16);
    };
}
pub unsafe fn NorfairLavajumpingEnemy_Main(game: &mut Game) {
    NextRandom(game);
    let mut E: *mut Enemy_NorfairLavajumpingEnemy = Get_NorfairLavajumpingEnemy(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    CallEnemyPreInstr((((*E).nley_var_F) as u32) | (0xa20000u32), game);
}
pub unsafe fn NorfairLavajumpingEnemy_Func_1(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_NorfairLavajumpingEnemy =
        Get_NorfairLavajumpingEnemy(k as ::core::ffi::c_int, game);
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
        .offset((v1 + 100 as ::core::ffi::c_int) as isize)
        != 0
    {
        let mut v3: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
            .offset((v1 + 109 as ::core::ffi::c_int) as isize);
        (*E).base.frozen_timer = v3;
        if v3 != 0u16
            || (*((&raw mut game.g_ram as *mut uint8).offset(0xf68) as *mut uint16)
                .offset((v1 + 2 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int)
                == 0i32
        {
            (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Invisible) as u16);
        } else {
            (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
                & !(kEnemyProps_Invisible as ::core::ffi::c_int))
                as uint16;
            (*E).base.y_pos = *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
                .offset((v1 + 93 as ::core::ffi::c_int) as isize);
        }
    } else {
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Deleted) as u16);
    };
}
pub unsafe fn NorfairLavajumpingEnemy_Func_2(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_NorfairLavajumpingEnemy =
        Get_NorfairLavajumpingEnemy(k as ::core::ffi::c_int, game);
    (*E).nley_var_C = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa2be86)) as *const uint8
        as *mut uint16)
        .offset(
            ((((*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16 as *mut uint8)
                .offset(
                    (::core::mem::size_of::<uint16>())
                        .wrapping_div(::core::mem::size_of::<uint8>())
                        .wrapping_sub(1 as usize) as isize,
                )) as u16)
                & (6u16)) as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int) as isize,
        );
    (*E).nley_var_F = ((0xa2bf3ei32) as u16) & (0xffffu16);
    (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_ProcessedOffscreen) as u16);
    QueueSfx2_Max6(0xd as uint16, game);
}
pub unsafe fn NorfairLavajumpingEnemy_Func_3(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_NorfairLavajumpingEnemy =
        Get_NorfairLavajumpingEnemy(k as ::core::ffi::c_int, game);
    let mut t: ::core::ffi::c_int = (*E).base.y_subpos as ::core::ffi::c_int
        + ((*(&raw mut (*E).nley_var_C as *mut uint8).offset(0) as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int);
    (*E).base.y_subpos = t as uint16;
    (*E).base.y_pos = ((*E).base.y_pos as ::core::ffi::c_int
        + (*(&raw mut (*E).nley_var_C as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1 as usize) as isize,
        ) as int8 as ::core::ffi::c_int
            + (t >> 16 as ::core::ffi::c_int))) as uint16;
    let mut v3: uint16 = ((*E).nley_var_C).wrapping_add(56u16);
    (*E).nley_var_C = v3;
    if v3 >= 0xfc00u16 {
        NorfairLavajumpingEnemy_Func_6(addr_kNorfairLavajumpingEnemy_Ilist_BE42 as uint16, game);
        (*E).nley_var_F = ((0xa2bf7ci32) as u16) & (0xffffu16);
    }
}
pub unsafe fn NorfairLavajumpingEnemy_Func_4(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_NorfairLavajumpingEnemy =
        Get_NorfairLavajumpingEnemy(k as ::core::ffi::c_int, game);
    let mut t: ::core::ffi::c_int = (*E).base.y_subpos as ::core::ffi::c_int
        + ((*(&raw mut (*E).nley_var_C as *mut uint8).offset(0) as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int);
    (*E).base.y_subpos = t as uint16;
    (*E).base.y_pos = ((*E).base.y_pos as ::core::ffi::c_int
        + (*(&raw mut (*E).nley_var_C as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1 as usize) as isize,
        ) as int8 as ::core::ffi::c_int
            + (t >> 16 as ::core::ffi::c_int))) as uint16;
    (*E).nley_var_C = ((*E).nley_var_C).wrapping_add(56u16);
    if (*E).nley_var_00 != 0 {
        (*E).nley_var_00 = 0 as uint16;
        (*E).nley_var_F = ((0xa2bfbci32) as u16) & (0xffffu16);
    }
}
pub unsafe fn NorfairLavajumpingEnemy_Func_5(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_NorfairLavajumpingEnemy =
        Get_NorfairLavajumpingEnemy(k as ::core::ffi::c_int, game);
    let mut t: ::core::ffi::c_int = (*E).base.y_subpos as ::core::ffi::c_int
        + ((*(&raw mut (*E).nley_var_C as *mut uint8).offset(0) as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int);
    (*E).base.y_subpos = t as uint16;
    (*E).base.y_pos = ((*E).base.y_pos as ::core::ffi::c_int
        + *(&raw mut (*E).nley_var_C as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1 as usize) as isize,
        ) as int8 as ::core::ffi::c_int
        + (t >> 16 as ::core::ffi::c_int)) as uint16;
    if ((*E).base.y_pos as ::core::ffi::c_int & 0xf0 as ::core::ffi::c_int) == 240i32 {
        (*E).base.x_pos = (*E).nley_var_D;
        (*E).base.y_pos = (*E).nley_var_E;
        NorfairLavajumpingEnemy_Func_6(addr_kNorfairLavajumpingEnemy_Ilist_BE3C as uint16, game);
        (*E).nley_var_F = ((0xa2bf1ai32) as u16) & (0xffffu16);
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            & !(kEnemyProps_ProcessedOffscreen as ::core::ffi::c_int))
            as uint16;
    } else {
        (*E).nley_var_C = ((*E).nley_var_C).wrapping_add(56u16);
    };
}
pub unsafe fn NorfairLavajumpingEnemy_Func_6(mut a: uint16, game: &mut Game) {
    let mut E: *mut Enemy_NorfairLavajumpingEnemy = Get_NorfairLavajumpingEnemy(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if a != (*E).nley_var_01 {
        (*E).nley_var_01 = a;
        (*E).base.current_instruction = a;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
    }
}
pub unsafe fn NorfairRio_Instr_C1C9(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_NorfairRio(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .nro_var_01 = 1 as uint16;
    return jp;
}
pub unsafe fn NorfairRio_Instr_C1D4(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_NorfairRio(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .nro_var_02 = 8 as uint16;
    return jp;
}
pub unsafe fn NorfairRio_Instr_C1DF(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_NorfairRio(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .nro_var_02 = 8 as uint16;
    return jp;
}
pub unsafe fn NorfairRio_Instr_C1EA(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_NorfairRio(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .nro_var_02 = 12 as uint16;
    return jp;
}
pub unsafe fn NorfairRio_Instr_C1F5(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_NorfairRio(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .nro_var_02 = -(12 as ::core::ffi::c_int) as uint16;
    return jp;
}
pub unsafe fn NorfairRio_Instr_C200(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_NorfairRio(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .nro_var_02 = 4 as uint16;
    return jp;
}
pub unsafe fn NorfairRio_Instr_C20B(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_NorfairRio(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .nro_var_02 = 0 as uint16;
    return jp;
}
pub unsafe fn NorfairRio_Instr_C216(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_NorfairRio(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .nro_var_02 = -(4 as ::core::ffi::c_int) as uint16;
    return jp;
}
pub unsafe fn NorfairRio_Instr_C221(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_NorfairRio(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .nro_var_02 = -(12 as ::core::ffi::c_int) as uint16;
    return jp;
}
pub unsafe fn NorfairRio_Instr_C22C(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_NorfairRio(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .nro_var_02 = -(16 as ::core::ffi::c_int) as uint16;
    return jp;
}
pub unsafe fn NorfairRio_Instr_C237(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_NorfairRio(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .nro_var_02 = 12 as uint16;
    return jp;
}
pub unsafe fn NorfairRio_Init(game: &mut Game) {
    let mut E: *mut Enemy_NorfairRio = Get_NorfairRio(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).nro_var_01 = 0 as uint16;
    (*E).nro_var_02 = 0 as uint16;
    if ((*E).nro_parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        (*E).nro_var_00 = addr_kNorfairRio_Ilist_C18F as uint16;
        (*E).base.current_instruction = addr_kNorfairRio_Ilist_C18F as uint16;
        (*E).nro_var_F = ((0xa2c281i32) as u16) & (0xffffu16);
    } else {
        (*E).nro_var_00 = addr_kNorfairRio_Ilist_C0F1 as uint16;
        (*E).base.current_instruction = addr_kNorfairRio_Ilist_C0F1 as uint16;
        (*E).nro_var_F = ((0xa2c2e7i32) as u16) & (0xffffu16);
    };
}
pub unsafe fn NorfairRio_Main(game: &mut Game) {
    NextRandom(game);
    let mut E: *mut Enemy_NorfairRio = Get_NorfairRio(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    CallEnemyPreInstr((((*E).nro_var_F) as u32) | (0xa20000u32), game);
}
pub unsafe fn NorfairRio_Func_1(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_NorfairRio = Get_NorfairRio(k as ::core::ffi::c_int, game);
    let mut v6: *mut EnemySpawnData = ::core::ptr::null_mut::<EnemySpawnData>();
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
        .offset((v1 + 100 as ::core::ffi::c_int) as isize)
        != 0
    {
        let mut v3: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
            .offset((v1 + 109 as ::core::ffi::c_int) as isize);
        (*E).base.frozen_timer = v3;
        if v3 != 0u16 || {
            (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
                & !(kEnemyProps_Invisible as ::core::ffi::c_int))
                as uint16;
            (*gEnemySpawnData(k as ::core::ffi::c_int, game).offset(31)).some_flag
                as ::core::ffi::c_int
                == addr_kNorfairRio_Ilist_C0F1 as ::core::ffi::c_int
        } {
            (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Invisible) as u16);
        } else {
            let mut r18: uint16 = addr_kNorfairRio_Ilist_C18F as uint16;
            v6 = gEnemySpawnData(k as ::core::ffi::c_int, game);
            if ((*v6.offset(31)).field_4 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int)
                != 0i32
            {
                r18 = addr_kNorfairRio_Ilist_C1A3 as uint16;
            }
            NorfairRio_Func_7(r18, game);
            (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
                & !(kEnemyProps_Invisible as ::core::ffi::c_int))
                as uint16;
            (*E).base.x_pos = *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
                .offset((v1 + 91 as ::core::ffi::c_int) as isize);
            (*E).base.y_pos = ((*v6.offset(31)).field_4).wrapping_add(
                *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
                    .offset((v1 + 93 as ::core::ffi::c_int) as isize),
            );
        }
    } else {
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Deleted) as u16);
    };
}
pub unsafe fn NorfairRio_Func_2(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_NorfairRio = Get_NorfairRio(k as ::core::ffi::c_int, game);
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16) as ::core::ffi::c_int
        & 0x101 as ::core::ffi::c_int)
        != 0i32
        && CompareDistToSamus_X(k, 0xc0 as uint16, game) as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            == 0
    {
        (*E).nro_var_A = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa2c1c1)) as *const uint8
            as *mut uint16)
            .offset(
                ((*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
                    as ::core::ffi::c_int
                    >> 1 as ::core::ffi::c_int
                    & 2 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
                    >> 1 as ::core::ffi::c_int) as isize,
            );
        (*E).nro_var_B =
            *(game.g_rom.offset(crate::rtl::lorom_offset(0xa2c1c5)) as *const uint8 as *mut uint16);
        if ((*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)) as i16)
            .wrapping_sub(((*E).base.x_pos) as i16)
            < 0i16
        {
            (*E).nro_var_B = -((*E).nro_var_B as ::core::ffi::c_int) as uint16;
        }
        NorfairRio_Func_7(addr_kNorfairRio_Ilist_C107 as uint16, game);
        (*E).nro_var_F = ((0xa2c33fi32) as u16) & (0xffffu16);
    } else if (*E).nro_var_01 != 0 {
        (*E).nro_var_01 = 0 as uint16;
        NorfairRio_Func_7(addr_kNorfairRio_Ilist_C0F1 as uint16, game);
    }
}
pub unsafe fn NorfairRio_Func_3(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_NorfairRio = Get_NorfairRio(k as ::core::ffi::c_int, game);
    if (*E).nro_var_01 != 0 {
        (*E).nro_var_01 = 0 as uint16;
        NorfairRio_Func_7(addr_kNorfairRio_Ilist_C12F as uint16, game);
        (*E).nro_var_F = ((0xa2c361i32) as u16) & (0xffffu16);
        QueueSfx2_Max6(0x65 as uint16, game);
    }
}
pub unsafe fn NorfairRio_Func_4(mut k: uint16, game: &mut Game) {
    let mut v4: int16 = 0;
    let mut E: *mut Enemy_NorfairRio = Get_NorfairRio(k as ::core::ffi::c_int, game);
    if Enemy_MoveRight_IgnoreSlopes(
        k,
        ((*E).nro_var_B as int16 as int32) << 8 as ::core::ffi::c_int,
        game,
    ) != 0
    {
        (*E).nro_var_B = -((*E).nro_var_B as ::core::ffi::c_int) as uint16;
    }
    if Enemy_MoveDown(
        k,
        ((*E).nro_var_A as int16 as int32) << 8 as ::core::ffi::c_int,
        game,
    ) != 0u8
        || {
            v4 = (((*E).nro_var_A) as i16).wrapping_sub(32i16);
            (*E).nro_var_A = v4 as uint16;
            v4 < 0i16
        }
    {
        (*E).nro_var_A = -(1 as ::core::ffi::c_int) as uint16;
        NorfairRio_Func_7(addr_kNorfairRio_Ilist_C145 as uint16, game);
        (*E).nro_var_F = ((0xa2c3b1i32) as u16) & (0xffffu16);
    }
}
pub unsafe fn NorfairRio_Func_5(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_NorfairRio = Get_NorfairRio(k as ::core::ffi::c_int, game);
    if Enemy_MoveRight_IgnoreSlopes(
        k,
        ((*E).nro_var_B as int16 as int32) << 8 as ::core::ffi::c_int,
        game,
    ) != 0
    {
        (*E).nro_var_B = -((*E).nro_var_B as ::core::ffi::c_int) as uint16;
    }
    if Enemy_MoveDown(
        k,
        ((*E).nro_var_A as int16 as int32) << 8 as ::core::ffi::c_int,
        game,
    ) != 0
    {
        (*E).nro_var_F = ((0xa2c406i32) as u16) & (0xffffu16);
    } else {
        (*E).nro_var_A = ((*E).nro_var_A).wrapping_sub(32u16);
        if (*E).nro_var_01 != 0 {
            (*E).nro_var_01 = 0 as uint16;
            NorfairRio_Func_7(addr_kNorfairRio_Ilist_C179 as uint16, game);
        }
    };
}
pub unsafe fn NorfairRio_Func_6(mut k: uint16, game: &mut Game) {
    (*Get_NorfairRio(k as ::core::ffi::c_int, game)).nro_var_F =
        ((0xa2c2e7i32) as u16) & (0xffffu16);
}
pub unsafe fn NorfairRio_Func_7(mut a: uint16, game: &mut Game) {
    let mut E: *mut Enemy_NorfairRio = Get_NorfairRio(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if a != (*E).nro_var_00 {
        (*E).nro_var_00 = a;
        (*E).base.current_instruction = a;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
    }
}
pub unsafe fn LowerNorfairRio_Instr_C6D2(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_NorfairRio(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .nro_var_01 = 1 as uint16;
    return jp;
}
pub unsafe fn LowerNorfairRio_Instr_C6DD(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_NorfairRio(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .nro_var_02 = 0 as uint16;
    return jp;
}
pub unsafe fn LowerNorfairRio_Instr_C6E8(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_NorfairRio(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .nro_var_02 = 1 as uint16;
    return jp;
}
pub unsafe fn LowerNorfairRio_Init(game: &mut Game) {
    let mut E: *mut Enemy_NorfairRio = Get_NorfairRio(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).nro_var_01 = 0 as uint16;
    if ((*E).nro_parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) == 0i32 {
        (*E).nro_var_F = ((0xa2c771i32) as u16) & (0xffffu16);
        (*E).nro_var_00 = addr_kLowerNorfairRio_Ilist_C61A as uint16;
        (*E).base.current_instruction = addr_kLowerNorfairRio_Ilist_C61A as uint16;
    } else {
        (*E).nro_var_F = ((0xa2c72ei32) as u16) & (0xffffu16);
        (*E).nro_var_00 = addr_kLowerNorfairRio_Ilist_C6B0 as uint16;
        (*E).base.current_instruction = addr_kLowerNorfairRio_Ilist_C6B0 as uint16;
    };
}
pub unsafe fn LowerNorfairRio_Main(game: &mut Game) {
    NextRandom(game);
    let mut E: *mut Enemy_LowerNorfairRio = Get_LowerNorfairRio(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    CallEnemyPreInstr((((*E).lnro_var_F) as u32) | (0xa20000u32), game);
}
pub unsafe fn LowerNorfairRio_Func_1(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_LowerNorfairRio = Get_LowerNorfairRio(k as ::core::ffi::c_int, game);
    let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
        .offset((v1 + 100 as ::core::ffi::c_int) as isize)
        != 0
    {
        let mut v3: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
            .offset((v1 + 109 as ::core::ffi::c_int) as isize);
        (*E).base.frozen_timer = v3;
        if v3 == 0 && (*gEnemySpawnData(k as ::core::ffi::c_int, game).offset(31)).field_4 != 0u16 {
            (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
                & !(kEnemyProps_Invisible as ::core::ffi::c_int))
                as uint16;
            (*E).base.x_pos = *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
                .offset((v1 + 91 as ::core::ffi::c_int) as isize);
            (*E).base.y_pos = (*((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
                .offset((v1 + 93 as ::core::ffi::c_int) as isize))
            .wrapping_add(12u16);
        } else {
            (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Invisible) as u16);
        }
    } else {
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Deleted) as u16);
    };
}
pub unsafe fn LowerNorfairRio_Func_2(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_NorfairRio = Get_NorfairRio(k as ::core::ffi::c_int, game);
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16) as ::core::ffi::c_int
        & 0x101 as ::core::ffi::c_int)
        != 0i32
        && CompareDistToSamus_X(k, 0x70 as uint16, game) as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            == 0
    {
        (*E).nro_var_C =
            *(game.g_rom.offset(crate::rtl::lorom_offset(0xa2c6ca)) as *const uint8 as *mut uint16);
        (*E).nro_var_D =
            *(game.g_rom.offset(crate::rtl::lorom_offset(0xa2c6ce)) as *const uint8 as *mut uint16);
        if ((*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)) as i16)
            .wrapping_sub(((*E).base.x_pos) as i16)
            < 0i16
        {
            (*E).nro_var_D = -((*E).nro_var_D as ::core::ffi::c_int) as uint16;
        }
        LowerNorfairRio_Func_7(addr_kLowerNorfairRio_Ilist_C630 as uint16, game);
        (*E).nro_var_F = ((0xa2c7bbi32) as u16) & (0xffffu16);
    } else {
        (*E).nro_var_01 = 0 as uint16;
        LowerNorfairRio_Func_7(addr_kLowerNorfairRio_Ilist_C61A as uint16, game);
    };
}
pub unsafe fn LowerNorfairRio_Func_3(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_NorfairRio = Get_NorfairRio(k as ::core::ffi::c_int, game);
    if (*E).nro_var_01 != 0 {
        (*E).nro_var_01 = 0 as uint16;
        LowerNorfairRio_Func_7(addr_kLowerNorfairRio_Ilist_C65A as uint16, game);
        (*E).nro_var_F = ((0xa2c7d6i32) as u16) & (0xffffu16);
    }
}
pub unsafe fn LowerNorfairRio_Func_4(mut k: uint16, game: &mut Game) {
    let mut v4: int16 = 0;
    let mut E: *mut Enemy_LowerNorfairRio = Get_LowerNorfairRio(k as ::core::ffi::c_int, game);
    if Enemy_MoveRight_IgnoreSlopes(
        k,
        ((*E).lnro_var_D as int16 as int32) << 8 as ::core::ffi::c_int,
        game,
    ) != 0
    {
        (*E).lnro_var_D = -((*E).lnro_var_D as ::core::ffi::c_int) as uint16;
    }
    if Enemy_MoveDown(
        k,
        ((*E).lnro_var_C as int16 as int32) << 8 as ::core::ffi::c_int,
        game,
    ) != 0u8
        || {
            v4 = (((*E).lnro_var_C) as i16).wrapping_sub(32i16);
            (*E).lnro_var_C = v4 as uint16;
            v4 < 0i16
        }
    {
        (*E).lnro_var_C = -(1 as ::core::ffi::c_int) as uint16;
        LowerNorfairRio_Func_7(addr_kLowerNorfairRio_Ilist_C662 as uint16, game);
        (*E).lnro_var_F = ((0xa2c82di32) as u16) & (0xffffu16);
        QueueSfx2_Max6(0x64 as uint16, game);
    }
}
pub unsafe fn LowerNorfairRio_Func_5(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_NorfairRio = Get_NorfairRio(k as ::core::ffi::c_int, game);
    if Enemy_MoveRight_IgnoreSlopes(
        k,
        ((*E).nro_var_D as int16 as int32) << 8 as ::core::ffi::c_int,
        game,
    ) != 0
    {
        (*E).nro_var_D = -((*E).nro_var_D as ::core::ffi::c_int) as uint16;
    }
    if Enemy_MoveDown(
        k,
        ((*E).nro_var_C as int16 as int32) << 8 as ::core::ffi::c_int,
        game,
    ) != 0
    {
        LowerNorfairRio_Func_7(addr_kLowerNorfairRio_Ilist_C686 as uint16, game);
        (*E).nro_var_F = ((0xa2c888i32) as u16) & (0xffffu16);
    } else {
        (*E).nro_var_C = ((*E).nro_var_C).wrapping_sub(32u16);
        if (*E).nro_var_01 != 0 {
            (*E).nro_var_01 = 0 as uint16;
            LowerNorfairRio_Func_7(addr_kLowerNorfairRio_Ilist_C674 as uint16, game);
        }
    };
}
pub unsafe fn LowerNorfairRio_Func_6(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_NorfairRio = Get_NorfairRio(k as ::core::ffi::c_int, game);
    if (*E).nro_var_01 != 0 {
        (*E).nro_var_01 = 0 as uint16;
        LowerNorfairRio_Func_7(addr_kLowerNorfairRio_Ilist_C686 as uint16, game);
        (*E).nro_var_F = ((0xa2c771i32) as u16) & (0xffffu16);
    }
}
pub unsafe fn LowerNorfairRio_Func_7(mut a: uint16, game: &mut Game) {
    let mut E: *mut Enemy_NorfairRio = Get_NorfairRio(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if a != (*E).nro_var_00 {
        (*E).nro_var_00 = a;
        (*E).base.current_instruction = a;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
    }
}
pub unsafe fn MaridiaLargeSnail_Instr_CB6B(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    QueueSfx2_Max6(0xe as uint16, game);
    return jp;
}
pub unsafe fn MaridiaLargeSnail_Instr_CCB3(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_MaridiaLargeSnail(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .mlsl_var_02 = 1 as uint16;
    return jp;
}
pub unsafe fn MaridiaLargeSnail_Instr_CCBE(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_MaridiaLargeSnail(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .mlsl_var_03 = 1 as uint16;
    return jp;
}
pub unsafe fn MaridiaLargeSnail_Instr_CCC9(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_MaridiaLargeSnail(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .mlsl_var_03 = 0 as uint16;
    return jp;
}
pub unsafe fn MaridiaLargeSnail_Init(game: &mut Game) {
    let mut E: *mut Enemy_MaridiaLargeSnail = Get_MaridiaLargeSnail(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).mlsl_var_B = 0 as uint16;
    (*E).mlsl_var_D = 0 as uint16;
    (*E).mlsl_var_00 = 0 as uint16;
    (*E).mlsl_var_01 = 0 as uint16;
    (*E).mlsl_var_02 = 0 as uint16;
    (*E).mlsl_var_03 = 0 as uint16;
    (*E).mlsl_var_0A = 0 as uint16;
    (*E).mlsl_var_E = 3 as uint16;
    (*E).mlsl_var_C = 128 as uint16;
    (*E).base.current_instruction = addr_kMaridiaLargeSnail_Ilist_CA4B as uint16;
    (*E).mlsl_var_A = ((0xa2cde6i32) as u16) & (0xffffu16);
    (*E).mlsl_var_F = ((0xa2cf66i32) as u16) & (0xffffu16);
}
pub unsafe fn CallMaridiaLargeSnailFunc(mut ea: uint32, game: &mut Game) {
    match ea {
        10669542 => {
            MaridiaLargeSnail_Func_4(game);
            return;
        }
        10669611 => {
            MaridiaLargeSnail_Func_5(game);
            return;
        }
        10669888 => {
            MaridiaLargeSnail_Func_6(game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn MaridiaLargeSnail_Main(game: &mut Game) {
    MaridiaLargeSnail_Func_1(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
    let mut E: *mut Enemy_MaridiaLargeSnail = Get_MaridiaLargeSnail(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    CallMaridiaLargeSnailFunc((((*E).mlsl_var_A) as u32) | (0xa20000u32), game);
    MaridiaLargeSnail_Func_2(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
    MaridiaLargeSnail_Func_3(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
}
pub unsafe fn MaridiaLargeSnail_Func_1(mut k: uint16, game: &mut Game) {
    MaridiaLargeSnail_Func_11(game);
    let mut E: *mut Enemy_MaridiaLargeSnail = Get_MaridiaLargeSnail(k as ::core::ffi::c_int, game);
    (*E).mlsl_var_06 = (*E).base.x_pos;
    (*E).mlsl_var_08 = (*E).base.y_pos;
}
pub unsafe fn MaridiaLargeSnail_Func_2(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_MaridiaLargeSnail = Get_MaridiaLargeSnail(k as ::core::ffi::c_int, game);
    (*E).mlsl_var_0B = 0 as uint16;
    if CheckIfEnemyTouchesSamus(k, game) != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0xb5c) as *mut uint16) =
            ((*E).base.y_pos).wrapping_sub((*E).mlsl_var_08);
    }
    if (*E).mlsl_var_04 != 0 {
        let mut r18: uint16 = ((*E).base.x_pos).wrapping_sub((*E).mlsl_var_06);
        if (r18 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
            if (*E).mlsl_var_05 == 0 {
                return;
            }
        } else if (*E).mlsl_var_05 != 0 {
            return;
        }
        let ref mut fresh28 = *((&raw mut game.g_ram as *mut uint8).offset(0xb58) as *mut uint16);
        *fresh28 = (*fresh28).wrapping_add(r18);
    }
}
pub unsafe fn MaridiaLargeSnail_Func_3(mut k: uint16, game: &mut Game) {
    let mut v2: int16 = 0;
    let mut E: *mut Enemy_MaridiaLargeSnail = Get_MaridiaLargeSnail(k as ::core::ffi::c_int, game);
    if (*E).mlsl_var_04 != 0 {
        v2 = (((*E).base.x_pos) as i16).wrapping_sub(((*E).mlsl_var_06) as i16);
        if v2 == 0 {
            if (*E).mlsl_var_05 != 0 {
                if (*E).mlsl_var_D == 0
                    || (*((&raw mut game.g_ram as *mut uint8).offset(0x8b) as *mut uint16)
                        as ::core::ffi::c_int
                        & 0x100 as ::core::ffi::c_int)
                        == 0i32
                {
                    return;
                }
            } else if (*E).mlsl_var_D != 0u16
                || (*((&raw mut game.g_ram as *mut uint8).offset(0x8b) as *mut uint16)
                    as ::core::ffi::c_int
                    & 0x200 as ::core::ffi::c_int)
                    == 0i32
            {
                return;
            }
            (*E).mlsl_var_0B = 1 as uint16;
            (*E).base.x_pos = (*E).mlsl_var_06;
            return;
        }
        if v2 < 0i16 {
            if (*((&raw mut game.g_ram as *mut uint8).offset(0x8b) as *mut uint16)
                as ::core::ffi::c_int
                & 0x100 as ::core::ffi::c_int)
                != 0i32
            {
                (*E).mlsl_var_0B = 1 as uint16;
                (*E).base.x_pos = (*E).mlsl_var_06;
            }
        } else if (*((&raw mut game.g_ram as *mut uint8).offset(0x8b) as *mut uint16)
            as ::core::ffi::c_int
            & 0x200 as ::core::ffi::c_int)
            != 0i32
        {
            (*E).mlsl_var_0B = 1 as uint16;
            (*E).base.x_pos = (*E).mlsl_var_06;
        }
    }
}
pub unsafe fn MaridiaLargeSnail_Func_4(game: &mut Game) {
    let mut E: *mut Enemy_MaridiaLargeSnail = Get_MaridiaLargeSnail(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).mlsl_var_E != 0 {
        CallEnemyPreInstr((((*E).mlsl_var_F) as u32) | (0xa20000u32), game);
    } else {
        (*E).mlsl_var_00 = 0 as uint16;
        if (GetSamusEnemyDelta_X(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        ) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int)
            != 0i32
        {
            (*E).mlsl_var_D = 1 as uint16;
            (*E).mlsl_var_00 = 1 as uint16;
        }
        MaridiaLargeSnail_Func_10(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        );
        if IsSamusWithinEnemy_X(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            0x18 as uint16,
            game,
        ) != 0
        {
            (*E).mlsl_var_00 = ((*E).mlsl_var_00) | (2u16);
            MaridiaLargeSnail_Func_10(
                *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
                game,
            );
            (*E).mlsl_var_A = ((0xa2ce2bi32) as u16) & (0xffffu16);
        }
    };
}
pub unsafe fn MaridiaLargeSnail_Func_5(game: &mut Game) {
    let mut E: *mut Enemy_MaridiaLargeSnail = Get_MaridiaLargeSnail(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).mlsl_var_E != 0 {
        CallEnemyPreInstr((((*E).mlsl_var_F) as u32) | (0xa20000u32), game);
    } else if Enemy_MoveDown(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        (1 as int16 as int32) << 16 as ::core::ffi::c_int,
        game,
    ) == 0
    {
        (*E).mlsl_var_B = 0 as uint16;
        (*E).mlsl_var_F = ((0xa2cf66i32) as u16) & (0xffffu16);
        (*E).mlsl_var_E = 3 as uint16;
    }
    if (*E).mlsl_var_0B == 0 {
        let mut R36: uint16 = 0 as uint16;
        (*E).mlsl_var_C = (*E).mlsl_var_C.wrapping_sub(1);
        let mut v4: bool =
            ((*E).mlsl_var_C as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32;
        if v4
            && {
                (*E).mlsl_var_C = 0 as uint16;
                IsSamusWithinEnemy_X(
                    *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
                    0x20 as uint16,
                    game,
                ) != 0u16
            }
            && (*E).mlsl_var_03 != 0u16
            && (*E).mlsl_var_E == 0
        {
            (*E).mlsl_var_00 = 0 as uint16;
            (*E).mlsl_var_D = 0 as uint16;
            if (GetSamusEnemyDelta_X(
                *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
                game,
            ) as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int)
                != 0i32
            {
                (*E).mlsl_var_00 = 1 as uint16;
                (*E).mlsl_var_D = 1 as uint16;
            }
            (*E).mlsl_var_C = 128 as uint16;
            (*E).mlsl_var_00 = ((*E).mlsl_var_00 as ::core::ffi::c_int & 1 as ::core::ffi::c_int
                | 4 as ::core::ffi::c_int) as uint16;
            MaridiaLargeSnail_Func_10(
                *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
                game,
            );
            (*E).mlsl_var_A = ((0xa2cf40i32) as u16) & (0xffffu16);
        } else {
            let mut v5: uint16 = 128 as uint16;
            if (*E).mlsl_var_D != 0 {
                v5 = 132 as uint16;
            }
            let mut v6: ::core::ffi::c_int = v5 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            let mut r18: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187))
                as *const uint8 as *mut uint16)
                .offset((v6 + 1 as ::core::ffi::c_int) as isize);
            let mut r20: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187))
                as *const uint8 as *mut uint16)
                .offset(v6 as isize);
            if (*E).mlsl_var_04 != 0u16 && (*E).mlsl_var_D == (*E).mlsl_var_05 {
                if (*E).mlsl_var_D != 0 {
                    r20 = (r20).wrapping_sub(16u16);
                } else {
                    r20 = (r20).wrapping_add(16u16);
                }
            }
            if Enemy_MoveRight_SlopesAsWalls(
                *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
                ((r20 as uint32) << 16 as ::core::ffi::c_int | r18 as uint32) as int32,
                game,
            ) != 0
            {
                R36 = R36.wrapping_add(1);
            }
            if (*E).mlsl_var_04 != 0u16 && (*E).mlsl_var_D == (*E).mlsl_var_05 {
                let mut v7: uint16 = 0;
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
                MaridiaLargeSnail_Func_10(
                    *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
                    game,
                );
            }
        }
    }
}
pub unsafe fn MaridiaLargeSnail_Func_6(game: &mut Game) {
    let mut E: *mut Enemy_MaridiaLargeSnail = Get_MaridiaLargeSnail(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).mlsl_var_02 != 0 {
        (*E).mlsl_var_02 = 0 as uint16;
        (*E).mlsl_var_00 = ((*E).mlsl_var_00).wrapping_sub(2u16);
        MaridiaLargeSnail_Func_10(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        );
        (*E).mlsl_var_A = ((0xa2ce2bi32) as u16) & (0xffffu16);
    }
}
pub unsafe fn MaridiaLargeSnail_Func_7(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_MaridiaLargeSnail = Get_MaridiaLargeSnail(k as ::core::ffi::c_int, game);
    let mut v2: uint16 = ((*E).mlsl_var_B).wrapping_add(384u16);
    if v2 as ::core::ffi::c_int - 0x4000 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
        v2 = 0x4000 as uint16;
    }
    (*E).mlsl_var_B = v2;
    let mut v5: ::core::ffi::c_int = 8 as ::core::ffi::c_int
        * (v2 as ::core::ffi::c_int >> 8 as ::core::ffi::c_int)
        >> 1 as ::core::ffi::c_int;
    if Enemy_MoveDown(
        k,
        *(game.g_rom.offset(crate::rtl::lorom_offset(0xa0cbc7)) as *const uint8 as *mut uint32)
            .offset((v5 >> 1 as ::core::ffi::c_int) as isize) as int32,
        game,
    ) != 0
    {
        (*E).mlsl_var_E = (*E).mlsl_var_E.wrapping_sub(1);
        let mut v6: int16 = (((*E).mlsl_var_B) as i16).wrapping_sub(4096i16);
        if v6 < 0i16 {
            (*E).mlsl_var_E = 0 as uint16;
        }
        (*E).mlsl_var_B = v6 as uint16;
        (*E).mlsl_var_F = ((0xa2cfa9i32) as u16) & (0xffffu16);
    }
}
pub unsafe fn MaridiaLargeSnail_Func_8(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_MaridiaLargeSnail = Get_MaridiaLargeSnail(k as ::core::ffi::c_int, game);
    let mut v2: int16 = (((*E).mlsl_var_B) as i16).wrapping_sub(384i16);
    (*E).mlsl_var_B = v2 as uint16;
    if v2 >= 0i16 {
        let mut v3: uint16 = ((v2 as ::core::ffi::c_int & 0x7f00 as ::core::ffi::c_int)
            >> 8 as ::core::ffi::c_int) as uint16;
        let mut v5: ::core::ffi::c_int =
            8 as ::core::ffi::c_int * v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        Enemy_MoveDown(
            k,
            *(game.g_rom.offset(crate::rtl::lorom_offset(0xa0cbc7)) as *const uint8 as *mut uint32)
                .offset((v5 + 2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                as int32,
            game,
        );
    } else {
        (*E).mlsl_var_B = 0 as uint16;
        (*E).mlsl_var_F = ((0xa2cf66i32) as u16) & (0xffffu16);
    };
}
pub unsafe fn MaridiaLargeSnail_Func_9(game: &mut Game) {
    QueueSfx2_Max6(0x63 as uint16, game);
}
pub unsafe fn MaridiaLargeSnail_Func_10(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_MaridiaLargeSnail = Get_MaridiaLargeSnail(k as ::core::ffi::c_int, game);
    let mut mlsl_var_00: uint16 = (*E).mlsl_var_00;
    if mlsl_var_00 != (*E).mlsl_var_01 {
        (*E).mlsl_var_01 = mlsl_var_00;
        (*E).base.current_instruction = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa2cb77))
            as *const uint8 as *mut uint16)
            .offset(mlsl_var_00 as isize);
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
    }
}
pub unsafe fn MaridiaLargeSnail_Func_11(game: &mut Game) {
    let mut E: *mut Enemy_MaridiaLargeSnail = Get_MaridiaLargeSnail(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).mlsl_var_04 = 0 as uint16;
    (*E).mlsl_var_05 = 0 as uint16;
    if IsSamusWithinEnemy_Y(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        0x20 as uint16,
        game,
    ) != 0u16
        && IsSamusWithinEnemy_X(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            0x18 as uint16,
            game,
        ) != 0u16
    {
        (*E).mlsl_var_04 = 1 as uint16;
        if (GetSamusEnemyDelta_X(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        ) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int)
            != 0i32
        {
            (*E).mlsl_var_05 = 1 as uint16;
        }
    }
}
pub unsafe fn MaridiaLargeSnail_Func_12(game: &mut Game) {
    Enemy_NormalTouchAI_A2(game);
    MaridiaLargeSnail_Touch(game);
}
pub unsafe fn MaridiaLargeSnail_Touch(game: &mut Game) {
    let mut k: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16);
    if CheckIfEnemyTouchesSamus(k, game) == 0 {
        if ((*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)) as i16)
            .wrapping_sub(
                ((*Get_MaridiaLargeSnail(
                    *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                        as ::core::ffi::c_int,
                    game,
                ))
                .base
                .x_pos) as i16,
            )
            < 0i16
        {
            let ref mut fresh26 =
                *((&raw mut game.g_ram as *mut uint8).offset(0xb58) as *mut uint16);
            *fresh26 = (*fresh26).wrapping_sub(4u16);
        } else {
            let ref mut fresh27 =
                *((&raw mut game.g_ram as *mut uint8).offset(0xb58) as *mut uint16);
            *fresh27 = (*fresh27).wrapping_add(4u16);
        }
    }
}
pub unsafe fn MaridiaLargeSnail_Shot(game: &mut Game) {
    NormalEnemyShotAi(game);
    QueueSfx2_Max6(0x57 as uint16, game);
}
unsafe fn HirisingSlowfalling_Func_1(mut k: uint16, game: &mut Game) -> uint16 {
    let mut E: *mut Enemy_HirisingSlowfalling =
        Get_HirisingSlowfalling(k as ::core::ffi::c_int, game);
    return Mult8x8((*E).hsg_parameter_1 as uint8, (*E).hsg_var_F as uint8);
}
pub unsafe fn HirisingSlowfalling_Init(game: &mut Game) {
    HirisingSlowfalling_Func_3(addr_kHirisingSlowfalling_Ilist_D82C as uint16, game);
    let mut E: *mut Enemy_HirisingSlowfalling = Get_HirisingSlowfalling(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).hsg_var_A = ((0xa2e035i32) as u16) & (0xffffu16);
    (*E).hsg_var_00 = (*E).base.x_pos;
    (*E).hsg_var_01 = (*E).base.y_pos;
    let mut v2: ::core::ffi::c_int = *(&raw mut (*E).hsg_parameter_1 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>())
            .wrapping_div(::core::mem::size_of::<uint8>())
            .wrapping_sub(1 as usize) as isize,
    ) as ::core::ffi::c_int;
    (*E).hsg_var_E = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa2df5e)) as *const uint8
        as *mut uint16)
        .offset(v2 as isize);
    (*E).hsg_var_F = *(RomPtr_A2(
        *(game.g_rom.offset(crate::rtl::lorom_offset(0xa2df6a)) as *const uint8 as *mut uint16)
            .offset(v2 as isize),
        game,
    ) as *mut uint16);
    let mut r18: uint16 = HirisingSlowfalling_Func_1(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
    HirisingSlowfalling_Func_2(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        r18,
        game,
    );
    (*E).hsg_var_03 = ((*E).hsg_var_01).wrapping_sub(r18);
    (*E).hsg_var_02 = (*E).hsg_var_00;
    (*E).hsg_var_B = (*E).hsg_var_06;
}
pub unsafe fn HirisingSlowfalling_Func_2(mut k: uint16, mut r18: uint16, game: &mut Game) {
    let mut v: uint32 = 0 as uint32;
    let mut r24: uint16 = 0 as uint16;
    loop {
        r24 = (r24).wrapping_add(512u16);
        let mut v1: ::core::ffi::c_int =
            4 as ::core::ffi::c_int * (r24 as ::core::ffi::c_int >> 8 as ::core::ffi::c_int);
        v = v.wrapping_add(
            (*(game.g_rom.offset(crate::rtl::lorom_offset(0xa2838f)) as *const uint8 as *mut uint16)
                .offset((v1 + 1 as ::core::ffi::c_int) as isize) as uint32)
                << 16 as ::core::ffi::c_int
                | *(game.g_rom.offset(crate::rtl::lorom_offset(0xa2838f)) as *const uint8
                    as *mut uint16)
                    .offset(v1 as isize) as uint32,
        );
        if !((v >> 16 as ::core::ffi::c_int).wrapping_sub(r18 as uint32) & 0x8000 as uint32 != 0) {
            break;
        }
    }
    (*Get_HirisingSlowfalling(k as ::core::ffi::c_int, game)).hsg_var_06 = r24;
}
pub unsafe fn HirisingSlowfalling_Func_3(mut a: uint16, game: &mut Game) {
    let mut E: *mut Enemy_HirisingSlowfalling = Get_HirisingSlowfalling(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.current_instruction = a;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
}
pub unsafe fn CallHirisingSlowfalling(mut ea: uint32, game: &mut Game) {
    match ea {
        10674229 => {
            HirisingSlowfalling_Func_4(game);
            return;
        }
        10674255 => {
            HirisingSlowfalling_Func_5(game);
            return;
        }
        10674282 => {
            HirisingSlowfalling_Func_6(game);
            return;
        }
        10674381 => {
            HirisingSlowfalling_Func_7(game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn HirisingSlowfalling_Main(game: &mut Game) {
    let mut E: *mut Enemy_HirisingSlowfalling = Get_HirisingSlowfalling(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    CallHirisingSlowfalling((((*E).hsg_var_A) as u32) | (0xa20000u32), game);
}
pub unsafe fn HirisingSlowfalling_Func_4(game: &mut Game) {
    if IsSamusWithinEnemy_X(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        0x50 as uint16,
        game,
    ) != 0
    {
        let mut E: *mut Enemy_HirisingSlowfalling = Get_HirisingSlowfalling(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        );
        (*E).hsg_var_07 = (*E).hsg_parameter_2;
        (*E).hsg_var_A = ((0xa2e04fi32) as u16) & (0xffffu16);
    }
}
pub unsafe fn HirisingSlowfalling_Func_5(game: &mut Game) {
    let mut E: *mut Enemy_HirisingSlowfalling = Get_HirisingSlowfalling(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut v1: int16 = (((*E).hsg_var_07) as i16).wrapping_sub(1i16);
    (*E).hsg_var_07 = v1 as uint16;
    if v1 < 0i16 {
        HirisingSlowfalling_Func_3(addr_kHirisingSlowfalling_Ilist_D834 as uint16, game);
        (*E).hsg_var_A = ((0xa2e06ai32) as u16) & (0xffffu16);
    }
}
pub unsafe fn HirisingSlowfalling_Func_6(game: &mut Game) {
    let mut E: *mut Enemy_HirisingSlowfalling = Get_HirisingSlowfalling(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut v2: ::core::ffi::c_int = 4 as ::core::ffi::c_int
        * *(&raw mut (*E).hsg_var_B as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1 as usize) as isize,
        ) as ::core::ffi::c_int;
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0xa2838f)) as *const uint8 as *mut uint16)
            .offset((v2 + 3 as ::core::ffi::c_int) as isize) as uint32)
            << 16 as ::core::ffi::c_int
            | *(game.g_rom.offset(crate::rtl::lorom_offset(0xa2838f)) as *const uint8
                as *mut uint16)
                .offset((v2 + 2 as ::core::ffi::c_int) as isize) as uint32,
    );
    let mut v5: int16 = (((*E).hsg_var_B) as i16).wrapping_sub(512i16);
    (*E).hsg_var_B = v5 as uint16;
    if v5 < 0i16 {
        (*E).hsg_var_04 = (*E).hsg_var_02;
        (*E).base.x_pos = (*E).hsg_var_04;
        (*E).hsg_var_05 = (*E).hsg_var_03;
        (*E).base.y_pos = (*E).hsg_var_05;
        (*E).hsg_var_C = 0 as uint16;
        (*E).hsg_var_D =
            ((*(&raw mut (*E).hsg_parameter_1 as *mut uint8).offset(0)) as u16).wrapping_sub(1u16);
        HirisingSlowfalling_Func_3(addr_kHirisingSlowfalling_Ilist_D840 as uint16, game);
        (*E).hsg_var_A = ((0xa2e0cdi32) as u16) & (0xffffu16);
    }
}
pub unsafe fn HirisingSlowfalling_Func_7(game: &mut Game) {
    let mut E: *mut Enemy_HirisingSlowfalling = Get_HirisingSlowfalling(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut v1: *const uint8 = RomPtr_A2(
        ((*E).hsg_var_E as ::core::ffi::c_int
            + 4 as ::core::ffi::c_int
                * *(&raw mut (*E).hsg_var_C as *mut uint8).offset(
                    (::core::mem::size_of::<uint16>())
                        .wrapping_div(::core::mem::size_of::<uint8>())
                        .wrapping_sub(1 as usize) as isize,
                ) as ::core::ffi::c_int) as uint16_t,
        game,
    );
    if *(v1 as *mut uint16) == 0x8000u16 {
        (*E).hsg_var_05 = ((*E).hsg_var_05).wrapping_add((*E).hsg_var_F);
        (*E).hsg_var_C = 0 as uint16;
        (*E).hsg_var_D = (*E).hsg_var_D.wrapping_sub(1);
        if ((*E).hsg_var_D as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
            (*E).hsg_var_B = (*E).hsg_var_06;
            (*E).base.x_pos = (*E).hsg_var_00;
            (*E).base.x_subpos = 0 as uint16;
            (*E).base.y_pos = (*E).hsg_var_01;
            (*E).base.y_subpos = 0 as uint16;
            HirisingSlowfalling_Func_3(addr_kHirisingSlowfalling_Ilist_D82C as uint16, game);
            (*E).hsg_var_A = ((0xa2e035i32) as u16) & (0xffffu16);
        }
    } else {
        (*E).base.x_pos = (*(v1 as *mut uint16)).wrapping_add((*E).hsg_var_04);
        (*E).base.y_pos = (*(v1.offset(2) as *mut uint16)).wrapping_add((*E).hsg_var_05);
        (*E).hsg_var_C = ((*E).hsg_var_C).wrapping_add(256u16);
    };
}
pub unsafe fn Gripper_Init(game: &mut Game) {
    let mut E: *mut Enemy_Gripper = Get_Gripper(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut v1: uint16 = (8u16)
        .wrapping_mul((*(&raw mut (*E).base.current_instruction as *mut uint8).offset(0)) as u16);
    (*E).gripper_var_E = v1;
    let mut v2: ::core::ffi::c_int = v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v3: uint16 = 0;
    if ((*E).base.current_instruction as ::core::ffi::c_int & 0xfeff as ::core::ffi::c_int) != 0i32
    {
        (*E).gripper_var_D =
            *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8 as *mut uint16)
                .offset(v2 as isize);
        v3 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8
            as *mut uint16)
            .offset((v2 + 1 as ::core::ffi::c_int) as isize);
    } else {
        (*E).gripper_var_D =
            *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8 as *mut uint16)
                .offset((v2 + 2 as ::core::ffi::c_int) as isize);
        v3 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8
            as *mut uint16)
            .offset((v2 + 3 as ::core::ffi::c_int) as isize);
    }
    (*E).gripper_var_C = v3;
    let mut v4: uint16 = addr_kGripper_Ilist_E19B as uint16;
    if ((*E).gripper_var_D as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) == 0i32 {
        v4 = addr_kGripper_Ilist_E1AF as uint16;
    }
    (*E).base.current_instruction = v4;
    (*E).gripper_var_A = (*E).gripper_parameter_1;
    (*E).gripper_var_B = (*E).gripper_parameter_2;
}
pub unsafe fn Gripper_Main(game: &mut Game) {
    let mut E: *mut Enemy_Gripper = Get_Gripper(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if Enemy_MoveRight_IgnoreSlopes(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        (((*E).gripper_var_D as uint32) << 16 as ::core::ffi::c_int | (*E).gripper_var_C as uint32)
            as int32,
        game,
    ) != 0u8
        || Gripper_Func_1(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        ) as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            != 0
        || Gripper_Func_2(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        ) as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            != 0
    {
        let mut v3: ::core::ffi::c_int =
            (*E).gripper_var_E as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let mut v4: uint16 = 0;
        if ((*E).gripper_var_D as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) == 0i32 {
            (*E).gripper_var_D = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187))
                as *const uint8 as *mut uint16)
                .offset((v3 + 2 as ::core::ffi::c_int) as isize);
            (*E).gripper_var_C = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187))
                as *const uint8 as *mut uint16)
                .offset((v3 + 3 as ::core::ffi::c_int) as isize);
            v4 = addr_kGripper_Ilist_E19B as uint16;
        } else {
            (*E).gripper_var_D = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187))
                as *const uint8 as *mut uint16)
                .offset(v3 as isize);
            (*E).gripper_var_C = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187))
                as *const uint8 as *mut uint16)
                .offset((v3 + 1 as ::core::ffi::c_int) as isize);
            v4 = addr_kGripper_Ilist_E1AF as uint16;
        }
        (*E).base.current_instruction = v4;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
    }
}
pub unsafe fn Gripper_Func_1(mut k: uint16, game: &mut Game) -> uint8 {
    let mut E: *mut Enemy_Gripper = Get_Gripper(k as ::core::ffi::c_int, game);
    return (((*E).gripper_var_D as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32
        && (((*E).base.x_pos) as i16).wrapping_sub(((*E).gripper_var_A) as i16) < 0i16)
        as uint8;
}
pub unsafe fn Gripper_Func_2(mut k: uint16, game: &mut Game) -> uint8 {
    let mut E: *mut Enemy_Gripper = Get_Gripper(k as ::core::ffi::c_int, game);
    return (((*E).gripper_var_D as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) == 0i32
        && (((*E).base.x_pos) as i16).wrapping_sub(((*E).gripper_var_B) as i16) >= 0i16)
        as uint8;
}
pub unsafe fn Gripper_Func_3(game: &mut Game) {
    Enemy_NormalFrozenAI_A2(game);
}
pub unsafe fn Gripper_Func_4(game: &mut Game) {
    NormalEnemyShotAi(game);
    let mut E: *mut Enemy_Gripper = Get_Gripper(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).base.frozen_timer != 0 {
        let mut v1: uint16 = addr_kGripper_Sprmap_E43F as uint16;
        if ((*E).gripper_var_D as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) == 0i32 {
            v1 = addr_kGripper_Sprmap_E44B as uint16;
        }
        (*E).base.spritemap_pointer = v1;
    }
}
pub unsafe fn JetPowerRipper_Init(game: &mut Game) {
    let mut v0: uint16 = addr_kRipper_Ilist_E2E0 as uint16;
    let mut E: *mut Enemy_JetPowerRipper = Get_JetPowerRipper(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).jprr_parameter_2 != 0 {
        v0 = addr_kRipper_Ilist_E2F4 as uint16;
    }
    (*E).base.current_instruction = v0;
    let mut v2: uint16 = (8u16).wrapping_mul((*E).jprr_parameter_1);
    (*E).jprr_var_E = v2;
    let mut v3: ::core::ffi::c_int = v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v4: uint16 = 0;
    if (*E).jprr_parameter_2 != 0 {
        (*E).jprr_var_D = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8
            as *mut uint16)
            .offset(v3 as isize);
        v4 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8
            as *mut uint16)
            .offset((v3 + 1 as ::core::ffi::c_int) as isize);
    } else {
        (*E).jprr_var_D = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8
            as *mut uint16)
            .offset((v3 + 2 as ::core::ffi::c_int) as isize);
        v4 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8
            as *mut uint16)
            .offset((v3 + 3 as ::core::ffi::c_int) as isize);
    }
    (*E).jprr_var_C = v4;
}
pub unsafe fn JetPowerRipper_Main(game: &mut Game) {
    let mut v3: int16 = 0;
    let mut E: *mut Enemy_JetPowerRipper = Get_JetPowerRipper(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if Enemy_MoveRight_IgnoreSlopes(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        (((*E).jprr_var_D as uint32) << 16 as ::core::ffi::c_int | (*E).jprr_var_C as uint32)
            as int32,
        game,
    ) != 0
    {
        let mut v2: ::core::ffi::c_int =
            (*E).jprr_var_E as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        if ((*E).jprr_var_D as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) == 0i32 {
            (*E).jprr_var_D = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187))
                as *const uint8 as *mut uint16)
                .offset((v2 + 2 as ::core::ffi::c_int) as isize);
            (*E).jprr_var_C = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187))
                as *const uint8 as *mut uint16)
                .offset((v2 + 3 as ::core::ffi::c_int) as isize);
            v3 = -(7456 as ::core::ffi::c_int) as int16;
        } else {
            (*E).jprr_var_D = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187))
                as *const uint8 as *mut uint16)
                .offset(v2 as isize);
            (*E).jprr_var_C = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187))
                as *const uint8 as *mut uint16)
                .offset((v2 + 1 as ::core::ffi::c_int) as isize);
            v3 = -(7436 as ::core::ffi::c_int) as int16;
        }
        (*E).base.current_instruction = v3 as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
    }
}
pub unsafe fn Ripper_Func_1(game: &mut Game) {
    Enemy_NormalFrozenAI_A2(game);
}
pub unsafe fn JetPowerRipper_Shot(game: &mut Game) {
    let mut v1: int16 = 0;
    NormalEnemyShotAi(game);
    let mut E: *mut Enemy_JetPowerRipper = Get_JetPowerRipper(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).base.frozen_timer != 0 {
        v1 = -(7105 as ::core::ffi::c_int) as int16;
        if ((*E).jprr_var_D as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) == 0i32 {
            v1 = -(7093 as ::core::ffi::c_int) as int16;
        }
        (*E).base.spritemap_pointer = v1 as uint16;
    }
}
pub unsafe fn Ripper_Init(game: &mut Game) {
    let mut v0: uint16 = addr_kRipper_Ilist_E477 as uint16;
    let mut E: *mut Enemy_Ripper = Get_Ripper(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).ripper_parameter_2 == 0 {
        v0 = addr_kRipper_Ilist_E48B as uint16;
    }
    (*E).base.current_instruction = v0;
    let mut v2: uint16 = (8u16).wrapping_mul((*E).ripper_parameter_1);
    (*E).ripper_var_E = v2;
    let mut v3: ::core::ffi::c_int = v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let mut v4: uint16 = 0;
    if (*E).ripper_parameter_2 != 0 {
        (*E).ripper_var_D = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8
            as *mut uint16)
            .offset(v3 as isize);
        v4 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8
            as *mut uint16)
            .offset((v3 + 1 as ::core::ffi::c_int) as isize);
    } else {
        (*E).ripper_var_D = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8
            as *mut uint16)
            .offset((v3 + 2 as ::core::ffi::c_int) as isize);
        v4 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8
            as *mut uint16)
            .offset((v3 + 3 as ::core::ffi::c_int) as isize);
    }
    (*E).ripper_var_C = v4;
}
pub unsafe fn Ripper_Main(game: &mut Game) {
    let mut v3: int16 = 0;
    let mut E: *mut Enemy_Ripper = Get_Ripper(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if Enemy_MoveRight_IgnoreSlopes(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        (((*E).ripper_var_D as uint32) << 16 as ::core::ffi::c_int | (*E).ripper_var_C as uint32)
            as int32,
        game,
    ) != 0
    {
        let mut v2: ::core::ffi::c_int =
            (*E).ripper_var_E as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        if ((*E).ripper_var_D as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) == 0i32 {
            (*E).ripper_var_D = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187))
                as *const uint8 as *mut uint16)
                .offset((v2 + 2 as ::core::ffi::c_int) as isize);
            (*E).ripper_var_C = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187))
                as *const uint8 as *mut uint16)
                .offset((v2 + 3 as ::core::ffi::c_int) as isize);
            v3 = -(7029 as ::core::ffi::c_int) as int16;
        } else {
            (*E).ripper_var_D = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187))
                as *const uint8 as *mut uint16)
                .offset(v2 as isize);
            (*E).ripper_var_C = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187))
                as *const uint8 as *mut uint16)
                .offset((v2 + 1 as ::core::ffi::c_int) as isize);
            v3 = -(7049 as ::core::ffi::c_int) as int16;
        }
        (*E).base.current_instruction = v3 as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
    }
}
pub unsafe fn LavaSeahorse_Instr_E5FB(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_LavaSeahorse(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .lse_var_02 = 1 as uint16;
    return jp;
}
pub unsafe fn LavaSeahorse_Init(game: &mut Game) {
    let mut E: *mut Enemy_LavaSeahorse = Get_LavaSeahorse(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).lse_var_02 = 0 as uint16;
    if (*E).lse_parameter_1 != 0 {
        (*E).lse_var_00 = 2 as uint16;
        (*E).lse_var_01 = 2 as uint16;
        (*E).base.current_instruction = addr_kLavaSeahorse_Ilist_E5A1 as uint16;
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
        (*E).lse_var_F = ((0xa2e781i32) as u16) & (0xffffu16);
    } else {
        (*E).lse_var_00 = 0 as uint16;
        (*E).lse_var_01 = 0 as uint16;
        (*E).base.current_instruction = addr_kLavaSeahorse_Ilist_E59B as uint16;
        (*E).lse_var_F = ((0xa2e654i32) as u16) & (0xffffu16);
    };
}
pub unsafe fn LavaSeahorse_Main(game: &mut Game) {
    let mut E: *mut Enemy_LavaSeahorse = Get_LavaSeahorse(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    CallEnemyPreInstr((((*E).lse_var_F) as u32) | (0xa20000u32), game);
}
pub unsafe fn LavaSeahorse_Func_1(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_LavaSeahorse = Get_LavaSeahorse(k as ::core::ffi::c_int, game);
    (*E).lse_var_D = (*E).lse_var_D.wrapping_sub(1);
    if ((*E).lse_var_D as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        (*E).lse_var_D = 48 as uint16;
        (*E).lse_var_F = ((0xa2e6adi32) as u16) & (0xffffu16);
        let mut t: uint16 = GetSamusEnemyDelta_X(k, game);
        (*E).lse_var_A = ((*E).lse_var_A as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int
            | t as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int)
            as uint16;
        if ((*E).lse_var_A as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
            let ref mut fresh16 = *(&raw mut (*E).lse_var_00 as *mut uint8).offset(0);
            *fresh16 = (*fresh16 as ::core::ffi::c_int & !(1 as ::core::ffi::c_int)) as uint8;
            let mut E1: *mut Enemy_LavaSeahorse =
                Get_LavaSeahorse(k as ::core::ffi::c_int + 64 as ::core::ffi::c_int, game);
            let ref mut fresh17 = *(&raw mut (*E1).lse_var_00 as *mut uint8).offset(0);
            *fresh17 = (*fresh17 as ::core::ffi::c_int & !(1 as ::core::ffi::c_int)) as uint8;
        } else {
            let ref mut fresh18 = *(&raw mut (*E).lse_var_00 as *mut uint8).offset(0);
            *fresh18 = (*fresh18) | (1u8);
            let mut E1_0: *mut Enemy_LavaSeahorse =
                Get_LavaSeahorse(k as ::core::ffi::c_int + 64 as ::core::ffi::c_int, game);
            let ref mut fresh19 = *(&raw mut (*E1_0).lse_var_00 as *mut uint8).offset(0);
            *fresh19 = (*fresh19) | (1u8);
        }
        LavaSeahorse_Func_6(game);
        LavaSeahorse_Func_7(game);
    }
}
pub unsafe fn LavaSeahorse_Func_2(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_LavaSeahorse = Get_LavaSeahorse(k as ::core::ffi::c_int, game);
    (*E).lse_var_D = (*E).lse_var_D.wrapping_sub(1);
    if ((*E).lse_var_D as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        (*E).lse_var_00 = ((*E).lse_var_00).wrapping_add(4u16);
        (*E).lse_var_D = 3 as uint16;
        (*E).lse_var_F = ((0xa2e6f1i32) as u16) & (0xffffu16);
    }
    (*E).base.y_pos = (*E).base.y_pos.wrapping_sub(1);
    let mut E1: *mut Enemy_LavaSeahorse =
        Get_LavaSeahorse(k as ::core::ffi::c_int + 64 as ::core::ffi::c_int, game);
    (*E1).base.y_pos = (*E1).base.y_pos.wrapping_sub(1);
}
pub unsafe fn LavaSeahorse_Func_3(mut k: uint16, game: &mut Game) {
    LavaSeahorse_Func_6(game);
    let mut E: *mut Enemy_LavaSeahorse = Get_LavaSeahorse(k as ::core::ffi::c_int, game);
    if (*E).lse_var_02 != 0 {
        (*E).lse_var_02 = 0 as uint16;
        (*E).lse_var_01 = -(1 as ::core::ffi::c_int) as uint16;
        SpawnEprojWithGfx(
            0xffff as uint16,
            k,
            addr_kEproj_LavaSeahorseFireball as uint16,
            game,
        );
        QueueSfx2_Max6(0x61 as uint16, game);
        let fresh20 = (*E).lse_var_D;
        (*E).lse_var_D = (*E).lse_var_D.wrapping_sub(1);
        if fresh20 == 1u16 {
            (*E).lse_var_00 = ((*E).lse_var_00).wrapping_sub(4u16);
            (*E).lse_var_D = 96 as uint16;
            (*E).lse_var_F = ((0xa2e734i32) as u16) & (0xffffu16);
        }
    }
}
pub unsafe fn LavaSeahorse_Func_4(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_LavaSeahorse = Get_LavaSeahorse(k as ::core::ffi::c_int, game);
    let fresh21 = (*E).lse_var_D;
    (*E).lse_var_D = (*E).lse_var_D.wrapping_sub(1);
    if fresh21 == 1u16 {
        (*E).lse_var_D = 48 as uint16;
        (*E).lse_var_F = ((0xa2e749i32) as u16) & (0xffffu16);
        LavaSeahorse_Func_6(game);
    }
}
pub unsafe fn LavaSeahorse_Func_5(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_LavaSeahorse = Get_LavaSeahorse(k as ::core::ffi::c_int, game);
    (*E).lse_var_D = (*E).lse_var_D.wrapping_sub(1);
    if ((*E).lse_var_D as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        (*E).lse_var_D = 128 as uint16;
        (*E).lse_var_F = ((0xa2e654i32) as u16) & (0xffffu16);
    }
    (*E).base.y_pos = (*E).base.y_pos.wrapping_add(1);
    let mut E1: *mut Enemy_LavaSeahorse =
        Get_LavaSeahorse(k as ::core::ffi::c_int + 64 as ::core::ffi::c_int, game);
    (*E1).base.y_pos = (*E1).base.y_pos.wrapping_add(1);
}
pub unsafe fn LavaSeahorse_Func_6(game: &mut Game) {
    let mut E: *mut Enemy_LavaSeahorse = Get_LavaSeahorse(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut lse_var_00: uint16 = (*E).lse_var_00;
    if lse_var_00 != (*E).lse_var_01 {
        (*E).lse_var_01 = lse_var_00;
        (*E).base.current_instruction = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa2e5ef))
            as *const uint8 as *mut uint16)
            .offset(lse_var_00 as isize);
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
    }
}
pub unsafe fn LavaSeahorse_Func_7(game: &mut Game) {
    let mut E: *mut Enemy_LavaSeahorse = Get_LavaSeahorse(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int,
        game,
    );
    let mut lse_var_00: uint16 = (*E).lse_var_00;
    if lse_var_00 != (*E).lse_var_01 {
        (*E).lse_var_01 = lse_var_00;
        (*E).base.current_instruction = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa2e5ef))
            as *const uint8 as *mut uint16)
            .offset(lse_var_00 as isize);
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
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
    let mut E: *mut Enemy_LavaSeahorse = Get_LavaSeahorse(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).base.health != 0 {
        let mut shake_timer: uint16 = (*E).base.shake_timer;
        let mut E1: *mut Enemy_LavaSeahorse = Get_LavaSeahorse(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int
                + 64 as ::core::ffi::c_int,
            game,
        );
        (*E1).base.shake_timer = shake_timer;
        (*E1).base.invincibility_timer = (*E).base.invincibility_timer;
        (*E1).base.flash_timer = (*E).base.flash_timer;
        (*E1).base.frozen_timer = (*E).base.frozen_timer;
        (*E1).base.ai_handler_bits = (*E).base.ai_handler_bits;
    } else {
        let mut E1_0: *mut Enemy_LavaSeahorse = Get_LavaSeahorse(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int
                + 64 as ::core::ffi::c_int,
            game,
        );
        (*E1_0).base.properties = ((*E1_0).base.properties) | (0x200u16);
    };
}
pub unsafe fn TimedShutter_Init(game: &mut Game) {
    let mut E: *mut Enemy_TimedShutter = Get_TimedShutter(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).tsr_var_A = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa2ea4e)) as *const uint8
        as *mut uint16)
        .offset(
            ((*E).base.current_instruction as ::core::ffi::c_int
                + 2 as ::core::ffi::c_int * (*E).base.extra_properties as ::core::ffi::c_int)
                as isize,
        );
    let mut y_pos: uint16 = (*E).base.y_pos;
    let mut v3: uint16 = 0;
    if (*E).base.extra_properties != 0 {
        (*E).tsr_var_B = y_pos;
        let mut v2: uint16 = (y_pos).wrapping_sub(8u16);
        (*E).tsr_var_C = v2;
        v2 = (v2).wrapping_sub(8u16);
        (*E).tsr_var_D = v2;
        v3 = (v2).wrapping_sub(8u16);
    } else {
        (*E).tsr_var_B = y_pos;
        let mut v4: uint16 = (y_pos).wrapping_add(8u16);
        (*E).tsr_var_C = v4;
        v4 = (v4).wrapping_add(8u16);
        (*E).tsr_var_D = v4;
        v3 = (v4).wrapping_add(8u16);
    }
    (*E).tsr_var_E = v3;
    (*E).base.extra_properties = 0 as uint16;
    (*E).tsr_var_F = 0 as uint16;
    (*E).base.current_instruction = addr_kTimedShutter_Ilist_E998 as uint16;
    let mut v5: ::core::ffi::c_int = ((4u16)
        .wrapping_mul((*(&raw mut (*E).tsr_parameter_2 as *mut uint8).offset(0)) as u16))
        as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    (*E).tsr_var_00 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa2ea56)) as *const uint8
        as *mut uint16)
        .offset(v5 as isize);
    (*E).tsr_var_01 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa2ea56)) as *const uint8
        as *mut uint16)
        .offset((v5 + 1 as ::core::ffi::c_int) as isize);
}
pub unsafe fn CallTimedShutterFunc(mut ea: uint32, mut k: uint16, game: &mut Game) {
    match ea {
        10676925 => {
            TimedShutter_Func_1(k, game);
            return;
        }
        10676945 => {
            TimedShutter_Func_2(k, game);
            return;
        }
        10676967 => {
            TimedShutter_Func_3(k, game);
            return;
        }
        10676989 => {
            TimedShutter_Func_4(k, game);
            return;
        }
        10677009 => {
            TimedShutter_Func_5(k, game);
            return;
        }
        10677267 => {
            TimedShutter_Func_10(k, game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn TimedShutter_Main(game: &mut Game) {
    let mut E: *mut Enemy_TimedShutter = Get_TimedShutter(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    CallTimedShutterFunc(
        (((*E).tsr_var_A) as u32) | (0xa20000u32),
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
}
pub unsafe fn TimedShutter_Func_1(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_TimedShutter = Get_TimedShutter(k as ::core::ffi::c_int, game);
    let mut tsr_parameter_1: uint16 = (*E).tsr_parameter_1;
    if tsr_parameter_1 != 0 {
        (*E).tsr_parameter_1 = (tsr_parameter_1).wrapping_sub(1u16);
    } else {
        RisingFallingPlatform_Func_8(game);
        (*E).tsr_var_A = ((0xa2ec13i32) as u16) & (0xffffu16);
    };
}
pub unsafe fn TimedShutter_Func_2(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_TimedShutter = Get_TimedShutter(k as ::core::ffi::c_int, game);
    if IsSamusWithinEnemy_X(k, (*E).tsr_parameter_1, game) != 0 {
        RisingFallingPlatform_Func_8(game);
        (*E).tsr_var_A = ((0xa2ec13i32) as u16) & (0xffffu16);
    }
}
pub unsafe fn TimedShutter_Func_3(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_TimedShutter = Get_TimedShutter(k as ::core::ffi::c_int, game);
    if IsSamusWithinEnemy_X(k, (*E).tsr_parameter_1, game) != 0 {
        RisingFallingPlatform_Func_8(game);
        (*E).tsr_var_A = ((0xa2eb11i32) as u16) & (0xffffu16);
    }
}
pub unsafe fn TimedShutter_Func_4(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_TimedShutter = Get_TimedShutter(k as ::core::ffi::c_int, game);
    let mut tsr_parameter_1: uint16 = (*E).tsr_parameter_1;
    if tsr_parameter_1 != 0 {
        (*E).tsr_parameter_1 = (tsr_parameter_1).wrapping_sub(1u16);
    } else {
        RisingFallingPlatform_Func_8(game);
        (*E).tsr_var_A = ((0xa2eb11i32) as u16) & (0xffffu16);
    };
}
pub fn TimedShutter_Func_Null(game: &mut Game) {}
static off_A2EB1A: [Option<Func_V>; 5] = unsafe {
    [
        Some(TimedShutter_Func_6 as unsafe fn(&mut Game) -> ()),
        Some(TimedShutter_Func_7 as unsafe fn(&mut Game) -> ()),
        Some(TimedShutter_Func_8 as unsafe fn(&mut Game) -> ()),
        Some(TimedShutter_Func_9 as unsafe fn(&mut Game) -> ()),
        Some(TimedShutter_Func_Null as unsafe fn(&mut Game) -> ()),
    ]
};
pub unsafe fn TimedShutter_Func_5(mut k: uint16, game: &mut Game) {
    let mut v1: ::core::ffi::c_int =
        (*Get_TimedShutter(k as ::core::ffi::c_int, game)).tsr_var_F as ::core::ffi::c_int;
    off_A2EB1A[v1 as usize].expect("non-null function pointer")(game);
}
pub unsafe fn TimedShutter_Func_6(game: &mut Game) {
    let mut E: *mut Enemy_TimedShutter = Get_TimedShutter(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        ((*E).tsr_var_00 as uint32) << 16 as ::core::ffi::c_int | (*E).tsr_var_01 as uint32,
    );
    if (((*E).tsr_var_B as ::core::ffi::c_int + 16 as ::core::ffi::c_int
        - (*E).base.y_pos as ::core::ffi::c_int) as int16)
        < 0i16
    {
        (*E).base.y_pos = ((*E).tsr_var_B).wrapping_add(9u16);
        (*E).tsr_var_F = (*E).tsr_var_F.wrapping_add(1);
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.current_instruction = addr_kTimedShutter_Ilist_E99E as uint16;
        (*E).base.y_height = 16 as uint16;
    }
}
pub unsafe fn TimedShutter_Func_7(game: &mut Game) {
    let mut E: *mut Enemy_TimedShutter = Get_TimedShutter(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        ((*E).tsr_var_00 as uint32) << 16 as ::core::ffi::c_int | (*E).tsr_var_01 as uint32,
    );
    if (((*E).tsr_var_C as ::core::ffi::c_int + 16 as ::core::ffi::c_int
        - (*E).base.y_pos as ::core::ffi::c_int) as int16)
        < 0i16
    {
        (*E).base.y_pos = ((*E).tsr_var_C).wrapping_add(9u16);
        (*E).tsr_var_F = (*E).tsr_var_F.wrapping_add(1);
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.current_instruction = addr_kTimedShutter_Ilist_E9A4 as uint16;
        (*E).base.y_height = 24 as uint16;
    }
}
pub unsafe fn TimedShutter_Func_8(game: &mut Game) {
    let mut E: *mut Enemy_TimedShutter = Get_TimedShutter(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        ((*E).tsr_var_00 as uint32) << 16 as ::core::ffi::c_int | (*E).tsr_var_01 as uint32,
    );
    if (((*E).tsr_var_D as ::core::ffi::c_int + 16 as ::core::ffi::c_int
        - (*E).base.y_pos as ::core::ffi::c_int) as int16)
        < 0i16
    {
        (*E).base.y_pos = ((*E).tsr_var_D).wrapping_add(9u16);
        (*E).tsr_var_F = (*E).tsr_var_F.wrapping_add(1);
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.current_instruction = addr_kTimedShutter_Ilist_E9AA as uint16;
        (*E).base.y_height = 32 as uint16;
    }
}
pub unsafe fn TimedShutter_Func_9(game: &mut Game) {
    let mut E: *mut Enemy_TimedShutter = Get_TimedShutter(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        ((*E).tsr_var_00 as uint32) << 16 as ::core::ffi::c_int | (*E).tsr_var_01 as uint32,
    );
    let mut v3: uint16 = ((*E).tsr_var_E).wrapping_add(16u16);
    if ((v3) as i16).wrapping_sub(((*E).base.y_pos) as i16) < 0i16 {
        (*E).base.y_pos = v3;
        (*E).tsr_var_F = (*E).tsr_var_F.wrapping_add(1);
    }
}
pub fn TimedShutter_Func_Null2(game: &mut Game) {}
static off_A2EC3A: [Option<Func_V>; 5] = unsafe {
    [
        Some(TimedShutter_Func_11 as unsafe fn(&mut Game) -> ()),
        Some(TimedShutter_Func_12 as unsafe fn(&mut Game) -> ()),
        Some(TimedShutter_Func_13 as unsafe fn(&mut Game) -> ()),
        Some(TimedShutter_Func_14 as unsafe fn(&mut Game) -> ()),
        Some(TimedShutter_Func_Null2 as unsafe fn(&mut Game) -> ()),
    ]
};
pub unsafe fn TimedShutter_Func_10(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_TimedShutter = Get_TimedShutter(k as ::core::ffi::c_int, game);
    (*E).tsr_var_40 = (*E).base.y_pos;
    let mut v2: uint16 = (2u16).wrapping_mul((*E).tsr_var_F);
    off_A2EC3A[(v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize]
        .expect("non-null function pointer")(game);
    if CheckIfEnemyTouchesSamus(k, game) != 0 {
        let mut v4: int16 = (((*E).base.y_pos) as i16).wrapping_sub(((*E).tsr_var_40) as i16);
        if v4 < 0i16 {
            let ref mut fresh30 =
                *((&raw mut game.g_ram as *mut uint8).offset(0xb5c) as *mut uint16);
            *fresh30 = (*fresh30).wrapping_add((v4) as u16);
        }
    }
}
pub unsafe fn TimedShutter_Func_11(game: &mut Game) {
    let mut E: *mut Enemy_TimedShutter = Get_TimedShutter(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        -(((*E).tsr_var_00 as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
            | (*E).tsr_var_01 as ::core::ffi::c_int) as uint32,
    );
    let mut v3: int16 = (((*E).tsr_var_B) as i16).wrapping_sub(16i16);
    if (v3).wrapping_sub(((*E).base.y_pos) as i16) >= 0i16 {
        (*E).base.y_pos = ((v3) as u16).wrapping_add(7u16);
        (*E).tsr_var_F = (*E).tsr_var_F.wrapping_add(1);
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.current_instruction = addr_kTimedShutter_Ilist_E99E as uint16;
        (*E).base.y_height = 16 as uint16;
    }
}
pub unsafe fn TimedShutter_Func_12(game: &mut Game) {
    let mut E: *mut Enemy_TimedShutter = Get_TimedShutter(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        -(((*E).tsr_var_00 as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
            | (*E).tsr_var_01 as ::core::ffi::c_int) as uint32,
    );
    let mut v3: int16 = (((*E).tsr_var_C) as i16).wrapping_sub(16i16);
    if (v3).wrapping_sub(((*E).base.y_pos) as i16) >= 0i16 {
        (*E).base.y_pos = ((v3) as u16).wrapping_add(7u16);
        (*E).tsr_var_F = (*E).tsr_var_F.wrapping_add(1);
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.current_instruction = addr_kTimedShutter_Ilist_E9A4 as uint16;
        (*E).base.y_height = 24 as uint16;
    }
}
pub unsafe fn TimedShutter_Func_13(game: &mut Game) {
    let mut E: *mut Enemy_TimedShutter = Get_TimedShutter(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        -(((*E).tsr_var_00 as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
            | (*E).tsr_var_01 as ::core::ffi::c_int) as uint32,
    );
    let mut v3: int16 = (((*E).tsr_var_D) as i16).wrapping_sub(16i16);
    if (v3).wrapping_sub(((*E).base.y_pos) as i16) >= 0i16 {
        (*E).base.y_pos = ((v3) as u16).wrapping_add(7u16);
        (*E).tsr_var_F = (*E).tsr_var_F.wrapping_add(1);
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.current_instruction = addr_kTimedShutter_Ilist_E9AA as uint16;
        (*E).base.y_height = 32 as uint16;
    }
}
pub unsafe fn TimedShutter_Func_14(game: &mut Game) {
    let mut E: *mut Enemy_TimedShutter = Get_TimedShutter(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        -(((*E).tsr_var_00 as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
            | (*E).tsr_var_01 as ::core::ffi::c_int) as uint32,
    );
    let mut v3: uint16 = ((*E).tsr_var_E).wrapping_sub(16u16);
    if ((v3) as i16).wrapping_sub(((*E).base.y_pos) as i16) >= 0i16 {
        (*E).base.y_pos = v3;
        (*E).tsr_var_F = (*E).tsr_var_F.wrapping_add(1);
    }
}
pub fn TimedShutter_Func_15() {}
pub unsafe fn RisingFallingPlatform_Init(game: &mut Game) {
    RisingFallingPlatform_Func_1(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
    (*Get_RisingFallingPlatform(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .base
    .current_instruction = addr_kRisingFallingPlatform_Ilist_EDE7 as uint16;
}
pub unsafe fn ShootableShutter_Init(game: &mut Game) {
    RisingFallingPlatform_Func_1(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
    (*gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .current_instruction = addr_kTimedShutter_Ilist_E9AA as uint16;
}
pub unsafe fn RisingFallingPlatform_Func_1(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_RisingFallingPlatform =
        Get_RisingFallingPlatform(k as ::core::ffi::c_int, game);
    let mut current_instruction_low: uint16 =
        *(&raw mut (*E).base.current_instruction as *mut uint8).offset(0) as uint16;
    (*E).rfpm_var_00 = current_instruction_low;
    let mut v3: ::core::ffi::c_int = 8 as ::core::ffi::c_int
        * current_instruction_low as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    (*E).rfpm_var_D = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8
        as *mut uint16)
        .offset(v3 as isize);
    (*E).rfpm_var_C = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8
        as *mut uint16)
        .offset((v3 + 1 as ::core::ffi::c_int) as isize);
    (*E).rfpm_var_F = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8
        as *mut uint16)
        .offset((v3 + 2 as ::core::ffi::c_int) as isize);
    (*E).rfpm_var_E = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8
        as *mut uint16)
        .offset((v3 + 3 as ::core::ffi::c_int) as isize);
    let mut current_instruction_high: uint16 =
        *(&raw mut (*E).base.current_instruction as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1 as usize) as isize,
        ) as uint16;
    (*E).rfpm_var_01 = current_instruction_high;
    (*E).rfpm_var_20 = current_instruction_high;
    let mut extra_properties_low: uint16 =
        *(&raw mut (*E).base.extra_properties as *mut uint8).offset(0) as uint16;
    (*E).rfpm_var_02 = extra_properties_low;
    (*E).rfpm_var_08 = (16u16).wrapping_mul(extra_properties_low);
    let mut extra_properties_high: uint16 = *(&raw mut (*E).base.extra_properties as *mut uint8)
        .offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1 as usize) as isize,
        ) as uint16;
    (*E).rfpm_var_03 = extra_properties_high;
    (*E).rfpm_var_09 = (16u16).wrapping_mul(extra_properties_high);
    let mut rfpm_parameter_1_low: uint16 =
        *(&raw mut (*E).rfpm_parameter_1 as *mut uint8).offset(0) as uint16;
    (*E).rfpm_var_04 = rfpm_parameter_1_low;
    (*E).rfpm_var_07 = (2u16).wrapping_mul(rfpm_parameter_1_low);
    (*E).rfpm_var_05 = *(&raw mut (*E).rfpm_parameter_1 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>())
            .wrapping_div(::core::mem::size_of::<uint8>())
            .wrapping_sub(1 as usize) as isize,
    ) as uint16;
    let mut rfpm_parameter_2: uint16 = (*E).rfpm_parameter_2;
    (*E).rfpm_var_06 = rfpm_parameter_2;
    (*E).rfpm_var_B = rfpm_parameter_2;
    let mut y_pos: uint16 = (*E).base.y_pos;
    (*E).rfpm_var_0F = y_pos;
    (*E).rfpm_var_10 = ((*E).rfpm_var_05).wrapping_add(y_pos);
    if (*E).rfpm_var_01 == 0 {
        let mut v10: uint16 = (*E).base.y_pos;
        (*E).rfpm_var_10 = v10;
        (*E).rfpm_var_0F = (v10).wrapping_sub((*E).rfpm_var_05);
    }
    (*E).rfpm_var_A = ((0xa2ef09i32) as u16) & (0xffffu16);
    (*E).base.extra_properties = 0 as uint16;
    (*E).rfpm_var_0A = 0 as uint16;
}
pub unsafe fn CallRisingFallingPlatformFunc(mut ea: uint32, game: &mut Game) {
    match ea {
        10678025 => {
            RisingFallingPlatform_Func_2(game);
            return;
        }
        10678056 => {
            RisingFallingPlatform_Func_4(game);
            return;
        }
        10678120 => {
            RisingFallingPlatform_Func_9(game);
            return;
        }
        10678228 => {
            RisingFallingPlatform_Func_10(game);
            return;
        }
        10678336 => {
            RisingFallingPlatform_Func_11(game);
            return;
        }
        10678386 => {
            RisingFallingPlatform_Func_12(game);
            return;
        }
        10678425 => {
            RisingFallingPlatform_Func_13();
            return;
        }
        10678820 => {
            HorizontalShootableShutter_Func_2(game);
            return;
        }
        10678851 => {
            HorizontalShootableShutter_Func_4(game);
            return;
        }
        10678898 => {
            HorizontalShootableShutter_Func_8(game);
            return;
        }
        10679012 => {
            HorizontalShootableShutter_Func_9(game);
            return;
        }
        10679180 => {
            HorizontalShootableShutter_Func_12(game);
            return;
        }
        10679216 => {
            HorizontalShootableShutter_Func_13(game);
            return;
        }
        10679252 => {
            HorizontalShootableShutter_Func_14();
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn RisingFallingPlatform_Main(game: &mut Game) {
    let mut E: *mut Enemy_RisingFallingPlatform = Get_RisingFallingPlatform(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    CallRisingFallingPlatformFunc((((*E).rfpm_var_A) as u32) | (0xa20000u32), game);
    if (*E).rfpm_var_A as ::core::ffi::c_int
        != 0xa2ef68 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
        && (*E).rfpm_var_A as ::core::ffi::c_int
            != 0xa2efd4 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
        && (*((&raw mut game.g_ram as *mut uint8).offset(0x182c) as *mut uint16).offset(3)
            as ::core::ffi::c_int
            & (*((&raw mut game.g_ram as *mut uint8).offset(0x182c) as *mut uint16).offset(2)
                as ::core::ffi::c_int
                & *((&raw mut game.g_ram as *mut uint8).offset(0x182c) as *mut uint16).offset(1)
                    as ::core::ffi::c_int
                & *((&raw mut game.g_ram as *mut uint8).offset(0x182c) as *mut uint16).offset(0)
                    as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int)
            != 0xffffi32
        && *((&raw mut game.g_ram as *mut uint8).offset(0x182c) as *mut uint16).offset(3)
            as ::core::ffi::c_int
            & (*((&raw mut game.g_ram as *mut uint8).offset(0x182c) as *mut uint16).offset(2)
                as ::core::ffi::c_int
                & *((&raw mut game.g_ram as *mut uint8).offset(0x182c) as *mut uint16).offset(1)
                    as ::core::ffi::c_int
                & *((&raw mut game.g_ram as *mut uint8).offset(0x182c) as *mut uint16).offset(0)
                    as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
            == *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0xa6e) as *mut uint16) != 0 {
            RisingFallingPlatform_Powerbomb(game);
        }
    }
}
static off_A2EDFB: [Option<Func_V>; 5] = unsafe {
    [
        Some(RisingFallingPlatform_Func_3 as unsafe fn(&mut Game) -> ()),
        Some(RisingFallingPlatform_Func_4 as unsafe fn(&mut Game) -> ()),
        Some(RisingFallingPlatform_Func_5 as unsafe fn(&mut Game) -> ()),
        Some(RisingFallingPlatform_Func_6 as unsafe fn(&mut Game) -> ()),
        Some(RisingFallingPlatform_Func_6 as unsafe fn(&mut Game) -> ()),
    ]
};
pub unsafe fn RisingFallingPlatform_Func_2(game: &mut Game) {
    let mut v0: ::core::ffi::c_int = (*Get_RisingFallingPlatform(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .rfpm_var_07 as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    off_A2EDFB[v0 as usize].expect("non-null function pointer")(game);
}
pub unsafe fn RisingFallingPlatform_Func_3(game: &mut Game) {
    let mut E: *mut Enemy_RisingFallingPlatform = Get_RisingFallingPlatform(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let fresh15 = (*E).rfpm_var_B;
    (*E).rfpm_var_B = (*E).rfpm_var_B.wrapping_sub(1);
    if fresh15 == 1u16 {
        (*E).rfpm_var_B = (*E).rfpm_var_06;
        RisingFallingPlatform_Func_7(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        );
    }
}
pub unsafe fn RisingFallingPlatform_Func_4(game: &mut Game) {
    let mut E: *mut Enemy_RisingFallingPlatform = Get_RisingFallingPlatform(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if IsSamusWithinEnemy_X(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        (*E).rfpm_var_06,
        game,
    ) != 0
    {
        RisingFallingPlatform_Func_7(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        );
    }
}
pub unsafe fn RisingFallingPlatform_Func_5(game: &mut Game) {
    RisingFallingPlatform_Func_7(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
}
pub fn RisingFallingPlatform_Func_6(game: &mut Game) {}
pub unsafe fn RisingFallingPlatform_Func_7(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_RisingFallingPlatform =
        Get_RisingFallingPlatform(k as ::core::ffi::c_int, game);
    (*E).rfpm_var_A = ((0xa2ef68i32) as u16) & (0xffffu16);
    if (*E).rfpm_var_01 != 0 {
        (*E).rfpm_var_A = ((0xa2efd4i32) as u16) & (0xffffu16);
    }
    RisingFallingPlatform_Func_8(game);
}
pub unsafe fn RisingFallingPlatform_Func_8(game: &mut Game) {
    if CheckIfEnemyIsOnScreen(game) == 0 {
        QueueSfx3_Max6(0xe as uint16, game);
    }
}
pub unsafe fn RisingFallingPlatform_Func_9(game: &mut Game) {
    let mut E: *mut Enemy_RisingFallingPlatform = Get_RisingFallingPlatform(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).rfpm_var_0E = (*E).base.y_pos;
    (*E).rfpm_var_0A = 0 as uint16;
    if CheckIfEnemyTouchesSamus(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    ) != 0
    {
        (*E).rfpm_var_0A = 1 as uint16;
    }
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        ((*E).rfpm_var_F as uint32) << 16 as ::core::ffi::c_int | (*E).rfpm_var_E as uint32,
    );
    if (*E).rfpm_var_0A != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0xb5c) as *mut uint16) =
            ((*E).base.y_pos).wrapping_sub((*E).rfpm_var_0E);
    }
    if (((*E).rfpm_var_0F) as i16).wrapping_sub(((*E).base.y_pos) as i16) >= 0i16 {
        if (*E).rfpm_var_08 == 4080u16 {
            (*E).rfpm_var_A = ((0xa2f099i32) as u16) & (0xffffu16);
        } else {
            (*E).rfpm_var_B = (*E).rfpm_var_08;
            (*E).rfpm_var_A = ((0xa2f040i32) as u16) & (0xffffu16);
        }
    }
}
pub unsafe fn RisingFallingPlatform_Func_10(game: &mut Game) {
    let mut E: *mut Enemy_RisingFallingPlatform = Get_RisingFallingPlatform(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).rfpm_var_0E = (*E).base.y_pos;
    (*E).rfpm_var_0A = 0 as uint16;
    if CheckIfEnemyTouchesSamus(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    ) != 0
    {
        (*E).rfpm_var_0A = 1 as uint16;
    }
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        ((*E).rfpm_var_D as uint32) << 16 as ::core::ffi::c_int | (*E).rfpm_var_C as uint32,
    );
    if (*E).rfpm_var_0A != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0xb5c) as *mut uint16) =
            ((*E).base.y_pos).wrapping_sub((*E).rfpm_var_0E);
    }
    if (((*E).base.y_pos) as i16).wrapping_sub(((*E).rfpm_var_10) as i16) >= 0i16 {
        if (*E).rfpm_var_09 == 4080u16 {
            (*E).rfpm_var_A = ((0xa2f099i32) as u16) & (0xffffu16);
        } else {
            (*E).rfpm_var_B = (*E).rfpm_var_09;
            (*E).rfpm_var_A = ((0xa2f072i32) as u16) & (0xffffu16);
        }
    }
}
pub unsafe fn RisingFallingPlatform_Func_11(game: &mut Game) {
    let mut E: *mut Enemy_RisingFallingPlatform = Get_RisingFallingPlatform(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).rfpm_var_B = (*E).rfpm_var_B.wrapping_sub(1);
    if ((*E).rfpm_var_B as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        (*E).rfpm_var_A = ((0xa2efd4i32) as u16) & (0xffffu16);
        if (*E).rfpm_var_04 == 1u16 && (*E).rfpm_var_01 != 0u16 {
            (*E).rfpm_var_A = ((0xa2ef28i32) as u16) & (0xffffu16);
        }
        if (*Get_RisingFallingPlatform(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        ))
        .base
        .enemy_ptr
            != 0xd83fu16
        {
            RisingFallingPlatform_Func_8(game);
        }
    }
}
pub unsafe fn RisingFallingPlatform_Func_12(game: &mut Game) {
    let mut E: *mut Enemy_RisingFallingPlatform = Get_RisingFallingPlatform(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).rfpm_var_B = (*E).rfpm_var_B.wrapping_sub(1);
    if ((*E).rfpm_var_B as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        RisingFallingPlatform_Func_8(game);
        (*E).rfpm_var_A = ((0xa2ef68i32) as u16) & (0xffffu16);
        if (*E).rfpm_var_04 == 1u16 && (*E).rfpm_var_01 == 0 {
            (*E).rfpm_var_A = ((0xa2ef28i32) as u16) & (0xffffu16);
        }
    }
}
pub fn RisingFallingPlatform_Func_13() {}
pub unsafe fn RisingFallingPlatform_Touch(game: &mut Game) {
    RisingFallingPlatform_Powerbomb(game);
}
pub unsafe fn RisingFallingPlatform_Shot(game: &mut Game) {
    RisingFallingPlatform_Powerbomb(game);
}
pub unsafe fn ShootableShutter_Shot(game: &mut Game) {
    NormalEnemyShotAi(game);
    RisingFallingPlatform_Powerbomb(game);
}
pub unsafe fn RisingFallingPlatform_Powerbomb(game: &mut Game) {
    let mut E: *mut Enemy_RisingFallingPlatform = Get_RisingFallingPlatform(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if !((*E).rfpm_var_07 as ::core::ffi::c_int - 6 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0)
    {
        if (*E).rfpm_var_07 != 8u16 {
            if (*E).rfpm_var_0C != 0 {
                return;
            }
            (*E).rfpm_var_0C = 1 as uint16;
        }
        if (*E).rfpm_var_A as ::core::ffi::c_int
            != 0xa2ef68 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
            && (*E).rfpm_var_A as ::core::ffi::c_int
                != 0xa2efd4 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
        {
            (*E).rfpm_var_A = ((0xa2ef68i32) as u16) & (0xffffu16);
            if (*E).rfpm_var_20 != 0 {
                (*E).rfpm_var_A = ((0xa2efd4i32) as u16) & (0xffffu16);
            }
            (*E).rfpm_var_20 = ((*E).rfpm_var_20) ^ (1u16);
        } else {
            return;
        }
    }
    RisingFallingPlatform_Func_8(game);
}
pub unsafe fn HorizontalShootableShutter_Init(game: &mut Game) {
    HorizontalShootableShutter_Func_1(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
    (*Get_HorizontalShootableShutter(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .base
    .current_instruction = addr_kTimedShutter_Ilist_E9D4 as uint16;
}
pub unsafe fn HorizontalShootableShutter_Func_1(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_RisingFallingPlatform =
        Get_RisingFallingPlatform(k as ::core::ffi::c_int, game);
    let mut current_instruction_low: uint16 =
        *(&raw mut (*E).base.current_instruction as *mut uint8).offset(0) as uint16;
    (*E).rfpm_var_00 = current_instruction_low;
    let mut v3: ::core::ffi::c_int = 8 as ::core::ffi::c_int
        * current_instruction_low as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    (*E).rfpm_var_D = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8
        as *mut uint16)
        .offset(v3 as isize);
    (*E).rfpm_var_C = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8
        as *mut uint16)
        .offset((v3 + 1 as ::core::ffi::c_int) as isize);
    (*E).rfpm_var_F = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8
        as *mut uint16)
        .offset((v3 + 2 as ::core::ffi::c_int) as isize);
    (*E).rfpm_var_E = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8
        as *mut uint16)
        .offset((v3 + 3 as ::core::ffi::c_int) as isize);
    let mut current_instruction_high: uint16 =
        *(&raw mut (*E).base.current_instruction as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1 as usize) as isize,
        ) as uint16;
    (*E).rfpm_var_01 = current_instruction_high;
    (*E).rfpm_var_20 = (current_instruction_high) ^ (1u16);
    let mut extra_properties_low: uint16 =
        *(&raw mut (*E).base.extra_properties as *mut uint8).offset(0) as uint16;
    (*E).rfpm_var_02 = extra_properties_low;
    (*E).rfpm_var_08 = (16u16).wrapping_mul(extra_properties_low);
    let mut extra_properties_high: uint16 = *(&raw mut (*E).base.extra_properties as *mut uint8)
        .offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1 as usize) as isize,
        ) as uint16;
    (*E).rfpm_var_03 = extra_properties_high;
    (*E).rfpm_var_09 = (16u16).wrapping_mul(extra_properties_high);
    let mut rfpm_parameter_1_low: uint16 =
        *(&raw mut (*E).rfpm_parameter_1 as *mut uint8).offset(0) as uint16;
    (*E).rfpm_var_04 = rfpm_parameter_1_low;
    (*E).rfpm_var_07 = (2u16).wrapping_mul(rfpm_parameter_1_low);
    (*E).rfpm_var_05 = *(&raw mut (*E).rfpm_parameter_1 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>())
            .wrapping_div(::core::mem::size_of::<uint8>())
            .wrapping_sub(1 as usize) as isize,
    ) as uint16;
    let mut rfpm_parameter_2: uint16 = (*E).rfpm_parameter_2;
    (*E).rfpm_var_06 = rfpm_parameter_2;
    (*E).rfpm_var_B = rfpm_parameter_2;
    let mut x_pos: uint16 = (*E).base.x_pos;
    (*E).rfpm_var_11 = x_pos;
    (*E).rfpm_var_12 = ((*E).rfpm_var_05).wrapping_add(x_pos);
    if (*E).rfpm_var_01 == 0 {
        let mut v10: uint16 = (*E).base.x_pos;
        (*E).rfpm_var_12 = v10;
        (*E).rfpm_var_11 = (v10).wrapping_sub((*E).rfpm_var_05);
    }
    (*E).rfpm_var_A = ((0xa2f224i32) as u16) & (0xffffu16);
    (*E).rfpm_var_15 = *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16);
    (*E).base.extra_properties = 0 as uint16;
    (*E).rfpm_var_0A = 0 as uint16;
    (*E).rfpm_var_0B = 0 as uint16;
}
pub unsafe fn HorizontalShootableShutter_Main(game: &mut Game) {
    let mut E: *mut Enemy_RisingFallingPlatform = Get_RisingFallingPlatform(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    CallRisingFallingPlatformFunc((((*E).rfpm_var_A) as u32) | (0xa20000u32), game);
    if (*E).rfpm_var_A as ::core::ffi::c_int
        != 0xa2f272 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
        && (*E).rfpm_var_A as ::core::ffi::c_int
            != 0xa2f2e4 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
        && (*((&raw mut game.g_ram as *mut uint8).offset(0x182c) as *mut uint16).offset(3)
            as ::core::ffi::c_int
            & (*((&raw mut game.g_ram as *mut uint8).offset(0x182c) as *mut uint16).offset(2)
                as ::core::ffi::c_int
                & *((&raw mut game.g_ram as *mut uint8).offset(0x182c) as *mut uint16).offset(1)
                    as ::core::ffi::c_int
                & *((&raw mut game.g_ram as *mut uint8).offset(0x182c) as *mut uint16).offset(0)
                    as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int)
            != 0xffffi32
        && *((&raw mut game.g_ram as *mut uint8).offset(0x182c) as *mut uint16).offset(3)
            as ::core::ffi::c_int
            & (*((&raw mut game.g_ram as *mut uint8).offset(0x182c) as *mut uint16).offset(2)
                as ::core::ffi::c_int
                & *((&raw mut game.g_ram as *mut uint8).offset(0x182c) as *mut uint16).offset(1)
                    as ::core::ffi::c_int
                & *((&raw mut game.g_ram as *mut uint8).offset(0x182c) as *mut uint16).offset(0)
                    as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
            == *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int
        && *((&raw mut game.g_ram as *mut uint8).offset(0xa6e) as *mut uint16) != 0u16
    {
        HorizontalShootableShutter_Powerbomb(game);
    }
    (*E).rfpm_var_15 = *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16);
    (*E).rfpm_var_16 = *((&raw mut game.g_ram as *mut uint8).offset(0xaf8) as *mut uint16);
}
static off_A2F107: [Option<Func_V>; 5] = unsafe {
    [
        Some(HorizontalShootableShutter_Func_3 as unsafe fn(&mut Game) -> ()),
        Some(HorizontalShootableShutter_Func_4 as unsafe fn(&mut Game) -> ()),
        Some(HorizontalShootableShutter_Func_5 as unsafe fn(&mut Game) -> ()),
        Some(HorizontalShootableShutter_Func_6 as unsafe fn(&mut Game) -> ()),
        Some(HorizontalShootableShutter_Func_6 as unsafe fn(&mut Game) -> ()),
    ]
};
pub unsafe fn HorizontalShootableShutter_Func_2(game: &mut Game) {
    let mut v0: ::core::ffi::c_int = (*Get_RisingFallingPlatform(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .rfpm_var_07 as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    off_A2F107[v0 as usize].expect("non-null function pointer")(game);
}
pub unsafe fn HorizontalShootableShutter_Func_3(game: &mut Game) {
    let mut E: *mut Enemy_RisingFallingPlatform = Get_RisingFallingPlatform(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let fresh14 = (*E).rfpm_var_B;
    (*E).rfpm_var_B = (*E).rfpm_var_B.wrapping_sub(1);
    if fresh14 == 1u16 {
        (*E).rfpm_var_B = (*E).rfpm_var_06;
        HorizontalShootableShutter_Func_7(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        );
    }
}
pub unsafe fn HorizontalShootableShutter_Func_4(game: &mut Game) {
    let mut E: *mut Enemy_RisingFallingPlatform = Get_RisingFallingPlatform(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if IsSamusWithinEnemy_X(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        (*E).rfpm_var_06,
        game,
    ) != 0
    {
        HorizontalShootableShutter_Func_7(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        );
    }
}
pub unsafe fn HorizontalShootableShutter_Func_5(game: &mut Game) {
    HorizontalShootableShutter_Func_7(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
}
pub fn HorizontalShootableShutter_Func_6(game: &mut Game) {}
pub unsafe fn HorizontalShootableShutter_Func_7(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_RisingFallingPlatform =
        Get_RisingFallingPlatform(k as ::core::ffi::c_int, game);
    (*E).rfpm_var_A = ((0xa2f272i32) as u16) & (0xffffu16);
    if (*E).rfpm_var_01 != 0 {
        (*E).rfpm_var_A = ((0xa2f2e4i32) as u16) & (0xffffu16);
    }
}
pub unsafe fn HorizontalShootableShutter_Func_8(game: &mut Game) {
    let mut E: *mut Enemy_RisingFallingPlatform = Get_RisingFallingPlatform(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).rfpm_var_0D = (*E).base.x_pos;
    (*E).rfpm_var_13 = 0 as uint16;
    if EnemyFunc_AC67(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    ) != 0u16
        && ((*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)) as i16)
            .wrapping_sub(((*E).base.x_pos) as i16)
            < 0i16
    {
        (*E).rfpm_var_13 = 1 as uint16;
    }
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        ((*E).rfpm_var_F as uint32) << 16 as ::core::ffi::c_int | (*E).rfpm_var_E as uint32,
    );
    if (*E).rfpm_var_13 != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0xb56) as *mut uint16) = (*E).rfpm_var_E;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb58) as *mut uint16) = (*E).rfpm_var_F;
        HorizontalShootableShutter_Func_10(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        );
    }
    if (((*E).rfpm_var_11) as i16).wrapping_sub(((*E).base.x_pos) as i16) >= 0i16 {
        if (*E).rfpm_var_08 == 4080u16 {
            (*E).rfpm_var_A = ((0xa2f3d4i32) as u16) & (0xffffu16);
        } else {
            (*E).rfpm_var_B = (*E).rfpm_var_08;
            (*E).rfpm_var_A = ((0xa2f38ci32) as u16) & (0xffffu16);
        }
    }
}
pub unsafe fn HorizontalShootableShutter_Func_9(game: &mut Game) {
    let mut E: *mut Enemy_RisingFallingPlatform = Get_RisingFallingPlatform(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).rfpm_var_0D = (*E).base.x_pos;
    (*E).rfpm_var_13 = 0 as uint16;
    if EnemyFunc_AC67(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    ) != 0u16
        && ((*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)) as i16)
            .wrapping_sub(((*E).base.x_pos) as i16)
            >= 0i16
    {
        (*E).rfpm_var_13 = 1 as uint16;
    }
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        ((*E).rfpm_var_D as uint32) << 16 as ::core::ffi::c_int | (*E).rfpm_var_C as uint32,
    );
    if (*E).rfpm_var_13 != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0xb56) as *mut uint16) = (*E).rfpm_var_C;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb58) as *mut uint16) = (*E).rfpm_var_D;
        HorizontalShootableShutter_Func_11(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        );
    }
    if (((*E).base.x_pos) as i16).wrapping_sub(((*E).rfpm_var_12) as i16) >= 0i16 {
        if (*E).rfpm_var_09 == 4080u16 {
            (*E).rfpm_var_A = ((0xa2f3d4i32) as u16) & (0xffffu16);
        } else {
            (*E).rfpm_var_B = (*E).rfpm_var_09;
            (*E).rfpm_var_A = ((0xa2f3b0i32) as u16) & (0xffffu16);
        }
    }
}
pub unsafe fn HorizontalShootableShutter_Func_10(mut k: uint16, game: &mut Game) {
    if (*Get_RisingFallingPlatform(k as ::core::ffi::c_int, game)).rfpm_var_13 != 0 {
        if (*((&raw mut game.g_ram as *mut uint8).offset(0x8b) as *mut uint16)
            as ::core::ffi::c_int
            & 0x100 as ::core::ffi::c_int)
            != 0i32
        {
            let ref mut fresh12 =
                *((&raw mut game.g_ram as *mut uint8).offset(0xb58) as *mut uint16);
            *fresh12 = (*fresh12).wrapping_sub(4u16);
            *((&raw mut game.g_ram as *mut uint8).offset(0xb5a) as *mut uint16) = 0 as uint16;
        }
    }
}
pub unsafe fn HorizontalShootableShutter_Func_11(mut k: uint16, game: &mut Game) {
    if (*Get_RisingFallingPlatform(k as ::core::ffi::c_int, game)).rfpm_var_13 != 0 {
        if (*((&raw mut game.g_ram as *mut uint8).offset(0x8b) as *mut uint16)
            as ::core::ffi::c_int
            & 0x200 as ::core::ffi::c_int)
            != 0i32
        {
            let ref mut fresh13 =
                *((&raw mut game.g_ram as *mut uint8).offset(0xb58) as *mut uint16);
            *fresh13 = (*fresh13).wrapping_add(4u16);
            *((&raw mut game.g_ram as *mut uint8).offset(0xb5a) as *mut uint16) = 0 as uint16;
        }
    }
}
pub unsafe fn HorizontalShootableShutter_Func_12(game: &mut Game) {
    let mut E: *mut Enemy_RisingFallingPlatform = Get_RisingFallingPlatform(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).rfpm_var_B = (*E).rfpm_var_B.wrapping_sub(1);
    if ((*E).rfpm_var_B as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        (*E).rfpm_var_A = ((0xa2f2e4i32) as u16) & (0xffffu16);
        if (*E).rfpm_var_04 == 1u16 {
            if (*E).rfpm_var_01 != 0 {
                (*E).rfpm_var_A = ((0xa2f243i32) as u16) & (0xffffu16);
            }
        }
    }
}
pub unsafe fn HorizontalShootableShutter_Func_13(game: &mut Game) {
    let mut E: *mut Enemy_RisingFallingPlatform = Get_RisingFallingPlatform(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).rfpm_var_B = (*E).rfpm_var_B.wrapping_sub(1);
    if ((*E).rfpm_var_B as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        (*E).rfpm_var_A = ((0xa2f272i32) as u16) & (0xffffu16);
        if (*E).rfpm_var_04 == 1u16 && (*E).rfpm_var_01 == 0 {
            (*E).rfpm_var_A = ((0xa2f243i32) as u16) & (0xffffu16);
        }
    }
}
pub fn HorizontalShootableShutter_Func_14() {}
pub unsafe fn HorizontalShootableShutter_Touch(game: &mut Game) {
    let mut E: *mut Enemy_HorizontalShootableShutter = Get_HorizontalShootableShutter(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).hssr_var_A as ::core::ffi::c_int
        == 0xa2f3d4 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
    {
        if ((*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)) as i16)
            .wrapping_sub(((*E).base.x_pos) as i16)
            >= 0i16
        {
            if (*((&raw mut game.g_ram as *mut uint8).offset(0x8b) as *mut uint16)
                as ::core::ffi::c_int
                & 0x200 as ::core::ffi::c_int)
                != 0i32
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0xb58) as *mut uint16) = 4 as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0xb5a) as *mut uint16) = 0 as uint16;
            }
        } else if (*((&raw mut game.g_ram as *mut uint8).offset(0x8b) as *mut uint16)
            as ::core::ffi::c_int
            & 0x100 as ::core::ffi::c_int)
            != 0i32
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xb58) as *mut uint16) =
                -(4 as ::core::ffi::c_int) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xb5a) as *mut uint16) = 0 as uint16;
        }
    }
}
pub unsafe fn HorizontalShootableShutter_Shot(game: &mut Game) {
    NormalEnemyShotAiSkipDeathAnim_CurEnemy(game);
    HorizontalShootableShutter_Powerbomb(game);
}
pub unsafe fn HorizontalShootableShutter_Powerbomb(game: &mut Game) {
    let mut E: *mut Enemy_RisingFallingPlatform = Get_RisingFallingPlatform(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).rfpm_var_07 as ::core::ffi::c_int - 6 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        if (*E).rfpm_var_07 != 8u16 {
            if (*E).rfpm_var_0C != 0 {
                return;
            }
            (*E).rfpm_var_0C = 1 as uint16;
        }
        if (*E).rfpm_var_A as ::core::ffi::c_int
            != 0xa2f272 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
            && (*E).rfpm_var_A as ::core::ffi::c_int
                != 0xa2f2e4 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
        {
            (*E).rfpm_var_20 = ((*E).rfpm_var_20) ^ (1u16);
            (*E).rfpm_var_A = ((0xa2f272i32) as u16) & (0xffffu16);
            if (*E).rfpm_var_20 != 0 {
                (*E).rfpm_var_A = ((0xa2f2e4i32) as u16) & (0xffffu16);
            }
        }
    }
}
