//! First genuine source frame diagnostic through the existing common executor.

use super::runtime_helpers::admitted;
use super::*;
use data::assets::world::rooms::{CERES_ELEVATOR_ROOM, CERES_RIDLEY_ROOM};
use data::timing::MasterTicks;

#[test]
#[ignore = "requires reviewed integrated current room and player ROM"]
fn actual_room_owner_identity_guards_dependent_boundaries_and_holds() {
    let mut admitted = admitted();
    let InputReader::Frame(inputs) = &admitted.inputs else {
        panic!("frame inputs");
    };
    let budget = inputs.budget;
    for phase in 0..10 {
        advance_to_room_phase(&mut admitted, budget, phase);
        let game::state::owned::OwnedState::Frame(actual) = admitted.production.state() else {
            panic!("frame state");
        };
        let ProductionContent::Frame(content) = &admitted.content else {
            panic!("frame content");
        };
        let ProductionRules::Frame(rules) = &admitted.rules else {
            panic!("frame rules");
        };
        let mut changed = (**actual).clone();
        let current = changed
            .inside_world
            .as_ref()
            .unwrap()
            .0
            .room_identity()
            .unwrap();

        let wrong = if current == CERES_RIDLEY_ROOM {
            CERES_ELEVATOR_ROOM
        } else {
            CERES_RIDLEY_ROOM
        };
        let room = content.movement_room.as_ref().unwrap();
        changed.inside_world = Some(Model(
            frame::inside_world::InsideWorld::for_room(
                wrong,
                room.width,
                room.cells.clone(),
                room.boundary,
            )
            .unwrap(),
        ));
        let before = changed.clone();
        assert!(
            changed.advance(content, rules, budget).is_err(),
            "actual room owner identity phase {phase}"
        );
        assert!(
            changed == before,
            "actual room identity rejection atomicity"
        );
        let saved = admitted.production.save().unwrap();
        let mut receiver =
            Production::restore(&saved, admitted.content.clone(), admitted.rules.clone()).unwrap();
        let left = admitted
            .production
            .advance(game::production::AdvanceInput::Frame { budget })
            .unwrap();
        let right = receiver
            .advance(game::production::AdvanceInput::Frame { budget })
            .unwrap();
        assert!(left == right && receiver.state() == admitted.production.state());
    }
}

fn advance_to_room_phase(admitted: &mut AdmittedCase, budget: MasterTicks, phase: u8) {
    let mut found = false;
    for _ in 0..100_000 {
        if let game::state::owned::OwnedState::Frame(state) = admitted.production.state()
            && let frame::Caller::PlayerContinuation {
                run,
                ordinary,
                support,
                ground,
                down,
                post_down,
                charging,
                ..
            } = &state.caller
        {
            found = match phase {
                0 => {
                    matches!(run, frame::movement_caller::Run::Complete(_))
                        && ordinary.is_none()
                        && !charging
                }
                1 => {
                    ordinary
                        .as_ref()
                        .is_some_and(frame::OrdinaryWork::is_charging)
                        && *charging
                }
                2 => {
                    ordinary
                        .as_ref()
                        .is_some_and(frame::OrdinaryWork::is_complete)
                        && matches!(support, frame::movement_support_contract::Stage::Idle)
                        && !charging
                }
                3 => {
                    matches!(
                        support,
                        frame::movement_support_contract::Stage::Charging { .. }
                    ) && *charging
                }
                4 => {
                    matches!(
                        support,
                        frame::movement_support_contract::Stage::Complete(_)
                    ) && matches!(ground, frame::movement_ground_contract::Stage::Idle)
                        && !charging
                }
                5 => {
                    matches!(ground, frame::movement_ground_contract::Stage::Charging(_))
                        && *charging
                }
                6 => {
                    matches!(ground, frame::movement_ground_contract::Stage::Complete(_))
                        && matches!(down, frame::movement_down_contract::Stage::Idle)
                        && !charging
                }
                7 => {
                    matches!(down, frame::movement_down_contract::Stage::Charging { .. })
                        && *charging
                }
                8 => {
                    matches!(down, frame::movement_down_contract::Stage::Complete(_))
                        && matches!(post_down, frame::post_down_caller::Stage::Idle)
                        && !charging
                }
                _ => {
                    matches!(post_down, frame::post_down_caller::Stage::Contact { .. }) && *charging
                }
            };
            if found {
                break;
            }
        }
        admitted
            .production
            .advance(game::production::AdvanceInput::Frame { budget })
            .unwrap();
    }
    assert!(found, "actual current room phase {phase}");
}

