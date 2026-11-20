//! Spec entry metadata, separate from health decisions.

use serde::Deserialize;
use types::{Checked, Kind, Switch};

/// Entries implemented by this bounded model.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub enum BehaviourId {
    /// Accumulated periodic loss reduces energy once.
    PeriodicLoss,
    /// Whole energy determines warning transitions after loss.
    LowEnergyWarning,
}

impl BehaviourId {
    /// Every entry in the specification, for catalogue and coverage enumeration.
    pub const ALL: [Self; 2] = [Self::PeriodicLoss, Self::LowEnergyWarning];
}

/// One specification entry and its required metadata.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Behaviour {
    /// Stable specification entry identifier.
    pub id: BehaviourId,
    /// Player-visible rule.
    pub summary: &'static str,
    /// Mechanic or output.
    pub kind: Kind,
    /// Fixed or adjustable parameter.
    pub switch: Switch,
    /// How the entry is checked.
    pub checked: Checked,
}

/// Spec: `PeriodicLoss`, `LowEnergyWarning`. Exhaustive catalogue with shared metadata.
///
/// Both entries are checked by vectors. `R-maintenance-current` is the unchanged
/// recorded maintenance witness, not evidence of a loss or warning transition.
/// Constructed arithmetic contrasts check `PeriodicLoss`; constructed threshold
/// and warning-transition contrasts check `LowEnergyWarning`. Integration admission
/// contrasts are not model coverage.
#[must_use]
pub const fn behaviour(id: BehaviourId) -> Behaviour {
    match id {
        BehaviourId::PeriodicLoss => Behaviour {
            id: BehaviourId::PeriodicLoss,
            summary: "accumulated periodic loss reduces energy, including fractional loss, once.",
            kind: Kind::Mechanic,
            switch: Switch::Fixed,
            checked: Checked::ByVectors,
        },
        BehaviourId::LowEnergyWarning => Behaviour {
            id: BehaviourId::LowEnergyWarning,
            summary: "entering low energy requests a warning, leaving it requests silence.",
            kind: Kind::Output,
            switch: Switch::Parameter,
            checked: Checked::ByVectors,
        },
    }
}
