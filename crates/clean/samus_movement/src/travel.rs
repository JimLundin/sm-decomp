//! Spec: `TravelledDistances`. Caller-held distances outlive movement until replaced or refreshed.

use types::{Direction, DirectionalTravels, TravelDistance};

/// Spec: `TravelledDistances`. The kind of completed travel, distinct from position adjustments.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TravelKind {
    /// An ordinary horizontal move, including one made by the dash alone.
    Walk,
    /// An ordinary vertical move by speed, ground pull or elevator descent.
    Vertical,
    /// An aim change's forward step, which preserves ordinary distances.
    AimStep,
    /// Fitting, shrinking, lifting, seating or alignment, which preserves ordinary distances.
    Placement,
    /// A near-wall or pose-fit look, which preserves ordinary distances.
    Look,
}

/// Spec: `TravelledDistances`. A complete caller-held movement interval.
///
/// Firing reads this before a new interval starts. Skipping an interval requires no call:
/// keep this value. Each direction can be restored independently to any wrapping signed value.
/// Serde derives preserve the complete `travels` field; its `EnumMap` serializes every named
/// direction. `Default` starts the same all-zero interval as `fresh`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TravelledDistances {
    /// All four latest signed accepted travels, including their fractions.
    pub travels: DirectionalTravels,
}

impl Default for TravelledDistances {
    /// Spec: `TravelledDistances`. A new interval has exactly the all-zero values of `fresh`.
    fn default() -> Self {
        Self::fresh()
    }
}

impl TravelledDistances {
    /// Spec: `TravelledDistances`. Start a fresh interval with all directions zero.
    #[must_use]
    pub fn fresh() -> Self {
        Self {
            travels: DirectionalTravels::from_fn(|_| TravelDistance::ZERO),
        }
    }

    /// Spec: `TravelledDistances`. Refresh only after the previous values were available to firing.
    pub fn refresh(&mut self) {
        *self = Self::fresh();
    }

    /// Spec: `TravelledDistances`. An ordinary completed travel replaces its direction, even at zero.
    ///
    /// Adjustments and looks preserve every direction. `accepted` excludes independent subpixel snaps.
    pub fn record(&mut self, kind: TravelKind, direction: Direction, accepted: TravelDistance) {
        if matches!(kind, TravelKind::Walk | TravelKind::Vertical) {
            self.travels[direction] = accepted;
        }
    }
}
