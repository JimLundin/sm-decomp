pub use crate::game::constants::bank_84::*;
pub use crate::game::constants::bank_86::*;
pub use crate::game::constants::bank_8d::*;
pub use crate::game::constants::enemy_properties::*;
pub use crate::game::constants::samus_poses::*;
use crate::game::enemy_instruction;
use crate::game::enemy_layouts::enemy_layout;
use crate::game::sm_80::{
    CheckBossBitForCurArea, NextRandom, QueueMusic_Delayed8, QueueSfx2_Max6, SetBossBitForCurArea,
};
use crate::game::sm_82::sub_82DAF7;
use crate::game::sm_84::{SetGoldenTorizoPalette, SpawnHardcodedPlm, SpawnPLM};
use crate::game::sm_86::{SpawnEprojWithGfx, SpawnEprojWithRoomGfx};
use crate::game::sm_88::SpawnBombTorizoHaze;
use crate::game::sm_8d::SpawnPalfxObject;
use crate::game::sm_90::CallSomeSamusCode;
use crate::game::sm_94::kSinCosTable8bit_Sext;
use crate::game::sm_a0::{
    CalculateAngleOfEnemyXfromEnemyY, CalculateBlockContainingPixelPos, CallEnemyPreInstr,
    CompareDistToSamus_X, EnemyFunc_C8AD, Enemy_ItemDrop_BombTorizo, Enemy_ItemDrop_GoldenTorizo,
    Enemy_MoveDown, Enemy_MoveRight_IgnoreSlopes, Enemy_SwitchToFrozenAi, NormalEnemyFrozenAI,
    NormalEnemyShotAi, NormalEnemyShotAiSkipDeathAnim_CurEnemy, NormalEnemyTouchAi,
    NormalEnemyTouchAiSkipDeathAnim_CurEnemy,
};
use crate::game_state::Game;
use crate::ram_map::ram_ptr;
use crate::ram_map::set_ram_words;
use crate::ram_map::{ram_word, set_ram_word, update_ram_word};
use crate::rtl::abs16;
use crate::rtl::{AddToHiLo, SetHiLo};
use crate::rtl::{RomPtr, Unreachable};
use ::core::ffi::c_long;
use ::core::ptr::null;

pub use crate::types::Consts_84;
pub use crate::types::Consts_86;
pub use crate::types::Consts_8D;
pub use crate::types::Consts_AA;
pub use crate::types::EnemyProps;
pub use crate::types::{EnemyBase, EnemyData, SamusPose, VoidP};
crate::game::constants::typed_constants! { Consts_AA;
    addr_kShaktool_Ilist_E461 = 58465, addr_kShaktool_Ilist_E3A7 = 58279,
    addr_kTorizo_Ilist_D2BF = 53951, addr_kTorizo_Ilist_D2AD = 53933,
    addr_kTorizo_Ilist_D203 = 53763, addr_kTorizo_Ilist_D1F1 = 53745,
    addr_kTorizo_Ilist_CEFF = 52991, addr_kTorizo_Ilist_CEA5 = 52901,
    addr_kTorizo_Ilist_CE43 = 52803, addr_kTorizo_Ilist_CDE1 = 52705,
    addr_locret_AAC95E = 51550, addr_kTorizo_Ilist_C188 = 49544,
    addr_off_AAC0F2 = 49394, addr_kTorizo_Ilist_BDD8 = 48600,
    addr_kTorizo_Ilist_BD0E = 48398, addr_off_AABC78 = 48248,
    addr_kTorizo_Ilist_B962 = 47458, addr_kTorizo_Ilist_B1C8 = 45512,
    addr_kTorizo_Ilist_B155 = 45397, addr_kTorizo_Ilist_B0E5 = 45285,
    addr_kTorizo_ExtSprmap_87D0 = 34768, addr_kSpritemap_Nothing_AA = 32845,
}
pub use crate::types::SpawnHardcodedPlmArgs;
enemy_layout! {
    Enemy_Shaktool {
        base: EnemyBase,
        words: [
            shakt_var_A, shakt_var_B, shakt_var_C, shakt_var_D, shakt_var_E, shakt_var_F,
            shakt_parameter_1, shakt_parameter_2,
        ],
    }
}

enemy_layout! {
    Enemy_Torizo {
        base: EnemyBase,
        words: [
            toriz_var_A, toriz_var_B, toriz_var_C, toriz_var_D, toriz_var_E, toriz_var_F,
            toriz_parameter_1, toriz_parameter_2,
        ],
        pad1: [u8; 26696],
        words: [
            toriz_var_00, toriz_var_01, toriz_var_02, toriz_var_03, toriz_var_04,
            toriz_var_05, toriz_var_06, toriz_var_07, toriz_var_08, toriz_var_09,
            toriz_var_0A, toriz_var_0B, toriz_var_0C, toriz_var_0D, toriz_var_0E,
            toriz_var_0F, toriz_var_10, toriz_var_11, toriz_var_12, toriz_var_13,
            toriz_var_14, toriz_var_15, toriz_var_16, toriz_var_17, toriz_var_18,
            toriz_var_19, toriz_var_1A, toriz_var_1B, toriz_var_1C, toriz_var_1D,
            toriz_var_1E, toriz_var_1F,
        ],
    }
}

