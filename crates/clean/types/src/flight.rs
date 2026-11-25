//! The gunship's flight to Ceres: the scene's content and the State the flight carries from one
//! frame to the next, saved and restored whole.
//!
//! Positions are pixels with a fraction in 65,536ths (`Fixed`), x rightward and y downward, the
//! whole pixels signed 16-bit. Relations between content fields are the importer's.

use core::num;
use core::num::Wrapping;
use enum_map::{Enum, EnumMap};
use serde::{Deserialize, Serialize};

use crate::{Colour, Fixed, Pair, Parts, px};

/// A whole-pixel place on the screen, signed: where content puts the plane or a sprite.
pub type WholePlace = Pair<i16>;

/// A place on the screen with fractions, each axis `Fixed`.
pub type Place = Pair<Fixed>;

/// The place at these whole pixels, fractions 0.
#[must_use]
pub fn place_at(whole: WholePlace) -> Place {
    Pair {
        x: px(Wrapping(whole.x.cast_unsigned())),
        y: px(Wrapping(whole.y.cast_unsigned())),
    }
}

/// The whole pixels of a place, signed 16-bit: its fractions dropped.
#[must_use]
pub fn whole_pixels(place: Place) -> WholePlace {
    Pair {
        x: place.x.pixel().0.cast_signed(),
        y: place.y.pixel().0.cast_signed(),
    }
}

/// The largest zoom.
const MAX_ZOOM: u16 = 32_767;
/// The brightest level.
const MAX_LEVEL: u8 = 15;
/// The longest the fade's count or rhythm runs, in frames.
const MAX_FADE_FRAMES: u16 = 32_768;

/// An angle of the plane, in 256ths of a turn, clockwise as the picture turns: every byte is
/// one. `0x20` is an eighth of a turn.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Angle(pub u8);

impl Angle {
    /// The plane square, not turned.
    pub const SQUARE: Self = Self(0);
}

/// The plane's zoom, in 256ths, 0 to 32,767: 256 shows the plane at its size, a larger zoom
/// shows it smaller.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u16", into = "u16")]
pub struct Zoom(u16);

impl Zoom {
    /// No zoom: the cleared plane's.
    pub const NONE: Self = Self(0);
    /// The plane shown at its size.
    pub const AT_SIZE: Self = Self(256);

    /// The zoom of this many 256ths, if it is 0 to 32,767.
    #[must_use]
    pub const fn new(zoom: u16) -> Option<Self> {
        if zoom <= MAX_ZOOM {
            Some(Self(zoom))
        } else {
            None
        }
    }

    /// Its 256ths.
    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }

    /// The zoom moved by `by` 256ths, held within 0 to 32,767.
    #[must_use]
    pub const fn shifted(self, by: i16) -> Self {
        let moved = self.0.saturating_add_signed(by);
        Self(if moved > MAX_ZOOM { MAX_ZOOM } else { moved })
    }
}

impl TryFrom<u16> for Zoom {
    type Error = &'static str;

    fn try_from(zoom: u16) -> Result<Self, Self::Error> {
        Self::new(zoom).ok_or("zoom must be 0 to 32,767")
    }
}

impl From<Zoom> for u16 {
    fn from(zoom: Zoom) -> Self {
        zoom.get()
    }
}

/// The turning plane that shows the gunship: its angle, its zoom and its position.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Plane {
    /// How it is turned.
    pub angle: Angle,
    /// How large it looks.
    pub zoom: Zoom,
    /// Where it stands on the screen.
    pub position: Place,
}

/// The rushing stars' speed, in 256ths of a px per frame, signed 16-bit: it wraps.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RushSpeed(pub i16);

/// A sprite's sideways drift, in 65,536ths of a px per frame, signed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Drift(pub i32);

/// Whether a drifting sprite's x comes back in on the other side of 512 px.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpriteWrap {
    /// Its whole pixels are taken modulo 512, 0 to 511.
    Wraps,
    /// Its whole pixels run on, signed 16-bit.
    RunsOn,
}

