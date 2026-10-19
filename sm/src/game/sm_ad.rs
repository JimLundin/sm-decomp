pub use crate::game::constants::bank_86::*;
pub use crate::game::constants::enemy_properties::*;
use crate::game::sm_80::QueueSfx2_Max6;
use crate::game::sm_84::SpawnHardcodedPlm;
use crate::game::sm_86::SpawnEprojWithRoomGfx;
use crate::game::sm_a9::WriteColorsToPalette;
use crate::game_state::Game;
use crate::ram_map::set_ram_word;
use crate::rtl::RomPtr;
use ::core::ffi::c_char;
extern "C" {
    fn printf(__format: *const c_char, ...) -> i32;

}
pub use crate::types::Consts_86;
pub use crate::types::Consts_AD;
pub use crate::types::{EnemyBase, EnemyData, EnemyProps, Func_V, Point16U, VoidP};
pub const addr_kTurnOffLightsForShitroidDeath: Consts_AD = 61999;
pub use crate::types::{Enemy_MotherBrain, Func_CalcHdma, Ram8000_Default, SpawnHardcodedPlmArgs};
#[inline]
unsafe fn RomPtr_AD(addr: u16, game: &mut Game) -> *const u8 {
    RomPtr((0xad0000u32) | ((addr) as u32), game)
}
pub const fnMotherBrainBody_FakeDeath_Descent_0_Pause: i32 = 0xa9881d;
#[inline]
unsafe fn gEnemyData(o: i32, game: &mut Game) -> *mut EnemyData {
    (&raw mut game.g_ram as *mut u8)
        .offset(0xf78)
        .offset(o as isize) as *mut EnemyData
}
#[inline]
unsafe fn gRam8000_Default(o: i32, game: &mut Game) -> *mut Ram8000_Default {
    (&raw mut game.g_ram as *mut u8)
        .offset(0x8000)
        .offset(o as isize) as *mut Ram8000_Default
}
#[inline]
unsafe fn Get_MotherBrain(o: i32, game: &mut Game) -> *mut Enemy_MotherBrain {
    (&raw mut game.g_ram as *mut u8)
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
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    let E1: *mut Enemy_MotherBrain = Get_MotherBrain(0x40i32, game);
    let mut r18: u16 =
        (*(&raw mut (*E).mbn_var_33 as *mut u8).offset(1isize) as i32 >> 1i32) as u16;
    (*E).mbn_var_3A = ((*E).mbn_var_31).wrapping_sub(r18);
    (*E).mbn_var_3B = (r18).wrapping_add((*E).mbn_var_31);
    let v2: u16 = (*((&raw mut (*E1).base.enemy_ptr as *mut u8).offset(1) as *mut u16) as i32
        + 3584i32
        & 0xff00i32) as u16;
    (*E).mbn_var_3C = v2;
    (*E).mbn_var_3E = v2;
    let v3: u16 = ((*E1).base.y_pos).wrapping_add(5u16);
    (*E).mbn_var_3D = v3;
    (*E).mbn_var_3F = v3;
    r18 = (4i32 * ((*E).mbn_var_3A as i32 & 0xc0i32)) as u16;
    let i: i32 = (r18 as i32 | (*E).mbn_var_3B as i32 & 0xc0i32) >> 6i32;
    funcs_BE56D[i as usize].expect("non-null function pointer")(game);
}
pub unsafe fn MotherBrain_CalcHdma_BeamAimedRight2(game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    MotherBrain_CalcHdma_BeamAimedRight((*E).mbn_var_3C, (*E).mbn_var_3E, game);
    *((&raw mut game.g_ram as *mut u8).offset(0x9c00) as *mut u16) = 16u16;
    *((&raw mut game.g_ram as *mut u8).offset(0x9c00).offset(1) as *mut u8 as *mut u16) =
        -25600i32 as u16;
    *((&raw mut game.g_ram as *mut u8).offset(0x9c00).offset(3) as *mut u8 as *mut u16) = 16u16;
    *((&raw mut game.g_ram as *mut u8).offset(0x9c00).offset(4) as *mut u8 as *mut u16) =
        -25597i32 as u16;
    *((&raw mut game.g_ram as *mut u8).offset(0x9c00).offset(6) as *mut u8 as *mut u16) = 240u16;
    *((&raw mut game.g_ram as *mut u8).offset(0x9c00).offset(7) as *mut u8 as *mut u16) =
        -25340i32 as u16;
    *((&raw mut game.g_ram as *mut u8).offset(0x9c00).offset(9) as *mut u8 as *mut u16) = 244u16;
    *((&raw mut game.g_ram as *mut u8).offset(0x9c00).offset(10) as *mut u8 as *mut u16) =
        -25108i32 as u16;
    *((&raw mut game.g_ram as *mut u8).offset(0x9c00).offset(12) as *mut u8 as *mut u16) = 0u16;
}
pub unsafe fn MotherBrain_CalcHdma_BeamAimedRight(mut r22: u16, mut r24: u16, game: &mut Game) {
    set_ram_word!(game.hdma_table_2[0] = 255u16);
    set_ram_word!(game.hdma_table_2[1] = 255u16);
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    let r18: u16 = crate::rtl::rom_table_word!(
        game,
        0x91c9d4,
        *(&raw mut (*E).mbn_var_3A as *mut u8).offset(0) as isize
    );
    let r20: u16 = crate::rtl::rom_table_word!(
        game,
        0x91c9d4,
        *(&raw mut (*E).mbn_var_3B as *mut u8).offset(0) as isize
    );
    let mut mbn_var_3F: u16 = (*E).mbn_var_3F;
    let mut v2: *mut u16 = ((&raw mut game.g_ram as *mut u8).offset(0x9d00) as *mut u16)
        .offset((mbn_var_3F as i32 - 32i32) as isize)
        .offset(1);
    *v2.offset(1) = 255u16;
    *v2.offset(2) = 255u16;
    let v8: *mut u16 = v2;
    loop {
        let mut v4: u16 = (r20).wrapping_add(r24);
        if u16::overflowing_add(r20, r24).1 {
            break;
        }
        r24 = (r24).wrapping_add(r20);
        let fresh11 = v2;
        v2 = v2.offset(-1);
        *fresh11 = ((*(&raw mut v4 as *mut u8).offset(1isize)) as u16) | (0xff00u16);
        mbn_var_3F = mbn_var_3F.wrapping_sub(1);
        if (mbn_var_3F == 32u16) {
            break;
        }
    }
    loop {
        let fresh12 = v2;
        v2 = v2.offset(-1);
        *fresh12 = 255u16;
        mbn_var_3F = mbn_var_3F.wrapping_sub(1);
        if (mbn_var_3F == 32u16) {
            break;
        }
    }
    let mut v5: *mut u16 = v8.offset(1);
    let mut var_3F: u16 = (*gRam8000_Default(0i32, game)).var_3F;
    loop {
        let mut v7: u16 = (r18).wrapping_add(r22);
        if u16::overflowing_add(r18, r22).1 {
            break;
        }
        r22 = (r22).wrapping_add(r18);
        let fresh13 = v5;
        v5 = v5.offset(1);
        *fresh13 = ((*(&raw mut v7 as *mut u8).offset(1isize)) as u16) | (0xff00u16);
        var_3F = var_3F.wrapping_add(1);
        if var_3F == 232u16 {
            return;
        }
    }
    loop {
        let fresh14 = v5;
        v5 = v5.offset(1);
        *fresh14 = 255u16;
        var_3F = var_3F.wrapping_add(1);
        if (var_3F == 232u16) {
            break;
        }
    }
}
static off_ADE024: [Option<Func_CalcHdma>; 4] = unsafe {
    [
        Some(MotherBrain_CalcHdma_Up_UpRight as unsafe fn(u16, u16, &mut Game) -> ()),
        Some(MotherBrain_CalcHdma_Up_Up as unsafe fn(u16, u16, &mut Game) -> ()),
        None,
        Some(MotherBrain_CalcHdma_Up_UpLeft as unsafe fn(u16, u16, &mut Game) -> ()),
    ]
};
pub unsafe fn MotherBrain_CalcHdma_BeamAimedUp(game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    off_ADE024
        [((*E).mbn_var_3A as i32 >> 6i32 & 2i32 | (*E).mbn_var_3B as i32 >> 7i32 & 1i32) as usize]
        .expect("non-null function pointer")((*E).mbn_var_3C, (*E).mbn_var_3E, game);
    *((&raw mut game.g_ram as *mut u8).offset(0x9c00) as *mut u16) = 16u16;
    *((&raw mut game.g_ram as *mut u8).offset(0x9c00).offset(1) as *mut u8 as *mut u16) =
        -25344i32 as u16;
    *((&raw mut game.g_ram as *mut u8).offset(0x9c00).offset(3) as *mut u8 as *mut u16) = 16u16;
    *((&raw mut game.g_ram as *mut u8).offset(0x9c00).offset(4) as *mut u8 as *mut u16) =
        -25342i32 as u16;
    let v2: u16 = ((*E).mbn_var_3D).wrapping_sub(32u16);
    printf(b"Carry crap!\n\0" as *const u8 as *const c_char);
    if v2 >= 0x80u16 {
        *((&raw mut game.g_ram as *mut u8).offset(0x9c00).offset(6) as *mut u8 as *mut u16) =
            (v2 as i32 - 127i32 | 0x80i32) as u16;
        *((&raw mut game.g_ram as *mut u8).offset(0x9c00).offset(7) as *mut u8 as *mut u16) =
            -25340i32 as u16;
        *((&raw mut game.g_ram as *mut u8).offset(0x9c00).offset(9) as *mut u8 as *mut u16) =
            255u16;
        *((&raw mut game.g_ram as *mut u8).offset(0x9c00).offset(10) as *mut u8 as *mut u16) =
            (2i32 * (v2 as i32 - 127i32) - 25340i32) as u16;
        *((&raw mut game.g_ram as *mut u8).offset(0x9c00).offset(12) as *mut u8 as *mut u16) =
            127u16;
        *((&raw mut game.g_ram as *mut u8).offset(0x9c00).offset(13) as *mut u8 as *mut u16) =
            (2i32 * (v2 as i32 - 127i32) - 25340i32 + 254i32) as u16;
        *((&raw mut game.g_ram as *mut u8).offset(0x9c00).offset(15) as *mut u8 as *mut u16) = 0u16;
    } else {
        *((&raw mut game.g_ram as *mut u8).offset(0x9c00).offset(6) as *mut u8 as *mut u16) =
            (v2) | (0x80u16);
        *((&raw mut game.g_ram as *mut u8).offset(0x9c00).offset(7) as *mut u8 as *mut u16) =
            -25340i32 as u16;
        *((&raw mut game.g_ram as *mut u8).offset(0x9c00).offset(9) as *mut u8 as *mut u16) =
            127u16;
        *((&raw mut game.g_ram as *mut u8).offset(0x9c00).offset(10) as *mut u8 as *mut u16) =
            (2i32 * v2 as i32 - 25340i32) as u16;
        *((&raw mut game.g_ram as *mut u8).offset(0x9c00).offset(12) as *mut u8 as *mut u16) = 0u16;
    };
}
pub unsafe fn MotherBrain_CalcHdma_Up_UpRight(mut r22: u16, mut r24: u16, game: &mut Game) {
    set_ram_word!(game.hdma_table_2[0] = 255u16);
    set_ram_word!(game.hdma_table_2[1] = 255u16);
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    let r18: u16 = crate::rtl::rom_table_word!(
        game,
        0x91c9d4,
        *(&raw mut (*E).mbn_var_3B as *mut u8).offset(0) as isize
    );
    let r20: u16 = crate::rtl::rom_table_word!(
        game,
        0x91c9d4,
        *(&raw mut (*E).mbn_var_3A as *mut u8).offset(0) as isize
    );
    let mut mbn_var_3D: u16 = (*E).mbn_var_3D;
    let mut v2: *mut u16 = ((&raw mut game.g_ram as *mut u8).offset(0x9d00) as *mut u16)
        .offset((mbn_var_3D as i32 - 32i32) as isize)
        .offset(1);
    *v2.offset(1) = 255u16;
    *v2.offset(2) = 255u16;
    loop {
        let mut v4: u16 = (r18).wrapping_add(r22);
        if u16::overflowing_add(r18, r22).1 {
            v4 = -1i32 as u16;
        }
        r22 = v4;
        let R36: u16 = (v4 as i32 >> 8i32 & 0xffi32) as u16;
        let mut v7: u16 = (r20).wrapping_add(r24);
        if u16::overflowing_add(r20, r24).1 {
            v7 = -1i32 as u16;
        }
        r24 = v7;
        let mut v8: u16 = (R36 as i32 | v7 as i32 & 0xff00i32) as u16;
        if v8 == 0xffffu16 {
            v8 = 255u16;
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
pub unsafe fn MotherBrain_CalcHdma_Up_Up(mut r22: u16, mut r24: u16, game: &mut Game) {
    set_ram_word!(game.hdma_table_2[0] = 255u16);
    set_ram_word!(game.hdma_table_2[1] = 255u16);
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    let r18: u16 = crate::rtl::rom_table_word!(
        game,
        0x91c9d4,
        -(*(&raw mut (*E).mbn_var_3B as *mut u8).offset(0) as i32) as u8 as isize
    );
    let r20: u16 = crate::rtl::rom_table_word!(
        game,
        0x91c9d4,
        *(&raw mut (*E).mbn_var_3A as *mut u8).offset(0) as isize
    );
    let mut mbn_var_3D: u16 = (*E).mbn_var_3D;
    let mut v2: *mut u16 = ((&raw mut game.g_ram as *mut u8).offset(0x9d00) as *mut u16)
        .offset((mbn_var_3D as i32 - 32i32) as isize)
        .offset(1);
    *v2.offset(1) = 255u16;
    *v2.offset(2) = 255u16;
    loop {
        let mut v4: u16 = (r22).wrapping_sub(r18);
        if r22 < r18 {
            v4 = 0u16;
        }
        r22 = v4;
        let r26: u16 = (v4 as i32 >> 8i32) as u8 as u16;
        let mut v7: u16 = (r20).wrapping_add(r24);
        if u16::overflowing_add(r20, r24).1 {
            v7 = -1i32 as u16;
        }
        r24 = v7;
        let mut v8: u16 = (r26 as i32 | v7 as i32 & 0xff00i32) as u16;
        if v8 == 0xffffu16 {
            v8 = 255u16;
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
pub unsafe fn MotherBrain_CalcHdma_Up_UpLeft(mut r22: u16, mut r24: u16, game: &mut Game) {
    set_ram_word!(game.hdma_table_2[0] = 255u16);
    set_ram_word!(game.hdma_table_2[1] = 255u16);
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    let r18: u16 = crate::rtl::rom_table_word!(
        game,
        0x91c9d4,
        -(*(&raw mut (*E).mbn_var_3B as *mut u8).offset(0) as i32) as u8 as isize
    );
    let r20: u16 = crate::rtl::rom_table_word!(
        game,
        0x91c9d4,
        -(*(&raw mut (*E).mbn_var_3A as *mut u8).offset(0) as i32) as u8 as isize
    );
    let mut mbn_var_3D: u16 = (*E).mbn_var_3D;
    let mut v2: *mut u16 = ((&raw mut game.g_ram as *mut u8).offset(0x9d00) as *mut u16)
        .offset((mbn_var_3D as i32 - 32i32) as isize)
        .offset(1);
    *v2.offset(1) = 255u16;
    *v2.offset(2) = 255u16;
    loop {
        let mut v4: u16 = (r22).wrapping_sub(r18);
        if r22 < r18 {
            v4 = 0u16;
        }
        r22 = v4;
        let R36: u16 = (v4 as i32 >> 8i32) as u16;
        let mut v7: u16 = (r24).wrapping_sub(r20);
        if r24 < r20 {
            v7 = 0u16;
        }
        r24 = v7;
        let mut v8: u16 = (R36 as i32 | v7 as i32 & 0xff00i32) as u16;
        if v8 == 0xffffu16 {
            v8 = 255u16;
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
        Some(MotherBrain_CalcHdma_Down_DownRight as unsafe fn(u16, u16, &mut Game) -> ()),
        None,
        Some(MotherBrain_CalcHdma_Down_Down as unsafe fn(u16, u16, &mut Game) -> ()),
        Some(MotherBrain_CalcHdma_Down_DownLeft as unsafe fn(u16, u16, &mut Game) -> ()),
    ]
};
pub unsafe fn MotherBrain_CalcHdma_Down(game: &mut Game) {
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    g_off_ADE20E
        [((*E).mbn_var_3A as i32 >> 6i32 & 2i32 | (*E).mbn_var_3B as i32 >> 7i32 & 1i32) as usize]
        .expect("non-null function pointer")((*E).mbn_var_3C, (*E).mbn_var_3E, game);
    *((&raw mut game.g_ram as *mut u8).offset(0x9c00) as *mut u16) = 16u16;
    *((&raw mut game.g_ram as *mut u8).offset(0x9c00).offset(1) as *mut u8 as *mut u16) =
        -25600i32 as u16;
    *((&raw mut game.g_ram as *mut u8).offset(0x9c00).offset(3) as *mut u8 as *mut u16) = 16u16;
    *((&raw mut game.g_ram as *mut u8).offset(0x9c00).offset(4) as *mut u8 as *mut u16) =
        -25597i32 as u16;
    *((&raw mut game.g_ram as *mut u8).offset(0x9c00).offset(6) as *mut u8 as *mut u16) = 240u16;
    *((&raw mut game.g_ram as *mut u8).offset(0x9c00).offset(7) as *mut u8 as *mut u16) =
        -25340i32 as u16;
    *((&raw mut game.g_ram as *mut u8).offset(0x9c00).offset(9) as *mut u8 as *mut u16) = 244u16;
    *((&raw mut game.g_ram as *mut u8).offset(0x9c00).offset(10) as *mut u8 as *mut u16) =
        -25114i32 as u16;
    *((&raw mut game.g_ram as *mut u8).offset(0x9c00).offset(12) as *mut u8 as *mut u16) = 0u16;
}
pub unsafe fn MotherBrain_CalcHdma_Down_DownRight(mut r22: u16, mut r24: u16, game: &mut Game) {
    set_ram_word!(game.hdma_table_2[0] = 255u16);
    set_ram_word!(game.hdma_table_2[1] = 255u16);
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    let r18: u16 = crate::rtl::rom_table_word!(
        game,
        0x91c9d4,
        *(&raw mut (*E).mbn_var_3A as *mut u8).offset(0) as isize
    );
    let r20: u16 = crate::rtl::rom_table_word!(
        game,
        0x91c9d4,
        *(&raw mut (*E).mbn_var_3B as *mut u8).offset(0) as isize
    );
    let mut v1: u16 = ((*E).mbn_var_3D).wrapping_sub(32u16);
    let mut dst: *mut u16 = ((&raw mut game.g_ram as *mut u8).offset(0x9d00) as *mut u16).offset(2);
    loop {
        let fresh6 = dst;
        dst = dst.offset(1);
        *fresh6 = 255u16;
        v1 = v1.wrapping_sub(1);
        if (v1 == 0) {
            break;
        }
    }
    let mut mbn_var_3D: u16 = (*Get_MotherBrain(0i32, game)).mbn_var_3D;
    loop {
        let mut v4: u16 = (r18).wrapping_add(r22);
        if u16::overflowing_add(r18, r22).1 {
            v4 = -1i32 as u16;
        }
        r22 = v4;
        let r26: u16 = (v4 as i32 >> 8i32) as u16;
        let mut v7: u16 = (r20).wrapping_add(r24);
        if u16::overflowing_add(r20, r24).1 {
            v7 = -1i32 as u16;
        }
        r24 = v7;
        let mut v8: u16 = (r26 as i32 | v7 as i32 & 0xff00i32) as u16;
        if v8 == 0xffffu16 {
            v8 = 255u16;
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
pub unsafe fn MotherBrain_CalcHdma_Down_Down(mut r22: u16, mut r24: u16, game: &mut Game) {
    set_ram_word!(game.hdma_table_2[0] = 255u16);
    set_ram_word!(game.hdma_table_2[1] = 255u16);
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    let r18: u16 = crate::rtl::rom_table_word!(
        game,
        0x91c9d4,
        -(*(&raw mut (*E).mbn_var_3A as *mut u8).offset(0) as i32) as u8 as isize
    );
    let r20: u16 = crate::rtl::rom_table_word!(
        game,
        0x91c9d4,
        *(&raw mut (*E).mbn_var_3B as *mut u8).offset(0) as isize
    );
    let mut v1: u16 = ((*E).mbn_var_3D).wrapping_sub(32u16);
    let mut dst: *mut u16 = ((&raw mut game.g_ram as *mut u8).offset(0x9d00) as *mut u16).offset(2);
    loop {
        let fresh4 = dst;
        dst = dst.offset(1);
        *fresh4 = 255u16;
        v1 = v1.wrapping_sub(1);
        if (v1 == 0) {
            break;
        }
    }
    let mut mbn_var_3D: u16 = (*Get_MotherBrain(0i32, game)).mbn_var_3D;
    loop {
        let mut v4: u16 = (r22).wrapping_sub(r18);
        if r22 < r18 {
            v4 = 0u16;
        }
        r22 = v4;
        let r26: u16 = (v4 as i32 >> 8i32) as u16;
        let mut v7: u16 = (r20).wrapping_add(r24);
        if u16::overflowing_add(r20, r24).1 {
            v7 = -1i32 as u16;
        }
        r24 = v7;
        let mut v8: u16 = (r26 as i32 | v7 as i32 & 0xff00i32) as u16;
        if v8 == 0xffffu16 {
            v8 = 255u16;
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
pub unsafe fn MotherBrain_CalcHdma_Down_DownLeft(mut r22: u16, mut r24: u16, game: &mut Game) {
    set_ram_word!(game.hdma_table_2[0] = 255u16);
    set_ram_word!(game.hdma_table_2[1] = 255u16);
    let E: *mut Enemy_MotherBrain = Get_MotherBrain(0i32, game);
    let r18: u16 = crate::rtl::rom_table_word!(
        game,
        0x91c9d4,
        -(*(&raw mut (*E).mbn_var_3A as *mut u8).offset(0) as i32) as u8 as isize
    );
    let r20: u16 = crate::rtl::rom_table_word!(
        game,
        0x91c9d4,
        -(*(&raw mut (*E).mbn_var_3B as *mut u8).offset(0) as i32) as u8 as isize
    );
    let mut v1: u16 = ((*E).mbn_var_3D).wrapping_sub(32u16);
    let mut dst: *mut u16 = ((&raw mut game.g_ram as *mut u8).offset(0x9d00) as *mut u16).offset(2);
    loop {
        let fresh2 = dst;
        dst = dst.offset(1);
        *fresh2 = 255u16;
        v1 = v1.wrapping_sub(1);
        if (v1 == 0) {
            break;
        }
    }
    let mut mbn_var_3D: u16 = (*Get_MotherBrain(0i32, game)).mbn_var_3D;
    loop {
        let mut v4: u16 = (r22).wrapping_sub(r18);
        if r22 < r18 {
            v4 = 0u16;
        }
        r22 = v4;
        let v5: u16 = (v4 as i32 >> 8i32) as u16;
        let r26: u16 = v5;
        let mut v7: u16 = (r24).wrapping_sub(r20);
        if r24 < r20 {
            v7 = 0u16;
        }
        r24 = v7;
        let mut v8: u16 = (r26 as i32 | v7 as i32 & 0xff00i32) as u16;
        if v8 == 0 {
            v8 = 255u16;
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
        x: 248u16,
        y: 72u16,
    };
    SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 9u16, game);
    game.eproj_spawn_pt = Point16U {
        x: 248u16,
        y: 152u16,
    };
    SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 9u16, game);
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs::new(0xfu8, 0x4u8, 0xb673 as VoidP),
        game,
    );
    SpawnHardcodedPlm(
        SpawnHardcodedPlmArgs::new(0xfu8, 0x9u8, 0xb673 as VoidP),
        game,
    );
    (*Get_MotherBrain(0i32, game)).mbn_var_A = ((0xa9881di32) as u16);
}
pub unsafe fn MotherBrain_HealthBasedPaletteHandling(game: &mut Game) {
    if (*Get_MotherBrain(0i32, game)).mbn_var_1F >= 2u16 {
        let mut v0: u16 = 0u16;
        let health: u16 = (*Get_MotherBrain(0x40i32, game)).base.health;
        if health < 0x2328u16 {
            v0 = 2u16;
            if health < 0x1518u16 {
                v0 = 4u16;
                if health < 0x708u16 {
                    v0 = 6u16;
                }
            }
        }
        WriteColorsToPalette(
            0x82u16,
            0xadu8,
            crate::rtl::rom_table_word!(game, 0xade6a2, (v0 as i32 >> 1i32) as isize),
            0xfu16,
            game,
        );
        WriteColorsToPalette(
            0x122u16,
            0xadu8,
            crate::rtl::rom_table_word!(game, 0xade6a2, (v0 as i32 >> 1i32) as isize),
            0xfu16,
            game,
        );
        WriteColorsToPalette(
            0x162u16,
            0xadu8,
            crate::rtl::rom_table_word!(game, 0xade742, (v0 as i32 >> 1i32) as isize),
            0xfu16,
            game,
        );
    }
}
pub unsafe fn MotherBrain_FadePalToBlack(a: u16, game: &mut Game) -> u8 {
    let v1: i32 = a as i32;
    if crate::rtl::rom_table_word!(game, 0xade9e8, v1 as isize) == 0 {
        return 1u8;
    }
    let j: u16 = crate::rtl::rom_table_word!(game, 0xade9e8, v1 as isize);
    WriteColorsToPalette(0x82u16, 0xadu8, j, 14u16, game);
    WriteColorsToPalette(0x122u16, 0xadu8, j, 14u16, game);
    WriteColorsToPalette(0x162u16, 0xadu8, (j).wrapping_add(28u16), 14u16, game);
    0u8
}
pub unsafe fn MotherBrain_FadeToGray_FakeDeath(a: u16, game: &mut Game) -> u8 {
    let v1: u16 = crate::rtl::rom_table_word!(game, 0xaded8a, a as isize);
    if v1 != 0 {
        (MotherBrain_EEF6(v1, game)) & (1u8)
    } else {
        1u8
    }
}
pub unsafe fn MotherBrain_FadeFromGray_FakeDeath(a: u16, game: &mut Game) -> u8 {
    let v1: u16 = crate::rtl::rom_table_word!(game, 0xaded9c, a as isize);
    if v1 != 0 {
        (MotherBrain_EEF6(v1, game)) & (1u8)
    } else {
        1u8
    }
}
pub unsafe fn MotherBrain_EEF6(a: u16, game: &mut Game) -> u8 {
    WriteColorsToPalette(0x122u16, 0xadu8, a, 3u16, game);
    0u8
}
pub unsafe fn MotherBrain_FadeFromGray_Drained(a: u16, game: &mut Game) -> u8 {
    let v1: i32 = a as i32;
    if crate::rtl::rom_table_word!(game, 0xaded9c, v1 as isize) == 0 {
        return 1u8;
    }
    let j: u16 = crate::rtl::rom_table_word!(game, 0xaded9c, v1 as isize);
    WriteColorsToPalette(0x82u16, 0xadu8, j, 13u16, game);
    WriteColorsToPalette(0x122u16, 0xadu8, j, 13u16, game);
    WriteColorsToPalette(0x168u16, 0xadu8, (j).wrapping_add(26u16), 5u16, game);
    *((&raw mut game.g_ram as *mut u8).offset(0x17c) as *mut u8 as *mut u16) =
        *(RomPtr_AD((j).wrapping_add(36u16), game) as *mut u16);
    0u8
}
pub unsafe fn MotherBrain_FadeToGray_Drained(a: u16, game: &mut Game) -> u8 {
    let v1: i32 = a as i32;
    if crate::rtl::rom_table_word!(game, 0xadef87, v1 as isize) == 0 {
        return 1u8;
    }
    let j: u16 = crate::rtl::rom_table_word!(game, 0xadef87, v1 as isize);
    WriteColorsToPalette(0x82u16, 0xadu8, j, 15u16, game);
    WriteColorsToPalette(0x122u16, 0xadu8, j, 15u16, game);
    WriteColorsToPalette(0x168u16, 0xadu8, (j).wrapping_add(30u16), 5u16, game);
    *((&raw mut game.g_ram as *mut u8).offset(0x17c) as *mut u8 as *mut u16) =
        *(RomPtr_AD((j).wrapping_add(40u16), game) as *mut u16);
    0u8
}
pub unsafe fn MotherBrain_FadeToGray_RealDeath(a: u16, game: &mut Game) -> u8 {
    let v1: i32 = a as i32;
    if crate::rtl::rom_table_word!(game, 0xadf107, v1 as isize) == 0 {
        return 1u8;
    }
    WriteColorsToPalette(
        0x1e2u16,
        0xadu8,
        crate::rtl::rom_table_word!(game, 0xadf107, v1 as isize),
        0xfu16,
        game,
    );
    0u8
}
pub unsafe fn TurnOffLightsForShitroidDeath(game: &mut Game) {
    WriteColorsToPalette(
        0x62u16,
        0xadu8,
        addr_kTurnOffLightsForShitroidDeath as u16,
        14u16,
        game,
    );
    WriteColorsToPalette(
        0xa2u16,
        0xadu8,
        addr_kTurnOffLightsForShitroidDeath as u16,
        14u16,
        game,
    );
}
pub unsafe fn MotherBrain_Phase3_TurnLightsBackOn(a: u16, game: &mut Game) -> u8 {
    let v1: i32 = a as i32;
    if crate::rtl::rom_table_word!(game, 0xadf273, v1 as isize) == 0 {
        return 1u8;
    }
    let v3: u16 = crate::rtl::rom_table_word!(game, 0xadf273, v1 as isize);
    WriteColorsToPalette(0x62u16, 0xadu8, v3, 14u16, game);
    WriteColorsToPalette(0xa2u16, 0xadu8, (v3).wrapping_add(28u16), 14u16, game);
    0u8
}
pub unsafe fn EnableEarthquakeAframes(a: u16, game: &mut Game) {
    set_ram_word!(game.earthquake_type = a);
    set_ram_word!(game.earthquake_timer = 20u16);
    QueueSfx2_Max6(0x25u16, game);
}
pub unsafe fn HandleMotherBrainBodyFlickering(game: &mut Game) {
    let v0: *mut EnemyData = gEnemyData(0i32, game);
    if (*v0).frame_counter as i32 & 1i32 != 0 {
        let ref mut fresh0 = *((&raw mut game.g_ram as *mut u8).offset(0x69) as *mut u16);
        *fresh0 = (*fresh0) | (2u16);
        (*v0).properties = ((*v0).properties as i32 & !(kEnemyProps_Invisible as i32)) as u16;
    } else {
        let ref mut fresh1 = *((&raw mut game.g_ram as *mut u8).offset(0x69) as *mut u16);
        *fresh1 = (*fresh1 as i32 & !(2i32)) as u16;
        (*v0).properties = ((*v0).properties) | ((kEnemyProps_Invisible) as u16);
    };
}
