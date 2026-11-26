//! The station explosion: the scene's imported content (`ExplosionScene`) and the State it
//! carries from one finished frame to the next, saved and restored whole.
//!
//! Positions are signed pixels with a fraction in 65,536ths (`Subpixels`), x rightward and y
//! downward. Relations between content fields are the importer's.

use core::num;

use enum_map::{Enum, EnumMap};
use serde::{Deserialize, Serialize};

use crate::Pair;

/// 65,536 subpixels make one pixel.
const SUBPIXELS_PER_PIXEL: i32 = 65_536;
/// Round this position down to the whole pixel.
const WHOLE_PIXEL_BITS: i32 = -65_536;
/// Spec: `StationBrightens`. The brightest level of the light.
const BRIGHTEST: u8 = 15;

/// A signed length or place along one axis, in 65,536ths of a pixel. Its sums wrap at 32 bits.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Subpixels(pub i32);

impl Subpixels {
    /// This many whole pixels.
    #[must_use]
    pub const fn whole(pixels: i32) -> Self {
        Self(pixels.wrapping_mul(SUBPIXELS_PER_PIXEL))
    }

    /// The fraction `pixels / parts` of a pixel (`fraction(-1, 4)` is a quarter pixel back),
    /// rounded toward zero; no parts make no length.
    #[must_use]
    pub const fn fraction(pixels: i32, parts: i32) -> Self {
        match Self::whole(pixels).0.checked_div(parts) {
            Some(length) => Self(length),
            None => Self(0),
        }
    }

    /// The sum of the two.
    #[must_use]
    pub const fn plus(self, other: Self) -> Self {
        Self(self.0.wrapping_add(other.0))
    }

    /// This less `other`.
    #[must_use]
    pub const fn minus(self, other: Self) -> Self {
        Self(self.0.wrapping_sub(other.0))
    }

    /// Its whole pixels: round toward the lower pixel (the left or the top), the fraction
    /// cleared.
    #[must_use]
    pub const fn floor(self) -> Self {
        Self(self.0 & WHOLE_PIXEL_BITS)
    }

    /// Brought within 0 up to `width` by wrapping around; a width of none leaves it as it is.
    #[must_use]
    pub const fn wrapped(self, width: Self) -> Self {
        match self.0.checked_rem_euclid(width.0) {
            Some(within) => Self(within),
            None => self,
        }
    }
}

/// A place on the screen, or a move across it, in subpixels.
pub type ScreenPoint = Pair<Subpixels>;

impl Pair<Subpixels> {
    /// The place at these whole pixels.
    #[must_use]
    pub fn at_pixels(whole: Pair<i16>) -> Self {
        Self {
            x: Subpixels::whole(i32::from(whole.x)),
            y: Subpixels::whole(i32::from(whole.y)),
        }
    }

    /// Moved by `by`.
    #[must_use]
    pub const fn plus(self, by: Self) -> Self {
        Self {
            x: self.x.plus(by.x),
            y: self.y.plus(by.y),
        }
    }

    /// This less `other`, axis by axis.
    #[must_use]
    pub const fn minus(self, other: Self) -> Self {
        Self {
            x: self.x.minus(other.x),
            y: self.y.minus(other.y),
        }
    }

    /// Each axis rounded down to its whole pixels.
    #[must_use]
    pub const fn floor(self) -> Self {
        Self {
            x: self.x.floor(),
            y: self.y.floor(),
        }
    }
}

/// Where the scene has got to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExplosionStage {
    /// Black while the scheduler sets the scene up.
    Pending,
    /// Prepared and dark until the audio service is ready.
    WaitingForMusic,
    /// The station drifts into view and brightens.
    Revealing,
    /// The station at full light, receding while the bursts cross it.
    Station,
    /// After the flash: the gunship turns and recedes.
    Departing,
    /// The ship has gone; space holds for the frames left.
    Holding(HoldFrames),
    /// Space dims to black.
    Dimming,
    /// Blacked out and handed to the flight to Zebes.
    Complete,
}

/// Spec: `SpaceHolds`. Whole frames of the hold: 1 to 65,535. Every value is one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HoldFrames(pub num::NonZeroU16);

impl HoldFrames {
    /// One frame fewer, or none when this was the last.
    #[must_use]
    pub const fn counted_down(self) -> Option<Self> {
        match num::NonZeroU16::new(self.0.get().saturating_sub(1)) {
            Some(left) => Some(Self(left)),
            None => None,
        }
    }
}

