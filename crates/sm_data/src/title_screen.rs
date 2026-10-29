//! Title screen tables in bank $8C.
pub use crate::table::LengthError;
use crate::table::Rows;

/// $8C:BC5D: the bank $8C backdrop gradient HDMA table for each zoom level,
/// chosen by bits 4-7 of the zoom.
pub const GRADIENT_ADDRESS: u32 = 0x8c_bc5d;
pub const GRADIENTS: usize = 16;
pub type Gradients = Rows<u16, GRADIENTS>;

pub fn import_gradients(bytes: &[u8]) -> Result<Gradients, LengthError> {
    Rows::import(bytes, |[gradient]| gradient)
}
