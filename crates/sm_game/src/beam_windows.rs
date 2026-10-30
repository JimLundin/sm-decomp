//! HDMA window tables for the X-ray beam and Mother Brain's rainbow beam.
#![forbid(unsafe_code)]
use crate::actors::mother_brain::{MotherBrainBody, MotherBrainBrain};
use crate::imported::{Imported, word};
use crate::menus::host::Host;
use crate::menus::prelude::do_until;
use crate::ram::RawRam as _;
use crate::ram::{slot, word};
use sm_data::beam_gradients::{ADDRESS, BYTES, BeamGradients};

/// The X-ray beam's edge gradients, one word per angle, in bank $91.
const XRAY_GRADIENTS: u32 = 0x91_c9d4;
/// Bytes of window words a beam table covers (230 rows).
const TABLE_BYTES: u16 = 460;
/// The rainbow beam's indirect HDMA header and its window rows.
const RAINBOW_HEADER: isize = 0x9c00;
const RAINBOW_ROWS: isize = 0x9d00;
/// The window word that hides a row.
const CLOSED: u16 = 255;

/// Enemy extra RAM in the decompilation's default layout; the rainbow beam
/// keeps its downward row counter in the last word.
#[repr(C)]
pub struct BeamScratch {
    _words: [u16; 31],
    pub row: u16,
}

pub type ImportedTables = Option<Imported<BeamGradients>>;

pub fn import(image: &sm_data::cartridge::RomImage) -> ImportedTables {
    Imported::import(image, ADDRESS, BYTES, sm_data::beam_gradients::import)
}

/// The rainbow edge gradient for `angle`; other images and angles past 80h
/// run the original indexed load.
fn rainbow_gradient<H: Host + ?Sized>(game: &H, angle: isize) -> u16 {
    word(game.beam_gradients(), game, ADDRESS, angle)
}

/// Advance the two 8.8 fixed-point edges and pack one SNES window word.
fn advance_window_edges(edges: &mut [u32; 2], steps: [u32; 2], saturated: bool) -> u16 {
    for (edge, step) in edges.iter_mut().zip(steps) {
        *edge = edge.wrapping_add(step);
        if saturated {
            *edge = (*edge as i32).clamp(0, 65535) as u32;
        }
    }
    let left = (edges[0] as i32).clamp(0, 65535) as u16;
    let right = (edges[1] as i32).clamp(0, 65535) as u16;
    (left >> 8) | (right & 0xff00)
}

/// Close every row from `position` to the table's end (down) or start (up).
fn close_rows(write: &mut impl FnMut(i32, u16), mut position: u16, down: bool) {
    loop {
        write(i32::from(position & !1), CLOSED);
        position = position.wrapping_add(if down { 2 } else { 65534 });
        let past_end = if down {
            position >= TABLE_BYTES
        } else {
            position & 0x8000 != 0
        };
        if past_end {
            return;
        }
    }
}

/// Rasterize two edges row by row from `position`, closing the remaining rows
/// once both edges leave the screen on the same side.
fn rasterize_xray_edges(
    write: &mut impl FnMut(i32, u16),
    mut position: i32,
    row_step: i32,
    mut edges: [u32; 2],
    steps: [u32; 2],
    finish_adjustment: i32,
) {
    let outside_rows = |position| {
        if row_step < 0 {
            position < 0
        } else {
            position >= i32::from(TABLE_BYTES)
        }
    };
    loop {
        let packed = advance_window_edges(&mut edges, steps, false);
        let hidden = (edges[1] as i32) < 0 || edges[0] as i32 >= 65536;
        let finished = edges[0] & !65535 != 0
            && edges[1] & !65535 != 0
            && (((steps[0] ^ edges[0]) | (steps[1] ^ edges[1])) >> 31) == 0;
        write((position >> 1) * 2, if hidden { CLOSED } else { packed });
        if finished {
            position += finish_adjustment;
            loop {
                position += row_step;
                if outside_rows(position) {
                    return;
                }
                write((position >> 1) * 2, CLOSED);
            }
        }
        position += row_step;
        if outside_rows(position) {
            return;
        }
    }
}

