//! The station, sprite drawings and navigation lights installed after the elevator blacks out.

pub use types::{Pair, explosion::*};

use crate::spritemap::Spritemap;

/// One timed pair of navigation light colours.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NavigationSample {
    /// Finished frames in this sample.
    pub frames: u16,
    /// The two adjacent colours the light replaces.
    pub colours: [u16; 2],
}

/// Player-ROM content addressed by the cinematic's typed drawing and effect requests.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Station {
    /// The content read by the clean scene.
    pub scene: ExplosionScene,
    /// Mode 7 map cells at the station and ship views.
    pub station_map: Vec<u8>,
    /// Front, back and transparent lower half of the gunship.
    pub ship_front: Vec<u8>,
    /// Rear of the ship, retained for the flight.
    pub ship_back: Vec<u8>,
    /// Transparent map placed below the gunship.
    pub empty_map: Vec<u8>,
    /// The shared affine graphics.
    pub plane_tiles: Vec<u8>,
    /// The space sprite tiles after the common tiles have been installed over them.
    pub sprite_tiles: Vec<u8>,
    /// Sprite pictures named by the scene.
    pub drawings: Vec<(SpriteDrawing, Spritemap)>,
    /// Colour group selected for the sky.
    pub sky_palette: u8,
    /// Colour group selected for the bursts.
    pub burst_palette: u8,
    /// Initial colours and the cyclic light samples.
    pub palette: [u16; 256],
    /// First colour replaced by the navigation lights.
    pub light_first: usize,
    /// The navigation light sequence.
    pub lights: Vec<NavigationSample>,
}
