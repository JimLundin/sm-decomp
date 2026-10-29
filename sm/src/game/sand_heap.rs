//! Dead Torizo's sand heap tiles through the imported `sm_data` record.
#![forbid(unsafe_code)]
use crate::game::imported::Imported;
use crate::game_state::Game;
use crate::rom::{RomImage, lorom_offset};
use sm_data::sand_heap::{ADDRESS, BYTES, SandHeapTiles};

pub(crate) type ImportedSandHeap = Imported<SandHeapTiles>;

pub(crate) fn import(image: &RomImage) -> Option<ImportedSandHeap> {
    Imported::import(image, ADDRESS, BYTES, SandHeapTiles::import)
}

/// The word at a word index from $B7:A800 in the loaded image. The record
/// serves it while that image is the imported allocation; otherwise, and past
/// the tiles, the original indexed load runs with its failures.
pub(crate) fn word(game: &Game, index: usize) -> u16 {
    let rom = game.loaded_rom.as_ref().expect("ROM must be loaded");
    Imported::from(&game.sand_heap, rom)
        .and_then(|tiles| tiles.word(index))
        .unwrap_or_else(|| rom.table_word(lorom_offset(ADDRESS) as usize, index as isize))
}