#[test]
#[ignore = "requires actual reviewed Contact source and player ROM"]
fn actual_contact_room_identity_guards_boundary_and_hold() {
    let path = std::env::var_os("SM_FRAME_DIAGNOSTIC_CASE").unwrap();
    let case: Case = toml::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let sources = Sources {
        rom: std::env::var_os("SM_ROM").unwrap().into(),
        ..Sources::default()
    };
    let mut admitted = admit(&case, &sources).unwrap();
    let InputReader::Frame(inputs) = &admitted.inputs else {
        panic!("frame inputs");
    };
    let budget = inputs.budget;
    for phase in 0..2 {
        let mut found = false;
        for _ in 0..100_000 {
            if let game::state::owned::OwnedState::Frame(state) = admitted.production.state()
                && let frame::Caller::PlayerContinuation {
                    down,
                    post_down,
                    charging,
                    ..
                } = &state.caller
            {
                found = if phase == 0 {
                    matches!(down, frame::movement_down_contract::Stage::Complete(_))
                        && matches!(post_down, frame::post_down_caller::Stage::Idle)
                        && !charging
                } else {
                    matches!(post_down, frame::post_down_caller::Stage::Contact { .. }) && *charging
                };
                if found {
                    break;
                }
            }
            admitted
                .production
                .advance(game::production::AdvanceInput::Frame { budget })
                .unwrap();
        }
        assert!(found, "actual Contact room phase {phase}");
        let game::state::owned::OwnedState::Frame(actual) = admitted.production.state() else {
            panic!("frame state");
        };
        let ProductionContent::Frame(content) = &admitted.content else {
            panic!("frame content");
        };
        let ProductionRules::Frame(rules) = &admitted.rules else {
            panic!("frame rules");
        };
        let mut changed = (**actual).clone();
        let current = &mut changed.contact_current.as_mut().unwrap().0.context;
        assert_observation_eq!(current.area, types::Area::Ceres);

        current.room = if current.room == CERES_RIDLEY_ROOM {
            CERES_ELEVATOR_ROOM
        } else {
            CERES_RIDLEY_ROOM
        };
        let before = changed.clone();
        let result = changed.advance(content, rules, budget);
        assert!(
            result.is_err(),
            "actual same-area Contact room identity phase {phase}"
        );
        assert!(changed == before, "Contact room rejection atomicity");
        let saved = admitted.production.save().unwrap();
        let mut receiver =
            Production::restore(&saved, admitted.content.clone(), admitted.rules.clone()).unwrap();
        let left = admitted
            .production
            .advance(game::production::AdvanceInput::Frame { budget })
            .unwrap();
        let right = receiver
            .advance(game::production::AdvanceInput::Frame { budget })
            .unwrap();
        assert!(left == right && receiver.state() == admitted.production.state());
    }
}

#[test]
#[ignore = "requires reviewed complete Contact/SoundOffer/Poststep source and player ROM"]
fn actual_post_down_owners_hold_current_and_restore() {
    let mut admitted = admitted();
    let InputReader::Frame(inputs) = &admitted.inputs else {
        panic!("frame inputs");
    };
    let budget = inputs.budget;
    for phase in 0..6 {
        advance_to_post_down_phase(&mut admitted, budget, phase);
        let game::state::owned::OwnedState::Frame(actual) = admitted.production.state() else {
            panic!("frame state");
        };
        let ProductionContent::Frame(content) = &admitted.content else {
            panic!("frame content");
        };
        let ProductionRules::Frame(rules) = &admitted.rules else {
            panic!("frame rules");
        };
        for mutation in 0..5 {
            let mut changed = (**actual).clone();
            let mut changed_content = content.as_ref().clone();
            damage_post_down(&mut changed, &mut changed_content, phase, mutation);
            let before = changed.clone();
            assert!(
                changed.advance(&changed_content, rules, budget).is_err(),
                "post-Down phase {phase} mutation {mutation}"
            );
            assert!(changed == before, "post-Down rejection atomicity");
        }
        let saved = admitted.production.save().unwrap();
        let mut receiver =
            Production::restore(&saved, admitted.content.clone(), admitted.rules.clone()).unwrap();
        assert!(receiver.state().eq(admitted.production.state()));
        let left = admitted
            .production
            .advance(game::production::AdvanceInput::Frame { budget })
            .unwrap();
        let right = receiver
            .advance(game::production::AdvanceInput::Frame { budget })
            .unwrap();
        assert!(left.eq(&right));
        assert!(receiver.state().eq(admitted.production.state()));
    }
    for _ in 0..100_000 {
        if let game::state::owned::OwnedState::Frame(state) = admitted.production.state()
            && let frame::Caller::PlayerContinuation {
                post_down: post_down @ frame::post_down_caller::Stage::Complete(_),
                ..
            } = &state.caller
        {
            assert_observation_eq!(
                post_down.completed_identities(),
                [
                    game::production::timing::WorkId(1100),
                    game::production::timing::WorkId(1105),
                    game::production::timing::WorkId(1106)
                ]
            );
            return;
        }
        admitted
            .production
            .advance(game::production::AdvanceInput::Frame { budget })
            .unwrap();
    }
    panic!("actual distinct post-Down completions absent");
}

fn damage_post_down(
    changed: &mut frame::State,
    changed_content: &mut frame::Content,
    phase: u8,
    mutation: u8,
) {
    match (phase, mutation) {
        (0 | 1, 0) => {
            changed.contact_current.as_mut().unwrap().0.suppressors
                [samus_appearance::ContactSuppressor::Cinematic] = true;
        }
        (0 | 1, 1) => {
            changed.contact_current.as_mut().unwrap().0.context.area = types::Area::Crateria;
        }
        (0 | 1, 2) => {
            changed.contact_current.as_mut().unwrap().0.context.water =
                frame::contact_contract::Surface::Present;
        }
        (2 | 3, 0) => {
            changed.sound_offer_current.as_mut().unwrap().sounds =
                frame::post_down_contract::Sounds::Disabled;
        }
        (2 | 3, 1) => {
            changed.sound_offer_current.as_mut().unwrap().demo =
                frame::post_down_contract::Demo::Active;
        }
        (2 | 3, 2) => {
            changed.sound_offer_current.as_mut().unwrap().power_bomb =
                frame::post_down_contract::PowerBomb::Active;
        }
        (4 | 5, 0) => changed.appearance.model.0.state.stage = types::BoostStage::Stage1,
        (4 | 5, 1) => changed.appearance.model.0.state.stage = types::BoostStage::Stage2,
        (_, 3) => changed.appearance.model.0.state.stage = types::BoostStage::Boosting,
        (0 | 1, 4) => {
            changed_content.contact = Some(frame::contact_contract::PhysicalContract::new(
                changed_content
                    .post_down
                    .as_ref()
                    .unwrap()
                    .sound_offer
                    .clone(),
            ));
        }
        (2 | 3, 4) => {
            let run = changed_content
                .post_down
                .as_mut()
                .unwrap()
                .sound_offer
                .runs
                .first_mut()
                .unwrap();
            run.count = run.count.wrapping_add(1);
        }
        (4 | 5, 4) => {
            let run = changed_content
                .post_down
                .as_mut()
                .unwrap()
                .poststep
                .runs
                .first_mut()
                .unwrap();
            run.count = run.count.wrapping_add(1);
        }
        _ => {
            let frame::movement::Movement::Work(work) = &mut changed.movement else {
                panic!("movement work");
            };
            work.model.0.state.run.speed.0 = work.model.0.state.run.speed.0.wrapping_add(1);
        }
    }
}

