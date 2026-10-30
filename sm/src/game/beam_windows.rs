use crate::game::imported::{Imported, word};
use crate::game_state::Game;
use crate::ram_map::{slot, word};
use crate::types::Ram8000_Default;
use sm_data::beam_gradients::{ADDRESS, BYTES, BeamGradients};
use sm_game::actors::mother_brain::{MotherBrainBody, MotherBrainBrain};

pub(crate) type ImportedTables = Option<Imported<BeamGradients>>;

pub(crate) fn import(image: &crate::rom::RomImage) -> ImportedTables {
    Imported::import(image, ADDRESS, BYTES, sm_data::beam_gradients::import)
}

/// The edge gradient for `angle`; other images and angles past 80h run the
/// original indexed load.
fn gradient(game: &Game, angle: isize) -> u16 {
    word(&game.beam_gradients, game, ADDRESS, angle)
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

fn fill_closed_xray_rows(write: &mut impl FnMut(i32, u16), mut position: u16, down: bool) {
    loop {
        write(i32::from(position & !1), 255);
        position = position.wrapping_add(if down { 2 } else { 65534 });
        if if down {
            position >= 460
        } else {
            position & 0x8000 != 0
        } {
            return;
        }
    }
}

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
            position >= 460
        }
    };
    loop {
        let packed = advance_window_edges(&mut edges, steps, false);
        let hidden = (edges[1] as i32) < 0 || edges[0] as i32 >= 65536;
        let finished = edges[0] & !65535 != 0
            && edges[1] & !65535 != 0
            && ((steps[0] ^ edges[0] | steps[1] ^ edges[1]) >> 31) == 0;
        write((position >> 1) * 2, if hidden { 255 } else { packed });
        if finished {
            position += finish_adjustment;
            loop {
                position += row_step;
                if outside_rows(position) {
                    return;
                }
                write((position >> 1) * 2, 255);
            }
        }
        position += row_step;
        if outside_rows(position) {
            return;
        }
    }
}

fn xray_edge(origin: u16, gradient: u16, positive: bool, offscreen: bool) -> (u32, u32) {
    let offset = if offscreen { 65536 } else { 0 };
    if positive {
        (u32::from(origin).wrapping_sub(offset), u32::from(gradient))
    } else {
        (
            u32::from(origin).wrapping_add(offset),
            (-(i32::from(gradient))) as u32,
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

fn xray_onscreen_positive_stores(
    mut write: impl FnMut(u32, u8),
    base: u32,
    start: u16,
    origin: u16,
    low_step: u16,
    high_step: u16,
) {
    let mut low_edge = origin;
    let mut high_edge = origin;
    // Zero doubles as the ROM's no-carry sentinel, even for a carry at row 0.
    let mut carry_row = 0;
    // $91:C87A uses the N flag after CPY #$01CC, including wrapped Y.
    fn continues(index: u16) -> bool {
        index.wrapping_sub(460) & 0x8000 != 0
    }
    fn word(write: &mut impl FnMut(u32, u8), base: u32, index: u16, value: u16) {
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
                word(&mut write, base, index, 0x00ff);
                index = index.wrapping_add(2);
            }
            break;
        }
        // The high-edge pass overwrites this word's high byte.
        let value = low_edge >> 8;
        word(&mut write, base, index, value);
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
            word(&mut write, base, index, 0x00ff);
            index = index.wrapping_add(2);
        }
    }
}

