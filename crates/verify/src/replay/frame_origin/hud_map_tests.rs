use super::runtime_helpers::{admitted, advance_until, restore_next};
use super::*;
use data::assets::world::rooms::{CERES_ELEVATOR_ROOM, CERES_RIDLEY_ROOM};
use frame::{hud_camera_caller::Refresh, hud_map_caller::Stage};
use game::state::owned::OwnedState;

fn phase(state: &frame::State, wanted: u8) -> bool {
    if !matches!(
        state.caller,
        frame::Caller::PlayerContinuation {
            post_down: frame::post_down_caller::Stage::Complete(_),
            ..
        }
    ) {
        return false;
    }
    matches!(
        (&state.hud_map, wanted),
        (Stage::Pending, 0)
            | (
                Stage::Active {
                    phase: Refresh::Charging,
                    ..
                },
                1,
            )
            | (
                Stage::Active {
                    phase: Refresh::Ready,
                    ..
                },
                2,
            )
            | (
                Stage::Active {
                    phase: Refresh::Returning,
                    ..
                },
                3,
            )
            | (
                Stage::Active {
                    phase: Refresh::Complete,
                    ..
                },
                4,
            )
    )
}

fn turn_off_map(state: &mut frame::State) {
    let frame::Caller::PlayerContinuation { controls, .. } = &state.caller else {
        panic!("actual controls")
    };
    let mut quantities = state.hud.state().clone();
    quantities.mini_map = interface_hud::MiniMapPower::Off;
    let mut hud = frame::hud::Hud::new(quantities);
    frame::hud_stage::choose_controls(&mut hud, controls.answer.0.inputs).unwrap();
    frame::hud_stage::complete_controls(&mut hud).unwrap();
    assert!(hud.controls().eq(&state.hud.controls()));
    state.hud = hud;
}

#[test]
#[ignore = "requires accepted Display-v2 origin, source profiles and player ROM"]
fn actual_hud_map_current_owners_ready_return_and_restore() {
    let mut admitted = admitted();
    let InputReader::Frame(inputs) = &admitted.inputs else {
        panic!("frame inputs")
    };
    let budget = inputs.budget;
    for current_phase in 0..5 {
        advance_until(&mut admitted, budget, |state| phase(state, current_phase));
        let OwnedState::Frame(actual) = admitted.production.state() else {
            panic!("frame state")
        };
        assert_eq!(actual.hud.publication().is_some(), current_phase >= 2);
        let ProductionContent::Frame(content) = &admitted.content else {
            panic!("frame content")
        };
        let ProductionRules::Frame(rules) = &admitted.rules else {
            panic!("frame rules")
        };
        for mutation in 0..6 {
            let mut changed = (**actual).clone();
            damage_hud_map(&mut changed, content, mutation);
            let before = changed.clone();
            assert!(
                matches!(
                    changed.advance(content, rules, budget),
                    Err(game::production::ProductionFault::Unsupported(
                        game::production::Unsupported::Frame(
                            frame::Dependency::HudDisplayPreparation
                        )
                    ))
                ),
                "HUDMap typed rejection phase {current_phase} mutation {mutation}"
            );
            assert!(changed == before, "HUDMap rejection atomicity");
        }
        if current_phase >= 2 {
            let current = content.hud_refresh.as_ref().unwrap();
            let expected = frame::hud::tilemap(actual.hud.state()).unwrap();
            let regions = actual.content.regions();
            let matching: Vec<_> = regions
                .iter()
                .filter(|region| region.start == current.address)
                .collect();
            assert_eq!(matching.len(), 1);
            assert!(matching.first().unwrap().bytes.eq(&expected));
            assert_eq!(actual.hud.events().len(), 0);
        }
        let expected_terminal = if actual.scene_countdown.is_some() {
            frame::Dependency::CountdownPreparation
        } else {
            frame::Dependency::CountdownSceneScheduling
        };
        let next = restore_next(&mut admitted, budget);
        if current_phase == 4 {
            assert!(
                matches!(next, Err(game::production::ProductionFault::Unsupported(game::production::Unsupported::Frame(actual))) if actual == expected_terminal)
            );
        } else {
            next.unwrap();
        }
    }
}

fn damage_hud_map(changed: &mut frame::State, content: &frame::Content, mutation: u8) {
    match mutation {
        0 => {
            let current = &mut changed.hud_display_current.as_mut().unwrap().0;
            current.status.energy.now = current.status.energy.now.wrapping_add(1);
        }
        1 => {
            changed
                .hud_display_current
                .as_mut()
                .unwrap()
                .0
                .premises
                .events
                .turned_on = true;
        }
        2 => turn_off_map(changed),
        3 => changed.hud_display_current = None,
        4 => {
            let frame::movement::Movement::Work(work) = &mut changed.movement else {
                panic!("actual movement")
            };
            work.model.0.state.position.x =
                core::num::Wrapping(work.model.0.state.position.x.0.wrapping_add(1 << 16));
        }
        5 => {
            let room = content.movement_room.as_ref().unwrap();
            let current = changed
                .inside_world
                .as_ref()
                .unwrap()
                .0
                .room_identity()
                .unwrap();

            let other = if current == CERES_RIDLEY_ROOM {
                CERES_ELEVATOR_ROOM
            } else {
                CERES_RIDLEY_ROOM
            };
            changed.inside_world = Some(Model(
                frame::inside_world::InsideWorld::for_room(
                    other,
                    room.width,
                    room.cells.clone(),
                    room.boundary,
                )
                .unwrap(),
            ));
        }
        _ => panic!("mutation"),
    }
}
