//! Dead Torizo's sand heap tiles through the imported `sm_data` record.
use crate::imported::Imported;
use crate::records::Records;
use sm_data::cartridge::{RomImage, lorom_offset};
use sm_data::sand_heap::{ADDRESS, BYTES, SandHeapTiles};

pub type ImportedSandHeap = Imported<SandHeapTiles>;

pub fn import(image: &RomImage) -> Option<ImportedSandHeap> {
    Imported::import(image, ADDRESS, BYTES, SandHeapTiles::import)
}

/// The word at a word index from $B7:A800 in the loaded image. The record
/// serves it while that image is the imported allocation; otherwise, and past
/// the tiles, the original indexed load runs with its failures.
pub fn word(game: &(impl Records + ?Sized), index: usize) -> u16 {
    let rom = game.loaded_image();
    Imported::from(&game.records().sand_heap, rom)
        .and_then(|tiles| tiles.word(index))
        .unwrap_or_else(|| rom.table_word(lorom_offset(ADDRESS) as usize, index as isize))
}