/// An edge's fixed-point start and per-row increment.
fn xray_edge(origin: u16, gradient: u16, positive: bool, offscreen: bool) -> (u32, u32) {
    let offset = if offscreen { 65536 } else { 0 };
    if positive {
        (u32::from(origin).wrapping_sub(offset), u32::from(gradient))
    } else {
        (
            u32::from(origin).wrapping_add(offset),
            (-i32::from(gradient)) as u32,
        )
    }
}

enum XrayBeamDirection {
    Right,
    Up,
    Down,
    Left,
    Horizontal,
}

/// A downward beam whose edges both advance positively: the table base, the
/// first row's byte offset, the shared origin and the two edge steps.
#[derive(Clone, Copy)]
struct PositiveRows {
    base: u32,
    start: u16,
    origin: u16,
    low_step: u16,
    high_step: u16,
}

/// $91:C87A's on-screen stores: the low edge writes whole words, then the
/// high edge replaces their high bytes, then the low edge's carry row closes.
fn store_onscreen_positive_rows(mut write: impl FnMut(u32, u8), rows: PositiveRows) {
    let PositiveRows {
        base,
        start,
        origin,
        low_step,
        high_step,
    } = rows;
    let mut low_edge = origin;
    let mut high_edge = origin;
    // Zero doubles as the ROM's no-carry sentinel, even for a carry at row 0.
    let mut carry_row = 0;
    // The ROM uses the N flag after CPY #$01CC, including wrapped Y.
    fn continues(index: u16) -> bool {
        index.wrapping_sub(TABLE_BYTES) & 0x8000 != 0
    }
    fn store_word(write: &mut impl FnMut(u32, u8), base: u32, index: u16, value: u16) {
        let address = base + u32::from(index);
        let [low, high] = value.to_le_bytes();
        write(address, low);
        write(address + 1, high);
    }

    let mut index = start;
    do_until! { !continues(index);
        let (value, carry) = low_edge.overflowing_add(low_step);
        low_edge = value;
        if carry {
            carry_row = index;
            do_until! { !continues(index);
                store_word(&mut write, base, index, 0x00ff);
                index = index.wrapping_add(2);
            }
            break;
        }
        // The high-edge pass overwrites this word's high byte.
        store_word(&mut write, base, index, low_edge >> 8);
        index = index.wrapping_add(2);
    }

    index = start.wrapping_add(1);
    loop {
        let (value, carry) = high_edge.overflowing_add(high_step);
        high_edge = value;
        if carry {
            do_until! { !continues(index);
                write(base + u32::from(index), 0xff);
                index = index.wrapping_add(2);
            }
            break;
        }
        write(base + u32::from(index), (high_edge >> 8) as u8);
        index = index.wrapping_add(2);
        if !continues(index) {
            // The ROM reaches cleanup only when this edge carries.
            return;
        }
    }

    if carry_row != 0 {
        index = carry_row.wrapping_add(2);
        while continues(index) {
            store_word(&mut write, base, index, 0x00ff);
            index = index.wrapping_add(2);
        }
    }
}

