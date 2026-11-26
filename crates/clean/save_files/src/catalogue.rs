//! The behaviour catalogue: one entry per spec entry.

use types::{Checked, Kind, Switch};

/// A spec entry's id.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BehaviourId {
    /// Accept either complete matching checksum pair, or clear the damaged file.
    ReadSlot,
    /// Check the cartridge and find out how many demos are available.
    BootFiles,
    /// Inspect all three slots and recover the last cursor choice.
    MenuFiles,
    /// Record the player's choice and provide the run to start or resume.
    ChooseFile,
    /// Save the active run at the two cutscene transitions.
    WriteFile,
}

/// A catalogue entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Behaviour {
    /// The spec entry it carries out.
    pub id: BehaviourId,
    /// What the player sees.
    pub summary: &'static str,
    /// Its kind.
    pub kind: Kind,
    /// How it can be adjusted.
    pub switch: Switch,
    /// How the spec checks it.
    pub checked: Checked,
}

/// Every behaviour of the save files, in spec order.
pub const CATALOGUE: &[Behaviour] = &[
    Behaviour {
        id: BehaviourId::ReadSlot,
        summary: "accept either complete matching checksum pair, or clear the damaged file",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::BootFiles,
        summary: "check the cartridge and find out how many demos are available",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::MenuFiles,
        summary: "inspect all three slots and recover the last cursor choice",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::ChooseFile,
        summary: "record the player's choice and provide the run to start or resume",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::WriteFile,
        summary: "save the active run at the two cutscene transitions",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
];
