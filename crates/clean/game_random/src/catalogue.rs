//! Catalogue of the shared sequence's specified behaviours.

/// A named entry in the shared random sequence spec.
#[derive(Clone, Copy, Debug, PartialEq, Eq, enum_map::Enum)]
pub enum BehaviourId {
    /// A completed draw advances and returns the shared random number.
    AdvanceRandom,
}

/// The kind of behaviour an entry describes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// A game rule.
    Mechanic,
}

/// How an entry can vary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Switch {
    /// The behaviour has no alternative or parameters.
    Fixed,
}

/// How the spec checks an entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Check {
    /// Recorded oracle draws, supplemented by constructed states and continuations.
    Oracle {
        /// Recorded vector identifiers.
        recorded: &'static [&'static str],
    },
}

/// The classification and evidence for one spec entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Behaviour {
    /// The spec entry's ID.
    pub id: BehaviourId,
    /// What the entry does in the game.
    pub summary: &'static str,
    /// The entry's kind.
    pub kind: Kind,
    /// Whether the entry can vary.
    pub switch: Switch,
    /// How the entry is checked.
    pub checked: Check,
}

/// One row per shared random sequence spec entry.
pub const CATALOGUE: &[Behaviour] = &[Behaviour {
    id: BehaviourId::AdvanceRandom,
    summary: "a completed draw advances and returns the shared random number.",
    kind: Kind::Mechanic,
    switch: Switch::Fixed,
    checked: Check::Oracle {
        recorded: &["R-first", "R-adjacent"],
    },
}];
