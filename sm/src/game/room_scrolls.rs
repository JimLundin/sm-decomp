//! Room scroll definitions, camera following and their shared executors.
//! Dispatch identities retain the full address; cell writes retain their order.
#![forbid(unsafe_code)]
mod camera;
pub(crate) use camera::{calculate_blocks, calculate_scrolls, debug_position, layer2_position};
pub(crate) mod door;

use crate::game::display_refresh::{ScrollingDmaLayout, queue_scrolling_tiles};
use crate::game::room_entry::content::Axis as CameraAxis;
use crate::game_state::Game;

#[derive(Clone, Copy)]
#[repr(u8)]
enum Scroll {
    Red = 0,
    Blue = 1,
    Green = 2,
}
use Scroll::{Blue, Green, Red};

/// Returns false for other door effects without changing RAM.
pub(super) fn apply(address: u32, game: &mut Game) -> bool {
    let cells: &[(u8, Scroll)] = match address {
        0x8fb981 | 0x8fba21 | 0x8fbe1a => &[(6, Green)],
        0x8fb98c | 0x8fc0d3 => &[(0, Blue)],
        0x8fb997 => &[(19, Blue)],
        // The old "4Blue_8Green" callback actually set cell 4 red.
        0x8fb9a2 => &[(4, Red), (8, Green)],
        0x8fb9b3 => &[(8, Red), (9, Red), (10, Red), (11, Red)],
        0x8fb9ca => &[
            (2, Red),
            (3, Red),
            (4, Red),
            (5, Red),
            (11, Red),
            (12, Red),
            (13, Red),
            (17, Red),
        ],
        0x8fb9f1 => &[(1, Green), (4, Green)],
        0x8fba00 | 0x8fbd8a => &[(2, Blue)],
        0x8fba0b => &[(23, Blue)],
        0x8fba16 => &[(4, Blue)],
        0x8fba2c => &[(3, Green)],
        0x8fbd07 => &[(24, Green), (28, Green)],
        0x8fbd16 => &[(5, Blue), (6, Blue)],
        0x8fbd25 => &[(29, Blue)],
        0x8fbd30 => &[(2, Green), (3, Green)],
        0x8fbd3f => &[(0, Red), (1, Green)],
        0x8fbd50 => &[(11, Green)],
        0x8fbd5b => &[(28, Red), (29, Blue)],
        0x8fbd6c | 0x8fe3a3 => &[(4, Red)],
        0x8fbd77 => &[(32, Green), (36, Green), (37, Green)],
        0x8fbd95 | 0x8fc0a2 | 0x8fe1f3 => &[(0, Green)],
        0x8fbda0 => &[(6, Green), (7, Green)],
        0x8fbdaf => &[(1, Blue), (2, Red)],
        0x8fbdc0 => &[(1, Blue), (3, Red)],
        0x8fbdd1 | 0x8fe345 => &[(0, Red), (4, Blue)],
        0x8fbde2 => &[(2, Blue), (3, Blue)],
        0x8fbdf1 | 0x8fe4c0 => &[(0, Green), (1, Green)],
        0x8fbe00 => &[(1, Green)],
        0x8fbe0b => &[(15, Green), (18, Green)],
        0x8fbe25 => &[(0, Green), (1, Blue)],
        0x8fbe36 => &[(2, Green)],
        0x8fbf9e => &[(3, Red), (4, Red), (6, Blue), (7, Blue), (8, Blue)],
        0x8fbfbb => &[(1, Blue), (2, Blue), (3, Blue), (4, Green), (6, Red)],
        0x8fbfda => &[(0, Blue), (1, Blue)],
        0x8fbfe9 => &[(1, Red), (0, Blue)],
        0x8fbffa => &[(10, Green)],
        0x8fc016 | 0x8fe3d9 => &[(0, Green), (2, Green)],
        0x8fc025 => &[(6, Blue), (7, Blue), (8, Red)],
        0x8fc03a => &[(2, Red), (3, Blue)],
        0x8fc04b => &[(7, Green)],
        0x8fc056 => &[(1, Red), (2, Blue)],
        0x8fc067 => &[(3, Red), (0, Blue)],
        0x8fc078 => &[(1, Blue), (4, Red)],
        0x8fc089 => &[(0, Blue), (1, Red), (2, Red), (3, Red)],
        0x8fc0ad => &[(0, Blue), (1, Blue), (4, Red)],
        0x8fc0c2 => &[(0, Blue), (3, Red)],
        0x8fc0de => &[(0, Blue), (1, Red)],
        0x8fc0ef => &[(24, Blue)],
        0x8fc0fa => &[(3, Red), (2, Blue)],
        0x8fc10b => &[(14, Red)],
        0x8fe1e8 | 0x8fe38d => &[(1, Blue)],
        0x8fe1fe => &[(3, Red), (4, Blue)],
        0x8fe20f => &[(41, Blue)],
        0x8fe21a => &[(40, Green), (46, Green)],
        0x8fe229 => &[(6, Red), (7, Red), (8, Red), (9, Red), (10, Red), (11, Red)],
        0x8fe318 => &[(10, Red), (11, Blue)],
        0x8fe356 => &[(0, Red), (1, Blue)],
        0x8fe367 => &[(9, Red), (10, Blue)],
        0x8fe378 => &[(0, Red), (2, Red), (1, Blue)],
        0x8fe398 => &[(6, Blue)],
        0x8fe3b9 => &[(4, Red), (7, Red)],
        0x8fe3c8 => &[(2, Red), (1, Blue)],
        0x8fe4cf => &[(24, Blue), (25, Red)],
        _ => return false,
    };
    for &(cell, color) in cells {
        game.g_ram[0xcd20 + usize::from(cell)] = color as u8;
    }
    true
}

