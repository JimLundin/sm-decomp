use crate::c_library::printf;
use crate::game::callback_tables::callback_table;
pub use crate::game::constants::bank_86::*;
pub use crate::game::constants::bank_a6::*;
pub use crate::game::constants::bank_a9::*;
pub use crate::game::constants::enemy_properties::*;
use crate::game::enemy_instruction;
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
use crate::ram_map::ram_ptr;
use crate::ram_map::set_ram_words;
use crate::ram_map::{ram_word, set_ram_word, update_ram_word};
use crate::rtl::abs16;
use crate::rtl::AddToHiLo;
use crate::rtl::{mov24, Mult8x8, RomPtr, SnesDivide, Unreachable};
use crate::set_fields;
pub use crate::types::Consts_A6;
pub use crate::types::Consts_A9;
pub use crate::types::{Consts_86, ExtraEnemyRam7800, ExtraEnemyRam8000, Ram7800_Kraid};
pub use crate::types::{
    EnemyBase, EnemyData, EnemyProps, EnemySpawnData, Func_V, LongPtr, OamEnt, PairU16, Point16U,
    Rect16U, VoidP, VramWriteEntry,
};
use ::core::ffi::{c_char, c_long};
use ::core::ptr::{null, null_mut};
enemy_layout! {
    Enemy_Ridley {
        base: EnemyBase,
        words: [
            ridley_var_A, ridley_var_B, ridley_var_C, ridley_var_D, ridley_var_E,
            ridley_var_F, ridley_parameter_1, ridley_parameter_2,
        ],
        pad1: [u8; 26696],
        words: [
            ridley_var_00, ridley_var_01, ridley_var_02, ridley_var_03, ridley_var_04,
            ridley_var_05, ridley_var_06, ridley_var_07, ridley_var_08, ridley_var_09,
            ridley_var_0A, ridley_var_0B, ridley_var_0C, ridley_var_0D, ridley_var_0E,
            ridley_var_0F, ridley_var_10, ridley_var_11, ridley_var_12, ridley_var_13,
            ridley_var_14, ridley_var_15, ridley_var_16, ridley_var_17, ridley_var_18,
            ridley_var_19, ridley_var_1A, ridley_var_1B, ridley_var_1C, ridley_var_1D,
            ridley_var_1E, ridley_var_1F,
        ],
        pad2: [u8; 1984],
        words: [
            ridley_var_20, ridley_var_21, ridley_var_22, ridley_var_23, ridley_var_24,
            ridley_var_25, ridley_var_26, ridley_var_27, ridley_var_28, ridley_var_29,
            ridley_var_2A, ridley_var_2B, ridley_var_2C, ridley_var_2D, ridley_var_2E,
            ridley_var_2F, ridley_var_30, ridley_var_31, ridley_var_32, ridley_var_33,
            ridley_var_34, ridley_var_35, ridley_var_36, ridley_var_37, ridley_var_38,
            ridley_var_39, ridley_var_3A, ridley_var_3B, ridley_var_3C, ridley_var_3D,
            ridley_var_3E, ridley_var_3F,
        ],
        pad3: [u8; 1984],
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
        pad1: [u8; 26696],
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
        pad1: [u8; 26696],
        words: [
            nwe_var_00, nwe_var_01, nwe_var_02, nwe_var_03, nwe_var_04, nwe_var_05,
            nwe_var_06, nwe_var_07, nwe_var_08, nwe_var_09, nwe_var_0A, nwe_var_0B,
            nwe_var_0C, nwe_var_0D, nwe_var_0E, nwe_var_0F, nwe_var_10, nwe_var_11,
            nwe_var_12, nwe_var_13, nwe_var_14, nwe_var_15, nwe_var_16, nwe_var_17,
            nwe_var_18, nwe_var_19, nwe_var_1A, nwe_var_1B, nwe_var_1C, nwe_var_1D,
            nwe_var_1E, nwe_var_1F,
        ],
        pad2: [u8; 1984],
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
        pad1: [u8; 26696],
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
        pad1: [u8; 26696],
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
        pad1: [u8; 26696],
        words: [
            cry_var_00, cry_var_01, cry_var_02, cry_var_03, cry_var_04, cry_var_05,
            cry_var_06, cry_var_07, cry_var_08, cry_var_09, cry_var_0A, cry_var_0B,
            cry_var_0C, cry_var_0D, cry_var_0E, cry_var_0F, cry_var_10, cry_var_11,
            cry_var_12, cry_var_13, cry_var_14, cry_var_15, cry_var_16, cry_var_17,
            cry_var_18, cry_var_19, cry_var_1A, cry_var_1B, cry_var_1C, cry_var_1D,
            cry_var_1E, cry_var_1F,
        ],
        pad2: [u8; 1984],
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

crate::typed_struct! {
    #[derive(Copy, Clone)]
    #[repr(C)]
    pub struct BabyMetroidExecState {
        pub ip, timer: u16;
    }
}
crate::typed_struct! {
    #[derive(Copy, Clone)]
    #[repr(C)]
    pub struct Ram8800_Default {
        pub var_40, var_41, var_42, var_43, var_44, var_45, var_46, var_47, var_48, var_49, var_4A,
            var_4B, var_4C, var_4D, var_4E, var_4F, var_50, var_51, var_52, var_53, var_54, var_55,
            var_56, var_57, var_58, var_59, var_5A, var_5B, var_5C, var_5D, var_5E, var_5F: u16;
    }
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

crate::game::constants::typed_constants! { i32;
    true_0 = 1, false_0 = 0,
}
const RomPtr_A0: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0xa0>;
const RomPtr_A6: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0xa6>;
crate::ram_map::ram_pointer_functions! {
    gOamEnt(v: i32) -> OamEnt = [(0x370i32 + v) as isize];
    gVramWriteEntry(a: i32) -> VramWriteEntry = [(0xd0i32 + a) as isize];
}
crate::ram_map::ram_pointer_functions! {
    gEnemyData(o: i32) -> EnemyData = [0xf78, o as isize];
    gEnemySpawnData(o: i32) -> EnemySpawnData = [0x7000, o as isize];
    gRam8800_Default(o: i32) -> Ram8800_Default = [0x8800, o as isize];
    Get_Boulder(o: i32) -> Enemy_Boulder = [0xf78, o as isize];
    Get_CeresDoor(o: i32) -> Enemy_CeresDoor = [0xf78, o as isize];
    Get_CeresRidley(o: i32) -> Enemy_CeresRidley = [0xf78, o as isize];
    Get_CeresSteam(o: i32) -> Enemy_CeresSteam = [0xf78, o as isize];
    Get_FakeKraid(o: i32) -> Enemy_FakeKraid = [0xf78, o as isize];
    Get_FireGeyser(o: i32) -> Enemy_FireGeyser = [0xf78, o as isize];
    Get_MotherBrain(o: i32) -> Enemy_MotherBrain = [0xf78, o as isize];
    Get_NuclearWaffle(o: i32) -> Enemy_NuclearWaffle = [0xf78, o as isize];
    Get_Ridley(o: i32) -> Enemy_Ridley = [0xf78, o as isize];
    Get_RidleysExplosion(o: i32) -> Enemy_RidleysExplosion = [0xf78, o as isize];
    Get_SpikeyPlatform(o: i32) -> Enemy_SpikeyPlatform = [0xf78, o as isize];
    Get_Zebetites(o: i32) -> Enemy_Zebetites = [0xf78, o as isize];
}
static g_word_A686F1: [u16; 2] = [0x1000, 0x1800];
static g_word_A68DBB: u16 = 5u16;
static g_word_A68DBD: u16 = 0xau16;
static g_word_A68DBF: u16 = 0xfu16;
static g_word_A68DC1: u16 = 0x14u16;
static g_word_A68DC3: u16 = 0x19u16;
static g_word_A68DC5: u16 = 0x1eu16;
static g_word_A68DC7: u16 = 0x23u16;
static g_word_A68DC9: u16 = 0x28u16;
static g_word_A68DCB: u16 = 0x2du16;
static g_word_A68DCD: u16 = 0x32u16;
static g_word_A68DCF: u16 = 0x37u16;
static g_word_A68DD1: u16 = 0x3cu16;
static g_word_A68DD3: u16 = 0x41u16;
static g_word_A68DD5: u16 = 0x46u16;
static g_word_A68DD7: u16 = 0x4bu16;
static g_word_A68DD9: u16 = 0x50u16;
static g_word_A68DDB: u16 = 0x55u16;
static g_word_A68DDD: u16 = 0x5au16;
static g_word_A68DDF: u16 = 0x5fu16;
static g_word_A68DE1: u16 = 0x64u16;
static g_word_A68DE3: u16 = 0x69u16;
static g_word_A68DE5: u16 = 0x6eu16;
static g_word_A68DE7: u16 = 0x18u16;
static g_word_A68DE9: u16 = 0x18u16;
static g_word_A68DEB: u16 = 0x18u16;
static g_word_A68DED: u16 = 0x18u16;
static g_word_A68DEF: u16 = 0x18u16;
static g_word_A68DF1: u16 = 0x18u16;
static g_word_A68DF3: u16 = 0x18u16;
static g_word_A68DF5: u16 = 0x18u16;
static g_word_A68DF7: u16 = 0x18u16;
static g_word_A68DF9: u16 = 0x18u16;
static g_word_A68DFB: u16 = 0x18u16;
static g_word_A68DFD: u16 = 0x18u16;
static g_word_A68DFF: u16 = 0x18u16;
static g_word_A68E01: u16 = 0x18u16;
static g_word_A68E03: u16 = 0x18u16;
static g_word_A68E05: u16 = 0x18u16;
static g_word_A68E07: u16 = 0x18u16;
static g_word_A68E09: u16 = 0x18u16;
static g_word_A68E0B: u16 = 0x14u16;
static g_word_A68E0D: u16 = 0x10u16;
static g_word_A68E0F: u16 = 0xcu16;
static g_word_A68E11: u16 = 8u16;
static g_word_A695F6: [u16; 4] = [0x190, 0xf0, 0xf0, 0x190];
static g_word_A695FE: [u16; 4] = [0xffe8, 0xfff4, 0x18, 0xc];
static g_word_A69606: [u16; 4] = [0x180, 0x100, 0x100, 0x180];
static g_word_A69A48: [u16; 8] = [0xfe00, 0xfb00, 0xfc00, 0xfb00, 0x200, 0xfb00, 0x400, 0xfb00];
static g_word_A6B288: u16 = 8u16;
static g_word_A6BBEB: [u16; 3] = [0x40, 0, 0xd0];
static g_word_A6BC62: [u16; 3] = [0xb0, 0, 0x50];
static g_word_A6C66E: [i16; 20] = [
    -24, -24, -20, 20, 16, -30, 30, -3, 14, -13, -2, 18, -2, -32, -31, 8, -4, -10, 19, 19,
];
static g_word_A6CF54: [i16; 3] = [0x20, 0, -0x20];
static g_byte_A6D61F: [u8; 16] = [
    0x10, 0xf, 0xe, 0xd, 0xc, 0xb, 0xa, 9, 8, 7, 6, 5, 4, 3, 2, 1,
];
static g_byte_A6D712: [u8; 16] = [
    0x10, 0xf, 0xe, 0xd, 0xc, 0xb, 0xa, 9, 8, 7, 6, 5, 4, 3, 2, 1,
];
static g_word_A6D9ED: [u16; 8] = [0xc, 0xe, 0x10, 0x12, 0x1c, 0x20, 0x28, 0x30];
crate::forward_calls! { pub unsafe fn (game: &mut Game);
    Enemy_GrappleReact_NoInteract_A6 => SwitchEnemyAiToMainAi(game);
    Enemy_GrappleReact_CancelBeam_A6 => Enemy_SwitchToFrozenAi(game);
    Enemy_NormalTouchAI_A6 => NormalEnemyTouchAi(game);
    Enemy_NormalShotAI_A6 => NormalEnemyShotAi(game);
    Enemy_NormalFrozenAI_A6 => NormalEnemyFrozenAI(game);
}
pub unsafe fn Boulder_Init(game: &mut Game) {
    let mut boulder_parameter_2_high: i16 = 0;
    let E = Get_Boulder(ram_word!(game, cur_enemy_index) as i32, game);
    set_fields! {[(*E)], 0u16; boulder_var_C, boulder_var_B}
    (*E).boulder_var_D = 2u16;
    (*E).boulder_var_A = ((0xa6879ai32) as u16);
    (*E).boulder_var_04 = (*E).base.x_pos;
    let y_pos: u16 = (*E).base.y_pos;
    (*E).boulder_var_05 = y_pos;
    (*E).boulder_var_03 = y_pos;
    (*E).boulder_var_02 = (*E).base.y_subpos;
    boulder_parameter_2_high =
        *(&raw mut (*E).boulder_parameter_2 as *mut u8).offset(1isize) as i16;
    if *(&raw mut (*E).boulder_parameter_2 as *mut u8).offset(1isize) == 0 {
        boulder_parameter_2_high = 1i16;
        (*E).boulder_var_07 = 1u16;
    }
    (*E).base.y_pos = ((*E).boulder_var_05).wrapping_sub((boulder_parameter_2_high) as u16);
    (*E).boulder_var_00 =
        -(*(&raw mut (*E).boulder_parameter_2 as *mut u8).offset(0) as i32) as u16;
    (*E).boulder_var_06 = *(&raw mut (*E).base.current_instruction as *mut u8).offset(0) as u16;
    (*E).base.current_instruction = addr_kBoulder_Ilist_86A7 as u16;
    let boulder_parameter_1_high: u16 =
        *(&raw mut (*E).boulder_parameter_1 as *mut u8).offset(1isize) as u16;
    (*E).boulder_var_E = boulder_parameter_1_high;
    if boulder_parameter_1_high == 0 {
        (*E).boulder_var_00 = -((*E).boulder_var_00 as i32) as u16;
        (*E).base.current_instruction = addr_kBoulder_Ilist_86CB as u16;
    }
    (*E).boulder_var_01 = 2u16;
    if *(&raw mut (*E).boulder_parameter_1 as *mut u8).offset(0) != 0 {
        (*E).boulder_var_01 = 5u16;
    }
}
crate::game::address_dispatch! {
    pub unsafe fn CallBoulderFunc(ea: u32, game: &mut Game) => ea;
        call (game); default { Unreachable(game); };
    {
        10913690 => Boulder_Func_1; 10913773 => Boulder_Func_2; 10913842 => Boulder_Func_3;
        10913931 => Boulder_Func_4; 10914114 => Boulder_Func_5; 10914300 => [Boulder_Func_6()];
    }
}
pub unsafe fn Boulder_Main(game: &mut Game) {
    let E = Get_Boulder(ram_word!(game, cur_enemy_index) as i32, game);
    CallBoulderFunc((((*E).boulder_var_A) as u32) | (0xa60000u32), game);
}
pub unsafe fn Boulder_Func_1(game: &mut Game) {
    let mut SamusEnemyDelta_Y: i16 =
        GetSamusEnemyDelta_Y(ram_word!(game, cur_enemy_index), game) as i16;
    if SamusEnemyDelta_Y >= 0i16 {
        let E = Get_Boulder(ram_word!(game, cur_enemy_index) as i32, game);
        if (SamusEnemyDelta_Y).wrapping_sub(((*E).boulder_var_06) as i16) < 0i16 {
            if (*E).boulder_var_E != 0 {
                let mut SamusEnemyDelta_X: i16 =
                    GetSamusEnemyDelta_X(ram_word!(game, cur_enemy_index), game) as i16;
                if SamusEnemyDelta_X < 0i16
                    && (SamusEnemyDelta_X).wrapping_sub(((*E).boulder_var_00) as i16) >= 0i16
                {
                    (*E).boulder_var_A = ((0xa687edi32) as u16);
                    if (*E).boulder_var_07 != 0 {
                        (*E).boulder_var_A = ((0xa68942i32) as u16);
                    }
                }
            } else {
                let mut v3: i16 =
                    GetSamusEnemyDelta_X(ram_word!(game, cur_enemy_index), game) as i16;
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
    let E = Get_Boulder(ram_word!(game, cur_enemy_index) as i32, game);
    Boulder_Func_9(
        (8u16).wrapping_mul((*(&raw mut (*E).boulder_var_C as *mut u8).offset(1isize)) as u16),
        game,
    );
    if (((*E).base.y_pos) as i16).wrapping_sub(((*E).boulder_var_05) as i16) < 0i16 {
        let v1: u16 = ((*E).boulder_var_C).wrapping_add(256u16);
        (*E).boulder_var_C = v1;
        if v1 as i32 - 20480i32 & 0x8000i32 == 0 {
            (*E).boulder_var_C = 20480u16;
        }
    } else {
        (*E).base.y_pos = (*E).boulder_var_05;
        (*E).boulder_var_A = ((0xa68832i32) as u16);
        (*E).boulder_var_C = 0x2000u16;
    };
}
pub unsafe fn Boulder_Func_3(game: &mut Game) {
    let E = Get_Boulder(ram_word!(game, cur_enemy_index) as i32, game);
    Boulder_Func_9(
        (8i32 * *(&raw mut (*E).boulder_var_C as *mut u8).offset(1isize) as i32 + 4i32) as u16,
        game,
    );
    let mut v1: i16 = (((*E).boulder_var_C) as i16).wrapping_sub(256i16);
    (*E).boulder_var_C = v1 as u16;
    if v1 >= 0i16 {
        let mut v2: u16 =
            (8u16).wrapping_mul((*(&raw mut (*E).boulder_var_B as *mut u8).offset(1isize)) as u16);
        if (*E).boulder_var_E != 0 {
            v2 = (v2).wrapping_add(4u16);
        }
        Boulder_Func_7(v2, game);
        let v3: u16 = ((*E).boulder_var_B).wrapping_add(32u16);
        (*E).boulder_var_B = v3;
        if v3 as i32 - 20480i32 & 0x8000i32 == 0 {
            (*E).boulder_var_B = 20480u16;
        }
    } else {
        (*E).boulder_var_C = 0u16;
        (*E).boulder_var_A = ((0xa6888bi32) as u16);
    };
}
pub unsafe fn Boulder_Func_4(game: &mut Game) {
    let E = Get_Boulder(ram_word!(game, cur_enemy_index) as i32, game);
    let v1: i32 = 8i32 * *(&raw mut (*E).boulder_var_C as *mut u8).offset(1isize) as i32 >> 1i32;
    if Enemy_MoveDown(
        ram_word!(game, cur_enemy_index),
        *(game.g_rom.offset(crate::rtl::lorom_offset(0xa0cbc7)) as *mut u32)
            .offset((v1 >> 1i32) as isize) as i32,
        game,
    ) != 0
    {
        QueueSfx2_Max6(0x42u16, game);
        if (*E).boulder_var_E == 2u16 {
            (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Deleted) as u16);
            game.eproj_spawn_pt = Point16U {
                x: (*E).base.x_pos,
                y: (*E).base.y_pos,
            };
            SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 0x11u16, game);
            QueueSfx2_Max6(0x43u16, game);
        } else {
            (*E).boulder_var_A = ((0xa68832i32) as u16);
            (*E).boulder_var_C = g_word_A686F1[((*E).boulder_var_D as i32 - 1i32) as usize];
            (*E).boulder_var_D = (*E).boulder_var_D.wrapping_sub(1);
            if ((*E).boulder_var_D & 0x8000) != 0 {
                (*E).boulder_var_03 = (*E).base.y_pos;
                (*E).boulder_var_02 = (*E).base.y_subpos;
                (*E).boulder_var_A = ((0xa68942i32) as u16);
            }
        }
    } else {
        (*E).boulder_var_C = ((*E).boulder_var_C).wrapping_add(256u16);
        let mut v3: u16 =
            (8u16).wrapping_mul((*(&raw mut (*E).boulder_var_B as *mut u8).offset(1isize)) as u16);
        if (*E).boulder_var_E != 0 {
            v3 = (v3).wrapping_add(4u16);
        }
        Boulder_Func_7(v3, game);
        let v4: u16 = ((*E).boulder_var_B).wrapping_add(32u16);
        (*E).boulder_var_B = v4;
        if v4 as i32 - 20480i32 & 0x8000i32 == 0 {
            (*E).boulder_var_B = 20480u16;
        }
    };
}
pub unsafe fn Boulder_Func_5(game: &mut Game) {
    let E = Get_Boulder(ram_word!(game, cur_enemy_index) as i32, game);
    let mut v2: u16 =
        (8u16).wrapping_mul((*(&raw mut (*E).boulder_var_B as *mut u8).offset(1isize)) as u16);
    let v3: i32 = v2 as i32 >> 1i32;
    Enemy_MoveDown(
        ram_word!(game, cur_enemy_index),
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0xa0cbc7)) as *mut u32)
            .offset((v3 >> 1i32) as isize))
        .wrapping_add((((*E).boulder_var_01 as i32) << 16i32) as u32) as i32,
        game,
    );
    (*E).base.y_pos = ((*E).base.y_pos).wrapping_sub((*E).boulder_var_01);
    if (*E).boulder_var_E != 0 {
        v2 = (v2).wrapping_add(4u16);
    }
    let v4: i32 = v2 as i32 >> 1i32;
    if Enemy_MoveRight_IgnoreSlopes(
        ram_word!(game, cur_enemy_index),
        *(game.g_rom.offset(crate::rtl::lorom_offset(0xa0cbc7)) as *mut u32)
            .offset((v4 >> 1i32) as isize) as i32,
        game,
    ) != 0
    {
        (*E).base.properties = ((*E).base.properties) | (0x300u16);
        (*E).boulder_var_A = ((0xa689fci32) as u16);
        QueueSfx2_Max6(0x42u16, game);
        game.eproj_spawn_pt = Point16U {
            x: (*E).base.x_pos,
            y: (*E).base.y_pos,
        };
        SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 0x11u16, game);
        QueueSfx2_Max6(0x43u16, game);
    } else {
        let v5: u16 = ((*E).boulder_var_B).wrapping_add(64u16);
        (*E).boulder_var_B = v5;
        if v5 as i32 - 0x4000i32 & 0x8000i32 == 0 {
            (*E).boulder_var_B = 0x4000u16;
        }
        if (*E).base.y_pos == (*E).boulder_var_03 && (*E).base.y_subpos == (*E).boulder_var_02 {
            (*E).boulder_var_01 = 0u16;
        }
    }
    (*E).boulder_var_03 = (*E).base.y_pos;
    (*E).boulder_var_02 = (*E).base.y_subpos;
}
pub fn Boulder_Func_6() {}
pub unsafe fn Boulder_Func_7(j: u16, game: &mut Game) {
    let E = Get_Boulder(ram_word!(game, cur_enemy_index) as i32, game);
    let v3: i32 = j as i32 >> 1i32;
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        (crate::rtl::rom_table_word!(game, 0xa2838f, (v3 + 1i32) as isize) as u32) << 16i32
            | crate::rtl::rom_table_word!(game, 0xa2838f, v3 as isize) as u32,
    );
}
pub unsafe fn Boulder_Func_8(j: u16, game: &mut Game) {
    let E = Get_Boulder(ram_word!(game, cur_enemy_index) as i32, game);
    let v3: i32 = j as i32 >> 1i32;
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        (crate::rtl::rom_table_word!(game, 0xa28187, (v3 + 1i32) as isize) as u32) << 16i32
            | crate::rtl::rom_table_word!(game, 0xa28187, v3 as isize) as u32,
    );
}
pub unsafe fn Boulder_Func_9(j: u16, game: &mut Game) {
    let E = Get_Boulder(ram_word!(game, cur_enemy_index) as i32, game);
    let v3: i32 = j as i32 >> 1i32;
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        (crate::rtl::rom_table_word!(game, 0xa2838f, (v3 + 1i32) as isize) as u32) << 16i32
            | crate::rtl::rom_table_word!(game, 0xa2838f, v3 as isize) as u32,
    );
}
pub unsafe fn SpikeyPlatform_Init(game: &mut Game) {
    let E = Get_SpikeyPlatform(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.current_instruction = addr_kSpikeyPlatform_Ilist_8B29 as u16;
    (*E).spm_var_A = ((0xa68bb4i32) as u16);
    let v1: u16 =
        (8u16).wrapping_mul((*(&raw mut (*E).spm_parameter_1 as *mut u8).offset(0)) as u16);
    (*E).spm_var_02 = v1;
    let v2: i32 = v1 as i32 >> 1i32;
    (*E).spm_var_E = crate::rtl::rom_table_word!(game, 0xa28187, (v2 + 1i32) as isize);
    (*E).spm_var_F = crate::rtl::rom_table_word!(game, 0xa28187, v2 as isize);
    (*E).spm_var_D = ((*E).base.y_pos)
        .wrapping_add((*(&raw mut (*E).spm_parameter_2 as *mut u8).offset(1isize)) as u16);
    (*E).spm_var_C = (*E).base.y_pos;
    let spm_parameter_2_low: u16 = *(&raw mut (*E).spm_parameter_2 as *mut u8).offset(0) as u16;
    (*E).spm_var_00 = spm_parameter_2_low;
    (*E).spm_var_B = spm_parameter_2_low;
}
pub unsafe fn SpikeyPlatform2ndEnemy_Init(game: &mut Game) {
    let v0: i32 = ram_word!(game, cur_enemy_index) as i32 >> 1i32;
    let v1 = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
    (*v1).x_pos = ram_word!(game, enemy_drawing_queue[(v0 + 91i32)]);
    (*v1).y_pos = (ram_word!(game, enemy_drawing_queue[(v0 + 93i32)])).wrapping_add(12u16);
}
pub unsafe fn SpikeyPlatform2ndEnemy_Main(game: &mut Game) {
    let v0: i32 = ram_word!(game, cur_enemy_index) as i32 >> 1i32;
    let v1 = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
    (*v1).x_pos = ram_word!(game, enemy_drawing_queue[(v0 + 91i32)]);
    (*v1).y_pos = (ram_word!(game, enemy_drawing_queue[(v0 + 93i32)])).wrapping_add(12u16);
}
crate::game::address_dispatch! {
    pub unsafe fn CallSpikeyPlatformFunc(ea: u32, k: u16, game: &mut Game) => ea;
        call (game); default { Unreachable(game); };
    {
        10914740 => SpikeyPlatform_Func_1; 10914780 => SpikeyPlatform_Func_2;
        10914890 => [SpikeyPlatform_Func_3(k, game)]; 10914909 => SpikeyPlatform_Func_4;
    }
}
pub unsafe fn SpikeyPlatform_Main(game: &mut Game) {
    let E = Get_SpikeyPlatform(ram_word!(game, cur_enemy_index) as i32, game);
    CallSpikeyPlatformFunc(
        (((*E).spm_var_A) as u32) | (0xa60000u32),
        ram_word!(game, cur_enemy_index),
        game,
    );
}
pub unsafe fn SpikeyPlatform_Func_1(game: &mut Game) {
    let E = Get_SpikeyPlatform(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).spm_var_B = (*E).spm_var_B.wrapping_sub(1);
    if (*E).spm_var_B == 0 {
        (*E).spm_var_B = (*E).spm_var_00;
        (*E).spm_var_02 =
            (8u16).wrapping_mul((*(&raw mut (*E).spm_parameter_1 as *mut u8).offset(0)) as u16);
        (*E).spm_var_A = ((0xa68bdci32) as u16);
    }
}
pub unsafe fn SpikeyPlatform_Func_2(game: &mut Game) {
    let E = Get_SpikeyPlatform(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).spm_var_01 = (*E).base.y_pos;
    let v1: i32 = (*E).spm_var_02 as i32 >> 1i32;
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        (crate::rtl::rom_table_word!(game, 0xa28187, v1 as isize) as u32) << 16i32
            | crate::rtl::rom_table_word!(game, 0xa28187, (v1 + 1i32) as isize) as u32,
    );
    if (((*E).base.y_pos) as i16).wrapping_sub(((*E).spm_var_D) as i16) >= 0i16 {
        (*E).spm_var_03 = 64u16;
        (*E).spm_var_A = ((0xa68c4ai32) as u16);
        (*E).base.y_pos = (*E).spm_var_D;
        QueueSfx2_Max6(0x1bu16, game);
    }
    if SpikeyPlatform_Func_5(ram_word!(game, cur_enemy_index), game) != 0 {
        update_ram_word!(game.extra_samus_y_displacement, |fresh17| (fresh17 as i32
            + ((*E).base.y_pos as i32 - (*E).spm_var_01 as i32))
            as u16);
    }
    let mut v4: i16 = (((*E).spm_var_02) as i16).wrapping_add(8i16);
    if (*E).spm_var_02 as i32 - 504i32 & 0x8000i32 == 0 {
        v4 = 512i16;
    }
    (*E).spm_var_02 = v4 as u16;
}
pub unsafe fn SpikeyPlatform_Func_3(k: u16, game: &mut Game) {
    let E = Get_SpikeyPlatform(k as i32, game);
    let v2: u16 = ((*E).spm_var_03).wrapping_sub(1u16);
    (*E).spm_var_03 = v2;
    if v2 == 0 {
        (*E).spm_var_A = ((0xa68c5di32) as u16);
    }
}
pub unsafe fn SpikeyPlatform_Func_4(game: &mut Game) {
    let E = Get_SpikeyPlatform(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).spm_var_01 = (*E).base.y_pos;
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        -0x8000i32 as u32,
    );
    if (((*E).base.y_pos) as i16).wrapping_sub(((*E).spm_var_C) as i16) < 0i16 {
        (*E).spm_var_A = ((0xa68bb4i32) as u16);
        (*E).base.y_pos = (*E).spm_var_C;
    }
    if SpikeyPlatform_Func_5(ram_word!(game, cur_enemy_index), game) != 0 {
        update_ram_word!(game.extra_samus_y_displacement, |fresh18| (fresh18 as i32
            + ((*E).base.y_pos as i32 - (*E).spm_var_01 as i32))
            as u16);
    }
}
pub unsafe fn SpikeyPlatform_Func_5(k: u16, game: &mut Game) -> u16 {
    let E = Get_SpikeyPlatform(k as i32, game);
    let v2: u16 = abs16((ram_word!(game, samus_x_pos)).wrapping_sub((*E).base.x_pos));
    let mut v3: bool = v2 < ram_word!(game, samus_x_radius);
    let v4: u16 = (v2).wrapping_sub(ram_word!(game, samus_x_radius));
    if !v3 && v4 >= (*E).base.x_width {
        return 0u16;
    }
    if ((ram_word!(game, samus_y_pos) as i32 + 5i32 - (*E).base.y_pos as i32) as i16) < 0i16 {
        let v6: u16 =
            ((*E).base.y_pos as i32 - (ram_word!(game, samus_y_pos) as i32 + 5i32)) as u16;
        v3 = v6 < ram_word!(game, samus_y_radius);
        let v7: u16 = (v6).wrapping_sub(ram_word!(game, samus_y_radius));
        if v3 || v7 == (*E).base.y_height || v7 < (*E).base.y_height {
            return -1i32 as u16;
        }
    }
    0u16
}
pub unsafe fn FireGeyser_Instr_1(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    QueueSfx2_Max6(0x61u16, game);
    jp
}
// Each animation step reads the live parent, then selects the adjacent child.
unsafe fn set_fire_geyser_shape(offset: u16, height: u16, begin: bool, game: &mut Game) {
    let y = (*Get_FireGeyser(ram_word!(game, cur_enemy_index) as i32, game))
        .fgr_var_D
        .wrapping_sub(offset);
    let child = Get_FireGeyser(ram_word!(game, cur_enemy_index) as i32 + 64, game);
    (*child).base.y_pos = y;
    (*child).base.y_height = height;
    if begin {
        (*child).base.x_width = 8;
    }
}

