//! Spec entry metadata, separate from health decisions.

use serde::Deserialize;
use types::{Checked, Kind, Switch};

/// Entries implemented by this bounded model.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub enum BehaviourId {
    /// Accumulated periodic loss reduces energy once.
    PeriodicLoss,
    /// An eligible unprotected Ceres tail hit takes energy and grants protection.
    TailHit,
    /// Protection ages at the close of active gameplay.
    ProtectionTime,
    /// Whole energy determines warning transitions after loss.
    LowEnergyWarning,
}

impl BehaviourId {
    /// Every entry in the specification, for catalogue and coverage enumeration.
    pub const ALL: [Self; 4] = [
        Self::PeriodicLoss,
        Self::TailHit,
        Self::ProtectionTime,
        Self::LowEnergyWarning,
    ];
}

/// One specification entry and its required metadata.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Behaviour {
    /// Stable specification entry identifier.
    pub id: BehaviourId,
    /// Player-visible rule.
    pub summary: &'static str,
    /// Behaviour classification.
    pub kind: Kind,
    /// Fixed or adjustable parameter.
    pub switch: Switch,
    /// How the entry is checked.
    pub checked: Checked,
}

/// Exhaustive catalogue with shared metadata. Integration admission contrasts
/// are not model coverage.
///
/// `R-maintenance-current` witnesses Maintenance without a loss or warning
/// transition. `R-tail-hit` and `R-tail-protected` witness contact and aging.
/// Constructed vectors contrast arithmetic, warning thresholds, duplicate
/// contact and expiry.
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
        BehaviourId::TailHit => Behaviour {
            id: BehaviourId::TailHit,
            summary: "Ridley's eligible tail hit in Ceres takes energy and briefly protects Samus.",
            kind: Kind::Mechanic,
            switch: Switch::Fixed,
            checked: Checked::ByVectors,
        },
        BehaviourId::ProtectionTime => Behaviour {
            id: BehaviourId::ProtectionTime,
            summary: "protection spends one frame at the end of each admitted active gameplay opportunity.",
            kind: Kind::Timing,
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