/// The off-screen stores: each edge runs until it carries, and the cleanup
/// passes depend on both edges' crossing rows.
fn store_offscreen_positive_rows(mut write: impl FnMut(u32, u8), rows: PositiveRows) {
    let PositiveRows {
        base,
        start,
        origin,
        low_step,
        high_step,
    } = rows;
    // The ROM wraps the 16-bit index before adding the table base.
    // At start zero, cleanup can therefore reach the following WRAM bank.
    fn store_byte(write: &mut impl FnMut(u32, u8), base: u32, index: i32, value: u8) {
        write(base + u32::from(index as u16), value);
    }
    fn store_word(write: &mut impl FnMut(u32, u8), base: u32, index: i32, value: u16) {
        let address = base + u32::from(index as u16);
        write(address, value as u8);
        write(address + 1, (value >> 8) as u8);
    }
    fn store(write: &mut impl FnMut(u32, u8), base: u32, index: i32, value: u16, high: bool) {
        if high {
            store_byte(write, base, index, value as u8);
        } else {
            store_word(write, base, index, value);
        }
    }
    /// Rasterize one edge; returns the row index the cleanup passes use.
    fn edge(
        write: &mut impl FnMut(u32, u8),
        base: u32,
        mut index: i32,
        mut value: u16,
        step: u16,
        high: bool,
    ) -> i32 {
        let end = i32::from(TABLE_BYTES);
        loop {
            let (next, carry) = value.overflowing_add(step);
            value = next;
            if carry {
                break;
            }
            index += 2;
            if index >= end {
                return index;
            }
        }
        store(write, base, index, value >> 8, high);
        index += 2;
        if index >= end {
            return if high { end - 1 } else { end - 2 };
        }
        let crossing = index - 4;
        loop {
            let (next, carry) = value.overflowing_add(step);
            value = next;
            if carry {
                break;
            }
            store(write, base, index, value >> 8, high);
            index += 2;
            if index >= end {
                return crossing;
            }
        }
        while index < end {
            store(write, base, index, CLOSED, high);
            index += 2;
        }
        crossing
    }
    // The low edge writes whole words first; the high edge then replaces
    // their high bytes. The cleanup passes depend on both crossing indices.
    let low = edge(&mut write, base, i32::from(start), origin, low_step, false);
    let high = edge(
        &mut write,
        base,
        i32::from(start) + 1,
        origin,
        high_step,
        true,
    );
    let mut index = low;
    do_until! { index < high;
        store_byte(&mut write, base, index, 0);
        index -= 2;
    }
    let mut index = high - 1;
    do_until! { index < 0;
        store_word(&mut write, base, index, CLOSED);
        index -= 2;
    }
}

/// The on-screen positive table: the anchor row, the edge stores, then the
/// rows above the start reopened until an already-open word.
fn fill_onscreen_positive_table(ram: &(impl crate::ram::RawRam + ?Sized), rows: PositiveRows) {
    let table = rows.base as usize;
    // Wrap the byte offset before adding the table base, including at row zero.
    let anchor = table + usize::from(rows.start.wrapping_sub(2));
    ram.fill(anchor..anchor + 2, (rows.origin >> 8) as u8);
    store_onscreen_positive_rows(
        |address, value| ram.write_byte(address as usize, value),
        rows,
    );

    let mut index = rows.start.wrapping_sub(4);
    loop {
        // The ROM checks the first word even when the initial index is negative.
        let address = table + usize::from(index);
        if u16::from_le_bytes([ram.read_byte(address), ram.read_byte(address + 1)]) == 0x00ff {
            return;
        }
        ram.copy_in(address, &0x00ffu16.to_le_bytes());
        index = index.wrapping_sub(2);
        if index & 0x8000 != 0 {
            return;
        }
    }
}

/// An X-ray style beam window: the beam origin relative to the camera, its
/// centre angle and half-width, whether the origin is off screen, and the
/// RAM address of the window table to fill.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct XrayWindow {
    pub x: u16,
    pub y: u16,
    pub angle: u16,
    pub width: u16,
    pub off_screen: bool,
    pub table: usize,
}

