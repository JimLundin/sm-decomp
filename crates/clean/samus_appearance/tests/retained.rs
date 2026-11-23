//! Her appearance State saved and restored, the ranges its parts keep, and whole frames whose
//! sound rules have no hidden guards and ask in the frame's order.
#[cfg(test)]
mod retained {
    use enum_map::EnumMap;
    use samus_appearance::{
        Activity, AppearanceState, CannonDelay, CannonEvent, Charge, ChargeGlowPosition, Clip,
        ColourSequences, Content, Control, Dash, Figure, Frame, FrameOutputs, FramePart, Frames,
        Hold, Inputs, Instruction, Laps, LapsLeft, Liquid, Pace, Paces, PaletteOpportunity,
        Playhead, Rules, SoundQueue, SoundRequest, SoundsWaiting, State, Step, VisorDelay,
        VisorStep, update,
    };
    use types::{
        Action, Aim, Body, BoostStage, Facing, Gun, Item, Legs, Pose, Set, Sprite, ZERO, set_of,
    };

    /// Her figure upright, facing right, unaimed, doing `action`.
    const fn upright(action: Action) -> Figure {
        Figure {
            pose: Pose {
                body: Body::Upright,
                action,
                facing: Facing::Right,
                aim: Aim::Forward,
                gun: Gun::Stowed,
                legs: Legs::Together,
            },
            sprite: Sprite::Plain,
        }
    }

    fn hold(frames: u8) -> Hold {
        Hold::new(frames).expect("a hold in range")
    }

    /// A running clip: step 0, a contact held 3 frames, then back to it.
    fn stride_clip() -> Clip {
        Clip {
            instructions: vec![
                Instruction::Show(hold(3)),
                Instruction::Control(Control::Again(Step::FIRST)),
            ],
            kick: None,
            fast_fall: None,
            rising: None,
            contacts: std::iter::once(Step::FIRST).collect(),
        }
    }

    /// One clip for every figure, and every pace holding 3 frames a step.
    struct OneClip {
        clip: Clip,
        paces: Paces,
    }

    impl OneClip {
        fn of(clip: Clip) -> Self {
            let pace = || Pace(vec![hold(3), hold(3)]);
            Self {
                clip,
                paces: Paces {
                    plain: pace(),
                    boosted: EnumMap::from_fn(|_| pace()),
                },
            }
        }
    }

    impl Content for OneClip {
        fn clip(&self, _: &Figure) -> &Clip {
            &self.clip
        }

        fn paces(&self) -> &Paces {
            &self.paces
        }
    }

    const COLOURS: ColourSequences = ColourSequences {
        visor_delay: VisorDelay(7),
        visor_step: VisorStep(5),
        charge_glow_position: ChargeGlowPosition(2),
    };

    /// Her appearance State with `animation`, a kept spin, her cannon waiting a frame and some
    /// colours.
    fn appearance(animation: State) -> AppearanceState {
        AppearanceState {
            animation,
            kept_action: Action::Spinning,
            cannon: CannonDelay::new(1).expect("a delay in range"),
            colours: COLOURS,
        }
    }

    /// A frame's inputs: she starts in `figure` and ends in `figure_after`, out of a liquid,
    /// with nothing else happening.
    fn inputs(figure: Figure, figure_after: Figure) -> Inputs {
        Inputs {
            figure,
            figure_after,
            moves: Set::default(),
            dash: Dash::Off,
            run_held: false,
            heading_down: false,
            vertical_speed: ZERO,
            energy: 99,
            items: Set::default(),
            liquid: Liquid::NONE,
            sounds_waiting: SoundsWaiting::NONE,
            activities: Set::default(),
            shoot_held: false,
            charge: Charge::new(0).expect("no charge"),
        }
    }

    /// A frame of `inputs` with `parts`, with no item controls and no palette opportunity.
    fn frame<const N: usize>(inputs: Inputs, parts: [FramePart; N]) -> Frame {
        Frame {
            inputs,
            parts: set_of(parts),
            cannon: CannonEvent::ControlsSkipped,
            palette: PaletteOpportunity::Absent,
        }
    }

    fn on_contact(stage: BoostStage, laps_left: LapsLeft) -> State {
        State {
            playhead: Playhead {
                step: Step::FIRST,
                time_left: Frames::LAST,
            },
            stage,
            laps_left,
        }
    }

    /// Plays `frame` from `state` with the stride clip for every figure.
    fn play(state: AppearanceState, frame: &Frame) -> (AppearanceState, FrameOutputs) {
        update(state, frame, &OneClip::of(stride_clip()), &Rules::VANILLA)
    }

