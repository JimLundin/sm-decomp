//! Item graphics and reveal algorithms over shared owned room state.
#![forbid(unsafe_code)]
use super::{
    content::Script,
    host::{
        ContentAddress, ContentTable, GraphicsTransferKind, Host, InstructionRecord, SlotMember,
        StoreOrder, TilemapDestination,
    },
};

pub fn load_graphics<H: Host>(
    h: &mut H,
    mut p: Script<H::Record>,
    handle: u16,
) -> Script<H::Record> {
    let graphic = h.session().mechanisms.next_graphics;
    let current = h.session().mechanisms.next_graphics;
    let slot = usize::from(handle >> 1);
    if let Some(item) = h.session_mut().mechanisms.slots.get_mut(slot) {
        item.work1 = current;
    } else {
        h.set_mechanism_word(
            SlotMember::Work1,
            slot as isize,
            current,
            StoreOrder::SliceLittle,
        );
    }
    h.session_mut().mechanisms.next_graphics = graphic.wrapping_add(2) & 6;
    let index = usize::from(graphic >> 1);
    let destination = h.table_word(ContentTable::ItemGraphicDestination, index as isize);
    let tile_offset = usize::from(h.table_word(ContentTable::ItemTileOffset, index as isize));
    let mut tile_number = h.table_word(ContentTable::ItemTileNumber, index as isize);
    let instruction = p.position();
    if let Some(saved) = h.session_mut().mechanisms.saved_graphics.get_mut(index) {
        *saved = instruction;
    } else {
        h.set_saved_graphics_word(index, instruction, StoreOrder::SliceLittle);
    }
    h.queue_graphics_transfer(
        &p,
        GraphicsTransferKind::Item {
            destination: TilemapDestination(destination),
        },
    );
    p = p.advance(2);
    for tile in 0..8 {
        let attributes = u16::from(p.byte(0)) << 10;
        let value = tile_number.wrapping_add(attributes);
        h.set_item_tile_definition(tile_offset + tile * 2, value);
        tile_number = tile_number.wrapping_add(1);
        p = p.advance(1);
    }
    p
}
pub fn copy_vram<H: Host>(h: &mut H, p: Script<H::Record>) -> Script<H::Record> {
    h.queue_graphics_transfer(&p, GraphicsTransferKind::CopyScript);
    p.advance(7)
}
pub fn reload_graphics(h: &mut impl Host) {
    h.session_mut().mechanisms.next_graphics = 0;
    for index in 0..4 {
        if h.session().mechanisms.saved_graphics[index] != 0 {
            let address = h.session().mechanisms.saved_graphics[index];
            let record = h.loaded_record(ContentAddress {
                bank: 0x84,
                offset: address,
            });
            load_graphics(h, Script::new(record, address | 0x8000), (index * 2) as u16);
        }
    }
}
pub fn reveal_items(h: &mut impl Host) {
    for slot in (0..40).rev() {
        if h.session().mechanisms.slots[slot].header < 0xdf89 {
            continue;
        }
        let argument = h.session().mechanisms.slots[slot].room_argument;
        if h.item_collected(argument) {
            continue;
        }
        super::creation::coordinates(h, slot);
        let graphic = h.session().mechanisms.slots[slot].work1 >> 1;
        let address = h.table_word(ContentTable::ItemReveal, graphic as isize);
        let sequence = h.native_record(ContentAddress {
            bank: 0x84,
            offset: address,
        });
        let block = sequence.word(2) & 0xfff;
        let x = h.session().mechanisms.block_x;
        let y = h.session().mechanisms.block_y;
        h.reveal_block(block, x, y);
    }
}
