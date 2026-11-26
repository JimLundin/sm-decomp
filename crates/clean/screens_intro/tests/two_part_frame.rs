//! The frame in two parts, around room play (`BattleBegins`, `BattleDissolves`, `PageTwoBegins`,
//! `DiscoveryBegins`, `DiscoveryReturns`): the encounters' starts and closes are handed to the
//! caller before their replies are taken, the hide lands only once room play has run, a return
//! the room reports starts its page on the next finished frame, and a cut between the parts
//! resumes the room's turn exactly once.

#[cfg(test)]
mod two_part_frame {
    use core::num::Wrapping;

    use enum_map::EnumMap;
    use screens_intro::{
        DiscoveryReply, DiscoveryRequest, DissolveSets, FightReply, FlashbackRequest,
        FrameAwaitingReply, FrameInputs, IntroContent, IntroEvent, Letter, Report, RoomTurnAwaited,
        Rules, Shows, Step, TextTiles, TrackReply, begin_frame, step,
    };
    use serde::{Deserialize, Serialize};
    use types::intro::{
        Brightness, Caret, CaretScript, ClickTurn, FadeFrames, FadingColours, FightHide,
        FightVisibility, Glows, IntroStage, IntroState, PageScript, Pan, RoomTurnCheckpoint,
        ScriptPlace, StageWait, StepAt, TextPlace,
    };
    use types::{Colour, PALETTE_SLOTS, Pair, Palette};

    /// The dissolve's count as a battle dissolve begins.
    const DISSOLVE_FROM: u16 = 127;

    /// A scene whose page 2 opens on a blank held one frame, then types a letter; page 3 is a
    /// blank held one frame; the steady caret shows nothing for 5 frames, over and over.
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
        pages[PageScript::Three] = vec![Step::Show {
            shows: Shows::Nothing,
            hold: 1,
        }];
        let mut caret = EnumMap::default();
        caret[CaretScript::Steady] = vec![
            Step::Show {
                shows: Shows::Nothing,
                hold: 5,
            },
            Step::Back(0),
        ];
        IntroContent {
            pages,
            eyes: EnumMap::default(),
            baby: EnumMap::default(),
            caret,
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
            pan: Pan::AT_REST,
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

    /// One finished frame in two parts, with no press, no track waiting and no discovery cue.
    fn two_part(state: &mut IntroState, scene: &IntroContent, fight: FightReply) -> Report {
        begin_frame(state, false, TrackReply::NoneWaits, scene, Rules::REFERENCE)
            .finish(fight, DiscoveryReply::Playing)
    }

    /// The first part of a pressed frame, cut before the room's turn.
    fn pressed_and_cut(state: &mut IntroState, scene: &IntroContent) -> (Report, RoomTurnAwaited) {
        begin_frame(state, true, TrackReply::NoneWaits, scene, Rules::REFERENCE).interrupt()
    }

    fn discovery_requests(report: &Report) -> Vec<DiscoveryRequest> {
        report
            .events
            .iter()
            .filter_map(|event| {
                if let IntroEvent::Discovery(request) = event {
                    Some(*request)
                } else {
                    None
                }
            })
            .collect()
    }

    #[test]
    fn a_cut_discovery_start_resumes_its_turn_once() {
        let scene = scene();
        let waiting = at(IntroStage::PageTwoWaits, u16::MAX);
        let mut whole = waiting.clone();
        let one = begin_frame(
            &mut whole,
            true,
            TrackReply::NoneWaits,
            &scene,
            Rules::REFERENCE,
        )
        .finish(FightReply::Fighting, DiscoveryReply::Playing);

        let mut cut = waiting;
        let (started, turn) = pressed_and_cut(&mut cut, &scene);
        assert_eq!(cut.stage, IntroStage::IntoDiscovery);
        assert_eq!(
            cut.caret.map(|caret| caret.script.progress.step),
            Some(StepAt::Start)
        );
        assert_eq!(
            discovery_requests(&started),
            [DiscoveryRequest::StartDiscovery]
        );
        let resumed = turn.resume(
            &mut cut,
            &scene,
            Rules::REFERENCE,
            FightReply::Fighting,
            DiscoveryReply::Playing,
        );
        assert_eq!(cut, whole);
        assert_eq!(cut.dissolve, DISSOLVE_FROM);
        assert_eq!(discovery_requests(&resumed), []);
        assert_eq!(resumed.colours, None);
        let mut joined = started.events;
        joined.extend(resumed.events);
        assert_eq!(joined, one.events);
    }

    /// What a caller saves at a cut: the intro's state and the room's turn it awaits.
    #[derive(Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct SavedCut {
        state: IntroState,
        turn: RoomTurnCheckpoint,
    }

