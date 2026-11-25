//! The intro from the opening through page 2, and from page 4's dissolve to the fade out.
//!
//! A completed work frame advances the stage and scripts and publishes their outputs. The
//! opening and page 1 leave the setup's palette to its loader. The Zebes flashback publishes
//! encounter, layout and text requests. Its scoped entry reads the boss's return cue; integrated
//! play requires the room as soon as the intro asks to start it.

use data::assets::intro::{self as asset, Cue, Picture, Scene, Shows, Step};
use enum_map::enum_map;
use screens_intro::{
    Dissolve, DissolveSets, FightReply, FlashbackRequest, FrameAwaitingReply, IndexedColour,
    IntroContent, Language, Letter, PictureName, Report, Rules, Signal, TrackReply, Unsupported,
    begin_frame, plays,
};
use types::intro::{
    Brightness, ClickTurn, FadeFrames, FadingColours, FightVisibility, Glows, IntroStage,
    IntroState, StageWait,
};
use types::{Colour, Pair, Palette};

/// How a frame ended: its work finished, with whether a button went down as it began and
/// audio's reply, or the frame lagged.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameEnd {
    /// The frame's work finished; whether any button was pressed anew, and whether a tune asked
    /// for still waits to load.
    Finished {
        /// Any button pressed anew this frame.
        pressed: bool,
        /// Audio's reply as the frame began.
        track: TrackReply,
        /// The selected encounter producer for the work frame.
        fight: FightInput,
    },
    /// The frame lagged: its work goes on into the next.
    Lagged,
}

/// The fight producer selected by the caller.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FightInput {
    /// Integrated production requires a room and encounter to run the fight.
    Integrated,
    /// An independent encounter reply for a scoped intro frame.
    Scoped(FightReply),
}

/// Why a frame was not played. State is unchanged.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// The intro is done and this frame is the flight's.
    FlightBegins,
    /// A finished frame that requires the Zebes fight room, demo and encounter.
    ZebesFightRoom,
    /// The press after page 2 starts the discovery.
    DiscoveryBegins,
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

