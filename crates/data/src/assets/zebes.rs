//! Imported pages, sprite maps and colour samples for the flight to Zebes.

use std::num::NonZeroU16;

use crate::spritemap::Spritemap;
pub use types::zebes::{
    CaptionContent, CaptionHolds, ColourEffectRef, DrawingPages, PageRefs, PaletteRef, PixelPoint,
    PlacedPicture, PlanetPictures, SpriteGraphicsRef, SpritePictureRef, StarField, TileGraphicsRef,
    TileMapRef, ZebesScene,
};

/// The colours an effect writes for one timed sample.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sample {
    /// Finished frames before the next sample.
    pub frames: NonZeroU16,
    /// Consecutive BGR555 colours in the effect's palette range.
    pub colours: Vec<u16>,
}

/// A resolved palette effect.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Effect {
    /// Identity used by the scene's requests.
    pub id: ColourEffectRef,
    /// First CGRAM colour affected.
    pub first: usize,
    /// Consecutive timed palette writes.
    pub samples: Vec<Sample>,
    /// Whether the effect loops after its last sample.
    pub repeats: bool,
}

/// The flight's scene and the content its consumers resolve.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Zebes {
    /// Scene-facing typed references and placement data.
    pub scene: ZebesScene,
    /// The newly installed rear of the ship, in Mode 7 map-cell order.
    pub ship_map: Vec<u8>,
    /// The shared Mode 7 graphics inherited from the station scene.
    pub ship_tiles: Vec<u8>,
    /// The newly installed flat page, in native tilemap words.
    pub flat_map: Vec<u16>,
    /// The newly installed 4bpp graphics shared by the flat page and sprites.
    pub zebes_tiles: Vec<u16>,
    /// Sprite pictures in compositing order.
    pub pictures: Vec<(SpritePictureRef, Spritemap)>,
    /// Gunship accent and caption fades.
    pub effects: Vec<Effect>,
}
