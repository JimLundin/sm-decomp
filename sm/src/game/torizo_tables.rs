//! The Torizos' bank $AA instruction lists through the imported `sm_data` record.
//!
//! The accessor serves the record while the loaded image is the imported
//! allocation; any other image, and indices outside the table, keep the
//! original indexed load with its failures.
#![forbid(unsafe_code)]
use crate::game::imported::{Imported, load, rom, row};
use crate::game_state::Game;
use crate::rom::RomImage;
use sm_data::torizo::*;

#[derive(Default)]
pub(crate) struct ImportedTorizos {
    torizos: Option<Imported<Torizos>>,
}

pub(crate) fn import(image: &RomImage) -> ImportedTorizos {
    ImportedTorizos {
        torizos: Imported::import(
            image,
            INSTRUCTION_LIST_ADDRESS,
            INSTRUCTION_LIST_BYTES,
            import_torizos,
        ),
    }
}

/// Torizo `index` at $AA:C967.
pub(crate) fn torizo(game: &Game, index: isize) -> Torizo {
    let rom = rom(game);
    row(&game.torizo_tables.torizos, rom, index).unwrap_or_else(|| Torizo {
        instruction_list: load(rom, INSTRUCTION_LIST_ADDRESS, index),
    })
}
