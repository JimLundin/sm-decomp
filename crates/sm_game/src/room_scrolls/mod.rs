//! Room scroll definitions, camera following and their shared executors.
//! Dispatch identities retain the full address; cell writes retain their order.
#![forbid(unsafe_code)]
mod camera;
pub use camera::{calculate_blocks, calculate_scrolls, debug_position, layer2_position};
pub mod door;

use crate::display::transfer::ScrollingLayout as ScrollingDmaLayout;

/// Room scrolling runs on the owned-Session view: the camera is
/// `Session::room.camera`, other words are read and written through the
/// view's projected bytes, and the display work is the runtime's.
pub trait Host:
    crate::session_access::SessionAccess
    + crate::runtime_access::SessionRam
    + crate::ram::CompatibilityRam
    + crate::player_position::PlayerPosition
    + crate::player_position::PlayerFacts
{
    /// The original's unreachable-state failure.
    fn unreachable(&mut self) -> bool;
    /// Queue the scrolling tile transfer for layer offset `k`.
    fn queue_scrolling_tiles(&mut self, k: u16, layout: ScrollingDmaLayout);
    /// Flush the queued tilemap transfers.
    fn flush_tilemaps(&mut self);
}
use crate::room_entry::content::Axis as CameraAxis;

#[derive(Clone, Copy)]
#[repr(u8)]
enum Scroll {
    Red = 0,
    Blue = 1,
    Green = 2,
}
use Scroll::{Blue, Green, Red};

/// Returns false for other door effects without changing RAM.
pub fn apply<H: Host + ?Sized>(address: u32, game: &mut H) -> bool {
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
        game.set_byte(0xcd20 + usize::from(cell), color as u8);
    }
    true
}

// Camera effects keep their legacy publication order until layer state is owned.
use crate::camera::set_layer1;
use crate::menus::prelude::do_until;
use crate::player_collision::operations::Axis;
use crate::ram::{word, words as fields};
use crate::ram_words as words;
use crate::word_transfers;
use camera::{auto_x, auto_y, trigger_down, trigger_left, trigger_right, trigger_up};
// Native split motion reads low then high and writes low then high.
fn add_native_motion<H: Host + ?Sized>(high: usize, low: usize, amount: u32, game: &mut H) {
    let low_word = u16::from_ne_bytes([game.byte(low), game.byte(low + 1)]);
    let high_word = u16::from_ne_bytes([game.byte(high), game.byte(high + 1)]);
    let position = u32::from(low_word) | (u32::from(high_word) << 16);
    let next = position.wrapping_add(amount);
    word_transfers!(game, ascending, 1;
        native(low) <- value(next as u16),
        native(high) <- value((next >> 16) as u16),
    );
}

pub fn advance_camera<H: Host + ?Sized>(game: &mut H) {
    if word!(game.slow_grabble_scrolling_flag) != 0 {
        if (game.player_x() & 0x8000) == 0 {
            let v0: u16 = (game.player_x()).wrapping_sub(game.session().room.camera.layer1[0]);
            if game.player_x() < game.session().room.camera.layer1[0] || v0 < 0x60 {
                set_layer1(
                    game,
                    0,
                    game.session().room.camera.layer1[0].wrapping_sub(3),
                );
            } else if v0 >= 0xa0 {
                set_layer1(
                    game,
                    0,
                    game.session().room.camera.layer1[0].wrapping_add(3),
                );
            }
            if (game.player_y() & 0x8000) == 0 {
                let v1: u16 = (game.player_y()).wrapping_sub(game.session().room.camera.layer1[1]);
                if game.player_y() < game.session().room.camera.layer1[1] || v1 < 0x70 {
                    set_layer1(
                        game,
                        1,
                        game.session().room.camera.layer1[1].wrapping_sub(3),
                    );
                } else if v1 >= 0x90 {
                    set_layer1(
                        game,
                        1,
                        game.session().room.camera.layer1[1].wrapping_add(3),
                    );
                }
            }
        }
        auto_x(game);
        auto_y(game);
    } else {
        crate::player_movement::speed::record_camera_distance(game.session_mut(), Axis::X);
        follow_x(game);
        crate::player_movement::speed::record_camera_distance(game.session_mut(), Axis::Y);
        follow_y(game);
    }
    match word!(game.scrolling_finished_hook) {
        0 => {}
        0x9589 => {
            if game.session().room.camera.layer1[1] <= 0x1d0 {
                set_layer1(game, 1, 464);
            }
        }
        _ => {
            game.unreachable();
        }
    }
    game.set_previous_player_x(game.player_x());
    game.set_previous_player_x_fraction(game.player_x_fraction());
    game.set_previous_player_y(game.player_y());
    game.set_previous_player_y_fraction(game.player_y_fraction());
}
// Layer 1's fraction and whole position form one native split motion, as
// add_native_motion: native loads of fraction then whole, stores likewise.
fn add_layer1_motion<H: Host + ?Sized>(axis: usize, amount: u32, game: &mut H) {
    let camera = game.session().room.camera;
    let low = u16::from_ne_bytes(camera.layer1_fraction[axis].to_le_bytes());
    let high = u16::from_ne_bytes(camera.layer1[axis].to_le_bytes());
    let next = (u32::from(low) | (u32::from(high) << 16)).wrapping_add(amount);
    game.session_mut().room.camera.layer1_fraction[axis] =
        u16::from_le_bytes((next as u16).to_ne_bytes());
    set_layer1(
        game,
        axis,
        u16::from_le_bytes(((next >> 16) as u16).to_ne_bytes()),
    );
}