/// The intro as the options hand on to it, before its first frame (`IntroBegins`).
///
/// The stage `Begins`, the screen black and blanked with the fade's count 0, as the front end
/// leaves it, and the rest at rest; the screen's colours black. Only the English pages are
/// played.
///
/// # Errors
/// [`Unsupported`] for a Japanese setting.
pub fn begins(language: Language) -> Result<State, Unsupported> {
    let intro = IntroState {
        stage: IntroStage::Begins,
        wait: StageWait::ZERO,
        colours: FadingColours::still_black(),
        dissolve: 0,
        height: 0,
        fight: FightVisibility::Hidden,
        baby: None,
        page: None,
        eyes: None,
        caret: None,
        glows: Glows::none(),
        click: ClickTurn::Off,
        brightness: Brightness::BLANKED,
        fade: FadeFrames::ZERO,
    };
    admit(
        language,
        intro,
        Palette([Colour::BLACK; types::PALETTE_SLOTS]),
    )
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
/// [`Fault::FlightBegins`] on a finished frame once the intro is done, and
/// [`Fault::ZebesFightRoom`] when the frame needs the Zebes fight room, and
/// [`Fault::DiscoveryBegins`] on a press while page 2 waits. State is unchanged.
pub fn played(
    state: &mut State,
    frame: FrameEnd,
    content: &IntroContent,
    rules: Rules,
) -> Result<Option<Report>, Fault> {
    let FrameEnd::Finished {
        pressed,
        track,
        fight,
    } = frame
    else {
        return Ok(None);
    };
    let before = state.intro.stage;
    play_work_frame(state, pressed, track, content, rules, |room| match fight {
        FightInput::Integrated => without_fight_room(before, room),
        FightInput::Scoped(reply) => Ok(reply),
    })
    .map(Some)
}

/// Advance a completed intro work frame with its room-play phase.
///
/// The callback applies a start request before it steps the room and applies a hide request
/// after room play, before drawing. It returns the encounter's cue to the intro before the
/// scripts run. The caller commits its room candidate together with the successful intro.
///
/// # Errors
/// A missing downstream owner or a refusal by room play leaves the intro unchanged.
pub fn play_work_frame(
    state: &mut State,
    pressed: bool,
    track: TrackReply,
    content: &IntroContent,
    rules: Rules,
    room_play: impl FnOnce(&FrameAwaitingReply<'_>) -> Result<FightReply, Fault>,
) -> Result<Report, Fault> {
    if state.intro.stage == IntroStage::Done {
        return Err(Fault::FlightBegins);
    }
    if pressed && state.intro.stage == IntroStage::PageTwoWaits {
        return Err(Fault::DiscoveryBegins);
    }
    let mut next = state.intro.clone();
    let room = begin_frame(&mut next, pressed, track, content, rules);
    let reply = room_play(&room)?;
    let report = room.finish(reply);
    state.intro = next;
    if let Some(colours) = &report.colours {
        state.colours = colours.clone();
    }
    Ok(report)
}

fn without_fight_room(
    stage: IntroStage,
    room: &FrameAwaitingReply<'_>,
) -> Result<FightReply, Fault> {
    if matches!(
        stage,
        IntroStage::IntoFight
            | IntroStage::Fight
            | IntroStage::PageTwoPending
            | IntroStage::OutOfFight
    ) || room.state().fight == FightVisibility::InFront
        || room
            .requests()
            .any(|request| request == FlashbackRequest::StartZebesFight)
    {
        return Err(Fault::ZebesFightRoom);
    }
    Ok(FightReply::Fighting)
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
            types::intro::PageScript::One => script(&scene.page_one),
            types::intro::PageScript::Two => script(&scene.page_two),
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
            Dissolve::IntoFight => sets(&scene.into_fight),
            Dissolve::OutOfFight => sets(&scene.out_of_fight),
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
                Cue::WaitForPressThenFight => Signal::WaitForPressThenFight,
                Cue::WaitForDiscovery => Signal::WaitForDiscovery,
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
    use super::{
        Fault, FightInput, FrameEnd, State, admit, begins, content, play_work_frame, played,
    };
    use data::assets::intro::{Colour, Cue, DissolveSets, Palette, Scene, Shows, Step};
    use screens_intro::{
        Equipment, FightReply, FlashbackRequest, IntroEvent, Language, Rules, TrackReply,
    };
    use types::intro::{
        Blanking, Brightness, BrightnessLevel, ClickTurn, FadeFrames, FadingColours,
        FightVisibility, Glows, IntroStage, IntroState, StageWait,
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
            page_one: wait.clone(),
            page_two: wait.clone(),
            page_five: wait.clone(),
            page_six: wait.clone(),
            eyes_blinking: wait.clone(),
            eyes_staring: wait.clone(),
            baby: wait.clone(),
            caret_steady: wait.clone(),
            caret_blinking: wait,
            full: Palette([Colour::BLACK; 256]),
            into_fight: none(),
            out_of_fight: none(),
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
            wait: StageWait::ZERO,
            colours: FadingColours::still_black(),
            dissolve: 0,
            height: 0,
            fight: FightVisibility::Hidden,
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

    const FINISHED: FrameEnd = FrameEnd::Finished {
        pressed: false,
        track: TrackReply::NoneWaits,
        fight: FightInput::Integrated,
    };

    #[test]
    fn the_intro_begins_dark_and_lends_the_equipment_on_its_first_frame() {
        let mut state = begins(Language::English).unwrap();
        assert_eq!(state.intro.stage, IntroStage::Begins);
        assert!(begins(Language::Japanese).is_err());
        let content = content(&scene());
        let report = played(&mut state, FINISHED, &content, Rules::REFERENCE)
            .unwrap()
            .unwrap();
        assert_eq!(state.intro.stage, IntroStage::OpeningLoads);
        assert_eq!(state.intro.brightness, Brightness::BLANKED);
        assert!(
            report
                .events
                .contains(&IntroEvent::EquipmentSet(Equipment::LENT))
        );
    }

    #[test]
    fn the_fight_start_requires_its_room_to_run() {
        let mut state = fading();
        state.intro.stage = IntroStage::PageOneWaits;
        let waiting = state.clone();
        let content = content(&scene());
        let pressed = FrameEnd::Finished {
            pressed: true,
            track: TrackReply::NoneWaits,
            fight: FightInput::Integrated,
        };
        assert_eq!(
            played(&mut state, pressed, &content, Rules::REFERENCE),
            Err(Fault::ZebesFightRoom)
        );
        assert_eq!(state, waiting, "a refused frame changes nothing");
        assert!(
            played(&mut state, FINISHED, &content, Rules::REFERENCE)
                .unwrap()
                .is_some()
        );
    }

    #[test]
    fn room_play_sees_the_start_and_the_last_visible_frame() {
        let mut state = fading();
        state.intro.stage = IntroStage::PageOneWaits;
        let content = content(&scene());
        play_work_frame(
            &mut state,
            true,
            TrackReply::NoneWaits,
            &content,
            Rules::REFERENCE,
            |room| {
                assert_eq!(room.state().stage, IntroStage::IntoFight);
                assert!(room.requests().eq([FlashbackRequest::StartZebesFight]));
                Ok(FightReply::Fighting)
            },
        )
        .unwrap();

        state.intro.stage = IntroStage::OutOfFight;
        state.intro.dissolve = 1;
        let report = play_work_frame(
            &mut state,
            false,
            TrackReply::NoneWaits,
            &content,
            Rules::REFERENCE,
            |room| {
                assert_eq!(room.state().dissolve, 0);
                assert_eq!(room.state().fight, FightVisibility::InFront);
                assert!(
                    room.requests()
                        .eq([FlashbackRequest::HideFightSamusAndShots])
                );
                Ok(FightReply::Fighting)
            },
        )
        .unwrap();
        assert_eq!(state.intro.fight, FightVisibility::Hidden);
        assert!(report.events.contains(&IntroEvent::Flashback(
            FlashbackRequest::HideFightSamusAndShots
        )));
    }

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
        assert!(
            report
                .events
                .contains(&IntroEvent::EquipmentSet(Equipment::RETURNED))
        );
        let done = state.clone();
        assert_eq!(
            played(&mut state, FINISHED, &content, Rules::REFERENCE),
            Err(Fault::FlightBegins)
        );
        assert_eq!(state, done, "a refused frame changes nothing");
    }
}
