//! The flight to Zebes (F4): the imported `ZebesScene` and the State the scene carries from
//! one finished frame to the next, saved and restored whole.
//!
//! Screen positions are signed pixels, x rightward and y downward. A position that moves keeps
//! a fraction in 65,536ths (`Subpixels`); an anchor from content is whole pixels
//! (`PixelPoint`). Relations between content fields are the importer's.

use core::num;

use enum_map::{Enum, EnumMap};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------------------------
// Content references: each owner's content, named by a typed id so that no two kinds mix.
// ---------------------------------------------------------------------------------------------

/// A tile map for E2 or E4, by content id.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TileMapRef(pub u16);

/// The tile graphics a page's map draws from, by content id.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TileGraphicsRef(pub u16);

/// The sprite sheet the planet, the star fields and the caption pictures are cut from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpriteGraphicsRef(pub u16);

/// One sprite picture E1 draws, by content id.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpritePictureRef(pub u16);

/// The palette a sprite picture is drawn in, by content id.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaletteRef(pub u16);

/// A colour effect R4 plays: the palette entries it changes and its timed colour samples.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColourEffectRef(pub u16);

/// A whole-pixel place on the screen, signed: a content anchor or the ship picture's origin.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PixelPoint {
    /// Rightward.
    pub x: i16,
    /// Downward.
    pub y: i16,
}

// ---------------------------------------------------------------------------------------------
// The imported scene.
// ---------------------------------------------------------------------------------------------

/// One drawing page: its map and the graphics the map draws from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PageRefs {
    /// The page's tile map.
    pub map: TileMapRef,
    /// The graphics its map draws from.
    pub graphics: TileGraphicsRef,
}

/// `DrawingPages`: what E2 installs at setup, and the ship picture E4 transforms.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DrawingPages {
    /// The flat page shown under the mosaic.
    pub flat: PageRefs,
    /// The affine ship picture; its overflow is transparent.
    pub ship: PageRefs,
    /// The ship picture's pixel origin, which E4 turns and zooms about.
    pub ship_origin: PixelPoint,
    /// The sheet of the planet, star and caption pictures.
    pub sprite_graphics: SpriteGraphicsRef,
}

/// `PlacedPicture`: a sprite picture E1 draws, its palette and its anchor in whole pixels.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlacedPicture {
    /// The picture.
    pub picture: SpritePictureRef,
    /// Its palette.
    pub palette: PaletteRef,
    /// Where it is placed.
    pub anchor: PixelPoint,
}

/// One of the four star fields, named by its place in the imported compositing order: E1
/// draws `First` first.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum, Serialize, Deserialize)]
pub enum StarField {
    /// Drawn first of the four.
    First,
    /// Drawn second.
    Second,
    /// Drawn third.
    Third,
    /// Drawn last of the four.
    Fourth,
}

/// `PlanetPictures`: the planet and its four star fields.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanetPictures {
    /// The planet.
    pub planet: PlacedPicture,
    /// The star fields, in compositing order.
    pub star_fields: EnumMap<StarField, PlacedPicture>,
    /// Spec: `PlanetAndStarsSlide`. The field marked `last_field`: the slide ends when it
    /// leaves, wherever the pictures stand.
    pub last_field: StarField,
}

/// The four caption holds, in finished frames, in the order they run.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaptionHolds {
    /// Dark, before the caption shows.
    pub hidden_lead: num::NonZeroU16,
    /// The English caption alone.
    pub english_lead: num::NonZeroU16,
    /// The full caption, with the subtitle where chosen.
    pub full_caption: num::NonZeroU16,
    /// The caption dimming.
    pub fading_caption: num::NonZeroU16,
}

/// `CaptionContent`: the planet's label, its timing and its colour effects.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaptionContent {
    /// The English caption.
    pub english: PlacedPicture,
    /// The Japanese subtitle, where the content has one.
    pub japanese: Option<PlacedPicture>,
    /// The four holds.
    pub holds: CaptionHolds,
    /// R4's effect as the caption appears.
    pub brightening: ColourEffectRef,
    /// R4's effect as the caption fades.
    pub dimming: ColourEffectRef,
}

/// `ZebesScene`: everything the flight imports, checked by the importer before play.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZebesScene {
    /// The pages E2 installs and E4 transforms.
    pub pages: DrawingPages,
    /// The planet and its star fields.
    pub planet: PlanetPictures,
    /// The planet's label.
    pub caption: CaptionContent,
    /// R4's effect that replaces the preceding scene's effects at setup.
    pub engine_accent: ColourEffectRef,
}

// ---------------------------------------------------------------------------------------------
// Picture values.
// ---------------------------------------------------------------------------------------------

