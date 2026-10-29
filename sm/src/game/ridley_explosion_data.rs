//! Ridley's explosion tables in bank $A6 through the imported `sm_data` records.
//!
//! Each accessor serves the record while the loaded image is the imported
//! allocation; any other image, and indices past a table, keep the original
//! indexed loads with their failures.
#![forbid(unsafe_code)]
use crate::game::imported::{Imported, ImportedWords};
use crate::game_state::Game;
use crate::rom::{RomImage, lorom_offset};
use sm_data::ridley_explosion::{
    LIFETIME_ADDRESS, LIFETIMES, PART_ADDRESSES, PART_BYTES, PART_SIDES, Part, PartPlacement,
    import_part,
};

#[derive(Default)]
pub(crate) struct ImportedRidleyExplosion {
    lifetimes: ImportedWords<LIFETIME_ADDRESS, LIFETIMES>,
    parts: [Option<Imported<Part>>; PART_ADDRESSES.len()],
}

pub(crate) fn import(image: &RomImage) -> ImportedRidleyExplosion {
    ImportedRidleyExplosion {
        lifetimes: ImportedWords::import(image),
        parts: PART_ADDRESSES
            .map(|address| Imported::import(image, address, PART_BYTES, import_part)),
    }
}

fn rom(game: &Game) -> &RomImage {
    game.loaded_rom.as_ref().expect("ROM must be loaded")
}

/// $A6:C6CE: the lifetime at word `index`.
pub(crate) fn lifetime(game: &Game, index: isize) -> u16 {
    game.ridley_explosion_tables
        .lifetimes
        .word(rom(game), index)
}

/// The placement on `side` for the part whose table starts at `part`, one of
/// `PART_ADDRESSES`.
pub(crate) fn placement(game: &Game, part: u32, side: isize) -> PartPlacement {
    let rom = rom(game);
    let imported = PART_ADDRESSES
        .iter()
        .position(|&address| address == part)
        .and_then(|at| Imported::from(&game.ridley_explosion_tables.parts[at], rom));
    usize::try_from(side)
        .ok()
        .and_then(|row| imported?.row(row))
        .unwrap_or_else(|| {
            let list = |column: u32| {
                rom.table_word(
                    lorom_offset(part + column * 2 * PART_SIDES as u32) as usize,
                    side,
                )
            };
            PartPlacement {
                x_offset: list(0),
                instruction_list: list(1),
            }
        })
}