fn xray_offscreen_positive_stores(
    mut write: impl FnMut(u32, u8),
    base: u32,
    start: u16,
    origin: u16,
    low_step: u16,
    high_step: u16,
) {
    // The ROM wraps the 16-bit index before adding the table base.
    // At start zero, cleanup can therefore reach the following WRAM bank.
    fn byte(write: &mut impl FnMut(u32, u8), base: u32, index: i32, value: u8) {
        write(base + u32::from(index as u16), value);
    }
    fn word(write: &mut impl FnMut(u32, u8), base: u32, index: i32, value: u16) {
        let address = base + u32::from(index as u16);
        write(address, value as u8);
        write(address + 1, (value >> 8) as u8);
    }
    fn edge(
        write: &mut impl FnMut(u32, u8),
        base: u32,
        mut index: i32,
        mut value: u16,
        step: u16,
        high: bool,
    ) -> i32 {
        loop {
            let (next, carry) = value.overflowing_add(step);
            value = next;
            if carry {
                break;
            }
            index += 2;
            if index >= 460 {
                return index;
            }
        }
        if high {
            byte(write, base, index, (value >> 8) as u8);
        } else {
            word(write, base, index, value >> 8);
        }
        index += 2;
        if index >= 460 {
            return if high { 459 } else { 458 };
        }
        let crossing = index - 4;
        loop {
            let (next, carry) = value.overflowing_add(step);
            value = next;
            if carry {
                break;
            }
            if high {
                byte(write, base, index, (value >> 8) as u8);
            } else {
                word(write, base, index, value >> 8);
            }
            index += 2;
            if index >= 460 {
                return crossing;
            }
        }
        while index < 460 {
            if high {
                byte(write, base, index, 255);
            } else {
                word(write, base, index, 255);
            }
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
        byte(&mut write, base, index, 0);
        index -= 2;
    }
    let mut index = high - 1;
    do_until! { index < 0;
        word(&mut write, base, index, 255);
        index -= 2;
    }
}

fn fill_xray_onscreen_positive_table(
    ram: &crate::game_state::WorkRam,
    table: usize,
    start: u16,
    origin: u16,
    low_step: u16,
    high_step: u16,
) {
    // Wrap the byte offset before adding the table base, including at row zero.
    let anchor = table + usize::from(start.wrapping_sub(2));
    ram.fill(anchor..anchor + 2, (origin >> 8) as u8);
    xray_onscreen_positive_stores(
        |address, value| ram.write_byte(address as usize, value),
        table as u32,
        start,
        origin,
        low_step,
        high_step,
    );

    let mut index = start.wrapping_sub(4);
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

pub fn calculate_xray_window(
    relative_x: u16,
    relative_y: u16,
    angle: u16,
    width: u16,
    off_screen: bool,
    table: usize,
    game: &mut Game,
) {
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
    let lower_gradient = game.read_rom_word(0x91c9d4, gradient_index(lower_angle) * 2);
    let upper_gradient = game.read_rom_word(0x91c9d4, gradient_index(upper_angle) * 2);
    let direction = if width == 0 && (angle == 64 || angle == 192) {
        XrayBeamDirection::Horizontal
    } else if lower_angle.wrapping_sub(128) & 0x8000 != 0 {
        if lower_angle.wrapping_sub(64) & 0x8000 == 0 {
            XrayBeamDirection::Down
        } else if upper_angle.wrapping_sub(64) & 0x8000 == 0 {
            XrayBeamDirection::Right
        } else {
            XrayBeamDirection::Up
        }
    } else if lower_angle.wrapping_sub(192) & 0x8000 == 0 {
        XrayBeamDirection::Up
    } else if upper_angle.wrapping_sub(192) & 0x8000 == 0 {
        XrayBeamDirection::Left
    } else {
        XrayBeamDirection::Down
    };
    let origin = relative_x << 8;
    let position = 2 * i32::from(relative_y);
    let up_start = position - if off_screen { 2 } else { 4 };
    use XrayBeamDirection::*;
    // Generic rows retain signed offsets; positive-down helpers wrap their u16 offsets.
    let mut write = |offset: i32, value: u16| {
        let address = table
            .checked_add_signed(offset as isize)
            .expect("X-ray table offset");
        game.ram().copy_in(address, &value.to_ne_bytes());
    };
    if matches!(direction, Horizontal) {
        let anchor = relative_y.wrapping_sub(1).wrapping_mul(2);
        let edge = origin >> 8;
        let value = if off_screen {
            0xff00
        } else if angle == 64 {
            edge | 0xff00
        } else {
            edge << 8
        };
        write(i32::from(anchor & !1), value);
        fill_closed_xray_rows(&mut write, anchor.wrapping_sub(2), false);
        fill_closed_xray_rows(&mut write, anchor.wrapping_add(2), true);
        return;
    }
    if matches!(direction, Down) && upper_angle.wrapping_sub(128) & 0x8000 != 0 {
        let start = relative_y.wrapping_mul(2);
        if off_screen {
            xray_offscreen_positive_stores(
                |address, value| game.ram().write_byte(address as usize, value),
                table as u32,
                start,
                origin,
                upper_gradient,
                lower_gradient,
            );
        } else {
            fill_xray_onscreen_positive_table(
                game.ram(),
                table,
                start,
                origin,
                upper_gradient,
                lower_gradient,
            );
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
        fill_closed_xray_rows(
            &mut write,
            if up { position as u16 } else { up_start as u16 },
            up,
        );
    }
}

use crate::c_library::printf;
#[forbid(unsafe_code)]
fn write_rainbow_word(value: u16, at: isize, game: &mut Game) {
    word_transfers!(game, ascending, 1; native(at) <- value(value));
}

pub unsafe fn calculate_rainbow_window(game: &mut Game) {
    unsafe {
        let mut r18 = u16::from(slot!(byte game, MotherBrainBody[0].beam_width[1]) >> 1);
        slot!(
            game,
            MotherBrainBody[0].beam_right_edge =
                slot!(game, MotherBrainBody[0].beam_angle).wrapping_sub(r18)
        );
        slot!(
            game,
            MotherBrainBody[0].beam_left_edge =
                r18.wrapping_add(slot!(game, MotherBrainBody[0].beam_angle))
        );
        let v2 =
            (i32::from(slot!(game, MotherBrainBody[0x41].base.enemy_ptr)) + 3584 & 0xff00) as u16;
        slot!(game, MotherBrainBody[0]; beam_left_origin_x = v2, beam_right_origin_x = v2);
        let v3 = slot!(game, MotherBrainBrain[0x40].base.y_pos).wrapping_add(5);
        slot!(game, MotherBrainBody[0]; beam_left_origin_y = v3, beam_right_origin_y = v3);
        r18 = (4 * (i32::from(slot!(game, MotherBrainBody[0].beam_right_edge)) & 0xc0)) as u16;
        let i = (i32::from(r18) | i32::from(slot!(game, MotherBrainBody[0].beam_left_edge)) & 0xc0)
            >> 6;
        match i {
            0 | 12 | 15 => {
                let direction = match (slot!(game, MotherBrainBody[0].beam_right_edge) >> 6 & 2)
                    | (slot!(game, MotherBrainBody[0].beam_left_edge) >> 7 & 1)
                {
                    0 => RainbowDirection::DownRight,
                    2 => RainbowDirection::Down,
                    3 => RainbowDirection::DownLeft,
                    _ => panic!("non-null function pointer"),
                };
                let edges = [
                    slot!(game, MotherBrainBody[0].beam_left_origin_x),
                    slot!(game, MotherBrainBody[0].beam_right_origin_x),
                ];
                rasterize_rainbow_vertical(direction, edges[0], edges[1], game);
                write_rainbow_fixed_header(0x9de6, game);
            }
            1 => {
                let edges = [
                    slot!(game, MotherBrainBody[0].beam_left_origin_x),
                    slot!(game, MotherBrainBody[0].beam_right_origin_x),
                ];
                rasterize_rainbow_right_edges(edges[0], edges[1], game);
                write_rainbow_fixed_header(0x9dec, game);
            }
            5 | 6 | 10 => rasterize_rainbow_up(game),
            11 => (),
            _ => panic!("non-null function pointer"),
        }
    }
}
#[forbid(unsafe_code)]
fn write_rainbow_fixed_header(tail: u16, game: &mut Game) {
    write_rainbow_word(16, 0x9c00, game);
    write_rainbow_word(-25600i32 as u16, 0x9c01, game);
    write_rainbow_word(16, 0x9c03, game);
    write_rainbow_word(-25597i32 as u16, 0x9c04, game);
    write_rainbow_word(240, 0x9c06, game);
    write_rainbow_word(-25340i32 as u16, 0x9c07, game);
    write_rainbow_word(244, 0x9c09, game);
    write_rainbow_word(tail, 0x9c0a, game);
    write_rainbow_word(0, 0x9c0c, game);
}
#[forbid(unsafe_code)]
fn rasterize_rainbow_right_edges(mut r22: u16, mut r24: u16, game: &mut Game) {
    word! { game, 255; hdma_table_2[0], hdma_table_2[1] }
    let r18 = gradient(
        game,
        slot!(byte game, MotherBrainBody[0].beam_right_edge[0]) as isize,
    );
    let r20 = gradient(
        game,
        slot!(byte game, MotherBrainBody[0].beam_left_edge[0]) as isize,
    );
    let mut mbn_var_3F = slot!(game, MotherBrainBody[0].beam_right_origin_y);
    let mut dst = 0x9d00 + 2 * (mbn_var_3F as isize - 32 + 1);
    write_rainbow_word(255, dst + 2, game);
    write_rainbow_word(255, dst + 4, game);
    let initial_dst = dst;
    do_until! { mbn_var_3F == 32;
        let (value, overflow) = r20.overflowing_add(r24);
        if overflow {
            break;
        }
        r24 = value;
        crate::update_place! { previous current = dst => dst - 2; }
        write_rainbow_word(u16::from(value.to_ne_bytes()[1]) | 0xff00, current, game);
        mbn_var_3F = mbn_var_3F.wrapping_sub(1);
    }
    do_until! { mbn_var_3F == 32;
        crate::update_place! { previous current = dst => dst - 2; }
        write_rainbow_word(255, current, game);
        mbn_var_3F = mbn_var_3F.wrapping_sub(1);
    }
    let mut dst = initial_dst + 2;
    let mut var_3F = slot!(game, Ram8000_Default[0x7088].var_3F);
    loop {
        let (value, overflow) = r18.overflowing_add(r22);
        if overflow {
            break;
        }
        r22 = value;
        crate::update_place! { previous current = dst => dst + 2; }
        write_rainbow_word(u16::from(value.to_ne_bytes()[1]) | 0xff00, current, game);
        var_3F = var_3F.wrapping_add(1);
        if var_3F == 232 {
            return;
        }
    }
    do_until! { var_3F == 232;
        crate::update_place! { previous current = dst => dst + 2; }
        write_rainbow_word(255, current, game);
        var_3F = var_3F.wrapping_add(1);
    }
}
unsafe fn rasterize_rainbow_up(game: &mut Game) {
    unsafe {
        let direction = match (slot!(game, MotherBrainBody[0].beam_right_edge) >> 6 & 2)
            | (slot!(game, MotherBrainBody[0].beam_left_edge) >> 7 & 1)
        {
            0 => RainbowDirection::UpRight,
            1 => RainbowDirection::Up,
            3 => RainbowDirection::UpLeft,
            _ => panic!("non-null function pointer"),
        };
        let edges = [
            slot!(game, MotherBrainBody[0].beam_left_origin_x),
            slot!(game, MotherBrainBody[0].beam_right_origin_x),
        ];
        rasterize_rainbow_vertical(direction, edges[0], edges[1], game);
        write_rainbow_word(16, 0x9c00, game);
        write_rainbow_word(-25344i32 as u16, 0x9c01, game);
        write_rainbow_word(16, 0x9c03, game);
        write_rainbow_word(-25342i32 as u16, 0x9c04, game);
        let v2 = slot!(game, MotherBrainBody[0].beam_left_origin_y).wrapping_sub(32);
        printf(c"Carry crap!\n".as_ptr());
        if v2 >= 0x80 {
            write_rainbow_word((v2 as i32 - 127 | 0x80) as u16, 0x9c06, game);
            write_rainbow_word(-25340i32 as u16, 0x9c07, game);
            write_rainbow_word(255, 0x9c09, game);
            write_rainbow_word((2 * (v2 as i32 - 127) - 25340) as u16, 0x9c0a, game);
            write_rainbow_word(127, 0x9c0c, game);
            write_rainbow_word((2 * (v2 as i32 - 127) - 25340 + 254) as u16, 0x9c0d, game);
            write_rainbow_word(0, 0x9c0f, game);
        } else {
            write_rainbow_word(v2 | 0x80, 0x9c06, game);
            write_rainbow_word(-25340i32 as u16, 0x9c07, game);
            write_rainbow_word(127, 0x9c09, game);
            write_rainbow_word((2 * v2 as i32 - 25340) as u16, 0x9c0a, game);
            write_rainbow_word(0, 0x9c0c, game);
        };
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

#[forbid(unsafe_code)]
fn rasterize_rainbow_vertical(direction: RainbowDirection, left: u16, right: u16, game: &mut Game) {
    use RainbowDirection::*;
    let down = matches!(direction, DownRight | Down | DownLeft);
    let add_left = matches!(direction, UpRight | DownRight);
    let add_right = matches!(direction, UpRight | Up | DownRight | Down);
    let zero_sentinel = matches!(direction, DownLeft);
    word! { game, 255; hdma_table_2[0], hdma_table_2[1] }
    let left_angle = {
        let angle = if down {
            slot!(byte game, MotherBrainBody[0].beam_right_edge[0])
        } else {
            slot!(byte game, MotherBrainBody[0].beam_left_edge[0])
        };
        (if add_left {
            angle
        } else {
            (-(i32::from(angle))) as u8
        }) as isize
    };
    let left_step = gradient(game, left_angle);
    let right_angle = {
        let angle = if down {
            slot!(byte game, MotherBrainBody[0].beam_left_edge[0])
        } else {
            slot!(byte game, MotherBrainBody[0].beam_right_edge[0])
        };
        (if add_right {
            angle
        } else {
            (-(i32::from(angle))) as u8
        }) as isize
    };
    let right_step = gradient(game, right_angle);
    let mut row = slot!(game, MotherBrainBody[0].beam_left_origin_y);
    let mut dst = if down {
        let mut remaining = row.wrapping_sub(32);
        let mut dst = 0x9d00 + 4;
        do_until! { remaining == 0;
            crate::update_place! { previous current = dst => dst + 2; }
            write_rainbow_word(255, current, game);
            remaining = remaining.wrapping_sub(1);
        }
        // Preserve the original post-fill read of the starting row.
        row = slot!(game, MotherBrainBody[0].beam_left_origin_y);
        dst
    } else {
        let dst = 0x9d00 + 2 * (row as isize - 32 + 1);
        write_rainbow_word(255, dst + 2, game);
        write_rainbow_word(255, dst + 4, game);
        dst
    };
    let mut edges = [u32::from(left), u32::from(right)];
    let steps = [
        if add_left {
            u32::from(left_step)
        } else {
            (-(i32::from(left_step))) as u32
        },
        if add_right {
            u32::from(right_step)
        } else {
            (-(i32::from(right_step))) as u32
        },
    ];
    do_until! { row == if down { 232 } else { 32 };
        let mut packed = advance_window_edges(&mut edges, steps, true);
        if packed == if zero_sentinel { 0 } else { 0xffff } {
            packed = 255;
        }
        crate::update_place! { previous current = dst => dst + if down { 2 } else { -2 }; }
        write_rainbow_word(packed, current, game);
        row = if down {
            row.wrapping_add(1)
        } else {
            row.wrapping_sub(1)
        };
    }
}