/// Fill `window.table` with the beam's per-row window words.
pub fn calculate_xray_window<H: Host + ?Sized>(window: XrayWindow, game: &mut H) {
    use XrayBeamDirection::*;
    let XrayWindow {
        x,
        y,
        angle,
        width,
        off_screen,
        table,
    } = window;
    fn gradient_index(angle: u16) -> usize {
        let half_turn_offset = angle.wrapping_sub(128);
        usize::from(if half_turn_offset & 0x8000 != 0 {
            angle
        } else {
            half_turn_offset
        })
    }

    // Preserve the SNES sign-bit tests; angle 256 stays distinct from zero.
    let mut lower_angle = angle.wrapping_sub(width);
    if lower_angle & 0x8000 != 0 {
        lower_angle = lower_angle.wrapping_add(256);
    }
    let mut upper_angle = angle.wrapping_add(width);
    if upper_angle.wrapping_sub(257) & 0x8000 == 0 {
        upper_angle = upper_angle.wrapping_sub(256);
    }
    let lower_gradient = game.read_rom_word(XRAY_GRADIENTS, gradient_index(lower_angle) * 2);
    let upper_gradient = game.read_rom_word(XRAY_GRADIENTS, gradient_index(upper_angle) * 2);
    let direction = if width == 0 && (angle == 64 || angle == 192) {
        Horizontal
    } else if lower_angle.wrapping_sub(128) & 0x8000 != 0 {
        if lower_angle.wrapping_sub(64) & 0x8000 == 0 {
            Down
        } else if upper_angle.wrapping_sub(64) & 0x8000 == 0 {
            Right
        } else {
            Up
        }
    } else if lower_angle.wrapping_sub(192) & 0x8000 == 0 {
        Up
    } else if upper_angle.wrapping_sub(192) & 0x8000 == 0 {
        Left
    } else {
        Down
    };
    let origin = x << 8;
    let position = 2 * i32::from(y);
    let up_start = position - if off_screen { 2 } else { 4 };
    // Generic rows retain signed offsets; positive-down helpers wrap their u16 offsets.
    let mut write = |offset: i32, value: u16| {
        let address = table
            .checked_add_signed(offset as isize)
            .expect("X-ray table offset");
        game.ram().copy_in(address, &value.to_ne_bytes());
    };
    if matches!(direction, Horizontal) {
        let anchor = y.wrapping_sub(1).wrapping_mul(2);
        let edge = origin >> 8;
        let value = if off_screen {
            0xff00
        } else if angle == 64 {
            edge | 0xff00
        } else {
            edge << 8
        };
        write(i32::from(anchor & !1), value);
        close_rows(&mut write, anchor.wrapping_sub(2), false);
        close_rows(&mut write, anchor.wrapping_add(2), true);
        return;
    }
    if matches!(direction, Down) && upper_angle.wrapping_sub(128) & 0x8000 != 0 {
        let rows = PositiveRows {
            base: table as u32,
            start: y.wrapping_mul(2),
            origin,
            low_step: upper_gradient,
            high_step: lower_gradient,
        };
        if off_screen {
            store_offscreen_positive_rows(
                |address, value| game.ram().write_byte(address as usize, value),
                rows,
            );
        } else {
            fill_onscreen_positive_table(game.ram(), rows);
        }
        return;
    }
    if !off_screen {
        let edge = origin >> 8;
        let value = match direction {
            Right => edge | 0xff00,
            Left => edge << 8,
            _ => edge * 0x101,
        };
        write(((position - 2) >> 1) * 2, value);
    }
    if matches!(direction, Right | Left) {
        let rightward = matches!(direction, Right);
        let gradients = if rightward {
            [lower_gradient, upper_gradient]
        } else {
            [upper_gradient, lower_gradient]
        };
        for (start, step, gradient) in [(up_start, -2, gradients[0]), (position, 2, gradients[1])] {
            let (edge, increment) = xray_edge(origin, gradient, rightward, off_screen);
            let (edges, steps) = if rightward {
                ([edge, 65535], [increment, 1])
            } else {
                ([0, edge], [u32::MAX, increment])
            };
            rasterize_xray_edges(&mut write, start, step, edges, steps, -step);
        }
    } else {
        let up = matches!(direction, Up);
        let (positive_left, positive_right, gradients) = if up {
            let left = lower_angle.wrapping_sub(192) & 0x8000 != 0;
            (
                left,
                left || upper_angle.wrapping_sub(192) & 0x8000 != 0,
                [lower_gradient, upper_gradient],
            )
        } else {
            (
                false,
                lower_angle.wrapping_sub(128) & 0x8000 != 0,
                [upper_gradient, lower_gradient],
            )
        };
        let left = xray_edge(origin, gradients[0], positive_left, off_screen);
        let right = xray_edge(origin, gradients[1], positive_right, off_screen);
        rasterize_xray_edges(
            &mut write,
            if up { up_start } else { position },
            if up { -2 } else { 2 },
            [left.0, right.0],
            [left.1, right.1],
            if up && !positive_right { 2 } else { 0 },
        );
        close_rows(
            &mut write,
            if up { position as u16 } else { up_start as u16 },
            up,
        );
    }
}