fn advance_to_post_down_phase(admitted: &mut AdmittedCase, budget: MasterTicks, phase: u8) {
    let mut found = false;
    for _ in 0..100_000 {
        if let game::state::owned::OwnedState::Frame(state) = admitted.production.state()
            && let frame::Caller::PlayerContinuation {
                down,
                post_down,
                charging,
                ..
            } = &state.caller
        {
            found = match phase {
                0 => {
                    matches!(down, frame::movement_down_contract::Stage::Complete(_))
                        && matches!(post_down, frame::post_down_caller::Stage::Idle)
                        && !charging
                }
                1 => {
                    matches!(post_down, frame::post_down_caller::Stage::Contact { .. }) && *charging
                }
                2 => {
                    matches!(
                        post_down,
                        frame::post_down_caller::Stage::ContactComplete(_)
                    ) && !charging
                }
                3 => {
                    matches!(post_down, frame::post_down_caller::Stage::SoundOffer { .. })
                        && *charging
                }
                4 => {
                    matches!(post_down, frame::post_down_caller::Stage::SoundComplete(_))
                        && !charging
                }
                _ => {
                    matches!(post_down, frame::post_down_caller::Stage::Poststep { .. })
                        && *charging
                }
            };
            if found {
                break;
            }
        }
        admitted
            .production
            .advance(game::production::AdvanceInput::Frame { budget })
            .unwrap();
    }
    assert!(found, "actual post-Down phase {phase}");
}

#[test]
#[ignore = "requires reviewed whole Inside and downward source admission and player ROM"]
fn actual_inside_provenance_guards_boundaries_and_holds() {
    let mut admitted = admitted();
    let InputReader::Frame(inputs) = &admitted.inputs else {
        panic!("frame inputs")
    };
    let budget = inputs.budget;
    for phase in 0..4 {
        advance_to_inside_phase(&mut admitted, budget, phase);
        let game::state::owned::OwnedState::Frame(actual) = admitted.production.state() else {
            panic!("frame state")
        };
        let ProductionContent::Frame(content) = &admitted.content else {
            panic!("frame content")
        };
        let ProductionRules::Frame(rules) = &admitted.rules else {
            panic!("frame rules")
        };
        for mutation in 0..3 {
            let mut changed = (**actual).clone();
            match mutation {
                0 => {
                    let inside = changed.inside_completed.as_mut().unwrap();
                    let frame::movement::Movement::Work(work) = &mut inside.retained else {
                        panic!("held Inside owner")
                    };
                    work.model.0.state.run.speed.0 = work.model.0.state.run.speed.0.wrapping_add(1);
                }
                1 => {
                    changed.inside_completed.as_mut().unwrap().receipt.group =
                        frame::movement_stage::Group::Decision;
                }
                _ => {
                    let room = content.movement_room.as_ref().unwrap();
                    let mut cells = room.cells.clone();
                    let first = cells.first_mut().unwrap();
                    *first = if *first == types::Block::Air {
                        types::Block::Solid
                    } else {
                        types::Block::Air
                    };
                    changed.inside_world = Some(Model(
                        frame::inside_world::InsideWorld::new(room.width, cells, room.boundary)
                            .unwrap(),
                    ));
                }
            }
            let before = changed.clone();
            assert!(
                changed.advance(content, rules, budget).is_err(),
                "Inside provenance phase {phase} mutation {mutation}"
            );
            assert!(changed.eq(&before));
        }
        let saved = admitted.production.save().unwrap();
        let mut restored =
            Production::restore(&saved, admitted.content.clone(), admitted.rules.clone()).unwrap();
        let left = admitted
            .production
            .advance(game::production::AdvanceInput::Frame { budget })
            .unwrap();
        let right = restored
            .advance(game::production::AdvanceInput::Frame { budget })
            .unwrap();
        assert!(left.eq(&right));
        assert!(restored.state().eq(admitted.production.state()));
    }
}

