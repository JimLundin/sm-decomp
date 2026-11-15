//! Public save/restore, admission descriptions and range edges.
#[cfg(test)]
mod retained {
    use samus_appearance::{
        Charge, Frames, Laps, LapsLeft, Playhead, SoundQueue, SoundRequest, State, Step,
    };
    use types::BoostStage;

    #[test]
    fn state_serializes_every_quantity_and_restores_it() {
        for stage in BoostStage::ALL {
            for laps_left in [
                LapsLeft::WithoutBooster,
                LapsLeft::WithBooster(Laps::new(u8::MAX).unwrap()),
            ] {
                let state = State {
                    playhead: Playhead {
                        step: Step(u8::MAX),
                        time_left: Frames(u8::MAX),
                    },
                    stage,
                    laps_left,
                };
                let text = toml::to_string(&state).unwrap();
                assert!(text.contains("step = 255"));
                assert!(text.contains("time_left = 255"));
                assert!(text.contains("stage ="));
                assert!(text.contains("laps_left"));
                assert_eq!(toml::from_str::<State>(&text).unwrap(), state);
            }
        }
    }

    #[test]
    fn restored_laps_cannot_bypass_the_positive_range() {
        let text =
            "stage = 'Stage0'\n[playhead]\nstep = 0\ntime_left = 0\n[laps_left]\nWithBooster = 0\n";
        assert!(toml::from_str::<State>(text).is_err());
        assert_eq!(Laps::new(0), None);
        assert_eq!(Charge::new(0).unwrap().frames(), 0);
        assert_eq!(Charge::new(Charge::MOST).unwrap().frames(), Charge::MOST);
        assert_eq!(Charge::new(Charge::MOST + 1), None);
    }

    #[test]
    fn contact_suppressor_combinations_keep_graphics() {
        use samus_appearance::{
            Clip, ContactSuppressor, Control, Figure, Hold, Instruction, Opportunity,
        };
        use types::{Action, Aim, Body, Facing, Gun, Legs, Pose, Set, Sprite};
        let figure = Figure {
            pose: Pose {
                body: Body::Upright,
                action: Action::Running,
                facing: Facing::Right,
                aim: Aim::Forward,
                gun: Gun::Stowed,
                legs: Legs::Together,
            },
            sprite: Sprite::Plain,
        };
        let mut clip = Clip {
            instructions: vec![
                Instruction::Show(Hold::new(1).unwrap()),
                Instruction::Control(Control::Rest),
            ],
            kick: Some(Step::FIRST),
            fast_fall: None,
            rising: None,
            contacts: [Step::FIRST, Step::SECOND, Step(u8::MAX)]
                .into_iter()
                .collect(),
        };
        for cinematic in [false, true] {
            for boss in [false, true] {
                for colours in [false, true] {
                    let suppressors: Set<ContactSuppressor> = [
                        (ContactSuppressor::Cinematic, cinematic),
                        (ContactSuppressor::Boss, boss),
                        (ContactSuppressor::SpecialColours, colours),
                    ]
                    .into_iter()
                    .collect();
                    let output = Opportunity::Contact {
                        figure,
                        playhead: Playhead {
                            step: Step::FIRST,
                            time_left: Frames::LAST,
                        },
                        stage: BoostStage::Stage3,
                        clip: &clip,
                        suppressors,
                    }
                    .read();
                    assert!(output.contact.is_some());
                    assert_eq!(
                        output.sound,
                        (!cinematic && !boss && !colours).then_some(SoundRequest::Footstep)
                    );
                }
            }
        }
        // Shape relations are importer's work; the model stays total with control/missing marks.
        clip.instructions.clear();
        assert_eq!(
            Opportunity::Contact {
                figure,
                playhead: Playhead {
                    step: Step::FIRST,
                    time_left: Frames::LAST
                },
                stage: BoostStage::Stage0,
                clip: &clip,
                suppressors: Set::default()
            }
            .read()
            .contact,
            None
        );
    }

    #[test]
    fn drawing_and_passage_have_no_hidden_engine_admission_guards() {
        use samus_appearance::{CinematicActivity, Figure, Opportunity};
        use types::{Action, Aim, Body, Facing, Gun, Legs, Pose, Set, Sprite};
        let from = Figure {
            pose: Pose {
                body: Body::Upright,
                action: Action::Running,
                facing: Facing::Right,
                aim: Aim::Forward,
                gun: Gun::Stowed,
                legs: Legs::Together,
            },
            sprite: Sprite::Plain,
        };
        let selected = Figure {
            pose: Pose {
                action: Action::Spinning,
                ..from.pose
            },
            ..from
        };
        assert_eq!(
            Opportunity::PoseSelected {
                from,
                selected,
                items: Set::default(),
                submerged: false,
                cinematic: CinematicActivity::Inactive
            }
            .read()
            .sound,
            Some(SoundRequest::PlainSpin)
        );
        assert_eq!(
            Opportunity::NormalDraw {
                previous: Action::WallJumping,
                current: Action::Unmorphing
            }
            .read()
            .sound,
            Some(SoundRequest::SpinEnded)
        );
        assert_eq!(
            Opportunity::PassageEntry {
                current: Action::Spinning,
                shoot_held: true,
                charge: Charge::new(Charge::MOST).unwrap()
            }
            .read()
            .sound,
            Some(SoundRequest::SpinEnded)
        );
        assert_eq!(
            Opportunity::PassageEntry {
                current: Action::Running,
                shoot_held: false,
                charge: Charge::new(Charge::MOST).unwrap()
            }
            .read()
            .sound,
            None
        );
    }

    #[test]
    fn requests_describe_admission_not_driver_work() {
        assert_eq!(SoundRequest::Footstep.queue(), SoundQueue::Footsteps);
        for sound in [
            SoundRequest::Footstep,
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
