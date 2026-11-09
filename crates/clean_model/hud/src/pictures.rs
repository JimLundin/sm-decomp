//! The HUD's tiles and where things sit on it (§1.2, §1.4): a tile is a picture in one of the
//! HUD's colour sets, perhaps flipped, and the HUD is a grid of them, 32 across by 3 down. The
//! rest is data the status bar's and the mini-map's rules read.

use clean_types::Pair;

use crate::grid::{Grid, Pos};
use crate::item::SelectableItem;

/// The HUD's colour sets (§1.4: eight palettes).
const COLOUR_SETS: u8 = 8;

/// A picture of the HUD's graphics, by the number the importer gives it (§1.4).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Picture(u16);

impl Picture {
    /// The picture numbered `number`.
    #[must_use]
    pub const fn new(number: u16) -> Self {
        Self(number)
    }

    /// Its number.
    #[must_use]
    pub const fn number(self) -> u16 {
        self.0
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

    /// A colour set the spec names; the tables of §1.4 give only numbers below eight.
    const fn named(number: u8) -> Self {
        Self(number)
    }

    /// Its number, 0 to 7.
    #[must_use]
    pub const fn number(self) -> u8 {
        self.0
    }
}

/// How a picture is flipped.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Flip {
    /// As drawn.
    None,
    /// Flipped left to right.
    Horizontal,
    /// Flipped top to bottom.
    Vertical,
    /// Flipped both ways.
    Both,
}

impl Flip {
    /// Whether it is flipped left to right.
    #[must_use]
    pub const fn horizontal(self) -> bool {
        matches!(self, Self::Horizontal | Self::Both)
    }

    /// Whether it is flipped top to bottom.
    #[must_use]
    pub const fn vertical(self) -> bool {
        matches!(self, Self::Vertical | Self::Both)
    }
}

/// What one of the HUD's tiles shows: a picture in a colour set, perhaps flipped.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Tile {
    /// The picture.
    pub picture: Picture,
    /// Its colour set.
    pub colours: ColourSet,
    /// Its flips.
    pub flip: Flip,
}

impl Tile {
    /// A picture in a colour set, flipped as `flip` says.
    #[must_use]
    pub const fn new(picture: Picture, colours: ColourSet, flip: Flip) -> Self {
        Self {
            picture,
            colours,
            flip,
        }
    }

    /// The same picture and flips in other colours.
    #[must_use]
    pub const fn in_colours(self, colours: ColourSet) -> Self {
        Self { colours, ..self }
    }
}

impl Default for Tile {
    /// The blank tile, which every tile of a new HUD shows.
    fn default() -> Self {
        BLANK
    }
}

/// The HUD's tiles: 32 across by 3 down, the status bar and the mini-map's window (Notation).
pub type HudGrid = Grid<Tile, 32, 3>;
/// A tile's place on the HUD: column 0 to 31 across, row 0 to 2 down.
pub type HudPos = Pos<32, 3>;

/// The colours of the digits, the blank and the dim reserve sign.
const COLOURS_3: ColourSet = ColourSet::named(3);

/// An icon's colours when it isn't selected (§1.4: plain).
pub const PLAIN: ColourSet = ColourSet::named(5);
/// An icon's colours when it is selected, or the one-shot item flashes (§1.4: highlighted).
pub const HIGHLIGHTED: ColourSet = ColourSet::named(4);
/// A mini-map cell she has explored (§1.4).
pub const EXPLORED: ColourSet = ColourSet::named(2);
/// A mini-map cell the collected map charts, and the map's blank (§1.4).
pub const CHARTED: ColourSet = ColourSet::named(3);
/// Her mini-map cell while it blinks (§1.4).
pub const BLINK: ColourSet = ColourSet::named(7);

/// The blank tile: `00F` in colours 3, unflipped (§1.4).
pub const BLANK: Tile = Tile::new(Picture::new(0x00F), COLOURS_3, Flip::None);

/// The digits 0 to 9, in order (§1.4).
const DIGITS: [Picture; 10] = [
    Picture::new(0x009),
    Picture::new(0x000),
    Picture::new(0x001),
    Picture::new(0x002),
    Picture::new(0x003),
    Picture::new(0x004),
    Picture::new(0x005),
    Picture::new(0x006),
    Picture::new(0x007),
    Picture::new(0x008),
];

/// The tile that shows `digit` (0 to 9), in colours 3 (§1.4).
pub fn digit(digit: u16) -> Tile {
    let picture = DIGITS
        .get(usize::from(digit))
        .copied()
        .unwrap_or(BLANK.picture);
    Tile::new(picture, COLOURS_3, Flip::None)
}

/// A full energy tank (§1.4).
pub const FULL_TANK: Tile = Tile::new(Picture::new(0x031), ColourSet::named(2), Flip::None);
/// An empty energy tank (§1.4).
pub const EMPTY_TANK: Tile = Tile::new(Picture::new(0x030), ColourSet::named(5), Flip::None);

