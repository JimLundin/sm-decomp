//! The catalogue: one behaviour for each entry of the screens-playback spec.

use enum_map::{Enum, EnumMap, enum_map};
use types::{Checked, Kind, Switch};

/// A behaviour of the spec, by its entry id.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum)]
pub enum BehaviourId {
    /// The selected demo starts without spending its first room actor turn.
    DemoReceivesScene,
    /// The timed pad and the preceding sample go to the room actors.
    DemoPlaysPad,
    /// The walk gives way to the watch when Samus reaches the baby.
    SamusReachesEgg,
    /// The battle's controls finish with Samus standing looking left.
    BattleLocksOnLeft,
    /// The discovery's control ends as the return crosses zero.
    DiscoveryReleasesSamus,
}

/// A catalogue entry: what the player sees, what kind of rule it is, how it may be adjusted and
/// how the spec checks it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Behaviour {
    /// Its entry id.
    pub id: BehaviourId,
    /// What the player sees.
    pub summary: &'static str,
    /// What kind of behaviour it is.
    pub kind: Kind,
    /// How it may be adjusted.
    pub switch: Switch,
    /// How the spec checks it.
    pub checked: Checked,
}

/// The catalogue, one behaviour for each spec entry.
#[must_use]
pub fn catalogue() -> EnumMap<BehaviourId, Behaviour> {
    enum_map! {
        BehaviourId::DemoReceivesScene => Behaviour {
            id: BehaviourId::DemoReceivesScene,
            summary: "Start the selected demo without spending its first room actor turn.",
            kind: Kind::Mechanic,
            switch: Switch::Fixed,
            checked: Checked::ByVectors,
        },
        BehaviourId::DemoPlaysPad => Behaviour {
            id: BehaviourId::DemoPlaysPad,
            summary: "Supply the timed pad and the preceding sample to the room actors.",
            kind: Kind::Output,
            switch: Switch::Fixed,
            checked: Checked::ByVectors,
        },
        BehaviourId::SamusReachesEgg => Behaviour {
            id: BehaviourId::SamusReachesEgg,
            summary: "Trade the walk for the watch when Samus reaches the baby.",
            kind: Kind::Mechanic,
            switch: Switch::Fixed,
            checked: Checked::ByVectors,
        },
        BehaviourId::BattleLocksOnLeft => Behaviour {
            id: BehaviourId::BattleLocksOnLeft,
            summary: "Finish the battle's controls with Samus standing left.",
            kind: Kind::Mechanic,
            switch: Switch::Fixed,
            checked: Checked::ByVectors,
        },
        BehaviourId::DiscoveryReleasesSamus => Behaviour {
            id: BehaviourId::DiscoveryReleasesSamus,
            summary: "End the discovery's control as the return crosses zero.",
            kind: Kind::Mechanic,
            switch: Switch::Fixed,
            checked: Checked::ByVectors,
        },
    }
}
