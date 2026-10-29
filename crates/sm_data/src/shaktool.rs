//! Shaktool's tables in bank $AA: the seven pieces' initial data, the head's
//! instruction list per direction and the sine tables that place its pieces.
pub use crate::table::LengthError;
use crate::table::Rows;

/// $AA:DE95: ten columns of seven words, one row per piece from the initially
/// right saw hand (the primary piece) to the initially left saw hand.
pub const PIECE_ADDRESS: u32 = 0xaa_de95;
pub const PIECE_ROWS: usize = 7;
pub const PIECE_BYTES: usize = PIECE_ROWS * 20;
/// The address of each column after the first, as the original loads name them.
pub const RAM_OFFSET_ADDRESS: u32 = 0xaa_dea3;
pub const NEIGHBOUR_ANGLE_ADDRESS: u32 = 0xaa_deb1;
pub const INSTRUCTION_LIST_ADDRESS: u32 = 0xaa_debf;
pub const LAYER_ADDRESS: u32 = 0xaa_decd;
pub const FUNCTION_ADDRESS: u32 = 0xaa_dedb;
pub const CURLING_DELTA_ADDRESS: u32 = 0xaa_dee9;
pub const ZERO_ADDRESS: u32 = 0xaa_def7;
pub const UNUSED_ADDRESS: u32 = 0xaa_df05;
pub const HEAD_BOB_ADDRESS: u32 = 0xaa_df13;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Piece {
    pub properties: u16,
    /// Enemy RAM offset from the primary piece.
    pub ram_offset: u16,
    pub neighbour_angle: u16,
    /// Bank $AA pointer to the initial instruction list.
    pub instruction_list: u16,
    pub layer: u16,
    /// Bank $AA pointer to the piece's function.
    pub function: u16,
    /// Initial curling neighbour angle delta.
    pub curling_delta: u16,
    pub zero: u16,
    pub unused: u16,
    /// Bank $AA pointer to the head bob instruction list.
    pub head_bob: u16,
}

pub type Pieces = Rows<Piece, PIECE_ROWS>;

pub fn import_pieces(bytes: &[u8]) -> Result<Pieces, LengthError> {
    Rows::import_columns(
        bytes,
        |[
            properties,
            ram_offset,
            neighbour_angle,
            instruction_list,
            layer,
            function,
            curling_delta,
            zero,
            unused,
            head_bob,
        ]| Piece {
            properties,
            ram_offset,
            neighbour_angle,
            instruction_list,
            layer,
            function,
            curling_delta,
            zero,
            unused,
            head_bob,
        },
    )
}

/// $AA:DD15: the head's instruction lists for its eight directions.
pub const HEAD_DIRECTION_ADDRESS: u32 = 0xaa_dd15;
pub const HEAD_DIRECTION_ROWS: usize = 8;
pub const HEAD_DIRECTION_BYTES: usize = HEAD_DIRECTION_ROWS * 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HeadDirection {
    /// Bank $AA pointer to the head's instruction list.
    pub instruction_list: u16,
}

pub type HeadDirections = Rows<HeadDirection, HEAD_DIRECTION_ROWS>;

pub fn import_head_directions(bytes: &[u8]) -> Result<HeadDirections, LengthError> {
    Rows::import(bytes, |[instruction_list]| HeadDirection {
        instruction_list,
    })
}

/// $AA:E0BD sine and $AA:E03D negative cosine over a full turn of 256 angles;
/// both run on through the following tables of $E03D..$E2BD.
pub const SINE_ADDRESS: u32 = 0xaa_e0bd;
pub const NEGATIVE_COSINE_ADDRESS: u32 = 0xaa_e03d;
pub const ANGLES: usize = 256;
pub const ANGLE_BYTES: usize = ANGLES * 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Trig {
    pub value: u16,
}

pub type TrigTable = Rows<Trig, ANGLES>;

pub fn import_trig(bytes: &[u8]) -> Result<TrigTable, LengthError> {
    Rows::import(bytes, |[value]| Trig { value })
}
