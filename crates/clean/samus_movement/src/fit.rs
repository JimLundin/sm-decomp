//! Spec: `PoseFitAfterMoves`. Semantic outcomes of ordinary body consideration.
use core::num::Wrapping;
use types::{Action, Aim, Body, Facing, Gun, Legs, Pose, Px};
/// Spec: `PoseFitAfterMoves`. Tunnel substitution keeps facing and stows the gun.
pub const fn crouched(facing: Facing) -> Pose {
    Pose {
        body: Body::Crouched,
        action: Action::Still,
        facing,
        aim: Aim::Forward,
        gun: Gun::Stowed,
        legs: Legs::Together,
    }
}
/// Spec: `PoseFitAfterMoves`. Fit answer, retained before its placement effect.
#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub enum Fitted {
    /// Target accepted without placement.
    Taken,
    /// Target accepted with a push away from contact.
    Pushed,
    /// Target rejected, with no target effects.
    Refused,
    /// Tunnel substitutes crouched Still.
    Crouched {
        /// Whole pixels raised to retain the feet position.
        rise: Px,
    },
}
impl Fitted {
    /// Spec: `PoseFitAfterMoves`. Only an actual push or nonzero crouch lift places her.
    #[must_use]
    pub fn moved(self) -> bool {
        match self {
            Self::Pushed => true,
            Self::Crouched { rise } => rise != Wrapping(0),
            Self::Taken | Self::Refused => false,
        }
    }
}
