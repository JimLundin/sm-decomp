//! The recorded controls of the intro's two played scenes (screens-playback): the State a caller
//! carries from one turn to the next, saves and restores alongside the selected scene's pad
//! content.
//!
//! Samus's movement and the room keep their own States; this one holds only what the recorded
//! pad itself remembers: the scene phase, the timeline progress and the last pad sample.

use enum_map::Enum;
use serde::{Deserialize, Serialize};

use crate::{Set, set_of};

/// A control the scenes' recordings use, as an action. With the intro's standard bindings,
/// Jump is A, Fire is X and Aim Up is R.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum, Serialize, Deserialize)]
pub enum PadAction {
    /// Run left.
    Left,
    /// Jump.
    Jump,
    /// Fire the selected weapon.
    Fire,
    /// Aim up.
    AimUp,
}

/// A pad sample: the controls held, and those freshly pressed this turn. A fresh press is also
/// held.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PadSample {
    /// The controls held this turn.
    pub held: Set<PadAction>,
    /// The controls that receive a fresh press on this turn: those of a hold's first turn. An
    /// action held on the turn before may receive a fresh press again when a new hold begins.
    /// Each is also in `held`.
    pub pressed: Set<PadAction>,
}

impl PadSample {
    /// Nothing held and nothing pressed: the last pad on a start and after a finish, and what
    /// the finishes issue.
    #[must_use]
    pub fn neutral() -> Self {
        Self {
            held: Set::default(),
            pressed: Set::default(),
        }
    }

    /// Spec: `DemoPlaysPad`. Left held with no fresh press: what a discovery timeline issues
    /// once it has run out.
    #[must_use]
    pub fn walking_left() -> Self {
        Self {
            held: set_of([PadAction::Left]),
            pressed: Set::default(),
        }
    }

    /// The same controls still held on a later turn of their hold, with no fresh press.
    #[must_use]
    pub fn held_on(self) -> Self {
        Self {
            held: self.held,
            pressed: Set::default(),
        }
    }
}

/// Spec: timeline progress. The number of room actor turns already issued in the active
/// phase, zero before its first turn. It never goes past the length of the phase's timeline.
///
/// Counted against the timeline's holds, it names the current hold, the one whose turns it
/// falls within, and the elapsed turn within that hold: zero on the hold's first turn, the
/// one that carries its fresh presses. At the timeline's length there is no current hold.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TimelineProgress(pub u32);

impl TimelineProgress {
    /// No turn issued yet.
    pub const START: Self = Self(0);

    /// One more turn issued.
    #[must_use]
    pub const fn after_turn(self) -> Self {
        Self(self.0.saturating_add(1))
    }
}

/// The current hold and the elapsed turn within it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimelinePlace {
    /// The room actor turns already issued in this phase.
    pub progress: TimelineProgress,
    /// The most recent sample issued, carried from Approach into Watch.
    pub last: PadSample,
}

impl TimelinePlace {
    /// Spec: `DemoReceivesScene`. A scene's place as it starts: no turn issued and a neutral
    /// last pad.
    #[must_use]
    pub fn at_start() -> Self {
        Self {
            progress: TimelineProgress::START,
            last: PadSample::neutral(),
        }
    }
}

/// The recorded pad's complete State: the scene phase.
///
/// Battle, Approach and Watch carry their timeline progress and last pad sample. Outside them
/// there is no progress and the last pad is neutral.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScenePhase {
    /// No scene has started.
    Off,
    /// The battle plays its timeline.
    Battle(TimelinePlace),
    /// The discovery's walk toward the baby.
    Approach(TimelinePlace),
    /// The discovery's watch by the baby.
    Watch(TimelinePlace),
    /// The battle has finished with Samus standing looking left.
    BattleLocked,
    /// The discovery has released Samus.
    Released,
}
