//! The five supplies she holds and a tank raises, and a value kept for each.

use crate::table::{Key, Table};

/// A supply she holds, whose capacity a tank raises.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Supply {
    /// Energy, raised by an energy tank.
    Energy,
    /// The reserve: energy kept in reserve tanks, raised by a reserve tank.
    Reserve,
    /// Missiles, raised by a missile tank.
    Missiles,
    /// Super missiles, raised by a super missile tank.
    SuperMissiles,
    /// Power bombs, raised by a power bomb tank.
    PowerBombs,
}

impl Supply {
    /// Every supply, in the order listed.
    pub const ALL: &[Self] = &[
        Self::Energy,
        Self::Reserve,
        Self::Missiles,
        Self::SuperMissiles,
        Self::PowerBombs,
    ];
}

/// One value for each supply: a capacity, what she holds, or a divisor.
pub type PerSupply<V> = Table<Supply, V, { Supply::ALL.len() }>;

impl Key<{ Self::ALL.len() }> for Supply {
    fn pick<T>(self, row: [T; Self::ALL.len()]) -> T {
        let [energy, reserve, missiles, super_missiles, power_bombs] = row;
        match self {
            Self::Energy => energy,
            Self::Reserve => reserve,
            Self::Missiles => missiles,
            Self::SuperMissiles => super_missiles,
            Self::PowerBombs => power_bombs,
        }
    }
}
