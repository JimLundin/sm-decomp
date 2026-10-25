pub use crate::game::prelude::*;
use crate::game::session_runtime::player_command as CallSomeSamusCode;

pub unsafe fn RoomCode_CeresElevatorShaft(game: &mut Game) {
    unsafe {
        let mut v1: i16 = 0;
        if (word!(game.ceres_status) & 0x8000) != 0 {
            if (112i16).wrapping_sub((word!(game.samus_x_pos)) as i16) < 0
                && (144i16).wrapping_sub((word!(game.samus_x_pos)) as i16) >= 0
                && word!(game.samus_y_pos) as i32 - 128i32 & 0x8000 != 0
                && word!(game.samus_y_pos) as i32 - 75i32 & 0x8000 == 0
                && word!(game.samus_y_speed) == 0
                && word!(game.samus_y_subspeed) == 0
                && word!(game.game_state) as i32 == kGameState_8_MainGameplay as i32
            {
                CallSomeSamusCode(2, game);
                word! { game, 0; screen_fade_delay, screen_fade_counter }
                word!(game.game_state = 32);
            }
            crate::update_place! { unaligned_word fresh0 = &raw mut (*(ram_ptr!(game, u16; 0x7e1, 2))) => (u16::from_le(fresh0.read_unaligned())).wrapping_sub(1); }
            if *(ram_ptr!(game, i16; 0x7e1, 2)) < 0 {
                let v0: i32 = ((6u16).wrapping_mul(*(ram_ptr!(game, u16; 0x7e1)))) as i32 >> 1;
                *(ram_ptr!(game, u16; 0x7e1, 2)) =
                    crate::rtl::rom_table_word!(game, 0x89ad5f, v0 as isize);
                word!(
                    game.reg_M7B = crate::rtl::rom_table_word!(game, 0x89ad5f, (v0 + 1) as isize)
                );
                word!(game.reg_M7C = -(word!(game.reg_M7B) as i32) as u16);
                word!(
                    game.reg_M7A = crate::rtl::rom_table_word!(game, 0x89ad5f, (v0 + 2) as isize)
                );
                word!(game.reg_M7D = word!(game.reg_M7A));
                if *(ram_ptr!(game, i16; 0x7e1)) < 0 {
                    v1 = ((*(ram_ptr!(game, u16; 0x7e1))) as i16).wrapping_sub(1);
                    if *(ram_ptr!(game, u16; 0x7e1)) == 0x8001 {
                        v1 = 0;
                    }
                } else {
                    v1 = ((*(ram_ptr!(game, u16; 0x7e1))) as i16).wrapping_add(1);
                    if *(ram_ptr!(game, u16; 0x7e1)) == 67 {
                        v1 = -32700i16;
                    }
                }
                *(ram_ptr!(game, u16; 0x7e1)) = v1 as u16;
            }
        }
    }
}
