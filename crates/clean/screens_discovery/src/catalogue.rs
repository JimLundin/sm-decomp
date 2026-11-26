//! The catalogue: one behaviour for each entry of the spec.

use enum_map::{Enum, EnumMap, enum_map};
use types::{Checked, Kind, Switch};

/// A spec entry, by its id.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum)]
pub enum BehaviourId {
    /// The egg waits for Samus, then rocks and opens.
    EggHatches,
    /// The shell breaks, and fragments arc away.
    ShellScatters,
    /// The baby rises above Samus then holds still.
    BabyEmerges,
    /// The rising baby sheds droplets that burst against the floor.
    SlimeFalls,
    /// The baby's shape cycles through its pulse, even behind the egg.
    BabyPulses,
    /// The baby chases Samus and cries twice.
    BabyFollows,
    /// The hatch calls for the next page, then the room's figures disappear into the dissolve.
    PageThreeApproaches,
}

/// One behaviour of the catalogue.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Behaviour {
    /// Its spec entry.
    pub id: BehaviourId,
    /// What the player sees.
    pub summary: &'static str,
    /// What kind of behaviour it is.
    pub kind: Kind,
    /// How it can be adjusted.
    pub switch: Switch,
    /// How the spec checks it.
    pub checked: Checked,
}

/// The catalogue, one behaviour per spec entry.
#[must_use]
pub fn catalogue() -> EnumMap<BehaviourId, Behaviour> {
    enum_map! {
        BehaviourId::EggHatches => Behaviour {
            id: BehaviourId::EggHatches,
            summary: "the egg waits for Samus, then rocks and opens",
            kind: Kind::Mechanic,
            switch: Switch::Fixed,
            checked: Checked::ByVectors,
        },
        BehaviourId::ShellScatters => Behaviour {
            id: BehaviourId::ShellScatters,
            summary: "the shell breaks, and fragments arc away",
            kind: Kind::Mechanic,
            switch: Switch::Fixed,
            checked: Checked::ByVectors,
        },
        BehaviourId::BabyEmerges => Behaviour {
            id: BehaviourId::BabyEmerges,
            summary: "the baby rises above Samus then holds still",
            kind: Kind::Mechanic,
            switch: Switch::Fixed,
            checked: Checked::ByVectors,
        },
        BehaviourId::SlimeFalls => Behaviour {
            id: BehaviourId::SlimeFalls,
            summary: "the rising baby sheds droplets that burst against the floor",
            kind: Kind::Mechanic,
            switch: Switch::Fixed,
            checked: Checked::ByVectors,
        },
        BehaviourId::BabyPulses => Behaviour {
            id: BehaviourId::BabyPulses,
            summary: "the baby's shape cycles through its pulse, even behind the egg",
            kind: Kind::Mechanic,
            switch: Switch::Fixed,
            checked: Checked::ByVectors,
        },
        BehaviourId::BabyFollows => Behaviour {
            id: BehaviourId::BabyFollows,
            summary: "the baby chases Samus and cries twice",
            kind: Kind::Mechanic,
            switch: Switch::Fixed,
            checked: Checked::ByVectors,
        },
        BehaviourId::PageThreeApproaches => Behaviour {
            id: BehaviourId::PageThreeApproaches,
            summary: "the hatch calls for the next page, then the room's figures disappear into the dissolve",
            kind: Kind::Mechanic,
            switch: Switch::Fixed,
            checked: Checked::ByVectors,
        },
    }
}