    #[test]
    fn state_serializes_every_quantity_and_restores_it() {
        for stage in BoostStage::ALL {
            for laps_left in [
                LapsLeft::WithoutBooster,
                LapsLeft::WithBooster(Laps::new(u8::MAX).expect("255 laps")),
            ] {
                let state = State {
                    playhead: Playhead {
                        step: Step(u8::MAX),
                        time_left: Frames(u8::MAX),
                    },
                    stage,
                    laps_left,
                };
                let text = toml::to_string(&state).expect("the State saves");
                assert!(text.contains("step = 255"));
                assert!(text.contains("time_left = 255"));
                assert!(text.contains("stage ="));
                assert!(text.contains("laps_left"));
                assert_eq!(toml::from_str::<State>(&text).expect("restores"), state);
            }
        }
    }

    #[test]
    fn the_appearance_state_saves_all_it_remembers() {
        let state = AppearanceState {
            kept_action: Action::WallJumping,
            ..appearance(on_contact(BoostStage::Stage3, LapsLeft::WithoutBooster))
        };
        let text = toml::to_string(&state).expect("the appearance State saves");
        for part in [
            "animation",
            "kept_action = \"WallJumping\"",
            "cannon",
            "colours",
            "visor_delay = 7",
            "visor_step = 5",
            "charge_glow_position = 2",
        ] {
            assert!(text.contains(part), "{part} in {text}");
        }
        assert_eq!(
            toml::from_str::<AppearanceState>(&text).expect("restores"),
            state
        );
    }

    #[test]
    fn restored_laps_cannot_bypass_the_positive_range() {
        let text =
            "stage = 'Stage0'\n[playhead]\nstep = 0\ntime_left = 0\n[laps_left]\nWithBooster = 0\n";
        assert!(toml::from_str::<State>(text).is_err());
        assert_eq!(Laps::new(0), None);
        assert_eq!(Charge::new(0).map(Charge::frames), Some(0));
        assert_eq!(
            Charge::new(Charge::MOST).map(Charge::frames),
            Some(Charge::MOST)
        );
        assert_eq!(Charge::new(Charge::MOST + 1), None);
    }

    #[test]
    fn contact_suppressor_combinations_keep_graphics() {
        let combinations = [false, true]
            .into_iter()
            .flat_map(|a| [false, true].map(|b| (a, b)))
            .flat_map(|(a, b)| [false, true].map(|c| (a, b, c)));
        for (cinematic, boss, colours) in combinations {
            let running = upright(Action::Running);
            let mut given = inputs(running, running);
            given.activities = [
                (Activity::Cinematic, cinematic),
                (Activity::Boss, boss),
                (Activity::SpecialColourTime, colours),
            ]
            .into_iter()
            .collect();
            let before = appearance(on_contact(BoostStage::Stage3, LapsLeft::WithoutBooster));
            let (_, outputs) = play(before, &frame(given, [FramePart::Movement]));
            let contact = outputs.contact.expect("a contact");
            assert_eq!(contact.playhead, before.animation.playhead, "as she starts");
            let quiet = cinematic || boss || colours;
            let footstep = (!quiet).then_some(SoundRequest::Footstep);
            assert_eq!(outputs.sounds, footstep.into_iter().collect::<Vec<_>>());
        }
    }

    #[test]
    fn a_contact_on_a_control_is_none() {
        // Shape relations are the importer's: the model stays total on a mark it can't show.
        let mut clip = stride_clip();
        clip.contacts = std::iter::once(Step::SECOND).collect();
        let state = State {
            playhead: Playhead {
                step: Step::SECOND,
                time_left: Frames::LAST,
            },
            ..on_contact(BoostStage::Stage0, LapsLeft::WithoutBooster)
        };
        let running = upright(Action::Running);
        let (_, outputs) = update(
            appearance(state),
            &frame(inputs(running, running), [FramePart::Movement]),
            &OneClip::of(clip),
            &Rules::VANILLA,
        );
        assert_eq!(outputs.contact, None);
        assert_eq!(outputs.sounds, []);
    }

    #[test]
    fn drawing_and_passage_have_no_hidden_engine_admission_guards() {
        let most = Charge::new(Charge::MOST).expect("in range");
        let spinning = upright(Action::Spinning);
        let cases = [
            (
                frame(
                    inputs(spinning, upright(Action::Unmorphing)),
                    [FramePart::Drawing],
                ),
                vec![SoundRequest::SpinEnded],
            ),
            (
                frame(
                    Inputs {
                        shoot_held: true,
                        charge: most,
                        ..inputs(spinning, spinning)
                    },
                    [FramePart::PassageEntry],
                ),
                vec![SoundRequest::SpinEnded],
            ),
            (
                frame(
                    Inputs {
                        charge: most,
                        ..inputs(spinning, upright(Action::Running))
                    },
                    [FramePart::PassageEntry],
                ),
                Vec::new(),
            ),
        ];
        for (frame, expected) in cases {
            let before = appearance(on_contact(BoostStage::Stage0, LapsLeft::WithoutBooster));
            let (after, outputs) = play(before, &frame);
            assert_eq!(after, before);
            assert_eq!(outputs.sounds, expected, "{frame:?}");
        }
    }