fn advance_to_inside_phase(admitted: &mut AdmittedCase, budget: MasterTicks, phase: u8) {
    let mut found = false;
    for _ in 0..100_000 {
        if let game::state::owned::OwnedState::Frame(state) = admitted.production.state()
            && let frame::Caller::PlayerContinuation {
                support,
                ground,
                down,
                charging,
                ..
            } = &state.caller
        {
            found = match phase {
                0 => {
                    matches!(
                        support,
                        frame::movement_support_contract::Stage::Complete(_)
                    ) && matches!(ground, frame::movement_ground_contract::Stage::Idle)
                        && !charging
                }
                1 => {
                    matches!(ground, frame::movement_ground_contract::Stage::Charging(_))
                        && *charging
                }
                2 => {
                    matches!(ground, frame::movement_ground_contract::Stage::Complete(_))
                        && matches!(down, frame::movement_down_contract::Stage::Idle)
                        && !charging
                }
                _ => {
                    matches!(down, frame::movement_down_contract::Stage::Charging { .. })
                        && *charging
                }
            };
            if found {
                break;
            }
        }
        admitted
            .production
            .advance(game::production::AdvanceInput::Frame { budget })
            .unwrap();
    }
    assert!(found, "actual Inside provenance phase {phase}");
}

#[test]
#[ignore = "requires reviewed downward source admission and player ROM"]
fn actual_down_admission_reports_phase() {
    let path = std::env::var_os("SM_FRAME_DIAGNOSTIC_CASE").unwrap();
    let case: Case = toml::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let sources = Sources {
        rom: std::env::var_os("SM_ROM").unwrap().into(),
        ..Sources::default()
    };
    let mut admitted = admit(&case, &sources).unwrap();
    let InputReader::Frame(inputs) = &admitted.inputs else {
        panic!("frame inputs")
    };
    let budget = inputs.budget;
    for _ in 0..100_000 {
        if matches!(admitted.production.state(), game::state::owned::OwnedState::Frame(state)
            if matches!(state.caller, frame::Caller::PlayerContinuation {
                ground: frame::movement_ground_contract::Stage::Complete(_),
                down: frame::movement_down_contract::Stage::Idle, charging: false, .. }))
        {
            break;
        }
        admitted
            .production
            .advance(game::production::AdvanceInput::Frame { budget })
            .unwrap();
    }
    let game::state::owned::OwnedState::Frame(state) = admitted.production.state() else {
        panic!("frame state")
    };
    let ProductionContent::Frame(content) = &admitted.content else {
        panic!("frame content")
    };
    let ProductionRules::Frame(rules) = &admitted.rules else {
        panic!("frame rules")
    };
    let frame::Caller::PlayerContinuation {
        remaining,
        run: frame::movement_caller::Run::Complete(run),
        ground: frame::movement_ground_contract::Stage::Complete(ground),
        ..
    } = &state.caller
    else {
        panic!("actual preDown")
    };
    let current = frame::movement_caller::Current {
        appearance: &state.appearance,
        figure: state.playing_figure.0,
        actors: remaining.0.actors.as_ref().unwrap(),
        inputs: &run.ahead.inputs.0,
    };
    let inputs = frame::movement_down_contract::Inputs {
        inside: state.inside_completed.as_ref().unwrap(),
        current: &current,
        room: content.movement_room.as_ref().unwrap(),
        content: &content.movement,
        rules: &rules.movement,
        ground,
    };
    let mut downward = frame::movement_down_contract::Stage::Idle;
    let chosen = downward.choose(&state.movement, &inputs);
    assert!(chosen.is_ok());
    let Some(frame::movement_down_contract::Receipt { decisions, .. }) = downward.receipt() else {
        panic!("chosen Down receipt")
    };
    let [
        _,
        samus_movement::suspended::MovementDecision::MotionAnswer { outputs, .. },
        _,
    ] = decisions.0.as_slice()
    else {
        panic!("actual Down answer")
    };
    let vertical_annotation_absent = outputs
        .collision_tests
        .iter()
        .all(|test| test.vertical_motion.is_none());
    assert!(vertical_annotation_absent);
    let physical = content
        .ground_following_down
        .as_ref()
        .unwrap()
        .physical(&downward, &state.movement, &inputs)
        .map(|_| ());
    assert!(physical.is_ok());
}

#[test]
#[ignore = "requires reviewed ordinary and support source admission and player ROM"]
fn actual_support_rejects_actor_replacement_after_ordinary() {
    let path = std::env::var_os("SM_FRAME_DIAGNOSTIC_CASE").unwrap();
    let case: Case = toml::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let sources = Sources {
        rom: std::env::var_os("SM_ROM").unwrap().into(),
        ..Sources::default()
    };
    let mut admitted = admit(&case, &sources).unwrap();
    let InputReader::Frame(inputs) = &admitted.inputs else {
        panic!("frame inputs")
    };
    let budget = inputs.budget;
    let mut found = false;
    for _ in 0..100_000 {
        if matches!(admitted.production.state(), game::state::owned::OwnedState::Frame(state)
            if matches!(state.caller, frame::Caller::PlayerContinuation {
                ordinary: Some(ref ordinary), support: frame::movement_support_contract::Stage::Idle,
                charging: false, .. } if ordinary.is_complete()))
        {
            found = true;
            break;
        }
        admitted
            .production
            .advance(game::production::AdvanceInput::Frame { budget })
            .unwrap();
    }
    assert!(found);
    let game::state::owned::OwnedState::Frame(actual) = admitted.production.state() else {
        panic!("frame state")
    };
    let ProductionContent::Frame(content) = &admitted.content else {
        panic!("frame content")
    };
    let ProductionRules::Frame(rules) = &admitted.rules else {
        panic!("frame rules")
    };
    for mutation in 0..3 {
        let mut changed = (**actual).clone();
        let frame::Caller::PlayerContinuation { remaining, .. } = &mut changed.caller else {
            panic!("player continuation")
        };
        let actors = remaining.0.actors.as_mut().unwrap();
        match mutation {
            0 => {
                actors.dependency.clocks.simulation =
                    frame::borrowed_player_preparation::ClockDisposition::Stopped;
            }
            1 => {
                actors.dependency.clocks.enemy =
                    frame::borrowed_player_preparation::ClockDisposition::Stopped;
            }
            _ => actors.interactive.clear(),
        }
        let before = changed.clone();
        assert!(
            changed.advance(content, rules, budget).is_err(),
            "support actor mutation {mutation}"
        );
        assert!(changed.eq(&before));
    }
}