macro_rules! chase_ideal_layer_position {
    (
        $game:ident, $axis:expr,
        $movement:ident, $movement_fraction:ident, $scroll_negative:path,
        $scroll_positive:path $(,)?
    ) => {
        let camera = $game.session().room.camera;
        if camera.ideal_layer1[$axis] != camera.layer1[$axis] {
            let negative =
                (camera.ideal_layer1[$axis] as i16).wrapping_sub(camera.layer1[$axis] as i16) < 0;
            let amount = if negative {
                -(words!(i32, $game.$movement, $movement_fraction)) as u32
            } else {
                words!($game.$movement, $movement_fraction)
            };
            add_layer1_motion($axis, amount, $game);
            if negative {
                $scroll_negative($game);
            } else {
                $scroll_positive($game);
            }
        }
    };
}

fn follow_x<H: Host + ?Sized>(game: &mut H) {
    static SAMUS_HANDLE_SCROLL_X_FACE_LEFT: [u16; 4] = [0xa0, 0x50, 0x20, 0xe0];
    static SAMUS_HANDLE_SCROLL_X_FACE_RIGHT: [u16; 4] = [0x60, 0x40, 0x20, 0xe0];
    if game.previous_player_x() == game.player_x() {
        auto_x(game);
        return;
    }
    let distances = if (word!(game.knockback_dir) != 0
        || game.movement_type() == 16
        || word!(game.samus_x_accel_mode) == 1) as i32
        ^ (game.facing() != 4) as i32
        != 0
    {
        &SAMUS_HANDLE_SCROLL_X_FACE_RIGHT
    } else {
        &SAMUS_HANDLE_SCROLL_X_FACE_LEFT
    };
    game.session_mut().room.camera.ideal_layer1[0] = game
        .player_x()
        .wrapping_sub(distances[(word!(game.camera_distance_index) >> 1) as usize]);
    chase_ideal_layer_position!(
        game,
        0,
        absolute_moved_last_frame_x,
        absolute_moved_last_frame_x_fract,
        trigger_left,
        trigger_right
    );
}
fn follow_y<H: Host + ?Sized>(game: &mut H) {
    if game.previous_player_y() == game.player_y() {
        auto_y(game);
    } else {
        if word!(game.samus_y_dir) == 1 {
            game.session_mut().room.camera.ideal_layer1[1] =
                (game.player_y()).wrapping_sub(word!(game.down_scroller));
        } else {
            game.session_mut().room.camera.ideal_layer1[1] =
                (game.player_y()).wrapping_sub(word!(game.up_scroller));
        }
        chase_ideal_layer_position!(
            game,
            1,
            absolute_moved_last_frame_y,
            absolute_moved_last_frame_y_fract,
            trigger_up,
            trigger_down
        );
    };
}

