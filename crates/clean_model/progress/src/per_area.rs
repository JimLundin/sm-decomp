//! A value kept for each area of the planet.

use clean_types::Area;

use crate::table::{Key, Table};

/// One value for each area: the bosses defeated there, its map, its stations saved at.
pub type PerArea<V> = Table<Area, V, { Area::ALL.len() }>;

impl Key<{ Self::ALL.len() }> for Area {
    fn pick<T>(self, row: [T; Self::ALL.len()]) -> T {
        let [
            brinstar,
            ceres,
            crateria,
            maridia,
            norfair,
            tourian,
            wrecked_ship,
        ] = row;
        match self {
            Self::Brinstar => brinstar,
            Self::Ceres => ceres,
            Self::Crateria => crateria,
            Self::Maridia => maridia,
            Self::Norfair => norfair,
            Self::Tourian => tourian,
            Self::WreckedShip => wrecked_ship,
        }
    }
}
