use crate::game::sm_80::QueueMusic_DelayedY;
use crate::game::sm_81::DrawSamusSpritemap;
use crate::game::sm_82::PlayRoomMusicTrackAfterAFrames;
use crate::game_state::Game;
use crate::ram_map::{ram_word, set_ram_word, update_ram_word};
use crate::rtl::RomPtr;

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct SamusTileAnimationDefs {
    pub top_half_idx: u8,
    pub top_half_pos: u8,
    pub bottom_half_idx: u8,
    pub bottom_half_pos: u8,
}
#[inline]
unsafe fn get_SamusTileAnimationDefs(a: u16, game: &mut Game) -> *mut SamusTileAnimationDefs {
    RomPtr((0x920000u32) | ((a) as u32), game) as *mut SamusTileAnimationDefs
}
pub unsafe fn SetSamusTilesDefsForCurAnim(game: &mut Game) {
    let v0: u16 = (4 as i32 * ram_word!(game, samus_anim_frame) as i32
        + *(game.g_rom.offset(crate::rtl::lorom_offset(0x92d94e)) as *const u8 as *mut u16)
            .offset(ram_word!(game, samus_pose) as isize) as i32) as u16;
    let AD: *mut SamusTileAnimationDefs = get_SamusTileAnimationDefs(v0, game);
    set_ram_word!(
        game.nmi_copy_samus_top_half_src = (7 as i32 * (*AD).top_half_pos as i32
            + *(game.g_rom.offset(crate::rtl::lorom_offset(0x92d91e)) as *const u8 as *mut u16)
                .offset((*AD).top_half_idx as isize) as i32)
            as u16
    );
    *((&raw mut game.g_ram as *mut u8).offset(0x71d) as *mut u16 as *mut u8).offset(0) = 1 as u8;
    if (*AD).bottom_half_idx != 255u8 {
        set_ram_word!(
            game.nmi_copy_samus_bottom_half_src = (7 as i32 * (*AD).bottom_half_pos as i32
                + *(game.g_rom.offset(crate::rtl::lorom_offset(0x92d938)) as *const u8 as *mut u16)
                    .offset((*AD).bottom_half_idx as isize) as i32)
                as u16
        );
        *((&raw mut game.g_ram as *mut u8).offset(0x71d) as *mut u16 as *mut u8).offset(
            (::core::mem::size_of::<u16>())
                .wrapping_div(::core::mem::size_of::<u8>())
                .wrapping_sub(1_usize) as isize,
        ) = 1 as u8;
    }
}
pub unsafe fn PlaySamusFanfare(game: &mut Game) -> u8 {
    if ram_word!(game, substate) != 0 {
        if ram_word!(game, substate) == 5u16 {
            PlayRoomMusicTrackAfterAFrames(0x168 as u16, game);
        }
    } else {
        QueueMusic_DelayedY(1 as u16, 0xe as u16, game);
    }
    if ram_word!(game, substate) as i32 - 359 as i32 & 0x8000 as i32 != 0 {
        update_ram_word!(game.substate, wrapping_add(1));
        0 as u8
    } else {
        set_ram_word!(game.samus_last_different_pose = ram_word!(game, samus_prev_pose));
        *((&raw mut game.g_ram as *mut u8).offset(0xa26) as *mut u16) =
            *((&raw mut game.g_ram as *mut u8).offset(0xa22) as *mut u16);
        set_ram_word!(game.samus_prev_pose = ram_word!(game, samus_pose));
        *((&raw mut game.g_ram as *mut u8).offset(0xa22) as *mut u16) =
            *((&raw mut game.g_ram as *mut u8).offset(0xa1e) as *mut u16);
        set_ram_word!(game.substate = 0 as u16);
        1 as u8
    }
}
pub unsafe fn Unused_SamusTileViewer(game: &mut Game) {
    DrawSamusSpritemap(0x182 as u16, 0x40 as u16, 0x40 as u16, game);
    DrawSamusSpritemap(0x183 as u16, 0xc0 as u16, 0x40 as u16, game);
    DrawSamusSpritemap(0x184 as u16, 0x80 as u16, 0x60 as u16, game);
    DrawSamusSpritemap(0x185 as u16, 0x80 as u16, 0x50 as u16, game);
}
pub unsafe fn DrawSamusSuitExploding(game: &mut Game) {
    let r18: u16 = (if *(&raw mut game.g_ram as *mut u8).offset(0xa1e) == 4u8 {
        ram_word!(game, g_word_7E0DE4) as i32 + 2085 as i32
    } else {
        ram_word!(game, g_word_7E0DE4) as i32 + 2076 as i32
    }) as u16;
    DrawSamusSpritemap(
        r18,
        ram_word!(game, samus_x_pos),
        ram_word!(game, samus_y_pos),
        game,
    );
}
