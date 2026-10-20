use crate::game_state::Game;
use crate::ram_map::{ram_ptr, word};
use crate::types::{Enemy_MotherBrain, Ram8000_Default};

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

unsafe fn fill_closed_xray_rows(dest: *mut u16, mut position: u16, down: bool) {
    loop {
        *dest.add(usize::from(position >> 1)) = 255;
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

unsafe fn rasterize_xray_edges(
    dest: *mut u16,
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
        *dest.offset((position >> 1) as isize) = if hidden { 255 } else { packed };
        if finished {
            position += finish_adjustment;
            loop {
                position += row_step;
                if outside_rows(position) {
                    return;
                }
                *dest.offset((position >> 1) as isize) = 255;
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
    ram: &mut [u8],
    table: usize,
    start: u16,
    origin: u16,
    low_step: u16,
    high_step: u16,
) {
    // Wrap the byte offset before adding the table base, including at row zero.
    let anchor = table + usize::from(start.wrapping_sub(2));
    ram[anchor..anchor + 2].fill((origin >> 8) as u8);
    xray_onscreen_positive_stores(
        |address, value| ram[address as usize] = value,
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
        if u16::from_le_bytes([ram[address], ram[address + 1]]) == 0x00ff {
            return;
        }
        ram[address..address + 2].copy_from_slice(&0x00ffu16.to_le_bytes());
        index = index.wrapping_sub(2);
        if index & 0x8000 != 0 {
            return;
        }
    }
}

pub unsafe fn calculate_xray_window(
    relative_x: u16,
    relative_y: u16,
    angle: u16,
    width: u16,
    off_screen: bool,
    dest_addr: *mut u16,
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
    if matches!(direction, Horizontal) {
        let anchor = relative_y.wrapping_sub(1).wrapping_mul(2);
        let edge = origin >> 8;
        *dest_addr.add(usize::from(anchor >> 1)) = if off_screen {
            0xff00
        } else if angle == 64 {
            edge | 0xff00
        } else {
            edge << 8
        };
        fill_closed_xray_rows(dest_addr, anchor.wrapping_sub(2), false);
        fill_closed_xray_rows(dest_addr, anchor.wrapping_add(2), true);
        return;
    }
    if matches!(direction, Down) && upper_angle.wrapping_sub(128) & 0x8000 != 0 {
        let table = dest_addr.cast::<u8>().offset_from(game.g_ram.as_ptr()) as usize;
        let start = relative_y.wrapping_mul(2);
        if off_screen {
            xray_offscreen_positive_stores(
                |address, value| game.g_ram[address as usize] = value,
                table as u32,
                start,
                origin,
                upper_gradient,
                lower_gradient,
            );
        } else {
            fill_xray_onscreen_positive_table(
                &mut game.g_ram,
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
        *dest_addr.offset(((position - 2) >> 1) as isize) = match direction {
            Right => edge | 0xff00,
            Left => edge << 8,
            _ => edge * 0x101,
        };
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
            rasterize_xray_edges(dest_addr, start, step, edges, steps, -step);
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
            dest_addr,
            if up { up_start } else { position },
            if up { -2 } else { 2 },
            [left.0, right.0],
            [left.1, right.1],
            if up && !positive_right { 2 } else { 0 },
        );
        fill_closed_xray_rows(
            dest_addr,
            if up { position as u16 } else { up_start as u16 },
            up,
        );
    }
}

use crate::c_library::printf;
use core::ffi::c_char;
crate::ram_map::ram_pointer_functions! {
    rainbow_enemy(o: i32) -> Enemy_MotherBrain = [0xf78, o as isize];
    rainbow_shared_state(o: i32) -> Ram8000_Default = [0x8000, o as isize];
}
pub unsafe fn calculate_rainbow_window(game: &mut Game) {
    let E = rainbow_enemy(0, game);
    let E1 = rainbow_enemy(0x40, game);
    let mut r18: u16 = (byte!((*E).var_33; 1) >> 1) as u16;
    (*E).var_3A = ((*E).var_31).wrapping_sub(r18);
    (*E).var_3B = (r18).wrapping_add((*E).var_31);
    let v2: u16 = (*((&raw mut (*E1).base.enemy_ptr as *mut u8).offset(1) as *mut u16) as i32
        + 3584
        & 0xff00) as u16;
    (*E).var_3C = v2;
    (*E).var_3E = v2;
    let v3: u16 = ((*E1).base.y_pos).wrapping_add(5);
    (*E).var_3D = v3;
    (*E).var_3F = v3;
    r18 = (4 * ((*E).var_3A as i32 & 0xc0)) as u16;
    let i: i32 = (r18 as i32 | (*E).var_3B as i32 & 0xc0) >> 6;
    match i {
        0 | 12 | 15 => rasterize_rainbow_down(game),
        1 => rasterize_rainbow_right(game),
        5 | 6 | 10 => rasterize_rainbow_up(game),
        11 => (),
        _ => panic!("non-null function pointer"),
    }
}
unsafe fn rasterize_rainbow_right(game: &mut Game) {
    let E = rainbow_enemy(0, game);
    rasterize_rainbow_right_edges((*E).var_3C, (*E).var_3E, game);
    write_rainbow_fixed_header(0x9dec, game);
}
unsafe fn write_rainbow_fixed_header(tail: u16, game: &mut Game) {
    *(ram_ptr!(game, u16; 0x9c00)) = 16;
    *(ram_ptr!(game, u16; 0x9c00, 1)) = -25600i32 as u16;
    *(ram_ptr!(game, u16; 0x9c00, 3)) = 16;
    *(ram_ptr!(game, u16; 0x9c00, 4)) = -25597i32 as u16;
    *(ram_ptr!(game, u16; 0x9c00, 6)) = 240;
    *(ram_ptr!(game, u16; 0x9c00, 7)) = -25340i32 as u16;
    *(ram_ptr!(game, u16; 0x9c00, 9)) = 244;
    *(ram_ptr!(game, u16; 0x9c00, 10)) = tail;
    *(ram_ptr!(game, u16; 0x9c00, 12)) = 0;
}
unsafe fn rasterize_rainbow_right_edges(mut r22: u16, mut r24: u16, game: &mut Game) {
    word! { game, 255; hdma_table_2[0], hdma_table_2[1] }
    let E = rainbow_enemy(0, game);
    let r18: u16 =
        crate::rtl::rom_table_word!(game, 0x91c9d4, *(&raw mut (*E).var_3A as *mut u8) as isize);
    let r20: u16 =
        crate::rtl::rom_table_word!(game, 0x91c9d4, *(&raw mut (*E).var_3B as *mut u8) as isize);
    let mut mbn_var_3F: u16 = (*E).var_3F;
    let mut v2 = (ram_ptr!(game, u16; 0x9d00))
        .offset((mbn_var_3F as i32 - 32i32) as isize)
        .offset(1);
    *v2.offset(1) = 255;
    *v2.offset(2) = 255;
    let v8 = v2;
    do_until! { (mbn_var_3F == 32);
        let mut v4: u16 = (r20).wrapping_add(r24);
        if u16::overflowing_add(r20, r24).1 {
            break;
        }
        r24 = (r24).wrapping_add(r20);
        crate::update_place! { previous fresh11 = v2 => v2.offset(-1); }
        *fresh11 = ((byte!(v4; 1)) as u16) | (0xff00u16);
        mbn_var_3F = mbn_var_3F.wrapping_sub(1);
    }
    do_until! { (mbn_var_3F == 32);
        crate::update_place! { previous fresh12 = v2 => v2.offset(-1); }
        *fresh12 = 255;
        mbn_var_3F = mbn_var_3F.wrapping_sub(1);
    }
    let mut v5 = v8.offset(1);
    let mut var_3F: u16 = (*rainbow_shared_state(0i32, game)).var_3F;
    loop {
        let mut v7: u16 = (r18).wrapping_add(r22);
        if u16::overflowing_add(r18, r22).1 {
            break;
        }
        r22 = (r22).wrapping_add(r18);
        crate::update_place! { previous fresh13 = v5 => v5.offset(1); }
        *fresh13 = ((byte!(v7; 1)) as u16) | (0xff00u16);
        var_3F = var_3F.wrapping_add(1);
        if var_3F == 232 {
            return;
        }
    }
    do_until! { (var_3F == 232);
        crate::update_place! { previous fresh14 = v5 => v5.offset(1); }
        *fresh14 = 255;
        var_3F = var_3F.wrapping_add(1);
    }
}
unsafe fn rasterize_rainbow_up(game: &mut Game) {
    let E = rainbow_enemy(0, game);
    let direction = match ((*E).var_3A >> 6 & 2) | ((*E).var_3B >> 7 & 1) {
        0 => RainbowDirection::UpRight,
        1 => RainbowDirection::Up,
        3 => RainbowDirection::UpLeft,
        _ => panic!("non-null function pointer"),
    };
    rasterize_rainbow_vertical(direction, (*E).var_3C, (*E).var_3E, game);
    *(ram_ptr!(game, u16; 0x9c00)) = 16;
    *(ram_ptr!(game, u16; 0x9c00, 1)) = -25344i32 as u16;
    *(ram_ptr!(game, u16; 0x9c00, 3)) = 16;
    *(ram_ptr!(game, u16; 0x9c00, 4)) = -25342i32 as u16;
    let v2: u16 = ((*E).var_3D).wrapping_sub(32);
    printf(b"Carry crap!\n\0" as *const u8 as *const c_char);
    if v2 >= 0x80 {
        *(ram_ptr!(game, u16; 0x9c00, 6)) = (v2 as i32 - 127i32 | 0x80) as u16;
        *(ram_ptr!(game, u16; 0x9c00, 7)) = -25340i32 as u16;
        *(ram_ptr!(game, u16; 0x9c00, 9)) = 255;
        *(ram_ptr!(game, u16; 0x9c00, 10)) = (2 * (v2 as i32 - 127i32) - 25340i32) as u16;
        *(ram_ptr!(game, u16; 0x9c00, 12)) = 127;
        *(ram_ptr!(game, u16; 0x9c00, 13)) = (2 * (v2 as i32 - 127i32) - 25340i32 + 254) as u16;
        *(ram_ptr!(game, u16; 0x9c00, 15)) = 0;
    } else {
        *(ram_ptr!(game, u16; 0x9c00, 6)) = (v2) | (0x80u16);
        *(ram_ptr!(game, u16; 0x9c00, 7)) = -25340i32 as u16;
        *(ram_ptr!(game, u16; 0x9c00, 9)) = 127;
        *(ram_ptr!(game, u16; 0x9c00, 10)) = (2 * v2 as i32 - 25340i32) as u16;
        *(ram_ptr!(game, u16; 0x9c00, 12)) = 0;
    };
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

unsafe fn rasterize_rainbow_vertical(
    direction: RainbowDirection,
    mut left: u16,
    mut right: u16,
    game: &mut Game,
) {
    use RainbowDirection::*;
    let down = matches!(direction, DownRight | Down | DownLeft);
    let add_left = matches!(direction, UpRight | DownRight);
    let add_right = matches!(direction, UpRight | Up | DownRight | Down);
    let zero_sentinel = matches!(direction, DownLeft);
    word! { game, 255; hdma_table_2[0], hdma_table_2[1] }
    let enemy = rainbow_enemy(0, game);
    let left_step = crate::rtl::rom_table_word!(game, 0x91c9d4, {
        let angle = if down {
            *(&raw mut (*enemy).var_3A as *mut u8)
        } else {
            *(&raw mut (*enemy).var_3B as *mut u8)
        };
        (if add_left {
            angle
        } else {
            (-(angle as i32)) as u8
        }) as isize
    });
    let right_step = crate::rtl::rom_table_word!(game, 0x91c9d4, {
        let angle = if down {
            *(&raw mut (*enemy).var_3B as *mut u8)
        } else {
            *(&raw mut (*enemy).var_3A as *mut u8)
        };
        (if add_right {
            angle
        } else {
            (-(angle as i32)) as u8
        }) as isize
    });
    let mut row = (*enemy).var_3D;
    let mut dst = if down {
        let mut remaining = row.wrapping_sub(32);
        let mut dst = ram_ptr!(game, u16; 0x9d00).offset(2);
        do_until! { remaining == 0;
            crate::update_place! { previous current = dst => dst.offset(1); }
            *current = 255;
            remaining = remaining.wrapping_sub(1);
        }
        // Preserve the original post-fill read of the starting row.
        row = (*rainbow_enemy(0, game)).var_3D;
        dst
    } else {
        let dst = ram_ptr!(game, u16; 0x9d00)
            .offset((row as i32 - 32) as isize)
            .offset(1);
        *dst.offset(1) = 255;
        *dst.offset(2) = 255;
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
        crate::update_place! { previous current = dst => dst.offset(if down { 1 } else { -1 }); }
        *current = packed;
        row = if down {
            row.wrapping_add(1)
        } else {
            row.wrapping_sub(1)
        };
    }
}
unsafe fn rasterize_rainbow_down(game: &mut Game) {
    let E = rainbow_enemy(0, game);
    let direction = match ((*E).var_3A >> 6 & 2) | ((*E).var_3B >> 7 & 1) {
        0 => RainbowDirection::DownRight,
        2 => RainbowDirection::Down,
        3 => RainbowDirection::DownLeft,
        _ => panic!("non-null function pointer"),
    };
    rasterize_rainbow_vertical(direction, (*E).var_3C, (*E).var_3E, game);
    write_rainbow_fixed_header(0x9de6, game);
}