/// Spec: `GunshipAppears`. The brightest level, 15.
const FULL_LEVEL: u8 = 15;
/// Spec: `GunshipAppears`. The first level of light, 1.
const FIRST_LEVEL: u8 = 1;
/// Spec: `GunshipAppears`. The widest mosaic block, 9 px.
const WIDEST_BLOCK: u8 = 9;
/// The narrowest mosaic block, 1 px: no mosaic.
const SINGLE_PIXEL: u8 = 1;
/// `GunshipAppears`: the mosaic clock counts finished frames modulo 4.
const BEATS: u8 = 4;

/// `FlightView`: which drawing the screen shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FlightView {
    /// The flat page under the mosaic, sprites enabled.
    MosaicPage,
    /// The ship plane, with the planet, stars and caption as sprites.
    ShipAndSprites,
    /// The sprites alone: the ship plane hidden.
    SpritesOnly,
}

/// A level of shown light, 1 (dimmest) through 15 (full).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub struct LitLevel(u8);

impl LitLevel {
    /// The first light, level 1.
    pub const FIRST: Self = Self(FIRST_LEVEL);
    /// Full light, level 15.
    pub const FULL: Self = Self(FULL_LEVEL);

    /// The level of this number, if it is 1 through 15.
    #[must_use]
    pub const fn new(level: u8) -> Option<Self> {
        if level >= FIRST_LEVEL && level <= FULL_LEVEL {
            Some(Self(level))
        } else {
            None
        }
    }

    /// Its number.
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }
}

impl TryFrom<u8> for LitLevel {
    type Error = &'static str;

    fn try_from(level: u8) -> Result<Self, Self::Error> {
        Self::new(level).ok_or("a shown level is 1 through 15")
    }
}

impl From<LitLevel> for u8 {
    fn from(level: LitLevel) -> Self {
        level.get()
    }
}

/// `Brightness`: the screen's light, blanked or shown at a level.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Brightness {
    /// Nothing shows.
    Blanked,
    /// The picture shows at this level.
    Shown(LitLevel),
}

impl Brightness {
    /// The picture at full light.
    pub const FULL: Self = Self::Shown(LitLevel::FULL);

    /// Spec: `GunshipAppears`. One level brighter: blanked becomes level 1, and full stays full.
    #[must_use]
    pub fn brighter(self) -> Self {
        match self {
            Self::Blanked => Self::Shown(LitLevel::FIRST),
            Self::Shown(level) => {
                Self::Shown(LitLevel::new(level.get().saturating_add(1)).unwrap_or(LitLevel::FULL))
            }
        }
    }
}

/// `GunshipAppears`: the mosaic's square width in screen pixels, 1 through 9.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub struct MosaicWidth(u8);

impl MosaicWidth {
    /// Single pixels: no mosaic.
    pub const SINGLE_PIXEL: Self = Self(SINGLE_PIXEL);
    /// The widest blocks, 9 px, where the reveal starts.
    pub const WIDEST: Self = Self(WIDEST_BLOCK);

    /// The width of this many pixels, if it is 1 through 9.
    #[must_use]
    pub const fn new(pixels: u8) -> Option<Self> {
        if pixels >= SINGLE_PIXEL && pixels <= WIDEST_BLOCK {
            Some(Self(pixels))
        } else {
            None
        }
    }

    /// Its pixels.
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }

    /// Spec: `GunshipAppears`. One pixel narrower, down to single pixels.
    #[must_use]
    pub fn finer(self) -> Self {
        Self::new(self.0.saturating_sub(1)).unwrap_or(Self::SINGLE_PIXEL)
    }
}

impl TryFrom<u8> for MosaicWidth {
    type Error = &'static str;

    fn try_from(pixels: u8) -> Result<Self, Self::Error> {
        Self::new(pixels).ok_or("a mosaic block is 1 through 9 px wide")
    }
}

impl From<MosaicWidth> for u8 {
    fn from(width: MosaicWidth) -> Self {
        width.get()
    }
}

/// A signed screen position along one axis: the whole pixel it lies in, rounded toward the
/// lower pixel, and the fraction past it in 65,536ths. So −0.5 px is pixel −1 and fraction
/// 32,768.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Subpixels {
    /// The whole pixel, rounding toward the lower pixel: what E1 and E4 are given.
    pub pixel: i16,
    /// The fraction past it, in 65,536ths.
    pub fraction: u16,
}

impl Subpixels {
    /// The position at this whole pixel, its fraction 0.
    #[must_use]
    pub const fn whole(pixel: i16) -> Self {
        Self { pixel, fraction: 0 }
    }

    /// The position in 65,536ths of a pixel: the carry between the two parts made plain.
    #[must_use]
    pub const fn in_65536ths(self) -> i32 {
        let [low, high] = self.fraction.to_le_bytes();
        let [pixel_low, pixel_high] = self.pixel.to_le_bytes();
        i32::from_le_bytes([low, high, pixel_low, pixel_high])
    }

