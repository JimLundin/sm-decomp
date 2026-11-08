//! The five supplies a tank raises, and a value kept for each.

/// A supply she holds and a tank raises the capacity of.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Supply {
    /// Energy, raised by an energy tank.
    Energy,
    /// Reserve energy, raised by a reserve tank.
    Reserve,
    /// Missiles, raised by a missile tank.
    Missiles,
    /// Super missiles, raised by a super missile tank.
    SuperMissiles,
    /// Power bombs, raised by a power bomb tank.
    PowerBombs,
}

impl Supply {
    /// Every supply.
    pub const ALL: &[Self] = &[
        Self::Energy,
        Self::Reserve,
        Self::Missiles,
        Self::SuperMissiles,
        Self::PowerBombs,
    ];
}

/// One value for each supply: a capacity, what she holds, or a divisor.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PerSupply<T> {
    /// For energy.
    pub energy: T,
    /// For reserve energy.
    pub reserve: T,
    /// For missiles.
    pub missiles: T,
    /// For super missiles.
    pub super_missiles: T,
    /// For power bombs.
    pub power_bombs: T,
}

impl<T: Copy> PerSupply<T> {
    /// The value for `supply`.
    #[must_use]
    pub const fn get(&self, supply: Supply) -> T {
        match supply {
            Supply::Energy => self.energy,
            Supply::Reserve => self.reserve,
            Supply::Missiles => self.missiles,
            Supply::SuperMissiles => self.super_missiles,
            Supply::PowerBombs => self.power_bombs,
        }
    }

    /// Sets the value for `supply`.
    pub const fn set(&mut self, supply: Supply, value: T) {
        match supply {
            Supply::Energy => self.energy = value,
            Supply::Reserve => self.reserve = value,
            Supply::Missiles => self.missiles = value,
            Supply::SuperMissiles => self.super_missiles = value,
            Supply::PowerBombs => self.power_bombs = value,
        }
    }
}
