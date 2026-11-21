//! Ceres haze state shared by the room, display preparation and save system.

/// Number of bands, ordered from the dim end to the bright end.
pub const HAZE_BAND_COUNT: usize = 16;
const MAX_LEVEL: u8 = 15;
const SETTLE_PEAK: u8 = 16;

/// Spec: `HazeBrightens`. The single colour used for every band.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HazeColour {
    /// Red haze.
    Red,
    /// Green haze.
    Green,
    /// Blue haze.
    Blue,
}

/// Spec: `HazeBrightens`, `HazeSettles`. Next bright-edge level, or 16 when the fade is due to settle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "u8")]
pub struct HazePeak(u8);

impl HazePeak {
    /// Builds a peak from 0 through 16 inclusive.
    #[must_use]
    pub const fn new(peak: u8) -> Option<Self> {
        if peak <= SETTLE_PEAK {
            Some(Self(peak))
        } else {
            None
        }
    }

    /// The saved peak, including the settle marker.
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }

    /// The level to prepare at the bright edge, absent when it is time to settle instead.
    #[must_use]
    pub const fn level_to_prepare(self) -> Option<HazeLevel> {
        HazeLevel::new(self.0)
    }
}

impl TryFrom<u8> for HazePeak {
    type Error = &'static str;

    fn try_from(peak: u8) -> Result<Self, Self::Error> {
        Self::new(peak).ok_or("haze peak must be in 0..=16")
    }
}

/// Spec: `HazeBrightens`. A single band's brightness, from 0 through 15 inclusive.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "u8")]
pub struct HazeLevel(u8);

impl HazeLevel {
    /// Builds a valid band level.
    #[must_use]
    pub const fn new(level: u8) -> Option<Self> {
        if level <= MAX_LEVEL {
            Some(Self(level))
        } else {
            None
        }
    }

    /// The band's brightness level.
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }

    /// Spec: `HazeBrightens`. The adjacent level toward the dim end, stopping at zero.
    #[must_use]
    pub const fn dimmed(self) -> Self {
        Self(self.0.saturating_sub(1))
    }

    /// Spec: `HazeBrightens`. One peak advance after preparing this level at the bright end.
    #[must_use]
    pub const fn following_peak(self) -> HazePeak {
        HazePeak(self.0.saturating_add(1))
    }
}

impl TryFrom<u8> for HazeLevel {
    type Error = &'static str;

    fn try_from(level: u8) -> Result<Self, Self::Error> {
        Self::new(level).ok_or("haze band level must be in 0..=15")
    }
}

/// Spec: `HazeSettles`, `HazeHolds`. The visible band levels, dim end first.
/// These are saved independently of the fade, even if they are not a regular gradient.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct HazeBands(pub [HazeLevel; HAZE_BAND_COUNT]);

/// Spec: `HazeBrightens`, `HazeSettles`, `HazeHolds`. Where the haze is in its fade.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum HazeFade {
    /// The next ordinary opportunity prepares the indicated peak or settles at the marker.
    Brightening(HazePeak),
    /// Ordinary opportunities retain the visible gradient.
    Steady,
}

/// Complete carried haze state. Room code supplies the initial or restored values;
/// the fade does not imply a particular set of bands on restore.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HazeState {
    /// Selected colour for the entire gradient.
    pub colour: HazeColour,
    /// Fade progress for the next ordinary opportunity.
    pub fade: HazeFade,
    /// Current visible band levels.
    pub bands: HazeBands,
}
