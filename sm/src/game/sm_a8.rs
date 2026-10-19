pub use crate::game::constants::bank_86::*;
pub use crate::game::constants::enemy_properties::*;
use crate::game::enemy_layouts::enemy_layout;
use crate::game::sm_80::{NextRandom, QueueSfx2_Max6, QueueSfx2_Max9, QueueSfx3_Max6};
use crate::game::sm_86::SpawnEprojWithGfx;
use crate::game::sm_88::SpawnMorphBallEyeBeamHdma;
use crate::game::sm_90::CallSomeSamusCode;
use crate::game::sm_a0::{
    Abs16, CalculateAngleFromXY, CalculateAngleOfSamusFromEnemy, CallEnemyPreInstr,
    CheckIfEnemyIsOnScreen, CheckIfEnemyTouchesSamus, CosineMult8bit, CosineMult8bitFull,
    EnemyDeathAnimation, EnemyFunc_ACA8, EnemyFunc_ADA3, EnemyFunc_B7A1, EnemyFunc_C8AD,
    EnemyGrappleDeath, Enemy_MoveDown, Enemy_MoveRight_IgnoreSlopes, Enemy_SwitchToFrozenAi,
    GetSamusEnemyDelta_X, GetSamusEnemyDelta_Y, IsSamusWithinEnemy_X, IsSamusWithinEnemy_Y,
    NormalEnemyFrozenAI, NormalEnemyPowerBombAi, NormalEnemyShotAi,
    NormalEnemyShotAiSkipDeathAnim_CurEnemy, NormalEnemyTouchAi, SamusHurtFromGrapple,
    SamusLatchesOnWithGrappleNoInvinc, SignExtend8, SineMult8bit, SineMult8bitFull,
    SwitchEnemyAiToMainAi,
};
use crate::game::sm_b4::CreateSpriteAtPos;
use crate::game_state::Game;
use crate::ram_map::ram_word;
use crate::rtl::abs16;
use crate::rtl::{AddToHiLo, SetHiLo};
use crate::rtl::{Mult8x8, RomPtr, Unreachable, WriteReg};
pub use crate::snes::registers::*;
extern "C" {
    fn printf(__format: *const ::core::ffi::c_char, ...) -> ::core::ffi::c_int;

}
pub use crate::types::Consts_86;
pub use crate::types::Consts_A8;
pub use crate::types::{
    __int16_t, __int32_t, __uint16_t, __uint32_t, __uint8_t, int16, int16_t, int32, int32_t,
    uint16, uint16_t, uint32, uint32_t, uint8, uint8_t, uintptr_t, Func_V, LongPtr, PairU16,
    Point16U, SnesRegs, VoidP, VramWriteEntry,
};
pub use crate::types::{EnemyBase, EnemyData, EnemyProps, EnemySpawnData};
pub const addr_locret_A8F5E3: Consts_A8 = 62947;
pub const addr_kKiHunter_Ilist_EB10: Consts_A8 = 60176;
pub const addr_kKiHunter_Ilist_EAF2: Consts_A8 = 60146;
pub const addr_kKiHunter_Ilist_EADA: Consts_A8 = 60122;
pub const addr_kKiHunter_Ilist_EAC2: Consts_A8 = 60098;
pub const addr_kKiHunter_Ilist_EAA6: Consts_A8 = 60070;
pub const addr_kKiHunter_Ilist_EA8A: Consts_A8 = 60042;
pub const addr_kKiHunter_Ilist_EA7E: Consts_A8 = 60030;
pub const addr_kKiHunter_Ilist_EA5E: Consts_A8 = 59998;
pub const addr_kKiHunter_Ilist_EA4E: Consts_A8 = 59982;
pub const addr_kKiHunter_Ilist_EA32: Consts_A8 = 59954;
pub const addr_kKiHunter_Ilist_EA24: Consts_A8 = 59940;
pub const addr_kKiHunter_Ilist_EA08: Consts_A8 = 59912;
pub const addr_kKiHunter_Ilist_E9FA: Consts_A8 = 59898;
pub const addr_kBlueBrinstarFaceBlock_Ilist_E828: Consts_A8 = 59432;
pub const addr_kBlueBrinstarFaceBlock_Ilist_E81A: Consts_A8 = 59418;
pub const addr_kBlueBrinstarFaceBlock_Ilist_E80C: Consts_A8 = 59404;
pub const addr_kWreckedShipSpark_Ilist_E5E5: Consts_A8 = 58853;
pub const addr_kWreckedShipSpark_Ilist_E5A7: Consts_A8 = 58791;
pub const addr_kWalkingLavaSeahorse_Ilist_DCC1: Consts_A8 = 56513;
pub const addr_stru_A8DCBB: Consts_A8 = 56507;
pub const addr_kWalkingLavaSeahorse_Ilist_DC73: Consts_A8 = 56435;
pub const addr_stru_A8DC59: Consts_A8 = 56409;
pub const addr_kWalkingLavaSeahorse_Ilist_DC57: Consts_A8 = 56407;
pub const addr_kWalkingLavaSeahorse_Ilist_DC51: Consts_A8 = 56401;
pub const addr_stru_A8DC4B: Consts_A8 = 56395;
pub const addr_kWalkingLavaSeahorse_Ilist_DC03: Consts_A8 = 56323;
pub const addr_stru_A8DBE9: Consts_A8 = 56297;
pub const addr_kWalkingLavaSeahorse_Ilist_DBE7: Consts_A8 = 56295;
pub const addr_kMaridiaPuffer_Ilist_D855: Consts_A8 = 55381;
pub const addr_kMaridiaPuffer_Ilist_D841: Consts_A8 = 55361;
pub const addr_loc_A8D02E: Consts_A8 = 53294;
pub const addr_loc_A8CF26: Consts_A8 = 53030;
pub const addr_loc_A8CF18: Consts_A8 = 53016;
pub const addr_kWreckedShipRobot_Ilist_CB65: Consts_A8 = 52069;
pub const addr_kWreckedShipRobot_Ilist_CB1D: Consts_A8 = 51997;
pub const addr_kWreckedShipRobot_Ilist_CB09: Consts_A8 = 51977;
pub const addr_kWreckedShipRobot_Ilist_CAFD: Consts_A8 = 51965;
pub const addr_kWreckedShipRobot_Ilist_CA7D: Consts_A8 = 51837;
pub const addr_kWreckedShipRobot_Ilist_CA01: Consts_A8 = 51713;
pub const addr_kWreckedShipRobot_Ilist_C985: Consts_A8 = 51589;
pub const addr_kWreckedShipRobot_Ilist_C92D: Consts_A8 = 51501;
pub const addr_kWreckedShipRobot_Ilist_C91B: Consts_A8 = 51483;
pub const addr_kWreckedShipRobot_Ilist_C8D1: Consts_A8 = 51409;
pub const addr_kWreckedShipRobot_Ilist_C8BD: Consts_A8 = 51389;
pub const addr_kWreckedShipRobot_Ilist_C8B1: Consts_A8 = 51377;
pub const addr_kWreckedShipRobot_Ilist_C833: Consts_A8 = 51251;
pub const addr_kWreckedShipRobot_Ilist_C7BB: Consts_A8 = 51131;
pub const addr_kWreckedShipRobot_Ilist_C73F: Consts_A8 = 51007;
pub const addr_stru_A8C6E9: Consts_A8 = 50921;
pub const addr_kWreckedShipRobot_Ilist_C6E5: Consts_A8 = 50917;
pub const addr_stru_A8C199: Consts_A8 = 49561;
pub const addr_kMaridiaFloater_Ilist_C191: Consts_A8 = 49553;
pub const addr_kMaridiaFloater_Ilist_C183: Consts_A8 = 49539;
pub const addr_kMaridiaFloater_Ilist_C173: Consts_A8 = 49523;
pub const addr_kMaridiaFloater_Ilist_C163: Consts_A8 = 49507;
pub const addr_kBeetom_Ilist_B728: Consts_A8 = 46888;
pub const addr_kBeetom_Ilist_B708: Consts_A8 = 46856;
pub const addr_kBeetom_Ilist_B6F2: Consts_A8 = 46834;
pub const addr_kBeetom_Ilist_B6CC: Consts_A8 = 46796;
pub const addr_kBeetom_Ilist_B6AC: Consts_A8 = 46764;
pub const addr_kBeetom_Ilist_B696: Consts_A8 = 46742;
pub const addr_kNorfairLavaMan_Ilist_AE0C: Consts_A8 = 44556;
pub const addr_kNorfairLavaMan_Ilist_ADDC: Consts_A8 = 44508;
pub const addr_kNorfairLavaMan_Ilist_ADAC: Consts_A8 = 44460;
pub const addr_kNorfairLavaMan_Ilist_AD7E: Consts_A8 = 44414;
pub const addr_kNorfairLavaMan_Ilist_AD50: Consts_A8 = 44368;
pub const addr_kNorfairLavaMan_Ilist_AD3C: Consts_A8 = 44348;
pub const addr_kNorfairLavaMan_Ilist_AD0C: Consts_A8 = 44300;
pub const addr_kNorfairLavaMan_Ilist_ACDE: Consts_A8 = 44254;
pub const addr_kNorfairLavaMan_Ilist_ACB0: Consts_A8 = 44208;
pub const addr_kNorfairLavaMan_Ilist_AC9C: Consts_A8 = 44188;
pub const addr_kKago_Ilist_AB32: Consts_A8 = 43826;
pub const addr_kKago_Ilist_AB1E: Consts_A8 = 43806;
pub const addr_kYappingMaw_Ilist_A079: Consts_A8 = 41081;
pub const addr_off_A8A061: Consts_A8 = 41057;
pub const addr_kYappingMaw_Ilist_A05B: Consts_A8 = 41051;
pub const addr_kYappingMaw_Ilist_A03D: Consts_A8 = 41021;
pub const addr_off_A8A025: Consts_A8 = 40997;
pub const addr_kYappingMaw_Ilist_A01F: Consts_A8 = 40991;
pub const addr_kYappingMaw_Ilist_9FC7: Consts_A8 = 40903;
pub const addr_kYappingMaw_Ilist_9F6F: Consts_A8 = 40815;
pub const addr_loc_A89E90: Consts_A8 = 40592;
pub const addr_kWreckedShipGhost_Ilist_9A8C: Consts_A8 = 39564;
pub const addr_off_A896DF: Consts_A8 = 38623;
pub const addr_off_A896D7: Consts_A8 = 38615;
pub const addr_kMorphBallEye_Ilist_9026: Consts_A8 = 36902;
pub const addr_kMorphBallEye_Ilist_9014: Consts_A8 = 36884;
pub const addr_stru_A8900E: Consts_A8 = 36878;
pub const addr_kMorphBallEye_Ilist_9002: Consts_A8 = 36866;
pub const addr_stru_A88FFC: Consts_A8 = 36860;
pub const addr_kMorphBallEye_Ilist_8FF0: Consts_A8 = 36848;
pub const addr_kMiniDraygon_Ilist_8775: Consts_A8 = 34677;
pub const addr_kMiniDraygon_Ilist_876F: Consts_A8 = 34671;
pub const addr_kMiniDraygon_Ilist_8727: Consts_A8 = 34599;
pub const addr_kMiniDraygon_Ilist_870B: Consts_A8 = 34571;
pub const addr_kMiniDraygon_Ilist_86C3: Consts_A8 = 34499;
pub const addr_kMiniDraygon_Ilist_86A7: Consts_A8 = 34471;
pub const addr_kSpritemap_Nothing_A8: Consts_A8 = 32845;
enemy_layout! {
    Enemy_Fune {
        base: EnemyBase,
        words: [
            fune_var_A, fune_var_B, fune_var_C, fune_var_D, fune_var_E, fune_var_F,
            fune_parameter_1, fune_parameter_2,
        ],
        pad1: [uint8; 26696],
        words: [
            fune_var_00, fune_var_01, fune_var_02, fune_var_03, fune_var_04, fune_var_05,
            fune_var_06, fune_var_07, fune_var_08, fune_var_09, fune_var_0A, fune_var_0B,
            fune_var_0C, fune_var_0D, fune_var_0E, fune_var_0F, fune_var_10, fune_var_11,
            fune_var_12, fune_var_13, fune_var_14, fune_var_15, fune_var_16, fune_var_17,
            fune_var_18, fune_var_19, fune_var_1A, fune_var_1B, fune_var_1C, fune_var_1D,
            fune_var_1E, fune_var_1F,
        ],
    }
}

enemy_layout! {
    Enemy_KiHunter {
        base: EnemyBase,
        words: [
            khr_var_A, khr_var_B, khr_var_C, khr_var_D, khr_var_E, khr_var_F,
            khr_parameter_1, khr_parameter_2,
        ],
        pad1: [uint8; 26696],
        words: [
            khr_var_00, khr_var_01, khr_var_02, khr_var_03, khr_var_04, khr_var_05,
            khr_var_06, khr_var_07, khr_var_08, khr_var_09, khr_var_0A, khr_var_0B,
            khr_var_0C, khr_var_0D, khr_var_0E, khr_var_0F, khr_var_10, khr_var_11,
            khr_var_12, khr_var_13, khr_var_14, khr_var_15, khr_var_16, khr_var_17,
            khr_var_18, khr_var_19, khr_var_1A, khr_var_1B, khr_var_1C, khr_var_1D,
            khr_var_1E, khr_var_1F,
        ],
    }
}

enemy_layout! {
    Enemy_MiniDraygon {
        base: EnemyBase,
        words: [
            mdn_var_A, mdn_var_B, mdn_var_C, mdn_var_D, mdn_var_E, mdn_var_F,
            mdn_parameter_1, mdn_parameter_2,
        ],
        pad1: [uint8; 26696],
        words: [
            mdn_var_00, mdn_var_01, mdn_var_02, mdn_var_03, mdn_var_04, mdn_var_05,
            mdn_var_06, mdn_var_07, mdn_var_08, mdn_var_09, mdn_var_0A, mdn_var_0B,
            mdn_var_0C, mdn_var_0D, mdn_var_0E, mdn_var_0F, mdn_var_10, mdn_var_11,
            mdn_var_12, mdn_var_13, mdn_var_14, mdn_var_15, mdn_var_16, mdn_var_17,
            mdn_var_18, mdn_var_19, mdn_var_1A, mdn_var_1B, mdn_var_1C, mdn_var_1D,
            mdn_var_1E, mdn_var_1F,
        ],
    }
}

enemy_layout! {
    Enemy_NorfairLavaMan {
        base: EnemyBase,
        words: [
            nlmn_var_A, nlmn_var_B, nlmn_var_C, nlmn_var_D, nlmn_var_E, nlmn_var_F,
            nlmn_parameter_1, nlmn_parameter_2,
        ],
        pad1: [uint8; 26696],
        words: [
            nlmn_var_00, nlmn_var_01, nlmn_var_02, nlmn_var_03, nlmn_var_04, nlmn_var_05,
            nlmn_var_06, nlmn_var_07, nlmn_var_08, nlmn_var_09, nlmn_var_0A, nlmn_var_0B,
            nlmn_var_0C, nlmn_var_0D, nlmn_var_0E, nlmn_var_0F, nlmn_var_10, nlmn_var_11,
            nlmn_var_12, nlmn_var_13, nlmn_var_14, nlmn_var_15, nlmn_var_16, nlmn_var_17,
            nlmn_var_18, nlmn_var_19, nlmn_var_1A, nlmn_var_1B, nlmn_var_1C, nlmn_var_1D,
            nlmn_var_1E, nlmn_var_1F,
        ],
    }
}

enemy_layout! {
    Enemy_WalkingLavaSeahorse {
        base: EnemyBase,
        words: [
            wlse_var_A, wlse_var_B, wlse_var_C, wlse_var_D, wlse_var_E, wlse_var_F,
            wlse_parameter_1, wlse_parameter_2,
        ],
        pad1: [uint8; 26696],
        words: [
            wlse_var_00, wlse_var_01, wlse_var_02, wlse_var_03, wlse_var_04, wlse_var_05,
            wlse_var_06, wlse_var_07, wlse_var_08, wlse_var_09, wlse_var_0A, wlse_var_0B,
            wlse_var_0C, wlse_var_0D, wlse_var_0E, wlse_var_0F, wlse_var_10, wlse_var_11,
            wlse_var_12, wlse_var_13, wlse_var_14, wlse_var_15, wlse_var_16, wlse_var_17,
            wlse_var_18, wlse_var_19, wlse_var_1A, wlse_var_1B, wlse_var_1C, wlse_var_1D,
            wlse_var_1E, wlse_var_1F,
        ],
    }
}

enemy_layout! {
    Enemy_WreckedShipGhost {
        base: EnemyBase,
        words: [
            wsgt_var_A, wsgt_var_B, wsgt_var_C, wsgt_var_D, wsgt_var_E, wsgt_var_F,
            wsgt_parameter_1, wsgt_parameter_2,
        ],
        pad1: [uint8; 26696],
        words: [
            wsgt_var_00, wsgt_var_01, wsgt_var_02, wsgt_var_03, wsgt_var_04, wsgt_var_05,
            wsgt_var_06, wsgt_var_07, wsgt_var_08, wsgt_var_09, wsgt_var_0A, wsgt_var_0B,
            wsgt_var_0C, wsgt_var_0D, wsgt_var_0E, wsgt_var_0F, wsgt_var_10, wsgt_var_11,
            wsgt_var_12, wsgt_var_13, wsgt_var_14, wsgt_var_15, wsgt_var_16, wsgt_var_17,
            wsgt_var_18, wsgt_var_19, wsgt_var_1A, wsgt_var_1B, wsgt_var_1C, wsgt_var_1D,
            wsgt_var_1E, wsgt_var_1F,
        ],
    }
}

enemy_layout! {
    Enemy_WreckedShipRobot {
        base: EnemyBase,
        words: [
            wsrt_var_A, wsrt_var_B, wsrt_var_C, wsrt_var_D, wsrt_var_E, wsrt_var_F,
            wsrt_parameter_1, wsrt_parameter_2,
        ],
    }
}

enemy_layout! {
    Enemy_WreckedShipSpark {
        base: EnemyBase,
        words: [
            wssk_var_A, wssk_var_B, wssk_var_C, wssk_var_D, wssk_var_E, wssk_var_F,
            wssk_parameter_1, wssk_parameter_2,
        ],
    }
}

enemy_layout! {
    Enemy_YappingMaw {
        base: EnemyBase,
        words: [
            ymw_var_A, ymw_var_B, ymw_var_C, ymw_var_D, ymw_var_E, ymw_var_F,
            ymw_parameter_1, ymw_parameter_2,
        ],
        pad1: [uint8; 26696],
        words: [
            ymw_var_00, ymw_var_01, ymw_var_02, ymw_var_03, ymw_var_04, ymw_var_05,
            ymw_var_06, ymw_var_07, ymw_var_08, ymw_var_09, ymw_var_0A, ymw_var_0B,
            ymw_var_0C, ymw_var_0D, ymw_var_0E, ymw_var_0F, ymw_var_10, ymw_var_11,
            ymw_var_12, ymw_var_13, ymw_var_14, ymw_var_15, ymw_var_16, ymw_var_17,
            ymw_var_18, ymw_var_19, ymw_var_1A, ymw_var_1B, ymw_var_1C, ymw_var_1D,
            ymw_var_1E, ymw_var_1F,
        ],
        pad2: [uint8; 1984],
        words: [
            ymw_var_20, ymw_var_21, ymw_var_22, ymw_var_23, ymw_var_24, ymw_var_25,
            ymw_var_26, ymw_var_27, ymw_var_28, ymw_var_29, ymw_var_2A, ymw_var_2B,
            ymw_var_2C, ymw_var_2D, ymw_var_2E, ymw_var_2F, ymw_var_30, ymw_var_31,
            ymw_var_32, ymw_var_33, ymw_var_34, ymw_var_35, ymw_var_36, ymw_var_37,
            ymw_var_38, ymw_var_39, ymw_var_3A, ymw_var_3B, ymw_var_3C, ymw_var_3D,
            ymw_var_3E, ymw_var_3F,
        ],
        pad3: [uint8; 1984],
        words: [
            ymw_var_40, ymw_var_41, ymw_var_42, ymw_var_43, ymw_var_44, ymw_var_45,
            ymw_var_46, ymw_var_47, ymw_var_48, ymw_var_49, ymw_var_4A, ymw_var_4B,
            ymw_var_4C, ymw_var_4D, ymw_var_4E, ymw_var_4F, ymw_var_50, ymw_var_51,
            ymw_var_52, ymw_var_53, ymw_var_54, ymw_var_55, ymw_var_56, ymw_var_57,
            ymw_var_58, ymw_var_59, ymw_var_5A, ymw_var_5B, ymw_var_5C, ymw_var_5D,
            ymw_var_5E, ymw_var_5F,
        ],
    }
}

enemy_layout! {
    Enemy_Beetom {
        base: EnemyBase,
        words: [
            beetom_var_A, beetom_var_B, beetom_var_C, beetom_var_D, beetom_var_E,
            beetom_var_F, beetom_parameter_1, beetom_parameter_2,
        ],
        pad1: [uint8; 26696],
        words: [
            beetom_var_00, beetom_var_01, beetom_var_02, beetom_var_03, beetom_var_04,
            beetom_var_05, beetom_var_06, beetom_var_07, beetom_var_08, beetom_var_09,
            beetom_var_0A, beetom_var_0B, beetom_var_0C, beetom_var_0D, beetom_var_0E,
            beetom_var_0F, beetom_var_10, beetom_var_11, beetom_var_12, beetom_var_13,
            beetom_var_14, beetom_var_15, beetom_var_16, beetom_var_17, beetom_var_18,
            beetom_var_19, beetom_var_1A, beetom_var_1B, beetom_var_1C, beetom_var_1D,
            beetom_var_1E, beetom_var_1F,
        ],
    }
}

enemy_layout! {
    Enemy_Kago {
        base: EnemyBase,
        words: [
            kago_var_A, kago_var_B, kago_var_C, kago_var_D, kago_var_E, kago_var_F,
            kago_parameter_1, kago_parameter_2,
        ],
        pad1: [uint8; 26696],
        words: [
            kago_var_00, kago_var_01, kago_var_02, kago_var_03, kago_var_04, kago_var_05,
            kago_var_06, kago_var_07, kago_var_08, kago_var_09, kago_var_0A, kago_var_0B,
            kago_var_0C, kago_var_0D, kago_var_0E, kago_var_0F, kago_var_10, kago_var_11,
            kago_var_12, kago_var_13, kago_var_14, kago_var_15, kago_var_16, kago_var_17,
            kago_var_18, kago_var_19, kago_var_1A, kago_var_1B, kago_var_1C, kago_var_1D,
            kago_var_1E, kago_var_1F,
        ],
    }
}

enemy_layout! {
    Enemy_MaridiaFloater {
        base: EnemyBase,
        words: [
            mfr_var_A, mfr_var_B, mfr_var_C, mfr_var_D, mfr_var_E, mfr_var_F,
            mfr_parameter_1, mfr_parameter_2,
        ],
    }
}

enemy_layout! {
    Enemy_MaridiaPuffer {
        base: EnemyBase,
        words: [
            mpr_var_A, mpr_var_B, mpr_var_C, mpr_var_D, mpr_var_E, mpr_var_F,
            mpr_parameter_1, mpr_parameter_2,
        ],
        pad1: [uint8; 26696],
        words: [
            mpr_var_00, mpr_var_01, mpr_var_02, mpr_var_03, mpr_var_04, mpr_var_05,
            mpr_var_06, mpr_var_07, mpr_var_08, mpr_var_09, mpr_var_0A, mpr_var_0B,
            mpr_var_0C, mpr_var_0D, mpr_var_0E, mpr_var_0F, mpr_var_10, mpr_var_11,
            mpr_var_12, mpr_var_13, mpr_var_14, mpr_var_15, mpr_var_16, mpr_var_17,
            mpr_var_18, mpr_var_19, mpr_var_1A, mpr_var_1B, mpr_var_1C, mpr_var_1D,
            mpr_var_1E, mpr_var_1F,
        ],
        pad3: [uint8; 4032],
        words: [
            mpr_var_40, mpr_var_41, mpr_var_42, mpr_var_43, mpr_var_44, mpr_var_45,
            mpr_var_46, mpr_var_47, mpr_var_48, mpr_var_49, mpr_var_4A, mpr_var_4B,
            mpr_var_4C, mpr_var_4D, mpr_var_4E, mpr_var_4F, mpr_var_50, mpr_var_51,
            mpr_var_52, mpr_var_53, mpr_var_54, mpr_var_55, mpr_var_56, mpr_var_57,
            mpr_var_58, mpr_var_59, mpr_var_5A, mpr_var_5B, mpr_var_5C, mpr_var_5D,
            mpr_var_5E, mpr_var_5F,
        ],
    }
}

enemy_layout! {
    Enemy_MorphBallEye {
        base: EnemyBase,
        words: [
            mbee_var_A, mbee_var_B, mbee_var_C, mbee_var_D, mbee_var_E, mbee_var_F,
            mbee_parameter_1, mbee_parameter_2,
        ],
    }
}

enemy_layout! {
    Enemy_WreckedShipOrbs {
        base: EnemyBase,
        words: [
            wsos_var_A, wsos_var_B, wsos_var_C, wsos_var_D, wsos_var_E, wsos_var_F,
            wsos_parameter_1, wsos_parameter_2,
        ],
        pad1: [uint8; 26696],
        words: [
            wsos_var_00, wsos_var_01, wsos_var_02, wsos_var_03, wsos_var_04, wsos_var_05,
            wsos_var_06, wsos_var_07, wsos_var_08, wsos_var_09, wsos_var_0A, wsos_var_0B,
            wsos_var_0C, wsos_var_0D, wsos_var_0E, wsos_var_0F, wsos_var_10, wsos_var_11,
            wsos_var_12, wsos_var_13, wsos_var_14, wsos_var_15, wsos_var_16, wsos_var_17,
            wsos_var_18, wsos_var_19, wsos_var_1A, wsos_var_1B, wsos_var_1C, wsos_var_1D,
            wsos_var_1E, wsos_var_1F,
        ],
    }
}