/// A level of the shown light: 1 (dimmest) to 15 (full).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub struct LightLevel(u8);

impl LightLevel {
    /// The level, if it is 1 to 15.
    #[must_use]
    pub const fn new(level: u8) -> Option<Self> {
        if level >= 1 && level <= BRIGHTEST {
            Some(Self(level))
        } else {
            None
        }
    }

    /// Its number, 1 to 15.
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }
}

impl TryFrom<u8> for LightLevel {
    type Error = &'static str;

    fn try_from(level: u8) -> Result<Self, Self::Error> {
        Self::new(level).ok_or("a shown light level must be 1 to 15")
    }
}

impl From<LightLevel> for u8 {
    fn from(level: LightLevel) -> Self {
        level.get()
    }
}

/// The scene's light: the display blanked, or the picture shown at a level.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Light {
    /// Nothing shown; counts as level 0.
    Blanked,
    /// The picture shown at this level.
    Shown(LightLevel),
}

impl Light {
    /// The picture shown at full light, level 15.
    pub const FULL: Self = Self::Shown(LightLevel(BRIGHTEST));

    /// Spec: `StationBrightens`. One level up, blank counting as zero; full stays full.
    #[must_use]
    pub const fn brighter(self) -> Self {
        let level = match self {
            Self::Blanked => 0,
            Self::Shown(level) => level.0,
        };
        match LightLevel::new(level.saturating_add(1)) {
            Some(raised) => Self::Shown(raised),
            None => Self::FULL,
        }
    }

    /// Spec: `SceneBlacksOut`. One level down; below level 1 the display blanks.
    #[must_use]
    pub const fn dimmer(self) -> Self {
        match self {
            Self::Blanked => Self::Blanked,
            Self::Shown(level) => match LightLevel::new(level.0.saturating_sub(1)) {
                Some(lowered) => Self::Shown(lowered),
                None => Self::Blanked,
            },
        }
    }
}

/// Whether the fade waits a completed frame before its next level.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FadeWait {
    /// The next fade frame changes the light.
    NoFrame,
    /// The fade holds for one beat: the next fade frame only clears this wait.
    OneFrame,
}

/// The plane's angle, in 256ths of a clockwise turn: every byte is one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaneAngle(pub u8);

/// The plane's zoom, in 256ths of actual size: 256 shows it at its size, more shows it smaller.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PlaneZoom(pub u16);

/// The affine plane's pose: its angle, its zoom and its signed scroll.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanePose {
    /// How it is turned.
    pub angle: PlaneAngle,
    /// How large it looks.
    pub zoom: PlaneZoom,
    /// How far it is scrolled.
    pub scroll: ScreenPoint,
}

/// What the affine plane shows. The pivot it turns about is the scene's (`ExplosionScene`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlaneView {
    /// Nothing installed yet.
    Uninstalled,
    /// The station.
    Station,
    /// The gunship's front.
    Gunship,
}

/// Spec: `SpaceDrifts`. Finished scene frames since the pictures were placed, the placement
/// frame included.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PictureAge(pub u32);

/// One of the two asteroid fields that drift across space.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum, Serialize, Deserialize)]
pub enum AsteroidField {
    /// The large asteroids appear farther away than the bursts.
    Large,
    /// The small asteroids, drifting slowly so the asteroids appear farther away; drawn in
    /// front of the bursts.
    Small,
}

/// Which of its two drawings the purple background shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum, Serialize, Deserialize)]
pub enum BackdropFrame {
    /// The first drawing.
    First,
    /// The second drawing.
    Second,
}

/// A group of bursts. Listed front to back: a burst of an earlier group draws in front.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Enum, Serialize, Deserialize)]
pub enum BurstGroup {
    /// The one burst that marks the station's breakup.
    Final,
    /// The first group, crossing the station first.
    First,
    /// The middle group, launched one at a time.
    Middle,
    /// The last group before the breakup.
    Last,
}

/// A burst's identity: its group and its occurrence in the group's launch order, from 1. Ordered
/// front to back.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct BurstId {
    /// Its group.
    pub group: BurstGroup,
    /// Its place in the group's launch order.
    pub occurrence: num::NonZeroU8,
}

/// A burst's signed sample age, in frames: below zero while its first picture is delayed,
/// then the frames since its track began.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SampleAge(pub i16);

