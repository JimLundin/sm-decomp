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
use crate::rtl::abs16;
use crate::rtl::RomPtr;
use crate::rtl::{AddToHiLo, SetHiLo};
extern "C" {
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;

}
pub use crate::types::Consts_A4;
pub use crate::types::SpawnHardcodedPlmArgs;
pub use crate::types::{
    __int16_t, __int32_t, __int8_t, __uint16_t, __uint32_t, __uint8_t, int16, int16_t, int32,
    int32_t, int8, int8_t, uint16, uint16_t, uint32, uint32_t, uint8, uint8_t, uintptr_t,
    EnemyBase, EnemyData, EnemyProps, Func_V, LongPtr, Point16U, SpawnHdmaObject_Args, VoidP,
    VramWriteEntry,
};
pub use crate::types::{Consts_86, Ram4000, Ram4000_Backups};
enemy_layout! {
    Enemy_Crocomire {
        base: EnemyBase,
        words: [
            crocom_var_A, crocom_var_B, crocom_var_C, crocom_var_D, crocom_var_E,
            crocom_var_F, crocom_parameter_1, crocom_parameter_2,
        ],
        pad1: [uint8; 26696],
        words: [
            crocom_var_00, crocom_var_01, crocom_var_02, crocom_var_03, crocom_var_04,
            crocom_var_05, crocom_var_06, crocom_var_07, crocom_var_08, crocom_var_09,
            crocom_var_0A, crocom_var_0B, crocom_var_0C, crocom_var_0D, crocom_var_0E,
            crocom_var_0F, crocom_var_10, crocom_var_11, crocom_var_12, crocom_var_13,
            crocom_var_14, crocom_var_15, crocom_var_16, crocom_var_17, crocom_var_18,
            crocom_var_19, crocom_var_1A, crocom_var_1B, crocom_var_1C, crocom_var_1D,
            crocom_var_1E, crocom_var_1F,
        ],
        pad2: [uint8; 1984],
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
unsafe fn RomPtr_A4(mut addr: uint16_t, game: &mut Game) -> *const uint8 {
    return RomPtr(((0xa40000u32) | ((addr) as u32)), game);
}
#[inline]
unsafe fn RomPtrWithBank(mut bank: uint8, mut addr: uint16_t, game: &mut Game) -> *const uint8 {
    return RomPtr(
        ((bank as ::core::ffi::c_int) << 16 as ::core::ffi::c_int | addr as ::core::ffi::c_int)
            as uint32_t,
        game,
    );
}
#[inline]
unsafe fn RomBankBase(mut bank: uint8, game: &mut Game) -> *const uint8 {
    return RomPtr(
        (((bank as ::core::ffi::c_int) << 16 as ::core::ffi::c_int) + 0x8000 as ::core::ffi::c_int)
            as uint32_t,
        game,
    )
    .offset(-(0x8000 as ::core::ffi::c_int as isize));
}
#[inline]
unsafe fn gVramWriteEntry(mut a: ::core::ffi::c_int, game: &mut Game) -> *mut VramWriteEntry {
    return (&raw mut game.g_ram as *mut uint8).offset((0xd0 as ::core::ffi::c_int + a) as isize)
        as *mut uint8 as *mut VramWriteEntry;
}
#[inline]
unsafe fn gEnemyData(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut EnemyData {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut EnemyData;
}
#[inline]
unsafe fn Get_Crocomire(mut o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_Crocomire {
    return (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_Crocomire;
}
static g_word_A48692: uint16 = 8 as uint16;
static g_word_A48694: uint16 = 8 as uint16;
static g_word_A48698: uint16 = 2 as uint16;
static g_word_A4869A: uint16 = 1 as uint16;
static g_word_A4869C: uint16 = 3 as uint16;
static g_word_A4869E: uint16 = 3 as uint16;
static g_word_A486A0: uint16 = 8 as uint16;
static g_word_A486A2: uint16 = 0x300 as uint16;
static g_word_A486A4: uint16 = 0x640 as uint16;
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
    Crocomire_Func_27(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
    Crocomire_Func_31(game);
}
static off_A486B3: [Option<CrocomireFunc>; 21] = unsafe {
    [
        Some(Crocomire_Func_2 as unsafe fn(uint16, *const uint16, &mut Game) -> *const uint16),
        Some(Crocomire_Func_3 as unsafe fn(uint16, *const uint16, &mut Game) -> *const uint16),
        Some(Crocomire_Func_4 as unsafe fn(uint16, *const uint16, &mut Game) -> *const uint16),
        Some(Crocomire_Func_5 as unsafe fn(uint16, *const uint16, &mut Game) -> *const uint16),
        Some(Crocomire_Func_7 as unsafe fn(uint16, *const uint16, &mut Game) -> *const uint16),
        Some(Crocomire_Func_8 as unsafe fn(uint16, *const uint16, &mut Game) -> *const uint16),
        Some(Crocomire_Func_9 as unsafe fn(uint16, *const uint16, &mut Game) -> *const uint16),
        Some(Crocomire_Func_10 as unsafe fn(uint16, *const uint16, &mut Game) -> *const uint16),
        Some(Crocomire_Func_11 as unsafe fn(uint16, *const uint16, &mut Game) -> *const uint16),
        Some(Crocomire_Func_13 as unsafe fn(uint16, *const uint16, &mut Game) -> *const uint16),
        Some(Crocomire_Func_14 as unsafe fn(uint16, *const uint16, &mut Game) -> *const uint16),
        Some(Crocomire_Func_15 as unsafe fn(uint16, *const uint16, &mut Game) -> *const uint16),
        Some(Crocomire_Func_16 as unsafe fn(uint16, *const uint16, &mut Game) -> *const uint16),
        Some(Crocomire_Func_17 as unsafe fn(uint16, *const uint16, &mut Game) -> *const uint16),
        Some(Crocomire_Func_18 as unsafe fn(uint16, *const uint16, &mut Game) -> *const uint16),
        Some(Crocomire_Func_21 as unsafe fn(uint16, *const uint16, &mut Game) -> *const uint16),
        Some(Crocomire_Func_22 as unsafe fn(uint16, *const uint16, &mut Game) -> *const uint16),
        Some(Crocomire_Func_23 as unsafe fn(uint16, *const uint16, &mut Game) -> *const uint16),
        Some(Crocomire_Func_24 as unsafe fn(uint16, *const uint16, &mut Game) -> *const uint16),
        Some(Crocomire_Func_25 as unsafe fn(uint16, *const uint16, &mut Game) -> *const uint16),
        Some(Crocomire_Func_26 as unsafe fn(uint16, *const uint16, &mut Game) -> *const uint16),
    ]
};
pub unsafe fn Crocomire_Instr_1(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut crocom_var_C: uint16 = (*Get_Crocomire(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    ))
    .crocom_var_C;
    return off_A486B3[(crocom_var_C as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as usize]
        .expect("non-null function pointer")(crocom_var_C, jp, game);
}
pub unsafe fn Crocomire_Func_2(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_Crocomire(k as ::core::ffi::c_int, game))
        .base
        .instruction_timer = 1 as uint16;
    return addr_kCrocomire_Ilist_BADE as ::core::ffi::c_int as uintptr_t as *const uint16;
}
pub unsafe fn Crocomire_Func_3(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_Crocomire(0 as ::core::ffi::c_int, game)).crocom_var_C = 4 as uint16;
    return addr_kCrocomire_Ilist_BBCE as ::core::ffi::c_int as uintptr_t as *const uint16;
}
pub unsafe fn Crocomire_Func_4(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int, game);
    let mut v1: uint16 = abs16(
        (((*E).base.x_pos)
            .wrapping_sub((*((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)))),
    );
    if v1 as ::core::ffi::c_int - 224 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        (*E).crocom_var_B = (((*E).crocom_var_B) | (0x8000u16));
        (*E).crocom_var_C = 18 as uint16;
        return addr_kCrocomire_Ilist_BC56 as ::core::ffi::c_int as uintptr_t as *const uint16;
    }
    return jp;
}
pub unsafe fn Crocomire_Func_5(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int, game);
    if ((*E).crocom_var_B as ::core::ffi::c_int & 0x800 as ::core::ffi::c_int) != 0i32 && {
        (*E).crocom_var_B =
            ((*E).crocom_var_B as ::core::ffi::c_int & !(0x800 as ::core::ffi::c_int)) as uint16;
        (*E).crocom_var_D != 0u16
    } {
        jp = addr_kCrocomire_Ilist_BC30 as ::core::ffi::c_int as uintptr_t as *const uint16;
        (*E).crocom_var_C = 12 as uint16;
    } else if (((*E).base.x_pos) as i16).wrapping_sub(((g_word_A486A2) as i16)) < 0i16 {
        jp = addr_kCrocomire_Ilist_BE7E as ::core::ffi::c_int as uintptr_t as *const uint16;
        (*E).crocom_var_C = 10 as uint16;
    } else if (((jp as *mut uint8).offset_from(RomBankBase(
        (*gEnemyData(0 as ::core::ffi::c_int, game)).bank,
        game,
    )) as ::core::ffi::c_long
        - addr_kCrocomire_Ilist_BC34 as ::core::ffi::c_int as ::core::ffi::c_long)
        as int16)
        >= 0i16
    {
        return addr_kCrocomire_Ilist_BBCE as ::core::ffi::c_int as uintptr_t as *const uint16;
    }
    return jp;
}
pub unsafe fn Crocomire_Instr_14(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16) as ::core::ffi::c_int
        & 0xfff as ::core::ffi::c_int)
        - 1024 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int, game);
        (*E).crocom_var_C = 8 as uint16;
        (*E).crocom_var_F = 0 as uint16;
        return addr_kCrocomire_Ilist_BB36 as ::core::ffi::c_int as uintptr_t as *const uint16;
    }
    return jp;
}
pub unsafe fn Crocomire_Func_7(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int, game);
    let mut crocom_var_B: uint16 = (*E).crocom_var_B;
    if (crocom_var_B as ::core::ffi::c_int & 0x800 as ::core::ffi::c_int) != 0i32 {
        (*E).crocom_var_B = ((crocom_var_B) & (0xf7ffu16));
        jp = addr_kCrocomire_Ilist_BC30 as ::core::ffi::c_int as uintptr_t as *const uint16;
        (*E).crocom_var_C = 12 as uint16;
    } else {
        let mut crocom_var_F: uint16 = (*E).crocom_var_F;
        if crocom_var_F as ::core::ffi::c_int - 18 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            (*E).crocom_var_F = (((*E).crocom_var_F).wrapping_add((2u16)));
            SpawnEprojWithGfx(
                crocom_var_F,
                *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
                addr_stru_868F8F as uint16,
                game,
            );
            QueueSfx3_Max6(0x1c as uint16, game);
        } else {
            jp = addr_kCrocomire_Ilist_BBCA as ::core::ffi::c_int as uintptr_t as *const uint16;
            (*E).crocom_var_C = 6 as uint16;
        }
    }
    return jp;
}
pub unsafe fn Crocomire_Func_8(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int, game);
    let mut crocom_var_B: uint16 = (*E).crocom_var_B;
    if (crocom_var_B as ::core::ffi::c_int & 0x800 as ::core::ffi::c_int) != 0i32 {
        (*E).crocom_var_B = ((crocom_var_B) & (0xf7ffu16));
        jp = addr_kCrocomire_Ilist_BC30 as ::core::ffi::c_int as uintptr_t as *const uint16;
        (*E).crocom_var_C = 12 as uint16;
    }
    return jp;
}
pub unsafe fn Crocomire_Func_9(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut v2: uint16 = 0;
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int, game);
    let mut crocom_var_D: uint16 = (*E).crocom_var_D;
    if crocom_var_D != 0u16 && {
        v2 = ((crocom_var_D).wrapping_sub((1u16)));
        (*E).crocom_var_D = v2;
        (*E).crocom_var_D != 0u16
    } {
        (*E).crocom_var_C = 12 as uint16;
        return addr_kCrocomire_Ilist_BC34 as ::core::ffi::c_int as uintptr_t as *const uint16;
    } else {
        (*E).crocom_var_C = 6 as uint16;
        return addr_kCrocomire_Ilist_BBCE as ::core::ffi::c_int as uintptr_t as *const uint16;
    };
}
pub unsafe fn Crocomire_Func_10(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int, game);
    if (((*E).base.x_pos) as i16).wrapping_sub(((g_word_A486A2) as i16)) >= 0i16 {
        (*E).crocom_var_C = 6 as uint16;
        return addr_kCrocomire_Ilist_BBCE as ::core::ffi::c_int as uintptr_t as *const uint16;
    }
    return jp;
}
pub unsafe fn Crocomire_Func_11(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    (*Get_Crocomire(0 as ::core::ffi::c_int, game)).crocom_var_C = 6 as uint16;
    return addr_kCrocomire_Ilist_BD2A as ::core::ffi::c_int as uintptr_t as *const uint16;
}
pub unsafe fn Crocomire_Func_12(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int, game);
    (*E).crocom_var_B =
        ((*E).crocom_var_B as ::core::ffi::c_int & !(0x400 as ::core::ffi::c_int)) as uint16;
    return addr_kCrocomire_Ilist_BCD8 as ::core::ffi::c_int as uintptr_t as *const uint16;
}
pub unsafe fn Crocomire_Func_13(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int, game);
    if ((*E).crocom_var_B as ::core::ffi::c_int & 0x800 as ::core::ffi::c_int) != 0i32 {
        (*E).crocom_var_B =
            ((*E).crocom_var_B as ::core::ffi::c_int & !(0x800 as ::core::ffi::c_int)) as uint16;
        (*E).crocom_var_C = 20 as uint16;
        return addr_kCrocomire_Ilist_BC30 as ::core::ffi::c_int as uintptr_t as *const uint16;
    } else if (((jp as *mut uint8).offset_from(RomBankBase(
        (*gEnemyData(0 as ::core::ffi::c_int, game)).bank,
        game,
    )) as ::core::ffi::c_long
        - addr_kCrocomire_Ilist_BDA2 as ::core::ffi::c_int as ::core::ffi::c_long)
        as int16)
        >= 0i16
    {
        return addr_kCrocomire_Ilist_BD2A as ::core::ffi::c_int as uintptr_t as *const uint16;
    }
    return jp;
}
pub unsafe fn Crocomire_Func_14(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int, game);
    if ((*E).crocom_var_B as ::core::ffi::c_int & 0x800 as ::core::ffi::c_int) != 0i32 {
        (*E).crocom_var_B =
            ((*E).crocom_var_B as ::core::ffi::c_int & !(0x800 as ::core::ffi::c_int)) as uint16;
        (*E).crocom_var_C = 12 as uint16;
        return addr_kCrocomire_Ilist_BC30 as ::core::ffi::c_int as uintptr_t as *const uint16;
    } else if (((jp as *mut uint8).offset_from(RomBankBase(
        (*gEnemyData(0 as ::core::ffi::c_int, game)).bank,
        game,
    )) as ::core::ffi::c_long
        - addr_kCrocomire_Ilist_BDA2 as ::core::ffi::c_int as ::core::ffi::c_long)
        as int16)
        >= 0i16
    {
        return addr_kCrocomire_Ilist_BD2A as ::core::ffi::c_int as uintptr_t as *const uint16;
    }
    return jp;
}
pub unsafe fn Crocomire_Func_15(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int, game);
    if ((*E).crocom_var_B as ::core::ffi::c_int & 0x800 as ::core::ffi::c_int) != 0i32 {
        (*E).crocom_var_B =
            ((*E).crocom_var_B as ::core::ffi::c_int & !(0x800 as ::core::ffi::c_int)) as uint16;
        (*E).crocom_var_C = 12 as uint16;
        return addr_kCrocomire_Ilist_BC30 as ::core::ffi::c_int as uintptr_t as *const uint16;
    } else if (((jp as *mut uint8).offset_from(RomBankBase(
        (*gEnemyData(0 as ::core::ffi::c_int, game)).bank,
        game,
    )) as ::core::ffi::c_long
        - addr_kCrocomire_Ilist_BDA2 as ::core::ffi::c_int as ::core::ffi::c_long)
        as int16)
        >= 0i16
    {
        return addr_kCrocomire_Ilist_BD2A as ::core::ffi::c_int as uintptr_t as *const uint16;
    }
    return jp;
}
pub unsafe fn Crocomire_Func_16(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int, game);
    let mut v1: uint16 = (((*E).crocom_var_D).wrapping_sub((1u16)));
    (*E).crocom_var_D = v1;
    if v1 as ::core::ffi::c_int - 2 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        (*E).crocom_var_D = 0 as uint16;
        (*E).crocom_var_C = 6 as uint16;
        return addr_kCrocomire_Ilist_BBCE as ::core::ffi::c_int as uintptr_t as *const uint16;
    }
    return jp;
}
pub unsafe fn Crocomire_Func_17(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int, game);
    let mut crocom_var_B: uint16 = (*E).crocom_var_B;
    if (crocom_var_B as ::core::ffi::c_int & 0x800 as ::core::ffi::c_int) != 0i32 {
        (*E).crocom_var_B = (crocom_var_B as ::core::ffi::c_int & 0x1f00 as ::core::ffi::c_int
            | 0xa000 as ::core::ffi::c_int) as uint16;
        (*E).crocom_var_D = 1 as uint16;
        (*E).crocom_var_E = 10 as uint16;
        (*E).crocom_var_C = 12 as uint16;
        QueueSfx2_Max6(0x54 as uint16, game);
    } else {
        (*Get_Crocomire(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        ))
        .crocom_var_C = 10 as uint16;
        return addr_kCrocomire_Ilist_BD8E as ::core::ffi::c_int as uintptr_t as *const uint16;
    }
    return jp;
}
pub unsafe fn Crocomire_Func_18(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut result: *const uint16 = Crocomire_Func_2(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        jp,
        game,
    );
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int, game);
    (*E).crocom_var_B = (((*E).crocom_var_B) | (0x200u16));
    (*E).crocom_var_D = 32 as uint16;
    (*E).crocom_var_C = 30 as uint16;
    return result;
}
pub unsafe fn Crocomire_Func_19(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut v2: bool = false;
    jp = Crocomire_Func_2(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        jp,
        game,
    );
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int, game);
    if (*E).crocom_var_D == 0 || {
        v2 = (*E).crocom_var_D == 1u16;
        (*E).crocom_var_D = (*E).crocom_var_D.wrapping_sub(1);
        v2 as ::core::ffi::c_int != 0
    } {
        jp = Crocomire_Func_20(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        );
        (*E).crocom_var_C = 32 as uint16;
    }
    return jp;
}
pub unsafe fn Crocomire_Func_20(mut k: uint16, game: &mut Game) -> *const uint16 {
    (*Get_Crocomire(k as ::core::ffi::c_int, game)).crocom_var_C = 20 as uint16;
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int, game);
    (*E).crocom_var_B = (*E).crocom_var_B;
    return addr_kCrocomire_Ilist_BAEA as ::core::ffi::c_int as uintptr_t as *const uint16;
}
pub unsafe fn Crocomire_Func_21(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int, game);
    if ((*E).crocom_var_B as ::core::ffi::c_int & 0x100 as ::core::ffi::c_int) != 0i32 {
        jp = Crocomire_Func_2(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            jp,
            game,
        );
        (*E).crocom_var_D = 16 as uint16;
        (*E).crocom_var_C = 34 as uint16;
    } else {
        jp = Crocomire_Func_20(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        );
        (*E).crocom_var_C = 32 as uint16;
    }
    return jp;
}
pub unsafe fn Crocomire_Func_22(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    k = *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16);
    jp = Crocomire_Func_2(k, jp, game);
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int, game);
    if (*E).crocom_var_D == 0 {
        (*E).crocom_var_B = (((*E).crocom_var_B) | (0x2000u16));
        jp = Crocomire_Func_3(k, jp, game);
        (*E).crocom_var_C = 36 as uint16;
    }
    return jp;
}
pub unsafe fn Crocomire_Func_23(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    k = *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16);
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int, game);
    if (*E).base.x_pos as ::core::ffi::c_int - 672 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        jp = Crocomire_Func_3(k, jp, game);
        (*E).crocom_var_C = 36 as uint16;
        (*E).crocom_var_D = 3 as uint16;
    } else {
        if ((*E).crocom_var_B as ::core::ffi::c_int & 0x4000 as ::core::ffi::c_int) == 0i32 {
            (*E).crocom_var_C = 38 as uint16;
            jp = Crocomire_Func_12(k, jp, game);
        }
        if ((*E).crocom_var_B as ::core::ffi::c_int & 0x4000 as ::core::ffi::c_int) != 0i32 {
            (*E).crocom_var_D = 5 as uint16;
            jp = addr_kCrocomire_Ilist_BCD8 as ::core::ffi::c_int as uintptr_t as *const uint16;
            (*E).crocom_var_F = (*E).crocom_var_C;
            (*E).crocom_var_C = 42 as uint16;
        }
    }
    return jp;
}
pub unsafe fn Crocomire_Func_24(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut v2: bool = false;
    k = *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16);
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int, game);
    if (*E).crocom_var_D == 0 || {
        v2 = (*E).crocom_var_D == 1u16;
        (*E).crocom_var_D = (*E).crocom_var_D.wrapping_sub(1);
        v2 as ::core::ffi::c_int != 0
    } {
        jp = Crocomire_Func_3(k, jp, game);
        (*E).crocom_var_C = 40 as uint16;
        (*E).crocom_var_B =
            ((*E).crocom_var_B as ::core::ffi::c_int & !(0x400 as ::core::ffi::c_int)) as uint16;
    } else {
        (*E).crocom_var_C = 36 as uint16;
        (*Get_Crocomire(0x40 as ::core::ffi::c_int, game)).crocom_var_D = 0 as uint16;
        (*E).crocom_var_B = (((*E).crocom_var_B) | (0x400u16));
        return addr_kCrocomire_Ilist_BCD8 as ::core::ffi::c_int as uintptr_t as *const uint16;
    }
    return jp;
}
pub unsafe fn Crocomire_Func_25(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    k = *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16);
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int, game);
    let mut crocom_var_B: uint16 = (*E).crocom_var_B;
    if (crocom_var_B as ::core::ffi::c_int & 0x2000 as ::core::ffi::c_int) == 0i32 {
        (*E).crocom_var_B = ((crocom_var_B) & (0xfcffu16));
    }
    jp = Crocomire_Func_3(k, jp, game);
    (*E).crocom_var_C = 40 as uint16;
    return jp;
}
pub unsafe fn Crocomire_Func_26(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int, game);
    if (*E).crocom_var_D != 0 {
        let mut crocom_var_B: uint16 = (*E).crocom_var_B;
        if (crocom_var_B as ::core::ffi::c_int & 0x4000 as ::core::ffi::c_int) != 0i32 {
            (*E).crocom_var_D = (*E).crocom_var_D.wrapping_sub(1);
            QueueSfx2_Max6(0x3b as uint16, game);
            return addr_kCrocomire_Ilist_BCD8 as ::core::ffi::c_int as uintptr_t as *const uint16;
        } else {
            (*E).crocom_var_B = ((crocom_var_B) & (0xbfffu16));
            (*E).crocom_var_C = 12 as uint16;
        }
    } else {
        (*E).crocom_var_B =
            ((*E).crocom_var_B as ::core::ffi::c_int & !(0x4000 as ::core::ffi::c_int)) as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).crocom_var_C = (*E).crocom_var_F;
        return addr_kCrocomire_Ilist_BCD8 as ::core::ffi::c_int as uintptr_t as *const uint16;
    }
    return jp;
}
pub unsafe fn Crocomire_Init(game: &mut Game) {
    let mut v7: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    *((&raw mut game.g_ram as *mut uint8).offset(0x179c) as *mut uint16) = 6 as uint16;
    let mut v0: uint16 = 0 as uint16;
    loop {
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 824 as uint16;
        v0 = ((v0).wrapping_add((2u16)));
        if !(((v0) as i16).wrapping_sub((4096i16)) < 0i16) {
            break;
        }
    }
    if (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xd828)
        .offset(*((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) as isize)
        as *mut uint8 as *mut uint16) as ::core::ffi::c_int
        & 2 as ::core::ffi::c_int)
        != 0i32
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xcd20) as *mut uint16) = 257 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(2) as *mut uint8
            as *mut uint16) = 257 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x688) as *mut uint16) = 0 as uint16;
        let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int, game);
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            & 0x7bff as ::core::ffi::c_int
            | 0x400 as ::core::ffi::c_int) as uint16;
        SpawnHardcodedPlm(
            SpawnHardcodedPlmArgs {
                field_0: 0x20 as uint8,
                field_1: 0x3 as uint8,
                field_2: 0xb753 as VoidP,
            },
            game,
        );
        SpawnHardcodedPlm(
            SpawnHardcodedPlmArgs {
                field_0: 0x1e as uint8,
                field_1: 0x3 as uint8,
                field_2: 0xb753 as VoidP,
            },
            game,
        );
        SpawnHardcodedPlm(
            SpawnHardcodedPlmArgs {
                field_0: 0x61 as uint8,
                field_1: 0xb as uint8,
                field_2: 0xb747 as VoidP,
            },
            game,
        );
        (*E).crocom_var_A = 84 as uint16;
        (*E).base.current_instruction = addr_kCrocomire_Ilist_E1CC as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.x_pos = 576 as uint16;
        (*E).base.y_pos = 144 as uint16;
        (*E).base.y_height = 28 as uint16;
        (*E).base.x_width = 40 as uint16;
        let mut v6: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16);
        v7 = gVramWriteEntry(
            *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        );
        (*v7).size = 2048 as uint16;
        (*v7).src.addr = 0x2000 as VoidP;
        *(&raw mut (*v7).src.bank as *mut uint16) = 126 as uint16;
        (*v7).vram_dst = ((*(&raw mut game.g_ram as *mut uint8).offset(0x59) as ::core::ffi::c_int
            & 0xfc as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) =
            ((v6).wrapping_add((7u16)));
    } else {
        DisableMinimapAndMarkBossRoomAsExplored(game);
        *((&raw mut game.g_ram as *mut uint8).offset(0x69a) as *mut uint16) = 0 as uint16;
        let mut E_0: *mut Enemy_Crocomire = Get_Crocomire(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        );
        (*E_0).crocom_var_A = 0 as uint16;
        (*E_0).crocom_var_E = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xcd20) as *mut uint16) = 0 as uint16;
        let mut i: ::core::ffi::c_int = 32 as ::core::ffi::c_int;
        while i >= 0i32 {
            let mut v3: ::core::ffi::c_int = i >> 1 as ::core::ffi::c_int;
            *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16)
                .offset((v3 + 160 as ::core::ffi::c_int) as isize) =
                *(game.g_rom.offset(crate::rtl::lorom_offset(0xa4b8bd)) as *const uint8
                    as *mut uint16)
                    .offset(v3 as isize);
            *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16)
                .offset((v3 + 208 as ::core::ffi::c_int) as isize) =
                *(game.g_rom.offset(crate::rtl::lorom_offset(0xa4b8dd)) as *const uint8
                    as *mut uint16)
                    .offset(v3 as isize);
            i -= 2 as ::core::ffi::c_int;
        }
        (*E_0).crocom_var_C = 4 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x179e) as *mut uint16) = 16 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x941) as *mut uint16) = 2 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x179a) as *mut uint16) = 1024 as uint16;
        (*E_0).base.current_instruction = addr_kCrocomire_Ilist_BADE as uint16;
        (*E_0).base.extra_properties = (((*E_0).base.extra_properties) | (4u16));
        (*E_0).base.instruction_timer = 1 as uint16;
    };
}
pub unsafe fn Crocomire_Func_27(mut k: uint16, game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xb7) as *mut uint16) =
        ((67u16).wrapping_sub(((*Get_Crocomire(k as ::core::ffi::c_int, game)).base.y_pos)));
    let mut v1: uint16 = 32 as uint16;
    let mut spritemap_pointer: uint16 = 0;
    loop {
        spritemap_pointer = (*Get_Crocomire(0 as ::core::ffi::c_int, game))
            .base
            .spritemap_pointer;
        if spritemap_pointer
            == *(game.g_rom.offset(crate::rtl::lorom_offset(0xa48b79)) as *const uint8
                as *mut uint16)
                .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
        {
            break;
        }
        v1 = ((v1).wrapping_sub((2u16)));
        if (v1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
            Crocomire_8BA4(game);
            return;
        }
    }
    let ref mut fresh8 = *((&raw mut game.g_ram as *mut uint8).offset(0xb7) as *mut uint16);
    *fresh8 =
        ((*fresh8).wrapping_add((*(RomPtr_A4(spritemap_pointer, game) as *mut uint16).offset(14))));
    Crocomire_8BA4(game);
}
pub unsafe fn Crocomire_8BA4(game: &mut Game) {
    let mut current_block: u64;
    let mut E0: *mut Enemy_Crocomire = Get_Crocomire(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut E1: *mut Enemy_Crocomire = Get_Crocomire(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int,
        game,
    );
    (*E1).base.x_pos = (((*E1).crocom_var_A).wrapping_add(((*E0).base.x_pos)));
    (*E1).base.y_pos = (*E0).base.y_pos;
    let mut x_pos: uint16 = (*E0).base.x_pos;
    if ((x_pos) as i16).wrapping_sub(
        ((*((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16)) as i16),
    ) >= 0i16
    {
        if (*E0).base.x_pos as ::core::ffi::c_int
            - 128 as ::core::ffi::c_int
            - (*((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16)
                as ::core::ffi::c_int
                + 256 as ::core::ffi::c_int)
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            current_block = 6937071982253665452;
        } else {
            current_block = 1762290943679506230;
        }
    } else if ((x_pos as ::core::ffi::c_int + 128 as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16) as ::core::ffi::c_int)
        as int16)
        < 0i16
    {
        current_block = 1762290943679506230;
    } else {
        current_block = 6937071982253665452;
    }
    match current_block {
        6937071982253665452 => {
            let mut v6: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x911)
                as *mut uint16) as ::core::ffi::c_int
                - (*E0).base.x_pos as ::core::ffi::c_int
                + 51 as ::core::ffi::c_int) as uint16;
            let mut v4: uint16 = abs16(v6);
            if v4 as ::core::ffi::c_int - 284 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                == 0
            {
                v6 = 256 as uint16;
            }
            *((&raw mut game.g_ram as *mut uint8).offset(0xb5) as *mut uint16) = v6;
            return;
        }
        _ => {
            *((&raw mut game.g_ram as *mut uint8).offset(0xb5) as *mut uint16) = 256 as uint16;
            return;
        }
    };
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
    let mut v0: ::core::ffi::c_int = (*Get_Crocomire(0 as ::core::ffi::c_int, game)).crocom_var_A
        as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    off_A48C14[v0 as usize].expect("non-null function pointer")(game);
    Crocomire_Func_30(game);
    Crocomire_Func_31(game);
}
pub unsafe fn Crocomire_Func_28(game: &mut Game) {
    Crocomire_Func_37(game);
    *((&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(4) as *mut uint8 as *mut uint16) =
        257 as uint16;
    if *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) as ::core::ffi::c_int
        - 1312 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(4) as *mut uint8
            as *mut uint16) = 256 as uint16;
    }
    Crocomire_Func_27(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
}
pub unsafe fn Crocomire_Func_29(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xb5) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xb7) as *mut uint16) = 0 as uint16;
}
pub unsafe fn Crocomire_Func_30(game: &mut Game) {
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int, game);
    if (*E).crocom_var_A == 0
        && (((*E).base.x_pos as ::core::ffi::c_int
            - (*E).base.x_width as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0xafe) as *mut uint16)
                as ::core::ffi::c_int
            - *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16)
                as ::core::ffi::c_int) as int16)
            < 0i16
    {
        NormalEnemyTouchAi(game);
        *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) =
            ((*E).base.x_pos as ::core::ffi::c_int
                - (*E).base.x_width as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0xafe) as *mut uint16)
                    as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb10) as *mut uint16) =
            *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16);
        *((&raw mut game.g_ram as *mut uint8).offset(0xb58) as *mut uint16) =
            -(4 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xb5c) as *mut uint16) =
            -(1 as ::core::ffi::c_int) as uint16;
    }
}
pub unsafe fn Crocomire_Func_31(game: &mut Game) {
    let mut j: uint16 = 0;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x797) as *mut uint16) == 0 {
        if (*Get_Crocomire(0 as ::core::ffi::c_int, game))
            .base
            .flash_timer
            != 0u16
            && (*((&raw mut game.g_ram as *mut uint8).offset(0xe44) as *mut uint16)
                as ::core::ffi::c_int
                & 2 as ::core::ffi::c_int)
                != 0i32
        {
            let mut i: ::core::ffi::c_int = 14 as ::core::ffi::c_int;
            while i >= 0i32 {
                *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(
                    ((i >> 1 as ::core::ffi::c_int) + 112 as ::core::ffi::c_int) as isize,
                ) = 0x7fff as uint16;
                i -= 2 as ::core::ffi::c_int;
            }
        } else {
            j = 14 as uint16;
            while (j as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) == 0i32 {
                *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(
                    ((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int)
                        + 112 as ::core::ffi::c_int) as isize,
                ) = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa4b89d)) as *const uint8
                    as *mut uint16)
                    .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
                j = ((j).wrapping_sub((2u16)));
            }
        }
    }
}
pub unsafe fn Crocomire_Instr_11(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    QueueSfx2_Max6(0x74 as uint16, game);
    return jp;
}
pub unsafe fn Crocomire_Instr_7(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    QueueSfx2_Max6(0x25 as uint16, game);
    return jp;
}
pub unsafe fn Crocomire_Instr_19(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    QueueSfx2_Max6(0x75 as uint16, game);
    return jp;
}
pub unsafe fn Crocomire_Func_35(game: &mut Game) {
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int, game);
    let mut crocom_var_20: uint16 = (*E).crocom_var_20;
    if crocom_var_20 != 0 {
        let mut v2: uint16 = ((crocom_var_20).wrapping_sub((1u16)));
        (*E).crocom_var_20 = v2;
        if v2 == 0 {
            (*E).crocom_var_20 = 32 as uint16;
            QueueSfx3_Max6(0x22 as uint16, game);
        }
    }
}
pub unsafe fn Crocomire_Func_36(game: &mut Game) {
    Crocomire_Func_35(game);
    Crocomire_8BA4(game);
    Crocomire_8D47(game);
}
pub unsafe fn Crocomire_8D47(game: &mut Game) {
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int, game);
    let mut crocom_var_D: uint16 = (*E).crocom_var_D;
    if crocom_var_D != 0 {
        (*E).crocom_var_D = ((crocom_var_D).wrapping_sub((1u16)));
    } else {
        (*E).crocom_var_A = (*E).crocom_var_A.wrapping_add(1);
        (*E).crocom_var_A = (*E).crocom_var_A.wrapping_add(1);
        (*E).crocom_var_E = 768 as uint16;
    };
}
pub unsafe fn Crocomire_Func_37(game: &mut Game) {
    let mut E0: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int, game);
    if (*E0).base.x_pos as ::core::ffi::c_int - 1536 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x9000) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x9002) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x9006) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x900a) as *mut uint16) = 0 as uint16;
    } else if (*E0).base.x_pos as ::core::ffi::c_int - 1552 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x9000) as *mut uint16) == 0 {
            *((&raw mut game.g_ram as *mut uint8).offset(0x9000) as *mut uint16) = 1 as uint16;
            game.eproj_spawn_pt = Point16U {
                x: 1536 as uint16,
                y: 176 as uint16,
            };
            SpawnEprojWithRoomGfx(
                addr_kEproj_DustCloudExplosion as uint16,
                0x15 as uint16,
                game,
            );
        }
    } else if (*E0).base.x_pos as ::core::ffi::c_int - 1552 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x9002) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x9006) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x900a) as *mut uint16) = 0 as uint16;
    } else if (*E0).base.x_pos as ::core::ffi::c_int - 1568 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x9002) as *mut uint16) == 0 {
            *((&raw mut game.g_ram as *mut uint8).offset(0x9002) as *mut uint16) = 1 as uint16;
            SpawnHardcodedPlm(
                SpawnHardcodedPlmArgs {
                    field_0: 0x61 as uint8,
                    field_1: 0xb as uint8,
                    field_2: 0xb74b as VoidP,
                },
                game,
            );
            game.eproj_spawn_pt = Point16U {
                x: 1568 as uint16,
                y: 176 as uint16,
            };
            SpawnEprojWithRoomGfx(
                addr_kEproj_DustCloudExplosion as uint16,
                0x15 as uint16,
                game,
            );
        }
    } else {
        let mut x_pos: uint16 = (*E0).base.x_pos;
        if x_pos as ::core::ffi::c_int - 1568 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            != 0
        {
            *((&raw mut game.g_ram as *mut uint8).offset(0x9006) as *mut uint16) = 0 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x900a) as *mut uint16) = 0 as uint16;
        } else if x_pos as ::core::ffi::c_int - 1584 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            if *((&raw mut game.g_ram as *mut uint8).offset(0x9006) as *mut uint16) == 0 {
                *((&raw mut game.g_ram as *mut uint8).offset(0x9006) as *mut uint16) = 1 as uint16;
                SpawnHardcodedPlm(
                    SpawnHardcodedPlmArgs {
                        field_0: 0x62 as uint8,
                        field_1: 0xb as uint8,
                        field_2: 0xb74b as VoidP,
                    },
                    game,
                );
                SpawnHardcodedPlm(
                    SpawnHardcodedPlmArgs {
                        field_0: 0x63 as uint8,
                        field_1: 0xb as uint8,
                        field_2: 0xb74b as VoidP,
                    },
                    game,
                );
                game.eproj_spawn_pt = Point16U {
                    x: 1584 as uint16,
                    y: 176 as uint16,
                };
                SpawnEprojWithRoomGfx(
                    addr_kEproj_DustCloudExplosion as uint16,
                    0x15 as uint16,
                    game,
                );
            }
        } else if ((x_pos) as i16).wrapping_sub(((g_word_A486A4) as i16)) >= 0i16 {
            Crocomire_8EE5(game);
            (*E0).crocom_var_20 = 1 as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x9018) as *mut uint16) = 1 as uint16;
            let mut v3: uint16 =
                *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16);
            let mut EK: *mut Enemy_Crocomire = Get_Crocomire(
                *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                    as ::core::ffi::c_int,
                game,
            );
            (*EK).crocom_var_A = (((*EK).crocom_var_A).wrapping_add((2u16)));
            (*Get_Crocomire(0 as ::core::ffi::c_int, game)).crocom_var_00 = 2 as uint16;
            (*Get_Crocomire(0x40 as ::core::ffi::c_int, game)).crocom_var_00 = 2 as uint16;
            (*Get_Crocomire(0x80 as ::core::ffi::c_int, game)).crocom_var_00 = 2 as uint16;
            (*Get_Crocomire(0xc0 as ::core::ffi::c_int, game)).crocom_var_00 = 2 as uint16;
            (*Get_Crocomire(0x100 as ::core::ffi::c_int, game)).crocom_var_00 = 2 as uint16;
            (*Get_Crocomire(0x140 as ::core::ffi::c_int, game)).crocom_var_00 = 2 as uint16;
            (*Get_Crocomire(0 as ::core::ffi::c_int, game)).crocom_var_01 = 0 as uint16;
            (*Get_Crocomire(0x40 as ::core::ffi::c_int, game)).crocom_var_01 = 0 as uint16;
            (*Get_Crocomire(0x80 as ::core::ffi::c_int, game)).crocom_var_01 = 0 as uint16;
            (*Get_Crocomire(0xc0 as ::core::ffi::c_int, game)).crocom_var_01 = 0 as uint16;
            (*Get_Crocomire(0x100 as ::core::ffi::c_int, game)).crocom_var_01 = 0 as uint16;
            (*Get_Crocomire(0x140 as ::core::ffi::c_int, game)).crocom_var_01 = 0 as uint16;
            QueueSfx2_Max6(0x3b as uint16, game);
            *(((&raw mut game.g_ram as *mut uint8).offset(0x9015) as *mut uint16 as *mut uint8)
                .offset(1) as *mut uint16) = 0 as uint16;
            SpawnHardcodedPlm(
                SpawnHardcodedPlmArgs {
                    field_0: 0x4e as uint8,
                    field_1: 0x3 as uint8,
                    field_2: 0xb757 as VoidP,
                },
                game,
            );
            (*EK).base.current_instruction = addr_kCrocomire_Ilist_BFB0 as uint16;
            (*EK).base.instruction_timer = 1 as uint16;
            (*EK).base.properties = (((*EK).base.properties) | ((kEnemyProps_Tangible) as u16));
            let mut E1: *mut Enemy_Crocomire = Get_Crocomire(
                *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                    as ::core::ffi::c_int
                    + 64 as ::core::ffi::c_int,
                game,
            );
            (*E1).base.instruction_timer = 0x7fff as uint16;
            (*E1).base.current_instruction = addr_kCrocomire_Ilist_BF62 as uint16;
            let fresh11 = &raw mut (*Get_Crocomire(0x40 as ::core::ffi::c_int, game))
                .base
                .properties;
            fresh11.write_unaligned(((fresh11.read_unaligned()) | (0x100u16)));
            (*EK).crocom_var_E = 0 as uint16;
            (*EK).crocom_var_F = 0 as uint16;
            (*EK).crocom_var_D = 2048 as uint16;
            (*Get_Crocomire(0 as ::core::ffi::c_int, game))
                .base
                .y_height = 16 as uint16;
        }
    };
}
pub unsafe fn Crocomire_8EE5(game: &mut Game) {
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs {
            field_0: 0x61 as uint8,
            field_1: 0xb as uint8,
            field_2: 0xb74f as VoidP,
        },
        game,
    );
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs {
            field_0: 0x62 as uint8,
            field_1: 0xb as uint8,
            field_2: 0xb74f as VoidP,
        },
        game,
    );
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs {
            field_0: 0x63 as uint8,
            field_1: 0xb as uint8,
            field_2: 0xb74f as VoidP,
        },
        game,
    );
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs {
            field_0: 0x64 as uint8,
            field_1: 0xb as uint8,
            field_2: 0xb74f as VoidP,
        },
        game,
    );
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs {
            field_0: 0x65 as uint8,
            field_1: 0xb as uint8,
            field_2: 0xb74f as VoidP,
        },
        game,
    );
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs {
            field_0: 0x66 as uint8,
            field_1: 0xb as uint8,
            field_2: 0xb74f as VoidP,
        },
        game,
    );
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs {
            field_0: 0x67 as uint8,
            field_1: 0xb as uint8,
            field_2: 0xb74f as VoidP,
        },
        game,
    );
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs {
            field_0: 0x68 as uint8,
            field_1: 0xb as uint8,
            field_2: 0xb74f as VoidP,
        },
        game,
    );
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs {
            field_0: 0x69 as uint8,
            field_1: 0xb as uint8,
            field_2: 0xb74f as VoidP,
        },
        game,
    );
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs {
            field_0: 0x6a as uint8,
            field_1: 0xb as uint8,
            field_2: 0xb74f as VoidP,
        },
        game,
    );
    game.eproj_spawn_pt = Point16U {
        x: 1536 as uint16,
        y: 176 as uint16,
    };
    SpawnEprojWithRoomGfx(
        addr_kEproj_DustCloudExplosion as uint16,
        0x15 as uint16,
        game,
    );
    game.eproj_spawn_pt = Point16U {
        x: 1552 as uint16,
        y: 192 as uint16,
    };
    SpawnEprojWithRoomGfx(
        addr_kEproj_DustCloudExplosion as uint16,
        0x15 as uint16,
        game,
    );
    game.eproj_spawn_pt = Point16U {
        x: 1568 as uint16,
        y: 176 as uint16,
    };
    SpawnEprojWithRoomGfx(
        addr_kEproj_DustCloudExplosion as uint16,
        0x15 as uint16,
        game,
    );
    game.eproj_spawn_pt = Point16U {
        x: 1584 as uint16,
        y: 192 as uint16,
    };
    SpawnEprojWithRoomGfx(
        addr_kEproj_DustCloudExplosion as uint16,
        0x15 as uint16,
        game,
    );
    game.eproj_spawn_pt = Point16U {
        x: 1600 as uint16,
        y: 192 as uint16,
    };
    SpawnEprojWithRoomGfx(
        addr_kEproj_DustCloudExplosion as uint16,
        0x15 as uint16,
        game,
    );
    game.eproj_spawn_pt = Point16U {
        x: 1616 as uint16,
        y: 192 as uint16,
    };
    SpawnEprojWithRoomGfx(
        addr_kEproj_DustCloudExplosion as uint16,
        0x15 as uint16,
        game,
    );
    game.eproj_spawn_pt = Point16U {
        x: 1632 as uint16,
        y: 192 as uint16,
    };
    SpawnEprojWithRoomGfx(
        addr_kEproj_DustCloudExplosion as uint16,
        0x15 as uint16,
        game,
    );
}
pub unsafe fn Crocomire_Instr_2(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x183e) as *mut uint16) = 4 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1840) as *mut uint16) = 5 as uint16;
    QueueSfx2_Max6(0x76 as uint16, game);
    return jp;
}
pub unsafe fn Crocomire_Instr_4(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    if ((*Get_Crocomire(0 as ::core::ffi::c_int, game)).crocom_var_B as ::core::ffi::c_int
        & 0x800 as ::core::ffi::c_int)
        == 0i32
    {
        Enemy_MoveRight_IgnoreSlopes(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            (-(4 as ::core::ffi::c_int) as int16 as int32) << 16 as ::core::ffi::c_int,
            game,
        );
    }
    return jp;
}
pub unsafe fn Crocomire_Instr_3(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    Crocomire_Func_43(game);
    return Crocomire_Instr_4(k, jp, game);
}
pub unsafe fn Crocomire_Instr_15(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    Crocomire_Func_43(game);
    return Crocomire_Instr_4(k, jp, game);
}
pub unsafe fn Crocomire_Func_43(game: &mut Game) {
    let mut v1: uint16 =
        ((*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)) & (0x1fu16));
    if ((*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)) as i16)
        .wrapping_sub((4096i16))
        >= 0i16
    {
        v1 = -(v1 as ::core::ffi::c_int) as uint16;
    }
    Crocomire_Func_87(::core::ptr::null::<uint16>(), v1, game);
}
pub unsafe fn Crocomire_Instr_16(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    if Enemy_MoveRight_IgnoreSlopes(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        (-(4 as ::core::ffi::c_int) as int16 as int32) << 16 as ::core::ffi::c_int,
        game,
    ) != 0
    {
        (*Get_Crocomire(0 as ::core::ffi::c_int, game)).crocom_var_C = 14 as uint16;
        return addr_kCrocomire_Ilist_BF3C as ::core::ffi::c_int as uintptr_t as *const uint16;
    } else {
        let mut v2: uint16 = 32 as uint16;
        if *((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16) as ::core::ffi::c_int
            - 2048 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            v2 = -(32 as ::core::ffi::c_int) as uint16;
        }
        return Crocomire_Func_87(
            jp,
            (v2 as ::core::ffi::c_int
                + (*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
                    as ::core::ffi::c_int
                    & 0xf as ::core::ffi::c_int)) as uint16,
            game,
        );
    };
}
pub unsafe fn Crocomire_Instr_13(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int, game);
    if (((*E).base.x_pos as ::core::ffi::c_int
        - (*E).base.x_width as ::core::ffi::c_int
        - 260 as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16) as ::core::ffi::c_int)
        as int16)
        < 0i16
    {
        Enemy_MoveRight_IgnoreSlopes(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            (4 as int16 as int32) << 16 as ::core::ffi::c_int,
            game,
        );
    }
    return jp;
}
pub unsafe fn Crocomire_Instr_18(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    Enemy_MoveRight_IgnoreSlopes(k, (4 as int16 as int32) << 16 as ::core::ffi::c_int, game);
    return jp;
}
pub unsafe fn Crocomire_Instr_12(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    Crocomire_Func_43(game);
    return Crocomire_Instr_13(k, jp, game);
}
pub unsafe fn Crocomire_Instr_17(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    Crocomire_Func_43(game);
    return Crocomire_Instr_18(k, jp, game);
}
pub unsafe fn Crocomire_Func_49(game: &mut Game) {
    Crocomire_Func_50(game);
    Crocomire_Func_53(game);
    Crocomire_Func_55(game);
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int, game);
    if (*E).crocom_var_A == 62u16 {
        QueueMusic_Delayed8(6 as uint16, game);
        (*E).crocom_var_A = 88 as uint16;
        (*E).base.current_instruction = addr_kCrocomire_Ilist_E1D2 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(4) as *mut uint8
            as *mut uint16) = 257 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x5f7) as *mut uint16) = 0 as uint16;
        let fresh12 = &raw mut (*Get_Crocomire(0x40 as ::core::ffi::c_int, game))
            .base
            .properties;
        fresh12.write_unaligned(((fresh12.read_unaligned()) | (0x200u16)));
        SpawnHardcodedPlm(
            SpawnHardcodedPlmArgs {
                field_0: 0x4e as uint8,
                field_1: 0x3 as uint8,
                field_2: 0xb753 as VoidP,
            },
            game,
        );
        *((&raw mut game.g_ram as *mut uint8).offset(0x941) as *mut uint16) = 0 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x688) as *mut uint16) = 0 as uint16;
    }
}
pub unsafe fn Crocomire_Func_50(game: &mut Game) {
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int, game);
    let mut y_pos: uint16 = (*E).base.y_pos;
    let mut v2: uint16 = addr_kCrocomire_Ilist_BF7E as uint16;
    if y_pos as ::core::ffi::c_int - 280 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        v2 = addr_kCrocomire_Ilist_BF86 as uint16;
        if y_pos as ::core::ffi::c_int - 264 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            != 0
        {
            v2 = addr_kCrocomire_Ilist_BF8C as uint16;
            if y_pos as ::core::ffi::c_int - 248 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
            {
                v2 = addr_kCrocomire_Ilist_BF92 as uint16;
            }
        }
    }
    (*E).base.current_instruction = v2;
    (*E).base.instruction_timer = 1 as uint16;
}
pub unsafe fn Crocomire_Func_51(game: &mut Game) {
    Crocomire_Func_53(game);
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int, game);
    let mut y_pos: uint16 = (*E).base.y_pos;
    let mut v2: uint16 = addr_kCrocomire_Ilist_BF64 as uint16;
    if y_pos as ::core::ffi::c_int - 280 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        v2 = addr_kCrocomire_Ilist_BF6C as uint16;
        if y_pos as ::core::ffi::c_int - 264 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            != 0
        {
            v2 = addr_kCrocomire_Ilist_BF72 as uint16;
            if y_pos as ::core::ffi::c_int - 248 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
            {
                v2 = addr_kCrocomire_Ilist_BF78 as uint16;
            }
        }
    }
    (*E).base.current_instruction = v2;
    (*E).base.instruction_timer = 1 as uint16;
    Crocomire_Func_55(game);
}
pub unsafe fn Crocomire_Func_52(game: &mut Game) {
    let mut v0: uint16 = *(((&raw mut game.g_ram as *mut uint8).offset(0x9015) as *mut uint16
        as *mut uint8)
        .offset(1) as *mut uint16);
    if *(((&raw mut game.g_ram as *mut uint8).offset(0x9015) as *mut uint16 as *mut uint8).offset(1)
        as *mut uint16) as ::core::ffi::c_int
        - 22 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v1: uint16 = *(((&raw mut game.g_ram as *mut uint8).offset(0x9015) as *mut uint16
            as *mut uint8)
            .offset(1) as *mut uint16);
        let ref mut fresh17 = *(((&raw mut game.g_ram as *mut uint8).offset(0x9015) as *mut uint16
            as *mut uint8)
            .offset(1) as *mut uint16);
        *fresh17 = ((*fresh17).wrapping_add((2u16)));
        SpawnEprojWithGfx(
            *(game.g_rom.offset(crate::rtl::lorom_offset(0xa49156)) as *const uint8 as *mut uint16)
                .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize),
            v1,
            addr_stru_868F9D as uint16,
            game,
        );
    }
    Crocomire_Func_54(game);
}
pub unsafe fn Crocomire_Func_53(game: &mut Game) {
    let ref mut fresh9 = *((&raw mut game.g_ram as *mut uint8).offset(0x9018) as *mut uint16);
    *fresh9 = (*fresh9).wrapping_sub(1);
    if *fresh9 == 0 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x9018) as *mut uint16) = 6 as uint16;
        let mut v0: uint16 =
            ((*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)) & (0x3fu16));
        if (*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
            as ::core::ffi::c_int
            & 2 as ::core::ffi::c_int)
            == 0i32
        {
            v0 = !(v0 as ::core::ffi::c_int) as uint16;
        }
        let mut x: uint16 =
            (((*Get_Crocomire(0 as ::core::ffi::c_int, game)).base.x_pos).wrapping_add((v0)));
        let mut y: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0x1962) as *mut uint16)
            as ::core::ffi::c_int
            + 16 as ::core::ffi::c_int
            - (((*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)) & (0x1f00u16))
                as ::core::ffi::c_int
                >> 8 as ::core::ffi::c_int)) as uint16;
        CreateSpriteAtPos(x, y, 21 as uint16, 0 as uint16, game);
    }
}
pub unsafe fn Crocomire_Func_54(game: &mut Game) {
    Crocomire_Func_35(game);
    Crocomire_Func_53(game);
    Crocomire_Func_55(game);
}
pub unsafe fn Crocomire_Func_55(game: &mut Game) {
    let mut crocom_var_D: int8 = 0;
    let mut v7: int8 = 0;
    let mut v9: int8 = 0;
    let mut crocom_var_E: int8 = 0;
    let mut crocom_var_E_high: int8 = 0;
    let mut v12: int8 = 0;
    Crocomire_Func_58(game);
    let mut E0: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int, game);
    (*E0).crocom_var_B =
        ((*E0).crocom_var_B as ::core::ffi::c_int & !(0x800 as ::core::ffi::c_int)) as uint16;
    Crocomire_Func_27(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
        game,
    );
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*E).base.y_pos as ::core::ffi::c_int - 280 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut v3: uint16 = (8 as ::core::ffi::c_int
            * (*((&raw mut game.g_ram as *mut uint8).offset(0xb7) as *mut uint16)
                as ::core::ffi::c_int
                + 288 as ::core::ffi::c_int
                & 0xfff8 as ::core::ffi::c_int)) as uint16;
        let mut m: ::core::ffi::c_int = 32 as ::core::ffi::c_int;
        loop {
            *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
                .offset((v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
                824 as uint16;
            v3 = ((v3).wrapping_add((2u16)));
            m -= 1;
            if !(m != 0) {
                break;
            }
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0xe1e) as *mut uint16) = 1 as uint16;
        (*E0).base.extra_properties = ((*E0).base.extra_properties as ::core::ffi::c_int
            & !(0x8000 as ::core::ffi::c_int)) as uint16;
        crocom_var_D = (*E).crocom_var_D as int8;
        *(&raw mut (*E).crocom_var_D as *mut uint8).offset(0) =
            (((crocom_var_D) as u8).wrapping_add((0x80u8)));
        v7 = (u8::overflowing_add(crocom_var_D as uint8, 0x80 as uint8).1 as ::core::ffi::c_int
            + *(&raw mut (*E).crocom_var_D as *mut uint8).offset(
                (::core::mem::size_of::<uint16>())
                    .wrapping_div(::core::mem::size_of::<uint8>())
                    .wrapping_sub(1 as usize) as isize,
            ) as ::core::ffi::c_int
            + 3 as ::core::ffi::c_int) as int8;
        if v7 as ::core::ffi::c_int - 48 as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int == 0 {
            v7 = 48 as int8;
        }
        *(&raw mut (*E).crocom_var_D as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1 as usize) as isize,
        ) = v7 as uint8;
        let mut v8: bool = u8::overflowing_add(
            *(&raw mut (*E).crocom_var_E as *mut uint8).offset(0),
            v7 as uint8,
        )
        .1;
        let ref mut fresh13 = *(&raw mut (*E).crocom_var_E as *mut uint8).offset(0);
        *fresh13 = ((*fresh13).wrapping_add(((v7) as u8)));
        v9 = (((v8) as i8).wrapping_add(
            ((*(&raw mut (*E).crocom_var_E as *mut uint8).offset(
                (::core::mem::size_of::<uint16>())
                    .wrapping_div(::core::mem::size_of::<uint8>())
                    .wrapping_sub(1 as usize) as isize,
            )) as i8),
        ));
        if v9 as ::core::ffi::c_int - 3 as ::core::ffi::c_int & 0x80 as ::core::ffi::c_int == 0 {
            v9 = 3 as int8;
        }
        *(&raw mut (*E).crocom_var_E as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1 as usize) as isize,
        ) = v9 as uint8;
        crocom_var_E = (*E).crocom_var_E as int8;
        v8 = u8::overflowing_add(
            *(&raw mut (*E).crocom_var_F as *mut uint8).offset(
                (::core::mem::size_of::<uint16>())
                    .wrapping_div(::core::mem::size_of::<uint8>())
                    .wrapping_sub(1 as usize) as isize,
            ),
            crocom_var_E as uint8,
        )
        .1;
        let ref mut fresh14 = *(&raw mut (*E).crocom_var_F as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1 as usize) as isize,
        );
        *fresh14 = ((*fresh14).wrapping_add(((crocom_var_E) as u8)));
        crocom_var_E_high = *(&raw mut (*E).crocom_var_E as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1 as usize) as isize,
        ) as int8;
        let mut v13: bool = v8;
        v8 = u8::overflowing_add(v8 as uint8, crocom_var_E_high as uint8).1;
        v12 = (((v13) as i8).wrapping_add((crocom_var_E_high)));
        v8 = (v8 as ::core::ffi::c_int
            | u8::overflowing_add(
                *(&raw mut (*E).base.y_pos as *mut uint8).offset(0),
                v12 as uint8,
            )
            .1 as ::core::ffi::c_int)
            != 0;
        let ref mut fresh15 = *(&raw mut (*E).base.y_pos as *mut uint8).offset(0);
        *fresh15 = ((*fresh15).wrapping_add(((v12) as u8)));
        let ref mut fresh16 = *(&raw mut (*E).base.y_pos as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1 as usize) as isize,
        );
        *fresh16 = ((*fresh16).wrapping_add(((v8) as u8)));
    } else {
        (*E).crocom_var_A = (*E).crocom_var_A.wrapping_add(1);
        (*E).crocom_var_A = (*E).crocom_var_A.wrapping_add(1);
        (*E0).crocom_var_D = 48 as uint16;
    };
}
pub unsafe fn Crocomire_Func_56(game: &mut Game) {
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut y_pos: uint16 = (*E).base.y_pos;
    let mut v2: uint16 = addr_kCrocomire_Ilist_BF7E as uint16;
    if y_pos as ::core::ffi::c_int - 280 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        v2 = addr_kCrocomire_Ilist_BF86 as uint16;
        if y_pos as ::core::ffi::c_int - 264 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            != 0
        {
            v2 = addr_kCrocomire_Ilist_BF8C as uint16;
            if y_pos as ::core::ffi::c_int - 248 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
            {
                v2 = addr_kCrocomire_Ilist_BF92 as uint16;
            }
        }
    }
    (*E).base.current_instruction = v2;
    (*E).base.instruction_timer = 1 as uint16;
    Crocomire_Func_53(game);
    Crocomire_92D8(game);
}
pub unsafe fn Crocomire_929E(game: &mut Game) {
    Crocomire_Func_53(game);
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    let mut y_pos: uint16 = (*E).base.y_pos;
    let mut v2: uint16 = addr_kCrocomire_Ilist_BF64 as uint16;
    if y_pos as ::core::ffi::c_int - 280 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        v2 = addr_kCrocomire_Ilist_BF6C as uint16;
        if y_pos as ::core::ffi::c_int - 264 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
            != 0
        {
            v2 = addr_kCrocomire_Ilist_BF72 as uint16;
            if y_pos as ::core::ffi::c_int - 248 as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int
                != 0
            {
                v2 = addr_kCrocomire_Ilist_BF78 as uint16;
            }
        }
    }
    (*E).base.current_instruction = v2;
    (*E).base.instruction_timer = 1 as uint16;
    Crocomire_92D8(game);
}
pub unsafe fn Crocomire_92CE(game: &mut Game) {
    Crocomire_Func_35(game);
    Crocomire_Func_53(game);
    Crocomire_92D8(game);
}
pub unsafe fn Crocomire_92D8(game: &mut Game) {
    let mut v3: int16 = 0;
    let mut v6: int8 = 0;
    Crocomire_Func_58(game);
    let mut EK: *mut Enemy_Crocomire = Get_Crocomire(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*EK).base.y_pos as ::core::ffi::c_int - 218 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        (*EK).crocom_var_A = (*EK).crocom_var_A.wrapping_add(1);
        (*EK).crocom_var_A = (*EK).crocom_var_A.wrapping_add(1);
    } else {
        Crocomire_Func_27(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        );
        let mut E0: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int, game);
        v3 = ((((*E0).crocom_var_D) as i16).wrapping_add((256i16)));
        if (*E0).crocom_var_D as ::core::ffi::c_int - 7680 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            == 0
        {
            v3 = 7936 as int16;
        }
        (*E0).crocom_var_D = v3 as uint16;
        let mut crocom_var_E: uint8 = (*E0).crocom_var_E as uint8;
        let mut v5: bool = crocom_var_E
            < *(&raw mut (*E0).crocom_var_D as *mut uint8).offset(
                (::core::mem::size_of::<uint16>())
                    .wrapping_div(::core::mem::size_of::<uint8>())
                    .wrapping_sub(1 as usize) as isize,
            );
        *(&raw mut (*E0).crocom_var_E as *mut uint8).offset(0) = ((crocom_var_E).wrapping_sub(
            (*(&raw mut (*E0).crocom_var_D as *mut uint8).offset(
                (::core::mem::size_of::<uint16>())
                    .wrapping_div(::core::mem::size_of::<uint8>())
                    .wrapping_sub(1 as usize) as isize,
            )),
        ));
        v6 = (((*(&raw mut (*E0).crocom_var_E as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1 as usize) as isize,
        )) as i8)
            .wrapping_sub(((v5) as i8)));
        if v6 < 0i8 {
            *(&raw mut (*E0).crocom_var_E as *mut uint8).offset(0) =
                -(1 as ::core::ffi::c_int) as uint8;
            v6 = 0 as int8;
        }
        *(&raw mut (*E0).crocom_var_E as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1 as usize) as isize,
        ) = v6 as uint8;
        let mut crocom_var_F_high: uint8 = *(&raw mut (*E0).crocom_var_F as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1 as usize) as isize,
        );
        v5 = crocom_var_F_high < *(&raw mut (*E0).crocom_var_E as *mut uint8).offset(0);
        *(&raw mut (*E0).crocom_var_F as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1 as usize) as isize,
        ) = ((crocom_var_F_high)
            .wrapping_sub((*(&raw mut (*E0).crocom_var_E as *mut uint8).offset(0))));
        let mut y_pos: uint8 = (*E0).base.y_pos as uint8;
        let mut v9: uint8 = (((v5) as u8).wrapping_add(
            (*(&raw mut (*E0).crocom_var_E as *mut uint8).offset(
                (::core::mem::size_of::<uint16>())
                    .wrapping_div(::core::mem::size_of::<uint8>())
                    .wrapping_sub(1 as usize) as isize,
            )),
        ));
        *(&raw mut (*E0).base.y_pos as *mut uint8).offset(0) = ((y_pos).wrapping_sub((v9)));
        let ref mut fresh7 = *(&raw mut (*E0).base.y_pos as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1 as usize) as isize,
        );
        *fresh7 = ((*fresh7).wrapping_sub(((y_pos < v9) as u8)));
    };
}
pub unsafe fn Crocomire_Func_57(game: &mut Game) {
    let mut i: ::core::ffi::c_int = 0;
    *(&raw mut game.g_ram as *mut uint8).offset(0x68c) = 48 as uint8;
    *((&raw mut game.g_ram as *mut uint8).offset(0x688) as *mut uint16) = 48 as uint16;
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).crocom_var_A = (*E).crocom_var_A.wrapping_add(1);
    (*E).crocom_var_A = (*E).crocom_var_A.wrapping_add(1);
    (*E).base.current_instruction = addr_kCrocomire_Ilist_BF64 as uint16;
    (*E).base.instruction_timer = 1 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xe1e) as *mut uint16) = 1 as uint16;
    let mut E1: *mut Enemy_Crocomire = Get_Crocomire(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int
            + 64 as ::core::ffi::c_int,
        game,
    );
    (*E1).base.current_instruction = addr_kCrocomire_Ilist_BF98 as uint16;
    (*E1).base.instruction_timer = 1 as uint16;
    (*E1).base.properties = ((*E1).base.properties as ::core::ffi::c_int
        & 0xd2ff as ::core::ffi::c_int
        | 0x2c00 as ::core::ffi::c_int) as uint16;
    (*E1).base.x_pos = (*E).base.x_pos;
    (*E1).base.y_pos = (((*E).base.y_pos).wrapping_add((16u16)));
    let mut v3: uint16 = 0 as uint16;
    loop {
        let mut v4: ::core::ffi::c_int = v3 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16).offset(v4 as isize) =
            824 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v4 + 1 as ::core::ffi::c_int) as isize) = 824 as uint16;
        v3 = ((v3).wrapping_add((4u16)));
        if !(((v3) as i16).wrapping_sub((1024i16)) < 0i16) {
            break;
        }
    }
    i = 0 as ::core::ffi::c_int;
    loop {
        let mut v6: ::core::ffi::c_int = i >> 1 as ::core::ffi::c_int;
        if *(game.g_rom.offset(crate::rtl::lorom_offset(0xa49c79)) as *const uint8 as *mut uint16)
            .offset(v6 as isize)
            == 0xffffu16
        {
            break;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v6 + 32 as ::core::ffi::c_int) as isize) =
            *(game.g_rom.offset(crate::rtl::lorom_offset(0xa49c79)) as *const uint8 as *mut uint16)
                .offset(v6 as isize);
        i += 2 as ::core::ffi::c_int;
    }
    Crocomire_93BE((i + 1024 as ::core::ffi::c_int) as uint16, game);
}
pub unsafe fn Crocomire_93BE(mut k: uint16, game: &mut Game) {
    let mut v3: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    let mut v2: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16);
    v3 = gVramWriteEntry(
        *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*v3).size = k;
    (*v3).src.addr = 0x2000 as VoidP;
    *(&raw mut (*v3).src.bank as *mut uint16) = 126 as uint16;
    (*v3).vram_dst = ((*(&raw mut game.g_ram as *mut uint8).offset(0x59) as ::core::ffi::c_int
        & 0xfc as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) =
        ((v2).wrapping_add((7u16)));
}
pub unsafe fn Crocomire_Func_58(game: &mut Game) {
    let mut v0: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0xb7) as *mut uint16);
    let mut i: ::core::ffi::c_int = 510 as ::core::ffi::c_int;
    while i >= 0i32 {
        *((&raw mut game.g_ram as *mut uint8).offset(0xcaf0) as *mut uint16)
            .offset((i >> 1 as ::core::ffi::c_int) as isize) = v0;
        i -= 2 as ::core::ffi::c_int;
    }
}
pub unsafe fn Crocomire_Func_59(game: &mut Game) {
    let mut i: ::core::ffi::c_int = 0;
    Crocomire_Func_58(game);
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).crocom_var_A = (*E).crocom_var_A.wrapping_add(1);
    (*E).crocom_var_A = (*E).crocom_var_A.wrapping_add(1);
    (*E).base.instruction_timer = 1 as uint16;
    *(&raw mut game.g_ram as *mut uint8).offset(0x68c) = 48 as uint8;
    *((&raw mut game.g_ram as *mut uint8).offset(0x688) as *mut uint16) = 48 as uint16;
    (*E).base.current_instruction = addr_kCrocomire_Ilist_BF7E as uint16;
    let mut v1: uint16 = 0 as uint16;
    loop {
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 824 as uint16;
        v1 = ((v1).wrapping_add((2u16)));
        if !(((v1) as i16).wrapping_sub((2048i16)) < 0i16) {
            break;
        }
    }
    i = 0 as ::core::ffi::c_int;
    loop {
        let mut v3: ::core::ffi::c_int = i >> 1 as ::core::ffi::c_int;
        if *(game.g_rom.offset(crate::rtl::lorom_offset(0xa49e7b)) as *const uint8 as *mut uint16)
            .offset(v3 as isize)
            == 0xffffu16
        {
            break;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((v3 + 32 as ::core::ffi::c_int) as isize) =
            *(game.g_rom.offset(crate::rtl::lorom_offset(0xa49e7b)) as *const uint8 as *mut uint16)
                .offset(v3 as isize);
        i += 2 as ::core::ffi::c_int;
    }
    Crocomire_93BE((i + 1024 as ::core::ffi::c_int) as uint16, game);
}
pub unsafe fn Crocomire_Func_60(game: &mut Game) {
    let mut i: ::core::ffi::c_int = 0;
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).crocom_var_A = (((*E).crocom_var_A).wrapping_add((2u16)));
    *((&raw mut game.g_ram as *mut uint8).offset(0x692) as *mut uint16) = 256 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x690) as *mut uint16) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x698) as *mut uint16) =
        *((game.g_rom.offset(crate::rtl::lorom_offset(0xa49bc5)) as *const uint8 as *mut uint16
            as *mut uint8)
            .offset(
                *((&raw mut game.g_ram as *mut uint8).offset(0x69a) as *mut uint16)
                    as ::core::ffi::c_int as isize,
            ) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x694) as *mut uint16) =
        *((&raw mut game.g_ram as *mut uint8).offset(0x698) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x696) as *mut uint16) =
        *((game.g_rom.offset(crate::rtl::lorom_offset(0xa49bc7)) as *const uint8 as *mut uint16
            as *mut uint8)
            .offset(
                *((&raw mut game.g_ram as *mut uint8).offset(0x69a) as *mut uint16)
                    as ::core::ffi::c_int as isize,
            ) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x68e) as *mut uint16) =
        *((game.g_rom.offset(crate::rtl::lorom_offset(0xa49bc9)) as *const uint8 as *mut uint16
            as *mut uint8)
            .offset(
                *((&raw mut game.g_ram as *mut uint8).offset(0x69a) as *mut uint16)
                    as ::core::ffi::c_int as isize,
            ) as *mut uint16);
    let mut bank: uint8 = *((game.g_rom.offset(crate::rtl::lorom_offset(0xa49bcb)) as *const uint8
        as *mut uint16 as *mut uint8)
        .offset(
            *((&raw mut game.g_ram as *mut uint8).offset(0x69a) as *mut uint16)
                as ::core::ffi::c_int as isize,
        ) as *mut uint16) as uint8;
    let mut v6: uint16 = 0;
    i = *((&raw mut game.g_ram as *mut uint8).offset(0x69a) as *mut uint16) as ::core::ffi::c_int
        + 8 as ::core::ffi::c_int;
    loop {
        let mut v2: uint16 = *((game.g_rom.offset(crate::rtl::lorom_offset(0xa49bc5))
            as *const uint8 as *mut uint16 as *mut uint8)
            .offset(i as isize) as *mut uint16);
        if v2 == 0xffffu16 {
            break;
        }
        v6 = i as uint16;
        let mut v3: uint16 = *((game.g_rom.offset(crate::rtl::lorom_offset(0xa49bc7))
            as *const uint8 as *mut uint16 as *mut uint8)
            .offset(i as isize) as *mut uint16);
        let mut n: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x68e)
            as *mut uint16) as ::core::ffi::c_int;
        loop {
            *((&raw mut game.g_ram as *mut uint8).offset(v3 as isize) as *mut uint8
                as *mut uint16) = *(RomPtrWithBank(bank, v2, game) as *mut uint16);
            v3 = ((v3).wrapping_add((2u16)));
            v2 = ((v2).wrapping_add((2u16)));
            n -= 1;
            if !(n >= 0i32) {
                break;
            }
        }
        i = v6 as ::core::ffi::c_int + 4 as ::core::ffi::c_int;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x69a) as *mut uint16) =
        (i + 2 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x68a) as *mut uint16) =
        (i + 2 as ::core::ffi::c_int) as uint16;
    let mut j: ::core::ffi::c_int = 128 as ::core::ffi::c_int;
    while j >= 0i32 {
        *((&raw mut game.g_ram as *mut uint8)
            .offset(0x69c)
            .offset(j as uint16 as isize) as *mut uint8 as *mut uint16) = 0 as uint16;
        j -= 2 as ::core::ffi::c_int;
    }
}
pub unsafe fn Crocomire_Func_61(game: &mut Game) {
    Crocomire_Func_62(game);
}
pub unsafe fn Crocomire_Func_62(game: &mut Game) {
    let mut v2: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    let mut v0: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x68a) as *mut uint16);
    let mut v1: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16);
    if *((game.g_rom.offset(crate::rtl::lorom_offset(0xa49bc5)) as *const uint8 as *mut uint16
        as *mut uint8)
        .offset(
            *((&raw mut game.g_ram as *mut uint8).offset(0x68a) as *mut uint16)
                as ::core::ffi::c_int as isize,
        ) as *mut uint16)
        == 0xffffu16
    {
        let mut E: *mut Enemy_Crocomire = Get_Crocomire(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        );
        (*E).crocom_var_A = (*E).crocom_var_A.wrapping_add(1);
        (*E).crocom_var_A = (*E).crocom_var_A.wrapping_add(1);
        *((&raw mut game.g_ram as *mut uint8).offset(0x68a) as *mut uint16) = 0 as uint16;
    } else {
        v2 = gVramWriteEntry(
            *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16)
                as ::core::ffi::c_int,
            game,
        );
        (*v2).size = *((game.g_rom.offset(crate::rtl::lorom_offset(0xa49bc5)) as *const uint8
            as *mut uint16 as *mut uint8)
            .offset(
                *((&raw mut game.g_ram as *mut uint8).offset(0x68a) as *mut uint16)
                    as ::core::ffi::c_int as isize,
            ) as *mut uint16);
        (*v2).src.addr = *((game.g_rom.offset(crate::rtl::lorom_offset(0xa49bcb)) as *const uint8
            as *mut uint16 as *mut uint8)
            .offset(v0 as ::core::ffi::c_int as isize) as *mut uint16);
        *(&raw mut (*v2).src.bank as *mut uint16) =
            *((game.g_rom.offset(crate::rtl::lorom_offset(0xa49bc9)) as *const uint8 as *mut uint16
                as *mut uint8)
                .offset(v0 as ::core::ffi::c_int as isize) as *mut uint16);
        (*v2).vram_dst = *((game.g_rom.offset(crate::rtl::lorom_offset(0xa49bc7)) as *const uint8
            as *mut uint16 as *mut uint8)
            .offset(v0 as ::core::ffi::c_int as isize) as *mut uint16);
        *((&raw mut game.g_ram as *mut uint8).offset(0x68a) as *mut uint16) =
            ((v0).wrapping_add((8u16)));
        *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) =
            ((v1).wrapping_add((7u16)));
    };
}
pub unsafe fn Crocomire_Func_64(game: &mut Game) {
    QueueSfx2_Max6(0x77 as uint16, game);
    Crocomire_950F(game);
}
pub unsafe fn Crocomire_9506(game: &mut Game) {
    QueueSfx2_Max6(0x2d as uint16, game);
    Crocomire_950F(game);
}
pub unsafe fn Crocomire_950F(game: &mut Game) {
    static unk_A49559: SpawnHdmaObject_Args = SpawnHdmaObject_Args {
        field_0: 0x42 as uint8,
        field_1: 0x10 as uint8,
        addr: 0x9563 as VoidP,
    };
    let mut v4: *mut Enemy_Crocomire = ::core::ptr::null_mut::<Enemy_Crocomire>();
    let mut v1: uint16 = (2 as ::core::ffi::c_int
        * ((*Get_Crocomire(0x40 as ::core::ffi::c_int, game))
            .base
            .y_pos as ::core::ffi::c_int
            - 72 as ::core::ffi::c_int)) as uint16;
    let mut v2: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0xb7) as *mut uint16);
    loop {
        *((&raw mut game.g_ram as *mut uint8).offset(0xcaf0) as *mut uint16)
            .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = v2;
        v1 = ((v1).wrapping_sub((2u16)));
        if !((v1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) == 0i32) {
            break;
        }
    }
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).crocom_var_A = (*E).crocom_var_A.wrapping_add(1);
    (*E).crocom_var_A = (*E).crocom_var_A.wrapping_add(1);
    v4 = Get_Crocomire(0 as ::core::ffi::c_int, game);
    (*Get_Crocomire(0x80 as ::core::ffi::c_int, game)).crocom_var_D = (*v4).base.x_pos;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcae0) as *mut uint16) = 255 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcae0).offset(1) as *mut uint8 as *mut uint16) =
        -(13584 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcae0).offset(3) as *mut uint8 as *mut uint16) =
        225 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcae0).offset(4) as *mut uint8 as *mut uint16) =
        -(13330 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcae0).offset(6) as *mut uint8 as *mut uint16) =
        0 as uint16;
    (*v4).crocom_var_1F = SpawnHdmaObject(
        0xa4 as uint8,
        &raw const unk_A49559 as *const ::core::ffi::c_void,
        game,
    );
}
pub unsafe fn Crocomire_Func_63(game: &mut Game) {
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).crocom_var_A = (((*E).crocom_var_A).wrapping_add((2u16)));
}
pub unsafe fn Crocomire_Func_65(game: &mut Game) {
    let mut v4: uint16 = 0;
    let mut current_block: u64;
    let mut i: ::core::ffi::c_int = 0;
    Crocomire_Func_53(game);
    let mut E0: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int, game);
    let mut E1: *mut Enemy_Crocomire = Get_Crocomire(0x40 as ::core::ffi::c_int, game);
    let mut E2: *mut Enemy_Crocomire = Get_Crocomire(0x80 as ::core::ffi::c_int, game);
    let mut crocom_var_D: uint16 = (*E2).crocom_var_D;
    (*E1).crocom_var_D = (*E1).crocom_var_D.wrapping_sub(1);
    if ((*E1).crocom_var_D as ::core::ffi::c_int & 2 as ::core::ffi::c_int) != 0i32 {
        crocom_var_D = ((crocom_var_D).wrapping_add((4u16)));
    }
    (*E0).base.x_pos = crocom_var_D;
    if !(Crocomire_Func_67(game) == 0) {
        Crocomire_Func_27(
            *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
            game,
        );
        v4 = 0;
        v4 = ((*((&raw mut game.g_ram as *mut uint8).offset(0x694) as *mut uint16))
            .wrapping_sub((3u16)));
        if *((&raw mut game.g_ram as *mut uint8).offset(0x694) as *mut uint16) as ::core::ffi::c_int
            - 19 as ::core::ffi::c_int
            & 0x8000 as ::core::ffi::c_int
            != 0
        {
            if ((*((&raw mut game.g_ram as *mut uint8).offset(0x692) as *mut uint16)) as i16)
                .wrapping_sub((20480i16))
                >= 0i16
            {
                current_block = 11812396948646013369;
            } else {
                v4 = 16 as uint16;
                current_block = 15652330335145281839;
            }
        } else {
            current_block = 15652330335145281839;
        }
        match current_block {
            11812396948646013369 => {}
            _ => {
                *((&raw mut game.g_ram as *mut uint8).offset(0x694) as *mut uint16) = v4;
                let mut v5: uint16 = ((*((&raw mut game.g_ram as *mut uint8).offset(0x692)
                    as *mut uint16))
                    .wrapping_add((384u16)));
                if *((&raw mut game.g_ram as *mut uint8).offset(0x692) as *mut uint16)
                    as ::core::ffi::c_int
                    - 20096 as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    == 0
                {
                    v5 = 20480 as uint16;
                }
                *((&raw mut game.g_ram as *mut uint8).offset(0x692) as *mut uint16) = v5;
                let mut r18: uint16 = 0 as uint16;
                let mut v6: uint16 = (2 as ::core::ffi::c_int
                    * ((*E1).base.y_pos as ::core::ffi::c_int - 72 as ::core::ffi::c_int))
                    as uint16;
                let mut v7: uint16 =
                    *((&raw mut game.g_ram as *mut uint8).offset(0x694) as *mut uint16);
                *((&raw mut game.g_ram as *mut uint8).offset(0x5a6) as *mut uint16) =
                    *((&raw mut game.g_ram as *mut uint8).offset(0x694) as *mut uint16);
                loop {
                    *((&raw mut game.g_ram as *mut uint8).offset(0xcaf0) as *mut uint16)
                        .offset((v6 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
                        (*((&raw mut game.g_ram as *mut uint8).offset(0xb7) as *mut uint16)
                            as ::core::ffi::c_int
                            + v7 as ::core::ffi::c_int
                            - *((&raw mut game.g_ram as *mut uint8).offset(0x5a6) as *mut uint16)
                                as ::core::ffi::c_int) as uint16;
                    let mut v8: bool = u16::overflowing_add(
                        *((&raw mut game.g_ram as *mut uint8).offset(0x692) as *mut uint16),
                        r18,
                    )
                    .1;
                    r18 = ((r18).wrapping_add(
                        (*((&raw mut game.g_ram as *mut uint8).offset(0x692) as *mut uint16)),
                    ));
                    if !v8 {
                        v7 = v7.wrapping_add(1);
                    }
                    let ref mut fresh18 =
                        *((&raw mut game.g_ram as *mut uint8).offset(0x5a6) as *mut uint16);
                    *fresh18 = (*fresh18).wrapping_add(1);
                    v6 = ((v6).wrapping_add((2u16)));
                    if !(((v7) as i16).wrapping_sub(
                        ((*((&raw mut game.g_ram as *mut uint8).offset(0x698) as *mut uint16))
                            as i16),
                    ) < 0i16)
                    {
                        break;
                    }
                }
                if ((v6) as i16).wrapping_sub((512i16)) < 0i16 {
                    let mut v9: uint16 =
                        *((&raw mut game.g_ram as *mut uint8).offset(0xb7) as *mut uint16);
                    loop {
                        *((&raw mut game.g_ram as *mut uint8).offset(0xcaf0) as *mut uint16)
                            .offset(
                                (v6 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize,
                            ) = v9;
                        v6 = ((v6).wrapping_add((2u16)));
                        if !(((v6) as i16).wrapping_sub((512i16)) < 0i16) {
                            break;
                        }
                    }
                }
                return;
            }
        }
    }
    let mut EK: *mut Enemy_Crocomire = Get_Crocomire(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*EK).crocom_var_A = (((*EK).crocom_var_A).wrapping_add((2u16)));
    i = *((&raw mut game.g_ram as *mut uint8).offset(0x69a) as *mut uint16) as ::core::ffi::c_int;
    while *((game.g_rom.offset(crate::rtl::lorom_offset(0xa49bc5)) as *const uint8 as *mut uint16
        as *mut uint8)
        .offset(i as isize) as *mut uint16)
        != 0xffffu16
    {
        i += 8 as ::core::ffi::c_int;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x69a) as *mut uint16) =
        (i + 2 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x18b4) as *mut uint16)
        .offset(((*E0).crocom_var_1F as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        0 as uint16;
}
pub unsafe fn Crocomire_Func_66(game: &mut Game) {
    let mut v3: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*E).crocom_var_E = 0 as uint16;
    (*E).crocom_var_F = 0 as uint16;
    (*E).crocom_var_D = 2048 as uint16;
    let mut i: ::core::ffi::c_int = 4094 as ::core::ffi::c_int;
    while i >= 0i32 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x2000) as *mut uint16)
            .offset((i >> 1 as ::core::ffi::c_int) as isize) = 824 as uint16;
        i -= 2 as ::core::ffi::c_int;
    }
    let mut v2: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16);
    v3 = gVramWriteEntry(
        *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*v3).size = 2048 as uint16;
    (*v3).src.addr = 0x2000 as VoidP;
    *(&raw mut (*v3).src.bank as *mut uint16) = 126 as uint16;
    (*v3).vram_dst = ((*(&raw mut game.g_ram as *mut uint8).offset(0x59) as ::core::ffi::c_int
        & 0xfe as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) =
        ((v2).wrapping_add((7u16)));
    Crocomire_9BB3(game);
}
static kCrocoEraseLineMasks: [uint8; 8] = [0x7f, 0xbf, 0xdf, 0xef, 0xf7, 0xfb, 0xfd, 0xfe];
pub unsafe fn Crocomire_Func_67(game: &mut Game) -> uint16 {
    let mut n: ::core::ffi::c_int =
        *(&raw mut game.g_ram as *mut uint8).offset(0x68c) as ::core::ffi::c_int;
    loop {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x690) as *mut uint16) > 48u16 {
            return 1 as uint16;
        }
        let mut rr: ::core::ffi::c_int = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa49697))
            as *const uint8 as *mut uint8)
            .offset(*((&raw mut game.g_ram as *mut uint8).offset(0x690) as *mut uint16) as isize)
            as ::core::ffi::c_int;
        if ((*(&raw mut game.g_ram as *mut uint8)
            .offset(0x69c)
            .offset(rr as isize)) as i8)
            .wrapping_sub(
                ((*((&raw mut game.g_ram as *mut uint8).offset(0x688) as *mut uint16)) as i8),
            )
            < 0i8
        {
            break;
        }
        let ref mut fresh0 = *((&raw mut game.g_ram as *mut uint8).offset(0x690) as *mut uint16);
        *fresh0 = (*fresh0).wrapping_add(1);
        if *fresh0 >= 128u16 {
            *((&raw mut game.g_ram as *mut uint8).offset(0x690) as *mut uint16) = 0 as uint16;
            return 0 as uint16;
        }
    }
    '_c2rust_label: {
        if *((&raw mut game.g_ram as *mut uint8).offset(0x690) as *mut uint16) <= 48u16 {
        } else {
            __assert_fail(
                b"croco_cur_vline_idx <= 48\0" as *const u8 as *const ::core::ffi::c_char,
                b"src/sm_a4.c\0" as *const u8 as *const ::core::ffi::c_char,
                1215 as ::core::ffi::c_uint,
                b"uint16 Crocomire_Func_67(void)\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
    };
    let mut vline_idx: ::core::ffi::c_int = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa49697))
        as *const uint8 as *mut uint8)
        .offset(*((&raw mut game.g_ram as *mut uint8).offset(0x690) as *mut uint16) as isize)
        as ::core::ffi::c_int;
    let mut mask: uint8 = kCrocoEraseLineMasks[(vline_idx & 7 as ::core::ffi::c_int) as usize];
    loop {
        let mut q: ::core::ffi::c_int = *(&raw mut game.g_ram as *mut uint8)
            .offset(0x69c)
            .offset(vline_idx as isize)
            as ::core::ffi::c_int;
        let mut j: ::core::ffi::c_int = 2 as ::core::ffi::c_int * (q & 7 as ::core::ffi::c_int)
            + ((q & !(7 as ::core::ffi::c_int)) << 6 as ::core::ffi::c_int)
            + 4 as ::core::ffi::c_int * (vline_idx & !(7 as ::core::ffi::c_int));
        let ref mut fresh1 = (*((&raw mut game.g_ram as *mut uint8).offset(0x4000)
            as *mut Ram4000))
            .backups
            .field_0[(j + 0 as ::core::ffi::c_int) as usize];
        *fresh1 = ((*fresh1) & (mask));
        let ref mut fresh2 = (*((&raw mut game.g_ram as *mut uint8).offset(0x4000)
            as *mut Ram4000))
            .backups
            .field_0[(j + 1 as ::core::ffi::c_int) as usize];
        *fresh2 = ((*fresh2) & (mask));
        let ref mut fresh3 = (*((&raw mut game.g_ram as *mut uint8).offset(0x4000)
            as *mut Ram4000))
            .backups
            .field_0[(j + 16 as ::core::ffi::c_int) as usize];
        *fresh3 = ((*fresh3) & (mask));
        let ref mut fresh4 = (*((&raw mut game.g_ram as *mut uint8).offset(0x4000)
            as *mut Ram4000))
            .backups
            .field_0[(j + 17 as ::core::ffi::c_int) as usize];
        *fresh4 = ((*fresh4) & (mask));
        if *(&raw mut game.g_ram as *mut uint8)
            .offset(0x69c)
            .offset(vline_idx as isize)
            == 48u8
        {
            break;
        }
        let ref mut fresh5 = *(&raw mut game.g_ram as *mut uint8)
            .offset(0x69c)
            .offset(vline_idx as isize);
        *fresh5 = (*fresh5).wrapping_add(1);
        n -= 1;
        if !(n != 0) {
            break;
        }
    }
    let mut v7: uint16 = 0;
    let mut v8: uint16 = 0;
    loop {
        v7 = ((*((&raw mut game.g_ram as *mut uint8).offset(0x68a) as *mut uint16))
            .wrapping_add((*((&raw mut game.g_ram as *mut uint8).offset(0x69a) as *mut uint16))));
        v8 = *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16);
        if *((game.g_rom.offset(crate::rtl::lorom_offset(0xa49bc5)) as *const uint8 as *mut uint16
            as *mut uint8)
            .offset(v7 as ::core::ffi::c_int as isize) as *mut uint16)
            != 0xffffu16
        {
            break;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x68a) as *mut uint16) = 0 as uint16;
    }
    let mut v9: *mut VramWriteEntry = gVramWriteEntry(
        *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    (*v9).size = *((game.g_rom.offset(crate::rtl::lorom_offset(0xa49bc5)) as *const uint8
        as *mut uint16 as *mut uint8)
        .offset(v7 as ::core::ffi::c_int as isize) as *mut uint16);
    (*v9).src.addr = *((game.g_rom.offset(crate::rtl::lorom_offset(0xa49bcb)) as *const uint8
        as *mut uint16 as *mut uint8)
        .offset(v7 as ::core::ffi::c_int as isize) as *mut uint16);
    *(&raw mut (*v9).src.bank as *mut uint16) =
        *((game.g_rom.offset(crate::rtl::lorom_offset(0xa49bc9)) as *const uint8 as *mut uint16
            as *mut uint8)
            .offset(v7 as ::core::ffi::c_int as isize) as *mut uint16);
    (*v9).vram_dst = *((game.g_rom.offset(crate::rtl::lorom_offset(0xa49bc7)) as *const uint8
        as *mut uint16 as *mut uint8)
        .offset(v7 as ::core::ffi::c_int as isize) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) =
        ((v8).wrapping_add((7u16)));
    let ref mut fresh6 = *((&raw mut game.g_ram as *mut uint8).offset(0x68a) as *mut uint16);
    *fresh6 = ((*fresh6).wrapping_add((8u16)));
    return 1 as uint16;
}
pub unsafe fn Crocomire_Func_68(game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16) as ::core::ffi::c_int
        - 640 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        QueueMusic_Delayed8(5 as uint16, game);
        *((&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(3) as *mut uint8
            as *mut uint16) = 256 as uint16;
        SpawnHardcodedPlm(
            SpawnHardcodedPlmArgs {
                field_0: 0x30 as uint8,
                field_1: 0x3 as uint8,
                field_2: 0xb757 as VoidP,
            },
            game,
        );
        *((&raw mut game.g_ram as *mut uint8).offset(0x941) as *mut uint16) = 6 as uint16;
        let mut E0: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int, game);
        (*E0).base.properties = ((*E0).base.properties as ::core::ffi::c_int
            & 0x7bff as ::core::ffi::c_int
            | kEnemyProps_Tangible as ::core::ffi::c_int) as uint16;
        let mut E1: *mut Enemy_Crocomire = Get_Crocomire(0x40 as ::core::ffi::c_int, game);
        (*E1).base.properties = (((*E1).base.properties) | (0x500u16));
        (*E0).crocom_var_D = 4 as uint16;
        (*E1).crocom_var_D = 0 as uint16;
        (*Get_Crocomire(0x80 as ::core::ffi::c_int, game)).crocom_var_D = 10 as uint16;
        (*Get_Crocomire(0xc0 as ::core::ffi::c_int, game)).crocom_var_D = 1 as uint16;
        (*E0).crocom_var_B = 0 as uint16;
        (*E0).base.y_height = 56 as uint16;
        Crocomire_9BB3(game);
    }
}
pub unsafe fn Crocomire_Func_69(game: &mut Game) {
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int, game);
    let mut v1: uint16 = (((*E).base.x_pos).wrapping_sub((2u16)));
    (*E).base.x_pos = v1;
    if v1 as ::core::ffi::c_int - 480 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        (*E).base.x_pos = 480 as uint16;
        (*E).base.y_pos = 54 as uint16;
        (*E).crocom_var_A = 62 as uint16;
    } else {
        (*E).base.y_pos = 220 as uint16;
    };
}
pub unsafe fn Crocomire_Func_70(game: &mut Game) {
    let mut E0: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int, game);
    let mut E1: *mut Enemy_Crocomire = Get_Crocomire(0x40 as ::core::ffi::c_int, game);
    let mut E2: *mut Enemy_Crocomire = Get_Crocomire(0x80 as ::core::ffi::c_int, game);
    let mut E3: *mut Enemy_Crocomire = Get_Crocomire(0xc0 as ::core::ffi::c_int, game);
    if *(game.g_rom.offset(crate::rtl::lorom_offset(0xa498ca)) as *const uint8 as *mut uint16)
        .offset(((*E0).crocom_var_D as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
        == 0x8080u16
    {
        (*E1).crocom_var_D = -(32640 as ::core::ffi::c_int) as uint16;
        (*E0).crocom_var_D = 128 as uint16;
        let mut i: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
        while i >= 0i32 {
            *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                .offset(((i >> 1 as ::core::ffi::c_int) + 176 as ::core::ffi::c_int) as isize) =
                *(game.g_rom.offset(crate::rtl::lorom_offset(0xa4b91d)) as *const uint8
                    as *mut uint16)
                    .offset((i >> 1 as ::core::ffi::c_int) as isize);
            i -= 2 as ::core::ffi::c_int;
        }
        Crocomire_9BB3(game);
    } else {
        let mut crocom_var_D: uint16 = (*E0).crocom_var_D;
        let mut v3: uint16 = (*E1).crocom_var_D;
        let mut v4: ::core::ffi::c_int =
            crocom_var_D as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        if v3
            == *(game.g_rom.offset(crate::rtl::lorom_offset(0xa498ca)) as *const uint8
                as *mut uint16)
                .offset(v4 as isize)
        {
            if (*(game.g_rom.offset(crate::rtl::lorom_offset(0xa498ca)) as *const uint8
                as *mut uint16)
                .offset((crocom_var_D as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
                as ::core::ffi::c_int
                & 0x8000 as ::core::ffi::c_int)
                != 0i32
            {
                let mut v7: uint16 = (*E2).crocom_var_D;
                if v7 != 0 {
                    (*E2).crocom_var_D = ((v7).wrapping_sub((1u16)));
                    (*E0).crocom_var_D = ((crocom_var_D).wrapping_sub((2u16)));
                    QueueSfx2_Max6(0x2b as uint16, game);
                    return;
                }
                let mut v8: uint16 = ((crocom_var_D).wrapping_add((2u16)));
                (*E2).crocom_var_D = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa498ca))
                    as *const uint8 as *mut uint16)
                    .offset((v8 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
                crocom_var_D = ((v8).wrapping_add((2u16)));
                (*E3).crocom_var_D = *(game.g_rom.offset(crate::rtl::lorom_offset(0xa498ca))
                    as *const uint8 as *mut uint16)
                    .offset(
                        (crocom_var_D as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize,
                    );
            }
            (*E0).crocom_var_D = ((crocom_var_D).wrapping_add((2u16)));
        } else {
            let mut v5: uint16 = 0;
            if ((v3) as i16).wrapping_sub(
                ((*(game.g_rom.offset(crate::rtl::lorom_offset(0xa498ca)) as *const uint8
                    as *mut uint16)
                    .offset(v4 as isize)) as i16),
            ) >= 0i16
            {
                v5 = ((v3).wrapping_sub(((*E3).crocom_var_D)));
            } else {
                v5 = (((*E3).crocom_var_D).wrapping_add((v3)));
            }
            (*E1).crocom_var_D = v5;
        }
    };
}
pub unsafe fn Crocomire_Func_71(game: &mut Game) {
    let mut v3: int16 = 0;
    let mut v6: *mut VramWriteEntry = ::core::ptr::null_mut::<VramWriteEntry>();
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int, game);
    let mut crocom_var_D: uint16 = (*E).crocom_var_D;
    if crocom_var_D != 0 {
        (*E).crocom_var_D = ((crocom_var_D).wrapping_sub((1u16)));
        v3 = *((&raw mut game.g_ram as *mut uint8).offset(0x688) as *mut uint16) as int16;
        let mut v4: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x688)
            as *mut uint16) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int;
        if *(game.g_rom.offset(crate::rtl::lorom_offset(0xa499cb)) as *const uint8 as *mut uint16)
            .offset(v4 as isize)
            != 0xffffu16
        {
            let mut v5: uint16 =
                *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16);
            v6 = gVramWriteEntry(
                *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16)
                    as ::core::ffi::c_int,
                game,
            );
            (*v6).size = 512 as uint16;
            (*v6).src.addr =
                *(game.g_rom.offset(crate::rtl::lorom_offset(0xa499d9)) as *const uint8
                    as *mut uint16)
                    .offset(v4 as isize);
            *(&raw mut (*v6).src.bank as *mut uint16) = 173 as uint16;
            (*v6).vram_dst = (*(game.g_rom.offset(crate::rtl::lorom_offset(0xa499cb))
                as *const uint8 as *mut uint16)
                .offset(v4 as isize) as ::core::ffi::c_int
                + ((*(&raw mut game.g_ram as *mut uint8).offset(0x52) as ::core::ffi::c_int
                    & 7 as ::core::ffi::c_int)
                    << 13 as ::core::ffi::c_int)) as uint16;
            *((&raw mut game.g_ram as *mut uint8).offset(0x330) as *mut uint16) =
                ((v5).wrapping_add((7u16)));
            *((&raw mut game.g_ram as *mut uint8).offset(0x688) as *mut uint16) =
                (((v3) as u16).wrapping_add((2u16)));
        }
    } else {
        (*E).base.x_pos = 480 as uint16;
        (*E).base.y_pos = 54 as uint16;
        (*Get_Crocomire(0x80 as ::core::ffi::c_int, game)).crocom_var_D = 80 as uint16;
        (*E).crocom_var_E = 0 as uint16;
        (*E).crocom_var_F = 0 as uint16;
        SpawnHardcodedPlm(
            SpawnHardcodedPlmArgs {
                field_0: 0x20 as uint8,
                field_1: 0x3 as uint8,
                field_2: 0xb753 as VoidP,
            },
            game,
        );
        SpawnHardcodedPlm(
            SpawnHardcodedPlmArgs {
                field_0: 0x1e as uint8,
                field_1: 0x3 as uint8,
                field_2: 0xb757 as VoidP,
            },
            game,
        );
        SpawnHardcodedPlm(
            SpawnHardcodedPlmArgs {
                field_0: 0x70 as uint8,
                field_1: 0xb as uint8,
                field_2: 0xb747 as VoidP,
            },
            game,
        );
        QueueSfx2_Max6(0x29 as uint16, game);
        (*E).base.current_instruction = addr_kCrocomire_Ilist_E158 as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.palette_index = 0 as uint16;
        let mut i: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
        while i >= 0i32 {
            *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                .offset(((i >> 1 as ::core::ffi::c_int) + 144 as ::core::ffi::c_int) as isize) =
                *(game.g_rom.offset(crate::rtl::lorom_offset(0xa4b8fd)) as *const uint8
                    as *mut uint16)
                    .offset((i >> 1 as ::core::ffi::c_int) as isize);
            i -= 2 as ::core::ffi::c_int;
        }
        ClearEprojs(game);
        let mut v8: uint16 = 8 as uint16;
        let mut n: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
        loop {
            SpawnEprojWithGfx(
                v8,
                *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16),
                addr_kEproj_CrocomireSpikeWallPieces as uint16,
                game,
            );
            n -= 1;
            if !(n != 0) {
                break;
            }
        }
        QueueSfx2_Max6(0x30 as uint16, game);
        Crocomire_9BB3(game);
    };
}
pub unsafe fn Crocomire_Func_72(game: &mut Game) {
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int, game);
    if (*E).base.x_pos as ::core::ffi::c_int - 224 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut t: uint32 = (((*E).crocom_var_F as uint32) << 16 as ::core::ffi::c_int
            | (*E).crocom_var_E as uint32)
            .wrapping_add(0x8000 as uint32);
        if (t >> 16 as ::core::ffi::c_int).wrapping_sub(2 as uint32) & 0x8000 as uint32 == 0 {
            t = t & 0xffff as uint32
                | ((2 as ::core::ffi::c_int) << 16 as ::core::ffi::c_int) as uint32;
        }
        SetHiLo(&raw mut (*E).crocom_var_F, &raw mut (*E).crocom_var_E, t);
        AddToHiLo(&raw mut (*E).base.x_pos, &raw mut (*E).base.x_subpos, t);
    }
    let mut E2: *mut Enemy_Crocomire = Get_Crocomire(0x80 as ::core::ffi::c_int, game);
    if (*E2).crocom_var_D != 0u16 && {
        let fresh19 = (*E2).crocom_var_D;
        (*E2).crocom_var_D = (*E2).crocom_var_D.wrapping_sub(1);
        fresh19 == 1u16
    } {
        (*E).crocom_var_E = 0 as uint16;
        (*E).base.current_instruction = addr_kCrocomire_Ilist_E14A as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        Crocomire_9BB3(game);
    }
}
pub unsafe fn Crocomire_Func_73(game: &mut Game) {
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int, game);
    let mut t: uint32 = (((*E).crocom_var_F as uint32) << 16 as ::core::ffi::c_int
        | (*E).crocom_var_E as uint32)
        .wrapping_add(2048 as uint32);
    if (t >> 16 as ::core::ffi::c_int).wrapping_sub(5 as uint32) & 0x8000 as uint32 == 0 {
        t = t & 0xffff as uint32
            | ((5 as ::core::ffi::c_int) << 16 as ::core::ffi::c_int) as uint32;
    }
    SetHiLo(&raw mut (*E).crocom_var_F, &raw mut (*E).crocom_var_E, t);
    AddToHiLo(
        &raw mut (*E).base.y_pos,
        &raw mut (*E).base.y_subpos,
        0xe000 as uint32,
    );
    AddToHiLo(&raw mut (*E).base.x_pos, &raw mut (*E).base.x_subpos, t);
    if (*E).base.x_pos as ::core::ffi::c_int - 576 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        QueueSfx2_Max6(0x25 as uint16, game);
        (*Get_Crocomire(0x40 as ::core::ffi::c_int, game))
            .base
            .palette_index = (*E).base.palette_index;
        (*E).base.current_instruction = addr_kCrocomire_Ilist_E158 as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        Crocomire_9BB3(game);
    }
}
pub unsafe fn Crocomire_Instr_8(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    return Crocomire_Func_87(jp, 0xffe0 as uint16, game);
}
pub unsafe fn Crocomire_Instr_6(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    return Crocomire_Func_87(jp, 0 as uint16, game);
}
pub unsafe fn Crocomire_Instr_9(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    return Crocomire_Func_87(jp, 0xfff0 as uint16, game);
}
pub unsafe fn Crocomire_Instr_5(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    return Crocomire_Func_87(jp, 0x10 as uint16, game);
}
pub unsafe fn Crocomire_Instr_20(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    return Crocomire_Func_87(jp, 0 as uint16, game);
}
pub unsafe fn Crocomire_Instr_21(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    return Crocomire_Func_87(jp, 8 as uint16, game);
}
pub unsafe fn Crocomire_Instr_22(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    return Crocomire_Func_87(jp, 0x10 as uint16, game);
}
pub unsafe fn Crocomire_Instr_23(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    return Crocomire_Func_87(jp, 0x18 as uint16, game);
}
pub unsafe fn Crocomire_Instr_24(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    return Crocomire_Func_87(jp, 0x20 as uint16, game);
}
pub unsafe fn Crocomire_Instr_10(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    return Crocomire_Func_87(jp, 0x28 as uint16, game);
}
pub unsafe fn Crocomire_Instr_25(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    return Crocomire_Func_87(jp, 0x30 as uint16, game);
}
pub unsafe fn Crocomire_Instr_26(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    return Crocomire_Func_87(jp, 0x38 as uint16, game);
}
pub unsafe fn Crocomire_Instr_27(
    mut k: uint16,
    mut jp: *const uint16,
    game: &mut Game,
) -> *const uint16 {
    return Crocomire_Func_87(jp, 0x40 as uint16, game);
}
pub unsafe fn Crocomire_Func_87(
    mut jp: *const uint16,
    mut a: uint16,
    game: &mut Game,
) -> *const uint16 {
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int, game);
    game.eproj_spawn_pt = Point16U {
        x: (a as ::core::ffi::c_int
            + (*E).base.x_pos as ::core::ffi::c_int
            + (*((&raw mut game.g_ram as *mut uint8).offset(0x5e5) as *mut uint16)
                as ::core::ffi::c_int
                & 7 as ::core::ffi::c_int)) as uint16,
        y: ((*E).base.y_height as ::core::ffi::c_int + (*E).base.y_pos as ::core::ffi::c_int
            - 16 as ::core::ffi::c_int) as uint16,
    };
    SpawnEprojWithRoomGfx(
        addr_kEproj_DustCloudExplosion as uint16,
        0x15 as uint16,
        game,
    );
    return jp;
}
pub unsafe fn Crocomire_Func_88(game: &mut Game) {
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int, game);
    if (*E).base.current_instruction as ::core::ffi::c_int + 0x1e3a as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let mut t: uint32 = (((*E).crocom_var_F as uint32) << 16 as ::core::ffi::c_int
            | (*E).crocom_var_E as uint32)
            .wrapping_add(4096 as uint32);
        if (t >> 16 as ::core::ffi::c_int).wrapping_sub(6 as uint32) & 0x8000 as uint32 == 0 {
            t = t & 0xffff as uint32
                | ((6 as ::core::ffi::c_int) << 16 as ::core::ffi::c_int) as uint32;
        }
        SetHiLo(&raw mut (*E).crocom_var_F, &raw mut (*E).crocom_var_E, t);
    } else {
        (*E).base.current_instruction = addr_kCrocomire_Ilist_E1CC as uint16;
        (*E).base.instruction_timer = 1 as uint16;
        (*E).base.x_pos = (((*E).base.x_pos).wrapping_add((64u16)));
        (*E).base.y_pos = (((*E).base.y_pos).wrapping_add((21u16)));
        (*E).base.y_height = 28 as uint16;
        (*E).base.x_width = 40 as uint16;
        SpawnHardcodedPlm(
            SpawnHardcodedPlmArgs {
                field_0: 0x30 as uint8,
                field_1: 0x3 as uint8,
                field_2: 0xb753 as VoidP,
            },
            game,
        );
        Enemy_ItemDrop_Crocomire(0 as uint16, game);
        Crocomire_9BB3(game);
    };
}
pub unsafe fn Crocomire_9B65(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xcd20) as *mut uint16) = 257 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xcd20).offset(2) as *mut uint8 as *mut uint16) =
        257 as uint16;
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs {
            field_0: 0x1e as uint8,
            field_1: 0x3 as uint8,
            field_2: 0xb753 as VoidP,
        },
        game,
    );
    Crocomire_9BB3(game);
}
pub unsafe fn Crocomire_Func_89(game: &mut Game) {
    Crocomire_9BB3(game);
}
pub unsafe fn Crocomire_Func_90(game: &mut Game) {
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int, game);
    if (*E).base.current_instruction as ::core::ffi::c_int + 0x1e3a as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        == 0
    {
        Crocomire_9BB3(game);
    }
}
pub unsafe fn Crocomire_9B86(game: &mut Game) {
    QueueMusic_Delayed8(6 as uint16, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x941) as *mut uint16) = 0 as uint16;
    let ref mut fresh10 = *((&raw mut game.g_ram as *mut uint8)
        .offset(0xd828)
        .offset(*((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) as isize)
        as *mut uint8 as *mut uint16);
    *fresh10 = ((*fresh10) | (2u16));
    QueueMusic_Delayed8(6 as uint16, game);
    Crocomire_Func_87(::core::ptr::null::<uint16>(), 0xfff0 as uint16, game);
    Crocomire_Func_87(::core::ptr::null::<uint16>(), 0x10 as uint16, game);
    Crocomire_9BB3(game);
}
pub unsafe fn Crocomire_9BB3(game: &mut Game) {
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int, game);
    (*E).crocom_var_A = (*E).crocom_var_A.wrapping_add(1);
    (*E).crocom_var_A = (*E).crocom_var_A.wrapping_add(1);
}
pub unsafe fn Crocomire_Func_91(game: &mut Game) {
    Crocomire_9BB3(game);
}
pub unsafe fn Crocomire_Func_92(game: &mut Game) {
    NormalEnemyTouchAi(game);
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int, game);
    (*E).crocom_var_B = (((*E).crocom_var_B) | (0x4000u16));
    *((&raw mut game.g_ram as *mut uint8).offset(0xb58) as *mut uint16) =
        -(4 as ::core::ffi::c_int) as uint16;
}
pub unsafe fn Crocomire_Func_93(game: &mut Game) {
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int, game);
    let mut v1: uint16 = (((*E).crocom_var_B) & (0xfu16));
    if v1 as ::core::ffi::c_int - 15 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
        v1 = v1.wrapping_add(1);
    }
    (*E).crocom_var_B = (((*E).crocom_var_B) | (v1));
}
pub unsafe fn Crocomire_Func_94(game: &mut Game) {
    let mut v0: ::core::ffi::c_int =
        *((&raw mut game.g_ram as *mut uint8).offset(0x18a6) as *mut uint16) as ::core::ffi::c_int;
    game.eproj_spawn_pt = Point16U {
        x: *((&raw mut game.g_ram as *mut uint8).offset(0xb64) as *mut uint16).offset(v0 as isize),
        y: *((&raw mut game.g_ram as *mut uint8).offset(0xb78) as *mut uint16).offset(v0 as isize),
    };
    let mut v1: uint16 = (if (*((&raw mut game.g_ram as *mut uint8).offset(0xc18) as *mut uint16)
        .offset(v0 as isize) as ::core::ffi::c_int
        & 0x200 as ::core::ffi::c_int)
        == 0i32
    {
        6 as ::core::ffi::c_int
    } else {
        29 as ::core::ffi::c_int
    }) as uint16;
    SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as uint16, v1, game);
}
pub unsafe fn Crocomire_Powerbomb(game: &mut Game) {
    let mut v2: uint16 = 0;
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int, game);
    if (*E).crocom_var_A == 0 {
        if g_word_A4869E != 0 {
            (*E).crocom_var_D = g_word_A4869E;
            if (*E).crocom_var_C != 24u16 {
                (*E).crocom_var_B = ((*E).crocom_var_B as ::core::ffi::c_int
                    & 0x3ff0 as ::core::ffi::c_int
                    | 0x8000 as ::core::ffi::c_int) as uint16;
                (*E).crocom_var_E = 10 as uint16;
                (*E).base.flash_timer = (((*E).base.flash_timer).wrapping_add((4u16)));
                (*E).base.ai_handler_bits = (((*E).base.ai_handler_bits) | (2u16));
                (*E).crocom_var_C = 24 as uint16;
                let mut spritemap_pointer: uint16 = (*E).base.spritemap_pointer;
                let mut v3: uint16 = 0;
                let mut n: ::core::ffi::c_int =
                    *(RomPtr_A4(spritemap_pointer, game) as *mut uint16) as ::core::ffi::c_int;
                loop {
                    v2 = *(RomPtr_A4(spritemap_pointer, game) as *mut uint16).offset(3);
                    v3 = addr_kCrocomire_Ilist_BDAE as uint16;
                    if v2 as ::core::ffi::c_int
                        == addr_kCrocomire_BigSprmap_D600 as ::core::ffi::c_int
                    {
                        break;
                    }
                    v3 = addr_kCrocomire_Ilist_BDB2 as uint16;
                    if v2 as ::core::ffi::c_int
                        == addr_kCrocomire_BigSprmap_D51C as ::core::ffi::c_int
                    {
                        break;
                    }
                    spritemap_pointer = ((spritemap_pointer).wrapping_add((8u16)));
                    n -= 1;
                    if !(n == 0) {
                        continue;
                    }
                    v3 = addr_kCrocomire_Ilist_BDB6 as uint16;
                    break;
                }
                let mut E0: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int, game);
                (*E0).base.current_instruction = v3;
                (*E0).base.instruction_timer = 1 as uint16;
            }
        }
    }
}
pub unsafe fn Crocomire_Func_95(game: &mut Game) {
    let mut v1: uint16 = 0;
    let mut v2: uint16 = 0;
    let mut current_block: u64;
    let mut v3: int16 = 0;
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(0 as ::core::ffi::c_int, game);
    (*E).base.invincibility_timer = 0 as uint16;
    if (((*E).base.x_pos as ::core::ffi::c_int
        - (*E).base.x_width as ::core::ffi::c_int
        - 256 as ::core::ffi::c_int
        - *((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16) as ::core::ffi::c_int)
        as int16)
        >= 0i16
    {
        current_block = 4956146061682418353;
    } else {
        v1 = 0;
        v2 = 0;
        v1 = *((&raw mut game.g_ram as *mut uint8).offset(0xc18) as *mut uint16)
            .offset(*((&raw mut game.g_ram as *mut uint8).offset(0x18a6) as *mut uint16) as isize);
        if (v1 as ::core::ffi::c_int & 0xf00 as ::core::ffi::c_int) != 0i32 {
            v3 = (((v1) as i16) & (0xf00i16));
            v2 = g_word_A4869A;
            if v3 != 256i16 {
                v2 = g_word_A4869C;
                if v3 != 512i16 {
                    v2 = 0 as uint16;
                }
            }
        } else {
            v2 = g_word_A48698;
            if (v1 as ::core::ffi::c_int & 0x10 as ::core::ffi::c_int) == 0i32 {
                (*E).base.instruction_timer = g_word_A486A0;
                Crocomire_Func_1(game);
                return;
            }
        }
        if v2 != 0 {
            (*E).crocom_var_D = (((*E).crocom_var_D).wrapping_add((v2)));
            current_block = 4956146061682418353;
        } else {
            current_block = 7172762164747879670;
        }
    }
    match current_block {
        4956146061682418353 => {
            let mut v4: uint16 = (((*E).crocom_var_B) & (0xfu16));
            if v4 as ::core::ffi::c_int - 15 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int
                != 0
            {
                v4 = v4.wrapping_add(1);
            }
            if ((*E).crocom_var_B as ::core::ffi::c_int & 0x800 as ::core::ffi::c_int) == 0i32 {
                let mut v5: uint16 = g_word_A48692;
                if (*E).crocom_var_C == 8u16 {
                    v5 = g_word_A48694;
                }
                (*E).base.instruction_timer = (((*E).base.instruction_timer).wrapping_add((v5)));
            }
            (*E).crocom_var_B = (v4 as ::core::ffi::c_int
                | (*E).crocom_var_B as ::core::ffi::c_int & 0xb7f0 as ::core::ffi::c_int
                | 0x800 as ::core::ffi::c_int) as uint16;
            (*E).crocom_var_E = 10 as uint16;
        }
        _ => {}
    }
    (*E).base.flash_timer = (((*E).base.flash_timer).wrapping_add((14u16)));
    (*E).base.ai_handler_bits = (((*E).base.ai_handler_bits) | (2u16));
}
pub unsafe fn Crocomire_Func_1(game: &mut Game) {
    let mut v0: ::core::ffi::c_int =
        *((&raw mut game.g_ram as *mut uint8).offset(0x18a6) as *mut uint16) as ::core::ffi::c_int;
    game.eproj_spawn_pt = Point16U {
        x: *((&raw mut game.g_ram as *mut uint8).offset(0xb64) as *mut uint16).offset(v0 as isize),
        y: *((&raw mut game.g_ram as *mut uint8).offset(0xb78) as *mut uint16).offset(v0 as isize),
    };
    let mut v1: uint16 = (if (*((&raw mut game.g_ram as *mut uint8).offset(0xc18) as *mut uint16)
        .offset(v0 as isize) as ::core::ffi::c_int
        & 0x200 as ::core::ffi::c_int)
        == 0i32
    {
        6 as ::core::ffi::c_int
    } else {
        29 as ::core::ffi::c_int
    }) as uint16;
    SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as uint16, v1, game);
}
pub unsafe fn CrocomireTongue_Init(game: &mut Game) {
    let mut E: *mut Enemy_Crocomire = Get_Crocomire(
        *((&raw mut game.g_ram as *mut uint8).offset(0xe54) as *mut uint16) as ::core::ffi::c_int,
        game,
    );
    if (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xd828)
        .offset(*((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) as isize)
        as *mut uint8 as *mut uint16) as ::core::ffi::c_int
        & 2 as ::core::ffi::c_int)
        != 0i32
    {
        (*E).base.properties = ((*E).base.properties as ::core::ffi::c_int
            & 0xdcff as ::core::ffi::c_int
            | 0x300 as ::core::ffi::c_int) as uint16;
    } else {
        (*E).base.current_instruction = addr_kCrocomire_Ilist_BE56 as uint16;
        (*E).base.extra_properties = (((*E).base.extra_properties) | (0x404u16));
        (*E).base.instruction_timer = 1 as uint16;
        (*E).crocom_var_A = 23 as uint16;
        (*E).base.palette_index = 3584 as uint16;
    };
}
pub fn CrocomireTongue_Main() {}
