//! Spec: `PlanetIsNamed`. The planet's label waits in the dark, brightens, holds and dims
//! across the four imported holds; then the ship resumes its flight.

use types::zebes::{
    CaptionAge, CaptionHolds, PictureMotion, PixelPoint, PlanePose, ScenePicture, SubtitleChoice,
    ZebesScene, ZebesStage, ZebesState,
};

use crate::report::{ColourEffectRequest, Requests};

/// Spec: `PlanetIsNamed`. The ship plane as the flight resumes: angle 32, zoom 16, scroll
/// (62, −112) px.
const FLIGHT_START_POSE: PlanePose = PlanePose::new(32, 16, PixelPoint { x: 62, y: -112 });

/// What the caption does on the frame its age reaches.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CaptionCue {
    /// The hidden lead ends: the English caption shows and brightens.
    EnglishAppears,
    /// The English lead ends: the subtitle joins it where chosen.
    SubtitleAppears,
    /// The full caption ends: the subtitle goes and the caption dims.
    CaptionDims,
    /// The fading caption ends: the caption goes and the flight resumes.
    CaptionEnds,
    /// Between those edges: the picture holds.
    Holds,
}

/// The cue at an age: each hold ends where the holds before it and it add up.
const fn cue_at(age: u16, holds: CaptionHolds) -> CaptionCue {
    let english = holds.hidden_lead.get();
    let subtitle = english.saturating_add(holds.english_lead.get());
    let dims = subtitle.saturating_add(holds.full_caption.get());
    let ends = dims.saturating_add(holds.fading_caption.get());
    if age >= ends {
        CaptionCue::CaptionEnds
    } else if age == english {
        CaptionCue::EnglishAppears
    } else if age == subtitle {
        CaptionCue::SubtitleAppears
    } else if age == dims {
        CaptionCue::CaptionDims
    } else {
        CaptionCue::Holds
    }
}

/// Spec: `PlanetIsNamed`. One caption frame: the age rises by one and the cue it reaches acts.
#[must_use]
pub fn name_planet(
    state: &mut ZebesState,
    scene: &ZebesScene,
    subtitles: SubtitleChoice,
    age: CaptionAge,
) -> Requests {
    let caption = scene.caption;
    let age = age.0.saturating_add(1);
    state.stage = ZebesStage::Caption {
        age: CaptionAge(age),
    };
    match cue_at(age, caption.holds) {
        CaptionCue::EnglishAppears => {
            show(state, ScenePicture::English, caption.english.anchor);
            Requests::colour_effect(ColourEffectRequest::CaptionBrightens(caption.brightening))
        }
        CaptionCue::SubtitleAppears => {
            if let (SubtitleChoice::WithJapanese, Some(japanese)) = (subtitles, caption.japanese) {
                show(state, ScenePicture::Japanese, japanese.anchor);
            }
            Requests::NONE
        }
        CaptionCue::CaptionDims => {
            state.pictures[ScenePicture::Japanese] = None;
            Requests::colour_effect(ColourEffectRequest::CaptionDims(caption.dimming))
        }
        CaptionCue::CaptionEnds => {
            flight_resumes(state);
            Requests::NONE
        }
        CaptionCue::Holds => Requests::NONE,
    }
}

/// A caption picture shown at rest on its imported anchor.
fn show(state: &mut ZebesState, which: ScenePicture, anchor: PixelPoint) {
    state.pictures[which] = Some(PictureMotion::placed(anchor));
}

/// Spec: `PlanetIsNamed` [C-caption-ends]. The caption goes, the ship takes the flight's
/// starting pose and the rightward drift begins on the next frame.
fn flight_resumes(state: &mut ZebesState) {
    state.pictures[ScenePicture::English] = None;
    state.pictures[ScenePicture::Japanese] = None;
    state.pose = FLIGHT_START_POSE;
    state.stage = ZebesStage::Rightward;
}
