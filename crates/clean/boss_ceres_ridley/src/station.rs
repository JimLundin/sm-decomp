//! The station around the doors, the shaft and Ridley's room: its stage, the doors' two colour
//! sets and the palettes they go to, and the shakes the doors start.

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

/// Which palette the doors' colour set goes to when a room places a door (`TheDoorsColours`).
/// Listed alphabetically; the order means nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DoorPalette {
    /// The palette the doors share.
    Doors,
    /// A palette of the door's own: the door of Ridley's room while he is in it, whose colours
    /// hold the doors' palette.
    Own,
}

/// The doors' colours a room's start hands the passage's fade (`TheDoorsColours`): which set,
/// in which palette.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DoorColours {
    /// The colour set the palette takes.
    pub set: ColourSet,
    /// The palette that takes it.
    pub palette: DoorPalette,
}

/// How strong and how long a shake is (`EscapeShakes`, `Flies`). The screen's shaking moves the
/// view and counts its frames down.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shake {
    /// How far it moves the view, in pixels.
    pub strength: u16,
    /// How many frames it lasts.
    pub length: u16,
}

/// What a shake moves (`EscapeShakes`, `Flies`). Listed alphabetically; the order means nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShakeLayers {
    /// The second background and the enemies, not the first background: the shakes the door
    /// of Ridley's room starts, and the one Ridley starts as he meets his room's left side.
    Back,
    /// Both backgrounds and the enemies: the shakes every other door starts.
    Whole,
}

/// The way a shake moves what it moves. Listed alphabetically; the order means nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShakeMotion {
    /// Diagonally: the doors' shakes during the escape (`EscapeShakes`).
    Diagonal,
    /// Sideways: Ridley's as he meets his room's left side fast (`Flies`).
    Sideways,
}

/// A shake a run starts, a door's or Ridley's: how strong and long it is, what it moves and
/// which way.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShakeStart {
    /// How strong and long it is.
    pub shake: Shake,
    /// What it moves.
    pub layers: ShakeLayers,
    /// Which way it moves them.
    pub motion: ShakeMotion,
}
