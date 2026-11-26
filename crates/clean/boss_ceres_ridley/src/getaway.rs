//! `GetawayBackdrop`: the stopped run prepares the picture and places both walls while calm.

use types::{Colour, Pair, Px, Side};

use crate::door::{CeresDoor, DoorKind, Placement};
use crate::station::Stage;

const BODY_COLOURS: usize = 15;
const WALL_COLOURS: usize = 8;

/// Room content used when Ridley stops coasting.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GetawayContent {
    /// The left wall's centre in whole pixels.
    pub left_wall: Pair<Px>,
    /// The right wall's centre in whole pixels.
    pub right_wall: Pair<Px>,
    /// The getaway body's colours.
    pub body_colours: [Colour; BODY_COLOURS],
    /// The colours used for both wall palettes.
    pub wall_colours: [Colour; WALL_COLOURS],
}

/// Current colours for drawing, not the room passage's fade goals.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GetawayColours {
    /// Colours for the `BackgroundBody` palette.
    pub background_body: [Colour; BODY_COLOURS],
    /// Colours for the `BackgroundWalls` palette.
    pub background_walls: [Colour; WALL_COLOURS],
    /// Colours for the `SpriteWalls` palette.
    pub sprite_walls: [Colour; WALL_COLOURS],
}

impl GetawayColours {
    /// The transparent colour stays in the first slot; each selection starts after it.
    pub const FIRST_INDEX: usize = 1;
}

/// The stopped run's outputs for the actor framework, `TheDoorsColours` and drawing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GetawayBackdrop {
    /// The placed left wall, including the ordinary door colour selection for its placement.
    pub left_wall: Placement,
    /// The placed right wall, including the ordinary door colour selection for its placement.
    pub right_wall: Placement,
    /// The three current colour selections for drawing.
    pub colours: GetawayColours,
}

impl GetawayContent {
    /// `GetawayBackdrop`: place both walls without starting the getaway or changing the stage.
    pub(crate) const fn prepare(self) -> GetawayBackdrop {
        GetawayBackdrop {
            left_wall: CeresDoor::place(
                DoorKind::GetawayWall,
                Side::Left,
                self.left_wall,
                Stage::Calm,
            ),
            right_wall: CeresDoor::place(
                DoorKind::GetawayWall,
                Side::Right,
                self.right_wall,
                Stage::Calm,
            ),
            colours: GetawayColours {
                background_body: self.body_colours,
                background_walls: self.wall_colours,
                sprite_walls: self.wall_colours,
            },
        }
    }
}
