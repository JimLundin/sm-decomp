//! Mother Brain's rainbow beam ($AD:DE00-$AD:E433, $88:E767-$88:E832): the
//! beam's two edges from its angle and width, rasterized row by row into the
//! window HDMA table at $7E:9D00, the indirect HDMA header at $7E:9C00 that
//! points into it, and the HDMA object's initialisation and pre-instruction,
//! which cycle the colour math backdrop through the rainbow.
//!
//! The beam window runs as one of Mother Brain's state steps: its slot words
//! are the Session's inside a lease, and the tables are named RAM.
#![forbid(unsafe_code)]
use crate::actors::mother_brain::{BODY, BRAIN, MotherBrainBody, MotherBrainBrain};
use crate::actors::mother_brain_fight::State;
use crate::actors::session_slots::session_slot;
use crate::beam_windows::advance_window_edges;
use crate::cartridge::LoadedImage;
use sm_data::beam_gradients::{ADDRESS as GRADIENTS, BeamGradients};
use sm_data::mother_brain::{RAINBOW_ADDRESS, RainbowColors};

/// $7E:9C00: the indirect HDMA header.
const HEADER: isize = crate::ram::bytes::mother_brain_indirect_hdma as isize;
/// $7E:9D00: the window table the header points into.
const WINDOWS: isize = crate::ram::word_arrays::hdma_table_2 as isize;
/// The beam is drawn between rows 20h and E8h.
const TOP_ROW: u16 = 32;
const BOTTOM_ROW: u16 = 232;
/// A row the beam does not cover: left edge FFh, right edge 0.
const EMPTY_ROW: u16 = 0xff;
// The downward rows count from $7E:803E, the body's right origin Y.
const _: () =
    assert!(0xf78 + core::mem::offset_of!(MotherBrainBody, beam_right_origin_y) == 0x803e);

/// Which way the beam points, by the octants of its two edges.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Direction {
    UpRight,
    Up,
    UpLeft,
    DownRight,
    Down,
    DownLeft,
}

/// What the beam window reads beyond Mother Brain's state: the loaded
/// cartridge's edge gradients.
pub trait Host: State + LoadedImage {}
impl<T: State + LoadedImage + ?Sized> Host for T {}

/// $7E:0998, $7E:186E, $7E:18B2 and $7E:1914: the game state, the beam's
/// layer blending, the current HDMA object and its colour index.
const GAME_STATE: usize = crate::ram::words::game_state;
const LAYER_BLENDING: usize = crate::ram::words::fx_layer_blending_config_c;
const HDMA_OBJECT_INDEX: usize = crate::ram::words::hdma_object_index;
const COLOUR_INDEX: usize = crate::ram::word_arrays::hdma_object_A;

/// The edge gradient for `angle`: |tan(angle * pi / 80h)| * 100h ($91:C9D4).
fn gradient(game: &mut impl Host, angle: isize) -> u16 {
    crate::imported::word(
        &None::<crate::imported::Imported<BeamGradients>>,
        game,
        GRADIENTS,
        angle,
    )
}

/// Store `value` natively at `at`.
fn write(game: &mut impl Host, at: isize, value: u16) {
    game.write_slice_native(usize::try_from(at).expect("beam window outside RAM"), value);
}

