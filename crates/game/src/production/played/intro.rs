//! The intro's last pages: page 4's text dissolving into the baby being examined, page 5 typed,
//! the wait for a press, page 6 and the fade to black (screens-intro).
//!
//! The intro runs ahead of the new game, with no room, and reads nothing of room play. It steps
//! once per frame of work that finished; a lag frame is the platform's and changes nothing of
//! it. The screen keeps the colours the intro last took. On the first finished frame after the
//! intro is done the flight to Ceres begins, which is its own owner's.

use data::assets::intro::{self as asset, Cue, Picture, Scene, Shows, Step};
use enum_map::enum_map;
use screens_intro::{
    Dissolve, DissolveSets, IndexedColour, IntroContent, Language, Letter, PictureName, Report,
    Rules, Signal, Unsupported, plays, step,
};
use types::intro::IntroState;
use types::{Pair, Palette};

/// How a frame ended: its work finished, with whether a button went down as it began, or the
/// frame lagged.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameEnd {
    /// The frame's work finished; whether any button was pressed anew.
    Finished {
        /// Any button pressed anew this frame.
        pressed: bool,
    },
    /// The frame lagged: its work goes on into the next.
    Lagged,
}

/// Why a frame was not played. State is unchanged.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// The intro is done and this frame is the flight's.
    FlightBegins,
}

/// The intro's State and the colours the screen shows, which the intro sets when a frame takes
/// them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    /// The model's State.
    pub intro: IntroState,
    /// The screen's 256 colours.
    pub colours: Palette,
}

/// The intro as admitted, in the options' language: only the English pages are played.
///
/// # Errors
/// [`Unsupported`] for a Japanese setting.
pub fn admit(
    language: Language,
    intro: IntroState,
    colours: Palette,
) -> Result<State, Unsupported> {
    plays(language)?;
    Ok(State { intro, colours })
}

/// Play one frame of the intro. A lagged frame changes nothing and reports nothing.
///
/// # Errors
/// [`Fault::FlightBegins`] on a finished frame once the intro is done.
pub fn played(
    state: &mut State,
    frame: FrameEnd,
    content: &IntroContent,
    rules: Rules,
) -> Result<Option<Report>, Fault> {
    let FrameEnd::Finished { pressed } = frame else {
        return Ok(None);
    };
    if state.intro.stage == types::intro::IntroStage::Done {
        return Err(Fault::FlightBegins);
    }
    let report = step(&mut state.intro, pressed, content, rules);
    if let Some(colours) = &report.colours {
        state.colours = colours.clone();
    }
    Ok(Some(report))
}

/// The model's content from the imported scene.
#[must_use]
pub fn content(scene: &Scene) -> IntroContent {
    let sets = |sets: &asset::DissolveSets| DissolveSets {
        dims: sets.dims.clone(),
        brightens: sets.brightens.clone(),
    };
    let [x, y] = scene.caret_home;
    IntroContent {
        pages: enum_map! {
            types::intro::PageScript::Five => script(&scene.page_five),
            types::intro::PageScript::Six => script(&scene.page_six),
        },
        eyes: enum_map! {
            types::intro::EyesScript::Blinking => script(&scene.eyes_blinking),
            types::intro::EyesScript::Staring => script(&scene.eyes_staring),
        },
        baby: script(&scene.baby),
        caret: enum_map! {
            types::intro::CaretScript::Steady => script(&scene.caret_steady),
            types::intro::CaretScript::Blinking => script(&scene.caret_blinking),
        },
        full: scene.full.clone(),
        sets: enum_map! {
            Dissolve::ToScene => sets(&scene.to_scene),
            Dissolve::ToText => sets(&scene.to_text),
        },
        text: scene.text.clone(),
        caret_colours: scene
            .caret_colours
            .iter()
            .map(|&(index, colour)| IndexedColour { index, colour })
            .collect(),
        caret_home: Pair {
            x: core::num::Wrapping(x),
            y: core::num::Wrapping(y),
        },
    }
}

