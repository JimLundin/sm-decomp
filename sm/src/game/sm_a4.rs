pub use crate::game::constants::bank_86::*;
pub use crate::game::constants::bank_a4::*;
pub use crate::game::constants::enemy_properties::*;
use crate::game::enemy_layouts::enemy_layout;
use crate::game::sm_80::{QueueMusic_Delayed8, QueueSfx2_Max6, QueueSfx3_Max6};
use crate::game::sm_84::SpawnHardcodedPlm;
use crate::game::sm_86::{ClearEprojs, SpawnEprojWithGfx, SpawnEprojWithRoomGfx};
use crate::game::sm_88::SpawnHdmaObject;
use crate::game::sm_90::DisableMinimapAndMarkBossRoomAsExplored;
use crate::game::sm_a0::{
    Enemy_ItemDrop_Crocomire, Enemy_MoveRight_IgnoreSlopes, NormalEnemyFrozenAI, NormalEnemyShotAi,
    NormalEnemyTouchAi, SamusLatchesOnWithGrapple,
};
use crate::game::sm_b4::CreateSpriteAtPos;
use crate::game_state::Game;
use crate::ram_map::{ram_word, set_ram_word, update_ram_word};
use crate::rtl::abs16;
use crate::rtl::RomPtr;
use crate::rtl::{AddToHiLo, SetHiLo};
extern "C" {
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: u32,
        __function: *const ::core::ffi::c_char,
    ) -> !;

}
pub use crate::types::Consts_A4;
pub use crate::types::SpawnHardcodedPlmArgs;
pub use crate::types::{Consts_86, Ram4000, Ram4000_Backups};
pub use crate::types::{
    EnemyBase, EnemyData, EnemyProps, Func_V, LongPtr, Point16U, SpawnHdmaObject_Args, VoidP,
    VramWriteEntry,
};
enemy_layout! {
    Enemy_Crocomire {
        base: EnemyBase,
        words: [
            crocom_var_A, crocom_var_B, crocom_var_C, crocom_var_D, crocom_var_E,
            crocom_var_F, crocom_parameter_1, crocom_parameter_2,
        ],
        pad1: [u8; 26696],
        words: [
            crocom_var_00, crocom_var_01, crocom_var_02, crocom_var_03, crocom_var_04,
            crocom_var_05, crocom_var_06, crocom_var_07, crocom_var_08, crocom_var_09,
            crocom_var_0A, crocom_var_0B, crocom_var_0C, crocom_var_0D, crocom_var_0E,
            crocom_var_0F, crocom_var_10, crocom_var_11, crocom_var_12, crocom_var_13,
            crocom_var_14, crocom_var_15, crocom_var_16, crocom_var_17, crocom_var_18,
            crocom_var_19, crocom_var_1A, crocom_var_1B, crocom_var_1C, crocom_var_1D,
            crocom_var_1E, crocom_var_1F,
        ],
        pad2: [u8; 1984],
        words: [
            crocom_var_20, crocom_var_21, crocom_var_22, crocom_var_23, crocom_var_24,
            crocom_var_25, crocom_var_26, crocom_var_27, crocom_var_28, crocom_var_29,
            crocom_var_2A, crocom_var_2B, crocom_var_2C, crocom_var_2D, crocom_var_2E,
            crocom_var_2F, crocom_var_30, crocom_var_31, crocom_var_32, crocom_var_33,
            crocom_var_34, crocom_var_35, crocom_var_36, crocom_var_37, crocom_var_38,
            crocom_var_39, crocom_var_3A, crocom_var_3B, crocom_var_3C, crocom_var_3D,
            crocom_var_3E, crocom_var_3F,
        ],
    }
}