    /// A frame cut after its stage's step, saved as text, restored and resumed, against the same
    /// frame run whole: the same state, and the setup's report then the resumed one making the
    /// whole frame's events, with the hide the checkpoint carries.
    fn saved_cut_finishes_the_frame(stage: IntroStage, dissolve: u16, pressed: bool) -> FightHide {
        let scene = scene();
        let start = at(stage, dissolve);
        let mut whole = start.clone();
        let one = begin_frame(
            &mut whole,
            pressed,
            TrackReply::NoneWaits,
            &scene,
            Rules::REFERENCE,
        )
        .finish(FightReply::Fighting, DiscoveryReply::Playing);

        let mut cut = start;
        let (started, turn) = begin_frame(
            &mut cut,
            pressed,
            TrackReply::NoneWaits,
            &scene,
            Rules::REFERENCE,
        )
        .interrupt();
        let checkpoint = turn.checkpoint();
        let saved = toml::to_string(&SavedCut {
            state: cut,
            turn: checkpoint,
        })
        .expect("the cut saves");

        let SavedCut {
            mut state,
            turn: restored,
        } = toml::from_str(&saved).expect("the saved cut reads");
        assert_eq!(restored, checkpoint);
        let resumed = RoomTurnAwaited::restore(restored).resume(
            &mut state,
            &scene,
            Rules::REFERENCE,
            FightReply::Fighting,
            DiscoveryReply::Playing,
        );
        assert_eq!(state, whole);
        let mut joined = started.events;
        joined.extend(resumed.events);
        assert_eq!(joined, one.events);
        checkpoint.hide
    }

    #[test]
    fn a_saved_discovery_start_restores_its_turn() {
        let hide = saved_cut_finishes_the_frame(IntroStage::PageTwoWaits, u16::MAX, true);
        assert_eq!(hide, FightHide::NotAsked);
    }

    #[test]
    fn a_saved_cut_keeps_the_hide_it_asked_for() {
        let hide = saved_cut_finishes_the_frame(IntroStage::OutOfFight, 1, false);
        assert_eq!(hide, FightHide::Asked);
    }

    #[test]
    fn the_discovery_finish_is_handed_before_room_play_ends() {
        let scene = scene();
        let mut state = at(IntroStage::OutOfDiscovery, 1);
        let frame = begin_frame(
            &mut state,
            false,
            TrackReply::NoneWaits,
            &scene,
            Rules::REFERENCE,
        );
        let requests: Vec<DiscoveryRequest> = frame.discovery_requests().collect();
        assert_eq!(requests, [DiscoveryRequest::FinishDiscovery]);
        assert_eq!(frame.state().dissolve, 0);
        let _report = frame.finish(FightReply::Fighting, DiscoveryReply::Playing);
    }

    #[test]
    fn the_eggs_cue_starts_page_three_on_the_next_finished_frame() {
        let scene = scene();
        let mut state = at(IntroStage::Discovery, u16::MAX);
        let report = begin_frame(
            &mut state,
            false,
            TrackReply::NoneWaits,
            &scene,
            Rules::REFERENCE,
        )
        .finish(FightReply::Fighting, DiscoveryReply::ReadyForPageThree);
        assert_eq!(state.stage, IntroStage::PageThreePending);
        assert_eq!(text_updates(&report), [TextTiles::Paused]);
        let report = two_part(&mut state, &scene, FightReply::Fighting);
        assert_eq!(state.stage, IntroStage::OutOfDiscovery);
        assert_eq!(state.dissolve, DISSOLVE_FROM);
        assert_eq!(state.page.map(|page| page.script), Some(PageScript::Three));
        assert_eq!(text_updates(&report), [TextTiles::Resumed]);
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
        let report = frame.finish(FightReply::Fighting, DiscoveryReply::Playing);
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
        let report = frame.finish(FightReply::Fighting, DiscoveryReply::Playing);
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
        let report = frame.finish(FightReply::ReadyToReturn, DiscoveryReply::Playing);
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
        let report = frame.finish(FightReply::Fighting, DiscoveryReply::Playing);
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
                discovery: DiscoveryReply::Playing,
            };
            let one = step(&mut whole, inputs, &scene, Rules::REFERENCE);
            let two = two_part(&mut parts, &scene, fight);
            assert_eq!(one, two);
            assert_eq!(whole, parts);
        }
    }
}