    /// The position of this many 65,536ths of a pixel; the whole pixel wraps at 16 bits.
    #[must_use]
    pub const fn from_65536ths(position: i32) -> Self {
        let [low, high, pixel_low, pixel_high] = position.to_le_bytes();
        Self {
            pixel: i16::from_le_bytes([pixel_low, pixel_high]),
            fraction: u16::from_le_bytes([low, high]),
        }
    }

    /// Moved by `by` 65,536ths, the fraction carrying into the whole pixel.
    #[must_use]
    pub const fn moved(self, by: i32) -> Self {
        Self::from_65536ths(self.in_65536ths().wrapping_add(by))
    }
}

/// A screen position with fractions on both axes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubpixelPoint {
    /// Rightward.
    pub x: Subpixels,
    /// Downward.
    pub y: Subpixels,
}

impl SubpixelPoint {
    /// The position of a whole-pixel anchor.
    #[must_use]
    pub const fn at(anchor: PixelPoint) -> Self {
        Self {
            x: Subpixels::whole(anchor.x),
            y: Subpixels::whole(anchor.y),
        }
    }
}

/// The ship plane's angle in 256ths of a turn, increasing clockwise on screen. It wraps.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaneAngle(pub u8);

/// The ship plane's zoom in 256ths: 256 is actual size.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaneZoom(pub u16);

/// `PlanePose`: the ship plane's angle, zoom and scroll position.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanePose {
    /// How it is turned.
    pub angle: PlaneAngle,
    /// How it is zoomed.
    pub zoom: PlaneZoom,
    /// Its scroll position.
    pub scroll: SubpixelPoint,
}

impl PlanePose {
    /// The pose of this angle, zoom and whole-pixel scroll.
    #[must_use]
    pub const fn new(angle: u8, zoom: u16, scroll: PixelPoint) -> Self {
        Self {
            angle: PlaneAngle(angle),
            zoom: PlaneZoom(zoom),
            scroll: SubpixelPoint::at(scroll),
        }
    }
}

/// A placed picture's upward speed in 256ths of a pixel per frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpwardSpeed(pub u16);

impl UpwardSpeed {
    /// At rest, as every picture is placed.
    pub const AT_REST: Self = Self(0);
}

/// Where a placed picture stands and how fast it rises.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PictureMotion {
    /// Its position.
    pub position: SubpixelPoint,
    /// Its upward speed.
    pub speed: UpwardSpeed,
}

impl PictureMotion {
    /// At rest on a content anchor, fractions zero.
    #[must_use]
    pub const fn placed(anchor: PixelPoint) -> Self {
        Self {
            position: SubpixelPoint::at(anchor),
            speed: UpwardSpeed::AT_REST,
        }
    }
}

/// A picture the scene can place, in E1's compositing order: the planet, the star fields in
/// their imported order, then the English caption and the Japanese subtitle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum, Serialize, Deserialize)]
pub enum ScenePicture {
    /// The planet.
    Planet,
    /// A star field.
    Stars(StarField),
    /// The English caption.
    English,
    /// The Japanese subtitle.
    Japanese,
}

/// The placed pictures: each one's motion while it is placed. Iterating gives E1's order.
pub type PlacedPictures = EnumMap<ScenePicture, Option<PictureMotion>>;

impl ZebesScene {
    /// The imported picture, palette and anchor of a scene picture; none for a Japanese
    /// subtitle the content doesn't have.
    #[must_use]
    pub fn picture(&self, which: ScenePicture) -> Option<PlacedPicture> {
        match which {
            ScenePicture::Planet => Some(self.planet.planet),
            ScenePicture::Stars(field) => Some(self.planet.star_fields[field]),
            ScenePicture::English => Some(self.caption.english),
            ScenePicture::Japanese => self.caption.japanese,
        }
    }
}

// ---------------------------------------------------------------------------------------------
// The stage and its counters.
// ---------------------------------------------------------------------------------------------

/// `GunshipAppears`: how many more frames the light holds its level before it rises, one or
/// none.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FadeWait {
    /// The light holds its level for one more frame.
    OneFrame,
    /// The light no longer holds: the next frame brightens it, unless it is already full.
    Elapsed,
}

/// `QuarterBeat`: finished reveal frames since the mosaic last stepped, 0 through 3.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub struct QuarterBeat(u8);

impl QuarterBeat {
    /// The beat itself: the mosaic steps on this frame.
    pub const ON_BEAT: Self = Self(0);
    /// One frame after the beat, as the reveal starts.
    pub const AFTER_BEAT: Self = Self(1);

    /// The clock at this many frames, if it is 0 through 3.
    #[must_use]
    pub const fn new(frames: u8) -> Option<Self> {
        if frames < BEATS {
            Some(Self(frames))
        } else {
            None
        }
    }

