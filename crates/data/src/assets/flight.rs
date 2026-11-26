//! The flight to Ceres's scene (docs/architecture/assets.md): how the flight is made ready, how
//! its view turns with the sprites it places, and the station's label, in the flight's own
//! format.

pub use types::flight::{
    Angle, Drift, Drifter, Keystroke, LabelPlace, LabelStep, RushSpeed, Scene, SceneSetup,
    SceneTurn, SpritePlacement, SpriteWrap, TypedLetter, WholePlace, Zoom,
};

/// One flight object's sprite art and palette selection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpriteArt {
    /// The object's fixed drawing, or the first vortex frame.
    pub map: crate::spritemap::Spritemap,
    /// The palette bits combined with the map's attributes.
    pub palette: u8,
}

/// Drawings for the objects placed during the flight.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpritePictures {
    /// The rushing star field.
    pub rushing: SpriteArt,
    /// The large asteroids.
    pub large_asteroids: SpriteArt,
    /// The station under attack.
    pub station: SpriteArt,
    /// The small asteroids.
    pub small_asteroids: SpriteArt,
    /// The vortex's first drawing.
    pub vortex: SpriteArt,
    /// The distant stars drifting sideways.
    pub distant_stars: SpriteArt,
    /// The vortex's second drawing, shown on alternating frames.
    pub alternate_vortex: crate::spritemap::Spritemap,
}
