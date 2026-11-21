use super::super::{
    AdvanceInput, ProductionContent, ProductionFault, Unsupported,
    tests::{empty_world, movement, stimuli},
};
use crate::state::owned::OwnedState;
use std::sync::Arc;

#[test]
fn missing_selected_body_refuses_after_an_actual_pose_transition() {
    let (mut production, _, _) = movement();
    let mut control = production.clone();
    let world = empty_world();
    let input = || AdvanceInput::Movement {
        inputs: super::super::MovementInputs {
            pad: types::button::UP,
            ..stimuli()
        },
        world: &world,
    };
    control.advance(input()).unwrap();
    let OwnedState::Movement(before) = production.state() else {
        panic!("movement scope");
    };
    let OwnedState::Movement(after) = control.state() else {
        panic!("movement scope");
    };
    let initial = before.0.pose;
    let selected = after.0.pose;
    assert_eq!(
        selected,
        types::Pose {
            aim: types::Aim::Up,
            ..initial
        }
    );

    // Exercise a later lookup whose absence would normally be excluded by content admission.
    let ProductionContent::Movement(content) = &mut Arc::make_mut(&mut production.context).content
    else {
        panic!("movement content");
    };
    content.poses.retain(|entry| entry.pose != selected);
    assert!(content.pose(initial).is_some());
    assert!(content.pose(selected).is_none());

    let before = production.clone();
    let checkpoint = production.save().unwrap();
    assert_eq!(
        production.advance(input()),
        Err(ProductionFault::Unsupported(Unsupported::PoseContent(
            selected
        ))),
    );
    assert_eq!(production, before);
    assert_eq!(production.save().unwrap(), checkpoint);
}
