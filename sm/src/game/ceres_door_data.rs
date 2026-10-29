//! The Ceres steam and door tables in bank $A6 through the imported `sm_data` records.
//!
//! Each accessor serves the record while the loaded image is the imported
//! allocation; any other image, and indices past a table, keep the original
//! indexed loads with their failures.
#![forbid(unsafe_code)]
use crate::game::imported::{Imported, ImportedGrid, ImportedWords};
use crate::game_state::Game;
use crate::rom::{RomImage, lorom_offset};
use sm_data::ceres_doors::{
    DOOR_FUNCTION_ADDRESS, DOOR_INSTRUCTION_ADDRESS, DOOR_KINDS, EXPLOSION_OFFSET_ADDRESS,
    EXPLOSION_OFFSETS, PLATFORM_FRAME_ADDRESS, PLATFORM_FRAMES, STEAM_ADDRESS, STEAM_BYTES,
    STEAM_KINDS, Steam, SteamKind, import_steam,
};

#[derive(Default)]
pub(crate) struct ImportedCeresDoors {
    steam: Option<Imported<Steam>>,
    door_instructions: ImportedWords<DOOR_INSTRUCTION_ADDRESS, DOOR_KINDS>,
    door_functions: ImportedWords<DOOR_FUNCTION_ADDRESS, DOOR_KINDS>,
    explosion_offsets: ImportedGrid<EXPLOSION_OFFSET_ADDRESS, 2, EXPLOSION_OFFSETS>,
    platform_frames: ImportedWords<PLATFORM_FRAME_ADDRESS, PLATFORM_FRAMES>,
}

pub(crate) fn import(image: &RomImage) -> ImportedCeresDoors {
    ImportedCeresDoors {
        steam: Imported::import(image, STEAM_ADDRESS, STEAM_BYTES, import_steam),
        door_instructions: ImportedWords::import(image),
        door_functions: ImportedWords::import(image),
        explosion_offsets: ImportedGrid::import(image),
        platform_frames: ImportedWords::import(image),
    }
}

fn rom(game: &Game) -> &RomImage {
    game.loaded_rom.as_ref().expect("ROM must be loaded")
}

/// $A6:EFF5 and $A6:F001: steam kind `index`'s instruction list and function.
pub(crate) fn steam(game: &Game, index: isize) -> SteamKind {
    let rom = rom(game);
    usize::try_from(index)
        .ok()
        .and_then(|row| Imported::from(&game.ceres_door_tables.steam, rom)?.row(row))
        .unwrap_or_else(|| {
            let list = |column: u32| {
                rom.table_word(
                    lorom_offset(STEAM_ADDRESS + column * 2 * STEAM_KINDS as u32) as usize,
                    index,
                )
            };
            SteamKind {
                instruction_list: list(0),
                function: list(1),
            }
        })
}

/// $A6:F52C: door kind `index`'s instruction list.
pub(crate) fn door_instructions(game: &Game, index: isize) -> u16 {
    game.ceres_door_tables
        .door_instructions
        .word(rom(game), index)
}

/// $A6:F72B: door kind `index`'s function.
pub(crate) fn door_function(game: &Game, index: isize) -> u16 {
    game.ceres_door_tables.door_functions.word(rom(game), index)
}

/// $A6:F840: the explosion spawn offset at word `index`, X then Y per row.
pub(crate) fn explosion_offset(game: &Game, index: isize) -> u16 {
    game.ceres_door_tables
        .explosion_offsets
        .word(rom(game), index)
}

/// $A6:F900: the elevator platform's tile transfers for animation `frame`.
pub(crate) fn platform_frame(game: &Game, frame: isize) -> u16 {
    game.ceres_door_tables
        .platform_frames
        .word(rom(game), frame)
}
