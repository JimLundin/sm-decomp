//! The end of the scene: the gunship turns and recedes, space holds, then dims to black and
//! hands over to the flight to Zebes.

use types::explosion::{
    ExplosionStage, ExplosionState, FadeWait, HoldFrames, Light, PlaneAngle, PlaneZoom, Subpixels,
};

use crate::events::SceneEvent;
use crate::rules::Rules;

/// Spec: `ShipDeparts`. The gunship's drift each frame: 2 px right.
const DEPARTURE_DRIFT: Subpixels = Subpixels::whole(2);
/// Spec: `ShipDeparts`. How much the zoom falls each frame.
const DEPARTURE_ZOOM_STEP: u16 = 16;

/// Spec: `ShipDeparts`. The gunship drifts and turns back one 256th of a turn, wrapping. While
/// its zoom (read at the frame's start) holds a whole step, the zoom falls by it; once it
/// doesn't, the zoom stays and space holds for the rules' hold.
pub const fn depart(state: &mut ExplosionState, rules: Rules) {
    let pose = &mut state.pose;
    pose.scroll.x = pose.scroll.x.plus(DEPARTURE_DRIFT);
    pose.angle = PlaneAngle(pose.angle.0.wrapping_sub(1));
    match pose.zoom.0.checked_sub(DEPARTURE_ZOOM_STEP) {
        Some(zoom) => pose.zoom = PlaneZoom(zoom),
        None => state.stage = ExplosionStage::Holding(rules.hold_frames),
    }
}

/// Spec: `SpaceHolds`.
/// Begin the departure and hold the fade for a beat.
pub const fn hold(state: &mut ExplosionState, left: HoldFrames) {
    if let Some(left) = left.counted_down() {
        state.stage = ExplosionStage::Holding(left);
    } else {
        state.stage = ExplosionStage::Dimming;
        state.light = Light::FULL;
        state.fade = FadeWait::OneFrame;
    }
}

/// Spec: `SceneBlacksOut`. A frame that waits only clears the wait; otherwise the light falls a
/// level and the next frame waits. Below level 1 the display blanks, the scene completes with
/// its plane and sprites kept under the blank, and the flight to Zebes begins on this cut.
pub fn black_out(state: &mut ExplosionState) -> Vec<SceneEvent> {
    if state.fade == FadeWait::OneFrame {
        state.fade = FadeWait::NoFrame;
        return Vec::new();
    }
    state.light = state.light.dimmer();
    if state.light == Light::Blanked {
        state.stage = ExplosionStage::Complete;
        state.fade = FadeWait::NoFrame;
        return vec![SceneEvent::BeginZebesFlight];
    }
    state.fade = FadeWait::OneFrame;
    Vec::new()
}
