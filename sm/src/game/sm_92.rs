use crate::game::sm_80::QueueMusic_DelayedY;
use crate::game::sm_81::DrawSamusSpritemap;
use crate::game::sm_82::PlayRoomMusicTrackAfterAFrames;
use crate::game_state::Game;
use crate::ram_map::ram_ptr;
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
    let v0: u16 = (4i32 * ram_word!(game, samus_anim_frame) as i32
        + crate::rtl::rom_table_word!(game, 0x92d94e, ram_word!(game, samus_pose) as isize) as i32)
        as u16;
    let AD: *mut SamusTileAnimationDefs = get_SamusTileAnimationDefs(v0, game);
    set_ram_word!(
        game.nmi_copy_samus_top_half_src = (7i32 * (*AD).top_half_pos as i32
            + crate::rtl::rom_table_word!(game, 0x92d91e, (*AD).top_half_idx as isize) as i32)
            as u16
    );
    *(ram_ptr!(game, u16; 0x71d) as *mut u8).offset(0) = 1u8;
    if (*AD).bottom_half_idx != 255u8 {
        set_ram_word!(
            game.nmi_copy_samus_bottom_half_src = (7i32 * (*AD).bottom_half_pos as i32
                + crate::rtl::rom_table_word!(game, 0x92d938, (*AD).bottom_half_idx as isize)
                    as i32) as u16
        );
        *(ram_ptr!(game, u16; 0x71d) as *mut u8).offset(1isize) = 1u8;
    }
}
pub unsafe fn PlaySamusFanfare(game: &mut Game) -> u8 {
    if ram_word!(game, substate) != 0 {
        if ram_word!(game, substate) == 5u16 {
            PlayRoomMusicTrackAfterAFrames(0x168u16, game);
        }
    } else {
        QueueMusic_DelayedY(1u16, 0xeu16, game);
    }
    if ram_word!(game, substate) as i32 - 359i32 & 0x8000i32 != 0 {
        update_ram_word!(game.substate, wrapping_add(1));
        0u8
    } else {
        set_ram_word!(game.samus_last_different_pose = ram_word!(game, samus_prev_pose));
        *(ram_ptr!(game, u16; 0xa26)) = *(ram_ptr!(game, u16; 0xa22));
        set_ram_word!(game.samus_prev_pose = ram_word!(game, samus_pose));
        *(ram_ptr!(game, u16; 0xa22)) = *(ram_ptr!(game, u16; 0xa1e));
        set_ram_word!(game.substate = 0u16);
        1u8
    }
}
pub unsafe fn Unused_SamusTileViewer(game: &mut Game) {
    DrawSamusSpritemap(0x182u16, 0x40u16, 0x40u16, game);
    DrawSamusSpritemap(0x183u16, 0xc0u16, 0x40u16, game);
    DrawSamusSpritemap(0x184u16, 0x80u16, 0x60u16, game);
    DrawSamusSpritemap(0x185u16, 0x80u16, 0x50u16, game);
}
pub unsafe fn DrawSamusSuitExploding(game: &mut Game) {
    let r18: u16 = (if *ram_ptr!(game; 0xa1e) == 4u8 {
        ram_word!(game, g_word_7E0DE4) as i32 + 2085i32
    } else {
        ram_word!(game, g_word_7E0DE4) as i32 + 2076i32
    }) as u16;
    DrawSamusSpritemap(
        r18,
        ram_word!(game, samus_x_pos),
        ram_word!(game, samus_y_pos),
        game,
    );
}
