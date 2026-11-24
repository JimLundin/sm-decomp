//! The power-on sequence's State: its stage, the screen and the count of finished frames.
//!
//! The game carries it from one frame end to the next, from the console's power-on until the
//! title begins, and may save and restore it.

use serde::{Deserialize, Serialize};

/// Spec: `LogoDims`. The darkest level, 0.
const DARK_LEVEL: u8 = 0;
/// Spec: `LogoBrightens`. The fullest level, 15.
const FULL_LEVEL: u8 = 15;

/// A brightness level of the screen, from 0 (dark) to 15 (full).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub struct Level(u8);

impl Level {
    /// Dark: level 0.
    pub const DARK: Self = Self(DARK_LEVEL);
    /// Full: level 15.
    pub const FULL: Self = Self(FULL_LEVEL);

    /// The level of this number, if it is one (0 to 15).
    #[must_use]
    pub const fn new(level: u8) -> Option<Self> {
        if level <= FULL_LEVEL {
            Some(Self(level))
        } else {
            None
        }
    }

    /// Its number, 0 to 15.
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }
}

impl TryFrom<u8> for Level {
    type Error = &'static str;

    fn try_from(level: u8) -> Result<Self, Self::Error> {
        Self::new(level).ok_or("a brightness level must be 0 to 15")
    }
}

impl From<Level> for u8 {
    fn from(level: Level) -> Self {
        level.get()
    }
}

/// The screen at a frame end: blank (nothing shows, whatever its level) or lit at its level.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScreenLight {
    /// Nothing shows; the level is kept all the same.
    Blank(Level),
    /// The picture shows at this level.
    Lit(Level),
}

impl ScreenLight {
    /// Its level, blank or lit.
    #[must_use]
    pub const fn level(self) -> Level {
        match self {
            Self::Blank(level) | Self::Lit(level) => level,
        }
    }
}

/// Whole finished frames of the logo's hold still to show (spec: `LogoHeld`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HoldFrames(pub u16);

/// The count of finished frames since power-on, or since the game began to settle: 16 bits,
/// wrapping from 65,535 to 0 (spec: `FramesCounted`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FinishedFrames(pub u16);

impl FinishedFrames {
    /// No frame finished yet.
    pub const ZERO: Self = Self(0);
}

/// Where the power-on sequence stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PowerOnStage {
    /// Black, before the logo's first frame finishes.
    BeforeLogo,
    /// The logo brightening.
    Brightening,
    /// The logo held at full, with the frames of the hold left.
    Held(HoldFrames),
    /// The logo dimming.
    Dimming,
    /// Black while the sound driver loads.
    SoundLoading,
    /// Black while the game settles, before the title.
    Settling,
}

/// The power-on sequence's complete State. A caller may set any stage, screen and count, read
/// them back, carry them forward and restore them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PowerOnState {
    /// Where the sequence stands.
    pub stage: PowerOnStage,
    /// The screen at the last frame end.
    pub screen: ScreenLight,
    /// The count of finished frames.
    pub frames: FinishedFrames,
}

impl PowerOnState {
    /// The State at the console's power-on: before the logo, blank at full level, no frame
    /// finished.
    pub const AT_POWER_ON: Self = Self {
        stage: PowerOnStage::BeforeLogo,
        screen: ScreenLight::Blank(Level::FULL),
        frames: FinishedFrames::ZERO,
    };
}
