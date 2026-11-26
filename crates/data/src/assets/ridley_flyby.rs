//! Ridley's getaway path, body palettes and tilemap updates.

use super::art::Colour;
use types::Pair;

/// One picture's scroll change in pixels and zoom in 256ths.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PathPicture {
    /// How far the view moves on each axis.
    pub shift: Pair<i16>,
    /// Scale in 256ths.
    pub zoom: u16,
}

/// A contiguous update of low bytes in the flyby tilemap.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tiles {
    /// First tilemap word to update.
    pub start: u16,
    /// Low bytes in word order.
    pub low: Vec<u8>,
}

/// Art and motion for the Ceres flyby.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Flyby {
    /// Each picture in playback order.
    pub path: Vec<PathPicture>,
    /// Body colours for each whole zoom multiplier, from zero through eight.
    pub body: [[Colour; 15]; 9],
    /// The baby's first, middle and third looks.
    pub baby: [Vec<Tiles>; 3],
    /// The wings' alternating looks.
    pub wings: [Vec<Tiles>; 2],
}
