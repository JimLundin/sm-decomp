//! Spec: `PlanetHolds`. Once the ship disappears the planet and stars stay still for the
//! adjustable `hold_frames`, then begin to slide.

use types::zebes::{HoldFrames, ZebesStage};

use crate::rules::Rules;

/// Spec: `PlanetHolds` [R-hold-starts, C-short-hold]. The ship disappears: the hold starts
/// with `hold_frames` left.
#[must_use]
pub const fn hold_begins(rules: Rules) -> ZebesStage {
    ZebesStage::Still {
        hold: rules.hold_frames,
    }
}

/// Spec: `PlanetHolds` [R-hold-ticks, R-slide-starts]. One frame off the hold; on its last
/// frame the slide starts, and the pictures first move on the frame after.
#[must_use]
pub const fn hold(left: HoldFrames) -> ZebesStage {
    match left.counted_down() {
        Some(hold) => ZebesStage::Still { hold },
        None => ZebesStage::Sliding,
    }
}
