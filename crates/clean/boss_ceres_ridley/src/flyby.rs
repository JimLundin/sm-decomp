//! `GetawayPass` and `GetawayPicture`: the flyby runs on a room path and the game clock,
//! not on the airborne body or on Samus's movement.

use core::num;

use enum_map::{Enum, EnumMap};
use types::ceres_getaway::{
    FlybyBabyBeat, FlybyProgress, FlybyTurn, FlybyWingBeat, GetawayState, OutOfTheWay,
};
use types::quake::{QuakeDirection, QuakeDuration, QuakeKind, QuakeReach, QuakeRequest, QuakeSize};
use types::{Colour, GameFrames, Pair, Picture};

use crate::station::Stage;
use crate::turn::{QUARTER_TURN, WHOLE_TURN, sine};

const MAX_ZOOM: u16 = 2048;
const SCALE: u16 = 256;
const COEFFICIENT_SCALE: i32 = 256;
const SHADES: usize = 9;
const COLOURS_PER_SHADE: usize = 15;
const INITIAL_SCROLL: Pair<i16> = Pair { x: -128, y: 32 };
const TURN_STEP: u16 = 48;
const PIVOT: Pair<i16> = Pair { x: 64, y: 64 };
const BABY_PERIOD: u16 = 4;
const WING_PERIOD: u16 = 8;
/// The pass acts on the run after 104 pictures, even if that run restores the room.
const BEFORE_PASS: usize = 104;
const PASS_QUAKE: QuakeRequest = QuakeRequest {
    kind: QuakeKind {
        reach: QuakeReach::Back,
        direction: QuakeDirection::Diagonal,
        size: QuakeSize::LARGE,
    },
    duration: QuakeDuration(64),
};

/// `GetawayPicture`: zoom in 256ths, from zero through 2,048.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FlybyZoom(u16);

impl FlybyZoom {
    /// A zoom of this many 256ths, or none outside the supported range.
    #[must_use]
    pub const fn new(units: u16) -> Option<Self> {
        if units <= MAX_ZOOM {
            Some(Self(units))
        } else {
            None
        }
    }

    /// Zoom in 256ths.
    #[must_use]
    pub const fn units(self) -> u16 {
        self.0
    }
}

/// `GetawayPass`: a row of the room's flyby path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FlybyFrame {
    /// How much to scroll the picture, in signed whole pixels.
    pub shift: Pair<i16>,
    /// The zoom this frame is drawn at.
    pub zoom: FlybyZoom,
}

/// `GetawayPass`: the frames of the flyby, with a first picture and any following pictures.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlybyPath {
    /// The picture on the run that starts the flyby.
    pub first: FlybyFrame,
    /// The remaining pictures in order.
    pub rest: Vec<FlybyFrame>,
}

/// `GetawayPicture`: the baby's looks in the flyby plane.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum)]
pub enum FlybyBabyLook {
    /// The first look, shown at beat zero.
    First,
    /// The look between the first and third, shared by beats one and three.
    Middle,
    /// The third look, shown at beat two.
    Third,
}

/// `GetawayPass`, `GetawayPicture`: the room's imported path, palettes and art.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlybyContent {
    /// At least one picture. The game's path has 112 pictures.
    pub path: FlybyPath,
    /// The nine body palettes, from the smallest zoom to the largest.
    pub body_shades: [[Colour; COLOURS_PER_SHADE]; SHADES],
    /// The art for each of the baby's three looks.
    pub baby_looks: EnumMap<FlybyBabyLook, Picture>,
    /// The art for the wings' two looks.
    pub wing_looks: EnumMap<FlybyWingBeat, Picture>,
}

impl FlybyContent {
    /// `GetawayPicture`: the baby's art at a saved beat, also used to restore a picture between
    /// clock boundaries when no change of look is requested.
    #[must_use]
    pub fn baby_picture(&self, beat: FlybyBabyBeat) -> Picture {
        let look = match beat {
            FlybyBabyBeat::First => FlybyBabyLook::First,
            FlybyBabyBeat::ToThird | FlybyBabyBeat::ToFirst => FlybyBabyLook::Middle,
            FlybyBabyBeat::Third => FlybyBabyLook::Third,
        };
        self.baby_looks[look]
    }
}

/// `GetawayPicture`: the affine transform of the view.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FlybyTransform {
    /// Picture scroll, in signed whole pixels.
    pub scroll: Pair<i16>,
    /// Pivot, in whole pixels.
    pub pivot: Pair<i16>,
    /// Drawing coefficients in 256ths, in row order.
    pub matrix: [i32; 4],
}

/// `GetawayPicture`: the flyby plane is transparent outside its picture, and selects the
/// body's colours and any changed art. `GetawayPass`: the return shows only the room.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GetawayView {
    /// The flyby plane and the selections that go with this picture.
    Flyby {
        /// Scroll, pivot and drawing coefficients.
        transform: FlybyTransform,
        /// The frame's zoom.
        zoom: FlybyZoom,
        /// The 15 colours of `BackgroundBody` after transparency.
        body_colours: [Colour; COLOURS_PER_SHADE],
        /// A replacement look for the baby, only on a baby beat.
        baby_look: Option<Picture>,
        /// A replacement look for the wings, only on a wing beat.
        wing_look: Option<Picture>,
    },
    /// On return, the room is visible by itself and all flyby transform values are zero.
    Room,
}