pub use crate::types::CrocomireFunc;
#[inline]
unsafe fn RomPtr_A4(addr: u16, game: &mut Game) -> *const u8 {
    RomPtr((0xa40000u32) | ((addr) as u32), game)
}
#[inline]
unsafe fn RomPtrWithBank(bank: u8, addr: u16, game: &mut Game) -> *const u8 {
    RomPtr(((bank as i32) << 16i32 | addr as i32) as u32, game)
}
#[inline]
unsafe fn RomBankBase(bank: u8, game: &mut Game) -> *const u8 {
    RomPtr((((bank as i32) << 16i32) + 0x8000i32) as u32, game).offset(-(0x8000i32 as isize))
}
#[inline]
unsafe fn gVramWriteEntry(a: i32, game: &mut Game) -> *mut VramWriteEntry {
    (&raw mut game.g_ram as *mut u8).offset((0xd0i32 + a) as isize) as *mut u8
        as *mut VramWriteEntry
}
#[inline]
unsafe fn gEnemyData(o: i32, game: &mut Game) -> *mut EnemyData {
    (&raw mut game.g_ram as *mut u8)
        .offset(0xf78)
        .offset(o as isize) as *mut EnemyData
}
#[inline]
unsafe fn Get_Crocomire(o: i32, game: &mut Game) -> *mut Enemy_Crocomire {
    (&raw mut game.g_ram as *mut u8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_Crocomire
}
static g_word_A48692: u16 = 8u16;
static g_word_A48694: u16 = 8u16;
static g_word_A48698: u16 = 2u16;
static g_word_A4869A: u16 = 1u16;
static g_word_A4869C: u16 = 3u16;
static g_word_A4869E: u16 = 3u16;
static g_word_A486A0: u16 = 8u16;
static g_word_A486A2: u16 = 0x300u16;
static g_word_A486A4: u16 = 0x640u16;
pub unsafe fn Enemy_GrappleReact_SamusLatchesOn_A4(game: &mut Game) {
    SamusLatchesOnWithGrapple(game);
}
pub unsafe fn Enemy_NormalTouchAI_A4(game: &mut Game) {
    NormalEnemyTouchAi(game);
}
pub unsafe fn Enemy_NormalShotAI_A4(game: &mut Game) {
    NormalEnemyShotAi(game);
}
pub unsafe fn Enemy_NormalFrozenAI_A4(game: &mut Game) {
    NormalEnemyFrozenAI(game);
}
pub unsafe fn Crocomire_Hurt(game: &mut Game) {
    Crocomire_Func_27(ram_word!(game, cur_enemy_index), game);
    Crocomire_Func_31(game);
}
static off_A486B3: [Option<CrocomireFunc>; 21] = unsafe {
    [
        Some(Crocomire_Func_2 as unsafe fn(u16, *const u16, &mut Game) -> *const u16),
        Some(Crocomire_Func_3 as unsafe fn(u16, *const u16, &mut Game) -> *const u16),
        Some(Crocomire_Func_4 as unsafe fn(u16, *const u16, &mut Game) -> *const u16),
        Some(Crocomire_Func_5 as unsafe fn(u16, *const u16, &mut Game) -> *const u16),
        Some(Crocomire_Func_7 as unsafe fn(u16, *const u16, &mut Game) -> *const u16),
        Some(Crocomire_Func_8 as unsafe fn(u16, *const u16, &mut Game) -> *const u16),
        Some(Crocomire_Func_9 as unsafe fn(u16, *const u16, &mut Game) -> *const u16),
        Some(Crocomire_Func_10 as unsafe fn(u16, *const u16, &mut Game) -> *const u16),
        Some(Crocomire_Func_11 as unsafe fn(u16, *const u16, &mut Game) -> *const u16),
        Some(Crocomire_Func_13 as unsafe fn(u16, *const u16, &mut Game) -> *const u16),
        Some(Crocomire_Func_14 as unsafe fn(u16, *const u16, &mut Game) -> *const u16),
        Some(Crocomire_Func_15 as unsafe fn(u16, *const u16, &mut Game) -> *const u16),
        Some(Crocomire_Func_16 as unsafe fn(u16, *const u16, &mut Game) -> *const u16),
        Some(Crocomire_Func_17 as unsafe fn(u16, *const u16, &mut Game) -> *const u16),
        Some(Crocomire_Func_18 as unsafe fn(u16, *const u16, &mut Game) -> *const u16),
        Some(Crocomire_Func_21 as unsafe fn(u16, *const u16, &mut Game) -> *const u16),
        Some(Crocomire_Func_22 as unsafe fn(u16, *const u16, &mut Game) -> *const u16),
        Some(Crocomire_Func_23 as unsafe fn(u16, *const u16, &mut Game) -> *const u16),
        Some(Crocomire_Func_24 as unsafe fn(u16, *const u16, &mut Game) -> *const u16),
        Some(Crocomire_Func_25 as unsafe fn(u16, *const u16, &mut Game) -> *const u16),
        Some(Crocomire_Func_26 as unsafe fn(u16, *const u16, &mut Game) -> *const u16),
    ]
};
pub unsafe fn Crocomire_Instr_1(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let crocom_var_C: u16 =
        (*Get_Crocomire(ram_word!(game, cur_enemy_index) as i32, game)).crocom_var_C;
    off_A486B3[(crocom_var_C as i32 >> 1i32) as usize].expect("non-null function pointer")(
        crocom_var_C,
        jp,
        game,
    )
}
pub unsafe fn Crocomire_Func_2(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*Get_Crocomire(k as i32, game)).base.instruction_timer = 1u16;
    addr_kCrocomire_Ilist_BADE as i32 as usize as *const u16
}
pub unsafe fn Crocomire_Func_3(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*Get_Crocomire(0i32, game)).crocom_var_C = 4u16;
    addr_kCrocomire_Ilist_BBCE as i32 as usize as *const u16
}
pub unsafe fn Crocomire_Func_4(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E: *mut Enemy_Crocomire = Get_Crocomire(0i32, game);
    let v1: u16 = abs16(((*E).base.x_pos).wrapping_sub(ram_word!(game, samus_x_pos)));
    if v1 as i32 - 224i32 & 0x8000i32 != 0 {
        (*E).crocom_var_B = ((*E).crocom_var_B) | (0x8000u16);
        (*E).crocom_var_C = 18u16;
        return addr_kCrocomire_Ilist_BC56 as i32 as usize as *const u16;
    }
    jp
}
pub unsafe fn Crocomire_Func_5(k: u16, mut jp: *const u16, game: &mut Game) -> *const u16 {
    let E: *mut Enemy_Crocomire = Get_Crocomire(0i32, game);
    if ((*E).crocom_var_B as i32 & 0x800i32) != 0i32 && {
        (*E).crocom_var_B = ((*E).crocom_var_B as i32 & !(0x800i32)) as u16;
        (*E).crocom_var_D != 0u16
    } {
        jp = addr_kCrocomire_Ilist_BC30 as i32 as usize as *const u16;
        (*E).crocom_var_C = 12u16;
    } else if (((*E).base.x_pos) as i16).wrapping_sub((g_word_A486A2) as i16) < 0i16 {
        jp = addr_kCrocomire_Ilist_BE7E as i32 as usize as *const u16;
        (*E).crocom_var_C = 10u16;
    } else if (((jp as *mut u8).offset_from(RomBankBase((*gEnemyData(0i32, game)).bank, game))
        as ::core::ffi::c_long
        - addr_kCrocomire_Ilist_BC34 as i32 as ::core::ffi::c_long) as i16)
        >= 0i16
    {
        return addr_kCrocomire_Ilist_BBCE as i32 as usize as *const u16;
    }
    jp
}
pub unsafe fn Crocomire_Instr_14(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    if (ram_word!(game, random_number) as i32 & 0xfffi32) - 1024i32 & 0x8000i32 != 0 {
        let E: *mut Enemy_Crocomire = Get_Crocomire(0i32, game);
        (*E).crocom_var_C = 8u16;
        (*E).crocom_var_F = 0u16;
        return addr_kCrocomire_Ilist_BB36 as i32 as usize as *const u16;
    }
    jp
}
pub unsafe fn Crocomire_Func_7(k: u16, mut jp: *const u16, game: &mut Game) -> *const u16 {
    let E: *mut Enemy_Crocomire = Get_Crocomire(0i32, game);
    let crocom_var_B: u16 = (*E).crocom_var_B;
    if (crocom_var_B as i32 & 0x800i32) != 0i32 {
        (*E).crocom_var_B = (crocom_var_B) & (0xf7ffu16);
        jp = addr_kCrocomire_Ilist_BC30 as i32 as usize as *const u16;
        (*E).crocom_var_C = 12u16;
    } else {
        let crocom_var_F: u16 = (*E).crocom_var_F;
        if crocom_var_F as i32 - 18i32 & 0x8000i32 != 0 {
            (*E).crocom_var_F = ((*E).crocom_var_F).wrapping_add(2u16);
            SpawnEprojWithGfx(
                crocom_var_F,
                ram_word!(game, cur_enemy_index),
                addr_stru_868F8F as u16,
                game,
            );
            QueueSfx3_Max6(0x1cu16, game);
        } else {
            jp = addr_kCrocomire_Ilist_BBCA as i32 as usize as *const u16;
            (*E).crocom_var_C = 6u16;
        }
    }
    jp
}
pub unsafe fn Crocomire_Func_8(k: u16, mut jp: *const u16, game: &mut Game) -> *const u16 {
    let E: *mut Enemy_Crocomire = Get_Crocomire(0i32, game);
    let crocom_var_B: u16 = (*E).crocom_var_B;
    if (crocom_var_B as i32 & 0x800i32) != 0i32 {
        (*E).crocom_var_B = (crocom_var_B) & (0xf7ffu16);
        jp = addr_kCrocomire_Ilist_BC30 as i32 as usize as *const u16;
        (*E).crocom_var_C = 12u16;
    }
    jp
}
pub unsafe fn Crocomire_Func_9(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let mut v2: u16 = 0;
    let E: *mut Enemy_Crocomire = Get_Crocomire(0i32, game);
    let crocom_var_D: u16 = (*E).crocom_var_D;
    if crocom_var_D != 0u16 && {
        v2 = (crocom_var_D).wrapping_sub(1u16);
        (*E).crocom_var_D = v2;
        (*E).crocom_var_D != 0u16
    } {
        (*E).crocom_var_C = 12u16;
        addr_kCrocomire_Ilist_BC34 as i32 as usize as *const u16
    } else {
        (*E).crocom_var_C = 6u16;
        addr_kCrocomire_Ilist_BBCE as i32 as usize as *const u16
    }
}
pub unsafe fn Crocomire_Func_10(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E: *mut Enemy_Crocomire = Get_Crocomire(0i32, game);
    if (((*E).base.x_pos) as i16).wrapping_sub((g_word_A486A2) as i16) >= 0i16 {
        (*E).crocom_var_C = 6u16;
        return addr_kCrocomire_Ilist_BBCE as i32 as usize as *const u16;
    }
    jp
}
pub unsafe fn Crocomire_Func_11(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    (*Get_Crocomire(0i32, game)).crocom_var_C = 6u16;
    addr_kCrocomire_Ilist_BD2A as i32 as usize as *const u16
}
pub unsafe fn Crocomire_Func_12(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E: *mut Enemy_Crocomire = Get_Crocomire(0i32, game);
    (*E).crocom_var_B = ((*E).crocom_var_B as i32 & !(0x400i32)) as u16;
    addr_kCrocomire_Ilist_BCD8 as i32 as usize as *const u16
}
pub unsafe fn Crocomire_Func_13(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E: *mut Enemy_Crocomire = Get_Crocomire(0i32, game);
    if ((*E).crocom_var_B as i32 & 0x800i32) != 0i32 {
        (*E).crocom_var_B = ((*E).crocom_var_B as i32 & !(0x800i32)) as u16;
        (*E).crocom_var_C = 20u16;
        return addr_kCrocomire_Ilist_BC30 as i32 as usize as *const u16;
    } else if (((jp as *mut u8).offset_from(RomBankBase((*gEnemyData(0i32, game)).bank, game))
        as ::core::ffi::c_long
        - addr_kCrocomire_Ilist_BDA2 as i32 as ::core::ffi::c_long) as i16)
        >= 0i16
    {
        return addr_kCrocomire_Ilist_BD2A as i32 as usize as *const u16;
    }
    jp
}
pub unsafe fn Crocomire_Func_14(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E: *mut Enemy_Crocomire = Get_Crocomire(0i32, game);
    if ((*E).crocom_var_B as i32 & 0x800i32) != 0i32 {
        (*E).crocom_var_B = ((*E).crocom_var_B as i32 & !(0x800i32)) as u16;
        (*E).crocom_var_C = 12u16;
        return addr_kCrocomire_Ilist_BC30 as i32 as usize as *const u16;
    } else if (((jp as *mut u8).offset_from(RomBankBase((*gEnemyData(0i32, game)).bank, game))
        as ::core::ffi::c_long
        - addr_kCrocomire_Ilist_BDA2 as i32 as ::core::ffi::c_long) as i16)
        >= 0i16
    {
        return addr_kCrocomire_Ilist_BD2A as i32 as usize as *const u16;
    }
    jp
}
pub unsafe fn Crocomire_Func_15(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E: *mut Enemy_Crocomire = Get_Crocomire(0i32, game);
    if ((*E).crocom_var_B as i32 & 0x800i32) != 0i32 {
        (*E).crocom_var_B = ((*E).crocom_var_B as i32 & !(0x800i32)) as u16;
        (*E).crocom_var_C = 12u16;
        return addr_kCrocomire_Ilist_BC30 as i32 as usize as *const u16;
    } else if (((jp as *mut u8).offset_from(RomBankBase((*gEnemyData(0i32, game)).bank, game))
        as ::core::ffi::c_long
        - addr_kCrocomire_Ilist_BDA2 as i32 as ::core::ffi::c_long) as i16)
        >= 0i16
    {
        return addr_kCrocomire_Ilist_BD2A as i32 as usize as *const u16;
    }
    jp
}
pub unsafe fn Crocomire_Func_16(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E: *mut Enemy_Crocomire = Get_Crocomire(0i32, game);
    let v1: u16 = ((*E).crocom_var_D).wrapping_sub(1u16);
    (*E).crocom_var_D = v1;
    if v1 as i32 - 2i32 & 0x8000i32 != 0 {
        (*E).crocom_var_D = 0u16;
        (*E).crocom_var_C = 6u16;
        return addr_kCrocomire_Ilist_BBCE as i32 as usize as *const u16;
    }
    jp
}
pub unsafe fn Crocomire_Func_17(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E: *mut Enemy_Crocomire = Get_Crocomire(0i32, game);
    let crocom_var_B: u16 = (*E).crocom_var_B;
    if (crocom_var_B as i32 & 0x800i32) != 0i32 {
        (*E).crocom_var_B = (crocom_var_B as i32 & 0x1f00i32 | 0xa000i32) as u16;
        (*E).crocom_var_D = 1u16;
        (*E).crocom_var_E = 10u16;
        (*E).crocom_var_C = 12u16;
        QueueSfx2_Max6(0x54u16, game);
    } else {
        (*Get_Crocomire(ram_word!(game, cur_enemy_index) as i32, game)).crocom_var_C = 10u16;
        return addr_kCrocomire_Ilist_BD8E as i32 as usize as *const u16;
    }
    jp
}
pub unsafe fn Crocomire_Func_18(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let result: *const u16 = Crocomire_Func_2(ram_word!(game, cur_enemy_index), jp, game);
    let E: *mut Enemy_Crocomire = Get_Crocomire(0i32, game);
    (*E).crocom_var_B = ((*E).crocom_var_B) | (0x200u16);
    (*E).crocom_var_D = 32u16;
    (*E).crocom_var_C = 30u16;
    result
}
pub unsafe fn Crocomire_Func_19(k: u16, mut jp: *const u16, game: &mut Game) -> *const u16 {
    let mut v2: bool = false;
    jp = Crocomire_Func_2(ram_word!(game, cur_enemy_index), jp, game);
    let E: *mut Enemy_Crocomire = Get_Crocomire(0i32, game);
    if (*E).crocom_var_D == 0 || {
        v2 = (*E).crocom_var_D == 1u16;
        (*E).crocom_var_D = (*E).crocom_var_D.wrapping_sub(1);
        v2 as i32 != 0
    } {
        jp = Crocomire_Func_20(ram_word!(game, cur_enemy_index), game);
        (*E).crocom_var_C = 32u16;
    }
    jp
}
pub unsafe fn Crocomire_Func_20(k: u16, game: &mut Game) -> *const u16 {
    (*Get_Crocomire(k as i32, game)).crocom_var_C = 20u16;
    let E: *mut Enemy_Crocomire = Get_Crocomire(0i32, game);
    (*E).crocom_var_B = (*E).crocom_var_B;
    addr_kCrocomire_Ilist_BAEA as i32 as usize as *const u16
}
pub unsafe fn Crocomire_Func_21(k: u16, mut jp: *const u16, game: &mut Game) -> *const u16 {
    let E: *mut Enemy_Crocomire = Get_Crocomire(0i32, game);
    if ((*E).crocom_var_B as i32 & 0x100i32) != 0i32 {
        jp = Crocomire_Func_2(ram_word!(game, cur_enemy_index), jp, game);
        (*E).crocom_var_D = 16u16;
        (*E).crocom_var_C = 34u16;
    } else {
        jp = Crocomire_Func_20(ram_word!(game, cur_enemy_index), game);
        (*E).crocom_var_C = 32u16;
    }
    jp
}
pub unsafe fn Crocomire_Func_22(mut k: u16, mut jp: *const u16, game: &mut Game) -> *const u16 {
    k = ram_word!(game, cur_enemy_index);
    jp = Crocomire_Func_2(k, jp, game);
    let E: *mut Enemy_Crocomire = Get_Crocomire(0i32, game);
    if (*E).crocom_var_D == 0 {
        (*E).crocom_var_B = ((*E).crocom_var_B) | (0x2000u16);
        jp = Crocomire_Func_3(k, jp, game);
        (*E).crocom_var_C = 36u16;
    }
    jp
}
pub unsafe fn Crocomire_Func_23(mut k: u16, mut jp: *const u16, game: &mut Game) -> *const u16 {
    k = ram_word!(game, cur_enemy_index);
    let E: *mut Enemy_Crocomire = Get_Crocomire(0i32, game);
    if (*E).base.x_pos as i32 - 672i32 & 0x8000i32 != 0 {
        jp = Crocomire_Func_3(k, jp, game);
        (*E).crocom_var_C = 36u16;
        (*E).crocom_var_D = 3u16;
    } else {
        if ((*E).crocom_var_B as i32 & 0x4000i32) == 0i32 {
            (*E).crocom_var_C = 38u16;
            jp = Crocomire_Func_12(k, jp, game);
        }
        if ((*E).crocom_var_B as i32 & 0x4000i32) != 0i32 {
            (*E).crocom_var_D = 5u16;
            jp = addr_kCrocomire_Ilist_BCD8 as i32 as usize as *const u16;
            (*E).crocom_var_F = (*E).crocom_var_C;
            (*E).crocom_var_C = 42u16;
        }
    }
    jp
}
pub unsafe fn Crocomire_Func_24(mut k: u16, mut jp: *const u16, game: &mut Game) -> *const u16 {
    let mut v2: bool = false;
    k = ram_word!(game, cur_enemy_index);
    let E: *mut Enemy_Crocomire = Get_Crocomire(0i32, game);
    if (*E).crocom_var_D == 0 || {
        v2 = (*E).crocom_var_D == 1u16;
        (*E).crocom_var_D = (*E).crocom_var_D.wrapping_sub(1);
        v2 as i32 != 0
    } {
        jp = Crocomire_Func_3(k, jp, game);
        (*E).crocom_var_C = 40u16;
        (*E).crocom_var_B = ((*E).crocom_var_B as i32 & !(0x400i32)) as u16;
    } else {
        (*E).crocom_var_C = 36u16;
        (*Get_Crocomire(0x40i32, game)).crocom_var_D = 0u16;
        (*E).crocom_var_B = ((*E).crocom_var_B) | (0x400u16);
        return addr_kCrocomire_Ilist_BCD8 as i32 as usize as *const u16;
    }
    jp
}
pub unsafe fn Crocomire_Func_25(mut k: u16, mut jp: *const u16, game: &mut Game) -> *const u16 {
    k = ram_word!(game, cur_enemy_index);
    let E: *mut Enemy_Crocomire = Get_Crocomire(0i32, game);
    let crocom_var_B: u16 = (*E).crocom_var_B;
    if (crocom_var_B as i32 & 0x2000i32) == 0i32 {
        (*E).crocom_var_B = (crocom_var_B) & (0xfcffu16);
    }
    jp = Crocomire_Func_3(k, jp, game);
    (*E).crocom_var_C = 40u16;
    jp
}
pub unsafe fn Crocomire_Func_26(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E: *mut Enemy_Crocomire = Get_Crocomire(0i32, game);
    if (*E).crocom_var_D != 0 {
        let crocom_var_B: u16 = (*E).crocom_var_B;
        if (crocom_var_B as i32 & 0x4000i32) != 0i32 {
            (*E).crocom_var_D = (*E).crocom_var_D.wrapping_sub(1);
            QueueSfx2_Max6(0x3bu16, game);
            return addr_kCrocomire_Ilist_BCD8 as i32 as usize as *const u16;
        } else {
            (*E).crocom_var_B = (crocom_var_B) & (0xbfffu16);
            (*E).crocom_var_C = 12u16;
        }
    } else {
        (*E).crocom_var_B = ((*E).crocom_var_B as i32 & !(0x4000i32)) as u16;
        (*E).base.instruction_timer = 1u16;
        (*E).crocom_var_C = (*E).crocom_var_F;
        return addr_kCrocomire_Ilist_BCD8 as i32 as usize as *const u16;
    }
    jp
}
pub unsafe fn Crocomire_Init(game: &mut Game) {
    let mut v7: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    set_ram_word!(game.boss_id = 6u16);
    let mut v0: u16 = 0u16;
    loop {
        set_ram_word!(game.tilemap_stuff[(v0 as i32 >> 1i32)] = 824u16);
        v0 = (v0).wrapping_add(2u16);
        if (((v0) as i16).wrapping_sub(4096i16) >= 0i16) {
            break;
        }
    }
    if (*((&raw mut game.g_ram as *mut u8)
        .offset(0xd828)
        .offset(ram_word!(game, area_index) as isize) as *mut u8 as *mut u16) as i32
        & 2i32)
        != 0i32
    {
        *((&raw mut game.g_ram as *mut u8).offset(0xcd20) as *mut u16) = 257u16;
        *((&raw mut game.g_ram as *mut u8).offset(0xcd20).offset(2) as *mut u8 as *mut u16) =
            257u16;
        set_ram_word!(game.croco_target_0688 = 0u16);
        let E: *mut Enemy_Crocomire = Get_Crocomire(0i32, game);
        (*E).base.properties = ((*E).base.properties as i32 & 0x7bffi32 | 0x400i32) as u16;
        SpawnHardcodedPlm(
            SpawnHardcodedPlmArgs::new(0x20u8, 0x3u8, 0xb753 as VoidP),
            game,
        );
        SpawnHardcodedPlm(
            SpawnHardcodedPlmArgs::new(0x1eu8, 0x3u8, 0xb753 as VoidP),
            game,
        );
        SpawnHardcodedPlm(
            SpawnHardcodedPlmArgs::new(0x61u8, 0xbu8, 0xb747 as VoidP),
            game,
        );
        (*E).crocom_var_A = 84u16;
        (*E).base.current_instruction = addr_kCrocomire_Ilist_E1CC as u16;
        (*E).base.instruction_timer = 1u16;
        (*E).base.x_pos = 576u16;
        (*E).base.y_pos = 144u16;
        (*E).base.y_height = 28u16;
        (*E).base.x_width = 40u16;
        let v6: u16 = ram_word!(game, vram_write_queue_tail);
        v7 = gVramWriteEntry(ram_word!(game, vram_write_queue_tail) as i32, game);
        (*v7).size = 2048u16;
        (*v7).src.addr = 0x2000 as VoidP;
        *(&raw mut (*v7).src.bank as *mut u16) = 126u16;
        (*v7).vram_dst =
            ((*(&raw mut game.g_ram as *mut u8).offset(0x59) as i32 & 0xfci32) << 8i32) as u16;
        set_ram_word!(game.vram_write_queue_tail = (v6).wrapping_add(7u16));
    } else {
        DisableMinimapAndMarkBossRoomAsExplored(game);
        set_ram_word!(game.croco_word_7E069A = 0u16);
        let E_0: *mut Enemy_Crocomire =
            Get_Crocomire(ram_word!(game, cur_enemy_index) as i32, game);
        (*E_0).crocom_var_A = 0u16;
        (*E_0).crocom_var_E = 0u16;
        *((&raw mut game.g_ram as *mut u8).offset(0xcd20) as *mut u16) = 0u16;
        let mut i: i32 = 32i32;
        while i >= 0i32 {
            let v3: i32 = i >> 1i32;
            set_ram_word!(
                game.target_palettes[(v3 + 160i32)] =
                    *(game.g_rom.offset(crate::rtl::lorom_offset(0xa4b8bd)) as *const u8
                        as *mut u16)
                        .offset(v3 as isize)
            );
            set_ram_word!(
                game.target_palettes[(v3 + 208i32)] =
                    *(game.g_rom.offset(crate::rtl::lorom_offset(0xa4b8dd)) as *const u8
                        as *mut u16)
                        .offset(v3 as isize)
            );
            i -= 2i32;
        }
        (*E_0).crocom_var_C = 4u16;
        set_ram_word!(game.UNUSED_word_7E179E = 16u16);
        set_ram_word!(game.camera_distance_index = 2u16);
        set_ram_word!(game.enemy_bg2_tilemap_size = 1024u16);
        (*E_0).base.current_instruction = addr_kCrocomire_Ilist_BADE as u16;
        (*E_0).base.extra_properties = ((*E_0).base.extra_properties) | (4u16);
        (*E_0).base.instruction_timer = 1u16;
    };
}
pub unsafe fn Crocomire_Func_27(k: u16, game: &mut Game) {
    set_ram_word!(
        game.reg_BG2VOFS = (67u16).wrapping_sub((*Get_Crocomire(k as i32, game)).base.y_pos)
    );
    let mut v1: u16 = 32u16;
    let mut spritemap_pointer: u16 = 0;
    loop {
        spritemap_pointer = (*Get_Crocomire(0i32, game)).base.spritemap_pointer;
        if spritemap_pointer
            == *(game.g_rom.offset(crate::rtl::lorom_offset(0xa48b79)) as *const u8 as *mut u16)
                .offset((v1 as i32 >> 1i32) as isize)
        {
            break;
        }
        v1 = (v1).wrapping_sub(2u16);
        if (v1 as i32 & 0x8000i32) != 0i32 {
            Crocomire_8BA4(game);
            return;
        }
    }
    update_ram_word!(
        game.reg_BG2VOFS,
        wrapping_add(*(RomPtr_A4(spritemap_pointer, game) as *mut u16).offset(14))
    );
    Crocomire_8BA4(game);
}
pub unsafe fn Crocomire_8BA4(game: &mut Game) {
    let current_block: u64;
    let E0: *mut Enemy_Crocomire = Get_Crocomire(ram_word!(game, cur_enemy_index) as i32, game);
    let E1: *mut Enemy_Crocomire =
        Get_Crocomire(ram_word!(game, cur_enemy_index) as i32 + 64i32, game);
    (*E1).base.x_pos = ((*E1).crocom_var_A).wrapping_add((*E0).base.x_pos);
    (*E1).base.y_pos = (*E0).base.y_pos;
    let x_pos: u16 = (*E0).base.x_pos;
    if ((x_pos) as i16).wrapping_sub((ram_word!(game, layer1_x_pos)) as i16) >= 0i16 {
        if (*E0).base.x_pos as i32 - 128i32 - (ram_word!(game, layer1_x_pos) as i32 + 256i32)
            & 0x8000i32
            != 0
        {
            current_block = 6937071982253665452;
        } else {
            current_block = 1762290943679506230;
        }
    } else if ((x_pos as i32 + 128i32 - ram_word!(game, layer1_x_pos) as i32) as i16) < 0i16 {
        current_block = 1762290943679506230;
    } else {
        current_block = 6937071982253665452;
    }
    match current_block {
        6937071982253665452 => {
            let mut v6: u16 =
                (ram_word!(game, layer1_x_pos) as i32 - (*E0).base.x_pos as i32 + 51i32) as u16;
            let v4: u16 = abs16(v6);
            if v4 as i32 - 284i32 & 0x8000i32 == 0 {
                v6 = 256u16;
            }
            set_ram_word!(game.reg_BG2HOFS = v6);
        }
        _ => {
            set_ram_word!(game.reg_BG2HOFS = 256u16);
        }
    }
}
pub fn nullsub_305(game: &mut Game) {}
static off_A48C14: [Option<Func_V>; 45] = unsafe {
    [
        Some(Crocomire_Func_28 as unsafe fn(&mut Game) -> ()),
        Some(Crocomire_Func_52 as unsafe fn(&mut Game) -> ()),
        Some(Crocomire_Func_36 as unsafe fn(&mut Game) -> ()),
        Some(Crocomire_92CE as unsafe fn(&mut Game) -> ()),
        Some(Crocomire_Func_54 as unsafe fn(&mut Game) -> ()),
        Some(Crocomire_Func_36 as unsafe fn(&mut Game) -> ()),
        Some(Crocomire_92CE as unsafe fn(&mut Game) -> ()),
        Some(Crocomire_Func_54 as unsafe fn(&mut Game) -> ()),
        Some(Crocomire_Func_57 as unsafe fn(&mut Game) -> ()),
        Some(Crocomire_Func_60 as unsafe fn(&mut Game) -> ()),
        Some(Crocomire_Func_62 as unsafe fn(&mut Game) -> ()),
        Some(Crocomire_929E as unsafe fn(&mut Game) -> ()),
        Some(Crocomire_Func_64 as unsafe fn(&mut Game) -> ()),
        Some(Crocomire_Func_65 as unsafe fn(&mut Game) -> ()),
        Some(Crocomire_Func_66 as unsafe fn(&mut Game) -> ()),
        Some(Crocomire_Func_51 as unsafe fn(&mut Game) -> ()),
        Some(Crocomire_8D47 as unsafe fn(&mut Game) -> ()),
        Some(Crocomire_929E as unsafe fn(&mut Game) -> ()),
        Some(Crocomire_Func_51 as unsafe fn(&mut Game) -> ()),
        Some(Crocomire_8D47 as unsafe fn(&mut Game) -> ()),
        Some(Crocomire_929E as unsafe fn(&mut Game) -> ()),
        Some(Crocomire_Func_51 as unsafe fn(&mut Game) -> ()),
        Some(Crocomire_Func_59 as unsafe fn(&mut Game) -> ()),
        Some(Crocomire_Func_60 as unsafe fn(&mut Game) -> ()),
        Some(Crocomire_Func_62 as unsafe fn(&mut Game) -> ()),
        Some(Crocomire_Func_63 as unsafe fn(&mut Game) -> ()),
        Some(Crocomire_Func_56 as unsafe fn(&mut Game) -> ()),
        Some(Crocomire_9506 as unsafe fn(&mut Game) -> ()),
        Some(Crocomire_Func_65 as unsafe fn(&mut Game) -> ()),
        Some(Crocomire_Func_66 as unsafe fn(&mut Game) -> ()),
        Some(Crocomire_Func_49 as unsafe fn(&mut Game) -> ()),
        Some(Crocomire_Func_68 as unsafe fn(&mut Game) -> ()),
        Some(Crocomire_Func_70 as unsafe fn(&mut Game) -> ()),
        Some(Crocomire_Func_71 as unsafe fn(&mut Game) -> ()),
        Some(Crocomire_Func_72 as unsafe fn(&mut Game) -> ()),
        Some(Crocomire_Func_73 as unsafe fn(&mut Game) -> ()),
        Some(Crocomire_Func_88 as unsafe fn(&mut Game) -> ()),
        Some(Crocomire_9B65 as unsafe fn(&mut Game) -> ()),
        Some(Crocomire_Func_90 as unsafe fn(&mut Game) -> ()),
        Some(Crocomire_Func_89 as unsafe fn(&mut Game) -> ()),
        Some(Crocomire_9B86 as unsafe fn(&mut Game) -> ()),
        Some(nullsub_305 as unsafe fn(&mut Game) -> ()),
        Some(Crocomire_Func_91 as unsafe fn(&mut Game) -> ()),
        Some(Crocomire_Func_29 as unsafe fn(&mut Game) -> ()),
        Some(Crocomire_Func_69 as unsafe fn(&mut Game) -> ()),
    ]
};
pub unsafe fn Crocomire_Main(game: &mut Game) {
    let v0: i32 = (*Get_Crocomire(0i32, game)).crocom_var_A as i32 >> 1i32;
    off_A48C14[v0 as usize].expect("non-null function pointer")(game);
    Crocomire_Func_30(game);
    Crocomire_Func_31(game);
}
pub unsafe fn Crocomire_Func_28(game: &mut Game) {
    Crocomire_Func_37(game);
    *((&raw mut game.g_ram as *mut u8).offset(0xcd20).offset(4) as *mut u8 as *mut u16) = 257u16;
    if ram_word!(game, samus_x_pos) as i32 - 1312i32 & 0x8000i32 == 0 {
        *((&raw mut game.g_ram as *mut u8).offset(0xcd20).offset(4) as *mut u8 as *mut u16) =
            256u16;
    }
    Crocomire_Func_27(ram_word!(game, cur_enemy_index), game);
}
pub unsafe fn Crocomire_Func_29(game: &mut Game) {
    set_ram_word!(game.reg_BG2HOFS = 0u16);
    set_ram_word!(game.reg_BG2VOFS = 0u16);
}
pub unsafe fn Crocomire_Func_30(game: &mut Game) {
    let E: *mut Enemy_Crocomire = Get_Crocomire(0i32, game);
    if (*E).crocom_var_A == 0
        && (((*E).base.x_pos as i32
            - (*E).base.x_width as i32
            - ram_word!(game, samus_x_radius) as i32
            - ram_word!(game, samus_x_pos) as i32) as i16)
            < 0i16
    {
        NormalEnemyTouchAi(game);
        set_ram_word!(
            game.samus_x_pos = ((*E).base.x_pos as i32
                - (*E).base.x_width as i32
                - ram_word!(game, samus_x_radius) as i32) as u16
        );
        set_ram_word!(game.samus_prev_x_pos = ram_word!(game, samus_x_pos));
        set_ram_word!(game.extra_samus_x_displacement = -(4i32) as u16);
        set_ram_word!(game.extra_samus_y_displacement = -(1i32) as u16);
    }
}
pub unsafe fn Crocomire_Func_31(game: &mut Game) {
    let mut j: u16 = 0;
    if ram_word!(game, door_transition_flag_enemies) == 0 {
        if (*Get_Crocomire(0i32, game)).base.flash_timer != 0u16
            && (ram_word!(game, random_enemy_counter) as i32 & 2i32) != 0i32
        {
            let mut i: i32 = 14i32;
            while i >= 0i32 {
                *((&raw mut game.g_ram as *mut u8).offset(0xc000) as *mut u16)
                    .offset(((i >> 1i32) + 112i32) as isize) = 0x7fffu16;
                i -= 2i32;
            }
        } else {
            j = 14u16;
            while (j as i32 & 0x8000i32) == 0i32 {
                *((&raw mut game.g_ram as *mut u8).offset(0xc000) as *mut u16)
                    .offset(((j as i32 >> 1i32) + 112i32) as isize) =
                    *(game.g_rom.offset(crate::rtl::lorom_offset(0xa4b89d)) as *const u8
                        as *mut u16)
                        .offset((j as i32 >> 1i32) as isize);
                j = (j).wrapping_sub(2u16);
            }
        }
    }
}
pub unsafe fn Crocomire_Instr_11(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    QueueSfx2_Max6(0x74u16, game);
    jp
}
pub unsafe fn Crocomire_Instr_7(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    QueueSfx2_Max6(0x25u16, game);
    jp
}
pub unsafe fn Crocomire_Instr_19(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    QueueSfx2_Max6(0x75u16, game);
    jp
}
pub unsafe fn Crocomire_Func_35(game: &mut Game) {
    let E: *mut Enemy_Crocomire = Get_Crocomire(0i32, game);
    let crocom_var_20: u16 = (*E).crocom_var_20;
    if crocom_var_20 != 0 {
        let v2: u16 = (crocom_var_20).wrapping_sub(1u16);
        (*E).crocom_var_20 = v2;
        if v2 == 0 {
            (*E).crocom_var_20 = 32u16;
            QueueSfx3_Max6(0x22u16, game);
        }
    }
}
pub unsafe fn Crocomire_Func_36(game: &mut Game) {
    Crocomire_Func_35(game);
    Crocomire_8BA4(game);
    Crocomire_8D47(game);
}
pub unsafe fn Crocomire_8D47(game: &mut Game) {
    let E: *mut Enemy_Crocomire = Get_Crocomire(0i32, game);
    let crocom_var_D: u16 = (*E).crocom_var_D;
    if crocom_var_D != 0 {
        (*E).crocom_var_D = (crocom_var_D).wrapping_sub(1u16);
    } else {
        (*E).crocom_var_A = (*E).crocom_var_A.wrapping_add(1);
        (*E).crocom_var_A = (*E).crocom_var_A.wrapping_add(1);
        (*E).crocom_var_E = 768u16;
    };
}
pub unsafe fn Crocomire_Func_37(game: &mut Game) {
    let E0: *mut Enemy_Crocomire = Get_Crocomire(0i32, game);
    if (*E0).base.x_pos as i32 - 1536i32 & 0x8000i32 != 0 {
        set_ram_word!(game.kraid_unk9000 = 0u16);
        set_ram_word!(game.g_word_7E9002 = 0u16);
        set_ram_word!(game.g_word_7E9006 = 0u16);
        set_ram_word!(game.g_word_7E900A = 0u16);
    } else if (*E0).base.x_pos as i32 - 1552i32 & 0x8000i32 != 0 {
        if ram_word!(game, kraid_unk9000) == 0 {
            set_ram_word!(game.kraid_unk9000 = 1u16);
            game.eproj_spawn_pt = Point16U {
                x: 1536u16,
                y: 176u16,
            };
            SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 0x15u16, game);
        }
    } else if (*E0).base.x_pos as i32 - 1552i32 & 0x8000i32 != 0 {
        set_ram_word!(game.g_word_7E9002 = 0u16);
        set_ram_word!(game.g_word_7E9006 = 0u16);
        set_ram_word!(game.g_word_7E900A = 0u16);
    } else if (*E0).base.x_pos as i32 - 1568i32 & 0x8000i32 != 0 {
        if ram_word!(game, g_word_7E9002) == 0 {
            set_ram_word!(game.g_word_7E9002 = 1u16);
            SpawnHardcodedPlm(
                SpawnHardcodedPlmArgs::new(0x61u8, 0xbu8, 0xb74b as VoidP),
                game,
            );
            game.eproj_spawn_pt = Point16U {
                x: 1568u16,
                y: 176u16,
            };
            SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 0x15u16, game);
        }
    } else {
        let x_pos: u16 = (*E0).base.x_pos;
        if x_pos as i32 - 1568i32 & 0x8000i32 != 0 {
            set_ram_word!(game.g_word_7E9006 = 0u16);
            set_ram_word!(game.g_word_7E900A = 0u16);
        } else if x_pos as i32 - 1584i32 & 0x8000i32 != 0 {
            if ram_word!(game, g_word_7E9006) == 0 {
                set_ram_word!(game.g_word_7E9006 = 1u16);
                SpawnHardcodedPlm(
                    SpawnHardcodedPlmArgs::new(0x62u8, 0xbu8, 0xb74b as VoidP),
                    game,
                );
                SpawnHardcodedPlm(
                    SpawnHardcodedPlmArgs::new(0x63u8, 0xbu8, 0xb74b as VoidP),
                    game,
                );
                game.eproj_spawn_pt = Point16U {
                    x: 1584u16,
                    y: 176u16,
                };
                SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 0x15u16, game);
            }
        } else if ((x_pos) as i16).wrapping_sub((g_word_A486A4) as i16) >= 0i16 {
            Crocomire_8EE5(game);
            (*E0).crocom_var_20 = 1u16;
            set_ram_word!(game.g_word_7E9018 = 1u16);
            let v3: u16 = ram_word!(game, cur_enemy_index);
            let EK: *mut Enemy_Crocomire =
                Get_Crocomire(ram_word!(game, cur_enemy_index) as i32, game);
            (*EK).crocom_var_A = ((*EK).crocom_var_A).wrapping_add(2u16);
            (*Get_Crocomire(0i32, game)).crocom_var_00 = 2u16;
            (*Get_Crocomire(0x40i32, game)).crocom_var_00 = 2u16;
            (*Get_Crocomire(0x80i32, game)).crocom_var_00 = 2u16;
            (*Get_Crocomire(0xc0i32, game)).crocom_var_00 = 2u16;
            (*Get_Crocomire(0x100i32, game)).crocom_var_00 = 2u16;
            (*Get_Crocomire(0x140i32, game)).crocom_var_00 = 2u16;
            (*Get_Crocomire(0i32, game)).crocom_var_01 = 0u16;
            (*Get_Crocomire(0x40i32, game)).crocom_var_01 = 0u16;
            (*Get_Crocomire(0x80i32, game)).crocom_var_01 = 0u16;
            (*Get_Crocomire(0xc0i32, game)).crocom_var_01 = 0u16;
            (*Get_Crocomire(0x100i32, game)).crocom_var_01 = 0u16;
            (*Get_Crocomire(0x140i32, game)).crocom_var_01 = 0u16;
            QueueSfx2_Max6(0x3bu16, game);
            *(((&raw mut game.g_ram as *mut u8).offset(0x9015) as *mut u16 as *mut u8).offset(1)
                as *mut u16) = 0u16;
            SpawnHardcodedPlm(
                SpawnHardcodedPlmArgs::new(0x4eu8, 0x3u8, 0xb757 as VoidP),
                game,
            );
            (*EK).base.current_instruction = addr_kCrocomire_Ilist_BFB0 as u16;
            (*EK).base.instruction_timer = 1u16;
            (*EK).base.properties = ((*EK).base.properties) | ((kEnemyProps_Tangible) as u16);
            let E1: *mut Enemy_Crocomire =
                Get_Crocomire(ram_word!(game, cur_enemy_index) as i32 + 64i32, game);
            (*E1).base.instruction_timer = 0x7fffu16;
            (*E1).base.current_instruction = addr_kCrocomire_Ilist_BF62 as u16;
            let fresh11 = &raw mut (*Get_Crocomire(0x40i32, game)).base.properties;
            fresh11.write_unaligned((fresh11.read_unaligned()) | (0x100u16));
            (*EK).crocom_var_E = 0u16;
            (*EK).crocom_var_F = 0u16;
            (*EK).crocom_var_D = 2048u16;
            (*Get_Crocomire(0i32, game)).base.y_height = 16u16;
        }
    };
}
pub unsafe fn Crocomire_8EE5(game: &mut Game) {
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs::new(0x61u8, 0xbu8, 0xb74f as VoidP),
        game,
    );
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs::new(0x62u8, 0xbu8, 0xb74f as VoidP),
        game,
    );
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs::new(0x63u8, 0xbu8, 0xb74f as VoidP),
        game,
    );
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs::new(0x64u8, 0xbu8, 0xb74f as VoidP),
        game,
    );
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs::new(0x65u8, 0xbu8, 0xb74f as VoidP),
        game,
    );
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs::new(0x66u8, 0xbu8, 0xb74f as VoidP),
        game,
    );
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs::new(0x67u8, 0xbu8, 0xb74f as VoidP),
        game,
    );
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs::new(0x68u8, 0xbu8, 0xb74f as VoidP),
        game,
    );
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs::new(0x69u8, 0xbu8, 0xb74f as VoidP),
        game,
    );
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs::new(0x6au8, 0xbu8, 0xb74f as VoidP),
        game,
    );
    game.eproj_spawn_pt = Point16U {
        x: 1536u16,
        y: 176u16,
    };
    SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 0x15u16, game);
    game.eproj_spawn_pt = Point16U {
        x: 1552u16,
        y: 192u16,
    };
    SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 0x15u16, game);
    game.eproj_spawn_pt = Point16U {
        x: 1568u16,
        y: 176u16,
    };
    SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 0x15u16, game);
    game.eproj_spawn_pt = Point16U {
        x: 1584u16,
        y: 192u16,
    };
    SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 0x15u16, game);
    game.eproj_spawn_pt = Point16U {
        x: 1600u16,
        y: 192u16,
    };
    SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 0x15u16, game);
    game.eproj_spawn_pt = Point16U {
        x: 1616u16,
        y: 192u16,
    };
    SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 0x15u16, game);
    game.eproj_spawn_pt = Point16U {
        x: 1632u16,
        y: 192u16,
    };
    SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 0x15u16, game);
}
pub unsafe fn Crocomire_Instr_2(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    set_ram_word!(game.earthquake_type = 4u16);
    set_ram_word!(game.earthquake_timer = 5u16);
    QueueSfx2_Max6(0x76u16, game);
    jp
}
pub unsafe fn Crocomire_Instr_4(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    if ((*Get_Crocomire(0i32, game)).crocom_var_B as i32 & 0x800i32) == 0i32 {
        Enemy_MoveRight_IgnoreSlopes(
            ram_word!(game, cur_enemy_index),
            (-(4i32) as i16 as i32) << 16i32,
            game,
        );
    }
    jp
}
pub unsafe fn Crocomire_Instr_3(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    Crocomire_Func_43(game);
    Crocomire_Instr_4(k, jp, game)
}
pub unsafe fn Crocomire_Instr_15(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    Crocomire_Func_43(game);
    Crocomire_Instr_4(k, jp, game)
}
pub unsafe fn Crocomire_Func_43(game: &mut Game) {
    let mut v1: u16 = (ram_word!(game, random_number)) & (0x1fu16);
    if ((ram_word!(game, random_number)) as i16).wrapping_sub(4096i16) >= 0i16 {
        v1 = -(v1 as i32) as u16;
    }
    Crocomire_Func_87(::core::ptr::null::<u16>(), v1, game);
}
pub unsafe fn Crocomire_Instr_16(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    if Enemy_MoveRight_IgnoreSlopes(
        ram_word!(game, cur_enemy_index),
        (-(4i32) as i16 as i32) << 16i32,
        game,
    ) != 0
    {
        (*Get_Crocomire(0i32, game)).crocom_var_C = 14u16;
        addr_kCrocomire_Ilist_BF3C as i32 as usize as *const u16
    } else {
        let mut v2: u16 = 32u16;
        if ram_word!(game, random_number) as i32 - 2048i32 & 0x8000i32 == 0 {
            v2 = -(32i32) as u16;
        }
        Crocomire_Func_87(
            jp,
            (v2 as i32 + (ram_word!(game, random_number) as i32 & 0xfi32)) as u16,
            game,
        )
    }
}
pub unsafe fn Crocomire_Instr_13(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    let E: *mut Enemy_Crocomire = Get_Crocomire(0i32, game);
    if (((*E).base.x_pos as i32
        - (*E).base.x_width as i32
        - 260i32
        - ram_word!(game, layer1_x_pos) as i32) as i16)
        < 0i16
    {
        Enemy_MoveRight_IgnoreSlopes(
            ram_word!(game, cur_enemy_index),
            (4i16 as i32) << 16i32,
            game,
        );
    }
    jp
}
pub unsafe fn Crocomire_Instr_18(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    Enemy_MoveRight_IgnoreSlopes(k, (4i16 as i32) << 16i32, game);
    jp
}
pub unsafe fn Crocomire_Instr_12(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    Crocomire_Func_43(game);
    Crocomire_Instr_13(k, jp, game)
}
pub unsafe fn Crocomire_Instr_17(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    Crocomire_Func_43(game);
    Crocomire_Instr_18(k, jp, game)
}
pub unsafe fn Crocomire_Func_49(game: &mut Game) {
    Crocomire_Func_50(game);
    Crocomire_Func_53(game);
    Crocomire_Func_55(game);
    let E: *mut Enemy_Crocomire = Get_Crocomire(0i32, game);
    if (*E).crocom_var_A == 62u16 {
        QueueMusic_Delayed8(6u16, game);
        (*E).crocom_var_A = 88u16;
        (*E).base.current_instruction = addr_kCrocomire_Ilist_E1D2 as u16;
        *((&raw mut game.g_ram as *mut u8).offset(0xcd20).offset(4) as *mut u8 as *mut u16) =
            257u16;
        set_ram_word!(game.debug_disable_minimap = 0u16);
        let fresh12 = &raw mut (*Get_Crocomire(0x40i32, game)).base.properties;
        fresh12.write_unaligned((fresh12.read_unaligned()) | (0x200u16));
        SpawnHardcodedPlm(
            SpawnHardcodedPlmArgs::new(0x4eu8, 0x3u8, 0xb753 as VoidP),
            game,
        );
        set_ram_word!(game.camera_distance_index = 0u16);
        set_ram_word!(game.croco_target_0688 = 0u16);
    }
}
pub unsafe fn Crocomire_Func_50(game: &mut Game) {
    let E: *mut Enemy_Crocomire = Get_Crocomire(0i32, game);
    let y_pos: u16 = (*E).base.y_pos;
    let mut v2: u16 = addr_kCrocomire_Ilist_BF7E as u16;
    if y_pos as i32 - 280i32 & 0x8000i32 != 0 {
        v2 = addr_kCrocomire_Ilist_BF86 as u16;
        if y_pos as i32 - 264i32 & 0x8000i32 != 0 {
            v2 = addr_kCrocomire_Ilist_BF8C as u16;
            if y_pos as i32 - 248i32 & 0x8000i32 != 0 {
                v2 = addr_kCrocomire_Ilist_BF92 as u16;
            }
        }
    }
    (*E).base.current_instruction = v2;
    (*E).base.instruction_timer = 1u16;
}
pub unsafe fn Crocomire_Func_51(game: &mut Game) {
    Crocomire_Func_53(game);
    let E: *mut Enemy_Crocomire = Get_Crocomire(0i32, game);
    let y_pos: u16 = (*E).base.y_pos;
    let mut v2: u16 = addr_kCrocomire_Ilist_BF64 as u16;
    if y_pos as i32 - 280i32 & 0x8000i32 != 0 {
        v2 = addr_kCrocomire_Ilist_BF6C as u16;
        if y_pos as i32 - 264i32 & 0x8000i32 != 0 {
            v2 = addr_kCrocomire_Ilist_BF72 as u16;
            if y_pos as i32 - 248i32 & 0x8000i32 != 0 {
                v2 = addr_kCrocomire_Ilist_BF78 as u16;
            }
        }
    }
    (*E).base.current_instruction = v2;
    (*E).base.instruction_timer = 1u16;
    Crocomire_Func_55(game);
}
pub unsafe fn Crocomire_Func_52(game: &mut Game) {
    let v0: u16 = *(((&raw mut game.g_ram as *mut u8).offset(0x9015) as *mut u16 as *mut u8)
        .offset(1) as *mut u16);
    if *(((&raw mut game.g_ram as *mut u8).offset(0x9015) as *mut u16 as *mut u8).offset(1)
        as *mut u16) as i32
        - 22i32
        & 0x8000i32
        != 0
    {
        let v1: u16 = *(((&raw mut game.g_ram as *mut u8).offset(0x9015) as *mut u16 as *mut u8)
            .offset(1) as *mut u16);
        let ref mut fresh17 = *(((&raw mut game.g_ram as *mut u8).offset(0x9015) as *mut u16
            as *mut u8)
            .offset(1) as *mut u16);
        *fresh17 = (*fresh17).wrapping_add(2u16);
        SpawnEprojWithGfx(
            *(game.g_rom.offset(crate::rtl::lorom_offset(0xa49156)) as *const u8 as *mut u16)
                .offset((v0 as i32 >> 1i32) as isize),
            v1,
            addr_stru_868F9D as u16,
            game,
        );
    }
    Crocomire_Func_54(game);
}
pub unsafe fn Crocomire_Func_53(game: &mut Game) {
    if update_ram_word!(game.g_word_7E9018, wrapping_sub(1), reread) == 0 {
        set_ram_word!(game.g_word_7E9018 = 6u16);
        let mut v0: u16 = (ram_word!(game, random_number)) & (0x3fu16);
        if (ram_word!(game, random_number) as i32 & 2i32) == 0i32 {
            v0 = !(v0 as i32) as u16;
        }
        let x: u16 = ((*Get_Crocomire(0i32, game)).base.x_pos).wrapping_add(v0);
        let y: u16 = (ram_word!(game, lava_acid_y_pos) as i32 + 16i32
            - (((ram_word!(game, random_number)) & (0x1f00u16)) as i32 >> 8i32))
            as u16;
        CreateSpriteAtPos(x, y, 21u16, 0u16, game);
    }
}
pub unsafe fn Crocomire_Func_54(game: &mut Game) {
    Crocomire_Func_35(game);
    Crocomire_Func_53(game);
    Crocomire_Func_55(game);
}
pub unsafe fn Crocomire_Func_55(game: &mut Game) {
    let mut crocom_var_D: i8 = 0;
    let mut v7: i8 = 0;
    let mut v9: i8 = 0;
    let mut crocom_var_E: i8 = 0;
    let mut crocom_var_E_high: i8 = 0;
    let mut v12: i8 = 0;
    Crocomire_Func_58(game);
    let E0: *mut Enemy_Crocomire = Get_Crocomire(0i32, game);
    (*E0).crocom_var_B = ((*E0).crocom_var_B as i32 & !(0x800i32)) as u16;
    Crocomire_Func_27(ram_word!(game, cur_enemy_index), game);
    let E: *mut Enemy_Crocomire = Get_Crocomire(ram_word!(game, cur_enemy_index) as i32, game);
    if (*E).base.y_pos as i32 - 280i32 & 0x8000i32 != 0 {
        let mut v3: u16 =
            (8i32 * (ram_word!(game, reg_BG2VOFS) as i32 + 288i32 & 0xfff8i32)) as u16;
        let mut m: i32 = 32i32;
        loop {
            set_ram_word!(game.tilemap_stuff[(v3 as i32 >> 1i32)] = 824u16);
            v3 = (v3).wrapping_add(2u16);
            m -= 1;
            if (m == 0) {
                break;
            }
        }
        set_ram_word!(game.nmi_flag_bg2_enemy_vram_transfer = 1u16);
        (*E0).base.extra_properties = ((*E0).base.extra_properties as i32 & !(0x8000i32)) as u16;
        crocom_var_D = (*E).crocom_var_D as i8;
        *(&raw mut (*E).crocom_var_D as *mut u8).offset(0) =
            ((crocom_var_D) as u8).wrapping_add(0x80u8);
        v7 = (u8::overflowing_add(crocom_var_D as u8, 0x80u8).1 as i32
            + *(&raw mut (*E).crocom_var_D as *mut u8).offset(1isize) as i32
            + 3i32) as i8;
        if v7 as i32 - 48i32 & 0x80i32 == 0 {
            v7 = 48i8;
        }
        *(&raw mut (*E).crocom_var_D as *mut u8).offset(1isize) = v7 as u8;
        let mut v8: bool =
            u8::overflowing_add(*(&raw mut (*E).crocom_var_E as *mut u8).offset(0), v7 as u8).1;
        let ref mut fresh13 = *(&raw mut (*E).crocom_var_E as *mut u8).offset(0);
        *fresh13 = (*fresh13).wrapping_add((v7) as u8);
        v9 = ((v8) as i8)
            .wrapping_add((*(&raw mut (*E).crocom_var_E as *mut u8).offset(1isize)) as i8);
        if v9 as i32 - 3i32 & 0x80i32 == 0 {
            v9 = 3i8;
        }
        *(&raw mut (*E).crocom_var_E as *mut u8).offset(1isize) = v9 as u8;
        crocom_var_E = (*E).crocom_var_E as i8;
        v8 = u8::overflowing_add(
            *(&raw mut (*E).crocom_var_F as *mut u8).offset(1isize),
            crocom_var_E as u8,
        )
        .1;
        let ref mut fresh14 = *(&raw mut (*E).crocom_var_F as *mut u8).offset(1isize);
        *fresh14 = (*fresh14).wrapping_add((crocom_var_E) as u8);
        crocom_var_E_high = *(&raw mut (*E).crocom_var_E as *mut u8).offset(1isize) as i8;
        let v13: bool = v8;
        v8 = u8::overflowing_add(v8 as u8, crocom_var_E_high as u8).1;
        v12 = ((v13) as i8).wrapping_add(crocom_var_E_high);
        v8 = (v8 as i32
            | u8::overflowing_add(*(&raw mut (*E).base.y_pos as *mut u8).offset(0), v12 as u8).1
                as i32)
            != 0;
        let ref mut fresh15 = *(&raw mut (*E).base.y_pos as *mut u8).offset(0);
        *fresh15 = (*fresh15).wrapping_add((v12) as u8);
        let ref mut fresh16 = *(&raw mut (*E).base.y_pos as *mut u8).offset(1isize);
        *fresh16 = (*fresh16).wrapping_add((v8) as u8);
    } else {
        (*E).crocom_var_A = (*E).crocom_var_A.wrapping_add(1);
        (*E).crocom_var_A = (*E).crocom_var_A.wrapping_add(1);
        (*E0).crocom_var_D = 48u16;
    };
}
pub unsafe fn Crocomire_Func_56(game: &mut Game) {
    let E: *mut Enemy_Crocomire = Get_Crocomire(ram_word!(game, cur_enemy_index) as i32, game);
    let y_pos: u16 = (*E).base.y_pos;
    let mut v2: u16 = addr_kCrocomire_Ilist_BF7E as u16;
    if y_pos as i32 - 280i32 & 0x8000i32 != 0 {
        v2 = addr_kCrocomire_Ilist_BF86 as u16;
        if y_pos as i32 - 264i32 & 0x8000i32 != 0 {
            v2 = addr_kCrocomire_Ilist_BF8C as u16;
            if y_pos as i32 - 248i32 & 0x8000i32 != 0 {
                v2 = addr_kCrocomire_Ilist_BF92 as u16;
            }
        }
    }
    (*E).base.current_instruction = v2;
    (*E).base.instruction_timer = 1u16;
    Crocomire_Func_53(game);
    Crocomire_92D8(game);
}
pub unsafe fn Crocomire_929E(game: &mut Game) {
    Crocomire_Func_53(game);
    let E: *mut Enemy_Crocomire = Get_Crocomire(ram_word!(game, cur_enemy_index) as i32, game);
    let y_pos: u16 = (*E).base.y_pos;
    let mut v2: u16 = addr_kCrocomire_Ilist_BF64 as u16;
    if y_pos as i32 - 280i32 & 0x8000i32 != 0 {
        v2 = addr_kCrocomire_Ilist_BF6C as u16;
        if y_pos as i32 - 264i32 & 0x8000i32 != 0 {
            v2 = addr_kCrocomire_Ilist_BF72 as u16;
            if y_pos as i32 - 248i32 & 0x8000i32 != 0 {
                v2 = addr_kCrocomire_Ilist_BF78 as u16;
            }
        }
    }
    (*E).base.current_instruction = v2;
    (*E).base.instruction_timer = 1u16;
    Crocomire_92D8(game);
}
pub unsafe fn Crocomire_92CE(game: &mut Game) {
    Crocomire_Func_35(game);
    Crocomire_Func_53(game);
    Crocomire_92D8(game);
}
pub unsafe fn Crocomire_92D8(game: &mut Game) {
    let mut v3: i16 = 0;
    let mut v6: i8 = 0;
    Crocomire_Func_58(game);
    let EK: *mut Enemy_Crocomire = Get_Crocomire(ram_word!(game, cur_enemy_index) as i32, game);
    if (*EK).base.y_pos as i32 - 218i32 & 0x8000i32 != 0 {
        (*EK).crocom_var_A = (*EK).crocom_var_A.wrapping_add(1);
        (*EK).crocom_var_A = (*EK).crocom_var_A.wrapping_add(1);
    } else {
        Crocomire_Func_27(ram_word!(game, cur_enemy_index), game);
        let E0: *mut Enemy_Crocomire = Get_Crocomire(0i32, game);
        v3 = (((*E0).crocom_var_D) as i16).wrapping_add(256i16);
        if (*E0).crocom_var_D as i32 - 7680i32 & 0x8000i32 == 0 {
            v3 = 7936i16;
        }
        (*E0).crocom_var_D = v3 as u16;
        let crocom_var_E: u8 = (*E0).crocom_var_E as u8;
        let mut v5: bool = crocom_var_E < *(&raw mut (*E0).crocom_var_D as *mut u8).offset(1isize);
        *(&raw mut (*E0).crocom_var_E as *mut u8).offset(0) =
            (crocom_var_E).wrapping_sub(*(&raw mut (*E0).crocom_var_D as *mut u8).offset(1isize));
        v6 = ((*(&raw mut (*E0).crocom_var_E as *mut u8).offset(1isize)) as i8)
            .wrapping_sub((v5) as i8);
        if v6 < 0i8 {
            *(&raw mut (*E0).crocom_var_E as *mut u8).offset(0) = -(1i32) as u8;
            v6 = 0i8;
        }
        *(&raw mut (*E0).crocom_var_E as *mut u8).offset(1isize) = v6 as u8;
        let crocom_var_F_high: u8 = *(&raw mut (*E0).crocom_var_F as *mut u8).offset(1isize);
        v5 = crocom_var_F_high < *(&raw mut (*E0).crocom_var_E as *mut u8).offset(0);
        *(&raw mut (*E0).crocom_var_F as *mut u8).offset(1isize) =
            (crocom_var_F_high).wrapping_sub(*(&raw mut (*E0).crocom_var_E as *mut u8).offset(0));
        let y_pos: u8 = (*E0).base.y_pos as u8;
        let v9: u8 =
            ((v5) as u8).wrapping_add(*(&raw mut (*E0).crocom_var_E as *mut u8).offset(1isize));
        *(&raw mut (*E0).base.y_pos as *mut u8).offset(0) = (y_pos).wrapping_sub(v9);
        let ref mut fresh7 = *(&raw mut (*E0).base.y_pos as *mut u8).offset(1isize);
        *fresh7 = (*fresh7).wrapping_sub((y_pos < v9) as u8);
    };
}
pub unsafe fn Crocomire_Func_57(game: &mut Game) {
    let mut i: i32 = 0;
    *(&raw mut game.g_ram as *mut u8).offset(0x68c) = 48u8;
    set_ram_word!(game.croco_target_0688 = 48u16);
    let E: *mut Enemy_Crocomire = Get_Crocomire(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).crocom_var_A = (*E).crocom_var_A.wrapping_add(1);
    (*E).crocom_var_A = (*E).crocom_var_A.wrapping_add(1);
    (*E).base.current_instruction = addr_kCrocomire_Ilist_BF64 as u16;
    (*E).base.instruction_timer = 1u16;
    set_ram_word!(game.nmi_flag_bg2_enemy_vram_transfer = 1u16);
    let E1: *mut Enemy_Crocomire =
        Get_Crocomire(ram_word!(game, cur_enemy_index) as i32 + 64i32, game);
    (*E1).base.current_instruction = addr_kCrocomire_Ilist_BF98 as u16;
    (*E1).base.instruction_timer = 1u16;
    (*E1).base.properties = ((*E1).base.properties as i32 & 0xd2ffi32 | 0x2c00i32) as u16;
    (*E1).base.x_pos = (*E).base.x_pos;
    (*E1).base.y_pos = ((*E).base.y_pos).wrapping_add(16u16);
    let mut v3: u16 = 0u16;
    loop {
        let v4: i32 = v3 as i32 >> 1i32;
        set_ram_word!(game.tilemap_stuff[v4] = 824u16);
        set_ram_word!(game.tilemap_stuff[(v4 + 1i32)] = 824u16);
        v3 = (v3).wrapping_add(4u16);
        if (((v3) as i16).wrapping_sub(1024i16) >= 0i16) {
            break;
        }
    }
    i = 0i32;
    loop {
        let v6: i32 = i >> 1i32;
        if *(game.g_rom.offset(crate::rtl::lorom_offset(0xa49c79)) as *const u8 as *mut u16)
            .offset(v6 as isize)
            == 0xffffu16
        {
            break;
        }
        set_ram_word!(
            game.tilemap_stuff[(v6 + 32i32)] =
                *(game.g_rom.offset(crate::rtl::lorom_offset(0xa49c79)) as *const u8 as *mut u16)
                    .offset(v6 as isize)
        );
        i += 2i32;
    }
    Crocomire_93BE((i + 1024i32) as u16, game);
}
pub unsafe fn Crocomire_93BE(k: u16, game: &mut Game) {
    let mut v3: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    let v2: u16 = ram_word!(game, vram_write_queue_tail);
    v3 = gVramWriteEntry(ram_word!(game, vram_write_queue_tail) as i32, game);
    (*v3).size = k;
    (*v3).src.addr = 0x2000 as VoidP;
    *(&raw mut (*v3).src.bank as *mut u16) = 126u16;
    (*v3).vram_dst =
        ((*(&raw mut game.g_ram as *mut u8).offset(0x59) as i32 & 0xfci32) << 8i32) as u16;
    set_ram_word!(game.vram_write_queue_tail = (v2).wrapping_add(7u16));
}
pub unsafe fn Crocomire_Func_58(game: &mut Game) {
    let v0: u16 = ram_word!(game, reg_BG2VOFS);
    let mut i: i32 = 510i32;
    while i >= 0i32 {
        set_ram_word!(game.crocomire_bg2_scroll_hdma_data[(i >> 1i32)] = v0);
        i -= 2i32;
    }
}
pub unsafe fn Crocomire_Func_59(game: &mut Game) {
    let mut i: i32 = 0;
    Crocomire_Func_58(game);
    let E: *mut Enemy_Crocomire = Get_Crocomire(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).crocom_var_A = (*E).crocom_var_A.wrapping_add(1);
    (*E).crocom_var_A = (*E).crocom_var_A.wrapping_add(1);
    (*E).base.instruction_timer = 1u16;
    *(&raw mut game.g_ram as *mut u8).offset(0x68c) = 48u8;
    set_ram_word!(game.croco_target_0688 = 48u16);
    (*E).base.current_instruction = addr_kCrocomire_Ilist_BF7E as u16;
    let mut v1: u16 = 0u16;
    loop {
        set_ram_word!(game.tilemap_stuff[(v1 as i32 >> 1i32)] = 824u16);
        v1 = (v1).wrapping_add(2u16);
        if (((v1) as i16).wrapping_sub(2048i16) >= 0i16) {
            break;
        }
    }
    i = 0i32;
    loop {
        let v3: i32 = i >> 1i32;
        if *(game.g_rom.offset(crate::rtl::lorom_offset(0xa49e7b)) as *const u8 as *mut u16)
            .offset(v3 as isize)
            == 0xffffu16
        {
            break;
        }
        set_ram_word!(
            game.tilemap_stuff[(v3 + 32i32)] =
                *(game.g_rom.offset(crate::rtl::lorom_offset(0xa49e7b)) as *const u8 as *mut u16)
                    .offset(v3 as isize)
        );
        i += 2i32;
    }
    Crocomire_93BE((i + 1024i32) as u16, game);
}
pub unsafe fn Crocomire_Func_60(game: &mut Game) {
    let mut i: i32 = 0;
    let E: *mut Enemy_Crocomire = Get_Crocomire(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).crocom_var_A = ((*E).crocom_var_A).wrapping_add(2u16);
    set_ram_word!(game.g_word_7E0692 = 256u16);
    set_ram_word!(game.croco_cur_vline_idx = 0u16);
    set_ram_word!(
        game.g_word_7E0698 = *((game.g_rom.offset(crate::rtl::lorom_offset(0xa49bc5)) as *const u8
            as *mut u16 as *mut u8)
            .offset(ram_word!(game, croco_word_7E069A) as i32 as isize)
            as *mut u16)
    );
    set_ram_word!(game.g_word_7E0694 = ram_word!(game, g_word_7E0698));
    set_ram_word!(
        game.g_word_7E0696 = *((game.g_rom.offset(crate::rtl::lorom_offset(0xa49bc7)) as *const u8
            as *mut u16 as *mut u8)
            .offset(ram_word!(game, croco_word_7E069A) as i32 as isize)
            as *mut u16)
    );
    set_ram_word!(
        game.g_word_7E068E = *((game.g_rom.offset(crate::rtl::lorom_offset(0xa49bc9)) as *const u8
            as *mut u16 as *mut u8)
            .offset(ram_word!(game, croco_word_7E069A) as i32 as isize)
            as *mut u16)
    );
    let bank: u8 = *((game.g_rom.offset(crate::rtl::lorom_offset(0xa49bcb)) as *const u8 as *mut u16
        as *mut u8)
        .offset(ram_word!(game, croco_word_7E069A) as i32 as isize)
        as *mut u16) as u8;
    let mut v6: u16 = 0;
    i = ram_word!(game, croco_word_7E069A) as i32 + 8i32;
    loop {
        let mut v2: u16 = *((game.g_rom.offset(crate::rtl::lorom_offset(0xa49bc5)) as *const u8
            as *mut u16 as *mut u8)
            .offset(i as isize) as *mut u16);
        if v2 == 0xffffu16 {
            break;
        }
        v6 = i as u16;
        let mut v3: u16 = *((game.g_rom.offset(crate::rtl::lorom_offset(0xa49bc7)) as *const u8
            as *mut u16 as *mut u8)
            .offset(i as isize) as *mut u16);
        let mut n: i32 = ram_word!(game, g_word_7E068E) as i32;
        loop {
            *((&raw mut game.g_ram as *mut u8).offset(v3 as isize) as *mut u8 as *mut u16) =
                *(RomPtrWithBank(bank, v2, game) as *mut u16);
            v3 = (v3).wrapping_add(2u16);
            v2 = (v2).wrapping_add(2u16);
            n -= 1;
            if (n < 0i32) {
                break;
            }
        }
        i = v6 as i32 + 4i32;
    }
    set_ram_word!(game.croco_word_7E069A = (i + 2i32) as u16);
    set_ram_word!(game.g_word_7E068A = (i + 2i32) as u16);
    let mut j: i32 = 128i32;
    while j >= 0i32 {
        *((&raw mut game.g_ram as *mut u8)
            .offset(0x69c)
            .offset(j as u16 as isize) as *mut u8 as *mut u16) = 0u16;
        j -= 2i32;
    }
}
pub unsafe fn Crocomire_Func_61(game: &mut Game) {
    Crocomire_Func_62(game);
}
pub unsafe fn Crocomire_Func_62(game: &mut Game) {
    let mut v2: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    let v0: u16 = ram_word!(game, g_word_7E068A);
    let v1: u16 = ram_word!(game, vram_write_queue_tail);
    if *((game.g_rom.offset(crate::rtl::lorom_offset(0xa49bc5)) as *const u8 as *mut u16 as *mut u8)
        .offset(ram_word!(game, g_word_7E068A) as i32 as isize) as *mut u16)
        == 0xffffu16
    {
        let E: *mut Enemy_Crocomire = Get_Crocomire(ram_word!(game, cur_enemy_index) as i32, game);
        (*E).crocom_var_A = (*E).crocom_var_A.wrapping_add(1);
        (*E).crocom_var_A = (*E).crocom_var_A.wrapping_add(1);
        set_ram_word!(game.g_word_7E068A = 0u16);
    } else {
        v2 = gVramWriteEntry(ram_word!(game, vram_write_queue_tail) as i32, game);
        (*v2).size = *((game.g_rom.offset(crate::rtl::lorom_offset(0xa49bc5)) as *const u8
            as *mut u16 as *mut u8)
            .offset(ram_word!(game, g_word_7E068A) as i32 as isize)
            as *mut u16);
        (*v2).src.addr = *((game.g_rom.offset(crate::rtl::lorom_offset(0xa49bcb)) as *const u8
            as *mut u16 as *mut u8)
            .offset(v0 as i32 as isize) as *mut u16);
        *(&raw mut (*v2).src.bank as *mut u16) =
            *((game.g_rom.offset(crate::rtl::lorom_offset(0xa49bc9)) as *const u8 as *mut u16
                as *mut u8)
                .offset(v0 as i32 as isize) as *mut u16);
        (*v2).vram_dst = *((game.g_rom.offset(crate::rtl::lorom_offset(0xa49bc7)) as *const u8
            as *mut u16 as *mut u8)
            .offset(v0 as i32 as isize) as *mut u16);
        set_ram_word!(game.g_word_7E068A = (v0).wrapping_add(8u16));
        set_ram_word!(game.vram_write_queue_tail = (v1).wrapping_add(7u16));
    };
}
pub unsafe fn Crocomire_Func_64(game: &mut Game) {
    QueueSfx2_Max6(0x77u16, game);
    Crocomire_950F(game);
}
pub unsafe fn Crocomire_9506(game: &mut Game) {
    QueueSfx2_Max6(0x2du16, game);
    Crocomire_950F(game);
}
pub unsafe fn Crocomire_950F(game: &mut Game) {
    static unk_A49559: SpawnHdmaObject_Args =
        SpawnHdmaObject_Args::new(0x42u8, 0x10u8, 0x9563 as VoidP);
    let mut v4: *mut Enemy_Crocomire = ::core::ptr::null_mut::<Enemy_Crocomire>();
    let mut v1: u16 = (2i32 * ((*Get_Crocomire(0x40i32, game)).base.y_pos as i32 - 72i32)) as u16;
    let v2: u16 = ram_word!(game, reg_BG2VOFS);
    loop {
        set_ram_word!(game.crocomire_bg2_scroll_hdma_data[(v1 as i32 >> 1i32)] = v2);
        v1 = (v1).wrapping_sub(2u16);
        if ((v1 as i32 & 0x8000i32) != 0i32) {
            break;
        }
    }
    let E: *mut Enemy_Crocomire = Get_Crocomire(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).crocom_var_A = (*E).crocom_var_A.wrapping_add(1);
    (*E).crocom_var_A = (*E).crocom_var_A.wrapping_add(1);
    v4 = Get_Crocomire(0i32, game);
    (*Get_Crocomire(0x80i32, game)).crocom_var_D = (*v4).base.x_pos;
    *((&raw mut game.g_ram as *mut u8).offset(0xcae0) as *mut u16) = 255u16;
    *((&raw mut game.g_ram as *mut u8).offset(0xcae0).offset(1) as *mut u8 as *mut u16) =
        -(13584i32) as u16;
    *((&raw mut game.g_ram as *mut u8).offset(0xcae0).offset(3) as *mut u8 as *mut u16) = 225u16;
    *((&raw mut game.g_ram as *mut u8).offset(0xcae0).offset(4) as *mut u8 as *mut u16) =
        -(13330i32) as u16;
    *((&raw mut game.g_ram as *mut u8).offset(0xcae0).offset(6) as *mut u8 as *mut u16) = 0u16;
    (*v4).crocom_var_1F = SpawnHdmaObject(
        0xa4u8,
        &raw const unk_A49559 as *const ::core::ffi::c_void,
        game,
    );
}
pub unsafe fn Crocomire_Func_63(game: &mut Game) {
    let E: *mut Enemy_Crocomire = Get_Crocomire(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).crocom_var_A = ((*E).crocom_var_A).wrapping_add(2u16);
}
pub unsafe fn Crocomire_Func_65(game: &mut Game) {
    let mut v4: u16 = 0;
    let current_block: u64;
    let mut i: i32 = 0;
    Crocomire_Func_53(game);
    let E0: *mut Enemy_Crocomire = Get_Crocomire(0i32, game);
    let E1: *mut Enemy_Crocomire = Get_Crocomire(0x40i32, game);
    let E2: *mut Enemy_Crocomire = Get_Crocomire(0x80i32, game);
    let mut crocom_var_D: u16 = (*E2).crocom_var_D;
    (*E1).crocom_var_D = (*E1).crocom_var_D.wrapping_sub(1);
    if ((*E1).crocom_var_D as i32 & 2i32) != 0i32 {
        crocom_var_D = (crocom_var_D).wrapping_add(4u16);
    }
    (*E0).base.x_pos = crocom_var_D;
    if (Crocomire_Func_67(game) != 0) {
        Crocomire_Func_27(ram_word!(game, cur_enemy_index), game);
        v4 = 0;
        v4 = (ram_word!(game, g_word_7E0694)).wrapping_sub(3u16);
        if ram_word!(game, g_word_7E0694) as i32 - 19i32 & 0x8000i32 != 0 {
            if ((ram_word!(game, g_word_7E0692)) as i16).wrapping_sub(20480i16) >= 0i16 {
                current_block = 11812396948646013369;
            } else {
                v4 = 16u16;
                current_block = 15652330335145281839;
            }
        } else {
            current_block = 15652330335145281839;
        }
        if current_block == 11812396948646013369 {
        } else {
            set_ram_word!(game.g_word_7E0694 = v4);
            let mut v5: u16 = (ram_word!(game, g_word_7E0692)).wrapping_add(384u16);
            if ram_word!(game, g_word_7E0692) as i32 - 20096i32 & 0x8000i32 == 0 {
                v5 = 20480u16;
            }
            set_ram_word!(game.g_word_7E0692 = v5);
            let mut r18: u16 = 0u16;
            let mut v6: u16 = (2i32 * ((*E1).base.y_pos as i32 - 72i32)) as u16;
            let mut v7: u16 = ram_word!(game, g_word_7E0694);
            set_ram_word!(game.message_box_animation_y1 = ram_word!(game, g_word_7E0694));
            loop {
                set_ram_word!(
                    game.crocomire_bg2_scroll_hdma_data[(v6 as i32 >> 1i32)] =
                        (ram_word!(game, reg_BG2VOFS) as i32 + v7 as i32
                            - ram_word!(game, message_box_animation_y1) as i32)
                            as u16
                );
                let v8: bool = u16::overflowing_add(ram_word!(game, g_word_7E0692), r18).1;
                r18 = (r18).wrapping_add(ram_word!(game, g_word_7E0692));
                if !v8 {
                    v7 = v7.wrapping_add(1);
                }
                update_ram_word!(game.message_box_animation_y1, wrapping_add(1));
                v6 = (v6).wrapping_add(2u16);
                if (((v7) as i16).wrapping_sub((ram_word!(game, g_word_7E0698)) as i16) >= 0i16) {
                    break;
                }
            }
            if ((v6) as i16).wrapping_sub(512i16) < 0i16 {
                let v9: u16 = ram_word!(game, reg_BG2VOFS);
                loop {
                    set_ram_word!(game.crocomire_bg2_scroll_hdma_data[(v6 as i32 >> 1i32)] = v9);
                    v6 = (v6).wrapping_add(2u16);
                    if (((v6) as i16).wrapping_sub(512i16) >= 0i16) {
                        break;
                    }
                }
            }
            return;
        }
    }
    let EK: *mut Enemy_Crocomire = Get_Crocomire(ram_word!(game, cur_enemy_index) as i32, game);
    (*EK).crocom_var_A = ((*EK).crocom_var_A).wrapping_add(2u16);
    i = ram_word!(game, croco_word_7E069A) as i32;
    while *((game.g_rom.offset(crate::rtl::lorom_offset(0xa49bc5)) as *const u8 as *mut u16
        as *mut u8)
        .offset(i as isize) as *mut u16)
        != 0xffffu16
    {
        i += 8i32;
    }
    set_ram_word!(game.croco_word_7E069A = (i + 2i32) as u16);
    set_ram_word!(game.hdma_object_channels_bitmask[((*E0).crocom_var_1F as i32 >> 1i32)] = 0u16);
}
pub unsafe fn Crocomire_Func_66(game: &mut Game) {
    let mut v3: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    let E: *mut Enemy_Crocomire = Get_Crocomire(ram_word!(game, cur_enemy_index) as i32, game);
    (*E).crocom_var_E = 0u16;
    (*E).crocom_var_F = 0u16;
    (*E).crocom_var_D = 2048u16;
    let mut i: i32 = 4094i32;
    while i >= 0i32 {
        set_ram_word!(game.tilemap_stuff[(i >> 1i32)] = 824u16);
        i -= 2i32;
    }
    let v2: u16 = ram_word!(game, vram_write_queue_tail);
    v3 = gVramWriteEntry(ram_word!(game, vram_write_queue_tail) as i32, game);
    (*v3).size = 2048u16;
    (*v3).src.addr = 0x2000 as VoidP;
    *(&raw mut (*v3).src.bank as *mut u16) = 126u16;
    (*v3).vram_dst =
        ((*(&raw mut game.g_ram as *mut u8).offset(0x59) as i32 & 0xfei32) << 8i32) as u16;
    set_ram_word!(game.vram_write_queue_tail = (v2).wrapping_add(7u16));
    Crocomire_9BB3(game);
}
static kCrocoEraseLineMasks: [u8; 8] = [0x7f, 0xbf, 0xdf, 0xef, 0xf7, 0xfb, 0xfd, 0xfe];
pub unsafe fn Crocomire_Func_67(game: &mut Game) -> u16 {
    let mut n: i32 = *(&raw mut game.g_ram as *mut u8).offset(0x68c) as i32;
    loop {
        if ram_word!(game, croco_cur_vline_idx) > 48u16 {
            return 1u16;
        }
        let rr: i32 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa49697)) as *const u8
            as *mut u8)
            .offset(ram_word!(game, croco_cur_vline_idx) as isize) as i32;
        if ((*(&raw mut game.g_ram as *mut u8)
            .offset(0x69c)
            .offset(rr as isize)) as i8)
            .wrapping_sub((ram_word!(game, croco_target_0688)) as i8)
            < 0i8
        {
            break;
        }
        if update_ram_word!(game.croco_cur_vline_idx, wrapping_add(1), reread) >= 128u16 {
            set_ram_word!(game.croco_cur_vline_idx = 0u16);
            return 0u16;
        }
    }
    '_c2rust_label: {
        if ram_word!(game, croco_cur_vline_idx) <= 48u16 {
        } else {
            __assert_fail(
                b"croco_cur_vline_idx <= 48\0" as *const u8 as *const ::core::ffi::c_char,
                b"src/sm_a4.c\0" as *const u8 as *const ::core::ffi::c_char,
                1215u32,
                b"uint16 Crocomire_Func_67(void)\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
    };
    let vline_idx: i32 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa49697)) as *const u8
        as *mut u8)
        .offset(ram_word!(game, croco_cur_vline_idx) as isize) as i32;
    let mask: u8 = kCrocoEraseLineMasks[(vline_idx & 7i32) as usize];
    loop {
        let q: i32 = *(&raw mut game.g_ram as *mut u8)
            .offset(0x69c)
            .offset(vline_idx as isize) as i32;
        let j: i32 = 2i32 * (q & 7i32) + ((q & !(7i32)) << 6i32) + 4i32 * (vline_idx & !(7i32));
        let ref mut fresh1 = (*((&raw mut game.g_ram as *mut u8).offset(0x4000) as *mut Ram4000))
            .backups
            .field_0[(j + 0i32) as usize];
        *fresh1 = (*fresh1) & (mask);
        let ref mut fresh2 = (*((&raw mut game.g_ram as *mut u8).offset(0x4000) as *mut Ram4000))
            .backups
            .field_0[(j + 1i32) as usize];
        *fresh2 = (*fresh2) & (mask);
        let ref mut fresh3 = (*((&raw mut game.g_ram as *mut u8).offset(0x4000) as *mut Ram4000))
            .backups
            .field_0[(j + 16i32) as usize];
        *fresh3 = (*fresh3) & (mask);
        let ref mut fresh4 = (*((&raw mut game.g_ram as *mut u8).offset(0x4000) as *mut Ram4000))
            .backups
            .field_0[(j + 17i32) as usize];
        *fresh4 = (*fresh4) & (mask);
        if *(&raw mut game.g_ram as *mut u8)
            .offset(0x69c)
            .offset(vline_idx as isize)
            == 48u8
        {
            break;
        }
        let ref mut fresh5 = *(&raw mut game.g_ram as *mut u8)
            .offset(0x69c)
            .offset(vline_idx as isize);
        *fresh5 = (*fresh5).wrapping_add(1);
        n -= 1;
        if (n == 0) {
            break;
        }
    }
    let mut v7: u16 = 0;
    let mut v8: u16 = 0;
    loop {
        v7 = (ram_word!(game, g_word_7E068A)).wrapping_add(ram_word!(game, croco_word_7E069A));
        v8 = ram_word!(game, vram_write_queue_tail);
        if *((game.g_rom.offset(crate::rtl::lorom_offset(0xa49bc5)) as *const u8 as *mut u16
            as *mut u8)
            .offset(v7 as i32 as isize) as *mut u16)
            != 0xffffu16
        {
            break;
        }
        set_ram_word!(game.g_word_7E068A = 0u16);
    }
    let v9: *mut VramWriteEntry =
        gVramWriteEntry(ram_word!(game, vram_write_queue_tail) as i32, game);
    (*v9).size = *((game.g_rom.offset(crate::rtl::lorom_offset(0xa49bc5)) as *const u8 as *mut u16
        as *mut u8)
        .offset(v7 as i32 as isize) as *mut u16);
    (*v9).src.addr = *((game.g_rom.offset(crate::rtl::lorom_offset(0xa49bcb)) as *const u8
        as *mut u16 as *mut u8)
        .offset(v7 as i32 as isize) as *mut u16);
    *(&raw mut (*v9).src.bank as *mut u16) =
        *((game.g_rom.offset(crate::rtl::lorom_offset(0xa49bc9)) as *const u8 as *mut u16
            as *mut u8)
            .offset(v7 as i32 as isize) as *mut u16);
    (*v9).vram_dst = *((game.g_rom.offset(crate::rtl::lorom_offset(0xa49bc7)) as *const u8
        as *mut u16 as *mut u8)
        .offset(v7 as i32 as isize) as *mut u16);
    set_ram_word!(game.vram_write_queue_tail = (v8).wrapping_add(7u16));
    update_ram_word!(game.g_word_7E068A, wrapping_add(8u16));
    1u16
}
pub unsafe fn Crocomire_Func_68(game: &mut Game) {
    if ram_word!(game, samus_x_pos) as i32 - 640i32 & 0x8000i32 != 0 {
        QueueMusic_Delayed8(5u16, game);
        *((&raw mut game.g_ram as *mut u8).offset(0xcd20).offset(3) as *mut u8 as *mut u16) =
            256u16;
        SpawnHardcodedPlm(
            SpawnHardcodedPlmArgs::new(0x30u8, 0x3u8, 0xb757 as VoidP),
            game,
        );
        set_ram_word!(game.camera_distance_index = 6u16);
        let E0: *mut Enemy_Crocomire = Get_Crocomire(0i32, game);
        (*E0).base.properties =
            ((*E0).base.properties as i32 & 0x7bffi32 | kEnemyProps_Tangible as i32) as u16;
        let E1: *mut Enemy_Crocomire = Get_Crocomire(0x40i32, game);
        (*E1).base.properties = ((*E1).base.properties) | (0x500u16);
        (*E0).crocom_var_D = 4u16;
        (*E1).crocom_var_D = 0u16;
        (*Get_Crocomire(0x80i32, game)).crocom_var_D = 10u16;
        (*Get_Crocomire(0xc0i32, game)).crocom_var_D = 1u16;
        (*E0).crocom_var_B = 0u16;
        (*E0).base.y_height = 56u16;
        Crocomire_9BB3(game);
    }
}
pub unsafe fn Crocomire_Func_69(game: &mut Game) {
    let E: *mut Enemy_Crocomire = Get_Crocomire(0i32, game);
    let v1: u16 = ((*E).base.x_pos).wrapping_sub(2u16);
    (*E).base.x_pos = v1;
    if v1 as i32 - 480i32 & 0x8000i32 != 0 {
        (*E).base.x_pos = 480u16;
        (*E).base.y_pos = 54u16;
        (*E).crocom_var_A = 62u16;
    } else {
        (*E).base.y_pos = 220u16;
    };
}
pub unsafe fn Crocomire_Func_70(game: &mut Game) {
    let E0: *mut Enemy_Crocomire = Get_Crocomire(0i32, game);
    let E1: *mut Enemy_Crocomire = Get_Crocomire(0x40i32, game);
    let E2: *mut Enemy_Crocomire = Get_Crocomire(0x80i32, game);
    let E3: *mut Enemy_Crocomire = Get_Crocomire(0xc0i32, game);
    if *(game.g_rom.offset(crate::rtl::lorom_offset(0xa498ca)) as *const u8 as *mut u16)
        .offset(((*E0).crocom_var_D as i32 >> 1i32) as isize)
        == 0x8080u16
    {
        (*E1).crocom_var_D = -(32640i32) as u16;
        (*E0).crocom_var_D = 128u16;
        let mut i: i32 = 30i32;
        while i >= 0i32 {
            set_ram_word!(
                game.palette_buffer[((i >> 1i32) + 176i32)] =
                    *(game.g_rom.offset(crate::rtl::lorom_offset(0xa4b91d)) as *const u8
                        as *mut u16)
                        .offset((i >> 1i32) as isize)
            );
            i -= 2i32;
        }
        Crocomire_9BB3(game);
    } else {
        let mut crocom_var_D: u16 = (*E0).crocom_var_D;
        let v3: u16 = (*E1).crocom_var_D;
        let v4: i32 = crocom_var_D as i32 >> 1i32;
        if v3
            == *(game.g_rom.offset(crate::rtl::lorom_offset(0xa498ca)) as *const u8 as *mut u16)
                .offset(v4 as isize)
        {
            if (*(game.g_rom.offset(crate::rtl::lorom_offset(0xa498ca)) as *const u8 as *mut u16)
                .offset((crocom_var_D as i32 >> 1i32) as isize) as i32
                & 0x8000i32)
                != 0i32
            {
                let v7: u16 = (*E2).crocom_var_D;
                if v7 != 0 {
                    (*E2).crocom_var_D = (v7).wrapping_sub(1u16);
                    (*E0).crocom_var_D = (crocom_var_D).wrapping_sub(2u16);
                    QueueSfx2_Max6(0x2bu16, game);
                    return;
                }
                let v8: u16 = (crocom_var_D).wrapping_add(2u16);
                (*E2).crocom_var_D = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa498ca))
                    as *const u8 as *mut u16)
                    .offset((v8 as i32 >> 1i32) as isize);
                crocom_var_D = (v8).wrapping_add(2u16);
                (*E3).crocom_var_D = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa498ca))
                    as *const u8 as *mut u16)
                    .offset((crocom_var_D as i32 >> 1i32) as isize);
            }
            (*E0).crocom_var_D = (crocom_var_D).wrapping_add(2u16);
        } else {
            let mut v5: u16 = 0;
            if ((v3) as i16).wrapping_sub(
                (*(game.g_rom.offset(crate::rtl::lorom_offset(0xa498ca)) as *const u8 as *mut u16)
                    .offset(v4 as isize)) as i16,
            ) >= 0i16
            {
                v5 = (v3).wrapping_sub((*E3).crocom_var_D);
            } else {
                v5 = ((*E3).crocom_var_D).wrapping_add(v3);
            }
            (*E1).crocom_var_D = v5;
        }
    };
}
pub unsafe fn Crocomire_Func_71(game: &mut Game) {
    let mut v3: i16 = 0;
    let mut v6: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    let E: *mut Enemy_Crocomire = Get_Crocomire(0i32, game);
    let crocom_var_D: u16 = (*E).crocom_var_D;
    if crocom_var_D != 0 {
        (*E).crocom_var_D = (crocom_var_D).wrapping_sub(1u16);
        v3 = ram_word!(game, croco_target_0688) as i16;
        let v4: i32 = ram_word!(game, croco_target_0688) as i32 >> 1i32;
        if *(game.g_rom.offset(crate::rtl::lorom_offset(0xa499cb)) as *const u8 as *mut u16)
            .offset(v4 as isize)
            != 0xffffu16
        {
            let v5: u16 = ram_word!(game, vram_write_queue_tail);
            v6 = gVramWriteEntry(ram_word!(game, vram_write_queue_tail) as i32, game);
            (*v6).size = 512u16;
            (*v6).src.addr = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa499d9)) as *const u8
                as *mut u16)
                .offset(v4 as isize);
            *(&raw mut (*v6).src.bank as *mut u16) = 173u16;
            (*v6).vram_dst = (*(game.g_rom.offset(crate::rtl::lorom_offset(0xa499cb)) as *const u8
                as *mut u16)
                .offset(v4 as isize) as i32
                + ((*(&raw mut game.g_ram as *mut u8).offset(0x52) as i32 & 7i32) << 13i32))
                as u16;
            set_ram_word!(game.vram_write_queue_tail = (v5).wrapping_add(7u16));
            set_ram_word!(game.croco_target_0688 = ((v3) as u16).wrapping_add(2u16));
        }
    } else {
        (*E).base.x_pos = 480u16;
        (*E).base.y_pos = 54u16;
        (*Get_Crocomire(0x80i32, game)).crocom_var_D = 80u16;
        (*E).crocom_var_E = 0u16;
        (*E).crocom_var_F = 0u16;
        SpawnHardcodedPlm(
            SpawnHardcodedPlmArgs::new(0x20u8, 0x3u8, 0xb753 as VoidP),
            game,
        );
        SpawnHardcodedPlm(
            SpawnHardcodedPlmArgs::new(0x1eu8, 0x3u8, 0xb757 as VoidP),
            game,
        );
        SpawnHardcodedPlm(
            SpawnHardcodedPlmArgs::new(0x70u8, 0xbu8, 0xb747 as VoidP),
            game,
        );
        QueueSfx2_Max6(0x29u16, game);
        (*E).base.current_instruction = addr_kCrocomire_Ilist_E158 as u16;
        (*E).base.instruction_timer = 1u16;
        (*E).base.palette_index = 0u16;
        let mut i: i32 = 30i32;
        while i >= 0i32 {
            set_ram_word!(
                game.palette_buffer[((i >> 1i32) + 144i32)] =
                    *(game.g_rom.offset(crate::rtl::lorom_offset(0xa4b8fd)) as *const u8
                        as *mut u16)
                        .offset((i >> 1i32) as isize)
            );
            i -= 2i32;
        }
        ClearEprojs(game);
        let v8: u16 = 8u16;
        let mut n: i32 = 8i32;
        loop {
            SpawnEprojWithGfx(
                v8,
                ram_word!(game, cur_enemy_index),
                addr_kEproj_CrocomireSpikeWallPieces as u16,
                game,
            );
            n -= 1;
            if (n == 0) {
                break;
            }
        }
        QueueSfx2_Max6(0x30u16, game);
        Crocomire_9BB3(game);
    };
}
pub unsafe fn Crocomire_Func_72(game: &mut Game) {
    let E: *mut Enemy_Crocomire = Get_Crocomire(0i32, game);
    if (*E).base.x_pos as i32 - 224i32 & 0x8000i32 != 0 {
        let mut t: u32 = (((*E).crocom_var_F as u32) << 16i32 | (*E).crocom_var_E as u32)
            .wrapping_add(0x8000u32);
        if (t >> 16i32).wrapping_sub(2u32) & 0x8000u32 == 0 {
            t = t & 0xffffu32 | ((2i32) << 16i32) as u32;
        }
        SetHiLo(&raw mut (*E).crocom_var_F, &raw mut (*E).crocom_var_E, t);
        AddToHiLo(&raw mut (*E).base.x_pos, &raw mut (*E).base.x_subpos, t);
    }
    let E2: *mut Enemy_Crocomire = Get_Crocomire(0x80i32, game);
    if (*E2).crocom_var_D != 0u16 && {
        let fresh19 = (*E2).crocom_var_D;
        (*E2).crocom_var_D = (*E2).crocom_var_D.wrapping_sub(1);
        fresh19 == 1u16
    } {
        (*E).crocom_var_E = 0u16;
        (*E).base.current_instruction = addr_kCrocomire_Ilist_E14A as u16;
        (*E).base.instruction_timer = 1u16;
        Crocomire_9BB3(game);
    }
}
pub unsafe fn Crocomire_Func_73(game: &mut Game) {
    let E: *mut Enemy_Crocomire = Get_Crocomire(0i32, game);
    let mut t: u32 =
        (((*E).crocom_var_F as u32) << 16i32 | (*E).crocom_var_E as u32).wrapping_add(2048u32);
    if (t >> 16i32).wrapping_sub(5u32) & 0x8000u32 == 0 {
        t = t & 0xffffu32 | ((5i32) << 16i32) as u32;
    }
    SetHiLo(&raw mut (*E).crocom_var_F, &raw mut (*E).crocom_var_E, t);
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        0xe000u32,
    );
    AddToHiLo(&raw mut (*E).base.x_pos, &raw mut (*E).base.x_subpos, t);
    if (*E).base.x_pos as i32 - 576i32 & 0x8000i32 == 0 {
        QueueSfx2_Max6(0x25u16, game);
        (*Get_Crocomire(0x40i32, game)).base.palette_index = (*E).base.palette_index;
        (*E).base.current_instruction = addr_kCrocomire_Ilist_E158 as u16;
        (*E).base.instruction_timer = 1u16;
        Crocomire_9BB3(game);
    }
}
pub unsafe fn Crocomire_Instr_8(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    Crocomire_Func_87(jp, 0xffe0u16, game)
}
pub unsafe fn Crocomire_Instr_6(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    Crocomire_Func_87(jp, 0u16, game)
}
pub unsafe fn Crocomire_Instr_9(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    Crocomire_Func_87(jp, 0xfff0u16, game)
}
pub unsafe fn Crocomire_Instr_5(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    Crocomire_Func_87(jp, 0x10u16, game)
}
pub unsafe fn Crocomire_Instr_20(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    Crocomire_Func_87(jp, 0u16, game)
}
pub unsafe fn Crocomire_Instr_21(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    Crocomire_Func_87(jp, 8u16, game)
}
pub unsafe fn Crocomire_Instr_22(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    Crocomire_Func_87(jp, 0x10u16, game)
}
pub unsafe fn Crocomire_Instr_23(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    Crocomire_Func_87(jp, 0x18u16, game)
}
pub unsafe fn Crocomire_Instr_24(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    Crocomire_Func_87(jp, 0x20u16, game)
}
pub unsafe fn Crocomire_Instr_10(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    Crocomire_Func_87(jp, 0x28u16, game)
}
pub unsafe fn Crocomire_Instr_25(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    Crocomire_Func_87(jp, 0x30u16, game)
}
pub unsafe fn Crocomire_Instr_26(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    Crocomire_Func_87(jp, 0x38u16, game)
}
pub unsafe fn Crocomire_Instr_27(k: u16, jp: *const u16, game: &mut Game) -> *const u16 {
    Crocomire_Func_87(jp, 0x40u16, game)
}
pub unsafe fn Crocomire_Func_87(jp: *const u16, a: u16, game: &mut Game) -> *const u16 {
    let E: *mut Enemy_Crocomire = Get_Crocomire(0i32, game);
    game.eproj_spawn_pt = Point16U {
        x: (a as i32 + (*E).base.x_pos as i32 + (ram_word!(game, random_number) as i32 & 7i32))
            as u16,
        y: ((*E).base.y_height as i32 + (*E).base.y_pos as i32 - 16i32) as u16,
    };
    SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 0x15u16, game);
    jp
}
pub unsafe fn Crocomire_Func_88(game: &mut Game) {
    let E: *mut Enemy_Crocomire = Get_Crocomire(0i32, game);
    if (*E).base.current_instruction as i32 + 0x1e3ai32 & 0x8000i32 != 0 {
        let mut t: u32 =
            (((*E).crocom_var_F as u32) << 16i32 | (*E).crocom_var_E as u32).wrapping_add(4096u32);
        if (t >> 16i32).wrapping_sub(6u32) & 0x8000u32 == 0 {
            t = t & 0xffffu32 | ((6i32) << 16i32) as u32;
        }
        SetHiLo(&raw mut (*E).crocom_var_F, &raw mut (*E).crocom_var_E, t);
    } else {
        (*E).base.current_instruction = addr_kCrocomire_Ilist_E1CC as u16;
        (*E).base.instruction_timer = 1u16;
        (*E).base.x_pos = ((*E).base.x_pos).wrapping_add(64u16);
        (*E).base.y_pos = ((*E).base.y_pos).wrapping_add(21u16);
        (*E).base.y_height = 28u16;
        (*E).base.x_width = 40u16;
        SpawnHardcodedPlm(
            SpawnHardcodedPlmArgs::new(0x30u8, 0x3u8, 0xb753 as VoidP),
            game,
        );
        Enemy_ItemDrop_Crocomire(0u16, game);
        Crocomire_9BB3(game);
    };
}
pub unsafe fn Crocomire_9B65(game: &mut Game) {
    *((&raw mut game.g_ram as *mut u8).offset(0xcd20) as *mut u16) = 257u16;
    *((&raw mut game.g_ram as *mut u8).offset(0xcd20).offset(2) as *mut u8 as *mut u16) = 257u16;
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs::new(0x1eu8, 0x3u8, 0xb753 as VoidP),
        game,
    );
    Crocomire_9BB3(game);
}
pub unsafe fn Crocomire_Func_89(game: &mut Game) {
    Crocomire_9BB3(game);
}
pub unsafe fn Crocomire_Func_90(game: &mut Game) {
    let E: *mut Enemy_Crocomire = Get_Crocomire(0i32, game);
    if (*E).base.current_instruction as i32 + 0x1e3ai32 & 0x8000i32 == 0 {
        Crocomire_9BB3(game);
    }
}
pub unsafe fn Crocomire_9B86(game: &mut Game) {
    QueueMusic_Delayed8(6u16, game);
    set_ram_word!(game.camera_distance_index = 0u16);
    let ref mut fresh10 = *((&raw mut game.g_ram as *mut u8)
        .offset(0xd828)
        .offset(ram_word!(game, area_index) as isize) as *mut u8
        as *mut u16);
    *fresh10 = (*fresh10) | (2u16);
    QueueMusic_Delayed8(6u16, game);
    Crocomire_Func_87(::core::ptr::null::<u16>(), 0xfff0u16, game);
    Crocomire_Func_87(::core::ptr::null::<u16>(), 0x10u16, game);
    Crocomire_9BB3(game);
}
pub unsafe fn Crocomire_9BB3(game: &mut Game) {
    let E: *mut Enemy_Crocomire = Get_Crocomire(0i32, game);
    (*E).crocom_var_A = (*E).crocom_var_A.wrapping_add(1);
    (*E).crocom_var_A = (*E).crocom_var_A.wrapping_add(1);
}
pub unsafe fn Crocomire_Func_91(game: &mut Game) {
    Crocomire_9BB3(game);
}
pub unsafe fn Crocomire_Func_92(game: &mut Game) {
    NormalEnemyTouchAi(game);
    let E: *mut Enemy_Crocomire = Get_Crocomire(0i32, game);
    (*E).crocom_var_B = ((*E).crocom_var_B) | (0x4000u16);
    set_ram_word!(game.extra_samus_x_displacement = -(4i32) as u16);
}
pub unsafe fn Crocomire_Func_93(game: &mut Game) {
    let E: *mut Enemy_Crocomire = Get_Crocomire(0i32, game);
    let mut v1: u16 = ((*E).crocom_var_B) & (0xfu16);
    if v1 as i32 - 15i32 & 0x8000i32 != 0 {
        v1 = v1.wrapping_add(1);
    }
    (*E).crocom_var_B = ((*E).crocom_var_B) | (v1);
}
pub unsafe fn Crocomire_Func_94(game: &mut Game) {
    let v0: i32 = ram_word!(game, collision_detection_index) as i32;
    game.eproj_spawn_pt = Point16U {
        x: ram_word!(game, projectile_x_pos[v0]),
        y: ram_word!(game, projectile_y_pos[v0]),
    };
    let v1: u16 = (if (ram_word!(game, projectile_type[v0]) as i32 & 0x200i32) == 0i32 {
        6i32
    } else {
        29i32
    }) as u16;
    SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, v1, game);
}
pub unsafe fn Crocomire_Powerbomb(game: &mut Game) {
    let mut v2: u16 = 0;
    let E: *mut Enemy_Crocomire = Get_Crocomire(0i32, game);
    if (*E).crocom_var_A == 0 && g_word_A4869E != 0 {
        (*E).crocom_var_D = g_word_A4869E;
        if (*E).crocom_var_C != 24u16 {
            (*E).crocom_var_B = ((*E).crocom_var_B as i32 & 0x3ff0i32 | 0x8000i32) as u16;
            (*E).crocom_var_E = 10u16;
            (*E).base.flash_timer = ((*E).base.flash_timer).wrapping_add(4u16);
            (*E).base.ai_handler_bits = ((*E).base.ai_handler_bits) | (2u16);
            (*E).crocom_var_C = 24u16;
            let mut spritemap_pointer: u16 = (*E).base.spritemap_pointer;
            let mut v3: u16 = 0;
            let mut n: i32 = *(RomPtr_A4(spritemap_pointer, game) as *mut u16) as i32;
            loop {
                v2 = *(RomPtr_A4(spritemap_pointer, game) as *mut u16).offset(3);
                v3 = addr_kCrocomire_Ilist_BDAE as u16;
                if v2 as i32 == addr_kCrocomire_BigSprmap_D600 as i32 {
                    break;
                }
                v3 = addr_kCrocomire_Ilist_BDB2 as u16;
                if v2 as i32 == addr_kCrocomire_BigSprmap_D51C as i32 {
                    break;
                }
                spritemap_pointer = (spritemap_pointer).wrapping_add(8u16);
                n -= 1;
                if (n != 0) {
                    continue;
                }
                v3 = addr_kCrocomire_Ilist_BDB6 as u16;
                break;
            }
            let E0: *mut Enemy_Crocomire = Get_Crocomire(0i32, game);
            (*E0).base.current_instruction = v3;
            (*E0).base.instruction_timer = 1u16;
        }
    }
}
pub unsafe fn Crocomire_Func_95(game: &mut Game) {
    let mut v1: u16 = 0;
    let mut v2: u16 = 0;
    let current_block: u64;
    let mut v3: i16 = 0;
    let E: *mut Enemy_Crocomire = Get_Crocomire(0i32, game);
    (*E).base.invincibility_timer = 0u16;
    if (((*E).base.x_pos as i32
        - (*E).base.x_width as i32
        - 256i32
        - ram_word!(game, layer1_x_pos) as i32) as i16)
        >= 0i16
    {
        current_block = 4956146061682418353;
    } else {
        v1 = 0;
        v2 = 0;
        v1 = ram_word!(
            game,
            projectile_type[ram_word!(game, collision_detection_index)]
        );
        if (v1 as i32 & 0xf00i32) != 0i32 {
            v3 = ((v1) as i16) & (0xf00i16);
            v2 = g_word_A4869A;
            if v3 != 256i16 {
                v2 = g_word_A4869C;
                if v3 != 512i16 {
                    v2 = 0u16;
                }
            }
        } else {
            v2 = g_word_A48698;
            if (v1 as i32 & 0x10i32) == 0i32 {
                (*E).base.instruction_timer = g_word_A486A0;
                Crocomire_Func_1(game);
                return;
            }
        }
        if v2 != 0 {
            (*E).crocom_var_D = ((*E).crocom_var_D).wrapping_add(v2);
            current_block = 4956146061682418353;
        } else {
            current_block = 7172762164747879670;
        }
    }
    if current_block == 4956146061682418353 {
        let mut v4: u16 = ((*E).crocom_var_B) & (0xfu16);
        if v4 as i32 - 15i32 & 0x8000i32 != 0 {
            v4 = v4.wrapping_add(1);
        }
        if ((*E).crocom_var_B as i32 & 0x800i32) == 0i32 {
            let mut v5: u16 = g_word_A48692;
            if (*E).crocom_var_C == 8u16 {
                v5 = g_word_A48694;
            }
            (*E).base.instruction_timer = ((*E).base.instruction_timer).wrapping_add(v5);
        }
        (*E).crocom_var_B = (v4 as i32 | (*E).crocom_var_B as i32 & 0xb7f0i32 | 0x800i32) as u16;
        (*E).crocom_var_E = 10u16;
    }
    (*E).base.flash_timer = ((*E).base.flash_timer).wrapping_add(14u16);
    (*E).base.ai_handler_bits = ((*E).base.ai_handler_bits) | (2u16);
}
pub unsafe fn Crocomire_Func_1(game: &mut Game) {
    let v0: i32 = ram_word!(game, collision_detection_index) as i32;
    game.eproj_spawn_pt = Point16U {
        x: ram_word!(game, projectile_x_pos[v0]),
        y: ram_word!(game, projectile_y_pos[v0]),
    };
    let v1: u16 = (if (ram_word!(game, projectile_type[v0]) as i32 & 0x200i32) == 0i32 {
        6i32
    } else {
        29i32
    }) as u16;
    SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, v1, game);
}
pub unsafe fn CrocomireTongue_Init(game: &mut Game) {
    let E: *mut Enemy_Crocomire = Get_Crocomire(ram_word!(game, cur_enemy_index) as i32, game);
    if (*((&raw mut game.g_ram as *mut u8)
        .offset(0xd828)
        .offset(ram_word!(game, area_index) as isize) as *mut u8 as *mut u16) as i32
        & 2i32)
        != 0i32
    {
        (*E).base.properties = ((*E).base.properties as i32 & 0xdcffi32 | 0x300i32) as u16;
    } else {
        (*E).base.current_instruction = addr_kCrocomire_Ilist_BE56 as u16;
        (*E).base.extra_properties = ((*E).base.extra_properties) | (0x404u16);
        (*E).base.instruction_timer = 1u16;
        (*E).crocom_var_A = 23u16;
        (*E).base.palette_index = 3584u16;
    };
}
pub fn CrocomireTongue_Main() {}