/// A live burst.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Burst {
    /// Which burst it is.
    pub id: BurstId,
    /// How far its track has played.
    pub age: SampleAge,
    /// Where it is on the screen.
    pub position: ScreenPoint,
}

/// Everything the explosion remembers from one finished frame to the next. A caller may build
/// it at any value its types allow, and carry, save and restore it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExplosionState {
    /// Where the scene has got to, with the hold's frames while it holds.
    pub stage: ExplosionStage,
    /// The scene's light.
    pub light: Light,
    /// Whether the fade holds for one beat.
    pub fade: FadeWait,
    /// The affine plane's pose.
    pub pose: PlanePose,
    /// What the affine plane shows.
    pub view: PlaneView,
    /// Finished frames since the pictures were placed.
    pub age: PictureAge,
    /// Each asteroid field's x; its y is the scene's.
    pub asteroids: EnumMap<AsteroidField, Subpixels>,
    /// The live bursts, front to back.
    pub bursts: Vec<Burst>,
}

impl ExplosionState {
    /// The State as `BeginStationExplosion` arrives: pending, the display blank, nothing placed,
    /// the plane at rest at zero.
    #[must_use]
    pub fn pending() -> Self {
        let zero = Subpixels(0);
        Self {
            stage: ExplosionStage::Pending,
            light: Light::Blanked,
            fade: FadeWait::NoFrame,
            pose: PlanePose {
                angle: PlaneAngle(0),
                zoom: PlaneZoom(0),
                scroll: Pair { x: zero, y: zero },
            },
            view: PlaneView::Uninstalled,
            age: PictureAge(0),
            asteroids: EnumMap::from_fn(|_| zero),
            bursts: Vec::new(),
        }
    }
}

/// The stage and sprite art installed during preparation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtSet(pub String);

/// The initial space palette and its navigation light effect.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpacePalette(pub String);

/// The presentation owner's style for the station flash.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StationFlashStyle(pub String);

/// A drawing installed into a region of the affine plane's map.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MapDrawing(pub String);

/// A sprite drawing.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpriteDrawing(pub String);

/// The colour group a picture's sprites draw with: the sky's, or a burst track's.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PictureColours(pub String);

/// An asteroid field as imported: its start, its drift each frame and its drawing.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AsteroidContent {
    /// Where it starts, in whole pixels; its y stays there.
    pub start: Pair<i16>,
    /// How far right it moves each frame.
    pub speed: Subpixels,
    /// Its drawing.
    pub drawing: SpriteDrawing,
}

/// The purple background as imported: its place and its two alternating drawings.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackdropContent {
    /// Its position in pixels, where it stays.
    pub place: Pair<i16>,
    /// Its two drawings.
    pub drawings: EnumMap<BackdropFrame, SpriteDrawing>,
}

/// The sky's three pictures and their palette.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkyContent {
    /// The two asteroid fields.
    pub asteroids: EnumMap<AsteroidField, AsteroidContent>,
    /// The purple background.
    pub backdrop: BackdropContent,
    /// The sky pictures' colours.
    pub palette: PictureColours,
}

/// What a sample of a burst track shows.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SampleLook {
    /// Nothing.
    Blank,
    /// This drawing.
    Drawing(SpriteDrawing),
}

/// One sample of a burst track: what it shows, for a positive number of frames.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrackSample {
    /// Its frames.
    pub frames: num::NonZeroU8,
    /// What it shows.
    pub look: SampleLook,
}

/// A burst group's track: one pass of samples, played a positive number of passes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BurstTrack {
    /// One pass, in order.
    pub samples: Vec<TrackSample>,
    /// How many passes play.
    pub passes: num::NonZeroU8,
    /// The colours its drawings use.
    pub palette: PictureColours,
}

/// The station explosion's imported content.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExplosionScene {
    /// The stage and sprite art installed during preparation.
    pub art: ArtSet,
    /// The space palette and navigation light installed during preparation.
    pub space_palette: SpacePalette,
    /// The station flash's style.
    pub flash_style: StationFlashStyle,
    /// The gunship front, installed in the map's upper region at the breakup.
    pub ship_front: MapDrawing,
    /// The empty drawing, installed in the map's lower region at the breakup.
    pub empty_drawing: MapDrawing,
    /// The point the plane turns about, in whole screen pixels.
    pub pivot: Pair<i16>,
    /// The sky.
    pub sky: SkyContent,
    /// Each burst group's track.
    pub tracks: EnumMap<BurstGroup, BurstTrack>,
}