/// One native-order word of the rainbow beam's HDMA tables at RAM `at`.
fn store_rainbow_word<H: Host + ?Sized>(at: isize, value: u16, game: &mut H) {
    crate::word_transfers!(game, ascending, 1; native(at) <- value(value));
}

/// The two quadrant bits of a beam edge angle.
fn quadrant(angle: u16) -> u16 {
    (angle >> 6) & 3
}

/// Fill the rainbow beam's window table for Mother Brain's current beam.
pub fn calculate_rainbow_window<H: Host + ?Sized>(game: &mut H) {
    let half_width = u16::from(slot!(byte game, MotherBrainBody[0].beam_width[1]) >> 1);
    let angle = slot!(game, MotherBrainBody[0].beam_angle);
    slot!(
        game,
        MotherBrainBody[0].beam_right_edge = angle.wrapping_sub(half_width)
    );
    let angle = slot!(game, MotherBrainBody[0].beam_angle);
    slot!(
        game,
        MotherBrainBody[0].beam_left_edge = half_width.wrapping_add(angle)
    );
    let origin_x =
        ((i32::from(slot!(game, MotherBrainBody[0x41].base.enemy_ptr)) + 3584) & 0xff00) as u16;
    slot!(game, MotherBrainBody[0]; beam_left_origin_x = origin_x, beam_right_origin_x = origin_x);
    let origin_y = slot!(game, MotherBrainBrain[0x40].base.y_pos).wrapping_add(5);
    slot!(game, MotherBrainBody[0]; beam_left_origin_y = origin_y, beam_right_origin_y = origin_y);
    let right_quadrant = quadrant(slot!(game, MotherBrainBody[0].beam_right_edge));
    let left_quadrant = quadrant(slot!(game, MotherBrainBody[0].beam_left_edge));
    match (right_quadrant << 2) | left_quadrant {
        0 | 12 | 15 => {
            let direction = match (right_quadrant & 2) | (left_quadrant >> 1) {
                0 => RainbowDirection::DownRight,
                2 => RainbowDirection::Down,
                3 => RainbowDirection::DownLeft,
                _ => panic!("non-null function pointer"),
            };
            let edges = rainbow_origins(game);
            rasterize_rainbow_vertical(direction, edges, game);
            store_rainbow_header(0x9de6, game);
        }
        1 => {
            let edges = rainbow_origins(game);
            rasterize_rainbow_right_edges(edges, game);
            store_rainbow_header(0x9dec, game);
        }
        5 | 6 | 10 => rasterize_rainbow_up(game),
        11 => {}
        _ => panic!("non-null function pointer"),
    }
}

/// The beam's left and right origin X, as the rasterizers start from them.
fn rainbow_origins<H: Host + ?Sized>(game: &mut H) -> [u16; 2] {
    [
        slot!(game, MotherBrainBody[0].beam_left_origin_x),
        slot!(game, MotherBrainBody[0].beam_right_origin_x),
    ]
}

/// The downward header: two closed 16-row blocks, then the rows at `tail`.
fn store_rainbow_header<H: Host + ?Sized>(tail: u16, game: &mut H) {
    store_rainbow_word(RAINBOW_HEADER, 16, game);
    store_rainbow_word(RAINBOW_HEADER + 1, 0x9c00, game);
    store_rainbow_word(RAINBOW_HEADER + 3, 16, game);
    store_rainbow_word(RAINBOW_HEADER + 4, 0x9c03, game);
    store_rainbow_word(RAINBOW_HEADER + 6, 240, game);
    store_rainbow_word(RAINBOW_HEADER + 7, 0x9d04, game);
    store_rainbow_word(RAINBOW_HEADER + 9, 244, game);
    store_rainbow_word(RAINBOW_HEADER + 10, tail, game);
    store_rainbow_word(RAINBOW_HEADER + 12, 0, game);
}

