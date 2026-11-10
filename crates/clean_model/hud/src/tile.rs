//! What a cell of the HUD shows (§1.4): a picture, perhaps flipped, in one of the HUD's eight
//! colour sets; and the colour sets by the role the spec gives each.

use clean_types::Flip;

use crate::grid::Grid;

/// The HUD's colour sets (§1.4: eight palettes).
const COLOUR_SETS: u8 = 8;

/// A picture with its flips (§1.4): a number the importer gives it, the HUD's and her map's
/// numbered alike.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Picture {
    /// Its number.
    pub number: u16,
    /// How it is flipped.
    pub flip: Flip,
}

impl Picture {
    /// The picture numbered `number`, flipped as `flip` says.
    #[must_use]
    pub const fn new(number: u16, flip: Flip) -> Self {
        Self { number, flip }
    }

    /// This picture as a cell in `colours`, its flips kept.
    #[must_use]
    pub const fn in_colours(self, colours: ColourSet) -> Tile {
        Tile {
            picture: self,
            colours,
        }
    }
}

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

    /// A colour set §1.4 names by its role; it names only numbers below eight.
    const fn named(number: u8) -> Self {
        Self(number)
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
    pub picture: Picture,
    /// Its colour set.
    pub colours: ColourSet,
}

impl Tile {
    /// The same picture and flips in other colours.
    #[must_use]
    pub const fn in_colours(self, colours: ColourSet) -> Self {
        self.picture.in_colours(colours)
    }
}

/// The HUD's cells: 32 across by 3 down, the status bar and the mini-map's window (Notation).
pub type HudGrid = Grid<Tile, 32, 3>;

/// The blank's colours (§1.4).
pub const BLANK_COLOURS: ColourSet = ColourSet::named(3);
/// The digits' colours (§1.4).
pub const DIGIT_COLOURS: ColourSet = ColourSet::named(3);
/// A full energy tank's colours (§1.4: full).
pub const FULL_TANK_COLOURS: ColourSet = ColourSet::named(2);
/// An empty energy tank's colours (§1.4: empty).
pub const EMPTY_TANK_COLOURS: ColourSet = ColourSet::named(5);
/// The reserve sign's colours with energy in the reserve tanks (§1.4: lit).
pub const SIGN_LIT: ColourSet = ColourSet::named(7);
/// The reserve sign's colours with the reserve tanks empty (§1.4: dim).
pub const SIGN_DIM: ColourSet = ColourSet::named(3);
/// An icon's colours while its item isn't the one lit (§1.4: plain).
pub const PLAIN: ColourSet = ColourSet::named(5);
/// An icon's colours while its item is selected, or its flash is on (§1.4: highlighted).
pub const HIGHLIGHTED: ColourSet = ColourSet::named(4);
/// A mini-map cell she has explored (§1.4: explored).
pub const EXPLORED: ColourSet = ColourSet::named(2);
/// A mini-map cell a collected map charts, and the map's blank (§1.4: charted).
pub const CHARTED: ColourSet = ColourSet::named(3);
/// Her mini-map cell while it blinks (§1.4: her blink).
pub const BLINK: ColourSet = ColourSet::named(7);
