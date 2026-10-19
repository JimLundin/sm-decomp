use crate::game::sm_80::QueueMusic_DelayedY;
use crate::game::sm_81::DrawSamusSpritemap;
use crate::game::sm_82::PlayRoomMusicTrackAfterAFrames;
use crate::game_state::Game;
use crate::ram_map::ram_word;
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
unsafe fn get_SamusTileAnimationDefs(a: uint16, game: &mut Game) -> *mut SamusTileAnimationDefs {
    RomPtr((0x920000u32) | ((a) as u32), game) as *mut SamusTileAnimationDefs
}
pub unsafe fn SetSamusTilesDefsForCurAnim(game: &mut Game) {
    let v0: uint16 = (4 as ::core::ffi::c_int
        * ram_word!(game, samus_anim_frame) as ::core::ffi::c_int
        + *(game.g_rom.offset(crate::rtl::lorom_offset(0x92d94e)) as *const uint8 as *mut uint16)
            .offset(ram_word!(game, samus_pose) as isize) as ::core::ffi::c_int)
        as uint16;
    let AD: *mut SamusTileAnimationDefs = get_SamusTileAnimationDefs(v0, game);
    *((&raw mut game.g_ram as *mut uint8).offset(0x71f) as *mut uint16) = (7 as ::core::ffi::c_int
        * (*AD).top_half_pos as ::core::ffi::c_int
        + *(game.g_rom.offset(crate::rtl::lorom_offset(0x92d91e)) as *const uint8 as *mut uint16)
            .offset((*AD).top_half_idx as isize) as ::core::ffi::c_int)
        as uint16;
    *((&raw mut game.g_ram as *mut uint8).offset(0x71d) as *mut uint16 as *mut uint8).offset(0) =
        1 as uint8;
    if (*AD).bottom_half_idx != 255u8 {
        *((&raw mut game.g_ram as *mut uint8).offset(0x721) as *mut uint16) =
            (7 as ::core::ffi::c_int * (*AD).bottom_half_pos as ::core::ffi::c_int
                + *(game.g_rom.offset(crate::rtl::lorom_offset(0x92d938)) as *const uint8
                    as *mut uint16)
                    .offset((*AD).bottom_half_idx as isize) as ::core::ffi::c_int)
                as uint16;
        *((&raw mut game.g_ram as *mut uint8).offset(0x71d) as *mut uint16 as *mut uint8).offset(
            (::core::mem::size_of::<uint16>())
                .wrapping_div(::core::mem::size_of::<uint8>())
                .wrapping_sub(1_usize) as isize,
        ) = 1 as uint8;
    }
}
pub unsafe fn PlaySamusFanfare(game: &mut Game) -> uint8 {
    if ram_word!(game, substate) != 0 {
        if ram_word!(game, substate) == 5u16 {
            PlayRoomMusicTrackAfterAFrames(0x168 as uint16, game);
        }
    } else {
        QueueMusic_DelayedY(1 as uint16, 0xe as uint16, game);
    }
    if ram_word!(game, substate) as ::core::ffi::c_int - 359 as ::core::ffi::c_int
        & 0x8000 as ::core::ffi::c_int
        != 0
    {
        let ref mut fresh0 = *((&raw mut game.g_ram as *mut uint8).offset(0xdec) as *mut uint16);
        *fresh0 = (*fresh0).wrapping_add(1);
        0 as uint8
    } else {
        *((&raw mut game.g_ram as *mut uint8).offset(0xa24) as *mut uint16) =
            ram_word!(game, samus_prev_pose);
        *((&raw mut game.g_ram as *mut uint8).offset(0xa26) as *mut uint16) =
            *((&raw mut game.g_ram as *mut uint8).offset(0xa22) as *mut uint16);
        *((&raw mut game.g_ram as *mut uint8).offset(0xa20) as *mut uint16) =
            ram_word!(game, samus_pose);
        *((&raw mut game.g_ram as *mut uint8).offset(0xa22) as *mut uint16) =
            *((&raw mut game.g_ram as *mut uint8).offset(0xa1e) as *mut uint16);
        *((&raw mut game.g_ram as *mut uint8).offset(0xdec) as *mut uint16) = 0 as uint16;
        1 as uint8
    }
}
pub unsafe fn Unused_SamusTileViewer(game: &mut Game) {
    DrawSamusSpritemap(0x182 as uint16, 0x40 as uint16, 0x40 as uint16, game);
    DrawSamusSpritemap(0x183 as uint16, 0xc0 as uint16, 0x40 as uint16, game);
    DrawSamusSpritemap(0x184 as uint16, 0x80 as uint16, 0x60 as uint16, game);
    DrawSamusSpritemap(0x185 as uint16, 0x80 as uint16, 0x50 as uint16, game);
}
pub unsafe fn DrawSamusSuitExploding(game: &mut Game) {
    let r18: uint16 = (if *(&raw mut game.g_ram as *mut uint8).offset(0xa1e) == 4u8 {
        ram_word!(game, g_word_7E0DE4) as ::core::ffi::c_int + 2085 as ::core::ffi::c_int
    } else {
        ram_word!(game, g_word_7E0DE4) as ::core::ffi::c_int + 2076 as ::core::ffi::c_int
    }) as uint16;
    DrawSamusSpritemap(
        r18,
        ram_word!(game, samus_x_pos),
        ram_word!(game, samus_y_pos),
        game,
    );
}
