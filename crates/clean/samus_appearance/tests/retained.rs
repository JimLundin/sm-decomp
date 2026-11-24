//! Her appearance State saved and restored, the ranges its parts keep, and whole frames whose
//! sound rules have no hidden guards and ask in the frame's order.
#[cfg(test)]
mod retained {
    use enum_map::EnumMap;
    use samus_appearance::{
        Activity, Animation, AnimationContent, AppearanceState, CannonDelay, Charge,
        ChargeGlowPosition, Clip, ColourSequences, Control, ControlEligibility, Dash, Figure,
        Frame, FrameOutputs, FramePart, Frames, Hold, Inputs, Instruction, ItemControlAnswer,
        ItemControls, Laps, LapsLeft, Liquid, Pace, Paces, PaletteOpportunity, Playhead, Rules,
        SelectedItem, ShownFrames, Sound, SoundQueue, SoundsWaiting, Step, VisorDelay, VisorStep,
        update,
    };
    use types::{
        Action, Aim, Body, BoostStage, Facing, Gun, Item, Legs, Pose, QueueAllowance, Set, Sprite,
        ZERO, set_of,
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
        Hold::new(frames).expect("expected a hold of 1 to 127 frames")
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

    impl AnimationContent for OneClip {
        fn clip(&self, _: &Figure) -> Option<&Clip> {
            Some(&self.clip)
        }

        fn paces(&self) -> &Paces {
            &self.paces
        }
    }

    const COLOURS: ColourSequences = ColourSequences {
        visor_delay: VisorDelay::new(7),
        visor_step: VisorStep::new(5),
        charge_glow_position: ChargeGlowPosition::new(2),
    };

    /// Her appearance State with `animation`, a kept spin, her cannon waiting a frame and some
    /// colours.
    fn appearance(animation: Animation) -> AppearanceState {
        AppearanceState {
            animation,
            kept_action: Action::Spinning,
            cannon: CannonDelay::new(1).expect("expected a delay of 1 in range"),
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
            sounds_waiting: SoundsWaiting::new(0).expect("expected 0 sounds waiting in range"),
            activities: Set::default(),
            shoot_held: false,
            charge: Charge::new(0).expect("expected no charge in range"),
        }
    }

    /// A frame of `inputs` with `parts`, with no item controls and no palette opportunity.
    fn frame<const N: usize>(inputs: Inputs, parts: [FramePart; N]) -> Frame {
        Frame {
            inputs,
            parts: set_of(parts),
            item_controls: ItemControls::Skipped,
            palette: PaletteOpportunity::Absent,
        }
    }

    fn on_contact(stage: BoostStage, laps_left: LapsLeft) -> Animation {
        Animation {
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
            .expect("expected a clip for every figure")
    }

    /// The sounds of a frame's requests, in order.
    fn asked(outputs: &FrameOutputs) -> Vec<Sound> {
        outputs.sounds.iter().map(|offer| offer.sound).collect()
    }

    #[test]
    fn her_animation_saves_every_quantity_and_restores_it() {
        for stage in BoostStage::ALL {
            for laps_left in [
                LapsLeft::WithoutBooster,
                LapsLeft::WithBooster(Laps::new(u8::MAX).expect("expected 255 laps in range")),
            ] {
                let animation = Animation {
                    playhead: Playhead {
                        step: Step(u8::MAX),
                        time_left: Frames(u8::MAX),
                    },
                    stage,
                    laps_left,
                };
                let text = toml::to_string(&animation).expect("expected her animation to save");
                for part in ["step = 255", "time_left = 255", "stage =", "laps_left"] {
                    assert!(text.contains(part), "expected {part} in {text}");
                }
                assert_eq!(
                    toml::from_str::<Animation>(&text).expect("expected her animation to restore"),
                    animation,
                    "expected her animation restored as saved"
                );
            }
        }
    }

    #[test]
    fn the_appearance_state_saves_all_it_remembers() {
        let state = AppearanceState {
            kept_action: Action::WallJumping,
            ..appearance(on_contact(BoostStage::Stage3, LapsLeft::WithoutBooster))
        };
        let text = toml::to_string(&state).expect("expected the appearance State to save");
        for part in [
            "animation",
            "kept_action = \"WallJumping\"",
            "cannon",
            "colours",
            "visor_delay = 7",
            "visor_step = 5",
            "charge_glow_position = 2",
        ] {
            assert!(text.contains(part), "expected {part} in {text}");
        }
        assert_eq!(
            toml::from_str::<AppearanceState>(&text).expect("expected the State to restore"),
            state,
            "expected the State restored as saved"
        );
    }

    #[test]
    fn restored_laps_cannot_bypass_the_positive_range() {
        let text =
            "stage = 'Stage0'\n[playhead]\nstep = 0\ntime_left = 0\n[laps_left]\nWithBooster = 0\n";
        assert!(
            toml::from_str::<Animation>(text).is_err(),
            "expected a restored 0 laps with the booster refused"
        );
        assert_eq!(Laps::new(0), None, "expected 0 laps refused");
        assert_eq!(
            Charge::new(0).map(Charge::frames),
            Some(0),
            "expected no charge built"
        );
        assert_eq!(
            Charge::new(Charge::MOST).map(Charge::frames),
            Some(Charge::MOST),
            "expected the most charge built"
        );
        assert_eq!(
            Charge::new(Charge::MOST + 1),
            None,
            "expected a charge past the most refused"
        );
    }

    /// A tuner's frames shown are 1 to 126, built and restored alike, and the vanilla tuning's
    /// are the frames she shows: 15, 7 and 3.
    #[test]
    fn frames_shown_are_checked_and_counted_as_shown() {
        assert_eq!(ShownFrames::new(0), None, "expected 0 frames refused");
        assert_eq!(ShownFrames::new(127), None, "expected 127 frames refused");
        assert_eq!(
            ShownFrames::new(126).map(ShownFrames::count),
            Some(126),
            "expected 126 frames built"
        );
        let vanilla = Rules::VANILLA;
        let counts = [
            vanilla.restart_frames,
            vanilla.fast_fall_frames,
            vanilla.rising_frames,
        ]
        .map(ShownFrames::count);
        assert_eq!(counts, [15, 7, 3], "expected the spec's frames shown");
        let text = toml::to_string(&vanilla).expect("expected the tuning to save");
        assert_eq!(
            toml::from_str::<Rules>(&text).expect("expected the tuning to restore"),
            vanilla,
            "expected the tuning restored as saved"
        );
        let beyond = text.replace("restart_frames = 15", "restart_frames = 127");
        assert_ne!(beyond, text, "expected restart_frames saved as 15: {text}");
        assert!(
            toml::from_str::<Rules>(&beyond).is_err(),
            "expected a restored 127 frames refused"
        );
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
            let contact = outputs.contact.expect("expected a graphical contact");
            assert_eq!(
                contact.playhead, before.animation.playhead,
                "expected the contact's playhead as she starts the frame"
            );
            let quiet = cinematic || boss || colours;
            let footstep = (!quiet).then_some(Sound::Footstep);
            assert_eq!(
                asked(&outputs),
                footstep.into_iter().collect::<Vec<_>>(),
                "expected a footstep unless a suppressor is active"
            );
        }
    }

    #[test]
    fn a_contact_on_a_control_is_none() {
        // Shape relations are the importer's: the model answers on a mark it can't show.
        let mut clip = stride_clip();
        clip.contacts = std::iter::once(Step::SECOND).collect();
        let animation = Animation {
            playhead: Playhead {
                step: Step::SECOND,
                time_left: Frames::LAST,
            },
            ..on_contact(BoostStage::Stage0, LapsLeft::WithoutBooster)
        };
        let running = upright(Action::Running);
        let (_, outputs) = update(
            appearance(animation),
            &frame(inputs(running, running), [FramePart::Movement]),
            &OneClip::of(clip),
            &Rules::VANILLA,
        )
        .expect("expected a clip for every figure");
        assert_eq!(outputs.contact, None, "expected no contact on a control");
        assert_eq!(asked(&outputs), [], "expected no sound");
    }

    #[test]
    fn drawing_and_passage_have_no_hidden_engine_admission_guards() {
        let most = Charge::new(Charge::MOST).expect("expected the most charge in range");
        let spinning = upright(Action::Spinning);
        let cases = [
            (
                frame(
                    inputs(spinning, upright(Action::Unmorphing)),
                    [FramePart::Drawing],
                ),
                vec![Sound::SpinEnded],
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
                vec![Sound::SpinEnded],
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
            assert_eq!(after, before, "expected the State as it was: {frame:?}");
            assert_eq!(
                asked(&outputs),
                expected,
                "expected the rule's sound alone: {frame:?}"
            );
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
        assert_eq!(after, before, "expected the State as it was");
        assert_eq!(
            asked(&outputs),
            [Sound::SamusSilence, Sound::SpinEnded],
            "expected the door's request, then drawing's"
        );
    }

    /// Her initialization is a part of its own, first in its frame, and the frame plays on from
    /// it: drawing reads the kept action initialization leaves, `Still`, and the frame leaves
    /// her State as at initialization, whatever State it was given.
    #[test]
    fn a_frame_plays_on_from_her_initialization() {
        let falling = upright(Action::Falling);
        let frame = frame(
            inputs(falling, falling),
            [FramePart::Initialization, FramePart::Drawing],
        );
        let before = appearance(on_contact(BoostStage::Boosting, LapsLeft::WithoutBooster));
        let (after, outputs) = play(before, &frame);
        assert_eq!(
            after,
            AppearanceState::INITIAL,
            "expected her whole State at initialization"
        );
        assert_eq!(asked(&outputs), [], "expected no spin's end");
        assert!(
            !outputs.cannon_may_follow,
            "expected the cannon waiting after initialization"
        );
    }

    /// Her initialization with her item controls in the same frame: initialization sets the
    /// delay to 2 first, and the controls then bring it down.
    #[test]
    fn item_controls_play_on_from_her_initialization() {
        let standing = upright(Action::Still);
        let frame = Frame {
            item_controls: ItemControls::Ran(ItemControlAnswer {
                before: SelectedItem::None,
                after: SelectedItem::None,
                eligibility: ControlEligibility::Eligible,
            }),
            ..frame(inputs(standing, standing), [FramePart::Initialization])
        };
        let before = AppearanceState {
            cannon: CannonDelay::new(0).expect("expected a delay of 0 in range"),
            ..appearance(on_contact(BoostStage::Stage0, LapsLeft::WithoutBooster))
        };
        let (after, _) = play(before, &frame);
        assert_eq!(
            after.cannon.remaining(),
            1,
            "expected 2 from initialization, less 1 for the unchanged controls"
        );
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
            assert_eq!(
                asked(&outputs),
                [Sound::SpinEnded],
                "expected the kept spin's end"
            );
            assert_eq!(
                after.kept_action,
                Action::Spinning,
                "expected the kept spin unchanged"
            );
            state = after;
        }
        let prepared = frame(
            inputs(running, falling),
            [FramePart::Preparation, FramePart::Drawing],
        );
        let (after, outputs) = play(state, &prepared);
        assert_eq!(asked(&outputs), [], "expected no spin's end");
        assert_eq!(
            after.kept_action,
            Action::Running,
            "expected the prepared frame to keep her run"
        );
    }

    /// One frame whose stride, animation update, spin start and passage entry each ask for a
    /// sound: all four, in the frame's order, with the one animation update's State.
    #[test]
    fn the_frames_sounds_come_in_the_frames_order() {
        let (running, spinning) = (upright(Action::Running), upright(Action::Spinning));
        let at_start = on_contact(
            BoostStage::Stage3,
            LapsLeft::WithBooster(Laps::new(1).expect("expected a lap in range")),
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
            asked(&outputs),
            [
                Sound::Footstep,
                Sound::Boosting,
                Sound::PlainSpin,
                Sound::SpinEnded
            ],
            "expected the stride's, the update's, the spin start's and the door's, in order"
        );
        assert!(outputs.contact.is_some(), "expected the stride's contact");
        assert_eq!(
            after.animation.stage,
            BoostStage::Boosting,
            "expected her boosting"
        );
        assert_eq!(
            after.animation.playhead.step,
            Step::FIRST,
            "expected her spin's clip from step 0"
        );
        assert_eq!(
            after.cannon,
            appearance(at_start).cannon,
            "expected the cannon delay unchanged"
        );
        assert_eq!(after.colours, COLOURS, "expected her colours unchanged");
    }

    /// Each request the frame reports carries its sound's queue and allowance, as the sound
    /// table gives them.
    #[test]
    fn each_request_carries_its_queue_and_allowance() {
        let rows = [
            (Sound::Footstep, SoundQueue::Footsteps, 6),
            (Sound::Boosting, SoundQueue::Footsteps, 6),
            (Sound::PlainSpin, SoundQueue::Samus, 6),
            (Sound::SpaceJump, SoundQueue::Samus, 6),
            (Sound::ScrewAttack, SoundQueue::Samus, 6),
            (Sound::SpinEnded, SoundQueue::Samus, 15),
            (Sound::SamusSilence, SoundQueue::Samus, 15),
        ];
        for (sound, queue, allowance) in rows {
            let offer = sound.offer();
            assert_eq!(offer.sound, sound, "expected the request of {sound:?}");
            assert_eq!(offer.queue, queue, "expected the queue of {sound:?}");
            assert_eq!(
                offer.allowance,
                QueueAllowance(allowance),
                "expected the allowance of {sound:?}"
            );
        }
        let running = upright(Action::Running);
        let before = appearance(on_contact(BoostStage::Stage0, LapsLeft::WithoutBooster));
        let (_, outputs) = play(
            before,
            &frame(inputs(running, running), [FramePart::Movement]),
        );
        assert_eq!(
            outputs.sounds,
            [Sound::Footstep.offer()],
            "expected the footstep's request, with its queue and allowance"
        );
    }
}