/// A sprite that drifts sideways once the view has turned. Its order means nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum, Serialize, Deserialize)]
pub enum Drifter {
    /// The station.
    Station,
    /// The small asteroids.
    SmallAsteroids,
    /// The large asteroids.
    LargeAsteroids,
    /// The vortex.
    Vortex,
    /// The distant stars.
    DistantStars,
}

/// A sprite of the flight, as the sprite drawing is told it (`FlightSprite`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FlightSprite {
    /// The stars that streak past as the gunship rushes in.
    RushingStars,
    /// A sprite that drifts sideways.
    Drifting(Drifter),
}

/// The rushing stars as they stand: their place and their speed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StarRush {
    /// Where they are.
    pub place: Place,
    /// How fast they and the plane move, down and to the right alike.
    pub speed: RushSpeed,
}

/// A drifting sprite as it stands: its place, its drift and its wrap.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DriftingSprite {
    /// Where it is.
    pub place: Place,
    /// How far it moves sideways each frame.
    pub drift: Drift,
    /// Whether its x wraps across 0 to 511.
    pub wrap: SpriteWrap,
}

/// The flight's sprites: the rushing stars, if placed, and each drifting sprite placed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct FlightSprites {
    /// The rushing stars, while they are placed.
    pub rushing_stars: Option<StarRush>,
    /// Each drifting sprite, where it is placed.
    pub drifting: EnumMap<Drifter, Option<DriftingSprite>>,
}

/// A place on the label's layer: its column and row.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LabelPlace {
    /// The column, from the left.
    pub column: u16,
    /// The row, from the top.
    pub row: u16,
}

/// A letter of the label and where it is typed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypedLetter {
    /// The letter.
    pub letter: char,
    /// Its place on the label's layer.
    pub place: LabelPlace,
}

/// What a label's step types as it begins.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Keystroke {
    /// A letter, typed at its place.
    Letter(TypedLetter),
    /// No letter: the step only holds.
    Pause,
}

/// One step of the label: what it types and how many frames it holds, 1 to 65,535.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LabelStep {
    /// What it types as it begins.
    pub keystroke: Keystroke,
    /// Its hold, in frames.
    pub hold: num::NonZeroU16,
}

/// How the flight is made ready (`FlightIsMadeReady`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SceneSetup {
    /// The plane's angle.
    pub angle: Angle,
    /// The plane's zoom.
    pub zoom: Zoom,
    /// The plane's place.
    pub plane: WholePlace,
    /// The rushing stars' place.
    pub stars: WholePlace,
    /// The rushing stars' speed.
    pub speed: RushSpeed,
}

/// A sprite placed at the turn: its place, its drift and its wrap.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpritePlacement {
    /// Where it is placed.
    pub place: WholePlace,
    /// How far it moves sideways each frame.
    pub drift: Drift,
    /// Whether its x wraps across 0 to 511.
    pub wrap: SpriteWrap,
}

/// How the view turns away (`ViewTurnsAway`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SceneTurn {
    /// The plane's angle.
    pub angle: Angle,
    /// The plane's whole pixels.
    pub plane: WholePlace,
    /// Each sprite placed, by its kind.
    pub sprites: EnumMap<Drifter, Option<SpritePlacement>>,
}

/// The flight's scene, in our format: how it is made ready, how it turns, and the label in the
/// language the caller chose.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Scene {
    /// How it is made ready.
    pub setup: SceneSetup,
    /// How it turns.
    pub turn: SceneTurn,
    /// The label's steps, in order.
    pub label: Vec<LabelStep>,
}

/// Where the label's typing has got to: its current step and the frames its hold has left,
/// 0 to 65,535.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LabelProgress {
    /// The index of the current step, 0 the first.
    pub step: usize,
    /// The frames its hold has left.
    pub hold_left: u16,
}

/// Where the flight has got to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FlightStage {
    /// The first frame: the flight is yet to be made ready.
    Starting,
    /// In the dark, while the music is still queued.
    WaitingForMusic,
    /// The gunship rushes at the camera.
    RushingIn,
    /// The gunship flies away toward the station.
    FlyingAway,
    /// The station's view, the label typed, with where its typing has got to.
    Label(LabelProgress),
    /// The picture fades to black.
    FadingOut,
    /// The flight is over: the next frame starts the new game.
    Done,
}

