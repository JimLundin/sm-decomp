pub use crate::game::constants::bank_84::*;
pub use crate::game::constants::bank_86::*;
pub use crate::game::constants::bank_8d::*;
pub use crate::game::constants::enemy_properties::*;
pub use crate::game::constants::samus_poses::*;
use crate::game::enemy_instruction;
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
use crate::ram_map::word;
use crate::rtl::abs16;
use crate::rtl::rom_ptr;
use crate::rtl::words;
use crate::rtl::{AddToHiLo, SetHiLo};
use crate::rtl::{RomPtr, Unreachable};
use crate::set_fields;
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
pub type Enemy_Shaktool = crate::game::enemy_layouts::EnemyLayout8;

pub type Enemy_Torizo = crate::game::enemy_layouts::EnemyLayout40;

#[inline]
unsafe fn RomBankBase(bank: u8, game: &mut Game) -> *const u8 {
    RomPtr((((bank as i32) << 16i32) + 0x8000i32) as u32, game).offset(-(0x8000i32 as isize))
}
crate::game::constants::typed_constants! { i32;
    fnnullsub_171_AA = 0xaa807b,
}
crate::game::constants::typed_constants! { i32;
    fnnullsub_274 = 0xaadcaa, fnnullsub_276 = 0xaae7a6,
}
crate::ram_map::ram_pointer_functions! {
    gEnemyData(o: i32) -> EnemyData = [0xf78, o as isize];
    Get_Shaktool(o: i32) -> Enemy_Shaktool = [0xf78, o as isize];
    Get_Torizo(o: i32) -> Enemy_Torizo = [0xf78, o as isize];
}
static g_word_AAB096: u16 = 6;
static g_word_AAB098: u16 = 5;
static g_word_AAB09A: u16 = 3;
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
crate::forward_calls! { pub unsafe fn (game: &mut Game);
    Enemy_GrappleReact_CancelBeam_AA => Enemy_SwitchToFrozenAi(game);
    Enemy_NormalFrozenAI_AA => NormalEnemyFrozenAI(game);
}
crate::instruction_callbacks! { pub unsafe fn (k: u16, jp: *const u16, game: &mut Game) -> *const u16;
    Enemy_ClearAiPreInstr_AA [jp] {
        (*gEnemyData(k as i32, game)).ai_preinstr = ((0xaa807bi32) as u16);
    };
    Enemy_SetAiPreInstr_AA [jp.offset(1)] {
        (*gEnemyData(k as i32, game)).ai_preinstr = *jp;
    };
    Torizo_Instr_3 [jp.offset(1)] { (*Get_Torizo(k as i32, game)).var_E = *jp; };
    Torizo_Instr_31 [jp] {
        let E = Get_Torizo(k as i32, game);
        (*E).parameter_2 = ((*E).parameter_2) | (0x8000u16);
        let mut parameter = SpawnEprojWithGfx((*E).parameter_2, k, addr_stru_86A95B as u16, game);
        parameter = SpawnEprojWithGfx(parameter, k, addr_stru_86A95B as u16, game);
        parameter = SpawnEprojWithGfx(parameter, k, addr_stru_86A95B as u16, game);
        parameter = SpawnEprojWithGfx(parameter, k, addr_stru_86A95B as u16, game);
        parameter = SpawnEprojWithGfx(parameter, k, addr_stru_86A95B as u16, game);
        SpawnEprojWithGfx(parameter, k, addr_stru_86A95B as u16, game);
    };
    Torizo_Instr_33 [jp] {
        let E = Get_Torizo(k as i32, game);
        (*E).parameter_2 = ((*E).parameter_2) | (0x4000u16);
    };
    Torizo_Instr_36 [jp] {
        let E = Get_Torizo(k as i32, game);
        (*E).base.properties = ((*E).base.properties as i32 & !(kEnemyProps_Invisible as i32)) as u16;
    };
    Torizo_Instr_37 [jp] {
        let E = Get_Torizo(k as i32, game);
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Invisible) as u16);
    };
    Torizo_Instr_35 [jp] {
        word_transfers!(game, descending, 16;
            le(target_palettes, 175) <- value(0),
            le(target_palettes, 159) <- value(0),
        );
    };
    Torizo_Instr_38 [jp] {
        SetBossBitForCurArea(4, game);
        QueueMusic_Delayed8(g_word_AAB09A, game);
        if word!(game.area_index) != 0 {
            Enemy_ItemDrop_GoldenTorizo(k, game);
        } else {
            Enemy_ItemDrop_BombTorizo(k, game);
        }
    };
    Torizo_Instr_6 [jp] { sub_82DAF7(0x600, game); };
    Torizo_Instr_5 [jp] { Torizo_C268(game); };
    Torizo_Instr_9 [jp] {
        QueueMusic_Delayed8(g_word_AAB098, game);
        SpawnPalfxObject(addr_stru_8DF759 as u16, game);
    };
}
pub unsafe fn Torizo_C20A(k: u16, game: &mut Game) {
    let E = Get_Torizo(k as i32, game);
    let mut v2: i16 = if ((*E).parameter_1 & 0x8000) != 0 {
        512
    } else {
        -512i16
    };
    (*E).var_A = v2 as u16;
    (*E).var_B = -1472i32 as u16;
    (*E).var_C = 40;
    (*E).base.instruction_timer = 1;
}
pub unsafe fn Torizo_C22D(k: u16, game: &mut Game) {
    let E = Get_Torizo(k as i32, game);
    let mut v2: i16 = if ((*E).parameter_1 & 0x8000) != 0 {
        -768i16
    } else {
        768
    };
    (*E).var_A = v2 as u16;
    (*E).var_B = -1152i32 as u16;
    (*E).var_C = 40;
    (*E).base.instruction_timer = 1;
}
pub unsafe fn Torizo_C250(game: &mut Game) {
    word_transfers!(game, descending, 16;
        le(target_palettes, 175) <- table(kTorizo_Palettes_3, 15),
        le(target_palettes, 159) <- table(kTorizo_Palettes_2, 15),
    );
}
pub unsafe fn Torizo_C268(game: &mut Game) {
    word_transfers!(game, descending, 16;
        le(target_palettes, 175) <- table(kTorizo_Palettes_5, 15),
        le(target_palettes, 159) <- table(kTorizo_Palettes_4, 15),
    );
}
pub unsafe fn Torizo_C280(game: &mut Game) {
    word_transfers!(game, descending, 16;
        le(target_palettes, 175) <- table(kTorizo_Palettes_7, 15),
        le(target_palettes, 159) <- table(kTorizo_Palettes_6, 15),
    );
}
pub unsafe fn Torizo_C298(game: &mut Game) {
    word_transfers!(game, descending, 16;
        le(target_palettes, 175) <- table(kTorizo_Palettes_10, 15),
        le(target_palettes, 159) <- table(kTorizo_Palettes_8, 15),
    );
}
pub unsafe fn Torizo_C2B0(game: &mut Game) {
    word_transfers!(game, descending, 16;
        le(palette_buffer, 175) <- table(kTorizo_Palettes_5, 15),
        le(palette_buffer, 159) <- table(kTorizo_Palettes_4, 15),
    );
}
pub fn Torizo_Instr_7(k: u16, jp: *const u16) -> *const u16 {
    jp
}
crate::instruction_callbacks! { pub unsafe fn (k: u16, jp: *const u16, game: &mut Game) -> *const u16;
    Torizo_Instr_2 [jp] { (*Get_Torizo(k as i32, game)).var_04 = 30583; };
    Torizo_Instr_8 [jp] { (*Get_Torizo(k as i32, game)).var_04 = 0; };
    Torizo_Instr_25 [] {
        if ((*Get_Torizo(k as i32, game)).parameter_2 & 0x4000) != 0 {
            return *jp as usize as *const u16;
        }
        if word!(game.area_index) != 0 {
            *jp.offset(1) as usize as *const u16
        } else {
            jp.offset(2)
        }
    };
    Torizo_Instr_22 [jp.offset(1)] { (*Get_Torizo(k as i32, game)).var_00 = *jp; };
    Torizo_Instr_19 [(*Get_Torizo(k as i32, game)).var_00 as usize as *const u16] { };
    Torizo_Instr_32 [(*Get_Torizo(k as i32, game)).var_01 as usize as *const u16] { };
    Torizo_Instr_30 [null::<u16>()] {
        let a: u16 = *jp;
        for _ in 0..=5 {
            SpawnEprojWithRoomGfx(addr_kEproj_BombTorizoLowHealthExplode as u16, a, game);
        }
        let E = Get_Torizo(k as i32, game);
        (*E).base.current_instruction = (jp.offset(1) as *mut u8)
            .offset_from(RomBankBase((*gEnemyData(k as i32, game)).bank, game))
            as c_long as u16;
        set_fields! {[(*E).base], 40; flash_timer, instruction_timer}
    };
    Torizo_Instr_34 [null::<u16>()] {
        // The instruction dispatcher leaves this handler address in A.
        SpawnEprojWithRoomGfx(addr_kEproj_BombTorizoDeathExplosion as u16, 0xc32f, game);
        let E = Get_Torizo(k as i32, game);
        (*E).base.current_instruction = (jp as *mut u8)
            .offset_from(RomBankBase((*gEnemyData(k as i32, game)).bank, game))
            as c_long as u16;
        set_fields! {[(*E).base], 1; flash_timer, instruction_timer}
    };
    Torizo_Instr_24 [jp] {
        let left_foot_parameter = SpawnEprojWithRoomGfx(
            addr_kEproj_TourianLandingDustCloudsRightFoot as u16,
            0xc34a,
            game,
        );
        SpawnEprojWithRoomGfx(addr_stru_86AFF3 as u16, left_foot_parameter, game);
    };
    Torizo_Instr_12 [jp] {
        let health: u16 = (*Get_Torizo(k as i32, game)).base.health;
        if health < 0x15e {
            SpawnEprojWithGfx(
                health,
                k,
                addr_kEproj_BombTorizoLowHealthInitialDrool as u16,
                game,
            );
        }
    };
    Torizo_Instr_10 [jp] {
        let E = Get_Torizo(k as i32, game);
        (*E).parameter_1 = ((*E).parameter_1) | (0x4000u16);
    };
    Torizo_Instr_11 [jp] {
        let E = Get_Torizo(k as i32, game);
        (*E).parameter_1 = ((*E).parameter_1) & (0x1fffu16);
        (*E).var_09 = (*E).var_09.wrapping_add(1);
    };
    Torizo_Instr_29 [jp] {
        let E = Get_Torizo(k as i32, game);
        (*E).parameter_1 = ((*E).parameter_1 & 0x1fff | 0x8000);
        (*E).var_09 = (*E).var_09.wrapping_add(1);
    };
    Torizo_Instr_1 [jp] {
        let E = Get_Torizo(k as i32, game);
        (*E).parameter_1 = ((*E).parameter_1 & 0x1fff | 0x2000);
        (*E).var_09 = (*E).var_09.wrapping_add(1);
    };
    Torizo_Instr_28 [jp] {
        let E = Get_Torizo(k as i32, game);
        (*E).parameter_1 = ((*E).parameter_1 & 0x1fff | 0xa000);
        (*E).var_09 = (*E).var_09.wrapping_add(1);
    };
    Torizo_Instr_4 [jp.offset(1)] {
        let v2: u16 = *jp;
        let E = Get_Torizo(k as i32, game);
        (*E).base.x_pos = ((*E).base.x_pos).wrapping_add((g_word_AAC3EE[(v2 >> 1) as usize]) as u16);
        (*E).base.y_pos = ((*E).base.y_pos)
            .wrapping_add((g_word_AAC40E[((v2 as i32 & 0xf) >> 1) as usize]) as u16);
    };
    Torizo_Instr_40 [jp.offset(1)] {
        let v2: u16 = *jp;
        let E = Get_Torizo(k as i32, game);
        (*E).base.x_pos = ((*E).base.x_pos).wrapping_sub((g_word_AAC440[(v2 >> 1) as usize]) as u16);
        (*E).base.y_pos = ((*E).base.y_pos)
            .wrapping_sub((g_word_AAC460[((v2 as i32 & 0xf) >> 1) as usize]) as u16);
    };
    Torizo_Instr_16 [] {
        let E = Get_Torizo(k as i32, game);
        (*E).var_A = g_word_AAC4BD[(*jp >> 1) as usize] as u16;
        if Enemy_MoveRight_IgnoreSlopes(k, ((*E).var_A as i16 as i32) << 16, game) != 0 {
            (*E).var_03 = 0;
            if ((*E).parameter_1 & 0x8000) != 0 {
                addr_kTorizo_Ilist_B962 as i32 as usize as *const u16
            } else {
                addr_kTorizo_Ilist_BDD8 as i32 as usize as *const u16
            }
        } else {
            EnemyFunc_C8AD(k, game);
            if (((*E).parameter_1 as i32
                ^ word!(game.samus_x_pos) as i32 - (*E).base.x_pos as i32)
                & 0x8000)
                == 0
            {
                if (*E).var_03 == 0 {
                    (*E).var_03 = 72;
                }
                jp.offset(1)
            } else {
                jp.offset(1)
            }
        }
    };
    Torizo_Instr_27 [] {
        let E = Get_Torizo(k as i32, game);
        (*E).var_A = g_word_AAC532[(*jp >> 1) as usize] as u16;
        if Enemy_MoveRight_IgnoreSlopes(k, ((*E).var_A as i16 as i32) << 16, game) != 0 {
            (*E).var_03 = 0;
            if ((*E).parameter_1 & 0x8000) != 0 {
                addr_kTorizo_Ilist_BD0E as i32 as usize as *const u16
            } else {
                addr_kTorizo_Ilist_C188 as i32 as usize as *const u16
            }
        } else {
            EnemyFunc_C8AD(k, game);
            if (((*E).parameter_1 as i32
                ^ word!(game.samus_x_pos) as i32 - (*E).base.x_pos as i32)
                & 0x8000)
                == 0
            {
                if (*E).var_03 == 0 {
                    (*E).var_03 = 72;
                }
                jp.offset(1)
            } else {
                jp.offset(1)
            }
        }
    };
    Torizo_Instr_23 [] {
        if ((*Get_Torizo(k as i32, game)).var_B & 0x8000) != 0 {
            *jp as usize as *const u16
        } else {
            jp.offset(1)
        }
    };
    Torizo_Instr_14 [*jp as usize as *const u16] {
        if CompareDistToSamus_X(k, 0x38, game) & 1 != 0 {
            return jp.offset(1);
        }
        let E = Get_Torizo(k as i32, game);
        if (((*E).parameter_1 as i32 ^ word!(game.samus_x_pos) as i32 - (*E).base.x_pos as i32)
            & 0x8000)
            == 0
        {
            return jp.offset(1);
        }
        (*E).var_00 = (jp.offset(1) as *mut u8)
            .offset_from(RomBankBase((*gEnemyData(k as i32, game)).bank, game))
            as c_long as u16;
    };
    Torizo_Instr_15 [*jp as usize as *const u16] {
        if CompareDistToSamus_X(k, 0x20, game) & 1 != 0 || (Torizo_Func_12(k, game) & 0x8000) != 0 {
            return jp.offset(1);
        }
        Torizo_C22D(k, game);
    };
    Torizo_Instr_26 [] {
        let E = Get_Torizo(k as i32, game);
        (*E).var_00 = (jp.offset(2) as *mut u8)
            .offset_from(RomBankBase((*gEnemyData(k as i32, game)).bank, game))
            as c_long as u16;
        if word!(game.samus_missiles) < 5
            || (word!(game.nmi_frame_counter_word) as i32
                + (word!(game.samus_x_pos) as i32 & 1)
                + (word!(game.samus_x_pos) as i32 >> 1)
                & 8)
                != 0
        {
            *jp as usize as *const u16
        } else {
            *jp.offset(1) as usize as *const u16
        }
    };
    Torizo_Instr_18 [jp] {
        // The instruction dispatcher leaves this handler address in A.
        let mut parameter = SpawnEprojWithGfx(0xc5cb, k, addr_kEproj_BombTorizosChozoOrbs as u16, game);
        parameter = SpawnEprojWithGfx(parameter, k, addr_kEproj_BombTorizosChozoOrbs as u16, game);
        SpawnEprojWithGfx(parameter, k, addr_kEproj_BombTorizosChozoOrbs as u16, game);
    };
    Torizo_Instr_20 [jp.offset(1)] {
        SpawnEprojWithGfx(
            *jp,
            k,
            addr_kEproj_BombTorizoSonicBoom as u16,
            game,
        );
    };
    Torizo_Instr_44 [jp.offset(1)] {
        SpawnEprojWithGfx(
            *jp,
            k,
            addr_kEproj_GoldenTorizoSonicBoom as u16,
            game,
        );
    };
    Torizo_Instr_21 [jp.offset(1)] {
        SpawnEprojWithRoomGfx(
            addr_kEproj_BombTorizoExplosiveSwipe as u16,
            *jp,
            game,
        );
    };
    Torizo_Instr_17 [jp] { QueueSfx2_Max6(0x27, game); };
    Torizo_Instr_13 [jp] { QueueSfx2_Max6(0x4b, game); };
}
pub unsafe fn Torizo_C620(k: u16, game: &mut Game) {
    if word!(game.area_index) == 0 && (word!(game.random_number) & 0x8142) == 0 {
        let health: u16 = (*Get_Torizo(k as i32, game)).base.health;
        if health != 0 && health < 0x15e {
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
    let E = Get_Torizo(k as i32, game);
    if Enemy_MoveDown(k, ((*E).var_B as i16 as i32) << 8, game) != 0 {
        let v3: i16 = (*E).var_B as i16;
        if v3 >= 0 && v3 != 256 {
            word!(game.earthquake_type = 4);
            word!(game.earthquake_timer = 32);
            (*E).var_B = 256;
        }
    } else {
        (*E).var_B = ((*E).var_B).wrapping_add(40);
    };
}
pub unsafe fn Torizo_Hurt(game: &mut Game) {
    Torizo_C620(word!(game.cur_enemy_index), game);
    if (*Get_Torizo(word!(game.cur_enemy_index) as i32, game))
        .base
        .flash_timer
        & 1
        != 0
    {
        word_transfers!(game, descending, 16;
            le(palette_buffer, 175) <- value(0x7fff),
            le(palette_buffer, 159) <- value(0x7fff),
        );
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
    let E = Get_Torizo(word!(game.cur_enemy_index) as i32, game);
    CallTorizoFunc(
        (((*E).var_E) as u32) | (0xaa0000u32),
        word!(game.cur_enemy_index),
        game,
    );
}
pub unsafe fn Torizo_Func_3(k: u16, game: &mut Game) {
    Torizo_C620(k, game);
    Torizo_C643(k, game);
}
pub unsafe fn Torizo_Func_4(k: u16, game: &mut Game) {
    let E = Get_Torizo(k as i32, game);
    (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
    let mut v2: u16 = 78;
    while word!(game.plm_header_ptr[(v2 >> 1)]) as i32 != addr_kPlmHeader_D6EA as i32 {
        v2 = (v2).wrapping_sub(2);
        if (v2 & 0x8000) != 0 {
            QueueMusic_Delayed8(g_word_AAB096, game);
            (*E).base.properties =
                ((*E).base.properties as i32 & !(kEnemyProps_Tangible as i32)) as u16;
            enemy_instruction! {[(*E).base]; instruction = ((*E).base.current_instruction).wrapping_add(2), timer = 1}
            return;
        }
    }
}
pub unsafe fn Torizo_Func_1(k: u16, game: &mut Game) {
    Torizo_C620(k, game);
    let E = Get_Torizo(k as i32, game);
    if ((*E).parameter_2 & 0x8000) != 0 || (*E).base.health >= 0x15e {
        if ((*E).parameter_2 & 0x4000) != 0 || (*E).base.health >= 0x64 {
            CallEnemyPreInstr((((*E).var_F) as u32) | (0xaa0000u32), game);
        } else {
            let mut v2: u16 = if ((*E).parameter_1 & 0x8000) != 0 {
                addr_kTorizo_Ilist_BD0E as u16
            } else {
                addr_kTorizo_Ilist_C188 as u16
            };
            (*E).var_00 = v2;
            enemy_instruction! {[(*E).base]; instruction = addr_kTorizo_Ilist_B155 as u16, timer = 1}
        }
    } else {
        (*E).var_01 = (*E).base.current_instruction;
        enemy_instruction! {[(*E).base]; instruction = addr_kTorizo_Ilist_B0E5 as u16, timer = 1}
    };
}

macro_rules! torizo_fall_until_timer {
    (
        $enemy:ident, $enemy_index:ident, $game:ident, $facing_left_instruction:ident,
        $facing_right_instruction:ident $(,)?
    ) => {
        let torizo_var_03: u16 = (*$enemy).var_03;
        if torizo_var_03 == 0 || {
            let mut v8: u16 = (torizo_var_03).wrapping_sub(1);
            (*$enemy).var_03 = v8;
            (*$enemy).var_03 != 0
        } {
            let mut v10: u16 = (abs16((*$enemy).var_A)).wrapping_add(1);
            if v10 >= 0x10 {
                v10 = 15;
            }
            if Enemy_MoveDown($enemy_index, (v10 as i16 as i32) << 16, $game) == 0 {
                let mut v11: u16 = if ((*$enemy).parameter_1 & 0x8000) != 0 {
                    addr_off_AAC0F2 as u16
                } else {
                    addr_off_AABC78 as u16
                };
                enemy_instruction! {[(*$enemy).base]; instruction = v11, timer = 1}
                (*$enemy).var_B = 256;
                (*$enemy).var_A = 0;
            }
        } else {
            (*$enemy).var_03 = 0;
            let mut v9: u16 = if ((*$enemy).parameter_1 & 0x8000) != 0 {
                $facing_left_instruction as u16
            } else {
                $facing_right_instruction as u16
            };
            enemy_instruction! {[(*$enemy).base]; instruction = v9, timer = 1}
        }
    };
}

pub unsafe fn Torizo_Func_5(k: u16, game: &mut Game) {
    let E = Get_Torizo(k as i32, game);
    if ((*E).parameter_2 & 0x4000) != 0 {
        torizo_fall_until_timer!(E, k, game, addr_kTorizo_Ilist_BD0E, addr_kTorizo_Ilist_C188);
    } else {
        torizo_fall_until_timer!(E, k, game, addr_kTorizo_Ilist_B962, addr_kTorizo_Ilist_BDD8);
    };
}
pub unsafe fn Torizo_Func_6(k: u16, game: &mut Game) {
    Torizo_C643(k, game);
}
pub unsafe fn Torizo_Func_7(k: u16, game: &mut Game) {
    let E = Get_Torizo(k as i32, game);
    Enemy_MoveRight_IgnoreSlopes(k, ((*E).var_A as i16 as i32) << 8, game);
    EnemyFunc_C8AD(k, game);
    if Enemy_MoveDown(k, ((*E).var_B as i16 as i32) << 8, game) != 0 {
        enemy_instruction! {[(*E).base]; instruction = (*E).var_00, timer = 1}
        (*E).var_B = 256;
        word!(game.earthquake_type = 4);
        word!(game.earthquake_timer = 32);
    } else {
        (*E).var_B = ((*E).var_B).wrapping_add((*E).var_C);
    };
}
pub unsafe fn Torizo_Init(game: &mut Game) {
    let E = Get_Torizo(word!(game.cur_enemy_index) as i32, game);
    if CheckBossBitForCurArea(4, game) & 1 != 0 {
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Deleted) as u16);
    } else {
        let v2: i32 = word!(game.area_index) as i32 >> 1;
        (*E).base.properties = ((*E).base.properties) | ((g_word_AAC96B[v2 as usize]) as u16);
        (*E).base.extra_properties = ((*E).base.extra_properties) | (4u16);
        (*E).base.x_width = g_word_AAC96F[v2 as usize] as u16;
        (*E).base.y_height = g_word_AAC973[v2 as usize] as u16;
        (*E).var_E = ((0xaac6bfi32) as u16);
        (*E).base.instruction_timer = 1;
        set_fields! {[(*E).base], 0; timer, palette_index}
        (*E).var_F = addr_locret_AAC95E as u16;
        (*E).base.current_instruction = crate::rtl::rom_table_word!(game, 0xaac967, v2 as isize);
        (*E).base.spritemap_pointer = addr_kTorizo_ExtSprmap_87D0 as u16;
        (*E).base.x_pos = g_word_AAC95F[v2 as usize] as u16;
        (*E).base.y_pos = g_word_AAC963[v2 as usize] as u16;
        (*E).var_A = 0;
        (*E).var_B = 256;
        word_transfers!(game, descending, 16;
            le(target_palettes, 255) <- table(kTorizo_Palettes_1, 15),
            le(target_palettes, 191) <- table(kTorizo_Palette, 15),
        );
        if word!(game.area_index) != 0 {
            Torizo_C280(game);
            if word!(game.joypad1_lastkeys) == 0xc0c0 {
                word! { game, 700; samus_health, samus_max_health }
                word! { game, 300; samus_max_reserve_health, samus_reserve_health }
                word! { game, 100; samus_missiles, samus_max_missiles }
                word! { game, 20;
                    samus_super_missiles, samus_max_super_missiles, samus_power_bombs,
                    samus_max_power_bombs,
                }
                word!(game.equipped_items = -3273i32 as u16);
                word!(game.collected_items = -3273i32 as u16);
                word! { game, 4111; equipped_beams, collected_beams }
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
    if word!(game.area_index) != 0 {
        GoldTorizo_Shot(game);
    } else {
        let E = Get_Torizo(word!(game.cur_enemy_index) as i32, game);
        if (*E).var_04 | (*E).base.flash_timer == 0 {
            NormalEnemyShotAiSkipDeathAnim_CurEnemy(game);
            if (*E).base.health == 0 {
                (*E).var_E = ((0xaac6abi32) as u16);
                enemy_instruction! {[(*E).base]; instruction = addr_kTorizo_Ilist_B1C8 as u16, timer = 1}
                (*E).parameter_2 = ((*E).parameter_2) | (0xc000u16);
                (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
            }
        }
    };
}
pub unsafe fn Torizo_Func_8(game: &mut Game) {
    if word!(game.area_index) != 0 {
        Torizo_D658(game);
    }
}
crate::instruction_callbacks! { pub unsafe fn (k: u16, jp: *const u16, game: &mut Game) -> *const u16;
    Torizo_Instr_39 [] {
        if (*Get_Torizo(k as i32, game)).base.y_pos == 375 {
            jp.offset(1)
        } else {
            *jp as usize as *const u16
        }
    };
    Torizo_Instr_41 [jp] { Torizo_C298(game); };
    Torizo_Instr_42 [jp] {
        QueueMusic_Delayed8(5, game);
        let E = Get_Torizo(k as i32, game);
        (*E).base.x_width = 18;
        (*E).base.y_height = 48;
        SpawnPalfxObject(addr_stru_8DF75D as u16, game);
    };
    Torizo_Instr_48 [jp] {
        let E = Get_Torizo(k as i32, game);
        (*E).parameter_2 = ((*E).parameter_2 & !(0x1000));
    };
    Torizo_Instr_57 [jp] { SpawnEprojWithRoomGfx(addr_stru_86B1C0 as u16, 0, game); };
    Torizo_Instr_58 [*jp as usize as *const u16] {
        let mut v2: i16 = 34;
        while *((ram_ptr!(game, u16; 0x1997) as *mut u8).offset(v2 as u16 as i32 as isize) as *mut u16)
            != 0xb1c0
        {
            v2 = (v2).wrapping_sub(2);
            if v2 < 0 {
                return jp.offset(1);
            }
        }
    };
}
crate::instruction_callbacks! { pub unsafe fn (k: u16, jp: *const u16, game: &mut Game) -> *const u16;
    @prefix { let E = Get_Torizo(k as i32, game); };
    Torizo_Instr_59 [jp] {  (*E).var_06 = ((*E).var_06 & !(0x8000)); };
    Torizo_Instr_62 [jp] {  (*E).var_06 = ((*E).var_06) | (0x8000u16); };
    Torizo_Instr_63 [jp] {  (*E).parameter_2 = ((*E).parameter_2 & !(0x2000)); };
}
pub unsafe fn GoldTorizo_Main(game: &mut Game) {
    let E = Get_Torizo(word!(game.cur_enemy_index) as i32, game);
    if word!(game.samus_pose) as i32 == kPose_1B_FaceR_SpaceJump as i32
        || word!(game.samus_pose) as i32 == kPose_1C_FaceL_SpaceJump as i32
    {
        (*E).var_07 = (*E).var_07.wrapping_add(1);
    } else {
        (*E).var_07 = 0;
    }
    CallTorizoFunc(
        (((*E).var_E) as u32) | (0xaa0000u32),
        word!(game.cur_enemy_index),
        game,
    );
}
crate::instruction_callbacks! { pub unsafe fn (k: u16, jp: *const u16, game: &mut Game) -> *const u16;
    Torizo_Instr_56 [jp] { QueueSfx2_Max6(0x34, game); };
    Torizo_Instr_60 [jp] { QueueSfx2_Max6(0x67, game); };
    Torizo_Instr_46 [jp] { QueueSfx2_Max6(0x48, game); };
}
pub unsafe fn Torizo_Func_12(k: u16, game: &mut Game) -> u16 {
    let E = Get_Torizo(k as i32, game);
    ((*E).parameter_1 as i32 ^ (*E).base.x_pos as i32 - word!(game.samus_x_pos) as i32) as u16
}
pub unsafe fn Torizo_Func_11(k: u16, game: &mut Game) {
    let E = Get_Torizo(k as i32, game);
    SetGoldenTorizoPalette((*E).base.health, game);
}
pub unsafe fn GoldTorizo_Hurt(game: &mut Game) {
    Torizo_C620(word!(game.cur_enemy_index), game);
    if (*gEnemyData(word!(game.cur_enemy_index) as i32, game)).flash_timer & 1 != 0 {
        word_transfers!(game, descending, 16;
            le(palette_buffer, 175) <- value(0x7fff),
            le(palette_buffer, 159) <- value(0x7fff),
        );
    } else {
        Torizo_Func_11(word!(game.cur_enemy_index), game);
    };
}
crate::instruction_callbacks! { pub unsafe fn (k: u16, jp: *const u16, game: &mut Game) -> *const u16;
    Torizo_Instr_47 [jp] {
        SpawnEprojWithRoomGfx(addr_kEproj_GoldenTorizoSuperMissile as u16, 0, game);
    };
    Torizo_Instr_49 [*jp as usize as *const u16] {
        if (Torizo_Func_12(k, game) & 0x8000) == 0
            || CompareDistToSamus_X(k, 4, game) & 1 == 0
            || CompareDistToSamus_X(k, 0x28, game) & 1 != 0
            || word!(game.samus_pose) as i32 != kPose_1D_FaceR_Morphball_Ground as i32
                && word!(game.samus_pose) as i32 != kPose_1E_MoveR_Morphball_Ground as i32
                && word!(game.samus_pose) as i32 != kPose_1F_MoveL_Morphball_Ground as i32
                && word!(game.samus_pose) as i32 != kPose_79_FaceR_Springball_Ground as i32
                && word!(game.samus_pose) as i32 != kPose_7A_FaceL_Springball_Ground as i32
                && word!(game.samus_pose) as i32 != kPose_7B_MoveR_Springball_Ground as i32
                && word!(game.samus_pose) as i32 != kPose_7C_MoveL_Springball_Ground as i32
        {
            return jp.offset(1);
        }
        (*Get_Torizo(k as i32, game)).var_09 = 0;
    };
    Torizo_Instr_61 [jp.offset(1)] {
        SpawnEprojWithRoomGfx(addr_kEproj_GoldenTorizoEyeBeam as u16, *jp, game);
    };
    Torizo_Instr_53 [*jp as usize as *const u16] {
        if (Torizo_Func_12(k, game) & 0x8000) != 0
            || CompareDistToSamus_X(k, 0x20, game) & 1 == 0
            || CompareDistToSamus_X(k, 0x60, game) & 1 != 0
            || (NextRandom(game) & 0x110) != 0
        {
            return jp.offset(1);
        }
        (*Get_Torizo(k as i32, game)).var_00 = (jp.offset(1) as *mut u8)
            .offset_from(RomBankBase((*gEnemyData(k as i32, game)).bank, game))
            as c_long as u16;
    };
    Torizo_Instr_55 [*jp as usize as *const u16] {
        let E = Get_Torizo(k as i32, game);
        if (*E).base.health > 0x788 || (NextRandom(game) & 0x102) != 0 {
            return jp.offset(1) as usize as *const u16;
        }
        (*E).var_09 = 0;
        (*E).var_00 = (jp.offset(1) as *mut u8)
            .offset_from(RomBankBase((*gEnemyData(k as i32, game)).bank, game))
            as c_long as u16;
    };
    Torizo_Instr_52 [*jp as usize as *const u16] {
        let E = Get_Torizo(k as i32, game);
        if (*E).base.health <= 0x2a30 || ((*E).parameter_2 & 0x2000) == 0 {
            return jp.offset(1);
        }
        (*E).var_00 = (jp.offset(1) as *mut u8)
            .offset_from(RomBankBase((*gEnemyData(k as i32, game)).bank, game))
            as c_long as u16;
    };
    Torizo_Instr_50 [*jp as usize as *const u16] {
        if CompareDistToSamus_X(k, 0x70, game) & 1 == 0 {
            return jp.offset(1);
        }
        if (Torizo_Func_12(k, game) & 0x8000) != 0 {
            return jp.offset(1);
        }
        let E = Get_Torizo(k as i32, game);
        if (*E).var_07 <= 0x168
            && ((word!(game.joypad1_lastkeys) & 0x300) == 0 || (NextRandom(game) & 0x101) == 0)
        {
            return jp.offset(1);
        }
        (*E).var_09 = 0;
        Torizo_C20A(k, game);
    };
    Torizo_Instr_43 [jp] {
        SpawnEprojWithGfx(0, k, addr_kEproj_GoldenTorizosChozoOrbs as u16, game);
    };
    Torizo_Instr_51 [*jp as usize as *const u16] {
        let E = Get_Torizo(k as i32, game);
        if (*E).var_09 < 8
            && (CompareDistToSamus_X(k, 0x20, game) & 1 != 0
                || (Torizo_Func_12(k, game) & 0x8000) != 0)
        {
            return jp.offset(1);
        }
        (*E).var_09 = 0;
        Torizo_C22D(k, game);
    };
    Torizo_Instr_45 [] {
        (*Get_Torizo(k as i32, game)).var_00 = (jp.offset(2) as *mut u8)
            .offset_from(RomBankBase((*gEnemyData(k as i32, game)).bank, game))
            as c_long as u16;
        if word!(game.samus_missiles) < 0x20
            || (word!(game.nmi_frame_counter_word) as i32
                + (word!(game.samus_x_pos) as i32 & 1)
                + (word!(game.samus_x_pos) as i32 >> 1)
                & 8)
                != 0
        {
            *jp as usize as *const u16
        } else {
            *jp.offset(1) as usize as *const u16
        }
    };
    Torizo_Instr_54 [] {
        let E = Get_Torizo(k as i32, game);
        (*E).var_A = g_word_AAD59A[(*jp >> 1) as usize] as u16;
        if Enemy_MoveRight_IgnoreSlopes(k, ((*E).var_A as i16 as i32) << 16, game) != 0 {
            (*E).var_03 = 0;
            if ((*E).parameter_1 & 0x8000) != 0 {
                addr_kTorizo_Ilist_D203 as i32 as usize as *const u16
            } else {
                addr_kTorizo_Ilist_D2BF as i32 as usize as *const u16
            }
        } else {
            EnemyFunc_C8AD(k, game);
            if (((*E).parameter_1 as i32
                ^ word!(game.samus_x_pos) as i32 - (*E).base.x_pos as i32)
                & 0x8000)
                == 0
            {
                if (*E).var_03 == 0 {
                    (*E).var_03 = 16;
                }
                jp.offset(1)
            } else {
                jp.offset(1)
            }
        }
    };
}
pub unsafe fn Torizo_Func_9(k: u16, game: &mut Game) {
    let E = Get_Torizo(word!(game.cur_enemy_index) as i32, game);
    if word!(game.samus_y_pos) > 0x140 && word!(game.samus_x_pos) > 0x170 {
        (*E).base.current_instruction = (*E).base.current_instruction.wrapping_add(1);
        enemy_instruction! {[(*E).base]; instruction = (*E).base.current_instruction.wrapping_add(1), timer = 1}
    }
}
pub use self::Torizo_Func_3 as Torizo_Func_10;
pub unsafe fn Torizo_D5E6(k: u16, game: &mut Game) {
    Torizo_C620(k, game);
    let E = Get_Torizo(k as i32, game);
    CallEnemyPreInstr((((*E).var_F) as u32) | (0xaa0000u32), game);
}
pub use self::Torizo_Func_6 as Torizo_D5ED;
pub unsafe fn Torizo_D5F1(k: u16, game: &mut Game) {
    let E = Get_Torizo(k as i32, game);
    torizo_fall_until_timer!(E, k, game, addr_kTorizo_Ilist_D203, addr_kTorizo_Ilist_D2BF);
}
pub unsafe fn Torizo_D658(game: &mut Game) {
    let E = Get_Torizo(word!(game.cur_enemy_index) as i32, game);
    if (*E).base.flash_timer == 0 && (*E).var_04 == 0 {
        Torizo_D6A6(game);
    }
}
pub unsafe fn GoldTorizo_Shot(game: &mut Game) {
    let E = Get_Torizo(word!(game.cur_enemy_index) as i32, game);
    if (*E).base.flash_timer == 0 {
        if (*E).var_04 != 0 {
            Torizo_D6A6(game);
            return;
        }
        let react_to_shot: bool = (((*E).parameter_2 & 0x1000) != 0)
            || ({
                let mut v2: u16 = (2u16).wrapping_mul(word!(game.collision_detection_index));
                let mut v3: u16 =
                    (word!(game.projectile_type[word!(game.collision_detection_index)]))
                        & (0xf00u16);
                (*E).var_05 = v3;
                if v3 == 256 {
                    Torizo_D6D1(word!(game.cur_enemy_index), v2, game);
                    return;
                }
                (v3 != 512)
                    || ({
                        Torizo_D6F7(word!(game.cur_enemy_index), v2, game);
                        false
                    })
            });
        if react_to_shot {
            (*E).parameter_2 = ((*E).parameter_2) | (0x2000u16);
            Torizo_D6A6(game);
        }
    }
}
pub unsafe fn Torizo_D6A6(game: &mut Game) {
    NormalEnemyShotAiSkipDeathAnim_CurEnemy(game);
    let E = Get_Torizo(word!(game.cur_enemy_index) as i32, game);
    if (*E).base.health == 0 {
        enemy_instruction! {[(*E).base]; instruction = addr_kTorizo_Ilist_B1C8 as u16, timer = 1}
        (*E).parameter_2 = ((*E).parameter_2) | (0xc000u16);
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
    }
}
pub unsafe fn Torizo_D6D1(k: u16, j: u16, game: &mut Game) {
    word!(game.projectile_dir[(j >> 1)], |fresh1| (fresh1 & !(0x10)));
    let E = Get_Torizo(k as i32, game);
    (*E).var_E = ((0xaac6abi32) as u16);
    (*E).base.instruction_timer = 1;
    let mut v3: u16 = if ((*E).parameter_1 & 0x8000) != 0 {
        addr_kTorizo_Ilist_D2AD as u16
    } else {
        addr_kTorizo_Ilist_D1F1 as u16
    };
    (*E).base.current_instruction = v3;
}
pub unsafe fn Torizo_D6F7(k: u16, j: u16, game: &mut Game) {
    let E = Get_Torizo(word!(game.cur_enemy_index) as i32, game);
    if (Torizo_Func_12(k, game) & 0x8000) == 0 {
        (*E).parameter_2 = ((*E).parameter_2) | (0x1000u16);
        (*E).var_E = ((0xaac6abi32) as u16);
        word!(game.projectile_dir[(j >> 1)], |fresh0| (fresh0) | (0x10u16));
        (*E).base.instruction_timer = 1;
        let mut v3: u16 = if ((*E).parameter_1 & 0x2000) != 0 {
            if ((*E).parameter_1 & 0x8000) != 0 {
                addr_kTorizo_Ilist_CEFF as u16
            } else {
                addr_kTorizo_Ilist_CE43 as u16
            }
        } else if ((*E).parameter_1 & 0x8000) != 0 {
            addr_kTorizo_Ilist_CEA5 as u16
        } else {
            addr_kTorizo_Ilist_CDE1 as u16
        };
        (*E).base.current_instruction = v3;
    } else {
        Torizo_D6A6(game);
    };
}
pub unsafe fn TourianEntranceStatue_Init(game: &mut Game) {
    let v0 = gEnemyData(word!(game.cur_enemy_index) as i32, game);
    (*v0).palette_index = 0;
    (*v0).instruction_timer = 1;
    (*v0).timer = 0;
    let v1: u16 = crate::rtl::rom_table_word!(game, 0xaad810, ((*v0).parameter_1 >> 1) as isize);
    (*v0).current_instruction = v1;
    if (*v0).parameter_1 == 0 {
        SpawnEprojWithRoomGfx(addr_kEproj_TourianStatueBaseDecoration as u16, 0, game);
        SpawnEprojWithRoomGfx(addr_kEproj_TourianStatueRidley as u16, 0, game);
        SpawnEprojWithRoomGfx(addr_kEproj_TourianStatuePhantoon as u16, 0, game);
    }
    word_transfers!(game, descending, 16;
        le(target_palettes, 255) <- rom(0xaad785, 15),
        le(target_palettes, 175) <- rom(0xaad765, 15),
    );
}
crate::instruction_callbacks! { pub unsafe fn (k: u16, jp: *const u16, game: &mut Game) -> *const u16;
    @prefix { let E = Get_Shaktool(k as i32, game); };
    Shaktool_Instr_2 [Shaktool_D956(k, jp, ((*EX).var_D) ^ (0x80u16), game)] {

        let EX = Get_Shaktool((*E).var_E as i32 + 192, game);
    };
    Shaktool_Instr_3 [Shaktool_D956(k, jp, (*EX).var_D, game)] {

        let EX = Get_Shaktool((*E).var_E as i32 + 192, game);
    };
    Shaktool_Instr_4 [Shaktool_D956(
            k,
            jp,
            (*((&raw mut (*E).var_A as *mut u8).offset(1) as *mut u16)) ^ (0x80u16),
            game,
        )] {
    };
    Shaktool_Instr_5 [Shaktool_D956(
            k,
            jp,
            *((&raw mut (*E).var_A as *mut u8).offset(1) as *mut u16),
            game,
        )] {
    };
}
pub unsafe fn Shaktool_D956(k: u16, jp: *const u16, a: u16, game: &mut Game) -> *const u16 {
    let E = Get_Shaktool(k as i32, game);
    let v3: i32 = a as u8 as i32;
    words!(
        (*E).base,
        x_pos,
        x_subpos += ((kSinCosTable8bit_Sext[(v3 + 64) as usize] as i32) << 8) as u32
    );
    words!(
        (*E).base,
        y_pos,
        y_subpos += ((kSinCosTable8bit_Sext[v3 as usize] as i32) << 8) as u32
    );
    jp
}
pub use self::Torizo_Instr_7 as Shaktool_Instr_6;
pub unsafe fn Shaktool_Instr_1(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let shakt_var_E: u16 = (*Get_Shaktool(k as i32, game)).var_E;
    (*Get_Shaktool(shakt_var_E as i32, game)).var_F =
        crate::rtl::rom_table_word!(game, 0xaadedb, 0);
    (*Get_Shaktool(shakt_var_E as i32 + 64i32, game)).var_F =
        crate::rtl::rom_table_word!(game, 0xaadedb, 1);
    (*Get_Shaktool(shakt_var_E as i32 + 128i32, game)).var_F =
        crate::rtl::rom_table_word!(game, 0xaadedb, 2);
    (*Get_Shaktool(shakt_var_E as i32 + 192i32, game)).var_F =
        crate::rtl::rom_table_word!(game, 0xaadedb, 3);
    (*Get_Shaktool(shakt_var_E as i32 + 256i32, game)).var_F =
        crate::rtl::rom_table_word!(game, 0xaadedb, 4);
    (*Get_Shaktool(shakt_var_E as i32 + 320i32, game)).var_F =
        crate::rtl::rom_table_word!(game, 0xaadedb, 5);
    (*Get_Shaktool(shakt_var_E as i32 + 384i32, game)).var_F =
        crate::rtl::rom_table_word!(game, 0xaadedb, 6);
    jp
}
pub unsafe fn Shaktool_DB0E(k: u16, a: u16, game: &mut Game) {
    let shakt_var_E: u16 = (*Get_Shaktool(k as i32, game)).var_E;
    (*Get_Shaktool(shakt_var_E as i32, game)).parameter_1 = a;
    (*Get_Shaktool(shakt_var_E as i32 + 64i32, game)).parameter_1 = a;
    (*Get_Shaktool(shakt_var_E as i32 + 128i32, game)).parameter_1 = a;
    (*Get_Shaktool(shakt_var_E as i32 + 192i32, game)).parameter_1 = a;
    (*Get_Shaktool(shakt_var_E as i32 + 256i32, game)).parameter_1 = a;
    (*Get_Shaktool(shakt_var_E as i32 + 320i32, game)).parameter_1 = a;
    (*Get_Shaktool(shakt_var_E as i32 + 384i32, game)).parameter_1 = a;
}
pub unsafe fn Shaktool_DB27(k: u16, a: u16, game: &mut Game) {
    let shakto_var_E: u16 = (*Get_Shaktool(k as i32, game)).var_E;
    (*Get_Shaktool(shakto_var_E as i32, game)).var_A = a;
    (*Get_Shaktool(shakto_var_E as i32 + 64i32, game)).var_A = a;
    (*Get_Shaktool(shakto_var_E as i32 + 128i32, game)).var_A = a;
    (*Get_Shaktool(shakto_var_E as i32 + 192i32, game)).var_A = a;
    (*Get_Shaktool(shakto_var_E as i32 + 256i32, game)).var_A = a;
    (*Get_Shaktool(shakto_var_E as i32 + 320i32, game)).var_A = a;
    (*Get_Shaktool(shakto_var_E as i32 + 384i32, game)).var_A = a;
}
pub unsafe fn Shaktool_DB40(k: u16, a: u16, game: &mut Game) {
    let shakto_var_E: u16 = (*Get_Shaktool(k as i32, game)).var_E;
    (*Get_Shaktool(shakto_var_E as i32, game)).var_B = a;
    (*Get_Shaktool(shakto_var_E as i32 + 64i32, game)).var_B = a;
    (*Get_Shaktool(shakto_var_E as i32 + 128i32, game)).var_B = a;
    (*Get_Shaktool(shakto_var_E as i32 + 192i32, game)).var_B = a;
    (*Get_Shaktool(shakto_var_E as i32 + 256i32, game)).var_B = a;
    (*Get_Shaktool(shakto_var_E as i32 + 320i32, game)).var_B = a;
    (*Get_Shaktool(shakto_var_E as i32 + 384i32, game)).var_B = a;
}
pub unsafe fn Shaktool_DB59(k: u16, game: &mut Game) {
    let mut shakto_var_E: u16 = (*Get_Shaktool(k as i32, game)).var_E;
    let Shaktool = Get_Shaktool(shakto_var_E as i32 + 192, game);
    (*Shaktool).var_D = ((*Shaktool).var_D) ^ (0x8000u16);
    let v3 = Get_Shaktool(shakto_var_E as i32 + 384, game);
    let mut shakto_var_B: u16 = (*v3).var_B;
    let v5 = Get_Shaktool(shakto_var_E as i32 + 64, game);
    (*v3).var_B = (((*v5).var_B ^ 0x8000) & 0xff00);
    (*v5).var_B = ((shakto_var_B ^ 0x8000) & 0xff00);
    let v6 = Get_Shaktool(shakto_var_E as i32 + 320, game);
    let mut v7: u16 = (*v6).var_B;
    let v8 = Get_Shaktool(shakto_var_E as i32 + 128, game);
    (*v6).var_B = (((*v8).var_B ^ 0x8000) & 0xff00);
    (*v8).var_B = ((v7 ^ 0x8000) & 0xff00);
    let v9 = Get_Shaktool(shakto_var_E as i32 + 256, game);
    let mut v10: u16 = (*v9).var_B;
    let v11 = Get_Shaktool(shakto_var_E as i32 + 192, game);
    (*v9).var_B = (((*v11).var_B ^ 0x8000) & 0xff00);
    (*v11).var_B = ((v10 ^ 0x8000) & 0xff00);
    let v12 = Get_Shaktool(shakto_var_E as i32 + 384, game);
    let mut x_subpos: u16 = (*v12).base.x_subpos;
    let v14 = Get_Shaktool(shakto_var_E as i32, game);
    (*v12).base.x_subpos = (*v14).base.x_subpos;
    (*v14).base.x_subpos = x_subpos;
    crate::update_place! { previous mut y_subpos: u16 = (*v12).base.y_subpos => (*v14).base.y_subpos; }
    (*v14).base.y_subpos = y_subpos;
    crate::update_place! { previous mut x_pos: u16 = (*v12).base.x_pos => (*v14).base.x_pos; }
    (*v14).base.x_pos = x_pos;
    crate::update_place! { previous mut y_pos: u16 = (*v12).base.y_pos => (*v14).base.y_pos; }
    (*v14).base.y_pos = y_pos;
    (*Get_Shaktool(shakto_var_E as i32 + 64i32, game))
        .base
        .x_subpos = 0x8000;
    (*Get_Shaktool(shakto_var_E as i32 + 128i32, game))
        .base
        .x_subpos = 0x8000;
    (*Get_Shaktool(shakto_var_E as i32 + 192i32, game))
        .base
        .x_subpos = 0x8000;
    (*Get_Shaktool(shakto_var_E as i32 + 256i32, game))
        .base
        .x_subpos = 0x8000;
    (*Get_Shaktool(shakto_var_E as i32 + 320i32, game))
        .base
        .x_subpos = 0x8000;
    (*Get_Shaktool(shakto_var_E as i32 + 64i32, game))
        .base
        .y_subpos = 0x8000;
    (*Get_Shaktool(shakto_var_E as i32 + 128i32, game))
        .base
        .y_subpos = 0x8000;
    (*Get_Shaktool(shakto_var_E as i32 + 192i32, game))
        .base
        .y_subpos = 0x8000;
    (*Get_Shaktool(shakto_var_E as i32 + 256i32, game))
        .base
        .y_subpos = 0x8000;
    (*Get_Shaktool(shakto_var_E as i32 + 320i32, game))
        .base
        .y_subpos = 0x8000;
}
pub unsafe fn Shaktool_DC07(k: u16, game: &mut Game) {
    let E = Get_Shaktool(k as i32, game);
    let mut v2: u16 = if ((*E).parameter_1 & 0x8000) != 0 {
        ((*E).var_B).wrapping_sub((*E).var_A)
    } else {
        ((*E).var_A).wrapping_sub((*E).var_B)
    };
    (*E).var_C = (4u16).wrapping_mul((byte!(v2; 1)) as u16);
}
pub unsafe fn Shaktool_DC2A(k: u16, game: &mut Game) {
    let E = Get_Shaktool(k as i32, game);
    let v3: i32 = byte!((*E).var_B; 1) as i32;
    let v5: i32 = k as i32 >> 1;
    words!(
        (*E).base,
        x_pos,
        x_subpos = (((crate::rtl::rom_table_word!(game, 0xaae0bd, v3 as isize) as i16 as i32)
            << 8i32) as u32)
            .wrapping_add(words!((
                word!(game.enemy_drawing_queue[(v5 + 91)]),
                word!(game.enemy_drawing_queue[(v5 + 92)])
            )),)
    );
    words!(
        (*E).base,
        y_pos,
        y_subpos = (((crate::rtl::rom_table_word!(game, 0xaae03d, v3 as isize) as i16 as i32)
            << 8i32) as u32)
            .wrapping_add(words!((
                word!(game.enemy_drawing_queue[(v5 + 93)]),
                word!(game.enemy_drawing_queue[(v5 + 94)])
            )),)
    );
}
pub unsafe fn Shaktool_DC6F(k: u16, game: &mut Game) {
    let E = Get_Shaktool(k as i32, game);
    Shaktool_DB40(k, (*E).var_A, game);
    let shakto_var_E: u16 = (*E).var_E;
    (*Get_Shaktool(shakto_var_E as i32, game)).var_C = g_word_AADEE9[0i32 as usize];
    (*Get_Shaktool(shakto_var_E as i32 + 64i32, game)).var_C = g_word_AADEE9[1i32 as usize];
    (*Get_Shaktool(shakto_var_E as i32 + 128i32, game)).var_C = g_word_AADEE9[2i32 as usize];
    (*Get_Shaktool(shakto_var_E as i32 + 192i32, game)).var_C = g_word_AADEE9[3i32 as usize];
    (*Get_Shaktool(shakto_var_E as i32 + 256i32, game)).var_C = g_word_AADEE9[4i32 as usize];
    (*Get_Shaktool(shakto_var_E as i32 + 320i32, game)).var_C = g_word_AADEE9[5i32 as usize];
    (*Get_Shaktool(shakto_var_E as i32 + 384i32, game)).var_C = g_word_AADEE9[6i32 as usize];
}
pub unsafe fn Shaktool_Hurt(game: &mut Game) {
    let E = Get_Shaktool(word!(game.cur_enemy_index) as i32, game);
    CallEnemyPreInstr((((*E).var_F) as u32) | (0xaa0000u32), game);
}
pub unsafe fn Shaktool_DCAC(k: u16, game: &mut Game) {
    Shaktool_DC2A(k, game);
    let E = Get_Shaktool(k as i32, game);
    let mut shakto_var_C: u16 = if ((*E).parameter_1 & 0x4000) != 0 {
        (*E).var_A = ((*E).var_A).wrapping_add(256);
        256
    } else {
        (*E).var_C
    };
    (*E).var_B = ((*E).var_B as i32
        + if (*E).parameter_1 & 0x8000 != 0 {
            -(shakto_var_C as i32)
        } else {
            shakto_var_C as i32
        }) as u16;
}
pub unsafe fn Shaktool_DCD7(k: u16, game: &mut Game) {
    Shaktool_DCAC(k, game);
    let E = Get_Shaktool(k as i32, game);
    let r18: u16 = ((*E).var_B) ^ (0x8000u16);
    let mut v2: i16 = (r18 as i32
        + ((((*Get_Shaktool(k as i32 + 64i32, game)).var_B).wrapping_sub(r18)) as i32 >> 1))
        as i16;
    if ((*E).var_D & 0x8000) != 0 {
        crate::update_place! { reference fresh2 = *(&raw mut v2 as *mut u8).offset(1) => (*fresh2) ^ (0x80u8); }
    }
    let v3: u16 = (byte!(v2; 1) as i32 + 8 & 0xe0) as u16;
    byte!((*E).var_D; 0) = v3 as u8;
    enemy_instruction! {[(*E).base]; instruction = crate::rtl::rom_table_word!(
        game,
        0xaadd15,
        ((v3 >> 4) >> 1) as isize
    ), timer = 1}
}
pub unsafe fn Shaktool_DD25(mut k: u16, game: &mut Game) {
    let E = Get_Shaktool(k as i32, game);
    let y_pos: u16 = (*E).base.y_pos;
    let x_pos: u16 = (*E).base.x_pos;
    Shaktool_DCAC(k, game);
    let v12: u16 = (*E).base.y_pos;
    crate::update_place! { previous v11: u16 = (*E).base.x_pos => x_pos; }
    (*E).base.y_pos = y_pos;
    if Enemy_MoveRight_IgnoreSlopes(
        k,
        ((((v11) as i16).wrapping_sub((x_pos) as i16)) as i32) << 16,
        game,
    ) != 0
        || {
            (*E).base.y_pos = y_pos;
            Enemy_MoveDown(
                k,
                ((((v12) as i16).wrapping_sub((y_pos) as i16)) as i32) << 16,
                game,
            ) != 0
        }
    {
        if ((*E).parameter_1 & 0x2000) != 0 {
            Shaktool_DB0E(k, (((*E).parameter_1 ^ 0x8000) & 0x8fff), game);
        } else {
            (*E).base.x_pos = x_pos;
            (*E).base.y_pos = y_pos;
            Shaktool_DB59(k, game);
            Shaktool_DB0E(
                word!(game.cur_enemy_index),
                ((*E).parameter_1) | (0x2000u16),
                game,
            );
            Shaktool_DB0E(
                word!(game.cur_enemy_index),
                ((*E).parameter_1) & (0xbfffu16),
                game,
            );
        }
        (*E).var_D = 0;
        let v5: u16 = CalculateAngleOfEnemyXfromEnemyY(k, (*E).var_E, game);
        let mut v7: i16 = ((v5 as i32) << 8) as i16;
        let mut v8: u16 = if ((*E).parameter_1 & 0x8000) != 0 {
            ((v7) as u16).wrapping_sub(0x4000)
        } else {
            ((v7) as u16).wrapping_add(0x4000)
        };
        Shaktool_DB27(k, v8, game);
        let mut i: i32 = 12;
        while i >= 0 {
            Shaktool_DC07(k, game);
            let EK = Get_Shaktool(k as i32, game);
            (*EK).var_F = ((0xaadcaai32) as u16);
            enemy_instruction! {[(*EK).base]; instruction = crate::rtl::rom_table_word!(game, 0xaadf13, (i >> 1) as isize), timer = 1}
            k = (k).wrapping_sub(64);
            i -= 2;
        }
    } else {
        (*E).base.x_pos = v11;
        (*E).base.y_pos = v12;
        if ((*E).parameter_1 & 0x4000) != 0 {
            (*E).var_A = ((*E).var_A).wrapping_add(256);
        } else {
            if (((*E).var_A ^ (*E).var_B) & 0xff00) == 0 {
                Shaktool_DC6F(k, game);
                (*E).var_D = 30720;
                Shaktool_DB0E(k, ((*E).parameter_1) & (0xdfffu16), game);
                (*E).var_D = ((byte!((*E).var_D; 1) as i32) << 8) as u16;
            }
            let v2: u16 = ((*E).var_D).wrapping_add((*E).var_C);
            (*E).var_D = v2;
            if v2 >= 0xf000 {
                Shaktool_DB0E(k, ((*E).parameter_1) | (0x4000u16), game);
            }
        }
    };
}
pub unsafe fn Shaktool_Init(game: &mut Game) {
    let E = Get_Shaktool(word!(game.cur_enemy_index) as i32, game);
    (*E).base.instruction_timer = 1;
    (*E).base.timer = 0;
    set_fields! {[(*E)], 0; var_A, var_D}
    let shakto_parameter_2: u16 = (*E).parameter_2;
    let v3: i32 = shakto_parameter_2 as i32 >> 1;
    (*E).base.properties =
        ((*E).base.properties) | (crate::rtl::rom_table_word!(game, 0xaade95, v3 as isize));
    (*E).var_E = (word!(game.cur_enemy_index)).wrapping_sub(crate::rtl::rom_table_word!(
        game,
        0xaadea3,
        v3 as isize
    ));
    (*E).var_F = *((rom_ptr!(game, u16; 0xaadedb) as *mut u8)
        .offset(shakto_parameter_2 as i32 as isize) as *mut u16);
    (*E).var_C = (*((&raw const g_word_AADEE9 as *const u16 as *mut u8)
        .offset(shakto_parameter_2 as i32 as isize) as *mut u16))
        .wrapping_sub(crate::rtl::rom_table_word!(game, 0xaadef7, v3 as isize));
    (*E).var_B = crate::rtl::rom_table_word!(game, 0xaadeb1, v3 as isize);
    (*E).base.current_instruction = crate::rtl::rom_table_word!(game, 0xaadeb1, (v3 + 7) as isize);
    (*E).base.layer = crate::rtl::rom_table_word!(game, 0xaadecd, v3 as isize);
    if shakto_parameter_2 != 0 {
        Shaktool_DC2A(word!(game.cur_enemy_index), game);
    }
}
pub unsafe fn Shaktool_Touch(game: &mut Game) {
    NormalEnemyTouchAi(game);
}
pub unsafe fn Shaktool_Shot(game: &mut Game) {
    NormalEnemyShotAi(game);
    let E = Get_Shaktool(word!(game.cur_enemy_index) as i32, game);
    if (*E).base.health == 0 {
        let shakto_var_E: u16 = (*E).var_E;
        (*Get_Shaktool(shakto_var_E as i32, game)).base.properties = 512;
        (*Get_Shaktool(shakto_var_E as i32 + 64i32, game))
            .base
            .properties = 512;
        (*Get_Shaktool(shakto_var_E as i32 + 128i32, game))
            .base
            .properties = 512;
        (*Get_Shaktool(shakto_var_E as i32 + 192i32, game))
            .base
            .properties = 512;
        (*Get_Shaktool(shakto_var_E as i32 + 256i32, game))
            .base
            .properties = 512;
        (*Get_Shaktool(shakto_var_E as i32 + 320i32, game))
            .base
            .properties = 512;
        (*Get_Shaktool(shakto_var_E as i32 + 384i32, game))
            .base
            .properties = 512;
    }
}
crate::instruction_callbacks! { pub unsafe fn (k: u16, jp: *const u16, game: &mut Game) -> *const u16;
    Shaktool_Instr_9 [jp] {
        word!(game.fx_timer = 32);
        word!(game.fx_y_vel = 64);
    };
    Shaktool_Instr_11 [jp] { word!(game.fx_base_y_pos = 722); };
    Shaktool_Instr_10 [jp] { CallSomeSamusCode(1, game); };
}
pub unsafe fn sub_AAE445(k: u16, game: &mut Game) {
    let v1 = gEnemyData(k as i32, game);
    if (*v1).parameter_1 != 0 {
        enemy_instruction! {[(*v1)]; instruction = addr_kShaktool_Ilist_E3A7 as u16, timer = 1}
    }
}
crate::instruction_callbacks! { pub unsafe fn (k: u16, jp: *const u16, game: &mut Game) -> *const u16;
    Shaktool_Instr_8 [jp] { QueueSfx2_Max6(0x1c, game); };
    Shaktool_Instr_13 [jp] { QueueSfx2_Max6(0x4b, game); };
    Shaktool_Instr_12 [jp.offset(1)] {
        let e = Get_Shaktool(k as i32, game);
        let arg: u16 = *jp;
        CalculateBlockContainingPixelPos(
            ((*e).base.x_pos).wrapping_add(arg),
            ((*e).base.y_pos).wrapping_add(28),
            game,
        );
        if (*(ram_ptr!(game, u16; 0x10002)).offset(
            (word!(game.room_width_in_blocks) as i32 + word!(game.cur_block_index) as i32)
                as isize,
        ) & 0xf000)
            == 0xa000
        {
            SpawnPLM(addr_kPlmHeader_D113 as u16, game);
            SpawnEprojWithRoomGfx(
                addr_kEproj_WreckedShipChozoSpikeFootsteps_1 as u16,
                arg,
                game,
            );
        }
    };
    Shaktool_Instr_7 [jp.offset(1)] {
        let E = Get_Shaktool(k as i32, game);
        (*E).var_C = *jp;
        Enemy_MoveRight_IgnoreSlopes(
            k,
            (crate::rtl::rom_table_word!(game, 0xaae630, ((*E).var_C >> 1) as isize) as i16 as i32)
                << 8,
            game,
        );
        Enemy_MoveDown(
            k,
            (abs16(crate::rtl::rom_table_word!(
                game,
                0xaae630,
                ((*E).var_C >> 1) as isize
            )) as i16 as i32)
                << 8,
            game,
        );
        EnemyFunc_C8AD(k, game);
        let v6: i32 = (*E).var_C as i32 >> 1;
        word!(
            game.samus_x_pos = (crate::rtl::rom_table_word!(game, 0xaae670, v6 as isize))
                .wrapping_add((*E).base.x_pos)
        );
        word!(
            game.samus_y_pos = (crate::rtl::rom_table_word!(game, 0xaae6b0, v6 as isize))
                .wrapping_add((*E).base.y_pos)
        );
    };
    Shaktool_Instr_14 [jp] {
        CallSomeSamusCode(1, game);
        *(ram_ptr!(game, u16; 0xcd20, 6)) = 0;
        *(ram_ptr!(game, u16; 0xcd20, 8)) = 0;
        *(ram_ptr!(game, u16; 0xcd20, 9)) = 0;
        *(ram_ptr!(game, u16; 0xcd20, 13)) = 1;
        SpawnHardcodedPlm(
            SpawnHardcodedPlmArgs::new(0x17, 0x1d, 0xd6fc as VoidP),
            game,
        );
    };
}
pub unsafe fn N00bTubeCracks_Init(game: &mut Game) {
    word_transfers!(game, descending, 32; le(target_palettes, 175) <- rom(0xaae2dd, 31));
}
pub unsafe fn ChozoStatue_Init(game: &mut Game) {
    let v0 = gEnemyData(word!(game.cur_enemy_index) as i32, game);
    (*v0).properties = ((*v0).properties as i32
        | (kEnemyProps_DisableSamusColl as i32 | kEnemyProps_ProcessedOffscreen as i32 | 0x8000))
        as u16;
    (*v0).spritemap_pointer = addr_kSpritemap_Nothing_AA as u16;
    (*v0).instruction_timer = 1;
    (*v0).timer = 0;
    (*v0).ai_preinstr = ((0xaae7a6i32) as u16);
    set_fields! {[(*v0)], 0; parameter_1, palette_index}
    (*gEnemyData(0i32, game)).layer = 0;
    let parameter_2: u16 = (*v0).parameter_2;
    (*v0).current_instruction =
        crate::rtl::rom_table_word!(game, 0xaae7a2, (parameter_2 >> 1) as isize);
    if parameter_2 != 0 {
        sub_AAE784(game);
    } else {
        word_transfers!(game, descending, 16;
            le(target_palettes, 175) <- table(kChozoStatue_Palettes, 15),
            le(target_palettes, 159) <- table(kChozoStatue_Palette, 15),
        );
        SpawnHardcodedPlm(
            SpawnHardcodedPlmArgs::new(0x4a, 0x17, 0xd6ee as VoidP),
            game,
        );
        SpawnHardcodedPlm(
            SpawnHardcodedPlmArgs::new(0x17, 0x1d, 0xd6fc as VoidP),
            game,
        );
    };
}
pub unsafe fn sub_AAE784(game: &mut Game) {
    word_transfers!(game, descending, 16;
        le(target_palettes, 175) <- table(kChozoStatue_Palettes3, 15),
        le(target_palettes, 159) <- table(kChozoStatue_Palettes2, 15),
    );
    SpawnHardcodedPlm(SpawnHardcodedPlmArgs::new(0xc, 0x1d, 0xd6d6 as VoidP), game);
}
pub unsafe fn ChozoStatue_Main(game: &mut Game) {
    let v0 = gEnemyData(word!(game.cur_enemy_index) as i32, game);
    CallEnemyPreInstr((((*v0).ai_preinstr) as u32) | (0xaa0000u32), game);
}
pub unsafe fn Shaktool_PreInstr_0(k: u16, game: &mut Game) {
    if (*(ram_ptr!(game, u16; 0xd828, word!(game.area_index) as isize)) & 1) != 0 {
        let E = Get_Shaktool(k as i32, game);
        if (*E).parameter_1 != 0 {
            enemy_instruction! {[(*E).base]; instruction = addr_kShaktool_Ilist_E461 as u16, timer = 1}
            (*E).var_A = -256i32 as u16;
            (*E).var_B = 256;
        }
    }
}
