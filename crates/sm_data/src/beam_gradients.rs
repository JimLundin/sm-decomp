//! Beam edge gradients at $91:C9D4: |tan(angle * pi / 80h)| * 100h for
//! angles 0 to 80h, read by the X-ray and Mother Brain rainbow beam windows.
pub use crate::table::LengthError;
use crate::table::Rows;

pub const ADDRESS: u32 = 0x91_c9d4;
pub const ANGLES: usize = 0x81;
pub const BYTES: usize = 2 * ANGLES;

pub type BeamGradients = Rows<u16, ANGLES>;

pub fn import(bytes: &[u8]) -> Result<BeamGradients, LengthError> {
    Rows::import(bytes, |[gradient]| gradient)
}