/// Calculate Mother Brain rainbow beam HDMA tables ($AD:DE00): set the beam's
/// edges and origin from its angle, width and the brain, then rasterize the
/// window for the beam's direction.
pub fn calculate_window(game: &mut impl Host) {
    let half_width = u16::from(session_slot!(byte game, MotherBrainBody[BODY].beam_width[1]) >> 1);
    session_slot!(
        game,
        MotherBrainBody[BODY].beam_right_edge =
            session_slot!(game, MotherBrainBody[BODY].beam_angle).wrapping_sub(half_width)
    );
    session_slot!(
        game,
        MotherBrainBody[BODY].beam_left_edge =
            half_width.wrapping_add(session_slot!(game, MotherBrainBody[BODY].beam_angle))
    );
    // The brain's X position, read a byte early so that its low byte lands in
    // the high byte, plus Eh pixels.
    let origin_x = ((i32::from(session_slot!(
        game,
        MotherBrainBody[BRAIN + 1].base.enemy_ptr
    )) + 0xe00)
        & 0xff00) as u16;
    session_slot!(game, MotherBrainBody[BODY]; beam_left_origin_x = origin_x, beam_right_origin_x = origin_x);
    let origin_y = session_slot!(game, MotherBrainBrain[BRAIN].base.y_pos).wrapping_add(5);
    session_slot!(game, MotherBrainBody[BODY]; beam_left_origin_y = origin_y, beam_right_origin_y = origin_y);
    let right_octant = 4 * (session_slot!(game, MotherBrainBody[BODY].beam_right_edge) & 0xc0);
    let octants =
        (right_octant | (session_slot!(game, MotherBrainBody[BODY].beam_left_edge) & 0xc0)) >> 6;
    match octants {
        0 | 12 | 15 => {
            let direction = match direction_bits(game) {
                0 => Direction::DownRight,
                2 => Direction::Down,
                3 => Direction::DownLeft,
                _ => panic!("non-null function pointer"),
            };
            let left = session_slot!(game, MotherBrainBody[BODY].beam_left_origin_x);
            let right = session_slot!(game, MotherBrainBody[BODY].beam_right_origin_x);
            rasterize_vertical(game, direction, left, right);
            write_downward_header(game, 0x9de6);
        }
        1 => {
            let left = session_slot!(game, MotherBrainBody[BODY].beam_left_origin_x);
            let right = session_slot!(game, MotherBrainBody[BODY].beam_right_origin_x);
            rasterize_rightward(game, left, right);
            write_downward_header(game, 0x9dec);
        }
        5 | 6 | 10 => rasterize_upward(game),
        11 => {}
        _ => panic!("non-null function pointer"),
    }
}

/// The right edge's second-octant bit and the left edge's top-half bit.
fn direction_bits(game: &mut impl Host) -> u16 {
    ((session_slot!(game, MotherBrainBody[BODY].beam_right_edge) >> 6) & 2)
        | ((session_slot!(game, MotherBrainBody[BODY].beam_left_edge) >> 7) & 1)
}

/// The indirect HDMA header for a beam drawn from the top: 10h blank rows,
/// 10h more, F0h and F4h rows of the window table, ending at `tail`.
fn write_downward_header(game: &mut impl Host, tail: u16) {
    write(game, HEADER, 16);
    write(game, HEADER + 1, 0x9c00);
    write(game, HEADER + 3, 16);
    write(game, HEADER + 4, 0x9c03);
    write(game, HEADER + 6, 240);
    write(game, HEADER + 7, 0x9d04);
    write(game, HEADER + 9, 244);
    write(game, HEADER + 10, tail);
    write(game, HEADER + 12, 0);
}

/// Rasterize a beam pointing right ($AD:DE7F-$AD:DF6D): the left edge up from the
/// origin row, starting at `up_x`, and the right edge down from it, starting
/// at `down_x`.
fn rasterize_rightward(game: &mut impl Host, mut down_x: u16, mut up_x: u16) {
    write(game, WINDOWS, EMPTY_ROW);
    write(game, WINDOWS + 2, EMPTY_ROW);
    let down_step = gradient(
        game,
        session_slot!(byte game, MotherBrainBody[BODY].beam_right_edge[0]) as isize,
    );
    let up_step = gradient(
        game,
        session_slot!(byte game, MotherBrainBody[BODY].beam_left_edge[0]) as isize,
    );
    let mut row = session_slot!(game, MotherBrainBody[BODY].beam_right_origin_y);
    let mut at = WINDOWS + 2 * (row as isize - 32 + 1);
    write(game, at + 2, EMPTY_ROW);
    write(game, at + 4, EMPTY_ROW);
    let origin_at = at;
    // Upwards: the left edge until it runs off the screen, then empty rows.
    loop {
        let (edge, overflow) = up_step.overflowing_add(up_x);
        if overflow {
            break;
        }
        up_x = edge;
        write(game, at, u16::from(edge.to_ne_bytes()[1]) | 0xff00);
        at -= 2;
        row = row.wrapping_sub(1);
        if row == TOP_ROW {
            break;
        }
    }
    loop {
        write(game, at, EMPTY_ROW);
        at -= 2;
        row = row.wrapping_sub(1);
        if row == TOP_ROW {
            break;
        }
    }
    // Downwards: the right edge until it runs off the screen, then empty rows.
    let mut at = origin_at + 2;
    let mut row = session_slot!(game, MotherBrainBody[BODY].beam_right_origin_y);
    loop {
        let (edge, overflow) = down_step.overflowing_add(down_x);
        if overflow {
            break;
        }
        down_x = edge;
        write(game, at, u16::from(edge.to_ne_bytes()[1]) | 0xff00);
        at += 2;
        row = row.wrapping_add(1);
        if row == BOTTOM_ROW {
            return;
        }
    }
    loop {
        write(game, at, EMPTY_ROW);
        at += 2;
        row = row.wrapping_add(1);
        if row == BOTTOM_ROW {
            break;
        }
    }
}

