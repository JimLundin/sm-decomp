use crate::game::sm_80::QueueMusic_DelayedY;
use crate::game::sm_81::DrawSamusSpritemap;
use crate::game::sm_82::PlayRoomMusicTrackAfterAFrames;
use crate::game_state::Game;
use crate::ram_map::ram_ptr;
use crate::ram_map::word;
use crate::rtl::RomPtr;

crate::typed_struct! {
    #[derive(Copy, Clone)]
    #[repr(C, packed)]
    pub struct SamusTileAnimationDefs {
        pub top_half_idx, top_half_pos, bottom_half_idx, bottom_half_pos: u8;
    }
}
#[inline]
unsafe fn get_SamusTileAnimationDefs(a: u16, game: &mut Game) -> *mut SamusTileAnimationDefs {
    RomPtr((0x920000u32) | ((a) as u32), game) as *mut SamusTileAnimationDefs
}
pub unsafe fn SetSamusTilesDefsForCurAnim(game: &mut Game) {
    let v0: u16 = (4 * word!(game.samus_anim_frame) as i32
        + crate::rtl::rom_table_word!(game, 0x92d94e, word!(game.samus_pose) as isize) as i32)
        as u16;
    let AD = get_SamusTileAnimationDefs(v0, game);
    word!(
        game.nmi_copy_samus_top_half_src = (7 * (*AD).top_half_pos as i32
            + crate::rtl::rom_table_word!(game, 0x92d91e, (*AD).top_half_idx as isize) as i32)
            as u16
    );
    byte!(@pointer ram_ptr!(game, u16; 0x71d); 0) = 1;
    if (*AD).bottom_half_idx != 255 {
        word!(
            game.nmi_copy_samus_bottom_half_src = (7 * (*AD).bottom_half_pos as i32
                + crate::rtl::rom_table_word!(game, 0x92d938, (*AD).bottom_half_idx as isize)
                    as i32) as u16
        );
        byte!(@pointer ram_ptr!(game, u16; 0x71d); 1) = 1;
    }
}
pub unsafe fn PlaySamusFanfare(game: &mut Game) -> u8 {
    if word!(game.substate) != 0 {
        if word!(game.substate) == 5 {
            PlayRoomMusicTrackAfterAFrames(0x168, game);
        }
    } else {
        QueueMusic_DelayedY(1, 0xe, game);
    }
    if word!(game.substate) as i32 - 359i32 & 0x8000 != 0 {
        word!(game.substate, wrapping_add(1));
        0
    } else {
        crate::game::samus_pose::record_pose_history(game);
        word!(game.substate = 0);
        1
    }
}
pub unsafe fn DrawSamusSuitExploding(game: &mut Game) {
    let r18: u16 = (if *ram_ptr!(game; 0xa1e) == 4 {
        word!(game.g_word_7E0DE4) as i32 + 2085
    } else {
        word!(game.g_word_7E0DE4) as i32 + 2076
    }) as u16;
    DrawSamusSpritemap(r18, word!(game.samus_x_pos), word!(game.samus_y_pos), game);
}
