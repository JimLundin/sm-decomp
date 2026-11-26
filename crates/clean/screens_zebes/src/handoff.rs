//! Spec: `LandingSiteLoadBegins`. After the marked star field leaves, the scene blacks out and
//! hands control to the Landing Site load; Samus's energy returns to its maximum.
//!
//! T1 keeps the destination context through the flight and passes it to the Loader (R2) on
//! the next scheduled frame; the scene only asks.

use types::zebes::{Brightness, PixelPoint, PlacedPictures, PlanePose, ZebesStage, ZebesState};

use crate::report::{BeginLandingSiteLoad, Handoff, Requests, RestoreFullEnergy};

/// Spec: `LandingSiteLoadBegins`. The cleared plane: angle 0, zoom 0, unscrolled.
const CLEARED_POSE: PlanePose = PlanePose::new(0, 0, PixelPoint { x: 0, y: 0 });

/// Spec: `LandingSiteLoadBegins` [R-handoff, C-handoff]. The final cut: blank the display,
/// clear the pose and pictures, keep the drawing selection and mosaic, and publish both
/// hand-off outputs on this cut.
#[must_use]
pub fn leave(state: &mut ZebesState) -> Requests {
    state.light = Brightness::Blanked;
    state.pose = CLEARED_POSE;
    state.pictures = PlacedPictures::default();
    state.stage = ZebesStage::Complete;
    Requests {
        handoff: Some(Handoff {
            health: RestoreFullEnergy,
            landing_site: BeginLandingSiteLoad,
        }),
        ..Requests::NONE
    }
}
