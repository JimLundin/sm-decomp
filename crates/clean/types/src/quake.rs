//! Room quake state carried between the scheduler, quake model and Drawing.

use serde::{Deserialize, Serialize};

/// Spec: `QuietQuakePhase`. Elapsed quake opportunities around a 65,536-step cycle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuakePhase(pub u16);

/// Spec: `QuietQuakePhase`. Quake time remaining in frames, independent of the phase.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuakeDuration(pub u16);

/// Spec: `QuietQuakePhase`. Complete room quake state to carry or restore.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoomQuakeState {
    /// The carried phase, including opportunities when the room stayed steady.
    pub phase: QuakePhase,
    /// The remaining time; only zero admits the quiet update.
    pub remaining: QuakeDuration,
}

impl RoomQuakeState {
    /// Spec: `QuietQuakePhase`. Only remaining duration, not phase, determines activity.
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.remaining.0 != 0
    }
}
