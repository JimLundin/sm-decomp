//! Grapple beam origins through the imported `sm_data` table.
#![forbid(unsafe_code)]
use crate::game::imported::Imported;
use crate::game_state::Game;
use crate::rom::{RomImage, lorom_offset};
pub(crate) use sm_data::grapple_origins::OriginSet;
use sm_data::grapple_origins::{ADDRESS, BYTES, BeamOrigin, GrappleOrigins};

pub(crate) type ImportedOrigins = Imported<GrappleOrigins>;

pub(crate) fn import(image: &RomImage) -> Option<ImportedOrigins> {
    Imported::import(image, ADDRESS, BYTES, GrappleOrigins::import)
}

/// The beam origin for `direction` in the loaded image. The table serves the
/// ten directions of the imported allocation; any other image or direction
/// keeps the original four indexed loads with their failures.
pub(crate) fn origin(game: &Game, set: OriginSet, direction: isize) -> BeamOrigin {
    let rom = game.loaded_rom.as_ref().expect("ROM must be loaded");
    usize::try_from(direction)
        .ok()
        .and_then(|direction| Imported::from(&game.grapple_origins, rom)?.origin(set, direction))
        .unwrap_or_else(|| {
            let column = |column| {
                rom.table_word(lorom_offset(set.column_address(column)) as usize, direction)
            };
            BeamOrigin {
                x: column(0),
                y: column(1),
                previous_x: column(2),
                previous_y: column(3),
            }
        })
}
