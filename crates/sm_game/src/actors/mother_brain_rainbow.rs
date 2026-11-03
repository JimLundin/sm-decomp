//! Mother Brain's rainbow beam ($AD:DE00-$AD:E433, $88:E767-$88:E832): the
//! beam's two edges from its angle and width, rasterized row by row into the
//! window HDMA table at $7E:9D00, the indirect HDMA header at $7E:9C00 that
//! points into it, and the HDMA object's initialisation and pre-instruction,
//! which cycle the colour math backdrop through the rainbow.
//!
//! The beam window runs as one of Mother Brain's state steps: its slot words
//! are the Session's typed slot state, and the tables are stored through the
//! FX buffers that hold them (`Sources::store_buffer_word`).
use crate::actors::mother_brain::{
    BODY_INDEX, BRAIN_INDEX, Body, BodyEdit, Brain, beam_right_origin_x, set_beam_right_origin_x,
};
use crate::actors::mother_brain_fight::State;
use crate::beam_windows::advance_window_edges;
use crate::cartridge::LoadedImage;
use crate::scene::hdma_sources::LiveOwner;
use sm_data::beam_gradients::{ADDRESS as GRADIENTS, BeamGradients};
use sm_data::mother_brain::{RAINBOW_ADDRESS, RainbowColors};

/// The indirect HDMA header's buffer: $7E:9C00, the liquid surface table's.
const HEADER: LiveOwner = LiveOwner::LiquidSurface;
/// The window table the header points into: HDMA table 2, $7E:9D00..$7E:9DFF.
const WINDOWS: LiveOwner = LiveOwner::HdmaTable2;
/// The window table's bytes.
const WINDOW_BYTES: usize = 0x100;
/// The beam is drawn between rows 20h and E8h.
const TOP_ROW: u16 = 32;
const BOTTOM_ROW: u16 = 232;
/// A row the beam does not cover: left edge FFh, right edge 0.
const EMPTY_ROW: u16 = 0xff;

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
/// cartridge's edge gradients, and the Session's HDMA objects.
pub trait Host:
    State
    + LoadedImage
    + crate::actors::slot_view::SlotState
    + crate::hdma_objects::HdmaTable
    + crate::fx_domain::Controls
{
}
impl<
    T: State
        + LoadedImage
        + crate::actors::slot_view::SlotState
        + crate::hdma_objects::HdmaTable
        + crate::fx_domain::Controls
        + ?Sized,
> Host for T
{
}

/// The edge gradient for `angle`: |tan(angle * pi / 80h)| * 100h ($91:C9D4).
fn gradient(game: &mut impl Host, angle: isize) -> u16 {
    crate::imported::word(
        &None::<crate::imported::Imported<BeamGradients>>,
        game,
        GRADIENTS,
        angle,
    )
}

/// Store `value` natively at byte `offset` of the indirect HDMA header.
fn write_header(game: &mut impl Host, offset: usize, value: u16) {
    game.store_buffer_word(HEADER, offset, crate::fx_domain::native_order(), value);
}

/// Store `value` natively at byte `offset` of the window table.
fn write_row(game: &mut impl Host, offset: isize, value: u16) {
    match usize::try_from(offset) {
        Ok(offset) if offset < WINDOW_BYTES - 1 => {
            game.store_buffer_word(WINDOWS, offset, crate::fx_domain::native_order(), value);
        }
        _ => {
            // LEGACY(original-overflow): the rasterizers' rows run past the
            // table's end ($7E:9DFF) into the window tables after it, and an
            // origin above the top row runs them before its start.
            let at = crate::fx_encoding::source_address(WINDOWS, 0)
                .and_then(|base| base.checked_add_signed(offset))
                .expect("beam window outside RAM");
            crate::legacy_overflow::ram_write_slice(game, at, &value.to_ne_bytes());
        }
    }
}

