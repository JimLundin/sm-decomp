//! Behaviour metadata for the room quake model.

/// Stable ids from the world-quake specification.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, enum_map::Enum)]
pub enum BehaviourId {
    /// The phase advances even while the room stays steady.
    QuietQuakePhase,
    /// A start replaces the shake under way; a stop ends it.
    ShakeStarts,
    /// Each update of a shaking room spends one of its frames.
    ShakeCountsDown,
    /// The layers a shake reaches jump back and forth along its direction by its size.
    ShakenLayers,
    /// A shake that reaches the actors asks them to shake and moves their projectiles.
    ShakenActors,
}

impl BehaviourId {
    /// The entry's name in the spec and vector file.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::QuietQuakePhase => "QuietQuakePhase",
            Self::ShakeStarts => "ShakeStarts",
            Self::ShakeCountsDown => "ShakeCountsDown",
            Self::ShakenLayers => "ShakenLayers",
            Self::ShakenActors => "ShakenActors",
        }
    }
}

/// Classification of a world-quake behaviour.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// A game rule.
    Mechanic,
}

/// Adjustability of a world-quake behaviour.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Switch {
    /// Always in force, with no parameters.
    Fixed,
}

/// Evidence used to check a behaviour.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Check {
    /// Vectors recorded from the original, with constructed vectors for what no record shows.
    OracleAndConstructed {
        /// What the recorded vectors cover.
        recorded: &'static str,
    },
}

/// One named rule and how it is checked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Behaviour {
    /// Spec entry id.
    pub id: BehaviourId,
    /// What the player sees.
    pub summary: &'static str,
    /// Classification of the rule.
    pub kind: Kind,
    /// Whether the rule is adjustable.
    pub switch: Switch,
    /// Evidence used to check the rule.
    pub checked: Check,
}

/// All world-quake spec entries, one row per behaviour.
pub const CATALOGUE: &[Behaviour] = &[
    Behaviour {
        id: BehaviourId::QuietQuakePhase,
        summary: "The quake phase advances while the room stays steady.",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Check::OracleAndConstructed {
            recorded: "a quiet frame of play; constructed starts and the wrap",
        },
    },
    Behaviour {
        id: BehaviourId::ShakeStarts,
        summary: "A shake another part of the game starts replaces the one under way, and a \
                  stop ends it.",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Check::OracleAndConstructed {
            recorded: "the frames the shake started or stopped in the Ceres rows",
        },
    },
    Behaviour {
        id: BehaviourId::ShakeCountsDown,
        summary: "Each update of a shaking room spends one of its frames.",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Check::OracleAndConstructed {
            recorded: "every frame of play the Ceres rows hold",
        },
    },
    Behaviour {
        id: BehaviourId::ShakenLayers,
        summary: "While a shake lasts, the layers it reaches jump back and forth along its \
                  direction by its size, two frames each way.",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Check::OracleAndConstructed {
            recorded: "the layers' places in every frame of play the Ceres rows hold",
        },
    },
    Behaviour {
        id: BehaviourId::ShakenActors,
        summary: "A shake that reaches the actors asks each of them to shake, and moves their \
                  projectiles as they are drawn, the opposite way to the layers' picture.",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Check::OracleAndConstructed {
            recorded: "the actors asked in every frame of play the Ceres rows hold",
        },
    },
];
