//! The station's slow drift on the affine plane, shared by its reveal and its recession.

use types::Pair;
use types::explosion::{PlanePose, PlaneZoom, ScreenPoint, Subpixels};

/// Spec: `StationBrightens`, `StationBreaks`. The station's drift each frame: 1/4 px left and
/// 1/16 px down.
const STATION_DRIFT: ScreenPoint = Pair {
    x: Subpixels::fraction(-1, 4),
    y: Subpixels::fraction(1, 16),
};

/// Spec: `StationBrightens`, `StationBreaks`. Moves the plane's scroll by the station's drift.
pub const fn drift(pose: &mut PlanePose) {
    pose.scroll = pose.scroll.plus(STATION_DRIFT);
}

/// Spec: `StationBrightens`, `StationBreaks`. Adds 1 to the zoom: the station recedes.
pub const fn recede(pose: &mut PlanePose) {
    pose.zoom = PlaneZoom(pose.zoom.0.saturating_add(1));
}
