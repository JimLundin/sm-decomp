//! The flight's stages, each frame's step from where the scene has got to: made ready in the
//! dark, the picture appearing, the rush in and the turn, the flight away, the station's view and
//! its label, and the fade out to the end.

use core::num::Wrapping;

use types::flight::{
    Angle, Brightness, BrightnessLevel, DriftingSprite, FadeFrames, FlightSprites, FlightStage,
    Keystroke, LabelProgress, LabelStep, Plane, Scene, SceneSetup, SceneTurn, StarRush, State,
    TypedLetter, Visibility, Zoom, place_at,
};
use types::{Colour, Fixed, Pair, Parts, ZERO};

use crate::rules::Rules;

/// Whether a track waits to play in audio's queue of tracks as the frame's step runs: audio's
/// reply, borrowed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MusicQueue {
    /// A track still waits: the flight waits in the dark.
    TrackWaits,
    /// No track waits.
    Clear,
}

/// Spec: `GunshipRushesIn`. The zoom falls by this each frame of the rush.
const RUSH_ZOOM_STEP: i16 = -16;
/// Spec: `GunshipRushesIn`. Once the zoom is under this, the view turns.
const TURN_ZOOM: u16 = 32;

/// Spec: `GunshipFliesAway`. A band of zooms below `below`, in which the zoom grows by `step`
/// each frame.
struct ZoomBand {
    /// The band holds zooms under this.
    below: u16,
    /// How much the zoom grows each frame within it.
    step: i16,
}

/// Spec: `GunshipFliesAway`. The gunship shrinks slowly at first, then twice as fast; past the
/// last band the station comes into view.
const AWAY_BANDS: [ZoomBand; 2] = [
    ZoomBand {
        below: 3072,
        step: 16,
    },
    ZoomBand {
        below: 8192,
        step: 32,
    },
];
/// Spec: `GunshipFliesAway`. The plane drifts left by an eighth of a pixel each frame.
const PLANE_DRIFT_LEFT: Fixed = Wrapping(8192);

/// Spec: `ViewTurnsAway`. The white flash at the turn.
const FLASH_WHITE: Colour = Colour::from_channels([31, 31, 31]);

/// The plane put back square, at its size and at `(0, 0)` (`StationComesIntoView`).
const PLANE_AT_REST: Plane = Plane {
    angle: Angle::SQUARE,
    zoom: Zoom::AT_SIZE,
    position: ORIGIN,
};
/// The plane cleared (`FlightEnds`).
const PLANE_CLEARED: Plane = Plane {
    angle: Angle::SQUARE,
    zoom: Zoom::NONE,
    position: ORIGIN,
};
/// The screen's top-left corner, `(0.0, 0.0)`.
const ORIGIN: Pair<Fixed> = Pair { x: ZERO, y: ZERO };

/// The stage's step for this frame; the letter the label types as a step begins, if any. Only
/// the flight away (which may bring the station's view) and the label type letters. Once done,
/// nothing of the flight changes here.
pub fn advance(
    state: &mut State,
    scene: &Scene,
    music: MusicQueue,
    rules: Rules,
) -> Option<TypedLetter> {
    match state.stage {
        FlightStage::FlyingAway => fly_away(state, scene, rules),
        FlightStage::Label(progress) => type_label(state, progress, &scene.label, rules),
        FlightStage::Starting => {
            make_ready(state, &scene.setup);
            None
        }
        FlightStage::WaitingForMusic => {
            appear(state, music);
            None
        }
        FlightStage::RushingIn => {
            rush_in(state, &scene.turn);
            None
        }
        FlightStage::FadingOut => {
            fade_out(state, rules);
            None
        }
        FlightStage::Done => None,
    }
}

/// Spec: `FlightIsMadeReady`. The plane as the scene makes it ready, the rushing stars alone
/// placed, the flash black and the screen dark; the fade's count is left.
fn make_ready(state: &mut State, setup: &SceneSetup) {
    state.stage = FlightStage::WaitingForMusic;
    state.plane = Plane {
        angle: setup.angle,
        zoom: setup.zoom,
        position: place_at(setup.plane),
    };
    state.sprites = FlightSprites {
        rushing_stars: Some(StarRush {
            place: place_at(setup.stars),
            speed: setup.speed,
        }),
        ..FlightSprites::default()
    };
    state.flash = Colour::BLACK;
    state.brightness = Brightness::DARK;
}

/// Spec: `PictureAppears`. While a track waits nothing changes; once none does, the picture
/// comes on at full brightness and the rush begins.
const fn appear(state: &mut State, music: MusicQueue) {
    match music {
        MusicQueue::TrackWaits => {}
        MusicQueue::Clear => {
            state.stage = FlightStage::RushingIn;
            state.brightness = Brightness::FULL;
        }
    }
}

