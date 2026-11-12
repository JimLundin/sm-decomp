//! The station around the doors: its stage, the doors' two colour sets and the shakes they
//! start.

/// The station's stage, which progress and the escape supply. Listed alphabetically; the order
/// means nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    /// Before Ridley's getaway.
    Calm,
    /// From the escape's start to the end of Ceres.
    Escape,
    /// Ridley flees; the escape has not yet started.
    Getaway,
}

impl Stage {
    /// `ShaftWall`: whether Ridley has fled, from his getaway on.
    #[must_use]
    pub const fn ridley_has_fled(self) -> bool {
        match self {
            Self::Calm => false,
            Self::Escape | Self::Getaway => true,
        }
    }

    /// `TheDoorsColours`: the colour set the doors' palette takes when a room places a door:
    /// the calm set while calm, the escape set from the getaway on.
    #[must_use]
    pub const fn door_colours(self) -> ColourSet {
        match self {
            Self::Calm => ColourSet::Calm,
            Self::Escape | Self::Getaway => ColourSet::Escape,
        }
    }
}

/// Which of the doors' two colour sets their palette takes (`TheDoorsColours`). The 15 colours of
/// each are the doors' content; fading them in with the room is the palettes'.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColourSet {
    /// The doors' colours before Ridley's getaway.
    Calm,
    /// The doors' colours from the getaway on.
    Escape,
}

/// A shake of the whole view, both backgrounds and the enemies, moving diagonally
/// (`EscapeShakes`). The screen's shaking moves the view and counts its frames down.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shake {
    /// How far it moves the view, in pixels.
    pub strength: u16,
    /// How many frames it lasts.
    pub length: u16,
}