/// The energy tanks' tiles in the order they fill: columns 1 to 7 of row 1, then of row 0
/// (§1.2). There are 14, so at most 14 tanks show.
pub const TANKS: [HudPos; 14] = [
    HudPos::at(1, 1),
    HudPos::at(2, 1),
    HudPos::at(3, 1),
    HudPos::at(4, 1),
    HudPos::at(5, 1),
    HudPos::at(6, 1),
    HudPos::at(7, 1),
    HudPos::at(1, 0),
    HudPos::at(2, 0),
    HudPos::at(3, 0),
    HudPos::at(4, 0),
    HudPos::at(5, 0),
    HudPos::at(6, 0),
    HudPos::at(7, 0),
];

/// The energy's digits: columns 6 and 7 of row 2 (§1.2).
pub const ENERGY_DIGITS: [HudPos; 2] = [HudPos::at(6, 2), HudPos::at(7, 2)];

/// A picture in its place, with its flips: one tile of an icon or of the reserve sign, whose
/// colours its rule chooses.
#[derive(Clone, Copy)]
pub struct Placed {
    /// Its tile.
    pub pos: HudPos,
    /// Its picture.
    pub picture: Picture,
    /// Its flips.
    pub flip: Flip,
}

/// A tile of an icon or the sign: `picture` at column `x`, row `y`, flipped `flip`.
const fn placed(x: u8, y: u8, picture: u16, flip: Flip) -> Placed {
    Placed {
        pos: HudPos::at(x, y),
        picture: Picture::new(picture),
        flip,
    }
}

/// The reserve sign (it reads AUTO), columns 8 and 9 of rows 0 to 2; the bottom row flipped top
/// to bottom (§1.2, §1.4).
pub const RESERVE_SIGN: [Placed; 6] = [
    placed(8, 0, 0x033, Flip::None),
    placed(9, 0, 0x046, Flip::None),
    placed(8, 1, 0x047, Flip::None),
    placed(9, 1, 0x048, Flip::None),
    placed(8, 2, 0x033, Flip::Vertical),
    placed(9, 2, 0x046, Flip::Vertical),
];
/// The reserve sign's colours with energy in the tanks (§1.4: lit).
pub const SIGN_LIT: ColourSet = ColourSet::named(7);
/// The reserve sign's colours with the tanks empty (§1.4: dim).
pub const SIGN_DIM: ColourSet = COLOURS_3;

/// The missiles' icon, columns 10 to 12 of rows 0 and 1; the third column flipped (§1.2, §1.4).
const MISSILES_ICON: [Placed; 6] = [
    placed(10, 0, 0x04B, Flip::None),
    placed(11, 0, 0x049, Flip::None),
    placed(12, 0, 0x04B, Flip::Horizontal),
    placed(10, 1, 0x04C, Flip::None),
    placed(11, 1, 0x04A, Flip::None),
    placed(12, 1, 0x04C, Flip::Horizontal),
];

/// A two-column icon from column `x`: `top` over `bottom`, the second column flipped left to
/// right (§1.4).
const fn two_column_icon(x: u8, top: u16, bottom: u16) -> [Placed; 4] {
    let second = x.saturating_add(1);
    [
        placed(x, 0, top, Flip::None),
        placed(second, 0, top, Flip::Horizontal),
        placed(x, 1, bottom, Flip::None),
        placed(second, 1, bottom, Flip::Horizontal),
    ]
}

/// The super missiles' icon, columns 14 and 15 (§1.2, §1.4).
const SUPER_MISSILES_ICON: [Placed; 4] = two_column_icon(14, 0x034, 0x035);
/// The power bombs' icon, columns 17 and 18 (§1.2, §1.4).
const POWER_BOMBS_ICON: [Placed; 4] = two_column_icon(17, 0x036, 0x037);
/// The grapple beam's icon, columns 20 and 21 (§1.2, §1.4).
const GRAPPLE_BEAM_ICON: [Placed; 4] = two_column_icon(20, 0x038, 0x039);
/// The x-ray scope's icon, columns 23 and 24 (§1.2, §1.4).
const X_RAY_SCOPE_ICON: [Placed; 4] = two_column_icon(23, 0x03A, 0x03B);

/// An item's icon, its top-left tile first (§1.2, §1.4).
pub const fn icon(item: SelectableItem) -> &'static [Placed] {
    match item {
        SelectableItem::Missiles => &MISSILES_ICON,
        SelectableItem::SuperMissiles => &SUPER_MISSILES_ICON,
        SelectableItem::PowerBombs => &POWER_BOMBS_ICON,
        SelectableItem::GrappleBeam => &GRAPPLE_BEAM_ICON,
        SelectableItem::XRayScope => &X_RAY_SCOPE_ICON,
    }
}

/// The mini-map window's middle tile, her cell: column 28 of row 1 (§1.2).
pub const HER_TILE: HudPos = HudPos::at(28, 1);

/// How far the window reaches from her tile: 2 columns to each side and 1 row above and below,
/// so columns 26 to 30 of rows 0 to 2 (§1.2, the 5 × 3 window).
pub const WINDOW_REACH: Pair<i32> = Pair { x: 2, y: 1 };