// Camera effects keep their legacy publication order until layer state is owned.
use crate::game::{
    player_collision::operations::Axis, player_movement::speed::record_camera_distance,
    session_runtime::Runtime,
};
use crate::ram_map::{word, words as fields};
use crate::rtl::{Unreachable, words};
use camera::{auto_x, auto_y, trigger_down, trigger_left, trigger_right, trigger_up};
// Native split motion reads low then high and writes low then high.
fn add_native_motion(high: usize, low: usize, amount: u32, game: &mut Game) {
    let low_word = u16::from_ne_bytes([game.g_ram[low], game.g_ram[low + 1]]);
    let high_word = u16::from_ne_bytes([game.g_ram[high], game.g_ram[high + 1]]);
    let position = u32::from(low_word) | (u32::from(high_word) << 16);
    let next = position.wrapping_add(amount);
    word_transfers!(game, ascending, 1;
        native(low) <- value(next as u16),
        native(high) <- value((next >> 16) as u16),
    );
}

pub(crate) fn advance_camera(game: &mut Game) {
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
        auto_x(game);
        auto_y(game);
    } else {
        record_camera_distance(Runtime::new(game).session_mut(), Axis::X);
        follow_x(game);
        record_camera_distance(Runtime::new(game).session_mut(), Axis::Y);
        follow_y(game);
    }
    match word!(game.scrolling_finished_hook) {
        0 => {}
        0x9589 => {
            if word!(game.layer1_y_pos) <= 0x1d0 {
                word!(game.layer1_y_pos = 464);
            }
        }
        _ => {
            Unreachable(game);
        }
    }
    word!(game.samus_prev_x_pos = word!(game.samus_x_pos));
    word!(game.samus_prev_x_subpos = word!(game.samus_x_subpos));
    word!(game.samus_prev_y_pos = word!(game.samus_y_pos));
    word!(game.samus_prev_y_subpos = word!(game.samus_y_subpos));
}
macro_rules! chase_ideal_layer_position {
    (
        $game:ident, $ideal_position:ident, $position:ident, $subposition:ident,
        $movement:ident, $movement_fraction:ident, $scroll_negative:path,
        $scroll_positive:path $(,)?
    ) => {
        if word!($game.$ideal_position) != word!($game.$position) {
            let negative = (word!($game.$ideal_position) as i16)
                .wrapping_sub(word!($game.$position) as i16)
                < 0;
            let amount = if negative {
                -(words!(i32, $game.$movement, $movement_fraction)) as u32
            } else {
                words!($game.$movement, $movement_fraction)
            };
            add_native_motion(fields::$position, fields::$subposition, amount, $game);
            if negative {
                $scroll_negative($game);
            } else {
                $scroll_positive($game);
            }
        }
    };
}

fn follow_x(game: &mut Game) {
    static kSamus_HandleScroll_X_FaceLeft: [u16; 4] = [0xa0, 0x50, 0x20, 0xe0];
    static kSamus_HandleScroll_X_FaceRight: [u16; 4] = [0x60, 0x40, 0x20, 0xe0];
    if word!(game.samus_prev_x_pos) == word!(game.samus_x_pos) {
        auto_x(game);
        return;
    }
    let distances = if (word!(game.knockback_dir) != 0
        || game.g_ram[0xa1f] == 16
        || word!(game.samus_x_accel_mode) == 1) as i32
        ^ (game.g_ram[0xa1e] != 4) as i32
        != 0
    {
        &kSamus_HandleScroll_X_FaceRight
    } else {
        &kSamus_HandleScroll_X_FaceLeft
    };
    word!(
        game.ideal_layer1_xpos = word!(game.samus_x_pos)
            .wrapping_sub(distances[(word!(game.camera_distance_index) >> 1) as usize])
    );
    chase_ideal_layer_position!(
        game,
        ideal_layer1_xpos,
        layer1_x_pos,
        layer1_x_subpos,
        absolute_moved_last_frame_x,
        absolute_moved_last_frame_x_fract,
        trigger_left,
        trigger_right
    );
}
fn follow_y(game: &mut Game) {
    if word!(game.samus_prev_y_pos) == word!(game.samus_y_pos) {
        auto_y(game);
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
            trigger_up,
            trigger_down
        );
    };
}

