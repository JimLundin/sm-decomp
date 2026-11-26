//! The opening: the station's picture arrives in the dark, waits for the music, then drifts
//! into view and brightens.

use types::Pair;
use types::explosion::{
    ExplosionScene, ExplosionStage, ExplosionState, FadeWait, Light, PlaneAngle, PlanePose,
    PlaneView, PlaneZoom, Subpixels,
};

use crate::events::{MusicDelay, MusicRequest, SceneEvent};
use crate::owners::MusicService;
use crate::{plane, sky};

/// Spec: `StationPictureArrives`. The plane's first pose: square, at its size, scrolled
/// (−44, −112) px.
const INITIAL_POSE: PlanePose = PlanePose {
    angle: PlaneAngle(0),
    zoom: PlaneZoom(256),
    scroll: Pair {
        x: Subpixels::whole(-44),
        y: Subpixels::whole(-112),
    },
};

/// Spec: `StationPictureArrives`. The song requests of the preparation, in order, with their
/// delays.
const SETUP_MUSIC: [SceneEvent; 3] = [
    SceneEvent::RequestMusic {
        request: MusicRequest::Stop,
        delay: MusicDelay(8),
    },
    SceneEvent::RequestMusic {
        request: MusicRequest::CeresMusicData,
        delay: MusicDelay(8),
    },
    SceneEvent::RequestMusic {
        request: MusicRequest::ZebesFlightSong,
        delay: MusicDelay(14),
    },
];

/// Spec: `StationPictureArrives`. Prepares the scene once the scheduler's setup completes: the
/// art is installed, the plane shows the station at its first pose, the sky starts, the space
/// palette and its navigation light are replaced and the music is requested. The view stays
/// blank, the fade holds for one beat, and the scene waits, prepared and dark, for the music.
pub fn prepare(state: &mut ExplosionState, scene: &ExplosionScene) -> Vec<SceneEvent> {
    state.view = PlaneView::Station;
    state.pose = INITIAL_POSE;
    sky::place(state, &scene.sky);
    state.light = Light::Blanked;
    state.fade = FadeWait::OneFrame;
    state.stage = ExplosionStage::WaitingForMusic;
    let mut events = vec![
        SceneEvent::InstallArt(scene.art.clone()),
        SceneEvent::ReplaceSpacePalette(scene.space_palette.clone()),
    ];
    events.extend(SETUP_MUSIC);
    events
}

/// Spec: `StationAwaitsMusic`. The station stays dark while the music is busy; once it is
/// ready the reveal begins, pose and light as they are.
pub fn await_music(state: &mut ExplosionState, music: &impl MusicService) {
    if !music.music_busy() {
        state.stage = ExplosionStage::Revealing;
    }
}

/// Spec: `StationBrightens`. The station drifts and recedes, then the light steps: a frame
/// that waits only clears the wait; otherwise the light rises a level and the next frame waits.
/// At full light the reveal ends and the station stage begins.
pub fn brighten(state: &mut ExplosionState) {
    plane::drift(&mut state.pose);
    plane::recede(&mut state.pose);
    if state.fade == FadeWait::OneFrame {
        state.fade = FadeWait::NoFrame;
        return;
    }
    state.light = state.light.brighter();
    if state.light == Light::FULL {
        state.fade = FadeWait::NoFrame;
        state.stage = ExplosionStage::Station;
    } else {
        state.fade = FadeWait::OneFrame;
    }
}