/// A beam opening to the right of both edges: the right edge's rows run up
/// from the origin row, the left edge's rows run down.
fn rasterize_rainbow_right_edges<H: Host + ?Sized>(origins: [u16; 2], game: &mut H) {
    let [mut lower_edge, mut upper_edge] = origins;
    word! { game, 255; hdma_table_2[0], hdma_table_2[1] }
    let lower_step = rainbow_gradient(
        game,
        slot!(byte game, MotherBrainBody[0].beam_right_edge[0]) as isize,
    );
    let upper_step = rainbow_gradient(
        game,
        slot!(byte game, MotherBrainBody[0].beam_left_edge[0]) as isize,
    );
    let mut row = slot!(game, MotherBrainBody[0].beam_right_origin_y);
    let mut dst = RAINBOW_ROWS + 2 * (row as isize - 32 + 1);
    store_rainbow_word(dst + 2, CLOSED, game);
    store_rainbow_word(dst + 4, CLOSED, game);
    let origin_dst = dst;
    do_until! { row == 32;
        let (value, overflow) = upper_step.overflowing_add(upper_edge);
        if overflow {
            break;
        }
        upper_edge = value;
        store_rainbow_word(dst, u16::from(value.to_ne_bytes()[1]) | 0xff00, game);
        dst -= 2;
        row = row.wrapping_sub(1);
    }
    do_until! { row == 32;
        store_rainbow_word(dst, CLOSED, game);
        dst -= 2;
        row = row.wrapping_sub(1);
    }
    let mut dst = origin_dst + 2;
    let mut row = slot!(game, BeamScratch[0x7088].row);
    loop {
        let (value, overflow) = lower_step.overflowing_add(lower_edge);
        if overflow {
            break;
        }
        lower_edge = value;
        store_rainbow_word(dst, u16::from(value.to_ne_bytes()[1]) | 0xff00, game);
        dst += 2;
        row = row.wrapping_add(1);
        if row == 232 {
            return;
        }
    }
    do_until! { row == 232;
        store_rainbow_word(dst, CLOSED, game);
        dst += 2;
        row = row.wrapping_add(1);
    }
}

/// An upward beam: its rows, then a header sized by the origin row.
fn rasterize_rainbow_up<H: Host + ?Sized>(game: &mut H) {
    let right_quadrant = quadrant(slot!(game, MotherBrainBody[0].beam_right_edge));
    let left_quadrant = quadrant(slot!(game, MotherBrainBody[0].beam_left_edge));
    let direction = match (right_quadrant & 2) | (left_quadrant >> 1) {
        0 => RainbowDirection::UpRight,
        1 => RainbowDirection::Up,
        3 => RainbowDirection::UpLeft,
        _ => panic!("non-null function pointer"),
    };
    let edges = rainbow_origins(game);
    rasterize_rainbow_vertical(direction, edges, game);
    store_rainbow_word(RAINBOW_HEADER, 16, game);
    store_rainbow_word(RAINBOW_HEADER + 1, 0x9d00, game);
    store_rainbow_word(RAINBOW_HEADER + 3, 16, game);
    store_rainbow_word(RAINBOW_HEADER + 4, 0x9d02, game);
    let rows_above = slot!(game, MotherBrainBody[0].beam_left_origin_y).wrapping_sub(32);
    game.print_carry_message();
    if rows_above >= 0x80 {
        // The rows above 127 need a second indirect block.
        let extra = i32::from(rows_above) - 127;
        store_rainbow_word(RAINBOW_HEADER + 6, (extra | 0x80) as u16, game);
        store_rainbow_word(RAINBOW_HEADER + 7, 0x9d04, game);
        store_rainbow_word(RAINBOW_HEADER + 9, 255, game);
        store_rainbow_word(RAINBOW_HEADER + 10, (2 * extra + 0x9d04) as u16, game);
        store_rainbow_word(RAINBOW_HEADER + 12, 127, game);
        store_rainbow_word(RAINBOW_HEADER + 13, (2 * extra + 0x9d04 + 254) as u16, game);
        store_rainbow_word(RAINBOW_HEADER + 15, 0, game);
    } else {
        store_rainbow_word(RAINBOW_HEADER + 6, rows_above | 0x80, game);
        store_rainbow_word(RAINBOW_HEADER + 7, 0x9d04, game);
        store_rainbow_word(RAINBOW_HEADER + 9, 127, game);
        store_rainbow_word(
            RAINBOW_HEADER + 10,
            (2 * i32::from(rows_above) + 0x9d04) as u16,
            game,
        );
        store_rainbow_word(RAINBOW_HEADER + 12, 0, game);
    }
}

