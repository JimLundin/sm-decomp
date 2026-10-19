pub use crate::game::constants::bank_86::*;
pub use crate::game::constants::enemy_properties::*;
use crate::game::sm_80::QueueSfx2_Max6;
use crate::game::sm_84::SpawnHardcodedPlm;
use crate::game::sm_86::SpawnEprojWithRoomGfx;
use crate::game::sm_a9::WriteColorsToPalette;
use crate::game_state::Game;
use crate::ram_map::set_ram_word;
use crate::rtl::RomPtr;
extern "C" {
    fn printf(__format: *const ::core::ffi::c_char, ...) -> ::core::ffi::c_int;

}
pub use crate::types::Consts_86;
pub use crate::types::Consts_AD;
pub use crate::types::{
    __uint16_t, __uint32_t, __uint8_t, uint16, uint16_t, uint32_t, uint8, uint8_t, EnemyBase,
    EnemyData, EnemyProps, Func_V, Point16U, VoidP,
};
pub const addr_kTurnOffLightsForShitroidDeath: Consts_AD = 61999;
pub use crate::types::{Enemy_MotherBrain, Func_CalcHdma, Ram8000_Default, SpawnHardcodedPlmArgs};
#[inline]
unsafe fn RomPtr_AD(addr: uint16_t, game: &mut Game) -> *const uint8 {
    RomPtr((0xad0000u32) | ((addr) as u32), game)
}
pub const fnMotherBrainBody_FakeDeath_Descent_0_Pause: ::core::ffi::c_int = 0xa9881d;
#[inline]
unsafe fn gEnemyData(o: ::core::ffi::c_int, game: &mut Game) -> *mut EnemyData {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut EnemyData
}
#[inline]
unsafe fn gRam8000_Default(o: ::core::ffi::c_int, game: &mut Game) -> *mut Ram8000_Default {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0x8000)
        .offset(o as isize) as *mut Ram8000_Default
}
#[inline]
unsafe fn Get_MotherBrain(o: ::core::ffi::c_int, game: &mut Game) -> *mut Enemy_MotherBrain {
    (&raw mut game.g_ram as *mut uint8)
        .offset(0xf78)
        .offset(o as isize) as *mut Enemy_MotherBrain
}
pub fn nullsub_341(game: &mut Game) {}
static funcs_BE56D: [Option<Func_V>; 16] = unsafe {
    [
        Some(MotherBrain_CalcHdma_Down as unsafe fn(&mut Game) -> ()),
        Some(MotherBrain_CalcHdma_BeamAimedRight2 as unsafe fn(&mut Game) -> ()),
        None,
        None,
        None,
        Some(MotherBrain_CalcHdma_BeamAimedUp as unsafe fn(&mut Game) -> ()),
        Some(MotherBrain_CalcHdma_BeamAimedUp as unsafe fn(&mut Game) -> ()),
        None,
        None,
        None,
        Some(MotherBrain_CalcHdma_BeamAimedUp as unsafe fn(&mut Game) -> ()),
        Some(nullsub_341 as unsafe fn(&mut Game) -> ()),
        Some(MotherBrain_CalcHdma_Down as unsafe fn(&mut Game) -> ()),
        None,
        None,
        Some(MotherBrain_CalcHdma_Down as unsafe fn(&mut Game) -> ()),
    ]
};
pub unsafe fn MotherBrain_CalcHdma(game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    let E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40 as ::core::ffi::c_int, game);
    let mut r18: uint16 = (*(&raw mut (*E).mbn_var_33 as *mut uint8).offset(
        (::core::mem::size_of::<uint16>())
            .wrapping_div(::core::mem::size_of::<uint8>())
            .wrapping_sub(1_usize) as isize,
    ) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int) as uint16;
    (*E).mbn_var_3A = ((*E).mbn_var_31).wrapping_sub(r18);
    (*E).mbn_var_3B = (r18).wrapping_add((*E).mbn_var_31);
    let v2: uint16 = (*((&raw mut (*E1).base.enemy_ptr as *mut uint8).offset(1) as *mut uint16)
        as ::core::ffi::c_int
        + 3584 as ::core::ffi::c_int
        & 0xff00 as ::core::ffi::c_int) as uint16;
    (*E).mbn_var_3C = v2;
    (*E).mbn_var_3E = v2;
    let v3: uint16 = ((*E1).base.y_pos).wrapping_add(5u16);
    (*E).mbn_var_3D = v3;
    (*E).mbn_var_3F = v3;
    r18 = (4 as ::core::ffi::c_int
        * ((*E).mbn_var_3A as ::core::ffi::c_int & 0xc0 as ::core::ffi::c_int)) as uint16;
    let i: ::core::ffi::c_int = (r18 as ::core::ffi::c_int
        | (*E).mbn_var_3B as ::core::ffi::c_int & 0xc0 as ::core::ffi::c_int)
        >> 6 as ::core::ffi::c_int;
    funcs_BE56D[i as usize].expect("non-null function pointer")(game);
}
pub unsafe fn MotherBrain_CalcHdma_BeamAimedRight2(game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    MotherBrain_CalcHdma_BeamAimedRight((*E).mbn_var_3C, (*E).mbn_var_3E, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x9c00) as *mut uint16) = 16 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9c00).offset(1) as *mut uint8 as *mut uint16) =
        -(25600 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9c00).offset(3) as *mut uint8 as *mut uint16) =
        16 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9c00).offset(4) as *mut uint8 as *mut uint16) =
        -(25597 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9c00).offset(6) as *mut uint8 as *mut uint16) =
        240 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9c00).offset(7) as *mut uint8 as *mut uint16) =
        -(25340 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9c00).offset(9) as *mut uint8 as *mut uint16) =
        244 as uint16;
    *((&raw mut game.g_ram as *mut uint8)
        .offset(0x9c00)
        .offset(10) as *mut uint8 as *mut uint16) = -(25108 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8)
        .offset(0x9c00)
        .offset(12) as *mut uint8 as *mut uint16) = 0 as uint16;
}
pub unsafe fn MotherBrain_CalcHdma_BeamAimedRight(
    mut r22: uint16,
    mut r24: uint16,
    game: &mut Game,
) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x9d00) as *mut uint16).offset(0) = 255 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9d00) as *mut uint16).offset(1) = 255 as uint16;
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    let r18: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0x91c9d4)) as *const uint8
        as *mut uint16)
        .offset(*(&raw mut (*E).mbn_var_3A as *mut uint8).offset(0) as isize);
    let r20: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0x91c9d4)) as *const uint8
        as *mut uint16)
        .offset(*(&raw mut (*E).mbn_var_3B as *mut uint8).offset(0) as isize);
    let mut mbn_var_3F: uint16 = (*E).mbn_var_3F;
    let mut v2: *mut uint16 = ((&raw mut game.g_ram as *mut uint8).offset(0x9d00) as *mut uint16)
        .offset((mbn_var_3F as ::core::ffi::c_int - 32 as ::core::ffi::c_int) as isize)
        .offset(1);
    *v2.offset(1) = 255 as uint16;
    *v2.offset(2) = 255 as uint16;
    let v8: *mut uint16 = v2;
    loop {
        let mut v4: uint16 = (r20).wrapping_add(r24);
        if u16::overflowing_add(r20, r24).1 {
            break;
        }
        r24 = (r24).wrapping_add(r20);
        let fresh11 = v2;
        v2 = v2.offset(-1);
        *fresh11 = ((*(&raw mut v4 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1_usize) as isize,
        )) as u16)
            | (0xff00u16);
        mbn_var_3F = mbn_var_3F.wrapping_sub(1);
        if (mbn_var_3F == 32u16) {
            break;
        }
    }
    loop {
        let fresh12 = v2;
        v2 = v2.offset(-1);
        *fresh12 = 255 as uint16;
        mbn_var_3F = mbn_var_3F.wrapping_sub(1);
        if (mbn_var_3F == 32u16) {
            break;
        }
    }
    let mut v5: *mut uint16 = v8.offset(1);
    let mut var_3F: uint16 = (*gRam8000_Default(0 as ::core::ffi::c_int, game)).var_3F;
    loop {
        let mut v7: uint16 = (r18).wrapping_add(r22);
        if u16::overflowing_add(r18, r22).1 {
            break;
        }
        r22 = (r22).wrapping_add(r18);
        let fresh13 = v5;
        v5 = v5.offset(1);
        *fresh13 = ((*(&raw mut v7 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1_usize) as isize,
        )) as u16)
            | (0xff00u16);
        var_3F = var_3F.wrapping_add(1);
        if var_3F == 232u16 {
            return;
        }
    }
    loop {
        let fresh14 = v5;
        v5 = v5.offset(1);
        *fresh14 = 255 as uint16;
        var_3F = var_3F.wrapping_add(1);
        if (var_3F == 232u16) {
            break;
        }
    }
}
static off_ADE024: [Option<Func_CalcHdma>; 4] = unsafe {
    [
        Some(MotherBrain_CalcHdma_Up_UpRight as unsafe fn(uint16, uint16, &mut Game) -> ()),
        Some(MotherBrain_CalcHdma_Up_Up as unsafe fn(uint16, uint16, &mut Game) -> ()),
        None,
        Some(MotherBrain_CalcHdma_Up_UpLeft as unsafe fn(uint16, uint16, &mut Game) -> ()),
    ]
};
pub unsafe fn MotherBrain_CalcHdma_BeamAimedUp(game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    off_ADE024[((*E).mbn_var_3A as ::core::ffi::c_int >> 6 as ::core::ffi::c_int
        & 2 as ::core::ffi::c_int
        | (*E).mbn_var_3B as ::core::ffi::c_int >> 7 as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int) as usize]
        .expect("non-null function pointer")((*E).mbn_var_3C, (*E).mbn_var_3E, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x9c00) as *mut uint16) = 16 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9c00).offset(1) as *mut uint8 as *mut uint16) =
        -(25344 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9c00).offset(3) as *mut uint8 as *mut uint16) =
        16 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9c00).offset(4) as *mut uint8 as *mut uint16) =
        -(25342 as ::core::ffi::c_int) as uint16;
    let v2: uint16 = ((*E).mbn_var_3D).wrapping_sub(32u16);
    printf(b"Carry crap!\n\0" as *const u8 as *const ::core::ffi::c_char);
    if v2 >= 0x80u16 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x9c00).offset(6) as *mut uint8
            as *mut uint16) = (v2 as ::core::ffi::c_int - 127 as ::core::ffi::c_int
            | 0x80 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x9c00).offset(7) as *mut uint8
            as *mut uint16) = -(25340 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x9c00).offset(9) as *mut uint8
            as *mut uint16) = 255 as uint16;
        *((&raw mut game.g_ram as *mut uint8)
            .offset(0x9c00)
            .offset(10) as *mut uint8 as *mut uint16) =
            (2 as ::core::ffi::c_int * (v2 as ::core::ffi::c_int - 127 as ::core::ffi::c_int)
                - 25340 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8)
            .offset(0x9c00)
            .offset(12) as *mut uint8 as *mut uint16) = 127 as uint16;
        *((&raw mut game.g_ram as *mut uint8)
            .offset(0x9c00)
            .offset(13) as *mut uint8 as *mut uint16) =
            (2 as ::core::ffi::c_int * (v2 as ::core::ffi::c_int - 127 as ::core::ffi::c_int)
                - 25340 as ::core::ffi::c_int
                + 254 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8)
            .offset(0x9c00)
            .offset(15) as *mut uint8 as *mut uint16) = 0 as uint16;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0x9c00).offset(6) as *mut uint8
            as *mut uint16) = (v2) | (0x80u16);
        *((&raw mut game.g_ram as *mut uint8).offset(0x9c00).offset(7) as *mut uint8
            as *mut uint16) = -(25340 as ::core::ffi::c_int) as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x9c00).offset(9) as *mut uint8
            as *mut uint16) = 127 as uint16;
        *((&raw mut game.g_ram as *mut uint8)
            .offset(0x9c00)
            .offset(10) as *mut uint8 as *mut uint16) =
            (2 as ::core::ffi::c_int * v2 as ::core::ffi::c_int - 25340 as ::core::ffi::c_int)
                as uint16;
        *((&raw mut game.g_ram as *mut uint8)
            .offset(0x9c00)
            .offset(12) as *mut uint8 as *mut uint16) = 0 as uint16;
    };
}
pub unsafe fn MotherBrain_CalcHdma_Up_UpRight(mut r22: uint16, mut r24: uint16, game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x9d00) as *mut uint16).offset(0) = 255 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9d00) as *mut uint16).offset(1) = 255 as uint16;
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    let r18: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0x91c9d4)) as *const uint8
        as *mut uint16)
        .offset(*(&raw mut (*E).mbn_var_3B as *mut uint8).offset(0) as isize);
    let r20: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0x91c9d4)) as *const uint8
        as *mut uint16)
        .offset(*(&raw mut (*E).mbn_var_3A as *mut uint8).offset(0) as isize);
    let mut mbn_var_3D: uint16 = (*E).mbn_var_3D;
    let mut v2: *mut uint16 = ((&raw mut game.g_ram as *mut uint8).offset(0x9d00) as *mut uint16)
        .offset((mbn_var_3D as ::core::ffi::c_int - 32 as ::core::ffi::c_int) as isize)
        .offset(1);
    *v2.offset(1) = 255 as uint16;
    *v2.offset(2) = 255 as uint16;
    loop {
        let mut v4: uint16 = (r18).wrapping_add(r22);
        if u16::overflowing_add(r18, r22).1 {
            v4 = -(1 as ::core::ffi::c_int) as uint16;
        }
        r22 = v4;
        let R36: uint16 = (v4 as ::core::ffi::c_int >> 8 as ::core::ffi::c_int
            & 0xff as ::core::ffi::c_int) as uint16;
        let mut v7: uint16 = (r20).wrapping_add(r24);
        if u16::overflowing_add(r20, r24).1 {
            v7 = -(1 as ::core::ffi::c_int) as uint16;
        }
        r24 = v7;
        let mut v8: uint16 = (R36 as ::core::ffi::c_int
            | v7 as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int)
            as uint16;
        if v8 == 0xffffu16 {
            v8 = 255 as uint16;
        }
        let fresh10 = v2;
        v2 = v2.offset(-1);
        *fresh10 = v8;
        mbn_var_3D = mbn_var_3D.wrapping_sub(1);
        if (mbn_var_3D == 32u16) {
            break;
        }
    }
}
pub unsafe fn MotherBrain_CalcHdma_Up_Up(mut r22: uint16, mut r24: uint16, game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x9d00) as *mut uint16).offset(0) = 255 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9d00) as *mut uint16).offset(1) = 255 as uint16;
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    let r18: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0x91c9d4)) as *const uint8
        as *mut uint16)
        .offset(
            -(*(&raw mut (*E).mbn_var_3B as *mut uint8).offset(0) as ::core::ffi::c_int) as uint8
                as isize,
        );
    let r20: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0x91c9d4)) as *const uint8
        as *mut uint16)
        .offset(*(&raw mut (*E).mbn_var_3A as *mut uint8).offset(0) as isize);
    let mut mbn_var_3D: uint16 = (*E).mbn_var_3D;
    let mut v2: *mut uint16 = ((&raw mut game.g_ram as *mut uint8).offset(0x9d00) as *mut uint16)
        .offset((mbn_var_3D as ::core::ffi::c_int - 32 as ::core::ffi::c_int) as isize)
        .offset(1);
    *v2.offset(1) = 255 as uint16;
    *v2.offset(2) = 255 as uint16;
    loop {
        let mut v4: uint16 = (r22).wrapping_sub(r18);
        if r22 < r18 {
            v4 = 0 as uint16;
        }
        r22 = v4;
        let r26: uint16 = (v4 as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as uint8 as uint16;
        let mut v7: uint16 = (r20).wrapping_add(r24);
        if u16::overflowing_add(r20, r24).1 {
            v7 = -(1 as ::core::ffi::c_int) as uint16;
        }
        r24 = v7;
        let mut v8: uint16 = (r26 as ::core::ffi::c_int
            | v7 as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int)
            as uint16;
        if v8 == 0xffffu16 {
            v8 = 255 as uint16;
        }
        let fresh9 = v2;
        v2 = v2.offset(-1);
        *fresh9 = v8;
        mbn_var_3D = mbn_var_3D.wrapping_sub(1);
        if (mbn_var_3D == 32u16) {
            break;
        }
    }
}
pub unsafe fn MotherBrain_CalcHdma_Up_UpLeft(mut r22: uint16, mut r24: uint16, game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x9d00) as *mut uint16).offset(0) = 255 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9d00) as *mut uint16).offset(1) = 255 as uint16;
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    let r18: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0x91c9d4)) as *const uint8
        as *mut uint16)
        .offset(
            -(*(&raw mut (*E).mbn_var_3B as *mut uint8).offset(0) as ::core::ffi::c_int) as uint8
                as isize,
        );
    let r20: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0x91c9d4)) as *const uint8
        as *mut uint16)
        .offset(
            -(*(&raw mut (*E).mbn_var_3A as *mut uint8).offset(0) as ::core::ffi::c_int) as uint8
                as isize,
        );
    let mut mbn_var_3D: uint16 = (*E).mbn_var_3D;
    let mut v2: *mut uint16 = ((&raw mut game.g_ram as *mut uint8).offset(0x9d00) as *mut uint16)
        .offset((mbn_var_3D as ::core::ffi::c_int - 32 as ::core::ffi::c_int) as isize)
        .offset(1);
    *v2.offset(1) = 255 as uint16;
    *v2.offset(2) = 255 as uint16;
    loop {
        let mut v4: uint16 = (r22).wrapping_sub(r18);
        if r22 < r18 {
            v4 = 0 as uint16;
        }
        r22 = v4;
        let R36: uint16 = (v4 as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as uint16;
        let mut v7: uint16 = (r24).wrapping_sub(r20);
        if r24 < r20 {
            v7 = 0 as uint16;
        }
        r24 = v7;
        let mut v8: uint16 = (R36 as ::core::ffi::c_int
            | v7 as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int)
            as uint16;
        if v8 == 0xffffu16 {
            v8 = 255 as uint16;
        }
        let fresh8 = v2;
        v2 = v2.offset(-1);
        *fresh8 = v8;
        mbn_var_3D = mbn_var_3D.wrapping_sub(1);
        if (mbn_var_3D == 32u16) {
            break;
        }
    }
}
static g_off_ADE20E: [Option<Func_CalcHdma>; 4] = unsafe {
    [
        Some(MotherBrain_CalcHdma_Down_DownRight as unsafe fn(uint16, uint16, &mut Game) -> ()),
        None,
        Some(MotherBrain_CalcHdma_Down_Down as unsafe fn(uint16, uint16, &mut Game) -> ()),
        Some(MotherBrain_CalcHdma_Down_DownLeft as unsafe fn(uint16, uint16, &mut Game) -> ()),
    ]
};
pub unsafe fn MotherBrain_CalcHdma_Down(game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    g_off_ADE20E[((*E).mbn_var_3A as ::core::ffi::c_int >> 6 as ::core::ffi::c_int
        & 2 as ::core::ffi::c_int
        | (*E).mbn_var_3B as ::core::ffi::c_int >> 7 as ::core::ffi::c_int
            & 1 as ::core::ffi::c_int) as usize]
        .expect("non-null function pointer")((*E).mbn_var_3C, (*E).mbn_var_3E, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x9c00) as *mut uint16) = 16 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9c00).offset(1) as *mut uint8 as *mut uint16) =
        -(25600 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9c00).offset(3) as *mut uint8 as *mut uint16) =
        16 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9c00).offset(4) as *mut uint8 as *mut uint16) =
        -(25597 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9c00).offset(6) as *mut uint8 as *mut uint16) =
        240 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9c00).offset(7) as *mut uint8 as *mut uint16) =
        -(25340 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9c00).offset(9) as *mut uint8 as *mut uint16) =
        244 as uint16;
    *((&raw mut game.g_ram as *mut uint8)
        .offset(0x9c00)
        .offset(10) as *mut uint8 as *mut uint16) = -(25114 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8)
        .offset(0x9c00)
        .offset(12) as *mut uint8 as *mut uint16) = 0 as uint16;
}
pub unsafe fn MotherBrain_CalcHdma_Down_DownRight(
    mut r22: uint16,
    mut r24: uint16,
    game: &mut Game,
) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x9d00) as *mut uint16).offset(0) = 255 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9d00) as *mut uint16).offset(1) = 255 as uint16;
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    let r18: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0x91c9d4)) as *const uint8
        as *mut uint16)
        .offset(*(&raw mut (*E).mbn_var_3A as *mut uint8).offset(0) as isize);
    let r20: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0x91c9d4)) as *const uint8
        as *mut uint16)
        .offset(*(&raw mut (*E).mbn_var_3B as *mut uint8).offset(0) as isize);
    let mut v1: uint16 = ((*E).mbn_var_3D).wrapping_sub(32u16);
    let mut dst: *mut uint16 =
        ((&raw mut game.g_ram as *mut uint8).offset(0x9d00) as *mut uint16).offset(2);
    loop {
        let fresh6 = dst;
        dst = dst.offset(1);
        *fresh6 = 255 as uint16;
        v1 = v1.wrapping_sub(1);
        if (v1 == 0) {
            break;
        }
    }
    let mut mbn_var_3D: uint16 = (*Get_MotherBrain(0 as ::core::ffi::c_int, game)).mbn_var_3D;
    loop {
        let mut v4: uint16 = (r18).wrapping_add(r22);
        if u16::overflowing_add(r18, r22).1 {
            v4 = -(1 as ::core::ffi::c_int) as uint16;
        }
        r22 = v4;
        let r26: uint16 = (v4 as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as uint16;
        let mut v7: uint16 = (r20).wrapping_add(r24);
        if u16::overflowing_add(r20, r24).1 {
            v7 = -(1 as ::core::ffi::c_int) as uint16;
        }
        r24 = v7;
        let mut v8: uint16 = (r26 as ::core::ffi::c_int
            | v7 as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int)
            as uint16;
        if v8 == 0xffffu16 {
            v8 = 255 as uint16;
        }
        let fresh7 = dst;
        dst = dst.offset(1);
        *fresh7 = v8;
        mbn_var_3D = mbn_var_3D.wrapping_add(1);
        if (mbn_var_3D == 232u16) {
            break;
        }
    }
}
pub unsafe fn MotherBrain_CalcHdma_Down_Down(mut r22: uint16, mut r24: uint16, game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x9d00) as *mut uint16).offset(0) = 255 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9d00) as *mut uint16).offset(1) = 255 as uint16;
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    let r18: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0x91c9d4)) as *const uint8
        as *mut uint16)
        .offset(
            -(*(&raw mut (*E).mbn_var_3A as *mut uint8).offset(0) as ::core::ffi::c_int) as uint8
                as isize,
        );
    let r20: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0x91c9d4)) as *const uint8
        as *mut uint16)
        .offset(*(&raw mut (*E).mbn_var_3B as *mut uint8).offset(0) as isize);
    let mut v1: uint16 = ((*E).mbn_var_3D).wrapping_sub(32u16);
    let mut dst: *mut uint16 =
        ((&raw mut game.g_ram as *mut uint8).offset(0x9d00) as *mut uint16).offset(2);
    loop {
        let fresh4 = dst;
        dst = dst.offset(1);
        *fresh4 = 255 as uint16;
        v1 = v1.wrapping_sub(1);
        if (v1 == 0) {
            break;
        }
    }
    let mut mbn_var_3D: uint16 = (*Get_MotherBrain(0 as ::core::ffi::c_int, game)).mbn_var_3D;
    loop {
        let mut v4: uint16 = (r22).wrapping_sub(r18);
        if r22 < r18 {
            v4 = 0 as uint16;
        }
        r22 = v4;
        let r26: uint16 = (v4 as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as uint16;
        let mut v7: uint16 = (r20).wrapping_add(r24);
        if u16::overflowing_add(r20, r24).1 {
            v7 = -(1 as ::core::ffi::c_int) as uint16;
        }
        r24 = v7;
        let mut v8: uint16 = (r26 as ::core::ffi::c_int
            | v7 as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int)
            as uint16;
        if v8 == 0xffffu16 {
            v8 = 255 as uint16;
        }
        let fresh5 = dst;
        dst = dst.offset(1);
        *fresh5 = v8;
        mbn_var_3D = mbn_var_3D.wrapping_add(1);
        if (mbn_var_3D == 232u16) {
            break;
        }
    }
}
pub unsafe fn MotherBrain_CalcHdma_Down_DownLeft(
    mut r22: uint16,
    mut r24: uint16,
    game: &mut Game,
) {
    *((&raw mut game.g_ram as *mut uint8).offset(0x9d00) as *mut uint16).offset(0) = 255 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x9d00) as *mut uint16).offset(1) = 255 as uint16;
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0 as ::core::ffi::c_int, game);
    let r18: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0x91c9d4)) as *const uint8
        as *mut uint16)
        .offset(
            -(*(&raw mut (*E).mbn_var_3A as *mut uint8).offset(0) as ::core::ffi::c_int) as uint8
                as isize,
        );
    let r20: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0x91c9d4)) as *const uint8
        as *mut uint16)
        .offset(
            -(*(&raw mut (*E).mbn_var_3B as *mut uint8).offset(0) as ::core::ffi::c_int) as uint8
                as isize,
        );
    let mut v1: uint16 = ((*E).mbn_var_3D).wrapping_sub(32u16);
    let mut dst: *mut uint16 =
        ((&raw mut game.g_ram as *mut uint8).offset(0x9d00) as *mut uint16).offset(2);
    loop {
        let fresh2 = dst;
        dst = dst.offset(1);
        *fresh2 = 255 as uint16;
        v1 = v1.wrapping_sub(1);
        if (v1 == 0) {
            break;
        }
    }
    let mut mbn_var_3D: uint16 = (*Get_MotherBrain(0 as ::core::ffi::c_int, game)).mbn_var_3D;
    loop {
        let mut v4: uint16 = (r22).wrapping_sub(r18);
        if r22 < r18 {
            v4 = 0 as uint16;
        }
        r22 = v4;
        let v5: uint16 = (v4 as ::core::ffi::c_int >> 8 as ::core::ffi::c_int) as uint16;
        let r26: uint16 = v5;
        let mut v7: uint16 = (r24).wrapping_sub(r20);
        if r24 < r20 {
            v7 = 0 as uint16;
        }
        r24 = v7;
        let mut v8: uint16 = (r26 as ::core::ffi::c_int
            | v7 as ::core::ffi::c_int & 0xff00 as ::core::ffi::c_int)
            as uint16;
        if v8 == 0 {
            v8 = 255 as uint16;
        }
        let fresh3 = dst;
        dst = dst.offset(1);
        *fresh3 = v8;
        mbn_var_3D = mbn_var_3D.wrapping_add(1);
        if (mbn_var_3D == 232u16) {
            break;
        }
    }
}
pub unsafe fn MotherBrain_SealWall(game: &mut Game) {
    game.eproj_spawn_pt = Point16U {
        x: 248 as uint16,
        y: 72 as uint16,
    };
    SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as uint16, 9 as uint16, game);
    game.eproj_spawn_pt = Point16U {
        x: 248 as uint16,
        y: 152 as uint16,
    };
    SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as uint16, 9 as uint16, game);
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs {
            field_0: 0xf as uint8,
            field_1: 0x4 as uint8,
            field_2: 0xb673 as VoidP,
        },
        game,
    );
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs {
            field_0: 0xf as uint8,
            field_1: 0x9 as uint8,
            field_2: 0xb673 as VoidP,
        },
        game,
    );
    (*Get_MotherBrain(0 as ::core::ffi::c_int, game)).mbn_var_A = ((0xa9881di32) as u16);
}
pub unsafe fn MotherBrain_HealthBasedPaletteHandling(game: &mut Game) {
    if (*Get_MotherBrain(0 as ::core::ffi::c_int, game)).mbn_var_1F >= 2u16 {
        let mut v0: uint16 = 0 as uint16;
        let health: uint16 = (*Get_MotherBrain(0x40 as ::core::ffi::c_int, game))
            .base
            .health;
        if health < 0x2328u16 {
            v0 = 2 as uint16;
            if health < 0x1518u16 {
                v0 = 4 as uint16;
                if health < 0x708u16 {
                    v0 = 6 as uint16;
                }
            }
        }
        WriteColorsToPalette(
            0x82 as uint16,
            0xad as uint8,
            *(game.g_rom.offset(crate::rtl::lorom_offset(0xade6a2)) as *const uint8 as *mut uint16)
                .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize),
            0xf as uint16,
            game,
        );
        WriteColorsToPalette(
            0x122 as uint16,
            0xad as uint8,
            *(game.g_rom.offset(crate::rtl::lorom_offset(0xade6a2)) as *const uint8 as *mut uint16)
                .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize),
            0xf as uint16,
            game,
        );
        WriteColorsToPalette(
            0x162 as uint16,
            0xad as uint8,
            *(game.g_rom.offset(crate::rtl::lorom_offset(0xade742)) as *const uint8 as *mut uint16)
                .offset((v0 as ::core::ffi::c_int >> 1 as ::core::ffi::c_int) as isize),
            0xf as uint16,
            game,
        );
    }
}
pub unsafe fn MotherBrain_FadePalToBlack(a: uint16, game: &mut Game) -> uint8 {
    let v1: ::core::ffi::c_int = a as ::core::ffi::c_int;
    if *(game.g_rom.offset(crate::rtl::lorom_offset(0xade9e8)) as *const uint8 as *mut uint16)
        .offset(v1 as isize)
        == 0
    {
        return 1 as uint8;
    }
    let j: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xade9e8)) as *const uint8
        as *mut uint16)
        .offset(v1 as isize);
    WriteColorsToPalette(0x82 as uint16, 0xad as uint8, j, 14 as uint16, game);
    WriteColorsToPalette(0x122 as uint16, 0xad as uint8, j, 14 as uint16, game);
    WriteColorsToPalette(
        0x162 as uint16,
        0xad as uint8,
        (j).wrapping_add(28u16),
        14 as uint16,
        game,
    );
    0 as uint8
}
pub unsafe fn MotherBrain_FadeToGray_FakeDeath(a: uint16, game: &mut Game) -> uint8 {
    let v1: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xaded8a)) as *const uint8
        as *mut uint16)
        .offset(a as isize);
    if v1 != 0 {
        (MotherBrain_EEF6(v1, game)) & (1u8)
    } else {
        1 as uint8
    }
}
pub unsafe fn MotherBrain_FadeFromGray_FakeDeath(a: uint16, game: &mut Game) -> uint8 {
    let v1: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xaded9c)) as *const uint8
        as *mut uint16)
        .offset(a as isize);
    if v1 != 0 {
        (MotherBrain_EEF6(v1, game)) & (1u8)
    } else {
        1 as uint8
    }
}
pub unsafe fn MotherBrain_EEF6(a: uint16, game: &mut Game) -> uint8 {
    WriteColorsToPalette(0x122 as uint16, 0xad as uint8, a, 3 as uint16, game);
    0 as uint8
}
pub unsafe fn MotherBrain_FadeFromGray_Drained(a: uint16, game: &mut Game) -> uint8 {
    let v1: ::core::ffi::c_int = a as ::core::ffi::c_int;
    if *(game.g_rom.offset(crate::rtl::lorom_offset(0xaded9c)) as *const uint8 as *mut uint16)
        .offset(v1 as isize)
        == 0
    {
        return 1 as uint8;
    }
    let j: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xaded9c)) as *const uint8
        as *mut uint16)
        .offset(v1 as isize);
    WriteColorsToPalette(0x82 as uint16, 0xad as uint8, j, 13 as uint16, game);
    WriteColorsToPalette(0x122 as uint16, 0xad as uint8, j, 13 as uint16, game);
    WriteColorsToPalette(
        0x168 as uint16,
        0xad as uint8,
        (j).wrapping_add(26u16),
        5 as uint16,
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x17c) as *mut uint8 as *mut uint16) =
        *(RomPtr_AD((j).wrapping_add(36u16), game) as *mut uint16);
    0 as uint8
}
pub unsafe fn MotherBrain_FadeToGray_Drained(a: uint16, game: &mut Game) -> uint8 {
    let v1: ::core::ffi::c_int = a as ::core::ffi::c_int;
    if *(game.g_rom.offset(crate::rtl::lorom_offset(0xadef87)) as *const uint8 as *mut uint16)
        .offset(v1 as isize)
        == 0
    {
        return 1 as uint8;
    }
    let j: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xadef87)) as *const uint8
        as *mut uint16)
        .offset(v1 as isize);
    WriteColorsToPalette(0x82 as uint16, 0xad as uint8, j, 15 as uint16, game);
    WriteColorsToPalette(0x122 as uint16, 0xad as uint8, j, 15 as uint16, game);
    WriteColorsToPalette(
        0x168 as uint16,
        0xad as uint8,
        (j).wrapping_add(30u16),
        5 as uint16,
        game,
    );
    *((&raw mut game.g_ram as *mut uint8).offset(0x17c) as *mut uint8 as *mut uint16) =
        *(RomPtr_AD((j).wrapping_add(40u16), game) as *mut uint16);
    0 as uint8
}
pub unsafe fn MotherBrain_FadeToGray_RealDeath(a: uint16, game: &mut Game) -> uint8 {
    let v1: ::core::ffi::c_int = a as ::core::ffi::c_int;
    if *(game.g_rom.offset(crate::rtl::lorom_offset(0xadf107)) as *const uint8 as *mut uint16)
        .offset(v1 as isize)
        == 0
    {
        return 1 as uint8;
    }
    WriteColorsToPalette(
        0x1e2 as uint16,
        0xad as uint8,
        *(game.g_rom.offset(crate::rtl::lorom_offset(0xadf107)) as *const uint8 as *mut uint16)
            .offset(v1 as isize),
        0xf as uint16,
        game,
    );
    0 as uint8
}
pub unsafe fn TurnOffLightsForShitroidDeath(game: &mut Game) {
    WriteColorsToPalette(
        0x62 as uint16,
        0xad as uint8,
        addr_kTurnOffLightsForShitroidDeath as uint16,
        14 as uint16,
        game,
    );
    WriteColorsToPalette(
        0xa2 as uint16,
        0xad as uint8,
        addr_kTurnOffLightsForShitroidDeath as uint16,
        14 as uint16,
        game,
    );
}
pub unsafe fn MotherBrain_Phase3_TurnLightsBackOn(a: uint16, game: &mut Game) -> uint8 {
    let v1: ::core::ffi::c_int = a as ::core::ffi::c_int;
    if *(game.g_rom.offset(crate::rtl::lorom_offset(0xadf273)) as *const uint8 as *mut uint16)
        .offset(v1 as isize)
        == 0
    {
        return 1 as uint8;
    }
    let v3: uint16 = *(game.g_rom.offset(crate::rtl::lorom_offset(0xadf273)) as *const uint8
        as *mut uint16)
        .offset(v1 as isize);
    WriteColorsToPalette(0x62 as uint16, 0xad as uint8, v3, 14 as uint16, game);
    WriteColorsToPalette(
        0xa2 as uint16,
        0xad as uint8,
        (v3).wrapping_add(28u16),
        14 as uint16,
        game,
    );
    0 as uint8
}
pub unsafe fn EnableEarthquakeAframes(a: uint16, game: &mut Game) {
    set_ram_word!(game.earthquake_type = a);
    set_ram_word!(game.earthquake_timer = 20 as uint16);
    QueueSfx2_Max6(0x25 as uint16, game);
}
pub unsafe fn HandleMotherBrainBodyFlickering(game: &mut Game) {
    let v0: *mut EnemyData = gEnemyData(0 as ::core::ffi::c_int, game);
    if (*v0).frame_counter as ::core::ffi::c_int & 1 as ::core::ffi::c_int != 0 {
        let ref mut fresh0 = *((&raw mut game.g_ram as *mut uint8).offset(0x69) as *mut uint16);
        *fresh0 = (*fresh0) | (2u16);
        (*v0).properties = ((*v0).properties as ::core::ffi::c_int
            & !(kEnemyProps_Invisible as ::core::ffi::c_int)) as uint16;
    } else {
        let ref mut fresh1 = *((&raw mut game.g_ram as *mut uint8).offset(0x69) as *mut uint16);
        *fresh1 = (*fresh1 as ::core::ffi::c_int & !(2 as ::core::ffi::c_int)) as uint16;
        (*v0).properties = ((*v0).properties) | ((kEnemyProps_Invisible) as u16);
    };
}
