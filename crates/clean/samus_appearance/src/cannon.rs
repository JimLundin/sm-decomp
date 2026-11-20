//! Persistent cannon readiness, independent of animation and HUD mode.

use serde::{Deserialize, Serialize};

use crate::state::State;

/// Spec: `CannonItemDelay`. Item identity from actual controls, not HUD one-shot mode
/// or a switch event. Selecting and firing these items remain the HUD/weapon owners' work.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SelectedItem {
    /// No selected item.
    None,
    /// Missiles selected.
    Missiles,
    /// Super missiles selected.
    SuperMissiles,
    /// Power bombs selected.
    PowerBombs,
    /// X-ray scope selected.
    Xray,
    /// Grapple beam selected.
    Grapple,
}

/// Spec: `CannonItemDelay`. Only performed eligible controls advance readiness;
/// frozen time alone is not ineligibility.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControlEligibility {
    /// The HUD can switch at these actual controls.
    Eligible,
    /// The HUD cannot switch; readiness remains untouched.
    Ineligible,
}

/// Spec: `RetainedCannonReadiness`. The actual chosen control answer carries its
/// starting and resulting identities and eligibility, independently of later selections.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemControlAnswer {
    /// Identity immediately before the actual controls.
    pub before: SelectedItem,
    /// Identity chosen by those controls, before any later weapon drop.
    pub after: SelectedItem,
    /// The HUD's can-switch answer for this observation.
    pub eligibility: ControlEligibility,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
enum Remaining {
    Ready,
    One,
    Two,
}

/// Spec: `CannonItemDelay`, `CannonReadinessReset`. A bounded delay of 0..=2
/// *eligible item-control observations*, not frames.
///
/// Zero permits following the
/// selection; it does not mean the cannon has finished opening or closing.
/// The finite representation also keeps deserialized values in the bounded domain.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CannonDelay(Remaining);

impl CannonDelay {
    /// Spec: `CannonReadinessReset`. Full Samus initialization (Ceres, Zebes or
    /// demo) starts with two observations remaining. Room passage is not initialization.
    pub const INITIAL: Self = Self(Remaining::Two);

    /// Spec: `CannonItemDelay`. Construct any allowed number of observations remaining.
    #[must_use]
    pub const fn new(observations: u8) -> Option<Self> {
        match observations {
            0 => Some(Self(Remaining::Ready)),
            1 => Some(Self(Remaining::One)),
            INITIAL_OBSERVATIONS => Some(Self::INITIAL),
            _ => None,
        }
    }

    /// Spec: `RetainedCannonReadiness`. Inspection has no effects.
    #[must_use]
    pub const fn remaining(self) -> u8 {
        match self.0 {
            Remaining::Ready => 0,
            Remaining::One => 1,
            Remaining::Two => INITIAL_OBSERVATIONS,
        }
    }

    /// Spec: `RetainedCannonReadiness`. A read-only permission for downstream drawing.
    #[must_use]
    pub const fn may_follow(self) -> bool {
        matches!(self.0, Remaining::Ready)
    }

    /// Spec: `CannonItemDelay`. Call once when the actual control effects complete.
    /// Changed eligible identity starts one observation's wait; unchanged eligible
    /// identity reduces it towards zero. Ineligible controls leave it untouched.
    /// Skipped controls, drawing and outside selection changes must not call this.
    pub fn controls_completed(&mut self, answer: ItemControlAnswer) {
        if answer.eligibility == ControlEligibility::Ineligible {
            return;
        }
        if answer.before != answer.after {
            self.0 = Remaining::One;
            return;
        }
        self.0 = match self.0 {
            Remaining::Ready | Remaining::One => Remaining::Ready,
            Remaining::Two => Remaining::One,
        };
    }

    /// Spec: `CannonReadinessReset`. Reset only at actual full initialization.
    pub const fn initialize(&mut self) {
        *self = Self::INITIAL;
    }
}

const INITIAL_OBSERVATIONS: u8 = 2;

/// Spec: `RetainedCannonReadiness`. The persistent animation and cannon portion of the look,
/// with the four animation quantities unchanged and independent cannon readiness included.
///
/// Serialize this owner, rather than reconstructing readiness from animation or HUD State.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Look {
    /// The existing animation playhead, boost stage and laps.
    pub animation: State,
    /// Independent persistent cannon readiness.
    pub cannon: CannonDelay,
}

/// Spec: `RetainedCannonReadiness`. One actual observation's lifecycle, retained
/// alongside its complete look. No observation is created for skipped controls.
///
/// Completion here is model completion, not external delivery acknowledgement.
/// A caller saves this progress and its own input/read/output delivery positions;
/// reading readiness neither consumes this progress nor publishes sounds or graphics.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CannonObservation {
    /// Actual controls have not chosen their answer yet.
    Unmade,
    /// Actual answer retained, with no readiness effects applied yet.
    Chosen(ItemControlAnswer),
    /// Actual effects consumed once; the answer is retained for inspection.
    Completed(ItemControlAnswer),
}

impl CannonObservation {
    /// Spec: `RetainedCannonReadiness`. Retain the actual answer only if unmade.
    /// Later unrelated inputs cannot replace a chosen or completed answer.
    /// Returns whether this call made the choice; readiness is still unchanged.
    pub fn choose(&mut self, answer: ItemControlAnswer) -> bool {
        if *self != Self::Unmade {
            return false;
        }
        *self = Self::Chosen(answer);
        true
    }

    /// Spec: `RetainedCannonReadiness`. Apply once at actual control completion.
    /// An unmade or completed observation does nothing; returns whether effects ran.
    pub fn complete(&mut self, look: &mut Look) -> bool {
        let Self::Chosen(answer) = *self else {
            return false;
        };
        look.cannon.controls_completed(answer);
        *self = Self::Completed(answer);
        true
    }
}