pub unsafe fn FireGeyser_Instr_2(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    set_fire_geyser_shape(g_word_A68DBB, g_word_A68DE7, true, game);
    jp
}
pub unsafe fn FireGeyser_Instr_3(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    set_fire_geyser_shape(g_word_A68DBD, g_word_A68DE9, false, game);
    jp
}
pub unsafe fn FireGeyser_Instr_4(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    set_fire_geyser_shape(g_word_A68DBF, g_word_A68DEB, false, game);
    jp
}
pub unsafe fn FireGeyser_Instr_5(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    set_fire_geyser_shape(g_word_A68DC1, g_word_A68DED, false, game);
    jp
}
pub unsafe fn FireGeyser_Instr_6(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    set_fire_geyser_shape(g_word_A68DC3, g_word_A68DEF, false, game);
    jp
}
pub unsafe fn FireGeyser_Instr_7(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    set_fire_geyser_shape(g_word_A68DC5, g_word_A68DF1, false, game);
    jp
}
pub unsafe fn FireGeyser_Instr_8(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    set_fire_geyser_shape(g_word_A68DC7, g_word_A68DF3, false, game);
    jp
}
pub unsafe fn FireGeyser_Instr_9(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    set_fire_geyser_shape(g_word_A68DC9, g_word_A68DF5, false, game);
    jp
}
pub unsafe fn FireGeyser_Instr_10(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    set_fire_geyser_shape(g_word_A68DCB, g_word_A68DF7, false, game);
    jp
}
pub unsafe fn FireGeyser_Instr_11(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    set_fire_geyser_shape(g_word_A68DCD, g_word_A68DF9, false, game);
    jp
}
pub unsafe fn FireGeyser_Instr_12(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    set_fire_geyser_shape(g_word_A68DCF, g_word_A68DFB, false, game);
    jp
}
pub unsafe fn FireGeyser_Instr_13(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    set_fire_geyser_shape(g_word_A68DD1, g_word_A68DFD, false, game);
    jp
}
pub unsafe fn FireGeyser_Instr_14(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    set_fire_geyser_shape(g_word_A68DD3, g_word_A68DFF, false, game);
    jp
}
pub unsafe fn FireGeyser_Instr_15(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    set_fire_geyser_shape(g_word_A68DD5, g_word_A68E01, false, game);
    jp
}
pub unsafe fn FireGeyser_Instr_16(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    set_fire_geyser_shape(g_word_A68DD7, g_word_A68E03, false, game);
    jp
}
pub unsafe fn FireGeyser_Instr_17(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    set_fire_geyser_shape(g_word_A68DD9, g_word_A68E05, false, game);
    jp
}
pub unsafe fn FireGeyser_Instr_18(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    set_fire_geyser_shape(g_word_A68DDB, g_word_A68E07, false, game);
    jp
}
pub unsafe fn FireGeyser_Instr_19(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    set_fire_geyser_shape(g_word_A68DDD, g_word_A68E09, false, game);
    jp
}
pub unsafe fn FireGeyser_Instr_20(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    set_fire_geyser_shape(g_word_A68DDF, g_word_A68E0B, false, game);
    jp
}
pub unsafe fn FireGeyser_Instr_21(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    set_fire_geyser_shape(g_word_A68DE1, g_word_A68E0D, false, game);
    jp
}
pub unsafe fn FireGeyser_Instr_22(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    set_fire_geyser_shape(g_word_A68DE3, g_word_A68E0F, false, game);
    jp
}
pub unsafe fn FireGeyser_Instr_23(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    set_fire_geyser_shape(g_word_A68DE5, g_word_A68E11, false, game);
    jp
}
pub unsafe fn FireGeyser_Instr_24(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E = Get_FireGeyser(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).fgr_var_C = 1u16;
    let E1 = Get_FireGeyser(ram_word!(game, cur_enemy_index) as i32 + 64i32, game);
    set_fields! {[(*E1).base], 0u16; x_width, y_height}
    (*E).base.y_pos = (*E).fgr_var_D;
    (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Invisible) as u16);
    (*E1).base.properties = ((*E1).base.properties) | (0x400u16);
    jp
}
pub unsafe fn FireGeyser_Init(game: &mut Game) {
    let E = Get_FireGeyser(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.current_instruction = addr_kFireGeyser_Ilist_8DA9 as u16;
    if (*E).fgr_parameter_2 == 0 {
        (*E).base.current_instruction = addr_kFireGeyser_Ilist_8D1B as u16;
        (*E).fgr_var_A = ((0xa6902fi32) as u16);
        (*E).fgr_var_D = (*E).base.y_pos;
        (*E).base.x_width = 0u16;
    }
}
crate::game::address_dispatch! {
    pub unsafe fn CallFireGeyserFunc(ea: u32, game: &mut Game) => ea;
        call (game); default { Unreachable(game); };
    {
        10915887 => FireGeyser_Func_1; 10915938 => FireGeyser_Func_2;
    }
}
pub unsafe fn FireGeyser_Main(game: &mut Game) {
    let E = Get_FireGeyser(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).fgr_parameter_2 == 0 {
        CallFireGeyserFunc((((*E).fgr_var_A) as u32) | (0xa60000u32), game);
    }
}
pub unsafe fn FireGeyser_Func_1(game: &mut Game) {
    let E = Get_FireGeyser(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).fgr_var_B = (*E).fgr_var_B.wrapping_sub(1);
    if ((*E).fgr_var_B & 0x8000) != 0 {
        (*E).fgr_var_A = ((0xa69062i32) as u16);
        (*E).fgr_var_C = 0u16;
        (*E).base.instruction_timer = 1u16;
        (*E).base.timer = 0u16;
        (*E).base.current_instruction = addr_kFireGeyser_Ilist_8D1B as u16;
        (*E).base.properties =
            ((*E).base.properties as i32 & !(kEnemyProps_Invisible as i32)) as u16;
        let E1 = Get_FireGeyser(ram_word!(game, cur_enemy_index) as i32 + 64i32, game);
        (*E1).base.properties = ((*E1).base.properties & !(0x400));
    }
}
pub unsafe fn FireGeyser_Func_2(game: &mut Game) {
    let E = Get_FireGeyser(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).fgr_var_C != 0 {
        (*E).fgr_var_B = (*E).fgr_parameter_1;
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Invisible) as u16);
        (*E).fgr_var_A = ((0xa6902fi32) as u16);
    }
}
pub unsafe fn NuclearWaffle_Init(game: &mut Game) {
    let E = Get_NuclearWaffle(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.current_instruction = addr_kNuclearWaffle_Ilist_9490 as u16;
    (*E).nwe_var_C = *(&raw mut (*E).nwe_parameter_1 as *mut u8).offset(0) as u16;
    (*E).nwe_var_D = *(&raw mut (*E).nwe_parameter_1 as *mut u8).offset(1isize) as u16;
    (*E).nwe_var_E = *(&raw mut (*E).nwe_parameter_2 as *mut u8).offset(0) as u16;
    let nwe_parameter_2_high: u16 =
        *(&raw mut (*E).nwe_parameter_2 as *mut u8).offset(1isize) as u16;
    (*E).nwe_var_F = nwe_parameter_2_high;
    (*E).nwe_var_B = nwe_parameter_2_high;
    (*E).nwe_var_A = ((0xa69615i32) as u16);
    let v2: i32 = ((4u16).wrapping_mul((*E).nwe_var_E)) as i32 >> 1i32;
    let v3: u16 = g_word_A695F6[v2 as usize];
    (*E).nwe_var_23 = v3;
    (*E).nwe_var_21 = v3;
    (*E).nwe_var_2E = g_word_A695F6[(v2 + 1i32) as usize];
    (*E).nwe_var_2B = g_word_A695FE[v2 as usize];
    (*E).nwe_var_2C = g_word_A695FE[(v2 + 1i32) as usize];
    (*E).nwe_var_30 = g_word_A69606[v2 as usize];
    (*E).nwe_var_2F = g_word_A69606[(v2 + 1i32) as usize];
    let mut v4: u16 = (8u16).wrapping_mul((*E).nwe_var_C);
    if (*E).nwe_var_E == 0 {
        v4 = (v4).wrapping_add(4u16);
    }
    let v5: i32 = v4 as i32 >> 1i32;
    (*E).nwe_var_25 = crate::rtl::rom_table_word!(game, 0xa28187, v5 as isize);
    (*E).nwe_var_24 = crate::rtl::rom_table_word!(game, 0xa28187, (v5 + 1i32) as isize);
    (*E).nwe_var_26 = (*E).base.x_pos;
    (*E).nwe_var_27 = (*E).base.y_pos;
    let v6: u16 =
        ((*E).nwe_var_26).wrapping_add(CosineMult8bit((*E).nwe_var_23, (*E).nwe_var_D, game));
    (*E).nwe_var_28 = v6;
    (*E).base.x_pos = v6;
    let v7: u16 =
        ((*E).nwe_var_27).wrapping_add(SineMult8bit((*E).nwe_var_23, (*E).nwe_var_D, game));
    (*E).nwe_var_29 = v7;
    (*E).base.y_pos = v7;
    let mut v8: u16 = 8u16;
    (*E).nwe_var_2A = 8u16;
    loop {
        SpawnEprojWithGfx(
            v8,
            ram_word!(game, cur_enemy_index),
            addr_kEproj_NuclearWaffleBody as u16,
            game,
        );
        v8 = ((*E).nwe_var_2A).wrapping_sub(2u16);
        (*E).nwe_var_2A = v8;
        if (v8 == 0) {
            break;
        }
    }
    (*E).nwe_var_2A = 6u16;
    loop {
        let v13: u16 = ((*E).base.vram_tiles_index) | ((*E).base.palette_index);
        (*E).nwe_var_34 = v13;
        let r18: u16 = CreateSpriteAtPos((*E).base.x_pos, (*E).base.y_pos, 43u16, v13, game);
        (*Get_NuclearWaffle(
            ram_word!(game, cur_enemy_index) as i32 + (*E).nwe_var_2A as i32,
            game,
        ))
        .nwe_var_10 = r18;
        let mut v15: u16 = ((*E).nwe_var_2A).wrapping_sub(2u16);
        (*E).nwe_var_2A = v15;
        if (v15 == 0) {
            break;
        }
    }
}
crate::game::address_dispatch! {
    pub unsafe fn CallNuclearWaffleFunc(ea: u32, game: &mut Game) => ea;
        call (game); default { Unreachable(game); };
    {
        10917397 => NuclearWaffle_Func_1; 10917506 => NuclearWaffle_Func_2;
    }
}
pub unsafe fn NuclearWaffle_Main(game: &mut Game) {
    let E = Get_NuclearWaffle(ram_word!(game, cur_enemy_index) as i32, game);
    CallNuclearWaffleFunc((((*E).nwe_var_A) as u32) | (0xa60000u32), game);
}
pub unsafe fn NuclearWaffle_Func_1(game: &mut Game) {
    let E = Get_NuclearWaffle(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).nwe_var_B = (*E).nwe_var_B.wrapping_sub(1);
    if ((*E).nwe_var_B & 0x8000) != 0 {
        (*E).nwe_var_B = (*E).nwe_var_F;
        (*E).nwe_var_21 = (*E).nwe_var_23;
        (*E).nwe_var_A = ((0xa69682i32) as u16);
        set_fields! {[(*E)], 0u16; nwe_var_33, nwe_var_08, nwe_var_09, nwe_var_0A, nwe_var_0B, nwe_var_0C, nwe_var_0D, nwe_var_0E, nwe_var_0F, nwe_var_18, nwe_var_19, nwe_var_1A, nwe_var_1B, nwe_var_1C, nwe_var_1D, nwe_var_1E, nwe_var_1F}
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_ProcessedOffscreen) as u16);
    }
}
pub unsafe fn NuclearWaffle_Func_2(game: &mut Game) {
    let E = Get_NuclearWaffle(ram_word!(game, cur_enemy_index) as i32, game);
    let varE32: u16 = (*E).nwe_var_D;
    let pair: PairU16 = NuclearWaffle_Func_6((*E).nwe_var_21, game);
    let v8: u16 = pair.j;
    if pair.j != (*E).nwe_var_33 {
        CreateSpriteAtPos(
            (*E).base.x_pos,
            (*E).base.y_pos,
            46u16,
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
    let mut tmp: u16 = 0;
    let v4: u16 = NuclearWaffle_Func_5((*E).nwe_var_21, &raw mut tmp, game);
    (*E).base.x_pos = ((*E).nwe_var_26).wrapping_add(CosineMult8bit(v4, (*E).nwe_var_D, game));
    let v5: u16 = NuclearWaffle_Func_5((*E).nwe_var_21, &raw mut tmp, game);
    (*E).base.y_pos = ((*E).nwe_var_27).wrapping_add(SineMult8bit(v5, (*E).nwe_var_D, game));
    NuclearWaffle_Func_3(varE32, game);
    NuclearWaffle_Func_4(varE32, game);
    AddToHiLo(
        &raw mut (*E).nwe_var_21,
        &raw mut (*E).nwe_var_20,
        ((*E).nwe_var_25 as u32) << 16i32 | (*E).nwe_var_24 as u32,
    );
}
pub unsafe fn NuclearWaffle_Func_3(varE32: u16, game: &mut Game) {
    let E = Get_NuclearWaffle(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).nwe_var_2A = 8u16;
    let mut R36: u16 = ((*E).nwe_var_2C).wrapping_add((*E).nwe_var_21);
    let mut r28: u16 = 0;
    loop {
        R36 = (R36).wrapping_sub((*E).nwe_var_2B);
        let ET = Get_NuclearWaffle(
            ram_word!(game, cur_enemy_index) as i32 + (*E).nwe_var_2A as i32,
            game,
        );
        let nwe_var_00: u16 = (*ET).nwe_var_00;
        let pair: PairU16 = NuclearWaffle_Func_6(R36, game);
        let v13: u16 = pair.j;
        if pair.j != (*ET).nwe_var_08 {
            let v5: i32 = nwe_var_00 as i32 >> 1i32;
            CreateSpriteAtPos(
                ram_word!(game, eproj_x_pos[v5]),
                ram_word!(game, eproj_y_pos[v5]),
                46u16,
                (*E).nwe_var_34,
                game,
            );
            CreateSpriteAtPos(
                ram_word!(game, eproj_x_pos[v5]),
                ram_word!(game, eproj_y_pos[v5]),
                (pair.k).wrapping_add(44u16),
                (*E).nwe_var_34,
                game,
            );
            NuclearWaffle_Func_7(pair.j, game);
        }
        (*ET).nwe_var_08 = v13;
        let v7: u16 = NuclearWaffle_Func_5(R36, &raw mut r28, game);
        let v10: i32 = nwe_var_00 as i32 >> 1i32;
        set_ram_word!(
            game.eproj_x_pos[v10] =
                ((*E).nwe_var_26).wrapping_add(CosineMult8bit(v7, varE32, game))
        );
        let v11: u16 = NuclearWaffle_Func_5(R36, &raw mut r28, game);
        set_ram_word!(
            game.eproj_y_pos[v10] = ((*E).nwe_var_27).wrapping_add(SineMult8bit(v11, varE32, game))
        );
        let mut v12: u16 = ((*E).nwe_var_2A).wrapping_sub(2u16);
        (*E).nwe_var_2A = v12;
        if (v12 == 0) {
            break;
        }
    }
    if r28 != 0 {
        (*E).nwe_var_A = ((0xa69615i32) as u16);
        (*E).base.properties =
            ((*E).base.properties as i32 & !(kEnemyProps_ProcessedOffscreen as i32)) as u16;
    }
}
pub unsafe fn NuclearWaffle_Func_4(varE32: u16, game: &mut Game) {
    let E = Get_NuclearWaffle(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).nwe_var_2A = 6u16;
    let mut R36: u16 = (*E).nwe_var_21;
    let mut r28: u16 = 0u16;
    loop {
        R36 = (R36).wrapping_sub((*E).nwe_var_2B);
        let ET = Get_NuclearWaffle(
            ram_word!(game, cur_enemy_index) as i32 + (*E).nwe_var_2A as i32,
            game,
        );
        let nwe_var_10: u16 = (*ET).nwe_var_10;
        let pair: PairU16 = NuclearWaffle_Func_6(R36, game);
        let v13: u16 = pair.j;
        if pair.j != (*ET).nwe_var_18 {
            let r38: u16 = (*E).nwe_var_34;
            let v5: i32 = nwe_var_10 as i32 >> 1i32;
            CreateSpriteAtPos(
                ram_word!(game, sprite_x_pos[v5]),
                ram_word!(game, sprite_y_pos[v5]),
                46u16,
                r38,
                game,
            );
            CreateSpriteAtPos(
                ram_word!(game, sprite_x_pos[v5]),
                ram_word!(game, sprite_y_pos[v5]),
                (pair.k).wrapping_add(44u16),
                r38,
                game,
            );
            NuclearWaffle_Func_7(pair.j, game);
        }
        (*ET).nwe_var_18 = v13;
        let v6: u16 = NuclearWaffle_Func_5(R36, &raw mut r28, game);
        let v7: u16 = CosineMult8bit(v6, varE32, game);
        let R32: u16 = ((*E).nwe_var_26).wrapping_add(v7);
        let v9: u16 = NuclearWaffle_Func_5(R36, &raw mut r28, game);
        let R34: u16 = ((*E).nwe_var_27).wrapping_add(SineMult8bit(v9, varE32, game));
        set_ram_word!(game.sprite_x_pos[(nwe_var_10 >> 1)] = R32);
        set_ram_word!(game.sprite_y_pos[(nwe_var_10 >> 1)] = R34);
        let mut v12: u16 = ((*E).nwe_var_2A).wrapping_sub(2u16);
        (*E).nwe_var_2A = v12;
        if (v12 == 0) {
            break;
        }
    }
}
pub unsafe fn NuclearWaffle_Func_5(a: u16, r28_out: *mut u16, game: &mut Game) -> u16 {
    *r28_out = 0u16;
    let E = Get_NuclearWaffle(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).nwe_var_E == 0 {
        if ((a) as i16).wrapping_sub(((*E).nwe_var_2E) as i16) >= 0i16 {
            if ((a) as i16).wrapping_sub(((*E).nwe_var_23) as i16) < 0i16 {
                return a;
            }
            return (*E).nwe_var_23;
        }
    } else if (((a) as i16).wrapping_sub(((*E).nwe_var_2E) as i16) < 0i16) {
        if ((a) as i16).wrapping_sub(((*E).nwe_var_23) as i16) < 0i16 {
            return (*E).nwe_var_23;
        }
        return a;
    }
    *r28_out = (*r28_out).wrapping_add(1);
    (*E).nwe_var_2E
}
unsafe fn NuclearWaffle_Func_6(a: u16, game: &mut Game) -> PairU16 {
    let E = Get_NuclearWaffle(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).nwe_var_E != 0 {
        if ((a) as i16).wrapping_sub(((*E).nwe_var_2F) as i16) >= 0i16 {
            PairU16 { k: 1u16, j: 2u16 }
        } else if ((a) as i16).wrapping_sub(((*E).nwe_var_30) as i16) >= 0i16 {
            PairU16 { k: 0u16, j: 1u16 }
        } else {
            PairU16 { k: 0u16, j: 0u16 }
        }
    } else if ((a) as i16).wrapping_sub(((*E).nwe_var_2F) as i16) < 0i16 {
        PairU16 { k: 0u16, j: 2u16 }
    } else if ((a) as i16).wrapping_sub(((*E).nwe_var_30) as i16) < 0i16 {
        PairU16 { k: 1u16, j: 1u16 }
    } else {
        PairU16 { k: 0u16, j: 0u16 }
    }
}
pub unsafe fn NuclearWaffle_Func_7(r30: u16, game: &mut Game) {
    if r30 != 2u16 {
        QueueSfx2_Max6(0x5eu16, game);
    }
}
pub unsafe fn FakeKraid_Init(game: &mut Game) {
    let mut v0: u16 = ((ram_word!(game, random_number) as i32 & 3i32) + 2i32) as u16;
    let E = Get_FakeKraid(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).fkd_var_D = v0;
    (*E).fkd_var_E = v0;
    v0 = (v0).wrapping_add(64u16);
    (*E).fkd_var_03 = v0;
    v0 = (v0).wrapping_add(32u16);
    (*E).fkd_var_04 = v0;
    (*E).fkd_var_05 = (v0).wrapping_sub(48u16);
    (*E).fkd_var_07 = 0u16;
    (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_DisableSamusColl) as u16);
    (*E).base.instruction_timer = 1u16;
    (*E).base.timer = 0u16;
    (*E).fkd_var_B = -4i32 as u16;
    (*E).fkd_var_C = -4i32 as u16;
    let mut v2: u16 = addr_stru_A699AE as u16;
    if (((*E).base.x_pos) as i16).wrapping_sub((ram_word!(game, samus_x_pos)) as i16) < 0i16 {
        set_fields! {[(*E)], 4u16; fkd_var_B, fkd_var_C}
        v2 = addr_stru_A699FC as u16;
    }
    (*E).base.current_instruction = v2;
}
pub unsafe fn FakeKraid_Main(game: &mut Game) {
    let E = Get_FakeKraid(ram_word!(game, cur_enemy_index) as i32, game);
    let fkd_var_07: u16 = (*E).fkd_var_07;
    let mut v3: u16 = (fkd_var_07).wrapping_add(2u16);
    if fkd_var_07 as i32 - 4i32 & 0x8000i32 == 0 {
        v3 = 0u16;
    }
    (*E).fkd_var_07 = v3;
    FakeKraid_Func_1(ram_word!(game, cur_enemy_index), fkd_var_07, game);
}
pub unsafe fn FakeKraid_Func_1(k: u16, j: u16, game: &mut Game) {
    let ET = Get_FakeKraid(k as i32 + j as i32, game);
    let fkd_var_03: u16 = (*ET).fkd_var_03;
    if fkd_var_03 != 0 {
        (*ET).fkd_var_03 = (fkd_var_03).wrapping_sub(1u16);
    } else {
        (*ET).fkd_var_03 = ((ram_word!(game, random_number) as i32 & 0x3fi32) + 16i32) as u16;
        let E = Get_FakeKraid(k as i32, game);
        (*E).fkd_var_06 = j;
        let mut v6: u16 = addr_kEproj_MiniKraidSpikesLeft as u16;
        let fkd_var_C: i16 = (*E).fkd_var_C as i16;
        if fkd_var_C >= 0i16 {
            v6 = addr_kEproj_MiniKraidSpikesRight as u16;
        }
        SpawnEprojWithGfx(fkd_var_C as u16, k, v6, game);
        if CheckIfEnemyIsOnScreen(game) == 0 {
            QueueSfx2_Max6(0x3fu16, game);
        }
    };
}
pub unsafe fn FakeKraid_Instr_2(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E = Get_FakeKraid(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).fkd_var_E != 0 {
        (*E).fkd_var_E = (*E).fkd_var_E.wrapping_sub(1);
    }
    crate::update_place! { previous fresh1 = (*E).fkd_var_D => (*E).fkd_var_D.wrapping_sub(1); }
    let reverse_direction: bool = if fresh1 == 1u16 {
        (*E).fkd_var_D = ((ram_word!(game, random_number) as i32 & 3i32) + 7i32) as u16;
        true
    } else {
        !(Enemy_MoveRight_IgnoreSlopes(
            ram_word!(game, cur_enemy_index),
            ((*E).fkd_var_B as i16 as i32) << 16i32,
            game,
        ) == 0)
    };
    if reverse_direction {
        (*E).fkd_var_B = -((*E).fkd_var_B as i32) as u16;
    }
    (*E).fkd_var_C = -4i32 as u16;
    if (((*E).base.x_pos) as i16).wrapping_sub((ram_word!(game, samus_x_pos)) as i16) < 0i16 {
        (*E).fkd_var_C = 4u16;
    }
    jp
}
pub unsafe fn FakeKraid_Instr_1(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E = Get_FakeKraid(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).fkd_var_E != 0 {
        if ((*E).fkd_var_C & 0x8000) != 0 {
            if ((*Get_FakeKraid(ram_word!(game, cur_enemy_index) as i32, game)).fkd_var_B & 0x8000)
                == 0
            {
                return addr_stru_A699C6 as i32 as usize as *const u16;
            }
            addr_stru_A699AE as i32 as usize as *const u16
        } else {
            if ((*E).fkd_var_B & 0x8000) != 0 {
                return addr_stru_A69A14 as i32 as usize as *const u16;
            }
            addr_stru_A699FC as i32 as usize as *const u16
        }
    } else {
        (*E).fkd_var_E = ((ram_word!(game, random_number) as i32 & 3i32) + 3i32) as u16;
        if ((*E).fkd_var_C & 0x8000) != 0 {
            return addr_kFakeKraid_Ilist_99DC as i32 as usize as *const u16;
        }
        addr_kFakeKraid_Ilist_9A2A as i32 as usize as *const u16
    }
}
pub unsafe fn FakeKraid_Instr_3(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    if CheckIfEnemyIsOnScreen(game) == 0u16 {
        QueueSfx2_Max6(0x16u16, game);
    }
    jp
}
pub unsafe fn FakeKraid_Instr_4(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    FakeKraid_InstrHelper_45(k, 0u16, 0xfffcu16, game);
    jp
}
pub unsafe fn FakeKraid_InstrHelper_45(k: u16, j: u16, a: u16, game: &mut Game) {
    let ka: u16 = ram_word!(game, cur_enemy_index);
    let E = Get_FakeKraid(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).fkd_var_02 = a;
    let v5: i32 = j as i32 >> 1i32;
    (*E).fkd_var_00 = g_word_A69A48[v5 as usize];
    let v6: u16 = g_word_A69A48[(v5 + 1i32) as usize];
    (*E).fkd_var_01 = v6;
    SpawnEprojWithGfx(
        v6,
        ram_word!(game, cur_enemy_index),
        addr_kEproj_MiniKraidSpit as u16,
        game,
    );
    (*E).fkd_var_00 = g_word_A69A48[(v5 + 2i32) as usize];
    let v8: u16 = g_word_A69A48[(v5 + 3i32) as usize];
    (*E).fkd_var_01 = v8;
    SpawnEprojWithGfx(v8, ka, addr_kEproj_MiniKraidSpit as u16, game);
}
pub unsafe fn FakeKraid_Instr_5(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    FakeKraid_InstrHelper_45(k, 8u16, 4u16, game);
    jp
}
pub unsafe fn sub_A69C0B(game: &mut Game) {
    let v0 = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
    set_ram_word!(game.special_death_item_drop_x_origin_pos = (*v0).x_pos);
    set_ram_word!(game.special_death_item_drop_y_origin_pos = (*v0).y_pos);
    NormalEnemyPowerBombAiSkipDeathAnim_CurEnemy(game);
    FakeKraid_9C50(game);
}
pub unsafe fn FakeKraid_Touch(game: &mut Game) {
    let E = Get_FakeKraid(ram_word!(game, cur_enemy_index) as i32, game);
    set_ram_word!(game.special_death_item_drop_x_origin_pos = (*E).base.x_pos);
    set_ram_word!(game.special_death_item_drop_y_origin_pos = (*E).base.y_pos);
    NormalEnemyTouchAiSkipDeathAnim_CurEnemy(game);
    FakeKraid_9C50(game);
}
pub unsafe fn FakeKraid_Shot(game: &mut Game) {
    let E = Get_FakeKraid(ram_word!(game, cur_enemy_index) as i32, game);
    set_ram_word!(game.special_death_item_drop_x_origin_pos = (*E).base.x_pos);
    set_ram_word!(game.special_death_item_drop_y_origin_pos = (*E).base.y_pos);
    NormalEnemyShotAiSkipDeathAnim_CurEnemy(game);
    FakeKraid_9C50(game);
}
pub unsafe fn FakeKraid_9C50(game: &mut Game) {
    if (*Get_FakeKraid(ram_word!(game, cur_enemy_index) as i32, game))
        .base
        .health
        == 0
    {
        // The death animation remains in the ROM spawn parameter for all four drops.
        let death_animation = 3;
        EnemyDeathAnimation(ram_word!(game, cur_enemy_index), death_animation, game);
        Enemy_ItemDrop_MiniKraid(ram_word!(game, cur_enemy_index), death_animation, game);
    }
}
pub unsafe fn CeresRidley_Init(game: &mut Game) {
    let E = Get_CeresRidley(0i32, game);
    if (*ram_ptr!(game; 0xd828, ram_word!(game, area_index) as isize) & 1) != 0 {
        (*E).base.properties = ((*E).base.properties as i32
            | (kEnemyProps_Tangible as i32
                | kEnemyProps_Deleted as i32
                | kEnemyProps_Invisible as i32)) as u16;
    } else {
        let mut i: i32 = 4094i32;
        while i >= 0i32 {
            set_ram_word!(game.tilemap_stuff[(i >> 1i32)] = 0u16);
            i -= 2i32;
        }
        DisableMinimapAndMarkBossRoomAsExplored(game);
        set_fields! {[(*E)], 0u16; cry_parameter_1, cry_parameter_2}
        Ridley_Func_99(addr_kRidley_Ilist_E538 as u16, game);
        (*E).base.palette_index = 3584u16;
        (*E).cry_var_0C = 3584u16;
        (*E).base.extra_properties = ((*E).base.extra_properties) | (4u16);
        set_fields! {[(*E)], 0u16; cry_var_01, cry_var_0D}
        set_ram_words! { game, 0u16; earthquake_type, earthquake_timer }
        Ridley_Func_92(game);
        Ridley_Func_117(game);
        if ram_word!(game, area_index) == 2u16 {
            (*E).base.properties = ((*E).base.properties as i32
                | (kEnemyProps_BlockPlasmaBeam as i32 | kEnemyProps_Tangible as i32))
                as u16;
            (*E).base.x_pos = 96u16;
            (*E).base.y_pos = 394u16;
            (*E).cry_var_A = ((0xa6a35bi32) as u16);
            set_fields! {[(*E)], 0u16; cry_var_B, cry_var_C}
            (*E).base.layer = 5u16;
            (*E).cry_var_01 = 0u16;
            (*E).cry_var_02 = 1u16;
            (*E).cry_var_20 = 64u16;
            (*E).cry_var_21 = 416u16;
            (*E).cry_var_22 = 64u16;
            (*E).cry_var_23 = 224u16;
            (*E).cry_var_10 = 2u16;
            (*E).cry_var_1C = 120u16;
            WriteColorsToTargetPalette(0xa6u8, 0x140u16, addr_word_A6E1CF as u16, 0x20u16, game);
            let mut v3: u16 = 0u16;
            for _ in 0i16..15i16 {
                let v5: i32 = v3 as i32 >> 1i32;
                set_ram_words! { game, 0u16;
                    target_palettes[(v5 + 113i32)], target_palettes[(v5 + 241i32)],
                }
                v3 = (v3).wrapping_add(2u16);
            }
        } else {
            (*E).base.properties = ((*E).base.properties as i32
                | (kEnemyProps_BlockPlasmaBeam as i32 | kEnemyProps_Tangible as i32))
                as u16;
            (*E).base.x_pos = 186u16;
            (*E).base.y_pos = 169u16;
            set_ram_word!(game.ceres_status = 0u16);
            set_fields! {[(*E)], 0u16; cry_var_10, cry_var_02}
            set_ram_word!(game.tilemap_stuff[1] = 1u16);
            (*E).cry_var_1C = 15u16;
            (*E).cry_var_A = ((0xa6a35bi32) as u16);
            set_fields! {[(*E)], 0u16; cry_var_B, cry_var_C}
            (*E).cry_var_20 = -32i32 as u16;
            (*E).cry_var_21 = 176u16;
            (*E).cry_var_22 = 40u16;
            (*E).cry_var_23 = 224u16;
            WriteColorsToTargetPalette(0xa6u8, 0x140u16, addr_word_A6E16F as u16, 0x20u16, game);
            let mut v7: u16 = 482u16;
            for _ in 0i16..15i16 {
                set_ram_word!(game.target_palettes[(v7 >> 1)] = 0u16);
                v7 = (v7).wrapping_add(2u16);
            }
            (*E).cry_var_03 = addr_kBabyMetroid_Ilist_BF31 as u16;
            (*E).cry_var_04 = 1u16;
            (*gRam8800_Default(0i32, game)).var_40 = ((0xa6be9ci32) as u16);
            (*E).cry_var_07 = 5u16;
            QueueMusic_Delayed8(0u16, game);
        }
    };
}
crate::game::address_dispatch! {
    pub unsafe fn CallRidleyFunc(ea: u32, game: &mut Game) => ea;
        call (game); default { Unreachable(game); };
    {
        10920788 => CeresRidley_Func_2; 10920795 => CeresRidley_Func_3;
        10920823 => CeresRidley_A377; 10920841 => CeresRidley_Func_4;
        10920927 => CeresRidley_Func_5; 10921045 => CeresRidley_Func_6;
        10921080 => CeresRidley_Func_7; 10921647 => CeresRidley_Func_9;
        10921672 => CeresRidley_Func_10; 10921704 => CeresRidley_Func_11;
        10921858 => CeresRidley_Func_13; 10921977 => CeresRidley_A7F9;
        10922044 => CeresRidley_Func_14; 10922062 => CeresRidley_Func_15;
        10922125 => CeresRidley_Func_16; 10922148 => CeresRidley_Func_17;
        10922196 => CeresRidley_Func_18; 10922232 => CeresRidley_Func_19;
        10922275 => CeresRidley_Func_20; 10922311 => CeresRidley_Func_21;
        10922353 => CeresRidley_Func_22; 10922400 => CeresRidley_Func_23;
        10922513 => CeresRidley_Func_24; 10922575 => [()]; 10922576 => CeresRidley_Func_26;
        10924787 => Ridley_Func_3; 10924833 => Ridley_Func_3b; 10925036 => Ridley_Func_5;
        10925048 => Ridley_Func_6; 10925121 => Ridley_Func_8; 10925141 => Ridley_B455;
        10925203 => Ridley_Func_9; 10925265 => Ridley_Func_10; 10925334 => Ridley_Func_11;
        10925396 => Ridley_Func_12; 10925460 => Ridley_Func_13; 10925508 => Ridley_Func_14;
        10925541 => Ridley_Func_15; 10925587 => Ridley_Func_16; 10925735 => Ridley_Func_19;
        10925789 => Ridley_Func_20; 10925838 => Ridley_Func_21; 10926009 => Ridley_Func_22;
        10926775 => Ridley_Func_33; 10926991 => Ridley_Func_33b; 10927044 => Ridley_Func_34;
        10927089 => Ridley_Func_35; 10927150 => Ridley_Func_36; 10927438 => Ridley_Func_43;
        10927514 => Ridley_Func_44; 10927548 => Ridley_Func_45; 10927602 => Ridley_Func_46;
        10927619 => Ridley_Func_47; 10928206 => Ridley_Func_54; 10929464 => Ridley_Func_63;
        10929470 => Ridley_C53E; 10929489 => Ridley_C551; 10929544 => Ridley_Func_64;
        10929576 => Ridley_Func_65; 10929608 => Ridley_Func_66; 10929626 => Ridley_Func_67;
        10929664 => [()];
    }
}
pub unsafe fn CeresRidley_Main(game: &mut Game) {
    let E = Get_CeresRidley(0i32, game);
    (*E).base.health = 0x7fffu16;
    CallRidleyFunc((((*E).cry_var_A) as u32) | (0xa60000u32), game);
    if ram_word!(game, ceres_status) == 0 {
        if (*Get_CeresRidley(0i32, game)).cry_var_02 != 0 {
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
    if ram_word!(game, random_number) >= 0xff00u16 {
        set_ram_word!(
            game.tilemap_stuff[15] =
                ((ram_word!(game, random_number) as i32 & 0xfi32) + 8i32) as u16
        );
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
    if (*Get_CeresRidley(0i32, game)).cry_var_02 != 0 {
        mov24(ram_ptr!(game, LongPtr; 0x178c), 0xa6a2f2u32);
    } else {
        Ridley_A2F2(game);
    };
}
pub unsafe fn Ridley_A2F2(game: &mut Game) {
    if ram_word!(game, ceres_status) == 0 {
        DrawBabyMetroid_0(game);
    }
    let E = Get_CeresRidley(0x40i32, game);
    if (*E).cry_var_B != 0 {
        static g_word_A6A321: [i16; 4] = [0, -1024, -4, -1];
        DrawSpritemap(
            0xa6u8,
            addr_kCeresRidley_Sprmap_A329 as u16,
            ((*E).base.x_pos).wrapping_add(
                (g_word_A6A321[(ram_word!(game, earthquake_timer) as i32 & 3i32) as usize]) as u16,
            ),
            (*E).base.y_pos,
            1024u16,
            game,
        );
    }
}
pub unsafe fn CeresRidley_Func_2(game: &mut Game) {
    let E = Get_CeresRidley(0i32, game);
    set_fields! {[(*E)], 0u16; cry_var_B, cry_var_C}
}
pub unsafe fn CeresRidley_Func_3(game: &mut Game) {
    if (ram_word!(game, door_transition_flag_enemies) == 0) {
        let E = Get_CeresRidley(0i32, game);
        (*E).cry_var_A = ((0xa6a377i32) as u16);
        let mut v1: i16 = 512i16;
        if ram_word!(game, area_index) == 2u16 {
            v1 = 170i16;
        }
        (*E).cry_var_F = v1 as u16;
        CeresRidley_A377(game);
    }
}
pub unsafe fn CeresRidley_A377(game: &mut Game) {
    let E = Get_CeresRidley(0i32, game);
    (*E).cry_var_F = (*E).cry_var_F.wrapping_sub(1);
    if ((*E).cry_var_F & 0x8000) != 0 {
        (*E).cry_var_A = ((0xa6a389i32) as u16);
        set_fields! {[(*E)], 0u16; cry_var_E, cry_var_F}
    }
}
pub unsafe fn CeresRidley_Func_4(game: &mut Game) {
    let E = Get_CeresRidley(0i32, game);
    if ((*E).cry_var_E & 0x8000) == 0 {
        let cry_var_E: u16 = (*E).cry_var_E;
        (*E).cry_var_F = (*E).cry_var_F.wrapping_add(1);
        if (*E).cry_var_F != 0 {
            (*E).cry_var_F = 0u16;
            let mut v2: i16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa6e269)) as *mut u8)
                .offset(cry_var_E as isize) as i16;
            if v2 == 255i16 {
                (*E).cry_var_E = 0u16;
                (*E).cry_var_A = ((0xa6a3dfi32) as u16);
                (*E).cry_var_02 = 1u16;
            } else {
                (*E).cry_var_E = (cry_var_E).wrapping_add(1u16);
                let v3: i32 = ((6u16).wrapping_mul((v2) as u16)) as i32 >> 1i32;
                set_ram_word!(
                    game.palette_buffer[252] =
                        crate::rtl::rom_table_word!(game, 0xa6e2aa, v3 as isize)
                );
                set_ram_word!(
                    game.palette_buffer[253] =
                        crate::rtl::rom_table_word!(game, 0xa6e2aa, (v3 + 1i32) as isize)
                );
                set_ram_word!(
                    game.palette_buffer[254] =
                        crate::rtl::rom_table_word!(game, 0xa6e2aa, (v3 + 2i32) as isize)
                );
            }
        }
    }
}
pub unsafe fn CeresRidley_Func_5(game: &mut Game) {
    let E = Get_CeresRidley(0i32, game);
    (*E).cry_var_F = (*E).cry_var_F.wrapping_add(1);
    if (*E).cry_var_F >= 2u16 {
        (*E).cry_var_F = 0u16;
        let mut cry_var_E: u16 = (*E).cry_var_E;
        let mut r18: u16 = 290u16;
        let mut r20: u16 = 482u16;
        let mut v2: i16 = 11i16;
        loop {
            let mut v5: i16 = v2;
            let v3: u16 = crate::rtl::rom_table_word!(game, 0xa6e30a, (cry_var_E >> 1) as isize);
            set_ram_word!(game.palette_buffer[(r18 >> 1)] = v3);
            set_ram_word!(game.palette_buffer[(r20 >> 1)] = v3);
            cry_var_E = (cry_var_E).wrapping_add(2u16);
            r18 = (r18).wrapping_add(2u16);
            r20 = (r20).wrapping_add(2u16);
            v2 = (v5).wrapping_sub(1i16);
            if (v5 == 1i16) {
                break;
            }
        }
        if cry_var_E >= 0x160u16 {
            if ram_word!(game, area_index) == 2u16 {
                (*E).base.layer = 2u16;
            }
            (*E).base.properties =
                ((*E).base.properties as i32 & !(kEnemyProps_Tangible as i32)) as u16;
            (*E).cry_var_E = 0u16;
            (*E).cry_var_A = ((0xa6a455i32) as u16);
            (*E).cry_var_F = 4u16;
            QueueMusic_Delayed8(5u16, game);
        } else {
            (*E).cry_var_E = cry_var_E;
        }
    }
}
pub unsafe fn CeresRidley_Func_6(game: &mut Game) {
    let E = Get_CeresRidley(0i32, game);
    (*E).cry_var_F = (*E).cry_var_F.wrapping_sub(1);
    if ((*E).cry_var_F & 0x8000) != 0 {
        Ridley_Func_99(addr_kRidley_Ilist_E690 as u16, game);
        (*E).cry_var_F = 0u16;
        (*E).cry_var_A = ((0xa6a478i32) as u16);
        if ram_word!(game, area_index) != 2u16 {
            (*E).cry_var_F = 252u16;
        }
    }
}
pub unsafe fn CeresRidley_Func_7(game: &mut Game) {
    let E = Get_CeresRidley(0i32, game);
    if ram_word!(game, area_index) == 2u16 {
        (*E).cry_var_F = (*E).cry_var_F.wrapping_sub(1);
        let v1: bool = ((*E).cry_var_F & 0x8000) != 0;
        if !v1 {
            return;
        }
        (*E).cry_var_F = 2u16;
        let v2: u16 = ((*E).cry_var_E).wrapping_add(1u16);
        (*E).cry_var_E = v2;
        if CeresRidley_Func_8((v2).wrapping_sub(1u16), game) & 1 == 0 {
            return;
        }
        set_ram_word!(game.fx_target_y_pos = 440u16);
        set_ram_word!(game.fx_y_vel = -96i32 as u16);
        set_ram_word!(game.fx_timer = 32u16);
    } else {
        (*E).cry_var_F = (*E).cry_var_F.wrapping_sub(1);
        let v1_0: bool = ((*E).cry_var_F & 0x8000) != 0;
        if !v1_0 {
            return;
        }
    }
    (*E).cry_var_E = 0u16;
    Ridley_Func_99(addr_kRidley_Ilist_E91D as u16, game);
    set_fields! {[(*E)], 8u16; cry_var_08, cry_var_09}
    Ridley_Func_95(game);
    set_ram_word!(game.tilemap_stuff[0] = 1u16);
    (*E).cry_var_A = ((0xa6a354i32) as u16);
}
pub unsafe fn CeresRidley_Func_8(a: u16, game: &mut Game) -> u8 {
    let v1: u16 = crate::rtl::rom_table_word!(game, 0xa6a4eb, a as isize);
    if v1 == 0 {
        return 1u8;
    }
    WriteColorsToPalette(0xe2u16, 0xa6u8, v1, 0xeu16, game);
    0u8
}
pub unsafe fn CeresRidley_Func_9(game: &mut Game) {
    let E = Get_CeresRidley(0i32, game);
    (*E).cry_var_C = ((*E).cry_var_C).wrapping_sub(16u16);
    if (*E).base.y_pos as i32 - 112i32 & 0x8000i32 != 0 {
        (*E).cry_var_A = ((0xa6a6c8i32) as u16);
        CeresRidley_Func_10(game);
    }
}
pub unsafe fn CeresRidley_Func_10(game: &mut Game) {
    let E = Get_CeresRidley(0i32, game);
    (*E).cry_var_C = ((*E).cry_var_C).wrapping_add(20u16);
    if (*E).base.y_pos as i32 - 80i32 & 0x8000i32 != 0 {
        (*E).cry_var_A = ((0xa6a6e8i32) as u16);
        (*E).cry_var_01 = 1u16;
    }
}
pub unsafe fn CeresRidley_Func_11(game: &mut Game) {
    let E = Get_CeresRidley(0i32, game);
    if (*E).cry_var_0D >= 0x64u16 {
        (*E).cry_var_01 = 0u16;
        (*E).cry_var_A = ((0xa6bd9ai32) as u16);
        Ridley_Func_44(game);
    } else if ram_word!(game, samus_health) as i32 - 30i32 & 0x8000i32 != 0 {
        (*E).cry_var_01 = 0u16;
        (*E).cry_var_A = ((0xa6a971i32) as u16);
        CeresRidley_Func_22(game);
    } else if CeresRidley_Func_12(game) & 1 == 0 || {
        let mut v1: u16 = ((*E).cry_var_00).wrapping_add(1u16);
        (*E).cry_var_00 = v1;
        v1 >= 0x7cu16
    } {
        (*E).cry_var_A = crate::rtl::rom_table_word!(
            game,
            0xa6a743,
            (ram_word!(game, random_number) as i32 & 0xfi32) as isize
        );
        (*E).cry_var_00 = 0u16;
    }
}
pub unsafe fn CeresRidley_Func_12(game: &mut Game) -> u8 {
    let rect: Rect16U = Rect16U {
        x: 192u16,
        y: 100u16,
        w: 8u16,
        h: 8u16,
    };
    Ridley_Func_106(0u16, 0u16, rect.x, rect.y, game);
    Shitroid_Func_2(0u16, rect, game)
}
pub unsafe fn CeresRidley_Func_13(game: &mut Game) {
    let E = Get_CeresRidley(0i32, game);
    let mut v1: u16 = abs16((*E).cry_var_C);
    if v1 < 0x80u16 {
        v1 = 128u16;
    }
    (*E).cry_var_C = (if (*E).cry_var_C & 0x8000 != 0 {
        -(v1 as i32)
    } else {
        v1 as i32
    }) as u16;
    Ridley_Func_106(0u16, 0u16, (*E).base.x_pos, 88u16, game);
    if (*E).base.y_pos as i32 - 80i32 & 0x8000i32 != 0 {
        let v3: u16 = ((*E).cry_var_00).wrapping_add(1u16);
        (*E).cry_var_00 = v3;
        if v3 >= 0x30u16 {
            (*E).cry_var_A = ((0xa6a88di32) as u16);
        }
    } else if (*E).base.y_pos as i32 - 128i32 & 0x8000i32 != 0 {
        (*E).cry_var_17 = (*E).base.x_pos;
        (*E).cry_var_18 = (*E).base.y_pos;
        Ridley_Func_99(addr_kRidley_Ilist_E73A as u16, game);
        (*E).cry_var_A = addr_loc_A6A7F9 as u16;
        (*E).cry_var_00 = 224u16;
        CeresRidley_A7F9(game);
    }
}
pub unsafe fn CeresRidley_A7F9(game: &mut Game) {
    let r22: u16 = (if ram_word!(game, random_number) & 0x8000 != 0 {
        -(ram_word!(game, random_number) as i32 & 7i32)
    } else {
        ram_word!(game, random_number) as i32 & 7i32
    }) as u16;
    let E = Get_CeresRidley(0i32, game);
    let r18: u16 = ((*E).cry_var_17).wrapping_add(r22);
    let r20: u16 = ((*E).cry_var_18).wrapping_add(r22);
    Ridley_Func_106(0u16, 0u16, r18, r20, game);
    let v4: i16 = (((*E).cry_var_00) as i16).wrapping_sub(1i16);
    (*E).cry_var_00 = v4 as u16;
    if v4 < 0i16 {
        (*E).cry_var_00 = 0u16;
        (*E).cry_var_A = ((0xa6a6e8i32) as u16);
    }
}
pub unsafe fn CeresRidley_Func_14(game: &mut Game) {
    Ridley_Func_99(addr_kRidley_Ilist_E548 as u16, game);
    let E = Get_CeresRidley(0i32, game);
    (*E).cry_var_A = ((0xa6a84ei32) as u16);
    (*E).cry_var_F = 64u16;
    CeresRidley_Func_15(game);
}
pub unsafe fn CeresRidley_Func_15(game: &mut Game) {
    let E = Get_CeresRidley(0i32, game);
    let r18: u16 = ram_word!(game, samus_x_pos);
    let mut v0: u16 = (ram_word!(game, samus_y_pos)).wrapping_sub(68u16);
    if ram_word!(game, samus_y_pos) as i32 - 132i32 & 0x8000i32 != 0 {
        v0 = 64u16;
    }
    let r20: u16 = v0;
    Ridley_Func_106(0u16, 0xdu16, r18, r20, game);
    let rect: Rect16U = Rect16U {
        x: r18,
        y: r20,
        w: 2u16,
        h: 2u16,
    };
    if Shitroid_Func_2(0u16, rect, game) == 0 || {
        let mut v2: bool = (((*E).cry_var_F) as i16).wrapping_sub(1i16) < 0i16;
        (*E).cry_var_F = (*E).cry_var_F.wrapping_sub(1);
        v2 as i32 != 0
    } {
        (*E).cry_var_00 = 0u16;
        (*E).cry_var_A = ((0xa6a6e8i32) as u16);
    }
}
pub unsafe fn CeresRidley_Func_16(game: &mut Game) {
    let E = Get_CeresRidley(0i32, game);
    (*E).cry_var_A = ((0xa6a8a4i32) as u16);
    (*E).cry_var_F = 10u16;
    (*E).cry_var_0A = 0u16;
    set_ram_word!(game.tilemap_stuff[1] = 0u16);
    CeresRidley_Func_17(game);
}
pub unsafe fn CeresRidley_Func_17(game: &mut Game) {
    Ridley_Func_106(0u16, 1u16, 192u16, 80u16, game);
    let E = Get_CeresRidley(0i32, game);
    if (*E).base.y_pos as i32 - 96i32 & 0x8000i32 != 0 {
        (*E).cry_var_A = ((0xa6a8d4i32) as u16);
        (*E).cry_var_F = 10u16;
        (*E).cry_var_0A = 0u16;
    }
}
pub unsafe fn CeresRidley_Func_18(game: &mut Game) {
    Ridley_Func_111(-32i32 as u16, -1024i32 as u16, 768u16, game);
    let E = Get_CeresRidley(0i32, game);
    (*E).cry_var_F = (*E).cry_var_F.wrapping_sub(1);
    if ((*E).cry_var_F & 0x8000) != 0 {
        (*E).cry_var_A = ((0xa6a8f8i32) as u16);
        (*E).cry_var_F = 36u16;
    }
}
pub unsafe fn CeresRidley_Func_19(game: &mut Game) {
    Ridley_Func_111(-512i32 as u16, -16384i32 as u16, 768u16, game);
    let E = Get_CeresRidley(0i32, game);
    (*E).cry_var_F = (*E).cry_var_F.wrapping_sub(1);
    if ((*E).cry_var_F & 0x8000) != 0 {
        (*E).cry_var_A = ((0xa6a923i32) as u16);
        (*E).cry_var_F = 28u16;
        set_ram_word!(game.tilemap_stuff[2] = 1u16);
    }
}
pub unsafe fn CeresRidley_Func_20(game: &mut Game) {
    Ridley_Func_111(-512i32 as u16, -30720i32 as u16, 768u16, game);
    let E = Get_CeresRidley(0i32, game);
    (*E).cry_var_F = (*E).cry_var_F.wrapping_sub(1);
    if ((*E).cry_var_F & 0x8000) != 0 {
        (*E).cry_var_A = ((0xa6a947i32) as u16);
        (*E).cry_var_F = 1u16;
    }
}
pub unsafe fn CeresRidley_Func_21(game: &mut Game) {
    Ridley_Func_111(-768i32 as u16, -30720i32 as u16, 768u16, game);
    let E = Get_CeresRidley(0i32, game);
    (*E).cry_var_F = (*E).cry_var_F.wrapping_sub(1);
    if ((*E).cry_var_F & 0x8000) != 0 {
        (*E).cry_var_A = ((0xa6a6e8i32) as u16);
        (*E).cry_var_00 = 0u16;
        set_ram_word!(game.tilemap_stuff[1] = 1u16);
    }
}
pub unsafe fn CeresRidley_Func_22(game: &mut Game) {
    let E = Get_CeresRidley(0i32, game);
    (*E).cry_var_20 = -192i32 as u16;
    Ridley_Func_106(0u16, 1u16, 192u16, -128i32 as u16, game);
    if (*E).base.y_pos as i32 + 128i32 & 0x8000i32 != 0 {
        CeresRidley_A994(game);
    }
}
pub unsafe fn CeresRidley_A994(game: &mut Game) {
    let E = Get_CeresRidley(0i32, game);
    (*E).cry_var_A = ((0xa6a9a0i32) as u16);
    (*E).cry_var_F = 64u16;
    CeresRidley_Func_23(game);
}
pub unsafe fn CeresRidley_Func_23(game: &mut Game) {
    let E = Get_CeresRidley(0i32, game);
    (*E).cry_var_F = (*E).cry_var_F.wrapping_sub(1);
    if ((*E).cry_var_F & 0x8000) != 0 {
        CeresRidley_Func_25(game);
        set_fields! {[(*E)], 0u16; cry_var_B, cry_var_C}
        set_ram_word!(game.tilemap_stuff[0] = 0u16);
        (*E).cry_var_A = ((0xa6aa11i32) as u16);
        WriteColorsToPalette(0xa2u16, 0xa6u8, addr_word_A6A9E3 as u16, 0xfu16, game);
        WriteColorsToPalette(0x42u16, 0xa6u8, addr_word_A6AA01 as u16, 8u16, game);
        WriteColorsToPalette(0x1e2u16, 0xa6u8, addr_word_A6AA01 as u16, 8u16, game);
    }
}
pub unsafe fn CeresRidley_Func_24(game: &mut Game) {
    (*Get_CeresRidley(0i32, game)).cry_var_A = ((0xa6aa4fi32) as u16);
    set_ram_word!(game.ceres_status = 1u16);
    CeresRidley_Func_27(game);
}
pub unsafe fn CeresRidley_Func_25(game: &mut Game) {
    SpawnEnemy(0xa6u8, addr_stru_A6AA2F as u16, game);
    SpawnEnemy(0xa6u8, addr_stru_A6AA3F as u16, game);
}
pub unsafe fn CeresRidley_Func_26(game: &mut Game) {
    Ridley_Func_61(game);
}
pub unsafe fn CeresRidley_Func_27(game: &mut Game) {
    *ram_ptr!(game; 0x56) = 7u8;
    set_ram_word!(game.irq_enable_mode7 = 1u16);
    *ram_ptr!(game; 0x5f) = 0x80u8;
    set_ram_words! { game, 256u16; reg_M7A, reg_M7B, reg_M7C, reg_M7D }
    set_ram_words! { game, 64u16; reg_M7X, reg_M7Y }
    let E = Get_CeresRidley(0i32, game);
    set_fields! {[(*E)], 0u16; cry_var_32, cry_var_33}
    (*E).cry_var_0A = 1u16;
    (*E).cry_var_34 = -128i32 as u16;
    set_ram_word!(game.reg_BG1HOFS = -128i32 as u16);
    (*E).cry_var_35 = 32u16;
    set_ram_word!(game.reg_BG1VOFS = 32u16);
    (*E).cry_var_30 = 2048u16;
    (*E).cry_var_31 = 1024u16;
}
pub unsafe fn HandleCeresRidleyGetawayCutscene(game: &mut Game) {
    if (*Get_CeresRidley(0i32, game)).cry_var_32 == 0 {
        CeresRidley_Func_28(game);
    }
}
pub unsafe fn CeresRidley_Func_28(game: &mut Game) {
    let E = Get_CeresRidley(0i32, game);
    crate::update_place! { previous cry_var_33: u16 = (*E).cry_var_33 => (cry_var_33).wrapping_add(2u16); }
    if cry_var_33 == 0 {
        QueueSfx2_Max6(0x4eu16, game);
    }
    if cry_var_33 == 208u16 {
        Samus_SetPushedOutOfCeresRidley(game);
        set_ram_word!(game.earthquake_type = 35u16);
        set_ram_word!(game.earthquake_timer = 64u16);
    }
    let v2: i32 = cry_var_33 as i32 >> 1i32;
    if crate::rtl::rom_table_word!(game, 0xa6ae4d, v2 as isize) == 0xffffu16 {
        (*E).cry_var_32 = (*E).cry_var_32.wrapping_add(1);
        *ram_ptr!(game; 0x56) = 9u8;
        *ram_ptr!(game; 0x5f) = 0u8;
        set_ram_words! { game, 0u16;
            reg_M7A, reg_M7B, reg_M7C, reg_M7D, reg_M7X, reg_M7Y, reg_BG1HOFS, reg_BG1VOFS,
        }
        (*E).cry_var_A = ((0xa6c04ei32) as u16);
        (*E).cry_var_F = 0u16;
    } else {
        (*E).cry_var_30 = crate::rtl::rom_table_word!(game, 0xa6ae4d, v2 as isize);
        let v3: u16 = (crate::rtl::rom_table_word!(game, 0xa6af2f, v2 as isize))
            .wrapping_add((*E).cry_var_35);
        (*E).cry_var_35 = v3;
        set_ram_word!(game.reg_BG1VOFS = v3);
        let v4: u16 = ((*E).cry_var_34).wrapping_sub(crate::rtl::rom_table_word!(
            game,
            0xa6b00f,
            v2 as isize
        ));
        (*E).cry_var_34 = v4;
        set_ram_word!(game.reg_BG1HOFS = v4);
        CeresRidley_Func_35((*E).cry_var_30, game);
        (*E).cry_var_0A = ((*E).cry_var_0A).wrapping_add(48u16);
        CeresRidley_Func_29(game);
        CeresRidley_Func_33(game);
        CeresRidley_Func_34(game);
    };
}
unsafe fn CeresMult(a: u16, b: u16) -> u16 {
    let r: u16 = (abs16(a) as i32 * b as i32 >> 8i32) as u16;
    ((if a & 0x8000 != 0 {
        -(r as i32)
    } else {
        r as i32
    }) as u16)
}
pub unsafe fn CeresRidley_Func_29(game: &mut Game) {
    let E = Get_CeresRidley(0i32, game);
    let r18: u16 = *(&raw mut (*E).cry_var_0A as *mut u8).offset(1isize) as u16;
    set_ram_word!(
        game.reg_M7A = CeresMult(
            CeresRidley_AC30((r18).wrapping_add(64u16), 0x100i16) as u16,
            (*E).cry_var_30,
        )
    );
    set_ram_word!(
        game.reg_M7B = CeresMult(CeresRidley_AC30(r18, 0x100i16) as u16, (*E).cry_var_30,)
    );
    set_ram_word!(
        game.reg_M7C = CeresMult(
            -(CeresRidley_AC30(r18, 0x100i16) as i32) as u16,
            (*E).cry_var_30,
        )
    );
    set_ram_word!(
        game.reg_M7D = CeresMult(
            CeresRidley_AC30((r18).wrapping_add(64u16), 0x100i16) as u16,
            (*E).cry_var_30,
        )
    );
}
pub unsafe fn CeresRidley_AC30(a: u16, r20: i16) -> i16 {
    (kSinCosTable8bit_Sext[((a as i32 & 0xffi32) + 64i32) as usize] as i32 * r20 as i32 >> 8i32)
        as i16
}
pub unsafe fn CeresRidley_Func_33(game: &mut Game) {
    if (ram_word!(game, nmi_frame_counter_word) & 3) == 0 {
        let E = Get_CeresRidley(0i32, game);
        let v1: u16 = ((*E).cry_var_36 as i32 + 1i32 & 3i32) as u16;
        (*E).cry_var_36 = v1;
        QueueMode7Transfers(
            0xa6u8,
            crate::rtl::rom_table_word!(game, 0xa6acda, v1 as isize),
            game,
        );
    }
}
pub unsafe fn CeresRidley_Func_34(game: &mut Game) {
    if (ram_word!(game, nmi_frame_counter_word) & 7) == 0 {
        let E = Get_CeresRidley(0i32, game);
        let v1: u16 = ((*E).cry_var_37 as i32 + 1i32 & 1i32) as u16;
        (*E).cry_var_37 = v1;
        QueueMode7Transfers(
            0xa6u8,
            crate::rtl::rom_table_word!(game, 0xa6ad45, v1 as isize),
            game,
        );
    }
}
pub unsafe fn CeresRidley_Func_35(mut a: u16, game: &mut Game) {
    WriteColorsToPalette(
        0xa2u16,
        0xa6u8,
        (32i32 * *(&raw mut a as *mut u8).offset(1isize) as i32 - 0x4ef9i32) as u16,
        0xfu16,
        game,
    );
}
pub unsafe fn Ridley_Main(game: &mut Game) {
    let E = Get_Ridley(0i32, game);
    let mut v1: i16 = (((*E).ridley_var_24) as i16).wrapping_sub(4i16);
    if v1 < 0i16 {
        v1 = 0i16;
    }
    (*E).ridley_var_24 = v1 as u16;
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
    let E = Get_Ridley(0i32, game);
    if (*E).ridley_var_1B | (*E).ridley_var_1E == 0 && Ridley_Func_26(4u16, 4u16, game) & 1 != 0 {
        Ridley_Func_37(game);
    }
}
pub unsafe fn Ridley_Func_2(game: &mut Game) {
    let E = Get_Ridley(0i32, game);
    (*E).ridley_var_24 = 0u16;
    (*E).base.frame_counter = 1u16;
    Ridley_Hurt(game);
}
pub unsafe fn Ridley_Hurt(game: &mut Game) {
    let E = Get_Ridley(0i32, game);
    let update_hurt_animation: bool = !(((*E).base.frame_counter & 1) == 0)
        || ({
            Ridley_Func_39(game);
            Ridley_Func_42(game);
            CallRidleyFunc((((*E).ridley_var_A) as u32) | (0xa60000u32), game);
            !((*Get_Ridley(0i32, game)).ridley_var_02 == 0)
                && ({
                    Ridley_Func_112(game);
                    Ridley_Func_115(game);
                    Ridley_Func_70(game);
                    true
                })
        });
    if update_hurt_animation {
        Ridley_Func_129(game);
        Ridley_Func_102(game);
        sub_A6DB2A(game);
        Ridley_Func_120(game);
        Ridley_Func_100(game);
        Ridley_Func_118(game);
        let mut v2: u16 = ((*E).ridley_var_24).wrapping_add(1u16);
        if ((v2) as i16).wrapping_sub((g_word_A6B288) as i16) >= 0i16 {
            v2 = g_word_A6B288;
        }
        (*E).ridley_var_24 = v2;
        if ((((*E).ridley_var_24) as i16).wrapping_sub((g_word_A6B288) as i16) < 0i16) {
            return;
        }
    }
    if (*Get_Ridley(0i32, game)).ridley_var_1B != 0 {
        Ridley_Func_30(game);
    }
}
pub unsafe fn Ridley_Func_3(game: &mut Game) {
    let rect: Rect16U = Rect16U {
        x: 64u16,
        y: 256u16,
        w: 8u16,
        h: 8u16,
    };
    Ridley_Func_104_0(0u16, 14u16, rect.x, rect.y, game);
    if Shitroid_Func_2(0u16, rect, game) == 0 {
        let E = Get_Ridley(0i32, game);
        (*E).ridley_var_01 = 1u16;
        (*E).ridley_var_A = ((0xa6b321i32) as u16);
        Ridley_Func_3b(game);
    }
}
pub unsafe fn Ridley_Func_3b(game: &mut Game) {
    let r18: u16 = Ridley_Func_4(game);
    let v0: i32 = 2i32 * (NextRandom(game) as i32 & 7i32);
    let v1: u16 = *((RomPtr_A6 as unsafe fn(u16, &mut Game) -> *const u8)(r18, game)
        .offset(v0 as isize) as *mut u16);
    let E = Get_Ridley(0i32, game);
    (*E).ridley_var_A = v1;
    CallRidleyFunc((((*E).ridley_var_A) as u32) | (0xa60000u32), game);
}
pub unsafe fn Ridley_Func_4(game: &mut Game) -> u16 {
    let mut r18: u16 = 0;
    if *ram_ptr!(game; 0xa1f) == 3u8 {
        r18 = addr_off_A6B3CC as u16;
    } else {
        let E = Get_Ridley(0i32, game);
        let health: u16 = (*E).base.health;
        if health != 0 {
            if health as i32 - 14400i32 & 0x8000i32 != 0 {
                r18 = addr_off_A6B38C as u16;
            } else if Ridley_Func_41(game) & 1 != 0 {
                r18 = addr_off_A6B3BC as u16;
            } else if (Ridley_Func_40_Carry(game) & 1) != 0 {
                r18 = addr_off_A6B3AC as u16;
            } else {
                r18 = (if (*E).base.health as i32 - 9000i32 & 0x8000i32 != 0 {
                    addr_off_A6B39C as i32
                } else {
                    addr_off_A6B38C as i32
                }) as u16;
            }
        } else {
            r18 = addr_off_A6B3DC as u16;
            (*E).ridley_var_25 = (*E).ridley_var_25.wrapping_add(1);
        }
    }
    r18
}
pub unsafe fn Ridley_Func_5(game: &mut Game) {
    let E = Get_Ridley(0i32, game);
    (*E).ridley_var_A = ((0xa6b3f8i32) as u16);
    (*E).ridley_var_F = 128u16;
    Ridley_Func_6(game);
}
pub unsafe fn Ridley_Func_6(game: &mut Game) {
    let mut rect: Rect16U = Rect16U {
        x: 0,
        y: 0,
        w: 0,
        h: 0,
    };

    let E = Get_Ridley(0i32, game);
    (*E).ridley_var_F = (*E).ridley_var_F.wrapping_sub(1);
    let change_ridley_state: bool = (((*E).ridley_var_F & 0x8000) != 0)
        || ({
            let mut v2: u16 = 0;
            v2 = 192u16;
            if (*E).ridley_var_10 != 0 {
                v2 = 96u16;
            }
            rect = Rect16U {
                x: v2,
                y: 256u16,
                w: 8u16,
                h: 8u16,
            };
            Ridley_Func_104_0(0u16, Ridley_Func_7(game), rect.x, rect.y, game);
            (Shitroid_Func_2(0u16, rect, game) == 0)
        });
    if change_ridley_state {
        (*E).ridley_var_A = ((0xa6b321i32) as u16);
    };
}
pub unsafe fn Ridley_Func_7(game: &mut Game) -> u16 {
    static g_word_A6B439: [u16; 4] = [4, 8, 0xa, 0xc];
    g_word_A6B439[(*Get_Ridley(0i32, game)).ridley_var_12 as usize]
}
pub unsafe fn Ridley_Func_8(game: &mut Game) {
    let E = Get_Ridley(0i32, game);
    (*E).ridley_var_A = addr_loc_A6B455 as u16;
    (*E).ridley_var_00 = 10u16;
    (*E).ridley_var_0A = 0u16;
    Ridley_B455(game);
}
pub unsafe fn Ridley_B455(game: &mut Game) {
    let E = Get_Ridley(0i32, game);
    let mut v1: u16 = 192u16;
    if (*E).ridley_var_10 != 0 {
        v1 = 64u16;
    }
    let rect: Rect16U = Rect16U {
        x: v1,
        y: 128u16,
        w: 8u16,
        h: 8u16,
    };
    Ridley_Func_104_0(0u16, 1u16, rect.x, rect.y, game);
    if Shitroid_Func_2(0u16, rect, game) == 0 {
        (*E).ridley_var_A = ((0xa6b493i32) as u16);
        (*E).ridley_var_00 = 32u16;
        (*E).ridley_var_0A = 0u16;
    }
}
pub unsafe fn Ridley_Func_9(game: &mut Game) {
    let E = Get_Ridley(0i32, game);
    if (*E).ridley_var_10 != 0 {
        Ridley_Func_111(32u16, 512u16, 1152u16, game);
    } else {
        Ridley_Func_111(-32i32 as u16, -512i32 as u16, 1152u16, game);
    }
    let ridley_var_00: u16 = (*E).ridley_var_00;
    if ridley_var_00 != 0 {
        (*E).ridley_var_00 = (ridley_var_00).wrapping_sub(1u16);
    } else {
        (*E).ridley_var_A = ((0xa6b4d1i32) as u16);
        (*E).ridley_var_00 = 20u16;
    };
}
pub unsafe fn Ridley_Func_10(game: &mut Game) {
    let E = Get_Ridley(0i32, game);
    if (*E).ridley_var_10 != 0 {
        Ridley_Func_111(320u16, 0x4000u16, 1280u16, game);
    } else {
        Ridley_Func_111(-320i32 as u16, -16384i32 as u16, 1280u16, game);
    }
    let ridley_var_00: u16 = (*E).ridley_var_00;
    if ridley_var_00 != 0 {
        (*E).ridley_var_00 = (ridley_var_00).wrapping_sub(1u16);
    } else {
        (*E).ridley_var_A = ((0xa6b516i32) as u16);
        (*E).ridley_var_00 = 16u16;
        set_ram_word!(game.tilemap_stuff[2] = 1u16);
    };
}
pub unsafe fn Ridley_Func_11(game: &mut Game) {
    let E = Get_Ridley(0i32, game);
    if (*E).ridley_var_10 != 0 {
        Ridley_Func_111(512u16, 30720u16, 768u16, game);
    } else {
        Ridley_Func_111(-512i32 as u16, -30720i32 as u16, 768u16, game);
    }
    let ridley_var_00: u16 = (*E).ridley_var_00;
    if ridley_var_00 != 0 {
        (*E).ridley_var_00 = (ridley_var_00).wrapping_sub(1u16);
    } else {
        (*E).ridley_var_A = ((0xa6b554i32) as u16);
        (*E).ridley_var_00 = 32u16;
    };
}
pub unsafe fn Ridley_Func_12(game: &mut Game) {
    let E = Get_Ridley(0i32, game);
    if (*E).ridley_var_10 != 0 {
        Ridley_Func_111(1024u16, 30720u16, 768u16, game);
    } else {
        Ridley_Func_111(-1024i32 as u16, -30720i32 as u16, 768u16, game);
    }
    let ridley_var_00: u16 = (*E).ridley_var_00;
    if ridley_var_00 != 0 {
        (*E).ridley_var_00 = (ridley_var_00).wrapping_sub(1u16);
    } else {
        (*E).ridley_var_A = ((0xa6b594i32) as u16);
        (*E).ridley_var_00 = 32u16;
        Ridley_Func_114(game);
    };
}
pub unsafe fn Ridley_Func_13(game: &mut Game) {
    Ridley_Func_111(0u16, 0x8000u16, 448u16, game);
    let E = Get_Ridley(0i32, game);
    let ridley_var_00: u16 = (*E).ridley_var_00;
    if ridley_var_00 != 0 {
        (*E).ridley_var_00 = (ridley_var_00).wrapping_sub(1u16);
    } else {
        (*E).ridley_var_A = (if Ridley_Func_40_Carry(game) != 0u8 {
            0xa6bab7i32 & 0xffffi32
        } else {
            0xa6b321i32 & 0xffffi32
        }) as u16;
    };
}
pub unsafe fn Ridley_Func_14(game: &mut Game) {
    set_ram_word!(game.tilemap_stuff[15] = 11u16);
    set_ram_word!(game.tilemap_stuff[9] = 384u16);
    let E = Get_Ridley(0i32, game);
    (*E).ridley_var_A = ((0xa6b5e5i32) as u16);
    (*E).ridley_var_F = ((ram_word!(game, random_number) as i32 & 0x1fi32) + 32i32) as u16;
    Ridley_Func_15(game);
}
pub unsafe fn Ridley_Func_15(game: &mut Game) {
    let E = Get_Ridley(0i32, game);
    if Ridley_Func_17(
        crate::rtl::rom_table_word!(game, 0xa6b60d, (*E).ridley_var_10 as isize),
        game,
    ) & 1
        != 0
    {
        Ridley_Func_18(game);
    } else {
        (*E).ridley_var_F = (*E).ridley_var_F.wrapping_sub(1);
        if ((*E).ridley_var_F & 0x8000) != 0 {
            (*E).ridley_var_A = ((0xa6b613i32) as u16);
            (*E).ridley_var_F = 128u16;
            Ridley_Func_114(game);
        }
    };
}
pub unsafe fn Ridley_Func_16(game: &mut Game) {
    let E = Get_Ridley(0i32, game);
    if Ridley_Func_17(
        crate::rtl::rom_table_word!(game, 0xa6b63b, (*E).ridley_var_10 as isize),
        game,
    ) & 1
        != 0
    {
        Ridley_Func_18(game);
    } else {
        (*E).ridley_var_F = (*E).ridley_var_F.wrapping_sub(1);
        if ((*E).ridley_var_F & 0x8000) != 0 {
            (*E).ridley_var_A = ((0xa6b5e5i32) as u16);
            (*E).ridley_var_F = 128u16;
            Ridley_Func_114(game);
        }
    };
}
pub unsafe fn Ridley_Func_17(r18: u16, game: &mut Game) -> u8 {
    let mut v0: u16 = ram_word!(game, samus_y_pos);
    if ram_word!(game, samus_y_pos) as i32 - 352i32 & 0x8000i32 == 0 {
        v0 = 352u16;
    }
    Ridley_Func_104_0(0u16, Ridley_Func_7(game), r18, v0, game);
    set_ram_word!(game.tilemap_stuff[2] = 1u16);
    if *ram_ptr!(game; 0xa1f) != 3u8 {
        return 1u8;
    }
    if (ram_word!(game, random_number) as u8) >= 0x80u8 {
        let E = Get_Ridley(0i32, game);
        if (*E).ridley_var_0F == 0 && (*E).ridley_var_10 != 1u16 {
            Ridley_Func_99(addr_kRidley_Ilist_E73A as u16, game);
        }
    }
    0u8
}
pub unsafe fn Ridley_Func_18(game: &mut Game) {
    set_ram_word!(game.tilemap_stuff[9] = 240u16);
    set_ram_word!(game.tilemap_stuff[15] = 16u16);
    set_ram_word!(game.tilemap_stuff[0] = 1u16);
    (*Get_Ridley(0i32, game)).ridley_var_A = ((0xa6b6a7i32) as u16);
}
pub unsafe fn Ridley_Func_19(game: &mut Game) {
    let E = Get_Ridley(0i32, game);
    if (*E).base.y_pos as i32 - 288i32 & 0x8000i32 != 0 {
        Ridley_Func_114(game);
        (*E).ridley_var_A = ((0xa6b6ddi32) as u16);
        (*E).ridley_var_F = 32u16;
        Ridley_Func_20(game);
    } else {
        Ridley_Func_104_0(
            0u16,
            0u16,
            crate::rtl::rom_table_word!(game, 0xa6b6c8, (*E).ridley_var_10 as isize),
            288u16,
            game,
        );
    };
}
pub unsafe fn Ridley_Func_20(game: &mut Game) {
    let E = Get_Ridley(0i32, game);
    Ridley_Func_104_0(0u16, 0u16, (*E).base.x_pos, 288u16, game);
    (*E).ridley_var_F = (*E).ridley_var_F.wrapping_sub(1);
    if ((*E).ridley_var_F & 0x8000) != 0 {
        Ridley_Func_71(game);
        Ridley_Func_29(game);
        (*E).ridley_var_A = ((0xa6b70ei32) as u16);
        (*E).ridley_var_F = ((ram_word!(game, random_number) as i32 & 0x3fi32) + 128i32) as u16;
    }
}
pub unsafe fn Ridley_Func_21(game: &mut Game) {
    if Ridley_Func_25(game) & 1 != 0 {
        Ridley_Func_24(game);
        Ridley_Func_27(game);
    } else {
        let E = Get_Ridley(0i32, game);
        let mut v1: i16 =
            ((ram_word!(game, tilemap_stuff[6])) as i16).wrapping_add(((*E).ridley_var_C) as i16);
        if v1 as i32 - 1536i32 & 0x8000i32 == 0 {
            v1 = 1536i16;
        }
        (*E).ridley_var_C = v1 as u16;
        if Ridley_Func_23(game) & 1 != 0 {
            game.eproj_spawn_pt = Point16U {
                x: ram_word!(game, tilemap_stuff[82]),
                y: ((ram_word!(game, tilemap_stuff[83])).wrapping_add(12u16)),
            };
            SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 9u16, game);
            QueueSfx2_Max6(0x76u16, game);
            set_ram_word!(game.earthquake_type = 13u16);
            set_ram_word!(game.earthquake_timer = 4u16);
            Ridley_Func_28(game);
            Ridley_Func_29(game);
            Ridley_Func_93(0x2026u16, addr_word_A6D37C as u16, game);
            set_ram_words! { game, 3072u16;
                tilemap_stuff[20], tilemap_stuff[30], tilemap_stuff[40], tilemap_stuff[50],
                tilemap_stuff[60], tilemap_stuff[70], tilemap_stuff[80],
            }
            set_ram_word!(game.tilemap_stuff[0] = 4u16);
            let mut v2: u16 = ((*E).ridley_var_26).wrapping_add(1u16);
            if (*E).ridley_var_26 as i32 - 1i32 & 0x8000i32 == 0 {
                if (*E).ridley_var_10 != 1u16 {
                    Ridley_Func_99(addr_kRidley_Ilist_E73A as u16, game);
                }
                v2 = 0u16;
            }
            (*E).ridley_var_26 = v2;
            (*E).ridley_var_A = ((0xa6b7b9i32) as u16);
        }
    };
}
pub unsafe fn Ridley_Func_22(game: &mut Game) {
    let E = Get_Ridley(0i32, game);
    let mut v1: bool = false;
    if Ridley_Func_41(game) & 1 == 0 || {
        v1 = (((*E).ridley_var_F) as i16).wrapping_sub(1i16) < 0i16;
        (*E).ridley_var_F = (*E).ridley_var_F.wrapping_sub(1);
        v1 as i32 != 0
    } {
        Ridley_Func_24(game);
        (*E).ridley_var_A = ((0xa6b321i32) as u16);
    } else {
        let ridley_var_C: u16 = (*E).ridley_var_C;
        v1 =
            ((ram_word!(game, tilemap_stuff[7])) as i16).wrapping_add((ridley_var_C) as i16) < 0i16;
        (*E).ridley_var_C = (ram_word!(game, tilemap_stuff[7])).wrapping_add(ridley_var_C);
        if !v1 {
            (*E).ridley_var_C = 0u16;
            (*E).ridley_var_A = ((0xa6b70ei32) as u16);
        }
    };
}
pub unsafe fn Ridley_Func_23(game: &mut Game) -> u8 {
    let mut v0: u8 = (Ridley_Func_103(
        ram_word!(game, tilemap_stuff[82]),
        (ram_word!(game, tilemap_stuff[83])).wrapping_add(16u16),
        game,
    )) & (1u8);
    if v0 == 0 {
        v0 = (Ridley_Func_103(
            ram_word!(game, tilemap_stuff[72]),
            (ram_word!(game, tilemap_stuff[73])).wrapping_add(18u16),
            game,
        )) & (1u8);
        if v0 == 0 {
            v0 = (Ridley_Func_103(
                ram_word!(game, tilemap_stuff[62]),
                (ram_word!(game, tilemap_stuff[63])).wrapping_add(18u16),
                game,
            )) & (1u8);
            if v0 == 0 {
                v0 = (Ridley_Func_103(
                    ram_word!(game, tilemap_stuff[52]),
                    (ram_word!(game, tilemap_stuff[53])).wrapping_add(18u16),
                    game,
                )) & (1u8);
                if v0 == 0 {
                    return (Ridley_Func_103(
                        ram_word!(game, tilemap_stuff[42]),
                        (ram_word!(game, tilemap_stuff[43])).wrapping_add(18u16),
                        game,
                    )) & (1u8);
                }
            }
        }
    }
    v0
}
pub unsafe fn Ridley_Func_24(game: &mut Game) {
    set_ram_words! { game, 1u16; tilemap_stuff[0], tilemap_stuff[10] }
}
pub unsafe fn Ridley_Func_25(game: &mut Game) -> u8 {
    if (Ridley_Func_40_Carry(game) & 1) != 0 {
        (Ridley_Func_26(4u16, 4u16, game)) & (1u8)
    } else {
        0u8
    }
}
pub unsafe fn Ridley_Func_26(k: u16, j: u16, game: &mut Game) -> u8 {
    let E = Get_Ridley(0i32, game);
    let rect: Rect16U = Rect16U {
        x: (((*E).base.x_pos).wrapping_add(crate::rtl::rom_table_word!(
            game,
            0xa6b9d5,
            (*E).ridley_var_10 as isize
        ))),
        y: (((*E).base.y_pos).wrapping_add(crate::rtl::rom_table_word!(
            game,
            0xa6b9db,
            ((*E).ridley_var_1D >> 1) as isize
        ))),
        w: k,
        h: j,
    };
    Ridley_Func_124(rect, game)
}
pub unsafe fn Ridley_Func_27(game: &mut Game) {
    let E = Get_Ridley(0i32, game);
    let mut ridley_var_C: i16 = (*E).ridley_var_C as i16;
    if ridley_var_C >= 0i16 {
        ridley_var_C = -(ridley_var_C as i32) as i16;
    }
    if ridley_var_C as i32 + 512i32 & 0x8000i32 == 0 {
        ridley_var_C = -512i16;
    }
    (*E).ridley_var_C = ridley_var_C as u16;
    Ridley_Func_24(game);
    (*E).ridley_var_A = ((0xa6bb8fi32) as u16);
    Ridley_Func_33b(game);
}
pub unsafe fn Ridley_Func_28(game: &mut Game) {
    let current_block: u64;
    let E = Get_Ridley(0i32, game);
    if (*E).ridley_var_B == 0 {
        (*E).ridley_var_B = (if (*E).base.x_pos & 0x80 != 0 {
            -0xc0i32
        } else {
            0xc0i32
        }) as u16;
    }
    if (((*E).ridley_var_B as i32 ^ (*E).base.x_pos as i32 - ram_word!(game, samus_x_pos) as i32)
        & 0x8000i32)
        == 0i32
    {
        let mut x_pos: u16 = 0;
        x_pos = (*E).base.x_pos;
        if ((x_pos) as i16).wrapping_sub(((*E).ridley_var_22) as i16) >= 0i16 {
            if ((x_pos) as i16).wrapping_sub(((*E).ridley_var_23) as i16) < 0i16 {
                if ram_word!(game, random_number) < 0x555u16 {
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
        let mut v4: u16 = 0;
        v4 = (*E).base.x_pos;
        if ((v4) as i16).wrapping_sub(((*E).ridley_var_22) as i16) < 0i16 {
            current_block = 3419095937434320988;
        } else if ((v4) as i16).wrapping_sub(((*E).ridley_var_23) as i16) >= 0i16 {
            current_block = 12558520430340354144;
        } else if ram_word!(game, random_number) < 0x555u16 {
            current_block = 7587493754414084492;
        } else {
            return;
        }
    }
    match current_block {
        3419095937434320988 => {
            if ((*E).ridley_var_B & 0x8000) == 0 {
                return;
            }
        }
        12558520430340354144 => {
            if ((*E).ridley_var_B & 0x8000) != 0 {
                return;
            }
        }
        _ => {}
    }
    (*E).ridley_var_B = -((*E).ridley_var_B as i32) as u16;
}
pub unsafe fn Ridley_Func_29(game: &mut Game) {
    let v0: i32 = ram_word!(game, random_number) as i32 & 3i32;
    let r18: u16 = crate::rtl::rom_table_word!(game, 0xa6b965, v0 as isize);
    let r20: u16 = crate::rtl::rom_table_word!(game, 0xa6b96d, v0 as isize);
    let E = Get_Ridley(0i32, game);
    let v2: u16 = (2i32 * ((*E).ridley_var_12 as i32 + 2i32)) as u16;
    let v3: i32 = v2 as i32 >> 1i32;
    set_ram_word!(game.tilemap_stuff[7] = crate::rtl::rom_table_word!(game, 0xa6b94d, v3 as isize));
    set_ram_word!(game.tilemap_stuff[6] = crate::rtl::rom_table_word!(game, 0xa6b959, v3 as isize));
    (*E).ridley_var_C = *((RomPtr_A6 as unsafe fn(u16, &mut Game) -> *const u8)(r20, game)
        .offset(v2 as isize) as *mut u16);
    let v4 = RomPtr_A6(r18, game);
    let t: u16 = *(v4.offset(v2 as isize) as *mut u16);
    (*E).ridley_var_B = (if (*E).ridley_var_B & 0x8000 != 0 {
        -(t as i32)
    } else {
        t as i32
    }) as u16;
}
pub unsafe fn Ridley_Func_30(game: &mut Game) {
    let E = Get_Ridley(0i32, game);
    let ridley_var_14: u16 = (*E).ridley_var_14;
    let mut v3: u16 = 0;
    let mut v6: u16 = 0;
    if ridley_var_14 != 0 {
        let v2: i16 = ((abs16(ridley_var_14)) as i16).wrapping_sub(4i16);
        if v2 >= 0i16 {
            v3 = (if ridley_var_14 & 0x8000 != 0 {
                -(v2 as i32)
            } else {
                v2 as i32
            }) as u16;
        } else {
            v3 = 0u16;
        }
        (*E).ridley_var_14 = v3;
    }
    let ridley_var_15: u16 = (*E).ridley_var_15;
    if ridley_var_15 != 0 {
        let v5: i16 = ((abs16(ridley_var_15)) as i16).wrapping_sub(4i16);
        if v5 >= 0i16 {
            v6 = (if ridley_var_15 & 0x8000 != 0 {
                -(v5 as i32)
            } else {
                v5 as i32
            }) as u16;
        } else {
            v6 = 0u16;
        }
        (*E).ridley_var_15 = v6;
    }
    set_ram_word!(
        game.samus_x_pos = ((*E).ridley_var_14 as i32
            + (*E).base.x_pos as i32
            + crate::rtl::rom_table_word!(game, 0xa6b9d5, (*E).ridley_var_10 as isize) as i32)
            as u16
    );
    set_ram_word!(
        game.samus_y_pos = ((*E).ridley_var_15 as i32
            + (*E).base.y_pos as i32
            + crate::rtl::rom_table_word!(game, 0xa6b9db, ((*E).ridley_var_1D >> 1) as isize)
                as i32) as u16
    );
}
pub unsafe fn Ridley_Func_31(game: &mut Game) {
    let E = Get_Ridley(0i32, game);
    (*E).ridley_var_14 = (ram_word!(game, samus_x_pos) as i32
        - ((*E).base.x_pos as i32
            + crate::rtl::rom_table_word!(game, 0xa6b9d5, (*E).ridley_var_10 as isize) as i32))
        as u16;
    (*E).ridley_var_15 = (ram_word!(game, samus_y_pos) as i32
        - ((*E).base.y_pos as i32
            + crate::rtl::rom_table_word!(game, 0xa6b9db, ((*E).ridley_var_1D >> 1) as isize)
                as i32)) as u16;
}
pub unsafe fn Ridley_Func_32(game: &mut Game) {
    let E = Get_Ridley(0i32, game);
    if (*E).ridley_var_25 as i32 - 10i32 & 0x8000i32 != 0 {
        if ram_word!(game, power_bomb_flag) != 0 {
            (*E).ridley_var_A = ((0xa6bd4ei32) as u16);
            Ridley_Func_43(game);
        } else {
            (*E).ridley_var_A = ((0xa6b3eci32) as u16);
            set_ram_word!(game.tilemap_stuff[2] = 1u16);
        }
    } else {
        Ridley_Func_126(game);
        (*E).ridley_var_A = ((0xa6c53ei32) as u16);
        Ridley_C53E(game);
    };
}
pub unsafe fn Ridley_Func_33(game: &mut Game) {
    let E = Get_Ridley(0i32, game);
    if (Ridley_Func_40_Carry(game) & 1) != 0
        && (*E).ridley_var_1F == 0
        && {
            let mut r18: u16 =
                crate::rtl::rom_table_word!(game, 0xa6bb48, (*E).ridley_var_10 as isize);
            let mut r20: u16 = ((*E).base.x_pos).wrapping_sub(ram_word!(game, samus_x_pos));
            ((r18 ^ r20) & 0x8000) == 0 || {
                let mut v1: u16 = abs16(r20);
                v1 as i32 - 32i32 & 0x8000i32 != 0
            }
        }
        && (((*E).base.y_pos as i32 + 35i32 - ram_word!(game, samus_y_pos) as i32) as i16) < 0i16
    {
        let mut v2: i16 = -16i16;
        if (*E).ridley_var_10 != 0 {
            v2 = 16i16;
        }
        Ridley_Func_104_0(
            0u16,
            crate::rtl::rom_table_word!(game, 0xa6bb4e, (*E).ridley_var_12 as isize),
            (ram_word!(game, samus_x_pos)).wrapping_add((v2) as u16),
            (ram_word!(game, samus_y_pos)).wrapping_sub(4u16),
            game,
        );
        let rect: Rect16U = Rect16U {
            x: (((*E).base.x_pos).wrapping_add(crate::rtl::rom_table_word!(
                game,
                0xa6b9d5,
                (*E).ridley_var_10 as isize
            ))),
            y: (((*E).base.y_pos).wrapping_add(35u16)),
            w: 8u16,
            h: 12u16,
        };
        if Ridley_Func_124(rect, game) != 0 {
            (*E).ridley_var_C = -((*E).ridley_var_C as i32) as u16;
            if (*E).base.health != 0 {
                if ram_word!(game, power_bomb_flag) != 0 {
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
    let E = Get_Ridley(0i32, game);
    (*E).ridley_var_17 = g_word_A6BBEB[(*E).ridley_var_10 as usize];
    let mut v1: i16 = (((*E).base.y_pos) as i16).wrapping_sub(64i16);
    if (*E).base.y_pos as i32 - 320i32 & 0x8000i32 != 0 {
        v1 = 256i16;
    }
    (*E).ridley_var_18 = v1 as u16;
    if (*E).ridley_var_1B == 0 {
        Ridley_Func_37(game);
    }
    (*E).ridley_var_A = ((0xa6bbc4i32) as u16);
    (*E).ridley_var_F = 32u16;
    Ridley_Func_34(game);
}
pub unsafe fn Ridley_Func_34(game: &mut Game) {
    let E = Get_Ridley(0i32, game);
    Ridley_Func_104_0(0u16, 0u16, (*E).ridley_var_17, (*E).ridley_var_18, game);
    (*E).ridley_var_F = (*E).ridley_var_F.wrapping_sub(1);
    if ((*E).ridley_var_F & 0x8000) != 0 {
        (*E).ridley_var_A = ((0xa6bbf1i32) as u16);
        (*E).ridley_var_F = 32u16;
    }
}
pub unsafe fn Ridley_Func_35(game: &mut Game) {
    let E = Get_Ridley(0i32, game);
    (*E).ridley_var_F = (*E).ridley_var_F.wrapping_sub(1);
    if ((*E).ridley_var_F & 0x8000) != 0 {
        set_ram_word!(game.tilemap_stuff[15] = 8u16);
        set_ram_word!(game.tilemap_stuff[9] = 240u16);
        Ridley_Func_38(game);
        (*E).ridley_var_A = ((0xa6bc2ei32) as u16);
        (*E).ridley_var_F = 64u16;
    } else {
        Ridley_Func_104_0(0u16, 0u16, (*E).ridley_var_17, 256u16, game);
    };
}
pub unsafe fn Ridley_Func_36(game: &mut Game) {
    let E = Get_Ridley(0i32, game);
    (*E).ridley_var_F = (*E).ridley_var_F.wrapping_sub(1);
    if ((*E).ridley_var_F & 0x8000) != 0 {
        set_ram_word!(game.tilemap_stuff[15] = 16u16);
        set_ram_word!(game.tilemap_stuff[9] = 240u16);
        (*E).ridley_var_A = ((0xa6b321i32) as u16);
    } else {
        Ridley_Func_104_0(
            0u16,
            0u16,
            g_word_A6BC62[(*E).ridley_var_10 as usize],
            224u16,
            game,
        );
    };
}
pub unsafe fn Ridley_Func_37(game: &mut Game) {
    Ridley_Func_31(game);
    let E = Get_Ridley(0i32, game);
    (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
    (*E).ridley_var_1B = 1u16;
    CallSomeSamusCode(0u16, game);
    Ridley_Func_119(1u8, game);
}
pub unsafe fn Ridley_Func_38(game: &mut Game) {
    set_ram_words! { game, 1u16; tilemap_stuff[2], tilemap_stuff[0] }
    let E = Get_Ridley(0i32, game);
    if ((*E).ridley_var_01 & 0x8000) == 0 {
        (*E).ridley_var_1E = (if Ridley_Func_40_Sign(game) != 0u8 {
            6i32
        } else {
            10i32
        }) as u16;
    }
    (*E).ridley_var_1B = 0u16;
    CallSomeSamusCode(1u16, game);
    Ridley_Func_119(0u8, game);
}
pub unsafe fn Ridley_Func_39(game: &mut Game) {
    let E = Get_Ridley(0i32, game);
    let mut ridley_var_01: i16 = (*E).ridley_var_01 as i16;
    if ridley_var_01 >= 0i16 {
        if ridley_var_01 != 0 {
            if Ridley_Func_121(game) & 1 != 0 {
                (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
                return;
            }
            (*E).base.properties =
                ((*E).base.properties as i32 & !(kEnemyProps_Tangible as i32)) as u16;
        }
        let ridley_var_1E: u16 = (*E).ridley_var_1E;
        if ridley_var_1E != 0 {
            let v3: u16 = (ridley_var_1E).wrapping_sub(1u16);
            (*E).ridley_var_1E = v3;
            if v3 == 0 && ((*E).ridley_var_01 & 0x8000) == 0 {
                (*E).base.properties =
                    ((*E).base.properties as i32 & !(kEnemyProps_Tangible as i32)) as u16;
            }
        }
    }
}
static byte_A6BD04: [u8; 28] = [
    0x80, 0x80, 0x80, 0, 0xff, 0x80, 0x80, 0xff, 0xff, 0xff, 0x80, 0, 0, 0x80, 0x80, 0x80, 0x80,
    0xff, 0xff, 0xff, 0x80, 0x80, 0, 0x80, 0x80, 0, 0, 0x80,
];
unsafe fn Ridley_Func_40_Carry(game: &mut Game) -> u8 {
    ((byte_A6BD04[*ram_ptr!(game; 0xa1f) as usize] & 0x80) != 0) as u8
}
unsafe fn Ridley_Func_40_Sign(game: &mut Game) -> u8 {
    ((byte_A6BD04[*ram_ptr!(game; 0xa1f) as usize] & 0x40) != 0) as u8
}
pub unsafe fn Ridley_Func_41(game: &mut Game) -> u8 {
    ((ram_word!(game, samus_y_pos) as i32 - 352i32 & 0x8000i32) == 0i32) as u8
}
pub unsafe fn Ridley_Func_42(game: &mut Game) {
    let E = Get_Ridley(0i32, game);
    let mut ridley_var_01: i16 = (*E).ridley_var_01 as i16;
    if ridley_var_01 > 0i16
        && ridley_var_01 != 2i16
        && ram_word!(game, power_bomb_flag) != 0u16
        && (*E).ridley_var_1B == 0
    {
        Ridley_Func_24(game);
        (*E).ridley_var_A = ((0xa6bab7i32) as u16);
    }
}
pub unsafe fn Ridley_Func_43(game: &mut Game) {
    if ram_word!(game, power_bomb_flag) != 0 {
        (*Get_Ridley(0i32, game)).ridley_var_01 = 2u16;
        let r18: u16 =
            (if ram_word!(game, power_bomb_explosion_x_pos) as i32 - 128i32 & 0x8000i32 != 0 {
                192i32
            } else {
                80i32
            }) as u16;
        let r20: u16 =
            (if ram_word!(game, power_bomb_explosion_y_pos) as i32 - 256i32 & 0x8000i32 != 0 {
                384i32
            } else {
                192i32
            }) as u16;
        Ridley_Func_104_0(0u16, Ridley_Func_7(game), r18, r20, game);
    } else {
        let E = Get_Ridley(0i32, game);
        (*E).ridley_var_01 = 1u16;
        (*E).ridley_var_A = (if (*E).ridley_var_1B != 0u16 {
            0xa6bb8fi32 & 0xffffi32
        } else {
            0xa6b321i32 & 0xffffi32
        }) as u16;
    };
}
pub unsafe fn Ridley_Func_44(game: &mut Game) {
    Ridley_Func_106(0u16, 1u16, 192u16, 128u16, game);
    let E = Get_Ridley(0i32, game);
    if (*E).base.x_pos as i32 - 192i32 & 0x8000i32 == 0 {
        (*E).ridley_var_A = ((0xa6bdbci32) as u16);
    }
}
pub unsafe fn Ridley_Func_45(game: &mut Game) {
    let E = Get_Ridley(0i32, game);
    (*E).ridley_var_20 = -192i32 as u16;
    Ridley_Func_106(0u16, 1u16, 192u16, -128i32 as u16, game);
    if (*E).base.y_pos as i32 - 32i32 & 0x8000i32 != 0 {
        (*E).ridley_var_40 = ((0xa6becai32) as u16);
        (*E).ridley_var_A = ((0xa6bdf2i32) as u16);
        (*E).ridley_var_F = 21u16;
    }
}
pub unsafe fn Ridley_Func_46(game: &mut Game) {
    let E = Get_Ridley(0i32, game);
    (*E).ridley_var_F = (*E).ridley_var_F.wrapping_sub(1);
    if ((*E).ridley_var_F & 0x8000) != 0 {
        Ridley_Func_99(addr_kRidley_Ilist_E658 as u16, game);
        (*E).ridley_var_A = ((0xa6be03i32) as u16);
        Ridley_Func_47(game);
    }
}
pub unsafe fn Ridley_Func_47(game: &mut Game) {
    let E = Get_Ridley(0i32, game);
    Ridley_Func_104_0(
        0u16,
        12u16,
        ((*E).ridley_var_42).wrapping_sub(10u16),
        ((*E).ridley_var_44).wrapping_sub(56u16),
        game,
    );
    let rect: Rect16U = Rect16U {
        x: (((*E).base.x_pos).wrapping_add(14u16)),
        y: (((*E).base.y_pos).wrapping_add(66u16)),
        w: 4u16,
        h: 4u16,
    };
    if Ridley_Func_48(rect, game) & 1 != 0 {
        (*E).ridley_var_1B = 1u16;
        Ridley_Func_119(1u8, game);
        (*E).ridley_var_C = -512i32 as u16;
        (*E).ridley_var_40 = ((0xa6beb3i32) as u16);
        (*E).ridley_var_A = ((0xa6a971i32) as u16);
    }
}
pub unsafe fn Ridley_Func_48(rect: Rect16U, game: &mut Game) -> u8 {
    let E = Get_Ridley(0i32, game);
    let v1: u16 = abs16(((*E).ridley_var_42).wrapping_sub(rect.x));
    let mut v2: bool = v1 < 4u16;
    let v3: u16 = (v1).wrapping_sub(4u16);
    let result: u8 = 0u8;
    if v2 || v3 < rect.w {
        let v4: u16 = abs16(((*E).ridley_var_44).wrapping_sub(rect.y));
        v2 = v4 < 4u16;
        let v5: u16 = (v4).wrapping_sub(4u16);
        if v2 || v5 < rect.h {
            return 1u8;
        }
    }
    result
}
crate::game::address_dispatch! {
    pub unsafe fn CallRidleyFunc_var40(ea: u32, game: &mut Game) => ea;
        call (game); default { Unreachable(game); };
    {
        10927772 => Ridley_Func_50; 10927795 => Ridley_Func_51; 10927818 => Ridley_Func_52;
        10927836 => Ridley_Func_53; 10927897 => [()];
    }
}
pub unsafe fn Ridley_Func_49(game: &mut Game) {
    CallRidleyFunc_var40(
        (((*Get_Ridley(0i32, game)).ridley_var_40) as u32) | (0xa60000u32),
        game,
    );
}
pub unsafe fn Ridley_Func_50(game: &mut Game) {
    let E = Get_Ridley(0i32, game);
    (*E).ridley_var_42 = ((*E).base.x_pos).wrapping_sub(16u16);
    (*E).ridley_var_44 = ((*E).base.y_pos).wrapping_add(22u16);
}
pub unsafe fn Ridley_Func_51(game: &mut Game) {
    let E = Get_Ridley(0i32, game);
    (*E).ridley_var_42 = ((*E).base.x_pos).wrapping_add(14u16);
    (*E).ridley_var_44 = ((*E).base.y_pos).wrapping_add(66u16);
}
pub unsafe fn Ridley_Func_52(game: &mut Game) {
    let E = Get_Ridley(0i32, game);
    set_fields! {[(*E)], 0u16; ridley_var_45, ridley_var_46}
    (*E).ridley_var_40 = ((0xa6bedci32) as u16);
    Ridley_Func_53(game);
}
pub unsafe fn Ridley_Func_53(game: &mut Game) {
    let E = Get_Ridley(0i32, game);
    let v1: u16 = ((*E).ridley_var_46).wrapping_add(8u16);
    (*E).ridley_var_46 = v1;
    let t: i32 =
        *(&raw mut (*E).ridley_var_43 as *mut u8).offset(1isize) as i32 + (v1 as i32 & 0xffi32);
    *(&raw mut (*E).ridley_var_43 as *mut u8).offset(1isize) = t as u8;
    (*E).ridley_var_44 =
        ((*E).ridley_var_44 as i32 + ((v1 >> 8) as i8 as i32 + (t >> 8i32))) as u16;
    if (*E).ridley_var_44 as i32 - 192i32 & 0x8000i32 == 0 {
        (*E).ridley_var_44 = 192u16;
        (*E).ridley_var_40 = addr_locret_A6BF19 as u16;
    }
}
pub unsafe fn DrawBabyMetroid_0(game: &mut Game) {
    let v0: i32 = BabyMetroid_DBCB_DoubleRetEx(
        ((ram_ptr!(game, ExtraEnemyRam7800; 0x7800) as *mut u8)
            .offset_from(&raw mut game.g_ram as *mut u8) as c_long
            + 6 as c_long) as u16,
        game,
    );
    if v0 < 0i32 {
        return;
    }
    let E = Get_Ridley(0i32, game);
    sub_A6DC13(
        v0 as u16,
        (*E).ridley_var_42,
        (*E).ridley_var_44,
        0u16,
        game,
    );
}
pub unsafe fn BabyMetroid_Instr_2(k: u16, game: &mut Game) -> u16 {
    if (*Get_Ridley(0i32, game)).ridley_var_46 == 0 && (ram_word!(game, random_number) & 1) != 0 {
        return BabyMetroid_Goto(k, game);
    }
    QueueSfx3_Max6(0x24u16, game);
    (k).wrapping_add(2u16)
}
pub unsafe fn BabyMetroid_Instr_3(k: u16, game: &mut Game) -> u16 {
    let v1: u16 = *(RomPtr_A6(k, game) as *mut u16);
    WriteColorsToPalette(0x162u16, 0xa6u8, v1, 0xfu16, game);
    (k).wrapping_add(2u16)
}
pub unsafe fn BabyMetroid_Instr_1(k: u16, game: &mut Game) -> u16 {
    if (*Get_Ridley(0i32, game)).ridley_var_46 != 0 {
        BabyMetroid_Goto(k, game)
    } else {
        (k).wrapping_add(2u16)
    }
}
pub unsafe fn BabyMetroid_Goto(k: u16, game: &mut Game) -> u16 {
    *(RomPtr_A6(k, game) as *mut u16)
}
pub unsafe fn Ridley_Func_54(game: &mut Game) {
    let ridley_var_F: u16 = (*Get_Ridley(0i32, game)).ridley_var_F;
    if ridley_var_F != 0 {
        match ridley_var_F as i32 {
            2 => Ridley_C08E(game),
            4 => Ridley_C09F(game),
            6 => Ridley_Func_56(game),
            8 => Ridley_Func_57(game),
            10 => Ridley_C104(game),
            12 => Ridley_Func_58(game),
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
    set_ram_word!(game.palette_buffer[97] = ram_word!(game, palette_buffer[1]));
    set_ram_word!(game.palette_buffer[99] = ram_word!(game, palette_buffer[3]));
    set_ram_word!(game.palette_buffer[81] = ram_word!(game, palette_buffer[17]));
    set_ram_word!(game.palette_buffer[83] = ram_word!(game, palette_buffer[19]));
    let E = Get_Ridley(0i32, game);
    (*E).ridley_var_E = -15157i32 as u16;
    (*E).ridley_var_F = (*E).ridley_var_F.wrapping_add(1);
    (*E).ridley_var_F = (*E).ridley_var_F.wrapping_add(1);
    Ridley_C08E(game);
}
pub unsafe fn Ridley_C08E(game: &mut Game) {
    if ProcessEscapeTimerTileTransfers(game) & 1 != 0 {
        let E = Get_Ridley(0i32, game);
        (*E).ridley_var_E = -15106i32 as u16;
        (*E).ridley_var_F = (*E).ridley_var_F.wrapping_add(1);
        (*E).ridley_var_F = (*E).ridley_var_F.wrapping_add(1);
        Ridley_C09F(game);
    }
}
pub unsafe fn Ridley_C09F(game: &mut Game) {
    if ProcessEscapeTimerTileTransfers(game) & 1 != 0 {
        let E = Get_Ridley(0i32, game);
        (*E).ridley_var_F = (*E).ridley_var_F.wrapping_add(1);
        (*E).ridley_var_F = (*E).ridley_var_F.wrapping_add(1);
        Ridley_Func_59(game);
        (*E).ridley_var_E = 128u16;
        QueueMusic_Delayed8(7u16, game);
    }
}
pub unsafe fn Ridley_Func_56(game: &mut Game) {
    Ridley_Func_61(game);
    let E = Get_Ridley(0i32, game);
    crate::update_place! { previous fresh16 = (*E).ridley_var_E => (*E).ridley_var_E.wrapping_sub(1); }
    if fresh16 == 1u16 {
        (*E).ridley_var_3B = -15280i32 as u16;
        set_fields! {[(*E)], 0u16; ridley_var_3C, ridley_var_3D, ridley_var_3E, ridley_var_3F}
        (*E).ridley_var_E = 32u16;
        if ram_word!(game, japanese_text_flag) == 0 {
            (*E).ridley_var_F = (*E).ridley_var_F.wrapping_add(1);
            (*E).ridley_var_F = (*E).ridley_var_F.wrapping_add(1);
        }
        (*E).ridley_var_F = (*E).ridley_var_F.wrapping_add(1);
        (*E).ridley_var_F = (*E).ridley_var_F.wrapping_add(1);
    }
}
pub unsafe fn Ridley_Func_57(game: &mut Game) {
    let E = Get_Ridley(0i32, game);
    crate::update_place! { previous fresh15 = (*E).ridley_var_E => (*E).ridley_var_E.wrapping_sub(1); }
    if fresh15 == 1u16 {
        (*E).ridley_var_F = (*E).ridley_var_F.wrapping_add(1);
        (*E).ridley_var_F = (*E).ridley_var_F.wrapping_add(1);
        Ridley_Func_62(game);
    }
    Ridley_C104(game);
}
pub unsafe fn Ridley_C104(game: &mut Game) {
    Ridley_Func_61(game);
    if HandleTypewriterText_Ext(0x3582u16, game) & 1 != 0 {
        let E = Get_Ridley(0i32, game);
        (*E).ridley_var_F = (*E).ridley_var_F.wrapping_add(1);
        (*E).ridley_var_F = (*E).ridley_var_F.wrapping_add(1);
    }
}
pub unsafe fn Ridley_Func_58(game: &mut Game) {
    Ridley_Func_61(game);
    CeresRidley_Func_2(game);
    let E = Get_Ridley(0i32, game);
    (*E).ridley_var_F = 0u16;
    (*E).ridley_var_A = ((0xa6aa50i32) as u16);
    set_ram_word!(game.timer_status = 1u16);
    set_ram_word!(game.ceres_status = 2u16);
    SetBossBitForCurArea(1u16, game);
}
pub unsafe fn Ridley_Func_59(game: &mut Game) {
    let v0: u16 = ram_word!(game, vram_write_queue_tail);
    let v1 = RomPtr_A6(addr_byte_A6C15D as u16, game);
    let mut v2 = gVramWriteEntry(ram_word!(game, vram_write_queue_tail) as i32, game);
    (*v2).size = *(v1 as *mut u16);
    *((&raw mut (*v2).src.addr as *mut u8).offset(1) as *mut VoidP) = *(v1.offset(3) as *mut u16);
    (*v2).src.addr = *(v1.offset(2) as *mut u16);
    (*v2).vram_dst = *(v1.offset(5) as *mut u16);
    set_ram_word!(game.vram_write_queue_tail = (v0).wrapping_add(7u16));
}
pub unsafe fn Ridley_Func_60(game: &mut Game) {
    let E = Get_Ridley(0i32, game);
    let mut v1: i16 = (((*E).ridley_var_39) as i16).wrapping_add(1i16);
    if (*E).ridley_var_39 as i32 - 31i32 & 0x8000i32 == 0 {
        v1 = -31i16;
    }
    (*E).ridley_var_39 = v1 as u16;
    set_ram_word!(game.palette_buffer[11] = (abs16((*E).ridley_var_39)) & (0x1fu16));
}
pub unsafe fn Ridley_Func_61(game: &mut Game) {
    if ram_word!(game, door_transition_flag_enemies) | ram_word!(game, palette_change_num) == 0
        && (ram_word!(game, nmi_frame_counter_word) & 3) == 0
    {
        let E = Get_Ridley(0i32, game);
        let mut v1: u16 = ((*E).ridley_var_38).wrapping_add(1u16);
        if v1 >= 0x10u16 {
            v1 = 0u16;
        }
        (*E).ridley_var_38 = v1;
        let v2: i32 = (*E).ridley_var_38 as i32 + 2i32 * v1 as i32;
        set_ram_word!(
            game.palette_buffer[97] = crate::rtl::rom_table_word!(game, 0xa6c1df, v2 as isize)
        );
        set_ram_word!(
            game.palette_buffer[98] =
                crate::rtl::rom_table_word!(game, 0xa6c1df, (v2 + 1i32) as isize)
        );
        set_ram_word!(
            game.palette_buffer[99] =
                crate::rtl::rom_table_word!(game, 0xa6c1df, (v2 + 2i32) as isize)
        );
    }
}
pub unsafe fn SetupZebesEscapeTypewriter(game: &mut Game) {
    set_ram_word!(game.palette_buffer[157] = ram_word!(game, palette_buffer[125]));
    set_ram_word!(game.palette_buffer[158] = ram_word!(game, palette_buffer[126]));
    let E = Get_MotherBrain(0i32, game);
    (*E).mbn_var_3B = addr_loc_A9C49C as u16;
    set_fields! {[(*E)], 0u16; mbn_var_3C, mbn_var_3D, mbn_var_3E, mbn_var_3F}
    set_ram_words! { game, 0u16; reg_BG2HOFS, reg_BG2VOFS }
}
pub unsafe fn ProcessEscapeTimerTileTransfers(game: &mut Game) -> u8 {
    let mut v4 = null_mut::<VramWriteEntry>();
    let v0 = gEnemyData(0i32, game);
    let ai_var_E: u16 = (*v0).ai_var_E;
    let v2: u16 = ram_word!(game, vram_write_queue_tail);
    let v3 = RomPtr_A6(ai_var_E, game);
    let result: u8 = 1u8;
    if *(v3 as *mut u16) != 0 {
        v4 = gVramWriteEntry(ram_word!(game, vram_write_queue_tail) as i32, game);
        (*v4).size = *(v3 as *mut u16);
        *((&raw mut (*v4).src.addr as *mut u8).offset(1) as *mut VoidP) =
            *(v3.offset(3) as *mut u16);
        (*v4).src.addr = *(v3.offset(2) as *mut u16);
        (*v4).vram_dst = *(v3.offset(5) as *mut u16);
        set_ram_word!(game.vram_write_queue_tail = (v2).wrapping_add(7u16));
        (*v0).ai_var_E = (ai_var_E).wrapping_add(7u16);
        if *(RomPtr_A6((*v0).ai_var_E, game) as *mut u16) != 0 {
            return 0u8;
        }
    }
    result
}
pub unsafe fn HandleTypewriterText_Ext(a: u16, game: &mut Game) -> u8 {
    let r18: u16 = a;
    let mut v4: i16 = 0;
    let mut v12 = null_mut::<VramWriteEntry>();
    let E = Get_MotherBrain(0i32, game);
    let mbn_var_3D: u16 = (*E).mbn_var_3D;
    if mbn_var_3D != 0 {
        (*E).mbn_var_3D = (mbn_var_3D).wrapping_sub(1u16);
        0u8
    } else {
        (*E).mbn_var_3D = (*E).mbn_var_3E;
        let mut i: i32 = (*E).mbn_var_3B as i32;
        loop {
            loop {
                v4 = *(RomPtr_A6(i as u16, game) as *mut u16) as i16;
                if v4 == 0 {
                    return 1u8;
                }
                if v4 != 1i16 {
                    break;
                }
                let v5: u16 = (i + 2i32) as u16;
                let v6: u16 = *(RomPtr_A6(v5, game) as *mut u16);
                (*E).mbn_var_3E = v6;
                i = v5 as i32 + 2i32;
            }
            if v4 != 13i16 {
                break;
            }
            let mut v7: i32 = i + 2i32;
            let v8: u16 = *(RomPtr_A6(v7 as u16, game) as *mut u16);
            (*E).mbn_var_3C = v8;
            i = v7 + 2i32;
        }
        v4 = v4 as u8 as i16;
        if (v4 as u8) == 32u8 {
            (*E).mbn_var_3C = (*E).mbn_var_3C.wrapping_add(1);
            (*E).mbn_var_3B = (i + 1i32) as u16;
            0u8
        } else {
            if (v4 as u8) == 33u8 {
                v4 = 91i16;
            }
            (*E).mbn_var_3B = (i + 1i32) as u16;
            let v11: u16 = ram_word!(game, vram_write_queue_tail);
            v12 = gVramWriteEntry(ram_word!(game, vram_write_queue_tail) as i32, game);
            (*v12).size = 2u16;
            *((&raw mut (*v12).src.addr as *mut u8).offset(1) as *mut VoidP) = 32256 as VoidP;
            (*E).mbn_var_3A = (r18 as i32 + v4 as i32 - 65i32) as u16;
            (*v12).src.addr = ((ram_ptr!(game, ExtraEnemyRam8000; 0x8000) as *mut u8)
                .offset_from(&raw mut game.g_ram as *mut u8)
                as c_long
                + 52 as c_long) as VoidP;
            let mbn_var_3C: u16 = (*E).mbn_var_3C;
            (*v12).vram_dst = mbn_var_3C;
            (*E).mbn_var_3C = (mbn_var_3C).wrapping_add(1u16);
            set_ram_word!(game.vram_write_queue_tail = (v11).wrapping_add(7u16));
            let v14: u16 = ((*E).mbn_var_3F).wrapping_add(1u16);
            (*E).mbn_var_3F = v14;
            if v14 as i32 - 2i32 & 0x8000i32 == 0 {
                (*E).mbn_var_3F = 0u16;
                if ram_word!(game, area_index) == 6u16 {
                    QueueSfx2_Max3(0x45u16, game);
                } else {
                    QueueSfx3_Max3(0xdu16, game);
                }
            }
            0u8
        }
    }
}
pub unsafe fn Ridley_Func_62(game: &mut Game) {
    let mut v3 = null_mut::<VramWriteEntry>();
    let mut v0: u16 = addr_stru_A6C3B8 as u16;
    let mut i: i32 = ram_word!(game, vram_write_queue_tail) as i32;
    loop {
        let v2 = RomPtr_A6(v0, game);
        if *(v2 as *mut u16) == 0 {
            break;
        }
        v3 = gVramWriteEntry(i, game);
        (*v3).size = *(v2 as *mut u16);
        *((&raw mut (*v3).src.addr as *mut u8).offset(1) as *mut VoidP) =
            *(v2.offset(3) as *mut u16);
        (*v3).src.addr = *(v2.offset(2) as *mut u16);
        (*v3).vram_dst = *(v2.offset(5) as *mut u16);
        v0 = (v0).wrapping_add(7u16);
        i += 7i32;
    }
    set_ram_word!(game.vram_write_queue_tail = i as u16);
}
pub unsafe fn Ridley_Func_63(game: &mut Game) {
    if Ridley_Func_68(game) & 1 == 0 {
        Ridley_C53E(game);
    }
}
pub unsafe fn Ridley_C53E(game: &mut Game) {
    Ridley_Func_99(addr_kRidley_Ilist_E6C8 as u16, game);
    let E = Get_Ridley(0i32, game);
    (*E).ridley_var_A = ((0xa6c551i32) as u16);
    (*E).ridley_var_F = 32u16;
}
pub unsafe fn Ridley_C551(game: &mut Game) {
    Ridley_Func_68(game);
    let E = Get_Ridley(0i32, game);
    (*E).ridley_var_F = (*E).ridley_var_F.wrapping_sub(1);
    if ((*E).ridley_var_F & 0x8000) != 0 {
        set_fields! {[(*E)], 0u16; ridley_var_B, ridley_var_C}
        set_ram_word!(game.fx_target_y_pos = 528u16);
        set_ram_word!(game.fx_y_vel = 64u16);
        set_ram_word!(game.fx_timer = 1u16);
        set_fields! {[(*E)], 0u16; ridley_var_27, ridley_var_28}
        (*E).ridley_var_A = ((0xa6c588i32) as u16);
        (*E).ridley_var_F = 160u16;
        Ridley_Func_64(game);
    }
}
pub unsafe fn Ridley_Func_64(game: &mut Game) {
    Ridley_Func_69(game);
    let E = Get_Ridley(0i32, game);
    (*E).ridley_var_F = (*E).ridley_var_F.wrapping_sub(1);
    if ((*E).ridley_var_F & 0x8000) != 0 {
        if (*E).ridley_var_1B != 0 {
            Ridley_Func_38(game);
        }
        (*E).ridley_var_A = ((0xa6c5a8i32) as u16);
        set_fields! {[(*E)], 0u16; ridley_var_B, ridley_var_C}
        RidleysExplosion_Func_2(game);
    }
}
pub unsafe fn Ridley_Func_65(game: &mut Game) {
    Ridley_Func_69(game);
    let E = Get_Ridley(0i32, game);
    (*E).ridley_var_02 = 0u16;
    (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Invisible) as u16);
    (*E).ridley_var_A = ((0xa6c5c8i32) as u16);
    (*E).ridley_var_F = 32u16;
}
pub unsafe fn Ridley_Func_66(game: &mut Game) {
    let E = Get_Ridley(0i32, game);
    (*E).ridley_var_F = (*E).ridley_var_F.wrapping_sub(1);
    if ((*E).ridley_var_F & 0x8000) != 0 {
        (*E).ridley_var_A = ((0xa6c5dai32) as u16);
        (*E).ridley_var_F = 256u16;
    }
}
pub unsafe fn Ridley_Func_67(game: &mut Game) {
    let E = Get_Ridley(0i32, game);
    (*E).ridley_var_F = (*E).ridley_var_F.wrapping_sub(1);
    if ((*E).ridley_var_F & 0x8000) != 0 {
        let k: u16 = 0u16;
        printf(b"Warning: X undefined\n\0" as *const u8 as *const c_char);
        SetBossBitForCurArea(1u16, game);
        Enemy_ItemDrop_Ridley(k, game);
        QueueMusic_Delayed8(3u16, game);
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Deleted) as u16);
        (*E).ridley_var_A = addr_locret_A6C600 as u16;
    }
}
pub unsafe fn Ridley_Func_68(game: &mut Game) -> u8 {
    let rect: Rect16U = Rect16U {
        x: 128u16,
        y: 328u16,
        w: 4u16,
        h: 4u16,
    };
    Ridley_Func_104(0u16, 0u16, 0x10u16, rect.x, rect.y, game);
    Shitroid_Func_2(0u16, rect, game)
}
pub unsafe fn Ridley_Func_69(game: &mut Game) {
    let E = Get_Ridley(0i32, game);
    let mut v1: i16 = (((*E).ridley_var_27) as i16).wrapping_sub(1i16);
    if v1 < 0i16 {
        (*E).ridley_var_27 = 4u16;
        let mut v2: u16 = ((*E).ridley_var_28).wrapping_add(1u16);
        if (*E).ridley_var_28 as i32 - 9i32 & 0x8000i32 == 0 {
            v2 = 0u16;
        }
        (*E).ridley_var_28 = v2;
        let v3: i32 = ((4u16).wrapping_mul(v2)) as i32 >> 1i32;
        game.eproj_spawn_pt = Point16U {
            x: (((*E).base.x_pos).wrapping_add((g_word_A6C66E[v3 as usize]) as u16)),
            y: (((*E).base.y_pos).wrapping_add((g_word_A6C66E[(v3 + 1i32) as usize]) as u16)),
        };
        SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 3u16, game);
        QueueSfx2_Max3(0x24u16, game);
    } else {
        (*E).ridley_var_27 = v1 as u16;
    };
}
pub unsafe fn RidleysExplosion_Init(game: &mut Game) {
    let E = Get_Ridley(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.instruction_timer = 1u16;
    set_fields! {[(*E).base], 0u16; timer, vram_tiles_index}
    (*E).base.palette_index = 3584u16;
    let ridley_parameter_1: u16 = (*E).ridley_parameter_1;
    (*E).ridley_var_F =
        crate::rtl::rom_table_word!(game, 0xa6c6ce, (ridley_parameter_1 >> 1) as isize);
    let Random: u16 = NextRandom(game);
    (*E).ridley_var_B = (if Random & 0x8000 != 0 {
        -(Random as i32 & 0x130i32)
    } else {
        Random as i32 & 0x130i32
    }) as u16;
    (*E).ridley_var_C = 0u16;
    if ridley_parameter_1 != 0 {
        match ridley_parameter_1 as i32 {
            2 => {
                (*E).base.x_pos = ram_word!(game, tilemap_stuff[32]);
                (*E).base.y_pos = ram_word!(game, tilemap_stuff[33]);
                (*E).base.current_instruction = addr_kRidleysExplosion_Ilist_CA47 as u16;
            }
            4 => {
                (*E).base.x_pos = ram_word!(game, tilemap_stuff[42]);
                (*E).base.y_pos = ram_word!(game, tilemap_stuff[43]);
                (*E).base.current_instruction = addr_kRidleysExplosion_Ilist_CA4D as u16;
            }
            6 => {
                (*E).base.x_pos = ram_word!(game, tilemap_stuff[52]);
                (*E).base.y_pos = ram_word!(game, tilemap_stuff[53]);
                (*E).base.current_instruction = addr_kRidleysExplosion_Ilist_CA4D as u16;
            }
            8 => {
                (*E).base.x_pos = ram_word!(game, tilemap_stuff[62]);
                (*E).base.y_pos = ram_word!(game, tilemap_stuff[63]);
                (*E).base.current_instruction = addr_kRidleysExplosion_Ilist_CA53 as u16;
            }
            10 => {
                (*E).base.x_pos = ram_word!(game, tilemap_stuff[72]);
                (*E).base.y_pos = ram_word!(game, tilemap_stuff[73]);
                (*E).base.current_instruction = addr_kRidleysExplosion_Ilist_CA53 as u16;
            }
            12 => {
                (*E).base.x_pos = ram_word!(game, tilemap_stuff[82]);
                (*E).base.y_pos = ram_word!(game, tilemap_stuff[83]);
                (*E).base.current_instruction =
                    *(game.g_rom.offset(crate::rtl::lorom_offset(0xa6c7ba)) as *mut u16).offset(
                        ((*((ram_ptr!(game, u16; 0x2000)).offset(71) as *mut u8).offset(0) as i32
                            + *((ram_ptr!(game, u16; 0x2000)).offset(81) as *mut u8).offset(0)
                                as i32
                            + 8i32
                            & 0xf0i32) as u8
                            >> 4) as isize,
                    );
            }
            14 => {
                let mut v10: u16 = 0u16;
                let E0 = Get_Ridley(0i32, game);
                if (*E0).ridley_var_10 != 0 {
                    v10 = 2u16;
                }
                let v13: i32 = v10 as i32 >> 1i32;
                let v14: u16 = ((*E0).base.x_pos).wrapping_add(crate::rtl::rom_table_word!(
                    game,
                    0xa6c804,
                    v13 as isize
                ));
                (*E).base.x_pos = v14;
                (*E).base.y_pos = (*E0).base.y_pos;
                (*E).base.current_instruction =
                    crate::rtl::rom_table_word!(game, 0xa6c808, v13 as isize);
            }
            16 => {
                let mut v16: u16 = 0u16;
                let E0_0 = Get_Ridley(0i32, game);
                if (*E0_0).ridley_var_10 != 0 {
                    v16 = 2u16;
                }
                let v19: i32 = v16 as i32 >> 1i32;
                let v20: u16 = ((*E0_0).base.x_pos).wrapping_add(crate::rtl::rom_table_word!(
                    game,
                    0xa6c836,
                    v19 as isize
                ));
                (*E).base.x_pos = v20;
                (*E).base.y_pos = ((*E0_0).base.y_pos).wrapping_add(22u16);
                (*E).base.current_instruction =
                    crate::rtl::rom_table_word!(game, 0xa6c83a, v19 as isize);
            }
            18 => {
                let mut v22: u16 = 0u16;
                let E0_1 = Get_Ridley(0i32, game);
                if (*E0_1).ridley_var_10 != 0 {
                    v22 = 2u16;
                }
                let v25: i32 = v22 as i32 >> 1i32;
                let v26: u16 = ((*E0_1).base.x_pos).wrapping_add(crate::rtl::rom_table_word!(
                    game,
                    0xa6c868,
                    v25 as isize
                ));
                (*E).base.x_pos = v26;
                (*E).base.y_pos = ((*E0_1).base.y_pos).wrapping_sub(24u16);
                (*E).base.current_instruction =
                    crate::rtl::rom_table_word!(game, 0xa6c86c, v25 as isize);
            }
            20 => {
                let mut v28: u16 = 0u16;
                let E0_2 = Get_Ridley(0i32, game);
                if (*E0_2).ridley_var_10 != 0 {
                    v28 = 2u16;
                }
                let v31: i32 = v28 as i32 >> 1i32;
                let v32: u16 = ((*E0_2).base.x_pos).wrapping_add(crate::rtl::rom_table_word!(
                    game,
                    0xa6c89a,
                    v31 as isize
                ));
                (*E).base.x_pos = v32;
                (*E).base.y_pos = (*E0_2).base.y_pos;
                (*E).base.current_instruction =
                    crate::rtl::rom_table_word!(game, 0xa6c89e, v31 as isize);
            }
            22 => {
                let mut v34: u16 = 0u16;
                let E0_3 = Get_Ridley(0i32, game);
                if (*E0_3).ridley_var_10 != 0 {
                    v34 = 2u16;
                }
                let v37: i32 = v34 as i32 >> 1i32;
                let v38: u16 = ((*E0_3).base.x_pos).wrapping_add(crate::rtl::rom_table_word!(
                    game,
                    0xa6c8cc,
                    v37 as isize
                ));
                (*E).base.x_pos = v38;
                (*E).base.y_pos = ((*E0_3).base.y_pos).wrapping_add(7u16);
                (*E).base.current_instruction =
                    crate::rtl::rom_table_word!(game, 0xa6c8d0, v37 as isize);
            }
            _ => {
                Unreachable(game);
                loop {}
            }
        }
    } else {
        (*E).base.x_pos = ram_word!(game, tilemap_stuff[22]);
        (*E).base.y_pos = ram_word!(game, tilemap_stuff[23]);
        (*E).base.current_instruction = addr_kRidleysExplosion_Ilist_CA47 as u16;
    };
}
pub unsafe fn RidleysExplosion_Main(game: &mut Game) {
    RidleysExplosion_Func_1(ram_word!(game, cur_enemy_index), game);
    let E = Get_RidleysExplosion(ram_word!(game, cur_enemy_index) as i32, game);
    let mut v2: i16 = ((abs16((*E).ren_var_B)) as i16).wrapping_sub(4i16);
    if v2 < 0i16 {
        v2 = 0i16;
    }
    (*E).ren_var_B = (if (*E).ren_var_B & 0x8000 != 0 {
        -(v2 as i32)
    } else {
        v2 as i32
    }) as u16;
    (*E).ren_var_C = ((*E).ren_var_C).wrapping_add(4u16);
    MoveEnemyWithVelocity(game);
    (*E).ren_var_F = (*E).ren_var_F.wrapping_sub(1);
    if ((*E).ren_var_F & 0x8000) != 0 {
        EnemyDeathAnimation(ram_word!(game, cur_enemy_index), 0u16, game);
    }
}
pub unsafe fn RidleysExplosion_Func_1(k: u16, game: &mut Game) {
    let E = Get_RidleysExplosion(k as i32, game);
    let v2: u16 = ((*E).base.frame_counter) & (1u16);
    let mut v3: u16 = 0;
    (*E).base.frame_counter = v2;
    if v2 != 0 {
        v3 = ((*E).base.properties) | ((kEnemyProps_Invisible) as u16);
    } else {
        v3 = ((*E).base.properties as i32 & !(kEnemyProps_Invisible as i32)) as u16;
    }
    (*E).base.properties = v3;
}
pub unsafe fn RidleysExplosion_Func_2(game: &mut Game) {
    SpawnEnemy(0xa6u8, addr_stru_A6C9E7 as u16, game);
    SpawnEnemy(0xa6u8, addr_stru_A6C9D7 as u16, game);
    SpawnEnemy(0xa6u8, addr_stru_A6C9C7 as u16, game);
    SpawnEnemy(0xa6u8, addr_stru_A6C9B7 as u16, game);
    SpawnEnemy(0xa6u8, addr_stru_A6C9A7 as u16, game);
    SpawnEnemy(0xa6u8, addr_stru_A6C997 as u16, game);
    SpawnEnemy(0xa6u8, addr_stru_A6C987 as u16, game);
    SpawnEnemy(0xa6u8, addr_stru_A6C9F7 as u16, game);
    SpawnEnemy(0xa6u8, addr_stru_A6CA07 as u16, game);
    SpawnEnemy(0xa6u8, addr_stru_A6CA27 as u16, game);
    SpawnEnemy(0xa6u8, addr_stru_A6CA17 as u16, game);
    SpawnEnemy(0xa6u8, addr_stru_A6CA37 as u16, game);
}
pub fn locret_A6CB20(game: &mut Game) {}
static off_A6CB21: [Option<Func_V>; 9] = callback_table! { unsafe Func_V;
    locret_A6CB20, Ridley_CBC0, Ridley_Func_71, Ridley_Func_72, Ridley_CBC7, Ridley_CBCE,
    Ridley_Func_73, Ridley_Func_74, Ridley_CBD5,
};
pub unsafe fn Ridley_Func_70(game: &mut Game) {
    Ridley_Func_91(game);
    off_A6CB21[ram_word!(game, tilemap_stuff[0]) as usize].expect("non-null function pointer")(
        game,
    );
    Ridley_Func_86(game);
    Ridley_Func_87(game);
    let E = Get_Ridley(0i32, game);
    if (*E).ridley_var_1B
        | (ram_word!(game, samus_invincibility_timer) as i32
            | (*E).base.properties as i32 & kEnemyProps_Tangible as i32) as u16
        == 0
    {
        Ridley_Func_127(game);
    }
}
pub unsafe fn Ridley_Func_71(game: &mut Game) {
    set_ram_word!(game.tilemap_stuff[10] = 8u16);
    Ridley_Func_75(game);
    set_ram_word!(game.tilemap_stuff[0] = 3u16);
}
pub unsafe fn Ridley_Func_72(game: &mut Game) {
    set_ram_word!(game.tilemap_stuff[10] = 8u16);
    Ridley_Func_75(game);
}
pub unsafe fn Ridley_Func_73(game: &mut Game) {
    set_ram_word!(game.tilemap_stuff[10] = 3u16);
    Ridley_Func_75(game);
    set_ram_word!(game.tilemap_stuff[0] = 6u16);
}
pub unsafe fn Ridley_Func_74(game: &mut Game) {
    set_ram_word!(game.tilemap_stuff[10] = 2u16);
    Ridley_Func_75(game);
    set_ram_word!(game.tilemap_stuff[0] = 7u16);
}
pub unsafe fn Ridley_Func_75(game: &mut Game) {
    if (*Get_Ridley(0i32, game)).ridley_var_10 != 0 {
        set_ram_word!(game.tilemap_stuff[13] = 0x4000u16);
        set_ram_word!(game.tilemap_stuff[14] = -1i32 as u16);
    } else {
        set_ram_word!(game.tilemap_stuff[14] = 0x4000u16);
        set_ram_word!(game.tilemap_stuff[13] = -1i32 as u16);
    }
    Ridley_CBC7(game);
    if ram_word!(game, tilemap_stuff[76])
        | (ram_word!(game, tilemap_stuff[66])
            | ram_word!(game, tilemap_stuff[56])
            | ram_word!(game, tilemap_stuff[46])
            | ram_word!(game, tilemap_stuff[36])
            | ram_word!(game, tilemap_stuff[26])
            | ram_word!(game, tilemap_stuff[16]))
        == 0
    {
        set_ram_word!(game.tilemap_stuff[0] = 4u16);
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
    let mut v0: u16 = 7u16;
    let mut v1: u16 = 0u16;
    loop {
        Ridley_Func_88(v1, game);
        v1 = (v1).wrapping_add(20u16);
        crate::update_place! { previous fresh11 = v0 => v0.wrapping_sub(1); }
        if (fresh11 == 1u16) {
            break;
        }
    }
}
pub unsafe fn Ridley_Func_77(game: &mut Game) {
    let v0: i32 = (*Get_Ridley(0i32, game)).ridley_var_10 as i32;
    set_ram_word!(
        game.tilemap_stuff[11] = crate::rtl::rom_table_word!(game, 0xa6cc12, v0 as isize)
    );
    set_ram_word!(
        game.tilemap_stuff[12] = crate::rtl::rom_table_word!(game, 0xa6cc18, v0 as isize)
    );
}
pub unsafe fn Ridley_Func_78(game: &mut Game) -> u8 {
    ((ram_word!(game, tilemap_stuff[76])
        & (ram_word!(game, tilemap_stuff[66])
            & ram_word!(game, tilemap_stuff[56])
            & ram_word!(game, tilemap_stuff[46])
            & ram_word!(game, tilemap_stuff[36])
            & ram_word!(game, tilemap_stuff[26])
            & ram_word!(game, tilemap_stuff[16])))
        != 0) as u8
}
pub unsafe fn Ridley_Func_79(game: &mut Game) {
    Ridley_Func_77(game);
    if Ridley_Func_78(game) != 0 && Ridley_Func_80_DoubleRet(game) != 0 {
        return;
    }
    if ram_word!(game, tilemap_stuff[76])
        | (ram_word!(game, tilemap_stuff[66])
            | ram_word!(game, tilemap_stuff[56])
            | ram_word!(game, tilemap_stuff[46])
            | ram_word!(game, tilemap_stuff[36])
            | ram_word!(game, tilemap_stuff[26])
            | ram_word!(game, tilemap_stuff[16]))
        == 0
    {
        set_ram_word!(game.tilemap_stuff[16] = 0x8000u16);
        set_ram_word!(game.tilemap_stuff[13] = -1i32 as u16);
        set_ram_word!(game.tilemap_stuff[14] = -1i32 as u16);
        let mut v0: u16 = 2u16;
        if ram_word!(game, samus_x_pos) < 0x70u16 {
            v0 = 1u16;
        }
        set_ram_word!(game.tilemap_stuff[10] = v0);
    }
}
pub unsafe fn Ridley_Func_80_DoubleRet(game: &mut Game) -> u8 {
    let mut current_block: u64;
    let E = Get_Ridley(0i32, game);
    let mut retval: bool = false_0 != 0;
    let mut v1: u16 = 0;
    if ram_word!(game, tilemap_stuff[2]) != 0u16
        && (ram_word!(game, tilemap_stuff[13]) & ram_word!(game, tilemap_stuff[14]) & 0x8000) != 0
    {
        current_block = 5875655901067838705;
    } else if ram_word!(game, tilemap_stuff[1]) == 0 {
        current_block = 15129621311889290514;
    } else if (ram_word!(game, random_number) as u8) >= 0xf0u8 {
        current_block = 5875655901067838705;
    } else if abs16((ram_word!(game, samus_x_pos)).wrapping_sub((*E).base.x_pos)) < 0x80u16 {
        v1 = 0u16;
        current_block = 993568416893248401;
    } else {
        current_block = 15129621311889290514;
    }
    if current_block == 5875655901067838705 {
        v1 = (ram_word!(game, tilemap_stuff[2])).wrapping_sub(1u16);
        current_block = 993568416893248401;
    }
    if current_block == 993568416893248401 {
        Ridley_Func_89(v1, game);
        retval = true_0 != 0;
    }
    set_ram_word!(game.tilemap_stuff[2] = 0u16);
    retval as u8
}
pub unsafe fn Ridley_Func_81(game: &mut Game) {
    Ridley_Func_77(game);
    if Ridley_Func_78(game) & 1 != 0 && Ridley_Func_82_DoubleRet(game) {
        return;
    }
    if ram_word!(game, tilemap_stuff[76])
        | (ram_word!(game, tilemap_stuff[66])
            | ram_word!(game, tilemap_stuff[56])
            | ram_word!(game, tilemap_stuff[46])
            | ram_word!(game, tilemap_stuff[36])
            | ram_word!(game, tilemap_stuff[26])
            | ram_word!(game, tilemap_stuff[16]))
        == 0
    {
        set_ram_word!(game.tilemap_stuff[16] = 0x8000u16);
        set_ram_word!(game.tilemap_stuff[13] = -1i32 as u16);
        set_ram_word!(game.tilemap_stuff[14] = -1i32 as u16);
        let mut v0: u16 = 2u16;
        if ram_word!(game, samus_x_pos) < 0x70u16 {
            v0 = 1u16;
        }
        set_ram_word!(game.tilemap_stuff[10] = v0);
    }
}
pub unsafe fn Ridley_Func_82_DoubleRet(game: &mut Game) -> bool {
    if ram_word!(game, tilemap_stuff[2]) != 0
        && (ram_word!(game, tilemap_stuff[13]) & ram_word!(game, tilemap_stuff[14]) & 0x8000) != 0
    {
        Ridley_Func_89(1u16, game);
        set_ram_word!(game.tilemap_stuff[2] = 0u16);
        return true_0 != 0;
    }
    false_0 != 0
}
pub unsafe fn Ridley_Func_83(game: &mut Game) {
    let E = Get_Ridley(0i32, game);
    Ridley_Func_77(game);
    if Ridley_Func_78(game) & 1 != 0
        && ((ram_word!(game, random_number) as u8) >= 0xf0u8
            || abs16((ram_word!(game, samus_x_pos)).wrapping_sub((*E).base.x_pos)) < 0x80u16)
        && (ram_word!(game, tilemap_stuff[13]) & ram_word!(game, tilemap_stuff[14]) & 0x8000) != 0
    {
        set_ram_word!(game.tilemap_stuff[13] = 16128u16);
        set_ram_word!(game.tilemap_stuff[10] = 8u16);
    } else if ram_word!(game, tilemap_stuff[76])
        | (ram_word!(game, tilemap_stuff[66])
            | ram_word!(game, tilemap_stuff[56])
            | ram_word!(game, tilemap_stuff[46])
            | ram_word!(game, tilemap_stuff[36])
            | ram_word!(game, tilemap_stuff[26])
            | ram_word!(game, tilemap_stuff[16]))
        == 0
    {
        if ((*Get_Ridley(0i32, game)).ridley_var_C & 0x8000) == 0 {
            set_ram_word!(game.tilemap_stuff[0] = 5u16);
        }
        set_ram_word!(game.tilemap_stuff[13] = -1i32 as u16);
        set_ram_word!(game.tilemap_stuff[14] = -1i32 as u16);
        let mut v1: u16 = ram_word!(game, tilemap_stuff[8]);
        if ram_word!(game, tilemap_stuff[8]) == 0
            || {
                v1 = (ram_word!(game, tilemap_stuff[8])).wrapping_sub(1u16);
                crate::update_place! { reference fresh14 = *(ram_ptr!(game, u16; 0x2000)).offset(8) => v1; }
                *fresh14 != 0u16
            }
        {
            set_ram_word!(game.tilemap_stuff[10] = v1);
        } else {
            set_ram_word!(game.tilemap_stuff[16] = 0x8000u16);
            Ridley_Func_85(game);
            set_ram_word!(game.tilemap_stuff[10] = 8u16);
        }
    }
}
pub unsafe fn Ridley_Func_84(game: &mut Game) {
    let E = Get_Ridley(0i32, game);
    Ridley_Func_77(game);
    if Ridley_Func_78(game) & 1 != 0
        && ((ram_word!(game, random_number) as u8) >= 0xf0u8
            || abs16((ram_word!(game, samus_x_pos)).wrapping_sub((*E).base.x_pos)) < 0x80u16)
        && (ram_word!(game, tilemap_stuff[13]) & ram_word!(game, tilemap_stuff[14]) & 0x8000) != 0
    {
        set_ram_word!(game.tilemap_stuff[13] = 16128u16);
        set_ram_word!(game.tilemap_stuff[10] = 8u16);
    } else if ram_word!(game, tilemap_stuff[76])
        | (ram_word!(game, tilemap_stuff[66])
            | ram_word!(game, tilemap_stuff[56])
            | ram_word!(game, tilemap_stuff[46])
            | ram_word!(game, tilemap_stuff[36])
            | ram_word!(game, tilemap_stuff[26])
            | ram_word!(game, tilemap_stuff[16]))
        == 0
    {
        if ((*E).ridley_var_C & 0x8000) == 0 {
            set_ram_word!(game.tilemap_stuff[0] = 6u16);
            set_ram_words! { game, 2560u16;
                tilemap_stuff[20], tilemap_stuff[30], tilemap_stuff[40], tilemap_stuff[50],
                tilemap_stuff[60], tilemap_stuff[70], tilemap_stuff[80],
            }
            set_ram_words! { game, 0x4000u16;
                tilemap_stuff[21], tilemap_stuff[31], tilemap_stuff[41], tilemap_stuff[51],
                tilemap_stuff[61], tilemap_stuff[71], tilemap_stuff[81],
            }
            set_ram_word!(game.tilemap_stuff[16] = 0x8000u16);
        }
        set_ram_word!(game.tilemap_stuff[13] = -1i32 as u16);
        set_ram_word!(game.tilemap_stuff[14] = -1i32 as u16);
        let mut v1: u16 = ram_word!(game, tilemap_stuff[8]);
        if ram_word!(game, tilemap_stuff[8]) != 0 {
            v1 = (ram_word!(game, tilemap_stuff[8])).wrapping_sub(1u16);
            set_ram_word!(game.tilemap_stuff[8] = v1);
            if v1 == 0 {
                set_ram_word!(game.tilemap_stuff[16] = 0x8000u16);
                Ridley_Func_85(game);
                v1 = 8u16;
            }
        }
        set_ram_word!(game.tilemap_stuff[10] = v1);
    }
}
// Tail words alias the shared tilemap workspace; keep each load/store ordered.
unsafe fn ridley_tail_word(ram: *mut u8, index: usize) -> u16 {
    u16::from_le(ram.add(0x2000 + index * 2).cast::<u16>().read_unaligned())
}
unsafe fn ridley_tail_store(ram: *mut u8, index: usize, value: u16) {
    ram.add(0x2000 + index * 2)
        .cast::<u16>()
        .write_unaligned(value.to_le());
}
unsafe fn ridley_tail_chain(ram: *mut u8, first: usize) {
    for index in (first + 10..=first + 50).step_by(20) {
        let value =
            ridley_tail_word(ram, index + 2).wrapping_add(ridley_tail_word(ram, index - 10));
        ridley_tail_store(ram, index, value);
        let value = ridley_tail_word(ram, index + 12)
            .wrapping_add(ridley_tail_word(ram, index + 2))
            .wrapping_add(ridley_tail_word(ram, index - 10));
        ridley_tail_store(ram, index + 10, value);
    }
}
unsafe fn ridley_tail_length(ram: *mut u8, index: usize, maximum: u16, resting: u16) {
    if ridley_tail_word(ram, index + 1) != 0 {
        if (ridley_tail_word(ram, index + 1) as i16)
            .wrapping_sub(ridley_tail_word(ram, index) as i16)
            < 0
        {
            ridley_tail_store(ram, index + 1, 0);
        }
        let mut value = ridley_tail_word(ram, 9).wrapping_add(ridley_tail_word(ram, index));
        if (ridley_tail_word(ram, 9) as i32 + ridley_tail_word(ram, index) as i32 - maximum as i32)
            & 0x8000
            == 0
        {
            value = maximum;
        }
        ridley_tail_store(ram, index, value);
    } else if (resting as i16).wrapping_sub(ridley_tail_word(ram, index) as i16) < 0 {
        let value = ridley_tail_word(ram, index).wrapping_sub(128);
        ridley_tail_store(ram, index, value);
    }
}
pub unsafe fn Ridley_Func_85(game: &mut Game) {
    let ram = (&raw mut game.g_ram).cast::<u8>();
    for index in [21usize, 31, 41, 51] {
        let value = ram.add(0x2000 + index * 2).read() as u16 | 0x4000;
        ridley_tail_store(ram, index, value);
    }
    // Segment 61 deliberately uses word 21, including its bit 10.
    let value = ridley_tail_word(ram, 21) & 0x4ff | 0x4000;
    ridley_tail_store(ram, 61, value);
    for index in [71usize, 81] {
        let value = ram.add(0x2000 + index * 2).read() as u16 | 0x4000;
        ridley_tail_store(ram, index, value);
    }
}
pub unsafe fn Ridley_Func_86(game: &mut Game) {
    let E = Get_Ridley(0i32, game);
    set_ram_word!(
        game.tilemap_stuff[23] =
            (ram_word!(game, tilemap_stuff[25]) as i32 + (*E).base.y_pos as i32 + 16i32) as u16
    );
    let ram = (&raw mut game.g_ram).cast::<u8>();
    ridley_tail_chain(ram, 23);
    let mut v1: i16 = (((*E).ridley_var_10) as i16).wrapping_sub(1i16);
    if (*E).ridley_var_10 != 1u16 {
        v1 = ((ram_word!(game, tilemap_stuff[24])) as i16)
            .wrapping_add(g_word_A6CF54[(*E).ridley_var_10 as usize]);
    }
    set_ram_word!(game.tilemap_stuff[22] = ((*E).base.x_pos).wrapping_add((v1) as u16));
    if (*E).ridley_var_10 == 1u16 {
        set_ram_word!(game.tilemap_stuff[32] = (*E).base.x_pos);
        for index in [42usize, 52, 62, 72, 82] {
            let value = ridley_tail_word(ram, 32);
            ridley_tail_store(ram, index, value);
        }
    } else {
        ridley_tail_chain(ram, 22);
    };
}
pub unsafe fn Ridley_Func_87(game: &mut Game) {
    let ram = (&raw mut game.g_ram).cast::<u8>();
    ridley_tail_length(ram, 29, 6144, 2048);
    ridley_tail_length(ram, 39, 6144, 2048);
    ridley_tail_length(ram, 49, 5632, 2048);
    ridley_tail_length(ram, 59, 5632, 2048);
    ridley_tail_length(ram, 69, 4608, 2048);
    ridley_tail_length(ram, 79, 1280, 1280);
}
pub unsafe fn Ridley_Func_88(k: u16, game: &mut Game) {
    let mut current_block: u64;
    let mut v6: i16 = 0;
    let mut v2: u16 = 0;
    let v1: i32 = k as i32 >> 1i32;
    if (ram_word!(game, tilemap_stuff[(v1 + 16i32)]) & 0x8000) != 0 {
        let v4: i32 = k as i32 >> 1i32;
        let v5: u16 = ram_word!(game, tilemap_stuff[(v4 + 17i32)]);
        if v5 < ram_word!(game, tilemap_stuff[15]) {
            update_ram_word!(
                game.tilemap_stuff[((k as i32 >> 1i32) + 17i32)],
                wrapping_add(ram_word!(game, tilemap_stuff[10]))
            );
            return;
        }
        if v5 != 0xffffu16 {
            set_ram_word!(game.tilemap_stuff[(v4 + 17i32)] = -1i32 as u16);
            set_ram_word!(game.tilemap_stuff[(v4 + 26i32)] = 0x8000u16);
            set_ram_word!(
                game.tilemap_stuff[(v4 + 28i32)] = ram_word!(game, tilemap_stuff[(v4 + 18i32)])
            );
        }
        if (ram_word!(game, tilemap_stuff[(v4 + 18i32)]) & 0x8000) == 0 {
            if (ram_word!(game, tilemap_stuff[14]) & 0x8000) != 0 {
                v2 = (ram_word!(game, tilemap_stuff[10]))
                    .wrapping_add(ram_word!(game, tilemap_stuff[(v4 + 21i32)]));
                if ((v2) as i16).wrapping_sub((ram_word!(game, tilemap_stuff[12])) as i16) >= 0i16 {
                    current_block = 4838848238792072683;
                } else {
                    current_block = 4216153537599944117;
                }
            } else {
                set_ram_word!(game.tilemap_stuff[(v4 + 20i32)] = 3072u16);
                v2 = (ram_word!(game, tilemap_stuff[10]))
                    .wrapping_add(ram_word!(game, tilemap_stuff[(v4 + 21i32)]));
                if ((v2) as i16).wrapping_sub((ram_word!(game, tilemap_stuff[14])) as i16) >= 0i16 {
                    if k == 0 || ram_word!(game, tilemap_stuff[(v4 + 6i32)]) == 0 {
                        set_ram_word!(
                            game.tilemap_stuff[(v4 + 21i32)] = ram_word!(game, tilemap_stuff[14])
                        );
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
                    set_ram_word!(game.tilemap_stuff[(v4 + 18i32)] = 0x8000u16);
                    v2 = ram_word!(game, tilemap_stuff[12]);
                    current_block = 4216153537599944117;
                }
            }
        } else {
            if (ram_word!(game, tilemap_stuff[13]) & 0x8000) != 0 {
                v6 = (ram_word!(game, tilemap_stuff[(v4 + 21i32)]) as i32
                    - ram_word!(game, tilemap_stuff[10]) as i32
                    - 1i32) as i16;
                if (v6).wrapping_sub((ram_word!(game, tilemap_stuff[11])) as i16) < 0i16 {
                    current_block = 9260896848985358503;
                } else {
                    current_block = 2873832966593178012;
                }
            } else {
                set_ram_word!(game.tilemap_stuff[(v4 + 20i32)] = 3072u16);
                v6 = (ram_word!(game, tilemap_stuff[(v4 + 21i32)]) as i32
                    - ram_word!(game, tilemap_stuff[10]) as i32
                    - 1i32) as i16;
                if (v6).wrapping_sub((ram_word!(game, tilemap_stuff[13])) as i16) < 0i16 {
                    if k == 0 || ram_word!(game, tilemap_stuff[(v4 + 6i32)]) == 0 {
                        set_ram_word!(
                            game.tilemap_stuff[(v4 + 21i32)] = ram_word!(game, tilemap_stuff[13])
                        );
                        current_block = 5689001924483802034;
                    } else {
                        current_block = 9260896848985358503;
                    }
                } else {
                    current_block = 2873832966593178012;
                }
            }
            if current_block == 5689001924483802034 {
            } else {
                if current_block == 2873832966593178012 {
                    v2 = ((v6) as u16).wrapping_add(1u16);
                } else {
                    set_ram_word!(game.tilemap_stuff[(v4 + 18i32)] = 0u16);
                    v2 = ram_word!(game, tilemap_stuff[11]);
                }
                current_block = 4216153537599944117;
            }
        }
        match current_block {
            5689001924483802034 => {
                let v3: i32 = k as i32 >> 1i32;
                set_ram_words! { game, 0u16;
                    tilemap_stuff[(v3 + 16i32)], tilemap_stuff[(v3 + 17i32)],
                }
                update_ram_word!(game.tilemap_stuff[(v3 + 18i32)], |fresh13| (fresh13)
                    ^ (0x8000u16));
                return;
            }
            _ => {
                set_ram_word!(game.tilemap_stuff[(v4 + 21i32)] = v2);
            }
        }
    } else {
        v2 = ram_word!(game, tilemap_stuff[(v1 + 21i32)]);
    }
    let mut r18: u16 = v2 as u8 as u16;
    if k != 0 {
        r18 = ((*((ram_ptr!(game, u16; 0x2000)).offset(((k as i32 >> 1i32) + 11i32) as isize)
            as *mut u8)
            .offset(0))
        .wrapping_add((r18) as u8)) as u16;
    }
    let v7: i32 = k as i32 >> 1i32;
    set_ram_word!(
        game.tilemap_stuff[(v7 + 24i32)] = ComputeSinMult(
            *((ram_ptr!(game, u16; 0x2000)).offset((v7 + 19i32) as isize) as *mut u8)
                .offset(1isize,) as u16,
            r18,
            game,
        )
    );
    set_ram_word!(
        game.tilemap_stuff[(v7 + 25i32)] = ComputeCosMult(
            *((ram_ptr!(game, u16; 0x2000)).offset((v7 + 19i32) as isize) as *mut u8)
                .offset(1isize,) as u16,
            r18,
            game,
        )
    );
}
pub unsafe fn Ridley_Func_89(a: u16, game: &mut Game) {
    let mut v1: u16 = 0;
    let mut r18: u16 = 0;
    let mut r20: u16 = 0;
    let mut v2: i8 = a as i8;
    *(&raw mut v1 as *mut u8).offset(0) = 0u8;
    *(&raw mut v1 as *mut u8).offset(1isize) = v2 as u8;
    set_ram_word!(game.tilemap_stuff[5] = v1);
    let E = Get_Ridley(0i32, game);
    if (*E).ridley_var_10 != 1u16 && (*E).ridley_var_01 != 0u16 {
        let mut pt: Point16U = Point16U { x: 0, y: 0 };
        if Ridley_Func_90(&raw mut pt, game) != 0 {
            r18 = (pt.x).wrapping_sub(ram_word!(game, tilemap_stuff[22]));
            r20 = (pt.y).wrapping_sub(ram_word!(game, tilemap_stuff[23]));
        } else {
            r18 = (ram_word!(game, samus_x_pos)).wrapping_sub(ram_word!(game, tilemap_stuff[22]));
            r20 = (ram_word!(game, samus_y_pos) as i32 + 24i32
                - ram_word!(game, tilemap_stuff[23]) as i32) as u16;
        }
        r18 = -(CalculateAngleFromXY(r18, r20, game) as i32 + 0x80i32) as u8 as u16;
        if (*E).ridley_var_10 != 0 {
            let mut v7: u16 = r18;
            if r18 >= 0x18u16 && r18 < 0xe8u16 {
                v7 = 24u16;
            }
            let v8: u16 = (ram_word!(game, tilemap_stuff[5]) as i32 + v7 as i32 + 0x4000i32) as u16;
            if v8 >= ram_word!(game, tilemap_stuff[21]) {
                set_ram_word!(game.tilemap_stuff[14] = v8);
                set_ram_word!(game.tilemap_stuff[10] = 8u16);
            }
        } else {
            let mut v5: u16 = r18;
            if r18 >= 0x18u16 && r18 < 0xe8u16 {
                v5 = 232u16;
            }
            let v6: u16 = (v5 as i32 + 16128i32 - ram_word!(game, tilemap_stuff[5]) as i32) as u16;
            if v6 < ram_word!(game, tilemap_stuff[21]) {
                set_ram_word!(game.tilemap_stuff[13] = v6);
                set_ram_word!(game.tilemap_stuff[10] = 8u16);
            }
        }
    }
}
pub unsafe fn Ridley_Func_90(out: *mut Point16U, game: &mut Game) -> u8 {
    let rect: Rect16U = Rect16U {
        x: ram_word!(game, tilemap_stuff[82]),
        y: ram_word!(game, tilemap_stuff[83]),
        w: 64u16,
        h: 64u16,
    };
    if ram_word!(game, projectile_counter) == 0 {
        return 0u8;
    }
    let mut v1: u16 = 0u16;
    loop {
        let v2: i32 = v1 as i32 >> 1i32;
        if (*((ram_ptr!(game, u16; 0xc18)).offset(v2 as isize) as *mut u8).offset(1isize) & 0xf)
            == 1
            || (*((ram_ptr!(game, u16; 0xc18)).offset(v2 as isize) as *mut u8).offset(1isize) & 0xf)
                == 2
        {
            let v3: u16 = abs16((ram_word!(game, projectile_x_pos[v2])).wrapping_sub(rect.x));
            let mut v4: bool = v3 < ram_word!(game, projectile_x_radius[v2]);
            let v5: u16 = (v3).wrapping_sub(ram_word!(game, projectile_x_radius[v2]));
            if v4 || v5 < rect.w {
                let v6: u16 = abs16((ram_word!(game, projectile_y_pos[v2])).wrapping_sub(rect.y));
                v4 = v6 < ram_word!(game, projectile_y_radius[v2]);
                let v7: u16 = (v6).wrapping_sub(ram_word!(game, projectile_y_radius[v2]));
                if v4 || v7 < rect.h {
                    break;
                }
            }
        }
        v1 = (v1).wrapping_add(2u16);
        if ((v1) as i16).wrapping_sub(10i16) >= 0i16 {
            return 0u8;
        }
    }
    let v8: i32 = v1 as i32 >> 1i32;
    (*out).x = ram_word!(game, projectile_x_pos[v8]);
    (*out).y = ram_word!(game, projectile_y_pos[v8]);
    1u8
}
pub unsafe fn Ridley_Func_91(game: &mut Game) {
    let E = Get_Ridley(0i32, game);
    if ram_word!(game, tilemap_stuff[10]) as i32 - 8i32 & 0x8000i32 == 0 {
        let v1: u16 = ((*E).ridley_var_0E).wrapping_add(1u16);
        (*E).ridley_var_0E = v1;
        if v1 as i32 - 16i32 & 0x8000i32 != 0 {
            return;
        }
        if (*E).ridley_var_0F == 0 {
            QueueSfx3_Max6(0x21u16, game);
        }
    }
    (*E).ridley_var_0E = 0u16;
}
pub unsafe fn Ridley_Func_92(game: &mut Game) {
    set_ram_word!(game.tilemap_stuff[10] = 1u16);
    set_ram_word!(game.tilemap_stuff[11] = 16368u16);
    set_ram_word!(game.tilemap_stuff[12] = 16448u16);
    set_ram_word!(game.tilemap_stuff[13] = -1i32 as u16);
    set_ram_word!(game.tilemap_stuff[14] = -1i32 as u16);
    set_ram_word!(game.tilemap_stuff[9] = 240u16);
    set_ram_word!(game.tilemap_stuff[15] = 16u16);
    set_ram_words! { game, 17u16;
        tilemap_stuff[17], tilemap_stuff[27], tilemap_stuff[37], tilemap_stuff[47],
        tilemap_stuff[57], tilemap_stuff[67], tilemap_stuff[77],
    }
    Ridley_Func_93(0x2024u16, addr_word_A6D36E as u16, game);
    Ridley_Func_93(0x2026u16, addr_word_A6D37C as u16, game);
    Ridley_Func_93(0x202au16, addr_word_A6D38A as u16, game);
    Ridley_Func_93(0x202cu16, addr_word_A6D398 as u16, game);
    Ridley_Func_93(0x202eu16, addr_word_A6D3A6 as u16, game);
    set_ram_words! { game, 0u16;
        tilemap_stuff[16], tilemap_stuff[26], tilemap_stuff[36], tilemap_stuff[46],
        tilemap_stuff[56], tilemap_stuff[66], tilemap_stuff[76],
    }
}
pub unsafe fn Ridley_Func_93(j: u16, k: u16, game: &mut Game) {
    let mut dst: *mut u16 = ram_ptr!(game, u16; j as i32 as isize);
    let mut src = RomPtr_A6(k, game) as *const u16;
    for _ in 0i32..7i32 {
        *dst = *src;
        dst = dst.offset(10);
        src = src.offset(1);
    }
}
crate::forward_calls! { pub unsafe fn (game: &mut Game);
    Ridley_Func_94 => Ridley_Func_D3DC(0u16, game);
    Ridley_Func_95 => Ridley_Func_D3DC(0x8000u16, game);
}
pub unsafe fn Ridley_Func_D3DC(a: u16, game: &mut Game) {
    set_ram_word!(game.tilemap_stuff[16] = a);
    set_ram_word!(game.tilemap_stuff[26] = a);
    set_ram_word!(game.tilemap_stuff[36] = a);
    set_ram_word!(game.tilemap_stuff[46] = a);
    set_ram_word!(game.tilemap_stuff[56] = a);
    set_ram_word!(game.tilemap_stuff[66] = a);
    set_ram_word!(game.tilemap_stuff[76] = a);
}
pub unsafe fn Ridley_Func_96(game: &mut Game) {
    Ridley_Func_93(0x2026u16, addr_word_A6D37C as u16, game);
    let mut v1: u16 = 0u16;
    for _ in 0i16..7i16 {
        let v2: i32 = v1 as i32 >> 1i32;
        set_ram_word!(
            game.tilemap_stuff[(v2 + 21i32)] =
                (0x8000u16).wrapping_sub(ram_word!(game, tilemap_stuff[(v2 + 21i32)]),)
        );
        update_ram_word!(game.tilemap_stuff[(v2 + 18i32)], |fresh3| (fresh3)
            | (0x8000u16));
        v1 = (v1).wrapping_add(20u16);
    }
}
pub unsafe fn Ridley_Func_97(game: &mut Game) -> u8 {
    ((ram_word!(game, tilemap_stuff[76])
        & (ram_word!(game, tilemap_stuff[66])
            & ram_word!(game, tilemap_stuff[56])
            & ram_word!(game, tilemap_stuff[46])
            & ram_word!(game, tilemap_stuff[36])
            & ram_word!(game, tilemap_stuff[26])
            & ram_word!(game, tilemap_stuff[16])))
        != 0) as u8
}
pub unsafe fn Ridley_Func_98(game: &mut Game) {
    let enemy_ptr: u16 = (*Get_Ridley(ram_word!(game, cur_enemy_index) as i32, game))
        .base
        .enemy_ptr;
    let v1 = RomPtr_A0(enemy_ptr, game);
    Samus_DealDamage(SuitDamageDivision(*(v1.offset(6) as *mut u16), game), game);
}
pub unsafe fn Ridley_Func_99(a: u16, game: &mut Game) {
    let E = Get_Ridley(0i32, game);
    enemy_instruction! {[(*E).base]; instruction = a, timer = 1u16}
    (*E).base.timer = 0u16;
}
pub unsafe fn Ridley_Func_100(game: &mut Game) {
    let mut v0: i16 = 0i16;
    let E = Get_Ridley(0i32, game);
    let health: u16 = (*E).base.health;
    if health < 0x2328u16 {
        v0 = 1i16;
        if health < 0x1518u16 {
            v0 = 2i16;
            if health < 0x708u16 {
                v0 = 3i16;
            }
        }
    }
    (*E).ridley_var_12 = v0 as u16;
    if (v0).wrapping_sub(1i16) >= 0i16 {
        Ridley_D495(((v0) as u16).wrapping_sub(1u16), game);
    }
}
pub unsafe fn Ridley_D495(r18: u16, game: &mut Game) {
    if ram_word!(game, palette_change_num) == 0 {
        WriteColorsToPalette(
            0x1e2u16,
            0xa6u8,
            (28i32 * r18 as i32 - 7062i32) as u16,
            0xeu16,
            game,
        );
    }
}
pub unsafe fn Ridley_Func_101(game: &mut Game) {
    let E = Get_Ridley(0i32, game);
    if (*E).ridley_var_01 != 0u16 && (*E).ridley_var_0D >= 0x32u16 {
        Ridley_D495(
            (if (*E).ridley_var_0D >= 0x46u16 {
                2i32
            } else {
                0i32
            }) as u16,
            game,
        );
    }
}
pub unsafe fn Ridley_Func_102(game: &mut Game) {
    let mut v0: i16 = 3584i16;
    let E = Get_Ridley(0i32, game);
    if (*E).base.flash_timer >= 2u16
        && (ram_word!(game, random_enemy_counter) as i32 + 1i32 & 2i32) != 0i32
    {
        v0 = 0i16;
    }
    (*E).ridley_var_0C = v0 as u16;
}
pub unsafe fn Ridley_Func_103(k: u16, j: u16, game: &mut Game) -> u8 {
    let prod: u16 = Mult8x8((j >> 4) as u8, ram_word!(game, room_width_in_blocks) as u8);
    ((*(ram_ptr!(game, u16; 0x10002)).offset((prod as i32 + (k as i32 >> 4i32)) as isize) & 0xf000)
        != 0) as u8
}
pub unsafe fn Ridley_Func_104_0(k: u16, j: u16, r18: u16, r20: u16, game: &mut Game) {
    Ridley_Func_104(k, j, 0u16, r18, r20, game);
}
pub unsafe fn Ridley_Func_104(k: u16, j: u16, a: u16, r18: u16, r20: u16, game: &mut Game) {
    let R26: u16 = a;
    let r24: u16 = g_byte_A6D61F[j as usize] as u16;
    Ridley_Func_105(k, r18, r24, a, game);
    let E = Get_Ridley(k as i32, game);
    let mut v4: i16 = (((*E).base.y_pos) as i16).wrapping_sub((r20) as i16);
    if v4 != 0 {
        if v4 >= 0i16 {
            let mut RegWord: u16 = SnesDivide(v4 as u16, r24 as u8);
            if RegWord == 0 {
                RegWord = 1u16;
            }
            let r22: u16 = RegWord;
            let mut ridley_var_C: i16 = (*E).ridley_var_C as i16;
            let v14: bool = 0u8 != 0u8;
            if ridley_var_C >= 0i16 {
                let mut v15: u16 = ((ridley_var_C) as u16).wrapping_sub(R26);
                v15 = (v15).wrapping_sub(8u16);
                let v16: u16 = r22;
                ridley_var_C = ((v15) as i16).wrapping_sub((v16) as i16);
            }
            let mut v17: i16 = (ridley_var_C).wrapping_sub((r22) as i16);
            if v17 as i32 + 1280i32 & 0x8000i32 != 0 {
                v17 = -1280i16;
            }
            (*E).ridley_var_C = v17 as u16;
        } else {
            let mut v5: u16 = SnesDivide((r20).wrapping_sub((*E).base.y_pos), r24 as u8);
            if v5 == 0 {
                v5 = 1u16;
            }
            let r22_0: u16 = v5;
            let mut v6: i16 = (*E).ridley_var_C as i16;
            let v7: bool = 0u8 != 0u8;
            if v6 < 0i16 {
                let mut v8: i16 = ((R26) as i16).wrapping_add(v6);
                v8 = (v8).wrapping_add(8i16);
                let mut v9: i16 = v8;
                v6 = ((r22_0) as i16).wrapping_add(v9);
            }
            let mut v11: i16 = ((r22_0) as i16).wrapping_add(v6);
            if v11 as i32 - 1280i32 & 0x8000i32 == 0 {
                v11 = 1280i16;
            }
            (*E).ridley_var_C = v11 as u16;
        }
    }
}
pub unsafe fn Ridley_Func_105(k: u16, r18: u16, r24: u16, r26: u16, game: &mut Game) {
    let E = Get_Ridley(k as i32, game);
    let mut v2: i16 = (((*E).base.x_pos) as i16).wrapping_sub((r18) as i16);
    if v2 != 0 {
        if v2 >= 0i16 {
            let mut RegWord: u16 = SnesDivide(v2 as u16, r24 as u8);
            if RegWord == 0 {
                RegWord = 1u16;
            }
            let r22: u16 = RegWord;
            let mut ridley_var_B: i16 = (*E).ridley_var_B as i16;
            let v12: bool = 0u8 != 0u8;
            if ridley_var_B >= 0i16 {
                let mut v13: u16 = ((ridley_var_B) as u16).wrapping_sub(r26);
                v13 = (v13).wrapping_sub(8u16);
                let v14: u16 = r22;
                ridley_var_B = ((v13) as i16).wrapping_sub((v14) as i16);
            }
            let mut v15: i16 = (ridley_var_B).wrapping_sub((r22) as i16);
            if v15 as i32 + 1280i32 & 0x8000i32 != 0 {
                v15 = -1280i16;
            }
            (*E).ridley_var_B = v15 as u16;
        } else {
            let mut v3: u16 = SnesDivide((r18).wrapping_sub((*E).base.x_pos), r24 as u8);
            if v3 == 0 {
                v3 = 1u16;
            }
            let r22_0: u16 = v3;
            let mut v4: i16 = (*E).ridley_var_B as i16;
            if v4 < 0i16 {
                let mut v6: i16 = ((r26) as i16).wrapping_add(v4);
                v6 = (v6).wrapping_add(8i16);
                let mut v7: i16 = v6;
                v4 = ((r22_0) as i16).wrapping_add(v7);
            }
            let mut v9: i16 = ((r22_0) as i16).wrapping_add(v4);
            if v9 as i32 - 1280i32 & 0x8000i32 == 0 {
                v9 = 1280i16;
            }
            (*E).ridley_var_B = v9 as u16;
        }
    }
}
pub unsafe fn Ridley_Func_106(k: u16, j: u16, r18: u16, r20: u16, game: &mut Game) {
    let r24: u16 = g_byte_A6D712[j as usize] as u16;
    Ridley_Func_107(k, r18, r24, game);
    let E = Get_Ridley(k as i32, game);
    let mut v3: i16 = (((*E).base.y_pos) as i16).wrapping_sub((r20) as i16);
    if v3 != 0 {
        if v3 >= 0i16 {
            let mut RegWord: u16 = SnesDivide(v3 as u16, r24 as u8);
            if RegWord == 0 {
                RegWord = 1u16;
            }
            let r22: u16 = RegWord;
            let mut ridley_var_C: i16 = (*E).ridley_var_C as i16;
            if ridley_var_C >= 0i16 {
                ridley_var_C = (ridley_var_C as i32 - r22 as i32 - r22 as i32) as i16;
            }
            let mut v9: i16 = (ridley_var_C).wrapping_sub((r22) as i16);
            if v9 as i32 + 1280i32 & 0x8000i32 != 0 {
                v9 = -1280i16;
            }
            (*E).ridley_var_C = v9 as u16;
        } else {
            let mut v4: u16 = SnesDivide((r20).wrapping_sub((*E).base.y_pos), r24 as u8);
            if v4 == 0 {
                v4 = 1u16;
            }
            let r22_0: u16 = v4;
            let mut v5: i16 = (*E).ridley_var_C as i16;
            if v5 < 0i16 {
                v5 = (v5 as i32 + (r22_0 as i32 + r22_0 as i32)) as i16;
            }
            let mut v6: i16 = ((r22_0) as i16).wrapping_add(v5);
            if v6 as i32 - 1280i32 & 0x8000i32 == 0 {
                v6 = 1280i16;
            }
            (*E).ridley_var_C = v6 as u16;
        }
    }
}
pub unsafe fn Ridley_Func_107(k: u16, r18: u16, r24: u16, game: &mut Game) {
    let E = Get_Ridley(k as i32, game);
    let v2: i16 = (((*E).base.x_pos) as i16).wrapping_sub((r18) as i16);
    if v2 == 0 {
        return;
    }
    if v2 >= 0i16 {
        let mut RegWord: u16 = SnesDivide(v2 as u16, r24 as u8);
        RegWord = (if RegWord != 0u16 {
            RegWord as i32
        } else {
            1i32
        }) as u16;
        let r22: u16 = RegWord;
        let mut s: i16 = (*E).ridley_var_B as i16;
        if s >= 0i16 {
            s = (s as i32 - 2i32 * RegWord as i32) as i16;
        }
        s = (s).wrapping_sub((RegWord) as i16);
        (*E).ridley_var_B = (if s as i32 + 1280i32 & 0x8000i32 != 0 {
            -1280i32
        } else {
            s as i32
        }) as u16;
    } else {
        let v3: u16 = SnesDivide((r18).wrapping_sub((*E).base.x_pos), r24 as u8);
        let r22_0: u16 = (if v3 != 0u16 { v3 as i32 } else { 1i32 }) as u16;
        let mut s_0: i16 = (*E).ridley_var_B as i16;
        if s_0 < 0i16 {
            s_0 = (s_0 as i32 + 2i32 * r22_0 as i32) as i16;
        }
        s_0 = (s_0).wrapping_add((r22_0) as i16);
        (*E).ridley_var_B = (if s_0 as i32 - 1280i32 & 0x8000i32 == 0 {
            1280i32
        } else {
            s_0 as i32
        }) as u16;
    };
}
pub unsafe fn Ridley_Func_111(mut r18: u16, r20: u16, r22: u16, game: &mut Game) {
    let E = Get_Ridley(0i32, game);
    if r22 != (*E).ridley_var_0B {
        if ((r22) as i16).wrapping_sub(((*E).ridley_var_0B) as i16) >= 0i16 {
            let mut v2: u16 = ((*E).ridley_var_0B).wrapping_add(32u16);
            if v2 as i32 - r22 as i32 & 0x8000i32 == 0 {
                v2 = r22;
            }
            (*E).ridley_var_0B = v2;
        } else {
            let mut v1: u16 = ((*E).ridley_var_0B).wrapping_sub(32u16);
            if v1 as i32 - r22 as i32 & 0x8000i32 != 0 {
                v1 = r22;
            }
            (*E).ridley_var_0B = v1;
        }
    }
    let mut v3: u16 = 0;

    let use_target_angle: bool = if (r18 & 0x8000) != 0 {
        v3 = ((*E).ridley_var_0A).wrapping_add(r18);
        (v3 as i32 - r20 as i32 & 0x8000i32 != 0)
    } else {
        v3 = ((*E).ridley_var_0A).wrapping_add(r18);
        (v3 as i32 - r20 as i32 & 0x8000i32 == 0)
    };
    if use_target_angle {
        v3 = r20;
    }
    (*E).ridley_var_0A = v3;
    r18 = *(&raw mut v3 as *mut u8).offset(1isize) as u16;
    (*E).ridley_var_B = Math_MultBySin((*E).ridley_var_0B, r18);
    (*E).ridley_var_C = Math_MultByCos((*E).ridley_var_0B, r18);
}
pub unsafe fn Ridley_Func_112(game: &mut Game) {
    let mut Ridley = Get_Ridley(0i32, game);
    (*Ridley).ridley_var_1F = 0u16;
    let mut v1: u16 = ram_word!(game, cur_enemy_index);
    let mut v2 = Get_Ridley(ram_word!(game, cur_enemy_index) as i32, game);
    let mut with_carry: i32 = *(&raw mut (*v2).base.x_subpos as *mut u8).offset(1isize) as i32
        + *(&raw mut (*v2).ridley_var_B as *mut u8).offset(0) as i32;
    *(&raw mut (*v2).base.x_subpos as *mut u8).offset(1isize) = with_carry as u8;
    let mut ridley_var_23: u16 = ((*v2).base.x_pos as i32
        + ((*v2).ridley_var_B >> 8) as i8 as i32
        + (with_carry >> 8i32)) as u16;
    if ((ridley_var_23) as i16).wrapping_sub(((*Ridley).ridley_var_22) as i16) >= 0i16 {
        if ((ridley_var_23) as i16).wrapping_sub(((*Ridley).ridley_var_23) as i16) >= 0i16 {
            (*v2).ridley_var_B = 0u16;
            (*Ridley).ridley_var_1F = 2u16;
            ridley_var_23 = (*Ridley).ridley_var_23;
        }
        (*v2).base.x_pos = ridley_var_23;
    } else {
        Ridley_Func_113(v1, game);
        (*v2).base.x_pos = (*Ridley).ridley_var_22;
        (*v2).ridley_var_B = 0u16;
        (*Ridley).ridley_var_1F = 1u16;
    }
    with_carry = *(&raw mut (*v2).base.y_subpos as *mut u8).offset(1isize) as i32
        + *(&raw mut (*v2).ridley_var_C as *mut u8).offset(0) as i32;
    *(&raw mut (*v2).base.y_subpos as *mut u8).offset(1isize) = with_carry as u8;
    let mut ridley_var_21: u16 = ((*v2).base.y_pos as i32
        + ((*v2).ridley_var_C >> 8) as i8 as i32
        + (with_carry >> 8i32)) as u16;
    if ((ridley_var_21) as i16).wrapping_sub(((*Ridley).ridley_var_20) as i16) >= 0i16 {
        if ((ridley_var_21) as i16).wrapping_sub(((*Ridley).ridley_var_21) as i16) >= 0i16 {
            (*v2).ridley_var_C = 0u16;
            (*Ridley).ridley_var_1F = 8u16;
            ridley_var_21 = (*Ridley).ridley_var_21;
        }
        (*v2).base.y_pos = ridley_var_21;
    } else {
        (*v2).base.y_pos = (*Ridley).ridley_var_20;
        (*v2).ridley_var_C = 0u16;
        (*Ridley).ridley_var_1F = 4u16;
    };
}
pub unsafe fn Ridley_Func_113(k: u16, game: &mut Game) {
    if ram_word!(game, area_index) != 2u16 {
        let E = Get_Ridley(k as i32, game);
        let r18: u16 = abs16((*E).ridley_var_B);
        let mut v2: u16 = abs16((*E).ridley_var_C);
        if v2 < r18 {
            v2 = r18;
        }
        if v2 >= 0x280u16 {
            let mut v3: u16 = 0;
            if ram_word!(game, area_index) == 2u16 {
                v3 = 24u16;
            } else {
                v3 = 33u16;
            }
            set_ram_word!(game.earthquake_type = v3);
            set_ram_word!(game.earthquake_timer = 12u16);
        }
    }
}
pub unsafe fn Ridley_Func_114(game: &mut Game) {
    let E = Get_Ridley(0i32, game);
    let ridley_var_10: u16 = (*E).ridley_var_10;
    let mut v2: u16 = 0;
    let change_instruction: bool = if ridley_var_10 == 0 {
        if *((&raw mut (*E).base.enemy_ptr as *mut u8).offset(1) as *mut i16) < 0i16 {
            return;
        }
        v2 = addr_kRidley_Ilist_E6F0 as u16;
        true
    } else {
        (ridley_var_10 != 1u16
            && *((&raw mut (*E).base.enemy_ptr as *mut u8).offset(1) as *mut i16) < 0i16)
            && ({
                v2 = addr_kRidley_Ilist_E706 as u16;
                true
            })
    };
    if change_instruction {
        enemy_instruction! {[(*E).base]; instruction = v2, timer = 2u16}
        (*E).base.timer = 0u16;
    };
}
pub unsafe fn Ridley_Func_115(game: &mut Game) {
    Ridley_Func_116(game);
    let E = Get_Ridley(0i32, game);
    let mut v1: i16 = (((*E).ridley_var_09) as i16).wrapping_sub(((*E).ridley_var_08) as i16);
    (*E).ridley_var_09 = v1 as u16;
    if v1 < 0i16 {
        (*E).ridley_var_09 = 32u16;
        let mut v2: u16 = ((*E).ridley_var_07).wrapping_add(1u16);
        if v2 >= 0xau16 {
            v2 = 0u16;
        }
        (*E).ridley_var_07 = v2;
    }
}
pub unsafe fn Ridley_Func_116(game: &mut Game) {
    let E = Get_Ridley(0i32, game);
    let r18: u16 = abs16((*E).ridley_var_B);
    let v1: u16 = (r18).wrapping_add(abs16((*E).ridley_var_C));
    if v1 != 0 {
        let mut v2: u16 = (v1).wrapping_sub(r18);
        if v2 as i32 - r18 as i32 & 0x8000i32 != 0 {
            v2 = r18;
        }
        let mut v5: i32 = (4i32 * v2 as i32 & 0xf00i32) >> 8i32;
        if v5 >= 7i32 {
            v5 = 7i32;
        }
        let mut v6: u16 = g_word_A6D9ED[v5 as usize];
        if ((*E).ridley_var_C & 0x8000) == 0 {
            v6 = (v6 >> 1);
        }
        (*E).ridley_var_08 = v6;
    } else {
        (*E).ridley_var_08 = 0u16;
    };
}
pub unsafe fn Ridley_Func_117(game: &mut Game) {
    let E = Get_Ridley(0i32, game);
    (*E).ridley_var_05 = addr_word_A6DA71 as u16;
    (*E).ridley_var_06 = 1u16;
}
pub unsafe fn Ridley_Func_118(game: &mut Game) {
    let mut v7 = null_mut::<VramWriteEntry>();
    let E = Get_Ridley(0i32, game);
    let v1: u16 = ((*E).ridley_var_06).wrapping_sub(1u16);
    (*E).ridley_var_06 = v1;
    if v1 == 0 {
        let mut v3: u16 = 0;
        let mut v4 = null_mut::<u16>();
        let mut i: i32 = (*E).ridley_var_05 as i32;
        loop {
            v3 = i as u16;
            v4 = RomPtr_A6(i as u16, game) as *mut u16;
            i = *v4 as i32;
            if (*v4 & 0x8000) == 0 {
                break;
            }
            (*E).ridley_var_05 = i as u16;
        }
        (*E).ridley_var_06 = i as u16;
        let v6: u16 = ram_word!(game, vram_write_queue_tail);
        v7 = gVramWriteEntry(ram_word!(game, vram_write_queue_tail) as i32, game);
        *((&raw mut (*v7).src.addr as *mut u8).offset(1) as *mut VoidP) = -20480i32 as VoidP;
        *((&raw mut (*v7.offset(1)).src.addr as *mut u8).offset(1) as *mut VoidP) =
            -20480i32 as VoidP;
        (*v7).src.addr = *v4.offset(1);
        (*v7.offset(1)).src.addr = *v4.offset(2);
        (*v7).vram_dst = 29216u16;
        (*v7.offset(1)).vram_dst = 29472u16;
        (*v7).size = 64u16;
        (*v7.offset(1)).size = 64u16;
        set_ram_word!(game.vram_write_queue_tail = (v6).wrapping_add(14u16));
        (*gVramWriteEntry(v6 as i32 + 14i32, game)).size = 0u16;
        (*E).ridley_var_05 = (v3).wrapping_add(6u16);
    }
}
pub unsafe fn Ridley_Func_119(carry: u8, game: &mut Game) {
    let mut v1: u16 = addr_off_A6DAD0 as u16;
    if carry != 0 {
        v1 = addr_off_A6DAD4 as u16;
    }
    let v2: u16 = ram_word!(game, vram_write_queue_tail);
    let mut v3 = gVramWriteEntry(ram_word!(game, vram_write_queue_tail) as i32, game);
    *((&raw mut (*v3).src.addr as *mut u8).offset(1) as *mut VoidP) = -20480i32 as VoidP;
    *((&raw mut (*v3.offset(1)).src.addr as *mut u8).offset(1) as *mut VoidP) = -20480i32 as VoidP;
    let v4 = RomPtr_A6(v1, game);
    (*v3).src.addr = *(v4 as *mut u16);
    (*v3.offset(1)).src.addr = *(v4.offset(2) as *mut u16);
    (*v3).vram_dst = 31424u16;
    (*v3.offset(1)).vram_dst = 31680u16;
    (*v3).size = 128u16;
    (*v3.offset(1)).size = 128u16;
    set_ram_word!(game.vram_write_queue_tail = (v2).wrapping_add(14u16));
    (*gVramWriteEntry(v2 as i32 + 14i32, game)).size = 0u16;
}
pub unsafe fn Ridley_Func_120(game: &mut Game) {
    let E = Get_Ridley(0i32, game);
    let mut ridley_var_10: i16 = (*E).ridley_var_10 as i16;
    if ridley_var_10 != 0 {
        if ridley_var_10 == 1i16 {
            return;
        }
        ridley_var_10 = 10i16;
    }
    let v2: u16 = crate::rtl::rom_table_word!(
        game,
        0xa6db02,
        ((*E).ridley_var_07 as i32 + ridley_var_10 as i32) as isize
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
    if ((*gEnemyData(0i32, game)).properties as i32 & kEnemyProps_Invisible as i32) == 0i32 {
        sub_A6DBC2(
            crate::rtl::rom_table_word!(
                game,
                0xa6dcba,
                ((*((ram_ptr!(game, u16; 0x2000)).offset(71) as *mut u8).offset(0) as i32
                    + *((ram_ptr!(game, u16; 0x2000)).offset(81) as *mut u8).offset(0) as i32
                    + 8i32
                    & 0xf0i32) as u8
                    >> 4) as isize,
            ),
            ram_word!(game, tilemap_stuff[82]),
            ram_word!(game, tilemap_stuff[83]),
            game,
        );
        sub_A6DBC2(
            0xdc9eu16,
            ram_word!(game, tilemap_stuff[72]),
            ram_word!(game, tilemap_stuff[73]),
            game,
        );
        sub_A6DBC2(
            0xdc9eu16,
            ram_word!(game, tilemap_stuff[62]),
            ram_word!(game, tilemap_stuff[63]),
            game,
        );
        sub_A6DBC2(
            0xdc97u16,
            ram_word!(game, tilemap_stuff[52]),
            ram_word!(game, tilemap_stuff[53]),
            game,
        );
        sub_A6DBC2(
            0xdc97u16,
            ram_word!(game, tilemap_stuff[42]),
            ram_word!(game, tilemap_stuff[43]),
            game,
        );
        sub_A6DBC2(
            0xdc90u16,
            ram_word!(game, tilemap_stuff[32]),
            ram_word!(game, tilemap_stuff[33]),
            game,
        );
        sub_A6DBC2(
            0xdc90u16,
            ram_word!(game, tilemap_stuff[22]),
            ram_word!(game, tilemap_stuff[23]),
            game,
        );
    }
}
pub unsafe fn sub_A6DBC2(j: u16, x: u16, y: u16, game: &mut Game) {
    sub_A6DC13(j, x, y, (*Get_Ridley(0i32, game)).ridley_var_0C, game);
}
crate::game::address_dispatch! {
    pub unsafe fn CallBabyMetroidInstr(ea: u32, k: u16, game: &mut Game) -> u16 => ea;
        call (k, game); default Unreachable(game) as u16;
    {
        10928073 => BabyMetroid_Instr_2; 10928097 => BabyMetroid_Instr_3;
        10928114 => BabyMetroid_Instr_1; 10928120 => BabyMetroid_Goto;
    }
}
unsafe fn BabyMetroid_DBCB_DoubleRetEx(a: u16, game: &mut Game) -> i32 {
    let mut execute_instruction: bool;
    let st: *mut BabyMetroidExecState = ram_ptr!(game, BabyMetroidExecState; a as isize);
    if ((*st).ip & 0x8000) == 0 {
        return -1i32;
    }
    let mut v2: u16 = (*st).ip;
    let mut v3: *const u16 = RomPtr_A6(v2, game) as *mut u16;
    execute_instruction = (*v3.offset(0) & 0x8000 != 0)
        || ({
            if (*st).timer != *v3.offset(0) {
                (*st).timer = (*st).timer.wrapping_add(1);
                return *v3.offset(1) as i32;
            }
            v2 = (v2).wrapping_add(4u16);
            false
        });
    loop {
        if execute_instruction {
            v2 = CallBabyMetroidInstr(
                ((*v3.offset(0)) as u32) | (0xa60000u32),
                (v2).wrapping_add(2u16),
                game,
            );
            execute_instruction = false;
        } else {
            v3 = RomPtr_A6(v2, game) as *mut u16;
            if *v3.offset(0) & 0x8000 == 0 {
                break;
            } else {
                execute_instruction = true;
            }
        }
    }
    (*st).timer = 1u16;
    (*st).ip = v2;
    *v3.offset(1) as i32
}
pub unsafe fn sub_A6DC13(j: u16, r18: u16, r20: u16, r22: u16, game: &mut Game) {
    let mut p = RomPtr_A6(j, game);
    let mut n: i32 = *(p as *mut u16) as i32;
    p = p.offset(2);
    let mut idx: i32 = ram_word!(game, oam_next_ptr) as i32;
    loop {
        let v9: i16 =
            (r20 as i32 + *p.offset(2) as i8 as i32 - ram_word!(game, layer1_y_pos) as i32) as i16;
        if v9 >= 0i16 && v9 as i32 - 224i32 & 0x8000i32 != 0 {
            let x: u16 = (r18 as i32 + *(p as *mut u16) as i32
                - ram_word!(game, layer1_x_pos) as i32) as u16;
            let v11 = gOamEnt(idx, game);
            (*v11).xcoord = x as u8;
            (*v11).ycoord = v9 as u8;
            *(&raw mut (*v11).charnum as *mut u16) = (r22) | (*(p.offset(3) as *mut u16));
            update_ram_word!(game.oam_ext[(idx >> 5i32)], |fresh4| (fresh4 as i32
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
pub unsafe fn Ridley_Func_121(game: &mut Game) -> u8 {
    let E = Get_Ridley(0i32, game);
    let mut y_pos: i16 = (*E).base.y_pos as i16;
    let result: u8 = 1u8;
    if y_pos >= 0i16 {
        let mut v2: i16 = (y_pos as i32 + 32i32 - ram_word!(game, layer1_y_pos) as i32) as i16;
        if v2 >= 0i16 && v2 as i32 - 288i32 & 0x8000i32 != 0 {
            let mut x_pos: i16 = (*E).base.x_pos as i16;
            if x_pos >= 0i16 {
                let mut v4: i16 =
                    (x_pos as i32 + 32i32 - ram_word!(game, layer1_x_pos) as i32) as i16;
                if v4 >= 0i16 && v4 as i32 - 320i32 & 0x8000i32 != 0 {
                    return 0u8;
                }
            }
        }
    }
    result
}
pub unsafe fn Ridley_Func_122(rect: Rect16U, game: &mut Game) -> u16 {
    let mut v2: i16 = 0;
    if ram_word!(game, projectile_counter) == 0 {
        return 0xffffu16;
    }
    let mut result: u16 = 0u16;
    loop {
        let v1: i32 = result as i32 >> 1i32;
        v2 = ram_word!(game, projectile_type[v1]) as i16;
        if v2 < 0i16
            && (*(&raw mut v2 as *mut u8).offset(1isize) as i32 & 0xfi32) - 3i32 & 0x8000i32 != 0
        {
            let v3: u16 = abs16((ram_word!(game, projectile_x_pos[v1])).wrapping_sub(rect.x));
            let mut v4: bool = v3 < ram_word!(game, projectile_x_radius[v1]);
            let v5: u16 = (v3).wrapping_sub(ram_word!(game, projectile_x_radius[v1]));
            if v4 || v5 < rect.w {
                let v6: u16 = abs16((ram_word!(game, projectile_y_pos[v1])).wrapping_sub(rect.y));
                v4 = v6 < ram_word!(game, projectile_y_radius[v1]);
                let v7: u16 = (v6).wrapping_sub(ram_word!(game, projectile_y_radius[v1]));
                if v4 || v7 < rect.h {
                    break;
                }
            }
        }
        result = (result).wrapping_add(2u16);
        if ((result) as i16).wrapping_sub(10i16) >= 0i16 {
            return 0xffffu16;
        }
    }
    let v8: i32 = result as i32 >> 1i32;
    set_ram_word!(game.projectile_x_pos[v8] = rect.x);
    set_ram_word!(game.projectile_y_pos[v8] = rect.y);
    update_ram_word!(game.projectile_dir[v8], |fresh2| (fresh2) | (0x10u16));
    result
}
pub unsafe fn Ridley_Func_123(j: u16, game: &mut Game) {
    let v1: i32 = j as i32 >> 1i32;
    let mut v2: u16 = 0;
    if (ram_word!(game, projectile_dir[v1]) & 0xf) == 7 {
        v2 = 1u16;
    } else if (ram_word!(game, projectile_dir[v1]) & 0xf) == 2 {
        v2 = 8u16;
    } else {
        v2 = 5u16;
    }
    set_ram_word!(game.projectile_dir[v1] = v2);
}
pub unsafe fn Ridley_Func_124(rect: Rect16U, game: &mut Game) -> u8 {
    let v0: u16 = abs16((ram_word!(game, samus_x_pos)).wrapping_sub(rect.x));
    let mut v1: bool = v0 < ram_word!(game, samus_x_radius);
    let v2: u16 = (v0).wrapping_sub(ram_word!(game, samus_x_radius));
    if v1 || v2 < rect.w {
        let v3: u16 = abs16((ram_word!(game, samus_y_pos)).wrapping_sub(rect.y));
        v1 = v3 < ram_word!(game, samus_y_radius);
        let v4: u16 = (v3).wrapping_sub(ram_word!(game, samus_y_radius));
        if v1 || v4 < rect.h {
            return 1u8;
        }
    }
    0u8
}
crate::forward_calls! { pub unsafe fn (game: &mut Game);
    sub_A6DF59 => NormalEnemyTouchAiSkipDeathAnim_CurEnemy(game);
    sub_A6DF60 => Ridley_Func_125(game);
}
pub unsafe fn Ridley_Func_125(game: &mut Game) {
    Ridley_Func_98(game);
    set_ram_word!(game.samus_invincibility_timer = 96u16);
    set_ram_word!(game.samus_knockback_timer = 5u16);
    set_ram_word!(
        game.knockback_x_dir = (((ram_word!(game, samus_x_pos)) as i16).wrapping_sub(
            ((*Get_Ridley(ram_word!(game, cur_enemy_index) as i32, game))
                .base
                .x_pos) as i16,
        ) >= 0i16) as u16
    );
}
pub unsafe fn Ridley_Shot(game: &mut Game) {
    if ram_word!(game, area_index) == 2u16 {
        NormalEnemyShotAiSkipDeathAnim_CurEnemy(game);
    } else {
        let mut v0: i16 = 13i16;
        let E = Get_Ridley(0i32, game);
        let flash_timer: u16 = (*E).base.flash_timer;
        if flash_timer != 0 && flash_timer & 1 != 0 {
            v0 = 14i16;
        }
        (*E).base.flash_timer = v0 as u16;
        (*E).ridley_var_0D = (*E).ridley_var_0D.wrapping_add(1);
    };
}
pub unsafe fn Ridley_Powerbomb(game: &mut Game) {
    NormalEnemyPowerBombAiSkipDeathAnim_CurEnemy(game);
}
pub unsafe fn Ridley_Func_126(game: &mut Game) {
    let E = Get_Ridley(0i32, game);
    if (*E).base.health == 0 && ((*E).ridley_var_01 & 0x8000) == 0 {
        (*E).ridley_var_01 = -1i32 as u16;
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
        (*E).ridley_var_A = ((0xa6c538i32) as u16);
    }
}
pub unsafe fn Ridley_Func_127(game: &mut Game) {
    let rect: Rect16U = Rect16U {
        x: ram_word!(game, tilemap_stuff[82]),
        y: ram_word!(game, tilemap_stuff[83]),
        w: 14u16,
        h: 14u16,
    };
    if Ridley_Func_124(rect, game) != 0 {
        let E = Get_Ridley(0i32, game);
        Samus_DealDamage(SuitDamageDivision((*E).ridley_var_1C, game), game);
        set_ram_word!(game.samus_invincibility_timer = 96u16);
        set_ram_word!(game.samus_knockback_timer = 5u16);
        set_ram_word!(
            game.knockback_x_dir = (((ram_word!(game, samus_x_pos)) as i16)
                .wrapping_sub((ram_word!(game, tilemap_stuff[82])) as i16,)
                >= 0i16) as u16
        );
    }
}
pub unsafe fn Ridley_Func_128(game: &mut Game) {
    let E = Get_Ridley(ram_word!(game, cur_enemy_index) as i32, game);
    let mut r18: u16 = -(CalculateAngleFromXY(
        (ram_word!(game, projectile_x_pos[0])).wrapping_sub((*E).base.x_pos),
        (ram_word!(game, projectile_y_pos[0])).wrapping_sub((*E).base.y_pos),
        game,
    ) as i32) as u8 as u16;
    let mut v1: u16 = (4u16).wrapping_mul(ram_word!(
        game,
        projectile_damage[ram_word!(game, collision_detection_index)]
    ));
    if v1 >= 0x300u16 {
        v1 = 768u16;
    }
    let a: u16 = v1;
    r18 = Math_MultBySin(v1, r18);
    if (((*E).ridley_var_B ^ r18) & 0x8000) != 0 {
        (*E).ridley_var_B = ((*E).ridley_var_B).wrapping_add(r18);
    }
    r18 = Math_MultByCos(a, r18);
    if (((*E).ridley_var_C ^ r18) & 0x8000) != 0 {
        (*E).ridley_var_C = ((*E).ridley_var_C).wrapping_add(r18);
    }
}
pub unsafe fn Ridley_Func_129(game: &mut Game) {
    if ((*Get_Ridley(0i32, game)).base.properties as i32 & kEnemyProps_Tangible as i32) == 0i32 {
        let mut v0: u16 = Ridley_Func_122(
            Rect16U {
                x: ram_word!(game, tilemap_stuff[82]),
                y: ram_word!(game, tilemap_stuff[83]),
                w: 14u16,
                h: 14u16,
            },
            game,
        );
        if v0 & 0x8000 != 0 {
            v0 = Ridley_Func_122(
                Rect16U {
                    x: ram_word!(game, tilemap_stuff[72]),
                    y: ram_word!(game, tilemap_stuff[73]),
                    w: 10u16,
                    h: 10u16,
                },
                game,
            );
            if v0 & 0x8000 != 0 {
                return;
            }
        }
        let v1: i32 = v0 as i32 >> 1i32;
        game.eproj_spawn_pt = Point16U {
            x: ram_word!(game, projectile_x_pos[v1]),
            y: ram_word!(game, projectile_y_pos[v1]),
        };
        let mut v2: u16 = 12u16;
        if (*((ram_ptr!(game, u16; 0xc18)).offset(v1 as isize) as *mut u8).offset(1isize) & 0xf)
            == 1
        {
            QueueSfx1_Max6(0x3du16, game);
            v2 = 6u16;
        }
        SpawnEprojWithRoomGfx(0xe509u16, v2, game);
    }
}
pub unsafe fn Ridley_Instr_5(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*Get_Ridley(0i32, game)).ridley_var_0F = 89u16;
    QueueSfx2_Max6(0x59u16, game);
    jp
}
pub unsafe fn Ridley_Instr_6(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*Get_Ridley(0i32, game)).ridley_var_0F = 0u16;
    jp
}
pub unsafe fn Ridley_Instr_10(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    if ram_word!(game, area_index) == 2u16
        || ram_word!(game, samus_health) as i32 - 30i32 & 0x8000i32 == 0
    {
        return jp.offset(1);
    }
    (*Get_Ridley(0i32, game)).ridley_var_00 = 8u16;
    *jp.offset(0) as usize as *const u16
}
pub unsafe fn Ridley_Instr_4(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    if (*Get_Ridley(0i32, game)).ridley_var_1B != 0 {
        *jp.offset(0) as usize as *const u16
    } else {
        *jp.offset(1) as usize as *const u16
    }
}
pub unsafe fn Ridley_Instr_3(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    if (*Get_Ridley(0i32, game)).ridley_var_1B != 0 {
        jp.offset(1)
    } else {
        *jp.offset(0) as usize as *const u16
    }
}
pub unsafe fn Ridley_Instr_2(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    set_ram_word!(game.samus_prev_x_pos = ram_word!(game, samus_x_pos));
    set_ram_word!(game.samus_prev_y_pos = ram_word!(game, samus_y_pos));
    (*Get_Ridley(0i32, game)).ridley_var_1D = *jp.offset(0);
    jp.offset(1)
}
pub unsafe fn Ridley_Instr_1(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    if (*Get_Ridley(0i32, game)).ridley_var_10 != 0 {
        *jp.offset(0) as usize as *const u16
    } else {
        jp.offset(1)
    }
}
pub unsafe fn Ridley_Instr_14(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E = Get_Ridley(0i32, game);
    (*E).base.x_pos = ((*E).base.x_pos).wrapping_add(*jp.offset(0));
    (*E).base.y_pos = ((*E).base.y_pos).wrapping_add(*jp.offset(1));
    jp.offset(2)
}
pub unsafe fn Ridley_Instr_9(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*Get_Ridley(0i32, game)).ridley_var_10 = 0u16;
    Ridley_Func_96(game);
    jp
}
pub unsafe fn Ridley_Instr_7(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*Get_Ridley(0i32, game)).ridley_var_10 = 1u16;
    jp
}
pub unsafe fn Ridley_Instr_8(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*Get_Ridley(0i32, game)).ridley_var_10 = 2u16;
    Ridley_Func_96(game);
    jp
}
pub unsafe fn Ridley_Func_131(k: u16, game: &mut Game) {
    Ridley_Func_132(k, 0u16, game);
    Ridley_Func_132(k, 1u16, game);
    Ridley_Func_132(k, 2u16, game);
    Ridley_Func_132(k, 3u16, game);
}
pub unsafe fn Ridley_Func_132(k: u16, a: u16, game: &mut Game) {
    (*Get_Ridley(0i32, game)).ridley_parameter_1 = a;
    SpawnEprojWithGfx(a, k, addr_stru_869634 as u16, game);
}
pub unsafe fn Ridley_Instr_11(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let current_block: u64;
    let mut r18: u16 = 0;
    let E = Get_Ridley(0i32, game);
    if (*E).ridley_var_10 != 0 {
        let mut v4: u16 = -(CalculateAngleFromXY(
            (ram_word!(game, samus_x_pos) as i32 - ((*E).base.x_pos as i32 + 25i32)) as u16,
            (ram_word!(game, samus_y_pos) as i32 - ((*E).base.y_pos as i32 - 43i32)) as u16,
            game,
        ) as i32
            + 0x80i32) as u8 as u16;
        if v4 < 0x50u16 {
            if v4 >= 0x15u16 {
                current_block = 14379314559172793947;
            } else {
                current_block = 11006700562992250127;
            }
        } else if v4 < 0xc0u16 {
            v4 = 80u16;
            current_block = 14379314559172793947;
        } else {
            current_block = 11006700562992250127;
        }
        if current_block == 11006700562992250127 {
            v4 = 21u16;
        }
        r18 = v4;
    } else {
        let mut v3: u16 = 0;
        v3 = -(CalculateAngleFromXY(
            (ram_word!(game, samus_x_pos) as i32 - ((*E).base.x_pos as i32 - 25i32)) as u16,
            (ram_word!(game, samus_y_pos) as i32 - ((*E).base.y_pos as i32 - 43i32)) as u16,
            game,
        ) as i32
            + 0x80i32) as u8 as u16;
        if v3 >= 0xb0u16 {
            if v3 < 0xebu16 {
                current_block = 16663838568255336676;
            } else {
                current_block = 11874048045265102773;
            }
        } else if v3 < 0x40u16 {
            current_block = 11874048045265102773;
        } else {
            v3 = 176u16;
            current_block = 16663838568255336676;
        }
        if current_block == 11874048045265102773 {
            v3 = 235u16;
        }
        r18 = v3;
    }
    (*E).ridley_var_19 = Math_MultBySin(0x500u16, r18);
    (*E).ridley_var_1A = Math_MultByCos(0x500u16, r18);
    jp
}
crate::forward_calls! { pub unsafe fn (k: u16, jp: *const u16, game: &mut Game) -> *const u16;
    Ridley_Instr_12 => Ridley_E90C(k, jp, 0u16, game);
    Ridley_Instr_13 => Ridley_E90C(k, jp, 0xeu16, game);
}
pub unsafe fn Ridley_E90C(k: u16, jp: *const u16, a: u16, game: &mut Game) -> *const u16 {
    set_ram_word!(game.cinematic_var9 = a);
    let E = Get_Ridley(0i32, game);
    SpawnEprojWithRoomGfx(0x9642u16, (*E).ridley_var_10, game);
    jp
}
pub unsafe fn Ridley_Instr_15(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E = Get_Ridley(0i32, game);
    (*E).ridley_var_A = ((0xa6a6afi32) as u16);
    (*E).ridley_var_C = -352i32 as u16;
    jp
}
pub unsafe fn Ridley_Instr_16(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E = Get_Ridley(0i32, game);
    (*E).ridley_var_A = ((0xa6b2f3i32) as u16);
    (*E).ridley_var_C = -352i32 as u16;
    jp
}
pub unsafe fn CeresSteam_Init(game: &mut Game) {
    let E = Get_CeresSteam(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.vram_tiles_index = 0u16;
    (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_DisableSamusColl) as u16);
    (*E).base.extra_properties = ((*E).base.extra_properties) | (4u16);
    (*E).base.instruction_timer = 1u16;
    (*E).base.timer = 0u16;
    (*E).base.palette_index = 2560u16;
    (*E).csm_var_D = ((NextRandom(game) as i32 & 0x1fi32) + 1i32) as u16;
    let v1: i32 = (*E).csm_parameter_1 as i32;
    (*E).base.current_instruction = crate::rtl::rom_table_word!(game, 0xa6eff5, v1 as isize);
    (*E).csm_var_A = crate::rtl::rom_table_word!(game, 0xa6f001, v1 as isize);
}
crate::game::address_dispatch! {
    pub unsafe fn CallCeresSteamFunc(ea: u32, k: u16, game: &mut Game) => ea;
        call (k, game); default { Unreachable(game); };
    {
        10940404 => [()]; 10940441 => CeresSteam_Func_1;
    }
}
pub unsafe fn CeresSteam_Main(game: &mut Game) {
    let E = Get_CeresSteam(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.health = 0x7fffu16;
    CallCeresSteamFunc(
        (((*E).csm_var_A) as u32) | (0xa60000u32),
        ram_word!(game, cur_enemy_index),
        game,
    );
}
pub unsafe fn CeresSteam_Func_1(k: u16, game: &mut Game) {
    let E = Get_CeresSteam(k as i32, game);
    let pt: Point16U = CalcCeresSteamPos_Mode7(
        Point16U {
            x: (*E).base.x_pos,
            y: (*E).base.y_pos,
        },
        game,
    );
    let v3 = gEnemySpawnData(ram_word!(game, cur_enemy_index) as i32, game);
    (*v3).xpos2 = (pt.x).wrapping_sub((*E).base.x_pos);
    (*v3).ypos2 = (pt.y).wrapping_sub((*E).base.y_pos);
}
pub unsafe fn CeresSteam_Touch(game: &mut Game) {
    (*Get_CeresSteam(ram_word!(game, cur_enemy_index) as i32, game))
        .base
        .health = 0x7fffu16;
    NormalEnemyTouchAi(game);
}
pub unsafe fn CeresSteam_Instr_1(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E = Get_CeresSteam(k as i32, game);
    (*E).base.properties = ((*E).base.properties as i32
        | (kEnemyProps_Tangible as i32 | kEnemyProps_Invisible as i32))
        as u16;
    jp
}
pub unsafe fn CeresSteam_Instr_2(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E = Get_CeresSteam(k as i32, game);
    crate::update_place! { previous fresh0 = (*E).csm_var_D => (*E).csm_var_D.wrapping_sub(1); }
    if fresh0 != 1u16 {
        return *jp.offset(0) as usize as *const u16;
    }
    CeresSteam_Instr_3(k, RomPtr_A6(*jp.offset(1), game) as *mut u16, game)
}
pub unsafe fn CeresSteam_Instr_3(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E = Get_CeresSteam(k as i32, game);
    (*E).base.properties = ((*E).base.properties) & (0xfaffu16);
    jp
}
pub unsafe fn CeresDoor_Instr_6(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E = Get_CeresDoor(k as i32, game);
    if abs16(((*E).base.x_pos).wrapping_sub(ram_word!(game, samus_x_pos))) >= 0x30u16
        || abs16(((*E).base.y_pos).wrapping_sub(ram_word!(game, samus_y_pos))) >= 0x30u16
    {
        *jp as usize as *const u16
    } else {
        jp.offset(1)
    }
}
pub unsafe fn CeresDoor_Instr_4(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    if *(ram_ptr!(game, u16; 0xd828, ram_word!(game, area_index) as isize)) & 1 != 0 {
        jp.offset(1)
    } else {
        *jp as usize as *const u16
    }
}
pub unsafe fn CeresDoor_Instr_8(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    if ram_word!(game, ceres_status) != 0 {
        jp.offset(1)
    } else {
        *jp as usize as *const u16
    }
}
pub unsafe fn CeresDoor_Func_6b(game: &mut Game) {
    if ram_word!(game, ceres_status) != 0 {
        set_ram_word!(game.ceres_status = 0x8000u16);
    }
}
pub unsafe fn CeresSteam_Instr_4(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E = Get_CeresSteam(k as i32, game);
    (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
    jp
}
pub unsafe fn CeresDoor_Instr_1(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E = Get_CeresDoor(k as i32, game);
    (*E).base.properties = ((*E).base.properties as i32 & !(kEnemyProps_Tangible as i32)) as u16;
    jp
}
pub unsafe fn CeresDoor_Instr_3(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*Get_CeresDoor(k as i32, game)).cdr_var_B = 1u16;
    jp
}
pub unsafe fn CeresSteam_Instr_5(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E = Get_CeresSteam(k as i32, game);
    (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Invisible) as u16);
    jp
}
pub unsafe fn CeresDoor_Instr_5(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*Get_CeresDoor(k as i32, game)).cdr_var_B = 0u16;
    CeresDoor_Instr_2(k, jp, game)
}
pub unsafe fn CeresDoor_Instr_2(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E = Get_CeresDoor(k as i32, game);
    (*E).base.properties = ((*E).base.properties as i32 & !(kEnemyProps_Invisible as i32)) as u16;
    jp
}
pub unsafe fn CeresDoor_Instr_7(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    QueueSfx3_Max6(0x2cu16, game);
    jp
}
pub unsafe fn CeresDoor_Init(game: &mut Game) {
    let E = Get_CeresDoor(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.spritemap_pointer = addr_kCeresDoor_Sprmap_FAC7 as u16;
    (*E).base.instruction_timer = 1u16;
    set_fields! {[(*E).base], 0u16; timer, vram_tiles_index}
    (*E).base.palette_index = 1024u16;
    let v2: i32 = (*E).cdr_parameter_1 as i32;
    (*E).cdr_var_A = crate::rtl::rom_table_word!(game, 0xa6f72b, v2 as isize);
    (*E).base.current_instruction = crate::rtl::rom_table_word!(game, 0xa6f52c, v2 as isize);
    (*E).cdr_var_B = 0u16;
    CeresDoor_Func_1(ram_word!(game, cur_enemy_index), game);
    let mut v3: u16 = 0;
    if ram_word!(game, ceres_status) != 0 {
        v3 = (addr_word_A6F50C as i32 + 2i32) as u16;
    } else {
        if (*E).cdr_parameter_1 == 3u16 {
            WriteColorsToTargetPalette(
                0xa6u8,
                0x142u16,
                (addr_kCeresDoor_Palette as i32 + 2i32) as u16,
                0xfu16,
                game,
            );
            return;
        }
        v3 = (addr_kCeresDoor_Palette as i32 + 2i32) as u16;
    }
    (*Get_CeresDoor(ram_word!(game, cur_enemy_index) as i32, game))
        .base
        .palette_index = 3584u16;
    WriteColorsToTargetPalette(0xa6u8, 0x1e2u16, v3, 0xfu16, game);
}
pub unsafe fn CeresDoor_Func_1(k: u16, game: &mut Game) {
    let mut v2 = null_mut::<VramWriteEntry>();
    if (*Get_CeresDoor(k as i32, game)).cdr_parameter_1 == 2u16 {
        let v1: u16 = ram_word!(game, vram_write_queue_tail);
        v2 = gVramWriteEntry(ram_word!(game, vram_write_queue_tail) as i32, game);
        (*v2).size = 1024u16;
        *((&raw mut (*v2).src.addr as *mut u8).offset(1) as *mut VoidP) = -20480i32 as VoidP;
        (*v2).src.addr = -15360i32 as VoidP;
        (*v2).vram_dst = 28672u16;
        set_ram_word!(game.vram_write_queue_tail = (v1).wrapping_add(7u16));
    }
}
pub unsafe fn CallCeresDoor(ea: u32, game: &mut Game) {
    let k: u16 = ram_word!(game, cur_enemy_index);
    match ea {
        10942315 => CeresDoor_Func_2(game),
        10942320 => CeresDoor_Func_3(game),
        10942373 => CeresDoor_Func_4(game),
        10942397 => CeresDoor_Func_5(k, game),
        10942428 => CeresDoor_Func_6(k, game),
        10942544 => CeresDoor_Func_7(game),
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn CeresDoor_Main(game: &mut Game) {
    let E = Get_CeresDoor(ram_word!(game, cur_enemy_index) as i32, game);
    CallCeresDoor((((*E).cdr_var_A) as u32) | (0xa60000u32), game);
}
crate::forward_calls! { pub unsafe fn (game: &mut Game);
    CeresDoor_Func_2 => CeresDoor_F773(0x14u16, game);
    CeresDoor_Func_3 => CeresDoor_F773(0x1du16, game);
}
pub unsafe fn CeresDoor_F773(j: u16, game: &mut Game) {
    if ram_word!(game, ceres_status) >= 2u16 && ram_word!(game, earthquake_timer) == 0 {
        if (ram_word!(game, random_number) & 0xfff) < 0x80 {
            set_ram_word!(game.earthquake_timer = 4u16);
            set_ram_word!(game.earthquake_type = (j).wrapping_add(6u16));
        } else {
            set_ram_word!(game.earthquake_timer = 2u16);
            set_ram_word!(game.earthquake_type = j);
        }
    }
}
pub unsafe fn CeresDoor_Func_4(game: &mut Game) {
    CeresSteam_Instr_5(ram_word!(game, cur_enemy_index), null::<u16>(), game);
    if ram_word!(game, ceres_status) & 1 != 0 {
        (*Get_CeresDoor(ram_word!(game, cur_enemy_index) as i32, game))
            .base
            .palette_index = 3584u16;
        CeresDoor_Instr_2(ram_word!(game, cur_enemy_index), null::<u16>(), game);
    }
}
pub unsafe fn CeresDoor_Func_5(k: u16, game: &mut Game) {
    CeresDoor_Func_7(game);
    if ram_word!(game, ceres_status) >= 2u16 {
        let E = Get_CeresDoor(k as i32, game);
        (*E).cdr_var_A = ((0xa6f7dci32) as u16);
        (*E).cdr_var_D = 48u16;
        let E0 = Get_CeresDoor(0i32, game);
        set_fields! {[(*E0)], 0u16; cdr_var_E, cdr_var_F}
    }
}
pub unsafe fn CeresDoor_Func_6(k: u16, game: &mut Game) {
    let E = Get_CeresDoor(k as i32, game);
    (*E).cdr_var_D = (*E).cdr_var_D.wrapping_sub(1);
    let mut v3: bool = ((*E).cdr_var_D & 0x8000) != 0;
    if v3 {
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Invisible) as u16);
        (*E).cdr_var_A = ((0xa6f850i32) as u16);
        CeresDoor_Func_6b(game);
    } else {
        let E0 = Get_CeresDoor(0i32, game);
        (*E0).cdr_var_E = (*E0).cdr_var_E.wrapping_sub(1);
        v3 = ((*E0).cdr_var_E & 0x8000) != 0;
        if v3 {
            (*E0).cdr_var_E = 4u16;
            (*E0).cdr_var_F = (*E0).cdr_var_F.wrapping_sub(1);
            v3 = ((*E0).cdr_var_F & 0x8000) != 0;
            if v3 {
                (*E0).cdr_var_F = 3u16;
            }
            let v5: i32 = ((4u16).wrapping_mul((*E0).cdr_var_F)) as i32 >> 1i32;
            game.eproj_spawn_pt = Point16U {
                x: (((*E).base.x_pos).wrapping_add(crate::rtl::rom_table_word!(
                    game,
                    0xa6f840,
                    v5 as isize
                ))),
                y: (((*E).base.y_pos).wrapping_add(crate::rtl::rom_table_word!(
                    game,
                    0xa6f840,
                    (v5 + 1i32) as isize
                ))),
            };
            let mut v6: u16 = 3u16;
            if NextRandom(game) < 0x4000u16 {
                v6 = 12u16;
            }
            SpawnEprojWithRoomGfx(0xe509u16, v6, game);
            QueueSfx2_Max6(0x25u16, game);
        }
    };
}
pub unsafe fn CeresDoor_Func_7(game: &mut Game) {
    CeresDoor_Func_8(game);
    if ram_word!(game, palette_change_num) == 0 {
        WriteColorsToPalette(
            0x52u16,
            0xa6u8,
            (2i32 * (ram_word!(game, nmi_frame_counter_word) as i32 & 0x38i32) - 0x78fi32) as u16,
            6u16,
            game,
        );
    }
}
pub unsafe fn CeresDoor_Func_8(game: &mut Game) {
    QueueMode7Transfers(
        0xa6u8,
        crate::rtl::rom_table_word!(
            game,
            0xa6f900,
            ((ram_word!(game, nmi_frame_counter_word) as i32 & 2i32) >> 1i32) as isize
        ),
        game,
    );
}
pub unsafe fn Zebetites_Init(game: &mut Game) {
    let E = Get_Zebetites(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.properties = ((*E).base.properties) | (0xa000u16);
    (*E).base.instruction_timer = 1u16;
    (*E).base.timer = 0u16;
    (*E).base.palette_index = 1024u16;
    (*E).base.vram_tiles_index = 128u16;
    (*E).zebet_var_C = 0u16;
    let mut v1: u16 = ((0xa6fc41i32) as u16);
    if (*E).zebet_parameter_1 != 0 {
        v1 = ((0xa6fc5bi32) as u16);
    }
    (*E).zebet_var_A = v1;
    let mut r18: u16 = 0u16;
    r18 = ((r18 as i32) << 1i32 | CheckEventHappened(5u16, game) as i32) as u16;
    r18 = ((r18 as i32) << 1i32 | CheckEventHappened(4u16, game) as i32) as u16;
    r18 = ((r18 as i32) << 1i32 | CheckEventHappened(3u16, game) as i32) as u16;
    let v2: u16 = r18;
    (*E).zebet_var_D = r18;
    if v2 as i32 - 4i32 & 0x8000i32 != 0 {
        let v3: i32 = v2 as i32;
        (*E).zebet_var_F = crate::rtl::rom_table_word!(game, 0xa6fc03, v3 as isize);
        (*E).base.y_height = crate::rtl::rom_table_word!(game, 0xa6fc0b, v3 as isize);
        (*E).base.current_instruction = crate::rtl::rom_table_word!(game, 0xa6fc13, v3 as isize);
        (*E).base.x_pos = crate::rtl::rom_table_word!(game, 0xa6fc1b, v3 as isize);
        let mut v4: u16 = 0;
        if (*E).zebet_parameter_1 != 0 {
            v4 = crate::rtl::rom_table_word!(game, 0xa6fc2b, v3 as isize);
        } else {
            v4 = crate::rtl::rom_table_word!(game, 0xa6fc23, v3 as isize);
        }
        (*E).base.y_pos = v4;
    } else {
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Deleted) as u16);
    };
}
crate::game::address_dispatch! {
    pub unsafe fn CallZebetitesFunc(ea: u32, k: u16, game: &mut Game) => ea;
        call (k, game); default { Unreachable(game); };
    {
        10943553 => Zebetites_Func_1; 10943579 => Zebetites_Func_2;
        10943591 => [Zebetites_Func_3(game)];
    }
}
pub unsafe fn Zebetites_Main(game: &mut Game) {
    if ram_word!(game, earthquake_timer) == 0 {
        (*Get_Zebetites(ram_word!(game, cur_enemy_index) as i32, game))
            .base
            .shake_timer = 0u16;
    }
    let E = Get_Zebetites(ram_word!(game, cur_enemy_index) as i32, game);
    CallZebetitesFunc(
        (((*E).zebet_var_A) as u32) | (0xa60000u32),
        ram_word!(game, cur_enemy_index),
        game,
    );
}
pub unsafe fn Zebetites_Func_1(k: u16, game: &mut Game) {
    let E = Get_Zebetites(k as i32, game);
    if ((*E).zebet_var_F & 0x8000) != 0 {
        let new_k: u16 = Zebetites_Func_9(game);
        (*Get_Zebetites(new_k as i32, game)).zebet_parameter_2 = k;
        (*Get_Zebetites(k as i32, game)).zebet_parameter_2 = new_k;
    }
    (*Get_Zebetites(k as i32, game)).zebet_var_A = ((0xa6fc5bi32) as u16);
    Zebetites_Func_2(k, game);
}
pub unsafe fn Zebetites_Func_2(k: u16, game: &mut Game) {
    if ram_word!(game, door_transition_flag_elevator_zebetites) == 0 {
        (*Get_Zebetites(k as i32, game)).zebet_var_A = ((0xa6fc67i32) as u16);
        Zebetites_Func_3(game);
    }
}
pub unsafe fn Zebetites_Func_3(game: &mut Game) {
    Zebetites_Func_4(game);
    Zebetites_Func_5(game);
    let E = Get_Zebetites(ram_word!(game, cur_enemy_index) as i32, game);
    let health: u16 = (*E).base.health;
    if health != 0 {
        let mut v3: i16 = ((health) as i16).wrapping_add(1i16);
        if v3 as i32 - 1000i32 & 0x8000i32 == 0 {
            v3 = 1000i16;
        }
        (*E).base.health = v3 as u16;
    } else if (*E).zebet_parameter_1 != 0u16 || {
        Zebetites_Func_6(game);
        (*E).zebet_var_D as i32 - 4i32 & 0x8000i32 == 0
    } {
        EnemyDeathAnimation(ram_word!(game, cur_enemy_index), 0u16, game);
    } else {
        EnemyDeathAnimation(ram_word!(game, cur_enemy_index), 0u16, game);
        Zebetites_Func_8(game);
    };
}
pub unsafe fn Zebetites_Func_6(game: &mut Game) {
    let E = Get_Zebetites(ram_word!(game, cur_enemy_index) as i32, game);
    let mut v1: i16 = (((*E).zebet_var_D) as i16).wrapping_add(1i16);
    (*E).zebet_var_D = v1 as u16;
    let r18: u16 = v1 as u16;
    Zebetites_Func_7(3u16, (r18 as i32 >> 0i32 & 1i32) as u8, game);
    Zebetites_Func_7(4u16, (r18 as i32 >> 1i32 & 1i32) as u8, game);
    Zebetites_Func_7(5u16, (r18 as i32 >> 2i32 & 1i32) as u8, game);
}
pub unsafe fn Zebetites_Func_7(j: u16, a: u8, game: &mut Game) {
    if a != 0 {
        SetEventHappened(j, game);
    } else {
        ClearEventHappened(j, game);
    };
}
pub unsafe fn Zebetites_Func_8(game: &mut Game) {
    SpawnEnemy(0xa6u8, addr_stru_A6FCE1 as u16, game);
}
pub unsafe fn Zebetites_Func_9(game: &mut Game) -> u16 {
    SpawnEnemy(0xa6u8, addr_stru_A6FCF9 as u16, game)
}
pub unsafe fn Zebetites_Func_5(game: &mut Game) {
    let mut v0: u16 = 0u16;
    let E = Get_Zebetites(ram_word!(game, cur_enemy_index) as i32, game);
    let health: u16 = (*E).base.health;
    if health as i32 - 800i32 & 0x8000i32 != 0 {
        v0 = 2u16;
        if health as i32 - 600i32 & 0x8000i32 != 0 {
            v0 = 4u16;
            if health as i32 - 400i32 & 0x8000i32 != 0 {
                v0 = 6u16;
                if health as i32 - 200i32 & 0x8000i32 != 0 {
                    v0 = 8u16;
                }
            }
        }
    }
    let v3: i32 = v0 as i32 >> 1i32;
    let mut v4: u16 = crate::rtl::rom_table_word!(game, 0xa6fd4a, v3 as isize);
    if ((*E).zebet_var_F & 0x8000) != 0 {
        v4 = crate::rtl::rom_table_word!(game, 0xa6fd54, v3 as isize);
    }
    enemy_instruction! {[(*E).base]; instruction = v4, timer = 1u16}
    (*E).base.timer = 0u16;
}
pub unsafe fn Zebetites_Func_4(game: &mut Game) {
    if ram_word!(game, palette_change_num) == 0
        && (*Get_Zebetites(ram_word!(game, cur_enemy_index) as i32, game)).zebet_parameter_1 == 0
    {
        let E = Get_Zebetites(0i32, game);
        let v1: u16 = ((*E).zebet_var_C as i32 + 1i32 & 7i32) as u16;
        (*E).zebet_var_C = v1;
        WriteColorsToPalette(
            0x158u16,
            0xa6u8,
            (4i32 * v1 as i32 - 0x279i32) as u16,
            2u16,
            game,
        );
    }
}
pub unsafe fn Zebetites_Touch(game: &mut Game) {
    NormalEnemyTouchAiSkipDeathAnim_CurEnemy(game);
}
pub unsafe fn Zebetites_Shot(game: &mut Game) {
    QueueSfx3_Max6(9u16, game);
    NormalEnemyShotAiSkipDeathAnim_CurEnemy(game);
    let E = Get_Zebetites(ram_word!(game, cur_enemy_index) as i32, game);
    let G = Get_Zebetites((*E).zebet_parameter_2 as i32, game);
    (*G).base.health = (*E).base.health;
    (*G).base.flash_timer = (*E).base.flash_timer;
}
