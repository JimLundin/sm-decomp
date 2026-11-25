//! Spec entry metadata, separate from health decisions.

use serde::Deserialize;
use types::{Checked, Kind, Switch};

/// Entries implemented by this bounded model.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub enum BehaviourId {
    /// Accumulated periodic loss reduces energy once.
    PeriodicLoss,
    /// An eligible hit in Ceres, judged on protection as the frame starts, takes its worth
    /// in energy and grants protection.
    TailHit,
    /// Protection is drawn as the frame's hit left it, then spends one frame.
    ProtectionTime,
    /// Whole energy after the frame's hit and loss decides the warning.
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

/// The catalogue entry for each spec entry. Integration admission contrasts are not
/// model coverage.
///
/// `R-maintenance-current` witnesses a frame without a loss or warning change.
/// `R-tail-hit` and `R-tail-protected` witness a hit and protection spending, and
/// `R-steam-hit` a hit worth nothing that still grants protection. Constructed vectors
/// contrast arithmetic, warning thresholds, hit worths, a hit during protection and expiry.
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
            summary: "an eligible hit in Ceres takes its worth in energy and briefly protects Samus.",
            kind: Kind::Mechanic,
            switch: Switch::Fixed,
            checked: Checked::ByVectors,
        },
        BehaviourId::ProtectionTime => Behaviour {
            id: BehaviourId::ProtectionTime,
            summary: "protection spends one frame in each admitted frame, after the frame's hit.",
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
