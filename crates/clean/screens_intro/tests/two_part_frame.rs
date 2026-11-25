//! The frame in two parts, around the flashback's room play (`BattleBegins`, `BattleDissolves`,
//! `PageTwoBegins`): the encounter's start and the hide are handed to the caller before the
//! encounter's reply is taken, the hide lands only once room play has run, and a return the
//! room reports starts page 2 on the next finished frame.

#[cfg(test)]
mod two_part_frame {
    use core::num::Wrapping;

    use enum_map::EnumMap;
    use screens_intro::{
        DissolveSets, FightReply, FlashbackRequest, FrameAwaitingReply, FrameInputs, IntroContent,
        IntroEvent, Letter, Report, Rules, Shows, Step, TextTiles, TrackReply, begin_frame, step,
    };
    use types::intro::{
        Brightness, Caret, CaretScript, ClickTurn, FadeFrames, FadingColours, FightVisibility,
        Glows, IntroStage, IntroState, PageScript, ScriptPlace, StageWait, StepAt, TextPlace,
    };
    use types::{Colour, PALETTE_SLOTS, Pair, Palette};

    /// The dissolve's count as a battle dissolve begins.
    const DISSOLVE_FROM: u16 = 127;

    /// A scene whose page 2 opens on a blank held one frame, then types a letter.
    fn scene() -> IntroContent {
        let mut pages = EnumMap::default();
        pages[PageScript::Two] = vec![
            Step::Show {
                shows: Shows::Nothing,
                hold: 1,
            },
            Step::Show {
                shows: Shows::Text(Letter {
                    glyph: 'A',
                    at: TextPlace { column: 2, row: 3 },
                }),
                hold: 4,
            },
            Step::End,
        ];
        IntroContent {
            pages,
            eyes: EnumMap::default(),
            baby: Vec::new(),
            caret: EnumMap::default(),
            full: Palette([Colour::BLACK; PALETTE_SLOTS]),
            sets: EnumMap::from_fn(|_| DissolveSets {
                dims: Vec::new(),
                brightens: Vec::new(),
            }),
            text: Vec::new(),
            caret_colours: Vec::new(),
            caret_home: Pair {
                x: Wrapping(8),
                y: Wrapping(16),
            },
        }
    }

    /// The intro at this stage, the dissolve at this count, Samus and her shots in front of the
    /// fight and the caret on its steady script.
    fn at(stage: IntroStage, dissolve: u16) -> IntroState {
        IntroState {
            stage,
            wait: StageWait::ZERO,
            colours: FadingColours::still_black(),
            dissolve,
            height: 0,
            fight: FightVisibility::InFront,
            baby: None,
            page: None,
            eyes: None,
            caret: Some(Caret {
                place: Pair {
                    x: Wrapping(8),
                    y: Wrapping(16),
                },
                script: ScriptPlace::start(CaretScript::Steady),
            }),
            glows: Glows::none(),
            click: ClickTurn::Off,
            brightness: Brightness::FULL,
            fade: FadeFrames::ZERO,
        }
    }

    fn requests(frame: &FrameAwaitingReply<'_>) -> Vec<FlashbackRequest> {
        frame.requests().collect()
    }

    fn text_updates(report: &Report) -> Vec<TextTiles> {
        report
            .events
            .iter()
            .filter_map(|event| {
                if let IntroEvent::TextUpdates(tiles) = event {
                    Some(*tiles)
                } else {
                    None
                }
            })
            .collect()
    }

    /// One finished frame in two parts, with no press and no track waiting.
    fn two_part(state: &mut IntroState, scene: &IntroContent, fight: FightReply) -> Report {
        begin_frame(state, false, TrackReply::NoneWaits, scene, Rules::REFERENCE).finish(fight)
    }