impl GetawayView {
    /// The active flyby transform. On return its scroll, pivot and matrix are zero.
    #[must_use]
    pub const fn transform(self) -> FlybyTransform {
        const ZERO: Pair<i16> = Pair { x: 0, y: 0 };
        match self {
            Self::Flyby { transform, .. } => transform,
            Self::Room => FlybyTransform {
                scroll: ZERO,
                pivot: ZERO,
                matrix: [0, 0, 0, 0],
            },
        }
    }
}

/// `GetawayPass`: the one-shot getaway sound.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GetawaySound;

impl GetawaySound {
    /// The second effects channel.
    pub const EFFECTS_CHANNEL: u8 = 2;
    /// Audio's queue limit for the request.
    pub const QUEUE_LIMIT: u8 = 6;
}

/// `GetawayPass`, `GetawayPicture`: what a flyby run hands to the other domains.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GetawayEffects {
    /// Publish the first run's new stage before the placed walls run.
    pub stage: Option<Stage>,
    /// Audio's request on the first run.
    pub sound: Option<GetawaySound>,
    /// A request to her movement, not a hit.
    pub push: Option<OutOfTheWay>,
    /// The shake request for the room's quake update on this run.
    pub quake: Option<QuakeRequest>,
    /// The picture to draw on this run.
    pub view: GetawayView,
}

/// `GetawayPass`: advance the path or restore the room. Return nothing once the warning owns it.
pub fn step(
    state: GetawayState,
    finished: GameFrames,
    content: &FlybyContent,
) -> Option<(GetawayState, GetawayEffects)> {
    let (shown, scroll, turn, beats, frame) = match state {
        GetawayState::GetawayReady(beats) => (
            0,
            INITIAL_SCROLL,
            FlybyTurn(1),
            beats,
            Some(&content.path.first),
        ),
        GetawayState::FlyingPast(progress) => (
            progress.shown.get(),
            progress.scroll,
            progress.turn,
            progress.beats,
            content.path.rest.get(progress.shown.get() - 1),
        ),
        GetawayState::WarningReady => return None,
    };
    let first = shown == 0;
    let passing = shown == BEFORE_PASS;
    let (state, view) = frame.map_or((GetawayState::WarningReady, GetawayView::Room), |frame| {
        let progress = FlybyProgress {
            shown: num::NonZeroUsize::MIN.saturating_add(shown),
            scroll: Pair {
                x: scroll.x.wrapping_add(frame.shift.x),
                y: scroll.y.wrapping_add(frame.shift.y),
            },
            turn: FlybyTurn(turn.0.wrapping_add(TURN_STEP)),
            beats,
        };
        picture(progress, frame.zoom, finished, content)
    });
    Some((
        state,
        GetawayEffects {
            stage: first.then_some(Stage::Getaway),
            sound: first.then_some(GetawaySound),
            push: passing.then_some(OutOfTheWay),
            quake: passing.then_some(PASS_QUAKE),
            view,
        },
    ))
}

/// `GetawayPicture`: take the new beats and art only at their own clock boundaries.
fn picture(
    mut progress: FlybyProgress,
    zoom: FlybyZoom,
    GameFrames(finished): GameFrames,
    content: &FlybyContent,
) -> (GetawayState, GetawayView) {
    let baby_look = finished.is_multiple_of(BABY_PERIOD).then(|| {
        progress.beats.baby = next_baby(progress.beats.baby);
        content.baby_picture(progress.beats.baby)
    });
    let wing_look = finished.is_multiple_of(WING_PERIOD).then(|| {
        progress.beats.wings = next_wings(progress.beats.wings);
        content.wing_looks[progress.beats.wings]
    });
    let shade = usize::from(zoom.units() / SCALE);
    let view = GetawayView::Flyby {
        transform: FlybyTransform {
            scroll: progress.scroll,
            pivot: PIVOT,
            matrix: matrix(progress.turn, zoom),
        },
        zoom,
        body_colours: content
            .body_shades
            .get(shade)
            .copied()
            .unwrap_or(content.body_shades[0]),
        baby_look,
        wing_look,
    };
    (GetawayState::FlyingPast(progress), view)
}

const fn next_baby(beat: FlybyBabyBeat) -> FlybyBabyBeat {
    match beat {
        FlybyBabyBeat::First => FlybyBabyBeat::ToThird,
        FlybyBabyBeat::ToThird => FlybyBabyBeat::Third,
        FlybyBabyBeat::Third => FlybyBabyBeat::ToFirst,
        FlybyBabyBeat::ToFirst => FlybyBabyBeat::First,
    }
}

const fn next_wings(beat: FlybyWingBeat) -> FlybyWingBeat {
    match beat {
        FlybyWingBeat::First => FlybyWingBeat::Second,
        FlybyWingBeat::Second => FlybyWingBeat::First,
    }
}

/// `GetawayPicture`: unlike the tail's projection, signed scaling truncates toward zero.
fn matrix(turn: FlybyTurn, zoom: FlybyZoom) -> [i32; 4] {
    let angle = turn.0 / WHOLE_TURN;
    let scaled = |angle| sine(angle).saturating_mul(i32::from(zoom.units())) / COEFFICIENT_SCALE;
    let s = scaled(angle);
    let c = scaled(angle.wrapping_add(QUARTER_TURN));
    [c, s, s.saturating_neg(), c]
}
