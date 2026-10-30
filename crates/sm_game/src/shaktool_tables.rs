//! Shaktool's bank $AA tables through the imported `sm_data` records.
//!
//! Each accessor serves the record while the loaded image is the imported
//! allocation; any other image, and indices outside a table, keep the original
//! indexed loads with their failures.
use crate::imported::{Imported, load, rom, row};
use crate::records::Records;
use sm_data::cartridge::RomImage;
use sm_data::shaktool::*;

#[derive(Default)]
pub struct ImportedShaktool {
    pieces: Option<Imported<Pieces>>,
    head_directions: Option<Imported<HeadDirections>>,
    sine: Option<Imported<TrigTable>>,
    negative_cosine: Option<Imported<TrigTable>>,
}

pub fn import(image: &RomImage) -> ImportedShaktool {
    ImportedShaktool {
        pieces: Imported::import(image, PIECE_ADDRESS, PIECE_BYTES, import_pieces),
        head_directions: Imported::import(
            image,
            HEAD_DIRECTION_ADDRESS,
            HEAD_DIRECTION_BYTES,
            import_head_directions,
        ),
        sine: Imported::import(image, SINE_ADDRESS, ANGLE_BYTES, import_trig),
        negative_cosine: Imported::import(image, NEGATIVE_COSINE_ADDRESS, ANGLE_BYTES, import_trig),
    }
}

/// Piece `index` at $AA:DE95.
pub fn piece(game: &(impl Records + ?Sized), index: isize) -> Piece {
    let rom = rom(game);
    row(&game.records().shaktool_tables.pieces, rom, index).unwrap_or_else(|| Piece {
        properties: load(rom, PIECE_ADDRESS, index),
        ram_offset: load(rom, RAM_OFFSET_ADDRESS, index),
        neighbour_angle: load(rom, NEIGHBOUR_ANGLE_ADDRESS, index),
        instruction_list: load(rom, INSTRUCTION_LIST_ADDRESS, index),
        layer: load(rom, LAYER_ADDRESS, index),
        function: load(rom, FUNCTION_ADDRESS, index),
        curling_delta: load(rom, CURLING_DELTA_ADDRESS, index),
        zero: load(rom, ZERO_ADDRESS, index),
        unused: load(rom, UNUSED_ADDRESS, index),
        head_bob: load(rom, HEAD_BOB_ADDRESS, index),
    })
}

/// Head direction `index` at $AA:DD15.
pub fn head_direction(game: &(impl Records + ?Sized), index: isize) -> HeadDirection {
    let rom = rom(game);
    row(&game.records().shaktool_tables.head_directions, rom, index).unwrap_or_else(|| {
        HeadDirection {
            instruction_list: load(rom, HEAD_DIRECTION_ADDRESS, index),
        }
    })
}

/// The sine of `angle` at $AA:E0BD.
pub fn sine(game: &(impl Records + ?Sized), angle: isize) -> u16 {
    let rom = rom(game);
    row(&game.records().shaktool_tables.sine, rom, angle)
        .map_or_else(|| load(rom, SINE_ADDRESS, angle), |trig| trig.value)
}

/// The negative cosine of `angle` at $AA:E03D.
pub fn negative_cosine(game: &(impl Records + ?Sized), angle: isize) -> u16 {
    let rom = rom(game);
    row(&game.records().shaktool_tables.negative_cosine, rom, angle).map_or_else(
        || load(rom, NEGATIVE_COSINE_ADDRESS, angle),
        |trig| trig.value,
    )
}