    #[test]
    fn the_press_hands_the_encounters_start_before_room_play() {
        let scene = scene();
        let mut state = at(IntroStage::PageOneWaits, 0);
        let frame = begin_frame(
            &mut state,
            true,
            TrackReply::NoneWaits,
            &scene,
            Rules::REFERENCE,
        );
        assert_eq!(requests(&frame), [FlashbackRequest::StartZebesFight]);
        assert_eq!(frame.state().stage, IntroStage::IntoFight);
        assert_eq!(frame.state().fight, FightVisibility::InFront);
        let report = frame.finish(FightReply::Fighting);
        assert!(
            report
                .events
                .contains(&IntroEvent::Flashback(FlashbackRequest::StartZebesFight))
        );
    }

    #[test]
    fn samus_takes_her_last_room_step_before_the_hide_lands() {
        let scene = scene();
        let mut state = at(IntroStage::OutOfFight, 1);
        let frame = begin_frame(
            &mut state,
            false,
            TrackReply::NoneWaits,
            &scene,
            Rules::REFERENCE,
        );
        assert_eq!(requests(&frame), [FlashbackRequest::HideFightSamusAndShots]);
        assert_eq!(frame.state().fight, FightVisibility::InFront);
        let report = frame.finish(FightReply::Fighting);
        assert_eq!(state.fight, FightVisibility::Hidden);
        assert!(report.events.contains(&IntroEvent::Flashback(
            FlashbackRequest::HideFightSamusAndShots
        )));
    }

    #[test]
    fn the_rooms_return_starts_page_two_on_the_next_finished_frame() {
        let scene = scene();
        let mut state = at(IntroStage::Fight, u16::MAX);

        let frame = begin_frame(
            &mut state,
            false,
            TrackReply::NoneWaits,
            &scene,
            Rules::REFERENCE,
        );
        assert_eq!(requests(&frame), Vec::new());
        assert_eq!(frame.state().stage, IntroStage::Fight);
        let report = frame.finish(FightReply::ReadyToReturn);
        assert_eq!(state.stage, IntroStage::PageTwoPending);
        assert_eq!(state.page, None);
        assert_eq!(text_updates(&report), [TextTiles::Paused]);

        let frame = begin_frame(
            &mut state,
            false,
            TrackReply::NoneWaits,
            &scene,
            Rules::REFERENCE,
        );
        assert_eq!(frame.state().stage, IntroStage::OutOfFight);
        assert_eq!(frame.state().dissolve, DISSOLVE_FROM);
        let report = frame.finish(FightReply::Fighting);
        let page = state.page.expect("page 2 is placed");
        assert_eq!(page.script, PageScript::Two);
        assert_eq!(page.progress.step, StepAt::Step(0));
        assert_eq!(report.typed, None);
        assert_eq!(text_updates(&report), [TextTiles::Resumed]);

        let report = two_part(&mut state, &scene, FightReply::Fighting);
        assert_eq!(state.dissolve, DISSOLVE_FROM - 1);
        let typed = report.typed.expect("page 2's first letter");
        assert_eq!(typed.glyph, 'A');
    }

    #[test]
    fn a_return_while_page_two_is_pending_changes_nothing_more() {
        let scene = scene();
        let mut early = at(IntroStage::Fight, u16::MAX);
        two_part(&mut early, &scene, FightReply::ReadyToReturn);
        let mut repeated = early.clone();
        two_part(&mut early, &scene, FightReply::Fighting);
        two_part(&mut repeated, &scene, FightReply::ReadyToReturn);
        assert_eq!(early, repeated);
    }

    #[test]
    fn one_call_and_two_parts_agree_through_the_return() {
        let scene = scene();
        let mut whole = at(IntroStage::Fight, u16::MAX);
        let mut parts = whole.clone();
        let replies = [
            FightReply::Fighting,
            FightReply::ReadyToReturn,
            FightReply::Fighting,
            FightReply::Fighting,
        ];
        for fight in replies {
            let inputs = FrameInputs {
                pressed: false,
                track: TrackReply::NoneWaits,
                fight,
            };
            let one = step(&mut whole, inputs, &scene, Rules::REFERENCE);
            let two = two_part(&mut parts, &scene, fight);
            assert_eq!(one, two);
            assert_eq!(whole, parts);
        }
    }
}
