//! Terrain programs and clipped emission over the composed owned room state.
use super::host::{
    ByteOrder, Diagnostic, DrawCursor, DrawLayout, DrawProgramId, DrawTarget, Host, SlotMember,
    TileDefinitionCorner, TilemapDestination,
};

fn draw_program(h: &impl Host, handle: u16) -> u16 {
    let slot = usize::from(handle >> 1);
    if let Some(item) = h.session().mechanisms.slots.get(slot) {
        item.draw_program
    } else {
        h.mechanism_word(SlotMember::DrawProgram, slot as isize)
    }
}
fn block_origin(h: &impl Host, handle: u16) -> u16 {
    let slot = usize::from(handle >> 1);
    if let Some(item) = h.session().mechanisms.slots.get(slot) {
        item.block
    } else {
        h.mechanism_word(SlotMember::Block, slot as isize)
    }
}
pub fn process_draw(h: &mut impl Host, handle: u16) {
    h.probe("mechanisms.terrain");
    let id = DrawProgramId(draw_program(h, handle));
    let program = h.capture_terrain_program(id);
    let origin = block_origin(h, handle);
    let mut destination = origin;
    let mut cursor = 0usize;
    loop {
        let command = h.terrain_program_word(&program, cursor);
        cursor += 2;
        let stride = if command & 0x8000 != 0 {
            usize::from(h.session().room.width) * 2
        } else {
            2
        };
        for block in 0..usize::from(command as u8) {
            let value = h.terrain_program_word(&program, cursor);
            cursor += 2;
            let byte_block = (usize::from(destination) + block * stride) & !1;
            if let Some(cell) = h
                .session_mut()
                .room
                .terrain
                .cells_mut()
                .get_mut(byte_block >> 1)
            {
                cell.level = value;
            } else {
                h.set_terrain_word(byte_block, value, ByteOrder::Little);
            }
        }
        let displacement = h.terrain_program_word(&program, cursor);
        if displacement == 0 {
            break;
        }
        let x = i32::from(displacement as u8 as i8);
        let y = i32::from((displacement >> 8) as u8 as i8);
        let width = i32::from(h.session().room.width);
        destination = (i32::from(origin) + 2 * (x + y * width)) as u16;
        cursor += 2;
    }
}
fn tilemap_destination(h: &impl Host, x: u16, y: u16) -> u16 {
    let column = x & 31;
    let right = column >= 16;
    let mut destination = ((y & 15) * 64)
        .wrapping_add(if right { 0x53e0 } else { 0x5000 })
        .wrapping_add(2 * column);
    if h.inputs().background_page_x & 0x100 != 0 {
        destination = if right {
            destination.wrapping_sub(1024)
        } else {
            destination.wrapping_add(1024)
        };
    }
    destination
}
fn emit_block(
    h: &mut impl Host,
    block: u16,
    first: &DrawTarget,
    second: &DrawTarget,
    offset: u16,
    vertical: bool,
) {
    let next = offset.wrapping_add(2);
    let corners = if vertical { [0, 1, 2, 3] } else { [0, 2, 1, 3] };
    for ((target, offset), corner) in [
        (first, offset),
        (second, offset),
        (first, next),
        (second, next),
    ]
    .into_iter()
    .zip(corners)
    {
        h.write_draw_tile(target, offset, TileDefinitionCorner { block, corner });
    }
}
pub fn draw(h: &mut impl Host, k: u16) {
    h.probe("mechanisms.draw");
    let mut command_address = draw_program(h, k);
    let mut x = h.session().mechanisms.block_x;
    let mut y = h.session().mechanisms.block_y;
    loop {
        let mut top = h.inputs().camera_y >> 4;
        if ((h.inputs().camera_y as i32 >> 4) + 15 - y as i32) & 0x8000 != 0 {
            return;
        }
        let command = h.drawing_word(DrawCursor(command_address));
        let vertical = command & 0x8000 != 0;
        let mut count = command & 0x7fff;
        let mut skip = 0u16;
        let mut left = x;
        let mut trailing;

        let (first, second) = if vertical {
            if !(h.inputs().camera_x as i32 >> 4 == x as i32
                || (((h.inputs().camera_x as i32 >> 4) - x as i32) as i16) < 0)
            {
                return;
            }
            let right = ((h.inputs().camera_x as i32 >> 4) + 17) as i16;
            if !(right as i32 != x as i32 && right.wrapping_sub(x as i16) >= 0) {
                return;
            }
            trailing = top.wrapping_add(16);
            if (top as i16).wrapping_sub(y as i16) < 0 {
                top = y;
            } else {
                skip = top.wrapping_sub(y);
                if (y as i32 + count as i32 - top as i32) & 0x8000 != 0 {
                    return;
                }
                let equal = count == skip;
                let negative = (count as i16).wrapping_sub(skip as i16) < 0;
                count = count.wrapping_sub(skip);
                if negative {
                    h.diagnostic(Diagnostic::Unreachable);
                }
                if equal {
                    return;
                }
            }
            trailing = (count as i32 + top as i32 - trailing as i32) as u16;
            if trailing & 0x8000 == 0 {
                let negative = (count as i16).wrapping_sub(trailing as i16) < 0;
                count = count.wrapping_sub(trailing);
                if count == 0 || negative {
                    return;
                }
            }
            let destination = tilemap_destination(h, left, top);
            let Some(targets) = h.queue_draw_transfers(
                TilemapDestination(destination),
                count,
                DrawLayout::VerticalPair,
            ) else {
                return;
            };
            (targets.first, targets.second)
        } else {
            if (y as i32 - top as i32) & 0x8000 != 0 {
                return;
            }
            top = y;
            trailing = ((h.inputs().camera_x.wrapping_add(15) as i32 >> 4) - 1) as u16;
            if (trailing as i16).wrapping_sub(x as i16) >= 0 && trailing != x {
                skip = trailing.wrapping_sub(x);
                if x as i32 + count as i32 == trailing as i32
                    || ((x as i32 + count as i32 - trailing as i32) as i16) < 0
                {
                    return;
                }
                count = count.wrapping_sub(skip);
                left = trailing;
            }
            trailing = trailing.wrapping_add(17);
            if (trailing as i32 - x as i32) & 0x8000 != 0 {
                return;
            }
            trailing = (count as i32 + left as i32 - 1 - trailing as i32) as u16;
            if trailing & 0x8000 == 0 {
                count = count.wrapping_sub(trailing);
                if count == 0 {
                    return;
                }
            }
            let destination = tilemap_destination(h, left, top);
            let layout = if (2 * count as i32 + (destination as i32 & 31) - 1) & 0xffe0 != 0 {
                DrawLayout::HorizontalSplit
            } else {
                DrawLayout::HorizontalPair
            };
            let Some(targets) =
                h.queue_draw_transfers(TilemapDestination(destination), count, layout)
            else {
                return;
            };
            (targets.first, targets.second)
        };
        let mut cursor = skip
            .wrapping_mul(2)
            .wrapping_add(command_address)
            .wrapping_add(2);
        let mut offset = 0u16;
        // The original do-until emits once even for a zero count.
        loop {
            let block = h.drawing_word(DrawCursor(cursor));
            emit_block(h, block, &first, &second, offset, vertical);
            offset = offset.wrapping_add(4);
            cursor = cursor.wrapping_add(2);
            let index = h.session().mechanisms.draw_cursor.wrapping_add(8);
            h.session_mut().mechanisms.draw_cursor = index;
            if (h.session().mechanisms.draw_cursor as i32 - 512) & 0x8000 == 0 {
                return;
            }
            count = count.wrapping_sub(1);
            if count == 0 {
                break;
            }
        }
        if trailing & 0x8000 == 0 {
            cursor = cursor.wrapping_add(trailing.wrapping_mul(2));
        }
        let displacement = h.drawing_word(DrawCursor(cursor));
        if displacement == 0 {
            return;
        }
        x = h
            .session()
            .mechanisms
            .block_x
            .wrapping_add(displacement as i8 as u16);
        let next = cursor.wrapping_add(1);
        // Native/RAM selection and the byte read happen again for Y.
        let base_y = h.session().mechanisms.block_y;
        y = base_y.wrapping_add(h.drawing_byte(DrawCursor(next)) as i8 as u16);
        command_address = next.wrapping_add(1);
    }
}