/// A script's steps in the model's terms.
fn script(steps: &[Step]) -> screens_intro::Script {
    steps
        .iter()
        .map(|&step| match step {
            Step::Show { shows, hold } => screens_intro::Step::Show {
                shows: match shows {
                    Shows::Nothing => screens_intro::Shows::Nothing,
                    Shows::Picture(picture) => screens_intro::Shows::Image(name(picture)),
                    Shows::Letter { glyph, at } => screens_intro::Shows::Text(Letter { glyph, at }),
                },
                hold,
            },
            Step::Back(to) => screens_intro::Step::Back(to),
            Step::Repeat { from, times } => screens_intro::Step::Repeat { from, times },
            Step::Cue(cue) => screens_intro::Step::Hand(match cue {
                Cue::PageFive => Signal::PageFive,
                Cue::WaitForPress => Signal::WaitForPress,
                Cue::CaretBlinks => Signal::CaretBlinks,
                Cue::Finish => Signal::Finish,
            }),
            Step::End => screens_intro::Step::End,
        })
        .collect()
}

/// A picture's name, for the platform to draw.
fn name(picture: Picture) -> PictureName {
    PictureName(format!("{picture:?}"))
}

#[cfg(test)]
mod tests {
    use super::{FrameEnd, State, admit, content, played};
    use data::assets::intro::{Colour, Cue, DissolveSets, Palette, Scene, Shows, Step};
    use screens_intro::{IntroEvent, Language, Rules};
    use types::intro::{
        Blanking, Brightness, BrightnessLevel, ClickTurn, FadeFrames, FadingColours, Glows,
        IntroStage, IntroState,
    };

    fn scene() -> Scene {
        let none = || DissolveSets {
            dims: vec![],
            brightens: vec![],
        };
        let wait = vec![
            Step::Show {
                shows: Shows::Nothing,
                hold: 1,
            },
            Step::Cue(Cue::Finish),
            Step::End,
        ];
        Scene {
            page_five: wait.clone(),
            page_six: wait.clone(),
            eyes_blinking: wait.clone(),
            eyes_staring: wait.clone(),
            baby: wait.clone(),
            caret_steady: wait.clone(),
            caret_blinking: wait,
            full: Palette([Colour::BLACK; 256]),
            to_scene: none(),
            to_text: none(),
            text: vec![],
            caret_colours: vec![],
            caret_home: [8, 24],
        }
    }

    fn fading() -> State {
        let intro = IntroState {
            stage: IntroStage::FadingOut,
            colours: FadingColours::still_black(),
            dissolve: 0,
            height: 0,
            baby: None,
            page: None,
            eyes: None,
            caret: None,
            glows: Glows::none(),
            click: ClickTurn::Off,
            brightness: Brightness {
                level: BrightnessLevel::new(1).unwrap(),
                blanking: Blanking::Shown,
            },
            fade: FadeFrames::ZERO,
        };
        admit(Language::English, intro, Palette([Colour::BLACK; 256])).unwrap()
    }

    const FINISHED: FrameEnd = FrameEnd::Finished { pressed: false };

    #[test]
    fn a_japanese_setting_is_not_played() {
        let state = fading();
        assert!(admit(Language::Japanese, state.intro, state.colours).is_err());
    }

    #[test]
    fn a_lagged_frame_changes_nothing() {
        let mut state = fading();
        let content = content(&scene());
        assert_eq!(
            played(&mut state, FrameEnd::Lagged, &content, Rules::REFERENCE),
            Ok(None)
        );
        assert_eq!(state, fading());
    }

    #[test]
    fn the_intro_ends_returning_the_equipment_and_then_hands_on() {
        let mut state = fading();
        let content = content(&scene());
        let report = played(&mut state, FINISHED, &content, Rules::REFERENCE)
            .unwrap()
            .unwrap();
        assert_eq!(state.intro.stage, IntroStage::Done);
        assert!(report.events.contains(&IntroEvent::EquipmentReturned));
        let done = state.clone();
        assert_eq!(
            played(&mut state, FINISHED, &content, Rules::REFERENCE),
            Err(super::Fault::FlightBegins)
        );
        assert_eq!(state, done, "a refused frame changes nothing");
    }
}