#[inline]
unsafe fn RomPtr_A8(addr: uint16_t, game: &mut Game) -> *const uint8 {
    RomPtr((0xa80000u32) | ((addr) as u32), game)
}
#[inline]
unsafe fn gVramWriteEntry(a: ::core::ffi::c_int, game: &mut Game) -> *mut VramWriteEntry {
    (&raw mut game.g_ram as *mut uint8).offset((0xd0 as ::core::ffi::c_int + a) as isize)
        as *mut uint8 as *mut VramWriteEntry
}
pub const fnnullsub_152: ::core::ffi::c_int = 0x90e90e;
pub const fnSamus_InputHandler_E913: ::core::ffi::c_int = 0x90e913;
pub const fnSamus_InputHandler_E91D: ::core::ffi::c_int = 0x90e91d;
pub const fnnullsub_170_A8: ::core::ffi::c_int = 0xa8804c;
pub const fnMiniDraygon_Func_4: ::core::ffi::c_int = 0xa88922;
pub const fnMiniDraygon_Func_8: ::core::ffi::c_int = 0xa88a34;
pub const fnMiniDraygon_Func_9: ::core::ffi::c_int = 0xa88a3b;
pub const fnMiniDraygon_Func_10: ::core::ffi::c_int = 0xa88a78;
pub const fnMorphBallEye_Func_1: ::core::ffi::c_int = 0xa890f1;
pub const fnMorphBallEye_Func_2: ::core::ffi::c_int = 0xa8912e;
pub const fnMorphBallEye_Func_3: ::core::ffi::c_int = 0xa89160;
pub const fnMorphBallEye_Func_4: ::core::ffi::c_int = 0xa891ce;
pub const fnnullsub_244: ::core::ffi::c_int = 0xa891dc;
pub const fnFune_Func_1: ::core::ffi::c_int = 0xa89737;
pub const fnFune_Func_2: ::core::ffi::c_int = 0xa8975c;
pub const fnnullsub_247: ::core::ffi::c_int = 0xa8978e;
pub const fnnullsub_248: ::core::ffi::c_int = 0xa8978f;
pub const fnWreckedShipGhost_Func_1: ::core::ffi::c_int = 0xa89b42;
pub const fnWreckedShipGhost_Func_2: ::core::ffi::c_int = 0xa89bad;
pub const fnWreckedShipGhost_Func_4: ::core::ffi::c_int = 0xa89c69;
pub const fnWreckedShipGhost_Func_5: ::core::ffi::c_int = 0xa89c8a;
pub const fnWreckedShipGhost_Func_6: ::core::ffi::c_int = 0xa89d13;
pub const fnWreckedShipGhost_Func_7: ::core::ffi::c_int = 0xa89d36;
pub const fnYappingMaw_Func_1: ::core::ffi::c_int = 0xa8a235;
pub const fnYappingMaw_Func_2: ::core::ffi::c_int = 0xa8a28c;
pub const fnYappingMaw_Func_8: ::core::ffi::c_int = 0xa8a445;
pub const fnYappingMaw_Func_11: ::core::ffi::c_int = 0xa8a68a;
pub const fnKago_Func_1: ::core::ffi::c_int = 0xa8ab7b;
pub const fnnullsub_306: ::core::ffi::c_int = 0xa8ab81;
pub const fnNorfairLavaMan_Func_6: ::core::ffi::c_int = 0xa8b0b2;
pub const fnNorfairLavaMan_Func_7: ::core::ffi::c_int = 0xa8b11a;
pub const fnNorfairLavaMan_Func_8: ::core::ffi::c_int = 0xa8b175;
pub const fnsub_A8B193: ::core::ffi::c_int = 0xa8b193;
pub const fnNorfairLavaMan_Func_9: ::core::ffi::c_int = 0xa8b1b8;
pub const fnNorfairLavaMan_Func_10: ::core::ffi::c_int = 0xa8b1dd;
pub const fnNorfairLavaMan_Func_11: ::core::ffi::c_int = 0xa8b204;
pub const fnsub_A8B291: ::core::ffi::c_int = 0xa8b291;
pub const fnNorfairLavaMan_Func_13: ::core::ffi::c_int = 0xa8b295;
pub const fnNorfairLavaMan_Func_15: ::core::ffi::c_int = 0xa8b30d;
pub const fnNorfairLavaMan_Func_16: ::core::ffi::c_int = 0xa8b31f;
pub const fnNorfairLavaMan_Func_17: ::core::ffi::c_int = 0xa8b356;
pub const fnNorfairLavaMan_Func_18: ::core::ffi::c_int = 0xa8b3a7;
pub const fnBeetom_Func_3: ::core::ffi::c_int = 0xa8b814;
pub const fnBeetom_Func_4: ::core::ffi::c_int = 0xa8b82f;
pub const fnBeetom_Func_5: uint32 = 11057231;
pub const fnBeetom_Func_6: ::core::ffi::c_int = 0xa8b85f;
pub const fnBeetom_Func_7: ::core::ffi::c_int = 0xa8b873;
pub const fnBeetom_Func_8: uint32 = 11057287;
pub const fnBeetom_Func_9: uint32 = 11057321;
pub const fnBeetom_Func_10: uint32 = 11057355;
pub const fnBeetom_Func_11: uint32 = 11057389;
pub const fnBeetom_Func_12: ::core::ffi::c_int = 0xa8b90f;
pub const fnBeetom_Func_13: ::core::ffi::c_int = 0xa8b952;
pub const fnBeetom_Func_14: ::core::ffi::c_int = 0xa8b966;
pub const fnBeetom_Func_15: ::core::ffi::c_int = 0xa8b97a;
pub const fnBeetom_Func_16: ::core::ffi::c_int = 0xa8b9a2;
pub const fnBeetom_Func_17: ::core::ffi::c_int = 0xa8b9b2;
pub const fnBeetom_Func_18: ::core::ffi::c_int = 0xa8b9c1;
pub const fnBeetom_Func_19: ::core::ffi::c_int = 0xa8ba24;
pub const fnBeetom_Func_20: ::core::ffi::c_int = 0xa8ba84;
pub const fnBeetom_Func_21: ::core::ffi::c_int = 0xa8bab7;
pub const fnBeetom_Func_24: ::core::ffi::c_int = 0xa8bb55;
pub const fnBeetom_Func_25: ::core::ffi::c_int = 0xa8bb88;
pub const fnBeetom_Func_28: ::core::ffi::c_int = 0xa8bc26;
pub const fnBeetom_Func_29: ::core::ffi::c_int = 0xa8bc5a;
pub const fnBeetom_Func_32: ::core::ffi::c_int = 0xa8bcf8;
pub const fnBeetom_Func_33: ::core::ffi::c_int = 0xa8bd42;
pub const fnBeetom_Func_35: ::core::ffi::c_int = 0xa8bd9d;
pub const fnBeetom_Func_36: ::core::ffi::c_int = 0xa8bdc5;
pub const fnMaridiaFloater_Func_3: ::core::ffi::c_int = 0xa8c283;
pub const fnMaridiaFloater_Func_4: ::core::ffi::c_int = 0xa8c2a6;
pub const fnMaridiaFloater_Func_5: ::core::ffi::c_int = 0xa8c2cf;
pub const fnMaridiaFloater_Func_6: ::core::ffi::c_int = 0xa8c36b;
pub const fnMaridiaFloater_Func_7: ::core::ffi::c_int = 0xa8c3e1;
pub const fnMaridiaFloater_Func_8: ::core::ffi::c_int = 0xa8c469;
pub const fnMaridiaFloater_Func_9: ::core::ffi::c_int = 0xa8c4dc;
pub const fnMaridiaFloater_Func_10: ::core::ffi::c_int = 0xa8c500;
pub const fnMaridiaFloater_Func_11: ::core::ffi::c_int = 0xa8c51d;
pub const fnnullsub_256: ::core::ffi::c_int = 0xa8c568;
pub const fnMaridiaFloater_Func_12: ::core::ffi::c_int = 0xa8c569;
pub const fnMaridiaFloater_Func_13: ::core::ffi::c_int = 0xa8c59f;
pub const fnWreckedShipRobot_Func_1: ::core::ffi::c_int = 0xa8cc67;
pub const fnMaridiaPuffer_Func_1: ::core::ffi::c_int = 0xa8d92b;
pub const fnMaridiaPuffer_Func_2: ::core::ffi::c_int = 0xa8d940;
pub const fnMaridiaPuffer_Func_3: ::core::ffi::c_int = 0xa8d963;
pub const fnMaridiaPuffer_Func_4: ::core::ffi::c_int = 0xa8d97c;
pub const fnWalkingLavaSeahorse_Func_3: ::core::ffi::c_int = 0xa8dd71;
pub const fnWalkingLavaSeahorse_Func_4: ::core::ffi::c_int = 0xa8ddc6;
pub const fnWalkingLavaSeahorse_Func_6: ::core::ffi::c_int = 0xa8de05;
pub const fnWalkingLavaSeahorse_Func_7: ::core::ffi::c_int = 0xa8de4b;
pub const fnnullsub_257: ::core::ffi::c_int = 0xa8decc;
pub const fnWalkingLavaSeahorse_Func_8: ::core::ffi::c_int = 0xa8decd;
pub const fnWalkingLavaSeahorse_Func_9: ::core::ffi::c_int = 0xa8deec;
pub const fnWreckedShipOrbs_Func_3: ::core::ffi::c_int = 0xa8e405;
pub const fnWreckedShipOrbs_Func_4: ::core::ffi::c_int = 0xa8e424;
pub const fnWreckedShipOrbs_Func_5: ::core::ffi::c_int = 0xa8e443;
pub const fnWreckedShipOrbs_Func_6: ::core::ffi::c_int = 0xa8e462;
pub const fnnullsub_259: uint32 = 11069076;
pub const fnWreckedShipSpark_Func_1: ::core::ffi::c_int = 0xa8e695;
pub const fnWreckedShipSpark_Func_2: ::core::ffi::c_int = 0xa8e6b7;
pub const fnWreckedShipSpark_Func_3: uint32 = 11069148;
pub const fnBlueBrinstarFaceBlock_Func_1: ::core::ffi::c_int = 0xa8e86e;
pub const fnKiHunter_Func_1: ::core::ffi::c_int = 0xa8f268;
pub const fnKiHunter_Func_2: ::core::ffi::c_int = 0xa8f3b8;
pub const fnKiHunter_Func_3: ::core::ffi::c_int = 0xa8f4ed;
pub const fnKiHunter_Func_4: ::core::ffi::c_int = 0xa8f55a;
pub const fnKiHunter_Func_5: ::core::ffi::c_int = 0xa8f58b;
pub const fnnullsub_346: uint32 = 11072995;
pub const fnKiHunter_Func_6: ::core::ffi::c_int = 0xa8f5f0;
pub const fnKiHunter_Func_7: ::core::ffi::c_int = 0xa8f68b;
pub const fnKiHunter_Func_8: ::core::ffi::c_int = 0xa8f6b3;
pub const fnKiHunter_Func_9: ::core::ffi::c_int = 0xa8f6f3;
pub const fnKiHunter_Func_10: ::core::ffi::c_int = 0xa8f7cf;
pub const fnKiHunter_Func_11: ::core::ffi::c_int = 0xa8f7db;
pub const fnKiHunter_Func_14: ::core::ffi::c_int = 0xa8f8ad;
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
unsafe fn Get_Beetom(o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_Beetom {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_Beetom
}
#[inline]
unsafe fn Get_Fune(o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_Fune {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_Fune
}
#[inline]
unsafe fn Get_Kago(o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_Kago {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_Kago
}
#[inline]
unsafe fn Get_KiHunter(o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_KiHunter {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_KiHunter
}
#[inline]
unsafe fn Get_MaridiaFloater(o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_MaridiaFloater {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_MaridiaFloater
}
#[inline]
unsafe fn Get_MaridiaPuffer(o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_MaridiaPuffer {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_MaridiaPuffer
}
#[inline]
unsafe fn Get_MiniDraygon(o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_MiniDraygon {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_MiniDraygon
}
#[inline]
unsafe fn Get_MorphBallEye(o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_MorphBallEye {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_MorphBallEye
}
#[inline]
unsafe fn Get_NorfairLavaMan(o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_NorfairLavaMan {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_NorfairLavaMan
}
#[inline]
unsafe fn Get_WalkingLavaSeahorse(
    o: ::core::ffi::c_int,
    game: &mut Game,
) -> *mut Enemy_WalkingLavaSeahorse {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_WalkingLavaSeahorse
}
#[inline]
unsafe fn Get_WreckedShipGhost(
    o: ::core::ffi::c_int,
    game: &mut Game,
) -> *mut Enemy_WreckedShipGhost {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_WreckedShipGhost
}
#[inline]
unsafe fn Get_WreckedShipOrbs(
    o: ::core::ffi::c_int,
    game: &mut Game,
) -> *mut Enemy_WreckedShipOrbs {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_WreckedShipOrbs
}
#[inline]
unsafe fn Get_WreckedShipRobot(
    o: ::core::ffi::c_int,
    game: &mut Game,
) -> *mut Enemy_WreckedShipRobot {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_WreckedShipRobot
}
#[inline]
unsafe fn Get_WreckedShipSpark(
    o: ::core::ffi::c_int,
    game: &mut Game,
) -> *mut Enemy_WreckedShipSpark {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_WreckedShipSpark
}
#[inline]
unsafe fn Get_YappingMaw(o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_YappingMaw {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_YappingMaw
}
static g_word_A890CA: [int16; 4] = [-(8), 8, 0, 0];
static g_word_A890D2: [int16; 4] = [0, 0, -(8), 8];
static g_word_A89050: uint16 = 0x80 as uint16;
static g_word_A89052: uint16 = 0xb0 as uint16;
static g_word_A89054: uint16 = 0x80 as uint16;
static g_word_A89056: uint16 = 0x80 as uint16;
static g_word_A89A9C: uint16 = 0x10 as uint16;
static g_word_A89A9E: uint16 = 0x40 as uint16;
static g_word_A89AA0: uint16 = 0x1800 as uint16;
static g_word_A89AA2: uint16 = 1 as uint16;
static g_word_A89AA4: uint16 = 0x78 as uint16;
static g_word_A89AA6: uint16 = 0x78 as uint16;
static g_word_A89AA8: [uint16; 18] = [
    0xffc0, 0xffc0, 0, 0xffc0, 0x40, 0, 0xffc0, 0, 0, 0, 0x40, 0, 0xffc0, 0x40, 0, 0x40, 0x40, 0x40,
];
static g_word_A89ACC: [uint16; 17] = [1, 8, 1, 8, 1, 7, 1, 7, 2, 6, 2, 6, 3, 5, 3, 5, 0xffff];
static kWreckedShipGhost_Palette: [uint16; 16] = [
    0x3800, 0x57ff, 0x42f7, 0x929, 0xa5, 0x4f5a, 0x36b5, 0x2610, 0x1dce, 0x1df, 0x1f, 0x18, 0xa,
    0x6b9, 0xea, 0x45,
];
static g_word_A89D32: uint16 = 1 as uint16;
static g_word_A89D34: uint16 = 1 as uint16;
static g_word_A8C277: [int16; 3] = [-(12), -(16), -(20)];
static g_word_A8C27D: [int16; 3] = [-(20), -(16), -(12)];
static g_word_A8C19F: uint16 = 0x40 as uint16;
static g_word_A8C1A1: [uint16; 12] = [0, 1, 2, 3, 2, 1, 0, 0xffff, 0xfffe, 0xfffd, 0xfffe, 0xffff];
static g_word_A8C1B9: uint16 = 0 as uint16;
static g_word_A8C1BB: uint16 = 0 as uint16;
static g_word_A8C1BD: uint16 = 0 as uint16;
static g_word_A8C1BF: uint16 = 0 as uint16;
static g_word_A8C1C1: uint16 = 1 as uint16;
static g_word_A8C1C3: uint16 = 0 as uint16;
static g_word_A8C1C5: uint16 = 0xffff as uint16;
static g_word_A8C1C7: uint16 = 0x8000 as uint16;
static g_word_A8CCC1: [int16; 31] = [
    0x1f,
    0x18,
    0xf,
    8,
    0x40,
    0x18,
    0xf,
    8,
    0x1f,
    0x10,
    0xf,
    8,
    0x1f,
    0x18,
    0x10,
    8,
    0x1f,
    0x18,
    0xf,
    0x40,
    0xf,
    8,
    0x1f,
    0x18,
    0x10,
    0x18,
    0xf,
    8,
    0x1f,
    0x10,
    -(1),
];
static g_word_A8D871: [int16; 10] = [0xc0, 0xe0, 0, 0x20, 0x40, 0x40, 0x60, 0x80, 0xa0, 0xc0];
static g_word_A8D885: [int16; 8] = [0x3ff, 0x4ff, 0x5ff, 0x6ff, 0x7ff, 0x8ff, 0x9ff, 0xaff];
static g_word_A8D895: [int16; 26] = [
    3, 1, 4, 1, 5, 2, 6, 2, 7, 2, 8, 3, 9, 3, 0xa, 4, 0xb, 4, 0xc, 5, 0xd, 5, 0xe, 6, 0xf, 6,
];
static g_word_A8DCC7: uint16 = 0x50 as uint16;
static g_word_A8DCCB: uint16 = 0x70 as uint16;
static g_word_A8F180: uint16 = 0x60 as uint16;
static g_word_A8F182: uint16 = 0xe000 as uint16;
static g_word_A8F184: uint16 = 0 as uint16;
static g_byte_A8F186: uint8 = 0x30 as uint8;
pub unsafe fn Enemy_GrappleReact_NoInteract_A8(game: &mut Game) {
    SwitchEnemyAiToMainAi(game);
}
pub unsafe fn Enemy_GrappleReact_KillEnemy_A8(game: &mut Game) {
    EnemyGrappleDeath(game);
}
pub unsafe fn Enemy_GrappleReact_CancelBeam_A8(game: &mut Game) {
    Enemy_SwitchToFrozenAi(game);
}
pub unsafe fn Enemy_GrappleReact_SamusLatchesNoInvinc_A8(game: &mut Game) {
    SamusLatchesOnWithGrappleNoInvinc(game);
}
pub unsafe fn Enemy_GrappleReact_HurtSamus_A8(game: &mut Game) {
    SamusHurtFromGrapple(game);
}
pub unsafe fn Enemy_NormalTouchAI_A8(game: &mut Game) {
    NormalEnemyTouchAi(game);
}
pub unsafe fn Enemy_NormalShotAI_A8(game: &mut Game) {
    NormalEnemyShotAi(game);
}
pub unsafe fn Enemy_NormalPowerBombAI_A8(game: &mut Game) {
    NormalEnemyPowerBombAi(game);
}
pub unsafe fn Enemy_NormalFrozenAI_A8(game: &mut Game) {
    NormalEnemyFrozenAI(game);
}
pub unsafe fn MiniDraygon_Instr_2(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    QueueSfx2_Max6(0x5e as uint16, game);
    jp
}
pub unsafe fn MiniDraygon_Instr_1(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    if (*gEnemySpawnData(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game).offset(30))
        .cause_of_death as ::core::ffi::c_int
        == addr_kMiniDraygon_Ilist_870B as ::core::ffi::c_int
    {
        (*Get_MiniDraygon(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game))
            .mdn_var_F = -(8 as ::core::ffi::c_int) as uint16;
    } else {
        (*Get_MiniDraygon(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game))
            .mdn_var_F = 8 as uint16;
    }
    jp
}
pub unsafe fn MiniDraygon_Instr_3(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let E: *mut Enemy_MiniDraygon =
        Get_MiniDraygon(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    if (*gEnemySpawnData(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game).offset(30))
        .cause_of_death as ::core::ffi::c_int
        == addr_kMiniDraygon_Ilist_870B as ::core::ffi::c_int
    {
        (*E).mdn_var_F = (*E).mdn_var_F.wrapping_add(1);
    } else {
        (*E).mdn_var_F = (*E).mdn_var_F.wrapping_sub(1);
    }
    jp
}
pub unsafe fn MiniDraygon_Instr_4(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let E: *mut Enemy_MiniDraygon =
        Get_MiniDraygon(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).mdn_var_0C = 0 as uint16;
    (*E).mdn_var_0B = 0 as uint16;
    (*E).mdn_var_C = ((0xa88a34i32) as u16);
    jp
}
pub unsafe fn MiniDraygon_Init(game: &mut Game) {
    let E: *mut Enemy_MiniDraygon =
        Get_MiniDraygon(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    if (*E).mdn_parameter_1 != 0 {
        MiniDraygon_Func_2(game);
        (*E).base.layer = 4 as uint16;
    } else {
        MiniDraygon_Func_1(game);
        let v2: uint16 =
            (8u16).wrapping_mul((*(&raw mut (*E).mdn_parameter_2 as *mut uint8).offset(0)) as u16);
        let v3: ::core::ffi::c_int = v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        (*E).mdn_var_04 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8
            as *mut uint16)
            .offset(v3 as isize);
        (*E).mdn_var_03 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8
            as *mut uint16)
            .offset((v3 + 1 as ::core::ffi::c_int) as isize);
        (*E).mdn_var_06 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8
            as *mut uint16)
            .offset((v3 + 2 as ::core::ffi::c_int) as isize);
        (*E).mdn_var_05 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8
            as *mut uint16)
            .offset((v3 + 3 as ::core::ffi::c_int) as isize);
        (*E).mdn_var_E = (*(&raw mut (*(Get_MiniDraygon
            as unsafe fn(::core::ffi::c_int, &mut Game) -> *mut Enemy_MiniDraygon)(
            v2 as ::core::ffi::c_int,
            game,
        ))
        .mdn_parameter_2 as *mut uint8)
            .offset(
                (::core::mem::size_of::<uint16>())
                    .wrapping_div(::core::mem::size_of::<uint8>())
                    .wrapping_sub(1_usize) as isize,
            ) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int) as uint16;
    }
    (*E).mdn_var_00 = 0 as uint16;
    (*E).mdn_var_01 = 0 as uint16;
    (*E).mdn_var_C = ((0xa88922i32) as u16);
}
pub unsafe fn MiniDraygon_Func_1(game: &mut Game) {
    (*Get_MiniDraygon(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game)).mdn_var_B =
        ((GetSamusEnemyDelta_X(ram_word!(game, cur_enemy_index), game) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int)
            == 0i32) as uint16;
    let E: *mut Enemy_MiniDraygon =
        Get_MiniDraygon(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).mdn_var_02 = addr_kMiniDraygon_Ilist_86A7 as uint16;
    if (*E).mdn_var_B != 0 {
        (*E).mdn_var_02 = addr_kMiniDraygon_Ilist_870B as uint16;
    }
    MiniDraygon_Func_12(game);
}
pub unsafe fn MiniDraygon_Func_2(game: &mut Game) {
    let v0: ::core::ffi::c_int =
        ram_word!(game, cur_enemy_index) as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let v1: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0xf68) as *mut uint16)
        .offset((v0 + 1 as ::core::ffi::c_int) as isize);
    let E: *mut Enemy_MiniDraygon =
        Get_MiniDraygon(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).mdn_var_B = v1;
    if v1 != 0 {
        (*E).base.x_pos = (*((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
            .offset((v0 + 91 as ::core::ffi::c_int) as isize))
        .wrapping_add(4u16);
        (*E).base.y_pos = (*((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
            .offset((v0 + 93 as ::core::ffi::c_int) as isize))
        .wrapping_add(10u16);
        (*E).mdn_var_02 = addr_kMiniDraygon_Ilist_8727 as uint16;
    } else {
        (*E).base.x_pos = (*((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
            .offset((v0 + 91 as ::core::ffi::c_int) as isize))
        .wrapping_sub(4u16);
        (*E).base.y_pos = (*((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
            .offset((v0 + 93 as ::core::ffi::c_int) as isize))
        .wrapping_add(10u16);
        (*E).mdn_var_02 = addr_kMiniDraygon_Ilist_86C3 as uint16;
    }
    MiniDraygon_Func_12(game);
}
pub unsafe fn EvirProjectile_Init(game: &mut Game) {
    let E: *mut Enemy_MiniDraygon =
        Get_MiniDraygon(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).mdn_var_02 = addr_kMiniDraygon_Ilist_876F as uint16;
    MiniDraygon_Func_12(game);
    let v2: ::core::ffi::c_int =
        ram_word!(game, cur_enemy_index) as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    (*E).base.palette_index = *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
        .offset((v2 + 73 as ::core::ffi::c_int) as isize);
    (*E).base.vram_tiles_index = *((&raw mut game.g_ram as *mut uint8).offset(0xe84)
        as *mut uint16)
        .offset((v2 + 74 as ::core::ffi::c_int) as isize);
    MiniDraygon_Func_3(game);
    (*E).mdn_var_01 = 0 as uint16;
    (*E).mdn_var_0C = 0 as uint16;
    (*E).mdn_var_0B = 0 as uint16;
    (*E).mdn_var_F = 0 as uint16;
    (*E).mdn_var_C = ((0xa88a34i32) as u16);
}
pub unsafe fn MiniDraygon_Func_3(game: &mut Game) {
    let v0: ::core::ffi::c_int =
        ram_word!(game, cur_enemy_index) as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let v1: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
        .offset((v0 + 83 as ::core::ffi::c_int) as isize);
    let E: *mut Enemy_MiniDraygon =
        Get_MiniDraygon(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).mdn_var_B = v1;
    if v1 != 0 {
        (*E).base.x_pos = (*((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
            .offset((v0 + 59 as ::core::ffi::c_int) as isize))
        .wrapping_add(4u16);
    } else {
        (*E).base.x_pos = (*((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
            .offset((v0 + 59 as ::core::ffi::c_int) as isize))
        .wrapping_sub(4u16);
    }
    (*E).base.y_pos = (*((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
        .offset((v0 + 61 as ::core::ffi::c_int) as isize))
    .wrapping_add(18u16);
}
pub unsafe fn CallMiniDraygonFunc(ea: uint32, game: &mut Game) {
    match ea {
        11045154 => {
            MiniDraygon_Func_4(game);
        }
        11045428 => {
            MiniDraygon_Func_8(game);
        }
        11045435 => {
            MiniDraygon_Func_9(game);
        }
        11045496 => {
            MiniDraygon_Func_10(game);
        }
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn MiniDraygon_Main(game: &mut Game) {
    let E: *mut Enemy_MiniDraygon =
        Get_MiniDraygon(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    CallMiniDraygonFunc((((*E).mdn_var_C) as u32) | (0xa80000u32), game);
}
pub unsafe fn MiniDraygon_Func_4(game: &mut Game) {
    if (*Get_MiniDraygon(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game))
        .mdn_parameter_1
        != 0
    {
        MiniDraygon_Func_6(game);
    } else {
        MiniDraygon_Func_5(game);
    };
}
pub unsafe fn MiniDraygon_Func_5(game: &mut Game) {
    if (*Get_MiniDraygon(
        ram_word!(game, cur_enemy_index) as ::core::ffi::c_int + 128 as ::core::ffi::c_int,
        game,
    ))
    .mdn_var_0C
        == 0
    {
        MiniDraygon_Func_1(game);
    }
    let E: *mut Enemy_MiniDraygon =
        Get_MiniDraygon(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    if (*E).mdn_var_00 != 0 {
        AddToHiLo(
            &raw mut (*E).base.y_pos,
            &raw mut (*E).base.y_subpos,
            ((*E).mdn_var_04 as uint32) << 16 as ::core::ffi::c_int | (*E).mdn_var_03 as uint32,
        );
    } else {
        AddToHiLo(
            &raw mut (*E).base.y_pos,
            &raw mut (*E).base.y_subpos,
            ((*E).mdn_var_06 as uint32) << 16 as ::core::ffi::c_int | (*E).mdn_var_05 as uint32,
        );
    }
    (*E).mdn_var_E = (*E).mdn_var_E.wrapping_sub(1);
    if ((*E).mdn_var_E as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        (*E).mdn_var_E = *(&raw mut (*E).mdn_parameter_2 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1_usize) as isize,
        ) as uint16;
        (*E).mdn_var_00 = ((*E).mdn_var_00) ^ (1u16);
    }
}
pub unsafe fn MiniDraygon_Func_6(game: &mut Game) {
    MiniDraygon_Func_2(game);
}
pub unsafe fn EvirProjectile_Main(game: &mut Game) {
    let E: *mut Enemy_MiniDraygon =
        Get_MiniDraygon(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    if (*E).base.frozen_timer == 0 {
        if (*E).mdn_var_0B != 0 {
            (*E).mdn_var_02 = addr_kMiniDraygon_Ilist_876F as uint16;
            MiniDraygon_Func_12(game);
        } else if (*E).mdn_var_0C != 0 {
            (*E).mdn_var_02 = addr_kMiniDraygon_Ilist_8775 as uint16;
            MiniDraygon_Func_12(game);
        } else {
            MiniDraygon_Func_7(game);
        }
    }
    CallMiniDraygonFunc((((*E).mdn_var_C) as u32) | (0xa80000u32), game);
}
pub unsafe fn MiniDraygon_Func_7(game: &mut Game) {
    if IsSamusWithinEnemy_X(
        (ram_word!(game, cur_enemy_index)).wrapping_sub(128u16),
        0x80 as uint16,
        game,
    ) != 0
    {
        let r22: uint16 =
            -((((CalculateAngleOfSamusFromEnemy(ram_word!(game, cur_enemy_index), game)) as u8)
                .wrapping_sub(64u8)) as ::core::ffi::c_int) as uint16;
        let E: *mut Enemy_MiniDraygon =
            Get_MiniDraygon(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
        SetHiLo(
            &raw mut (*E).mdn_var_07,
            &raw mut (*E).mdn_var_08,
            CosineMult8bitFull(r22, 4 as uint16, game),
        );
        SetHiLo(
            &raw mut (*E).mdn_var_09,
            &raw mut (*E).mdn_var_0A,
            SineMult8bitFull(r22, 4 as uint16, game),
        );
        (*E).mdn_var_02 = addr_kMiniDraygon_Ilist_876F as uint16;
        MiniDraygon_Func_12(game);
        (*E).mdn_var_0B = 1 as uint16;
        (*E).mdn_var_C = ((0xa88a3bi32) as u16);
    }
}
pub unsafe fn MiniDraygon_Func_8(game: &mut Game) {
    MiniDraygon_Func_3(game);
}
pub unsafe fn MiniDraygon_Func_9(game: &mut Game) {
    MiniDraygon_Func_11(game);
    let E: *mut Enemy_MiniDraygon =
        Get_MiniDraygon(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        ((*E).mdn_var_07 as uint32) << 16 as ::core::ffi::c_int | (*E).mdn_var_08 as uint32,
    );
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        ((*E).mdn_var_09 as uint32) << 16 as ::core::ffi::c_int | (*E).mdn_var_0A as uint32,
    );
}
pub unsafe fn MiniDraygon_Func_10(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16).offset(
        ((ram_word!(game, cur_enemy_index) as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
            + 77 as ::core::ffi::c_int) as isize,
    ) == 0
    {
        let E: *mut Enemy_MiniDraygon =
            Get_MiniDraygon(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
        if (*E).mdn_var_0C != 0 {
            MiniDraygon_Func_3(game);
            (*E).base.x_pos = ((*E).base.x_pos).wrapping_add((*E).mdn_var_F);
        } else {
            (*E).mdn_var_02 = addr_kMiniDraygon_Ilist_876F as uint16;
            MiniDraygon_Func_12(game);
            (*E).mdn_var_0C = 0 as uint16;
            (*E).mdn_var_0B = 0 as uint16;
            (*E).mdn_var_C = ((0xa88a34i32) as u16);
        }
    }
}
pub unsafe fn MiniDraygon_Func_11(game: &mut Game) {
    if EnemyFunc_ADA3(0x100 as uint16, game) != 0 {
        if *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16).offset(
            ((ram_word!(game, cur_enemy_index) as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
                + 77 as ::core::ffi::c_int) as isize,
        ) == 0
        {
            let E: *mut Enemy_MiniDraygon =
                Get_MiniDraygon(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
            (*E).mdn_var_0B = 0 as uint16;
            (*E).mdn_var_0C = 1 as uint16;
            (*E).mdn_var_C = ((0xa88a78i32) as u16);
            (*E).mdn_var_0C = 1 as uint16;
            (*E).mdn_var_02 = addr_kMiniDraygon_Ilist_8775 as uint16;
            MiniDraygon_Func_12(game);
        }
    }
}
pub unsafe fn MiniDraygon_Func_12(game: &mut Game) {
    let E: *mut Enemy_MiniDraygon =
        Get_MiniDraygon(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let mdn_var_02: uint16 = (*E).mdn_var_02;
    if mdn_var_02 != (*E).mdn_var_01 {
        (*E).base.current_instruction = mdn_var_02;
        (*E).mdn_var_01 = mdn_var_02;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
    }
}
pub unsafe fn MiniDraygon_Touch(game: &mut Game) {
    Enemy_NormalTouchAI_A8(game);
    MiniDraygon_Func_13(game);
}
pub unsafe fn MiniDraygon_Powerbomb(game: &mut Game) {
    Enemy_NormalPowerBombAI_A8(game);
    MiniDraygon_Func_13(game);
}
pub unsafe fn MiniDraygon_Shot(game: &mut Game) {
    NormalEnemyShotAi(game);
    MiniDraygon_Func_13(game);
}
pub unsafe fn MiniDraygon_Func_13(game: &mut Game) {
    let E: *mut Enemy_MiniDraygon =
        Get_MiniDraygon(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let E1: *mut Enemy_MiniDraygon = Get_MiniDraygon(
        ram_word!(game, cur_enemy_index) as ::core::ffi::c_int + 64 as ::core::ffi::c_int,
        game,
    );
    let E2: *mut Enemy_MiniDraygon = Get_MiniDraygon(
        ram_word!(game, cur_enemy_index) as ::core::ffi::c_int + 128 as ::core::ffi::c_int,
        game,
    );
    if (*E).base.health == 0 {
        (*E1).base.properties = ((*E1).base.properties) | (0x200u16);
        (*E2).base.properties = ((*E2).base.properties) | (0x200u16);
    }
    let frozen_timer: uint16 = (*E).base.frozen_timer;
    if frozen_timer != 0 {
        (*E1).base.frozen_timer = frozen_timer;
        (*E1).base.ai_handler_bits = ((*E1).base.ai_handler_bits) | (4u16);
        if (*E2).mdn_var_C as ::core::ffi::c_int
            != 0xa88a3b as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
        {
            (*E2).base.ai_handler_bits = ((*E2).base.ai_handler_bits) | (4u16);
            (*E2).base.frozen_timer = (*E).base.frozen_timer;
        }
    }
}
pub unsafe fn MorphBallEye_Init(game: &mut Game) {
    let E: *mut Enemy_MorphBallEye =
        Get_MorphBallEye(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_DisableSamusColl) as u16);
    (*E).base.spritemap_pointer = addr_kSpritemap_Nothing_A8 as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
    if ((*E).mbee_parameter_2 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        let v1: ::core::ffi::c_int =
            (*E).mbee_parameter_2 as ::core::ffi::c_int & 0xf as ::core::ffi::c_int;
        (*E).base.x_pos = ((*E).base.x_pos).wrapping_add((g_word_A890CA[v1 as usize]) as u16);
        (*E).base.y_pos = ((*E).base.y_pos).wrapping_add((g_word_A890D2[v1 as usize]) as u16);
        (*E).mbee_var_F = ((0xa891dci32) as u16);
        (*E).base.current_instruction = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa890da))
            as *const uint8 as *mut uint16)
            .offset(v1 as isize);
        let mut i: ::core::ffi::c_int = 510 as ::core::ffi::c_int;
        while i >= 0i32 {
            *(((&raw mut game.g_ram as *mut uint8).offset(0x9100) as *mut uint16 as *mut uint8)
                .offset(i as uint16 as ::core::ffi::c_int as isize) as *mut uint16) = 255 as uint16;
            i -= 2 as ::core::ffi::c_int;
        }
    } else {
        (*E).mbee_var_F = ((0xa890f1i32) as u16);
        if ((*E).mbee_parameter_1 as ::core::ffi::c_int & 1 as ::core::ffi::c_int) != 0i32 {
            (*E).base.current_instruction = addr_stru_A8900E as uint16;
        } else {
            (*E).base.current_instruction = addr_stru_A88FFC as uint16;
        }
    };
}
pub unsafe fn MorphBallEye_Main(game: &mut Game) {
    if (ram_word!(game, collected_items) as ::core::ffi::c_int & 4 as ::core::ffi::c_int) != 0i32 {
        let E: *mut Enemy_MorphBallEye =
            Get_MorphBallEye(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
        CallEnemyPreInstr((((*E).mbee_var_F) as u32) | (0xa80000u32), game);
    }
}
pub unsafe fn MorphBallEye_Func_1(k: uint16, game: &mut Game) {
    if IsSamusWithinEnemy_Y(k, g_word_A89054, game) != 0u16
        && IsSamusWithinEnemy_X(k, g_word_A89050, game) != 0u16
    {
        let E: *mut Enemy_MorphBallEye = Get_MorphBallEye(k as ::core::ffi::c_int, game);
        (*E).mbee_var_E = 32 as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        if ((*E).mbee_parameter_1 as ::core::ffi::c_int & 1 as ::core::ffi::c_int) != 0i32 {
            (*E).base.current_instruction = addr_kMorphBallEye_Ilist_9026 as uint16;
        } else {
            (*E).base.current_instruction = addr_kMorphBallEye_Ilist_9014 as uint16;
        }
        (*E).mbee_var_F = ((0xa8912ei32) as u16);
    }
}
pub unsafe fn MorphBallEye_Func_2(k: uint16, game: &mut Game) {
    let E: *mut Enemy_MorphBallEye = Get_MorphBallEye(k as ::core::ffi::c_int, game);
    let v2: bool = (*E).mbee_var_E == 1u16;
    (*E).mbee_var_E = (*E).mbee_var_E.wrapping_sub(1);
    let v3: bool = ((*E).mbee_var_E as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32;
    if v2 || v3 {
        QueueSfx2_Max6(0x17 as uint16, game);
        SpawnMorphBallEyeBeamHdma(game);
        (*E).mbee_var_F = ((0xa89160i32) as u16);
        (*E).mbee_var_D = CalculateAngleFromXY(
            (ram_word!(game, samus_x_pos)).wrapping_sub((*E).base.x_pos),
            (ram_word!(game, samus_y_pos)).wrapping_sub((*E).base.y_pos),
            game,
        );
    }
}
pub unsafe fn MorphBallEye_Func_3(k: uint16, game: &mut Game) {
    let E: *mut Enemy_MorphBallEye = Get_MorphBallEye(k as ::core::ffi::c_int, game);
    if IsSamusWithinEnemy_Y(k, g_word_A89056, game) != 0u16
        && IsSamusWithinEnemy_X(k, g_word_A89052, game) != 0u16
    {
        let v3: uint16 = CalculateAngleFromXY(
            (ram_word!(game, samus_x_pos)).wrapping_sub((*E).base.x_pos),
            (ram_word!(game, samus_y_pos)).wrapping_sub((*E).base.y_pos),
            game,
        );
        (*E).mbee_var_D = v3;
        (*E).base.current_instruction = (((v3 as ::core::ffi::c_int & 0xf0 as ::core::ffi::c_int)
            >> 2 as ::core::ffi::c_int)
            - 28756 as ::core::ffi::c_int) as uint16;
    } else {
        QueueSfx2_Max6(0x71 as uint16, game);
        (*E).mbee_var_C = 0 as uint16;
        (*E).mbee_var_E = 32 as uint16;
        if ((*E).mbee_parameter_1 as ::core::ffi::c_int & 1 as ::core::ffi::c_int) != 0i32 {
            (*E).base.current_instruction = addr_kMorphBallEye_Ilist_9002 as uint16;
        } else {
            (*E).base.current_instruction = addr_kMorphBallEye_Ilist_8FF0 as uint16;
        }
        (*E).mbee_var_F = ((0xa891cei32) as u16);
    }
    (*E).base.instruction_timer = 1 as uint16;
}
pub unsafe fn MorphBallEye_Func_4(k: uint16, game: &mut Game) {
    let E: *mut Enemy_MorphBallEye = Get_MorphBallEye(k as ::core::ffi::c_int, game);
    let v2: bool = (*E).mbee_var_E == 1u16;
    (*E).mbee_var_E = (*E).mbee_var_E.wrapping_sub(1);
    let v3: bool = ((*E).mbee_var_E as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32;
    if v2 || v3 {
        (*E).mbee_var_F = ((0xa890f1i32) as u16);
    }
}
pub unsafe fn Fune_Instr_2(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    QueueSfx2_Max9(0x1f as uint16, game);
    jp
}
pub unsafe fn Fune_Instr_6(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1995) as *mut uint16) =
        *(&raw mut (*(Get_Fune as unsafe fn(::core::ffi::c_int, &mut Game) -> *mut Enemy_Fune)(
            ram_word!(game, cur_enemy_index) as ::core::ffi::c_int,
            game,
        ))
        .fune_parameter_2 as *mut uint8)
            .offset(0) as uint16;
    SpawnEprojWithGfx(
        0 as uint16,
        ram_word!(game, cur_enemy_index),
        addr_stru_86DFBC as uint16,
        game,
    );
    jp
}
pub unsafe fn Fune_Instr_7(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1995) as *mut uint16) =
        *(&raw mut (*(Get_Fune as unsafe fn(::core::ffi::c_int, &mut Game) -> *mut Enemy_Fune)(
            ram_word!(game, cur_enemy_index) as ::core::ffi::c_int,
            game,
        ))
        .fune_parameter_2 as *mut uint8)
            .offset(0) as uint16;
    SpawnEprojWithGfx(
        1 as uint16,
        ram_word!(game, cur_enemy_index),
        addr_stru_86DFBC as uint16,
        game,
    );
    jp
}
pub unsafe fn Fune_Instr_1(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1995) as *mut uint16) =
        *(&raw mut (*(Get_Fune as unsafe fn(::core::ffi::c_int, &mut Game) -> *mut Enemy_Fune)(
            ram_word!(game, cur_enemy_index) as ::core::ffi::c_int,
            game,
        ))
        .fune_parameter_2 as *mut uint8)
            .offset(0) as uint16;
    SpawnEprojWithGfx(
        0 as uint16,
        ram_word!(game, cur_enemy_index),
        addr_stru_86DFCA as uint16,
        game,
    );
    jp
}
pub unsafe fn Fune_Instr_4(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1995) as *mut uint16) =
        *(&raw mut (*(Get_Fune as unsafe fn(::core::ffi::c_int, &mut Game) -> *mut Enemy_Fune)(
            ram_word!(game, cur_enemy_index) as ::core::ffi::c_int,
            game,
        ))
        .fune_parameter_2 as *mut uint8)
            .offset(0) as uint16;
    SpawnEprojWithGfx(
        1 as uint16,
        ram_word!(game, cur_enemy_index),
        addr_stru_86DFCA as uint16,
        game,
    );
    jp
}
pub unsafe fn Fune_Instr_3(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let E: *mut Enemy_Fune = Get_Fune(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).fune_var_A = ((*E).fune_var_A).wrapping_add(4u16);
    (*E).fune_var_B = ((0xa89737i32) as u16);
    if (*E).fune_var_D != 0 {
        (*E).fune_var_B = ((0xa8975ci32) as u16);
    }
    jp
}
pub unsafe fn Fune_Instr_5(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let E: *mut Enemy_Fune = Get_Fune(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).fune_var_A = ((*E).fune_var_A).wrapping_add(4u16);
    (*E).fune_var_B = ((0xa89737i32) as u16);
    if (*E).fune_var_D != 0 {
        (*E).fune_var_B = ((0xa8975ci32) as u16);
    }
    jp
}
pub unsafe fn Fune_Init(game: &mut Game) {
    let E: *mut Enemy_Fune = Get_Fune(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).fune_var_A = addr_off_A896D7 as uint16;
    (*E).fune_var_B = ((0xa89737i32) as u16);
    let v1: uint16 = ((*E).fune_parameter_1) & (0xfu16);
    (*E).fune_var_D = v1;
    if v1 != 0 {
        (*E).fune_var_A = addr_off_A896DF as uint16;
        (*E).fune_var_B = ((0xa8975ci32) as u16);
    }
    if ((*E).fune_parameter_1 as ::core::ffi::c_int & 0xf0 as ::core::ffi::c_int) != 0i32 {
        (*E).fune_var_A = ((*E).fune_var_A).wrapping_add(2u16);
    }
    Fune_Func_4(game);
    (*E).fune_var_C = *(&raw mut (*E).fune_parameter_2 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>())
            .wrapping_div(::core::mem::size_of::<uint8>())
            .wrapping_sub(1_usize) as isize,
    ) as uint16;
    (*E).fune_var_F = *(&raw mut (*E).fune_parameter_1 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>())
            .wrapping_div(::core::mem::size_of::<uint8>())
            .wrapping_sub(1_usize) as isize,
    ) as uint16;
    (*E).fune_var_E = 0 as uint16;
}
pub unsafe fn CallFuneFunc(ea: uint32, game: &mut Game) {
    match ea {
        11048759 => {
            Fune_Func_1(game);
        }
        11048796 => {
            Fune_Func_2(game);
        }
        11048846 => (),
        11048847 => (),
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn Fune_Main(game: &mut Game) {
    let E: *mut Enemy_Fune = Get_Fune(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    CallFuneFunc((((*E).fune_var_B) as u32) | (0xa80000u32), game);
}
pub unsafe fn Fune_Func_1(game: &mut Game) {
    let E: *mut Enemy_Fune = Get_Fune(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).fune_var_E = (*E).fune_var_E.wrapping_add(1);
    if (((*E).fune_var_E) as i16).wrapping_sub(((*E).fune_var_F) as i16) >= 0i16 {
        (*E).fune_var_A = ((*E).fune_var_A).wrapping_sub(4u16);
        Fune_Func_4(game);
        (*E).fune_var_B = ((0xa8978ei32) as u16);
        (*E).fune_var_E = 0 as uint16;
    }
}
pub unsafe fn Fune_Func_2(game: &mut Game) {
    let E: *mut Enemy_Fune = Get_Fune(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).fune_var_00 = (*E).fune_var_A;
    if Fune_Func_3(game) != 0 {
        let mut fune_var_00: uint16 = (*E).fune_var_00;
        (*E).fune_var_A = fune_var_00;
        fune_var_00 = (fune_var_00).wrapping_sub(4u16);
        (*E).fune_var_00 = fune_var_00;
        (*E).fune_var_A = fune_var_00;
        Fune_Func_4(game);
        (*E).fune_var_B = ((0xa8978fi32) as u16);
    }
    (*E).fune_var_A = (*E).fune_var_00;
}
pub unsafe fn Fune_Func_3(game: &mut Game) -> uint16 {
    let E: *mut Enemy_Fune = Get_Fune(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    IsSamusWithinEnemy_Y(ram_word!(game, cur_enemy_index), (*E).fune_var_C, game)
}
pub unsafe fn Fune_Func_4(game: &mut Game) {
    let E: *mut Enemy_Fune = Get_Fune(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
    (*E).base.current_instruction = *(RomPtr_A8((*E).fune_var_A, game) as *mut uint16);
}
pub unsafe fn WreckedShipGhost_Init(game: &mut Game) {
    let E: *mut Enemy_WreckedShipGhost =
        Get_WreckedShipGhost(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        | (kEnemyProps_DisableSamusColl as ::core::ffi::c_int
            | kEnemyProps_Tangible as ::core::ffi::c_int
            | kEnemyProps_Invisible as ::core::ffi::c_int)) as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
    (*E).base.current_instruction = addr_kWreckedShipGhost_Ilist_9A8C as uint16;
    (*E).wsgt_var_A = ((0xa89d13i32) as u16);
    (*E).wsgt_var_B = (g_word_A89AA4).wrapping_add(160u16);
    let v2: uint16 = u16::swap_bytes((*E).base.palette_index);
    let mut v3: uint16 =
        (16 as ::core::ffi::c_int * v2 as ::core::ffi::c_int + 256 as ::core::ffi::c_int) as uint16;
    let mut n: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
    loop {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16)
            .offset((v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
        v3 = (v3).wrapping_add(2u16);
        n -= 1;
        if (n < 0i32) {
            break;
        }
    }
}
pub unsafe fn CallWreckedShipGhost(ea: uint32, k: uint16, game: &mut Game) {
    match ea {
        11049794 => {
            WreckedShipGhost_Func_1(k, game);
        }
        11049901 => {
            WreckedShipGhost_Func_2(k, game);
        }
        11050089 => {
            WreckedShipGhost_Func_4(k, game);
        }
        11050122 => {
            WreckedShipGhost_Func_5(k, game);
        }
        11050259 => {
            WreckedShipGhost_Func_6(k, game);
        }
        11050294 => {
            WreckedShipGhost_Func_7(k, game);
        }
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn WreckedShipGhost_Main(game: &mut Game) {
    let E: *mut Enemy_WreckedShipGhost =
        Get_WreckedShipGhost(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    CallWreckedShipGhost(
        (((*E).wsgt_var_A) as u32) | (0xa80000u32),
        ram_word!(game, cur_enemy_index),
        game,
    );
}
pub unsafe fn WreckedShipGhost_Func_1(k: uint16, game: &mut Game) {
    WreckedShipGhost_Func_3(k, game);
    let E: *mut Enemy_WreckedShipGhost =
        Get_WreckedShipGhost(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let v3: uint16 = u16::swap_bytes((*E).base.palette_index);
    let mut v4: uint16 =
        (16 as ::core::ffi::c_int * v3 as ::core::ffi::c_int + 256 as ::core::ffi::c_int) as uint16;
    let mut n: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
    let mut v1: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
    loop {
        let v5: ::core::ffi::c_int = v4 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        if (*((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(v5 as isize)
            as ::core::ffi::c_int
            & 0x1f as ::core::ffi::c_int)
            - 31 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            let ref mut fresh12 = *((&raw mut game.g_ram as *mut uint8).offset(0xc000)
                as *mut uint16)
                .offset(v5 as isize);
            *fresh12 = (*fresh12).wrapping_add(1057u16);
            v1 -= 1;
        }
        v4 = (v4).wrapping_add(2u16);
        n -= 1;
        if (n == 0) {
            break;
        }
    }
    if ((v1 - 16 as ::core::ffi::c_int) as int16) >= 0i16 {
        (*E).wsgt_var_A = ((0xa89badi32) as u16);
        let v7: uint16 = u16::swap_bytes((*E).base.palette_index);
        let mut v8: uint16 = (16 as ::core::ffi::c_int * v7 as ::core::ffi::c_int
            + 256 as ::core::ffi::c_int) as uint16;
        let mut v9: uint16 = 0 as uint16;
        loop {
            *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16)
                .offset((v8 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
                kWreckedShipGhost_Palette
                    [(v9 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize];
            v8 = (v8).wrapping_add(2u16);
            v9 = (v9).wrapping_add(2u16);
            if (((v9) as i16).wrapping_sub(32i16) >= 0i16) {
                break;
            }
        }
    }
}
pub unsafe fn WreckedShipGhost_Func_2(k: uint16, game: &mut Game) {
    let v4: uint16 = WreckedShipGhost_Func_8(game);
    WreckedShipGhost_Func_3(ram_word!(game, cur_enemy_index), game);
    let E: *mut Enemy_WreckedShipGhost =
        Get_WreckedShipGhost(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    if (*E).wsgt_var_B as ::core::ffi::c_int | v4 as ::core::ffi::c_int == 0 {
        (*E).base.properties = ((*E).base.properties) & (0xfaffu16);
        (*E).wsgt_var_A = ((0xa89c8ai32) as u16);
        (*E).wsgt_var_00 = (*E).base.y_pos;
        (*E).wsgt_var_B = 1 as uint16;
        (*E).wsgt_var_C = 2 as uint16;
        (*E).wsgt_var_01 = 0 as uint16;
        (*E).wsgt_var_02 = g_word_A89AA2;
        (*E).wsgt_var_B = g_word_A89AA6;
        (*E).wsgt_var_05 = 4 as uint16;
        let v2: uint16 = ram_word!(game, samus_x_pos);
        (*E).wsgt_var_06 = ram_word!(game, samus_x_pos);
        (*E).wsgt_var_07 = v2;
        (*E).wsgt_var_08 = v2;
        (*E).wsgt_var_09 = 12 as uint16;
        let v3: uint16 = *(((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16
            as *mut uint8)
            .offset(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int as isize)
            as *mut uint16);
        (*E).wsgt_var_0A = v3;
        (*E).wsgt_var_0B = v3;
        (*E).wsgt_var_0C = v3;
        (*E).wsgt_var_0D = g_word_A89A9E;
        (*E).wsgt_var_0E = g_word_A89A9C;
    }
}
pub unsafe fn WreckedShipGhost_Func_3(k: uint16, game: &mut Game) {
    let mut v3: uint16 = 0;
    let mut wsgt_var_C: uint16 = 0;
    let current_block: u64;
    let mut v5: int16 = 0;
    let E: *mut Enemy_WreckedShipGhost = Get_WreckedShipGhost(k as ::core::ffi::c_int, game);
    let wsgt_var_B: uint16 = (*E).wsgt_var_B;
    if wsgt_var_B == 0 {
        current_block = 6804514963299881235;
    } else {
        v3 = 0;
        v3 = (wsgt_var_B).wrapping_sub(1u16);
        (*E).wsgt_var_B = v3;
        if v3 == 0 {
            wsgt_var_C = 0;
            wsgt_var_C = (*E).wsgt_var_C;
            v5 = g_word_A89ACC
                [(wsgt_var_C as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize]
                as int16;
            if v5 < 0i16 {
                (*E).wsgt_var_B = 0 as uint16;
                (*E).wsgt_var_C = 0 as uint16;
                return;
            }
            (*E).wsgt_var_B = v5 as uint16;
            (*E).wsgt_var_C = (wsgt_var_C).wrapping_add(2u16);
            if (wsgt_var_C as ::core::ffi::c_int & 2 as ::core::ffi::c_int) == 0i32 {
                current_block = 6804514963299881235;
            } else {
                current_block = 12800627514080957624;
            }
        } else {
            current_block = 12800627514080957624;
        }
    }
    if current_block == 6804514963299881235 {
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            & !(kEnemyProps_Invisible as ::core::ffi::c_int))
            as uint16;
    };
}
pub unsafe fn WreckedShipGhost_Func_4(k: uint16, game: &mut Game) {
    if WreckedShipGhost_Func_8(game) == 0 {
        let E: *mut Enemy_WreckedShipGhost =
            Get_WreckedShipGhost(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
        (*E).wsgt_var_A = ((0xa89d13i32) as u16);
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Invisible) as u16);
        (*E).wsgt_var_B = g_word_A89AA4;
    }
}
pub unsafe fn WreckedShipGhost_Func_5(k: uint16, game: &mut Game) {
    let E: *mut Enemy_WreckedShipGhost = Get_WreckedShipGhost(k as ::core::ffi::c_int, game);
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        ((*E).wsgt_var_02 as uint32) << 16 as ::core::ffi::c_int | (*E).wsgt_var_01 as uint32,
    );
    AddToHiLo(
        &raw mut (*E).wsgt_var_02,
        &raw mut (*E).wsgt_var_01,
        (if (((*E).base.y_pos) as i16).wrapping_sub(((*E).wsgt_var_00) as i16) < 0i16 {
            g_word_A89AA0 as ::core::ffi::c_int
        } else {
            -(g_word_A89AA0 as ::core::ffi::c_int)
        }) as uint32,
    );
    (*E).wsgt_var_B = (*E).wsgt_var_B.wrapping_sub(1);
    if (*E).wsgt_var_B == 0u16 {
        (*E).wsgt_var_A = ((0xa89c69i32) as u16);
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
        let t: ::core::ffi::c_int = u16::swap_bytes((*E).base.palette_index) as ::core::ffi::c_int;
        let mut v9: uint16 = (16 as ::core::ffi::c_int * t + 256 as ::core::ffi::c_int) as uint16;
        let mut n: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
        loop {
            *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16)
                .offset((v9 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
                0x7fff as uint16;
            v9 = (v9).wrapping_add(2u16);
            n -= 1;
            if ((n & 0x8000 as ::core::ffi::c_int) != 0i32) {
                break;
            }
        }
    }
}
pub unsafe fn WreckedShipGhost_Func_6(k: uint16, game: &mut Game) {
    let mut v3: uint16 = 0;
    let E: *mut Enemy_WreckedShipGhost = Get_WreckedShipGhost(k as ::core::ffi::c_int, game);
    let wsgt_var_B: uint16 = (*E).wsgt_var_B;
    if wsgt_var_B == 0 || {
        v3 = (wsgt_var_B).wrapping_sub(1u16);
        (*E).wsgt_var_B = v3;
        (*E).wsgt_var_B == 0u16
    } {
        (*E).wsgt_var_B = 1 as uint16;
        (*E).wsgt_var_C = 2 as uint16;
        (*E).wsgt_var_A = ((0xa89d36i32) as u16);
    }
}
pub unsafe fn WreckedShipGhost_Func_7(k: uint16, game: &mut Game) {
    let mut v2: int16 = 0;
    let mut v4: int16 = 0;
    let mut v7: int16 = 0;
    let mut v9: int16 = 0;
    let E: *mut Enemy_WreckedShipGhost = Get_WreckedShipGhost(k as ::core::ffi::c_int, game);
    let mut v3: uint16 = 0;
    let current_block_26: u64;
    if ((ram_word!(game, samus_x_pos)) as i16).wrapping_sub(((*E).wsgt_var_07) as i16) < 0i16
        || ((ram_word!(game, samus_x_pos)) as i16).wrapping_sub(((*E).wsgt_var_08) as i16) >= 0i16
        || {
            v2 = *(((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16 as *mut uint8)
                .offset(k as ::core::ffi::c_int as isize) as *mut uint16) as int16;
            (v2).wrapping_sub(((*E).wsgt_var_0B) as i16) < 0i16
        }
        || (v2).wrapping_sub(((*E).wsgt_var_0C) as i16) >= 0i16
    {
        (*E).wsgt_var_0D = g_word_A89A9E;
        v4 = 0 as int16;
        if ((ram_word!(game, samus_x_pos)) as i16).wrapping_sub(((*E).wsgt_var_06) as i16) >= 0i16 {
            v4 = (if ram_word!(game, samus_x_pos) == (*E).wsgt_var_06 {
                4 as ::core::ffi::c_int
            } else {
                8 as ::core::ffi::c_int
            }) as int16;
        }
        if v4 as ::core::ffi::c_int == (*E).wsgt_var_05 as ::core::ffi::c_int {
            v7 = 0 as int16;
            v9 = ((ram_word!(game, samus_y_pos)) as i16).wrapping_sub(((*E).wsgt_var_0A) as i16);
            if v9 >= 0i16 {
                v7 = (if v9 != 0i16 {
                    24 as ::core::ffi::c_int
                } else {
                    12 as ::core::ffi::c_int
                }) as int16;
            }
            if v7 as ::core::ffi::c_int == (*E).wsgt_var_09 as ::core::ffi::c_int {
                let v10: uint16 = ((*E).wsgt_var_0E).wrapping_sub(1u16);
                (*E).wsgt_var_0E = v10;
                if v10 == 0 {
                    current_block_26 = 1955728515491069284;
                } else {
                    current_block_26 = 3437258052017859086;
                }
            } else {
                (*E).wsgt_var_09 = v7 as uint16;
                (*E).wsgt_var_0E = g_word_A89A9C;
                current_block_26 = 3437258052017859086;
            }
        } else {
            (*E).wsgt_var_05 = v4 as uint16;
            (*E).wsgt_var_0E = g_word_A89A9C;
            current_block_26 = 3437258052017859086;
        }
    } else {
        v3 = 0;
        v3 = ((*E).wsgt_var_0D).wrapping_sub(1u16);
        (*E).wsgt_var_0D = v3;
        if v3 == 0 {
            (*E).wsgt_var_05 = 4 as uint16;
            (*E).wsgt_var_09 = 12 as uint16;
            current_block_26 = 1955728515491069284;
        } else {
            current_block_26 = 3437258052017859086;
        }
    }
    match current_block_26 {
        3437258052017859086 => {}
        _ => {
            (*E).wsgt_var_A = ((0xa89b42i32) as u16);
            (*E).wsgt_var_0D = g_word_A89A9E;
            (*E).wsgt_var_0E = g_word_A89A9C;
            let v6: ::core::ffi::c_int = (((*E).wsgt_var_09).wrapping_add((*E).wsgt_var_05))
                as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int;
            (*E).base.x_pos = (g_word_A89AA8[v6 as usize]).wrapping_add(
                *(((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16 as *mut uint8)
                    .offset(k as ::core::ffi::c_int as isize) as *mut uint16),
            );
            (*E).base.y_pos = (g_word_A89AA8[(v6 + 1 as ::core::ffi::c_int) as usize])
                .wrapping_add(
                    *(((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16
                        as *mut uint8)
                        .offset(k as ::core::ffi::c_int as isize)
                        as *mut uint16),
                );
            return;
        }
    }
    let v11: uint16 = ram_word!(game, samus_x_pos);
    (*E).wsgt_var_06 = ram_word!(game, samus_x_pos);
    (*E).wsgt_var_07 = (v11).wrapping_sub(g_word_A89D32);
    (*E).wsgt_var_08 = (g_word_A89D32).wrapping_add(ram_word!(game, samus_x_pos));
    let v13: uint16 = ram_word!(game, samus_y_pos);
    (*E).wsgt_var_0A = ram_word!(game, samus_y_pos);
    (*E).wsgt_var_0B = (v13).wrapping_sub(g_word_A89D34);
    (*E).wsgt_var_0C = (g_word_A89D34).wrapping_add(ram_word!(game, samus_y_pos));
}
pub unsafe fn WreckedShipGhost_Func_8(game: &mut Game) -> uint16 {
    let mut v0: uint16 = 0 as uint16;
    if ram_word!(game, door_transition_flag_enemies) == 0 {
        let E: *mut Enemy_WreckedShipGhost =
            Get_WreckedShipGhost(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
        let v2: uint16 = u16::swap_bytes((*E).base.palette_index);
        let mut v3: uint16 = (16 as ::core::ffi::c_int * v2 as ::core::ffi::c_int
            + 256 as ::core::ffi::c_int) as uint16;
        let r20: uint16 = (16 as ::core::ffi::c_int * v2 as ::core::ffi::c_int
            + 288 as ::core::ffi::c_int) as uint16;
        loop {
            let v4: ::core::ffi::c_int = v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            if *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16)
                .offset(v4 as isize)
                != *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                    .offset(v4 as isize)
            {
                let mut r18: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0xc200)
                    as *mut uint16)
                    .offset(v4 as isize))
                    & (0x1fu16);
                let v5: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0xc000)
                    as *mut uint16)
                    .offset(v4 as isize))
                    & (0x1fu16);
                if v5 != r18 {
                    let mut v6: uint16 = 0;
                    if ((v5) as i16).wrapping_sub((r18) as i16) >= 0i16 {
                        v6 = (v5).wrapping_sub(1u16);
                    } else {
                        v6 = (v5).wrapping_add(1u16);
                    }
                    r18 = v6;
                    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                        .offset(v4 as isize) = (v6 as ::core::ffi::c_int
                        | *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                            .offset(v4 as isize) as ::core::ffi::c_int
                            & 0xffe0 as ::core::ffi::c_int)
                        as uint16;
                    v0 = v0.wrapping_add(1);
                }
                r18 = (*((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16)
                    .offset(v4 as isize))
                    & (0x3e0u16);
                let v7: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0xc000)
                    as *mut uint16)
                    .offset(v4 as isize))
                    & (0x3e0u16);
                if v7 != r18 {
                    let mut v8: uint16 = 0;
                    if ((v7) as i16).wrapping_sub((r18) as i16) >= 0i16 {
                        v8 = (v7).wrapping_sub(32u16);
                    } else {
                        v8 = (v7).wrapping_add(32u16);
                    }
                    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                        .offset(v4 as isize) = (v8 as ::core::ffi::c_int
                        | *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                            .offset(v4 as isize) as ::core::ffi::c_int
                            & 0xfc1f as ::core::ffi::c_int)
                        as uint16;
                    v0 = v0.wrapping_add(1);
                }
                r18 = (*((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16)
                    .offset(v4 as isize))
                    & (0x7c00u16);
                let v9: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0xc000)
                    as *mut uint16)
                    .offset(v4 as isize))
                    & (0x7c00u16);
                if v9 != r18 {
                    let mut v10: uint16 = 0;
                    if ((v9) as i16).wrapping_sub((r18) as i16) >= 0i16 {
                        v10 = (v9).wrapping_sub(1024u16);
                    } else {
                        v10 = (v9).wrapping_add(1024u16);
                    }
                    r18 = v10;
                    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                        .offset(v4 as isize) = (v10 as ::core::ffi::c_int
                        | *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                            .offset(v4 as isize) as ::core::ffi::c_int
                            & 0x83ff as ::core::ffi::c_int)
                        as uint16;
                    v0 = v0.wrapping_add(1);
                }
            }
            v3 = (v3).wrapping_add(2u16);
            if (((v3) as i16).wrapping_sub((r20) as i16) >= 0i16) {
                break;
            }
        }
    }
    v0
}
pub unsafe fn YappingMaw_Instr_2(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let E: *mut Enemy_YappingMaw =
        Get_YappingMaw(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).ymw_var_32 =
        *(game.g_rom.offset(crate::rtl::lorom_offset(0xa8a0ab)) as *const uint8 as *mut uint16);
    (*E).ymw_var_33 =
        *(game.g_rom.offset(crate::rtl::lorom_offset(0xa8a0ad)) as *const uint8 as *mut uint16);
    jp
}
pub unsafe fn YappingMaw_Instr_4(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let E: *mut Enemy_YappingMaw =
        Get_YappingMaw(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).ymw_var_32 =
        *(game.g_rom.offset(crate::rtl::lorom_offset(0xa8a0c3)) as *const uint8 as *mut uint16);
    (*E).ymw_var_33 =
        *(game.g_rom.offset(crate::rtl::lorom_offset(0xa8a0c5)) as *const uint8 as *mut uint16);
    jp
}
pub unsafe fn YappingMaw_Instr_5(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let E: *mut Enemy_YappingMaw =
        Get_YappingMaw(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).ymw_var_32 =
        *(game.g_rom.offset(crate::rtl::lorom_offset(0xa8a0b3)) as *const uint8 as *mut uint16);
    (*E).ymw_var_33 =
        *(game.g_rom.offset(crate::rtl::lorom_offset(0xa8a0b5)) as *const uint8 as *mut uint16);
    jp
}
pub unsafe fn YappingMaw_Instr_7(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let E: *mut Enemy_YappingMaw =
        Get_YappingMaw(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).ymw_var_32 =
        *(game.g_rom.offset(crate::rtl::lorom_offset(0xa8a0bb)) as *const uint8 as *mut uint16);
    (*E).ymw_var_33 =
        *(game.g_rom.offset(crate::rtl::lorom_offset(0xa8a0bd)) as *const uint8 as *mut uint16);
    jp
}
pub unsafe fn YappingMaw_Instr_3(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let E: *mut Enemy_YappingMaw =
        Get_YappingMaw(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).ymw_var_32 =
        *(game.g_rom.offset(crate::rtl::lorom_offset(0xa8a0a7)) as *const uint8 as *mut uint16);
    (*E).ymw_var_33 =
        *(game.g_rom.offset(crate::rtl::lorom_offset(0xa8a0a9)) as *const uint8 as *mut uint16);
    jp
}
pub unsafe fn YappingMaw_Instr_6(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let E: *mut Enemy_YappingMaw =
        Get_YappingMaw(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).ymw_var_32 =
        *(game.g_rom.offset(crate::rtl::lorom_offset(0xa8a0b7)) as *const uint8 as *mut uint16);
    (*E).ymw_var_33 =
        *(game.g_rom.offset(crate::rtl::lorom_offset(0xa8a0b9)) as *const uint8 as *mut uint16);
    jp
}
pub unsafe fn YappingMaw_Instr_1(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    if (*Get_YappingMaw(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game)).ymw_var_36
        == 0
    {
        QueueSfx2_Max6(0x2f as uint16, game);
    }
    jp
}
pub unsafe fn YappingMaw_Init(game: &mut Game) {
    let mut v1: int16 = 0;
    let E: *mut Enemy_YappingMaw =
        Get_YappingMaw(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).ymw_var_0C = (*E).base.x_pos;
    (*E).ymw_var_0D = (*E).base.y_pos;
    (*E).ymw_var_00 = 0 as uint16;
    (*E).ymw_var_01 = 0 as uint16;
    (*E).ymw_var_02 = 0 as uint16;
    (*E).ymw_var_03 = 0 as uint16;
    (*E).ymw_var_04 = 0 as uint16;
    (*E).ymw_var_05 = 0 as uint16;
    (*E).ymw_var_06 = 0 as uint16;
    (*E).ymw_var_07 = 0 as uint16;
    (*E).ymw_var_30 = 0 as uint16;
    (*E).ymw_var_F = (*E).ymw_parameter_1;
    (*E).ymw_var_E = 64 as uint16;
    (*E).base.current_instruction = addr_kYappingMaw_Ilist_9F6F as uint16;
    let mut R36: uint16 = 57 as uint16;
    let mut R34: uint16 = 8 as uint16;
    if (*E).ymw_parameter_2 == 0 {
        (*E).base.current_instruction = addr_kYappingMaw_Ilist_9FC7 as uint16;
        R36 = 56 as uint16;
        R34 = -(8 as ::core::ffi::c_int) as uint16;
    }
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
    (*E).ymw_var_A = ((0xa8a235i32) as u16);
    (*E).ymw_var_45 = ((*E).base.palette_index) & (0xe00u16);
    v1 = 3 as int16;
    (*E).ymw_var_44 = 3 as uint16;
    loop {
        SpawnEprojWithGfx(
            v1 as uint16,
            ram_word!(game, cur_enemy_index),
            addr_kEproj_YappingMawsBody as uint16,
            game,
        );
        v1 = (((*E).ymw_var_44) as i16).wrapping_sub(1i16);
        (*E).ymw_var_44 = v1 as uint16;
        if (v1 < 0i16) {
            break;
        }
    }
    let v3: uint16 = ((*E).base.vram_tiles_index) | ((*E).base.palette_index);
    (*E).ymw_var_47 = v3;
    (*E).ymw_var_46 = CreateSpriteAtPos(
        (*E).base.x_pos,
        (R34).wrapping_add((*E).base.y_pos),
        R36,
        v3,
        game,
    );
}
pub unsafe fn CallYappingMawFunc(ea: uint32, game: &mut Game) {
    match ea {
        11051573 => {
            YappingMaw_Func_1(game);
        }
        11051660 => {
            YappingMaw_Func_2(game);
        }
        11052101 => {
            YappingMaw_Func_8(game);
        }
        11052682 => {
            YappingMaw_Func_11(game);
        }
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn YappingMaw_Main(game: &mut Game) {
    let E: *mut Enemy_YappingMaw =
        Get_YappingMaw(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).ymw_var_35 = (*E).ymw_var_35.wrapping_sub(1);
    (*E).ymw_var_36 = CheckIfEnemyIsOnScreen(game);
    CallYappingMawFunc((((*E).ymw_var_A) as u32) | (0xa80000u32), game);
    YappingMaw_Func_15(game);
    YappingMaw_Func_14(game);
    YappingMaw_Func_13(game);
    YappingMaw_Func_12(game);
}
pub unsafe fn YappingMaw_Func_1(game: &mut Game) {
    let E: *mut Enemy_YappingMaw =
        Get_YappingMaw(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let pair: PairU16 = EnemyFunc_ACA8(
        Point16U {
            x: (*E).base.x_pos,
            y: (*E).base.y_pos,
        },
        Point16U {
            x: ram_word!(game, samus_x_pos),
            y: ram_word!(game, samus_y_pos),
        },
        game,
    );
    let v2: uint16 = Abs16(pair.k);
    (*E).ymw_var_08 = v2;
    if v2 as ::core::ffi::c_int - 32 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        (*E).ymw_var_35 = 48 as uint16;
    } else if ((v2) as i16).wrapping_sub(((*E).ymw_var_F) as i16) < 0i16 {
        if (*E).ymw_var_08 as ::core::ffi::c_int - 64 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            (*E).ymw_var_08 = 64 as uint16;
        }
        (*E).ymw_var_0A = pair.j;
        (*E).ymw_var_A = ((0xa8a28ci32) as u16);
    }
}
pub unsafe fn YappingMaw_Func_2(game: &mut Game) {
    let E: *mut Enemy_YappingMaw =
        Get_YappingMaw(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).ymw_var_B = 0 as uint16;
    (*E).ymw_var_C = 0 as uint16;
    (*E).ymw_var_D = 0 as uint16;
    (*E).ymw_var_09 = ((*E).ymw_var_08 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
    let v1: uint16 = ((64u8).wrapping_sub(((*E).ymw_var_0A) as u8)) as uint16;
    (*E).ymw_var_0B = v1;
    if v1 as ::core::ffi::c_int - 128 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        (*E).ymw_var_2F = 0 as uint16;
    } else {
        (*E).ymw_var_2F = 1 as uint16;
    }
    YappingMaw_Func_3(game);
    let v2: uint16 = (2 as ::core::ffi::c_int
        * (((((*E).ymw_var_0A) as u8).wrapping_add(16u8)) as ::core::ffi::c_int
            >> 5 as ::core::ffi::c_int)) as uint16;
    (*E).ymw_var_34 = v2;
    (*E).base.current_instruction = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa8a097))
        as *const uint8 as *mut uint16)
        .offset((v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
    (*E).ymw_var_32 = *((game.g_rom.offset(crate::rtl::lorom_offset(0xa8a0a7)) as *const uint8
        as *mut uint16 as *mut uint8)
        .offset(((2u16).wrapping_mul(v2)) as ::core::ffi::c_int as isize)
        as *mut uint16);
    (*E).ymw_var_33 = *((game.g_rom.offset(crate::rtl::lorom_offset(0xa8a0a9)) as *const uint8
        as *mut uint16 as *mut uint8)
        .offset(((2u16).wrapping_mul(v2)) as ::core::ffi::c_int as isize)
        as *mut uint16);
    (*E).ymw_var_A = ((0xa8a445i32) as u16);
}
pub unsafe fn YappingMaw_Func_3(game: &mut Game) {
    let E: *mut Enemy_YappingMaw =
        Get_YappingMaw(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).ymw_var_0E = YappingMaw_Func_16(0x80 as uint16, (*E).ymw_var_09, game);
    (*E).ymw_var_0F = YappingMaw_Func_17(
        0x80 as uint16,
        ((*E).ymw_var_09 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16,
        game,
    );
}
pub unsafe fn YappingMaw_Func_4(game: &mut Game) {
    let E: *mut Enemy_YappingMaw =
        Get_YappingMaw(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let varE32: uint16 = (*E).ymw_var_00;
    (*E).ymw_var_10 = YappingMaw_Func_16(0 as uint16, varE32, game);
    (*E).ymw_var_11 = YappingMaw_Func_17(0 as uint16, varE32, game);
    (*E).ymw_var_27 =
        (YappingMaw_Func_16((*E).ymw_var_0B, varE32, game)).wrapping_sub((*E).ymw_var_10);
    (*E).ymw_var_28 =
        (YappingMaw_Func_17((*E).ymw_var_0B, varE32, game)).wrapping_sub((*E).ymw_var_11);
}
pub unsafe fn YappingMaw_Func_5(game: &mut Game) {
    let E: *mut Enemy_YappingMaw =
        Get_YappingMaw(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let varE32: uint16 = (*E).ymw_var_02;
    (*E).ymw_var_10 = YappingMaw_Func_16(0 as uint16, varE32, game);
    (*E).ymw_var_11 = YappingMaw_Func_17(0 as uint16, varE32, game);
    (*E).ymw_var_29 =
        (YappingMaw_Func_16((*E).ymw_var_0B, varE32, game)).wrapping_sub((*E).ymw_var_10);
    (*E).ymw_var_2A =
        (YappingMaw_Func_17((*E).ymw_var_0B, varE32, game)).wrapping_sub((*E).ymw_var_11);
}
pub unsafe fn YappingMaw_Func_6(game: &mut Game) {
    let E: *mut Enemy_YappingMaw =
        Get_YappingMaw(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let varE32: uint16 = (*E).ymw_var_04;
    (*E).ymw_var_10 = YappingMaw_Func_16(0 as uint16, varE32, game);
    (*E).ymw_var_11 = YappingMaw_Func_17(0 as uint16, varE32, game);
    (*E).ymw_var_2B =
        (YappingMaw_Func_16((*E).ymw_var_0B, varE32, game)).wrapping_sub((*E).ymw_var_10);
    (*E).ymw_var_2C =
        (YappingMaw_Func_17((*E).ymw_var_0B, varE32, game)).wrapping_sub((*E).ymw_var_11);
}
pub unsafe fn YappingMaw_Func_7(game: &mut Game) {
    let E: *mut Enemy_YappingMaw =
        Get_YappingMaw(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let varE32: uint16 = (*E).ymw_var_06;
    (*E).ymw_var_10 = YappingMaw_Func_16(0 as uint16, varE32, game);
    (*E).ymw_var_11 = YappingMaw_Func_17(0 as uint16, varE32, game);
    (*E).ymw_var_2D =
        (YappingMaw_Func_16((*E).ymw_var_0B, varE32, game)).wrapping_sub((*E).ymw_var_10);
    (*E).ymw_var_2E =
        (YappingMaw_Func_17((*E).ymw_var_0B, varE32, game)).wrapping_sub((*E).ymw_var_11);
}
pub unsafe fn YappingMaw_Func_8(game: &mut Game) {
    let E: *mut Enemy_YappingMaw =
        Get_YappingMaw(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let r20: uint16 = ((*E).ymw_var_B as ::core::ffi::c_int >> 2 as ::core::ffi::c_int) as uint16;
    if (*E).ymw_var_2F != 0 {
        (*E).ymw_var_21 = (128u16).wrapping_add(r20);
        (*E).ymw_var_22 = (128 as ::core::ffi::c_int
            + r20 as ::core::ffi::c_int * 2 as ::core::ffi::c_int)
            as uint16;
        (*E).ymw_var_23 = (128 as ::core::ffi::c_int
            + r20 as ::core::ffi::c_int * 3 as ::core::ffi::c_int)
            as uint16;
        (*E).ymw_var_24 = (128 as ::core::ffi::c_int
            + r20 as ::core::ffi::c_int * 4 as ::core::ffi::c_int)
            as uint16;
    } else {
        (*E).ymw_var_21 = (128u16).wrapping_sub(r20);
        (*E).ymw_var_22 = (128 as ::core::ffi::c_int
            - r20 as ::core::ffi::c_int * 2 as ::core::ffi::c_int)
            as uint16;
        (*E).ymw_var_23 = (128 as ::core::ffi::c_int
            - r20 as ::core::ffi::c_int * 3 as ::core::ffi::c_int)
            as uint16;
        (*E).ymw_var_24 = (128 as ::core::ffi::c_int
            - r20 as ::core::ffi::c_int * 4 as ::core::ffi::c_int)
            as uint16;
    }
    let mut varE32: uint16 = (*E).ymw_var_09;
    (*E).ymw_var_00 =
        (YappingMaw_Func_16((*E).ymw_var_21, varE32, game)).wrapping_sub((*E).ymw_var_0E);
    (*E).ymw_var_02 =
        (YappingMaw_Func_16((*E).ymw_var_22, varE32, game)).wrapping_sub((*E).ymw_var_0E);
    (*E).ymw_var_04 =
        (YappingMaw_Func_16((*E).ymw_var_23, varE32, game)).wrapping_sub((*E).ymw_var_0E);
    (*E).ymw_var_06 =
        (YappingMaw_Func_16((*E).ymw_var_24, varE32, game)).wrapping_sub((*E).ymw_var_0E);
    varE32 = ((*E).ymw_var_09 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as uint16;
    (*E).ymw_var_01 =
        (YappingMaw_Func_17((*E).ymw_var_21, varE32, game)).wrapping_sub((*E).ymw_var_0F);
    (*E).ymw_var_03 =
        (YappingMaw_Func_17((*E).ymw_var_22, varE32, game)).wrapping_sub((*E).ymw_var_0F);
    (*E).ymw_var_05 =
        (YappingMaw_Func_17((*E).ymw_var_23, varE32, game)).wrapping_sub((*E).ymw_var_0F);
    (*E).ymw_var_07 =
        (YappingMaw_Func_17((*E).ymw_var_24, varE32, game)).wrapping_sub((*E).ymw_var_0F);
    YappingMaw_Func_4(game);
    YappingMaw_Func_5(game);
    YappingMaw_Func_6(game);
    YappingMaw_Func_7(game);
    (*E).ymw_var_00 = ((*E).ymw_var_00).wrapping_add((*E).ymw_var_27);
    (*E).ymw_var_01 = ((*E).ymw_var_01).wrapping_add((*E).ymw_var_28);
    (*E).ymw_var_02 = ((*E).ymw_var_02).wrapping_add((*E).ymw_var_29);
    (*E).ymw_var_03 = ((*E).ymw_var_03).wrapping_add((*E).ymw_var_2A);
    (*E).ymw_var_04 = ((*E).ymw_var_04).wrapping_add((*E).ymw_var_2B);
    (*E).ymw_var_05 = ((*E).ymw_var_05).wrapping_add((*E).ymw_var_2C);
    (*E).ymw_var_06 = ((*E).ymw_var_06).wrapping_add((*E).ymw_var_2D);
    (*E).ymw_var_07 = ((*E).ymw_var_07).wrapping_add((*E).ymw_var_2E);
    (*E).base.x_pos = ((*E).ymw_var_06).wrapping_add((*E).ymw_var_0C);
    (*E).base.y_pos = ((*E).ymw_var_07).wrapping_add((*E).ymw_var_0D);
    YappingMaw_Func_9(ram_word!(game, cur_enemy_index), game);
    if (*E).ymw_var_B as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
        if (*E).ymw_var_B as ::core::ffi::c_int - 128 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            (*E).ymw_var_B = 128 as uint16;
            (*E).ymw_var_C = 0 as uint16;
            (*E).ymw_var_D = ((*E).ymw_var_D).wrapping_add(4u16);
        }
    } else {
        (*E).ymw_var_A = ((0xa8a68ai32) as u16);
        (*E).ymw_var_35 = 48 as uint16;
        if (*E).ymw_var_34 == 4u16 {
            (*E).base.current_instruction = addr_kYappingMaw_Ilist_A01F as uint16;
        } else if (*E).ymw_var_34 == 12u16 {
            (*E).base.current_instruction = addr_kYappingMaw_Ilist_A03D as uint16;
        } else {
            (*E).base.current_instruction = addr_off_A8A025 as uint16;
        }
        if (*E).ymw_parameter_2 == 0 {
            if (*E).ymw_var_34 == 4u16 {
                (*E).base.current_instruction = addr_kYappingMaw_Ilist_A05B as uint16;
            } else {
                if (*E).ymw_var_34 != 12u16 {
                    (*E).base.current_instruction = addr_off_A8A061 as uint16;
                    return;
                }
                (*E).base.current_instruction = addr_kYappingMaw_Ilist_A079 as uint16;
            }
        } else {
            return;
        }
    }
    if (*E).ymw_var_30 != 0 {
        YappingMaw_Func_10(game);
    }
}
pub unsafe fn YappingMaw_Func_9(k: uint16, game: &mut Game) {
    let E: *mut Enemy_YappingMaw = Get_YappingMaw(k as ::core::ffi::c_int, game);
    let v2: ::core::ffi::c_int = (*E).ymw_var_D as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    AddToHiLo(
        &raw mut (*E).ymw_var_B,
        &raw mut (*E).ymw_var_C,
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0xa2838f)) as *const uint8 as *mut uint16)
            .offset((v2 + 1 as ::core::ffi::c_int) as isize) as uint32)
            << 16 as ::core::ffi::c_int
            | *(game.g_rom.offset(crate::rtl::lorom_offset(0xa2838f)) as *const uint8
                as *mut uint16)
                .offset(v2 as isize) as uint32,
    );
    (*E).ymw_var_D = ((*E).ymw_var_D).wrapping_add(8u16);
}
pub unsafe fn YappingMaw_Func_10(game: &mut Game) {
    CallSomeSamusCode(3 as uint16, game);
    let E: *mut Enemy_YappingMaw =
        Get_YappingMaw(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) =
        ((*E).ymw_var_32).wrapping_add((*E).base.x_pos);
    *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16) =
        ((*E).ymw_var_33).wrapping_add((*E).base.y_pos);
    EnemyFunc_B7A1(game);
}
pub unsafe fn YappingMaw_Func_11(game: &mut Game) {
    let E: *mut Enemy_YappingMaw =
        Get_YappingMaw(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    if (*E).ymw_var_30 != 0 {
        YappingMaw_Func_10(game);
    }
    (*E).ymw_var_E = (*E).ymw_var_E.wrapping_sub(1);
    if ((*E).ymw_var_E as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32
        && ram_word!(game, samus_input_handler) as ::core::ffi::c_int
            != 0x90e91d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa60) as *mut uint16) =
            ((0x90e913i32) as u16);
        (*E).ymw_var_30 = 0 as uint16;
        (*E).ymw_var_35 = 48 as uint16;
        (*E).ymw_var_E = 64 as uint16;
        (*E).ymw_var_A = ((0xa8a235i32) as u16);
    }
}
pub unsafe fn YappingMaw_Func_12(game: &mut Game) {
    let E: *mut Enemy_YappingMaw =
        Get_YappingMaw(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let v1: ::core::ffi::c_int = (*E).ymw_var_40 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize) =
        (*E).ymw_var_0C;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize) =
        (*E).ymw_var_0D;
}
pub unsafe fn YappingMaw_Func_13(game: &mut Game) {
    let E: *mut Enemy_YappingMaw =
        Get_YappingMaw(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let v1: ::core::ffi::c_int = (*E).ymw_var_41 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize) =
        ((*E).ymw_var_00).wrapping_add((*E).ymw_var_0C);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize) =
        ((*E).ymw_var_01).wrapping_add((*E).ymw_var_0D);
}
pub unsafe fn YappingMaw_Func_14(game: &mut Game) {
    let E: *mut Enemy_YappingMaw =
        Get_YappingMaw(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let v1: ::core::ffi::c_int = (*E).ymw_var_42 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize) =
        ((*E).ymw_var_02).wrapping_add((*E).ymw_var_0C);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize) =
        ((*E).ymw_var_03).wrapping_add((*E).ymw_var_0D);
}
pub unsafe fn YappingMaw_Func_15(game: &mut Game) {
    let E: *mut Enemy_YappingMaw =
        Get_YappingMaw(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let v1: ::core::ffi::c_int = (*E).ymw_var_43 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a4b) as *mut uint16).offset(v1 as isize) =
        ((*E).ymw_var_04).wrapping_add((*E).ymw_var_0C);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1a93) as *mut uint16).offset(v1 as isize) =
        ((*E).ymw_var_05).wrapping_add((*E).ymw_var_0D);
}
pub unsafe fn YappingMaw_Func_16(a: uint16, varE32: uint16, game: &mut Game) -> uint16 {
    YappingMaw_Func_17((a).wrapping_sub(64u16), varE32, game)
}
pub unsafe fn YappingMaw_Func_17(a: uint16, varE32: uint16, game: &mut Game) -> uint16 {
    let mut v1: int16 = 0;
    let mut r20: uint16 = 0 as uint16;
    let mut r22: uint16 = 0 as uint16;
    v1 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa0b1c3)) as *const uint8 as *mut uint16)
        .offset(-(a as ::core::ffi::c_int) as uint8 as isize) as int16;
    if v1 < 0i16 {
        v1 = -(v1 as ::core::ffi::c_int) as int16;
        r20 = r20.wrapping_add(1);
    }
    WriteReg(
        WRMPYA as uint16,
        *(&raw mut v1 as *mut uint8).offset(
            (::core::mem::size_of::<int16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1_usize) as isize,
        ),
        game,
    );
    if varE32 as uint8 != 0 {
        let prod: uint16 = Mult8x8(
            *(&raw mut v1 as *mut uint8).offset(
                (::core::mem::size_of::<int16>())
                    .wrapping_div(::core::mem::size_of::<uint8>())
                    .wrapping_sub(1_usize) as isize,
            ),
            varE32 as uint8,
        );
        r22 = (2 as ::core::ffi::c_int
            * ((prod as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int)
                >> 8 as ::core::ffi::c_int)) as uint16;
        if r20 != 0 {
            r22 = (2 as ::core::ffi::c_int
                * (-(prod as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
                    >> 8 as ::core::ffi::c_int)
                | 0xff00 as ::core::ffi::c_int) as uint16;
        }
    }
    r22
}
pub unsafe fn YappingMaw_Touch(game: &mut Game) {
    let E: *mut Enemy_YappingMaw =
        Get_YappingMaw(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    if ((*E).ymw_var_35 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32
        && (*E).ymw_var_30 == 0
    {
        (*E).ymw_var_35 = 0 as uint16;
        (*E).ymw_var_30 = 1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xa60) as *mut uint16) =
            ((0x90e90ei32) as u16);
    }
}
pub unsafe fn YappingMaw_Shot(game: &mut Game) {
    NormalEnemyShotAi(game);
    let E: *mut Enemy_YappingMaw =
        Get_YappingMaw(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    if (*E).base.health != 0 {
        if (*E).base.frozen_timer != 0 {
            if ram_word!(game, samus_input_handler) as ::core::ffi::c_int
                != 0x90e91d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
            {
                *((&raw mut game.g_ram as *mut uint8).offset(0xa60) as *mut uint16) =
                    ((0x90e913i32) as u16);
            }
            (*E).ymw_var_30 = 0 as uint16;
        }
    } else {
        *(((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16 as *mut uint8)
            .offset((*E).ymw_var_40 as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
        *(((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16 as *mut uint8)
            .offset((*E).ymw_var_41 as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
        *(((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16 as *mut uint8)
            .offset((*E).ymw_var_42 as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
        *(((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16 as *mut uint8)
            .offset((*E).ymw_var_43 as ::core::ffi::c_int as isize) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xef78) as *mut uint16)
            .offset(((*E).ymw_var_46 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            0 as uint16;
        if ram_word!(game, samus_input_handler) as ::core::ffi::c_int
            != 0x90e91d as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0xa60) as *mut uint16) =
                ((0x90e913i32) as u16);
        }
        (*Get_YappingMaw(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game))
            .ymw_var_30 = 0 as uint16;
    };
}
pub unsafe fn YappingMaw_Frozen(game: &mut Game) {
    Enemy_NormalFrozenAI_A8(game);
    let E: *mut Enemy_YappingMaw =
        Get_YappingMaw(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    YappingMaw_Func_18((*E).ymw_var_40, game);
    YappingMaw_Func_18((*E).ymw_var_41, game);
    YappingMaw_Func_18((*E).ymw_var_42, game);
    YappingMaw_Func_18((*E).ymw_var_43, game);
    YappingMaw_Func_19(game);
}
pub unsafe fn YappingMaw_Func_18(j: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let E: *mut Enemy_YappingMaw =
        Get_YappingMaw(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x19bb) as *mut uint16).offset(v1 as isize) =
        ((*E).ymw_var_45 as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0x19bb) as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                & 0xf1ff as ::core::ffi::c_int) as uint16;
    if (*E).base.frozen_timer != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x19bb) as *mut uint16).offset(v1 as isize) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0x19bb) as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                & 0xf1ff as ::core::ffi::c_int
                | 0xc00 as ::core::ffi::c_int) as uint16;
        let frozen_timer: uint16 = (*E).base.frozen_timer;
        if frozen_timer as ::core::ffi::c_int - 90 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            if (frozen_timer as ::core::ffi::c_int & 2 as ::core::ffi::c_int) == 0i32 {
                *((&raw mut game.g_ram as *mut uint8).offset(0x19bb) as *mut uint16)
                    .offset(v1 as isize) = ((*E).ymw_var_45 as ::core::ffi::c_int
                    | *((&raw mut game.g_ram as *mut uint8).offset(0x19bb) as *mut uint16)
                        .offset(v1 as isize) as ::core::ffi::c_int
                        & 0xf1ff as ::core::ffi::c_int)
                    as uint16;
            }
        }
    }
}
pub unsafe fn YappingMaw_Func_19(game: &mut Game) {
    let E: *mut Enemy_YappingMaw =
        Get_YappingMaw(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let r18: uint16 = (*E).ymw_var_45;
    let r20: uint16 = (*E).ymw_var_46;
    *((&raw mut game.g_ram as *mut uint8).offset(0xf078) as *mut uint16)
        .offset((r20 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        (r18 as ::core::ffi::c_int
            | *((&raw mut game.g_ram as *mut uint8).offset(0xf078) as *mut uint16)
                .offset((r20 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int
                & 0xf1ff as ::core::ffi::c_int) as uint16;
    if (*E).base.frozen_timer != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0xf078) as *mut uint16)
            .offset((r20 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            (*((&raw mut game.g_ram as *mut uint8).offset(0xf078) as *mut uint16)
                .offset((r20 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int
                & 0xf1ff as ::core::ffi::c_int
                | 0xc00 as ::core::ffi::c_int) as uint16;
        let frozen_timer: uint16 = (*E).base.frozen_timer;
        if frozen_timer as ::core::ffi::c_int - 90 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            if (frozen_timer as ::core::ffi::c_int & 2 as ::core::ffi::c_int) == 0i32 {
                *((&raw mut game.g_ram as *mut uint8).offset(0xf078) as *mut uint16)
                    .offset((r20 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
                    (r18 as ::core::ffi::c_int
                        | *((&raw mut game.g_ram as *mut uint8).offset(0xf078) as *mut uint16)
                            .offset((r20 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                            as ::core::ffi::c_int
                            & 0xf1ff as ::core::ffi::c_int) as uint16;
            }
        }
    }
}
pub unsafe fn Kago_Init(game: &mut Game) {
    let E: *mut Enemy_Kago = Get_Kago(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_DisableSamusColl) as u16);
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
    (*E).kago_var_E = 0 as uint16;
    (*E).base.current_instruction = addr_kKago_Ilist_AB1E as uint16;
    (*E).kago_var_A = ((0xa8ab7bi32) as u16);
    (*E).kago_var_F = 0 as uint16;
    (*E).kago_var_04 = (*E).kago_parameter_1;
}
pub unsafe fn CallKagoFunc(ea: uint32, k: uint16, game: &mut Game) {
    match ea {
        11053947 => {
            Kago_Func_1(k, game);
        }
        11053953 => (),
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn Kago_Main(game: &mut Game) {
    let E: *mut Enemy_Kago = Get_Kago(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    CallKagoFunc(
        (((*E).kago_var_A) as u32) | (0xa80000u32),
        ram_word!(game, cur_enemy_index),
        game,
    );
}
pub unsafe fn Kago_Func_1(k: uint16, game: &mut Game) {
    (*Get_Kago(k as ::core::ffi::c_int, game)).kago_var_A = ((0xa8ab81i32) as u16);
}
pub unsafe fn Kago_Shot(game: &mut Game) {
    let mut v2: int16 = 0;
    NormalEnemyShotAi(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x183e) as *mut uint16) = 2 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1840) as *mut uint16) = 16 as uint16;
    let E: *mut Enemy_Kago = Get_Kago(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    if (*E).kago_var_B == 0 {
        (*E).kago_var_B = 1 as uint16;
        (*E).base.current_instruction = addr_kKago_Ilist_AB32 as uint16;
        (*E).base.instruction_timer = 1 as uint16;
    }
    v2 = (((*E).kago_var_04) as i16).wrapping_sub(1i16);
    (*E).kago_var_04 = v2 as uint16;
    if v2 < 0i16 {
        EnemyDeathAnimation(ram_word!(game, cur_enemy_index), 4 as uint16, game);
        (*E).kago_var_F = 1 as uint16;
    }
    SpawnEprojWithGfx(
        (*E).base.y_pos,
        ram_word!(game, cur_enemy_index),
        addr_loc_A8D02E as uint16,
        game,
    );
}
pub unsafe fn NorfairLavaMan_Instr_1(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    if CheckIfEnemyIsOnScreen(game) == 0 {
        QueueSfx2_Max6(*jp.offset(0), game);
    }
    jp.offset(1)
}
pub unsafe fn NorfairLavaMan_Instr_8(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let E: *mut Enemy_NorfairLavaMan =
        Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).base.y_pos = (*E).base.y_pos.wrapping_add(1);
    (*E).base.y_pos = (*E).base.y_pos.wrapping_add(1);
    jp
}
pub unsafe fn NorfairLavaMan_Instr_14(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let E: *mut Enemy_NorfairLavaMan =
        Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).base.y_pos = (*E).base.y_pos.wrapping_sub(1);
    (*E).base.y_pos = (*E).base.y_pos.wrapping_sub(1);
    jp
}
pub unsafe fn NorfairLavaMan_Instr_2(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game))
        .nlmn_var_01 = 1 as uint16;
    jp
}
pub unsafe fn NorfairLavaMan_Instr_7(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game))
        .nlmn_var_01 = 0 as uint16;
    jp
}
pub unsafe fn NorfairLavaMan_Instr_10(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let E: *mut Enemy_NorfairLavaMan =
        Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).base.y_pos = (*E).base.y_pos.wrapping_sub(1);
    let E1: *mut Enemy_NorfairLavaMan = Get_NorfairLavaMan(
        ram_word!(game, cur_enemy_index) as ::core::ffi::c_int + 64 as ::core::ffi::c_int,
        game,
    );
    (*E1).base.y_pos = (*E1).base.y_pos.wrapping_sub(1);
    jp
}
pub unsafe fn NorfairLavaMan_Instr_12(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let E: *mut Enemy_NorfairLavaMan =
        Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).base.y_pos = (*E).base.y_pos.wrapping_add(1);
    let E1: *mut Enemy_NorfairLavaMan = Get_NorfairLavaMan(
        ram_word!(game, cur_enemy_index) as ::core::ffi::c_int + 64 as ::core::ffi::c_int,
        game,
    );
    (*E1).base.y_pos = (*E1).base.y_pos.wrapping_add(1);
    jp
}
pub unsafe fn NorfairLavaMan_Instr_9(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let E: *mut Enemy_NorfairLavaMan =
        Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let v4: uint16 = ((*E).nlmn_var_03).wrapping_add(24u16);
    (*E).base.y_pos = v4;
    let E1: *mut Enemy_NorfairLavaMan = Get_NorfairLavaMan(
        ram_word!(game, cur_enemy_index) as ::core::ffi::c_int + 64 as ::core::ffi::c_int,
        game,
    );
    (*E1).base.y_pos = v4;
    (*E1).base.properties =
        ((*E1).base.properties as ::core::ffi::c_int & !(0x100 as ::core::ffi::c_int)) as uint16;
    let E2: *mut Enemy_NorfairLavaMan = Get_NorfairLavaMan(
        ram_word!(game, cur_enemy_index) as ::core::ffi::c_int + 128 as ::core::ffi::c_int,
        game,
    );
    (*E2).base.properties =
        ((*E2).base.properties as ::core::ffi::c_int & !(0x100 as ::core::ffi::c_int)) as uint16;
    jp
}
pub unsafe fn NorfairLavaMan_Instr_11(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let E: *mut Enemy_NorfairLavaMan =
        Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let nlmn_var_03: uint16 = (*E).nlmn_var_03;
    (*E).base.y_pos = nlmn_var_03;
    (*Get_NorfairLavaMan(
        ram_word!(game, cur_enemy_index) as ::core::ffi::c_int + 64 as ::core::ffi::c_int,
        game,
    ))
    .base
    .y_pos = nlmn_var_03;
    jp
}
pub unsafe fn NorfairLavaMan_Instr_13(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let E: *mut Enemy_NorfairLavaMan =
        Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let v4: uint16 = ((*E).nlmn_var_03).wrapping_add(4u16);
    (*E).base.y_pos = v4;
    let E1: *mut Enemy_NorfairLavaMan = Get_NorfairLavaMan(
        ram_word!(game, cur_enemy_index) as ::core::ffi::c_int + 64 as ::core::ffi::c_int,
        game,
    );
    (*E1).base.y_pos = v4;
    (*E1).base.properties = ((*E1).base.properties) | (0x100u16);
    let E2: *mut Enemy_NorfairLavaMan = Get_NorfairLavaMan(
        ram_word!(game, cur_enemy_index) as ::core::ffi::c_int + 128 as ::core::ffi::c_int,
        game,
    );
    (*E2).base.properties = ((*E2).base.properties) | (0x100u16);
    jp
}
pub unsafe fn NorfairLavaMan_Instr_5(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let E: *mut Enemy_NorfairLavaMan =
        Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    SpawnEprojWithGfx(
        (*E).nlmn_var_B,
        ram_word!(game, cur_enemy_index),
        addr_kEproj_LavaThrownByLavaman as uint16,
        game,
    );
    jp
}
pub unsafe fn NorfairLavaMan_Instr_15(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let E: *mut Enemy_NorfairLavaMan =
        Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).base.x_pos = ((*E).nlmn_var_12).wrapping_add(8u16);
    (*E).base.y_pos = ((*E).nlmn_var_13).wrapping_sub(4u16);
    jp
}
pub unsafe fn NorfairLavaMan_Instr_4(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let E: *mut Enemy_NorfairLavaMan =
        Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).base.x_pos = ((*E).nlmn_var_12).wrapping_sub(8u16);
    (*E).base.y_pos = ((*E).nlmn_var_13).wrapping_sub(4u16);
    jp
}
pub unsafe fn NorfairLavaMan_Instr_16(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let E: *mut Enemy_NorfairLavaMan =
        Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).base.x_pos = ((*E).nlmn_var_12).wrapping_add(8u16);
    (*E).base.y_pos = ((*E).nlmn_var_13).wrapping_sub(8u16);
    jp
}
pub unsafe fn NorfairLavaMan_Instr_6(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let E: *mut Enemy_NorfairLavaMan =
        Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).base.x_pos = ((*E).nlmn_var_12).wrapping_sub(8u16);
    (*E).base.y_pos = ((*E).nlmn_var_13).wrapping_sub(4u16);
    jp
}
pub unsafe fn sub_A8AF32(game: &mut Game) {
    let E: *mut Enemy_NorfairLavaMan =
        Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).base.x_pos = (*E).nlmn_var_12;
    (*E).base.y_pos = (*E).nlmn_var_13;
}
pub unsafe fn NorfairLavaMan_Instr_3(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game))
        .nlmn_var_04 = 256 as uint16;
    jp
}
static g_off_A8AF4F: [Option<Func_V>; 3] = unsafe {
    [
        Some(NorfairLavaMan_Func_1 as unsafe fn(&mut Game) -> ()),
        Some(NorfairLavaMan_Func_2 as unsafe fn(&mut Game) -> ()),
        Some(NorfairLavaMan_Func_3 as unsafe fn(&mut Game) -> ()),
    ]
};
pub unsafe fn NorfairLavaMan_Init(game: &mut Game) {
    let v0: uint16 = (2u16).wrapping_mul(
        (*Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game))
            .nlmn_parameter_1,
    );
    g_off_A8AF4F[(v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize]
        .expect("non-null function pointer")(game);
    NorfairLavaMan_Func_4(game);
    NorfairLavaMan_Func_5(ram_word!(game, cur_enemy_index), game);
}
pub unsafe fn NorfairLavaMan_Func_1(game: &mut Game) {
    let E: *mut Enemy_NorfairLavaMan =
        Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).nlmn_var_C = 0 as uint16;
    (*E).nlmn_var_00 = 0 as uint16;
    (*E).nlmn_var_02 = 0 as uint16;
    (*E).nlmn_var_03 = (*E).base.y_pos;
    if (GetSamusEnemyDelta_X(ram_word!(game, cur_enemy_index), game) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int)
        != 0i32
    {
        (*E).nlmn_var_00 = 1 as uint16;
    }
    (*E).nlmn_var_D = addr_kNorfairLavaMan_Ilist_AC9C as uint16;
    if (*E).nlmn_var_00 == 0 {
        (*E).nlmn_var_D = addr_kNorfairLavaMan_Ilist_AD3C as uint16;
    }
    NorfairLavaMan_Func_20(game);
    (*E).nlmn_var_F = ((0xa8b11ai32) as u16);
}
pub unsafe fn NorfairLavaMan_Func_2(game: &mut Game) {
    let E: *mut Enemy_NorfairLavaMan =
        Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).nlmn_var_03 = (*E).base.y_pos;
    (*E).nlmn_var_C = 0 as uint16;
    (*E).nlmn_var_0C = 0 as uint16;
    (*E).nlmn_var_0D = 1 as uint16;
    (*E).nlmn_var_D = addr_kNorfairLavaMan_Ilist_ADDC as uint16;
    NorfairLavaMan_Func_20(game);
    (*E).base.y_pos = ((*E).base.y_pos).wrapping_add(32u16);
    (*E).nlmn_var_F = ((0xa8b1ddi32) as u16);
    (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Invisible) as u16);
}
pub unsafe fn NorfairLavaMan_Func_3(game: &mut Game) {
    let E: *mut Enemy_NorfairLavaMan =
        Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).nlmn_var_03 = (*E).base.y_pos;
    (*E).nlmn_var_11 = (*E).base.x_pos;
    (*E).nlmn_var_C = 0 as uint16;
    (*E).nlmn_var_04 = 0 as uint16;
    (*E).nlmn_var_D = addr_kNorfairLavaMan_Ilist_AE0C as uint16;
    NorfairLavaMan_Func_20(game);
    (*E).base.y_pos = ((*E).base.y_pos).wrapping_add(32u16);
    (*E).nlmn_var_F = ((0xa8b30di32) as u16);
    (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Invisible) as u16);
}
pub unsafe fn NorfairLavaMan_Func_4(game: &mut Game) {
    let E: *mut Enemy_NorfairLavaMan =
        Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let v1: ::core::ffi::c_int = 8 as ::core::ffi::c_int
        * *(&raw mut (*E).nlmn_parameter_2 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1_usize) as isize,
        ) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    (*E).nlmn_var_08 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8
        as *mut uint16)
        .offset(v1 as isize);
    (*E).nlmn_var_07 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8
        as *mut uint16)
        .offset((v1 + 1 as ::core::ffi::c_int) as isize);
    (*E).nlmn_var_0A = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8
        as *mut uint16)
        .offset((v1 + 2 as ::core::ffi::c_int) as isize);
    (*E).nlmn_var_09 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8
        as *mut uint16)
        .offset((v1 + 3 as ::core::ffi::c_int) as isize);
}
pub unsafe fn NorfairLavaMan_Func_5(k: uint16, game: &mut Game) {
    (*((&raw mut game.g_ram as *mut uint8).offset(0x178c) as *mut LongPtr)).addr =
        ((0xa8b0b2i32) as u16);
    *(&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x178c) as *mut LongPtr)).bank
        as *mut uint16) = 168 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1794) as *mut uint16).offset(0) =
        (((((*Get_NorfairLavaMan(k as ::core::ffi::c_int, game))
            .base
            .palette_index)
            & (0xe00u16)) as ::core::ffi::c_int
            >> 4 as ::core::ffi::c_int)
            + 256 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1794) as *mut uint16).offset(2) = 8 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1794) as *mut uint16).offset(1) = 0 as uint16;
}
pub unsafe fn NorfairLavaMan_Func_6(game: &mut Game) {
    if ram_word!(game, door_transition_flag_enemies) == 0 && {
        let ref mut fresh10 =
            *((&raw mut game.g_ram as *mut uint8).offset(0x1794) as *mut uint16).offset(2);
        *fresh10 = (*fresh10).wrapping_sub(1);
        *fresh10 == 0
    } {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1794) as *mut uint16).offset(2) =
            8 as uint16;
        let ref mut fresh11 =
            *((&raw mut game.g_ram as *mut uint8).offset(0x1794) as *mut uint16).offset(1);
        *fresh11 = (*fresh11).wrapping_add(1);
        let v0: ::core::ffi::c_int = (32 as ::core::ffi::c_int
            * (*((&raw mut game.g_ram as *mut uint8).offset(0x1794) as *mut uint16).offset(1)
                as ::core::ffi::c_int
                & 3 as ::core::ffi::c_int)) as uint16
            as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int;
        let r18: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa8ac1c)) as *const uint8
            as *mut uint16)
            .offset((v0 + 9 as ::core::ffi::c_int) as isize);
        let r20: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa8ac1c)) as *const uint8
            as *mut uint16)
            .offset((v0 + 10 as ::core::ffi::c_int) as isize);
        let r22: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa8ac1c)) as *const uint8
            as *mut uint16)
            .offset((v0 + 11 as ::core::ffi::c_int) as isize);
        let r24: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa8ac1c)) as *const uint8
            as *mut uint16)
            .offset((v0 + 12 as ::core::ffi::c_int) as isize);
        let v1: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x1794)
            as *mut uint16)
            .offset(0) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
            .offset((v1 + 9 as ::core::ffi::c_int) as isize) = r18;
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
            .offset((v1 + 10 as ::core::ffi::c_int) as isize) = r20;
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
            .offset((v1 + 11 as ::core::ffi::c_int) as isize) = r22;
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
            .offset((v1 + 12 as ::core::ffi::c_int) as isize) = r24;
    }
}
pub unsafe fn NorfairLavaMan_Main(game: &mut Game) {
    let E: *mut Enemy_NorfairLavaMan =
        Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).nlmn_var_04 = (*E).nlmn_var_04.wrapping_sub(1);
    CallEnemyPreInstr((((*E).nlmn_var_F) as u32) | (0xa80000u32), game);
}
pub unsafe fn NorfairLavaMan_Func_7(k: uint16, game: &mut Game) {
    let E: *mut Enemy_NorfairLavaMan =
        Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).nlmn_var_00 = 0 as uint16;
    (*E).nlmn_var_D = addr_kNorfairLavaMan_Ilist_AC9C as uint16;
    if (GetSamusEnemyDelta_X(ram_word!(game, cur_enemy_index), game) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int)
        == 0i32
    {
        (*E).nlmn_var_D = addr_kNorfairLavaMan_Ilist_AD3C as uint16;
        (*E).nlmn_var_00 = 1 as uint16;
    }
    NorfairLavaMan_Func_20(game);
    let E2: *mut Enemy_NorfairLavaMan = Get_NorfairLavaMan(
        ram_word!(game, cur_enemy_index) as ::core::ffi::c_int + 128 as ::core::ffi::c_int,
        game,
    );
    if ((*E2).nlmn_var_04 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        (*E2).nlmn_var_04 = 0 as uint16;
        if IsSamusWithinEnemy_X(
            ram_word!(game, cur_enemy_index),
            *(&raw mut (*E).nlmn_parameter_2 as *mut uint8).offset(0) as uint16,
            game,
        ) != 0
        {
            (*E).nlmn_var_D = addr_kNorfairLavaMan_Ilist_ACDE as uint16;
            if (*E).nlmn_var_00 != 0 {
                (*E).nlmn_var_D = addr_kNorfairLavaMan_Ilist_AD7E as uint16;
            }
            NorfairLavaMan_Func_20(game);
            (*E).nlmn_var_F = ((0xa8b175i32) as u16);
        }
    }
}
pub unsafe fn NorfairLavaMan_Func_8(k: uint16, game: &mut Game) {
    let E: *mut Enemy_NorfairLavaMan =
        Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    if (*E).nlmn_var_01 == 0 {
        (*E).nlmn_var_02 = 1 as uint16;
        (*Get_NorfairLavaMan(
            ram_word!(game, cur_enemy_index) as ::core::ffi::c_int + 64 as ::core::ffi::c_int,
            game,
        ))
        .nlmn_var_0D = 0 as uint16;
        (*E).nlmn_var_F = ((0xa8b193i32) as u16);
    }
}
pub unsafe fn sub_A8B193(k: uint16, game: &mut Game) {
    let E: *mut Enemy_NorfairLavaMan =
        Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    if (*E).nlmn_var_02 == 0 {
        (*E).nlmn_var_D = addr_kNorfairLavaMan_Ilist_AD0C as uint16;
        if (GetSamusEnemyDelta_X(ram_word!(game, cur_enemy_index), game) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int)
            == 0i32
        {
            (*E).nlmn_var_D = addr_kNorfairLavaMan_Ilist_ADAC as uint16;
        }
        NorfairLavaMan_Func_20(game);
        (*E).nlmn_var_F = ((0xa8b1b8i32) as u16);
    }
}
pub unsafe fn NorfairLavaMan_Func_9(k: uint16, game: &mut Game) {
    let E: *mut Enemy_NorfairLavaMan =
        Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    if (*E).nlmn_var_01 == 0 {
        (*E).nlmn_var_D = addr_kNorfairLavaMan_Ilist_AC9C as uint16;
        if (GetSamusEnemyDelta_X(ram_word!(game, cur_enemy_index), game) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int)
            == 0i32
        {
            (*E).nlmn_var_D = addr_kNorfairLavaMan_Ilist_AD3C as uint16;
        }
        NorfairLavaMan_Func_20(game);
        (*E).nlmn_var_F = ((0xa8b11ai32) as u16);
    }
}
pub unsafe fn NorfairLavaMan_Func_10(k: uint16, game: &mut Game) {
    let E: *mut Enemy_NorfairLavaMan =
        Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    if (*E).nlmn_var_0D == 0 {
        (*E).nlmn_var_F = ((0xa8b204i32) as u16);
        (*E).nlmn_var_E = 0 as uint16;
        (*E).nlmn_var_0D = 0 as uint16;
        (*E).nlmn_var_B = 2 as uint16;
        (*E).base.y_pos = (*E).nlmn_var_03;
    }
}
pub unsafe fn NorfairLavaMan_Func_11(k: uint16, game: &mut Game) {
    let E: *mut Enemy_NorfairLavaMan =
        Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let v3: *mut EnemySpawnData =
        gEnemySpawnData(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let v: uint32 = (((*v3.offset(31)).ypos2) as u32).wrapping_add(((*E).base.y_subpos) as u32);
    (*E).base.y_subpos = v as uint16;
    (*E).base.y_pos = ((*E).base.y_pos as uint32).wrapping_add(
        ((*v3.offset(31)).field_14 as uint32).wrapping_add(v >> 16 as ::core::ffi::c_int),
    ) as uint16;
    (*E).nlmn_var_E = ((*E).nlmn_var_E as uint32).wrapping_add(
        ((*v3.offset(31)).field_14 as uint32).wrapping_add(v >> 16 as ::core::ffi::c_int),
    ) as uint16;
    NorfairLavaMan_Func_12(ram_word!(game, cur_enemy_index), game);
}
pub unsafe fn NorfairLavaMan_Func_12(k: uint16, game: &mut Game) {
    let current_block: u64;
    let mut v3: ::core::ffi::c_int = 0;
    let E: *mut Enemy_NorfairLavaMan = Get_NorfairLavaMan(k as ::core::ffi::c_int, game);
    if -((*E).nlmn_var_E as ::core::ffi::c_int) - 108 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        printf(
            b"Y undefined %d!\n\0" as *const u8 as *const ::core::ffi::c_char,
            -((*E).nlmn_var_E as int16 as ::core::ffi::c_int),
        );
    }
    let nlmn_var_B: uint16 = (*E).nlmn_var_B;
    if -((*E).nlmn_var_E as ::core::ffi::c_int) - 108 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
        || {
            v3 = nlmn_var_B as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            (((*E).base.y_pos as ::core::ffi::c_int
                - *(game.g_rom.offset(crate::rtl::lorom_offset(0xa8af79)) as *const uint8
                    as *mut uint16)
                    .offset(v3 as isize) as ::core::ffi::c_int
                - ram_word!(game, samus_y_pos) as ::core::ffi::c_int) as int16)
                < 0i16
        }
    {
        (*E).nlmn_var_F = ((0xa8b291i32) as u16);
        (*E).nlmn_var_0C = 1 as uint16;
        if ((-((*E).nlmn_var_E as ::core::ffi::c_int)
            - *(game.g_rom.offset(crate::rtl::lorom_offset(0xa8af55)) as *const uint8
                as *mut uint16)
                .offset((nlmn_var_B as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int) as int16)
            < 0i16
        {
            return;
        }
        current_block = 9609259077933178539;
    } else if ((-((*E).nlmn_var_E as ::core::ffi::c_int)
        - *(game.g_rom.offset(crate::rtl::lorom_offset(0xa8af55)) as *const uint8 as *mut uint16)
            .offset(v3 as isize) as ::core::ffi::c_int) as int16)
        >= 0i16
    {
        current_block = 9609259077933178539;
    } else {
        current_block = 7746791466490516765;
    }
    if current_block == 9609259077933178539 {
        (*E).nlmn_var_B = (*E).nlmn_var_B.wrapping_add(1);
        (*E).nlmn_var_B = (*E).nlmn_var_B.wrapping_add(1);
        (*E).base.y_pos = ((*E).base.y_pos).wrapping_add(8u16);
        (*E).nlmn_var_D = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa8af67)) as *const uint8
            as *mut uint16)
            .offset(((*E).nlmn_var_B as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        NorfairLavaMan_Func_20(game);
    };
}
pub fn sub_A8B291(k: uint16) {}
pub unsafe fn NorfairLavaMan_Func_13(game: &mut Game) {
    let mut v2: *mut EnemySpawnData = ::core::ptr::null_mut::<EnemySpawnData>();
    let E: *mut Enemy_NorfairLavaMan =
        Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    v2 = gEnemySpawnData(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let v4: bool = u16::overflowing_add((*v2.offset(31)).field_E, (*E).base.y_subpos).1;
    (*E).base.y_subpos = ((*v2.offset(31)).field_E).wrapping_add((*E).base.y_subpos);
    (*E).base.y_pos = ((*E).base.y_pos as ::core::ffi::c_int
        + ((*v2.offset(31)).xpos2 as ::core::ffi::c_int + v4 as ::core::ffi::c_int))
        as uint16;
    (*E).nlmn_var_E = ((*E).nlmn_var_E as ::core::ffi::c_int
        + ((*v2.offset(31)).xpos2 as ::core::ffi::c_int + v4 as ::core::ffi::c_int))
        as uint16;
    NorfairLavaMan_Func_14(ram_word!(game, cur_enemy_index), game);
}
pub unsafe fn NorfairLavaMan_Func_14(k: uint16, game: &mut Game) {
    let mut nlmn_var_E: int16 = 0;
    let E: *mut Enemy_NorfairLavaMan = Get_NorfairLavaMan(k as ::core::ffi::c_int, game);
    nlmn_var_E = (*E).nlmn_var_E as int16;
    if nlmn_var_E >= 0i16 {
        (*E).nlmn_var_0D = 1 as uint16;
        (*gEnemySpawnData(k as ::core::ffi::c_int, game).offset(31)).field_4 = 0 as uint16;
        (*E).nlmn_var_F = ((0xa8b1ddi32) as u16);
    } else if ((-(nlmn_var_E as ::core::ffi::c_int)
        - *(game.g_rom.offset(crate::rtl::lorom_offset(0xa8af55)) as *const uint8 as *mut uint16)
            .offset(
                ((((*E).nlmn_var_B).wrapping_sub(2u16)) as ::core::ffi::c_int
                    >> 1 as ::core::ffi::c_int) as isize,
            ) as ::core::ffi::c_int) as int16)
        < 0i16
    {
        (*E).nlmn_var_B = ((*E).nlmn_var_B).wrapping_sub(2u16);
        (*E).base.y_pos = ((*E).base.y_pos).wrapping_sub(8u16);
        (*E).nlmn_var_D = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa8af67)) as *const uint8
            as *mut uint16)
            .offset(((*E).nlmn_var_B as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        NorfairLavaMan_Func_20(game);
    }
}
pub unsafe fn NorfairLavaMan_Func_15(k: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16).offset(
        ((ram_word!(game, cur_enemy_index) as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
            + 87 as ::core::ffi::c_int) as isize,
    ) as ::core::ffi::c_int
        == 0xa8b175 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
    {
        (*Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game))
            .nlmn_var_F = ((0xa8b356i32) as u16);
    }
}
pub unsafe fn NorfairLavaMan_Func_16(k: uint16, game: &mut Game) {
    let E: *mut Enemy_NorfairLavaMan =
        Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    if (*E).nlmn_var_01 == 0 {
        (*E).nlmn_var_D = addr_kNorfairLavaMan_Ilist_AE0C as uint16;
        NorfairLavaMan_Func_20(game);
        *((&raw mut game.g_ram as *mut uint8).offset(0xf68) as *mut uint16).offset(
            ((ram_word!(game, cur_enemy_index) as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
                + 5 as ::core::ffi::c_int) as isize,
        ) = ((0xa8b295i32) as u16);
        (*E).nlmn_var_F = ((0xa8b3a7i32) as u16);
        (*gEnemySpawnData(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game)
            .offset(31))
        .field_18 = 0 as uint16;
        (*E).base.x_pos = (*E).nlmn_var_12;
        (*E).base.y_pos = (*E).nlmn_var_13;
    }
    NorfairLavaMan_Func_19(ram_word!(game, cur_enemy_index), game);
}
pub unsafe fn NorfairLavaMan_Func_17(k: uint16, game: &mut Game) {
    if (*gEnemySpawnData(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game).offset(31))
        .field_18
        != 0
    {
        let E: *mut Enemy_NorfairLavaMan =
            Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
        (*E).nlmn_var_D = addr_kNorfairLavaMan_Ilist_ACB0 as uint16;
        (*E).nlmn_var_B = 0 as uint16;
        if (GetSamusEnemyDelta_X(ram_word!(game, cur_enemy_index), game) as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int)
            == 0i32
        {
            (*E).nlmn_var_D = addr_kNorfairLavaMan_Ilist_AD50 as uint16;
            (*E).nlmn_var_B = 1 as uint16;
        }
        NorfairLavaMan_Func_20(game);
        (*E).nlmn_var_F = ((0xa8b31fi32) as u16);
        (*E).nlmn_var_12 = (*E).base.x_pos;
        (*E).nlmn_var_13 = (*E).base.y_pos;
    } else {
        (*Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game))
            .base
            .y_pos = (*((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16).offset(
            ((ram_word!(game, cur_enemy_index) as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
                + 93 as ::core::ffi::c_int) as isize,
        ))
        .wrapping_sub(
            *(game.g_rom.offset(crate::rtl::lorom_offset(0xa8af79)) as *const uint8 as *mut uint16)
                .offset(
                    (*((&raw mut game.g_ram as *mut uint8).offset(0xf68) as *mut uint16).offset(
                        ((ram_word!(game, cur_enemy_index) as ::core::ffi::c_int
                            >> 1 as ::core::ffi::c_int)
                            + 1 as ::core::ffi::c_int) as isize,
                    ) as ::core::ffi::c_int
                        >> 1 as ::core::ffi::c_int) as isize,
                ),
        );
    }
    NorfairLavaMan_Func_19(ram_word!(game, cur_enemy_index), game);
}
pub unsafe fn NorfairLavaMan_Func_18(k: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int =
        ram_word!(game, cur_enemy_index) as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
        .offset((v1 + 87 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
        == 0xa8b11a as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
    {
        (*Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game))
            .nlmn_var_F = ((0xa8b30di32) as u16);
    } else {
        (*Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game))
            .base
            .y_pos = (*((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
            .offset((v1 + 93 as ::core::ffi::c_int) as isize))
        .wrapping_sub(
            *(game.g_rom.offset(crate::rtl::lorom_offset(0xa8af79)) as *const uint8 as *mut uint16)
                .offset(
                    (*((&raw mut game.g_ram as *mut uint8).offset(0xf68) as *mut uint16)
                        .offset((v1 + 1 as ::core::ffi::c_int) as isize)
                        as ::core::ffi::c_int
                        >> 1 as ::core::ffi::c_int) as isize,
                ),
        );
    }
    NorfairLavaMan_Func_19(ram_word!(game, cur_enemy_index), game);
}
pub unsafe fn NorfairLavaMan_Func_19(k: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let v2: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
        .offset((v1 + 61 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        - (*Get_NorfairLavaMan(k as ::core::ffi::c_int, game))
            .base
            .y_pos as ::core::ffi::c_int
        + 2 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
        .offset((v1 + 64 as ::core::ffi::c_int) as isize) = v2;
    if v2 as ::core::ffi::c_int - 8 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
            .offset((v1 + 64 as ::core::ffi::c_int) as isize) = 8 as uint16;
    }
}
pub unsafe fn NorfairLavaMan_Func_20(game: &mut Game) {
    let E: *mut Enemy_NorfairLavaMan =
        Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let nlmn_var_D: uint16 = (*E).nlmn_var_D;
    if nlmn_var_D != (*E).nlmn_var_C {
        (*E).base.current_instruction = nlmn_var_D;
        (*E).nlmn_var_C = nlmn_var_D;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
    }
}
pub unsafe fn NorfairLavaMan_Powerbomb(game: &mut Game) {
    Enemy_NormalPowerBombAI_A8(game);
    NorfairLavaMan_Common(game);
}
pub unsafe fn NorfairLavaMan_Touch(game: &mut Game) {
    Enemy_NormalTouchAI_A8(game);
    NorfairLavaMan_Common(game);
}
pub unsafe fn NorfairLavaMan_Shot(game: &mut Game) {
    NormalEnemyShotAi(game);
    NorfairLavaMan_Common(game);
}
pub unsafe fn NorfairLavaMan_Common(game: &mut Game) {
    let E: *mut Enemy_NorfairLavaMan =
        Get_NorfairLavaMan(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let E1: *mut Enemy_NorfairLavaMan = Get_NorfairLavaMan(
        ram_word!(game, cur_enemy_index) as ::core::ffi::c_int + 64 as ::core::ffi::c_int,
        game,
    );
    let E2: *mut Enemy_NorfairLavaMan = Get_NorfairLavaMan(
        ram_word!(game, cur_enemy_index) as ::core::ffi::c_int + 128 as ::core::ffi::c_int,
        game,
    );
    if (*E).base.health == 0 {
        (*E1).base.properties = ((*E1).base.properties) | (0x200u16);
        (*E2).base.properties = ((*E2).base.properties) | (0x200u16);
    }
    let frozen_timer: uint16 = (*E).base.frozen_timer;
    if frozen_timer != 0 {
        (*E1).base.frozen_timer = frozen_timer;
        (*E1).base.ai_handler_bits = ((*E1).base.ai_handler_bits) | (4u16);
        (*E2).base.frozen_timer = frozen_timer;
        (*E2).base.ai_handler_bits = ((*E2).base.ai_handler_bits) | (4u16);
    }
}
pub fn Beetom_Instr_1(k: uint16, jp: *const uint16) -> *const uint16 {
    jp
}
pub unsafe fn Beetom_Func_1(game: &mut Game) {
    let E: *mut Enemy_Beetom =
        Get_Beetom(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).base.current_instruction = (*E).beetom_var_00;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
}
pub unsafe fn Beetom_Init(game: &mut Game) {
    let E: *mut Enemy_Beetom =
        Get_Beetom(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).beetom_var_D = 0 as uint16;
    (*E).beetom_var_C = 0 as uint16;
    (*E).beetom_var_05 = 0 as uint16;
    (*E).beetom_var_08 = 0 as uint16;
    (*E).beetom_var_A = 0 as uint16;
    (*E).beetom_var_E = 64 as uint16;
    (*E).beetom_var_F = ram_word!(game, joypad1_lastkeys);
    *((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16) = 23 as uint16;
    (*E).beetom_var_02 = Beetom_Func_2(12288 as uint16, 4 as uint16, game);
    (*E).beetom_var_03 = Beetom_Func_2(0x4000 as uint16, 5 as uint16, game);
    (*E).beetom_var_04 = Beetom_Func_2(12288 as uint16, 3 as uint16, game);
    (*E).beetom_var_00 = addr_kBeetom_Ilist_B6F2 as uint16;
    if (GetSamusEnemyDelta_X(ram_word!(game, cur_enemy_index), game) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int)
        != 0i32
    {
        (*E).beetom_var_00 = addr_kBeetom_Ilist_B696 as uint16;
    }
    Beetom_Func_1(game);
    (*E).beetom_var_C = ((0xa8b814i32) as u16);
}
pub unsafe fn Beetom_Func_2(r22: uint16, r24: uint16, game: &mut Game) -> uint16 {
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
pub unsafe fn Beetom_Func_3(game: &mut Game) {
    if IsSamusWithinEnemy_X(ram_word!(game, cur_enemy_index), 0x60 as uint16, game) != 0 {
        (*Get_Beetom(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game)).beetom_var_C =
            ((0xa8b90fi32) as u16);
    } else {
        (*Get_Beetom(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game)).beetom_var_C =
            ((0xa8b82fi32) as u16);
    };
}
pub unsafe fn Beetom_Func_4(game: &mut Game) {
    NextRandom(game);
    let E: *mut Enemy_Beetom =
        Get_Beetom(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).beetom_var_C = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa8b74e)) as *const uint8
        as *mut uint16)
        .offset(
            (ram_word!(game, random_number) as ::core::ffi::c_int & 7 as ::core::ffi::c_int)
                as isize,
        );
    (*E).beetom_var_09 = (ram_word!(game, random_number)) & (1u16);
}
pub unsafe fn Beetom_Func_5(game: &mut Game) {
    let E: *mut Enemy_Beetom =
        Get_Beetom(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).beetom_var_D = 32 as uint16;
    (*E).beetom_var_C = ((0xa8b9b2i32) as u16);
}
pub unsafe fn Beetom_Func_6(game: &mut Game) {
    let E: *mut Enemy_Beetom =
        Get_Beetom(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).beetom_var_C = ((0xa8b9c1i32) as u16);
    (*E).beetom_var_00 = addr_kBeetom_Ilist_B696 as uint16;
    Beetom_Func_1(game);
}
pub unsafe fn Beetom_Func_7(game: &mut Game) {
    let E: *mut Enemy_Beetom =
        Get_Beetom(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).beetom_var_C = ((0xa8ba24i32) as u16);
    (*E).beetom_var_00 = addr_kBeetom_Ilist_B6F2 as uint16;
    Beetom_Func_1(game);
}
pub unsafe fn Beetom_Func_8(game: &mut Game) {
    let E: *mut Enemy_Beetom =
        Get_Beetom(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).beetom_var_B = (*E).beetom_var_02;
    (*E).beetom_var_C = ((0xa8ba84i32) as u16);
    (*E).beetom_var_05 = 0 as uint16;
    (*E).beetom_var_00 = addr_kBeetom_Ilist_B6AC as uint16;
    Beetom_Func_1(game);
}
pub unsafe fn Beetom_Func_9(game: &mut Game) {
    let E: *mut Enemy_Beetom =
        Get_Beetom(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).beetom_var_B = (*E).beetom_var_02;
    (*E).beetom_var_C = ((0xa8bab7i32) as u16);
    (*E).beetom_var_05 = 0 as uint16;
    (*E).beetom_var_00 = addr_kBeetom_Ilist_B708 as uint16;
    Beetom_Func_1(game);
}
pub unsafe fn Beetom_Func_10(game: &mut Game) {
    let E: *mut Enemy_Beetom =
        Get_Beetom(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).beetom_var_B = (*E).beetom_var_03;
    (*E).beetom_var_C = ((0xa8bb55i32) as u16);
    (*E).beetom_var_05 = 0 as uint16;
    (*E).beetom_var_00 = addr_kBeetom_Ilist_B6AC as uint16;
    Beetom_Func_1(game);
}
pub unsafe fn Beetom_Func_11(game: &mut Game) {
    let E: *mut Enemy_Beetom =
        Get_Beetom(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).beetom_var_B = (*E).beetom_var_03;
    (*E).beetom_var_C = ((0xa8bb88i32) as u16);
    (*E).beetom_var_05 = 0 as uint16;
    (*E).beetom_var_00 = addr_kBeetom_Ilist_B708 as uint16;
    Beetom_Func_1(game);
}
pub unsafe fn Beetom_Func_12(game: &mut Game) {
    let E: *mut Enemy_Beetom =
        Get_Beetom(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).beetom_var_B = (*E).beetom_var_04;
    (*E).beetom_var_00 = addr_kBeetom_Ilist_B708 as uint16;
    (*E).beetom_var_C = ((0xa8bc5ai32) as u16);
    (*E).beetom_var_09 = 1 as uint16;
    if (GetSamusEnemyDelta_X(ram_word!(game, cur_enemy_index), game) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int)
        != 0i32
    {
        (*E).beetom_var_00 = addr_kBeetom_Ilist_B6AC as uint16;
        (*E).beetom_var_C = ((0xa8bc26i32) as u16);
        (*E).beetom_var_09 = 0 as uint16;
    }
    Beetom_Func_1(game);
    (*E).beetom_var_05 = 0 as uint16;
}
pub unsafe fn Beetom_Func_13(game: &mut Game) {
    let E: *mut Enemy_Beetom =
        Get_Beetom(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).beetom_var_00 = addr_kBeetom_Ilist_B6CC as uint16;
    Beetom_Func_1(game);
    (*E).beetom_var_C = ((0xa8bcf8i32) as u16);
}
pub unsafe fn Beetom_Func_14(game: &mut Game) {
    let E: *mut Enemy_Beetom =
        Get_Beetom(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).beetom_var_00 = addr_kBeetom_Ilist_B728 as uint16;
    Beetom_Func_1(game);
    (*E).beetom_var_C = ((0xa8bd42i32) as u16);
}
pub unsafe fn Beetom_Func_15(game: &mut Game) {
    let E: *mut Enemy_Beetom =
        Get_Beetom(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).beetom_var_00 = addr_kBeetom_Ilist_B696 as uint16;
    if (*E).beetom_var_09 != 0 {
        (*E).beetom_var_00 = addr_kBeetom_Ilist_B6F2 as uint16;
    }
    Beetom_Func_1(game);
    (*E).beetom_var_05 = 0 as uint16;
    (*E).beetom_var_C = ((0xa8bd9di32) as u16);
}
pub unsafe fn Beetom_Func_16(game: &mut Game) {
    let E: *mut Enemy_Beetom =
        Get_Beetom(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).beetom_var_B = 0 as uint16;
    (*E).beetom_var_C = ((0xa8bdc5i32) as u16);
}
pub unsafe fn Beetom_Func_17(game: &mut Game) {
    let E: *mut Enemy_Beetom =
        Get_Beetom(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).beetom_var_D = (*E).beetom_var_D.wrapping_sub(1);
    if ((*E).beetom_var_D as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        (*E).beetom_var_C = ((0xa8b814i32) as u16);
    }
}
pub unsafe fn Beetom_Func_18(game: &mut Game) {
    let E: *mut Enemy_Beetom =
        Get_Beetom(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).beetom_var_D = (*E).beetom_var_D.wrapping_sub(1);
    if ((*E).beetom_var_D as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        (*E).beetom_var_D = 64 as uint16;
        (*E).beetom_var_C = ((0xa8b814i32) as u16);
    } else {
        (*E).base.x_pos = ((*E).base.x_pos).wrapping_sub(8u16);
        if Enemy_MoveDown(
            ram_word!(game, cur_enemy_index),
            (1 as int16 as int32) << 16 as ::core::ffi::c_int,
            game,
        ) != 0
        {
            (*E).base.x_pos = ((*E).base.x_pos).wrapping_add(8u16);
            if Enemy_MoveRight_IgnoreSlopes(
                ram_word!(game, cur_enemy_index),
                -(16384 as int32),
                game,
            ) != 0
            {
                (*E).beetom_var_C = ((0xa8b873i32) as u16);
            }
        } else {
            (*E).beetom_var_C = ((0xa8b873i32) as u16);
            (*E).base.y_pos = (*E).base.y_pos.wrapping_sub(1);
            (*E).base.x_pos = ((*E).base.x_pos).wrapping_add(8u16);
        }
    };
}
pub unsafe fn Beetom_Func_19(game: &mut Game) {
    let E: *mut Enemy_Beetom =
        Get_Beetom(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).beetom_var_D = (*E).beetom_var_D.wrapping_sub(1);
    if ((*E).beetom_var_D as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        (*E).beetom_var_D = 64 as uint16;
        (*E).beetom_var_C = ((0xa8b814i32) as u16);
    } else {
        (*E).base.x_pos = ((*E).base.x_pos).wrapping_add(8u16);
        if Enemy_MoveDown(
            ram_word!(game, cur_enemy_index),
            (1 as int16 as int32) << 16 as ::core::ffi::c_int,
            game,
        ) != 0
        {
            (*E).base.x_pos = ((*E).base.x_pos).wrapping_sub(8u16);
            if Enemy_MoveRight_IgnoreSlopes(ram_word!(game, cur_enemy_index), 0x4000 as int32, game)
                != 0
            {
                (*E).beetom_var_C = ((0xa8b85fi32) as u16);
            }
        } else {
            (*E).beetom_var_C = ((0xa8b85fi32) as u16);
            (*E).base.y_pos = (*E).base.y_pos.wrapping_sub(1);
            (*E).base.x_pos = ((*E).base.x_pos).wrapping_sub(8u16);
        }
    };
}
pub unsafe fn Beetom_Func_22(k: uint16, game: &mut Game) {
    let E: *mut Enemy_Beetom = Get_Beetom(k as ::core::ffi::c_int, game);
    let v2: ::core::ffi::c_int = 8 as ::core::ffi::c_int * (*E).beetom_var_B as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    if Enemy_MoveDown(
        k,
        *(game.g_rom.offset(crate::rtl::lorom_offset(0xa0cbc7)) as *const uint8 as *mut uint32)
            .offset((v2 + 2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as int32,
        game,
    ) != 0
    {
        (*E).beetom_var_C = ((0xa8b97ai32) as u16);
    } else {
        (*E).beetom_var_B = ((*E).beetom_var_B).wrapping_sub(4u16);
        if (*E).beetom_var_B as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
            (*E).beetom_var_B = 0 as uint16;
            (*E).beetom_var_05 = 1 as uint16;
        }
    };
}
pub unsafe fn Beetom_Func_23(k: uint16, game: &mut Game) {
    let E: *mut Enemy_Beetom = Get_Beetom(k as ::core::ffi::c_int, game);
    let v2: ::core::ffi::c_int = 8 as ::core::ffi::c_int * (*E).beetom_var_B as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    if Enemy_MoveDown(
        k,
        *(game.g_rom.offset(crate::rtl::lorom_offset(0xa0cbc7)) as *const uint8 as *mut uint32)
            .offset((v2 >> 1 as ::core::ffi::c_int) as isize) as int32,
        game,
    ) != 0
    {
        (*E).beetom_var_C = ((0xa8b814i32) as u16);
    } else {
        (*E).beetom_var_B = ((*E).beetom_var_B).wrapping_add(4u16);
        if (*E).beetom_var_B as ::core::ffi::c_int - 64 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            (*E).beetom_var_B = 64 as uint16;
        }
    };
}
pub unsafe fn Beetom_Func_20(game: &mut Game) {
    let E: *mut Enemy_Beetom =
        Get_Beetom(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    if (*E).beetom_var_05 != 0 {
        Beetom_Func_23(ram_word!(game, cur_enemy_index), game);
    } else {
        Beetom_Func_22(ram_word!(game, cur_enemy_index), game);
    }
    if Enemy_MoveRight_IgnoreSlopes(ram_word!(game, cur_enemy_index), -(16384 as int32), game) != 0
    {
        (*E).beetom_var_09 = ((*E).beetom_var_09) ^ (1u16);
        (*E).beetom_var_C = ((0xa8b97ai32) as u16);
    }
}
pub unsafe fn Beetom_Func_21(game: &mut Game) {
    let E: *mut Enemy_Beetom =
        Get_Beetom(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    if (*E).beetom_var_05 != 0 {
        Beetom_Func_23(ram_word!(game, cur_enemy_index), game);
    } else {
        Beetom_Func_22(ram_word!(game, cur_enemy_index), game);
    }
    if Enemy_MoveRight_IgnoreSlopes(ram_word!(game, cur_enemy_index), 0x4000 as int32, game) != 0 {
        (*E).beetom_var_09 = ((*E).beetom_var_09) ^ (1u16);
        (*E).beetom_var_C = ((0xa8b97ai32) as u16);
    }
}
pub unsafe fn Beetom_Func_26(k: uint16, game: &mut Game) {
    let E: *mut Enemy_Beetom = Get_Beetom(k as ::core::ffi::c_int, game);
    let v2: ::core::ffi::c_int = 8 as ::core::ffi::c_int * (*E).beetom_var_B as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    if Enemy_MoveDown(
        k,
        *(game.g_rom.offset(crate::rtl::lorom_offset(0xa0cbc7)) as *const uint8 as *mut uint32)
            .offset((v2 + 2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as int32,
        game,
    ) != 0
    {
        (*E).beetom_var_C = ((0xa8b97ai32) as u16);
    } else {
        (*E).beetom_var_B = ((*E).beetom_var_B).wrapping_sub(5u16);
        if (*E).beetom_var_B as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
            (*E).beetom_var_B = 0 as uint16;
            (*E).beetom_var_05 = 1 as uint16;
        }
    };
}
pub unsafe fn Beetom_Func_27(k: uint16, game: &mut Game) {
    let E: *mut Enemy_Beetom = Get_Beetom(k as ::core::ffi::c_int, game);
    let v2: ::core::ffi::c_int = 8 as ::core::ffi::c_int * (*E).beetom_var_B as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    if Enemy_MoveDown(
        k,
        *(game.g_rom.offset(crate::rtl::lorom_offset(0xa0cbc7)) as *const uint8 as *mut uint32)
            .offset((v2 >> 1 as ::core::ffi::c_int) as isize) as int32,
        game,
    ) != 0
    {
        (*E).beetom_var_C = ((0xa8b814i32) as u16);
    } else {
        (*E).beetom_var_B = ((*E).beetom_var_B).wrapping_add(5u16);
        if (*E).beetom_var_B as ::core::ffi::c_int - 64 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            (*E).beetom_var_B = 64 as uint16;
        }
    };
}
pub unsafe fn Beetom_Func_24(game: &mut Game) {
    let E: *mut Enemy_Beetom =
        Get_Beetom(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    if (*E).beetom_var_05 != 0 {
        Beetom_Func_27(ram_word!(game, cur_enemy_index), game);
    } else {
        Beetom_Func_26(ram_word!(game, cur_enemy_index), game);
    }
    if Enemy_MoveRight_IgnoreSlopes(ram_word!(game, cur_enemy_index), -(16384 as int32), game) != 0
    {
        (*E).beetom_var_09 = ((*E).beetom_var_09) ^ (1u16);
        (*E).beetom_var_C = ((0xa8b97ai32) as u16);
    }
}
pub unsafe fn Beetom_Func_25(game: &mut Game) {
    let E: *mut Enemy_Beetom =
        Get_Beetom(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    if (*E).beetom_var_05 != 0 {
        Beetom_Func_27(ram_word!(game, cur_enemy_index), game);
    } else {
        Beetom_Func_26(ram_word!(game, cur_enemy_index), game);
    }
    if Enemy_MoveRight_IgnoreSlopes(ram_word!(game, cur_enemy_index), 0x4000 as int32, game) != 0 {
        (*E).beetom_var_09 = ((*E).beetom_var_09) ^ (1u16);
        (*E).beetom_var_C = ((0xa8b97ai32) as u16);
    }
}
pub unsafe fn Beetom_Func_30(k: uint16, game: &mut Game) {
    let E: *mut Enemy_Beetom = Get_Beetom(k as ::core::ffi::c_int, game);
    let v2: ::core::ffi::c_int = 8 as ::core::ffi::c_int * (*E).beetom_var_B as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    if Enemy_MoveDown(
        k,
        *(game.g_rom.offset(crate::rtl::lorom_offset(0xa0cbc7)) as *const uint8 as *mut uint32)
            .offset((v2 + 2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as int32,
        game,
    ) != 0
    {
        (*E).beetom_var_C = ((0xa8b97ai32) as u16);
    } else {
        let v3: int16 = (((*E).beetom_var_B) as i16).wrapping_sub(5i16);
        (*E).beetom_var_B = v3 as uint16;
        if v3 < 0i16 {
            (*E).beetom_var_B = 0 as uint16;
            (*E).beetom_var_05 = 1 as uint16;
        }
    };
}
pub unsafe fn Beetom_Func_31(k: uint16, game: &mut Game) {
    let E: *mut Enemy_Beetom = Get_Beetom(k as ::core::ffi::c_int, game);
    let v2: ::core::ffi::c_int = 8 as ::core::ffi::c_int * (*E).beetom_var_B as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    if Enemy_MoveDown(
        k,
        *(game.g_rom.offset(crate::rtl::lorom_offset(0xa0cbc7)) as *const uint8 as *mut uint32)
            .offset((v2 >> 1 as ::core::ffi::c_int) as isize) as int32,
        game,
    ) != 0
    {
        (*E).beetom_var_C = ((0xa8b814i32) as u16);
    } else {
        let v3: uint16 = ((*E).beetom_var_B).wrapping_add(3u16);
        (*E).beetom_var_B = v3;
        if v3 as ::core::ffi::c_int - 64 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
            (*E).beetom_var_B = 64 as uint16;
        }
    };
}
pub unsafe fn Beetom_Func_28(game: &mut Game) {
    let E: *mut Enemy_Beetom =
        Get_Beetom(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    if (*E).beetom_var_05 != 0 {
        Beetom_Func_31(ram_word!(game, cur_enemy_index), game);
    } else {
        Beetom_Func_30(ram_word!(game, cur_enemy_index), game);
    }
    if Enemy_MoveRight_IgnoreSlopes(
        ram_word!(game, cur_enemy_index),
        ((-(3 as ::core::ffi::c_int) as uint32) << 16 as ::core::ffi::c_int | 0 as uint16 as uint32)
            as int32,
        game,
    ) != 0
    {
        (*E).beetom_var_09 = ((*E).beetom_var_09) ^ (1u16);
        (*E).beetom_var_C = ((0xa8b97ai32) as u16);
    }
}
pub unsafe fn Beetom_Func_29(game: &mut Game) {
    let E: *mut Enemy_Beetom =
        Get_Beetom(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    if (*E).beetom_var_05 != 0 {
        Beetom_Func_31(ram_word!(game, cur_enemy_index), game);
    } else {
        Beetom_Func_30(ram_word!(game, cur_enemy_index), game);
    }
    if Enemy_MoveRight_IgnoreSlopes(
        ram_word!(game, cur_enemy_index),
        ((3 as uint32) << 16 as ::core::ffi::c_int | 0 as uint16 as uint32) as int32,
        game,
    ) != 0
    {
        (*E).beetom_var_09 = ((*E).beetom_var_09) ^ (1u16);
        (*E).beetom_var_C = ((0xa8b97ai32) as u16);
    }
}
pub unsafe fn Beetom_Func_34(game: &mut Game) {
    let E: *mut Enemy_Beetom =
        Get_Beetom(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    if ram_word!(game, joypad1_lastkeys) != (*E).beetom_var_F {
        (*E).beetom_var_F = ram_word!(game, joypad1_lastkeys);
        (*E).beetom_var_E = (*E).beetom_var_E.wrapping_sub(1);
    }
}
pub unsafe fn Beetom_Func_32(game: &mut Game) {
    let E: *mut Enemy_Beetom =
        Get_Beetom(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    if (*E).beetom_var_E != 0 {
        (*E).base.x_pos = ram_word!(game, samus_x_pos);
        (*E).base.y_pos = (ram_word!(game, samus_y_pos)).wrapping_sub(4u16);
        Beetom_Func_34(game);
    } else {
        if Enemy_MoveRight_IgnoreSlopes(
            ram_word!(game, cur_enemy_index),
            ((16 as uint32) << 16 as ::core::ffi::c_int | 0 as uint16 as uint32) as int32,
            game,
        ) != 0
        {
            (*E).beetom_var_09 = 1 as uint16;
            Enemy_MoveRight_IgnoreSlopes(
                ram_word!(game, cur_enemy_index),
                ((-(32 as ::core::ffi::c_int) as uint32) << 16 as ::core::ffi::c_int
                    | 0 as uint16 as uint32) as int32,
                game,
            );
        }
        (*E).beetom_var_08 = 0 as uint16;
        (*E).beetom_var_C = ((0xa8b9a2i32) as u16);
    };
}
pub unsafe fn Beetom_Func_33(game: &mut Game) {
    let E: *mut Enemy_Beetom =
        Get_Beetom(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    if (*E).beetom_var_E != 0 {
        (*E).base.x_pos = ram_word!(game, samus_x_pos);
        (*E).base.y_pos = (ram_word!(game, samus_y_pos)).wrapping_sub(4u16);
        Beetom_Func_34(game);
    } else {
        if Enemy_MoveRight_IgnoreSlopes(
            ram_word!(game, cur_enemy_index),
            ((-(16 as ::core::ffi::c_int) as uint32) << 16 as ::core::ffi::c_int
                | 0 as uint16 as uint32) as int32,
            game,
        ) != 0
        {
            (*E).beetom_var_09 = 0 as uint16;
            Enemy_MoveRight_IgnoreSlopes(
                ram_word!(game, cur_enemy_index),
                ((32 as uint32) << 16 as ::core::ffi::c_int | 0 as uint16 as uint32) as int32,
                game,
            );
        }
        (*E).beetom_var_08 = 0 as uint16;
        (*E).beetom_var_C = ((0xa8b9a2i32) as u16);
    };
}
pub unsafe fn Beetom_Func_35(game: &mut Game) {
    if Enemy_MoveDown(
        ram_word!(game, cur_enemy_index),
        ((3 as uint32) << 16 as ::core::ffi::c_int | 0 as uint16 as uint32) as int32,
        game,
    ) != 0
    {
        let E: *mut Enemy_Beetom =
            Get_Beetom(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
        if (*E).beetom_var_09 != 0 {
            (*E).beetom_var_C = ((0xa8b873i32) as u16);
        } else {
            (*E).beetom_var_C = ((0xa8b85fi32) as u16);
        }
    }
}
pub unsafe fn Beetom_Func_37(k: uint16, game: &mut Game) {
    let E: *mut Enemy_Beetom = Get_Beetom(k as ::core::ffi::c_int, game);
    let v2: ::core::ffi::c_int = 8 as ::core::ffi::c_int * (*E).beetom_var_B as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    if Enemy_MoveDown(
        k,
        *(game.g_rom.offset(crate::rtl::lorom_offset(0xa0cbc7)) as *const uint8 as *mut uint32)
            .offset((v2 >> 1 as ::core::ffi::c_int) as isize) as int32,
        game,
    ) != 0
    {
        (*E).beetom_var_C = ((0xa8b814i32) as u16);
    } else {
        let v3: uint16 = ((*E).beetom_var_B).wrapping_add(1u16);
        (*E).beetom_var_B = v3;
        if v3 as ::core::ffi::c_int - 64 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
            (*E).beetom_var_B = 64 as uint16;
        }
    }
    let v4: uint16 = (if (*E).beetom_var_09 != 0u16 {
        -(2 as ::core::ffi::c_int)
    } else {
        2 as ::core::ffi::c_int
    }) as uint16;
    if Enemy_MoveRight_IgnoreSlopes(
        k,
        ((v4 as uint32) << 16 as ::core::ffi::c_int | 0 as uint16 as uint32) as int32,
        game,
    ) != 0
    {
        (*E).beetom_var_09 = ((*E).beetom_var_09) ^ (1u16);
        (*E).beetom_var_C = ((0xa8b97ai32) as u16);
    }
}
pub unsafe fn Beetom_Func_36(game: &mut Game) {
    Beetom_Func_37(ram_word!(game, cur_enemy_index), game);
}
pub unsafe fn CallBeetomFunc(ea: uint32, game: &mut Game) {
    match ea {
        11057172 => {
            Beetom_Func_3(game);
        }
        11057199 => {
            Beetom_Func_4(game);
        }
        11057231 => {
            Beetom_Func_5(game);
        }
        11057247 => {
            Beetom_Func_6(game);
        }
        11057267 => {
            Beetom_Func_7(game);
        }
        11057287 => {
            Beetom_Func_8(game);
        }
        11057321 => {
            Beetom_Func_9(game);
        }
        11057355 => {
            Beetom_Func_10(game);
        }
        11057389 => {
            Beetom_Func_11(game);
        }
        11057423 => {
            Beetom_Func_12(game);
        }
        11057490 => {
            Beetom_Func_13(game);
        }
        11057510 => {
            Beetom_Func_14(game);
        }
        11057530 => {
            Beetom_Func_15(game);
        }
        11057570 => {
            Beetom_Func_16(game);
        }
        11057586 => {
            Beetom_Func_17(game);
        }
        11057601 => {
            Beetom_Func_18(game);
        }
        11057700 => {
            Beetom_Func_19(game);
        }
        11057796 => {
            Beetom_Func_20(game);
        }
        11057847 => {
            Beetom_Func_21(game);
        }
        11058005 => {
            Beetom_Func_24(game);
        }
        11058056 => {
            Beetom_Func_25(game);
        }
        11058214 => {
            Beetom_Func_28(game);
        }
        11058266 => {
            Beetom_Func_29(game);
        }
        11058424 => {
            Beetom_Func_32(game);
        }
        11058498 => {
            Beetom_Func_33(game);
        }
        11058589 => {
            Beetom_Func_35(game);
        }
        11058629 => {
            Beetom_Func_36(game);
        }
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn Beetom_Main(game: &mut Game) {
    let E: *mut Enemy_Beetom =
        Get_Beetom(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    CallBeetomFunc((((*E).beetom_var_C) as u32) | (0xa80000u32), game);
}
pub unsafe fn Beetom_Touch(game: &mut Game) {
    let current_block: u64;
    let E: *mut Enemy_Beetom =
        Get_Beetom(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    if (*E).beetom_var_08 == 0 {
        if (*E).beetom_var_09 != 0 {
            (*E).beetom_var_C = ((0xa8b966i32) as u16);
        } else {
            (*E).beetom_var_C = ((0xa8b952i32) as u16);
        }
        (*E).beetom_var_08 = 1 as uint16;
        (*E).beetom_var_E = 64 as uint16;
        (*E).base.layer = 2 as uint16;
        (*E).beetom_var_06 = (ram_word!(game, samus_x_pos)).wrapping_sub((*E).base.x_pos);
        (*E).beetom_var_07 = (ram_word!(game, samus_y_pos)).wrapping_sub((*E).base.y_pos);
    }
    if ram_word!(game, samus_contact_damage_index) != 0 {
        current_block = 12103599785691237014;
    } else {
        if (ram_word!(game, random_enemy_counter) as ::core::ffi::c_int & 7 as ::core::ffi::c_int)
            == 7i32
            && ram_word!(game, samus_health) as ::core::ffi::c_int - 30 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0
        {
            QueueSfx3_Max6(0x2d as uint16, game);
        }
        if ((*E).base.frame_counter as ::core::ffi::c_int & 0x3f as ::core::ffi::c_int) == 63i32 {
            current_block = 12103599785691237014;
        } else {
            current_block = 10599921512955367680;
        }
    }
    if current_block == 12103599785691237014 {
        Enemy_NormalTouchAI_A8(game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x18a8) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x18aa) as *mut uint16) = 0 as uint16;
    };
}
pub unsafe fn Beetom_Shot(game: &mut Game) {
    Enemy_NormalShotAI_A8(game);
    let E: *mut Enemy_Beetom =
        Get_Beetom(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    if (*E).base.frozen_timer != 0u16
        && ((*E).beetom_var_C as ::core::ffi::c_int
            == 0xa8bd42 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
            || (*E).beetom_var_C as ::core::ffi::c_int
                == 0xa8bcf8 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int)
    {
        (*E).beetom_var_C = ((0xa8b97ai32) as u16);
    }
    (*E).beetom_var_08 = 0 as uint16;
}
pub unsafe fn MaridiaFloater_Init(game: &mut Game) {
    let E: *mut Enemy_MaridiaFloater =
        Get_MaridiaFloater(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_DisableSamusColl) as u16);
    (*E).base.spritemap_pointer = addr_kSpritemap_Nothing_A8 as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
    if (*E).mfr_parameter_1 != 0 {
        (*E).mfr_var_A = (*E).base.x_pos;
        (*E).mfr_var_B = (*E).base.y_pos;
        (*E).mfr_var_F = ((0xa8c568i32) as u16);
        (*E).base.current_instruction = addr_stru_A8C199 as uint16;
        (*E).mfr_var_D = (*E).mfr_parameter_2;
    } else {
        (*E).mfr_var_E = 60 as uint16;
        (*E).mfr_var_F = ((0xa8c283i32) as u16);
        (*E).base.current_instruction = addr_kMaridiaFloater_Ilist_C173 as uint16;
    };
}
pub unsafe fn MaridiaFloater_Main(game: &mut Game) {
    let E: *mut Enemy_MaridiaFloater =
        Get_MaridiaFloater(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    CallEnemyPreInstr((((*E).mfr_var_F) as u32) | (0xa80000u32), game);
}
pub unsafe fn MaridiaFloater_Func_1(k: uint16, game: &mut Game) {
    let mut i: ::core::ffi::c_int = 7 as ::core::ffi::c_int;
    while i >= 0i32 {
        SpawnEprojWithGfx(
            i as uint16,
            k,
            addr_kEproj_MaridiaFloatersSpikes as uint16,
            game,
        );
        i -= 1;
    }
}
pub unsafe fn MaridiaFloater_Func_2(k: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let v2: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
        .offset((v1 + 103 as ::core::ffi::c_int) as isize);
    if v2 as ::core::ffi::c_int + 0x3e6f as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        let mut v3: uint16 = ((v2 as ::core::ffi::c_int - 4 as ::core::ffi::c_int
            + 15997 as ::core::ffi::c_int) as uint16
            as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int) as uint16;
        if v3 as ::core::ffi::c_int - 6 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
            v3 = 0 as uint16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
            .offset((v1 + 93 as ::core::ffi::c_int) as isize) = ((g_word_A8C277
            [(v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize])
            as u16)
            .wrapping_add(
                (*Get_MaridiaFloater(k as ::core::ffi::c_int, game))
                    .base
                    .y_pos,
            );
    } else {
        let mut v4: uint16 = ((v2 as ::core::ffi::c_int - 4 as ::core::ffi::c_int
            + 15983 as ::core::ffi::c_int) as uint16
            as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int) as uint16;
        if v4 as ::core::ffi::c_int - 6 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
            v4 = 0 as uint16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
            .offset((v1 + 93 as ::core::ffi::c_int) as isize) = ((g_word_A8C27D
            [(v4 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize])
            as u16)
            .wrapping_add(
                (*Get_MaridiaFloater(k as ::core::ffi::c_int, game))
                    .base
                    .y_pos,
            );
    };
}
pub unsafe fn MaridiaFloater_Func_3(k: uint16, game: &mut Game) {
    let E: *mut Enemy_MaridiaFloater = Get_MaridiaFloater(k as ::core::ffi::c_int, game);
    let v2: bool = (*E).mfr_var_E == 1u16;
    (*E).mfr_var_E = (*E).mfr_var_E.wrapping_sub(1);
    let v3: bool = ((*E).mfr_var_E as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32;
    if v2 || v3 {
        let v4: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
            .offset((v4 + 104 as ::core::ffi::c_int) as isize) = 1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
            .offset((v4 + 103 as ::core::ffi::c_int) as isize) =
            addr_kMaridiaFloater_Ilist_C183 as uint16;
        (*E).mfr_var_F = ((0xa8c2a6i32) as u16);
        (*E).mfr_var_E = 10 as uint16;
    }
    MaridiaFloater_Func_2(k, game);
}
pub unsafe fn MaridiaFloater_Func_4(k: uint16, game: &mut Game) {
    let E: *mut Enemy_MaridiaFloater = Get_MaridiaFloater(k as ::core::ffi::c_int, game);
    let v2: bool = (*E).mfr_var_E == 1u16;
    (*E).mfr_var_E = (*E).mfr_var_E.wrapping_sub(1);
    let v3: bool = ((*E).mfr_var_E as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32;
    if v2 || v3 {
        (*E).mfr_var_F = ((0xa8c2cfi32) as u16);
        (*E).mfr_var_A = g_word_A8C1C5;
        (*E).mfr_var_B = g_word_A8C1C7;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.current_instruction = addr_kMaridiaFloater_Ilist_C163 as uint16;
    }
    MaridiaFloater_Func_2(k, game);
}
pub unsafe fn MaridiaFloater_Func_5(k: uint16, game: &mut Game) {
    let E: *mut Enemy_MaridiaFloater = Get_MaridiaFloater(k as ::core::ffi::c_int, game);
    if ((*E).base.ai_handler_bits as ::core::ffi::c_int & 1 as ::core::ffi::c_int) != 0i32 {
        (*E).mfr_var_F = ((0xa8c3e1i32) as u16);
        MaridiaFloater_Func_7(k, game);
    } else {
        let v2: bool = (*E).mfr_var_D == 1u16;
        (*E).mfr_var_D = (*E).mfr_var_D.wrapping_sub(1);
        let v3: bool =
            ((*E).mfr_var_D as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32;
        if v2 || v3 {
            (*E).mfr_var_D = 5 as uint16;
            let v4: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            let v5: uint16 = (g_word_A8C1A1[(*E).mfr_var_C as usize]).wrapping_add(
                *((&raw mut game.g_ram as *mut uint8).offset(0xf68) as *mut uint16)
                    .offset(v4 as isize),
            );
            *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
                .offset((v4 + 91 as ::core::ffi::c_int) as isize) = v5;
            (*E).base.x_pos = v5;
            let mut v6: uint16 = ((*E).mfr_var_C).wrapping_add(1u16);
            if (*E).mfr_var_C as ::core::ffi::c_int - 11 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0
            {
                v6 = 0 as uint16;
            }
            (*E).mfr_var_C = v6;
        }
        AddToHiLo(
            &raw mut (*E).mfr_var_A,
            &raw mut (*E).mfr_var_B,
            (g_word_A8C1B9 as uint32) << 16 as ::core::ffi::c_int | g_word_A8C1BB as uint32,
        );
        if Enemy_MoveDown(
            k,
            (((*E).mfr_var_A as uint32) << 16 as ::core::ffi::c_int | (*E).mfr_var_B as uint32)
                as int32,
            game,
        ) != 0u8
            || ((*((&raw mut game.g_ram as *mut uint8).offset(0xf68) as *mut uint16).offset(
                ((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) + 1 as ::core::ffi::c_int)
                    as isize,
            ) as ::core::ffi::c_int
                - g_word_A8C19F as ::core::ffi::c_int
                - (*E).base.y_pos as ::core::ffi::c_int) as int16)
                >= 0i16
        {
            let mfr_var_C: uint16 = (*E).mfr_var_C;
            if mfr_var_C == 0 || mfr_var_C == 6u16 {
                (*E).mfr_var_F = ((0xa8c500i32) as u16);
                (*E).mfr_var_E = 10 as uint16;
                let v10: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
                *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
                    .offset((v10 + 104 as ::core::ffi::c_int) as isize) = 1 as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
                    .offset((v10 + 103 as ::core::ffi::c_int) as isize) =
                    addr_kMaridiaFloater_Ilist_C191 as uint16;
            } else {
                (*E).mfr_var_F = ((0xa8c36bi32) as u16);
            }
        }
        MaridiaFloater_Func_2(k, game);
    };
}
pub unsafe fn MaridiaFloater_Func_6(k: uint16, game: &mut Game) {
    let mut v4: ::core::ffi::c_int = 0;
    let current_block: u64;
    let E: *mut Enemy_MaridiaFloater = Get_MaridiaFloater(k as ::core::ffi::c_int, game);
    let v2: bool = (*E).mfr_var_D == 1u16;
    (*E).mfr_var_D = (*E).mfr_var_D.wrapping_sub(1);
    let v3: bool = ((*E).mfr_var_D as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32;
    if !v2 && !v3 {
        current_block = 14916268686031723178;
    } else {
        (*E).mfr_var_D = 5 as uint16;
        v4 = 0;
        v4 = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        let v5: uint16 = (g_word_A8C1A1[(*E).mfr_var_C as usize]).wrapping_add(
            *((&raw mut game.g_ram as *mut uint8).offset(0xf68) as *mut uint16).offset(v4 as isize),
        );
        *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
            .offset((v4 + 91 as ::core::ffi::c_int) as isize) = v5;
        (*E).base.x_pos = v5;
        if (*E).mfr_var_C != 0u16 && (*E).mfr_var_C != 6u16 {
            let mut v7: uint16 = ((*E).mfr_var_C).wrapping_add(1u16);
            if v7 as ::core::ffi::c_int - 12 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                == 0
            {
                v7 = 0 as uint16;
            }
            (*E).mfr_var_C = v7;
            current_block = 14916268686031723178;
        } else {
            (*E).mfr_var_F = ((0xa8c500i32) as u16);
            (*E).mfr_var_E = 10 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
                .offset((v4 + 104 as ::core::ffi::c_int) as isize) = 1 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
                .offset((v4 + 103 as ::core::ffi::c_int) as isize) =
                addr_kMaridiaFloater_Ilist_C191 as uint16;
            current_block = 14963264723745262966;
        }
    }
    if current_block == 14916268686031723178 {
        AddToHiLo(
            &raw mut (*E).mfr_var_A,
            &raw mut (*E).mfr_var_B,
            (g_word_A8C1B9 as uint32) << 16 as ::core::ffi::c_int | g_word_A8C1BB as uint32,
        );
        Enemy_MoveDown(
            k,
            (((*E).mfr_var_A as uint32) << 16 as ::core::ffi::c_int | (*E).mfr_var_B as uint32)
                as int32,
            game,
        );
    }
    MaridiaFloater_Func_2(k, game);
}
pub unsafe fn MaridiaFloater_Func_7(k: uint16, game: &mut Game) {
    let E: *mut Enemy_MaridiaFloater = Get_MaridiaFloater(k as ::core::ffi::c_int, game);
    if ((*E).base.ai_handler_bits as ::core::ffi::c_int & 1 as ::core::ffi::c_int) != 0i32 {
        let v2: bool = (*E).mfr_var_D == 1u16;
        (*E).mfr_var_D = (*E).mfr_var_D.wrapping_sub(1);
        let v3: bool =
            ((*E).mfr_var_D as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32;
        if v2 || v3 {
            (*E).mfr_var_D = 5 as uint16;
            let v4: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            let v5: uint16 = (g_word_A8C1A1[(*E).mfr_var_C as usize]).wrapping_add(
                *((&raw mut game.g_ram as *mut uint8).offset(0xf68) as *mut uint16)
                    .offset(v4 as isize),
            );
            *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
                .offset((v4 + 91 as ::core::ffi::c_int) as isize) = v5;
            (*E).base.x_pos = v5;
            let mut v6: uint16 = ((*E).mfr_var_C).wrapping_add(1u16);
            if (*E).mfr_var_C as ::core::ffi::c_int - 11 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0
            {
                v6 = 0 as uint16;
            }
            (*E).mfr_var_C = v6;
        }
        AddToHiLo(
            &raw mut (*E).mfr_var_A,
            &raw mut (*E).mfr_var_B,
            (g_word_A8C1B9 as uint32) << 16 as ::core::ffi::c_int | g_word_A8C1BB as uint32,
        );
        if Enemy_MoveDown(
            k,
            (((*E).mfr_var_A as uint32) << 16 as ::core::ffi::c_int | (*E).mfr_var_B as uint32)
                as int32,
            game,
        ) != 0u8
            || ((*((&raw mut game.g_ram as *mut uint8).offset(0xf68) as *mut uint16).offset(
                ((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) + 1 as ::core::ffi::c_int)
                    as isize,
            ) as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0xf68) as *mut uint16).offset(
                    ((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) + 3 as ::core::ffi::c_int)
                        as isize,
                ) as ::core::ffi::c_int
                - (*E).base.y_pos as ::core::ffi::c_int) as int16)
                >= 0i16
        {
            if (*E).mfr_var_C == 0 || (*E).mfr_var_C == 6u16 {
                (*E).mfr_var_F = ((0xa8c4dci32) as u16);
            } else {
                (*E).mfr_var_F = ((0xa8c469i32) as u16);
            }
        }
        MaridiaFloater_Func_2(k, game);
    } else {
        (*E).mfr_var_F = ((0xa8c36bi32) as u16);
    };
}
pub unsafe fn MaridiaFloater_Func_8(k: uint16, game: &mut Game) {
    let mut v4: ::core::ffi::c_int = 0;
    let mut v5: uint16 = 0;
    let current_block: u64;
    let E: *mut Enemy_MaridiaFloater = Get_MaridiaFloater(k as ::core::ffi::c_int, game);
    if ((*E).base.ai_handler_bits as ::core::ffi::c_int & 1 as ::core::ffi::c_int) == 0i32 {
        (*E).mfr_var_F = ((0xa8c36bi32) as u16);
        return;
    }
    let v2: bool = (*E).mfr_var_D == 1u16;
    (*E).mfr_var_D = (*E).mfr_var_D.wrapping_sub(1);
    let v3: bool = ((*E).mfr_var_D as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32;
    if !v2 && !v3 {
        current_block = 12800627514080957624;
    } else {
        (*E).mfr_var_D = 5 as uint16;
        v4 = 0;
        v4 = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        v5 = 0;
        v5 = (g_word_A8C1A1[(*E).mfr_var_C as usize]).wrapping_add(
            *((&raw mut game.g_ram as *mut uint8).offset(0xf68) as *mut uint16).offset(v4 as isize),
        );
        *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
            .offset((v4 + 91 as ::core::ffi::c_int) as isize) = v5;
        (*E).base.x_pos = v5;
        if (*E).mfr_var_C != 0u16 && (*E).mfr_var_C != 6u16 {
            let mut v7: uint16 = ((*E).mfr_var_C).wrapping_add(1u16);
            if v7 as ::core::ffi::c_int - 12 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                == 0
            {
                v7 = 0 as uint16;
            }
            (*E).mfr_var_C = v7;
            current_block = 12800627514080957624;
        } else {
            (*E).mfr_var_F = ((0xa8c4dci32) as u16);
            current_block = 4833750592470317582;
        }
    }
    if current_block == 12800627514080957624 {
        AddToHiLo(
            &raw mut (*E).mfr_var_A,
            &raw mut (*E).mfr_var_B,
            (g_word_A8C1B9 as uint32) << 16 as ::core::ffi::c_int | g_word_A8C1BB as uint32,
        );
        Enemy_MoveDown(
            k,
            (((*E).mfr_var_A as uint32) << 16 as ::core::ffi::c_int | (*E).mfr_var_B as uint32)
                as int32,
            game,
        );
    }
    MaridiaFloater_Func_2(k, game);
}
pub unsafe fn MaridiaFloater_Func_9(k: uint16, game: &mut Game) {
    let E: *mut Enemy_MaridiaFloater = Get_MaridiaFloater(k as ::core::ffi::c_int, game);
    if ((*E).base.ai_handler_bits as ::core::ffi::c_int & 1 as ::core::ffi::c_int) == 0i32 {
        (*E).mfr_var_F = ((0xa8c500i32) as u16);
        (*E).mfr_var_E = 10 as uint16;
        let v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
            .offset((v2 + 104 as ::core::ffi::c_int) as isize) = 1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
            .offset((v2 + 103 as ::core::ffi::c_int) as isize) =
            addr_kMaridiaFloater_Ilist_C191 as uint16;
    }
    MaridiaFloater_Func_2(k, game);
}
pub unsafe fn MaridiaFloater_Func_10(k: uint16, game: &mut Game) {
    let E: *mut Enemy_MaridiaFloater = Get_MaridiaFloater(k as ::core::ffi::c_int, game);
    let v2: bool = (*E).mfr_var_E == 1u16;
    (*E).mfr_var_E = (*E).mfr_var_E.wrapping_sub(1);
    let v3: bool = ((*E).mfr_var_E as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32;
    if v2 || v3 {
        (*E).mfr_var_F = ((0xa8c51di32) as u16);
        (*E).mfr_var_A = g_word_A8C1C1;
        (*E).mfr_var_B = g_word_A8C1C3;
    }
    MaridiaFloater_Func_2(k, game);
}
pub unsafe fn MaridiaFloater_Func_11(k: uint16, game: &mut Game) {
    let E: *mut Enemy_MaridiaFloater = Get_MaridiaFloater(k as ::core::ffi::c_int, game);
    AddToHiLo(
        &raw mut (*E).mfr_var_A,
        &raw mut (*E).mfr_var_B,
        (g_word_A8C1BD as uint32) << 16 as ::core::ffi::c_int | g_word_A8C1BF as uint32,
    );
    Enemy_MoveDown(
        k,
        (((*E).mfr_var_A as uint32) << 16 as ::core::ffi::c_int | (*E).mfr_var_B as uint32)
            as int32,
        game,
    );
    let v4: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if (((*E).base.y_pos) as i16).wrapping_sub(
        (*((&raw mut game.g_ram as *mut uint8).offset(0xf68) as *mut uint16)
            .offset((v4 + 1 as ::core::ffi::c_int) as isize)) as i16,
    ) >= 0i16
    {
        (*E).base.y_pos = *((&raw mut game.g_ram as *mut uint8).offset(0xf68) as *mut uint16)
            .offset((v4 + 1 as ::core::ffi::c_int) as isize);
        (*E).mfr_var_F = ((0xa8c283i32) as u16);
        (*E).mfr_var_E = 60 as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.current_instruction = addr_kMaridiaFloater_Ilist_C173 as uint16;
    }
    MaridiaFloater_Func_2(k, game);
}
pub unsafe fn MaridiaFloater_Func_12(k: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
        .offset((v1 + 103 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
        + 0x3e6f as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        let v2: uint16 = ((*((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
            .offset((v1 + 103 as ::core::ffi::c_int) as isize)
            as ::core::ffi::c_int
            - 4 as ::core::ffi::c_int
            + 15983 as ::core::ffi::c_int) as uint16
            as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int) as uint16;
        if v2 != 0 {
            *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
                .offset((v1 + 103 as ::core::ffi::c_int) as isize) =
                *(game.g_rom.offset(crate::rtl::lorom_offset(0xa8c599)) as *const uint8
                    as *mut uint16)
                    .offset((v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
            *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
                .offset((v1 + 104 as ::core::ffi::c_int) as isize) = 1 as uint16;
        }
    }
    let E: *mut Enemy_MaridiaFloater = Get_MaridiaFloater(k as ::core::ffi::c_int, game);
    (*E).mfr_var_F = ((0xa8c59fi32) as u16);
    (*E).mfr_var_E = 32 as uint16;
    MaridiaFloater_Func_2(k, game);
}
pub unsafe fn MaridiaFloater_Func_13(k: uint16, game: &mut Game) {
    let mut v5: uint16 = 0;
    let E: *mut Enemy_MaridiaFloater = Get_MaridiaFloater(k as ::core::ffi::c_int, game);
    let v2: bool = (*E).mfr_var_E == 1u16;
    (*E).mfr_var_E = (*E).mfr_var_E.wrapping_sub(1);
    let v3: bool = ((*E).mfr_var_E as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32;
    if v2 || v3 {
        let v4: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
            .offset((v4 + 100 as ::core::ffi::c_int) as isize) = 0 as uint16;
        let ref mut fresh2 = *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
            .offset((v4 + 97 as ::core::ffi::c_int) as isize);
        *fresh2 = (*fresh2) | (0x200u16);
        MaridiaFloater_Func_1(k, game);
        v5 = 0 as uint16;
        EnemyDeathAnimation(k, v5, game);
    } else {
        MaridiaFloater_Func_2(k, game);
    };
}
pub unsafe fn MaridiaFloater_Touch(game: &mut Game) {
    let E: *mut Enemy_MaridiaFloater =
        Get_MaridiaFloater(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    if (*E).mfr_parameter_2 == 0 {
        let palette_index: uint16 = (*E).base.palette_index;
        NormalEnemyTouchAi(game);
        if (*E).base.health == 0 {
            let ref mut fresh6 =
                *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16).offset(
                    ((ram_word!(game, cur_enemy_index) as ::core::ffi::c_int
                        >> 1 as ::core::ffi::c_int)
                        + 97 as ::core::ffi::c_int) as isize,
                );
            *fresh6 = (*fresh6) | (0x200u16);
            (*E).base.palette_index = palette_index;
            MaridiaFloater_Func_1(ram_word!(game, cur_enemy_index), game);
            (*E).base.palette_index = 2560 as uint16;
        }
    }
}
pub unsafe fn MaridiaFloater_Shot(game: &mut Game) {
    let E: *mut Enemy_MaridiaFloater =
        Get_MaridiaFloater(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    if (*E).mfr_parameter_2 == 0 {
        NormalEnemyShotAiSkipDeathAnim_CurEnemy(game);
        if ((*E).base.ai_handler_bits as ::core::ffi::c_int & 4 as ::core::ffi::c_int) != 0i32 {
            let v2: ::core::ffi::c_int =
                ram_word!(game, cur_enemy_index) as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
                .offset((v2 + 109 as ::core::ffi::c_int) as isize) = (*E).base.frozen_timer;
            let ref mut fresh4 = *((&raw mut game.g_ram as *mut uint8).offset(0xe84)
                as *mut uint16)
                .offset((v2 + 99 as ::core::ffi::c_int) as isize);
            *fresh4 = (*fresh4) | (4u16);
        }
        if ((*E).base.ai_handler_bits as ::core::ffi::c_int & 2 as ::core::ffi::c_int) != 0i32 {
            let v3: ::core::ffi::c_int =
                ram_word!(game, cur_enemy_index) as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
            *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
                .offset((v3 + 108 as ::core::ffi::c_int) as isize) = (*E).base.flash_timer;
            let ref mut fresh5 = *((&raw mut game.g_ram as *mut uint8).offset(0xe84)
                as *mut uint16)
                .offset((v3 + 99 as ::core::ffi::c_int) as isize);
            *fresh5 = (*fresh5) | (2u16);
        }
        if (*E).base.health == 0 {
            (*E).mfr_var_F = ((0xa8c569i32) as u16);
            (*E).mfr_parameter_2 = 1 as uint16;
        }
    }
}
pub unsafe fn MaridiaFloater_Powerbomb(game: &mut Game) {
    Enemy_NormalPowerBombAI_A8(game);
    let E: *mut Enemy_MaridiaFloater =
        Get_MaridiaFloater(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    if (*E).base.health != 0 {
        let v1: ::core::ffi::c_int =
            ram_word!(game, cur_enemy_index) as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
            .offset((v1 + 111 as ::core::ffi::c_int) as isize) = (*E).base.shake_timer;
        *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
            .offset((v1 + 110 as ::core::ffi::c_int) as isize) = (*E).base.invincibility_timer;
        *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
            .offset((v1 + 108 as ::core::ffi::c_int) as isize) = (*E).base.flash_timer;
        *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
            .offset((v1 + 109 as ::core::ffi::c_int) as isize) = (*E).base.frozen_timer;
        *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
            .offset((v1 + 99 as ::core::ffi::c_int) as isize) = (*E).base.ai_handler_bits;
    } else {
        let ref mut fresh3 = *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
            .offset(
                ((ram_word!(game, cur_enemy_index) as ::core::ffi::c_int
                    >> 1 as ::core::ffi::c_int)
                    + 97 as ::core::ffi::c_int) as isize,
            );
        *fresh3 = (*fresh3) | (0x200u16);
    };
}
pub unsafe fn WreckedShipRobot_Init(game: &mut Game) {
    if (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xd828)
        .offset(ram_word!(game, area_index) as isize) as *mut uint8 as *mut uint16)
        as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int)
        != 0i32
    {
        (*((&raw mut game.g_ram as *mut uint8).offset(0x178c) as *mut LongPtr)).bank =
            -(88 as ::core::ffi::c_int) as uint8;
        (*((&raw mut game.g_ram as *mut uint8).offset(0x178c) as *mut LongPtr)).addr =
            ((0xa8cc67i32) as u16);
        let E: *mut Enemy_WreckedShipRobot =
            Get_WreckedShipRobot(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
        (*E).base.properties = ((*E).base.properties) | (0xa000u16);
        (*E).base.instruction_timer = 4 as uint16;
        (*E).base.timer = 0 as uint16;
        (*E).base.current_instruction = addr_kWreckedShipRobot_Ilist_C6E5 as uint16;
        (*E).wsrt_var_A = -(512 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xd654) as *mut uint16) = 1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xd656) as *mut uint16) = 0 as uint16;
        (*E).wsrt_var_B = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xd652) as *mut uint16) =
            (*E).base.palette_index;
    } else {
        WreckedShipRobotDeactivated_Init(game);
    };
}
pub unsafe fn WreckedShipRobotDeactivated_Init(game: &mut Game) {
    let v0: *mut EnemyData =
        gEnemyData(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let mut parameter_1: uint16 = (*v0).parameter_1;
    if parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0
        || parameter_1 as ::core::ffi::c_int - 4 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
    {
        parameter_1 = 0 as uint16;
    }
    (*v0).parameter_1 = parameter_1;
    (*v0).current_instruction = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa8cc30))
        as *const uint8 as *mut uint16)
        .offset(parameter_1 as isize);
    (*v0).properties = ((*v0).properties) | (0x8000u16);
    (*v0).instruction_timer = 1 as uint16;
    (*v0).timer = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xd654) as *mut uint16) = 0 as uint16;
    (*v0).ai_var_E = 0 as uint16;
    (*v0).ai_preinstr = 1 as uint16;
    let v2: ::core::ffi::c_int = 16 as ::core::ffi::c_int
        * ((ram_word!(game, wrecked_ship_robot_palanim_palindex) as ::core::ffi::c_int
            & 0xff00 as ::core::ffi::c_int)
            >> 8 as ::core::ffi::c_int)
        >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
        .offset((v2 + 137 as ::core::ffi::c_int) as isize) = 10 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
        .offset((v2 + 138 as ::core::ffi::c_int) as isize) = 10 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
        .offset((v2 + 139 as ::core::ffi::c_int) as isize) = 10 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
        .offset((v2 + 140 as ::core::ffi::c_int) as isize) = 10 as uint16;
}
pub unsafe fn WreckedShipRobot_Main(game: &mut Game) {
    let E: *mut Enemy_WreckedShipRobot =
        Get_WreckedShipRobot(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    if Enemy_MoveDown(
        ram_word!(game, cur_enemy_index),
        (((*E).wsrt_var_F as uint32) << 16 as ::core::ffi::c_int | (*E).wsrt_var_E as uint32)
            as int32,
        game,
    ) == 0
    {
        (*E).base.instruction_timer = (*E).base.instruction_timer.wrapping_add(1);
        AddToHiLo(
            &raw mut (*E).wsrt_var_F,
            &raw mut (*E).wsrt_var_E,
            0x8000 as uint32,
        );
    }
}
pub unsafe fn WreckedShipRobot_Func_1(game: &mut Game) {
    let mut i: uint16 = 0;
    let mut v2: int16 = 0;
    if ram_word!(game, door_transition_flag_enemies) == 0 {
        if ram_word!(game, wrecked_ship_robot_palanim_timer) != 0 {
            let ref mut fresh13 =
                *((&raw mut game.g_ram as *mut uint8).offset(0xd654) as *mut uint16);
            *fresh13 = (*fresh13).wrapping_sub(1);
            if *fresh13 == 0 {
                let mut v1: ::core::ffi::c_int = 0;
                i = *((&raw mut game.g_ram as *mut uint8).offset(0xd656) as *mut uint16);
                loop {
                    v1 = i as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
                    v2 = g_word_A8CCC1[v1 as usize];
                    if v2 >= 0i16 {
                        break;
                    }
                    i = 0 as uint16;
                }
                let v3: ::core::ffi::c_int = 16 as ::core::ffi::c_int
                    * ((ram_word!(game, wrecked_ship_robot_palanim_palindex)
                        as ::core::ffi::c_int
                        & 0xff00 as ::core::ffi::c_int)
                        >> 8 as ::core::ffi::c_int)
                    >> 1 as ::core::ffi::c_int;
                *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                    .offset((v3 + 137 as ::core::ffi::c_int) as isize) = v2 as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                    .offset((v3 + 138 as ::core::ffi::c_int) as isize) =
                    g_word_A8CCC1[(v1 + 1 as ::core::ffi::c_int) as usize] as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                    .offset((v3 + 139 as ::core::ffi::c_int) as isize) =
                    g_word_A8CCC1[(v1 + 2 as ::core::ffi::c_int) as usize] as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                    .offset((v3 + 140 as ::core::ffi::c_int) as isize) =
                    g_word_A8CCC1[(v1 + 3 as ::core::ffi::c_int) as usize] as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0xd654) as *mut uint16) =
                    g_word_A8CCC1[(v1 + 4 as ::core::ffi::c_int) as usize] as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0xd656) as *mut uint16) =
                    (i).wrapping_add(10u16);
            }
        }
    }
}
pub unsafe fn WreckedShipRobot_Func_2(k: uint16, game: &mut Game) {
    let E: *mut Enemy_WreckedShipRobot = Get_WreckedShipRobot(k as ::core::ffi::c_int, game);
    let wsrt_var_B: uint16 = (*E).wsrt_var_B;
    if wsrt_var_B != 0 {
        (*E).wsrt_var_B = (wsrt_var_B).wrapping_sub(1u16);
    }
}
pub unsafe fn WreckedShipRobot_Instr_4(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    WreckedShipRobot_Func_2(ram_word!(game, cur_enemy_index), game);
    let E: *mut Enemy_WreckedShipRobot =
        Get_WreckedShipRobot(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).wsrt_var_A = -(512 as ::core::ffi::c_int) as uint16;
    if Enemy_MoveRight_IgnoreSlopes(
        ram_word!(game, cur_enemy_index),
        (-(4 as ::core::ffi::c_int) as int16 as int32) << 16 as ::core::ffi::c_int,
        game,
    ) != 0
    {
        (*E).wsrt_var_B = ((*E).wsrt_var_B).wrapping_add(8u16);
        return addr_kWreckedShipRobot_Ilist_C73F as ::core::ffi::c_int as uintptr_t
            as *const uint16;
    } else {
        if CheckIfEnemyTouchesSamus(ram_word!(game, cur_enemy_index), game) != 0 {
            *((&raw mut game.g_ram as *mut uint8).offset(0xb56) as *mut uint16) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xb58) as *mut uint16) =
                -(4 as ::core::ffi::c_int) as uint16;
        }
        (*E).wsrt_var_D = (*E).base.y_pos;
        let x_pos: uint16 = (*E).base.x_pos;
        (*E).wsrt_var_C = x_pos;
        (*E).base.x_pos = (x_pos as ::core::ffi::c_int
            - (*E).base.x_width as ::core::ffi::c_int
            - (*E).base.x_width as ::core::ffi::c_int) as uint16;
        if Enemy_MoveDown(
            ram_word!(game, cur_enemy_index),
            (1 as int16 as int32) << 16 as ::core::ffi::c_int,
            game,
        ) != 0
        {
            (*E).base.x_pos = (*E).wsrt_var_C;
            (*E).base.y_pos = (*E).wsrt_var_D;
        } else {
            (*E).wsrt_var_B = ((*E).wsrt_var_B).wrapping_add(8u16);
            (*E).base.x_pos = (*E).wsrt_var_C;
            (*E).base.y_pos = (*E).wsrt_var_D;
            (*E).wsrt_var_A = 512 as uint16;
            return addr_kWreckedShipRobot_Ilist_CB65 as ::core::ffi::c_int as uintptr_t
                as *const uint16;
        }
    }
    jp
}
pub unsafe fn WreckedShipRobot_Instr_9(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    WreckedShipRobot_Func_2(ram_word!(game, cur_enemy_index), game);
    let E: *mut Enemy_WreckedShipRobot =
        Get_WreckedShipRobot(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).wsrt_var_A = -(512 as ::core::ffi::c_int) as uint16;
    if Enemy_MoveRight_IgnoreSlopes(
        ram_word!(game, cur_enemy_index),
        (-(4 as ::core::ffi::c_int) as int16 as int32) << 16 as ::core::ffi::c_int,
        game,
    ) != 0
    {
        (*E).wsrt_var_B = ((*E).wsrt_var_B).wrapping_add(8u16);
        return addr_kWreckedShipRobot_Ilist_C73F as ::core::ffi::c_int as uintptr_t
            as *const uint16;
    } else if CheckIfEnemyTouchesSamus(ram_word!(game, cur_enemy_index), game) != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0xb56) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb58) as *mut uint16) =
            -(4 as ::core::ffi::c_int) as uint16;
    }
    jp
}
pub unsafe fn WreckedShipRobot_Instr_6(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    WreckedShipRobot_Func_2(ram_word!(game, cur_enemy_index), game);
    let E: *mut Enemy_WreckedShipRobot =
        Get_WreckedShipRobot(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).wsrt_var_A = -(512 as ::core::ffi::c_int) as uint16;
    if Enemy_MoveRight_IgnoreSlopes(
        ram_word!(game, cur_enemy_index),
        (4 as int16 as int32) << 16 as ::core::ffi::c_int,
        game,
    ) != 0
    {
        (*E).wsrt_var_B = ((*E).wsrt_var_B).wrapping_add(8u16);
        return addr_stru_A8C6E9 as ::core::ffi::c_int as uintptr_t as *const uint16;
    } else {
        if CheckIfEnemyTouchesSamus(ram_word!(game, cur_enemy_index), game) != 0 {
            *((&raw mut game.g_ram as *mut uint8).offset(0xb56) as *mut uint16) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xb58) as *mut uint16) = 4 as uint16;
        }
        (*E).wsrt_var_D = (*E).base.y_pos;
        let x_pos: uint16 = (*E).base.x_pos;
        (*E).wsrt_var_C = x_pos;
        (*E).base.x_pos = ((*E).base.x_width as ::core::ffi::c_int
            + (*E).base.x_width as ::core::ffi::c_int
            + x_pos as ::core::ffi::c_int) as uint16;
        if Enemy_MoveDown(
            ram_word!(game, cur_enemy_index),
            (1 as int16 as int32) << 16 as ::core::ffi::c_int,
            game,
        ) != 0
        {
            (*E).base.x_pos = (*E).wsrt_var_C;
            (*E).base.y_pos = (*E).wsrt_var_D;
        } else {
            (*E).wsrt_var_B = ((*E).wsrt_var_B).wrapping_add(8u16);
            (*E).base.x_pos = (*E).wsrt_var_C;
            (*E).base.y_pos = (*E).wsrt_var_D;
            (*E).wsrt_var_A = -(512 as ::core::ffi::c_int) as uint16;
            return addr_kWreckedShipRobot_Ilist_C91B as ::core::ffi::c_int as uintptr_t
                as *const uint16;
        }
    }
    jp
}
pub unsafe fn WreckedShipRobot_Instr_8(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    WreckedShipRobot_Func_2(ram_word!(game, cur_enemy_index), game);
    let E: *mut Enemy_WreckedShipRobot =
        Get_WreckedShipRobot(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).wsrt_var_A = -(512 as ::core::ffi::c_int) as uint16;
    if Enemy_MoveRight_IgnoreSlopes(
        ram_word!(game, cur_enemy_index),
        (4 as int16 as int32) << 16 as ::core::ffi::c_int,
        game,
    ) != 0
    {
        (*E).wsrt_var_B = ((*E).wsrt_var_B).wrapping_add(8u16);
        return addr_stru_A8C6E9 as ::core::ffi::c_int as uintptr_t as *const uint16;
    } else if CheckIfEnemyTouchesSamus(ram_word!(game, cur_enemy_index), game) != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0xb56) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb58) as *mut uint16) = 4 as uint16;
    }
    jp
}
pub unsafe fn WreckedShipRobot_Instr_7(k: uint16, jp: *const uint16) -> *const uint16 {
    addr_kWreckedShipRobot_Ilist_C92D as ::core::ffi::c_int as uintptr_t as *const uint16
}
pub unsafe fn WreckedShipRobot_Instr_15(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    WreckedShipRobot_Func_2(ram_word!(game, cur_enemy_index), game);
    let E: *mut Enemy_WreckedShipRobot =
        Get_WreckedShipRobot(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).wsrt_var_A = 512 as uint16;
    if Enemy_MoveRight_IgnoreSlopes(
        ram_word!(game, cur_enemy_index),
        (4 as int16 as int32) << 16 as ::core::ffi::c_int,
        game,
    ) != 0
    {
        (*E).wsrt_var_B = ((*E).wsrt_var_B).wrapping_add(8u16);
        return addr_kWreckedShipRobot_Ilist_C985 as ::core::ffi::c_int as uintptr_t
            as *const uint16;
    } else {
        if CheckIfEnemyTouchesSamus(ram_word!(game, cur_enemy_index), game) != 0 {
            *((&raw mut game.g_ram as *mut uint8).offset(0xb56) as *mut uint16) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xb58) as *mut uint16) = 4 as uint16;
        }
        (*E).wsrt_var_D = (*E).base.y_pos;
        let x_pos: uint16 = (*E).base.x_pos;
        (*E).wsrt_var_C = x_pos;
        (*E).base.x_pos = ((*E).base.x_width as ::core::ffi::c_int
            + (*E).base.x_width as ::core::ffi::c_int
            + x_pos as ::core::ffi::c_int) as uint16;
        if Enemy_MoveDown(
            ram_word!(game, cur_enemy_index),
            (1 as int16 as int32) << 16 as ::core::ffi::c_int,
            game,
        ) != 0
        {
            (*E).base.x_pos = (*E).wsrt_var_C;
            (*E).base.y_pos = (*E).wsrt_var_D;
        } else {
            (*E).wsrt_var_B = ((*E).wsrt_var_B).wrapping_add(8u16);
            (*E).base.x_pos = (*E).wsrt_var_C;
            (*E).base.y_pos = (*E).wsrt_var_D;
            (*E).wsrt_var_A = -(512 as ::core::ffi::c_int) as uint16;
            return addr_kWreckedShipRobot_Ilist_C91B as ::core::ffi::c_int as uintptr_t
                as *const uint16;
        }
    }
    jp
}
pub unsafe fn WreckedShipRobot_Instr_18(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    WreckedShipRobot_Func_2(ram_word!(game, cur_enemy_index), game);
    let E: *mut Enemy_WreckedShipRobot =
        Get_WreckedShipRobot(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).wsrt_var_A = 512 as uint16;
    if Enemy_MoveRight_IgnoreSlopes(
        ram_word!(game, cur_enemy_index),
        (4 as int16 as int32) << 16 as ::core::ffi::c_int,
        game,
    ) != 0
    {
        (*E).wsrt_var_B = ((*E).wsrt_var_B).wrapping_add(8u16);
        return addr_kWreckedShipRobot_Ilist_C985 as ::core::ffi::c_int as uintptr_t
            as *const uint16;
    } else if CheckIfEnemyTouchesSamus(ram_word!(game, cur_enemy_index), game) != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0xb56) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb58) as *mut uint16) = 4 as uint16;
    }
    jp
}
pub unsafe fn WreckedShipRobot_Instr_16(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    WreckedShipRobot_Func_2(ram_word!(game, cur_enemy_index), game);
    let E: *mut Enemy_WreckedShipRobot =
        Get_WreckedShipRobot(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).wsrt_var_A = 512 as uint16;
    if Enemy_MoveRight_IgnoreSlopes(
        ram_word!(game, cur_enemy_index),
        (-(4 as ::core::ffi::c_int) as int16 as int32) << 16 as ::core::ffi::c_int,
        game,
    ) != 0
    {
        (*E).wsrt_var_B = ((*E).wsrt_var_B).wrapping_add(8u16);
        return addr_stru_A8C6E9 as ::core::ffi::c_int as uintptr_t as *const uint16;
    } else {
        if CheckIfEnemyTouchesSamus(ram_word!(game, cur_enemy_index), game) != 0 {
            *((&raw mut game.g_ram as *mut uint8).offset(0xb56) as *mut uint16) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xb58) as *mut uint16) =
                -(4 as ::core::ffi::c_int) as uint16;
        }
        (*E).wsrt_var_D = (*E).base.y_pos;
        let x_pos: uint16 = (*E).base.x_pos;
        (*E).wsrt_var_C = x_pos;
        (*E).base.x_pos = (x_pos as ::core::ffi::c_int
            - (*E).base.x_width as ::core::ffi::c_int
            - (*E).base.x_width as ::core::ffi::c_int) as uint16;
        if Enemy_MoveDown(
            ram_word!(game, cur_enemy_index),
            (1 as int16 as int32) << 16 as ::core::ffi::c_int,
            game,
        ) != 0
        {
            (*E).base.y_pos = (*E).wsrt_var_D;
            (*E).base.x_pos = (*E).wsrt_var_C;
        } else {
            (*E).wsrt_var_B = ((*E).wsrt_var_B).wrapping_add(8u16);
            (*E).base.y_pos = (*E).wsrt_var_D;
            (*E).base.x_pos = (*E).wsrt_var_C;
            (*E).wsrt_var_A = 512 as uint16;
            return addr_kWreckedShipRobot_Ilist_CB65 as ::core::ffi::c_int as uintptr_t
                as *const uint16;
        }
    }
    jp
}
pub unsafe fn WreckedShipRobot_Instr_17(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    WreckedShipRobot_Func_2(ram_word!(game, cur_enemy_index), game);
    let E: *mut Enemy_WreckedShipRobot =
        Get_WreckedShipRobot(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).wsrt_var_A = 512 as uint16;
    if Enemy_MoveRight_IgnoreSlopes(
        ram_word!(game, cur_enemy_index),
        (-(4 as ::core::ffi::c_int) as int16 as int32) << 16 as ::core::ffi::c_int,
        game,
    ) != 0
    {
        (*E).wsrt_var_B = ((*E).wsrt_var_B).wrapping_add(8u16);
        return addr_stru_A8C6E9 as ::core::ffi::c_int as uintptr_t as *const uint16;
    } else if CheckIfEnemyTouchesSamus(ram_word!(game, cur_enemy_index), game) != 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0xb56) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb58) as *mut uint16) =
            -(4 as ::core::ffi::c_int) as uint16;
    }
    jp
}
pub unsafe fn WreckedShipRobot_Instr_3(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let E: *mut Enemy_WreckedShipRobot =
        Get_WreckedShipRobot(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    if ((ram_word!(game, layer1_x_pos)) as i16).wrapping_sub(((*E).base.x_pos) as i16) < 0i16
        && ((ram_word!(game, layer1_x_pos) as ::core::ffi::c_int + 256 as ::core::ffi::c_int
            - (*E).base.x_pos as ::core::ffi::c_int) as int16)
            >= 0i16
        && ((ram_word!(game, layer1_y_pos)) as i16).wrapping_sub(((*E).base.y_pos) as i16) < 0i16
        && ((ram_word!(game, layer1_y_pos) as ::core::ffi::c_int + 224 as ::core::ffi::c_int
            - (*E).base.y_pos as ::core::ffi::c_int) as int16)
            >= 0i16
    {
        QueueSfx2_Max6(0x68 as uint16, game);
    }
    jp
}
pub unsafe fn WreckedShipRobot_Instr_10(k: uint16, jp: *const uint16) -> *const uint16 {
    addr_stru_A8C6E9 as ::core::ffi::c_int as uintptr_t as *const uint16
}
pub unsafe fn WreckedShipRobot_Instr_14(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    WreckedShipRobot_CommonInstr(
        k,
        jp,
        addr_kWreckedShipRobot_Ilist_CB1D as uint16,
        addr_kEproj_WreckedShipRobotLaserUpRight as uint16,
        game,
    )
}
pub unsafe fn WreckedShipRobot_Instr_2(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    WreckedShipRobot_CommonInstr(
        k,
        jp,
        addr_kWreckedShipRobot_Ilist_C8D1 as uint16,
        addr_kEproj_WreckedShipRobotLaserUpLeft as uint16,
        game,
    )
}
pub unsafe fn WreckedShipRobot_CommonInstr(
    k: uint16,
    jp: *const uint16,
    r50: uint16,
    r48: uint16,
    game: &mut Game,
) -> *const uint16 {
    let E: *mut Enemy_WreckedShipRobot = Get_WreckedShipRobot(k as ::core::ffi::c_int, game);
    if (*E).wsrt_var_B != 0 {
        WreckedShipRobot_Func_2(k, game);
    } else {
        let v3: uint16 = ((ram_word!(game, random_number) as ::core::ffi::c_int
            & 0x1f as ::core::ffi::c_int)
            + 16 as ::core::ffi::c_int) as uint16;
        (*E).wsrt_var_B = v3;
        SpawnEprojWithGfx(v3, ram_word!(game, cur_enemy_index), r48, game);
        return r50 as uintptr_t as *const uint16;
    }
    jp
}
pub unsafe fn WreckedShipRobot_Instr_13(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    WreckedShipRobot_D10C(k, jp, addr_kWreckedShipRobot_Ilist_CB09 as uint16, game)
}
pub unsafe fn WreckedShipRobot_Instr_1(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    WreckedShipRobot_D10C(k, jp, addr_kWreckedShipRobot_Ilist_C8BD as uint16, game)
}
pub unsafe fn WreckedShipRobot_D10C(
    k: uint16,
    jp: *const uint16,
    r50: uint16,
    game: &mut Game,
) -> *const uint16 {
    let E: *mut Enemy_WreckedShipRobot = Get_WreckedShipRobot(k as ::core::ffi::c_int, game);
    if (*E).wsrt_var_B != 0 {
        WreckedShipRobot_Func_2(k, game);
    } else {
        let v3: uint16 = ((ram_word!(game, random_number) as ::core::ffi::c_int
            & 0x1f as ::core::ffi::c_int)
            + 16 as ::core::ffi::c_int) as uint16;
        (*E).wsrt_var_B = v3;
        SpawnEprojWithGfx(
            v3,
            ram_word!(game, cur_enemy_index),
            addr_kEproj_WreckedShipRobotLaserHorizontal as uint16,
            game,
        );
        return r50 as uintptr_t as *const uint16;
    }
    jp
}
pub unsafe fn WreckedShipRobot_Instr_12(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    WreckedShipRobot_D147(
        k,
        jp,
        addr_kEproj_WreckedShipRobotLaserDownRight as uint16,
        addr_kWreckedShipRobot_Ilist_CAFD as uint16,
        game,
    )
}
pub unsafe fn WreckedShipRobot_Instr_5(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    WreckedShipRobot_D147(
        k,
        jp,
        addr_kEproj_WreckedShipRobotLaserDownLeft as uint16,
        addr_kWreckedShipRobot_Ilist_C8B1 as uint16,
        game,
    )
}
pub unsafe fn WreckedShipRobot_D147(
    k: uint16,
    jp: *const uint16,
    r48: uint16,
    r50: uint16,
    game: &mut Game,
) -> *const uint16 {
    let E: *mut Enemy_WreckedShipRobot = Get_WreckedShipRobot(k as ::core::ffi::c_int, game);
    if (*E).wsrt_var_B != 0 {
        WreckedShipRobot_Func_2(k, game);
    } else {
        let v3: uint16 = ((ram_word!(game, random_number) as ::core::ffi::c_int
            & 0x1f as ::core::ffi::c_int)
            + 16 as ::core::ffi::c_int) as uint16;
        (*E).wsrt_var_B = v3;
        SpawnEprojWithGfx(v3, ram_word!(game, cur_enemy_index), r48, game);
        return r50 as uintptr_t as *const uint16;
    }
    jp
}
pub unsafe fn WreckedShipRobot_Instr_11(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    WreckedShipRobot_Func_2(ram_word!(game, cur_enemy_index), game);
    jp
}
pub unsafe fn WreckedShipRobotDeactivated_Touch(game: &mut Game) {
    if (((*gEnemyData(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game)).x_pos) as i16)
        .wrapping_sub((ram_word!(game, samus_x_pos)) as i16)
        < 0i16
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xb58) as *mut uint16) = 4 as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xb58) as *mut uint16) =
            -(4 as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn WreckedShipRobotDeactivated_Shot(game: &mut Game) {
    NormalEnemyShotAi(game);
}
pub unsafe fn WreckedShipRobot_Shot(game: &mut Game) {
    if (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xd828)
        .offset(ram_word!(game, area_index) as isize) as *mut uint8 as *mut uint16)
        as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int)
        != 0i32
    {
        NormalEnemyShotAi(game);
        let E: *mut Enemy_WreckedShipRobot =
            Get_WreckedShipRobot(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
        if (*E).base.health != 0 {
            let mut v1: uint16 = 0;
            if ((*E).wsrt_var_A as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
                if ((ram_word!(game, samus_x_pos)) as i16).wrapping_sub(((*E).base.x_pos) as i16)
                    >= 0i16
                {
                    v1 = addr_kWreckedShipRobot_Ilist_C833 as uint16;
                } else {
                    v1 = addr_kWreckedShipRobot_Ilist_C7BB as uint16;
                }
            } else if ((ram_word!(game, samus_x_pos)) as i16).wrapping_sub(((*E).base.x_pos) as i16)
                < 0i16
            {
                v1 = addr_kWreckedShipRobot_Ilist_CA7D as uint16;
            } else {
                v1 = addr_kWreckedShipRobot_Ilist_CA01 as uint16;
            }
            (*E).base.current_instruction = v1;
            (*E).base.instruction_timer = 1 as uint16;
            (*E).wsrt_var_B = ((*E).wsrt_var_B).wrapping_add(64u16);
        }
    }
}
pub unsafe fn MaridiaPuffer_Init(game: &mut Game) {
    let E: *mut Enemy_MaridiaPuffer =
        Get_MaridiaPuffer(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
    (*E).base.current_instruction = addr_kMaridiaPuffer_Ilist_D841 as uint16;
    let v1: ::core::ffi::c_int = ((4u16).wrapping_mul((*E).mpr_parameter_1)) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    let v2: uint16 = g_word_A8D895[v1 as usize] as uint16;
    (*E).mpr_var_05 = v2;
    (*E).mpr_var_F = v2;
    (*E).mpr_var_06 = g_word_A8D895[(v1 + 1 as ::core::ffi::c_int) as usize] as uint16;
    (*E).mpr_var_E = 16 as uint16;
    (*E).mpr_var_A = ((0xa8d92bi32) as u16);
    (*E).mpr_var_00 = g_word_A8D885[(*E).mpr_parameter_2 as usize] as uint16;
}
pub unsafe fn CallMaridiaPuffer(ea: uint32, game: &mut Game) {
    match ea {
        11065643 => {
            MaridiaPuffer_Func_1(game);
        }
        11065664 => {
            MaridiaPuffer_Func_2(game);
        }
        11065699 => {
            MaridiaPuffer_Func_3(game);
        }
        11065724 => {
            MaridiaPuffer_Func_4(game);
        }
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn MaridiaPuffer_Main(game: &mut Game) {
    let E: *mut Enemy_MaridiaPuffer =
        Get_MaridiaPuffer(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let v1: uint16 = ((*E).mpr_var_07).wrapping_sub(1u16);
    (*E).mpr_var_07 = v1;
    if v1 == 0 {
        (*E).mpr_var_07 = 1 as uint16;
        (*E).mpr_var_03 = 0 as uint16;
    }
    CallMaridiaPuffer((((*E).mpr_var_A) as u32) | (0xa80000u32), game);
}
pub unsafe fn MaridiaPuffer_Func_1(game: &mut Game) {
    let E: *mut Enemy_MaridiaPuffer =
        Get_MaridiaPuffer(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let fresh7 = (*E).mpr_var_E;
    (*E).mpr_var_E = (*E).mpr_var_E.wrapping_sub(1);
    if fresh7 == 1u16 {
        (*E).mpr_var_E = 16 as uint16;
        (*E).mpr_var_A = ((0xa8d940i32) as u16);
    }
}
pub unsafe fn MaridiaPuffer_Func_2(game: &mut Game) {
    let v1: uint8 = ((CalculateAngleOfSamusFromEnemy(ram_word!(game, cur_enemy_index), game))
        as u8)
        .wrapping_sub(64u8);
    let E: *mut Enemy_MaridiaPuffer =
        Get_MaridiaPuffer(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).mpr_var_01 = v1 as uint16;
    (*E).mpr_var_02 = v1 as uint16;
    (*E).mpr_var_A = ((0xa8d963i32) as u16);
    (*E).mpr_var_C = 24 as uint16;
}
pub unsafe fn MaridiaPuffer_Func_3(game: &mut Game) {
    let E: *mut Enemy_MaridiaPuffer =
        Get_MaridiaPuffer(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    if (((*E).mpr_var_D) as i16).wrapping_sub(((*E).mpr_var_00) as i16) < 0i16 {
        MaridiaPuffer_Func_8(ram_word!(game, cur_enemy_index), game);
    }
    MaridiaPuffer_Func_6(game);
    MaridiaPuffer_Func_7(game);
    MaridiaPuffer_Func_5(game);
}
pub unsafe fn MaridiaPuffer_Func_4(game: &mut Game) {
    let E: *mut Enemy_MaridiaPuffer =
        Get_MaridiaPuffer(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    if ((*E).mpr_var_D as int16) > 0i16 && ((*E).mpr_var_B as int16) > 0i16 {
        MaridiaPuffer_Func_6(game);
        MaridiaPuffer_Func_7(game);
        MaridiaPuffer_Func_9(ram_word!(game, cur_enemy_index), game);
    } else {
        (*E).mpr_var_A = ((0xa8d92bi32) as u16);
        (*E).mpr_var_B = 0 as uint16;
        (*E).mpr_var_C = 0 as uint16;
        (*E).mpr_var_D = 0 as uint16;
    };
}
pub unsafe fn MaridiaPuffer_Func_5(game: &mut Game) {
    let v1: uint8 = ((CalculateAngleOfSamusFromEnemy(ram_word!(game, cur_enemy_index), game))
        as u8)
        .wrapping_sub(64u8);
    let E: *mut Enemy_MaridiaPuffer =
        Get_MaridiaPuffer(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).mpr_var_01 = v1 as uint16;
    let v3: uint16 = SignExtend8(((v1) as u16).wrapping_sub((*E).mpr_var_02));
    let v4: uint16 = Abs16(v3);
    if v4 as ::core::ffi::c_int - 48 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
        (*E).mpr_var_A = ((0xa8d97ci32) as u16);
        (*E).mpr_var_C = 24 as uint16;
    }
}
pub unsafe fn MaridiaPuffer_Func_6(game: &mut Game) {
    let E: *mut Enemy_MaridiaPuffer =
        Get_MaridiaPuffer(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let mut R26: uint16 = 0 as uint16;
    let r18: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa0b1c3)) as *const uint8
        as *mut uint16)
        .offset(((((*E).mpr_var_02) as u8).wrapping_add(64u8)) as isize);
    if (r18 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        R26 = R26.wrapping_add(1);
    }
    let mut t: uint32 = (((Abs16(r18) as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int)
        >> 8 as ::core::ffi::c_int)
        * (*E).mpr_var_D as ::core::ffi::c_int) as uint32;
    if R26 != 0 {
        t = -(t as int32) as uint32;
    }
    AddToHiLo(&raw mut (*E).base.x_pos, &raw mut (*E).base.x_subpos, t);
}
pub unsafe fn MaridiaPuffer_Func_7(game: &mut Game) {
    let E: *mut Enemy_MaridiaPuffer =
        Get_MaridiaPuffer(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let mut R26: uint16 = 0 as uint16;
    let r18: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa0b1c3)) as *const uint8
        as *mut uint16)
        .offset((*E).mpr_var_02 as uint8 as isize);
    if (r18 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        R26 = R26.wrapping_add(1);
    }
    let mut t: uint32 = (((Abs16(r18) as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int)
        >> 8 as ::core::ffi::c_int)
        * (*E).mpr_var_D as ::core::ffi::c_int) as uint32;
    if R26 != 0 {
        t = -(t as int32) as uint32;
    }
    AddToHiLo(&raw mut (*E).base.y_pos, &raw mut (*E).base.y_subpos, t);
}
pub unsafe fn MaridiaPuffer_Func_8(k: uint16, game: &mut Game) {
    let E: *mut Enemy_MaridiaPuffer = Get_MaridiaPuffer(k as ::core::ffi::c_int, game);
    let fresh8 = (*E).mpr_var_F;
    (*E).mpr_var_F = (*E).mpr_var_F.wrapping_sub(1);
    if fresh8 == 1u16 {
        (*E).mpr_var_F = (*E).mpr_var_05;
        (*E).mpr_var_B = ((*E).mpr_var_B).wrapping_add((*E).mpr_var_C);
        (*E).mpr_var_D = ((*E).mpr_var_D).wrapping_add((*E).mpr_var_B);
    }
}
pub unsafe fn MaridiaPuffer_Func_9(k: uint16, game: &mut Game) {
    let E: *mut Enemy_MaridiaPuffer = Get_MaridiaPuffer(k as ::core::ffi::c_int, game);
    let fresh9 = (*E).mpr_var_F;
    (*E).mpr_var_F = (*E).mpr_var_F.wrapping_sub(1);
    if fresh9 == 1u16 {
        (*E).mpr_var_F = (*E).mpr_var_06;
        (*E).mpr_var_B = ((*E).mpr_var_B).wrapping_sub((*E).mpr_var_C);
        (*E).mpr_var_D = ((*E).mpr_var_D).wrapping_sub((*E).mpr_var_B);
    }
}
pub unsafe fn MaridiaPuffer_Shot(game: &mut Game) {
    let E: *mut Enemy_MaridiaPuffer =
        Get_MaridiaPuffer(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).mpr_var_40 = (*E).base.health;
    Enemy_NormalShotAI_A8(game);
    if (*E).base.health == (*E).mpr_var_40 && (*E).mpr_var_03 == 0 {
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
        (*E).base.current_instruction = addr_kMaridiaPuffer_Ilist_D855 as uint16;
        (*E).mpr_var_02 = g_word_A8D871[(*((&raw mut game.g_ram as *mut uint8).offset(0xc04)
            as *mut uint16)
            .offset(ram_word!(game, collision_detection_index) as isize)
            as ::core::ffi::c_int
            & 0xf as ::core::ffi::c_int) as usize] as uint16;
        (*E).mpr_var_B = 256 as uint16;
        (*E).mpr_var_D = 1536 as uint16;
        (*E).mpr_var_A = ((0xa8d97ci32) as u16);
        (*E).mpr_var_07 = 48 as uint16;
        (*E).mpr_var_03 = 1 as uint16;
    }
}
pub unsafe fn WalkingLavaSeahorse_Init(game: &mut Game) {
    let E: *mut Enemy_WalkingLavaSeahorse =
        Get_WalkingLavaSeahorse(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).wlse_var_04 = 0 as uint16;
    (*E).wlse_var_F = (*E).base.y_pos;
    (*E).wlse_var_02 = (*E).base.x_pos;
    (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_DisableSamusColl) as u16);
    (*E).base.current_instruction = addr_kWalkingLavaSeahorse_Ilist_DBE7 as uint16;
    (*E).wlse_var_A = ((0xa8dd71i32) as u16);
    WalkingLavaSeahorse_Func_1(ram_word!(game, cur_enemy_index), game);
    loop {
        AddToHiLo(
            &raw mut (*E).base.y_pos,
            &raw mut (*E).base.y_subpos,
            ((*E).wlse_var_B as uint32) << 16 as ::core::ffi::c_int | (*E).wlse_var_C as uint32,
        );
        if ((WalkingLavaSeahorse_Func_2(ram_word!(game, cur_enemy_index), game)
            as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int)
            == 0i32)
        {
            break;
        }
    }
    while (Enemy_MoveDown(
        ram_word!(game, cur_enemy_index),
        (((*E).wlse_var_B as uint32) << 16 as ::core::ffi::c_int | (*E).wlse_var_C as uint32)
            as int32,
        game,
    ) == 0)
    {
        WalkingLavaSeahorse_Func_2(ram_word!(game, cur_enemy_index), game);
    }
    (*E).wlse_var_03 = (*E).base.y_pos;
    (*E).base.y_pos = (*E).wlse_var_F;
}
pub unsafe fn WalkingLavaSeahorse_Func_1(k: uint16, game: &mut Game) {
    let E: *mut Enemy_WalkingLavaSeahorse = Get_WalkingLavaSeahorse(k as ::core::ffi::c_int, game);
    (*E).wlse_var_B = -(12 as ::core::ffi::c_int) as uint16;
    (*E).wlse_var_C = 0 as uint16;
    (*E).wlse_var_D = 0 as uint16;
    (*E).wlse_var_E = 0 as uint16;
    (*E).wlse_var_00 = 0 as uint16;
    (*E).wlse_var_01 = 0x8000 as uint16;
}
pub unsafe fn WalkingLavaSeahorse_Func_2(k: uint16, game: &mut Game) -> uint16 {
    let E: *mut Enemy_WalkingLavaSeahorse = Get_WalkingLavaSeahorse(k as ::core::ffi::c_int, game);
    AddToHiLo(
        &raw mut (*E).wlse_var_B,
        &raw mut (*E).wlse_var_C,
        ((*E).wlse_var_00 as uint32) << 16 as ::core::ffi::c_int | (*E).wlse_var_01 as uint32,
    );
    (*E).wlse_var_B
}
pub unsafe fn CallWalkingLavaSeahorseFunc(ea: uint32, k: uint16, game: &mut Game) {
    match ea {
        11067084 => (),
        11066737 => {
            WalkingLavaSeahorse_Func_3(k, game);
        }
        11066822 => {
            WalkingLavaSeahorse_Func_4(k, game);
        }
        11066885 => {
            WalkingLavaSeahorse_Func_6(k, game);
        }
        11066955 => {
            WalkingLavaSeahorse_Func_7(k, game);
        }
        11067085 => {
            WalkingLavaSeahorse_Func_8(k, game);
        }
        11067116 => {
            WalkingLavaSeahorse_Func_9(k, game);
        }
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn WalkingLavaSeahorse_Main(game: &mut Game) {
    let E: *mut Enemy_WalkingLavaSeahorse =
        Get_WalkingLavaSeahorse(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    CallWalkingLavaSeahorseFunc(
        (((*E).wlse_var_A) as u32) | (0xa80000u32),
        ram_word!(game, cur_enemy_index),
        game,
    );
}
pub unsafe fn WalkingLavaSeahorse_Func_3(k: uint16, game: &mut Game) {
    let mut v4: uint16 = 0;
    let mut v5: int16 = 0;
    let E: *mut Enemy_WalkingLavaSeahorse = Get_WalkingLavaSeahorse(k as ::core::ffi::c_int, game);
    let mut xxx: uint16 = 0;
    let v3: uint16 = abs16(((*E).wlse_var_03).wrapping_sub(ram_word!(game, samus_y_pos)));
    if v3 as ::core::ffi::c_int - 32 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 && {
        xxx = (ram_word!(game, samus_x_pos)).wrapping_sub((*E).base.x_pos);
        ((abs16(xxx)) as i16).wrapping_sub((g_word_A8DCC7) as i16) < 0i16
    } {
        WalkingLavaSeahorse_Func_1(k, game);
        v4 = addr_stru_A8DC4B as uint16;
        v5 = -(2 as ::core::ffi::c_int) as int16;
        if xxx as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0 {
            v4 = addr_stru_A8DCBB as uint16;
            v5 = 2 as int16;
        }
        (*E).wlse_var_D = v5 as uint16;
        (*E).base.current_instruction = v4;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).wlse_var_A = ((0xa8ddc6i32) as u16);
        QueueSfx2_Max6(0x5e as uint16, game);
    }
}
pub unsafe fn WalkingLavaSeahorse_Func_4(k: uint16, game: &mut Game) {
    let E: *mut Enemy_WalkingLavaSeahorse = Get_WalkingLavaSeahorse(k as ::core::ffi::c_int, game);
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        ((*E).wlse_var_B as uint32) << 16 as ::core::ffi::c_int | (*E).wlse_var_C as uint32,
    );
    if (WalkingLavaSeahorse_Func_2(k, game) as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int)
        == 0i32
    {
        sub_A8DDDE(k, game);
    }
}
pub unsafe fn sub_A8DDDE(k: uint16, game: &mut Game) {
    let v1: *mut EnemyData = gEnemyData(k as ::core::ffi::c_int, game);
    (*v1).ai_var_A = ((0xa8de05i32) as u16);
    let mut v2: uint16 = addr_kWalkingLavaSeahorse_Ilist_DC51 as uint16;
    if ((*v1).ai_var_D as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) == 0i32 {
        v2 = addr_kWalkingLavaSeahorse_Ilist_DCC1 as uint16;
    }
    (*v1).current_instruction = v2;
    (*v1).instruction_timer = 1 as uint16;
}
pub unsafe fn WalkingLavaSeahorse_Func_5(k: uint16, game: &mut Game) {
    let E: *mut Enemy_WalkingLavaSeahorse = Get_WalkingLavaSeahorse(k as ::core::ffi::c_int, game);
    (*E).wlse_var_B = 0 as uint16;
    (*E).wlse_var_C = 0 as uint16;
    sub_A8DDDE(k, game);
}
pub unsafe fn WalkingLavaSeahorse_Func_6(k: uint16, game: &mut Game) {
    let mut v3: int16 = 0;
    let E: *mut Enemy_WalkingLavaSeahorse = Get_WalkingLavaSeahorse(k as ::core::ffi::c_int, game);
    if Enemy_MoveDown(
        k,
        (((*E).wlse_var_B as uint32) << 16 as ::core::ffi::c_int | (*E).wlse_var_C as uint32)
            as int32,
        game,
    ) != 0
    {
        let t: ::core::ffi::c_int = ram_word!(game, samus_x_pos) as ::core::ffi::c_int
            - (*E).base.x_pos as ::core::ffi::c_int;
        let mut v2: uint16 = addr_kWalkingLavaSeahorse_Ilist_DBE7 as uint16;
        v3 = -(2 as ::core::ffi::c_int) as int16;
        if t & 0x8000 as ::core::ffi::c_int == 0 {
            v3 = 2 as int16;
            v2 = addr_kWalkingLavaSeahorse_Ilist_DC57 as uint16;
        }
        (*E).wlse_var_D = v3 as uint16;
        (*E).base.current_instruction = v2;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).wlse_var_A = ((0xa8de4bi32) as u16);
        (*E).wlse_var_04 = 1 as uint16;
    } else {
        WalkingLavaSeahorse_Func_2(k, game);
    };
}
pub unsafe fn WalkingLavaSeahorse_Func_7(k: uint16, game: &mut Game) {
    Enemy_MoveDown(k, (2 as int16 as int32) << 16 as ::core::ffi::c_int, game);
    let E: *mut Enemy_WalkingLavaSeahorse = Get_WalkingLavaSeahorse(k as ::core::ffi::c_int, game);
    if ((abs16(((*E).wlse_var_02).wrapping_sub((*E).base.x_pos))) as i16)
        .wrapping_sub((g_word_A8DCCB) as i16)
        >= 0i16
    {
        (*E).wlse_var_A = ((0xa8decdi32) as u16);
        (*E).wlse_var_B = -(4 as ::core::ffi::c_int) as uint16;
        (*E).wlse_var_C = 0 as uint16;
        let mut v4: uint16 = addr_kWalkingLavaSeahorse_Ilist_DC51 as uint16;
        if ((*E).wlse_var_D as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) == 0i32 {
            v4 = addr_kWalkingLavaSeahorse_Ilist_DCC1 as uint16;
        }
        (*E).base.current_instruction = v4;
        (*E).base.instruction_timer = 1 as uint16;
    } else {
        if (*E).wlse_var_04 != 0 {
            return;
        }
        let t: ::core::ffi::c_int = ram_word!(game, samus_x_pos) as ::core::ffi::c_int
            - (*E).base.x_pos as ::core::ffi::c_int;
        let mut v3: uint16 = 0;
        if t & 0x8000 as ::core::ffi::c_int == 0 {
            v3 = addr_kWalkingLavaSeahorse_Ilist_DC73 as uint16;
            if ((*E).wlse_var_D as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
                return;
            }
        } else {
            v3 = addr_kWalkingLavaSeahorse_Ilist_DC03 as uint16;
            if ((*E).wlse_var_D as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) == 0i32 {
                return;
            }
        }
        (*E).base.current_instruction = v3;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).wlse_var_A = ((0xa8decci32) as u16);
    };
}
pub unsafe fn WalkingLavaSeahorse_Func_8(k: uint16, game: &mut Game) {
    let E: *mut Enemy_WalkingLavaSeahorse = Get_WalkingLavaSeahorse(k as ::core::ffi::c_int, game);
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        ((*E).wlse_var_B as uint32) << 16 as ::core::ffi::c_int | (*E).wlse_var_C as uint32,
    );
    if (WalkingLavaSeahorse_Func_2(k, game) as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int)
        == 0i32
    {
        (*E).wlse_var_A = ((0xa8deeci32) as u16);
    }
}
pub unsafe fn WalkingLavaSeahorse_Func_9(k: uint16, game: &mut Game) {
    let E: *mut Enemy_WalkingLavaSeahorse = Get_WalkingLavaSeahorse(k as ::core::ffi::c_int, game);
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        ((*E).wlse_var_B as uint32) << 16 as ::core::ffi::c_int | (*E).wlse_var_C as uint32,
    );
    if (((*E).base.y_pos) as i16).wrapping_sub(((*E).wlse_var_F) as i16) >= 0i16 {
        (*E).base.y_pos = (*E).wlse_var_F;
        (*E).base.x_pos = (*E).wlse_var_02;
        (*E).wlse_var_A = ((0xa8dd71i32) as u16);
    } else {
        WalkingLavaSeahorse_Func_2(k, game);
    };
}
pub unsafe fn WalkingLavaSeahorse_Instr_4(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    WalkingLavaSeahorse_DF20(0 as uint16, game);
    jp
}
pub unsafe fn WalkingLavaSeahorse_DF20(a: uint16, game: &mut Game) {
    SpawnEprojWithGfx(
        a,
        ram_word!(game, cur_enemy_index),
        addr_loc_A89E90 as uint16,
        game,
    );
    QueueSfx2_Max6(0x3f as uint16, game);
}
pub unsafe fn WalkingLavaSeahorse_Instr_3(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    WalkingLavaSeahorse_DF20(2 as uint16, game);
    jp
}
pub unsafe fn WalkingLavaSeahorse_Instr_5(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    WalkingLavaSeahorse_DF20(4 as uint16, game);
    jp
}
pub unsafe fn WalkingLavaSeahorse_Instr_6(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut v3: int16 = 0;
    let E: *mut Enemy_WalkingLavaSeahorse =
        Get_WalkingLavaSeahorse(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).wlse_var_A = ((0xa8de4bi32) as u16);
    v3 = ((ram_word!(game, random_number)) as i16) & (3i16);
    if (ram_word!(game, random_number) as ::core::ffi::c_int & 3 as ::core::ffi::c_int) == 0i32 {
        v3 = 2 as int16;
    }
    (*E).wlse_var_04 = v3 as uint16;
    if ((*E).wlse_var_D as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) == 0i32 {
        return addr_kWalkingLavaSeahorse_Ilist_DC57 as ::core::ffi::c_int as uintptr_t
            as *const uint16;
    }
    addr_kWalkingLavaSeahorse_Ilist_DBE7 as ::core::ffi::c_int as uintptr_t as *const uint16
}
pub unsafe fn WalkingLavaSeahorse_Instr_2(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let E: *mut Enemy_WalkingLavaSeahorse =
        Get_WalkingLavaSeahorse(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let wlse_var_04: uint16 = (*E).wlse_var_04;
    if wlse_var_04 != 0 {
        (*E).wlse_var_04 = (wlse_var_04).wrapping_sub(1u16);
    }
    WalkingLavaSeahorse_Instr_1(ram_word!(game, cur_enemy_index), jp, game)
}
pub unsafe fn WalkingLavaSeahorse_Instr_1(
    k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut wlse_var_D: int16 = 0;
    let E: *mut Enemy_WalkingLavaSeahorse =
        Get_WalkingLavaSeahorse(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    if Enemy_MoveRight_IgnoreSlopes(
        ram_word!(game, cur_enemy_index),
        ((*E).wlse_var_D as int16 as int32) << 16 as ::core::ffi::c_int,
        game,
    ) != 0
    {
        jp = addr_stru_A8DBE9 as ::core::ffi::c_int as uintptr_t as *const uint16;
        wlse_var_D = (*E).wlse_var_D as int16;
        if wlse_var_D < 0i16 {
            jp = addr_stru_A8DC59 as ::core::ffi::c_int as uintptr_t as *const uint16;
        }
        (*E).wlse_var_D = -(wlse_var_D as ::core::ffi::c_int) as uint16;
    } else {
        EnemyFunc_C8AD(ram_word!(game, cur_enemy_index), game);
    }
    jp
}
pub unsafe fn WalkingLavaSeahorse_Func_10(game: &mut Game) {
    NormalEnemyShotAi(game);
}
pub unsafe fn WreckedShipOrbs_Init(game: &mut Game) {
    let E: *mut Enemy_WreckedShipOrbs =
        Get_WreckedShipOrbs(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
    (*E).base.current_instruction = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa8e380))
        as *const uint8 as *mut uint16)
        .offset((*E).wsos_parameter_1 as isize);
    let v1: ::core::ffi::c_int = 8 as ::core::ffi::c_int
        * (*E).wsos_parameter_2 as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    (*E).wsos_var_01 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8
        as *mut uint16)
        .offset(v1 as isize);
    (*E).wsos_var_00 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8
        as *mut uint16)
        .offset((v1 + 1 as ::core::ffi::c_int) as isize);
    (*E).wsos_var_03 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8
        as *mut uint16)
        .offset((v1 + 2 as ::core::ffi::c_int) as isize);
    (*E).wsos_var_02 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa28187)) as *const uint8
        as *mut uint16)
        .offset((v1 + 3 as ::core::ffi::c_int) as isize);
}
pub unsafe fn CallWreckedShipOrbsA(ea: uint32, game: &mut Game) {
    match ea {
        11068421 => {
            WreckedShipOrbs_Func_3(game);
        }
        11068452 => {
            WreckedShipOrbs_Func_4(game);
        }
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn CallWreckedShipOrbsB(ea: uint32, game: &mut Game) {
    match ea {
        11068483 => {
            WreckedShipOrbs_Func_5(game);
        }
        11068514 => {
            WreckedShipOrbs_Func_6(game);
        }
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn WreckedShipOrbs_Main(game: &mut Game) {
    WreckedShipOrbs_Func_1(game);
    WreckedShipOrbs_Func_2(game);
    let E: *mut Enemy_WreckedShipOrbs =
        Get_WreckedShipOrbs(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    CallWreckedShipOrbsA((((*E).wsos_var_A) as u32) | (0xa80000u32), game);
    CallWreckedShipOrbsB((((*E).wsos_var_B) as u32) | (0xa80000u32), game);
}
pub unsafe fn WreckedShipOrbs_Func_1(game: &mut Game) {
    let E: *mut Enemy_WreckedShipOrbs =
        Get_WreckedShipOrbs(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).wsos_var_A = ((0xa8e405i32) as u16);
    if (GetSamusEnemyDelta_Y(ram_word!(game, cur_enemy_index), game) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int)
        == 0i32
    {
        (*E).wsos_var_A = ((0xa8e424i32) as u16);
    }
}
pub unsafe fn WreckedShipOrbs_Func_2(game: &mut Game) {
    let E: *mut Enemy_WreckedShipOrbs =
        Get_WreckedShipOrbs(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).wsos_var_B = ((0xa8e443i32) as u16);
    if (GetSamusEnemyDelta_X(ram_word!(game, cur_enemy_index), game) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int)
        == 0i32
    {
        (*E).wsos_var_B = ((0xa8e462i32) as u16);
    }
}
pub unsafe fn WreckedShipOrbs_Func_3(game: &mut Game) {
    let E: *mut Enemy_WreckedShipOrbs =
        Get_WreckedShipOrbs(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        ((*E).wsos_var_03 as uint32) << 16 as ::core::ffi::c_int | (*E).wsos_var_02 as uint32,
    );
}
pub unsafe fn WreckedShipOrbs_Func_4(game: &mut Game) {
    let E: *mut Enemy_WreckedShipOrbs =
        Get_WreckedShipOrbs(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        ((*E).wsos_var_01 as uint32) << 16 as ::core::ffi::c_int | (*E).wsos_var_00 as uint32,
    );
}
pub unsafe fn WreckedShipOrbs_Func_5(game: &mut Game) {
    let E: *mut Enemy_WreckedShipOrbs =
        Get_WreckedShipOrbs(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        ((*E).wsos_var_03 as uint32) << 16 as ::core::ffi::c_int | (*E).wsos_var_02 as uint32,
    );
}
pub unsafe fn WreckedShipOrbs_Func_6(game: &mut Game) {
    let E: *mut Enemy_WreckedShipOrbs =
        Get_WreckedShipOrbs(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        ((*E).wsos_var_01 as uint32) << 16 as ::core::ffi::c_int | (*E).wsos_var_00 as uint32,
    );
}
pub unsafe fn WreckedShipSpark_Instr_2(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let E: *mut Enemy_WreckedShipSpark =
        Get_WreckedShipSpark(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
    jp
}
pub unsafe fn WreckedShipSpark_Instr_1(
    k: uint16,
    jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let E: *mut Enemy_WreckedShipSpark =
        Get_WreckedShipSpark(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        & !(kEnemyProps_Tangible as ::core::ffi::c_int)) as uint16;
    jp
}
unsafe fn WreckedShipSpark_Func_4(k: uint16, r18: uint16, game: &mut Game) {
    let mut wssk_var_E: int16 = 0;
    let E: *mut Enemy_WreckedShipSpark = Get_WreckedShipSpark(k as ::core::ffi::c_int, game);
    wssk_var_E = (*E).wssk_var_E as int16;
    if wssk_var_E < 0i16 {
        wssk_var_E = ((NextRandom(game) as ::core::ffi::c_int & 0x3f as ::core::ffi::c_int)
            + 4 as ::core::ffi::c_int) as int16;
    }
    (*E).wssk_var_F = (r18).wrapping_add((wssk_var_E) as u16);
}
pub unsafe fn WreckedShipSpark_Init(game: &mut Game) {
    let E: *mut Enemy_WreckedShipSpark =
        Get_WreckedShipSpark(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let v2: ::core::ffi::c_int =
        (*E).wssk_parameter_1 as ::core::ffi::c_int & 3 as ::core::ffi::c_int;
    (*E).wssk_var_B = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa8e688)) as *const uint8
        as *mut uint16)
        .offset(v2 as isize);
    (*E).wssk_var_E = (*E).wssk_parameter_2;
    WreckedShipSpark_Func_4(ram_word!(game, cur_enemy_index), 0 as uint16, game);
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.current_instruction = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa8e682))
        as *const uint8 as *mut uint16)
        .offset(v2 as isize);
    (*E).base.timer = 0 as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    if (*(&raw mut game.g_ram as *mut uint8)
        .offset(0xd828)
        .offset(ram_word!(game, area_index) as isize) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int)
        == 0i32
    {
        (*E).base.properties = ((*E).base.properties)
            | (*((&raw mut (*(gVramWriteEntry
                as unsafe fn(::core::ffi::c_int, &mut Game) -> *mut VramWriteEntry)(
                0 as ::core::ffi::c_int,
                game,
            )
            .offset(6))
            .vram_dst as *mut uint8)
                .offset(1) as *mut uint16));
    }
}
pub unsafe fn CallWreckedShipSpark(ea: uint32, k: uint16, game: &mut Game) {
    match ea {
        11069077 => {
            WreckedShipSpark_Func_1(k, game);
        }
        11069111 => {
            WreckedShipSpark_Func_2(k, game);
        }
        11069148 => {
            WreckedShipSpark_Func_3(k, game);
        }
        11069076 => (),
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn WreckedShipSpark_Main(game: &mut Game) {
    let E: *mut Enemy_WreckedShipSpark =
        Get_WreckedShipSpark(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    CallWreckedShipSpark(
        (((*E).wssk_var_B) as u32) | (0xa80000u32),
        ram_word!(game, cur_enemy_index),
        game,
    );
}
pub unsafe fn WreckedShipSpark_Func_1(k: uint16, game: &mut Game) {
    let E: *mut Enemy_WreckedShipSpark = Get_WreckedShipSpark(k as ::core::ffi::c_int, game);
    if (*E).wssk_var_F == 1u16 {
        (*E).wssk_var_B = ((0xa8e6b7i32) as u16);
        (*E).base.current_instruction = addr_kWreckedShipSpark_Ilist_E5A7 as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        WreckedShipSpark_Func_4(k, 0 as uint16, game);
    } else {
        (*E).wssk_var_F = (*E).wssk_var_F.wrapping_sub(1);
    };
}
pub unsafe fn WreckedShipSpark_Func_2(k: uint16, game: &mut Game) {
    let E: *mut Enemy_WreckedShipSpark = Get_WreckedShipSpark(k as ::core::ffi::c_int, game);
    if (*E).wssk_var_F == 1u16 {
        (*E).wssk_var_B = ((0xa8e695i32) as u16);
        (*E).base.current_instruction = addr_kWreckedShipSpark_Ilist_E5E5 as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        WreckedShipSpark_Func_4(k, 8 as uint16, game);
    } else {
        (*E).wssk_var_F = (*E).wssk_var_F.wrapping_sub(1);
    };
}
pub unsafe fn WreckedShipSpark_Func_3(k: uint16, game: &mut Game) {
    let E: *mut Enemy_WreckedShipSpark = Get_WreckedShipSpark(k as ::core::ffi::c_int, game);
    if (*E).wssk_var_F == 1u16 {
        let v2: uint16 = ram_word!(game, cur_enemy_index);
        SpawnEprojWithGfx(
            0 as uint16,
            ram_word!(game, cur_enemy_index),
            addr_kEproj_Sparks as uint16,
            game,
        );
        WreckedShipSpark_Func_4(v2, 0 as uint16, game);
    } else {
        (*E).wssk_var_F = (*E).wssk_var_F.wrapping_sub(1);
    };
}
pub unsafe fn WreckedShipSpark_Shot(game: &mut Game) {
    let ref mut fresh14 = *((&raw mut game.g_ram as *mut uint8).offset(0xc04) as *mut uint16)
        .offset(ram_word!(game, collision_detection_index) as isize);
    *fresh14 = (*fresh14 as ::core::ffi::c_int & !(0x10 as ::core::ffi::c_int)) as uint16;
}
pub unsafe fn BlueBrinstarFaceBlock_Init(game: &mut Game) {
    let v0: *mut EnemyData =
        gEnemyData(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*v0).current_instruction = addr_kBlueBrinstarFaceBlock_Ilist_E828 as uint16;
    let mut v1: uint16 = ((0xa8e86ei32) as u16);
    if (ram_word!(game, collected_items) as ::core::ffi::c_int & 4 as ::core::ffi::c_int) == 0i32 {
        v1 = ((0xa8804ci32) as u16);
    }
    (*((&raw mut game.g_ram as *mut uint8).offset(0x178c) as *mut LongPtr)).addr = v1;
    *(&raw mut (*((&raw mut game.g_ram as *mut uint8).offset(0x178c) as *mut LongPtr)).bank
        as *mut uint16) = 168 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1794) as *mut uint16).offset(0) =
        ((16 as ::core::ffi::c_int * (*v0).palette_index as ::core::ffi::c_int
            & 0xff00 as ::core::ffi::c_int)
            >> 8 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1794) as *mut uint16).offset(2) = 16 as uint16;
    (*v0).parameter_2 = (((*v0).parameter_2 as ::core::ffi::c_int & 1 as ::core::ffi::c_int)
        >> 2 as ::core::ffi::c_int
        | ((*v0).parameter_2 as ::core::ffi::c_int & 1 as ::core::ffi::c_int)
            << 15 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn BlueBrinstarFaceBlock_Func_1(game: &mut Game) {
    if ram_word!(game, door_transition_flag_enemies) == 0 && {
        let ref mut fresh0 =
            *((&raw mut game.g_ram as *mut uint8).offset(0x1794) as *mut uint16).offset(2);
        *fresh0 = (*fresh0).wrapping_sub(1);
        *fresh0 == 0
    } {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1794) as *mut uint16).offset(2) =
            16 as uint16;
        let mut v0: uint16 =
            *((&raw mut game.g_ram as *mut uint8).offset(0x1794) as *mut uint16).offset(0);
        let mut v1: uint16 = (8u16).wrapping_mul(
            *((&raw mut game.g_ram as *mut uint8).offset(0x1794) as *mut uint16).offset(1),
        );
        let mut n: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
        loop {
            *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(
                ((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) + 137 as ::core::ffi::c_int)
                    as isize,
            ) = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa8e7cc)) as *const uint8
                as *mut uint16)
                .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
            v1 = (v1).wrapping_add(2u16);
            v0 = (v0).wrapping_add(2u16);
            n -= 1;
            if (n == 0) {
                break;
            }
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x1794) as *mut uint16).offset(1) =
            (*(((&raw mut game.g_ram as *mut uint8).offset(0x1794) as *mut uint16).offset(1)
                as *mut uint16 as *mut uint8)
                .offset(0) as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                & 7 as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn BlueBrinstarFaceBlock_Main(game: &mut Game) {
    if (ram_word!(game, collected_items) as ::core::ffi::c_int & 4 as ::core::ffi::c_int) != 0i32 {
        (*((&raw mut game.g_ram as *mut uint8).offset(0x178c) as *mut LongPtr)).addr =
            ((0xa8e86ei32) as u16);
        let v1: *mut EnemyData =
            gEnemyData(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x1794) as *mut uint16).offset(0) =
            ((16 as ::core::ffi::c_int * (*v1).palette_index as ::core::ffi::c_int
                & 0xff00 as ::core::ffi::c_int)
                >> 8 as ::core::ffi::c_int) as uint16;
        if (*v1).ai_var_A == 0 {
            let SamusEnemyDelta_Y: uint16 =
                GetSamusEnemyDelta_Y(ram_word!(game, cur_enemy_index), game);
            if ((Abs16(SamusEnemyDelta_Y)) as i16).wrapping_sub(((*v1).parameter_1) as i16) < 0i16 {
                let SamusEnemyDelta_X: uint16 =
                    GetSamusEnemyDelta_X(ram_word!(game, cur_enemy_index), game);
                (*v1).ai_var_B = SamusEnemyDelta_X;
                if ((Abs16(SamusEnemyDelta_X)) as i16).wrapping_sub(((*v1).parameter_1) as i16)
                    < 0i16
                    && (*v1).ai_var_B as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                        != (*v1).parameter_2 as ::core::ffi::c_int
                {
                    let mut v4: uint16 = addr_kBlueBrinstarFaceBlock_Ilist_E80C as uint16;
                    if ((*v1).ai_var_B as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) == 0i32
                    {
                        v4 = addr_kBlueBrinstarFaceBlock_Ilist_E81A as uint16;
                    }
                    (*v1).current_instruction = v4;
                    (*v1).instruction_timer = 1 as uint16;
                    (*v1).ai_var_A = 1 as uint16;
                    *((&raw mut game.g_ram as *mut uint8).offset(0x1794) as *mut uint16)
                        .offset(2) = 16 as uint16;
                }
            }
        }
    }
}
pub unsafe fn BlueBrinstarFaceBlock_Shot(game: &mut Game) {
    let ref mut fresh1 = *((&raw mut game.g_ram as *mut uint8).offset(0xc04) as *mut uint16)
        .offset(ram_word!(game, collision_detection_index) as isize);
    *fresh1 = (*fresh1 as ::core::ffi::c_int & !(0x10 as ::core::ffi::c_int)) as uint16;
}
pub unsafe fn KiHunter_Init(game: &mut Game) {
    let E: *mut Enemy_KiHunter =
        Get_KiHunter(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_DisableSamusColl) as u16);
    (*E).khr_var_14 = 0 as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
    (*E).base.current_instruction = addr_kKiHunter_Ilist_E9FA as uint16;
    (*E).khr_var_F = 0 as uint16;
    (*E).khr_var_A = ((0xa8f268i32) as u16);
    (*E).khr_var_08 = 0 as uint16;
    (*E).khr_var_09 = 1 as uint16;
    (*E).khr_var_06 = 0 as uint16;
    (*E).khr_var_07 = -(1 as ::core::ffi::c_int) as uint16;
    let v1: uint16 = ((*E).base.y_pos).wrapping_sub(16u16);
    (*E).khr_var_0A = v1;
    (*E).khr_var_0B = (v1).wrapping_add(32u16);
    (*E).khr_var_0C = (*E).base.x_pos;
    (*E).khr_var_0D = (*E).base.y_pos;
    if ((*E).khr_parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        (*E).khr_var_14 = 1 as uint16;
        (*E).khr_var_A = ((0xa8f55ai32) as u16);
        (*E).khr_var_08 = 0 as uint16;
        (*E).khr_var_09 = 1 as uint16;
    }
}
pub unsafe fn KiHunterWings_Init(game: &mut Game) {
    let E: *mut Enemy_KiHunter =
        Get_KiHunter(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_DisableSamusColl) as u16);
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
    (*E).base.current_instruction = addr_kKiHunter_Ilist_EA4E as uint16;
    let v2: ::core::ffi::c_int =
        ram_word!(game, cur_enemy_index) as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    (*E).base.y_pos = *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
        .offset((v2 + 93 as ::core::ffi::c_int) as isize);
    (*E).base.x_pos = *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
        .offset((v2 + 91 as ::core::ffi::c_int) as isize);
    (*E).khr_var_A = ((0xa8f6f3i32) as u16);
    (*E).base.palette_index = *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
        .offset((v2 + 105 as ::core::ffi::c_int) as isize);
    (*E).base.vram_tiles_index = *((&raw mut game.g_ram as *mut uint8).offset(0xe84)
        as *mut uint16)
        .offset((v2 + 106 as ::core::ffi::c_int) as isize);
    if (*((&raw mut game.g_ram as *mut uint8).offset(0xf68) as *mut uint16)
        .offset((v2 + 6 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int)
        != 0i32
    {
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Deleted) as u16);
    }
}
pub unsafe fn CallKiHunterFunc(ea: uint32, k: uint16, game: &mut Game) {
    match ea {
        11072104 => {
            KiHunter_Func_1(k, game);
        }
        11072440 => {
            KiHunter_Func_2(k, game);
        }
        11072749 => {
            KiHunter_Func_3(k, game);
        }
        11072858 => {
            KiHunter_Func_4(k, game);
        }
        11072907 => {
            KiHunter_Func_5(k, game);
        }
        11073008 => {
            KiHunter_Func_6(k, game);
        }
        11073163 => {
            KiHunter_Func_7(k, game);
        }
        11073203 => {
            KiHunter_Func_8(k, game);
        }
        11073267 => {
            KiHunter_Func_9(k, game);
        }
        11073487 => {
            KiHunter_Func_10(k, game);
        }
        11072995 => (),
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn KiHunter_Main(game: &mut Game) {
    let E: *mut Enemy_KiHunter =
        Get_KiHunter(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    CallKiHunterFunc(
        (((*E).khr_var_A) as u32) | (0xa80000u32),
        ram_word!(game, cur_enemy_index),
        game,
    );
}
pub unsafe fn KiHunterWings_Main(game: &mut Game) {
    let E: *mut Enemy_KiHunter =
        Get_KiHunter(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    CallKiHunterFunc(
        (((*E).khr_var_A) as u32) | (0xa80000u32),
        ram_word!(game, cur_enemy_index),
        game,
    );
}
pub unsafe fn KiHunter_Func_1(k: uint16, game: &mut Game) {
    let current_block: u64;
    let mut v4: int16 = 0;
    let E: *mut Enemy_KiHunter = Get_KiHunter(k as ::core::ffi::c_int, game);
    if Enemy_MoveDown(
        k,
        (((*E).khr_var_09 as uint32) << 16 as ::core::ffi::c_int | (*E).khr_var_08 as uint32)
            as int32,
        game,
    ) != 0
    {
        v4 = -((*E).khr_var_09 as ::core::ffi::c_int) as int16;
        current_block = 8515828400728868193;
    } else {
        let y_pos: uint16 = (*E).base.y_pos;
        if ((y_pos) as i16).wrapping_sub(((*E).khr_var_0A) as i16) < 0i16 {
            v4 = 1 as int16;
            current_block = 8515828400728868193;
        } else if ((y_pos) as i16).wrapping_sub(((*E).khr_var_0B) as i16) < 0i16 {
            current_block = 3218563456939568490;
        } else {
            v4 = -(1 as ::core::ffi::c_int) as int16;
            current_block = 8515828400728868193;
        }
    }
    if current_block == 8515828400728868193 {
        (*E).khr_var_09 = v4 as uint16;
    }
    if Enemy_MoveRight_IgnoreSlopes(
        k,
        (((*E).khr_var_07 as uint32) << 16 as ::core::ffi::c_int | (*E).khr_var_06 as uint32)
            as int32,
        game,
    ) != 0
    {
        let mut v5: uint16 = 0 as uint16;
        let v6: bool =
            (-((*E).khr_var_07 as ::core::ffi::c_int) & 0x8000 as ::core::ffi::c_int) != 0i32;
        (*E).khr_var_07 = -((*E).khr_var_07 as ::core::ffi::c_int) as uint16;
        if !v6 {
            v5 = 4 as uint16;
        }
        let v7: ::core::ffi::c_int = v5 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        (*E).base.current_instruction = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa8f3b0))
            as *const uint8 as *mut uint16)
            .offset(v7 as isize);
        let E1: *mut Enemy_KiHunter =
            Get_KiHunter(k as ::core::ffi::c_int + 64 as ::core::ffi::c_int, game);
        (*E1).base.current_instruction = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa8f3b0))
            as *const uint8 as *mut uint16)
            .offset((v7 + 1 as ::core::ffi::c_int) as isize);
        (*E).base.instruction_timer = 1 as uint16;
        (*E1).base.instruction_timer = 1 as uint16;
    }
    EnemyFunc_C8AD(k, game);
    let r18: uint16 = (ram_word!(game, samus_x_pos)).wrapping_sub((*E).base.x_pos);
    let r20: uint16 = abs16(r18);
    if ((r20) as i16).wrapping_sub((g_word_A8F180) as i16) < 0i16
        && ram_word!(game, samus_y_pos) as ::core::ffi::c_int
            - (*E).base.y_pos as ::core::ffi::c_int
            - 32 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
    {
        let r24: uint16 = (ram_word!(game, samus_y_pos)).wrapping_sub((*E).base.y_pos);
        let mut v9: uint16 = 0;
        if (r18 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
            (*E).khr_var_00 = -(2 as ::core::ffi::c_int) as uint16;
            (*E).khr_var_02 = 0 as uint16;
            (*E).khr_var_03 = 0 as uint16;
            (*E).khr_var_04 = -(1 as ::core::ffi::c_int) as uint16;
            (*E).khr_var_07 = -(1 as ::core::ffi::c_int) as uint16;
            (*E).khr_var_05 = -(8192 as ::core::ffi::c_int) as uint16;
            (*E).khr_var_F = 255 as uint16;
            (*E).khr_var_0E = 240 as uint16;
            v9 = 0 as uint16;
        } else {
            (*E).khr_var_00 = 2 as uint16;
            (*E).khr_var_02 = 0 as uint16;
            (*E).khr_var_03 = 0 as uint16;
            (*E).khr_var_04 = 0 as uint16;
            (*E).khr_var_05 = 0x2000 as uint16;
            (*E).khr_var_F = 128 as uint16;
            (*E).khr_var_07 = 1 as uint16;
            (*E).khr_var_0E = 144 as uint16;
            v9 = 4 as uint16;
        }
        let v10: ::core::ffi::c_int = v9 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        (*E).base.current_instruction = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa8f3b0))
            as *const uint8 as *mut uint16)
            .offset(v10 as isize);
        let E1_0: *mut Enemy_KiHunter =
            Get_KiHunter(k as ::core::ffi::c_int + 64 as ::core::ffi::c_int, game);
        (*E1_0).base.current_instruction = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa8f3b0))
            as *const uint8 as *mut uint16)
            .offset((v10 + 1 as ::core::ffi::c_int) as isize);
        (*E).base.instruction_timer = 1 as uint16;
        (*E1_0).base.instruction_timer = 1 as uint16;
        (*E).khr_var_B = (r18).wrapping_add((*E).base.x_pos);
        (*E).khr_var_C = (*E).base.y_pos;
        (*E).khr_var_A = ((0xa8f3b8i32) as u16);
        (*E).khr_var_12 = r24;
        (*E).khr_var_11 = r20;
        (*E).khr_var_10 = 0 as uint16;
    }
}
pub unsafe fn KiHunter_Func_2(k: uint16, game: &mut Game) {
    let mut current_block: u64;
    let E: *mut Enemy_KiHunter = Get_KiHunter(k as ::core::ffi::c_int, game);
    if ((*E).khr_var_04 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) == 0i32 {
        if (((*E).khr_var_F) as i16).wrapping_sub(((*E).khr_var_0E) as i16) < 0i16 {
            current_block = 17315748505249742599;
        } else {
            current_block = 17179679302217393232;
        }
    } else if (((*E).khr_var_F) as i16).wrapping_sub(((*E).khr_var_0E) as i16) >= 0i16 {
        current_block = 17315748505249742599;
    } else {
        current_block = 17179679302217393232;
    }
    if current_block == 17179679302217393232 {
        if (*E).khr_var_10 == 0 {
            (*E).khr_var_10 = 1 as uint16;
            let mut v2: uint16 = addr_kKiHunter_Ilist_EA32 as uint16;
            if ((*E).khr_var_04 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
                v2 = addr_kKiHunter_Ilist_EA08 as uint16;
            }
            (*E).base.current_instruction = v2;
            (*E).base.instruction_timer = 1 as uint16;
        }
    }
    if ((*E).khr_var_04 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) == 0i32 {
        AddToHiLo(
            &raw mut (*E).khr_var_02,
            &raw mut (*E).khr_var_03,
            ((*E).khr_var_04 as uint32) << 16 as ::core::ffi::c_int | (*E).khr_var_05 as uint32,
        );
        if (((*E).khr_var_02) as i16).wrapping_sub(((*E).khr_var_00) as i16) >= 0i16 {
            (*E).khr_var_02 = (*E).khr_var_00;
        }
        (*E).khr_var_F = ((*E).khr_var_F).wrapping_add((*E).khr_var_02);
        if (*E).khr_var_F as ::core::ffi::c_int - 256 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            (*E).khr_var_A = ((0xa8f268i32) as u16);
            return;
        }
    } else {
        AddToHiLo(
            &raw mut (*E).khr_var_02,
            &raw mut (*E).khr_var_03,
            ((*E).khr_var_04 as uint32) << 16 as ::core::ffi::c_int | (*E).khr_var_05 as uint32,
        );
        if (((*E).khr_var_02) as i16).wrapping_sub(((*E).khr_var_00) as i16) < 0i16 {
            (*E).khr_var_02 = (*E).khr_var_00;
        }
        (*E).khr_var_F = ((*E).khr_var_F).wrapping_add((*E).khr_var_02);
        if (*E).khr_var_F as ::core::ffi::c_int - 128 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            (*E).khr_var_A = ((0xa8f268i32) as u16);
            return;
        }
    }
    let mut r20: uint16 = ((*E).khr_var_B as ::core::ffi::c_int
        + CosineMult8bit((*E).khr_var_F, (*E).khr_var_11, game) as ::core::ffi::c_int
        - (*E).base.x_pos as ::core::ffi::c_int) as uint16;
    if Enemy_MoveRight_IgnoreSlopes(k, (r20 as int16 as int32) << 16 as ::core::ffi::c_int, game)
        != 0
    {
        if ((*E).khr_var_04 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) == 0i32 {
            (*E).khr_var_06 = 0 as uint16;
            (*E).khr_var_07 = -(1 as ::core::ffi::c_int) as uint16;
        } else {
            (*E).khr_var_06 = 0 as uint16;
            (*E).khr_var_07 = 1 as uint16;
        }
        current_block = 14751541640656221110;
    } else {
        EnemyFunc_C8AD(k, game);
        r20 = ((*E).khr_var_C as ::core::ffi::c_int
            + SineMult8bit((*E).khr_var_F, (*E).khr_var_12, game) as ::core::ffi::c_int
            - (*E).base.y_pos as ::core::ffi::c_int) as uint16;
        if Enemy_MoveDown(k, (r20 as int16 as int32) << 16 as ::core::ffi::c_int, game) != 0 {
            current_block = 14751541640656221110;
        } else {
            current_block = 7828949454673616476;
        }
    }
    if current_block == 14751541640656221110 {
        (*E).khr_var_A = ((0xa8f4edi32) as u16);
        (*E).khr_var_08 = 0 as uint16;
        (*E).khr_var_09 = -(1 as ::core::ffi::c_int) as uint16;
    };
}
pub unsafe fn KiHunter_Func_3(k: uint16, game: &mut Game) {
    let E: *mut Enemy_KiHunter = Get_KiHunter(k as ::core::ffi::c_int, game);
    if Enemy_MoveRight_IgnoreSlopes(
        k,
        (((*E).khr_var_07 as uint32) << 16 as ::core::ffi::c_int | (*E).khr_var_06 as uint32)
            as int32,
        game,
    ) != 0u8
        || {
            EnemyFunc_C8AD(k, game);
            Enemy_MoveDown(
                k,
                (((*E).khr_var_09 as uint32) << 16 as ::core::ffi::c_int
                    | (*E).khr_var_08 as uint32) as int32,
                game,
            ) != 0u8
        }
        || (((*E).base.y_pos) as i16).wrapping_sub(((*E).khr_var_0D) as i16) < 0i16
    {
        (*E).khr_var_A = ((0xa8f268i32) as u16);
    }
}
pub unsafe fn KiHunter_Instr_1(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    let mut result: uint16 = addr_kKiHunter_Ilist_E9FA as uint16;
    let mut r18: uint16 = addr_kKiHunter_Ilist_EA4E as uint16;
    let E: *mut Enemy_KiHunter = Get_KiHunter(k as ::core::ffi::c_int, game);
    if ((*E).khr_var_07 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) == 0i32 {
        result = addr_kKiHunter_Ilist_EA24 as uint16;
        r18 = addr_kKiHunter_Ilist_EA5E as uint16;
    }
    (*E).base.current_instruction = r18;
    (*E).base.instruction_timer = 1 as uint16;
    let E1: *mut Enemy_KiHunter =
        Get_KiHunter(k as ::core::ffi::c_int + 64 as ::core::ffi::c_int, game);
    if (*E1).khr_var_A as ::core::ffi::c_int
        == 0xa8f6f3 as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
    {
        (*E1).base.current_instruction = r18;
        (*E1).base.instruction_timer = 1 as uint16;
    }
    result as uintptr_t as *const uint16
}
pub unsafe fn KiHunter_Func_4(k: uint16, game: &mut Game) {
    let E: *mut Enemy_KiHunter = Get_KiHunter(k as ::core::ffi::c_int, game);
    if Enemy_MoveDown(
        k,
        (((*E).khr_var_09 as uint32) << 16 as ::core::ffi::c_int | (*E).khr_var_08 as uint32)
            as int32,
        game,
    ) != 0
    {
        (*E).khr_var_A = ((0xa8f58bi32) as u16);
    } else {
        AddToHiLo(
            &raw mut (*E).khr_var_09,
            &raw mut (*E).khr_var_08,
            (g_word_A8F184 as uint32) << 16 as ::core::ffi::c_int | g_word_A8F182 as uint32,
        );
    };
}
pub unsafe fn KiHunter_Func_5(k: uint16, game: &mut Game) {
    let E: *mut Enemy_KiHunter = Get_KiHunter(k as ::core::ffi::c_int, game);
    (*E).khr_var_A = addr_locret_A8F5E3 as uint16;
    (*E).khr_var_08 = 0 as uint16;
    (*E).khr_var_09 = ((ram_word!(game, random_number) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int)
        - 8 as ::core::ffi::c_int) as uint16;
    if (((*E).base.x_pos) as i16).wrapping_sub((ram_word!(game, samus_x_pos)) as i16) >= 0i16 {
        (*E).khr_var_06 = 0 as uint16;
        (*E).khr_var_07 = -(2 as ::core::ffi::c_int) as uint16;
        (*E).base.current_instruction = addr_kKiHunter_Ilist_EA8A as uint16;
    } else {
        (*E).khr_var_06 = 0 as uint16;
        (*E).khr_var_07 = 2 as uint16;
        (*E).base.current_instruction = addr_kKiHunter_Ilist_EAA6 as uint16;
    }
    (*E).base.instruction_timer = 1 as uint16;
}
pub unsafe fn KiHunter_Instr_2(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    (*Get_KiHunter(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game)).khr_var_A =
        ((0xa8f5f0i32) as u16);
    jp
}
pub unsafe fn KiHunter_Func_6(k: uint16, game: &mut Game) {
    let E: *mut Enemy_KiHunter = Get_KiHunter(k as ::core::ffi::c_int, game);
    if Enemy_MoveDown(
        k,
        (((*E).khr_var_09 as uint32) << 16 as ::core::ffi::c_int | (*E).khr_var_08 as uint32)
            as int32,
        game,
    ) != 0
    {
        if ((*E).khr_var_09 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
            (*E).khr_var_09 = 1 as uint16;
        } else {
            (*E).khr_var_08 = 0 as uint16;
            (*E).khr_var_09 = -(4 as ::core::ffi::c_int) as uint16;
            (*E).khr_var_A = addr_locret_A8F5E3 as uint16;
            (*E).khr_var_0F = 12 as uint16;
            let mut v4: uint16 = addr_kKiHunter_Ilist_EAC2 as uint16;
            if (*E).base.current_instruction as ::core::ffi::c_int + 0x155a as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                == 0
            {
                v4 = addr_kKiHunter_Ilist_EADA as uint16;
            }
            (*E).base.current_instruction = v4;
            (*E).base.instruction_timer = 1 as uint16;
        }
    } else if Enemy_MoveRight_IgnoreSlopes(
        k,
        (((*E).khr_var_07 as uint32) << 16 as ::core::ffi::c_int | (*E).khr_var_06 as uint32)
            as int32,
        game,
    ) != 0
    {
        (*E).khr_var_07 = -((*E).khr_var_07 as ::core::ffi::c_int) as uint16;
    } else {
        EnemyFunc_C8AD(k, game);
        AddToHiLo(
            &raw mut (*E).khr_var_09,
            &raw mut (*E).khr_var_08,
            (g_word_A8F184 as uint32) << 16 as ::core::ffi::c_int | g_word_A8F182 as uint32,
        );
    };
}
pub unsafe fn KiHunter_Instr_3(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    (*Get_KiHunter(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game)).khr_var_A =
        ((0xa8f68bi32) as u16);
    jp
}
pub unsafe fn KiHunter_Func_7(k: uint16, game: &mut Game) {
    let E: *mut Enemy_KiHunter = Get_KiHunter(k as ::core::ffi::c_int, game);
    let v2: uint16 = ((*E).khr_var_0F).wrapping_sub(1u16);
    (*E).khr_var_0F = v2;
    if v2 == 0 {
        let mut v3: uint16 = ((0xa8f58bi32) as u16);
        let v4: uint16 = abs16(((*E).base.x_pos).wrapping_sub(ram_word!(game, samus_x_pos)));
        if v4 as ::core::ffi::c_int - 96 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
            v3 = ((0xa8f6b3i32) as u16);
        }
        (*E).khr_var_A = v3;
    }
}
pub unsafe fn KiHunter_Func_8(k: uint16, game: &mut Game) {
    let mut v1: uint16 = addr_kKiHunter_Ilist_EAF2 as uint16;
    let E: *mut Enemy_KiHunter = Get_KiHunter(k as ::core::ffi::c_int, game);
    if (((*E).base.x_pos) as i16).wrapping_sub((ram_word!(game, samus_x_pos)) as i16) < 0i16 {
        v1 = addr_kKiHunter_Ilist_EB10 as uint16;
    }
    (*E).base.current_instruction = v1;
    (*E).base.instruction_timer = 1 as uint16;
    (*E).khr_var_A = addr_locret_A8F5E3 as uint16;
}
pub unsafe fn KiHunter_Instr_4(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    sub_A8F6DC(k, addr_loc_A8CF18 as uint16, game);
    jp
}
pub unsafe fn KiHunter_Instr_5(k: uint16, jp: *const uint16, game: &mut Game) -> *const uint16 {
    sub_A8F6DC(k, addr_loc_A8CF26 as uint16, game);
    jp
}
pub unsafe fn sub_A8F6DC(k: uint16, j: uint16, game: &mut Game) {
    QueueSfx2_Max6(0x4c as uint16, game);
    SpawnEprojWithGfx(0 as uint16, ram_word!(game, cur_enemy_index), j, game);
    (*Get_KiHunter(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game)).khr_var_0F =
        24 as uint16;
}
pub unsafe fn KiHunter_Func_9(k: uint16, game: &mut Game) {
    let v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let E: *mut Enemy_KiHunter = Get_KiHunter(k as ::core::ffi::c_int, game);
    (*E).base.x_pos = *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
        .offset((v1 + 91 as ::core::ffi::c_int) as isize);
    (*E).base.y_pos = *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
        .offset((v1 + 93 as ::core::ffi::c_int) as isize);
}
pub unsafe fn KiHunter_Shot(game: &mut Game) {
    NormalEnemyShotAi(game);
    let old_cur_enemy_index: uint16 = ram_word!(game, cur_enemy_index);
    let EK: *mut Enemy_KiHunter =
        Get_KiHunter(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    let E1: *mut Enemy_KiHunter = Get_KiHunter(
        ram_word!(game, cur_enemy_index) as ::core::ffi::c_int + 64 as ::core::ffi::c_int,
        game,
    );
    let health: uint16 = (*EK).base.health;
    if health != 0 {
        if health == (*E1).khr_parameter_1
            || ((health) as i16).wrapping_sub(((*E1).khr_parameter_1) as i16) < 0i16
        {
            if (*EK).khr_var_14 == 0 {
                (*EK).khr_var_14 = 1 as uint16;
                (*EK).khr_var_A = ((0xa8f55ai32) as u16);
                (*EK).khr_var_08 = 0 as uint16;
                (*EK).khr_var_09 = 1 as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) =
                    (old_cur_enemy_index).wrapping_add(64u16);
                if (*E1).khr_var_A as ::core::ffi::c_int
                    != 0xa8f7cf as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
                {
                    (*E1).khr_var_07 = (*E1).base.y_pos;
                    (*E1).khr_var_08 = (*E1).base.x_pos;
                    KiHunter_Func_17(game);
                    KiHunter_Func_12(game);
                    KiHunter_Func_13(game);
                    (*E1).khr_var_F = -(8192 as ::core::ffi::c_int) as uint16;
                    (*E1).khr_var_A = ((0xa8f7cfi32) as u16);
                    (*E1).khr_var_00 = ((0xa8f7dbi32) as u16);
                    (*E1).khr_var_06 = ((*E1).khr_var_07).wrapping_sub((*E1).khr_var_0B);
                    (*E1).khr_var_05 = (*E1).base.x_pos;
                    (*E1).khr_var_B = (*E1).khr_var_0A;
                    (*E1).base.current_instruction = addr_kKiHunter_Ilist_EA7E as uint16;
                    (*E1).base.spritemap_pointer = addr_kSpritemap_Nothing_A8 as uint16;
                    (*E1).base.instruction_timer = 1 as uint16;
                    (*E1).base.timer = 0 as uint16;
                    (*E1).base.properties =
                        ((*E1).base.properties) | ((kEnemyProps_ProcessedOffscreen) as u16);
                }
                *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) =
                    old_cur_enemy_index;
            }
        } else {
            (*E1).base.ai_handler_bits = (*EK).base.ai_handler_bits;
            (*E1).base.frozen_timer = (*EK).base.frozen_timer;
            (*E1).base.invincibility_timer = (*EK).base.invincibility_timer;
            (*E1).base.flash_timer = (*EK).base.flash_timer;
        }
    } else {
        (*E1).base.properties = 512 as uint16;
    };
}
pub unsafe fn CallKiHunterBFunc(ea: uint32, game: &mut Game) {
    match ea {
        11073499 => {
            KiHunter_Func_11(game);
        }
        11073709 => {
            KiHunter_Func_14(game);
        }
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn KiHunter_Func_10(k: uint16, game: &mut Game) {
    let r18: uint16 =
        (*Get_KiHunter(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game)).khr_var_00;
    CallKiHunterBFunc(((r18) as u32) | (0xa80000u32), game);
}
pub unsafe fn KiHunter_Func_11(game: &mut Game) {
    let mut v2: int16 = 0;
    let E: *mut Enemy_KiHunter =
        Get_KiHunter(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).khr_var_F = ((*E).khr_var_F).wrapping_add(
        *(((game.g_rom.offset(crate::rtl::lorom_offset(0xa2838f)) as *const uint8 as *mut uint16)
            .offset(2) as *mut uint16 as *mut uint8)
            .offset(
                (8 as ::core::ffi::c_int
                    * *(&raw mut (*E).khr_var_B as *mut uint8).offset(
                        (::core::mem::size_of::<uint16>())
                            .wrapping_div(::core::mem::size_of::<uint8>())
                            .wrapping_sub(1_usize) as isize,
                    ) as ::core::ffi::c_int) as isize,
            )
            .offset(1) as *mut uint16),
    );
    (*E).base.y_pos = ((*E).khr_var_06 as ::core::ffi::c_int
        + SineMult8bit(
            *(&raw mut (*E).khr_var_F as *mut uint8).offset(
                (::core::mem::size_of::<uint16>())
                    .wrapping_div(::core::mem::size_of::<uint8>())
                    .wrapping_sub(1_usize) as isize,
            ) as uint16,
            g_byte_A8F186 as uint16,
            game,
        ) as ::core::ffi::c_int
        - (*E).khr_var_04 as ::core::ffi::c_int) as uint16;
    (*E).base.x_pos = ((*E).khr_var_05 as ::core::ffi::c_int
        + CosineMult8bit(
            *(&raw mut (*E).khr_var_F as *mut uint8).offset(
                (::core::mem::size_of::<uint16>())
                    .wrapping_div(::core::mem::size_of::<uint8>())
                    .wrapping_sub(1_usize) as isize,
            ) as uint16,
            g_byte_A8F186 as uint16,
            game,
        ) as ::core::ffi::c_int
        - (*E).khr_var_03 as ::core::ffi::c_int) as uint16;
    if (*E).khr_var_F as ::core::ffi::c_int + 0x4000 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        KiHunter_Func_16(ram_word!(game, cur_enemy_index), game);
    } else {
        v2 = (((*E).khr_var_B) as i16).wrapping_sub(384i16);
        if v2 < 0i16 {
            v2 = 256 as int16;
        }
        (*E).khr_var_B = v2 as uint16;
    };
}
pub unsafe fn KiHunter_Func_12(game: &mut Game) {
    let v1: uint16 = CosineMult8bit(0xe0 as uint16, g_byte_A8F186 as uint16, game);
    let E: *mut Enemy_KiHunter =
        Get_KiHunter(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).khr_var_03 = v1;
    (*E).khr_var_04 = SineMult8bit(0xe0 as uint16, g_byte_A8F186 as uint16, game);
}
pub unsafe fn KiHunter_Func_13(game: &mut Game) {
    let v1: uint16 = CosineMult8bit(0xa0 as uint16, g_byte_A8F186 as uint16, game);
    let E: *mut Enemy_KiHunter =
        Get_KiHunter(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).khr_var_01 = v1;
    (*E).khr_var_02 = SineMult8bit(0xa0 as uint16, g_byte_A8F186 as uint16, game);
}
pub unsafe fn KiHunter_Func_14(game: &mut Game) {
    let mut khr_var_07: int16 = 0;
    let E: *mut Enemy_KiHunter =
        Get_KiHunter(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).khr_var_F = ((*E).khr_var_F).wrapping_add(
        *((game.g_rom.offset(crate::rtl::lorom_offset(0xa2838f)) as *const uint8 as *mut uint16
            as *mut uint8)
            .offset(
                (8 as ::core::ffi::c_int
                    * *(&raw mut (*E).khr_var_B as *mut uint8).offset(
                        (::core::mem::size_of::<uint16>())
                            .wrapping_div(::core::mem::size_of::<uint8>())
                            .wrapping_sub(1_usize) as isize,
                    ) as ::core::ffi::c_int) as isize,
            )
            .offset(1) as *mut uint16),
    );
    let r20: uint16 = ((*E).khr_var_06 as ::core::ffi::c_int
        + SineMult8bit(
            *(&raw mut (*E).khr_var_F as *mut uint8).offset(
                (::core::mem::size_of::<uint16>())
                    .wrapping_div(::core::mem::size_of::<uint8>())
                    .wrapping_sub(1_usize) as isize,
            ) as uint16,
            g_byte_A8F186 as uint16,
            game,
        ) as ::core::ffi::c_int
        - (*E).khr_var_02 as ::core::ffi::c_int
        - (*E).base.y_pos as ::core::ffi::c_int) as uint16;
    if Enemy_MoveDown(
        ram_word!(game, cur_enemy_index),
        (r20 as int16 as int32) << 16 as ::core::ffi::c_int,
        game,
    ) != 0
    {
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Deleted) as u16);
        (*E).base.x_pos = (*E).khr_var_08;
        khr_var_07 = (*E).khr_var_07 as int16;
        (*E).base.y_pos = khr_var_07 as uint16;
    } else {
        (*E).base.x_pos = ((*E).khr_var_05 as ::core::ffi::c_int
            + CosineMult8bit(
                *(&raw mut (*E).khr_var_F as *mut uint8).offset(
                    (::core::mem::size_of::<uint16>())
                        .wrapping_div(::core::mem::size_of::<uint8>())
                        .wrapping_sub(1_usize) as isize,
                ) as uint16,
                g_byte_A8F186 as uint16,
                game,
            ) as ::core::ffi::c_int
            - (*E).khr_var_01 as ::core::ffi::c_int) as uint16;
        if (*E).khr_var_F as ::core::ffi::c_int + 0x4000 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            KiHunter_Func_15(ram_word!(game, cur_enemy_index), game);
            return;
        }
        khr_var_07 = (((*E).khr_var_B) as i16).wrapping_sub(384i16);
        if khr_var_07 < 0i16 {
            khr_var_07 = 256 as int16;
        }
    }
    (*Get_KiHunter(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game)).khr_var_B =
        khr_var_07 as uint16;
}
pub unsafe fn KiHunter_Func_15(k: uint16, game: &mut Game) {
    let E: *mut Enemy_KiHunter = Get_KiHunter(k as ::core::ffi::c_int, game);
    (*E).khr_var_00 = ((0xa8f7dbi32) as u16);
    (*E).khr_var_B = (*E).khr_var_0A;
    (*E).khr_var_F = -(8192 as ::core::ffi::c_int) as uint16;
    (*E).khr_var_05 = (*E).base.x_pos;
    (*E).khr_var_06 = (*E).base.y_pos;
}
pub unsafe fn KiHunter_Func_16(k: uint16, game: &mut Game) {
    let E: *mut Enemy_KiHunter = Get_KiHunter(k as ::core::ffi::c_int, game);
    (*E).khr_var_00 = ((0xa8f8adi32) as u16);
    (*E).khr_var_B = (*E).khr_var_0A;
    (*E).khr_var_F = -(24576 as ::core::ffi::c_int) as uint16;
    (*E).khr_var_05 = (*E).base.x_pos;
    (*E).khr_var_06 = (*E).base.y_pos;
}
pub unsafe fn KiHunter_Func_17(game: &mut Game) {
    let mut v5: uint16 = 0;
    let E: *mut Enemy_KiHunter =
        Get_KiHunter(ram_word!(game, cur_enemy_index) as ::core::ffi::c_int, game);
    (*E).khr_var_0A = 0 as uint16;
    (*E).khr_var_B = 0 as uint16;
    loop {
        let mut v3: uint16 = ((*E).khr_var_0A).wrapping_add(384u16);
        (*E).khr_var_0A = v3;
        v3 = (v3 as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as uint16;
        v5 = (*((game.g_rom.offset(crate::rtl::lorom_offset(0xa2838f)) as *const uint8
            as *mut uint16 as *mut uint8)
            .offset((8 as ::core::ffi::c_int * v3 as ::core::ffi::c_int) as isize)
            .offset(1) as *mut uint16))
            .wrapping_add((*E).khr_var_D);
        (*E).khr_var_D = v5;
        if (v5 as ::core::ffi::c_int - 0x2000 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            == 0)
        {
            break;
        }
    }
}