/// How bright the screen is, 0 (black) to 15 (full).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub struct BrightnessLevel(u8);

impl BrightnessLevel {
    /// Black.
    pub const BLACK: Self = Self(0);
    /// Full brightness.
    pub const FULL: Self = Self(MAX_LEVEL);

    /// The level, if it is 0 to 15.
    #[must_use]
    pub const fn new(level: u8) -> Option<Self> {
        if level <= MAX_LEVEL {
            Some(Self(level))
        } else {
            None
        }
    }

    /// Its level.
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }

    /// One level darker, stopping at black.
    #[must_use]
    pub const fn darker(self) -> Self {
        Self(self.0.saturating_sub(1))
    }
}

impl TryFrom<u8> for BrightnessLevel {
    type Error = &'static str;

    fn try_from(level: u8) -> Result<Self, Self::Error> {
        Self::new(level).ok_or("brightness level must be 0 to 15")
    }
}

impl From<BrightnessLevel> for u8 {
    fn from(level: BrightnessLevel) -> Self {
        level.get()
    }
}

/// Whether the screen shows its picture at all.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Visibility {
    /// It shows nothing, at any level.
    Blanked,
    /// It shows the picture at its level.
    Shown,
}

/// The screen's brightness: its level, and whether it is blanked.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Brightness {
    /// How bright.
    pub level: BrightnessLevel,
    /// Whether it shows anything.
    pub visibility: Visibility,
}

impl Brightness {
    /// Dark: blanked at level 0.
    pub const DARK: Self = Self {
        level: BrightnessLevel::BLACK,
        visibility: Visibility::Blanked,
    };
    /// The picture shown at full brightness.
    pub const FULL: Self = Self {
        level: BrightnessLevel::FULL,
        visibility: Visibility::Shown,
    };
}

/// A number of frames of the fade, 0 to 32,768: its count, or its rhythm.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u16", into = "u16")]
pub struct FadeFrames(u16);

impl FadeFrames {
    /// No frames.
    pub const ZERO: Self = Self(0);
    /// One frame.
    pub const ONE: Self = Self(1);

    /// The frames, if they are 0 to 32,768.
    #[must_use]
    pub const fn new(frames: u16) -> Option<Self> {
        if frames <= MAX_FADE_FRAMES {
            Some(Self(frames))
        } else {
            None
        }
    }

    /// Its frames.
    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }

    /// One frame fewer, or none if none is left.
    #[must_use]
    pub const fn counted_down(self) -> Option<Self> {
        match self.0.checked_sub(1) {
            Some(left) => Some(Self(left)),
            None => None,
        }
    }
}

impl TryFrom<u16> for FadeFrames {
    type Error = &'static str;

    fn try_from(frames: u16) -> Result<Self, Self::Error> {
        Self::new(frames).ok_or("fade frames must be 0 to 32,768")
    }
}

impl From<FadeFrames> for u16 {
    fn from(frames: FadeFrames) -> Self {
        frames.get()
    }
}

/// How many frames audio waits before it carries out a music request, 0 to 65,535.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TuneDelay(pub u16);

/// What the flight asks audio to do with its tune (`MusicRequest`), each request with its delay.
/// Audio carries the requests out in the order the flight hands them over.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MusicRequest {
    /// Load the Ceres flight tune, after this delay.
    LoadCeresFlightTune(TuneDelay),
    /// Play the Ceres flight tune, after this delay.
    PlayCeresFlightTune(TuneDelay),
}

/// Everything the flight remembers from one frame to the next.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    /// Where the scene has got to, with the label's step while it is typed.
    pub stage: FlightStage,
    /// The turning plane that shows the gunship.
    pub plane: Plane,
    /// The sprites placed, with their motion.
    pub sprites: FlightSprites,
    /// The white flash on the screen's backdrop, each colour 0 to 31.
    pub flash: Colour,
    /// The screen's brightness.
    pub brightness: Brightness,
    /// The frames before the picture next darkens.
    pub fade_count: FadeFrames,
}