/// Calculate Mother Brain rainbow beam HDMA tables ($AD:DE00): set the beam's
/// edges and origin from its angle, width and the brain, then rasterize the
/// window for the beam's direction.
pub fn calculate_window(game: &mut impl Host) {
    let half_width = u16::from(Body::of(game, BODY_INDEX).beam_width().to_le_bytes()[1] >> 1);
    *BodyEdit::of(game, BODY_INDEX).beam_right_edge() = Body::of(game, BODY_INDEX)
        .beam_angle()
        .wrapping_sub(half_width);
    *BodyEdit::of(game, BODY_INDEX).beam_left_edge() =
        half_width.wrapping_add(Body::of(game, BODY_INDEX).beam_angle());
    // The brain's X position, read a byte early so that its low byte lands in
    // the high byte, plus Eh pixels.
    let brain = Brain::of(game, BRAIN_INDEX).actor();
    let early = u16::from_le_bytes([
        brain.definition.to_le_bytes()[1],
        brain.x.whole.to_le_bytes()[0],
    ]);
    let origin_x = ((i32::from(early) + 0xe00) & 0xff00) as u16;
    *BodyEdit::of(game, BODY_INDEX).beam_left_origin_x() = origin_x;
    set_beam_right_origin_x(game, origin_x);
    let origin_y = Brain::of(game, BRAIN_INDEX).actor().y.whole.wrapping_add(5);
    *BodyEdit::of(game, BODY_INDEX).beam_left_origin_y() = origin_y;
    *BodyEdit::of(game, BODY_INDEX).beam_right_origin_y() = origin_y;
    let right_octant = 4 * (Body::of(game, BODY_INDEX).beam_right_edge() & 0xc0);
    let octants = (right_octant | (Body::of(game, BODY_INDEX).beam_left_edge() & 0xc0)) >> 6;
    match octants {
        0 | 12 | 15 => {
            let direction = match direction_bits(game) {
                0 => Direction::DownRight,
                2 => Direction::Down,
                3 => Direction::DownLeft,
                _ => panic!("non-null function pointer"),
            };
            let left = Body::of(game, BODY_INDEX).beam_left_origin_x();
            let right = beam_right_origin_x(game);
            rasterize_vertical(game, direction, left, right);
            write_downward_header(game, 0x9de6);
        }
        1 => {
            let left = Body::of(game, BODY_INDEX).beam_left_origin_x();
            let right = beam_right_origin_x(game);
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
    ((Body::of(game, BODY_INDEX).beam_right_edge() >> 6) & 2)
        | ((Body::of(game, BODY_INDEX).beam_left_edge() >> 7) & 1)
}

/// The indirect HDMA header for a beam drawn from the top: 10h blank rows,
/// 10h more, F0h and F4h rows of the window table, ending at `tail`.
fn write_downward_header(game: &mut impl Host, tail: u16) {
    write_header(game, 0, 16);
    write_header(game, 1, 0x9c00);
    write_header(game, 3, 16);
    write_header(game, 4, 0x9c03);
    write_header(game, 6, 240);
    write_header(game, 7, 0x9d04);
    write_header(game, 9, 244);
    write_header(game, 10, tail);
    write_header(game, 12, 0);
}

/// Rasterize a beam pointing right ($AD:DE7F-$AD:DF6D): the left edge up from the
/// origin row, starting at `up_x`, and the right edge down from it, starting
/// at `down_x`.
fn rasterize_rightward(game: &mut impl Host, mut down_x: u16, mut up_x: u16) {
    write_row(game, 0, EMPTY_ROW);
    write_row(game, 2, EMPTY_ROW);
    let down_step = gradient(
        game,
        Body::of(game, BODY_INDEX).beam_right_edge().to_le_bytes()[0] as isize,
    );
    let up_step = gradient(
        game,
        Body::of(game, BODY_INDEX).beam_left_edge().to_le_bytes()[0] as isize,
    );
    let mut row = Body::of(game, BODY_INDEX).beam_right_origin_y();
    let mut at = 2 * (row as isize - 32 + 1);
    write_row(game, at + 2, EMPTY_ROW);
    write_row(game, at + 4, EMPTY_ROW);
    let origin_at = at;
    // Upwards: the left edge until it runs off the screen, then empty rows.
    loop {
        let (edge, overflow) = up_step.overflowing_add(up_x);
        if overflow {
            break;
        }
        up_x = edge;
        write_row(game, at, u16::from(edge.to_ne_bytes()[1]) | 0xff00);
        at -= 2;
        row = row.wrapping_sub(1);
        if row == TOP_ROW {
            break;
        }
    }
    loop {
        write_row(game, at, EMPTY_ROW);
        at -= 2;
        row = row.wrapping_sub(1);
        if row == TOP_ROW {
            break;
        }
    }
    // Downwards: the right edge until it runs off the screen, then empty rows.
    let mut at = origin_at + 2;
    let mut row = Body::of(game, BODY_INDEX).beam_right_origin_y();
    loop {
        let (edge, overflow) = down_step.overflowing_add(down_x);
        if overflow {
            break;
        }
        down_x = edge;
        write_row(game, at, u16::from(edge.to_ne_bytes()[1]) | 0xff00);
        at += 2;
        row = row.wrapping_add(1);
        if row == BOTTOM_ROW {
            return;
        }
    }
    loop {
        write_row(game, at, EMPTY_ROW);
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
    let left = Body::of(game, BODY_INDEX).beam_left_origin_x();
    let right = beam_right_origin_x(game);
    rasterize_vertical(game, direction, left, right);
    write_header(game, 0, 16);
    write_header(game, 1, 0x9d00);
    write_header(game, 3, 16);
    write_header(game, 4, 0x9d02);
    let rows = Body::of(game, BODY_INDEX)
        .beam_left_origin_y()
        .wrapping_sub(32);
    if rows >= 0x80 {
        let first = rows.wrapping_sub(127);
        write_header(game, 6, first | 0x80);
        write_header(game, 7, 0x9d04);
        write_header(game, 9, 255);
        write_header(game, 10, first.wrapping_mul(2).wrapping_add(0x9d04));
        write_header(game, 12, 127);
        write_header(game, 13, first.wrapping_mul(2).wrapping_add(0x9d04 + 254));
        write_header(game, 15, 0);
    } else {
        write_header(game, 6, rows | 0x80);
        write_header(game, 7, 0x9d04);
        write_header(game, 9, 127);
        write_header(game, 10, rows.wrapping_mul(2).wrapping_add(0x9d04));
        write_header(game, 12, 0);
    }
}

/// Rasterize a beam pointing up or down ($AD:E02C-$AD:E433): both edges from
/// the origin row towards the top or bottom of the screen.
fn rasterize_vertical(game: &mut impl Host, direction: Direction, left: u16, right: u16) {
    use Direction::*;
    let down = matches!(direction, DownRight | Down | DownLeft);
    let add_left = matches!(direction, UpRight | DownRight);
    let add_right = matches!(direction, UpRight | Up | DownRight | Down);
    write_row(game, 0, EMPTY_ROW);
    write_row(game, 2, EMPTY_ROW);
    // Going down the left edge is the beam's right edge.
    let left_edge = if down {
        Body::of(game, BODY_INDEX).beam_right_edge().to_le_bytes()[0]
    } else {
        Body::of(game, BODY_INDEX).beam_left_edge().to_le_bytes()[0]
    };
    let left_step = gradient(game, edge_angle(left_edge, add_left));
    let right_edge = if down {
        Body::of(game, BODY_INDEX).beam_left_edge().to_le_bytes()[0]
    } else {
        Body::of(game, BODY_INDEX).beam_right_edge().to_le_bytes()[0]
    };
    let right_step = gradient(game, edge_angle(right_edge, add_right));
    let mut row = Body::of(game, BODY_INDEX).beam_left_origin_y();
    let mut at = if down {
        // The rows above the origin are empty.
        let mut remaining = row.wrapping_sub(32);
        let mut at = 4;
        loop {
            write_row(game, at, EMPTY_ROW);
            at += 2;
            remaining = remaining.wrapping_sub(1);
            if remaining == 0 {
                break;
            }
        }
        row = Body::of(game, BODY_INDEX).beam_left_origin_y();
        at
    } else {
        let at = 2 * (row as isize - 32 + 1);
        write_row(game, at + 2, EMPTY_ROW);
        write_row(game, at + 4, EMPTY_ROW);
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
        write_row(game, at, packed);
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

/// Select the beam's layer blending ($7E:1986, the FX blending).
fn set_beam_blending(game: &mut impl Host) {
    let mut fx = game.fx_controls();
    fx.blending = BEAM_BLENDING;
    game.publish_fx_controls(fx, &[crate::fx_state::Field::Blending]);
}

/// Initialise rainbow beam ($88:E767): a greenish blue backdrop, the beam's
/// layer blending, the indirect header's first row and the first three
/// window rows, then the window.
pub fn initialize_hdma(game: &mut impl Host) {
    game.write_slice(BACKDROP, &[0x20, 0x47, 0x8f]);
    set_beam_blending(game);
    game.store_buffer_byte(HEADER, 0, 0x64);
    write_header(game, 1, 0x9d00);
    write_header(game, 3, 0);
    write_row(game, 0, 0);
    write_row(game, 2, 0x8020);
    write_row(game, 4, 0);
    calculate_window(game);
}

/// Pre-instruction - Mother Brain rainbow beam ($88:E7BC): delete the HDMA
/// object once Samus's death sequence starts; otherwise keep the beam's
/// blending and, outside the reserve tanks' refill, redraw the window and
/// step the rainbow.
pub fn advance_hdma(game: &mut impl Host) {
    if game.session().game_loop.mode == DEATH_SEQUENCE_START {
        // Delete the HDMA object being run.
        let index = usize::from(
            crate::hdma_objects::HdmaTable::hdma_word(game, crate::hdma::Word::Selected) >> 1,
        );
        crate::hdma_word!(game, index, Channels => 0);
    } else {
        set_beam_blending(game);
        if game.session().game_loop.mode != RESERVE_TANKS_AUTO {
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
    // The colour index is HDMA object 0's A, a byte offset into the rainbow.
    let index = crate::hdma_word!(game, 0, A);
    let mut colour = rainbow_colour(game, (index >> 1) as isize);
    if colour & 0x8000 == 0 {
        crate::hdma_word!(game, 0, A => index.wrapping_add(4));
    } else {
        crate::hdma_word!(game, 0, A => 0);
        colour = rainbow_colour(game, 0);
    }
    game.set_byte(BACKDROP, ((colour as u8) & 0x1f) | 0x20);
    game.set_byte(BACKDROP + 1, (((colour >> 5) as u8) & 0x1f) | 0x40);
    game.set_byte(BACKDROP + 2, (((colour >> 10) as u8) & 0x1f) | 0x80);
}
