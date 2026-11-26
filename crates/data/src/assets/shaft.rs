//! The Ceres escape shaft's overlay art and ordered rotation samples.

/// Number of consecutive packed sprite tile bytes in the overlay's upload.
pub const OVERLAY_BYTES: usize = 1024;

/// The number of ordered samples along the shaft's rotation arc.
pub const ROTATION_SAMPLES: usize = 69;

/// A shaft rotation sample, in playback order along the arc.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RotationSample {
    /// Whole turns to hold this matrix before reading another sample.
    pub hold: u16,
    /// Signed 8.8 cross coefficient. The other cross coefficient has the opposite sign.
    pub cross: i16,
    /// Signed 8.8 coefficient shared by both diagonals.
    pub diagonal: i16,
}

/// Immutable art and rotation assets used during the Ceres shaft escape.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Shaft {
    /// Packed sprite tiles installed when the overlay is placed.
    pub overlay: [u8; OVERLAY_BYTES],
    /// The ordered spritemap for the overlay during its rumble.
    pub figure: crate::spritemap::Spritemap,
    /// The two platform looks in cycle order. Each replaces only the low tilemap bytes.
    pub platform: [[u8; 4]; 2],
    /// The samples from one end of the rotation arc to the other.
    pub rotation: [RotationSample; ROTATION_SAMPLES],
}
