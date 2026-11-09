//! What she has (spec §1.1, §4): her items and beams, hers and switched on, her capacities, what
//! she holds and her reserve mode; and what taking an item, a beam or a tank gives her.

use core::num::Wrapping;

use clean_types::{Beam, Equipment, Item, Set};
use enum_map::{Enum, EnumMap};

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

/// What she has: what is hers, and what is switched on.
///
/// The two are kept apart and neither is kept within the other. Taking an item or a beam makes
/// it both hers and switched on, but others switch her items and beams on and off too (the
/// equipment screen, and the rainbow beam late in the fight with Mother Brain, which switches on
/// charge, plasma and wave whichever are hers). So what is switched on need not be among what is
/// hers: a beam can be switched on that she never took.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Inventory {
    /// The items that are hers, switched on or not. Spec: TakingItems.
    pub items: Set<Item>,
    /// The items switched on, which decide what she can do and how she looks.
    pub items_switched_on: Set<Item>,
    /// The beams that are hers, switched on or not. Spec: TakingBeams.
    pub beams: Set<Beam>,
    /// The beams switched on, which decide the beam she shoots; not always among hers.
    pub beams_switched_on: Set<Beam>,
    /// The most of each supply she can hold.
    pub capacity: EnumMap<Supply, Count>,
    /// How much of each supply she holds now.
    pub held: EnumMap<Supply, Count>,
    /// Whether the reserve refills her on its own.
    pub reserve_mode: ReserveMode,
}

/// What a new file gives her: 99 energy of a capacity of 99, every other capacity and count at
/// 0, no reserve tank, and no items or beams, hers or switched on.
impl Default for Inventory {
    fn default() -> Self {
        Self {
            items: Set::default(),
            items_switched_on: Set::default(),
            beams: Set::default(),
            beams_switched_on: Set::default(),
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
    pub(crate) fn take(&mut self, pickup: Pickup, switches_off: EnumMap<Beam, Option<Beam>>) {
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

/// Each capacity and count at the start of a new file, in `Supply`'s declared order: energy 99,
/// every other 0.
const NEW_FILE_SUPPLIES: EnumMap<Supply, Count> = EnumMap::from_array([
    NEW_FILE_ENERGY,
    Wrapping(0),
    Wrapping(0),
    Wrapping(0),
    Wrapping(0),
]);
