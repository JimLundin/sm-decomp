//! Spec: `GunshipAppears` and `PlanetAppears`. The scene starts dark under a coarse mosaic,
//! brightens a level every two frames while the blocks shrink a pixel every four, and cuts to
//! the planet once it is fully lit at single pixels.

use types::zebes::{
    Brightness, CaptionAge, FadeWait, FlightView, MosaicWidth, PictureMotion, PixelPoint,
    PlacedPictures, PlanePose, QuarterBeat, ScenePicture, ZebesScene, ZebesStage, ZebesState,
};

use crate::report::{ColourEffectRequest, Requests, StartingPalette};

/// Spec: `GunshipAppears`. The ship plane at setup: square, actual size, unscrolled.
const SETUP_POSE: PlanePose = PlanePose::new(0, 256, PixelPoint { x: 0, y: 0 });

/// Spec: `PlanetAppears`. The ship plane at the cut to the planet: angle 32, actual size,
/// scroll (128, −104) px.
const PLANET_VIEW_POSE: PlanePose = PlanePose::new(32, 256, PixelPoint { x: 128, y: -104 });

/// Spec: `GunshipAppears` [C-setup]. The first step of the reveal, on the setup frame: the
/// flat page blanked under 9 px blocks, the light holding its level for one more frame, the
/// mosaic clock a frame past the beat, the plane square and nothing placed. E2 is asked to
/// install the pages and R4 to replace its effects with the scene's accent on the colours
/// already on screen.
#[must_use]
pub fn begin_reveal(state: &mut ZebesState, scene: &ZebesScene) -> Requests {
    *state = ZebesState {
        stage: ZebesStage::Revealing {
            fade: FadeWait::OneFrame,
            beat: QuarterBeat::AFTER_BEAT,
        },
        view: FlightView::MosaicPage,
        light: Brightness::Blanked,
        mosaic: MosaicWidth::WIDEST,
        pose: SETUP_POSE,
        pictures: PlacedPictures::default(),
    };
    Requests {
        pages: Some(scene.pages),
        colour_effect: Some(ColourEffectRequest::ReplaceActive {
            accent: scene.engine_accent,
            starting_palette: StartingPalette::ColoursOnScreen,
        }),
        handoff: None,
    }
}

/// Spec: `GunshipAppears`, then `PlanetAppears`. One reveal frame: the light steps, then the
/// mosaic on the beat sampled before the clock advances; fully lit at single pixels on that
/// beat, the planet appears.
pub fn reveal(state: &mut ZebesState, scene: &ZebesScene, fade: FadeWait, beat: QuarterBeat) {
    let fade = brighten(&mut state.light, fade);
    let on_beat = beat == QuarterBeat::ON_BEAT;
    if on_beat {
        state.mosaic = state.mosaic.finer();
    }
    if on_beat && state.light == Brightness::FULL && state.mosaic == MosaicWidth::SINGLE_PIXEL {
        planet_appears(state, scene);
    } else {
        state.stage = ZebesStage::Revealing {
            fade,
            beat: beat.next(),
        };
    }
}

/// Spec: `GunshipAppears`. The light's step, and how long it then holds its level. When the
/// light was due to hold for this frame, it keeps its level and is free to rise on the next
/// one [R-fade-waits]. At full light it simply stays full [R-lit-waits]. Otherwise it rises
/// one level and then holds that level for one frame, unless it has just reached full
/// [R-first-light, R-full-light].
fn brighten(light: &mut Brightness, fade: FadeWait) -> FadeWait {
    if fade == FadeWait::OneFrame || *light == Brightness::FULL {
        return FadeWait::Elapsed;
    }
    *light = light.brighter();
    if *light == Brightness::FULL {
        FadeWait::Elapsed
    } else {
        FadeWait::OneFrame
    }
}

/// Spec: `PlanetAppears` [R-planet, C-planet]. The ship plane shows with the planet and star
/// fields, each placed at rest on its imported anchor, and the caption's age starts at 0.
fn planet_appears(state: &mut ZebesState, scene: &ZebesScene) {
    state.view = FlightView::ShipAndSprites;
    state.pictures[ScenePicture::Planet] = Some(PictureMotion::placed(scene.planet.planet.anchor));
    for (field, placed) in &scene.planet.star_fields {
        state.pictures[ScenePicture::Stars(field)] = Some(PictureMotion::placed(placed.anchor));
    }
    state.pose = PLANET_VIEW_POSE;
    state.stage = ZebesStage::Caption { age: CaptionAge(0) };
}
