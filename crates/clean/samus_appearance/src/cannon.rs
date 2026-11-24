//! When her cannon may follow the selected item: a delay of its own, apart from her animation
//! and the HUD's mode.

use serde::{Deserialize, Serialize};

/// Spec: `CannonItemDelay`. A selected item's identity, as the item controls start from it and
/// as they leave it.
///
/// It is the same whatever the HUD's one-shot mode and its switch event. Selecting and firing
/// these items are the HUD's and the weapons' work.
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

/// Spec: `CannonItemDelay`. Whether the item controls that ran may switch (the HUD's `can
/// switch`), with time frozen or running alike.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControlEligibility {
    /// The HUD can switch: the controls count towards readiness.
    Eligible,
    /// The HUD cannot switch: the delay stays as it is.
    Ineligible,
}

/// Spec: `CannonItemDelay`. What a frame's item controls answer: the selected item they start
/// from, the one they leave before any later weapon drop, and whether they were eligible.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemControlAnswer {
    /// The identity immediately before the controls.
    pub before: SelectedItem,
    /// The identity the controls leave.
    pub after: SelectedItem,
    /// The HUD's `can switch`.
    pub eligibility: ControlEligibility,
}

/// Spec: `CannonItemDelay`. The frame's part of her item controls: whether they run, and their
/// answer when they do.
///
/// A frame whose controls are skipped leaves the delay alone, as does a selection change outside
/// item controls, a pose or clip change, a repeated drawing, a lag frame and a passage that
/// moves her to another room without initializing her.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ItemControls {
    /// Her item controls don't run this frame: skipped, facing the screen, or controlled by
    /// another domain.
    Skipped,
    /// Her item controls ran, with this answer.
    Ran(ItemControlAnswer),
}

/// The frames of eligible item controls left, one of three.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
enum Remaining {
    Ready,
    One,
    Two,
}

/// Spec: `CannonItemDelay`, `CannonReadinessReset`. The cannon selection delay: 0 to 2 frames
/// of eligible item controls left before her cannon may follow the selected item.
///
/// It is persistent appearance memory of its own, kept apart from the selected item, her gun
/// and her animation. At 0 the cannon may begin responding to the selection. It holds one of
/// the three values on construction and on restore alike.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CannonDelay(Remaining);

/// Spec: `CannonReadinessReset`. A newly initialized Samus waits two frames of eligible
/// unchanged item controls.
const INITIAL_FRAMES: u8 = 2;

impl CannonDelay {
    /// Spec: `CannonReadinessReset`. The delay of a newly initialized Samus: 2.
    pub const INITIAL: Self = Self(Remaining::Two);

    /// The delay of `frames` frames of eligible item controls left, if it is 0 to 2.
    #[must_use]
    pub const fn new(frames: u8) -> Option<Self> {
        match frames {
            0 => Some(Self(Remaining::Ready)),
            1 => Some(Self(Remaining::One)),
            INITIAL_FRAMES => Some(Self::INITIAL),
            _ => None,
        }
    }

    /// The frames of eligible item controls left.
    #[must_use]
    pub const fn remaining(self) -> u8 {
        match self.0 {
            Remaining::Ready => 0,
            Remaining::One => 1,
            Remaining::Two => INITIAL_FRAMES,
        }
    }

    /// Whether her cannon may follow the selected item: the delay is 0.
    #[must_use]
    pub const fn may_follow(self) -> bool {
        matches!(self.0, Remaining::Ready)
    }

    /// Spec: `CannonItemDelay`. The delay as the frame's item controls leave it, before her gun's
    /// picture reads it that frame. Eligible controls that leave another identity than they
    /// started from make it 1, a change to none and a change during a wait included. Eligible
    /// controls that leave the same identity bring it down by 1, to no less than 0, whatever the
    /// one-shot mode or the switch event. Ineligible or skipped controls leave it as it is.
    pub(crate) fn after(self, controls: ItemControls) -> Self {
        let answer = match controls {
            ItemControls::Skipped => return self,
            ItemControls::Ran(answer) => answer,
        };
        if answer.eligibility == ControlEligibility::Ineligible {
            return self;
        }
        if answer.before != answer.after {
            return Self(Remaining::One);
        }
        Self(match self.0 {
            Remaining::Ready | Remaining::One => Remaining::Ready,
            Remaining::Two => Remaining::One,
        })
    }
}
