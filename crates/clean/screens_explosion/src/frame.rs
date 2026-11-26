//! One scene frame: the stage's rule, then the sky and the bursts.

use types::explosion::{ExplosionScene, ExplosionStage, ExplosionState};

use crate::events::SceneEvent;
use crate::owners::{MusicService, Scheduler};
use crate::rules::Rules;
use crate::{bursts, departure, opening, sky, station};

/// Runs one scene frame and returns what it asks of the other owners, in order.
///
/// A stalled frame holds the last picture and asks nothing. A finished frame applies its
/// stage's rule first; then, from the placement frame through the handoff cut, the sky
/// advances, the bursts due at the new picture age launch from the plane's drifted scroll, and
/// every live burst updates.
pub fn step(
    state: &mut ExplosionState,
    scheduler: &impl Scheduler,
    music: &impl MusicService,
    scene: &ExplosionScene,
    rules: Rules,
) -> Vec<SceneEvent> {
    if !scheduler.frame_finished() {
        return Vec::new();
    }
    let started = state.stage;
    let events = apply_stage(state, scheduler, music, scene, rules);
    if pictures_move(started, state.stage) {
        sky::advance(state, &scene.sky);
        bursts::launch_scheduled(state, scene.pivot);
        bursts::update(&mut state.bursts, &scene.tracks);
    }
    events
}

/// The rule of the stage the frame starts in.
fn apply_stage(
    state: &mut ExplosionState,
    scheduler: &impl Scheduler,
    music: &impl MusicService,
    scene: &ExplosionScene,
    rules: Rules,
) -> Vec<SceneEvent> {
    match state.stage {
        ExplosionStage::Pending if scheduler.setup_complete() => opening::prepare(state, scene),
        ExplosionStage::WaitingForMusic => {
            opening::await_music(state, music);
            Vec::new()
        }
        ExplosionStage::Revealing => {
            opening::brighten(state);
            Vec::new()
        }
        ExplosionStage::Station => station::recede_or_break(state, scene),
        ExplosionStage::Departing => {
            departure::depart(state, rules);
            Vec::new()
        }
        ExplosionStage::Holding(left) => {
            departure::hold(state, left);
            Vec::new()
        }
        ExplosionStage::Dimming => departure::black_out(state),
        ExplosionStage::Pending | ExplosionStage::Complete => Vec::new(),
    }
}

/// Spec: `SpaceDrifts`. Whether the sky and the bursts move on this frame: once the pictures
/// are placed, through the cut that completes the scene, and not after.
const fn pictures_move(started: ExplosionStage, now: ExplosionStage) -> bool {
    !matches!(started, ExplosionStage::Complete) && !matches!(now, ExplosionStage::Pending)
}
