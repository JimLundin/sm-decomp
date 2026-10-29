//! The Torizo corpse's bank $A9 tables through the imported `sm_data` records.
//!
//! Each accessor serves the record while the loaded image is the imported
//! allocation; any other image, and indices outside a table, keep the original
//! indexed loads with their failures.
#![forbid(unsafe_code)]
use crate::game::imported::{Imported, load, rom, row};
use crate::game_state::Game;
use crate::rom::RomImage;
use sm_data::torizo_corpse::*;

#[derive(Default)]
pub(crate) struct ImportedTorizoCorpse {
    even_transfers: Option<Imported<TileTransfers>>,
    odd_transfers: Option<Imported<TileTransfers>>,
    sand_heap_lines: Option<Imported<SandHeapLines>>,
}

pub(crate) fn import(image: &RomImage) -> ImportedTorizoCorpse {
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
pub(crate) enum Frame {
    Even,
    Odd,
}

fn transfers(game: &Game, frame: Frame) -> (&Option<Imported<TileTransfers>>, u32) {
    match frame {
        Frame::Even => (
            &game.torizo_corpse_tables.even_transfers,
            EVEN_TRANSFER_ADDRESS,
        ),
        Frame::Odd => (
            &game.torizo_corpse_tables.odd_transfers,
            ODD_TRANSFER_ADDRESS,
        ),
    }
}

/// Rotting transfer `index` for `frame`.
pub(crate) fn transfer(game: &Game, frame: Frame, index: isize) -> TileTransfer {
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
pub(crate) fn transfer_size(game: &Game, frame: Frame, index: isize) -> u16 {
    let rom = rom(game);
    let (table, address) = transfers(game, frame);
    row(table, rom, index).map_or_else(|| load(rom, address, 4 * index), |row| row.size)
}

/// Sand heap line `index` at $A9:D67C/$A9:D69C.
pub(crate) fn sand_heap_line(game: &Game, index: isize) -> SandHeapLine {
    let rom = rom(game);
    row(&game.torizo_corpse_tables.sand_heap_lines, rom, index).unwrap_or_else(|| SandHeapLine {
        destination: load(rom, SAND_HEAP_LINE_ADDRESS, index),
        source: load(rom, SAND_HEAP_SOURCE_ADDRESS, index),
    })
}
