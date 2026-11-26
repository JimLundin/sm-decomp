//! Stationary room-effect clips as holds and ordered sprite pictures.

use crate::spritemap::Spritemap;
use core::num::NonZeroU16;

/// A picture and its positive duration in effect turns.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Picture {
    /// How long the picture shows.
    pub hold: NonZeroU16,
    /// The common sprite pieces drawn about the effect's centre.
    pub sprites: Spritemap,
}

/// Both finite effects requested by the Ceres shaft overlay.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShaftBurst {
    /// The smoke pictures in playback order, before the effect retires.
    pub smoke: [Picture; 4],
    /// The small-explosion pictures in playback order, before the effect retires.
    pub small_explosion: [Picture; 6],
}