#[test]
#[ignore = "requires reviewed private complete-frame source manifest and player ROM"]
fn actual_recorded_frame_uses_common_executor() {
    let path = std::env::var_os("SM_FRAME_DIAGNOSTIC_CASE").unwrap();
    let case: Case = toml::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    case.validate().unwrap();
    let sources = Sources {
        rom: std::env::var_os("SM_ROM").unwrap().into(),
        ..Sources::default()
    };
    let mut admitted = admit(&case, &sources).unwrap();
    let saved = admitted.production.save().unwrap();
    let restored =
        Production::restore(&saved, admitted.content.clone(), admitted.rules.clone()).unwrap();
    assert!(
        restored.state() == admitted.production.state(),
        "complete frame origin restoration"
    );
    let outcome = super::super::runner::execute(
        &case,
        &admitted.inputs,
        &admitted.expected,
        &mut admitted.production,
        &mut admitted.actual,
        case.end.cut_inclusive,
    );
    let paused = admitted.production.save().unwrap();
    let mut receiver =
        Production::restore(&paused, admitted.content.clone(), admitted.rules.clone()).unwrap();
    assert!(
        receiver.state() == admitted.production.state(),
        "actual continuation and receiver prefix restoration"
    );
    if let InputReader::Frame(inputs) = &admitted.inputs
        && let Err(game::production::ProductionFault::FrameHardware(fault)) =
            receiver.advance(game::production::AdvanceInput::Frame {
                budget: inputs.budget,
            })
    {
        assert_ne!(fault, game::production::timing::Fault::MissingContent);
    }
    assert!(matches!(
        admitted.production.state(),
        game::state::owned::OwnedState::Frame(_)
    ));
    assert!(!matches!(outcome, Err(Outcome::DataFailure { .. })));
}

#[test]
#[ignore = "requires reviewed complete player continuation source and player ROM"]
fn actual_motion_reset_guards_current_owners_before_billing() {
    let path = std::env::var_os("SM_FRAME_DIAGNOSTIC_CASE").unwrap();
    let case: Case = toml::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let sources = Sources {
        rom: std::env::var_os("SM_ROM").unwrap().into(),
        ..Sources::default()
    };
    let mut admitted = admit(&case, &sources).unwrap();
    let InputReader::Frame(inputs) = &admitted.inputs else {
        panic!("frame inputs");
    };
    let budget = inputs.budget;
    let mut found = false;
    for _ in 0..100_000 {
        if matches!(admitted.production.state(), game::state::owned::OwnedState::Frame(state)
            if matches!(state.caller, frame::Caller::PlayerContinuation {
                stage: frame::player_continuation::Stage::MotionReset, charging: false, .. }))
        {
            found = true;
            break;
        }
        admitted
            .production
            .advance(game::production::AdvanceInput::Frame { budget })
            .unwrap();
    }
    assert!(found, "actual pre-MotionReset opportunity reached");
    let game::state::owned::OwnedState::Frame(actual) = admitted.production.state() else {
        panic!("frame state");
    };
    let ProductionContent::Frame(content) = &admitted.content else {
        panic!("frame content");
    };
    let ProductionRules::Frame(rules) = &admitted.rules else {
        panic!("frame rules");
    };
    assert!(qualify_motion_reset(actual, content).is_ok());
    for mutation in 0..3 {
        let mut changed = (**actual).clone();
        match mutation {
            0 => {
                let frame::Caller::PlayerContinuation { completed, .. } = &mut changed.caller
                else {
                    panic!("player continuation");
                };
                completed
                    .0
                    .pose_input
                    .as_mut()
                    .unwrap()
                    .dependency
                    .pressed
                    .0 ^= content.early_hud_controls.as_ref().unwrap().select;
            }
            1 => {
                let frame::movement::Movement::Work(work) = &mut changed.movement else {
                    panic!("movement work");
                };
                work.model.0.state.pose.facing = types::Facing::Left;
            }
            _ => {
                changed.inside_completed.as_mut().unwrap().work =
                    game::production::timing::WorkId(14);
            }
        }
        assert_observation_eq!(
            qualify_motion_reset(&changed, content),
            Err(frame::motion_reset_contract::Fault::Dependency)
        );
        let before = changed.clone();
        assert!(changed.advance(content, rules, budget).is_err());
        assert!(
            changed == before,
            "failed actual admission leaves complete State unchanged"
        );
        assert!(changed.timing.hardware.now.eq(&before.timing.hardware.now));
        assert!(changed.timing.work.eq(&before.timing.work));
    }
}