/// Rasterize a beam pointing up ($AD:DF6E), then its indirect HDMA header:
/// the table runs from the top of the screen to the origin row, in one
/// stretch of at most 7Fh rows or two.
fn rasterize_upward(game: &mut impl Host) {
    let direction = match direction_bits(game) {
        0 => Direction::UpRight,
        1 => Direction::Up,
        3 => Direction::UpLeft,
        _ => panic!("non-null function pointer"),
    };
    let left = session_slot!(game, MotherBrainBody[BODY].beam_left_origin_x);
    let right = session_slot!(game, MotherBrainBody[BODY].beam_right_origin_x);
    rasterize_vertical(game, direction, left, right);
    write(game, HEADER, 16);
    write(game, HEADER + 1, 0x9d00);
    write(game, HEADER + 3, 16);
    write(game, HEADER + 4, 0x9d02);
    let rows = session_slot!(game, MotherBrainBody[BODY].beam_left_origin_y).wrapping_sub(32);
    if rows >= 0x80 {
        let first = rows.wrapping_sub(127);
        write(game, HEADER + 6, first | 0x80);
        write(game, HEADER + 7, 0x9d04);
        write(game, HEADER + 9, 255);
        write(
            game,
            HEADER + 10,
            first.wrapping_mul(2).wrapping_add(0x9d04),
        );
        write(game, HEADER + 12, 127);
        write(
            game,
            HEADER + 13,
            first.wrapping_mul(2).wrapping_add(0x9d04 + 254),
        );
        write(game, HEADER + 15, 0);
    } else {
        write(game, HEADER + 6, rows | 0x80);
        write(game, HEADER + 7, 0x9d04);
        write(game, HEADER + 9, 127);
        write(game, HEADER + 10, rows.wrapping_mul(2).wrapping_add(0x9d04));
        write(game, HEADER + 12, 0);
    }
}

/// Rasterize a beam pointing up or down ($AD:E02C-$AD:E433): both edges from
/// the origin row towards the top or bottom of the screen.
fn rasterize_vertical(game: &mut impl Host, direction: Direction, left: u16, right: u16) {
    use Direction::*;
    let down = matches!(direction, DownRight | Down | DownLeft);
    let add_left = matches!(direction, UpRight | DownRight);
    let add_right = matches!(direction, UpRight | Up | DownRight | Down);
    write(game, WINDOWS, EMPTY_ROW);
    write(game, WINDOWS + 2, EMPTY_ROW);
    // Going down the left edge is the beam's right edge.
    let left_edge = if down {
        session_slot!(byte game, MotherBrainBody[BODY].beam_right_edge[0])
    } else {
        session_slot!(byte game, MotherBrainBody[BODY].beam_left_edge[0])
    };
    let left_step = gradient(game, edge_angle(left_edge, add_left));
    let right_edge = if down {
        session_slot!(byte game, MotherBrainBody[BODY].beam_left_edge[0])
    } else {
        session_slot!(byte game, MotherBrainBody[BODY].beam_right_edge[0])
    };
    let right_step = gradient(game, edge_angle(right_edge, add_right));
    let mut row = session_slot!(game, MotherBrainBody[BODY].beam_left_origin_y);
    let mut at = if down {
        // The rows above the origin are empty.
        let mut remaining = row.wrapping_sub(32);
        let mut at = WINDOWS + 4;
        loop {
            write(game, at, EMPTY_ROW);
            at += 2;
            remaining = remaining.wrapping_sub(1);
            if remaining == 0 {
                break;
            }
        }
        row = session_slot!(game, MotherBrainBody[BODY].beam_left_origin_y);
        at
    } else {
        let at = WINDOWS + 2 * (row as isize - 32 + 1);
        write(game, at + 2, EMPTY_ROW);
        write(game, at + 4, EMPTY_ROW);
        at
    };
    let mut edges = [u32::from(left), u32::from(right)];
    let steps = [
        signed_step(left_step, add_left),
        signed_step(right_step, add_right),
    ];
    let (end, step) = if down { (BOTTOM_ROW, 2) } else { (TOP_ROW, -2) };
    let blank = if direction == DownLeft { 0 } else { 0xffff };
    loop {
        let mut packed = advance_window_edges(&mut edges, steps, true);
        if packed == blank {
            packed = EMPTY_ROW;
        }
        write(game, at, packed);
        at += step;
        row = if down {
            row.wrapping_add(1)
        } else {
            row.wrapping_sub(1)
        };
        if row == end {
            break;
        }
    }
}

