//! Room scroll definitions, camera following and their shared executors.
//! Dispatch identities retain the full address; cell writes retain their order.
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
use crate::player_collision::operations::Axis;
use crate::ram::{bytes as fields_bytes, word, words as fields};
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

/// Samus's camera distance from layer 1 within which slow grapple scrolling
/// leaves the camera alone, per axis: (near, far).
const SLOW_SCROLL_WINDOW: [(u16, u16); 2] = [(0x60, 0xa0), (0x70, 0x90)];

pub fn advance_camera<H: Host + ?Sized>(game: &mut H) {
    if word!(game.SLOW_GRABBLE_SCROLLING_FLAG) != 0 {
        // Slow grapple scrolling steps the camera 3 px toward Samus.
        let player = [game.player_x(), game.player_y()];
        for axis in 0..2 {
            if player[axis] & 0x8000 != 0 {
                break;
            }
            let layer1 = game.session().room.camera.layer1[axis];
            let offset = player[axis].wrapping_sub(layer1);
            let (near, far) = SLOW_SCROLL_WINDOW[axis];
            if player[axis] < layer1 || offset < near {
                set_layer1(game, axis, layer1.wrapping_sub(3));
            } else if offset >= far {
                set_layer1(game, axis, layer1.wrapping_add(3));
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
    match game.session().room.entry.scrolling_hook {
        NO_SCROLLING_HOOK => {}
        STOP_LAYER1_ABOVE_464 => {
            if game.session().room.camera.layer1[1] <= 464 {
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

/// The room's scrolling-finished hooks: none, or keep layer 1 at y >= 464.
const NO_SCROLLING_HOOK: u16 = 0;
const STOP_LAYER1_ABOVE_464: u16 = 0x9589;

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

/// How far ahead of Samus the camera aims, by camera distance index, when
/// she faces left or right (or is pushed back against her facing).
const FACE_LEFT_DISTANCES: [u16; 4] = [0xa0, 0x50, 0x20, 0xe0];
const FACE_RIGHT_DISTANCES: [u16; 4] = [0x60, 0x40, 0x20, 0xe0];

fn follow_x<H: Host + ?Sized>(game: &mut H) {
    if game.previous_player_x() == game.player_x() {
        auto_x(game);
        return;
    }
    let pushed = word!(game.KNOCKBACK_DIR) != 0
        || game.movement_type() == 16
        || word!(game.SAMUS_X_ACCEL_MODE) == 1;
    let distances = if pushed != (game.facing() != 4) {
        &FACE_RIGHT_DISTANCES
    } else {
        &FACE_LEFT_DISTANCES
    };
    let distance = distances[usize::from(word!(game.CAMERA_DISTANCE_INDEX) >> 1)];
    game.session_mut().room.camera.ideal_layer1[0] = game.player_x().wrapping_sub(distance);
    chase_ideal_layer_position!(
        game,
        0,
        ABSOLUTE_MOVED_LAST_FRAME_X,
        ABSOLUTE_MOVED_LAST_FRAME_X_FRACT,
        trigger_left,
        trigger_right
    );
}
fn follow_y<H: Host + ?Sized>(game: &mut H) {
    if game.previous_player_y() == game.player_y() {
        auto_y(game);
        return;
    }
    let scroller = if word!(game.SAMUS_Y_DIR) == 1 {
        game.session().room.entry.scroll_down
    } else {
        game.session().room.entry.scroll_up
    };
    game.session_mut().room.camera.ideal_layer1[1] = game.player_y().wrapping_sub(scroller);
    chase_ideal_layer_position!(
        game,
        1,
        ABSOLUTE_MOVED_LAST_FRAME_Y,
        ABSOLUTE_MOVED_LAST_FRAME_Y_FRACT,
        trigger_up,
        trigger_down
    );
}

/// Layer 2's scroll policy byte on `axis` (0 = x, 1 = y): 0 follows layer 1,
/// 1 is fixed, otherwise bits 1-7 scale layer 1's position.
pub(crate) fn layer2_policy<H: Host + ?Sized>(game: &H, axis: usize) -> u8 {
    let entry = &game.session().room.entry;
    [entry.layer2_x, entry.layer2_y][axis]
}

/// Queue the whole visible room: 17 columns from the camera's left edge,
/// flushing each, layer 2 too unless it is fixed.
pub fn fill_viewport<H: Host + ?Sized>(game: &mut H) {
    let screens = (i32::from(game.byte(fields_bytes::REG_BG1SC))
        - i32::from(game.byte(fields_bytes::REG_BG2SC)))
        << 8;
    word!(game.SIZE_OF_BG2 = (screens & 0xf800) as u16);
    calculate_blocks(game);
    for _ in 0..17 {
        let camera = game.session().room.camera;
        word!(game.BLOCKS_TO_UPDATE_X_BLOCK = camera.layer1_block[0]);
        word!(game.BLOCKS_TO_UPDATE_Y_BLOCK = camera.layer1_block[1]);
        word!(game.VRAM_BLOCKS_TO_UPDATE_X_BLOCK = camera.bg1_block[0]);
        word!(game.VRAM_BLOCKS_TO_UPDATE_Y_BLOCK = camera.bg1_block[1]);
        game.queue_scrolling_tiles(0x0, ScrollingDmaLayout::Column);
        if layer2_policy(game, 0) & 1 == 0 {
            let camera = game.session().room.camera;
            word!(game.BLOCKS_TO_UPDATE_X_BLOCK = camera.layer2_block[0]);
            word!(game.BLOCKS_TO_UPDATE_Y_BLOCK = camera.layer2_block[1]);
            word!(game.VRAM_BLOCKS_TO_UPDATE_X_BLOCK = camera.bg2_block[0]);
            word!(game.VRAM_BLOCKS_TO_UPDATE_Y_BLOCK = camera.bg2_block[1]);
            game.queue_scrolling_tiles(0x1c, ScrollingDmaLayout::Column);
        }
        game.flush_tilemaps();
        let camera = &mut game.session_mut().room.camera;
        for block in [
            &mut camera.layer1_block[0],
            &mut camera.bg1_block[0],
            &mut camera.layer2_block[0],
            &mut camera.bg2_block[0],
        ] {
            *block = block.wrapping_add(1);
        }
    }
}

pub fn update_map<H: Host + ?Sized>(game: &mut H) {
    calculate_scrolls(game);
    update_backgrounds(game);
}

pub fn update_scrolls<H: Host + ?Sized>(game: &mut H) {
    if word!(game.TIME_IS_FROZEN_FLAG) != 0 {
        return;
    }
    let camera = game.session().room.camera;
    word!(game.REG_BG1HOFS = camera.bg1_offset[0].wrapping_add(camera.layer1[0]));
    word!(game.REG_BG1VOFS = camera.bg1_offset[1].wrapping_add(camera.layer1[1]));
    if layer2_position(CameraAxis::X, game) == 0 {
        let camera = game.session().room.camera;
        word!(game.REG_BG2HOFS = camera.bg2_scroll[0].wrapping_add(camera.layer2[0]));
    }
    if layer2_position(CameraAxis::Y, game) == 0 {
        let camera = game.session().room.camera;
        word!(game.REG_BG2VOFS = camera.bg2_scroll[1].wrapping_add(camera.layer2[1]));
    }
    update_backgrounds(game);
}

/// Queue the scrolling tiles each layer's block coordinates crossed since the
/// last frame: columns for x, rows for y, layer 2 too unless it is fixed.
pub fn update_backgrounds<H: Host + ?Sized>(game: &mut H) {
    calculate_blocks(game);
    for (axis, layout) in [
        (0, ScrollingDmaLayout::Column),
        (1, ScrollingDmaLayout::Row),
    ] {
        for (second, parameter) in [(false, 0), (true, 0x1c)] {
            if second && layer2_policy(game, axis) & 1 != 0 {
                continue;
            }
            let camera = game.session().room.camera;
            let (current, previous) = if second {
                (
                    camera.layer2_block[axis],
                    camera.previous_layer2_block[axis],
                )
            } else {
                (
                    camera.layer1_block[axis],
                    camera.previous_layer1_block[axis],
                )
            };
            if current == previous {
                continue;
            }
            let backwards = (current as i16).wrapping_sub(previous as i16) < 0;
            let camera = &mut game.session_mut().room.camera;
            if second {
                camera.previous_layer2_block[axis] = current;
            } else {
                camera.previous_layer1_block[axis] = current;
            }
            // Columns take edges 0/16; rows take edges 1/15.
            let row = axis as i32;
            let edge = if backwards { row } else { 16 - row };
            for (coordinate, edge) in [(axis, edge), (1 - axis, 0)] {
                let camera = game.session().room.camera;
                let (block, bg) = if second {
                    (camera.layer2_block, camera.bg2_block)
                } else {
                    (camera.layer1_block, camera.bg1_block)
                };
                word!(@write game, fields::BLOCKS_TO_UPDATE_X_BLOCK + 2 * coordinate,
                    (i32::from(block[coordinate]) + edge) as u16);
                word!(@write game, fields::VRAM_BLOCKS_TO_UPDATE_X_BLOCK + 2 * coordinate,
                    (i32::from(bg[coordinate]) + edge) as u16);
            }
            game.queue_scrolling_tiles(parameter, layout);
        }
    }
}