/// Spec: `GunshipRushesIn`. The zoom falls by 16 while it is at least 32; under 32 the view
/// turns.
fn rush_in(state: &mut State, turn: &SceneTurn) {
    if state.plane.zoom.get() >= TURN_ZOOM {
        state.plane.zoom = state.plane.zoom.shifted(RUSH_ZOOM_STEP);
    } else {
        turn_away(state, turn);
    }
}

/// Spec: `ViewTurnsAway`. The plane takes the turn's angle and whole pixels, keeping its
/// fractions and its zoom; the flash is white; the rushing stars go and the turn's sprites are
/// placed, fractions 0.
fn turn_away(state: &mut State, turn: &SceneTurn) {
    let position = state.plane.position;
    let whole = place_at(turn.plane);
    state.stage = FlightStage::FlyingAway;
    state.plane.angle = turn.angle;
    state.plane.position = Pair {
        x: whole.x.with_sub(position.x.sub()),
        y: whole.y.with_sub(position.y.sub()),
    };
    state.flash = FLASH_WHITE;
    state.sprites = FlightSprites {
        rushing_stars: None,
        drifting: turn.sprites.map(|_, placement| {
            placement.map(|placement| DriftingSprite {
                place: place_at(placement.place),
                drift: placement.drift,
                wrap: placement.wrap,
            })
        }),
    };
}

/// Spec: `FlashFades`, `GunshipFliesAway`. The flash fades a level; the gunship shrinks by its
/// band's step and drifts left; past the last band the station comes into view.
fn fly_away(state: &mut State, scene: &Scene, rules: Rules) -> Option<TypedLetter> {
    state.flash = faded(state.flash);
    let zoom = state.plane.zoom;
    let Some(band) = AWAY_BANDS.iter().find(|band| zoom.get() < band.below) else {
        return station_comes_into_view(state, &scene.label, rules);
    };
    state.plane.zoom = zoom.shifted(band.step);
    state.plane.position.x -= PLANE_DRIFT_LEFT;
    None
}

/// Spec: `FlashFades`. Each colour one level darker, stopping at 0.
fn faded(flash: Colour) -> Colour {
    Colour::from_channels(flash.channels().map(|level| level.saturating_sub(1)))
}

/// Spec: `StationComesIntoView`. The plane put back square and at its size, at `(0, 0)`; the
/// label's first step begins.
fn station_comes_into_view(
    state: &mut State,
    label: &[LabelStep],
    rules: Rules,
) -> Option<TypedLetter> {
    state.plane = PLANE_AT_REST;
    begin_step(state, label, 0, rules)
}

/// Spec: `LabelIsTyped`. The hold counts down; when it runs out, the next step begins.
fn type_label(
    state: &mut State,
    progress: LabelProgress,
    label: &[LabelStep],
    rules: Rules,
) -> Option<TypedLetter> {
    if progress.hold_left > 1 {
        state.stage = FlightStage::Label(LabelProgress {
            hold_left: progress.hold_left.saturating_sub(1),
            ..progress
        });
        return None;
    }
    begin_step(state, label, progress.step.saturating_add(1), rules)
}

/// Spec: `LabelIsTyped`. A step begins: its letter, if any, is typed and its hold starts. With
/// no such step the label is over, and the fade's count starts at the rhythm.
fn begin_step(
    state: &mut State,
    label: &[LabelStep],
    step: usize,
    rules: Rules,
) -> Option<TypedLetter> {
    let Some(next) = label.get(step) else {
        state.stage = FlightStage::FadingOut;
        state.fade_count = rules.rhythm;
        return None;
    };
    state.stage = FlightStage::Label(LabelProgress {
        step,
        hold_left: next.hold.get(),
    });
    match next.keystroke {
        Keystroke::Letter(letter) => Some(letter),
        Keystroke::Pause => None,
    }
}

/// Spec: `PictureDarkens`, then `FlightEnds`.
fn fade_out(state: &mut State, rules: Rules) {
    darken(state, rules);
    if state.brightness == Brightness::DARK {
        end(state);
    }
}

/// Spec: `PictureDarkens`. The count counts down; at 0 it starts again at the rhythm and the
/// picture darkens a level, blanked once it reaches black. A picture already at level 0 is left
/// as it is.
fn darken(state: &mut State, rules: Rules) {
    if let Some(left) = state.fade_count.counted_down() {
        state.fade_count = left;
        return;
    }
    state.fade_count = rules.rhythm;
    if state.brightness.level == BrightnessLevel::BLACK {
        return;
    }
    let level = state.brightness.level.darker();
    state.brightness = Brightness {
        level,
        visibility: if level == BrightnessLevel::BLACK {
            Visibility::Blanked
        } else {
            Visibility::Shown
        },
    };
}

/// Spec: `FlightEnds`. The plane and the sprites cleared, the fade's count 0, the flash left:
/// the flight is done.
fn end(state: &mut State) {
    state.stage = FlightStage::Done;
    state.plane = PLANE_CLEARED;
    state.sprites = FlightSprites::default();
    state.fade_count = FadeFrames::ZERO;
}
