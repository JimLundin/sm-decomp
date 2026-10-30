//! Ordered packed transfers and live buffer identities at the backend boundary.
#![deny(unsafe_op_in_unsafe_fn)]
use super::host::{
    DrawLayout, DrawTarget, DrawTargets, Graphics, GraphicsTransferKind, InstructionRecord,
    TileDefinitionCorner, TilemapDestination,
};
use crate::game::session_runtime::Runtime;
use crate::game::xray_background::reveal_room_block;

#[forbid(unsafe_code)]
impl Runtime<'_> {
    fn queue_mechanism_pair(&mut self, destination: u16, tail: u16, count: u16, mode: u16) {
        let entry = 0xd0 + usize::from(tail);
        self.write_slice_native(entry + 5, mode | destination);
        let size = 4u16.wrapping_mul(count);
        let source = self.session().mechanisms.draw_cursor.wrapping_add(0xc6c8);
        self.write_slice_native(entry, size);
        self.write_slice_native(entry + 7, size);
        self.write_slice_native(entry + 2, source);
        self.set_byte(entry + 4, 126);
        self.write_slice_native(entry + 9, source.wrapping_add(size));
        self.set_byte(entry + 11, 126);
    }
}
#[forbid(unsafe_code)]
impl Graphics for Runtime<'_> {
    fn queue_graphics_transfer<R: InstructionRecord>(
        &mut self,
        record: &R,
        kind: GraphicsTransferKind,
    ) {
        let tail = self.word(0x330);
        let entry = 0xd0 + usize::from(self.word(0x330));
        match kind {
            GraphicsTransferKind::Item { destination } => {
                self.write_slice_le(entry, 256);
                self.write_slice_le(entry + 2, record.word(0));
                self.set_byte(entry + 4, 0x89);
                self.write_slice_le(entry + 5, destination.0);
            }
            GraphicsTransferKind::CopyScript => {
                self.write_slice_le(entry, record.word(0));
                self.write_slice_le(entry + 2, record.word(2));
                self.set_byte(entry + 4, record.byte(4));
                self.write_slice_le(entry + 5, record.word(5));
            }
        }
        self.write_slice_le(0x330, tail.wrapping_add(7));
    }
    fn queue_draw_transfers(
        &mut self,
        destination: TilemapDestination,
        count: u16,
        layout: DrawLayout,
    ) -> Option<DrawTargets> {
        let tail = self.word(0x330);
        let limit = if matches!(layout, DrawLayout::VerticalPair) {
            240
        } else {
            480
        };
        if !((self.word(0x330) as i16).wrapping_sub(limit) < 0
            && (((512u16.wrapping_sub(self.session().mechanisms.draw_cursor) as i32 >> 3)
                - count as i32)
                & 0x8000)
                == 0)
        {
            return None;
        }
        let destination = destination.0;
        let entry = 0xd0 + usize::from(tail);
        let first;
        let second;
        match layout {
            DrawLayout::VerticalPair => {
                let live_tail = self.word(0x330);
                self.queue_mechanism_pair(destination, live_tail, count, 0x8000);
                let next = self.native_word(entry + 5).wrapping_add(1);
                self.write_slice_native(entry + 12, next);
                first = usize::from(self.native_word(entry + 2));
                second = usize::from(self.native_word(entry + 9));
                self.write_slice_le(0x330, tail.wrapping_add(14));
            }
            DrawLayout::HorizontalPair => {
                self.queue_mechanism_pair(destination, tail, count, 0);
                let next = self.native_word(entry + 5).wrapping_add(32);
                self.write_slice_native(entry + 12, next);
                first = usize::from(self.native_word(entry + 2));
                second = usize::from(self.native_word(entry + 9));
                self.write_slice_le(0x330, tail.wrapping_add(14));
            }
            DrawLayout::HorizontalSplit => {
                let column = destination & 31;
                if (tail as i16).wrapping_sub(228) >= 0 || 32i16.wrapping_sub(column as i16) < 0 {
                    return None;
                }
                let size = (2 * (32 - column as i32)) as u16;
                self.write_slice_native(entry, size);
                self.write_slice_native(entry + 14, size);
                self.write_slice_native(entry + 5, destination);
                self.write_slice_native(entry + 12, (destination & 0xffe0) ^ 0x400);
                let next = self.native_word(entry + 12).wrapping_add(32);
                self.write_slice_native(entry + 26, next);
                let next = self.native_word(entry + 5).wrapping_add(32);
                self.write_slice_native(entry + 19, next);
                let remaining = (4 * count as i32 - self.native_word(entry) as i32) as u16;
                self.write_slice_native(entry + 7, remaining);
                self.write_slice_native(entry + 21, remaining);
                let source = self.session().mechanisms.draw_cursor.wrapping_sub(14648);
                let live_source = self.session().mechanisms.draw_cursor.wrapping_sub(14648);
                self.write_slice_native(entry + 2, live_source);
                first = usize::from(source);
                let next = self.native_word(entry).wrapping_add(source);
                self.write_slice_native(entry + 9, next);
                let next = self.native_word(entry + 7).wrapping_add(next);
                self.write_slice_native(entry + 16, next);
                second = usize::from(next);
                let next = self.native_word(entry + 14).wrapping_add(next);
                self.write_slice_native(entry + 23, next);
                for index in 0..4 {
                    self.set_byte(entry + index * 7 + 4, 126);
                }
                self.write_slice_le(0x330, tail.wrapping_add(28));
            }
        }
        Some(DrawTargets {
            first: DrawTarget(first),
            second: DrawTarget(second),
        })
    }
    fn write_draw_tile(&mut self, target: &DrawTarget, offset: u16, source: TileDefinitionCorner) {
        let flip = (source.block >> 10) & 3;
        let index = usize::from((source.block & 0x3ff) * 4 + (source.corner ^ flip));
        let tile = if let Some(definition) = self.session().room.terrain.tiles().get(index / 4) {
            definition[index % 4]
        } else {
            self.word(0xa000 + index * 2)
        };
        let tile = tile ^ ((source.block & 0xc00) << 4);
        // Identity resolves against THIS owner, including any current replacement
        // after a foreign callback. No allocation or buffer snapshot is retained.
        let destination = target.0 + usize::from(offset >> 1) * 2;
        self.write_slice_le(destination, tile);
    }
    fn set_item_tile_definition(&mut self, byte_offset: usize, value: u16) {
        if byte_offset & 1 == 0 {
            if let Some(definition) = self
                .session_mut()
                .room
                .terrain
                .tiles_mut()
                .get_mut(byte_offset / 8)
            {
                definition[(byte_offset / 2) % 4] = value;
                return;
            }
        }
        self.write_slice_le(0xa000 + byte_offset, value);
    }
    fn reveal_block(&mut self, block: u16, x: u16, y: u16) {
        reveal_room_block(block, x, y, self);
    }
}
// Transfer descriptors store bank7E and a byte identity, exactly as the existing
// NMI consumer expects. Runtime publication writes the current tile bytes to
// compatibility memory; NMI resolves those bytes when consuming each descriptor.
// Neither this queue nor DrawTarget retains a pointer into Session storage.
