//! What a cell of the HUD shows (§1.4): a picture with its flips (the shared `Look`), in one of
//! the HUD's eight colour sets; and the colour sets by the role the spec gives each.

use clean_types::Look;

use crate::grid::Grid;

/// The HUD's colour sets (§1.4: eight palettes).
const COLOUR_SETS: u8 = 8;

/// One of the HUD's eight colour sets, 0 to 7 (§1.4).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ColourSet(u8);

impl ColourSet {
    /// Colour set `number`, or `None` past the eight.
    #[must_use]
    pub const fn new(number: u8) -> Option<Self> {
        if number < COLOUR_SETS {
            Some(Self(number))
        } else {
            None
        }
    }

    /// Its number, 0 to 7.
    #[must_use]
    pub const fn number(self) -> u8 {
        self.0
    }
}

/// What one of the HUD's cells shows: a picture with its flips, in a colour set.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Tile {
    /// The picture and its flips.
    pub look: Look,
    /// Its colour set.
    pub colours: ColourSet,
}

impl Tile {
    /// The cell that shows `look`, its flips kept, in `colours`.
    #[must_use]
    pub const fn new(look: Look, colours: ColourSet) -> Self {
        Self { look, colours }
    }

    /// The same picture and flips in other colours.
    #[must_use]
    pub const fn in_colours(self, colours: ColourSet) -> Self {
        Self::new(self.look, colours)
    }
}

/// The HUD's cells: 32 across by 3 down, the status bar and the mini-map's window (Notation).
pub type HudGrid = Grid<Tile, 32, 3>;

// The colour sets §1.4 names by their roles, each one of the eight.

/// The blank's colours (§1.4).
pub const BLANK_COLOURS: ColourSet = ColourSet(3);
/// The digits' colours (§1.4).
pub const DIGIT_COLOURS: ColourSet = ColourSet(3);
/// A full energy tank's colours (§1.4: full).
pub const FULL_TANK_COLOURS: ColourSet = ColourSet(2);
/// An empty energy tank's colours (§1.4: empty).
pub const EMPTY_TANK_COLOURS: ColourSet = ColourSet(5);
/// The reserve sign's colours with energy in the reserve tanks (§1.4: lit).
pub const SIGN_LIT: ColourSet = ColourSet(7);
/// The reserve sign's colours with the reserve tanks empty (§1.4: dim).
pub const SIGN_DIM: ColourSet = ColourSet(3);
/// An icon's colours while its item isn't the one lit (§1.4: plain).
pub const PLAIN: ColourSet = ColourSet(5);
/// An icon's colours while its item is selected, or its flash is on (§1.4: highlighted).
pub const HIGHLIGHTED: ColourSet = ColourSet(4);
/// A mini-map cell she has explored (§1.4: explored).
pub const EXPLORED: ColourSet = ColourSet(2);
/// A mini-map cell a collected map charts, and the map's blank (§1.4: charted).
pub const CHARTED: ColourSet = ColourSet(3);
/// Her mini-map cell while it blinks (§1.4: her blink).
pub const BLINK: ColourSet = ColourSet(7);
