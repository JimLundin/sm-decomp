//! Where things sit on the HUD (§1.2), as places on its grid: column across, row down.

use types::Look;

use crate::grid::{Place, at, plus};
use crate::item::{Ammo, SelectableItem};

/// The energy tanks' cells in the order they fill: row 1 columns 1 to 7, then row 0 columns 1 to
/// 7. There are 14, so at most 14 tanks show (spec: EnergyGauge).
pub const TANKS: [Place; 14] = [
    at(1, 1),
    at(2, 1),
    at(3, 1),
    at(4, 1),
    at(5, 1),
    at(6, 1),
    at(7, 1),
    at(1, 0),
    at(2, 0),
    at(3, 0),
    at(4, 0),
    at(5, 0),
    at(6, 0),
    at(7, 0),
];

/// The energy's digits: row 2, columns 6 and 7.
pub const ENERGY_DIGITS: [Place; 2] = [at(6, 2), at(7, 2)];

/// The missiles' digits: row 2, columns 10 to 12.
const MISSILES_DIGITS: [Place; 3] = [at(10, 2), at(11, 2), at(12, 2)];
/// The super missiles' digits: row 2, columns 14 and 15.
const SUPER_MISSILES_DIGITS: [Place; 2] = [at(14, 2), at(15, 2)];
/// The power bombs' digits: row 2, columns 17 and 18.
const POWER_BOMBS_DIGITS: [Place; 2] = [at(17, 2), at(18, 2)];

/// Where `ammo`'s count sits: its digits' cells, the highest digit first, as many as its count
/// has digits.
pub const fn ammo_digits(ammo: Ammo) -> &'static [Place] {
    match ammo {
        Ammo::Missiles => &MISSILES_DIGITS,
        Ammo::SuperMissiles => &SUPER_MISSILES_DIGITS,
        Ammo::PowerBombs => &POWER_BOMBS_DIGITS,
    }
}

/// The reserve sign's top-left cell: it fills rows 0 to 2, columns 8 and 9.
pub const RESERVE_SIGN: Place = at(8, 0);

/// Where `item`'s icon sits: its top-left cell, in row 0; the icon fills rows 0 and 1. The
/// missiles' fills columns 10 to 12, the super missiles' 14 and 15, the power bombs' 17 and 18,
/// the grapple beam's 20 and 21, the x-ray scope's 23 and 24.
pub const fn icon_top_left(item: SelectableItem) -> Place {
    match item {
        SelectableItem::Ammo(Ammo::Missiles) => at(10, 0),
        SelectableItem::Ammo(Ammo::SuperMissiles) => at(14, 0),
        SelectableItem::Ammo(Ammo::PowerBombs) => at(17, 0),
        SelectableItem::GrappleBeam => at(20, 0),
        SelectableItem::XRayScope => at(23, 0),
    }
}

/// The mini-map window's middle cell, her cell: column 28, row 1.
pub const HER_CELL: Place = at(28, 1);

/// How far the window reaches from her cell: 2 columns each way and 1 row, so columns 26 to 30,
/// rows 0 to 2.
pub const WINDOW_REACH: Place = at(2, 1);

/// Rows of pictures laid from `top_left`: each picture with its cell, the rows downward, each
/// left to right.
pub fn laid<'p>(
    top_left: Place,
    rows: impl IntoIterator<Item = &'p [Look]>,
) -> impl Iterator<Item = (Place, Look)> {
    (0..).zip(rows).flat_map(move |(down, row)| {
        (0..)
            .zip(row)
            .map(move |(across, &look)| (plus(top_left, at(across, down)), look))
    })
}