    /// Its frames.
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }

    /// One frame later, modulo 4.
    #[must_use]
    pub const fn next(self) -> Self {
        Self(self.0.wrapping_add(1).rem_euclid(BEATS))
    }
}

impl TryFrom<u8> for QuarterBeat {
    type Error = &'static str;

    fn try_from(frames: u8) -> Result<Self, Self::Error> {
        Self::new(frames).ok_or("the mosaic clock is 0 through 3")
    }
}

impl From<QuarterBeat> for u8 {
    fn from(beat: QuarterBeat) -> Self {
        beat.get()
    }
}

/// `PlanetIsNamed`: frames since the planet appeared.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaptionAge(pub u16);

/// `PlanetHolds`: whole frames of the still picture, 1 through 65,535: the hold a flight is
/// given, or the frames left of it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u16", into = "u16")]
pub struct HoldFrames(u16);

impl HoldFrames {
    /// Spec: `PlanetHolds`. The reference hold, 64 finished frames.
    pub const REFERENCE: Self = Self(REFERENCE_HOLD_FRAMES);

    /// The hold of this many frames, if it is 1 through 65,535.
    #[must_use]
    pub const fn new(frames: u16) -> Option<Self> {
        if frames == 0 {
            None
        } else {
            Some(Self(frames))
        }
    }

    /// Its frames.
    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }

    /// One frame fewer, or none when this was the last frame.
    #[must_use]
    pub const fn counted_down(self) -> Option<Self> {
        Self::new(self.0.saturating_sub(1))
    }
}

impl TryFrom<u16> for HoldFrames {
    type Error = &'static str;

    fn try_from(frames: u16) -> Result<Self, Self::Error> {
        Self::new(frames).ok_or("a hold is 1 through 65,535 frames")
    }
}

impl From<HoldFrames> for u16 {
    fn from(hold: HoldFrames) -> Self {
        hold.get()
    }
}

/// Spec: `PlanetHolds`. The reference hold, 64 finished frames.
const REFERENCE_HOLD_FRAMES: u16 = 64;

/// The player's subtitle selection, read each frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubtitleChoice {
    /// The English caption alone.
    EnglishOnly,
    /// The English caption with the Japanese subtitle, where the content has one.
    WithJapanese,
}

/// `ZebesStage`: where the flight stands. A stage that keeps its own state carries it here;
/// the rest read only the picture values in `ZebesState`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZebesStage {
    /// Waiting for the setup frame. It keeps no state of its own.
    Pending,
    /// The mosaic reveal brightening. It keeps how long the light holds its level and the
    /// mosaic clock, and reads the light and the mosaic's width.
    Revealing {
        /// How many more frames the light holds its level before it rises.
        fade: FadeWait,
        /// The mosaic clock.
        beat: QuarterBeat,
    },
    /// The planet shown while its caption runs. It keeps the caption's age, shows and removes
    /// the caption pictures, and sets the ship plane's pose as it ends.
    Caption {
        /// Frames since the planet appeared.
        age: CaptionAge,
    },
    /// The ship drifting right and approaching. It uses the ship plane's pose alone.
    Rightward,
    /// The ship banking left. It uses the ship plane's pose alone.
    Banking,
    /// The ship receding as it drifts left. It uses the ship plane's pose, and hides the ship
    /// plane as it ends.
    Leftward,
    /// The planet and stars held still. It keeps the frames left of the hold.
    Still {
        /// Frames left of the hold.
        hold: HoldFrames,
    },
    /// The planet and stars sliding up. It uses the placed pictures' positions and speeds.
    Sliding,
    /// The marked star field has left: the next frame hands off. It keeps no state of its own.
    Leaving,
    /// The flight is over and handed off. It keeps no state of its own.
    Complete,
}

/// The flight's complete State. A caller may build any valid snapshot, read it back, carry it
/// forward and restore it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ZebesState {
    /// Where the flight stands.
    pub stage: ZebesStage,
    /// Which drawing shows.
    pub view: FlightView,
    /// The screen's light.
    pub light: Brightness,
    /// The mosaic's block width.
    pub mosaic: MosaicWidth,
    /// The ship plane's pose, kept while the plane is hidden.
    pub pose: PlanePose,
    /// The planet, star and caption pictures placed.
    pub pictures: PlacedPictures,
}

impl ZebesState {
    /// The State as the station explosion blacks out: pending, the flat page blanked, single
    /// pixels, the plane cleared and nothing placed.
    #[must_use]
    pub fn pending() -> Self {
        Self {
            stage: ZebesStage::Pending,
            view: FlightView::MosaicPage,
            light: Brightness::Blanked,
            mosaic: MosaicWidth::SINGLE_PIXEL,
            pose: PlanePose::new(0, 0, PixelPoint { x: 0, y: 0 }),
            pictures: PlacedPictures::default(),
        }
    }
}
