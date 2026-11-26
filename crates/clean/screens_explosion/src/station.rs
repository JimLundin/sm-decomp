//! Spec: `StationBreaks`. The station recedes until its zoom reaches the limit; then a flash
//! covers it, the final burst goes off and the plane shows the gunship.

use types::explosion::{ExplosionScene, ExplosionStage, ExplosionState, PlaneView, PlaneZoom};

use crate::events::{MapRegion, SceneEvent};
use crate::{bursts, plane};

/// Spec: `StationBreaks`. The station recedes while its zoom is below 640.
const STATION_ZOOM_LIMIT: PlaneZoom = PlaneZoom(640);
/// Spec: `StationBreaks`. The gunship's zoom as it departs.
const DEPARTING_ZOOM: PlaneZoom = PlaneZoom(768);

/// Spec: `StationBreaks`. The station drifts; below the zoom limit (read at the frame's start)
/// it recedes, and from the limit on it breaks up.
pub fn recede_or_break(state: &mut ExplosionState, scene: &ExplosionScene) -> Vec<SceneEvent> {
    let start_zoom = state.pose.zoom;
    plane::drift(&mut state.pose);
    if start_zoom < STATION_ZOOM_LIMIT {
        plane::recede(&mut state.pose);
        return Vec::new();
    }
    break_up(state, scene)
}

/// Spec: `StationBreaks`.
/// The drift locates the flash before the final burst appears.
fn break_up(state: &mut ExplosionState, scene: &ExplosionScene) -> Vec<SceneEvent> {
    let center = bursts::screen_anchor(scene.pivot, state.pose.scroll);
    bursts::launch_final(&mut state.bursts, center);
    state.pose.zoom = DEPARTING_ZOOM;
    state.view = PlaneView::Gunship;
    state.stage = ExplosionStage::Departing;
    vec![
        SceneEvent::StartStationFlash {
            style: scene.flash_style.clone(),
            center,
        },
        SceneEvent::InstallMapDrawing {
            drawing: scene.ship_front.clone(),
            region: MapRegion::Upper,
        },
        SceneEvent::InstallMapDrawing {
            drawing: scene.empty_drawing.clone(),
            region: MapRegion::Lower,
        },
    ]
}
