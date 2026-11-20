use super::runtime_helpers::{admitted, advance_until, restore_next};
use super::*;
use data::timing::MasterTicks;
use game::state::owned::OwnedState;
use samus_appearance::{CannonDelay, CannonObservation};

fn at_phase(state: &frame::State, phase: u8) -> bool {
    match phase {
        0 => matches!(
            state.caller,
            frame::Caller::Player {
                stage: frame::PlayerEntryStage::HudWeapons,
                charging: false,
                ..
            }
        ),
        1 => matches!(
            state.caller,
            frame::Caller::Player {
                stage: frame::PlayerEntryStage::HudWeapons,
                charging: true,
                ..
            }
        ),
        2 => matches!(state.caller, frame::Caller::PlayerContinuation { .. }),
        _ => false,
    }
}

fn advance_to(admitted: &mut AdmittedCase, phase: u8, budget: MasterTicks) {
    advance_until(admitted, budget, |state| at_phase(state, phase));
}

#[test]
#[ignore = "requires actual admitted frame origin and player ROM"]
fn actual_cannon_joins_controls_at_boundary_hold_and_completion_with_fresh_restore() {
    let mut admitted = admitted();
    let InputReader::Frame(inputs) = &admitted.inputs else {
        panic!("frame inputs")
    };
    let budget = inputs.budget;
    let OwnedState::Frame(initial) = admitted.production.state() else {
        panic!("frame state")
    };
    let initial_delay = initial.appearance.cannon.as_ref().unwrap().0;
    let initial_animation = initial.appearance.model.0.state;
    for phase in 0..3 {
        advance_to(&mut admitted, phase, budget);
        let OwnedState::Frame(actual) = admitted.production.state() else {
            panic!("frame state")
        };
        assert_observation_eq!(actual.appearance.model.0.state, initial_animation);
        let expected_delay = if phase == 2 {
            let mut expected = initial_delay;
            let CannonObservation::Completed(answer) = actual.appearance.cannon_observation.0
            else {
                panic!("actual controls did not complete cannon observation")
            };
            expected.controls_completed(answer);
            expected
        } else {
            initial_delay
        };
        assert_observation_eq!(actual.appearance.cannon.as_ref().unwrap().0, expected_delay);
        let ProductionContent::Frame(content) = &admitted.content else {
            panic!("frame content")
        };
        let ProductionRules::Frame(rules) = &admitted.rules else {
            panic!("frame rules")
        };
        for mutation in 0..3 {
            let mut changed = (**actual).clone();
            match mutation {
                0 => changed.appearance.cannon = None,
                1 => {
                    let different = if expected_delay.remaining() == 0 {
                        2
                    } else {
                        0
                    };
                    changed.appearance.cannon = Some(Model(CannonDelay::new(different).unwrap()));
                }
                2 => {
                    changed.appearance.cannon_observation = Model(match phase {
                        0 => CannonObservation::Completed(samus_appearance::ItemControlAnswer {
                            before: samus_appearance::SelectedItem::None,
                            after: samus_appearance::SelectedItem::None,
                            eligibility: samus_appearance::ControlEligibility::Eligible,
                        }),
                        _ => CannonObservation::Unmade,
                    });
                }
                _ => panic!("mutation"),
            }
            let before = changed.clone();
            assert!(
                changed.advance(content, rules, budget).is_err(),
                "cannon phase {phase} mutation {mutation}"
            );
            assert!(changed == before, "cannon rejection atomicity");
        }
        restore_next(&mut admitted, budget).unwrap();
    }
}

#[test]
#[ignore = "requires the retained frame and checked player ROM"]
fn unready_cannon_rejects_the_ready_drawing_profile() {
    use game::production::{ProductionFault, Unsupported};

    let (_, mut admitted) = super::runtime_helpers::retained_return();
    let InputReader::Frame(inputs) = &admitted.inputs else {
        panic!("frame inputs")
    };
    let budget = inputs.budget;
    advance_to(&mut admitted, 0, budget);
    let OwnedState::Frame(initial) = admitted.production.state() else {
        panic!("frame state")
    };
    let ProductionContent::Frame(content) = &admitted.content else {
        panic!("frame content")
    };
    let ProductionRules::Frame(rules) = &admitted.rules else {
        panic!("frame rules")
    };
    let mut state = initial.as_ref().clone();
    state.appearance.cannon = Some(Model(CannonDelay::new(2).unwrap()));
    state.appearance.cannon_before = state.appearance.cannon;
    for _ in 0..200_000 {
        let before_drawing = state.s6_camera.following().is_some().then(|| state.clone());
        match state.advance(content, rules, budget) {
            Ok(_) => assert!(state.s6_drawing.builder().is_none()),
            Err(fault) => {
                assert_observation_eq!(
                    fault,
                    ProductionFault::Unsupported(Unsupported::Frame(
                        frame::Dependency::DrawingCurrentOwners
                    ))
                );
                assert_eq!(state.appearance.cannon.as_ref().unwrap().0.remaining(), 1);
                assert_observation_eq!(Some(state), before_drawing);
                return;
            }
        }
    }
    panic!("the unsupported drawing profile was never reached");
}