fn qualify_motion_reset(
    state: &frame::State,
    content: &frame::Content,
) -> Result<(), frame::motion_reset_contract::Fault> {
    let source = content.motion_reset.as_ref().unwrap();
    let frame::Caller::PlayerContinuation {
        prepared,
        completed,
        controls,
        ..
    } = &state.caller
    else {
        panic!("actual player continuation");
    };
    frame::motion_reset_contract::qualify(
        frame::motion_reset_contract::Current {
            movement: &state.movement,
            inside: state.inside_completed.as_ref().unwrap(),
            hud: &state.hud,
            controls,
            bindings: content.early_hud_controls.as_ref().unwrap(),
            control_inputs: frame::hud_controls_inputs::CurrentControls {
                gameplay: prepared.gameplay.as_ref().unwrap(),
                scene: prepared.prelude.scene_current.unwrap(),
                pose_input: completed.0.pose_input.as_ref().unwrap(),
                pose: state.movement.state().pose,
                inventory: prepared.prelude.hud_inventory.as_ref().unwrap(),
            },
        },
        source.facing,
        &source.whole,
    )
    .map(|_| ())
}

#[test]
#[ignore = "requires reviewed selected Run projection and player ROM"]
fn actual_run_holds_query_and_current_owners_atomically() {
    let path = std::env::var_os("SM_FRAME_DIAGNOSTIC_CASE").unwrap();
    let case: Case = toml::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let sources = Sources {
        rom: std::env::var_os("SM_ROM").unwrap().into(),
        ..Sources::default()
    };
    let mut admitted = admit(&case, &sources).unwrap();
    let InputReader::Frame(inputs) = &admitted.inputs else {
        panic!("frame inputs")
    };
    let budget = inputs.budget;
    let mut found = false;
    for _ in 0..100_000 {
        if matches!(admitted.production.state(), game::state::owned::OwnedState::Frame(state)
            if matches!(state.caller, frame::Caller::PlayerContinuation {
                run: frame::movement_caller::Run::Charging { .. }, charging: true, .. }))
        {
            found = true;
            break;
        }
        admitted
            .production
            .advance(game::production::AdvanceInput::Frame { budget })
            .unwrap();
    }
    assert!(found, "actual interior Run reached");
    let saved = admitted.production.save().unwrap();
    let mut restored =
        Production::restore(&saved, admitted.content.clone(), admitted.rules.clone()).unwrap();
    assert!(restored.state().eq(admitted.production.state()));
    let left = admitted
        .production
        .advance(game::production::AdvanceInput::Frame { budget })
        .unwrap();
    let right = restored
        .advance(game::production::AdvanceInput::Frame { budget })
        .unwrap();
    assert!(left.eq(&right));
    assert!(restored.state().eq(admitted.production.state()));
    let game::state::owned::OwnedState::Frame(actual) = admitted.production.state() else {
        panic!("frame state")
    };
    let ProductionContent::Frame(content) = &admitted.content else {
        panic!("frame content")
    };
    let ProductionRules::Frame(rules) = &admitted.rules else {
        panic!("frame rules")
    };
    for mutation in 0..4 {
        let mut changed = (**actual).clone();
        let mut changed_rules = rules.as_ref().clone();
        match mutation {
            0 => changed.playing_figure.0.pose.facing = types::Facing::Left,
            1 => {
                let frame::movement::Movement::Work(work) = &mut changed.movement else {
                    panic!("movement work")
                };
                work.model.0.state.run.speed.0 = work.model.0.state.run.speed.0.wrapping_add(1);
            }
            2 => {
                let frame::Caller::PlayerContinuation { controls, .. } = &mut changed.caller else {
                    panic!("player continuation")
                };
                controls.work = game::production::timing::WorkId(15);
            }
            _ => {
                changed_rules.movement.dash_gain.0 =
                    changed_rules.movement.dash_gain.0.wrapping_add(1);
            }
        }
        let before = changed.clone();
        assert!(changed.advance(content, &changed_rules, budget).is_err());
        assert!(changed.eq(&before));
        assert!(changed.timing.hardware.now.eq(&before.timing.hardware.now));
        assert!(changed.timing.work.eq(&before.timing.work));
        assert!(changed.publications.eq(&before.publications));
    }
}

#[test]
#[ignore = "requires reviewed selected movement source admission and player ROM"]
fn actual_ordinary_holds_query_and_current_owners_atomically() {
    let path = std::env::var_os("SM_FRAME_DIAGNOSTIC_CASE").unwrap();
    let case: Case = toml::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let sources = Sources {
        rom: std::env::var_os("SM_ROM").unwrap().into(),
        ..Sources::default()
    };
    let mut admitted = admit(&case, &sources).unwrap();
    let InputReader::Frame(inputs) = &admitted.inputs else {
        panic!("frame inputs")
    };
    let budget = inputs.budget;
    let mut found = false;
    for _ in 0..100_000 {
        if matches!(admitted.production.state(), game::state::owned::OwnedState::Frame(state)
            if matches!(state.caller, frame::Caller::PlayerContinuation {
                ordinary: Some(ref ordinary), charging: true, .. } if ordinary.is_charging()))
        {
            found = true;
            break;
        }
        admitted
            .production
            .advance(game::production::AdvanceInput::Frame { budget })
            .unwrap();
    }
    assert!(found, "actual interior ordinary reached");
    let saved = admitted.production.save().unwrap();
    let mut restored =
        Production::restore(&saved, admitted.content.clone(), admitted.rules.clone()).unwrap();
    assert!(restored.state().eq(admitted.production.state()));
    let left = admitted
        .production
        .advance(game::production::AdvanceInput::Frame { budget })
        .unwrap();
    let right = restored
        .advance(game::production::AdvanceInput::Frame { budget })
        .unwrap();
    assert!(left.eq(&right));
    assert!(restored.state().eq(admitted.production.state()));
    let game::state::owned::OwnedState::Frame(actual) = admitted.production.state() else {
        panic!("frame state")
    };
    let ProductionContent::Frame(content) = &admitted.content else {
        panic!("frame content")
    };
    let ProductionRules::Frame(rules) = &admitted.rules else {
        panic!("frame rules")
    };
    for mutation in 0..4 {
        let mut changed = (**actual).clone();
        let mut changed_rules = rules.as_ref().clone();
        match mutation {
            0 => changed.playing_figure.0.pose.facing = types::Facing::Left,
            1 => {
                let frame::movement::Movement::Work(work) = &mut changed.movement else {
                    panic!("movement work")
                };
                work.model.0.state.run.speed.0 = work.model.0.state.run.speed.0.wrapping_add(1);
            }
            2 => {
                let frame::Caller::PlayerContinuation { controls, .. } = &mut changed.caller else {
                    panic!("player continuation")
                };
                controls.work = game::production::timing::WorkId(15);
            }
            _ => {
                changed_rules.movement.dash_gain.0 =
                    changed_rules.movement.dash_gain.0.wrapping_add(1);
            }
        }
        let before = changed.clone();
        assert!(changed.advance(content, &changed_rules, budget).is_err());
        assert!(changed.eq(&before));
        assert!(changed.timing.hardware.now.eq(&before.timing.hardware.now));
        assert!(changed.timing.work.eq(&before.timing.work));
        assert!(changed.publications.eq(&before.publications));
    }
}

