use crate::game::player_collision::operations::Axis;
use crate::game::player_movement::speed::record_camera_distance;

pub use crate::game::prelude::*;
use crate::game::session_runtime::Runtime;

pub use crate::game::constants::bank_8d::*;
pub use crate::game::constants::bank_60::*;
pub use crate::game::constants::bank_86::*;
pub use crate::game::constants::bank_90::*;

use crate::game::sm_80::{
    HandleAutoscrolling_X, HandleAutoscrolling_Y, HandleScrollingWhenTriggeringScrollDown,
    HandleScrollingWhenTriggeringScrollLeft, HandleScrollingWhenTriggeringScrollRight,
    HandleScrollingWhenTriggeringScrollUp,
};

crate::game::callback_dispatch! {
    pub unsafe fn CallScrollingFinishedHook(ea: u32, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    [9475465] Samus_ScrollFinishedHook_SporeSpawnFight [] {
        if word!(game.layer1_y_pos) <= 0x1d0 {
            word!(game.layer1_y_pos = 464);
        }
    };
}
pub unsafe fn MainScrollingRoutine(game: &mut Game) {
    unsafe {
        if word!(game.slow_grabble_scrolling_flag) != 0 {
            if (word!(game.samus_x_pos) & 0x8000) == 0 {
                let mut v0: u16 = (word!(game.samus_x_pos)).wrapping_sub(word!(game.layer1_x_pos));
                if word!(game.samus_x_pos) < word!(game.layer1_x_pos) || v0 < 0x60 {
                    word!(game.layer1_x_pos -= 3);
                } else if v0 >= 0xa0 {
                    word!(game.layer1_x_pos += 3);
                }
                if (word!(game.samus_y_pos) & 0x8000) == 0 {
                    let v1: u16 = (word!(game.samus_y_pos)).wrapping_sub(word!(game.layer1_y_pos));
                    if word!(game.samus_y_pos) < word!(game.layer1_y_pos) || v1 < 0x70 {
                        word!(game.layer1_y_pos -= 3);
                    } else if v1 >= 0x90 {
                        word!(game.layer1_y_pos += 3);
                    }
                }
            }
            HandleAutoscrolling_X(game);
            HandleAutoscrolling_Y(game);
        } else {
            record_camera_distance(Runtime::new(game).session_mut(), Axis::X);
            Samus_HandleScroll_X(game);
            record_camera_distance(Runtime::new(game).session_mut(), Axis::Y);
            Samus_HandleScroll_Y(game);
        }
        if word!(game.scrolling_finished_hook) != 0 {
            CallScrollingFinishedHook(
                ((word!(game.scrolling_finished_hook)) as u32) | (0x900000u32),
                game,
            );
        }
        word!(game.samus_prev_x_pos = word!(game.samus_x_pos));
        word!(game.samus_prev_x_subpos = word!(game.samus_x_subpos));
        word!(game.samus_prev_y_pos = word!(game.samus_y_pos));
        word!(game.samus_prev_y_subpos = word!(game.samus_y_subpos));
    }
}
macro_rules! chase_ideal_layer_position {
    (
        $game:ident, $ideal_position:ident, $position:ident, $subposition:ident,
        $movement:ident, $movement_fraction:ident, $scroll_negative:path,
        $scroll_positive:path $(,)?
    ) => {
        if word!($game.$ideal_position) != word!($game.$position) {
            if ((word!($game.$ideal_position)) as i16).wrapping_sub((word!($game.$position)) as i16)
                < 0
            {
                words!(
                    $game.$position,
                    $subposition += -(words!(i32, $game.$movement, $movement_fraction)) as u32
                );
                $scroll_negative($game);
            } else {
                words!(
                    $game.$position,
                    $subposition += words!($game.$movement, $movement_fraction)
                );
                $scroll_positive($game);
            }
        }
    };
}

pub unsafe fn Samus_HandleScroll_X(game: &mut Game) {
    unsafe {
        static kSamus_HandleScroll_X_FaceLeft: [u16; 4] = [0xa0, 0x50, 0x20, 0xe0];
        static kSamus_HandleScroll_X_FaceRight: [u16; 4] = [0x60, 0x40, 0x20, 0xe0];
        if word!(game.samus_prev_x_pos) == word!(game.samus_x_pos) {
            HandleAutoscrolling_X(game);
            return;
        }
        if (word!(game.knockback_dir) != 0
            || *ram_ptr!(game; 0xa1f) == 16
            || word!(game.samus_x_accel_mode) == 1) as i32
            ^ (*ram_ptr!(game; 0xa1e) != 4) as i32
            != 0
        {
            word!(
                game.ideal_layer1_xpos = (word!(game.samus_x_pos)).wrapping_sub(
                    kSamus_HandleScroll_X_FaceRight
                        [(word!(game.camera_distance_index) >> 1) as usize],
                )
            );
        } else {
            word!(
                game.ideal_layer1_xpos = (word!(game.samus_x_pos)).wrapping_sub(
                    kSamus_HandleScroll_X_FaceLeft
                        [(word!(game.camera_distance_index) >> 1) as usize],
                )
            );
        }
        chase_ideal_layer_position!(
            game,
            ideal_layer1_xpos,
            layer1_x_pos,
            layer1_x_subpos,
            absolute_moved_last_frame_x,
            absolute_moved_last_frame_x_fract,
            HandleScrollingWhenTriggeringScrollLeft,
            HandleScrollingWhenTriggeringScrollRight
        );
    }
}
pub unsafe fn Samus_HandleScroll_Y(game: &mut Game) {
    unsafe {
        if word!(game.samus_prev_y_pos) == word!(game.samus_y_pos) {
            HandleAutoscrolling_Y(game);
        } else {
            if word!(game.samus_y_dir) == 1 {
                word!(
                    game.ideal_layer1_ypos =
                        (word!(game.samus_y_pos)).wrapping_sub(word!(game.down_scroller))
                );
            } else {
                word!(
                    game.ideal_layer1_ypos =
                        (word!(game.samus_y_pos)).wrapping_sub(word!(game.up_scroller))
                );
            }
            chase_ideal_layer_position!(
                game,
                ideal_layer1_ypos,
                layer1_y_pos,
                layer1_y_subpos,
                absolute_moved_last_frame_y,
                absolute_moved_last_frame_y_fract,
                HandleScrollingWhenTriggeringScrollUp,
                HandleScrollingWhenTriggeringScrollDown
            );
        };
    }
}
pub(crate) use crate::game::player_projectiles::prepare as HandleControllerInputForGamePhysics;

pub fn Samus_SetRadius(game: &mut Game) {
    let pose_offset = usize::from(game.samus_pose()) * 8;
    let radius = u16::from(game.read_rom_byte(0x91b629, pose_offset + 6));
    game.g_ram[0xb00..0xb02].copy_from_slice(&radius.to_le_bytes());
    game.g_ram[0xafe..0xb00].copy_from_slice(&5u16.to_le_bytes());
}
pub fn Samus_GetBottom_R18(game: &Game) -> u16 {
    let y = game.samus_y_pos();
    // SamusPoseParams has eight bytes, with y_radius at byte 6.
    let pose_offset = usize::from(game.samus_pose()) * 8;
    let radius = game.read_rom_byte(0x91b629, pose_offset + 6);
    y.wrapping_add(u16::from(radius)).wrapping_sub(1)
}
pub fn Samus_PostGrappleCollisionDetect(game: &mut Game) {
    crate::game::player_collision::operations::settle_after_grapple(
        &mut crate::game::session_runtime::Runtime::new(game),
    );
}