    /// A frozen frame at a passage's door, with its work unprepared: no animation update, the
    /// door's request and then normal drawing's, from the spin her State kept, the State as it
    /// was.
    #[test]
    fn a_frozen_passage_frame_asks_at_the_door_then_draws() {
        let running = upright(Action::Running);
        let frame = frame(
            inputs(running, running),
            [FramePart::PassageEntry, FramePart::Drawing],
        );
        let before = appearance(on_contact(BoostStage::Boosting, LapsLeft::WithoutBooster));
        let (after, outputs) = play(before, &frame);
        assert_eq!(after, before);
        assert_eq!(
            outputs.sounds,
            [SoundRequest::SamusSilence, SoundRequest::SpinEnded]
        );
    }

    /// Her initialization comes first in its frame and the frame plays on from it: drawing reads
    /// the kept action initialization leaves, `Still`, not the spin the frame was given, and the
    /// frame leaves her State as at initialization.
    #[test]
    fn a_frame_plays_on_from_her_initialization() {
        let falling = upright(Action::Falling);
        let frame = Frame {
            cannon: CannonEvent::SamusInitialized,
            ..frame(inputs(falling, falling), [FramePart::Drawing])
        };
        let before = appearance(on_contact(BoostStage::Boosting, LapsLeft::WithoutBooster));
        let (after, outputs) = play(before, &frame);
        assert_eq!(after, AppearanceState::INITIAL);
        assert_eq!(outputs.sounds, []);
        assert!(!outputs.cannon_may_follow);
    }

    /// Drawing repeated on frames whose work is not prepared reads the same kept action each
    /// time; a prepared frame keeps her action as it starts, before drawing reads it.
    #[test]
    fn drawing_reads_the_action_her_last_prepared_frame_kept() {
        let (running, falling) = (upright(Action::Running), upright(Action::Falling));
        let mut state = appearance(on_contact(BoostStage::Stage0, LapsLeft::WithoutBooster));
        for _ in 0..2 {
            let (after, outputs) = play(
                state,
                &frame(inputs(falling, falling), [FramePart::Drawing]),
            );
            assert_eq!(outputs.sounds, [SoundRequest::SpinEnded]);
            assert_eq!(after.kept_action, Action::Spinning);
            state = after;
        }
        let prepared = frame(
            inputs(running, falling),
            [FramePart::Preparation, FramePart::Drawing],
        );
        let (after, outputs) = play(state, &prepared);
        assert_eq!(outputs.sounds, []);
        assert_eq!(after.kept_action, Action::Running);
    }

    /// One frame whose stride, animation update, spin start and passage entry each ask for a
    /// sound: all four, in the frame's order, with the one animation update's State.
    #[test]
    fn the_frames_sounds_come_in_the_frames_order() {
        let (running, spinning) = (upright(Action::Running), upright(Action::Spinning));
        let at_start = on_contact(
            BoostStage::Stage3,
            LapsLeft::WithBooster(Laps::new(1).expect("a lap")),
        );
        let given = Inputs {
            dash: Dash::On,
            run_held: true,
            items: set_of([Item::SpeedBooster]),
            ..inputs(running, spinning)
        };
        let frame = frame(given, [FramePart::Movement, FramePart::PassageEntry]);
        let (after, outputs) = play(appearance(at_start), &frame);
        assert_eq!(
            outputs.sounds,
            [
                SoundRequest::Footstep,
                SoundRequest::Boosting,
                SoundRequest::PlainSpin,
                SoundRequest::SpinEnded,
            ]
        );
        assert!(outputs.contact.is_some());
        assert_eq!(after.animation.stage, BoostStage::Boosting);
        assert_eq!(
            after.animation.playhead.step,
            Step::FIRST,
            "her spin's clip"
        );
        assert_eq!(after.cannon, appearance(at_start).cannon);
        assert_eq!(after.colours, COLOURS);
    }

    #[test]
    fn requests_describe_admission_not_driver_work() {
        assert_eq!(SoundRequest::Footstep.queue(), SoundQueue::Footsteps);
        assert_eq!(SoundRequest::Boosting.queue(), SoundQueue::Footsteps);
        for sound in [
            SoundRequest::Footstep,
            SoundRequest::Boosting,
            SoundRequest::PlainSpin,
            SoundRequest::SpaceJump,
            SoundRequest::ScrewAttack,
        ] {
            assert_eq!(sound.max_waiting(), 6);
        }
        for sound in [
            SoundRequest::PlainSpin,
            SoundRequest::SpaceJump,
            SoundRequest::ScrewAttack,
            SoundRequest::SpinEnded,
            SoundRequest::SamusSilence,
        ] {
            assert_eq!(sound.queue(), SoundQueue::Samus);
        }
        assert_eq!(SoundRequest::SpinEnded.max_waiting(), 15);
        assert_eq!(SoundRequest::SamusSilence.max_waiting(), 15);
    }
}
