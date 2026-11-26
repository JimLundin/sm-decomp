//! What one step reports: the picture it publishes and the requests it sends to their owners.

use types::zebes::{
    Brightness, ColourEffectRef, DrawingPages, FlightView, MosaicWidth, PaletteRef, PixelPoint,
    SpritePictureRef, ZebesScene, ZebesState,
};

use crate::plane::{self, PlaneTransform};

/// One picture E1 draws: the imported picture and palette at a whole-pixel anchor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpriteDraw {
    /// The imported picture.
    pub picture: SpritePictureRef,
    /// Its imported palette.
    pub palette: PaletteRef,
    /// Where it is drawn, rounded toward the lower whole pixel.
    pub anchor: PixelPoint,
}

/// The picture values a frame publishes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublishedPicture {
    /// Which drawing shows, for E2, E4 and E1.
    pub view: FlightView,
    /// The screen's light.
    pub light: Brightness,
    /// The mosaic's block width.
    pub mosaic: MosaicWidth,
    /// What E4 draws the ship plane with, kept while the plane is hidden.
    pub plane: PlaneTransform,
    /// What E1 draws, in compositing order: the planet, the star fields in their imported
    /// order, the English caption, then the Japanese subtitle.
    pub sprites: Vec<SpriteDraw>,
}

/// The palette R4 starts the replacing effect from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StartingPalette {
    /// The colours already on screen, carried forward.
    ColoursOnScreen,
}

/// A request to R4.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColourEffectRequest {
    /// Spec: `GunshipAppears`. Replace the preceding scene's active effects with the imported
    /// `engine_accent`, applied to the starting palette.
    ReplaceActive {
        /// The scene's `engine_accent`.
        accent: ColourEffectRef,
        /// Where its samples apply.
        starting_palette: StartingPalette,
    },
    /// Spec: `PlanetIsNamed`. Start the caption's imported brightening effect.
    CaptionBrightens(ColourEffectRef),
    /// Spec: `PlanetIsNamed`. Start the caption's imported dimming effect.
    CaptionDims(ColourEffectRef),
}

/// Spec: `LandingSiteLoadBegins`. To Samus's health owner: her energy returns to its maximum.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RestoreFullEnergy;

/// Spec: `LandingSiteLoadBegins`. To the scheduler (T1): dispatch the Landing Site load with
/// the destination context it holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BeginLandingSiteLoad;

/// Spec: `LandingSiteLoadBegins`. The completed flight's hand-off, published on its last cut
/// only: two independent outputs, one to each owner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Handoff {
    /// To Samus's health owner.
    pub health: RestoreFullEnergy,
    /// To T1, for the Loader (R2).
    pub landing_site: BeginLandingSiteLoad,
}

/// The requests a step sends, each on its own cut only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Requests {
    /// To E2: install the imported drawing pages and their sprite graphics.
    pub pages: Option<DrawingPages>,
    /// To R4: a colour effect.
    pub colour_effect: Option<ColourEffectRequest>,
    /// To the health owner and T1: the hand-off.
    pub handoff: Option<Handoff>,
}

impl Requests {
    /// No request on this cut.
    pub const NONE: Self = Self {
        pages: None,
        colour_effect: None,
        handoff: None,
    };

    /// A colour effect alone.
    #[must_use]
    pub const fn colour_effect(request: ColourEffectRequest) -> Self {
        Self {
            colour_effect: Some(request),
            ..Self::NONE
        }
    }
}

/// What one step reports.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrameReport {
    /// The picture published.
    pub picture: PublishedPicture,
    /// The requests sent.
    pub requests: Requests,
}

/// The picture values of a State: E4's transform of the pose about the imported origin, and
/// E1's draws of each placed picture with its imported picture and palette.
#[must_use]
pub fn publish(state: &ZebesState, scene: &ZebesScene) -> PublishedPicture {
    let sprites = state
        .pictures
        .iter()
        .filter_map(|(which, motion)| {
            let placed = (*motion)?;
            let content = scene.picture(which)?;
            Some(SpriteDraw {
                picture: content.picture,
                palette: content.palette,
                anchor: PixelPoint {
                    x: placed.position.x.pixel,
                    y: placed.position.y.pixel,
                },
            })
        })
        .collect();
    PublishedPicture {
        view: state.view,
        light: state.light,
        mosaic: state.mosaic,
        plane: plane::transform(state.pose, scene.pages.ship_origin),
        sprites,
    }
}