#[inline]
unsafe fn RomBankBase(bank: u8, game: &mut Game) -> *const u8 {
    RomPtr((((bank as i32) << 16i32) + 0x8000i32) as u32, game).offset(-(0x8000i32 as isize))
}
crate::game::constants::typed_constants! { i32;
    fnnullsub_171_AA = 0xaa807b, fnnullsub_270 = 0xaac6ab,
    fnTorizo_Func_3 = 0xaac6bf,
}
crate::game::constants::typed_constants! { u32;
    fnTorizo_Func_4 = 11192006, fnTorizo_Func_1 = 11192063,
    fnTorizo_Func_9 = 11195842, fnTorizo_Func_10 = 11195871,
    fnTorizo_D5E6 = 11195878,
}
crate::game::constants::typed_constants! { i32;
    fnnullsub_274 = 0xaadcaa, fnnullsub_276 = 0xaae7a6,
}
crate::ram_map::ram_pointer_functions! {
    gEnemyData(o: i32) -> EnemyData = [0xf78, o as isize];
    Get_Shaktool(o: i32) -> Enemy_Shaktool = [0xf78, o as isize];
    Get_Torizo(o: i32) -> Enemy_Torizo = [0xf78, o as isize];
}
static g_word_AAB096: u16 = 6u16;
static g_word_AAB098: u16 = 5u16;
static g_word_AAB09A: u16 = 3u16;
static g_word_AAC3EE: [i16; 16] = [-9, -6, -7, 5, -16, -7, 0, 0, 9, 6, 7, -5, 16, 7, 0, 0];
static g_word_AAC40E: [i16; 8] = [0, -6, -6, -7, 0, 0, 0, 0];
static g_word_AAC440: [i16; 16] = [-9, -6, -7, 5, -16, -7, 0, 0, 9, 6, 7, -5, 16, 7, 0, 0];
static g_word_AAC460: [i16; 8] = [0, -6, -6, -7, 0, 0, 0, 0];
static g_word_AAC4BD: [i16; 20] = [
    -5, 0, -5, -19, -16, -7, 0, -7, -17, -18, 5, 0, 5, 19, 16, 7, 0, 7, 17, 18,
];
static g_word_AAC532: [i16; 20] = [
    -5, 0, -5, -0x13, -0x10, -7, 0, -7, -0x11, -0x12, 5, 0, 5, 0x13, 0x10, 7, 0, 7, 0x11, 0x12,
];
static g_word_AAC95F: [i16; 2] = [0xdb, 0x1a8];
static g_word_AAC963: [i16; 2] = [0xb3, 0x90];
static g_word_AAC96B: [i16; 2] = [0x2800, 0x2800];
static g_word_AAC96F: [i16; 2] = [0x12, 0x12];
static g_word_AAC973: [i16; 2] = [0x30, 0x29];
static kTorizo_Palette: [u16; 16] = [
    0x3800, 0x3ff, 0x33b, 0x216, 0x113, 0x6b1e, 0x4a16, 0x3591, 0x20e9, 0x1580, 0x1580, 0x1580,
    0x1580, 0x1580, 0x1580, 0x1580,
];
static kTorizo_Palettes_1: [u16; 16] = [
    0x3800, 0x2df, 0x1d7, 0xac, 0x5a73, 0x41ad, 0x2d08, 0x1863, 0x1486, 0x145, 0x145, 0x145,
    0x7fff, 0x145, 0x145, 0,
];
static kTorizo_Palettes_2: [u16; 16] = [
    0x3800, 0x679f, 0x5299, 0x252e, 0x14aa, 0x5efc, 0x4657, 0x35b2, 0x2d70, 0x5b7f, 0x3df8, 0x2d0e,
    0x5f5f, 0x5e1a, 0x5d35, 0xc63,
];
static kTorizo_Palettes_3: [u16; 16] = [
    0x3800, 0x4aba, 0x35b2, 0x847, 3, 0x4215, 0x2970, 0x18cb, 0x1089, 0x463a, 0x28b3, 0x1809,
    0x6f7f, 0x51fd, 0x4113, 0xc63,
];
static kTorizo_Palettes_4: [u16; 16] = [
    0x3800, 0x56ba, 0x41b2, 0x1447, 0x403, 0x4e15, 0x3570, 0x24cb, 0x1868, 0x6f7f, 0x51f8, 0x410e,
    0x31f, 0x1da, 0xf5, 0xc63,
];
static kTorizo_Palettes_5: [u16; 16] = [
    0x3800, 0x4215, 0x2d0d, 2, 0, 0x3970, 0x20cb, 0xc26, 0x403, 0x463a, 0x28b3, 0x1809, 0x6f7f,
    0x51fd, 0x4113, 0xc63,
];
static kTorizo_Palettes_6: [u16; 16] = [
    0x3800, 0x6ab5, 0x49b0, 0x1c45, 0xc01, 0x5613, 0x416d, 0x2cc9, 0x2066, 0x5714, 0x31cc, 0x14e3,
    0x5630, 0x3569, 0x1883, 0xc66,
];
static kTorizo_Palettes_7: [u16; 16] = [
    0x3800, 0x5610, 0x350b, 0x800, 0, 0x416e, 0x2cc8, 0x1823, 0xc01, 0x6a31, 0x4caa, 0x2406,
    0x7f7b, 0x75f4, 0x4d10, 0xc63,
];
static kTorizo_Palettes_8: [u16; 16] = [
    0x3800, 0x4bbe, 0x6b9, 0xa8, 0, 0x173a, 0x276, 0x1f2, 0x14d, 0x73e0, 0x4f20, 0x2a20, 0x7fe0,
    0x5aa0, 0x5920, 0x43,
];
static kTorizo_Palettes_10: [u16; 16] = [
    0x3800, 0x3719, 0x214, 3, 0, 0x295, 0x1d1, 0x14d, 0xa8, 0x4b40, 0x25e0, 0xe0, 0x6b40, 0x4600,
    0x4480, 0,
];
static g_word_AAD59A: [i16; 20] = [
    -5, 0, -5, -19, -16, -7, 0, -7, -17, -18, 5, 0, 5, 19, 16, 7, 0, 7, 17, 18,
];
static g_word_AADEE9: [u16; 7] = [0, 0x20, 0x60, 0xc0, 0x140, 0x1a0, 0x1e0];
static kChozoStatue_Palette: [u16; 16] = [
    0x3800, 0x633f, 0x4a9f, 0x2ddf, 0x6739, 0x4e73, 0x318c, 0x18c6, 0x27ff, 0x1af7, 0xdce, 0xc6,
    0x3fff, 0x2b39, 0x7fff, 0,
];
static kChozoStatue_Palettes: [i16; 16] = [
    0x3800, 0x633f, 0x4a9f, 0x2ddf, 0x4210, 0x318c, 0x2108, 0x1084, 0x27ff, 0x1af7, 0xdce, 0xc6,
    0x3fff, 0x2b39, 0x5294, 0,
];
static kChozoStatue_Palettes2: [i16; 16] = [
    0x3800, 0x633f, 0x4a9f, 0x2ddf, 0x2f7c, 0x2295, 0x118d, 0x8e8, 0x27ff, 0x1af7, 0xdce, 0xc6,
    0x3fff, 0x2b39, 0x73df, 0x43,
];
static kChozoStatue_Palettes3: [i16; 16] = [
    0x3800, 0x633f, 0x4a9f, 0x2ddf, 0x2295, 0x118d, 0x8e8, 0x85, 0x27ff, 0x1af7, 0xdce, 0xc6,
    0x3fff, 0x2b39, 0x5294, 1,
];
pub unsafe fn Enemy_GrappleReact_CancelBeam_AA(game: &mut Game) {
    Enemy_SwitchToFrozenAi(game);
}
pub unsafe fn Enemy_NormalFrozenAI_AA(game: &mut Game) {
    NormalEnemyFrozenAI(game);
}
pub unsafe fn Enemy_ClearAiPreInstr_AA(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*gEnemyData(k as i32, game)).ai_preinstr = ((0xaa807bi32) as u16);
    jp
}
pub unsafe fn Enemy_SetAiPreInstr_AA(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*gEnemyData(k as i32, game)).ai_preinstr = *jp.offset(0);
    jp.offset(1)
}
pub unsafe fn Torizo_Instr_3(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*Get_Torizo(k as i32, game)).toriz_var_E = *jp.offset(0);
    jp.offset(1)
}
pub unsafe fn Torizo_Instr_31(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E: *mut Enemy_Torizo = Get_Torizo(k as i32, game);
    (*E).toriz_parameter_2 = ((*E).toriz_parameter_2) | (0x8000u16);
    let mut parameter = SpawnEprojWithGfx((*E).toriz_parameter_2, k, addr_stru_86A95B as u16, game);
    parameter = SpawnEprojWithGfx(parameter, k, addr_stru_86A95B as u16, game);
    parameter = SpawnEprojWithGfx(parameter, k, addr_stru_86A95B as u16, game);
    parameter = SpawnEprojWithGfx(parameter, k, addr_stru_86A95B as u16, game);
    parameter = SpawnEprojWithGfx(parameter, k, addr_stru_86A95B as u16, game);
    SpawnEprojWithGfx(parameter, k, addr_stru_86A95B as u16, game);
    jp
}
pub unsafe fn Torizo_Instr_33(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E: *mut Enemy_Torizo = Get_Torizo(k as i32, game);
    (*E).toriz_parameter_2 = ((*E).toriz_parameter_2) | (0x4000u16);
    jp
}
pub unsafe fn Torizo_Instr_36(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E: *mut Enemy_Torizo = Get_Torizo(k as i32, game);
    (*E).base.properties = ((*E).base.properties as i32 & !(kEnemyProps_Invisible as i32)) as u16;
    jp
}
pub unsafe fn Torizo_Instr_37(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E: *mut Enemy_Torizo = Get_Torizo(k as i32, game);
    (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Invisible) as u16);
    jp
}
pub unsafe fn Torizo_Instr_35(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let mut i: i32 = 30i32;
    while i >= 0i32 {
        let v3: i32 = i >> 1i32;
        set_ram_words! { game, 0u16;
            target_palettes[(v3 + 160i32)], target_palettes[(v3 + 144i32)],
        }
        i -= 2i32;
    }
    jp
}
pub unsafe fn Torizo_Instr_38(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    SetBossBitForCurArea(4u16, game);
    QueueMusic_Delayed8(g_word_AAB09A, game);
    if ram_word!(game, area_index) != 0 {
        Enemy_ItemDrop_GoldenTorizo(k, game);
    } else {
        Enemy_ItemDrop_BombTorizo(k, game);
    }
    jp
}
pub unsafe fn Torizo_Instr_6(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    sub_82DAF7(0x600u16, game);
    jp
}
pub unsafe fn Torizo_Instr_5(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    Torizo_C268(game);
    jp
}
pub unsafe fn Torizo_Instr_9(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    QueueMusic_Delayed8(g_word_AAB098, game);
    SpawnPalfxObject(addr_stru_8DF759 as u16, game);
    jp
}
pub unsafe fn Torizo_C20A(k: u16, game: &mut Game) {
    let mut v2: i16 = 0;
    let E: *mut Enemy_Torizo = Get_Torizo(k as i32, game);
    if ((*E).toriz_parameter_1 as i32 & 0x8000i32) != 0i32 {
        v2 = 512i16;
    } else {
        v2 = -512i16;
    }
    (*E).toriz_var_A = v2 as u16;
    (*E).toriz_var_B = -1472i32 as u16;
    (*E).toriz_var_C = 40u16;
    (*E).base.instruction_timer = 1u16;
}
pub unsafe fn Torizo_C22D(k: u16, game: &mut Game) {
    let mut v2: i16 = 0;
    let E: *mut Enemy_Torizo = Get_Torizo(k as i32, game);
    if ((*E).toriz_parameter_1 as i32 & 0x8000i32) != 0i32 {
        v2 = -768i16;
    } else {
        v2 = 768i16;
    }
    (*E).toriz_var_A = v2 as u16;
    (*E).toriz_var_B = -1152i32 as u16;
    (*E).toriz_var_C = 40u16;
    (*E).base.instruction_timer = 1u16;
}
pub unsafe fn Torizo_C250(game: &mut Game) {
    let mut i: i32 = 30i32;
    while i >= 0i32 {
        let v1: i32 = i >> 1i32;
        set_ram_word!(game.target_palettes[(v1 + 160i32)] = kTorizo_Palettes_3[v1 as usize]);
        set_ram_word!(game.target_palettes[(v1 + 144i32)] = kTorizo_Palettes_2[v1 as usize]);
        i -= 2i32;
    }
}
pub unsafe fn Torizo_C268(game: &mut Game) {
    let mut i: i32 = 30i32;
    while i >= 0i32 {
        let v1: i32 = i >> 1i32;
        set_ram_word!(game.target_palettes[(v1 + 160i32)] = kTorizo_Palettes_5[v1 as usize]);
        set_ram_word!(game.target_palettes[(v1 + 144i32)] = kTorizo_Palettes_4[v1 as usize]);
        i -= 2i32;
    }
}
pub unsafe fn Torizo_C280(game: &mut Game) {
    let mut i: i32 = 30i32;
    while i >= 0i32 {
        let v1: i32 = i >> 1i32;
        set_ram_word!(game.target_palettes[(v1 + 160i32)] = kTorizo_Palettes_7[v1 as usize]);
        set_ram_word!(game.target_palettes[(v1 + 144i32)] = kTorizo_Palettes_6[v1 as usize]);
        i -= 2i32;
    }
}
pub unsafe fn Torizo_C298(game: &mut Game) {
    let mut i: i32 = 30i32;
    while i >= 0i32 {
        let v1: i32 = i >> 1i32;
        set_ram_word!(game.target_palettes[(v1 + 160i32)] = kTorizo_Palettes_10[v1 as usize]);
        set_ram_word!(game.target_palettes[(v1 + 144i32)] = kTorizo_Palettes_8[v1 as usize]);
        i -= 2i32;
    }
}
pub unsafe fn Torizo_C2B0(game: &mut Game) {
    let mut i: i32 = 30i32;
    while i >= 0i32 {
        let v1: i32 = i >> 1i32;
        set_ram_word!(game.palette_buffer[(v1 + 160i32)] = kTorizo_Palettes_5[v1 as usize]);
        set_ram_word!(game.palette_buffer[(v1 + 144i32)] = kTorizo_Palettes_4[v1 as usize]);
        i -= 2i32;
    }
}
pub fn Torizo_Instr_7(k: u16, jp: *const u16) -> *const u16 {
    jp
}
pub unsafe fn Torizo_Instr_2(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*Get_Torizo(k as i32, game)).toriz_var_04 = 30583u16;
    jp
}
pub unsafe fn Torizo_Instr_8(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*Get_Torizo(k as i32, game)).toriz_var_04 = 0u16;
    jp
}
pub unsafe fn Torizo_Instr_25(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    if ((*Get_Torizo(k as i32, game)).toriz_parameter_2 as i32 & 0x4000i32) != 0i32 {
        return *jp.offset(0) as usize as *const u16;
    }
    if ram_word!(game, area_index) != 0 {
        *jp.offset(1) as usize as *const u16
    } else {
        jp.offset(2)
    }
}
pub unsafe fn Torizo_Instr_22(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*Get_Torizo(k as i32, game)).toriz_var_00 = *jp.offset(0);
    jp.offset(1)
}
pub unsafe fn Torizo_Instr_19(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*Get_Torizo(k as i32, game)).toriz_var_00 as usize as *const u16
}
pub unsafe fn Torizo_Instr_32(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*Get_Torizo(k as i32, game)).toriz_var_01 as usize as *const u16
}
pub unsafe fn Torizo_Instr_30(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let a: u16 = *jp.offset(0);
    let mut i: i32 = 5i32;
    while i >= 0i32 {
        SpawnEprojWithRoomGfx(addr_kEproj_BombTorizoLowHealthExplode as u16, a, game);
        i -= 1;
    }
    let E: *mut Enemy_Torizo = Get_Torizo(k as i32, game);
    (*E).base.current_instruction = (jp.offset(1) as *mut u8)
        .offset_from(RomBankBase((*gEnemyData(k as i32, game)).bank, game))
        as c_long as u16;
    (*E).base.flash_timer = 40u16;
    (*E).base.instruction_timer = 40u16;
    null::<u16>()
}
pub unsafe fn Torizo_Instr_34(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    // The instruction dispatcher leaves this handler address in A.
    SpawnEprojWithRoomGfx(addr_kEproj_BombTorizoDeathExplosion as u16, 0xc32f, game);
    let E: *mut Enemy_Torizo = Get_Torizo(k as i32, game);
    (*E).base.current_instruction = (jp as *mut u8)
        .offset_from(RomBankBase((*gEnemyData(k as i32, game)).bank, game))
        as c_long as u16;
    (*E).base.flash_timer = 1u16;
    (*E).base.instruction_timer = 1u16;
    null::<u16>()
}
pub unsafe fn Torizo_Instr_24(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let left_foot_parameter = SpawnEprojWithRoomGfx(
        addr_kEproj_TourianLandingDustCloudsRightFoot as u16,
        0xc34a,
        game,
    );
    SpawnEprojWithRoomGfx(addr_stru_86AFF3 as u16, left_foot_parameter, game);
    jp
}
pub unsafe fn Torizo_Instr_12(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let health: u16 = (*Get_Torizo(k as i32, game)).base.health;
    if health < 0x15eu16 {
        SpawnEprojWithGfx(
            health,
            k,
            addr_kEproj_BombTorizoLowHealthInitialDrool as u16,
            game,
        );
    }
    jp
}
pub unsafe fn Torizo_Instr_10(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E: *mut Enemy_Torizo = Get_Torizo(k as i32, game);
    (*E).toriz_parameter_1 = ((*E).toriz_parameter_1) | (0x4000u16);
    jp
}
pub unsafe fn Torizo_Instr_11(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E: *mut Enemy_Torizo = Get_Torizo(k as i32, game);
    (*E).toriz_parameter_1 = ((*E).toriz_parameter_1) & (0x1fffu16);
    (*E).toriz_var_09 = (*E).toriz_var_09.wrapping_add(1);
    jp
}
pub unsafe fn Torizo_Instr_29(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E: *mut Enemy_Torizo = Get_Torizo(k as i32, game);
    (*E).toriz_parameter_1 = ((*E).toriz_parameter_1 as i32 & 0x1fffi32 | 0x8000i32) as u16;
    (*E).toriz_var_09 = (*E).toriz_var_09.wrapping_add(1);
    jp
}
pub unsafe fn Torizo_Instr_1(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E: *mut Enemy_Torizo = Get_Torizo(k as i32, game);
    (*E).toriz_parameter_1 = ((*E).toriz_parameter_1 as i32 & 0x1fffi32 | 0x2000i32) as u16;
    (*E).toriz_var_09 = (*E).toriz_var_09.wrapping_add(1);
    jp
}
pub unsafe fn Torizo_Instr_28(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E: *mut Enemy_Torizo = Get_Torizo(k as i32, game);
    (*E).toriz_parameter_1 = ((*E).toriz_parameter_1 as i32 & 0x1fffi32 | 0xa000i32) as u16;
    (*E).toriz_var_09 = (*E).toriz_var_09.wrapping_add(1);
    jp
}
pub unsafe fn Torizo_Instr_4(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let v2: u16 = *jp.offset(0);
    let E: *mut Enemy_Torizo = Get_Torizo(k as i32, game);
    (*E).base.x_pos =
        ((*E).base.x_pos).wrapping_add((g_word_AAC3EE[(v2 as i32 >> 1i32) as usize]) as u16);
    (*E).base.y_pos = ((*E).base.y_pos)
        .wrapping_add((g_word_AAC40E[((v2 as i32 & 0xfi32) >> 1i32) as usize]) as u16);
    jp.offset(1)
}
pub unsafe fn Torizo_Instr_40(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let v2: u16 = *jp.offset(0);
    let E: *mut Enemy_Torizo = Get_Torizo(k as i32, game);
    (*E).base.x_pos =
        ((*E).base.x_pos).wrapping_sub((g_word_AAC440[(v2 as i32 >> 1i32) as usize]) as u16);
    (*E).base.y_pos = ((*E).base.y_pos)
        .wrapping_sub((g_word_AAC460[((v2 as i32 & 0xfi32) >> 1i32) as usize]) as u16);
    jp.offset(1)
}
pub unsafe fn Torizo_Instr_16(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E: *mut Enemy_Torizo = Get_Torizo(k as i32, game);
    (*E).toriz_var_A = g_word_AAC4BD[(*jp.offset(0) as i32 >> 1i32) as usize] as u16;
    if Enemy_MoveRight_IgnoreSlopes(k, ((*E).toriz_var_A as i16 as i32) << 16i32, game) != 0 {
        (*E).toriz_var_03 = 0u16;
        if ((*E).toriz_parameter_1 as i32 & 0x8000i32) != 0i32 {
            addr_kTorizo_Ilist_B962 as i32 as usize as *const u16
        } else {
            addr_kTorizo_Ilist_BDD8 as i32 as usize as *const u16
        }
    } else {
        EnemyFunc_C8AD(k, game);
        if (((*E).toriz_parameter_1 as i32
            ^ ram_word!(game, samus_x_pos) as i32 - (*E).base.x_pos as i32)
            & 0x8000i32)
            == 0i32
        {
            if (*E).toriz_var_03 == 0 {
                (*E).toriz_var_03 = 72u16;
            }
            jp.offset(1)
        } else {
            jp.offset(1)
        }
    }
}
pub unsafe fn Torizo_Instr_27(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E: *mut Enemy_Torizo = Get_Torizo(k as i32, game);
    (*E).toriz_var_A = g_word_AAC532[(*jp.offset(0) as i32 >> 1i32) as usize] as u16;
    if Enemy_MoveRight_IgnoreSlopes(k, ((*E).toriz_var_A as i16 as i32) << 16i32, game) != 0 {
        (*E).toriz_var_03 = 0u16;
        if ((*E).toriz_parameter_1 as i32 & 0x8000i32) != 0i32 {
            addr_kTorizo_Ilist_BD0E as i32 as usize as *const u16
        } else {
            addr_kTorizo_Ilist_C188 as i32 as usize as *const u16
        }
    } else {
        EnemyFunc_C8AD(k, game);
        if (((*E).toriz_parameter_1 as i32
            ^ ram_word!(game, samus_x_pos) as i32 - (*E).base.x_pos as i32)
            & 0x8000i32)
            == 0i32
        {
            if (*E).toriz_var_03 == 0 {
                (*E).toriz_var_03 = 72u16;
            }
            jp.offset(1)
        } else {
            jp.offset(1)
        }
    }
}
pub unsafe fn Torizo_Instr_23(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    if ((*Get_Torizo(k as i32, game)).toriz_var_B as i32 & 0x8000i32) != 0i32 {
        *jp.offset(0) as usize as *const u16
    } else {
        jp.offset(1)
    }
}
pub unsafe fn Torizo_Instr_14(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    if CompareDistToSamus_X(k, 0x38u16, game) as i32 & 1i32 != 0 {
        return jp.offset(1);
    }
    let E: *mut Enemy_Torizo = Get_Torizo(k as i32, game);
    if (((*E).toriz_parameter_1 as i32
        ^ ram_word!(game, samus_x_pos) as i32 - (*E).base.x_pos as i32)
        & 0x8000i32)
        == 0i32
    {
        return jp.offset(1);
    }
    (*E).toriz_var_00 = (jp.offset(1) as *mut u8)
        .offset_from(RomBankBase((*gEnemyData(k as i32, game)).bank, game))
        as c_long as u16;
    *jp.offset(0) as usize as *const u16
}
pub unsafe fn Torizo_Instr_15(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    if CompareDistToSamus_X(k, 0x20u16, game) as i32 & 1i32 != 0
        || (Torizo_Func_12(k, game) as i32 & 0x8000i32) != 0i32
    {
        return jp.offset(1);
    }
    Torizo_C22D(k, game);
    *jp.offset(0) as usize as *const u16
}
pub unsafe fn Torizo_Instr_26(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E: *mut Enemy_Torizo = Get_Torizo(k as i32, game);
    (*E).toriz_var_00 = (jp.offset(2) as *mut u8)
        .offset_from(RomBankBase((*gEnemyData(k as i32, game)).bank, game))
        as c_long as u16;
    if ram_word!(game, samus_missiles) < 5u16
        || (ram_word!(game, nmi_frame_counter_word) as i32
            + (ram_word!(game, samus_x_pos) as i32 & 1i32)
            + (ram_word!(game, samus_x_pos) as i32 >> 1i32)
            & 8i32)
            != 0i32
    {
        *jp.offset(0) as usize as *const u16
    } else {
        *jp.offset(1) as usize as *const u16
    }
}
pub unsafe fn Torizo_Instr_18(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    // The instruction dispatcher leaves this handler address in A.
    let mut parameter = SpawnEprojWithGfx(0xc5cb, k, addr_kEproj_BombTorizosChozoOrbs as u16, game);
    parameter = SpawnEprojWithGfx(parameter, k, addr_kEproj_BombTorizosChozoOrbs as u16, game);
    SpawnEprojWithGfx(parameter, k, addr_kEproj_BombTorizosChozoOrbs as u16, game);
    jp
}
pub unsafe fn Torizo_Instr_20(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    SpawnEprojWithGfx(
        *jp.offset(0),
        k,
        addr_kEproj_BombTorizoSonicBoom as u16,
        game,
    );
    jp.offset(1)
}
pub unsafe fn Torizo_Instr_44(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    SpawnEprojWithGfx(
        *jp.offset(0),
        k,
        addr_kEproj_GoldenTorizoSonicBoom as u16,
        game,
    );
    jp.offset(1)
}
pub unsafe fn Torizo_Instr_21(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    SpawnEprojWithRoomGfx(
        addr_kEproj_BombTorizoExplosiveSwipe as u16,
        *jp.offset(0),
        game,
    );
    jp.offset(1)
}
pub unsafe fn Torizo_Instr_17(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    QueueSfx2_Max6(0x27u16, game);
    jp
}
pub unsafe fn Torizo_Instr_13(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    QueueSfx2_Max6(0x4bu16, game);
    jp
}
pub unsafe fn Torizo_C620(k: u16, game: &mut Game) {
    if ram_word!(game, area_index) == 0
        && (ram_word!(game, random_number) as i32 & 0x8142i32) == 0i32
    {
        let health: u16 = (*Get_Torizo(k as i32, game)).base.health;
        if health != 0u16 && health < 0x15eu16 {
            SpawnEprojWithGfx(
                health,
                k,
                addr_kEproj_BombTorizoLowHealthInitialDrool as u16,
                game,
            );
        }
    }
}
pub unsafe fn Torizo_C643(k: u16, game: &mut Game) {
    let E: *mut Enemy_Torizo = Get_Torizo(k as i32, game);
    if Enemy_MoveDown(k, ((*E).toriz_var_B as i16 as i32) << 8i32, game) != 0 {
        let v3: i16 = (*E).toriz_var_B as i16;
        if v3 >= 0i16 && v3 != 256i16 {
            set_ram_word!(game.earthquake_type = 4u16);
            set_ram_word!(game.earthquake_timer = 32u16);
            (*E).toriz_var_B = 256u16;
        }
    } else {
        (*E).toriz_var_B = ((*E).toriz_var_B).wrapping_add(40u16);
    };
}
pub unsafe fn Torizo_Hurt(game: &mut Game) {
    Torizo_C620(ram_word!(game, cur_enemy_index), game);
    if (*Get_Torizo(ram_word!(game, cur_enemy_index) as i32, game))
        .base
        .flash_timer as i32
        & 1i32
        != 0
    {
        let mut i: i32 = 30i32;
        while i >= 0i32 {
            let v2: i32 = i >> 1i32;
            set_ram_words! { game, 0x7fffu16;
                palette_buffer[(v2 + 160i32)], palette_buffer[(v2 + 144i32)],
            }
            i -= 2i32;
        }
    } else {
        Torizo_C2B0(game);
    };
}
crate::game::address_dispatch! {
    pub unsafe fn CallTorizoFunc(ea: u32, k: u16, game: &mut Game) => ea;
        call (k, game); default { Unreachable(game); };
    {
        11191979 => [()]; 11191999 => Torizo_Func_3; 11192006 => Torizo_Func_4;
        11192063 => Torizo_Func_1; 11195842 => Torizo_Func_9; 11195871 => Torizo_Func_10;
        11195878 => Torizo_D5E6;
    }
}
pub unsafe fn Torizo_Main(game: &mut Game) {
    let E: *mut Enemy_Torizo = Get_Torizo(ram_word!(game, cur_enemy_index) as i32, game);
    CallTorizoFunc(
        (((*E).toriz_var_E) as u32) | (0xaa0000u32),
        ram_word!(game, cur_enemy_index),
        game,
    );
}
pub unsafe fn Torizo_Func_2(k: u16, game: &mut Game) {
    Torizo_C643(k, game);
    if sub_82DAF7(0x600u16, game) as i32 & 1i32 == 0 {
        (*Get_Torizo(k as i32, game)).toriz_var_E = ((0xaac6abi32) as u16);
    }
}
pub unsafe fn Torizo_Func_3(k: u16, game: &mut Game) {
    Torizo_C620(k, game);
    Torizo_C643(k, game);
}
pub unsafe fn Torizo_Func_4(k: u16, game: &mut Game) {
    let E: *mut Enemy_Torizo = Get_Torizo(k as i32, game);
    (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
    let mut v2: u16 = 78u16;
    while ram_word!(game, plm_header_ptr[(v2 as i32 >> 1i32)]) as i32 != addr_kPlmHeader_D6EA as i32
    {
        v2 = (v2).wrapping_sub(2u16);
        if (v2 as i32 & 0x8000i32) != 0i32 {
            QueueMusic_Delayed8(g_word_AAB096, game);
            (*E).base.properties =
                ((*E).base.properties as i32 & !(kEnemyProps_Tangible as i32)) as u16;
            enemy_instruction! {[(*E).base]; instruction = ((*E).base.current_instruction).wrapping_add(2u16), timer = 1u16}
            return;
        }
    }
}
pub unsafe fn Torizo_Func_1(k: u16, game: &mut Game) {
    Torizo_C620(k, game);
    let E: *mut Enemy_Torizo = Get_Torizo(k as i32, game);
    if ((*E).toriz_parameter_2 as i32 & 0x8000i32) != 0i32 || (*E).base.health >= 0x15eu16 {
        if ((*E).toriz_parameter_2 as i32 & 0x4000i32) != 0i32 || (*E).base.health >= 0x64u16 {
            CallEnemyPreInstr((((*E).toriz_var_F) as u32) | (0xaa0000u32), game);
        } else {
            let mut v2: u16 = 0;
            if ((*E).toriz_parameter_1 as i32 & 0x8000i32) != 0i32 {
                v2 = addr_kTorizo_Ilist_BD0E as u16;
            } else {
                v2 = addr_kTorizo_Ilist_C188 as u16;
            }
            (*E).toriz_var_00 = v2;
            enemy_instruction! {[(*E).base]; instruction = addr_kTorizo_Ilist_B155 as u16, timer = 1u16}
        }
    } else {
        (*E).toriz_var_01 = (*E).base.current_instruction;
        enemy_instruction! {[(*E).base]; instruction = addr_kTorizo_Ilist_B0E5 as u16, timer = 1u16}
    };
}
pub unsafe fn Torizo_Func_5(k: u16, game: &mut Game) {
    let mut v3: u16 = 0;
    let mut v8: u16 = 0;
    let E: *mut Enemy_Torizo = Get_Torizo(k as i32, game);
    if ((*E).toriz_parameter_2 as i32 & 0x4000i32) != 0i32 {
        let torizo_var_03: u16 = (*E).toriz_var_03;
        if torizo_var_03 == 0 || {
            v8 = (torizo_var_03).wrapping_sub(1u16);
            (*E).toriz_var_03 = v8;
            (*E).toriz_var_03 != 0u16
        } {
            let mut v10: u16 = (abs16((*E).toriz_var_A)).wrapping_add(1u16);
            if v10 >= 0x10u16 {
                v10 = 15u16;
            }
            if Enemy_MoveDown(k, (v10 as i16 as i32) << 16i32, game) == 0 {
                let mut v11: u16 = 0;
                if ((*E).toriz_parameter_1 as i32 & 0x8000i32) != 0i32 {
                    v11 = addr_off_AAC0F2 as u16;
                } else {
                    v11 = addr_off_AABC78 as u16;
                }
                enemy_instruction! {[(*E).base]; instruction = v11, timer = 1u16}
                (*E).toriz_var_B = 256u16;
                (*E).toriz_var_A = 0u16;
            }
        } else {
            (*E).toriz_var_03 = 0u16;
            let mut v9: u16 = 0;
            if ((*E).toriz_parameter_1 as i32 & 0x8000i32) != 0i32 {
                v9 = addr_kTorizo_Ilist_BD0E as u16;
            } else {
                v9 = addr_kTorizo_Ilist_C188 as u16;
            }
            enemy_instruction! {[(*E).base]; instruction = v9, timer = 1u16}
        }
    } else {
        let v2: u16 = (*E).toriz_var_03;
        if v2 == 0 || {
            v3 = (v2).wrapping_sub(1u16);
            (*E).toriz_var_03 = v3;
            (*E).toriz_var_03 != 0u16
        } {
            let mut v5: u16 = (abs16((*E).toriz_var_A)).wrapping_add(1u16);
            if v5 >= 0x10u16 {
                v5 = 15u16;
            }
            if Enemy_MoveDown(k, (v5 as i16 as i32) << 16i32, game) == 0 {
                let mut v6: u16 = 0;
                if ((*E).toriz_parameter_1 as i32 & 0x8000i32) != 0i32 {
                    v6 = addr_off_AAC0F2 as u16;
                } else {
                    v6 = addr_off_AABC78 as u16;
                }
                enemy_instruction! {[(*E).base]; instruction = v6, timer = 1u16}
                (*E).toriz_var_B = 256u16;
                (*E).toriz_var_A = 0u16;
            }
        } else {
            (*E).toriz_var_03 = 0u16;
            let mut v4: u16 = 0;
            if ((*E).toriz_parameter_1 as i32 & 0x8000i32) != 0i32 {
                v4 = addr_kTorizo_Ilist_B962 as u16;
            } else {
                v4 = addr_kTorizo_Ilist_BDD8 as u16;
            }
            enemy_instruction! {[(*E).base]; instruction = v4, timer = 1u16}
        }
    };
}
pub unsafe fn Torizo_Func_6(k: u16, game: &mut Game) {
    Torizo_C643(k, game);
}
pub unsafe fn Torizo_Func_7(k: u16, game: &mut Game) {
    let E: *mut Enemy_Torizo = Get_Torizo(k as i32, game);
    Enemy_MoveRight_IgnoreSlopes(k, ((*E).toriz_var_A as i16 as i32) << 8i32, game);
    EnemyFunc_C8AD(k, game);
    if Enemy_MoveDown(k, ((*E).toriz_var_B as i16 as i32) << 8i32, game) != 0 {
        enemy_instruction! {[(*E).base]; instruction = (*E).toriz_var_00, timer = 1u16}
        (*E).toriz_var_B = 256u16;
        set_ram_word!(game.earthquake_type = 4u16);
        set_ram_word!(game.earthquake_timer = 32u16);
    } else {
        (*E).toriz_var_B = ((*E).toriz_var_B).wrapping_add((*E).toriz_var_C);
    };
}
pub unsafe fn Torizo_Init(game: &mut Game) {
    let E: *mut Enemy_Torizo = Get_Torizo(ram_word!(game, cur_enemy_index) as i32, game);
    if CheckBossBitForCurArea(4u16, game) as i32 & 1i32 != 0 {
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Deleted) as u16);
    } else {
        let v2: i32 = ram_word!(game, area_index) as i32 >> 1i32;
        (*E).base.properties = ((*E).base.properties) | ((g_word_AAC96B[v2 as usize]) as u16);
        (*E).base.extra_properties = ((*E).base.extra_properties) | (4u16);
        (*E).base.x_width = g_word_AAC96F[v2 as usize] as u16;
        (*E).base.y_height = g_word_AAC973[v2 as usize] as u16;
        (*E).toriz_var_E = ((0xaac6bfi32) as u16);
        (*E).base.instruction_timer = 1u16;
        (*E).base.timer = 0u16;
        (*E).base.palette_index = 0u16;
        (*E).toriz_var_F = addr_locret_AAC95E as u16;
        (*E).base.current_instruction = crate::rtl::rom_table_word!(game, 0xaac967, v2 as isize);
        (*E).base.spritemap_pointer = addr_kTorizo_ExtSprmap_87D0 as u16;
        (*E).base.x_pos = g_word_AAC95F[v2 as usize] as u16;
        (*E).base.y_pos = g_word_AAC963[v2 as usize] as u16;
        (*E).toriz_var_A = 0u16;
        (*E).toriz_var_B = 256u16;
        let mut i: i32 = 30i32;
        while i >= 0i32 {
            let v4: i32 = i >> 1i32;
            set_ram_word!(game.target_palettes[(v4 + 240i32)] = kTorizo_Palettes_1[v4 as usize]);
            set_ram_word!(game.target_palettes[(v4 + 176i32)] = kTorizo_Palette[v4 as usize]);
            i -= 2i32;
        }
        if ram_word!(game, area_index) != 0 {
            Torizo_C280(game);
            if ram_word!(game, joypad1_lastkeys) == 0xc0c0u16 {
                set_ram_words! { game, 700u16; samus_health, samus_max_health }
                set_ram_words! { game, 300u16; samus_max_reserve_health, samus_reserve_health }
                set_ram_words! { game, 100u16; samus_missiles, samus_max_missiles }
                set_ram_words! { game, 20u16;
                    samus_super_missiles, samus_max_super_missiles, samus_power_bombs,
                    samus_max_power_bombs,
                }
                set_ram_word!(game.equipped_items = -3273i32 as u16);
                set_ram_word!(game.collected_items = -3273i32 as u16);
                set_ram_words! { game, 4111u16; equipped_beams, collected_beams }
            }
        } else {
            Torizo_C250(game);
            SpawnBombTorizoHaze(game);
        }
    };
}
pub unsafe fn GoldTorizo_Touch(game: &mut Game) {
    NormalEnemyTouchAiSkipDeathAnim_CurEnemy(game);
}
pub unsafe fn Torizo_Shot(game: &mut Game) {
    if ram_word!(game, area_index) != 0 {
        GoldTorizo_Shot(game);
    } else {
        let E: *mut Enemy_Torizo = Get_Torizo(ram_word!(game, cur_enemy_index) as i32, game);
        if (*E).toriz_var_04 as i32 | (*E).base.flash_timer as i32 == 0 {
            NormalEnemyShotAiSkipDeathAnim_CurEnemy(game);
            if (*E).base.health == 0 {
                (*E).toriz_var_E = ((0xaac6abi32) as u16);
                enemy_instruction! {[(*E).base]; instruction = addr_kTorizo_Ilist_B1C8 as u16, timer = 1u16}
                (*E).toriz_parameter_2 = ((*E).toriz_parameter_2) | (0xc000u16);
                (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
            }
        }
    };
}
pub unsafe fn Torizo_Func_8(game: &mut Game) {
    if ram_word!(game, area_index) != 0 {
        Torizo_D658(game);
    }
}
pub unsafe fn Torizo_Instr_39(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    if (*Get_Torizo(k as i32, game)).base.y_pos == 375u16 {
        jp.offset(1)
    } else {
        *jp.offset(0) as usize as *const u16
    }
}
pub unsafe fn Torizo_Instr_41(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    Torizo_C298(game);
    jp
}
pub unsafe fn Torizo_Instr_42(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    QueueMusic_Delayed8(5u16, game);
    let E: *mut Enemy_Torizo = Get_Torizo(k as i32, game);
    (*E).base.x_width = 18u16;
    (*E).base.y_height = 48u16;
    SpawnPalfxObject(addr_stru_8DF75D as u16, game);
    jp
}
pub unsafe fn Torizo_Instr_48(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E: *mut Enemy_Torizo = Get_Torizo(k as i32, game);
    (*E).toriz_parameter_2 = ((*E).toriz_parameter_2 as i32 & !(0x1000i32)) as u16;
    jp
}
pub unsafe fn Torizo_Instr_57(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    SpawnEprojWithRoomGfx(addr_stru_86B1C0 as u16, 0u16, game);
    jp
}
pub unsafe fn Torizo_Instr_58(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let mut v2: i16 = 0;
    v2 = 34i16;
    while *((ram_ptr!(game, u16; 0x1997) as *mut u8).offset(v2 as u16 as i32 as isize) as *mut u16)
        != 0xb1c0u16
    {
        v2 = (v2).wrapping_sub(2i16);
        if v2 < 0i16 {
            return jp.offset(1);
        }
    }
    *jp.offset(0) as usize as *const u16
}
pub unsafe fn Torizo_Instr_59(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E: *mut Enemy_Torizo = Get_Torizo(k as i32, game);
    (*E).toriz_var_06 = ((*E).toriz_var_06 as i32 & !(0x8000i32)) as u16;
    jp
}
pub unsafe fn Torizo_Instr_62(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E: *mut Enemy_Torizo = Get_Torizo(k as i32, game);
    (*E).toriz_var_06 = ((*E).toriz_var_06) | (0x8000u16);
    jp
}
pub unsafe fn Torizo_Instr_63(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E: *mut Enemy_Torizo = Get_Torizo(k as i32, game);
    (*E).toriz_parameter_2 = ((*E).toriz_parameter_2 as i32 & !(0x2000i32)) as u16;
    jp
}
pub unsafe fn GoldTorizo_Main(game: &mut Game) {
    let E: *mut Enemy_Torizo = Get_Torizo(ram_word!(game, cur_enemy_index) as i32, game);
    if ram_word!(game, samus_pose) as i32 == kPose_1B_FaceR_SpaceJump as i32
        || ram_word!(game, samus_pose) as i32 == kPose_1C_FaceL_SpaceJump as i32
    {
        (*E).toriz_var_07 = (*E).toriz_var_07.wrapping_add(1);
    } else {
        (*E).toriz_var_07 = 0u16;
    }
    CallTorizoFunc(
        (((*E).toriz_var_E) as u32) | (0xaa0000u32),
        ram_word!(game, cur_enemy_index),
        game,
    );
}
pub unsafe fn Torizo_Instr_56(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    QueueSfx2_Max6(0x34u16, game);
    jp
}
pub unsafe fn Torizo_Instr_60(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    QueueSfx2_Max6(0x67u16, game);
    jp
}
pub unsafe fn Torizo_Instr_46(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    QueueSfx2_Max6(0x48u16, game);
    jp
}
pub unsafe fn Torizo_Func_12(k: u16, game: &mut Game) -> u16 {
    let E: *mut Enemy_Torizo = Get_Torizo(k as i32, game);
    ((*E).toriz_parameter_1 as i32 ^ (*E).base.x_pos as i32 - ram_word!(game, samus_x_pos) as i32)
        as u16
}
pub unsafe fn Torizo_Func_11(k: u16, game: &mut Game) {
    let E: *mut Enemy_Torizo = Get_Torizo(k as i32, game);
    SetGoldenTorizoPalette((*E).base.health, game);
}
pub unsafe fn GoldTorizo_Hurt(game: &mut Game) {
    Torizo_C620(ram_word!(game, cur_enemy_index), game);
    if (*gEnemyData(ram_word!(game, cur_enemy_index) as i32, game)).flash_timer as i32 & 1i32 != 0 {
        let mut i: i32 = 30i32;
        while i >= 0i32 {
            let v2: i32 = i >> 1i32;
            set_ram_words! { game, 0x7fffu16;
                palette_buffer[(v2 + 160i32)], palette_buffer[(v2 + 144i32)],
            }
            i -= 2i32;
        }
    } else {
        Torizo_Func_11(ram_word!(game, cur_enemy_index), game);
    };
}
pub unsafe fn Torizo_Instr_47(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    SpawnEprojWithRoomGfx(addr_kEproj_GoldenTorizoSuperMissile as u16, 0u16, game);
    jp
}
pub unsafe fn Torizo_Instr_49(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    if (Torizo_Func_12(k, game) as i32 & 0x8000i32) == 0i32
        || CompareDistToSamus_X(k, 4u16, game) as i32 & 1i32 == 0
        || CompareDistToSamus_X(k, 0x28u16, game) as i32 & 1i32 != 0
        || ram_word!(game, samus_pose) as i32 != kPose_1D_FaceR_Morphball_Ground as i32
            && ram_word!(game, samus_pose) as i32 != kPose_1E_MoveR_Morphball_Ground as i32
            && ram_word!(game, samus_pose) as i32 != kPose_1F_MoveL_Morphball_Ground as i32
            && ram_word!(game, samus_pose) as i32 != kPose_79_FaceR_Springball_Ground as i32
            && ram_word!(game, samus_pose) as i32 != kPose_7A_FaceL_Springball_Ground as i32
            && ram_word!(game, samus_pose) as i32 != kPose_7B_MoveR_Springball_Ground as i32
            && ram_word!(game, samus_pose) as i32 != kPose_7C_MoveL_Springball_Ground as i32
    {
        return jp.offset(1);
    }
    (*Get_Torizo(k as i32, game)).toriz_var_09 = 0u16;
    *jp.offset(0) as usize as *const u16
}
pub unsafe fn Torizo_Instr_61(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    SpawnEprojWithRoomGfx(addr_kEproj_GoldenTorizoEyeBeam as u16, *jp.offset(0), game);
    jp.offset(1)
}
pub unsafe fn Torizo_Instr_53(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    if (Torizo_Func_12(k, game) as i32 & 0x8000i32) != 0i32
        || CompareDistToSamus_X(k, 0x20u16, game) as i32 & 1i32 == 0
        || CompareDistToSamus_X(k, 0x60u16, game) as i32 & 1i32 != 0
        || (NextRandom(game) as i32 & 0x110i32) != 0i32
    {
        return jp.offset(1);
    }
    (*Get_Torizo(k as i32, game)).toriz_var_00 = (jp.offset(1) as *mut u8)
        .offset_from(RomBankBase((*gEnemyData(k as i32, game)).bank, game))
        as c_long as u16;
    *jp.offset(0) as usize as *const u16
}
pub unsafe fn Torizo_Instr_55(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E: *mut Enemy_Torizo = Get_Torizo(k as i32, game);
    if (*E).base.health > 0x788u16 || (NextRandom(game) as i32 & 0x102i32) != 0i32 {
        return jp.offset(1) as usize as *const u16;
    }
    (*E).toriz_var_09 = 0u16;
    (*E).toriz_var_00 = (jp.offset(1) as *mut u8)
        .offset_from(RomBankBase((*gEnemyData(k as i32, game)).bank, game))
        as c_long as u16;
    *jp.offset(0) as usize as *const u16
}
pub unsafe fn Torizo_Instr_52(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E: *mut Enemy_Torizo = Get_Torizo(k as i32, game);
    if (*E).base.health <= 0x2a30u16 || ((*E).toriz_parameter_2 as i32 & 0x2000i32) == 0i32 {
        return jp.offset(1);
    }
    (*E).toriz_var_00 = (jp.offset(1) as *mut u8)
        .offset_from(RomBankBase((*gEnemyData(k as i32, game)).bank, game))
        as c_long as u16;
    *jp.offset(0) as usize as *const u16
}
pub unsafe fn Torizo_Instr_50(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    if CompareDistToSamus_X(k, 0x70u16, game) as i32 & 1i32 == 0 {
        return jp.offset(1);
    }
    if (Torizo_Func_12(k, game) as i32 & 0x8000i32) != 0i32 {
        return jp.offset(1);
    }
    let E: *mut Enemy_Torizo = Get_Torizo(k as i32, game);
    if (*E).toriz_var_07 <= 0x168u16
        && ((ram_word!(game, joypad1_lastkeys) as i32 & 0x300i32) == 0i32
            || (NextRandom(game) as i32 & 0x101i32) == 0i32)
    {
        return jp.offset(1);
    }
    (*E).toriz_var_09 = 0u16;
    Torizo_C20A(k, game);
    *jp.offset(0) as usize as *const u16
}
pub unsafe fn Torizo_Instr_43(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    SpawnEprojWithGfx(0u16, k, addr_kEproj_GoldenTorizosChozoOrbs as u16, game);
    jp
}
pub unsafe fn Torizo_Instr_51(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E: *mut Enemy_Torizo = Get_Torizo(k as i32, game);
    if (*E).toriz_var_09 < 8u16
        && (CompareDistToSamus_X(k, 0x20u16, game) as i32 & 1i32 != 0
            || (Torizo_Func_12(k, game) as i32 & 0x8000i32) != 0i32)
    {
        return jp.offset(1);
    }
    (*E).toriz_var_09 = 0u16;
    Torizo_C22D(k, game);
    *jp.offset(0) as usize as *const u16
}
pub unsafe fn Torizo_Instr_45(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*Get_Torizo(k as i32, game)).toriz_var_00 = (jp.offset(2) as *mut u8)
        .offset_from(RomBankBase((*gEnemyData(k as i32, game)).bank, game))
        as c_long as u16;
    if ram_word!(game, samus_missiles) < 0x20u16
        || (ram_word!(game, nmi_frame_counter_word) as i32
            + (ram_word!(game, samus_x_pos) as i32 & 1i32)
            + (ram_word!(game, samus_x_pos) as i32 >> 1i32)
            & 8i32)
            != 0i32
    {
        *jp.offset(0) as usize as *const u16
    } else {
        *jp.offset(1) as usize as *const u16
    }
}
pub unsafe fn Torizo_Instr_54(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E: *mut Enemy_Torizo = Get_Torizo(k as i32, game);
    (*E).toriz_var_A = g_word_AAD59A[(*jp.offset(0) as i32 >> 1i32) as usize] as u16;
    if Enemy_MoveRight_IgnoreSlopes(k, ((*E).toriz_var_A as i16 as i32) << 16i32, game) != 0 {
        (*E).toriz_var_03 = 0u16;
        if ((*E).toriz_parameter_1 as i32 & 0x8000i32) != 0i32 {
            addr_kTorizo_Ilist_D203 as i32 as usize as *const u16
        } else {
            addr_kTorizo_Ilist_D2BF as i32 as usize as *const u16
        }
    } else {
        EnemyFunc_C8AD(k, game);
        if (((*E).toriz_parameter_1 as i32
            ^ ram_word!(game, samus_x_pos) as i32 - (*E).base.x_pos as i32)
            & 0x8000i32)
            == 0i32
        {
            if (*E).toriz_var_03 == 0 {
                (*E).toriz_var_03 = 16u16;
            }
            jp.offset(1)
        } else {
            jp.offset(1)
        }
    }
}
pub unsafe fn Torizo_Func_9(k: u16, game: &mut Game) {
    let E: *mut Enemy_Torizo = Get_Torizo(ram_word!(game, cur_enemy_index) as i32, game);
    if ram_word!(game, samus_y_pos) > 0x140u16 && ram_word!(game, samus_x_pos) > 0x170u16 {
        (*E).base.current_instruction = (*E).base.current_instruction.wrapping_add(1);
        enemy_instruction! {[(*E).base]; instruction = (*E).base.current_instruction.wrapping_add(1), timer = 1u16}
    }
}
pub unsafe fn Torizo_Func_10(k: u16, game: &mut Game) {
    Torizo_C620(k, game);
    Torizo_C643(k, game);
}
pub unsafe fn Torizo_D5E6(k: u16, game: &mut Game) {
    Torizo_C620(k, game);
    let E: *mut Enemy_Torizo = Get_Torizo(k as i32, game);
    CallEnemyPreInstr((((*E).toriz_var_F) as u32) | (0xaa0000u32), game);
}
pub unsafe fn Torizo_D5ED(k: u16, game: &mut Game) {
    Torizo_C643(k, game);
}
pub unsafe fn Torizo_D5F1(k: u16, game: &mut Game) {
    let mut v3: u16 = 0;
    let E: *mut Enemy_Torizo = Get_Torizo(k as i32, game);
    let torizo_var_03: u16 = (*E).toriz_var_03;
    if torizo_var_03 == 0 || {
        v3 = (torizo_var_03).wrapping_sub(1u16);
        (*E).toriz_var_03 = v3;
        (*E).toriz_var_03 != 0u16
    } {
        let mut v5: u16 = (abs16((*E).toriz_var_A)).wrapping_add(1u16);
        if v5 >= 0x10u16 {
            v5 = 15u16;
        }
        if Enemy_MoveDown(k, (v5 as i16 as i32) << 16i32, game) == 0 {
            let mut v6: u16 = 0;
            if ((*E).toriz_parameter_1 as i32 & 0x8000i32) != 0i32 {
                v6 = addr_off_AAC0F2 as u16;
            } else {
                v6 = addr_off_AABC78 as u16;
            }
            enemy_instruction! {[(*E).base]; instruction = v6, timer = 1u16}
            (*E).toriz_var_B = 256u16;
            (*E).toriz_var_A = 0u16;
        }
    } else {
        (*E).toriz_var_03 = 0u16;
        let mut v4: u16 = 0;
        if ((*E).toriz_parameter_1 as i32 & 0x8000i32) != 0i32 {
            v4 = addr_kTorizo_Ilist_D203 as u16;
        } else {
            v4 = addr_kTorizo_Ilist_D2BF as u16;
        }
        enemy_instruction! {[(*E).base]; instruction = v4, timer = 1u16}
    };
}
pub unsafe fn Torizo_D658(game: &mut Game) {
    let E: *mut Enemy_Torizo = Get_Torizo(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).base.flash_timer == 0 && (*E).toriz_var_04 == 0 {
        Torizo_D6A6(game);
    }
}
pub unsafe fn GoldTorizo_Shot(game: &mut Game) {
    let E: *mut Enemy_Torizo = Get_Torizo(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).base.flash_timer == 0 {
        let mut v2: u16 = 0;
        let mut v3: u16 = 0;

        if (*E).toriz_var_04 != 0 {
            Torizo_D6A6(game);
            return;
        }
        let react_to_shot: bool = (((*E).toriz_parameter_2 as i32 & 0x1000i32) != 0i32)
            || ({
                v2 = 0;
                v3 = 0;
                v2 = (2u16).wrapping_mul(ram_word!(game, collision_detection_index));
                v3 = (ram_word!(
                    game,
                    projectile_type[ram_word!(game, collision_detection_index)]
                )) & (0xf00u16);
                (*E).toriz_var_05 = v3;
                if v3 == 256u16 {
                    Torizo_D6D1(ram_word!(game, cur_enemy_index), v2, game);
                    return;
                }
                (v3 != 512u16)
                    || ({
                        Torizo_D6F7(ram_word!(game, cur_enemy_index), v2, game);
                        false
                    })
            });
        if react_to_shot {
            (*E).toriz_parameter_2 = ((*E).toriz_parameter_2) | (0x2000u16);
            Torizo_D6A6(game);
        }
    }
}
pub unsafe fn Torizo_D6A6(game: &mut Game) {
    NormalEnemyShotAiSkipDeathAnim_CurEnemy(game);
    let E: *mut Enemy_Torizo = Get_Torizo(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).base.health == 0 {
        enemy_instruction! {[(*E).base]; instruction = addr_kTorizo_Ilist_B1C8 as u16, timer = 1u16}
        (*E).toriz_parameter_2 = ((*E).toriz_parameter_2) | (0xc000u16);
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
    }
}
pub unsafe fn Torizo_D6D1(k: u16, j: u16, game: &mut Game) {
    update_ram_word!(
        game.projectile_dir[(j as i32 >> 1i32)],
        |fresh1| (fresh1 as i32 & !(0x10i32)) as u16
    );
    let E: *mut Enemy_Torizo = Get_Torizo(k as i32, game);
    (*E).toriz_var_E = ((0xaac6abi32) as u16);
    (*E).base.instruction_timer = 1u16;
    let mut v3: u16 = 0;
    if ((*E).toriz_parameter_1 as i32 & 0x8000i32) != 0i32 {
        v3 = addr_kTorizo_Ilist_D2AD as u16;
    } else {
        v3 = addr_kTorizo_Ilist_D1F1 as u16;
    }
    (*E).base.current_instruction = v3;
}
pub unsafe fn Torizo_D6F7(k: u16, j: u16, game: &mut Game) {
    let E: *mut Enemy_Torizo = Get_Torizo(ram_word!(game, cur_enemy_index) as i32, game);
    if (Torizo_Func_12(k, game) as i32 & 0x8000i32) == 0i32 {
        (*E).toriz_parameter_2 = ((*E).toriz_parameter_2) | (0x1000u16);
        (*E).toriz_var_E = ((0xaac6abi32) as u16);
        update_ram_word!(game.projectile_dir[(j as i32 >> 1i32)], |fresh0| (fresh0)
            | (0x10u16));
        (*E).base.instruction_timer = 1u16;
        let mut v3: u16 = 0;
        if ((*E).toriz_parameter_1 as i32 & 0x2000i32) != 0i32 {
            if ((*E).toriz_parameter_1 as i32 & 0x8000i32) != 0i32 {
                v3 = addr_kTorizo_Ilist_CEFF as u16;
            } else {
                v3 = addr_kTorizo_Ilist_CE43 as u16;
            }
        } else if ((*E).toriz_parameter_1 as i32 & 0x8000i32) != 0i32 {
            v3 = addr_kTorizo_Ilist_CEA5 as u16;
        } else {
            v3 = addr_kTorizo_Ilist_CDE1 as u16;
        }
        (*E).base.current_instruction = v3;
    } else {
        Torizo_D6A6(game);
    };
}
pub unsafe fn TourianEntranceStatue_Init(game: &mut Game) {
    let v0: *mut EnemyData = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
    (*v0).palette_index = 0u16;
    (*v0).instruction_timer = 1u16;
    (*v0).timer = 0u16;
    let v1: u16 =
        crate::rtl::rom_table_word!(game, 0xaad810, ((*v0).parameter_1 as i32 >> 1i32) as isize);
    (*v0).current_instruction = v1;
    if (*v0).parameter_1 == 0 {
        SpawnEprojWithRoomGfx(addr_kEproj_TourianStatueBaseDecoration as u16, 0u16, game);
        SpawnEprojWithRoomGfx(addr_kEproj_TourianStatueRidley as u16, 0u16, game);
        SpawnEprojWithRoomGfx(addr_kEproj_TourianStatuePhantoon as u16, 0u16, game);
    }
    let mut i: i32 = 30i32;
    while i >= 0i32 {
        let v5: i32 = i >> 1i32;
        set_ram_word!(
            game.target_palettes[(v5 + 240i32)] =
                crate::rtl::rom_table_word!(game, 0xaad785, v5 as isize)
        );
        set_ram_word!(
            game.target_palettes[(v5 + 160i32)] =
                crate::rtl::rom_table_word!(game, 0xaad765, v5 as isize)
        );
        i -= 2i32;
    }
}
pub unsafe fn Shaktool_Instr_2(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E: *mut Enemy_Shaktool = Get_Shaktool(k as i32, game);
    let EX: *mut Enemy_Shaktool = Get_Shaktool((*E).shakt_var_E as i32 + 192i32, game);
    Shaktool_D956(k, jp, ((*EX).shakt_var_D) ^ (0x80u16), game)
}
pub unsafe fn Shaktool_Instr_3(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E: *mut Enemy_Shaktool = Get_Shaktool(k as i32, game);
    let EX: *mut Enemy_Shaktool = Get_Shaktool((*E).shakt_var_E as i32 + 192i32, game);
    Shaktool_D956(k, jp, (*EX).shakt_var_D, game)
}
pub unsafe fn Shaktool_Instr_4(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E: *mut Enemy_Shaktool = Get_Shaktool(k as i32, game);
    Shaktool_D956(
        k,
        jp,
        (*((&raw mut (*E).shakt_var_A as *mut u8).offset(1) as *mut u16)) ^ (0x80u16),
        game,
    )
}
pub unsafe fn Shaktool_Instr_5(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E: *mut Enemy_Shaktool = Get_Shaktool(k as i32, game);
    Shaktool_D956(
        k,
        jp,
        *((&raw mut (*E).shakt_var_A as *mut u8).offset(1) as *mut u16),
        game,
    )
}
pub unsafe fn Shaktool_D956(k: u16, jp: *const u16, a: u16, game: &mut Game) -> *const u16 {
    let E: *mut Enemy_Shaktool = Get_Shaktool(k as i32, game);
    let v3: i32 = a as u8 as i32;
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        ((kSinCosTable8bit_Sext[(v3 + 64i32) as usize] as i32) << 8i32) as u32,
    );
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        ((kSinCosTable8bit_Sext[v3 as usize] as i32) << 8i32) as u32,
    );
    jp
}
pub fn Shaktool_Instr_6(k: u16, jp: *const u16) -> *const u16 {
    jp
}
pub unsafe fn Shaktool_Func_2(k: u16, game: &mut Game) {
    SpawnEprojWithGfx(0u16, k, addr_kEproj_ShaktoolAttackFrontCircle as u16, game);
    SpawnEprojWithGfx(0u16, k, addr_kEproj_ShaktoolAttackMiddleCircle as u16, game);
    SpawnEprojWithGfx(0u16, k, addr_kEproj_ShaktoolAttackBackCircle as u16, game);
}
pub unsafe fn Shaktool_Instr_1(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let shakt_var_E: u16 = (*Get_Shaktool(k as i32, game)).shakt_var_E;
    (*Get_Shaktool(shakt_var_E as i32, game)).shakt_var_F =
        crate::rtl::rom_table_word!(game, 0xaadedb, 0);
    (*Get_Shaktool(shakt_var_E as i32 + 64i32, game)).shakt_var_F =
        crate::rtl::rom_table_word!(game, 0xaadedb, 1);
    (*Get_Shaktool(shakt_var_E as i32 + 128i32, game)).shakt_var_F =
        crate::rtl::rom_table_word!(game, 0xaadedb, 2);
    (*Get_Shaktool(shakt_var_E as i32 + 192i32, game)).shakt_var_F =
        crate::rtl::rom_table_word!(game, 0xaadedb, 3);
    (*Get_Shaktool(shakt_var_E as i32 + 256i32, game)).shakt_var_F =
        crate::rtl::rom_table_word!(game, 0xaadedb, 4);
    (*Get_Shaktool(shakt_var_E as i32 + 320i32, game)).shakt_var_F =
        crate::rtl::rom_table_word!(game, 0xaadedb, 5);
    (*Get_Shaktool(shakt_var_E as i32 + 384i32, game)).shakt_var_F =
        crate::rtl::rom_table_word!(game, 0xaadedb, 6);
    jp
}
pub unsafe fn Shaktool_DAE5(mut k: u16, game: &mut Game) {
    if (NextRandom(game) as i32 & 0x8431i32) == 0i32 {
        let mut i: i32 = 12i32;
        while i >= 0i32 {
            let E: *mut Enemy_Shaktool = Get_Shaktool(k as i32, game);
            (*E).shakt_var_F = ((0xaadcaai32) as u16);
            enemy_instruction! {[(*E).base]; instruction = crate::rtl::rom_table_word!(game, 0xaadf21, (i >> 1i32) as isize), timer = 1u16}
            k = (k).wrapping_sub(64u16);
            i -= 2i32;
        }
    }
}
pub unsafe fn Shaktool_DB0E(k: u16, a: u16, game: &mut Game) {
    let shakt_var_E: u16 = (*Get_Shaktool(k as i32, game)).shakt_var_E;
    (*Get_Shaktool(shakt_var_E as i32, game)).shakt_parameter_1 = a;
    (*Get_Shaktool(shakt_var_E as i32 + 64i32, game)).shakt_parameter_1 = a;
    (*Get_Shaktool(shakt_var_E as i32 + 128i32, game)).shakt_parameter_1 = a;
    (*Get_Shaktool(shakt_var_E as i32 + 192i32, game)).shakt_parameter_1 = a;
    (*Get_Shaktool(shakt_var_E as i32 + 256i32, game)).shakt_parameter_1 = a;
    (*Get_Shaktool(shakt_var_E as i32 + 320i32, game)).shakt_parameter_1 = a;
    (*Get_Shaktool(shakt_var_E as i32 + 384i32, game)).shakt_parameter_1 = a;
}
pub unsafe fn Shaktool_DB27(k: u16, a: u16, game: &mut Game) {
    let shakto_var_E: u16 = (*Get_Shaktool(k as i32, game)).shakt_var_E;
    (*Get_Shaktool(shakto_var_E as i32, game)).shakt_var_A = a;
    (*Get_Shaktool(shakto_var_E as i32 + 64i32, game)).shakt_var_A = a;
    (*Get_Shaktool(shakto_var_E as i32 + 128i32, game)).shakt_var_A = a;
    (*Get_Shaktool(shakto_var_E as i32 + 192i32, game)).shakt_var_A = a;
    (*Get_Shaktool(shakto_var_E as i32 + 256i32, game)).shakt_var_A = a;
    (*Get_Shaktool(shakto_var_E as i32 + 320i32, game)).shakt_var_A = a;
    (*Get_Shaktool(shakto_var_E as i32 + 384i32, game)).shakt_var_A = a;
}
pub unsafe fn Shaktool_DB40(k: u16, a: u16, game: &mut Game) {
    let shakto_var_E: u16 = (*Get_Shaktool(k as i32, game)).shakt_var_E;
    (*Get_Shaktool(shakto_var_E as i32, game)).shakt_var_B = a;
    (*Get_Shaktool(shakto_var_E as i32 + 64i32, game)).shakt_var_B = a;
    (*Get_Shaktool(shakto_var_E as i32 + 128i32, game)).shakt_var_B = a;
    (*Get_Shaktool(shakto_var_E as i32 + 192i32, game)).shakt_var_B = a;
    (*Get_Shaktool(shakto_var_E as i32 + 256i32, game)).shakt_var_B = a;
    (*Get_Shaktool(shakto_var_E as i32 + 320i32, game)).shakt_var_B = a;
    (*Get_Shaktool(shakto_var_E as i32 + 384i32, game)).shakt_var_B = a;
}
pub unsafe fn Shaktool_DB59(k: u16, game: &mut Game) {
    let mut shakto_var_E: u16 = 0;
    let mut shakto_var_B: u16 = 0;
    let mut v7: u16 = 0;
    let mut v10: u16 = 0;
    let mut x_subpos: u16 = 0;
    let mut y_subpos: u16 = 0;
    let mut x_pos: u16 = 0;
    let mut y_pos: u16 = 0;
    shakto_var_E = (*Get_Shaktool(k as i32, game)).shakt_var_E;
    let Shaktool: *mut Enemy_Shaktool = Get_Shaktool(shakto_var_E as i32 + 192i32, game);
    (*Shaktool).shakt_var_D = ((*Shaktool).shakt_var_D) ^ (0x8000u16);
    let v3: *mut Enemy_Shaktool = Get_Shaktool(shakto_var_E as i32 + 384i32, game);
    shakto_var_B = (*v3).shakt_var_B;
    let v5: *mut Enemy_Shaktool = Get_Shaktool(shakto_var_E as i32 + 64i32, game);
    (*v3).shakt_var_B = (((*v5).shakt_var_B as i32 ^ 0x8000i32) & 0xff00i32) as u16;
    (*v5).shakt_var_B = ((shakto_var_B as i32 ^ 0x8000i32) & 0xff00i32) as u16;
    let v6: *mut Enemy_Shaktool = Get_Shaktool(shakto_var_E as i32 + 320i32, game);
    v7 = (*v6).shakt_var_B;
    let v8: *mut Enemy_Shaktool = Get_Shaktool(shakto_var_E as i32 + 128i32, game);
    (*v6).shakt_var_B = (((*v8).shakt_var_B as i32 ^ 0x8000i32) & 0xff00i32) as u16;
    (*v8).shakt_var_B = ((v7 as i32 ^ 0x8000i32) & 0xff00i32) as u16;
    let v9: *mut Enemy_Shaktool = Get_Shaktool(shakto_var_E as i32 + 256i32, game);
    v10 = (*v9).shakt_var_B;
    let v11: *mut Enemy_Shaktool = Get_Shaktool(shakto_var_E as i32 + 192i32, game);
    (*v9).shakt_var_B = (((*v11).shakt_var_B as i32 ^ 0x8000i32) & 0xff00i32) as u16;
    (*v11).shakt_var_B = ((v10 as i32 ^ 0x8000i32) & 0xff00i32) as u16;
    let v12: *mut Enemy_Shaktool = Get_Shaktool(shakto_var_E as i32 + 384i32, game);
    x_subpos = (*v12).base.x_subpos;
    let v14: *mut Enemy_Shaktool = Get_Shaktool(shakto_var_E as i32, game);
    (*v12).base.x_subpos = (*v14).base.x_subpos;
    (*v14).base.x_subpos = x_subpos;
    y_subpos = (*v12).base.y_subpos;
    (*v12).base.y_subpos = (*v14).base.y_subpos;
    (*v14).base.y_subpos = y_subpos;
    x_pos = (*v12).base.x_pos;
    (*v12).base.x_pos = (*v14).base.x_pos;
    (*v14).base.x_pos = x_pos;
    y_pos = (*v12).base.y_pos;
    (*v12).base.y_pos = (*v14).base.y_pos;
    (*v14).base.y_pos = y_pos;
    (*Get_Shaktool(shakto_var_E as i32 + 64i32, game))
        .base
        .x_subpos = 0x8000u16;
    (*Get_Shaktool(shakto_var_E as i32 + 128i32, game))
        .base
        .x_subpos = 0x8000u16;
    (*Get_Shaktool(shakto_var_E as i32 + 192i32, game))
        .base
        .x_subpos = 0x8000u16;
    (*Get_Shaktool(shakto_var_E as i32 + 256i32, game))
        .base
        .x_subpos = 0x8000u16;
    (*Get_Shaktool(shakto_var_E as i32 + 320i32, game))
        .base
        .x_subpos = 0x8000u16;
    (*Get_Shaktool(shakto_var_E as i32 + 64i32, game))
        .base
        .y_subpos = 0x8000u16;
    (*Get_Shaktool(shakto_var_E as i32 + 128i32, game))
        .base
        .y_subpos = 0x8000u16;
    (*Get_Shaktool(shakto_var_E as i32 + 192i32, game))
        .base
        .y_subpos = 0x8000u16;
    (*Get_Shaktool(shakto_var_E as i32 + 256i32, game))
        .base
        .y_subpos = 0x8000u16;
    (*Get_Shaktool(shakto_var_E as i32 + 320i32, game))
        .base
        .y_subpos = 0x8000u16;
}
pub unsafe fn Shaktool_DC07(k: u16, game: &mut Game) {
    let E: *mut Enemy_Shaktool = Get_Shaktool(k as i32, game);
    let mut v2: u16 = 0;
    if ((*E).shakt_parameter_1 as i32 & 0x8000i32) != 0i32 {
        v2 = ((*E).shakt_var_B).wrapping_sub((*E).shakt_var_A);
    } else {
        v2 = ((*E).shakt_var_A).wrapping_sub((*E).shakt_var_B);
    }
    (*E).shakt_var_C = (4u16).wrapping_mul((*(&raw mut v2 as *mut u8).offset(1isize)) as u16);
}
pub unsafe fn Shaktool_DC2A(k: u16, game: &mut Game) {
    let E: *mut Enemy_Shaktool = Get_Shaktool(k as i32, game);
    let v3: i32 = *(&raw mut (*E).shakt_var_B as *mut u8).offset(1isize) as i32;
    let v5: i32 = k as i32 >> 1i32;
    SetHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        (((crate::rtl::rom_table_word!(game, 0xaae0bd, v3 as isize) as i16 as i32) << 8i32) as u32)
            .wrapping_add(
                (ram_word!(game, enemy_drawing_queue[(v5 + 91i32)]) as u32) << 16i32
                    | ram_word!(game, enemy_drawing_queue[(v5 + 92i32)]) as u32,
            ),
    );
    SetHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        (((crate::rtl::rom_table_word!(game, 0xaae03d, v3 as isize) as i16 as i32) << 8i32) as u32)
            .wrapping_add(
                (ram_word!(game, enemy_drawing_queue[(v5 + 93i32)]) as u32) << 16i32
                    | ram_word!(game, enemy_drawing_queue[(v5 + 94i32)]) as u32,
            ),
    );
}
pub unsafe fn Shaktool_DC6F(k: u16, game: &mut Game) {
    let E: *mut Enemy_Shaktool = Get_Shaktool(k as i32, game);
    Shaktool_DB40(k, (*E).shakt_var_A, game);
    let shakto_var_E: u16 = (*E).shakt_var_E;
    (*Get_Shaktool(shakto_var_E as i32, game)).shakt_var_C = g_word_AADEE9[0i32 as usize];
    (*Get_Shaktool(shakto_var_E as i32 + 64i32, game)).shakt_var_C = g_word_AADEE9[1i32 as usize];
    (*Get_Shaktool(shakto_var_E as i32 + 128i32, game)).shakt_var_C = g_word_AADEE9[2i32 as usize];
    (*Get_Shaktool(shakto_var_E as i32 + 192i32, game)).shakt_var_C = g_word_AADEE9[3i32 as usize];
    (*Get_Shaktool(shakto_var_E as i32 + 256i32, game)).shakt_var_C = g_word_AADEE9[4i32 as usize];
    (*Get_Shaktool(shakto_var_E as i32 + 320i32, game)).shakt_var_C = g_word_AADEE9[5i32 as usize];
    (*Get_Shaktool(shakto_var_E as i32 + 384i32, game)).shakt_var_C = g_word_AADEE9[6i32 as usize];
}
pub unsafe fn Shaktool_Hurt(game: &mut Game) {
    let E: *mut Enemy_Shaktool = Get_Shaktool(ram_word!(game, cur_enemy_index) as i32, game);
    CallEnemyPreInstr((((*E).shakt_var_F) as u32) | (0xaa0000u32), game);
}
pub unsafe fn Shaktool_DCAC(k: u16, game: &mut Game) {
    Shaktool_DC2A(k, game);
    let E: *mut Enemy_Shaktool = Get_Shaktool(k as i32, game);
    let mut shakto_var_C: u16 = 0;
    if ((*E).shakt_parameter_1 as i32 & 0x4000i32) != 0i32 {
        (*E).shakt_var_A = ((*E).shakt_var_A).wrapping_add(256u16);
        shakto_var_C = 256u16;
    } else {
        shakto_var_C = (*E).shakt_var_C;
    }
    (*E).shakt_var_B = ((*E).shakt_var_B as i32
        + if (*E).shakt_parameter_1 as i32 & 0x8000i32 != 0 {
            -(shakto_var_C as i32)
        } else {
            shakto_var_C as i32
        }) as u16;
}
pub unsafe fn Shaktool_DCD7(k: u16, game: &mut Game) {
    Shaktool_DCAC(k, game);
    let E: *mut Enemy_Shaktool = Get_Shaktool(k as i32, game);
    let r18: u16 = ((*E).shakt_var_B) ^ (0x8000u16);
    let mut v2: i16 = (r18 as i32
        + ((((*Get_Shaktool(k as i32 + 64i32, game)).shakt_var_B).wrapping_sub(r18)) as i32
            >> 1i32)) as i16;
    if ((*E).shakt_var_D as i32 & 0x8000i32) != 0i32 {
        let ref mut fresh2 = *(&raw mut v2 as *mut u8).offset(1isize);
        *fresh2 = (*fresh2) ^ (0x80u8);
    }
    let v3: u16 = (*(&raw mut v2 as *mut u8).offset(1isize) as i32 + 8i32 & 0xe0i32) as u16;
    *(&raw mut (*E).shakt_var_D as *mut u8).offset(0) = v3 as u8;
    enemy_instruction! {[(*E).base]; instruction = crate::rtl::rom_table_word!(
        game,
        0xaadd15,
        ((v3 as i32 >> 4i32) as u16 as i32 >> 1i32) as isize
    ), timer = 1u16}
}
pub unsafe fn Shaktool_DD25(mut k: u16, game: &mut Game) {
    let mut v7: i16 = 0;
    let E: *mut Enemy_Shaktool = Get_Shaktool(k as i32, game);
    let y_pos: u16 = (*E).base.y_pos;
    let x_pos: u16 = (*E).base.x_pos;
    Shaktool_DCAC(k, game);
    let v12: u16 = (*E).base.y_pos;
    let v11: u16 = (*E).base.x_pos;
    (*E).base.x_pos = x_pos;
    (*E).base.y_pos = y_pos;
    if Enemy_MoveRight_IgnoreSlopes(
        k,
        ((((v11) as i16).wrapping_sub((x_pos) as i16)) as i32) << 16i32,
        game,
    ) != 0u8
        || {
            (*E).base.y_pos = y_pos;
            Enemy_MoveDown(
                k,
                ((((v12) as i16).wrapping_sub((y_pos) as i16)) as i32) << 16i32,
                game,
            ) != 0u8
        }
    {
        if ((*E).shakt_parameter_1 as i32 & 0x2000i32) != 0i32 {
            Shaktool_DB0E(
                k,
                (((*E).shakt_parameter_1 as i32 ^ 0x8000i32) & 0x8fffi32) as u16,
                game,
            );
        } else {
            (*E).base.x_pos = x_pos;
            (*E).base.y_pos = y_pos;
            Shaktool_DB59(k, game);
            Shaktool_DB0E(
                ram_word!(game, cur_enemy_index),
                ((*E).shakt_parameter_1) | (0x2000u16),
                game,
            );
            Shaktool_DB0E(
                ram_word!(game, cur_enemy_index),
                ((*E).shakt_parameter_1) & (0xbfffu16),
                game,
            );
        }
        (*E).shakt_var_D = 0u16;
        let v5: u16 = CalculateAngleOfEnemyXfromEnemyY(k, (*E).shakt_var_E, game);
        v7 = ((v5 as i32) << 8i32) as i16;
        let mut v8: u16 = 0;
        if ((*E).shakt_parameter_1 as i32 & 0x8000i32) != 0i32 {
            v8 = ((v7) as u16).wrapping_sub(0x4000u16);
        } else {
            v8 = ((v7) as u16).wrapping_add(0x4000u16);
        }
        Shaktool_DB27(k, v8, game);
        let mut i: i32 = 12i32;
        while i >= 0i32 {
            Shaktool_DC07(k, game);
            let EK: *mut Enemy_Shaktool = Get_Shaktool(k as i32, game);
            (*EK).shakt_var_F = ((0xaadcaai32) as u16);
            enemy_instruction! {[(*EK).base]; instruction = crate::rtl::rom_table_word!(game, 0xaadf13, (i >> 1i32) as isize), timer = 1u16}
            k = (k).wrapping_sub(64u16);
            i -= 2i32;
        }
    } else {
        (*E).base.x_pos = v11;
        (*E).base.y_pos = v12;
        if ((*E).shakt_parameter_1 as i32 & 0x4000i32) != 0i32 {
            (*E).shakt_var_A = ((*E).shakt_var_A).wrapping_add(256u16);
        } else {
            if (((*E).shakt_var_A as i32 ^ (*E).shakt_var_B as i32) & 0xff00i32) == 0i32 {
                Shaktool_DC6F(k, game);
                (*E).shakt_var_D = 30720u16;
                Shaktool_DB0E(k, ((*E).shakt_parameter_1) & (0xdfffu16), game);
                (*E).shakt_var_D = ((*(&raw mut (*E).shakt_var_D as *mut u8).offset(1isize) as i32)
                    << 8i32) as u16;
            }
            let v2: u16 = ((*E).shakt_var_D).wrapping_add((*E).shakt_var_C);
            (*E).shakt_var_D = v2;
            if v2 >= 0xf000u16 {
                Shaktool_DB0E(k, ((*E).shakt_parameter_1) | (0x4000u16), game);
            }
        }
    };
}
pub unsafe fn Shaktool_Init(game: &mut Game) {
    let E: *mut Enemy_Shaktool = Get_Shaktool(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).base.instruction_timer = 1u16;
    (*E).base.timer = 0u16;
    (*E).shakt_var_A = 0u16;
    (*E).shakt_var_D = 0u16;
    let shakto_parameter_2: u16 = (*E).shakt_parameter_2;
    let v3: i32 = shakto_parameter_2 as i32 >> 1i32;
    (*E).base.properties =
        ((*E).base.properties) | (crate::rtl::rom_table_word!(game, 0xaade95, v3 as isize));
    (*E).shakt_var_E = (ram_word!(game, cur_enemy_index))
        .wrapping_sub(crate::rtl::rom_table_word!(game, 0xaadea3, v3 as isize));
    (*E).shakt_var_F = *((game.g_rom.offset(crate::rtl::lorom_offset(0xaadedb)) as *mut u16
        as *mut u8)
        .offset(shakto_parameter_2 as i32 as isize) as *mut u16);
    (*E).shakt_var_C = (*((&raw const g_word_AADEE9 as *const u16 as *mut u8)
        .offset(shakto_parameter_2 as i32 as isize) as *mut u16))
        .wrapping_sub(crate::rtl::rom_table_word!(game, 0xaadef7, v3 as isize));
    (*E).shakt_var_B = crate::rtl::rom_table_word!(game, 0xaadeb1, v3 as isize);
    (*E).base.current_instruction =
        crate::rtl::rom_table_word!(game, 0xaadeb1, (v3 + 7i32) as isize);
    (*E).base.layer = crate::rtl::rom_table_word!(game, 0xaadecd, v3 as isize);
    if shakto_parameter_2 != 0 {
        Shaktool_DC2A(ram_word!(game, cur_enemy_index), game);
    }
}
pub unsafe fn Shaktool_Touch(game: &mut Game) {
    NormalEnemyTouchAi(game);
}
pub unsafe fn Shaktool_Shot(game: &mut Game) {
    NormalEnemyShotAi(game);
    let E: *mut Enemy_Shaktool = Get_Shaktool(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).base.health == 0 {
        let shakto_var_E: u16 = (*E).shakt_var_E;
        (*Get_Shaktool(shakto_var_E as i32, game)).base.properties = 512u16;
        (*Get_Shaktool(shakto_var_E as i32 + 64i32, game))
            .base
            .properties = 512u16;
        (*Get_Shaktool(shakto_var_E as i32 + 128i32, game))
            .base
            .properties = 512u16;
        (*Get_Shaktool(shakto_var_E as i32 + 192i32, game))
            .base
            .properties = 512u16;
        (*Get_Shaktool(shakto_var_E as i32 + 256i32, game))
            .base
            .properties = 512u16;
        (*Get_Shaktool(shakto_var_E as i32 + 320i32, game))
            .base
            .properties = 512u16;
        (*Get_Shaktool(shakto_var_E as i32 + 384i32, game))
            .base
            .properties = 512u16;
    }
}
pub unsafe fn Shaktool_Instr_9(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    set_ram_word!(game.fx_timer = 32u16);
    set_ram_word!(game.fx_y_vel = 64u16);
    jp
}
pub unsafe fn Shaktool_Instr_11(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    set_ram_word!(game.fx_base_y_pos = 722u16);
    jp
}
pub unsafe fn Shaktool_Instr_10(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    CallSomeSamusCode(1u16, game);
    jp
}
pub unsafe fn sub_AAE445(k: u16, game: &mut Game) {
    let v1: *mut EnemyData = gEnemyData(k as i32, game);
    if (*v1).parameter_1 != 0 {
        enemy_instruction! {[(*v1)]; instruction = addr_kShaktool_Ilist_E3A7 as u16, timer = 1u16}
    }
}
pub unsafe fn Shaktool_Instr_8(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    QueueSfx2_Max6(0x1cu16, game);
    jp
}
pub unsafe fn Shaktool_Instr_13(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    QueueSfx2_Max6(0x4bu16, game);
    jp
}
pub unsafe fn Shaktool_Instr_12(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let e: *mut Enemy_Shaktool = Get_Shaktool(k as i32, game);
    let arg: u16 = *jp.offset(0);
    CalculateBlockContainingPixelPos(
        ((*e).base.x_pos).wrapping_add(arg),
        ((*e).base.y_pos).wrapping_add(28u16),
        game,
    );
    if (*(ram_ptr!(game, u16; 0x10002)).offset(
        (ram_word!(game, room_width_in_blocks) as i32 + ram_word!(game, cur_block_index) as i32)
            as isize,
    ) as i32
        & 0xf000i32)
        == 0xa000i32
    {
        SpawnPLM(addr_kPlmHeader_D113 as u16, game);
        SpawnEprojWithRoomGfx(
            addr_kEproj_WreckedShipChozoSpikeFootsteps_1 as u16,
            arg,
            game,
        );
    }
    jp.offset(1)
}
pub unsafe fn Shaktool_Instr_7(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E: *mut Enemy_Shaktool = Get_Shaktool(k as i32, game);
    (*E).shakt_var_C = *jp.offset(0);
    Enemy_MoveRight_IgnoreSlopes(
        k,
        (crate::rtl::rom_table_word!(game, 0xaae630, ((*E).shakt_var_C as i32 >> 1i32) as isize)
            as i16 as i32)
            << 8i32,
        game,
    );
    Enemy_MoveDown(
        k,
        (abs16(crate::rtl::rom_table_word!(
            game,
            0xaae630,
            ((*E).shakt_var_C as i32 >> 1i32) as isize
        )) as i16 as i32)
            << 8i32,
        game,
    );
    EnemyFunc_C8AD(k, game);
    let v6: i32 = (*E).shakt_var_C as i32 >> 1i32;
    set_ram_word!(
        game.samus_x_pos = (crate::rtl::rom_table_word!(game, 0xaae670, v6 as isize))
            .wrapping_add((*E).base.x_pos)
    );
    set_ram_word!(
        game.samus_y_pos = (crate::rtl::rom_table_word!(game, 0xaae6b0, v6 as isize))
            .wrapping_add((*E).base.y_pos)
    );
    jp.offset(1)
}
pub unsafe fn Shaktool_Instr_14(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    CallSomeSamusCode(1u16, game);
    *(ram_ptr!(game, u16; 0xcd20, 6)) = 0u16;
    *(ram_ptr!(game, u16; 0xcd20, 8)) = 0u16;
    *(ram_ptr!(game, u16; 0xcd20, 9)) = 0u16;
    *(ram_ptr!(game, u16; 0xcd20, 13)) = 1u16;
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs::new(0x17u8, 0x1du8, 0xd6fc as VoidP),
        game,
    );
    jp
}
pub unsafe fn N00bTubeCracks_Init(game: &mut Game) {
    let mut i: i32 = 62i32;
    while i >= 0i32 {
        set_ram_word!(
            game.target_palettes[((i >> 1i32) + 144i32)] =
                crate::rtl::rom_table_word!(game, 0xaae2dd, (i >> 1i32) as isize)
        );
        i -= 2i32;
    }
}
pub unsafe fn ChozoStatue_Init(game: &mut Game) {
    let v0: *mut EnemyData = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
    (*v0).properties = ((*v0).properties as i32
        | (kEnemyProps_DisableSamusColl as i32 | kEnemyProps_ProcessedOffscreen as i32 | 0x8000i32))
        as u16;
    (*v0).spritemap_pointer = addr_kSpritemap_Nothing_AA as u16;
    (*v0).instruction_timer = 1u16;
    (*v0).timer = 0u16;
    (*v0).ai_preinstr = ((0xaae7a6i32) as u16);
    (*v0).parameter_1 = 0u16;
    (*v0).palette_index = 0u16;
    (*gEnemyData(0i32, game)).layer = 0u16;
    let parameter_2: u16 = (*v0).parameter_2;
    (*v0).current_instruction =
        crate::rtl::rom_table_word!(game, 0xaae7a2, (parameter_2 as i32 >> 1i32) as isize);
    if parameter_2 != 0 {
        sub_AAE784(game);
    } else {
        let mut i: i32 = 30i32;
        while i >= 0i32 {
            let v3: i32 = i >> 1i32;
            set_ram_word!(
                game.target_palettes[(v3 + 160i32)] = kChozoStatue_Palettes[v3 as usize] as u16
            );
            set_ram_word!(game.target_palettes[(v3 + 144i32)] = kChozoStatue_Palette[v3 as usize]);
            i -= 2i32;
        }
        SpawnHardcodedPlm(
            SpawnHardcodedPlmArgs::new(0x4au8, 0x17u8, 0xd6ee as VoidP),
            game,
        );
        SpawnHardcodedPlm(
            SpawnHardcodedPlmArgs::new(0x17u8, 0x1du8, 0xd6fc as VoidP),
            game,
        );
    };
}
pub unsafe fn sub_AAE784(game: &mut Game) {
    let mut i: i32 = 30i32;
    while i >= 0i32 {
        let v1: i32 = i >> 1i32;
        set_ram_word!(
            game.target_palettes[(v1 + 160i32)] = kChozoStatue_Palettes3[v1 as usize] as u16
        );
        set_ram_word!(
            game.target_palettes[(v1 + 144i32)] = kChozoStatue_Palettes2[v1 as usize] as u16
        );
        i -= 2i32;
    }
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs::new(0xcu8, 0x1du8, 0xd6d6 as VoidP),
        game,
    );
}
pub unsafe fn ChozoStatue_Main(game: &mut Game) {
    let v0: *mut EnemyData = gEnemyData(ram_word!(game, cur_enemy_index) as i32, game);
    CallEnemyPreInstr((((*v0).ai_preinstr) as u32) | (0xaa0000u32), game);
}
pub unsafe fn Shaktool_PreInstr_0(k: u16, game: &mut Game) {
    if (*(ram_ptr!(game, u16; 0xd828, ram_word!(game, area_index) as isize)) as i32 & 1i32) != 0i32
    {
        let E: *mut Enemy_Shaktool = Get_Shaktool(k as i32, game);
        if (*E).shakt_parameter_1 != 0 {
            enemy_instruction! {[(*E).base]; instruction = addr_kShaktool_Ilist_E461 as u16, timer = 1u16}
            (*E).shakt_var_A = -256i32 as u16;
            (*E).shakt_var_B = 256u16;
        }
    }
}
