//! What she has: her items and beams, hers and equipped, her capacities, what she holds and her
//! reserve mode; and what taking an item, a beam or a tank gives her.

use core::num::Wrapping;

use clean_types::{Beam, Equipment, Item};

use crate::count::Count;
use crate::rules::Rules;
use crate::sets::flag_set;
use crate::supplies::{PerSupply, Supply};

flag_set! {
    /// Items: those that are hers, or those equipped (switched on). Spec: TakingItems.
    Items of Item {
        Bombs, GrappleBeam, GravitySuit, HiJumpBoots, MorphBall, ScrewAttack, SpaceJump,
        SpeedBooster, SpringBall, VariaSuit, XRayScope,
    }
}

flag_set! {
    /// Beams: those that are hers, or those equipped (switched on). Spec: TakingBeams.
    Beams of Beam { Charge, Ice, Plasma, Spazer, Wave }
}

/// Whether the reserve refills her on its own.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ReserveMode {
    /// No reserve tank yet.
    #[default]
    None,
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

/// What she has. Others switch her items and beams on and off too (the equipment screen, the
/// rainbow beam), and nothing here keeps those equipped within hers: a beam can be equipped
/// without being hers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Inventory {
    /// The items that are hers, equipped or not.
    pub items: Items,
    /// The items equipped (switched on), which decide what she can do and how she looks.
    pub equipped: Items,
    /// The beams that are hers, equipped or not.
    pub beams: Beams,
    /// The beams equipped (switched on), which decide the beam she shoots.
    pub beams_equipped: Beams,
    /// The most of each supply she can hold.
    pub capacity: PerSupply<Count>,
    /// How much of each supply she holds now.
    pub held: PerSupply<Count>,
    /// Whether the reserve refills her on its own.
    pub reserve_mode: ReserveMode,
}

impl Inventory {
    /// What a new file starts her with: 99 energy of a capacity of 99, every other capacity and
    /// count at 0, no reserve tank, and no items or beams, hers or equipped.
    #[must_use]
    pub fn new_file() -> Self {
        let mut energy_only = PerSupply::default();
        energy_only[Supply::Energy] = NEW_FILE_ENERGY;
        Self {
            items: Items::default(),
            equipped: Items::default(),
            beams: Beams::default(),
            beams_equipped: Beams::default(),
            capacity: energy_only,
            held: energy_only,
            reserve_mode: ReserveMode::None,
        }
    }

    /// What her motion reads: the items equipped, never the ones merely hers.
    #[must_use]
    pub const fn equipment(&self) -> Equipment {
        let on = self.equipped;
        Equipment {
            hi_jump: on.contains(Item::HiJumpBoots),
            space_jump: on.contains(Item::SpaceJump),
            spring_ball: on.contains(Item::SpringBall),
            screw_attack: on.contains(Item::ScrewAttack),
            other_suit: on.contains(Item::VariaSuit) || on.contains(Item::GravitySuit),
        }
    }

    /// She takes an item, a beam or a tank.
    pub(crate) fn take(&mut self, rules: &Rules, pickup: Pickup) {
        match pickup {
            // Spec: TakingItems. Hers and equipped, even one she had and switched off.
            Pickup::Item(item) => {
                self.items = self.items.with(item);
                self.equipped = self.equipped.with(item);
            }
            // Spec: TakingBeams. Hers and equipped, and any beam it doesn't fire with switched
            // off, though still hers.
            Pickup::Beam(beam) => {
                self.beams = self.beams.with(beam);
                self.beams_equipped = rules
                    .beams_switched_off
                    .iter()
                    .filter(|row| row.taken == beam)
                    .fold(self.beams_equipped.with(beam), |on, row| {
                        on.without(row.switched_off)
                    });
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
            Supply::Reserve if self.reserve_mode == ReserveMode::None => {
                self.reserve_mode = ReserveMode::Automatic;
            }
            Supply::Reserve => {}
            Supply::Missiles | Supply::SuperMissiles | Supply::PowerBombs => {
                self.held[supply] += amount;
            }
        }
    }
}

/// Energy and its capacity at the start of a new file; every other count starts at 0.
const NEW_FILE_ENERGY: Count = Wrapping(99);
