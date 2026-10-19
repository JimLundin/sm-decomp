pub use crate::game::constants::bank_86::*;
use crate::game::sm_80::{
    CheckBossBitForCurArea, CheckEventHappened, QueueMusic_Delayed8, QueueSfx1_Max6,
    QueueSfx2_Max6, QueueSfx3_Max6, SetBossBitForCurArea, SetEventHappened,
};
use crate::game::sm_86::SpawnEprojWithRoomGfx;
use crate::game::sm_8d::SpawnPalfxObject;
use crate::game::sm_90::CallSomeSamusCode;
use crate::game_state::Game;
use crate::rtl::{RomPtr, Unreachable};

pub use crate::types::{
    __int8_t, __uint16_t, __uint32_t, __uint8_t, int8, int8_t, uint16, uint16_t, uint32, uint32_t,
    uint8, uint8_t, Consts_86,
};
#[inline]
unsafe fn RomPtr_87(mut addr: uint16_t, game: &mut Game) -> *const uint8 {
    RomPtr((0x870000u32) | ((addr) as u32), game)
}
pub const fnAnimtilesInstr_Delete: uint32 = 8880306;
pub const fnAnimtilesInstr_Goto: uint32 = 8880311;
pub const fnAnimtilesInstr_GotoRel: uint32 = 8880316;
pub const fnAnimtilesInstr_DecrementTimerAndGoto: uint32 = 8880340;
pub const fnAnimtilesInstr_DecrementTimerAndGotoRel: uint32 = 8880348;
pub const fnAnimtilesInstr_SetTimer: uint32 = 8880355;
pub const fnAnimtilesInstr_QueueMusic: uint32 = 8880368;
pub const fnAnimtilesInstr_QueueSfx1: uint32 = 8880380;
pub const fnAnimtilesInstr_QueueSfx2: uint32 = 8880392;
pub const fnAnimtilesInstr_QueueSfx3: uint32 = 8880404;
pub const fnAnimtilesInstr_GotoIfBossBitSet: uint32 = 8880416;
pub const fnAnimtilesInstr_SetBossBit: uint32 = 8880435;
pub const fnAnimtilesInstr_GotoIfEventHappened: uint32 = 8880447;
pub const fnAnimtilesInstr_SetEventHappened: uint32 = 8880464;
pub const fnAnimtilesInstr_WaitUntilAreaBossDead_DoubleRet: uint32 = 8880570;
pub const fnAnimtilesInstr_GotoIfBossBitSetInArea: uint32 = 8880899;
pub const fnAnimtilesInstr_SpawnTourianStatueEyeGlow: uint32 = 8880928;
pub const fnAnimtilesInstr_SpawnTourianStatueSoul: uint32 = 8880943;
pub const fnAnimtilesInstr_GotoIfTourianStatueBusy: uint32 = 8880958;
pub const fnAnimtilesInstr_TourianStatueSetState: uint32 = 8880969;
pub const fnAnimtilesInstr_TourianStatueClearState: uint32 = 8880978;
pub const fnAnimtilesInstr_Clear3PaletteColors: uint32 = 8880987;
pub const fnAnimtilesInstr_SpawnPalfxObj: uint32 = 8881010;
pub const fnAnimtilesInstr_Write8PaletteColors: uint32 = 8881023;
pub unsafe fn EnableAnimtiles(game: &mut Game) {
    let ref mut fresh9 = *((&raw mut game.g_ram as *mut uint8).offset(0x1ef1) as *mut uint16);
    *fresh9 = (*fresh9) | (0x8000u16);
}
pub unsafe fn DisableAnimtiles(game: &mut Game) {
    let ref mut fresh8 = *((&raw mut game.g_ram as *mut uint8).offset(0x1ef1) as *mut uint16);
    *fresh8 = (*fresh8 as ::core::ffi::c_int & !(0x8000 as ::core::ffi::c_int)) as uint16;
}
pub unsafe fn ClearAnimtiles(game: &mut Game) {
    let mut i: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
    while i >= 0i32 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1ef5) as *mut uint16)
            .offset((i >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
        i -= 2 as ::core::ffi::c_int;
    }
}
pub unsafe fn SpawnAnimtiles(mut j: uint16, game: &mut Game) {
    let mut v0: uint16 = j;
    let mut v1: uint16 = 10 as uint16;
    while *((&raw mut game.g_ram as *mut uint8).offset(0x1ef5) as *mut uint16)
        .offset((v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize)
        != 0
    {
        v1 = (v1).wrapping_sub(2u16);
        if (v1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) != 0i32 {
            return;
        }
    }
    let mut v2: ::core::ffi::c_int = v1 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ef5) as *mut uint16).offset(v2 as isize) = v0;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1f01) as *mut uint16).offset(v2 as isize) =
        0 as uint16;
    let mut v3: *const uint16 = RomPtr_87(v0, game) as *const uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1f0d) as *mut uint16).offset(v2 as isize) = *v3;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1f25) as *mut uint16).offset(v2 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x1f31) as *mut uint16).offset(v2 as isize) =
        *v3.offset(1);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1f3d) as *mut uint16).offset(v2 as isize) =
        *v3.offset(2);
    *((&raw mut game.g_ram as *mut uint8).offset(0x1f19) as *mut uint16).offset(v2 as isize) =
        1 as uint16;
}
pub unsafe fn AnimtilesHandler(game: &mut Game) {
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x1ef1) as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int)
        != 0i32
    {
        let mut i: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
        while i >= 0i32 {
            *((&raw mut game.g_ram as *mut uint8).offset(0x1ef3) as *mut uint16) = i as uint16;
            if *((&raw mut game.g_ram as *mut uint8).offset(0x1ef5) as *mut uint16)
                .offset((i >> 1 as ::core::ffi::c_int) as isize)
                != 0
            {
                ProcessAnimtilesObject(game);
                i = *((&raw mut game.g_ram as *mut uint8).offset(0x1ef3) as *mut uint16)
                    as ::core::ffi::c_int;
            }
            i -= 2 as ::core::ffi::c_int;
        }
    }
}
pub unsafe fn CallAnimtilesInstr(
    mut ea: uint32,
    mut j: uint16,
    mut k: uint16,
    game: &mut Game,
) -> uint16 {
    match ea {
        8880306 => AnimtilesInstr_Delete(j, k, game),
        8880311 => AnimtilesInstr_Goto(j, k, game),
        8880316 => AnimtilesInstr_GotoRel(j, k, game),
        8880340 => AnimtilesInstr_DecrementTimerAndGoto(j, k, game),
        8880348 => AnimtilesInstr_DecrementTimerAndGotoRel(j, k, game),
        8880355 => AnimtilesInstr_SetTimer(j, k, game),
        8880368 => AnimtilesInstr_QueueMusic(j, k, game),
        8880380 => AnimtilesInstr_QueueSfx1(j, k, game),
        8880392 => AnimtilesInstr_QueueSfx2(j, k, game),
        8880404 => AnimtilesInstr_QueueSfx3(j, k, game),
        8880416 => AnimtilesInstr_GotoIfBossBitSet(j, k, game),
        8880435 => AnimtilesInstr_SetBossBit(j, k, game),
        8880447 => AnimtilesInstr_GotoIfEventHappened(j, k, game),
        8880464 => AnimtilesInstr_SetEventHappened(j, k, game),
        8880570 => AnimtilesInstr_WaitUntilAreaBossDead_DoubleRet(j, k, game),
        8880899 => AnimtilesInstr_GotoIfBossBitSetInArea(j, k, game),
        8880928 => AnimtilesInstr_SpawnTourianStatueEyeGlow(j, k, game),
        8880943 => AnimtilesInstr_SpawnTourianStatueSoul(j, k, game),
        8880958 => AnimtilesInstr_GotoIfTourianStatueBusy(j, k, game),
        8880969 => AnimtilesInstr_TourianStatueSetState(j, k, game),
        8880978 => AnimtilesInstr_TourianStatueClearState(j, k, game),
        8880987 => AnimtilesInstr_Clear3PaletteColors(j, k, game),
        8881010 => AnimtilesInstr_SpawnPalfxObj(j, k, game),
        8881023 => AnimtilesInstr_Write8PaletteColors(j, k, game),
        _ => Unreachable(game) as uint16,
    }
}
pub unsafe fn ProcessAnimtilesObject(game: &mut Game) {
    let mut v0: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x1ef3) as *mut uint16);
    let mut v1: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x1ef3)
        as *mut uint16) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    let ref mut fresh6 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f19) as *mut uint16).offset(v1 as isize);
    let fresh7 = *fresh6;
    *fresh6 = (*fresh6).wrapping_sub(1);
    if fresh7 == 1u16 {
        let mut v3: uint16 = *((&raw mut game.g_ram as *mut uint8).offset(0x1f0d) as *mut uint16)
            .offset(v1 as isize);
        let mut v5: uint16 = 0;
        loop {
            let mut v4: *const uint16 = RomPtr_87(v3, game) as *const uint16;
            v5 = *v4;
            if (*v4 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int) == 0i32 {
                break;
            }
            *((&raw mut game.g_ram as *mut uint8).offset(0x1f49) as *mut uint16) = *v4;
            v3 = CallAnimtilesInstr(
                ((v5) as u32) | (0x870000u32),
                v0,
                (v3).wrapping_add(2u16),
                game,
            );
            if v3 == 0 {
                return;
            }
        }
        let mut v6: ::core::ffi::c_int = v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f19) as *mut uint16).offset(v6 as isize) =
            v5;
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f25) as *mut uint16).offset(v6 as isize) =
            *(RomPtr_87(v3, game) as *mut uint16).offset(1);
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f0d) as *mut uint16).offset(v6 as isize) =
            (v3).wrapping_add(4u16);
    }
}
pub unsafe fn AnimtilesInstr_Delete(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1ef5) as *mut uint16)
        .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    0 as uint16
}
pub unsafe fn AnimtilesInstr_Goto(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    *(RomPtr_87(j, game) as *mut uint16)
}
pub unsafe fn AnimtilesInstr_GotoRel(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1f49) as *mut uint16) = j;
    (j).wrapping_add((*RomPtr_87(j, game) as int8) as u16)
}
pub unsafe fn AnimtilesInstr_DecrementTimerAndGoto(
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    let mut v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh0 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f01) as *mut uint16).offset(v2 as isize);
    let fresh1 = *fresh0;
    *fresh0 = (*fresh0).wrapping_sub(1);
    if fresh1 == 1u16 {
        (j).wrapping_add(2u16)
    } else {
        AnimtilesInstr_Goto(k, j, game)
    }
}
pub unsafe fn AnimtilesInstr_DecrementTimerAndGotoRel(
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    let mut v2: ::core::ffi::c_int = k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    let ref mut fresh2 =
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f01) as *mut uint16).offset(v2 as isize);
    let fresh3 = *fresh2;
    *fresh2 = (*fresh2).wrapping_sub(1);
    if fresh3 == 1u16 {
        (j).wrapping_add(1u16)
    } else {
        AnimtilesInstr_GotoRel(k, j, game)
    }
}
pub unsafe fn AnimtilesInstr_SetTimer(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    *((&raw mut game.g_ram as *mut uint8).offset(0x1f01) as *mut uint16 as *mut uint8)
        .offset(k as ::core::ffi::c_int as isize) = *RomPtr_87(j, game);
    (j).wrapping_add(1u16)
}
pub unsafe fn AnimtilesInstr_QueueMusic(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    let mut v2: *const uint8 = RomPtr_87(j, game);
    QueueMusic_Delayed8(*v2 as uint16, game);
    (j).wrapping_add(1u16)
}
pub unsafe fn AnimtilesInstr_QueueSfx1(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    let mut v2: *const uint8 = RomPtr_87(j, game);
    QueueSfx1_Max6(*v2 as uint16, game);
    (j).wrapping_add(1u16)
}
pub unsafe fn AnimtilesInstr_QueueSfx2(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    let mut v2: *const uint8 = RomPtr_87(j, game);
    QueueSfx2_Max6(*v2 as uint16, game);
    (j).wrapping_add(1u16)
}
pub unsafe fn AnimtilesInstr_QueueSfx3(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    let mut v2: *const uint8 = RomPtr_87(j, game);
    QueueSfx3_Max6(*v2 as uint16, game);
    (j).wrapping_add(1u16)
}
pub unsafe fn AnimtilesInstr_GotoIfBossBitSet(
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    let mut v2: *const uint8 = RomPtr_87(j, game);
    let mut v3: uint16 = (j).wrapping_add(1u16);
    if CheckBossBitForCurArea(*(v2 as *mut uint16) as uint8 as uint16, game) as ::core::ffi::c_int
        & 1 as ::core::ffi::c_int
        != 0
    {
        AnimtilesInstr_Goto(k, v3, game)
    } else {
        (v3).wrapping_add(2u16)
    }
}
pub unsafe fn AnimtilesInstr_SetBossBit(mut k: uint16, mut j: uint16, game: &mut Game) -> uint16 {
    let mut v2: *const uint8 = RomPtr_87(j, game);
    SetBossBitForCurArea(*v2 as uint16, game);
    (j).wrapping_add(1u16)
}
pub unsafe fn AnimtilesInstr_GotoIfEventHappened(
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    let mut v2: *const uint16 = RomPtr_87(j, game) as *const uint16;
    let mut v3: uint16 = (j).wrapping_add(2u16);
    if CheckEventHappened(*v2, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        AnimtilesInstr_Goto(k, v3, game)
    } else {
        (v3).wrapping_add(2u16)
    }
}
pub unsafe fn AnimtilesInstr_SetEventHappened(
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    let mut v2: *const uint16 = RomPtr_87(j, game) as *const uint16;
    SetEventHappened(*v2, game);
    (j).wrapping_add(2u16)
}
pub unsafe fn UNUSED_sub_87815A(game: &mut Game) {
    CallSomeSamusCode(0 as uint16, game);
}
pub unsafe fn UNUSED_sub_878162(game: &mut Game) {
    CallSomeSamusCode(1 as uint16, game);
}
pub unsafe fn AnimtilesInstr_WaitUntilAreaBossDead_DoubleRet(
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    if CheckBossBitForCurArea(1 as uint16, game) as ::core::ffi::c_int & 1 as ::core::ffi::c_int
        == 0
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0x1f19) as *mut uint16)
            .offset((k as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) = 1 as uint16;
        return 0 as uint16;
    }
    j
}
pub unsafe fn AnimtilesInstr_GotoIfBossBitSetInArea(
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    let mut v2: *const uint8 = RomPtr_87(j, game);
    let mut v3: uint16 = (j).wrapping_add(2u16);
    if (*v2 as ::core::ffi::c_int
        & *(&raw mut game.g_ram as *mut uint8)
            .offset(0xd828)
            .offset(*v2.offset(1) as isize) as ::core::ffi::c_int)
        != 0i32
    {
        AnimtilesInstr_Goto(k, v3, game)
    } else {
        (v3).wrapping_add(2u16)
    }
}
pub unsafe fn AnimtilesInstr_SpawnTourianStatueEyeGlow(
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    let mut v2: *const uint16 = RomPtr_87(j, game) as *const uint16;
    SpawnEprojWithRoomGfx(addr_kEproj_TourianStatueEyeGlow as uint16, *v2, game);
    (j).wrapping_add(2u16)
}
pub unsafe fn AnimtilesInstr_SpawnTourianStatueSoul(
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    let mut v2: *const uint16 = RomPtr_87(j, game) as *const uint16;
    SpawnEprojWithRoomGfx(addr_kEproj_TourianStatueSoul as uint16, *v2, game);
    (j).wrapping_add(2u16)
}
pub unsafe fn AnimtilesInstr_GotoIfTourianStatueBusy(
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    if (*((&raw mut game.g_ram as *mut uint8).offset(0x1e6f) as *mut uint16) as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int)
        == 0i32
    {
        (j).wrapping_add(2u16)
    } else {
        AnimtilesInstr_Goto(k, j, game)
    }
}
pub unsafe fn AnimtilesInstr_TourianStatueSetState(
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    let ref mut fresh5 = *((&raw mut game.g_ram as *mut uint8).offset(0x1e6f) as *mut uint16);
    *fresh5 = (*fresh5) | (*(RomPtr_87(j, game) as *mut uint16));
    (j).wrapping_add(2u16)
}
pub unsafe fn AnimtilesInstr_TourianStatueClearState(
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    let ref mut fresh4 = *((&raw mut game.g_ram as *mut uint8).offset(0x1e6f) as *mut uint16);
    *fresh4 = (*fresh4 as ::core::ffi::c_int
        & !(*(RomPtr_87(j, game) as *mut uint16) as ::core::ffi::c_int)) as uint16;
    (j).wrapping_add(2u16)
}
pub unsafe fn AnimtilesInstr_Clear3PaletteColors(
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    let mut v2: ::core::ffi::c_int =
        *(RomPtr_87(j, game) as *mut uint16) as ::core::ffi::c_int >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16).offset(v2 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
        .offset((v2 + 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xc000) as *mut uint16)
        .offset((v2 + 2 as ::core::ffi::c_int) as isize) = 0 as uint16;
    (j).wrapping_add(2u16)
}
pub unsafe fn AnimtilesInstr_SpawnPalfxObj(
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    let mut v2: *const uint16 = RomPtr_87(j, game) as *const uint16;
    SpawnPalfxObject(*v2, game);
    (j).wrapping_add(2u16)
}
pub unsafe fn AnimtilesInstr_Write8PaletteColors(
    mut k: uint16,
    mut j: uint16,
    game: &mut Game,
) -> uint16 {
    static kAnimtilesInstr_Write8PaletteColors: [uint16; 8] = [
        0x3800, 0x7f58, 0x6ed5, 0x5a71, 0x49ee, 0x356a, 0x24e7, 0x1083,
    ];
    let mut v2: uint16 = *(RomPtr_87(j, game) as *mut uint16);
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i != 16i32 {
        *((&raw mut game.g_ram as *mut uint8).offset(0xc200) as *mut uint16)
            .offset((v2 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize) =
            kAnimtilesInstr_Write8PaletteColors[(i >> 1 as ::core::ffi::c_int) as usize];
        v2 = (v2).wrapping_add(2u16);
        i += 2 as ::core::ffi::c_int;
    }
    (j).wrapping_add(2u16)
}