#[derive(Clone, Copy)]
enum RainbowDirection {
    UpRight,
    Up,
    UpLeft,
    DownRight,
    Down,
    DownLeft,
}

/// A beam opening up or down: both edges advance together from the origins.
fn rasterize_rainbow_vertical<H: Host + ?Sized>(
    direction: RainbowDirection,
    origins: [u16; 2],
    game: &mut H,
) {
    use RainbowDirection::*;
    let down = matches!(direction, DownRight | Down | DownLeft);
    let add_left = matches!(direction, UpRight | DownRight);
    let add_right = matches!(direction, UpRight | Up | DownRight | Down);
    let zero_sentinel = matches!(direction, DownLeft);
    word! { game, 255; hdma_table_2[0], hdma_table_2[1] }
    let edge_angle =
        |angle: u8, add: bool| (if add { angle } else { angle.wrapping_neg() }) as isize;
    let left_angle = if down {
        slot!(byte game, MotherBrainBody[0].beam_right_edge[0])
    } else {
        slot!(byte game, MotherBrainBody[0].beam_left_edge[0])
    };
    let left_step = rainbow_gradient(game, edge_angle(left_angle, add_left));
    let right_angle = if down {
        slot!(byte game, MotherBrainBody[0].beam_left_edge[0])
    } else {
        slot!(byte game, MotherBrainBody[0].beam_right_edge[0])
    };
    let right_step = rainbow_gradient(game, edge_angle(right_angle, add_right));
    let mut row = slot!(game, MotherBrainBody[0].beam_left_origin_y);
    let mut dst = if down {
        let mut remaining = row.wrapping_sub(32);
        let mut dst = RAINBOW_ROWS + 4;
        do_until! { remaining == 0;
            store_rainbow_word(dst, CLOSED, game);
            dst += 2;
            remaining = remaining.wrapping_sub(1);
        }
        // Preserve the original post-fill read of the starting row.
        row = slot!(game, MotherBrainBody[0].beam_left_origin_y);
        dst
    } else {
        let dst = RAINBOW_ROWS + 2 * (row as isize - 32 + 1);
        store_rainbow_word(dst + 2, CLOSED, game);
        store_rainbow_word(dst + 4, CLOSED, game);
        dst
    };
    let signed_step = |step: u16, add: bool| {
        if add {
            u32::from(step)
        } else {
            (-i32::from(step)) as u32
        }
    };
    let mut edges = origins.map(u32::from);
    let steps = [
        signed_step(left_step, add_left),
        signed_step(right_step, add_right),
    ];
    let last_row = if down { 232 } else { 32 };
    do_until! { row == last_row;
        let mut packed = advance_window_edges(&mut edges, steps, true);
        if packed == if zero_sentinel { 0 } else { 0xffff } {
            packed = CLOSED;
        }
        store_rainbow_word(dst, packed, game);
        dst += if down { 2 } else { -2 };
        row = if down {
            row.wrapping_add(1)
        } else {
            row.wrapping_sub(1)
        };
    }
}
