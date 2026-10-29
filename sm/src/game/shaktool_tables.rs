//! Shaktool's bank $AA tables through the imported `sm_data` records.
//!
//! Each accessor serves the record while the loaded image is the imported
//! allocation; any other image, and indices outside a table, keep the original
//! indexed loads with their failures.
#![forbid(unsafe_code)]
use crate::game::imported::{Imported, load, rom, row};
use crate::game_state::Game;
use crate::rom::RomImage;
use sm_data::shaktool::*;

#[derive(Default)]
pub(crate) struct ImportedShaktool {
    pieces: Option<Imported<Pieces>>,
    head_directions: Option<Imported<HeadDirections>>,
    sine: Option<Imported<TrigTable>>,
    negative_cosine: Option<Imported<TrigTable>>,
}

pub(crate) fn import(image: &RomImage) -> ImportedShaktool {
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
pub(crate) fn piece(game: &Game, index: isize) -> Piece {
    let rom = rom(game);
    row(&game.shaktool_tables.pieces, rom, index).unwrap_or_else(|| Piece {
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
pub(crate) fn head_direction(game: &Game, index: isize) -> HeadDirection {
    let rom = rom(game);
    row(&game.shaktool_tables.head_directions, rom, index).unwrap_or_else(|| HeadDirection {
        instruction_list: load(rom, HEAD_DIRECTION_ADDRESS, index),
    })
}

/// The sine of `angle` at $AA:E0BD.
pub(crate) fn sine(game: &Game, angle: isize) -> u16 {
    let rom = rom(game);
    row(&game.shaktool_tables.sine, rom, angle)
        .map_or_else(|| load(rom, SINE_ADDRESS, angle), |trig| trig.value)
}

/// The negative cosine of `angle` at $AA:E03D.
pub(crate) fn negative_cosine(game: &Game, angle: isize) -> u16 {
    let rom = rom(game);
    row(&game.shaktool_tables.negative_cosine, rom, angle).map_or_else(
        || load(rom, NEGATIVE_COSINE_ADDRESS, angle),
        |trig| trig.value,
    )
}
