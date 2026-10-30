//! The Torizo corpse's bank $A9 tables through the imported `sm_data` records.
//!
//! Each accessor serves the record while the loaded image is the imported
//! allocation; any other image, and indices outside a table, keep the original
//! indexed loads with their failures.
#![forbid(unsafe_code)]
use crate::imported::{Imported, load, rom, row};
use crate::records::Records;
use sm_data::cartridge::RomImage;
use sm_data::torizo_corpse::*;

#[derive(Default)]
pub struct ImportedTorizoCorpse {
    even_transfers: Option<Imported<TileTransfers>>,
    odd_transfers: Option<Imported<TileTransfers>>,
    sand_heap_lines: Option<Imported<SandHeapLines>>,
}

pub fn import(image: &RomImage) -> ImportedTorizoCorpse {
    ImportedTorizoCorpse {
        even_transfers: Imported::import(
            image,
            EVEN_TRANSFER_ADDRESS,
            TRANSFER_BYTES,
            import_tile_transfers,
        ),
        odd_transfers: Imported::import(
            image,
            ODD_TRANSFER_ADDRESS,
            TRANSFER_BYTES,
            import_tile_transfers,
        ),
        sand_heap_lines: Imported::import(
            image,
            SAND_HEAP_LINE_ADDRESS,
            SAND_HEAP_LINE_BYTES,
            import_sand_heap_lines,
        ),
    }
}

/// Which frame's rotting transfers to queue.
#[derive(Clone, Copy)]
pub enum Frame {
    Even,
    Odd,
}

fn transfers(
    game: &(impl Records + ?Sized),
    frame: Frame,
) -> (&Option<Imported<TileTransfers>>, u32) {
    match frame {
        Frame::Even => (
            &game.records().torizo_corpse_tables.even_transfers,
            EVEN_TRANSFER_ADDRESS,
        ),
        Frame::Odd => (
            &game.records().torizo_corpse_tables.odd_transfers,
            ODD_TRANSFER_ADDRESS,
        ),
    }
}

/// Rotting transfer `index` for `frame`.
pub fn transfer(game: &(impl Records + ?Sized), frame: Frame, index: isize) -> TileTransfer {
    let rom = rom(game);
    let (table, address) = transfers(game, frame);
    row(table, rom, index).unwrap_or_else(|| TileTransfer {
        size: load(rom, address, 4 * index),
        source_bank: load(rom, address, 4 * index + 1),
        source_address: load(rom, address, 4 * index + 2),
        vram_address: load(rom, address, 4 * index + 3),
    })
}

/// The size of rotting transfer `index` for `frame`; zero past the last.
pub fn transfer_size(game: &(impl Records + ?Sized), frame: Frame, index: isize) -> u16 {
    let rom = rom(game);
    let (table, address) = transfers(game, frame);
    row(table, rom, index).map_or_else(|| load(rom, address, 4 * index), |row| row.size)
}

/// Sand heap line `index` at $A9:D67C/$A9:D69C.
pub fn sand_heap_line(game: &(impl Records + ?Sized), index: isize) -> SandHeapLine {
    let rom = rom(game);
    row(
        &game.records().torizo_corpse_tables.sand_heap_lines,
        rom,
        index,
    )
    .unwrap_or_else(|| SandHeapLine {
        destination: load(rom, SAND_HEAP_LINE_ADDRESS, index),
        source: load(rom, SAND_HEAP_SOURCE_ADDRESS, index),
    })
}
