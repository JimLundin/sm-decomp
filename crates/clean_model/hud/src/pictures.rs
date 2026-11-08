//! Where things sit on the HUD and the pictures they show (§1.2, §1.4): data, read by the
//! status bar's and the mini-map's rules.

use clean_types::Item;

use crate::grid::{ColourSet, Flip, GridPos, Picture, Tile};
use crate::inputs::Ammo;

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

/// The map's blank picture, `01F` (§1.4).
pub const MAP_BLANK: Picture = Picture::new(0x01F);

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

/// The energy tanks' tiles in the order they fill: row 1 columns 1 to 7, then row 0 columns 1 to
/// 7 (§1.2). There are 14, so at most 14 tanks show.
pub const TANKS: [GridPos; 14] = [
    GridPos::at(1, 1),
    GridPos::at(1, 2),
    GridPos::at(1, 3),
    GridPos::at(1, 4),
    GridPos::at(1, 5),
    GridPos::at(1, 6),
    GridPos::at(1, 7),
    GridPos::at(0, 1),
    GridPos::at(0, 2),
    GridPos::at(0, 3),
    GridPos::at(0, 4),
    GridPos::at(0, 5),
    GridPos::at(0, 6),
    GridPos::at(0, 7),
];

/// The energy's digits: row 2, columns 6 and 7 (§1.2).
pub const ENERGY_DIGITS: [GridPos; 2] = [GridPos::at(2, 6), GridPos::at(2, 7)];

/// The missiles' digits: row 2, columns 10 to 12 (§1.2).
const MISSILE_DIGITS: [GridPos; 3] = [GridPos::at(2, 10), GridPos::at(2, 11), GridPos::at(2, 12)];
/// The super missiles' digits: row 2, columns 14 and 15 (§1.2).
const SUPER_MISSILE_DIGITS: [GridPos; 2] = [GridPos::at(2, 14), GridPos::at(2, 15)];
/// The power bombs' digits: row 2, columns 17 and 18 (§1.2).
const POWER_BOMB_DIGITS: [GridPos; 2] = [GridPos::at(2, 17), GridPos::at(2, 18)];

/// The tiles of one ammunition's digits, the highest first (§1.2).
pub const fn ammo_digits(ammo: Ammo) -> &'static [GridPos] {
    match ammo {
        Ammo::Missiles => &MISSILE_DIGITS,
        Ammo::SuperMissiles => &SUPER_MISSILE_DIGITS,
        Ammo::PowerBombs => &POWER_BOMB_DIGITS,
    }
}

/// A picture in its place, with its flips: one tile of an icon or of the reserve sign, whose
/// colours its rule chooses.
#[derive(Clone, Copy)]
pub struct Placed {
    /// Its tile.
    pub pos: GridPos,
    /// Its picture.
    pub picture: Picture,
    /// Its flips.
    pub flip: Flip,
}

/// A tile of an icon or the sign: `picture` at row `row`, column `column`, flipped `flip`.
const fn placed(row: u8, column: u8, picture: u16, flip: Flip) -> Placed {
    Placed {
        pos: GridPos::at(row, column),
        picture: Picture::new(picture),
        flip,
    }
}

/// The reserve sign (it reads AUTO), rows 0 to 2, columns 8 and 9; the bottom row flipped top to
/// bottom (§1.2, §1.4).
pub const RESERVE_SIGN: [Placed; 6] = [
    placed(0, 8, 0x033, Flip::None),
    placed(0, 9, 0x046, Flip::None),
    placed(1, 8, 0x047, Flip::None),
    placed(1, 9, 0x048, Flip::None),
    placed(2, 8, 0x033, Flip::Vertical),
    placed(2, 9, 0x046, Flip::Vertical),
];
/// The reserve sign's colours with energy in the tanks (§1.4: lit).
pub const SIGN_LIT: ColourSet = ColourSet::named(7);
/// The reserve sign's colours with the tanks empty (§1.4: dim).
pub const SIGN_DIM: ColourSet = COLOURS_3;

/// The missiles' icon, rows 0 and 1, columns 10 to 12; the third column flipped (§1.2, §1.4).
const MISSILES_ICON: [Placed; 6] = [
    placed(0, 10, 0x04B, Flip::None),
    placed(0, 11, 0x049, Flip::None),
    placed(0, 12, 0x04B, Flip::Horizontal),
    placed(1, 10, 0x04C, Flip::None),
    placed(1, 11, 0x04A, Flip::None),
    placed(1, 12, 0x04C, Flip::Horizontal),
];

/// A two-column icon at `column`: `top` over `bottom`, the second column flipped left to right
/// (§1.4).
const fn two_column_icon(column: u8, top: u16, bottom: u16) -> [Placed; 4] {
    let second = column.saturating_add(1);
    [
        placed(0, column, top, Flip::None),
        placed(0, second, top, Flip::Horizontal),
        placed(1, column, bottom, Flip::None),
        placed(1, second, bottom, Flip::Horizontal),
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
pub const fn icon(item: Item) -> &'static [Placed] {
    match item {
        Item::Missiles => &MISSILES_ICON,
        Item::SuperMissiles => &SUPER_MISSILES_ICON,
        Item::PowerBombs => &POWER_BOMBS_ICON,
        Item::GrappleBeam => &GRAPPLE_BEAM_ICON,
        Item::XRayScope => &X_RAY_SCOPE_ICON,
    }
}

/// The mini-map window's middle tile, her cell: row 1, column 28 (§1.2). The window spans rows 0
/// to 2 and columns 26 to 30, `WINDOW_REACH` around it.
pub const HER_WINDOW_TILE: GridPos = GridPos::at(1, 28);

/// How far the window reaches from her cell: 2 columns each way and 1 row (§1.2, the 5 × 3
/// window).
pub const WINDOW_REACH: WindowReach = WindowReach {
    columns: 2,
    rows: 1,
};

/// How far the mini-map window reaches from her cell, each way.
#[derive(Clone, Copy)]
pub struct WindowReach {
    /// Columns to each side.
    pub columns: i32,
    /// Rows above and below.
    pub rows: i32,
}

impl WindowReach {
    /// Every offset of the window from her cell, `(columns, rows)`, top row first, each row left
    /// to right.
    pub fn offsets(self) -> impl Iterator<Item = (i32, i32)> {
        (self.rows.saturating_neg()..=self.rows)
            .flat_map(move |j| (self.columns.saturating_neg()..=self.columns).map(move |k| (k, j)))
    }
}
