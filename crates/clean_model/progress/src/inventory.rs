//! What she has: her items and beams, those switched on, her capacities, what she holds, and her
//! reserve mode; and what taking an item, a beam or a tank gives her.

use std::collections::BTreeSet;

use clean_types::{Beam, Equipment, Item};

use crate::count::Count;
use crate::rules::Rules;
use crate::supplies::{PerSupply, Supply};

/// Energy and its capacity at the start of a new file; every other count starts at 0.
const NEW_FILE_ENERGY: Count = Count::new(99);

/// The pieces of one sort she has, and those switched on.
///
/// Others switch them on and off too (the equipment screen, the rainbow beam), and nothing here
/// keeps those switched on within hers: a beam can be switched on without being hers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Collection<T> {
    /// The pieces that are hers.
    pub hers: BTreeSet<T>,
    /// The pieces switched on, which decide what she can do.
    pub switched_on: BTreeSet<T>,
}

impl<T: Ord + Copy> Collection<T> {
    /// None, hers or switched on.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            hers: BTreeSet::new(),
            switched_on: BTreeSet::new(),
        }
    }

    /// Whether `piece` is hers, switched on or not.
    #[must_use]
    pub fn is_hers(&self, piece: T) -> bool {
        self.hers.contains(&piece)
    }

    /// Whether `piece` is switched on, hers or not.
    #[must_use]
    pub fn is_switched_on(&self, piece: T) -> bool {
        self.switched_on.contains(&piece)
    }

    /// A piece she takes is hers and switched on, even one she had and switched off.
    fn take(&mut self, piece: T) {
        self.hers.insert(piece);
        self.switched_on.insert(piece);
    }

    /// How many pieces are hers.
    fn count_hers(&self) -> Count {
        self.hers
            .iter()
            .fold(Count::ZERO, |n, _| n.plus(Count::ONE))
    }
}

impl<T: Ord + Copy> Default for Collection<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Whether her reserve tanks refill her on their own.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ReserveMode {
    /// No reserve tank yet.
    #[default]
    None,
    /// The reserves refill her on their own.
    Automatic,
    /// The reserves refill her only when she calls on them.
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
        /// How much it raises it by: in the original game 100 for energy and reserve tanks, 5
        /// for the others.
        amount: Count,
    },
}

/// What she has.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Inventory {
    /// Her items, and those switched on.
    pub items: Collection<Item>,
    /// Her beams, and those switched on.
    pub beams: Collection<Beam>,
    /// The most of each supply she can hold.
    pub capacity: PerSupply<Count>,
    /// How much of each supply she holds now.
    pub held: PerSupply<Count>,
    /// Whether her reserves refill her on their own.
    pub reserve_mode: ReserveMode,
}

impl Inventory {
    /// What a new file starts her with: 99 energy of a capacity of 99, every other capacity and
    /// count at 0, no reserve tank, and no items or beams, hers or switched on.
    #[must_use]
    pub fn new_file() -> Self {
        let energy_only = PerSupply {
            energy: NEW_FILE_ENERGY,
            ..PerSupply::default()
        };
        Self {
            items: Collection::new(),
            beams: Collection::new(),
            capacity: energy_only,
            held: energy_only,
            reserve_mode: ReserveMode::None,
        }
    }

    /// What she can do and how she looks, read from the items switched on, never from the ones
    /// that are merely hers.
    #[must_use]
    pub fn equipment(&self) -> Equipment {
        let on = |item| self.items.is_switched_on(item);
        Equipment {
            hi_jump: on(Item::HiJumpBoots),
            space_jump: on(Item::SpaceJump),
            spring_ball: on(Item::SpringBall),
            screw_attack: on(Item::ScrewAttack),
            other_suit: on(Item::VariaSuit) || on(Item::GravitySuit),
        }
    }

    /// The item percentage the ending shows: each capacity divided by its divisor, rounded down,
    /// summed, plus the count of her items and her beams. Only what is hers counts, switched on
    /// or not, so a beam switched on that isn't hers adds nothing; and the 99 energy she starts
    /// with counts 0. Spec: ItemPercentage.
    #[must_use]
    pub fn item_percentage(&self, rules: &Rules) -> Count {
        let tanks = Supply::ALL.iter().fold(Count::ZERO, |sum, &supply| {
            let divisor = rules.percentage_divisors.get(supply);
            sum.plus(self.capacity.get(supply).divided_by(divisor))
        });
        tanks
            .plus(self.items.count_hers())
            .plus(self.beams.count_hers())
    }

    /// She takes an item, a beam or a tank.
    pub(crate) fn take(&mut self, rules: &Rules, pickup: Pickup) {
        match pickup {
            Pickup::Item(item) => self.items.take(item),
            Pickup::Beam(beam) => self.take_beam(rules, beam),
            Pickup::Tank { supply, amount } => self.take_tank(supply, amount),
        }
    }

    /// A beam she takes is hers and switched on, and the beam it doesn't fire with is switched
    /// off, though still hers. Spec: TakingBeams.
    fn take_beam(&mut self, rules: &Rules, beam: Beam) {
        self.beams.take(beam);
        if let Some(other) = rules.switched_off_by(beam) {
            self.beams.switched_on.remove(&other);
        }
    }

    /// A tank raises its capacity by its amount. An energy tank then fills her energy to the new
    /// capacity; a reserve tank leaves the reserve energy as it was and, if she had no reserve
    /// tank, makes the reserves refill her on their own; any other tank adds its amount to what
    /// she holds, without filling her up. Spec: Tanks.
    fn take_tank(&mut self, supply: Supply, amount: Count) {
        let capacity = self.capacity.get(supply).plus(amount);
        self.capacity.set(supply, capacity);
        match supply {
            Supply::Energy => self.held.set(supply, capacity),
            Supply::Reserve => {
                if self.reserve_mode == ReserveMode::None {
                    self.reserve_mode = ReserveMode::Automatic;
                }
            }
            Supply::Missiles | Supply::SuperMissiles | Supply::PowerBombs => {
                self.held.set(supply, self.held.get(supply).plus(amount));
            }
        }
    }
}
