//! Where things sit on the HUD (§1.2), as places on its grid: column across, row down.

use clean_types::{Look, by_member};

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

/// Where a kind of ammunition's count sits: its digits' cells, the highest digit first.
pub struct AmmoDigits {
    /// The ammunition.
    pub ammo: Ammo,
    /// Its digits' cells: as many as its count has digits.
    pub digits: &'static [Place],
}

/// Each ammunition's digits, in row 2: the missiles' in columns 10 to 12, the super missiles' in
/// 14 and 15, the power bombs' in 17 and 18.
pub const AMMO_DIGITS: [AmmoDigits; 3] = [
    AmmoDigits {
        ammo: Ammo::Missiles,
        digits: &[at(10, 2), at(11, 2), at(12, 2)],
    },
    AmmoDigits {
        ammo: Ammo::SuperMissiles,
        digits: &[at(14, 2), at(15, 2)],
    },
    AmmoDigits {
        ammo: Ammo::PowerBombs,
        digits: &[at(17, 2), at(18, 2)],
    },
];

/// The reserve sign's top-left cell: it fills rows 0 to 2, columns 8 and 9.
pub const RESERVE_SIGN: Place = at(8, 0);

/// Where an item's icon sits: its top-left cell, in row 0; the icon fills rows 0 and 1.
pub struct IconPlace {
    /// The item.
    pub item: SelectableItem,
    /// Its icon's top-left cell.
    pub top_left: Place,
}

/// Each item's icon: the missiles' in columns 10 to 12, the super missiles' in 14 and 15, the
/// power bombs' in 17 and 18, the grapple beam's in 20 and 21, the x-ray scope's in 23 and 24.
pub const ICONS: [IconPlace; 5] = [
    IconPlace {
        item: SelectableItem::Ammo(Ammo::Missiles),
        top_left: at(10, 0),
    },
    IconPlace {
        item: SelectableItem::Ammo(Ammo::SuperMissiles),
        top_left: at(14, 0),
    },
    IconPlace {
        item: SelectableItem::Ammo(Ammo::PowerBombs),
        top_left: at(17, 0),
    },
    IconPlace {
        item: SelectableItem::GrappleBeam,
        top_left: at(20, 0),
    },
    IconPlace {
        item: SelectableItem::XRayScope,
        top_left: at(23, 0),
    },
];

/// `item`'s icon's top-left cell.
pub fn icon_top_left(item: SelectableItem) -> Place {
    by_member(ICONS, |place| place.item)[item].top_left
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

#[cfg(test)]
mod tests {
    use super::{AMMO_DIGITS, ICONS};
    use crate::item::{Ammo, SelectableItem};
    use crate::tests::names_each_member_once;

    #[test]
    fn the_ammo_digits_name_each_ammo_once() {
        assert!(names_each_member_once::<Ammo, _>(&AMMO_DIGITS, |row| row.ammo));
    }

    #[test]
    fn the_icons_name_each_item_once() {
        assert!(names_each_member_once::<SelectableItem, _>(&ICONS, |row| {
            row.item
        }));
    }
}
