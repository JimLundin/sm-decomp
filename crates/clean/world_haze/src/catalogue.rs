//! Catalogue of the ordinary Ceres haze rules.

use enum_map::{Enum, EnumMap, enum_map};

/// Entry identifiers from the Ceres haze specification.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum)]
pub enum BehaviourId {
    /// Prepare the next gradient.
    HazeBrightens,
    /// Finish the fade without replacing bands.
    HazeSettles,
    /// Hold the visible gradient.
    HazeHolds,
}

/// What sort of rule a catalogue entry describes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// A deliberate game rule.
    Mechanic,
}

/// Whether a rule can be adjusted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Switch {
    /// No toggle or parameters.
    Fixed,
}

/// Specification metadata for one haze entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Behaviour {
    /// Specification identifier.
    pub id: BehaviourId,
    /// What the player sees.
    pub summary: &'static str,
    /// The rule's kind.
    pub kind: Kind,
    /// Whether the rule is adjustable.
    pub switch: Switch,
    /// How the rule is checked.
    pub checked: &'static str,
}

/// All three haze entries and their verification metadata.
#[must_use]
pub fn catalogue() -> EnumMap<BehaviourId, Behaviour> {
    enum_map! {
        BehaviourId::HazeBrightens => Behaviour {
            id: BehaviourId::HazeBrightens,
            summary: "the haze's bright edge grows one level per ordinary opportunity.",
            kind: Kind::Mechanic,
            switch: Switch::Fixed,
            checked: "oracle (R-bright-edge), with constructed lower peaks from the source arithmetic.",
        },
        BehaviourId::HazeSettles => Behaviour {
            id: BehaviourId::HazeSettles,
            summary: "the haze finishes its fade one opportunity after preparing full brightness.",
            kind: Kind::Mechanic,
            switch: Switch::Fixed,
            checked: "oracle (R-reaches-steady, also with held input).",
        },
        BehaviourId::HazeHolds => Behaviour {
            id: BehaviourId::HazeHolds,
            summary: "the gradient remains steady during ordinary time in the room.",
            kind: Kind::Mechanic,
            switch: Switch::Fixed,
            checked: "oracle (R-steady, completed-cut comparison).",
        },
    }
}
