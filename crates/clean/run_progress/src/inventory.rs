//! What she has (spec §1.1, §4): her items and beams, hers and switched on, her capacities, what
//! she holds and her reserve mode; and what taking an item, a beam or a tank gives her.

use core::num::Wrapping;

use enum_map::{Enum, EnumMap, enum_map};
use types::{Beam, Item, Set};

/// A count or a capacity. The spec's notation: counts and capacities are whole numbers of 16 bits
/// and wrap at 65536, so 65535 and 2 more is 1: its `+` is `Wrapping`'s.
pub type Count = Wrapping<u16>;

/// A supply she holds, whose capacity a tank raises.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum)]
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

/// A count of each supply: her capacities, or what she holds. Read and changed as
/// `supplies.0[supply]`.
///
/// Its default is what a new file gives her of each supply, capacity and count alike: 99 energy,
/// and 0 of every other supply (spec §1.1, The start of a run).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Supplies(pub EnumMap<Supply, Count>);

impl Default for Supplies {
    fn default() -> Self {
        Self(enum_map! {
            Supply::Energy => NEW_FILE_ENERGY,
            Supply::Reserve => EMPTY,
            Supply::Missiles => EMPTY,
            Supply::SuperMissiles => EMPTY,
            Supply::PowerBombs => EMPTY,
        })
    }
}

/// The energy a new file gives her, and its capacity.
const NEW_FILE_ENERGY: Count = Wrapping(99);
/// None of a supply: a capacity or a count of 0.
const EMPTY: Count = Wrapping(0);

/// What she has: what is hers, and what is switched on.
///
/// The two are kept apart and neither is kept within the other. Taking an item or a beam makes
/// it both hers and switched on, but others switch her items and beams on and off too: the
/// equipment screen, and the rainbow beam she is given late in the fight with Mother Brain,
/// which switches on exactly charge, plasma and wave, whether or not they are hers. So a beam
/// can be switched on without being hers.
///
/// Its default is what a new file gives her: no items or beams, hers or switched on; the
/// supplies' own default, capacity and count alike; and no reserve tank.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Inventory {
    /// The items that are hers, switched on or not. Spec: TakingItems.
    pub items: Set<Item>,
    /// The items switched on, which decide what she can do and how she looks. Her motion reads
    /// the hi-jump boots and the space jump here, and her sprite the screw attack, the spring
    /// ball and a suit other than the first: never the items merely hers.
    pub items_switched_on: Set<Item>,
    /// The beams that are hers, switched on or not. Spec: TakingBeams.
    pub beams: Set<Beam>,
    /// The beams switched on, which decide the beam she shoots; not always among hers.
    pub beams_switched_on: Set<Beam>,
    /// The most of each supply she can hold.
    pub capacity: Supplies,
    /// How much of each supply she holds now.
    pub held: Supplies,
    /// Whether the reserve refills her on its own.
    pub reserve_mode: ReserveMode,
}

impl Inventory {
    /// She takes an item, a beam or a tank; `switches_off` gives, for each beam she takes, the
    /// beams that taking it switches off.
    pub(crate) fn take(&mut self, pickup: Pickup, switches_off: &EnumMap<Beam, Vec<Beam>>) {
        match pickup {
            // Spec: TakingItems. Hers and switched on, even one she had and switched off.
            Pickup::Item(item) => {
                self.items[item] = true;
                self.items_switched_on[item] = true;
            }
            // Spec: TakingBeams. Each beam of the taken beam's row is switched off: hers still
            // if it was, not hers if it wasn't (a beam the rainbow beam switched on). Then the
            // beam taken is hers and switched on, even one its own row names.
            Pickup::Beam(beam) => {
                for &other in &switches_off[beam] {
                    self.beams_switched_on[other] = false;
                }
                self.beams[beam] = true;
                self.beams_switched_on[beam] = true;
            }
            Pickup::Tank { supply, amount } => self.take_tank(supply, amount),
        }
    }

    /// A tank raises its capacity by its amount. An energy tank then fills her energy to the new
    /// capacity; a reserve tank leaves the reserve as it was and, if she had no reserve tank,
    /// makes it refill her on its own; any other tank adds its amount to what she holds, without
    /// filling her up. Spec: Tanks.
    fn take_tank(&mut self, supply: Supply, amount: Count) {
        self.capacity.0[supply] += amount;
        match supply {
            Supply::Energy => self.held.0[supply] = self.capacity.0[supply],
            Supply::Reserve if self.reserve_mode == ReserveMode::NoTank => {
                self.reserve_mode = ReserveMode::Automatic;
            }
            Supply::Reserve => {}
            Supply::Missiles | Supply::SuperMissiles | Supply::PowerBombs => {
                self.held.0[supply] += amount;
            }
        }
    }
}
