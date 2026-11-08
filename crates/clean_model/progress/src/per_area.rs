//! A value kept for each area of the planet.

use clean_types::Area;

/// One value for each area: the bosses defeated there, its map, its stations saved at.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PerArea<T> {
    /// For Brinstar.
    pub brinstar: T,
    /// For Ceres.
    pub ceres: T,
    /// For Crateria.
    pub crateria: T,
    /// For Maridia.
    pub maridia: T,
    /// For Norfair.
    pub norfair: T,
    /// For Tourian.
    pub tourian: T,
    /// For the wrecked ship.
    pub wrecked_ship: T,
}

impl<T> PerArea<T> {
    /// The value for `area`.
    #[must_use]
    pub const fn get(&self, area: Area) -> &T {
        match area {
            Area::Brinstar => &self.brinstar,
            Area::Ceres => &self.ceres,
            Area::Crateria => &self.crateria,
            Area::Maridia => &self.maridia,
            Area::Norfair => &self.norfair,
            Area::Tourian => &self.tourian,
            Area::WreckedShip => &self.wrecked_ship,
        }
    }

    /// The value for `area`, to change.
    pub const fn get_mut(&mut self, area: Area) -> &mut T {
        match area {
            Area::Brinstar => &mut self.brinstar,
            Area::Ceres => &mut self.ceres,
            Area::Crateria => &mut self.crateria,
            Area::Maridia => &mut self.maridia,
            Area::Norfair => &mut self.norfair,
            Area::Tourian => &mut self.tourian,
            Area::WreckedShip => &mut self.wrecked_ship,
        }
    }
}
