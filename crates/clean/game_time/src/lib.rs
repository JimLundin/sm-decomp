#![cfg(not(doctest))]
//! The game-time model, built from its behaviour spec alone.

pub use types::{PlayFrames, PlayHours, PlayMinutes, PlaySeconds, PlayTime};

use types::{Checked, Kind, Switch};

/// Spec: `PlayTimeAdvance`. One ordinary opportunity already qualified by the caller.
///
/// The gameplay scheduler supplies this input, not the clock or a hardware refresh.
/// In the supported production scope the scheduler admits only an established ordinary
/// opportunity with positive same-frame whole Health and a clock below frame 59.
/// The total clock rule also accepts qualified opportunities at carry and saturation boundaries.
#[derive(Debug)]
pub struct QualifiedOpportunity;

/// Spec: `PlayTimeAdvance`. Owns the elapsed play time and the result of each advance.
#[derive(Debug)]
pub struct Clock {
    time: PlayTime,
}

impl Clock {
    /// Starts a fresh clock from the given complete state, with no implicit initial time.
    #[must_use]
    pub const fn restore(time: PlayTime) -> Self {
        Self { time }
    }

    /// Reads the complete state, including any earned fraction of a second.
    #[must_use]
    pub const fn time(&self) -> PlayTime {
        self.time
    }

    /// Sets the clock to any legal complete state.
    pub const fn set(&mut self, time: PlayTime) {
        self.time = time;
    }

    /// Spec: `PlayTimeAdvance`. Earns one frame, carrying into seconds, minutes and hours
    /// as needed. At the maximum the clock stays unchanged. Returns the retained result.
    pub const fn advance(&mut self, _opportunity: QualifiedOpportunity) -> PlayTime {
        let mut next = self.time;
        if let Some(frames) = PlayFrames::new(next.frames.get().wrapping_add(1)) {
            next.frames = frames;
        } else if let Some(seconds) = PlaySeconds::new(next.seconds.get().wrapping_add(1)) {
            next.frames = PlayFrames::ZERO;
            next.seconds = seconds;
        } else if let Some(minutes) = PlayMinutes::new(next.minutes.get().wrapping_add(1)) {
            next.frames = PlayFrames::ZERO;
            next.seconds = PlaySeconds::ZERO;
            next.minutes = minutes;
        } else if let Some(hours) = PlayHours::new(next.hours.get().wrapping_add(1)) {
            next.frames = PlayFrames::ZERO;
            next.seconds = PlaySeconds::ZERO;
            next.minutes = PlayMinutes::ZERO;
            next.hours = hours;
        }
        self.time = next;
        self.time
    }
}

/// Identifies a rule in the game-time specification.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, enum_map::Enum)]
pub enum BehaviourId {
    /// Earn one frame on an ordinary opportunity, up to the maximum.
    PlayTimeAdvance,
}

/// Metadata for one game-time rule.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Behaviour {
    /// The rule's specification entry.
    pub id: BehaviourId,
    /// What the player sees.
    pub summary: &'static str,
    /// The kind of rule.
    pub kind: Kind,
    /// Whether and how the rule can be changed.
    pub switch: Switch,
    /// How the rule is checked.
    pub checked: Checked,
}

/// The game-time catalogue. The ordinary increment is checked by the recorded oracle vector;
/// carry and saturation have constructed vectors and are checked against the original only.
pub const BEHAVIOURS: &[Behaviour] = &[Behaviour {
    id: BehaviourId::PlayTimeAdvance,
    summary: "one ordinary opportunity earns one frame on the clock, capped at the maximum.",
    kind: Kind::Mechanic,
    switch: Switch::Fixed,
    checked: Checked::ByVectors,
}];