pub fn fill_viewport<H: Host + ?Sized>(game: &mut H) {
    let v1: i32 = (game.byte(0x58) as i32 - game.byte(0x59) as i32) << 8;
    word!(game.size_of_bg2 = (v1 & 0xf800) as u16);
    calculate_blocks(game);
    let mut v2: i32 = 0;
    do_until! { v3 == 16 ;
        let v3: i32 = v2;
        word!(game.blocks_to_update_x_block = game.session().room.camera.layer1_block[0]);
        word!(game.blocks_to_update_y_block = game.session().room.camera.layer1_block[1]);
        word!(game.vram_blocks_to_update_x_block = game.session().room.camera.bg1_block[0]);
        word!(game.vram_blocks_to_update_y_block = game.session().room.camera.bg1_block[1]);
        game.queue_scrolling_tiles(0x0, ScrollingDmaLayout::Column);
        if game.byte(0x91b) & 1 == 0 {
            word!(game.blocks_to_update_x_block = game.session().room.camera.layer2_block[0]);
            word!(game.blocks_to_update_y_block = game.session().room.camera.layer2_block[1]);
            word!(game.vram_blocks_to_update_x_block = game.session().room.camera.bg2_block[0]);
            word!(game.vram_blocks_to_update_y_block = game.session().room.camera.bg2_block[1]);
            game.queue_scrolling_tiles(0x1c, ScrollingDmaLayout::Column);
        }
        game.flush_tilemaps();
        game.session_mut().room.camera.layer1_block[0] = game.session().room.camera.layer1_block[0].wrapping_add(1);
        game.session_mut().room.camera.bg1_block[0] = game.session().room.camera.bg1_block[0].wrapping_add(1);
        game.session_mut().room.camera.layer2_block[0] = game.session().room.camera.layer2_block[0].wrapping_add(1);
        game.session_mut().room.camera.bg2_block[0] = game.session().room.camera.bg2_block[0].wrapping_add(1);
        v2 += 1;
    }
}

pub fn update_map<H: Host + ?Sized>(game: &mut H) {
    calculate_scrolls(game);
    update_backgrounds(game);
}

pub fn update_scrolls<H: Host + ?Sized>(game: &mut H) {
    if word!(game.time_is_frozen_flag) == 0 {
        word!(
            game.reg_BG1HOFS = (game.session().room.camera.bg1_offset[0])
                .wrapping_add(game.session().room.camera.layer1[0])
        );
        word!(
            game.reg_BG1VOFS = (game.session().room.camera.bg1_offset[1])
                .wrapping_add(game.session().room.camera.layer1[1])
        );
        if layer2_position(CameraAxis::X, game) == 0 {
            word!(
                game.reg_BG2HOFS = (game.session().room.camera.bg2_scroll[0])
                    .wrapping_add(game.session().room.camera.layer2[0])
            );
        }
        if layer2_position(CameraAxis::Y, game) == 0 {
            word!(
                game.reg_BG2VOFS = (game.session().room.camera.bg2_scroll[1])
                    .wrapping_add(game.session().room.camera.layer2[1])
            );
        }
        update_backgrounds(game);
    }
}

pub fn update_backgrounds<H: Host + ?Sized>(game: &mut H) {
    calculate_blocks(game);
    // RAM's coordinates are two bytes apart; BG2's fields follow BG1 by four.
    for (axis, layout) in [
        (0, ScrollingDmaLayout::Column),
        (2, ScrollingDmaLayout::Row),
    ] {
        for (layer, parameter) in [(0, 0), (4, 0x1c)] {
            if layer != 0 && game.byte(0x91b + axis / 2) & 1 != 0 {
                continue;
            }
            // Layer 0 is layer 1, layer 4 is layer 2; axis 0 is x, 2 is y.
            let (index, second) = (axis / 2, layer != 0);
            let camera = game.session().room.camera;
            let (current, previous) = if second {
                (camera.layer2_block, camera.previous_layer2_block)
            } else {
                (camera.layer1_block, camera.previous_layer1_block)
            };
            let negative = (current[index] as i16).wrapping_sub(previous[index] as i16) < 0;
            if current[index] != previous[index] {
                let camera = &mut game.session_mut().room.camera;
                if second {
                    camera.previous_layer2_block[index] = current[index];
                } else {
                    camera.previous_layer1_block[index] = current[index];
                }
                // Columns retain edges 0/16; rows retain edges 1/15.
                let row = axis as i32 / 2;
                let edge = if negative { row } else { 16 - row };
                for (coordinate, edge) in [(axis, edge), (axis ^ 2, 0)] {
                    let camera = game.session().room.camera;
                    let (block, bg) = if second {
                        (camera.layer2_block, camera.bg2_block)
                    } else {
                        (camera.layer1_block, camera.bg1_block)
                    };
                    word!(@write game, fields::blocks_to_update_x_block + coordinate,
                        (block[coordinate / 2] as i32 + edge) as u16);
                    word!(@write game, fields::vram_blocks_to_update_x_block + coordinate,
                        (bg[coordinate / 2] as i32 + edge) as u16);
                }
                game.queue_scrolling_tiles(parameter, layout);
            }
        }
    }
}