#[test]
#[ignore = "requires reviewed selected movement source admission and player ROM"]
fn actual_support_holds_query_and_current_owners_atomically() {
    let path = std::env::var_os("SM_FRAME_DIAGNOSTIC_CASE").unwrap();
    let case: Case = toml::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let sources = Sources {
        rom: std::env::var_os("SM_ROM").unwrap().into(),
        ..Sources::default()
    };
    let mut admitted = admit(&case, &sources).unwrap();
    let InputReader::Frame(inputs) = &admitted.inputs else {
        panic!("frame inputs")
    };
    let budget = inputs.budget;
    let mut found = false;
    for _ in 0..100_000 {
        if matches!(admitted.production.state(), game::state::owned::OwnedState::Frame(state)
            if matches!(state.caller, frame::Caller::PlayerContinuation {
                support: frame::movement_support_contract::Stage::Charging { .. }, charging: true, .. }))
        {
            found = true;
            break;
        }
        admitted
            .production
            .advance(game::production::AdvanceInput::Frame { budget })
            .unwrap();
    }
    assert!(found, "actual interior support reached");
    let saved = admitted.production.save().unwrap();
    let mut restored =
        Production::restore(&saved, admitted.content.clone(), admitted.rules.clone()).unwrap();
    assert!(restored.state().eq(admitted.production.state()));
    let left = admitted
        .production
        .advance(game::production::AdvanceInput::Frame { budget })
        .unwrap();
    let right = restored
        .advance(game::production::AdvanceInput::Frame { budget })
        .unwrap();
    assert!(left.eq(&right));
    assert!(restored.state().eq(admitted.production.state()));
    let game::state::owned::OwnedState::Frame(actual) = admitted.production.state() else {
        panic!("frame state")
    };
    let ProductionContent::Frame(content) = &admitted.content else {
        panic!("frame content")
    };
    let ProductionRules::Frame(rules) = &admitted.rules else {
        panic!("frame rules")
    };
    for mutation in 0..4 {
        let mut changed = (**actual).clone();
        let mut changed_rules = rules.as_ref().clone();
        match mutation {
            0 => changed.playing_figure.0.pose.facing = types::Facing::Left,
            1 => {
                let frame::movement::Movement::Work(work) = &mut changed.movement else {
                    panic!("movement work")
                };
                work.model.0.state.run.speed.0 = work.model.0.state.run.speed.0.wrapping_add(1);
            }
            2 => {
                let frame::Caller::PlayerContinuation { controls, .. } = &mut changed.caller else {
                    panic!("player continuation")
                };
                controls.work = game::production::timing::WorkId(15);
            }
            _ => {
                changed_rules.movement.dash_gain.0 =
                    changed_rules.movement.dash_gain.0.wrapping_add(1);
            }
        }
        let before = changed.clone();
        assert!(changed.advance(content, &changed_rules, budget).is_err());
        assert!(changed.eq(&before));
        assert!(changed.timing.hardware.now.eq(&before.timing.hardware.now));
        assert!(changed.timing.work.eq(&before.timing.work));
        assert!(changed.publications.eq(&before.publications));
    }
}

