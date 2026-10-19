pub use crate::game::constants::bank_8d::*;
use crate::game::sm_80::{QueueMusic_Delayed8, QueueSfx1_Max6, QueueSfx2_Max6, QueueSfx3_Max6};
use crate::game_state::Game;
use crate::rtl::AddToHiLo;
use crate::rtl::{MakePairU16, RomPtr, Unreachable};

pub use crate::types::{
    __uint16_t, __uint32_t, __uint8_t, uint16, uint16_t, uint32, uint32_t, uint8, uint8_t,
    EnemyData, PairU16, VoidP,
};
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct PalFxDef {
    pub func: VoidP,
    pub instrs: VoidP,
}
pub use crate::types::Consts_8D;
#[inline]
unsafe fn RomPtr_8D(mut addr: uint16_t, game: &mut Game) -> *const uint8 {
    return RomPtr(((0x8d0000u32) | ((addr) as u32)), game);
}
#[inline]
unsafe fn get_PalFxDef(mut a: uint16, game: &mut Game) -> *mut PalFxDef {
    return RomPtr(((0x8d0000u32) | ((a) as u32)), game) as *mut PalFxDef;
}
pub const fnCinematicFunction_Intro_Page2: ::core::ffi::c_int = 0x8bb35f;
pub const fnPalPreInstr_nullsub_129: ::core::ffi::c_int = 0x8dc526;
pub const fnPalInstr_Wait: uint32 = 9291157;
pub const fnPalInstr_ColorPlus2: uint32 = 9291161;
pub const fnPalInstr_ColorPlus3: uint32 = 9291170;
pub const fnPalInstr_ColorPlus4: uint32 = 9291179;
pub const fnPalInstr_ColorPlus8: uint32 = 9291188;
pub const fnPalInstr_ColorPlus9: uint32 = 9291197;
pub const fnPalInstr_ColorPlus15: uint32 = 9291206;
pub const fnPalInstr_Delete: uint32 = 9291215;
pub const fnPalInstr_SetPreInstr: uint32 = 9291220;
pub const fnPalInstr_ClearPreInstr: uint32 = 9291229;
pub const fnPalInstr_Goto: uint32 = 9291294;
pub const fnPalInstr_DecTimerGoto: uint32 = 9291321;
pub const fnPalInstr_SetTimer: uint32 = 9291336;
pub const fnPalInstr_SetColorIndex: uint32 = 9291349;
pub const fnPalInstr_QueueMusic: uint32 = 9291358;
pub const fnPalInstr_QueueSfx1: uint32 = 9291370;
pub const fnPalInstr_QueueSfx2: uint32 = 9291379;
pub const fnPalInstr_QueueSfx3: uint32 = 9291388;
pub const fnnullsub_131: uint32 = 9291397;
pub const fnPalInit_E1BC: uint32 = 9298436;
pub const fnPalPreInstr_E1BC: ::core::ffi::c_int = 0x8de20b;
pub const fnPalPreInstr_CheckEnemy0Health: uint32 = 9298656;
pub const fnPalPreInstr_SamusInHeat: uint32 = 9298809;
pub const fnPalInit_F761_Norfair1: uint32 = 9299008;
pub const fnPalPreInstr_SwitchIfYpos: uint32 = 9301081;
pub const fnPalPreInstr_SwitchIfYpos2: uint32 = 9301380;
pub const fnPalPreInstr_DeletePalfxIfMinibossDead: uint32 = 9301701;
pub const fnPalInstr_SetPalfxIndex: uint32 = 9302470;
pub const fnPalPreInstr_F621: uint32 = 9303585;
pub const fnPalInit_F779_Brinstar8: uint32 = 9303856;
pub unsafe fn EnablePaletteFx(game: &mut Game) {
    let ref mut fresh1 = *((&raw mut game.g_ram as *mut uint8).offset(0x1e79) as *mut uint16);
    *fresh1 = ((*fresh1) | (0x8000u16));
}
pub unsafe fn DisablePaletteFx(game: &mut Game) {
    let ref mut fresh0 = *((&raw mut game.g_ram as *mut uint8).offset(0x1e79) as *mut uint16);
    *fresh0 = (*fresh0 as ::core::ffi::c_int & !(0x8000 as ::core::ffi::c_int)) as uint16;
}
pub unsafe fn ClearPaletteFXObjects(game: &mut Game) {
    let mut i: ::core::ffi::c_int = 14 as ::core::ffi::c_int;
    while i >= 0i32 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1e7d) as *mut uint16)
            .offset((i >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
        i -= 2 as ::core::ffi::c_int;
    }
}
pub unsafe fn SpawnPalfxObject(mut j: uint16, game: &mut Game) {
    let mut PalFxDef: *mut PalFxDef = ::core::ptr::null_mut::<PalFxDef>();
    let mut v1: uint16 = 14 as uint16;
    while *((&raw mut game.g_ram as *mut uint8).offset(0x1e7d) as *mut uint16)
        .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
        != 0
    {
        v1 = ((v1).wrapping_sub((2u16)));
        if (v1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
            return;
        }
    }
    let mut v2: ::core::ffi::c_int = v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1e7d) as *mut uint16).offset(v2 as isize) = j;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1e8d) as *mut uint16).offset(v2 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ead) as *mut uint16).offset(v2 as isize) =
        (((0x8dc526i32) as u16) & (0xffffu16));
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ebd) as *mut uint16).offset(v2 as isize) =
        (*get_PalFxDef(j, game)).instrs;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ecd) as *mut uint16).offset(v2 as isize) =
        1 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1edd) as *mut uint16).offset(v2 as isize) =
        0 as uint16;
    PalFxDef = get_PalFxDef(j, game);
    CallPalfxFunc(
        ((((*PalFxDef).func) as u32) | (0x8d0000u32)),
        -(1 as ::core::ffi::c_int) as uint16,
        v1,
        game,
    );
}
pub fn PalPreInstr_nullsub_129(mut k: uint16) {}
pub unsafe fn PaletteFxHandler(game: &mut Game) {
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x1e79) as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int)
        != 0i32
    {
        let mut i: ::core::ffi::c_int = 14 as ::core::ffi::c_int;
        while i >= 0i32 {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1e7b) as *mut uint16) = i as uint16;
            if *((&raw mut game.g_ram as *mut uint8).offset(0x1e7d) as *mut uint16)
                .offset((i >> 1 as ::core::ffi::c_int) as isize)
                != 0
            {
                PalFx_ProcessOne(i as uint16, game);
                i = *((&raw mut game.g_ram as *mut uint8).offset(0x1e7b) as *mut uint16)
                    as ::core::ffi::c_int;
            }
            i -= 2 as ::core::ffi::c_int;
        }
    }
}
pub unsafe fn PalFx_ProcessOne(mut k: uint16, game: &mut Game) {
    CallPalFxPreInstr(
        (((*((&raw mut game.g_ram as *mut uint8).offset(0x1ead) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize))
            as u32)
            | (0x8d0000u32)),
        k,
        game,
    );
    let mut v1: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x1e7b) as *mut uint16);
    let ref mut fresh2 = *((&raw mut game.g_ram as *mut uint8).offset(0x1ecd) as *mut uint16)
        .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    let fresh3 = *fresh2;
    *fresh2 = (*fresh2).wrapping_sub(1);
    if fresh3 != 1u16 {
        return;
    }
    let mut j: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x1ebd) as *mut uint16)
        .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize);
    let mut v6: uint16 = 0;
    loop {
        let mut p: *const uint16 = RomPtr_8D(j, game) as *const uint16;
        v6 = *p;
        if (*p as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) == 0i32 {
            break;
        }
        let mut v7: PairU16 = CallPalInstr(
            (((v6) as u32) | (0x8d0000u32)),
            v1,
            ((j).wrapping_add((2u16))),
            game,
        );
        v1 = v7.k;
        j = v7.j;
        if v7.j == 0 {
            return;
        }
    }
    let mut v8: ::core::ffi::c_int = v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ecd) as *mut uint16).offset(v8 as isize) = v6;
    let mut color_idx: uint16 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1e8d) as *mut uint16).offset(v8 as isize);
    let mut v10: *const uint8 = ::core::ptr::null::<uint8>();
    let mut v11: uint16 = 0;
    loop {
        loop {
            v10 = RomPtr_8D(j, game);
            v11 = *(v10.offset(2) as *mut uint16);
            if v11 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
                break;
            }
            *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
                .offset((color_idx as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
                v11;
            color_idx = ((color_idx).wrapping_add((2u16)));
            j = ((j).wrapping_add((2u16)));
        }
        let mut v12: PairU16 = CallPalInstr((((v11) as u32) | (0x8d0000u32)), color_idx, j, game);
        color_idx = v12.k;
        j = v12.j;
        if !(j != 0) {
            break;
        }
    }
}
pub unsafe fn PalInstr_Wait(mut k: uint16, mut j: uint16, game: &mut Game) -> PairU16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ebd) as *mut uint16).offset(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x1e7b) as *mut uint16) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int) as isize,
    ) = ((j).wrapping_add((4u16)));
    return MakePairU16(0 as uint16, 0 as uint16);
}
pub unsafe fn PalInstr_ColorPlus2(mut k: uint16, mut j: uint16) -> PairU16 {
    return MakePairU16(((k).wrapping_add((4u16))), ((j).wrapping_add((2u16))));
}
pub unsafe fn PalInstr_ColorPlus3(mut k: uint16, mut j: uint16) -> PairU16 {
    return MakePairU16(((k).wrapping_add((6u16))), ((j).wrapping_add((2u16))));
}
pub unsafe fn PalInstr_ColorPlus4(mut k: uint16, mut j: uint16) -> PairU16 {
    return MakePairU16(((k).wrapping_add((8u16))), ((j).wrapping_add((2u16))));
}
pub unsafe fn PalInstr_ColorPlus8(mut k: uint16, mut j: uint16) -> PairU16 {
    return MakePairU16(((k).wrapping_add((16u16))), ((j).wrapping_add((2u16))));
}
pub unsafe fn PalInstr_ColorPlus9(mut k: uint16, mut j: uint16) -> PairU16 {
    return MakePairU16(((k).wrapping_add((18u16))), ((j).wrapping_add((2u16))));
}
pub unsafe fn PalInstr_ColorPlus15(mut k: uint16, mut j: uint16) -> PairU16 {
    return MakePairU16(((k).wrapping_add((30u16))), ((j).wrapping_add((2u16))));
}
pub unsafe fn PalInstr_Delete(mut k: uint16, mut j: uint16, game: &mut Game) -> PairU16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1e7d) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    return MakePairU16(k, 0 as uint16);
}
pub unsafe fn PalInstr_SetPreInstr(mut k: uint16, mut j: uint16, game: &mut Game) -> PairU16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ead) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        *(RomPtr_8D(j, game) as *mut uint16);
    return MakePairU16(k, ((j).wrapping_add((2u16))));
}
pub unsafe fn PalInstr_ClearPreInstr(mut k: uint16, mut j: uint16, game: &mut Game) -> PairU16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ead) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        addr_locret_8DC5E3 as uint16;
    return MakePairU16(k, j);
}
pub unsafe fn PalInstr_Goto(mut k: uint16, mut j: uint16, game: &mut Game) -> PairU16 {
    let mut v2: uint16 = *(RomPtr_8D(j, game) as *mut uint16);
    return MakePairU16(k, v2);
}
pub unsafe fn PalInstr_DecTimerGoto(mut k: uint16, mut j: uint16, game: &mut Game) -> PairU16 {
    let mut v4: PairU16 = PairU16 { k: 0, j: 0 };
    let mut v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh4 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1edd) as *mut uint16).offset(v2 as isize);
    let fresh5 = *fresh4;
    *fresh4 = (*fresh4).wrapping_sub(1);
    if fresh5 == 1u16 {
        return MakePairU16(k, ((j).wrapping_add((2u16))));
    }
    v4 = PalInstr_Goto(k, j, game);
    return MakePairU16(v4.k, v4.j);
}
pub unsafe fn PalInstr_SetTimer(mut k: uint16, mut j: uint16, game: &mut Game) -> PairU16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1edd) as *mut uint16 as *mut uint8)
        .offset(k as ::core::ffi::c_int as isize) = *RomPtr_8D(j, game);
    return MakePairU16(k, ((j).wrapping_add((1u16))));
}
pub unsafe fn PalInstr_SetColorIndex(mut k: uint16, mut j: uint16, game: &mut Game) -> PairU16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1e8d) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        *(RomPtr_8D(j, game) as *mut uint16);
    return MakePairU16(k, ((j).wrapping_add((2u16))));
}
pub unsafe fn PalInstr_QueueMusic(mut k: uint16, mut j: uint16, game: &mut Game) -> PairU16 {
    let mut v2: *const uint8 = RomPtr_8D(j, game);
    QueueMusic_Delayed8(*v2 as uint16, game);
    return MakePairU16(k, ((j).wrapping_add((1u16))));
}
pub unsafe fn PalInstr_QueueSfx1(mut k: uint16, mut j: uint16, game: &mut Game) -> PairU16 {
    let mut v2: *const uint16 = RomPtr_8D(j, game) as *const uint16;
    QueueSfx1_Max6(*v2, game);
    return MakePairU16(k, ((j).wrapping_add((1u16))));
}
pub unsafe fn PalInstr_QueueSfx2(mut k: uint16, mut j: uint16, game: &mut Game) -> PairU16 {
    let mut v2: *const uint16 = RomPtr_8D(j, game) as *const uint16;
    QueueSfx2_Max6(*v2, game);
    return MakePairU16(k, ((j).wrapping_add((1u16))));
}
pub unsafe fn PalInstr_QueueSfx3(mut k: uint16, mut j: uint16, game: &mut Game) -> PairU16 {
    let mut v2: *const uint16 = RomPtr_8D(j, game) as *const uint16;
    QueueSfx3_Max6(*v2, game);
    return MakePairU16(k, ((j).wrapping_add((1u16))));
}
pub unsafe fn PalInit_E1BC(mut k: uint16, mut j: uint16, game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ead) as *mut uint16)
        .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
        (((0x8de20bi32) as u16) & (0xffffu16));
}
pub unsafe fn PalPreInstr_E1BC(mut k: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1f51) as *mut uint16) as ::core::ffi::c_int
        == 0x8bb35f as ::core::ffi::c_int & 0xffff as ::core::ffi::c_int
    {
        let mut v1: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ebd) as *mut uint16).offset(v1 as isize) =
            addr_off_8DE192 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ecd) as *mut uint16).offset(v1 as isize) =
            1 as uint16;
    }
}
pub unsafe fn PalPreInstr_CheckEnemy0Health(mut k: uint16, game: &mut Game) {
    if (*((&raw mut game.g_ram as *mut uint8).offset(0xf78) as *mut EnemyData).offset(0)).health
        == 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1e7d) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    }
}
pub unsafe fn PalPreInstr_SamusInHeat(mut k: uint16, game: &mut Game) {
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x9a2) as *mut uint16) as ::core::ffi::c_int
        & 0x21 as ::core::ffi::c_int)
        == 0i32
    {
        AddToHiLo(
            (&raw mut game.g_ram as *mut uint8).offset(0xa50) as *mut uint16,
            (&raw mut game.g_ram as *mut uint8).offset(0xa4e) as *mut uint16,
            0x4000 as uint32,
        );
        if (*((&raw mut game.g_ram as *mut uint8).offset(0x5b6) as *mut uint16)
            as ::core::ffi::c_int
            & 7 as ::core::ffi::c_int)
            == 0i32
            && *((&raw mut game.g_ram as *mut uint8).offset(0x9c2) as *mut uint16) > 0x46u16
        {
            QueueSfx3_Max6(0x2d as uint16, game);
        }
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0x1eed) as *mut uint16)
        != *((&raw mut game.g_ram as *mut uint8).offset(0x1eef) as *mut uint16)
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1eef) as *mut uint16) =
            *((&raw mut game.g_ram as *mut uint8).offset(0x1eed) as *mut uint16);
        let mut v2: uint16 = ((2u16)
            .wrapping_mul((*((&raw mut game.g_ram as *mut uint8).offset(0x1eed) as *mut uint16))));
        let mut v4: uint16 = 0;
        let mut v3: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ecd) as *mut uint16).offset(v3 as isize) =
            1 as uint16;
        if (*((&raw mut game.g_ram as *mut uint8).offset(0x9a2) as *mut uint16)
            as ::core::ffi::c_int
            & 0x20 as ::core::ffi::c_int)
            != 0i32
        {
            v4 = addr_off_8DE3E0 as uint16;
        } else if (*((&raw mut game.g_ram as *mut uint8).offset(0x9a2) as *mut uint16)
            as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int)
            != 0i32
        {
            v4 = addr_off_8DE400 as uint16;
        } else {
            v4 = addr_off_8DE420 as uint16;
        }
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ebd) as *mut uint16).offset(v3 as isize) =
            *((RomPtr_8D as unsafe fn(uint16_t, &mut Game) -> *const uint8)(v4, game)
                .offset(v2 as isize) as *const uint8 as *mut uint16);
    }
}
pub unsafe fn PalInit_F761_Norfair1(mut k: uint16, mut j: uint16, game: &mut Game) {
    let mut v2: uint16 = 0;
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x9a2) as *mut uint16) as ::core::ffi::c_int
        & 0x20 as ::core::ffi::c_int)
        != 0i32
    {
        v2 = addr_off_8DE8B6 as uint16;
    } else if (*((&raw mut game.g_ram as *mut uint8).offset(0x9a2) as *mut uint16)
        as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int)
        != 0i32
    {
        v2 = addr_off_8DE68A as uint16;
    } else {
        v2 = addr_kPalfxInstrList_E45E as uint16;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ebd) as *mut uint16)
        .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = v2;
}
pub unsafe fn PalPreInstr_SwitchIfYpos(mut k: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16) < 0x380u16 {
        let mut v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ecd) as *mut uint16).offset(v2 as isize) =
            1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ebd) as *mut uint16).offset(v2 as isize) =
            addr_word_8DEB43 as uint16;
    }
}
pub unsafe fn PalPreInstr_SwitchIfYpos2(mut k: uint16, game: &mut Game) {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16) < 0x380u16 {
        let mut v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ecd) as *mut uint16).offset(v2 as isize) =
            1 as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ebd) as *mut uint16).offset(v2 as isize) =
            addr_word_8DEC76 as uint16;
    }
}
pub unsafe fn PalPreInstr_DeletePalfxIfMinibossDead(mut k: uint16, game: &mut Game) {
    if (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xd828)
        .offset(*((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) as isize)
        as *mut uint8 as *mut uint16) as ::core::ffi::c_int
        & 2 as ::core::ffi::c_int)
        != 0i32
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1e7d) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    }
}
pub unsafe fn PalInstr_SetPalfxIndex(mut k: uint16, mut j: uint16, game: &mut Game) -> PairU16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1eed) as *mut uint16) =
        *RomPtr_8D(j, game) as uint16;
    return MakePairU16(k, ((j).wrapping_add((1u16))));
}
pub unsafe fn PalPreInstr_F621(mut k: uint16, game: &mut Game) {
    if *(((&raw mut game.g_ram as *mut uint8).offset(0x1e79) as *mut uint16 as *mut uint8)
        .offset(k as ::core::ffi::c_int as isize) as *mut uint16)
        != 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1e7d) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    }
}
pub unsafe fn PalInit_F779_Brinstar8(mut k: uint16, mut j: uint16, game: &mut Game) {
    if (*((&raw mut game.g_ram as *mut uint8)
        .offset(0xd828)
        .offset(*((&raw mut game.g_ram as *mut uint8).offset(0x79f) as *mut uint16) as isize)
        as *mut uint8 as *mut uint16) as ::core::ffi::c_int
        & 2 as ::core::ffi::c_int)
        != 0i32
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1e7d) as *mut uint16)
            .offset((j as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    }
}
pub unsafe fn CallPalFxPreInstr(mut ea: uint32, mut k: uint16, game: &mut Game) {
    match ea {
        9291046 => {
            PalPreInstr_nullsub_129(k);
            return;
        }
        9298443 => {
            PalPreInstr_E1BC(k, game);
            return;
        }
        9298656 => {
            PalPreInstr_CheckEnemy0Health(k, game);
            return;
        }
        9298809 => {
            PalPreInstr_SamusInHeat(k, game);
            return;
        }
        9301701 => {
            PalPreInstr_DeletePalfxIfMinibossDead(k, game);
            return;
        }
        9301081 => {
            PalPreInstr_SwitchIfYpos(k, game);
            return;
        }
        9301380 => {
            PalPreInstr_SwitchIfYpos2(k, game);
            return;
        }
        9303585 => {
            PalPreInstr_F621(k, game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn CallPalInstr(
    mut ea: uint32,
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> PairU16 {
    match ea {
        9291157 => return PalInstr_Wait(k, j, game),
        9291161 => return PalInstr_ColorPlus2(k, j),
        9291170 => return PalInstr_ColorPlus3(k, j),
        9291179 => return PalInstr_ColorPlus4(k, j),
        9291188 => return PalInstr_ColorPlus8(k, j),
        9291197 => return PalInstr_ColorPlus9(k, j),
        9291206 => return PalInstr_ColorPlus15(k, j),
        9291215 => return PalInstr_Delete(k, j, game),
        9291220 => return PalInstr_SetPreInstr(k, j, game),
        9291229 => return PalInstr_ClearPreInstr(k, j, game),
        9291294 => return PalInstr_Goto(k, j, game),
        9291321 => return PalInstr_DecTimerGoto(k, j, game),
        9291336 => return PalInstr_SetTimer(k, j, game),
        9291349 => return PalInstr_SetColorIndex(k, j, game),
        9291358 => return PalInstr_QueueMusic(k, j, game),
        9291370 => return PalInstr_QueueSfx1(k, j, game),
        9291379 => return PalInstr_QueueSfx2(k, j, game),
        9291388 => return PalInstr_QueueSfx3(k, j, game),
        9302470 => return PalInstr_SetPalfxIndex(k, j, game),
        _ => {
            Unreachable(game);
            return PairU16 {
                k: 0 as uint16,
                j: 0 as uint16,
            };
        }
    };
}
pub unsafe fn CallPalfxFunc(mut ea: uint32, mut k: uint16, mut j: uint16, game: &mut Game) {
    match ea {
        9291397 => return,
        9298436 => {
            PalInit_E1BC(k, j, game);
            return;
        }
        9299008 => {
            PalInit_F761_Norfair1(k, j, game);
            return;
        }
        9303856 => {
            PalInit_F779_Brinstar8(k, j, game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
