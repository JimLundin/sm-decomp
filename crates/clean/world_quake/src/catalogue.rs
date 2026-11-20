//! Behaviour metadata for the room quake model.

/// Stable ids from the world-quake specification.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, enum_map::Enum)]
pub enum BehaviourId {
    /// The phase advances even while the room stays steady.
    QuietQuakePhase,
}

impl BehaviourId {
    /// The entry's name in the spec and vector file.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::QuietQuakePhase => "QuietQuakePhase",
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
    /// A recorded oracle vector, supplemented by independent constructed starts and wrap.
    OracleWithConstructedStartsAndWrap {
        /// Id of the recorded oracle vector.
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
pub const CATALOGUE: &[Behaviour] = &[Behaviour {
    id: BehaviourId::QuietQuakePhase,
    summary: "The quake phase advances while the room stays steady.",
    kind: Kind::Mechanic,
    switch: Switch::Fixed,
    checked: Check::OracleWithConstructedStartsAndWrap {
        recorded: "R-quiet-frame",
    },
}];