#[test]
#[ignore = "requires reviewed selected movement source admission and player ROM"]
fn actual_ground_holds_query_and_current_owners_atomically() {
    let path = std::env::var_os("SM_FRAME_DIAGNOSTIC_CASE").unwrap();
    let case: Case = toml::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let sources = Sources {
        rom: std::env::var_os("SM_ROM").unwrap().into(),
        ..Sources::default()
    };
    let mut admitted = admit(&case, &sources).unwrap();
    let InputReader::Frame(inputs) = &admitted.inputs else {
        panic!("frame inputs")
    };
    let budget = inputs.budget;
    let mut found = false;
    for _ in 0..100_000 {
        if matches!(admitted.production.state(), game::state::owned::OwnedState::Frame(state)
            if matches!(state.caller, frame::Caller::PlayerContinuation {
                ground: frame::movement_ground_contract::Stage::Charging(_), charging: true, .. }))
        {
            found = true;
            break;
        }
        admitted
            .production
            .advance(game::production::AdvanceInput::Frame { budget })
            .unwrap();
    }
    assert!(found, "actual interior ground reached");
    let saved = admitted.production.save().unwrap();
    let mut restored =
        Production::restore(&saved, admitted.content.clone(), admitted.rules.clone()).unwrap();
    assert!(restored.state().eq(admitted.production.state()));
    let left = admitted
        .production
        .advance(game::production::AdvanceInput::Frame { budget })
        .unwrap();
    let right = restored
        .advance(game::production::AdvanceInput::Frame { budget })
        .unwrap();
    assert!(left.eq(&right));
    assert!(restored.state().eq(admitted.production.state()));
    let game::state::owned::OwnedState::Frame(actual) = admitted.production.state() else {
        panic!("frame state")
    };
    let ProductionContent::Frame(content) = &admitted.content else {
        panic!("frame content")
    };
    let ProductionRules::Frame(rules) = &admitted.rules else {
        panic!("frame rules")
    };
    for mutation in [3, 0, 1, 2] {
        let mut changed = (**actual).clone();
        let mut changed_rules = rules.as_ref().clone();
        match mutation {
            0 => changed.playing_figure.0.pose.facing = types::Facing::Left,
            1 => {
                let frame::movement::Movement::Work(work) = &mut changed.movement else {
                    panic!("movement work")
                };
                work.model.0.state.run.speed.0 = work.model.0.state.run.speed.0.wrapping_add(1);
            }
            2 => {
                let frame::Caller::PlayerContinuation { controls, .. } = &mut changed.caller else {
                    panic!("player continuation")
                };
                controls.work = game::production::timing::WorkId(15);
            }
            _ => {
                changed_rules.movement.dash_gain.0 =
                    changed_rules.movement.dash_gain.0.wrapping_add(1);
            }
        }
        let before = changed.clone();
        assert!(
            changed.advance(content, &changed_rules, budget).is_err(),
            "ground current mutation {mutation}"
        );
        assert!(changed.eq(&before));
        assert!(changed.timing.hardware.now.eq(&before.timing.hardware.now));
        assert!(changed.timing.work.eq(&before.timing.work));
        assert!(changed.publications.eq(&before.publications));
    }
}

#[test]
#[ignore = "requires reviewed selected movement source admission and player ROM"]
fn actual_down_holds_query_and_current_owners_atomically() {
    let path = std::env::var_os("SM_FRAME_DIAGNOSTIC_CASE").unwrap();
    let case: Case = toml::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let sources = Sources {
        rom: std::env::var_os("SM_ROM").unwrap().into(),
        ..Sources::default()
    };
    let mut admitted = admit(&case, &sources).unwrap();
    let InputReader::Frame(inputs) = &admitted.inputs else {
        panic!("frame inputs")
    };
    let budget = inputs.budget;
    let mut found = false;
    for _ in 0..100_000 {
        if matches!(admitted.production.state(), game::state::owned::OwnedState::Frame(state)
            if matches!(state.caller, frame::Caller::PlayerContinuation {
                down: frame::movement_down_contract::Stage::Charging { .. }, charging: true, .. }))
        {
            found = true;
            break;
        }
        admitted
            .production
            .advance(game::production::AdvanceInput::Frame { budget })
            .unwrap();
    }
    assert!(found, "actual interior down reached");
    let saved = admitted.production.save().unwrap();
    let mut restored =
        Production::restore(&saved, admitted.content.clone(), admitted.rules.clone()).unwrap();
    assert!(restored.state().eq(admitted.production.state()));
    let left = admitted
        .production
        .advance(game::production::AdvanceInput::Frame { budget })
        .unwrap();
    let right = restored
        .advance(game::production::AdvanceInput::Frame { budget })
        .unwrap();
    assert!(left.eq(&right));
    assert!(restored.state().eq(admitted.production.state()));
    let game::state::owned::OwnedState::Frame(actual) = admitted.production.state() else {
        panic!("frame state")
    };
    let ProductionContent::Frame(content) = &admitted.content else {
        panic!("frame content")
    };
    let ProductionRules::Frame(rules) = &admitted.rules else {
        panic!("frame rules")
    };
    for mutation in [3, 0, 1, 2] {
        let mut changed = (**actual).clone();
        let mut changed_rules = rules.as_ref().clone();
        match mutation {
            0 => changed.playing_figure.0.pose.facing = types::Facing::Left,
            1 => {
                let frame::movement::Movement::Work(work) = &mut changed.movement else {
                    panic!("movement work")
                };
                work.model.0.state.run.speed.0 = work.model.0.state.run.speed.0.wrapping_add(1);
            }
            2 => {
                let frame::Caller::PlayerContinuation { controls, .. } = &mut changed.caller else {
                    panic!("player continuation")
                };
                controls.work = game::production::timing::WorkId(15);
            }
            _ => {
                changed_rules.movement.dash_gain.0 =
                    changed_rules.movement.dash_gain.0.wrapping_add(1);
            }
        }
        let before = changed.clone();
        assert!(
            changed.advance(content, &changed_rules, budget).is_err(),
            "down current mutation {mutation}"
        );
        assert!(changed.eq(&before));
        assert!(changed.timing.hardware.now.eq(&before.timing.hardware.now));
        assert!(changed.timing.work.eq(&before.timing.work));
        assert!(changed.publications.eq(&before.publications));
    }
}
