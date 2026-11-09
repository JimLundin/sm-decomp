//! What she has (spec §1.1, §4): her items and beams, hers and switched on, her capacities, what
//! she holds and her reserve mode; and what taking an item, a beam or a tank gives her.

use core::num::Wrapping;

use clean_types::{Beam, Equipment, Item, Key, Set, Table};

/// A count or a capacity. The spec's notation: counts and capacities are whole numbers of 16 bits
/// and wrap at 65536, so 65535 and 2 more is 1: its `+` is `Wrapping`'s.
pub type Count = Wrapping<u16>;

/// Items: those that are hers, or those switched on. Spec: TakingItems.
pub type Items = Set<Item, { Item::COUNT }>;

/// Beams: those that are hers, or those switched on. Spec: TakingBeams.
pub type Beams = Set<Beam, { Beam::COUNT }>;

/// One value for each beam.
pub type PerBeam<V> = Table<Beam, V, { Beam::COUNT }>;

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

impl Key<{ Self::ALL.len() }> for Supply {
    fn pick<T>(self, row: [T; Self::ALL.len()]) -> T {
        match (self, row) {
            (Self::Energy, [v, ..])
            | (Self::Reserve, [_, v, ..])
            | (Self::Missiles, [_, _, v, ..])
            | (Self::SuperMissiles, [_, _, _, v, ..])
            | (Self::PowerBombs, [_, _, _, _, v]) => v,
        }
    }
}

/// One value for each supply: a capacity, what she holds, or a divisor.
pub type PerSupply<V> = Table<Supply, V, { Supply::ALL.len() }>;

/// Whether the reserve refills her on its own.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ReserveMode {
    /// She has no reserve tank yet.
    #[default]
    NoTank,
    /// The reserve refills her on its own.
    Automatic,
    /// The reserve refills her only when she calls on it.
    Manual,
}

/// What she takes from an item's place.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pickup {
    /// An item.
    Item(Item),
    /// A beam.
    Beam(Beam),
    /// A tank, raising the capacity of `supply` by `amount`.
    Tank {
        /// The supply the tank raises.
        supply: Supply,
        /// How much it raises it by. The amount is the tank's own and comes in the report, not
        /// from `Rules`: the game's tanks carry 100 for energy and the reserve, 5 for the others.
        amount: Count,
    },
}

/// What she has.
///
/// Others switch her items and beams on and off too (the equipment screen, the rainbow beam),
/// and nothing here keeps those switched on within hers: a beam can be switched on without being
/// hers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Inventory {
    /// The items that are hers, switched on or not.
    pub items: Items,
    /// The items switched on, which decide what she can do and how she looks.
    pub items_switched_on: Items,
    /// The beams that are hers, switched on or not.
    pub beams: Beams,
    /// The beams switched on, which decide the beam she shoots.
    pub beams_switched_on: Beams,
    /// The most of each supply she can hold.
    pub capacity: PerSupply<Count>,
    /// How much of each supply she holds now.
    pub held: PerSupply<Count>,
    /// Whether the reserve refills her on its own.
    pub reserve_mode: ReserveMode,
}

/// A new file's: 99 energy of a capacity of 99, every other capacity and count at 0, no reserve
/// tank, and no items or beams, hers or switched on.
impl Default for Inventory {
    fn default() -> Self {
        Self {
            items: Items::default(),
            items_switched_on: Items::default(),
            beams: Beams::default(),
            beams_switched_on: Beams::default(),
            capacity: NEW_FILE_SUPPLIES,
            held: NEW_FILE_SUPPLIES,
            reserve_mode: ReserveMode::default(),
        }
    }
}

impl Inventory {
    /// What her motion reads: the items switched on, never the ones merely hers.
    #[must_use]
    pub fn equipment(&self) -> Equipment {
        let on = &self.items_switched_on;
        Equipment {
            hi_jump: on[Item::HiJumpBoots],
            space_jump: on[Item::SpaceJump],
            spring_ball: on[Item::SpringBall],
            screw_attack: on[Item::ScrewAttack],
            other_suit: on[Item::VariaSuit] || on[Item::GravitySuit],
        }
    }

    /// She takes an item, a beam or a tank; `switches_off` is the beam each beam taken switches
    /// off, if any.
    pub(crate) fn take(&mut self, pickup: Pickup, switches_off: PerBeam<Option<Beam>>) {
        match pickup {
            // Spec: TakingItems. Hers and switched on, even one she had and switched off.
            Pickup::Item(item) => {
                self.items[item] = true;
                self.items_switched_on[item] = true;
            }
            // Spec: TakingBeams. Hers and switched on, and the beam it doesn't fire with
            // switched off, though still hers.
            Pickup::Beam(beam) => {
                self.beams[beam] = true;
                self.beams_switched_on[beam] = true;
                if let Some(other) = switches_off[beam] {
                    self.beams_switched_on[other] = false;
                }
            }
            Pickup::Tank { supply, amount } => self.take_tank(supply, amount),
        }
    }

    /// A tank raises its capacity by its amount. An energy tank then fills her energy to the new
    /// capacity; a reserve tank leaves the reserve as it was and, if she had no reserve tank,
    /// makes it refill her on its own; any other tank adds its amount to what she holds, without
    /// filling her up. Spec: Tanks.
    fn take_tank(&mut self, supply: Supply, amount: Count) {
        self.capacity[supply] += amount;
        match supply {
            Supply::Energy => self.held[supply] = self.capacity[supply],
            Supply::Reserve if self.reserve_mode == ReserveMode::NoTank => {
                self.reserve_mode = ReserveMode::Automatic;
            }
            Supply::Reserve => {}
            Supply::Missiles | Supply::SuperMissiles | Supply::PowerBombs => {
                self.held[supply] += amount;
            }
        }
    }
}

/// Energy and its capacity at the start of a new file.
const NEW_FILE_ENERGY: Count = Wrapping(99);

/// Each capacity and count at the start of a new file, in `Supply::ALL`'s order: energy 99, every
/// other 0.
const NEW_FILE_SUPPLIES: PerSupply<Count> = PerSupply::new([
    NEW_FILE_ENERGY,
    Wrapping(0),
    Wrapping(0),
    Wrapping(0),
    Wrapping(0),
]);
