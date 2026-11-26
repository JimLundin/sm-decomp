//! Spec: `ShipApproachesPlanet`. The ship drifts right while it grows, banks left, then
//! recedes as it drifts left until it disappears.
//!
//! Every frame of the flight drifts the plane first. Each leg then tests the zoom as it was
//! before this frame's increment; the bank ends at its horizontal limit after its drift.

use types::zebes::{FlightView, PlaneAngle, PlanePose, PlaneZoom, ZebesStage, ZebesState};

use crate::hold;
use crate::rules::Rules;

/// Spec: `ShipApproachesPlanet`. The downward shift every frame: +1/8 px, in 65,536ths.
const SHIFT_Y: i32 = 8_192;
/// Spec: `ShipApproachesPlanet`. The shift along x while `Rightward` or `Banking`: −1/2 px,
/// in 65,536ths.
const SHIFT_X_APPROACHING: i32 = -32_768;
/// Spec: `ShipApproachesPlanet`. The shift along x while `Leftward`: +1/8 px, in 65,536ths.
const SHIFT_X_RECEDING: i32 = 8_192;

/// Spec: `ShipApproachesPlanet` [R-right]. The zoom where the rightward leg ends, 1,152.
const BANK_ZOOM: u16 = 1_152;
/// Spec: `ShipApproachesPlanet` [R-right]. The zoom added each rightward frame.
const RIGHTWARD_ZOOM_STEP: u16 = 4;
/// Spec: `ShipApproachesPlanet` [R-bank]. The zoom added each banking frame.
const BANKING_ZOOM_STEP: u16 = 16;
/// Spec: `ShipApproachesPlanet` [R-bank-held]. The angle the bank holds at, 224.
const BANKED_ANGLE: PlaneAngle = PlaneAngle(224);
/// Spec: `ShipApproachesPlanet` [R-left-starts]. The bank's horizontal limit, −128 px: the
/// bank ends at this limit after its drift, once the ship falls left of it, and the leftward
/// leg starts.
const LEFT_BOUND: i16 = -128;
/// Spec: `ShipApproachesPlanet` [R-left]. The zoom added each leftward frame.
const LEFTWARD_ZOOM_STEP: u16 = 32;
/// Spec: `ShipApproachesPlanet` [R-hold-starts]. The zoom where the ship disappears, 8,192.
const VANISH_ZOOM: u16 = 8_192;

/// The plane drifted by one frame's shift along x and +1/8 px along y.
const fn drift(pose: &mut PlanePose, shift_x: i32) {
    pose.scroll.x = pose.scroll.x.moved(shift_x);
    pose.scroll.y = pose.scroll.y.moved(SHIFT_Y);
}

/// The zoom grown by a step.
const fn grow(zoom: &mut PlaneZoom, step: u16) {
    zoom.0 = zoom.0.saturating_add(step);
}

/// Spec: `ShipApproachesPlanet` [R-right, R-bank-starts]. Drift; grow by 4 while the zoom is
/// below 1,152, then start banking.
pub const fn rightward(state: &mut ZebesState) {
    let zoom = state.pose.zoom.0;
    drift(&mut state.pose, SHIFT_X_APPROACHING);
    if zoom < BANK_ZOOM {
        grow(&mut state.pose.zoom, RIGHTWARD_ZOOM_STEP);
    } else {
        state.stage = ZebesStage::Banking;
    }
}

/// Spec: `ShipApproachesPlanet` [R-bank, C-turn-wrap, R-bank-held, R-left-starts]. Drift;
/// while the ship's horizontal position after the drift, in whole pixels, is still at or
/// right of the left bound (−128 px), grow by 16 and turn one 256th anticlockwise, wrapping,
/// until the angle reaches 224. Once the position falls left of the bound, start the
/// leftward leg.
pub const fn banking(state: &mut ZebesState) {
    drift(&mut state.pose, SHIFT_X_APPROACHING);
    let horizontal_position = state.pose.scroll.x.pixel;
    if horizontal_position < LEFT_BOUND {
        state.stage = ZebesStage::Leftward;
        return;
    }
    grow(&mut state.pose.zoom, BANKING_ZOOM_STEP);
    if state.pose.angle.0 != BANKED_ANGLE.0 {
        state.pose.angle = PlaneAngle(state.pose.angle.0.wrapping_sub(1));
    }
}

/// Spec: `ShipApproachesPlanet` [R-left, R-hold-starts]. Drift; grow by 32 while the zoom is
/// below 8,192, then hide the ship plane and start the hold.
pub const fn leftward(state: &mut ZebesState, rules: Rules) {
    let zoom = state.pose.zoom.0;
    drift(&mut state.pose, SHIFT_X_RECEDING);
    if zoom < VANISH_ZOOM {
        grow(&mut state.pose.zoom, LEFTWARD_ZOOM_STEP);
    } else {
        state.view = FlightView::SpritesOnly;
        state.stage = hold::hold_begins(rules);
    }
}
