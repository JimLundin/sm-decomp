//! The screen's brightness, which a fade changes and the renderer shows.

use serde::{Deserialize, Serialize};

/// The brightest level: the picture fully up.
const FULLEST: u8 = 15;

/// How bright the picture is, from 0 (black) to 15 (fully up).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub struct Level(u8);

impl Level {
    /// Black: the darkest level.
    pub const BLACK: Self = Self(0);
    /// Fully up: the brightest level.
    pub const FULL: Self = Self(FULLEST);

    /// The level of this number, if it is one (0 to 15).
    #[must_use]
    pub const fn new(level: u8) -> Option<Self> {
        if level <= FULLEST {
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

    /// The next level up, or none from the brightest.
    #[must_use]
    pub fn brighter(self) -> Option<Self> {
        self.0.checked_add(1).and_then(Self::new)
    }
}

impl TryFrom<u8> for Level {
    type Error = &'static str;

    fn try_from(level: u8) -> Result<Self, Self::Error> {
        Self::new(level).ok_or("brightness level must be 0 to 15")
    }
}

impl From<Level> for u8 {
    fn from(level: Level) -> Self {
        level.get()
    }
}

/// Whether the screen shows its picture at all.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Blanking {
    /// The screen shows nothing, whatever its level.
    Blanked,
    /// The screen shows the picture at its level.
    Shown,
}

/// The screen's brightness: its level, and whether it is blanked.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Brightness {
    /// How bright the picture is.
    pub level: Level,
    /// Whether the picture is shown.
    pub blanking: Blanking,
}

impl Brightness {
    /// The picture shown fully up.
    pub const FULLY_UP: Self = Self::shown(Level::FULL);

    /// The picture shown at `level`.
    #[must_use]
    pub const fn shown(level: Level) -> Self {
        Self {
            level,
            blanking: Blanking::Shown,
        }
    }
}
