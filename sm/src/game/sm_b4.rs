use crate::game::sm_81::DrawSpritemapWithBaseTile;
use crate::game_state::Game;
use crate::rtl::{RomPtr, Unreachable};

pub use crate::types::{
    __int16_t, __uint16_t, __uint32_t, __uint8_t, int16, int16_t, uint16, uint16_t, uint32,
    uint32_t, uint8, uint8_t,
};
#[inline]
unsafe fn RomPtr_B4(mut addr: uint16_t, game: &mut Game) -> *const uint8 {
    return RomPtr(
        (0xb40000 as ::core::ffi::c_int | addr as ::core::ffi::c_int) as uint32_t,
        game,
    );
}
pub const fnSpriteObject_Instr_RepeatLast: uint32 = 11844848;
pub const fnSpriteObject_Instr_Terminate: uint32 = 11844871;
pub const fnSpriteObject_Instr_Goto: uint32 = 11844882;
pub unsafe fn CreateSpriteAtPos(
    mut x_r18: uint16,
    mut y_r20: uint16,
    mut ilist_r22: uint16,
    mut pal_r24: uint16,
    game: &mut Game,
) -> uint16 {
    let mut v0: ::core::ffi::c_int = 62 as ::core::ffi::c_int;
    while *((&raw mut game.g_ram as *mut uint8).offset(0xef78) as *mut uint16)
        .offset((v0 >> 1 as ::core::ffi::c_int) as isize)
        != 0
    {
        v0 -= 2 as ::core::ffi::c_int;
        if v0 & 0x8000 as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
            return 0xffff as uint16;
        }
    }
    let mut v1: ::core::ffi::c_int = v0 >> 1 as ::core::ffi::c_int;
    *((&raw mut game.g_ram as *mut uint8).offset(0xf078) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xf178) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xf278) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xf2f8) as *mut uint16).offset(v1 as isize) =
        0 as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xf0f8) as *mut uint16).offset(v1 as isize) =
        x_r18;
    *((&raw mut game.g_ram as *mut uint8).offset(0xf1f8) as *mut uint16).offset(v1 as isize) =
        y_r20;
    *((&raw mut game.g_ram as *mut uint8).offset(0xf078) as *mut uint16).offset(v1 as isize) =
        pal_r24;
    let mut v2: uint16 = *(game.g_rom.offset(
        (((0xb4bda8 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
            | 0xb4bda8 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
            & 0x3fffff as ::core::ffi::c_int) as isize,
    ) as *const uint8 as *mut uint16)
        .offset(ilist_r22 as isize);
    *((&raw mut game.g_ram as *mut uint8).offset(0xef78) as *mut uint16).offset(v1 as isize) = v2;
    *((&raw mut game.g_ram as *mut uint8).offset(0xeff8) as *mut uint16).offset(v1 as isize) =
        *(RomPtr_B4(v2 as uint16_t, game) as *mut uint16);
    return v0 as uint16;
}
pub unsafe fn CallSpriteObjectInstr(mut ea: uint32, game: &mut Game) {
    match ea {
        11844848 => {
            SpriteObject_Instr_RepeatLast(game);
            return;
        }
        11844871 => {
            SpriteObject_Instr_Terminate(game);
            return;
        }
        11844882 => {
            SpriteObject_Instr_Goto(game);
            return;
        }
        _ => {
            Unreachable(game);
        }
    };
}
pub unsafe fn HandleSpriteObjects(game: &mut Game) {
    let mut v1: uint16 = 0;
    if *((&raw mut game.g_ram as *mut uint8).offset(0x185e) as *mut uint16) as ::core::ffi::c_int
        | *((&raw mut game.g_ram as *mut uint8).offset(0xa78) as *mut uint16) as ::core::ffi::c_int
        != 0
    {
        return;
    }
    *((&raw mut game.g_ram as *mut uint8).offset(0x1844) as *mut uint16) = 62 as uint16;
    let mut current_block_9: u64;
    loop {
        let mut v0: ::core::ffi::c_int = 0;
        v0 = *((&raw mut game.g_ram as *mut uint8).offset(0x1844) as *mut uint16)
            as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int;
        if !(*((&raw mut game.g_ram as *mut uint8).offset(0xef78) as *mut uint16)
            .offset(v0 as isize)
            == 0
            || *((&raw mut game.g_ram as *mut uint8).offset(0xf2f8) as *mut uint16)
                .offset(v0 as isize) as ::core::ffi::c_int
                & 1 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_int)
        {
            v1 = *((&raw mut game.g_ram as *mut uint8).offset(0xeff8) as *mut uint16)
                .offset(v0 as isize);
            if v1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
                current_block_9 = 8895462745870634114;
            } else {
                *((&raw mut game.g_ram as *mut uint8).offset(0xeff8) as *mut uint16)
                    .offset(v0 as isize) =
                    (v1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as uint16;
                if v1 as ::core::ffi::c_int == 1 as ::core::ffi::c_int {
                    let mut v3: uint16 =
                        (*((&raw mut game.g_ram as *mut uint8).offset(0xef78) as *mut uint16)
                            .offset(v0 as isize) as ::core::ffi::c_int
                            + 4 as ::core::ffi::c_int) as uint16;
                    *((&raw mut game.g_ram as *mut uint8).offset(0xef78) as *mut uint16)
                        .offset(v0 as isize) = v3;
                    v1 = *(RomPtr_B4(v3 as uint16_t, game) as *mut uint16);
                    if v1 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int != 0 {
                        current_block_9 = 8895462745870634114;
                    } else {
                        *((&raw mut game.g_ram as *mut uint8).offset(0xeff8) as *mut uint16)
                            .offset(
                                (*((&raw mut game.g_ram as *mut uint8).offset(0x1844)
                                    as *mut uint16)
                                    as ::core::ffi::c_int
                                    >> 1 as ::core::ffi::c_int)
                                    as isize,
                            ) = v1;
                        current_block_9 = 14916268686031723178;
                    }
                } else {
                    current_block_9 = 14916268686031723178;
                }
            }
            match current_block_9 {
                14916268686031723178 => {}
                _ => {
                    CallSpriteObjectInstr(
                        (v1 as ::core::ffi::c_int | 0xb40000 as ::core::ffi::c_int) as uint32,
                        game,
                    );
                }
            }
        }
        let ref mut fresh0 = *((&raw mut game.g_ram as *mut uint8).offset(0x1844) as *mut uint16);
        *fresh0 = (*fresh0 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as uint16;
        if !(*fresh0 as ::core::ffi::c_int & 0x8000 as ::core::ffi::c_int == 0) {
            break;
        }
    }
}
pub unsafe fn SpriteObject_Instr_RepeatLast(game: &mut Game) {
    let mut v2: ::core::ffi::c_int = *((&raw mut game.g_ram as *mut uint8).offset(0x1844)
        as *mut uint16) as ::core::ffi::c_int
        >> 1 as ::core::ffi::c_int;
    let ref mut fresh1 =
        *((&raw mut game.g_ram as *mut uint8).offset(0xef78) as *mut uint16).offset(v2 as isize);
    *fresh1 = (*fresh1 as ::core::ffi::c_int - 4 as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0xeff8) as *mut uint16).offset(v2 as isize) =
        0x7fff as uint16;
}
pub unsafe fn SpriteObject_Instr_Terminate(game: &mut Game) {
    *((&raw mut game.g_ram as *mut uint8).offset(0xef78) as *mut uint16).offset(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x1844) as *mut uint16) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int) as isize,
    ) = 0 as uint16;
}
pub unsafe fn SpriteObject_Instr_Goto(game: &mut Game) {
    let mut v2: uint16 = *(RomPtr_B4(
        *((&raw mut game.g_ram as *mut uint8).offset(0xef78) as *mut uint16).offset(
            (*((&raw mut game.g_ram as *mut uint8).offset(0x1844) as *mut uint16)
                as ::core::ffi::c_int
                >> 1 as ::core::ffi::c_int) as isize,
        ) as uint16_t,
        game,
    )
    .offset(2) as *mut uint16);
    *((&raw mut game.g_ram as *mut uint8).offset(0xef78) as *mut uint16).offset(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x1844) as *mut uint16) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int) as isize,
    ) = v2;
    *((&raw mut game.g_ram as *mut uint8).offset(0xeff8) as *mut uint16).offset(
        (*((&raw mut game.g_ram as *mut uint8).offset(0x1844) as *mut uint16) as ::core::ffi::c_int
            >> 1 as ::core::ffi::c_int) as isize,
    ) = *(RomPtr_B4(v2 as uint16_t, game) as *mut uint16);
}
pub unsafe fn DrawSpriteObjects(game: &mut Game) {
    let mut i: ::core::ffi::c_int = 62 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        let mut v1: ::core::ffi::c_int = i >> 1 as ::core::ffi::c_int;
        if *((&raw mut game.g_ram as *mut uint8).offset(0xef78) as *mut uint16).offset(v1 as isize)
            != 0
        {
            let mut x: uint16 = (*((&raw mut game.g_ram as *mut uint8).offset(0xf0f8)
                as *mut uint16)
                .offset(v1 as isize) as ::core::ffi::c_int
                - *((&raw mut game.g_ram as *mut uint8).offset(0x911) as *mut uint16)
                    as ::core::ffi::c_int) as uint16;
            if (x as ::core::ffi::c_int + 16 as ::core::ffi::c_int) as int16 as ::core::ffi::c_int
                >= 0 as ::core::ffi::c_int
            {
                if x as ::core::ffi::c_int - 272 as ::core::ffi::c_int
                    & 0x8000 as ::core::ffi::c_int
                    != 0
                {
                    let mut y: int16 =
                        (*((&raw mut game.g_ram as *mut uint8).offset(0xf1f8) as *mut uint16)
                            .offset(v1 as isize) as ::core::ffi::c_int
                            - *((&raw mut game.g_ram as *mut uint8).offset(0x915) as *mut uint16)
                                as ::core::ffi::c_int) as int16;
                    if y as ::core::ffi::c_int >= 0 as ::core::ffi::c_int {
                        if y as ::core::ffi::c_int - 272 as ::core::ffi::c_int
                            & 0x8000 as ::core::ffi::c_int
                            != 0
                        {
                            let mut r3: uint16 = (*((&raw mut game.g_ram as *mut uint8)
                                .offset(0xf078)
                                as *mut uint16)
                                .offset(v1 as isize)
                                as ::core::ffi::c_int
                                & 0xe00 as ::core::ffi::c_int)
                                as uint16;
                            let mut r0: uint16 = (*((&raw mut game.g_ram as *mut uint8)
                                .offset(0xf078)
                                as *mut uint16)
                                .offset(v1 as isize)
                                as ::core::ffi::c_int
                                & 0x1ff as ::core::ffi::c_int)
                                as uint16;
                            let mut v3: *const uint8 = RomPtr_B4(
                                *((&raw mut game.g_ram as *mut uint8).offset(0xef78) as *mut uint16)
                                    .offset(v1 as isize)
                                    as uint16_t,
                                game,
                            );
                            DrawSpritemapWithBaseTile(
                                0xb4 as uint8,
                                *(v3.offset(2) as *mut uint16),
                                x,
                                y as uint16,
                                r3,
                                r0,
                                game,
                            );
                        }
                    }
                }
            }
        }
        i -= 2 as ::core::ffi::c_int;
    }
}
pub unsafe fn ClearSpriteObjects(game: &mut Game) {
    let mut v0: ::core::ffi::c_int = 1022 as ::core::ffi::c_int;
    loop {
        *((&raw mut game.g_ram as *mut uint8).offset(0xef78) as *mut uint16)
            .offset((v0 >> 1 as ::core::ffi::c_int) as isize) = 0 as uint16;
        v0 -= 2 as ::core::ffi::c_int;
        if !(v0 != 0) {
            break;
        }
    }
}
