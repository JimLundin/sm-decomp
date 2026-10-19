use crate::game::sm_80::QueueMusic_DelayedY;
use crate::game::sm_81::DrawSamusSpritemap;
use crate::game::sm_82::PlayRoomMusicTrackAfterAFrames;
use crate::game_state::Game;
use crate::rtl::RomPtr;

pub use crate::types::{
    __uint16_t, __uint32_t, __uint8_t, uint16, uint16_t, uint32_t, uint8, uint8_t,
};
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct SamusTileAnimationDefs {
    pub top_half_idx: uint8,
    pub top_half_pos: uint8,
    pub bottom_half_idx: uint8,
    pub bottom_half_pos: uint8,
}
#[inline]
unsafe fn get_SamusTileAnimationDefs(
    mut a: uint16,
    game: &mut Game,
) -> *mut SamusTileAnimationDefs {
    return RomPtr(
        (0x920000 as ::core::ffi::c_int | a as ::core::ffi::c_int) as uint32_t,
        game,
    ) as *mut SamusTileAnimationDefs;
}
pub unsafe fn SetSamusTilesDefsForCurAnim(game: &mut Game) {
    let mut v0: uint16 = (4 as ::core::ffi::c_int
        * *((&raw mut game.g_ram as *mut uint8).offset(0xa96) as *mut uint16) as ::core::ffi::c_int
        + *(game.g_rom.offset(
            (((0x92d94e as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                << 15 as ::core::ffi::c_int
                | 0x92d94e as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                & 0x3fffff as ::core::ffi::c_int) as isize,
        ) as *const uint8 as *mut uint16)
            .offset(*((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16) as isize)
            as ::core::ffi::c_int) as uint16;
    let mut AD: *mut SamusTileAnimationDefs = get_SamusTileAnimationDefs(v0, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x71f) as *mut uint16) =
        (7 as ::core::ffi::c_int * (*AD).top_half_pos as ::core::ffi::c_int
            + *(game.g_rom.offset(
                (((0x92d91e as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                    << 15 as ::core::ffi::c_int
                    | 0x92d91e as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                    & 0x3fffff as ::core::ffi::c_int) as isize,
            ) as *const uint8 as *mut uint16)
                .offset((*AD).top_half_idx as isize) as ::core::ffi::c_int) as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x71d) as *mut uint16 as *mut uint8).offset(0) =
        1 as uint8;
    if (*AD).bottom_half_idx as ::core::ffi::c_int != 255 as ::core::ffi::c_int {
        *((&raw mut game.g_ram as *mut uint8).offset(0x721) as *mut uint16) =
            (7 as ::core::ffi::c_int * (*AD).bottom_half_pos as ::core::ffi::c_int
                + *(game.g_rom.offset(
                    (((0x92d938 as ::core::ffi::c_int >> 16 as ::core::ffi::c_int)
                        << 15 as ::core::ffi::c_int
                        | 0x92d938 as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                        & 0x3fffff as ::core::ffi::c_int) as isize,
                ) as *const uint8 as *mut uint16)
                    .offset((*AD).bottom_half_idx as isize) as ::core::ffi::c_int)
                as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x71d) as *mut uint16 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>() as usize)
                .wrapping_div(::core::mem::size_of::<uint8>() as usize)
                .wrapping_sub(1 as usize) as isize,
        ) = 1 as uint8;
    }
}
pub unsafe fn PlaySamusFanfare(game: &mut Game) -> uint8 {
    if *((&raw mut game.g_ram as *mut uint8).offset(0xdec) as *mut uint16) != 0 {
        if *((&raw mut game.g_ram as *mut uint8).offset(0xdec) as *mut uint16) as ::core::ffi::c_int
            == 5 as ::core::ffi::c_int
        {
            PlayRoomMusicTrackAfterAFrames(0x168 as uint16, game);
        }
    } else {
        QueueMusic_DelayedY(1 as uint16, 0xe as uint16, game);
    }
    if *((&raw mut game.g_ram as *mut uint8).offset(0xdec) as *mut uint16) as ::core::ffi::c_int
        - 359 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let ref mut fresh0 = *((&raw mut game.g_ram as *mut uint8).offset(0xdec) as *mut uint16);
        *fresh0 = (*fresh0).wrapping_add(1);
        return 0 as uint8;
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa24) as *mut uint16) =
            *((&raw mut game.g_ram as *mut uint8).offset(0xa20) as *mut uint16);
        *((&raw mut game.g_ram as *mut uint8).offset(0xa26) as *mut uint16) =
            *((&raw mut game.g_ram as *mut uint8).offset(0xa22) as *mut uint16);
        *((&raw mut game.g_ram as *mut uint8).offset(0xa20) as *mut uint16) =
            *((&raw mut game.g_ram as *mut uint8).offset(0xa1c) as *mut uint16);
        *((&raw mut game.g_ram as *mut uint8).offset(0xa22) as *mut uint16) =
            *((&raw mut game.g_ram as *mut uint8).offset(0xa1e) as *mut uint16);
        *((&raw mut game.g_ram as *mut uint8).offset(0xdec) as *mut uint16) = 0 as uint16;
        return 1 as uint8;
    };
}
pub unsafe fn Unused_SamusTileViewer(game: &mut Game) {
    DrawSamusSpritemap(0x182 as uint16, 0x40 as uint16, 0x40 as uint16, game);
    DrawSamusSpritemap(0x183 as uint16, 0xc0 as uint16, 0x40 as uint16, game);
    DrawSamusSpritemap(0x184 as uint16, 0x80 as uint16, 0x60 as uint16, game);
    DrawSamusSpritemap(0x185 as uint16, 0x80 as uint16, 0x50 as uint16, game);
}
pub unsafe fn DrawSamusSuitExploding(game: &mut Game) {
    let mut r18: uint16 = (if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e)
        as ::core::ffi::c_int
        == 4 as ::core::ffi::c_int
    {
        *((&raw mut game.g_ram as *mut uint8).offset(0xde4) as *mut uint16) as ::core::ffi::c_int
            + 2085 as ::core::ffi::c_int
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xde4) as *mut uint16) as ::core::ffi::c_int
            + 2076 as ::core::ffi::c_int
    }) as uint16;
    DrawSamusSpritemap(
        r18,
        *((&raw mut game.g_ram as *mut uint8).offset(0xaf6) as *mut uint16),
        *((&raw mut game.g_ram as *mut uint8).offset(0xafa) as *mut uint16),
        game,
    );
}
