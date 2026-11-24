//! The catalogue: one behaviour for each entry of the power-on's spec.

use enum_map::{Enum, EnumMap};
use types::{Checked, Kind, Switch};

/// A spec entry of the power-on, by its id.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum)]
pub enum BehaviourId {
    /// Until the logo's first frame finishes, the screen stays blank.
    QuietUntilTheLogo,
    /// The logo brightens by the fade step on each finished frame, from dark to full.
    LogoBrightens,
    /// At full brightness the logo stays for the hold's frames, then starts to dim.
    LogoHeld,
    /// The logo dims by the fade step on each finished frame; at dark the screen blanks.
    LogoDims,
    /// Black at full level while the sound driver loads; once its upload ends the game settles.
    SoundLoads,
    /// Black while the game settles; the next finished frame is the title's.
    Settling,
    /// Every finished frame adds one to the count; a frame that did not finish adds none.
    FramesCounted,
}

/// A behaviour of the catalogue: its spec entry, what the player sees, its kind, how it may be
/// adjusted and how the spec checks it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Behaviour {
    /// Its spec entry.
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

/// Every behaviour, one row for each spec entry.
const BEHAVIOURS: <BehaviourId as Enum>::Array<Behaviour> = [
    Behaviour {
        id: BehaviourId::QuietUntilTheLogo,
        summary: "until the logo's first frame finishes, the screen stays blank",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::LogoBrightens,
        summary: "the logo brightens by the fade step on each finished frame, from dark to full",
        kind: Kind::Mechanic,
        switch: Switch::Parameter,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::LogoHeld,
        summary: "at full brightness the logo stays for the hold's frames, then starts to dim",
        kind: Kind::Mechanic,
        switch: Switch::Parameter,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::LogoDims,
        summary: "the logo dims by the fade step on each finished frame; at dark the screen blanks",
        kind: Kind::Mechanic,
        switch: Switch::Parameter,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::SoundLoads,
        summary: "black at full level while the sound driver loads; once its upload ends the game settles",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::Settling,
        summary: "black while the game settles; the next finished frame is the title's",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::FramesCounted,
        summary: "every finished frame adds one to the count; a frame that did not finish adds none",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
];

/// The catalogue, keyed by spec entry.
#[must_use]
pub fn catalogue() -> EnumMap<BehaviourId, Behaviour> {
    types::by_member(BEHAVIOURS, |behaviour| behaviour.id)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Each spec entry has exactly one row: every key holds the row that names it.
    #[test]
    fn every_entry_has_its_own_row() {
        for (id, behaviour) in catalogue() {
            assert_eq!(behaviour.id, id);
        }
    }
}
