//! HDMA object instruction tables in bank $88.
pub use crate::table::LengthError;
use crate::table::Rows;

/// $88:D992: the BG3 X velocities $88:D981 picks from at random: -600h,
/// 600h, -400h and 400h.
pub const BG3_X_VELOCITY_ADDRESS: u32 = 0x88_d992;
pub const BG3_X_VELOCITIES: usize = 4;
pub type Bg3XVelocities = Rows<u16, BG3_X_VELOCITIES>;

pub fn import_bg3_x_velocities(bytes: &[u8]) -> Result<Bg3XVelocities, LengthError> {
    Rows::import(bytes, |[velocity]| velocity)
}
