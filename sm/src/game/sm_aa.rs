pub use crate::game::constants::bank_84::*;
pub use crate::game::constants::bank_86::*;
pub use crate::game::constants::bank_8d::*;
pub use crate::game::constants::enemy_properties::*;
pub use crate::game::constants::samus_poses::*;
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
use crate::rtl::abs16;
use crate::rtl::{AddToHiLo, SetHiLo};
use crate::rtl::{RomPtr, Unreachable};

pub use crate::types::Consts_84;
pub use crate::types::Consts_86;
pub use crate::types::Consts_8D;
pub use crate::types::Consts_AA;
pub use crate::types::EnemyProps;
pub use crate::types::{
    __int16_t, __int32_t, __uint16_t, __uint32_t, __uint8_t, int16, int16_t, int32, int32_t,
    uint16, uint16_t, uint32, uint32_t, uint8, uint8_t, uintptr_t, EnemyBase, EnemyData, SamusPose,
    VoidP,
};
pub const addr_kShaktool_Ilist_E461: Consts_AA = 58465;
pub const addr_kShaktool_Ilist_E3A7: Consts_AA = 58279;
pub const addr_kTorizo_Ilist_D2BF: Consts_AA = 53951;
pub const addr_kTorizo_Ilist_D2AD: Consts_AA = 53933;
pub const addr_kTorizo_Ilist_D203: Consts_AA = 53763;
pub const addr_kTorizo_Ilist_D1F1: Consts_AA = 53745;
pub const addr_kTorizo_Ilist_CEFF: Consts_AA = 52991;
pub const addr_kTorizo_Ilist_CEA5: Consts_AA = 52901;
pub const addr_kTorizo_Ilist_CE43: Consts_AA = 52803;
pub const addr_kTorizo_Ilist_CDE1: Consts_AA = 52705;
pub const addr_locret_AAC95E: Consts_AA = 51550;
pub const addr_kTorizo_Ilist_C188: Consts_AA = 49544;
pub const addr_off_AAC0F2: Consts_AA = 49394;
pub const addr_kTorizo_Ilist_BDD8: Consts_AA = 48600;
pub const addr_kTorizo_Ilist_BD0E: Consts_AA = 48398;
pub const addr_off_AABC78: Consts_AA = 48248;
pub const addr_kTorizo_Ilist_B962: Consts_AA = 47458;
pub const addr_kTorizo_Ilist_B1C8: Consts_AA = 45512;
pub const addr_kTorizo_Ilist_B155: Consts_AA = 45397;
pub const addr_kTorizo_Ilist_B0E5: Consts_AA = 45285;
pub const addr_kTorizo_ExtSprmap_87D0: Consts_AA = 34768;
pub const addr_kSpritemap_Nothing_AA: Consts_AA = 32845;
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
        pad1: [uint8; 26696],
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
unsafe fn RomBankBase(mut bank: uint8, game: &mut Game) -> *const uint8 {
    RomPtr(
        (((bank as ::core::ffi::c_int) << 16 as ::core::ffi::c_int) + 0x8000 as ::core::ffi::c_int)
            as uint32_t,
        game,
    )
    .offset(-(0x8000 as ::core::ffi::c_int as isize))
}
pub const fnnullsub_171_AA: ::core::ffi::c_int = 0xaa807b;
pub const fnnullsub_270: ::core::ffi::c_int = 0xaac6ab;
pub const fnTorizo_Func_3: ::core::ffi::c_int = 0xaac6bf;
pub const fnTorizo_Func_4: uint32 = 11192006;
pub const fnTorizo_Func_1: uint32 = 11192063;
pub const fnTorizo_Func_9: uint32 = 11195842;
pub const fnTorizo_Func_10: uint32 = 11195871;
pub const fnTorizo_D5E6: uint32 = 11195878;
pub const fnnullsub_274: ::core::ffi::c_int = 0xaadcaa;
pub const fnnullsub_276: ::core::ffi::c_int = 0xaae7a6;
#[inline]
unsafe fn gEnemyData(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut EnemyData {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut EnemyData
}
#[inline]
unsafe fn Get_Shaktool(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_Shaktool {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_Shaktool
}
#[inline]
unsafe fn Get_Torizo(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_Torizo {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_Torizo
}
static g_word_AAB096: uint16 = 6 as uint16;
static g_word_AAB098: uint16 = 5 as uint16;
static g_word_AAB09A: uint16 = 3 as uint16;
static g_word_AAC3EE: [int16; 16] = [
    -(9),
    -(6),
    -(7),
    5,
    -(16),
    -(7),
    0,
    0,
    9,
    6,
    7,
    -(5),
    16,
    7,
    0,
    0,
];
static g_word_AAC40E: [int16; 8] = [0, -(6), -(6), -(7), 0, 0, 0, 0];
static g_word_AAC440: [int16; 16] = [
    -(9),
    -(6),
    -(7),
    5,
    -(16),
    -(7),
    0,
    0,
    9,
    6,
    7,
    -(5),
    16,
    7,
    0,
    0,
];
static g_word_AAC460: [int16; 8] = [0, -(6), -(6), -(7), 0, 0, 0, 0];
static g_word_AAC4BD: [int16; 20] = [
    -(5),
    0,
    -(5),
    -(19),
    -(16),
    -(7),
    0,
    -(7),
    -(17),
    -(18),
    5,
    0,
    5,
    19,
    16,
    7,
    0,
    7,
    17,
    18,
];
static g_word_AAC532: [int16; 20] = [
    -(5),
    0,
    -(5),
    -(0x13),
    -(0x10),
    -(7),
    0,
    -(7),
    -(0x11),
    -(0x12),
    5,
    0,
    5,
    0x13,
    0x10,
    7,
    0,
    7,
    0x11,
    0x12,
];
static g_word_AAC95F: [int16; 2] = [0xdb, 0x1a8];
static g_word_AAC963: [int16; 2] = [0xb3, 0x90];
static g_word_AAC96B: [int16; 2] = [0x2800, 0x2800];
static g_word_AAC96F: [int16; 2] = [0x12, 0x12];
static g_word_AAC973: [int16; 2] = [0x30, 0x29];
static kTorizo_Palette: [uint16; 16] = [
    0x3800, 0x3ff, 0x33b, 0x216, 0x113, 0x6b1e, 0x4a16, 0x3591, 0x20e9, 0x1580, 0x1580, 0x1580,
    0x1580, 0x1580, 0x1580, 0x1580,
];
static kTorizo_Palettes_1: [uint16; 16] = [
    0x3800, 0x2df, 0x1d7, 0xac, 0x5a73, 0x41ad, 0x2d08, 0x1863, 0x1486, 0x145, 0x145, 0x145,
    0x7fff, 0x145, 0x145, 0,
];
static kTorizo_Palettes_2: [uint16; 16] = [
    0x3800, 0x679f, 0x5299, 0x252e, 0x14aa, 0x5efc, 0x4657, 0x35b2, 0x2d70, 0x5b7f, 0x3df8, 0x2d0e,
    0x5f5f, 0x5e1a, 0x5d35, 0xc63,
];
static kTorizo_Palettes_3: [uint16; 16] = [
    0x3800, 0x4aba, 0x35b2, 0x847, 3, 0x4215, 0x2970, 0x18cb, 0x1089, 0x463a, 0x28b3, 0x1809,
    0x6f7f, 0x51fd, 0x4113, 0xc63,
];
static kTorizo_Palettes_4: [uint16; 16] = [
    0x3800, 0x56ba, 0x41b2, 0x1447, 0x403, 0x4e15, 0x3570, 0x24cb, 0x1868, 0x6f7f, 0x51f8, 0x410e,
    0x31f, 0x1da, 0xf5, 0xc63,
];
static kTorizo_Palettes_5: [uint16; 16] = [
    0x3800, 0x4215, 0x2d0d, 2, 0, 0x3970, 0x20cb, 0xc26, 0x403, 0x463a, 0x28b3, 0x1809, 0x6f7f,
    0x51fd, 0x4113, 0xc63,
];
static kTorizo_Palettes_6: [uint16; 16] = [
    0x3800, 0x6ab5, 0x49b0, 0x1c45, 0xc01, 0x5613, 0x416d, 0x2cc9, 0x2066, 0x5714, 0x31cc, 0x14e3,
    0x5630, 0x3569, 0x1883, 0xc66,
];
static kTorizo_Palettes_7: [uint16; 16] = [
    0x3800, 0x5610, 0x350b, 0x800, 0, 0x416e, 0x2cc8, 0x1823, 0xc01, 0x6a31, 0x4caa, 0x2406,
    0x7f7b, 0x75f4, 0x4d10, 0xc63,
];
static kTorizo_Palettes_8: [uint16; 16] = [
    0x3800, 0x4bbe, 0x6b9, 0xa8, 0, 0x173a, 0x276, 0x1f2, 0x14d, 0x73e0, 0x4f20, 0x2a20, 0x7fe0,
    0x5aa0, 0x5920, 0x43,
];
static kTorizo_Palettes_10: [uint16; 16] = [
    0x3800, 0x3719, 0x214, 3, 0, 0x295, 0x1d1, 0x14d, 0xa8, 0x4b40, 0x25e0, 0xe0, 0x6b40, 0x4600,
    0x4480, 0,
];
static g_word_AAD59A: [int16; 20] = [
    -(5),
    0,
    -(5),
    -(19),
    -(16),
    -(7),
    0,
    -(7),
    -(17),
    -(18),
    5,
    0,
    5,
    19,
    16,
    7,
    0,
    7,
    17,
    18,
];
static g_word_AADEE9: [uint16; 7] = [0, 0x20, 0x60, 0xc0, 0x140, 0x1a0, 0x1e0];
static kChozoStatue_Palette: [uint16; 16] = [
    0x3800, 0x633f, 0x4a9f, 0x2ddf, 0x6739, 0x4e73, 0x318c, 0x18c6, 0x27ff, 0x1af7, 0xdce, 0xc6,
    0x3fff, 0x2b39, 0x7fff, 0,
];
static kChozoStatue_Palettes: [int16; 16] = [
    0x3800, 0x633f, 0x4a9f, 0x2ddf, 0x4210, 0x318c, 0x2108, 0x1084, 0x27ff, 0x1af7, 0xdce, 0xc6,
    0x3fff, 0x2b39, 0x5294, 0,
];
static kChozoStatue_Palettes2: [int16; 16] = [
    0x3800, 0x633f, 0x4a9f, 0x2ddf, 0x2f7c, 0x2295, 0x118d, 0x8e8, 0x27ff, 0x1af7, 0xdce, 0xc6,
    0x3fff, 0x2b39, 0x73df, 0x43,
];
static kChozoStatue_Palettes3: [int16; 16] = [
    0x3800, 0x633f, 0x4a9f, 0x2ddf, 0x2295, 0x118d, 0x8e8, 0x85, 0x27ff, 0x1af7, 0xdce, 0xc6,
    0x3fff, 0x2b39, 0x5294, 1,
];
pub unsafe fn Enemy_GrappleReact_CancelBeam_AA(game: &mut Game) {
    Enemy_SwitchToFrozenAi(game);
}
pub unsafe fn Enemy_NormalFrozenAI_AA(game: &mut Game) {
    NormalEnemyFrozenAI(game);
}
pub unsafe fn Enemy_ClearAiPreInstr_AA(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*gEnemyData(k as ::core::ffi::c_int, game)).ai_preinstr = ((0xaa807bi32) as u16) & (0xffffu16);
    jp
}
pub unsafe fn Enemy_SetAiPreInstr_AA(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*gEnemyData(k as ::core::ffi::c_int, game)).ai_preinstr = *jp.offset(0);
    jp.offset(1)
}
pub unsafe fn Torizo_Instr_3(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_Torizo(k as ::core::ffi::c_int, game)).toriz_var_E = *jp.offset(0);
    jp.offset(1)
}
pub unsafe fn Torizo_Instr_31(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int, game);
    (*E).toriz_parameter_2 = ((*E).toriz_parameter_2) | (0x8000u16);
    let mut parameter =
        SpawnEprojWithGfx((*E).toriz_parameter_2, k, addr_stru_86A95B as uint16, game);
    parameter = SpawnEprojWithGfx(parameter, k, addr_stru_86A95B as uint16, game);
    parameter = SpawnEprojWithGfx(parameter, k, addr_stru_86A95B as uint16, game);
    parameter = SpawnEprojWithGfx(parameter, k, addr_stru_86A95B as uint16, game);
    parameter = SpawnEprojWithGfx(parameter, k, addr_stru_86A95B as uint16, game);
    SpawnEprojWithGfx(parameter, k, addr_stru_86A95B as uint16, game);
    jp
}
pub unsafe fn Torizo_Instr_33(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int, game);
    (*E).toriz_parameter_2 = ((*E).toriz_parameter_2) | (0x4000u16);
    jp
}
pub unsafe fn Torizo_Instr_36(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int, game);
    (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
        & !(kEnemyProps_Invisible as ::core::ffi::c_int)) as uint16;
    jp
}
pub unsafe fn Torizo_Instr_37(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int, game);
    (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Invisible) as u16);
    jp
}
pub unsafe fn Torizo_Instr_35(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut i: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
    while i >= 0i32 {
        let mut v3: ::core::ffi::c_int = i >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16)
            .offset((v3 + 160 as ::core::ffi::c_int) as isize) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16)
            .offset((v3 + 144 as ::core::ffi::c_int) as isize) = 0 as uint16;
        i -= 2 as ::core::ffi::c_int;
    }
    jp
}
pub unsafe fn Torizo_Instr_38(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    SetBossBitForCurArea(4 as uint16, game);
    QueueMusic_Delayed8(g_word_AAB09A, game);
    if *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) != 0 {
        Enemy_ItemDrop_GoldenTorizo(k, game);
    } else {
        Enemy_ItemDrop_BombTorizo(k, game);
    }
    jp
}
pub unsafe fn Torizo_Instr_6(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    sub_82DAF7(0x600 as uint16, game);
    jp
}
pub unsafe fn Torizo_Instr_5(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    Torizo_C268(game);
    jp
}
pub unsafe fn Torizo_Instr_9(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    QueueMusic_Delayed8(g_word_AAB098, game);
    SpawnPalfxObject(addr_stru_8DF759 as uint16, game);
    jp
}
pub unsafe fn Torizo_C20A(mut k: uint16, game: &mut Game) {
    let mut v2: int16 = 0;
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int, game);
    if ((*E).toriz_parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        v2 = 512 as int16;
    } else {
        v2 = -(512 as ::core::ffi::c_int) as int16;
    }
    (*E).toriz_var_A = v2 as uint16;
    (*E).toriz_var_B = -(1472 as ::core::ffi::c_int) as uint16;
    (*E).toriz_var_C = 40 as uint16;
    (*E).base.instruction_timer = 1 as uint16;
}
pub unsafe fn Torizo_C22D(mut k: uint16, game: &mut Game) {
    let mut v2: int16 = 0;
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int, game);
    if ((*E).toriz_parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        v2 = -(768 as ::core::ffi::c_int) as int16;
    } else {
        v2 = 768 as int16;
    }
    (*E).toriz_var_A = v2 as uint16;
    (*E).toriz_var_B = -(1152 as ::core::ffi::c_int) as uint16;
    (*E).toriz_var_C = 40 as uint16;
    (*E).base.instruction_timer = 1 as uint16;
}
pub unsafe fn Torizo_C250(game: &mut Game) {
    let mut i: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
    while i >= 0i32 {
        let mut v1: ::core::ffi::c_int = i >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16)
            .offset((v1 + 160 as ::core::ffi::c_int) as isize) = kTorizo_Palettes_3[v1 as usize];
        *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16)
            .offset((v1 + 144 as ::core::ffi::c_int) as isize) = kTorizo_Palettes_2[v1 as usize];
        i -= 2 as ::core::ffi::c_int;
    }
}
pub unsafe fn Torizo_C268(game: &mut Game) {
    let mut i: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
    while i >= 0i32 {
        let mut v1: ::core::ffi::c_int = i >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16)
            .offset((v1 + 160 as ::core::ffi::c_int) as isize) = kTorizo_Palettes_5[v1 as usize];
        *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16)
            .offset((v1 + 144 as ::core::ffi::c_int) as isize) = kTorizo_Palettes_4[v1 as usize];
        i -= 2 as ::core::ffi::c_int;
    }
}
pub unsafe fn Torizo_C280(game: &mut Game) {
    let mut i: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
    while i >= 0i32 {
        let mut v1: ::core::ffi::c_int = i >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16)
            .offset((v1 + 160 as ::core::ffi::c_int) as isize) = kTorizo_Palettes_7[v1 as usize];
        *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16)
            .offset((v1 + 144 as ::core::ffi::c_int) as isize) = kTorizo_Palettes_6[v1 as usize];
        i -= 2 as ::core::ffi::c_int;
    }
}
pub unsafe fn Torizo_C298(game: &mut Game) {
    let mut i: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
    while i >= 0i32 {
        let mut v1: ::core::ffi::c_int = i >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16)
            .offset((v1 + 160 as ::core::ffi::c_int) as isize) = kTorizo_Palettes_10[v1 as usize];
        *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16)
            .offset((v1 + 144 as ::core::ffi::c_int) as isize) = kTorizo_Palettes_8[v1 as usize];
        i -= 2 as ::core::ffi::c_int;
    }
}
pub unsafe fn Torizo_C2B0(game: &mut Game) {
    let mut i: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
    while i >= 0i32 {
        let mut v1: ::core::ffi::c_int = i >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
            .offset((v1 + 160 as ::core::ffi::c_int) as isize) = kTorizo_Palettes_5[v1 as usize];
        *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
            .offset((v1 + 144 as ::core::ffi::c_int) as isize) = kTorizo_Palettes_4[v1 as usize];
        i -= 2 as ::core::ffi::c_int;
    }
}
pub fn Torizo_Instr_7(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    jp
}
pub unsafe fn Torizo_Instr_2(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_Torizo(k as ::core::ffi::c_int, game)).toriz_var_04 = 30583 as uint16;
    jp
}
pub unsafe fn Torizo_Instr_8(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_Torizo(k as ::core::ffi::c_int, game)).toriz_var_04 = 0 as uint16;
    jp
}
pub unsafe fn Torizo_Instr_25(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    if ((*Get_Torizo(k as ::core::ffi::c_int, game)).toriz_parameter_2 as ::core::ffi::c_int
        & 0x4000 as ::core::ffi::c_int)
        != 0i32
    {
        return *jp.offset(0) as uintptr_t as *const uint16;
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) != 0 {
        *jp.offset(1) as uintptr_t as *const uint16
    } else {
        jp.offset(2)
    }
}
pub unsafe fn Torizo_Instr_22(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_Torizo(k as ::core::ffi::c_int, game)).toriz_var_00 = *jp.offset(0);
    jp.offset(1)
}
pub unsafe fn Torizo_Instr_19(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_Torizo(k as ::core::ffi::c_int, game)).toriz_var_00 as uintptr_t as *const uint16
}
pub unsafe fn Torizo_Instr_32(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_Torizo(k as ::core::ffi::c_int, game)).toriz_var_01 as uintptr_t as *const uint16
}
pub unsafe fn Torizo_Instr_30(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut a: uint16 = *jp.offset(0);
    let mut i: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
    while i >= 0i32 {
        SpawnEprojWithRoomGfx(addr_kEproj_BombTorizoLowHealthExplode as uint16, a, game);
        i -= 1;
    }
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int, game);
    (*E).base.current_instruction = (jp.offset(1) as *mut uint8).offset_from(RomBankBase(
        (*gEnemyData(k as ::core::ffi::c_int, game)).bank,
        game,
    )) as ::core::ffi::c_long as uint16;
    (*E).base.flash_timer = 40 as uint16;
    (*E).base.instruction_timer = 40 as uint16;
    ::core::ptr::null::<uint16>()
}
pub unsafe fn Torizo_Instr_34(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    // The instruction dispatcher leaves this handler address in A.
    SpawnEprojWithRoomGfx(addr_kEproj_BombTorizoDeathExplosion as uint16, 0xc32f, game);
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int, game);
    (*E).base.current_instruction = (jp as *mut uint8).offset_from(RomBankBase(
        (*gEnemyData(k as ::core::ffi::c_int, game)).bank,
        game,
    )) as ::core::ffi::c_long as uint16;
    (*E).base.flash_timer = 1 as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    ::core::ptr::null::<uint16>()
}
pub unsafe fn Torizo_Instr_24(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let left_foot_parameter = SpawnEprojWithRoomGfx(
        addr_kEproj_TourianLandingDustCloudsRightFoot as uint16,
        0xc34a,
        game,
    );
    SpawnEprojWithRoomGfx(addr_stru_86AFF3 as uint16, left_foot_parameter, game);
    jp
}
pub unsafe fn Torizo_Instr_12(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut health: uint16 = (*Get_Torizo(k as ::core::ffi::c_int, game)).base.health;
    if health < 0x15eu16 {
        SpawnEprojWithGfx(
            health,
            k,
            addr_kEproj_BombTorizoLowHealthInitialDrool as uint16,
            game,
        );
    }
    jp
}
pub unsafe fn Torizo_Instr_10(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int, game);
    (*E).toriz_parameter_1 = ((*E).toriz_parameter_1) | (0x4000u16);
    jp
}
pub unsafe fn Torizo_Instr_11(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int, game);
    (*E).toriz_parameter_1 = ((*E).toriz_parameter_1) & (0x1fffu16);
    (*E).toriz_var_09 = (*E).toriz_var_09.wrapping_add(1);
    jp
}
pub unsafe fn Torizo_Instr_29(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int, game);
    (*E).toriz_parameter_1 = ((*E).toriz_parameter_1 as ::core::ffi::c_int
        & 0x1fff as ::core::ffi::c_int
        | 0x8000 as ::core::ffi::c_int) as uint16;
    (*E).toriz_var_09 = (*E).toriz_var_09.wrapping_add(1);
    jp
}
pub unsafe fn Torizo_Instr_1(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int, game);
    (*E).toriz_parameter_1 = ((*E).toriz_parameter_1 as ::core::ffi::c_int
        & 0x1fff as ::core::ffi::c_int
        | 0x2000 as ::core::ffi::c_int) as uint16;
    (*E).toriz_var_09 = (*E).toriz_var_09.wrapping_add(1);
    jp
}
pub unsafe fn Torizo_Instr_28(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int, game);
    (*E).toriz_parameter_1 = ((*E).toriz_parameter_1 as ::core::ffi::c_int
        & 0x1fff as ::core::ffi::c_int
        | 0xa000 as ::core::ffi::c_int) as uint16;
    (*E).toriz_var_09 = (*E).toriz_var_09.wrapping_add(1);
    jp
}
pub unsafe fn Torizo_Instr_4(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut v2: uint16 = *jp.offset(0);
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int, game);
    (*E).base.x_pos = ((*E).base.x_pos).wrapping_add(
        (g_word_AAC3EE[(v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize]) as u16,
    );
    (*E).base.y_pos = ((*E).base.y_pos).wrapping_add(
        (g_word_AAC40E[((v2 as ::core::ffi::c_int & 0xf as ::core::ffi::c_int)
            >> 1 as ::core::ffi::c_int) as usize]) as u16,
    );
    jp.offset(1)
}
pub unsafe fn Torizo_Instr_40(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut v2: uint16 = *jp.offset(0);
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int, game);
    (*E).base.x_pos = ((*E).base.x_pos).wrapping_sub(
        (g_word_AAC440[(v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize]) as u16,
    );
    (*E).base.y_pos = ((*E).base.y_pos).wrapping_sub(
        (g_word_AAC460[((v2 as ::core::ffi::c_int & 0xf as ::core::ffi::c_int)
            >> 1 as ::core::ffi::c_int) as usize]) as u16,
    );
    jp.offset(1)
}
pub unsafe fn Torizo_Instr_16(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int, game);
    (*E).toriz_var_A = g_word_AAC4BD
        [(*jp.offset(0) as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize]
        as uint16;
    if Enemy_MoveRight_IgnoreSlopes(
        k,
        ((*E).toriz_var_A as int16 as int32) << 16 as ::core::ffi::c_int,
        game,
    ) != 0
    {
        (*E).toriz_var_03 = 0 as uint16;
        if ((*E).toriz_parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
            addr_kTorizo_Ilist_B962 as ::core::ffi::c_int as uintptr_t as *const uint16
        } else {
            addr_kTorizo_Ilist_BDD8 as ::core::ffi::c_int as uintptr_t as *const uint16
        }
    } else {
        EnemyFunc_C8AD(k, game);
        if (((*E).toriz_parameter_1 as ::core::ffi::c_int
            ^ *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
                as ::core::ffi::c_int
                - (*E).base.x_pos as ::core::ffi::c_int)
            & 0x8000 as ::core::ffi::c_int)
            == 0i32
        {
            if (*E).toriz_var_03 == 0 {
                (*E).toriz_var_03 = 72 as uint16;
            }
            jp.offset(1)
        } else {
            jp.offset(1)
        }
    }
}
pub unsafe fn Torizo_Instr_27(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int, game);
    (*E).toriz_var_A = g_word_AAC532
        [(*jp.offset(0) as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize]
        as uint16;
    if Enemy_MoveRight_IgnoreSlopes(
        k,
        ((*E).toriz_var_A as int16 as int32) << 16 as ::core::ffi::c_int,
        game,
    ) != 0
    {
        (*E).toriz_var_03 = 0 as uint16;
        if ((*E).toriz_parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
            addr_kTorizo_Ilist_BD0E as ::core::ffi::c_int as uintptr_t as *const uint16
        } else {
            addr_kTorizo_Ilist_C188 as ::core::ffi::c_int as uintptr_t as *const uint16
        }
    } else {
        EnemyFunc_C8AD(k, game);
        if (((*E).toriz_parameter_1 as ::core::ffi::c_int
            ^ *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
                as ::core::ffi::c_int
                - (*E).base.x_pos as ::core::ffi::c_int)
            & 0x8000 as ::core::ffi::c_int)
            == 0i32
        {
            if (*E).toriz_var_03 == 0 {
                (*E).toriz_var_03 = 72 as uint16;
            }
            jp.offset(1)
        } else {
            jp.offset(1)
        }
    }
}
pub unsafe fn Torizo_Instr_23(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    if ((*Get_Torizo(k as ::core::ffi::c_int, game)).toriz_var_B as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int)
        != 0i32
    {
        *jp.offset(0) as uintptr_t as *const uint16
    } else {
        jp.offset(1)
    }
}
pub unsafe fn Torizo_Instr_14(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    if CompareDistToSamus_X(k, 0x38 as uint16, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int
        != 0
    {
        return jp.offset(1);
    }
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int, game);
    if (((*E).toriz_parameter_1 as ::core::ffi::c_int
        ^ *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
            as ::core::ffi::c_int
            - (*E).base.x_pos as ::core::ffi::c_int)
        & 0x8000 as ::core::ffi::c_int)
        == 0i32
    {
        return jp.offset(1);
    }
    (*E).toriz_var_00 = (jp.offset(1) as *mut uint8).offset_from(RomBankBase(
        (*gEnemyData(k as ::core::ffi::c_int, game)).bank,
        game,
    )) as ::core::ffi::c_long as uint16;
    *jp.offset(0) as uintptr_t as *const uint16
}
pub unsafe fn Torizo_Instr_15(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    if CompareDistToSamus_X(k, 0x20 as uint16, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int
        != 0
        || (Torizo_Func_12(k, game) as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32
    {
        return jp.offset(1);
    }
    Torizo_C22D(k, game);
    *jp.offset(0) as uintptr_t as *const uint16
}
pub unsafe fn Torizo_Instr_26(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int, game);
    (*E).toriz_var_00 = (jp.offset(2) as *mut uint8).offset_from(RomBankBase(
        (*gEnemyData(k as ::core::ffi::c_int, game)).bank,
        game,
    )) as ::core::ffi::c_long as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9c6) as *mut uint16) < 5u16
        || (*((&raw mut game.g_ram as *mut uint8).offset(0x5b6) as *mut uint16)
            as ::core::ffi::c_int
            + (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
                as ::core::ffi::c_int
                & 1 as ::core::ffi::c_int)
            + (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
                as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int)
            & 8 as ::core::ffi::c_int)
            != 0i32
    {
        *jp.offset(0) as uintptr_t as *const uint16
    } else {
        *jp.offset(1) as uintptr_t as *const uint16
    }
}
pub unsafe fn Torizo_Instr_18(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    // The instruction dispatcher leaves this handler address in A.
    let mut parameter =
        SpawnEprojWithGfx(0xc5cb, k, addr_kEproj_BombTorizosChozoOrbs as uint16, game);
    parameter = SpawnEprojWithGfx(
        parameter,
        k,
        addr_kEproj_BombTorizosChozoOrbs as uint16,
        game,
    );
    SpawnEprojWithGfx(
        parameter,
        k,
        addr_kEproj_BombTorizosChozoOrbs as uint16,
        game,
    );
    jp
}
pub unsafe fn Torizo_Instr_20(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    SpawnEprojWithGfx(
        *jp.offset(0),
        k,
        addr_kEproj_BombTorizoSonicBoom as uint16,
        game,
    );
    jp.offset(1)
}
pub unsafe fn Torizo_Instr_44(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    SpawnEprojWithGfx(
        *jp.offset(0),
        k,
        addr_kEproj_GoldenTorizoSonicBoom as uint16,
        game,
    );
    jp.offset(1)
}
pub unsafe fn Torizo_Instr_21(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    SpawnEprojWithRoomGfx(
        addr_kEproj_BombTorizoExplosiveSwipe as uint16,
        *jp.offset(0),
        game,
    );
    jp.offset(1)
}
pub unsafe fn Torizo_Instr_17(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    QueueSfx2_Max6(0x27 as uint16, game);
    jp
}
pub unsafe fn Torizo_Instr_13(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    QueueSfx2_Max6(0x4b as uint16, game);
    jp
}
pub unsafe fn Torizo_C620(mut k: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) == 0
        && (*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
            as ::core::ffi::c_int
            & 0x8142 as ::core::ffi::c_int)
            == 0i32
    {
        let mut health: uint16 = (*Get_Torizo(k as ::core::ffi::c_int, game)).base.health;
        if health != 0u16 && health < 0x15eu16 {
            SpawnEprojWithGfx(
                health,
                k,
                addr_kEproj_BombTorizoLowHealthInitialDrool as uint16,
                game,
            );
        }
    }
}
pub unsafe fn Torizo_C643(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int, game);
    if Enemy_MoveDown(
        k,
        ((*E).toriz_var_B as int16 as int32) << 8 as ::core::ffi::c_int,
        game,
    ) != 0
    {
        let mut v3: int16 = (*E).toriz_var_B as int16;
        if v3 >= 0i16 && v3 != 256i16 {
            *((&raw mut game.g_ram as *mut uint8).offset(0x183e) as *mut uint16) = 4 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x1840) as *mut uint16) = 32 as uint16;
            (*E).toriz_var_B = 256 as uint16;
        }
    } else {
        (*E).toriz_var_B = ((*E).toriz_var_B).wrapping_add(40u16);
    };
}
pub unsafe fn Torizo_Hurt(game: &mut Game) {
    Torizo_C620(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
    if (*Get_Torizo(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .base
    .flash_timer as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        let mut i: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
        while i >= 0i32 {
            let mut v2: ::core::ffi::c_int = i >> 1 as ::core::ffi::c_int;
            *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                .offset((v2 + 160 as ::core::ffi::c_int) as isize) = 0x7fff as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                .offset((v2 + 144 as ::core::ffi::c_int) as isize) = 0x7fff as uint16;
            i -= 2 as ::core::ffi::c_int;
        }
    } else {
        Torizo_C2B0(game);
    };
}
pub unsafe fn CallTorizoFunc(mut ea: uint32, mut k: uint16, game: &mut Game) {
    match ea {
        11191979 => (),
        11191999 => {
            Torizo_Func_3(k, game);
        }
        11192006 => {
            Torizo_Func_4(k, game);
        }
        11192063 => {
            Torizo_Func_1(k, game);
        }
        11195842 => {
            Torizo_Func_9(k, game);
        }
        11195871 => {
            Torizo_Func_10(k, game);
        }
        11195878 => {
            Torizo_D5E6(k, game);
        }
        _ => {
            Unreachable(game);
        }
    }
}
pub unsafe fn Torizo_Main(game: &mut Game) {
    let mut E: *mut Enemy_Torizo = Get_Torizo(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    CallTorizoFunc(
        (((*E).toriz_var_E) as u32) | (0xaa0000u32),
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
}
pub unsafe fn Torizo_Func_2(mut k: uint16, game: &mut Game) {
    Torizo_C643(k, game);
    if sub_82DAF7(0x600 as uint16, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int == 0 {
        (*Get_Torizo(k as ::core::ffi::c_int, game)).toriz_var_E =
            ((0xaac6abi32) as u16) & (0xffffu16);
    }
}
pub unsafe fn Torizo_Func_3(mut k: uint16, game: &mut Game) {
    Torizo_C620(k, game);
    Torizo_C643(k, game);
}
pub unsafe fn Torizo_Func_4(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int, game);
    (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
    let mut v2: uint16 = 78 as uint16;
    while *((&raw mut game.g_ram as *mut uint8).offset(0x1c37) as *mut uint16)
        .offset((v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        != addr_kPlmHeader_D6EA as ::core::ffi::c_int
    {
        v2 = (v2).wrapping_sub(2u16);
        if (v2 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
            QueueMusic_Delayed8(g_word_AAB096, game);
            (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
                & !(kEnemyProps_Tangible as ::core::ffi::c_int))
                as uint16;
            (*E).base.current_instruction = ((*E).base.current_instruction).wrapping_add(2u16);
            (*E).base.instruction_timer = 1 as uint16;
            return;
        }
    }
}
pub unsafe fn Torizo_Func_1(mut k: uint16, game: &mut Game) {
    Torizo_C620(k, game);
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int, game);
    if ((*E).toriz_parameter_2 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32
        || (*E).base.health >= 0x15eu16
    {
        if ((*E).toriz_parameter_2 as ::core::ffi::c_int & 0x4000 as ::core::ffi::c_int) != 0i32
            || (*E).base.health >= 0x64u16
        {
            CallEnemyPreInstr((((*E).toriz_var_F) as u32) | (0xaa0000u32), game);
        } else {
            let mut v2: uint16 = 0;
            if ((*E).toriz_parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32
            {
                v2 = addr_kTorizo_Ilist_BD0E as uint16;
            } else {
                v2 = addr_kTorizo_Ilist_C188 as uint16;
            }
            (*E).toriz_var_00 = v2;
            (*E).base.current_instruction = addr_kTorizo_Ilist_B155 as uint16;
            (*E).base.instruction_timer = 1 as uint16;
        }
    } else {
        (*E).toriz_var_01 = (*E).base.current_instruction;
        (*E).base.current_instruction = addr_kTorizo_Ilist_B0E5 as uint16;
        (*E).base.instruction_timer = 1 as uint16;
    };
}
pub unsafe fn Torizo_Func_5(mut k: uint16, game: &mut Game) {
    let mut v3: uint16 = 0;
    let mut v8: uint16 = 0;
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int, game);
    if ((*E).toriz_parameter_2 as ::core::ffi::c_int & 0x4000 as ::core::ffi::c_int) != 0i32 {
        let mut torizo_var_03: uint16 = (*E).toriz_var_03;
        if torizo_var_03 == 0 || {
            v8 = (torizo_var_03).wrapping_sub(1u16);
            (*E).toriz_var_03 = v8;
            (*E).toriz_var_03 != 0u16
        } {
            let mut v10: uint16 = (abs16((*E).toriz_var_A)).wrapping_add(1u16);
            if v10 >= 0x10u16 {
                v10 = 15 as uint16;
            }
            if Enemy_MoveDown(k, (v10 as int16 as int32) << 16 as ::core::ffi::c_int, game) == 0 {
                let mut v11: uint16 = 0;
                if ((*E).toriz_parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int)
                    != 0i32
                {
                    v11 = addr_off_AAC0F2 as uint16;
                } else {
                    v11 = addr_off_AABC78 as uint16;
                }
                (*E).base.current_instruction = v11;
                (*E).base.instruction_timer = 1 as uint16;
                (*E).toriz_var_B = 256 as uint16;
                (*E).toriz_var_A = 0 as uint16;
            }
        } else {
            (*E).toriz_var_03 = 0 as uint16;
            let mut v9: uint16 = 0;
            if ((*E).toriz_parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32
            {
                v9 = addr_kTorizo_Ilist_BD0E as uint16;
            } else {
                v9 = addr_kTorizo_Ilist_C188 as uint16;
            }
            (*E).base.current_instruction = v9;
            (*E).base.instruction_timer = 1 as uint16;
        }
    } else {
        let mut v2: uint16 = (*E).toriz_var_03;
        if v2 == 0 || {
            v3 = (v2).wrapping_sub(1u16);
            (*E).toriz_var_03 = v3;
            (*E).toriz_var_03 != 0u16
        } {
            let mut v5: uint16 = (abs16((*E).toriz_var_A)).wrapping_add(1u16);
            if v5 >= 0x10u16 {
                v5 = 15 as uint16;
            }
            if Enemy_MoveDown(k, (v5 as int16 as int32) << 16 as ::core::ffi::c_int, game) == 0 {
                let mut v6: uint16 = 0;
                if ((*E).toriz_parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int)
                    != 0i32
                {
                    v6 = addr_off_AAC0F2 as uint16;
                } else {
                    v6 = addr_off_AABC78 as uint16;
                }
                (*E).base.current_instruction = v6;
                (*E).base.instruction_timer = 1 as uint16;
                (*E).toriz_var_B = 256 as uint16;
                (*E).toriz_var_A = 0 as uint16;
            }
        } else {
            (*E).toriz_var_03 = 0 as uint16;
            let mut v4: uint16 = 0;
            if ((*E).toriz_parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32
            {
                v4 = addr_kTorizo_Ilist_B962 as uint16;
            } else {
                v4 = addr_kTorizo_Ilist_BDD8 as uint16;
            }
            (*E).base.current_instruction = v4;
            (*E).base.instruction_timer = 1 as uint16;
        }
    };
}
pub unsafe fn Torizo_Func_6(mut k: uint16, game: &mut Game) {
    Torizo_C643(k, game);
}
pub unsafe fn Torizo_Func_7(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int, game);
    Enemy_MoveRight_IgnoreSlopes(
        k,
        ((*E).toriz_var_A as int16 as int32) << 8 as ::core::ffi::c_int,
        game,
    );
    EnemyFunc_C8AD(k, game);
    if Enemy_MoveDown(
        k,
        ((*E).toriz_var_B as int16 as int32) << 8 as ::core::ffi::c_int,
        game,
    ) != 0
    {
        (*E).base.current_instruction = (*E).toriz_var_00;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).toriz_var_B = 256 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x183e) as *mut uint16) = 4 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1840) as *mut uint16) = 32 as uint16;
    } else {
        (*E).toriz_var_B = ((*E).toriz_var_B).wrapping_add((*E).toriz_var_C);
    };
}
pub unsafe fn Torizo_Init(game: &mut Game) {
    let mut E: *mut Enemy_Torizo = Get_Torizo(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if CheckBossBitForCurArea(4 as uint16, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int
        != 0
    {
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Deleted) as u16);
    } else {
        let mut v2: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x79f)
            as *mut uint16) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int;
        (*E).base.properties = ((*E).base.properties) | ((g_word_AAC96B[v2 as usize]) as u16);
        (*E).base.extra_properties = ((*E).base.extra_properties) | (4u16);
        (*E).base.x_width = g_word_AAC96F[v2 as usize] as uint16;
        (*E).base.y_height = g_word_AAC973[v2 as usize] as uint16;
        (*E).toriz_var_E = ((0xaac6bfi32) as u16) & (0xffffu16);
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.timer = 0 as uint16;
        (*E).base.palette_index = 0 as uint16;
        (*E).toriz_var_F = addr_locret_AAC95E as uint16;
        (*E).base.current_instruction = *(game.g_rom.offset(crate::rtl::lorom_offset(0xaac967))
            as *const uint8 as *mut uint16)
            .offset(v2 as isize);
        (*E).base.spritemap_pointer = addr_kTorizo_ExtSprmap_87D0 as uint16;
        (*E).base.x_pos = g_word_AAC95F[v2 as usize] as uint16;
        (*E).base.y_pos = g_word_AAC963[v2 as usize] as uint16;
        (*E).toriz_var_A = 0 as uint16;
        (*E).toriz_var_B = 256 as uint16;
        let mut i: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
        while i >= 0i32 {
            let mut v4: ::core::ffi::c_int = i >> 1 as ::core::ffi::c_int;
            *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16)
                .offset((v4 + 240 as ::core::ffi::c_int) as isize) =
                kTorizo_Palettes_1[v4 as usize];
            *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16)
                .offset((v4 + 176 as ::core::ffi::c_int) as isize) = kTorizo_Palette[v4 as usize];
            i -= 2 as ::core::ffi::c_int;
        }
        if *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) != 0 {
            Torizo_C280(game);
            if *((&raw mut game.g_ram as *mut uint8).offset(0x8b) as *mut uint16) == 0xc0c0u16 {
                *((&raw mut game.g_ram as *mut uint8).offset(0x9c2) as *mut uint16) = 700 as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0x9c4) as *mut uint16) = 700 as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0x9d4) as *mut uint16) = 300 as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0x9d6) as *mut uint16) = 300 as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0x9c6) as *mut uint16) = 100 as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0x9c8) as *mut uint16) = 100 as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0x9ca) as *mut uint16) = 20 as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0x9cc) as *mut uint16) = 20 as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0x9ce) as *mut uint16) = 20 as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0x9d0) as *mut uint16) = 20 as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0x9a2) as *mut uint16) =
                    -(3273 as ::core::ffi::c_int) as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0x9a4) as *mut uint16) =
                    -(3273 as ::core::ffi::c_int) as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0x9a6) as *mut uint16) =
                    4111 as uint16;
                *((&raw mut game.g_ram as *mut uint8).offset(0x9a8) as *mut uint16) =
                    4111 as uint16;
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
    if *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) != 0 {
        GoldTorizo_Shot(game);
    } else {
        let mut E: *mut Enemy_Torizo = Get_Torizo(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        );
        if (*E).toriz_var_04 as ::core::ffi::c_int | (*E).base.flash_timer as ::core::ffi::c_int
            == 0
        {
            NormalEnemyShotAiSkipDeathAnim_CurEnemy(game);
            if (*E).base.health == 0 {
                (*E).toriz_var_E = ((0xaac6abi32) as u16) & (0xffffu16);
                (*E).base.current_instruction = addr_kTorizo_Ilist_B1C8 as uint16;
                (*E).base.instruction_timer = 1 as uint16;
                (*E).toriz_parameter_2 = ((*E).toriz_parameter_2) | (0xc000u16);
                (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
            }
        }
    };
}
pub unsafe fn Torizo_Func_8(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) != 0 {
        Torizo_D658(game);
    }
}
pub unsafe fn Torizo_Instr_39(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    if (*Get_Torizo(k as ::core::ffi::c_int, game)).base.y_pos == 375u16 {
        jp.offset(1)
    } else {
        *jp.offset(0) as uintptr_t as *const uint16
    }
}
pub unsafe fn Torizo_Instr_41(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    Torizo_C298(game);
    jp
}
pub unsafe fn Torizo_Instr_42(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    QueueMusic_Delayed8(5 as uint16, game);
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int, game);
    (*E).base.x_width = 18 as uint16;
    (*E).base.y_height = 48 as uint16;
    SpawnPalfxObject(addr_stru_8DF75D as uint16, game);
    jp
}
pub unsafe fn Torizo_Instr_48(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int, game);
    (*E).toriz_parameter_2 =
        ((*E).toriz_parameter_2 as ::core::ffi::c_int & !(0x1000 as ::core::ffi::c_int)) as uint16;
    jp
}
pub unsafe fn Torizo_Instr_57(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    SpawnEprojWithRoomGfx(addr_stru_86B1C0 as uint16, 0 as uint16, game);
    jp
}
pub unsafe fn Torizo_Instr_58(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut v2: int16 = 0;
    v2 = 34 as int16;
    while *(((&raw mut game.g_ram as *mut uint8).offset(0x1997) as *mut uint16 as *mut uint8)
        .offset(v2 as uint16 as ::core::ffi::c_int as isize) as *mut uint16)
        != 0xb1c0u16
    {
        v2 = (v2).wrapping_sub(2i16);
        if v2 < 0i16 {
            return jp.offset(1);
        }
    }
    *jp.offset(0) as uintptr_t as *const uint16
}
pub unsafe fn Torizo_Instr_59(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int, game);
    (*E).toriz_var_06 =
        ((*E).toriz_var_06 as ::core::ffi::c_int & !(0x8000 as ::core::ffi::c_int)) as uint16;
    jp
}
pub unsafe fn Torizo_Instr_62(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int, game);
    (*E).toriz_var_06 = ((*E).toriz_var_06) | (0x8000u16);
    jp
}
pub unsafe fn Torizo_Instr_63(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int, game);
    (*E).toriz_parameter_2 =
        ((*E).toriz_parameter_2 as ::core::ffi::c_int & !(0x2000 as ::core::ffi::c_int)) as uint16;
    jp
}
pub unsafe fn GoldTorizo_Main(game: &mut Game) {
    let mut E: *mut Enemy_Torizo = Get_Torizo(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16) as ::core::ffi::c_int
        == kPose_1B_FaceR_SpaceJump as ::core::ffi::c_int
        || *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16) as ::core::ffi::c_int
            == kPose_1C_FaceL_SpaceJump as ::core::ffi::c_int
    {
        (*E).toriz_var_07 = (*E).toriz_var_07.wrapping_add(1);
    } else {
        (*E).toriz_var_07 = 0 as uint16;
    }
    CallTorizoFunc(
        (((*E).toriz_var_E) as u32) | (0xaa0000u32),
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
}
pub unsafe fn Torizo_Instr_56(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    QueueSfx2_Max6(0x34 as uint16, game);
    jp
}
pub unsafe fn Torizo_Instr_60(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    QueueSfx2_Max6(0x67 as uint16, game);
    jp
}
pub unsafe fn Torizo_Instr_46(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    QueueSfx2_Max6(0x48 as uint16, game);
    jp
}
pub unsafe fn Torizo_Func_12(mut k: uint16, game: &mut Game) -> uint16 {
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int, game);
    ((*E).toriz_parameter_1 as ::core::ffi::c_int
        ^ (*E).base.x_pos as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
                as ::core::ffi::c_int) as uint16
}
pub unsafe fn Torizo_Func_11(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int, game);
    SetGoldenTorizoPalette((*E).base.health, game);
}
pub unsafe fn GoldTorizo_Hurt(game: &mut Game) {
    Torizo_C620(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
    if (*gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .flash_timer as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        let mut i: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
        while i >= 0i32 {
            let mut v2: ::core::ffi::c_int = i >> 1 as ::core::ffi::c_int;
            *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                .offset((v2 + 160 as ::core::ffi::c_int) as isize) = 0x7fff as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                .offset((v2 + 144 as ::core::ffi::c_int) as isize) = 0x7fff as uint16;
            i -= 2 as ::core::ffi::c_int;
        }
    } else {
        Torizo_Func_11(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        );
    };
}
pub unsafe fn Torizo_Instr_47(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    SpawnEprojWithRoomGfx(
        addr_kEproj_GoldenTorizoSuperMissile as uint16,
        0 as uint16,
        game,
    );
    jp
}
pub unsafe fn Torizo_Instr_49(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    if (Torizo_Func_12(k, game) as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) == 0i32
        || CompareDistToSamus_X(k, 4 as uint16, game) as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            == 0
        || CompareDistToSamus_X(k, 0x28 as uint16, game) as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            != 0
        || *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16) as ::core::ffi::c_int
            != kPose_1D_FaceR_Morphball_Ground as ::core::ffi::c_int
            && *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16)
                as ::core::ffi::c_int
                != kPose_1E_MoveR_Morphball_Ground as ::core::ffi::c_int
            && *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16)
                as ::core::ffi::c_int
                != kPose_1F_MoveL_Morphball_Ground as ::core::ffi::c_int
            && *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16)
                as ::core::ffi::c_int
                != kPose_79_FaceR_Springball_Ground as ::core::ffi::c_int
            && *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16)
                as ::core::ffi::c_int
                != kPose_7A_FaceL_Springball_Ground as ::core::ffi::c_int
            && *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16)
                as ::core::ffi::c_int
                != kPose_7B_MoveR_Springball_Ground as ::core::ffi::c_int
            && *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16)
                as ::core::ffi::c_int
                != kPose_7C_MoveL_Springball_Ground as ::core::ffi::c_int
    {
        return jp.offset(1);
    }
    (*Get_Torizo(k as ::core::ffi::c_int, game)).toriz_var_09 = 0 as uint16;
    *jp.offset(0) as uintptr_t as *const uint16
}
pub unsafe fn Torizo_Instr_61(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    SpawnEprojWithRoomGfx(
        addr_kEproj_GoldenTorizoEyeBeam as uint16,
        *jp.offset(0),
        game,
    );
    jp.offset(1)
}
pub unsafe fn Torizo_Instr_53(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    if (Torizo_Func_12(k, game) as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32
        || CompareDistToSamus_X(k, 0x20 as uint16, game) as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            == 0
        || CompareDistToSamus_X(k, 0x60 as uint16, game) as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            != 0
        || (NextRandom(game) as ::core::ffi::c_int & 0x110 as ::core::ffi::c_int) != 0i32
    {
        return jp.offset(1);
    }
    (*Get_Torizo(k as ::core::ffi::c_int, game)).toriz_var_00 =
        (jp.offset(1) as *mut uint8).offset_from(RomBankBase(
            (*gEnemyData(k as ::core::ffi::c_int, game)).bank,
            game,
        )) as ::core::ffi::c_long as uint16;
    *jp.offset(0) as uintptr_t as *const uint16
}
pub unsafe fn Torizo_Instr_55(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int, game);
    if (*E).base.health > 0x788u16
        || (NextRandom(game) as ::core::ffi::c_int & 0x102 as ::core::ffi::c_int) != 0i32
    {
        return jp.offset(1) as uintptr_t as *const uint16;
    }
    (*E).toriz_var_09 = 0 as uint16;
    (*E).toriz_var_00 = (jp.offset(1) as *mut uint8).offset_from(RomBankBase(
        (*gEnemyData(k as ::core::ffi::c_int, game)).bank,
        game,
    )) as ::core::ffi::c_long as uint16;
    *jp.offset(0) as uintptr_t as *const uint16
}
pub unsafe fn Torizo_Instr_52(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int, game);
    if (*E).base.health <= 0x2a30u16
        || ((*E).toriz_parameter_2 as ::core::ffi::c_int & 0x2000 as ::core::ffi::c_int) == 0i32
    {
        return jp.offset(1);
    }
    (*E).toriz_var_00 = (jp.offset(1) as *mut uint8).offset_from(RomBankBase(
        (*gEnemyData(k as ::core::ffi::c_int, game)).bank,
        game,
    )) as ::core::ffi::c_long as uint16;
    *jp.offset(0) as uintptr_t as *const uint16
}
pub unsafe fn Torizo_Instr_50(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    if CompareDistToSamus_X(k, 0x70 as uint16, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int
        == 0
    {
        return jp.offset(1);
    }
    if (Torizo_Func_12(k, game) as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        return jp.offset(1);
    }
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int, game);
    if (*E).toriz_var_07 <= 0x168u16
        && ((*((&raw mut game.g_ram as *mut uint8).offset(0x8b) as *mut uint16)
            as ::core::ffi::c_int
            & 0x300 as ::core::ffi::c_int)
            == 0i32
            || (NextRandom(game) as ::core::ffi::c_int & 0x101 as ::core::ffi::c_int) == 0i32)
    {
        return jp.offset(1);
    }
    (*E).toriz_var_09 = 0 as uint16;
    Torizo_C20A(k, game);
    *jp.offset(0) as uintptr_t as *const uint16
}
pub unsafe fn Torizo_Instr_43(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    SpawnEprojWithGfx(
        0 as uint16,
        k,
        addr_kEproj_GoldenTorizosChozoOrbs as uint16,
        game,
    );
    jp
}
pub unsafe fn Torizo_Instr_51(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int, game);
    if (*E).toriz_var_09 < 8u16
        && (CompareDistToSamus_X(k, 0x20 as uint16, game) as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int
            != 0
            || (Torizo_Func_12(k, game) as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int)
                != 0i32)
    {
        return jp.offset(1);
    }
    (*E).toriz_var_09 = 0 as uint16;
    Torizo_C22D(k, game);
    *jp.offset(0) as uintptr_t as *const uint16
}
pub unsafe fn Torizo_Instr_45(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_Torizo(k as ::core::ffi::c_int, game)).toriz_var_00 =
        (jp.offset(2) as *mut uint8).offset_from(RomBankBase(
            (*gEnemyData(k as ::core::ffi::c_int, game)).bank,
            game,
        )) as ::core::ffi::c_long as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x9c6) as *mut uint16) < 0x20u16
        || (*((&raw mut game.g_ram as *mut uint8).offset(0x5b6) as *mut uint16)
            as ::core::ffi::c_int
            + (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
                as ::core::ffi::c_int
                & 1 as ::core::ffi::c_int)
            + (*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
                as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int)
            & 8 as ::core::ffi::c_int)
            != 0i32
    {
        *jp.offset(0) as uintptr_t as *const uint16
    } else {
        *jp.offset(1) as uintptr_t as *const uint16
    }
}
pub unsafe fn Torizo_Instr_54(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int, game);
    (*E).toriz_var_A = g_word_AAD59A
        [(*jp.offset(0) as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize]
        as uint16;
    if Enemy_MoveRight_IgnoreSlopes(
        k,
        ((*E).toriz_var_A as int16 as int32) << 16 as ::core::ffi::c_int,
        game,
    ) != 0
    {
        (*E).toriz_var_03 = 0 as uint16;
        if ((*E).toriz_parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
            addr_kTorizo_Ilist_D203 as ::core::ffi::c_int as uintptr_t as *const uint16
        } else {
            addr_kTorizo_Ilist_D2BF as ::core::ffi::c_int as uintptr_t as *const uint16
        }
    } else {
        EnemyFunc_C8AD(k, game);
        if (((*E).toriz_parameter_1 as ::core::ffi::c_int
            ^ *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
                as ::core::ffi::c_int
                - (*E).base.x_pos as ::core::ffi::c_int)
            & 0x8000 as ::core::ffi::c_int)
            == 0i32
        {
            if (*E).toriz_var_03 == 0 {
                (*E).toriz_var_03 = 16 as uint16;
            }
            jp.offset(1)
        } else {
            jp.offset(1)
        }
    }
}
pub unsafe fn Torizo_Func_9(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Torizo = Get_Torizo(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16) > 0x140u16
        && *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) > 0x170u16
    {
        (*E).base.current_instruction = (*E).base.current_instruction.wrapping_add(1);
        (*E).base.current_instruction = (*E).base.current_instruction.wrapping_add(1);
        (*E).base.instruction_timer = 1 as uint16;
    }
}
pub unsafe fn Torizo_Func_10(mut k: uint16, game: &mut Game) {
    Torizo_C620(k, game);
    Torizo_C643(k, game);
}
pub unsafe fn Torizo_D5E6(mut k: uint16, game: &mut Game) {
    Torizo_C620(k, game);
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int, game);
    CallEnemyPreInstr((((*E).toriz_var_F) as u32) | (0xaa0000u32), game);
}
pub unsafe fn Torizo_D5ED(mut k: uint16, game: &mut Game) {
    Torizo_C643(k, game);
}
pub unsafe fn Torizo_D5F1(mut k: uint16, game: &mut Game) {
    let mut v3: uint16 = 0;
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int, game);
    let mut torizo_var_03: uint16 = (*E).toriz_var_03;
    if torizo_var_03 == 0 || {
        v3 = (torizo_var_03).wrapping_sub(1u16);
        (*E).toriz_var_03 = v3;
        (*E).toriz_var_03 != 0u16
    } {
        let mut v5: uint16 = (abs16((*E).toriz_var_A)).wrapping_add(1u16);
        if v5 >= 0x10u16 {
            v5 = 15 as uint16;
        }
        if Enemy_MoveDown(k, (v5 as int16 as int32) << 16 as ::core::ffi::c_int, game) == 0 {
            let mut v6: uint16 = 0;
            if ((*E).toriz_parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32
            {
                v6 = addr_off_AAC0F2 as uint16;
            } else {
                v6 = addr_off_AABC78 as uint16;
            }
            (*E).base.current_instruction = v6;
            (*E).base.instruction_timer = 1 as uint16;
            (*E).toriz_var_B = 256 as uint16;
            (*E).toriz_var_A = 0 as uint16;
        }
    } else {
        (*E).toriz_var_03 = 0 as uint16;
        let mut v4: uint16 = 0;
        if ((*E).toriz_parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
            v4 = addr_kTorizo_Ilist_D203 as uint16;
        } else {
            v4 = addr_kTorizo_Ilist_D2BF as uint16;
        }
        (*E).base.current_instruction = v4;
        (*E).base.instruction_timer = 1 as uint16;
    };
}
pub unsafe fn Torizo_D658(game: &mut Game) {
    let mut E: *mut Enemy_Torizo = Get_Torizo(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).base.flash_timer == 0 && (*E).toriz_var_04 == 0 {
        Torizo_D6A6(game);
    }
}
pub unsafe fn GoldTorizo_Shot(game: &mut Game) {
    let mut E: *mut Enemy_Torizo = Get_Torizo(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).base.flash_timer == 0 {
        let mut v2: uint16 = 0;
        let mut v3: uint16 = 0;
        let mut current_block_14: u64;
        if (*E).toriz_var_04 != 0 {
            Torizo_D6A6(game);
            return;
        }
        if ((*E).toriz_parameter_2 as ::core::ffi::c_int & 0x1000 as ::core::ffi::c_int) != 0i32 {
            current_block_14 = 17149359649428287120;
        } else {
            v2 = 0;
            v3 = 0;
            v2 = (2u16)
                .wrapping_mul(*((&raw mut game.g_ram as *mut uint8).offset(0x18a6) as *mut uint16));
            v3 = (*((&raw mut game.g_ram as *mut uint8).offset(0xc18) as *mut uint16).offset(
                *((&raw mut game.g_ram as *mut uint8).offset(0x18a6) as *mut uint16) as isize,
            )) & (0xf00u16);
            (*E).toriz_var_05 = v3;
            if v3 == 256u16 {
                Torizo_D6D1(
                    *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
                    v2,
                    game,
                );
                return;
            }
            if v3 != 512u16 {
                current_block_14 = 17149359649428287120;
            } else {
                Torizo_D6F7(
                    *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
                    v2,
                    game,
                );
                current_block_14 = 3512920355445576850;
            }
        }
        match current_block_14 {
            17149359649428287120 => {
                (*E).toriz_parameter_2 = ((*E).toriz_parameter_2) | (0x2000u16);
                Torizo_D6A6(game);
            }
            _ => {}
        }
    }
}
pub unsafe fn Torizo_D6A6(game: &mut Game) {
    NormalEnemyShotAiSkipDeathAnim_CurEnemy(game);
    let mut E: *mut Enemy_Torizo = Get_Torizo(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).base.health == 0 {
        (*E).base.current_instruction = addr_kTorizo_Ilist_B1C8 as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).toriz_parameter_2 = ((*E).toriz_parameter_2) | (0xc000u16);
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
    }
}
pub unsafe fn Torizo_D6D1(mut k: uint16, mut j: uint16, game: &mut Game) {
    let ref mut fresh1 = *((&raw mut game.g_ram as *mut uint8).offset(0xc04) as *mut uint16)
        .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    *fresh1 = (*fresh1 as ::core::ffi::c_int & !(0x10 as ::core::ffi::c_int)) as uint16;
    let mut E: *mut Enemy_Torizo = Get_Torizo(k as ::core::ffi::c_int, game);
    (*E).toriz_var_E = ((0xaac6abi32) as u16) & (0xffffu16);
    (*E).base.instruction_timer = 1 as uint16;
    let mut v3: uint16 = 0;
    if ((*E).toriz_parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        v3 = addr_kTorizo_Ilist_D2AD as uint16;
    } else {
        v3 = addr_kTorizo_Ilist_D1F1 as uint16;
    }
    (*E).base.current_instruction = v3;
}
pub unsafe fn Torizo_D6F7(mut k: uint16, mut j: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Torizo = Get_Torizo(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (Torizo_Func_12(k, game) as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) == 0i32 {
        (*E).toriz_parameter_2 = ((*E).toriz_parameter_2) | (0x1000u16);
        (*E).toriz_var_E = ((0xaac6abi32) as u16) & (0xffffu16);
        let ref mut fresh0 = *((&raw mut game.g_ram as *mut uint8).offset(0xc04) as *mut uint16)
            .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
        *fresh0 = (*fresh0) | (0x10u16);
        (*E).base.instruction_timer = 1 as uint16;
        let mut v3: uint16 = 0;
        if ((*E).toriz_parameter_1 as ::core::ffi::c_int & 0x2000 as ::core::ffi::c_int) != 0i32 {
            if ((*E).toriz_parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32
            {
                v3 = addr_kTorizo_Ilist_CEFF as uint16;
            } else {
                v3 = addr_kTorizo_Ilist_CE43 as uint16;
            }
        } else if ((*E).toriz_parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int)
            != 0i32
        {
            v3 = addr_kTorizo_Ilist_CEA5 as uint16;
        } else {
            v3 = addr_kTorizo_Ilist_CDE1 as uint16;
        }
        (*E).base.current_instruction = v3;
    } else {
        Torizo_D6A6(game);
    };
}
pub unsafe fn TourianEntranceStatue_Init(game: &mut Game) {
    let mut v0: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*v0).palette_index = 0 as uint16;
    (*v0).instruction_timer = 1 as uint16;
    (*v0).timer = 0 as uint16;
    let mut v1: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xaad810)) as *const uint8
        as *mut uint16)
        .offset(((*v0).parameter_1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    (*v0).current_instruction = v1;
    if (*v0).parameter_1 == 0 {
        SpawnEprojWithRoomGfx(
            addr_kEproj_TourianStatueBaseDecoration as uint16,
            0 as uint16,
            game,
        );
        SpawnEprojWithRoomGfx(addr_kEproj_TourianStatueRidley as uint16, 0 as uint16, game);
        SpawnEprojWithRoomGfx(
            addr_kEproj_TourianStatuePhantoon as uint16,
            0 as uint16,
            game,
        );
    }
    let mut i: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
    while i >= 0i32 {
        let mut v5: ::core::ffi::c_int = i >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16)
            .offset((v5 + 240 as ::core::ffi::c_int) as isize) =
            *(game.g_rom.offset(crate::rtl::lorom_offset(0xaad785)) as *const uint8 as *mut uint16)
                .offset(v5 as isize);
        *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16)
            .offset((v5 + 160 as ::core::ffi::c_int) as isize) =
            *(game.g_rom.offset(crate::rtl::lorom_offset(0xaad765)) as *const uint8 as *mut uint16)
                .offset(v5 as isize);
        i -= 2 as ::core::ffi::c_int;
    }
}
pub unsafe fn Shaktool_Instr_2(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_Shaktool = Get_Shaktool(k as ::core::ffi::c_int, game);
    let mut EX: *mut Enemy_Shaktool = Get_Shaktool(
        (*E).shakt_var_E as ::core::ffi::c_int + 192 as ::core::ffi::c_int,
        game,
    );
    Shaktool_D956(k, jp, ((*EX).shakt_var_D) ^ (0x80u16), game)
}
pub unsafe fn Shaktool_Instr_3(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_Shaktool = Get_Shaktool(k as ::core::ffi::c_int, game);
    let mut EX: *mut Enemy_Shaktool = Get_Shaktool(
        (*E).shakt_var_E as ::core::ffi::c_int + 192 as ::core::ffi::c_int,
        game,
    );
    Shaktool_D956(k, jp, (*EX).shakt_var_D, game)
}
pub unsafe fn Shaktool_Instr_4(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_Shaktool = Get_Shaktool(k as ::core::ffi::c_int, game);
    Shaktool_D956(
        k,
        jp,
        (*((&raw mut (*E).shakt_var_A as *mut uint8).offset(1) as *mut uint16)) ^ (0x80u16),
        game,
    )
}
pub unsafe fn Shaktool_Instr_5(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_Shaktool = Get_Shaktool(k as ::core::ffi::c_int, game);
    Shaktool_D956(
        k,
        jp,
        *((&raw mut (*E).shakt_var_A as *mut uint8).offset(1) as *mut uint16),
        game,
    )
}
pub unsafe fn Shaktool_D956(
    mut k: uint16,
    mut jp: *const uint16,
    mut a: uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_Shaktool = Get_Shaktool(k as ::core::ffi::c_int, game);
    let mut v3: ::core::ffi::c_int = a as uint8 as ::core::ffi::c_int;
    AddToHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        ((kSinCosTable8bit_Sext[(v3 + 64 as ::core::ffi::c_int) as usize] as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint32,
    );
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        ((kSinCosTable8bit_Sext[v3 as usize] as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)
            as uint32,
    );
    jp
}
pub fn Shaktool_Instr_6(mut k: uint16, mut jp: *const uint16) -> *const uint16 {
    jp
}
pub unsafe fn Shaktool_Func_2(mut k: uint16, game: &mut Game) {
    SpawnEprojWithGfx(
        0 as uint16,
        k,
        addr_kEproj_ShaktoolAttackFrontCircle as uint16,
        game,
    );
    SpawnEprojWithGfx(
        0 as uint16,
        k,
        addr_kEproj_ShaktoolAttackMiddleCircle as uint16,
        game,
    );
    SpawnEprojWithGfx(
        0 as uint16,
        k,
        addr_kEproj_ShaktoolAttackBackCircle as uint16,
        game,
    );
}
pub unsafe fn Shaktool_Instr_1(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut shakt_var_E: uint16 = (*Get_Shaktool(k as ::core::ffi::c_int, game)).shakt_var_E;
    (*Get_Shaktool(shakt_var_E as ::core::ffi::c_int, game)).shakt_var_F =
        *(game.g_rom.offset(crate::rtl::lorom_offset(0xaadedb)) as *const uint8 as *mut uint16)
            .offset(0);
    (*Get_Shaktool(
        shakt_var_E as ::core::ffi::c_int + 64 as ::core::ffi::c_int,
        game,
    ))
    .shakt_var_F = *(game.g_rom.offset(crate::rtl::lorom_offset(0xaadedb)) as *const uint8
        as *mut uint16)
        .offset(1);
    (*Get_Shaktool(
        shakt_var_E as ::core::ffi::c_int + 128 as ::core::ffi::c_int,
        game,
    ))
    .shakt_var_F = *(game.g_rom.offset(crate::rtl::lorom_offset(0xaadedb)) as *const uint8
        as *mut uint16)
        .offset(2);
    (*Get_Shaktool(
        shakt_var_E as ::core::ffi::c_int + 192 as ::core::ffi::c_int,
        game,
    ))
    .shakt_var_F = *(game.g_rom.offset(crate::rtl::lorom_offset(0xaadedb)) as *const uint8
        as *mut uint16)
        .offset(3);
    (*Get_Shaktool(
        shakt_var_E as ::core::ffi::c_int + 256 as ::core::ffi::c_int,
        game,
    ))
    .shakt_var_F = *(game.g_rom.offset(crate::rtl::lorom_offset(0xaadedb)) as *const uint8
        as *mut uint16)
        .offset(4);
    (*Get_Shaktool(
        shakt_var_E as ::core::ffi::c_int + 320 as ::core::ffi::c_int,
        game,
    ))
    .shakt_var_F = *(game.g_rom.offset(crate::rtl::lorom_offset(0xaadedb)) as *const uint8
        as *mut uint16)
        .offset(5);
    (*Get_Shaktool(
        shakt_var_E as ::core::ffi::c_int + 384 as ::core::ffi::c_int,
        game,
    ))
    .shakt_var_F = *(game.g_rom.offset(crate::rtl::lorom_offset(0xaadedb)) as *const uint8
        as *mut uint16)
        .offset(6);
    jp
}
pub unsafe fn Shaktool_DAE5(mut k: uint16, game: &mut Game) {
    if (NextRandom(game) as ::core::ffi::c_int & 0x8431 as ::core::ffi::c_int) == 0i32 {
        let mut i: ::core::ffi::c_int = 12 as ::core::ffi::c_int;
        while i >= 0i32 {
            let mut E: *mut Enemy_Shaktool = Get_Shaktool(k as ::core::ffi::c_int, game);
            (*E).shakt_var_F = ((0xaadcaai32) as u16) & (0xffffu16);
            (*E).base.current_instruction = *(game.g_rom.offset(crate::rtl::lorom_offset(0xaadf21))
                as *const uint8 as *mut uint16)
                .offset((i >> 1 as ::core::ffi::c_int) as isize);
            (*E).base.instruction_timer = 1 as uint16;
            k = (k).wrapping_sub(64u16);
            i -= 2 as ::core::ffi::c_int;
        }
    }
}
pub unsafe fn Shaktool_DB0E(mut k: uint16, mut a: uint16, game: &mut Game) {
    let mut shakt_var_E: uint16 = (*Get_Shaktool(k as ::core::ffi::c_int, game)).shakt_var_E;
    (*Get_Shaktool(shakt_var_E as ::core::ffi::c_int, game)).shakt_parameter_1 = a;
    (*Get_Shaktool(
        shakt_var_E as ::core::ffi::c_int + 64 as ::core::ffi::c_int,
        game,
    ))
    .shakt_parameter_1 = a;
    (*Get_Shaktool(
        shakt_var_E as ::core::ffi::c_int + 128 as ::core::ffi::c_int,
        game,
    ))
    .shakt_parameter_1 = a;
    (*Get_Shaktool(
        shakt_var_E as ::core::ffi::c_int + 192 as ::core::ffi::c_int,
        game,
    ))
    .shakt_parameter_1 = a;
    (*Get_Shaktool(
        shakt_var_E as ::core::ffi::c_int + 256 as ::core::ffi::c_int,
        game,
    ))
    .shakt_parameter_1 = a;
    (*Get_Shaktool(
        shakt_var_E as ::core::ffi::c_int + 320 as ::core::ffi::c_int,
        game,
    ))
    .shakt_parameter_1 = a;
    (*Get_Shaktool(
        shakt_var_E as ::core::ffi::c_int + 384 as ::core::ffi::c_int,
        game,
    ))
    .shakt_parameter_1 = a;
}
pub unsafe fn Shaktool_DB27(mut k: uint16, mut a: uint16, game: &mut Game) {
    let mut shakto_var_E: uint16 = (*Get_Shaktool(k as ::core::ffi::c_int, game)).shakt_var_E;
    (*Get_Shaktool(shakto_var_E as ::core::ffi::c_int, game)).shakt_var_A = a;
    (*Get_Shaktool(
        shakto_var_E as ::core::ffi::c_int + 64 as ::core::ffi::c_int,
        game,
    ))
    .shakt_var_A = a;
    (*Get_Shaktool(
        shakto_var_E as ::core::ffi::c_int + 128 as ::core::ffi::c_int,
        game,
    ))
    .shakt_var_A = a;
    (*Get_Shaktool(
        shakto_var_E as ::core::ffi::c_int + 192 as ::core::ffi::c_int,
        game,
    ))
    .shakt_var_A = a;
    (*Get_Shaktool(
        shakto_var_E as ::core::ffi::c_int + 256 as ::core::ffi::c_int,
        game,
    ))
    .shakt_var_A = a;
    (*Get_Shaktool(
        shakto_var_E as ::core::ffi::c_int + 320 as ::core::ffi::c_int,
        game,
    ))
    .shakt_var_A = a;
    (*Get_Shaktool(
        shakto_var_E as ::core::ffi::c_int + 384 as ::core::ffi::c_int,
        game,
    ))
    .shakt_var_A = a;
}
pub unsafe fn Shaktool_DB40(mut k: uint16, mut a: uint16, game: &mut Game) {
    let mut shakto_var_E: uint16 = (*Get_Shaktool(k as ::core::ffi::c_int, game)).shakt_var_E;
    (*Get_Shaktool(shakto_var_E as ::core::ffi::c_int, game)).shakt_var_B = a;
    (*Get_Shaktool(
        shakto_var_E as ::core::ffi::c_int + 64 as ::core::ffi::c_int,
        game,
    ))
    .shakt_var_B = a;
    (*Get_Shaktool(
        shakto_var_E as ::core::ffi::c_int + 128 as ::core::ffi::c_int,
        game,
    ))
    .shakt_var_B = a;
    (*Get_Shaktool(
        shakto_var_E as ::core::ffi::c_int + 192 as ::core::ffi::c_int,
        game,
    ))
    .shakt_var_B = a;
    (*Get_Shaktool(
        shakto_var_E as ::core::ffi::c_int + 256 as ::core::ffi::c_int,
        game,
    ))
    .shakt_var_B = a;
    (*Get_Shaktool(
        shakto_var_E as ::core::ffi::c_int + 320 as ::core::ffi::c_int,
        game,
    ))
    .shakt_var_B = a;
    (*Get_Shaktool(
        shakto_var_E as ::core::ffi::c_int + 384 as ::core::ffi::c_int,
        game,
    ))
    .shakt_var_B = a;
}
pub unsafe fn Shaktool_DB59(mut k: uint16, game: &mut Game) {
    let mut shakto_var_E: uint16 = 0;
    let mut shakto_var_B: uint16 = 0;
    let mut v7: uint16 = 0;
    let mut v10: uint16 = 0;
    let mut x_subpos: uint16 = 0;
    let mut y_subpos: uint16 = 0;
    let mut x_pos: uint16 = 0;
    let mut y_pos: uint16 = 0;
    shakto_var_E = (*Get_Shaktool(k as ::core::ffi::c_int, game)).shakt_var_E;
    let mut Shaktool: *mut Enemy_Shaktool = Get_Shaktool(
        shakto_var_E as ::core::ffi::c_int + 192 as ::core::ffi::c_int,
        game,
    );
    (*Shaktool).shakt_var_D = ((*Shaktool).shakt_var_D) ^ (0x8000u16);
    let mut v3: *mut Enemy_Shaktool = Get_Shaktool(
        shakto_var_E as ::core::ffi::c_int + 384 as ::core::ffi::c_int,
        game,
    );
    shakto_var_B = (*v3).shakt_var_B;
    let mut v5: *mut Enemy_Shaktool = Get_Shaktool(
        shakto_var_E as ::core::ffi::c_int + 64 as ::core::ffi::c_int,
        game,
    );
    (*v3).shakt_var_B = (((*v5).shakt_var_B as ::core::ffi::c_int ^ 0x8000 as ::core::ffi::c_int)
        & 0xff00 as ::core::ffi::c_int) as uint16;
    (*v5).shakt_var_B = ((shakto_var_B as ::core::ffi::c_int ^ 0x8000 as ::core::ffi::c_int)
        & 0xff00 as ::core::ffi::c_int) as uint16;
    let mut v6: *mut Enemy_Shaktool = Get_Shaktool(
        shakto_var_E as ::core::ffi::c_int + 320 as ::core::ffi::c_int,
        game,
    );
    v7 = (*v6).shakt_var_B;
    let mut v8: *mut Enemy_Shaktool = Get_Shaktool(
        shakto_var_E as ::core::ffi::c_int + 128 as ::core::ffi::c_int,
        game,
    );
    (*v6).shakt_var_B = (((*v8).shakt_var_B as ::core::ffi::c_int ^ 0x8000 as ::core::ffi::c_int)
        & 0xff00 as ::core::ffi::c_int) as uint16;
    (*v8).shakt_var_B = ((v7 as ::core::ffi::c_int ^ 0x8000 as ::core::ffi::c_int)
        & 0xff00 as ::core::ffi::c_int) as uint16;
    let mut v9: *mut Enemy_Shaktool = Get_Shaktool(
        shakto_var_E as ::core::ffi::c_int + 256 as ::core::ffi::c_int,
        game,
    );
    v10 = (*v9).shakt_var_B;
    let mut v11: *mut Enemy_Shaktool = Get_Shaktool(
        shakto_var_E as ::core::ffi::c_int + 192 as ::core::ffi::c_int,
        game,
    );
    (*v9).shakt_var_B = (((*v11).shakt_var_B as ::core::ffi::c_int ^ 0x8000 as ::core::ffi::c_int)
        & 0xff00 as ::core::ffi::c_int) as uint16;
    (*v11).shakt_var_B = ((v10 as ::core::ffi::c_int ^ 0x8000 as ::core::ffi::c_int)
        & 0xff00 as ::core::ffi::c_int) as uint16;
    let mut v12: *mut Enemy_Shaktool = Get_Shaktool(
        shakto_var_E as ::core::ffi::c_int + 384 as ::core::ffi::c_int,
        game,
    );
    x_subpos = (*v12).base.x_subpos;
    let mut v14: *mut Enemy_Shaktool = Get_Shaktool(shakto_var_E as ::core::ffi::c_int, game);
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
    (*Get_Shaktool(
        shakto_var_E as ::core::ffi::c_int + 64 as ::core::ffi::c_int,
        game,
    ))
    .base
    .x_subpos = 0x8000 as uint16;
    (*Get_Shaktool(
        shakto_var_E as ::core::ffi::c_int + 128 as ::core::ffi::c_int,
        game,
    ))
    .base
    .x_subpos = 0x8000 as uint16;
    (*Get_Shaktool(
        shakto_var_E as ::core::ffi::c_int + 192 as ::core::ffi::c_int,
        game,
    ))
    .base
    .x_subpos = 0x8000 as uint16;
    (*Get_Shaktool(
        shakto_var_E as ::core::ffi::c_int + 256 as ::core::ffi::c_int,
        game,
    ))
    .base
    .x_subpos = 0x8000 as uint16;
    (*Get_Shaktool(
        shakto_var_E as ::core::ffi::c_int + 320 as ::core::ffi::c_int,
        game,
    ))
    .base
    .x_subpos = 0x8000 as uint16;
    (*Get_Shaktool(
        shakto_var_E as ::core::ffi::c_int + 64 as ::core::ffi::c_int,
        game,
    ))
    .base
    .y_subpos = 0x8000 as uint16;
    (*Get_Shaktool(
        shakto_var_E as ::core::ffi::c_int + 128 as ::core::ffi::c_int,
        game,
    ))
    .base
    .y_subpos = 0x8000 as uint16;
    (*Get_Shaktool(
        shakto_var_E as ::core::ffi::c_int + 192 as ::core::ffi::c_int,
        game,
    ))
    .base
    .y_subpos = 0x8000 as uint16;
    (*Get_Shaktool(
        shakto_var_E as ::core::ffi::c_int + 256 as ::core::ffi::c_int,
        game,
    ))
    .base
    .y_subpos = 0x8000 as uint16;
    (*Get_Shaktool(
        shakto_var_E as ::core::ffi::c_int + 320 as ::core::ffi::c_int,
        game,
    ))
    .base
    .y_subpos = 0x8000 as uint16;
}
pub unsafe fn Shaktool_DC07(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Shaktool = Get_Shaktool(k as ::core::ffi::c_int, game);
    let mut v2: uint16 = 0;
    if ((*E).shakt_parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        v2 = ((*E).shakt_var_B).wrapping_sub((*E).shakt_var_A);
    } else {
        v2 = ((*E).shakt_var_A).wrapping_sub((*E).shakt_var_B);
    }
    (*E).shakt_var_C = (4u16).wrapping_mul(
        (*(&raw mut v2 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1 as usize) as isize,
        )) as u16,
    );
}
pub unsafe fn Shaktool_DC2A(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Shaktool = Get_Shaktool(k as ::core::ffi::c_int, game);
    let mut v3: ::core::ffi::c_int = *(&raw mut (*E).shakt_var_B as *mut uint8).offset(
        (::core::mem::size_of::<uint16>())
            .wrapping_div(::core::mem::size_of::<uint8>())
            .wrapping_sub(1 as usize) as isize,
    ) as ::core::ffi::c_int;
    let mut v5: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    SetHiLo(
        &raw mut (*E).base.x_pos,
        &raw mut (*E).base.x_subpos,
        (((*(game.g_rom.offset(crate::rtl::lorom_offset(0xaae0bd)) as *const uint8 as *mut uint16)
            .offset(v3 as isize) as int16 as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint32)
            .wrapping_add(
                (*((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
                    .offset((v5 + 91 as ::core::ffi::c_int) as isize) as uint32)
                    << 16 as ::core::ffi::c_int
                    | *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
                        .offset((v5 + 92 as ::core::ffi::c_int) as isize)
                        as uint32,
            ),
    );
    SetHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        (((*(game.g_rom.offset(crate::rtl::lorom_offset(0xaae03d)) as *const uint8 as *mut uint16)
            .offset(v3 as isize) as int16 as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint32)
            .wrapping_add(
                (*((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
                    .offset((v5 + 93 as ::core::ffi::c_int) as isize) as uint32)
                    << 16 as ::core::ffi::c_int
                    | *((&raw mut game.g_ram as *mut uint8).offset(0xe84) as *mut uint16)
                        .offset((v5 + 94 as ::core::ffi::c_int) as isize)
                        as uint32,
            ),
    );
}
pub unsafe fn Shaktool_DC6F(mut k: uint16, game: &mut Game) {
    let mut E: *mut Enemy_Shaktool = Get_Shaktool(k as ::core::ffi::c_int, game);
    Shaktool_DB40(k, (*E).shakt_var_A, game);
    let mut shakto_var_E: uint16 = (*E).shakt_var_E;
    (*Get_Shaktool(shakto_var_E as ::core::ffi::c_int, game)).shakt_var_C =
        g_word_AADEE9[0 as ::core::ffi::c_int as usize];
    (*Get_Shaktool(
        shakto_var_E as ::core::ffi::c_int + 64 as ::core::ffi::c_int,
        game,
    ))
    .shakt_var_C = g_word_AADEE9[1 as ::core::ffi::c_int as usize];
    (*Get_Shaktool(
        shakto_var_E as ::core::ffi::c_int + 128 as ::core::ffi::c_int,
        game,
    ))
    .shakt_var_C = g_word_AADEE9[2 as ::core::ffi::c_int as usize];
    (*Get_Shaktool(
        shakto_var_E as ::core::ffi::c_int + 192 as ::core::ffi::c_int,
        game,
    ))
    .shakt_var_C = g_word_AADEE9[3 as ::core::ffi::c_int as usize];
    (*Get_Shaktool(
        shakto_var_E as ::core::ffi::c_int + 256 as ::core::ffi::c_int,
        game,
    ))
    .shakt_var_C = g_word_AADEE9[4 as ::core::ffi::c_int as usize];
    (*Get_Shaktool(
        shakto_var_E as ::core::ffi::c_int + 320 as ::core::ffi::c_int,
        game,
    ))
    .shakt_var_C = g_word_AADEE9[5 as ::core::ffi::c_int as usize];
    (*Get_Shaktool(
        shakto_var_E as ::core::ffi::c_int + 384 as ::core::ffi::c_int,
        game,
    ))
    .shakt_var_C = g_word_AADEE9[6 as ::core::ffi::c_int as usize];
}
pub unsafe fn Shaktool_Hurt(game: &mut Game) {
    let mut E: *mut Enemy_Shaktool = Get_Shaktool(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    CallEnemyPreInstr((((*E).shakt_var_F) as u32) | (0xaa0000u32), game);
}
pub unsafe fn Shaktool_DCAC(mut k: uint16, game: &mut Game) {
    Shaktool_DC2A(k, game);
    let mut E: *mut Enemy_Shaktool = Get_Shaktool(k as ::core::ffi::c_int, game);
    let mut shakto_var_C: uint16 = 0;
    if ((*E).shakt_parameter_1 as ::core::ffi::c_int & 0x4000 as ::core::ffi::c_int) != 0i32 {
        (*E).shakt_var_A = ((*E).shakt_var_A).wrapping_add(256u16);
        shakto_var_C = 256 as uint16;
    } else {
        shakto_var_C = (*E).shakt_var_C;
    }
    (*E).shakt_var_B = ((*E).shakt_var_B as ::core::ffi::c_int
        + if (*E).shakt_parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
            -(shakto_var_C as ::core::ffi::c_int)
        } else {
            shakto_var_C as ::core::ffi::c_int
        }) as uint16;
}
pub unsafe fn Shaktool_DCD7(mut k: uint16, game: &mut Game) {
    Shaktool_DCAC(k, game);
    let mut E: *mut Enemy_Shaktool = Get_Shaktool(k as ::core::ffi::c_int, game);
    let mut r18: uint16 = ((*E).shakt_var_B) ^ (0x8000u16);
    let mut v2: int16 = (r18 as ::core::ffi::c_int
        + ((((*Get_Shaktool(k as ::core::ffi::c_int + 64 as ::core::ffi::c_int, game)).shakt_var_B)
            .wrapping_sub(r18)) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int)) as int16;
    if ((*E).shakt_var_D as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
        let ref mut fresh2 = *(&raw mut v2 as *mut uint8).offset(
            (::core::mem::size_of::<int16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1 as usize) as isize,
        );
        *fresh2 = (*fresh2) ^ (0x80u8);
    }
    let mut v3: uint16 = (*(&raw mut v2 as *mut uint8).offset(
        (::core::mem::size_of::<int16>())
            .wrapping_div(::core::mem::size_of::<uint8>())
            .wrapping_sub(1 as usize) as isize,
    ) as ::core::ffi::c_int
        + 8 as ::core::ffi::c_int
        & 0xe0 as ::core::ffi::c_int) as uint16;
    *(&raw mut (*E).shakt_var_D as *mut uint8).offset(0) = v3 as uint8;
    (*E).base.current_instruction = *(game.g_rom.offset(crate::rtl::lorom_offset(0xaadd15))
        as *const uint8 as *mut uint16)
        .offset(
            ((v3 as ::core::ffi::c_int >> 4 as ::core::ffi::c_int) as uint16 as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int) as isize,
        );
    (*E).base.instruction_timer = 1 as uint16;
}
pub unsafe fn Shaktool_DD25(mut k: uint16, game: &mut Game) {
    let mut v7: int16 = 0;
    let mut E: *mut Enemy_Shaktool = Get_Shaktool(k as ::core::ffi::c_int, game);
    let mut y_pos: uint16 = (*E).base.y_pos;
    let mut x_pos: uint16 = (*E).base.x_pos;
    Shaktool_DCAC(k, game);
    let mut v12: uint16 = (*E).base.y_pos;
    let mut v11: uint16 = (*E).base.x_pos;
    (*E).base.x_pos = x_pos;
    (*E).base.y_pos = y_pos;
    if Enemy_MoveRight_IgnoreSlopes(
        k,
        ((((v11) as i16).wrapping_sub((x_pos) as i16)) as int32) << 16 as ::core::ffi::c_int,
        game,
    ) != 0u8
        || {
            (*E).base.y_pos = y_pos;
            Enemy_MoveDown(
                k,
                ((((v12) as i16).wrapping_sub((y_pos) as i16)) as int32)
                    << 16 as ::core::ffi::c_int,
                game,
            ) != 0u8
        }
    {
        if ((*E).shakt_parameter_1 as ::core::ffi::c_int & 0x2000 as ::core::ffi::c_int) != 0i32 {
            Shaktool_DB0E(
                k,
                (((*E).shakt_parameter_1 as ::core::ffi::c_int ^ 0x8000 as ::core::ffi::c_int)
                    & 0x8fff as ::core::ffi::c_int) as uint16,
                game,
            );
        } else {
            (*E).base.x_pos = x_pos;
            (*E).base.y_pos = y_pos;
            Shaktool_DB59(k, game);
            Shaktool_DB0E(
                *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
                ((*E).shakt_parameter_1) | (0x2000u16),
                game,
            );
            Shaktool_DB0E(
                *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
                ((*E).shakt_parameter_1) & (0xbfffu16),
                game,
            );
        }
        (*E).shakt_var_D = 0 as uint16;
        let mut v5: uint16 = CalculateAngleOfEnemyXfromEnemyY(k, (*E).shakt_var_E, game);
        v7 = ((v5 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int) as int16;
        let mut v8: uint16 = 0;
        if ((*E).shakt_parameter_1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
            v8 = ((v7) as u16).wrapping_sub(0x4000u16);
        } else {
            v8 = ((v7) as u16).wrapping_add(0x4000u16);
        }
        Shaktool_DB27(k, v8, game);
        let mut i: ::core::ffi::c_int = 12 as ::core::ffi::c_int;
        while i >= 0i32 {
            Shaktool_DC07(k, game);
            let mut EK: *mut Enemy_Shaktool = Get_Shaktool(k as ::core::ffi::c_int, game);
            (*EK).shakt_var_F = ((0xaadcaai32) as u16) & (0xffffu16);
            (*EK).base.current_instruction =
                *(game.g_rom.offset(crate::rtl::lorom_offset(0xaadf13)) as *const uint8
                    as *mut uint16)
                    .offset((i >> 1 as ::core::ffi::c_int) as isize);
            (*EK).base.instruction_timer = 1 as uint16;
            k = (k).wrapping_sub(64u16);
            i -= 2 as ::core::ffi::c_int;
        }
    } else {
        (*E).base.x_pos = v11;
        (*E).base.y_pos = v12;
        if ((*E).shakt_parameter_1 as ::core::ffi::c_int & 0x4000 as ::core::ffi::c_int) != 0i32 {
            (*E).shakt_var_A = ((*E).shakt_var_A).wrapping_add(256u16);
        } else {
            if (((*E).shakt_var_A as ::core::ffi::c_int ^ (*E).shakt_var_B as ::core::ffi::c_int)
                & 0xff00 as ::core::ffi::c_int)
                == 0i32
            {
                Shaktool_DC6F(k, game);
                (*E).shakt_var_D = 30720 as uint16;
                Shaktool_DB0E(k, ((*E).shakt_parameter_1) & (0xdfffu16), game);
                (*E).shakt_var_D = ((*(&raw mut (*E).shakt_var_D as *mut uint8).offset(
                    (::core::mem::size_of::<uint16>())
                        .wrapping_div(::core::mem::size_of::<uint8>())
                        .wrapping_sub(1 as usize) as isize,
                ) as ::core::ffi::c_int)
                    << 8 as ::core::ffi::c_int) as uint16;
            }
            let mut v2: uint16 = ((*E).shakt_var_D).wrapping_add((*E).shakt_var_C);
            (*E).shakt_var_D = v2;
            if v2 >= 0xf000u16 {
                Shaktool_DB0E(k, ((*E).shakt_parameter_1) | (0x4000u16), game);
            }
        }
    };
}
pub unsafe fn Shaktool_Init(game: &mut Game) {
    let mut E: *mut Enemy_Shaktool = Get_Shaktool(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).base.instruction_timer = 1 as uint16;
    (*E).base.timer = 0 as uint16;
    (*E).shakt_var_A = 0 as uint16;
    (*E).shakt_var_D = 0 as uint16;
    let mut shakto_parameter_2: uint16 = (*E).shakt_parameter_2;
    let mut v3: ::core::ffi::c_int =
        shakto_parameter_2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    (*E).base.properties = ((*E).base.properties)
        | (*(game.g_rom.offset(crate::rtl::lorom_offset(0xaade95)) as *const uint8 as *mut uint16)
            .offset(v3 as isize));
    (*E).shakt_var_E = (*((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16))
        .wrapping_sub(
            *(game.g_rom.offset(crate::rtl::lorom_offset(0xaadea3)) as *const uint8 as *mut uint16)
                .offset(v3 as isize),
        );
    (*E).shakt_var_F = *((game.g_rom.offset(crate::rtl::lorom_offset(0xaadedb)) as *const uint8
        as *mut uint16 as *mut uint8)
        .offset(shakto_parameter_2 as ::core::ffi::c_int as isize)
        as *mut uint16);
    (*E).shakt_var_C = (*((&raw const g_word_AADEE9 as *const uint16 as *mut uint8)
        .offset(shakto_parameter_2 as ::core::ffi::c_int as isize)
        as *mut uint16))
        .wrapping_sub(
            *(game.g_rom.offset(crate::rtl::lorom_offset(0xaadef7)) as *const uint8 as *mut uint16)
                .offset(v3 as isize),
        );
    (*E).shakt_var_B = *(game.g_rom.offset(crate::rtl::lorom_offset(0xaadeb1)) as *const uint8
        as *mut uint16)
        .offset(v3 as isize);
    (*E).base.current_instruction = *(game.g_rom.offset(crate::rtl::lorom_offset(0xaadeb1))
        as *const uint8 as *mut uint16)
        .offset((v3 + 7 as ::core::ffi::c_int) as isize);
    (*E).base.layer = *(game.g_rom.offset(crate::rtl::lorom_offset(0xaadecd)) as *const uint8
        as *mut uint16)
        .offset(v3 as isize);
    if shakto_parameter_2 != 0 {
        Shaktool_DC2A(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        );
    }
}
pub unsafe fn Shaktool_Touch(game: &mut Game) {
    NormalEnemyTouchAi(game);
}
pub unsafe fn Shaktool_Shot(game: &mut Game) {
    NormalEnemyShotAi(game);
    let mut E: *mut Enemy_Shaktool = Get_Shaktool(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).base.health == 0 {
        let mut shakto_var_E: uint16 = (*E).shakt_var_E;
        (*Get_Shaktool(shakto_var_E as ::core::ffi::c_int, game))
            .base
            .properties = 512 as uint16;
        (*Get_Shaktool(
            shakto_var_E as ::core::ffi::c_int + 64 as ::core::ffi::c_int,
            game,
        ))
        .base
        .properties = 512 as uint16;
        (*Get_Shaktool(
            shakto_var_E as ::core::ffi::c_int + 128 as ::core::ffi::c_int,
            game,
        ))
        .base
        .properties = 512 as uint16;
        (*Get_Shaktool(
            shakto_var_E as ::core::ffi::c_int + 192 as ::core::ffi::c_int,
            game,
        ))
        .base
        .properties = 512 as uint16;
        (*Get_Shaktool(
            shakto_var_E as ::core::ffi::c_int + 256 as ::core::ffi::c_int,
            game,
        ))
        .base
        .properties = 512 as uint16;
        (*Get_Shaktool(
            shakto_var_E as ::core::ffi::c_int + 320 as ::core::ffi::c_int,
            game,
        ))
        .base
        .properties = 512 as uint16;
        (*Get_Shaktool(
            shakto_var_E as ::core::ffi::c_int + 384 as ::core::ffi::c_int,
            game,
        ))
        .base
        .properties = 512 as uint16;
    }
}
pub unsafe fn Shaktool_Instr_9(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1980) as *mut uint16) = 32 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x197c) as *mut uint16) = 64 as uint16;
    jp
}
pub unsafe fn Shaktool_Instr_11(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1978) as *mut uint16) = 722 as uint16;
    jp
}
pub unsafe fn Shaktool_Instr_10(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    CallSomeSamusCode(1 as uint16, game);
    jp
}
pub unsafe fn sub_AAE445(mut k: uint16, game: &mut Game) {
    let mut v1: *mut EnemyData = gEnemyData(k as ::core::ffi::c_int, game);
    if (*v1).parameter_1 != 0 {
        (*v1).current_instruction = addr_kShaktool_Ilist_E3A7 as uint16;
        (*v1).instruction_timer = 1 as uint16;
    }
}
pub unsafe fn Shaktool_Instr_8(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    QueueSfx2_Max6(0x1c as uint16, game);
    jp
}
pub unsafe fn Shaktool_Instr_13(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    QueueSfx2_Max6(0x4b as uint16, game);
    jp
}
pub unsafe fn Shaktool_Instr_12(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut e: *mut Enemy_Shaktool = Get_Shaktool(k as ::core::ffi::c_int, game);
    let mut arg: uint16 = *jp.offset(0);
    CalculateBlockContainingPixelPos(
        ((*e).base.x_pos).wrapping_add(arg),
        ((*e).base.y_pos).wrapping_add(28u16),
        game,
    );
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x10002) as *mut uint16).offset(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x7a5) as *mut uint16) as ::core::ffi::c_int
            + *((&raw mut game.g_ram as *mut uint8).offset(0xdc4) as *mut uint16)
                as ::core::ffi::c_int) as isize,
    ) as ::core::ffi::c_int
        & 0xf000 as ::core::ffi::c_int)
        == 0xa000i32
    {
        SpawnPLM(addr_kPlmHeader_D113 as uint16, game);
        SpawnEprojWithRoomGfx(
            addr_kEproj_WreckedShipChozoSpikeFootsteps_1 as uint16,
            arg,
            game,
        );
    }
    jp.offset(1)
}
pub unsafe fn Shaktool_Instr_7(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_Shaktool = Get_Shaktool(k as ::core::ffi::c_int, game);
    (*E).shakt_var_C = *jp.offset(0);
    Enemy_MoveRight_IgnoreSlopes(
        k,
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0xaae630)) as *const uint8 as *mut uint16)
            .offset(((*E).shakt_var_C as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
            as int16 as int32)
            << 8 as ::core::ffi::c_int,
        game,
    );
    Enemy_MoveDown(
        k,
        (abs16(
            *(game.g_rom.offset(crate::rtl::lorom_offset(0xaae630)) as *const uint8 as *mut uint16)
                .offset(
                    ((*E).shakt_var_C as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize,
                ),
        ) as int16 as int32)
            << 8 as ::core::ffi::c_int,
        game,
    );
    EnemyFunc_C8AD(k, game);
    let mut v6: ::core::ffi::c_int =
        (*E).shakt_var_C as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) =
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0xaae670)) as *const uint8 as *mut uint16)
            .offset(v6 as isize))
        .wrapping_add((*E).base.x_pos);
    *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16) =
        (*(game.g_rom.offset(crate::rtl::lorom_offset(0xaae6b0)) as *const uint8 as *mut uint16)
            .offset(v6 as isize))
        .wrapping_add((*E).base.y_pos);
    jp.offset(1)
}
pub unsafe fn Shaktool_Instr_14(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    CallSomeSamusCode(1 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(6) as *mut uint8 as *mut uint16) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(8) as *mut uint8 as *mut uint16) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(9) as *mut uint8 as *mut uint16) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8)
        .offset(0xcd20)
        .offset(13) as *mut uint8 as *mut uint16) = 1 as uint16;
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs {
            field_0: 0x17 as uint8,
            field_1: 0x1d as uint8,
            field_2: 0xd6fc as VoidP,
        },
        game,
    );
    jp
}
pub unsafe fn N00bTubeCracks_Init(game: &mut Game) {
    let mut i: ::core::ffi::c_int = 62 as ::core::ffi::c_int;
    while i >= 0i32 {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16)
            .offset(((i >> 1 as ::core::ffi::c_int) + 144 as ::core::ffi::c_int) as isize) =
            *(game.g_rom.offset(crate::rtl::lorom_offset(0xaae2dd)) as *const uint8 as *mut uint16)
                .offset((i >> 1 as ::core::ffi::c_int) as isize);
        i -= 2 as ::core::ffi::c_int;
    }
}
pub unsafe fn ChozoStatue_Init(game: &mut Game) {
    let mut v0: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*v0).properties = ((*v0).properties as ::core::ffi::c_int
        | (kEnemyProps_DisableSamusColl as ::core::ffi::c_int
            | kEnemyProps_ProcessedOffscreen as ::core::ffi::c_int
            | 0x8000 as ::core::ffi::c_int)) as uint16;
    (*v0).spritemap_pointer = addr_kSpritemap_Nothing_AA as uint16;
    (*v0).instruction_timer = 1 as uint16;
    (*v0).timer = 0 as uint16;
    (*v0).ai_preinstr = ((0xaae7a6i32) as u16) & (0xffffu16);
    (*v0).parameter_1 = 0 as uint16;
    (*v0).palette_index = 0 as uint16;
    (*gEnemyData(0 as ::core::ffi::c_int, game)).layer = 0 as uint16;
    let mut parameter_2: uint16 = (*v0).parameter_2;
    (*v0).current_instruction = *(game.g_rom.offset(crate::rtl::lorom_offset(0xaae7a2))
        as *const uint8 as *mut uint16)
        .offset((parameter_2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    if parameter_2 != 0 {
        sub_AAE784(game);
    } else {
        let mut i: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
        while i >= 0i32 {
            let mut v3: ::core::ffi::c_int = i >> 1 as ::core::ffi::c_int;
            *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16)
                .offset((v3 + 160 as ::core::ffi::c_int) as isize) =
                kChozoStatue_Palettes[v3 as usize] as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16)
                .offset((v3 + 144 as ::core::ffi::c_int) as isize) =
                kChozoStatue_Palette[v3 as usize];
            i -= 2 as ::core::ffi::c_int;
        }
        SpawnHardcodedPlm(
            SpawnHardcodedPlmArgs {
                field_0: 0x4a as uint8,
                field_1: 0x17 as uint8,
                field_2: 0xd6ee as VoidP,
            },
            game,
        );
        SpawnHardcodedPlm(
            SpawnHardcodedPlmArgs {
                field_0: 0x17 as uint8,
                field_1: 0x1d as uint8,
                field_2: 0xd6fc as VoidP,
            },
            game,
        );
    };
}
pub unsafe fn sub_AAE784(game: &mut Game) {
    let mut i: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
    while i >= 0i32 {
        let mut v1: ::core::ffi::c_int = i >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16)
            .offset((v1 + 160 as ::core::ffi::c_int) as isize) =
            kChozoStatue_Palettes3[v1 as usize] as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16)
            .offset((v1 + 144 as ::core::ffi::c_int) as isize) =
            kChozoStatue_Palettes2[v1 as usize] as uint16;
        i -= 2 as ::core::ffi::c_int;
    }
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs {
            field_0: 0xc as uint8,
            field_1: 0x1d as uint8,
            field_2: 0xd6d6 as VoidP,
        },
        game,
    );
}
pub unsafe fn ChozoStatue_Main(game: &mut Game) {
    let mut v0: *mut EnemyData = gEnemyData(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    CallEnemyPreInstr((((*v0).ai_preinstr) as u32) | (0xaa0000u32), game);
}
pub unsafe fn Shaktool_PreInstr_0(mut k: uint16, game: &mut Game) {
    if (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xd828)
        .offset(*((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) as isize)
        as *mut uint8 as *mut uint16) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int)
        != 0i32
    {
        let mut E: *mut Enemy_Shaktool = Get_Shaktool(k as ::core::ffi::c_int, game);
        if (*E).shakt_parameter_1 != 0 {
            (*E).base.current_instruction = addr_kShaktool_Ilist_E461 as uint16;
            (*E).base.instruction_timer = 1 as uint16;
            (*E).shakt_var_A = -(256 as ::core::ffi::c_int) as uint16;
            (*E).shakt_var_B = 256 as uint16;
        }
    }
}