pub(crate) fn fill_viewport(game: &mut Game, mut flush: impl FnMut(&mut Game)) {
    let mut v1: i32 = (game.g_ram[0x58] as i32 - game.g_ram[0x59] as i32) << 8;
    word!(game.size_of_bg2 = (v1 & 0xf800) as u16);
    calculate_blocks(game);
    let mut v2: i32 = 0;
    do_until! { v3 == 16 ;
        let mut v3: i32 = v2;
        word!(game.blocks_to_update_x_block = word!(game.layer1_x_block));
        word!(game.blocks_to_update_y_block = word!(game.layer1_y_block));
        word!(game.vram_blocks_to_update_x_block = word!(game.bg1_x_block));
        word!(game.vram_blocks_to_update_y_block = word!(game.bg1_y_block));
        queue_scrolling_tiles(0x0, ScrollingDmaLayout::Column, game);
        if game.g_ram[0x91b] & 1 == 0 {
            word!(game.blocks_to_update_x_block = word!(game.layer2_x_block));
            word!(game.blocks_to_update_y_block = word!(game.layer2_y_block));
            word!(game.vram_blocks_to_update_x_block = word!(game.bg2_x_block));
            word!(game.vram_blocks_to_update_y_block = word!(game.bg2_y_block));
            queue_scrolling_tiles(0x1c, ScrollingDmaLayout::Column, game);
        }
        flush(game);
        word!(game.layer1_x_block += 1);
        word!(game.bg1_x_block += 1);
        word!(game.layer2_x_block += 1);
        word!(game.bg2_x_block += 1);
        v2 += 1;
    }
}

pub(crate) fn update_map(game: &mut Game) {
    calculate_scrolls(game);
    update_backgrounds(game);
}

pub(crate) fn update_scrolls(game: &mut Game) {
    if word!(game.time_is_frozen_flag) == 0 {
        word!(game.reg_BG1HOFS = (word!(game.bg1_x_offset)).wrapping_add(word!(game.layer1_x_pos)));
        word!(game.reg_BG1VOFS = (word!(game.bg1_y_offset)).wrapping_add(word!(game.layer1_y_pos)));
        if layer2_position(CameraAxis::X, game) == 0 {
            word!(
                game.reg_BG2HOFS =
                    (word!(game.bg2_x_scroll)).wrapping_add(word!(game.layer2_x_pos))
            );
        }
        if layer2_position(CameraAxis::Y, game) == 0 {
            word!(
                game.reg_BG2VOFS =
                    (word!(game.bg2_y_scroll)).wrapping_add(word!(game.layer2_y_pos))
            );
        }
        update_backgrounds(game);
    }
}

pub(crate) fn update_backgrounds(game: &mut Game) {
    calculate_blocks(game);
    // RAM's coordinates are two bytes apart; BG2's fields follow BG1 by four.
    for (axis, layout) in [
        (0, ScrollingDmaLayout::Column),
        (2, ScrollingDmaLayout::Row),
    ] {
        for (layer, parameter) in [(0, 0), (4, 0x1c)] {
            if layer != 0 && game.g_ram[0x91b + axis / 2] & 1 != 0 {
                continue;
            }
            let current = fields::layer1_x_block + layer + axis;
            let previous = fields::previous_layer1_x_block + layer + axis;
            let negative = (word!(@read game, current) as i16)
                .wrapping_sub(word!(@read game, previous) as i16)
                < 0;
            if word!(@read game, current) != word!(@read game, previous) {
                word!(@write game, previous, word!(@read game, current));
                // Columns retain edges 0/16; rows retain edges 1/15.
                let row = axis as i32 / 2;
                let edge = if negative { row } else { 16 - row };
                for (coordinate, edge) in [(axis, edge), (axis ^ 2, 0)] {
                    word!(@write game, fields::blocks_to_update_x_block + coordinate,
                        (word!(@read game, fields::layer1_x_block + layer + coordinate) as i32 + edge) as u16);
                    word!(@write game, fields::vram_blocks_to_update_x_block + coordinate,
                        (word!(@read game, fields::bg1_x_block + layer + coordinate) as i32 + edge) as u16);
                }
                queue_scrolling_tiles(parameter, layout, game);
            }
        }
    }
}
