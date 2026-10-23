//! Room terrain updates and the shared tile/VRAM emission path.
//! Queue fields retain native order; terrain and tile tables retain little endian.
use super::native_content::validate_ram_draw_address;
use super::{port::Execution, state::Column};
use crate::game::enemy_script::bank_base;

const DRAW_INDEX: usize = 0x1c25;
const QUEUE_TAIL: usize = 0x330;
const BLOCK_X: usize = 0x1c29;
const BLOCK_Y: usize = 0x1c2b;
const CAMERA_X: usize = 0x911;
const CAMERA_Y: usize = 0x915;

impl Execution<'_> {
    pub(super) fn process_draw(&mut self, k: u16) {
        crate::verify_probe::hit("mechanisms.terrain");
        let address = self.port().field(Column::DrawProgram, usize::from(k >> 1));
        let image = if address & 0x8000 != 0 {
            Some(self.port().rom())
        } else {
            if address >= 0x2000 {
                self.foreign(|_| validate_ram_draw_address(address));
            }
            None
        };
        let mut cursor = usize::from(address)
            + if image.is_some() {
                bank_base(0x84) as usize
            } else {
                0
            };
        let read = |position: usize, execution: &Self| {
            u16::from_ne_bytes(core::array::from_fn(|i| match &image {
                Some(image) => image.read(position + i),
                None => execution.port().byte(position + i),
            }))
        };
        let origin = self.port().field(Column::Block, usize::from(k >> 1));
        let mut destination = origin;
        loop {
            let command = read(cursor, self);
            cursor += 2;
            let stride = if command & 0x8000 != 0 {
                usize::from(self.port().read_le(0x7a5)) * 2
            } else {
                2
            };
            for block in 0..usize::from(command as u8) {
                let value = read(cursor, self);
                cursor += 2;
                self.port_mut().write_slice_le(
                    0x10002 + ((usize::from(destination) + block * stride) >> 1) * 2,
                    value,
                );
            }
            let displacement = read(cursor, self);
            if displacement == 0 {
                break;
            }
            let x = i32::from(displacement as u8 as i8);
            let y = i32::from((displacement >> 8) as u8 as i8);
            let width = i32::from(self.port().read_le(0x7a5));
            destination = (i32::from(origin) + 2 * (x + y * width)) as u16;
            cursor += 2;
        }
    }

    fn tilemap_destination(&self, x: u16, y: u16) -> u16 {
        let column = x & 31;
        let right = column >= 16;
        let mut destination = ((y & 15) * 64)
            .wrapping_add(if right { 0x53e0 } else { 0x5000 })
            .wrapping_add(2 * column);
        if self.port().read_le(0x91d) & 0x100 != 0 {
            destination = if right {
                destination.wrapping_sub(1024)
            } else {
                destination.wrapping_add(1024)
            };
        }
        destination
    }

    fn queue_pair(&mut self, destination: u16, tail: u16, count: u16, mode: u16) {
        let entry = 0xd0 + usize::from(tail);
        self.port_mut()
            .write_slice_native(entry + 5, mode | destination);
        let size = 4u16.wrapping_mul(count);
        let source = self.port().read_le(DRAW_INDEX).wrapping_add(0xc6c8);
        self.port_mut().write_slice_native(entry, size);
        self.port_mut().write_slice_native(entry + 7, size);
        self.port_mut().write_slice_native(entry + 2, source);
        self.port_mut().set_byte(entry + 4, 126);
        self.port_mut()
            .write_slice_native(entry + 9, source.wrapping_add(size));
        self.port_mut().set_byte(entry + 11, 126);
    }

    /// Each tile is read immediately before writing: destinations can alias later sources.
    fn emit_block(&mut self, block: u16, first: usize, second: usize, offset: u16, vertical: bool) {
        let next = offset.wrapping_add(2);
        let destinations = [
            first + usize::from(offset >> 1) * 2,
            second + usize::from(offset >> 1) * 2,
            first + usize::from(next >> 1) * 2,
            second + usize::from(next >> 1) * 2,
        ];
        let corners = if vertical { [0, 1, 2, 3] } else { [0, 2, 1, 3] };
        let flip = (block >> 10) & 3;
        let first_tile = (block & 0x3ff) * 4;
        for (corner, destination) in corners.into_iter().zip(destinations) {
            let tile = self
                .port()
                .read_le(0xa000 + usize::from(first_tile + (corner ^ flip)) * 2)
                ^ ((block & 0xc00) << 4);
            self.port_mut().write_slice_le(destination, tile);
        }
    }

    /// Native ROM streams retain their valid allocation/index preconditions.
    pub(super) unsafe fn draw(&mut self, k: u16) {
        crate::verify_probe::hit("mechanisms.draw");
        unsafe {
            let mut command_address = self.port().field(Column::DrawProgram, usize::from(k >> 1));
            let mut x = self.port().read_le(BLOCK_X);
            let mut y = self.port().read_le(BLOCK_Y);
            loop {
                let mut top = self.port().read_le(CAMERA_Y) >> 4;
                if ((self.port().read_le(CAMERA_Y) as i32 >> 4) + 15 - y as i32) & 0x8000 != 0 {
                    return;
                }
                let command = self.draw_word(command_address);
                let vertical = command & 0x8000 != 0;
                let mut count = command & 0x7fff;
                let mut skip = 0u16;
                let mut left = x;
                let mut trailing;
                let first;
                let second;
                if vertical {
                    if !(self.port().read_le(CAMERA_X) as i32 >> 4 == x as i32
                        || (((self.port().read_le(CAMERA_X) as i32 >> 4) - x as i32) as i16) < 0)
                    {
                        return;
                    }
                    let right = ((self.port().read_le(CAMERA_X) as i32 >> 4) + 17) as i16;
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
                            self.foreign(|game| crate::rtl::Unreachable(game));
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
                    let tail = self.port().read_le(QUEUE_TAIL);
                    if !((self.port().read_le(QUEUE_TAIL) as i16).wrapping_sub(240) < 0
                        && (((512u16.wrapping_sub(self.port().read_le(DRAW_INDEX)) as i32 >> 3)
                            - count as i32)
                            & 0x8000)
                            == 0)
                    {
                        return;
                    }
                    let destination = self.tilemap_destination(left, top);
                    self.queue_pair(destination, self.port().read_le(QUEUE_TAIL), count, 0x8000);
                    let entry = 0xd0 + usize::from(tail);
                    let destination = self.port().read_native(entry + 5).wrapping_add(1);
                    self.port_mut().write_slice_native(entry + 12, destination);
                    first = usize::from(self.port().read_native(entry + 2));
                    second = usize::from(self.port().read_native(entry + 9));
                    self.port_mut()
                        .write_slice_le(QUEUE_TAIL, tail.wrapping_add(14));
                } else {
                    if (y as i32 - top as i32) & 0x8000 != 0 {
                        return;
                    }
                    top = y;
                    trailing =
                        ((self.port().read_le(CAMERA_X).wrapping_add(15) as i32 >> 4) - 1) as u16;
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
                    let tail = self.port().read_le(QUEUE_TAIL);
                    if !((self.port().read_le(QUEUE_TAIL) as i16).wrapping_sub(480) < 0
                        && (((512u16.wrapping_sub(self.port().read_le(DRAW_INDEX)) as i32 >> 3)
                            - count as i32)
                            & 0x8000)
                            == 0)
                    {
                        return;
                    }
                    let destination = self.tilemap_destination(left, top);
                    let column = destination & 31;
                    let entry = 0xd0 + usize::from(tail);
                    if (2 * count as i32 + (destination as i32 & 31) - 1) & 0xffe0 != 0 {
                        if (tail as i16).wrapping_sub(228) >= 0
                            || 32i16.wrapping_sub(column as i16) < 0
                        {
                            return;
                        }
                        let size = (2 * (32 - column as i32)) as u16;
                        self.port_mut().write_slice_native(entry, size);
                        self.port_mut().write_slice_native(entry + 14, size);
                        self.port_mut().write_slice_native(entry + 5, destination);
                        self.port_mut()
                            .write_slice_native(entry + 12, (destination & 0xffe0) ^ 0x400);
                        let next = self.port().read_native(entry + 12).wrapping_add(32);
                        self.port_mut().write_slice_native(entry + 26, next);
                        let next = self.port().read_native(entry + 5).wrapping_add(32);
                        self.port_mut().write_slice_native(entry + 19, next);
                        let remaining =
                            (4 * count as i32 - self.port().read_native(entry) as i32) as u16;
                        self.port_mut().write_slice_native(entry + 7, remaining);
                        self.port_mut().write_slice_native(entry + 21, remaining);
                        let source = self.port().read_le(DRAW_INDEX).wrapping_sub(14648);
                        let live_source = self.port().read_le(DRAW_INDEX).wrapping_sub(14648);
                        self.port_mut().write_slice_native(entry + 2, live_source);
                        first = usize::from(source);
                        let next = self.port().read_native(entry).wrapping_add(source);
                        self.port_mut().write_slice_native(entry + 9, next);
                        let next = self.port().read_native(entry + 7).wrapping_add(next);
                        self.port_mut().write_slice_native(entry + 16, next);
                        second = usize::from(next);
                        let next = self.port().read_native(entry + 14).wrapping_add(next);
                        self.port_mut().write_slice_native(entry + 23, next);
                        for index in 0..4 {
                            self.port_mut().set_byte(entry + index * 7 + 4, 126);
                        }
                        self.port_mut()
                            .write_slice_le(QUEUE_TAIL, tail.wrapping_add(28));
                    } else {
                        self.queue_pair(destination, tail, count, 0);
                        let next = self.port().read_native(entry + 5).wrapping_add(32);
                        self.port_mut().write_slice_native(entry + 12, next);
                        first = usize::from(self.port().read_native(entry + 2));
                        second = usize::from(self.port().read_native(entry + 9));
                        self.port_mut()
                            .write_slice_le(QUEUE_TAIL, tail.wrapping_add(14));
                    }
                }
                let mut cursor = skip
                    .wrapping_mul(2)
                    .wrapping_add(command_address)
                    .wrapping_add(2);
                let mut offset = 0u16;
                // The original do-until emits once even for a zero count.
                loop {
                    let block = self.draw_word(cursor);
                    self.emit_block(block, first, second, offset, vertical);
                    offset = offset.wrapping_add(4);
                    cursor = cursor.wrapping_add(2);
                    let index = self.port().read_le(DRAW_INDEX).wrapping_add(8);
                    self.port_mut().write_slice_le(DRAW_INDEX, index);
                    if (self.port().read_le(DRAW_INDEX) as i32 - 512) & 0x8000 == 0 {
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
                let displacement = self.draw_word(cursor);
                if displacement == 0 {
                    return;
                }
                x = self
                    .port()
                    .read_le(BLOCK_X)
                    .wrapping_add(displacement as i8 as u16);
                let next = cursor.wrapping_add(1);
                // Native/RAM selection and the byte read happen again for Y.
                let base_y = self.port().read_le(BLOCK_Y);
                y = base_y.wrapping_add(self.draw_byte(next) as i8 as u16);
                command_address = next.wrapping_add(1);
            }
        }
    }
}