/// The gradient table index for an edge at `angle` (its low byte), negated
/// for an edge that moves left.
fn edge_angle(angle: u8, positive: bool) -> isize {
    (if positive {
        angle
    } else {
        (-i32::from(angle)) as u8
    }) as isize
}

/// The 8.8 step for `gradient`, negated for an edge that moves left.
fn signed_step(gradient: u16, positive: bool) -> u32 {
    if positive {
        u32::from(gradient)
    } else {
        (-i32::from(gradient)) as u32
    }
}

/// The colour math subscreen backdrop colour mirror ($7E:0074..76).
const BACKDROP: usize = crate::ram::mirror::COLDATA;
/// Layer blending configuration 24h: the beam's colour math window.
const BEAM_BLENDING: u16 = 0x24;
/// Game states: death sequence start, and reserve tanks auto.
const DEATH_SEQUENCE_START: u16 = 0x13;
const RESERVE_TANKS_AUTO: u16 = 0x1b;
/// $7E:18B4: the HDMA objects' channel table.
const HDMA_OBJECT_CHANNELS: usize = 0x18b4;

/// Initialise rainbow beam ($88:E767): a greenish blue backdrop, the beam's
/// layer blending, the indirect header's first row and the first three
/// window rows, then the window.
pub fn initialize_hdma(game: &mut impl Host) {
    game.write_slice(BACKDROP, &[0x20, 0x47, 0x8f]);
    game.write_word_le(BEAM_BLENDING, LAYER_BLENDING);
    game.set_byte(HEADER as usize, 0x64);
    write(game, HEADER + 1, 0x9d00);
    write(game, HEADER + 3, 0);
    write(game, WINDOWS, 0);
    write(game, WINDOWS + 2, 0x8020);
    write(game, WINDOWS + 4, 0);
    calculate_window(game);
}

/// Pre-instruction - Mother Brain rainbow beam ($88:E7BC): delete the HDMA
/// object once Samus's death sequence starts; otherwise keep the beam's
/// blending and, outside the reserve tanks' refill, redraw the window and
/// step the rainbow.
pub fn advance_hdma(game: &mut impl Host) {
    if game.word(GAME_STATE) == DEATH_SEQUENCE_START {
        let at = HDMA_OBJECT_CHANNELS + 2 * usize::from(game.word(HDMA_OBJECT_INDEX) >> 1);
        write(game, at as isize, 0);
    } else {
        game.write_word_le(BEAM_BLENDING, LAYER_BLENDING);
        if game.word(GAME_STATE) != RESERVE_TANKS_AUTO {
            calculate_window(game);
            set_backdrop_colour(game);
        }
    }
}

/// The rainbow's colour at word `index` ($88:E833), ending in a word with bit
/// 15 set.
fn rainbow_colour(game: &mut impl Host, index: isize) -> u16 {
    crate::imported::word(
        &None::<crate::imported::Imported<RainbowColors>>,
        game,
        RAINBOW_ADDRESS,
        index,
    )
}

/// Set rainbow beam colour math subscreen backdrop colour ($88:E7ED): the
/// next colour of the rainbow, wrapping to its first at the end marker.
fn set_backdrop_colour(game: &mut impl Host) {
    let index = (game.word(COLOUR_INDEX) >> 1) as isize;
    let mut colour = rainbow_colour(game, index);
    if colour & 0x8000 == 0 {
        for _ in 0..4 {
            let index = game.word(COLOUR_INDEX).wrapping_add(1);
            game.write_word_le(index, COLOUR_INDEX);
        }
    } else {
        game.write_word_le(0, COLOUR_INDEX);
        colour = rainbow_colour(game, 0);
    }
    game.set_byte(BACKDROP, ((colour as u8) & 0x1f) | 0x20);
    game.set_byte(BACKDROP + 1, (((colour >> 5) as u8) & 0x1f) | 0x40);
    game.set_byte(BACKDROP + 2, (((colour >> 10) as u8) & 0x1f) | 0x80);
}
