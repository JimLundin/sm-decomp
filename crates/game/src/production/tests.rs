#[test]
fn movement_state_shape_remains_pinned() {
    assert_eq!(
        canonical::model::pin_holds::<crate::state::owned::MovementPin>(),
        Ok(())
    );
}

#[test]
fn audio_state_shape_remains_pinned() {
    assert_eq!(
        canonical::model::pin_holds::<crate::state::owned::AudioPin>(),
        Ok(())
    );
}
use super::*;
use crate::state::owned::OwnedState;
use core::num::Wrapping;
use types::{Block, Pair, Set, Sprite};

mod audio_checkpoint;
mod continuation;
mod entries;
pub(super) mod fixtures;
mod weapons;

pub(super) fn movement() -> (Production, ProductionContent, ProductionRules) {
    let content = ProductionContent::Movement(
        MovementContent::new(
            data::assets::samus::map::MAP
                .iter()
                .map(|entry| PoseContent {
                    pose: entry.key,
                    radius: Pair {
                        x: Wrapping(5),
                        y: Wrapping(21),
                    },
                    cannon_heading: None,
                })
                .collect(),
        )
        .unwrap(),
    );
    let rules = ProductionRules::Movement(samus_movement::Rules::default());
    let production = Production::admit(
        ProductionSeed::Movement(fixtures::samus()),
        content.clone(),
        rules.clone(),
    )
    .unwrap();
    (production, content, rules)
}

pub(super) fn stimuli() -> MovementInputs {
    MovementInputs {
        sprite_before_movement: Sprite::Plain,
        cinematic_active: false,
        pad: types::Buttons::default(),
        pressed: types::Buttons::default(),
        items: Set::default(),
        spin_turned_far: false,
        elevator_moving: false,
        parity: types::Parity::Even,
        animation_ends: None,
    }
}

pub(super) fn empty_world() -> world::MovementWorld {
    world::MovementWorld::new(
        32,
        vec![Block::Air; 32 * 32],
        vec![false; 32 * 32],
        vec![],
        world::Boundary::Air,
    )
    .unwrap()
}

#[test]
fn carried_movement_and_restored_continuation_preserve_accumulated_fall() {
    let (mut carried, content, rules) = movement();
    let world = empty_world();
    let first = carried
        .advance(AdvanceInput::Movement {
            inputs: stimuli(),
            world: &world,
        })
        .unwrap();
    let OwnedState::Movement(after_first) = carried.state() else {
        panic!("movement scope");
    };
    let first_y = after_first.0.position.y;
    assert_eq!(first_y, fixtures::samus().position.y);
    assert!(after_first.0.vertical.speed.0 > 0);
    let mut restored = Production::restore(&carried.save().unwrap(), content, rules).unwrap();
    assert_eq!(restored.state(), carried.state());
    let second = carried
        .advance(AdvanceInput::Movement {
            inputs: stimuli(),
            world: &world,
        })
        .unwrap();
    let continued = restored
        .advance(AdvanceInput::Movement {
            inputs: stimuli(),
            world: &world,
        })
        .unwrap();
    assert_eq!(second, continued);
    assert_eq!(carried.state(), restored.state());
    let OwnedState::Movement(after_second) = carried.state() else {
        panic!("movement scope");
    };
    assert!(after_second.0.position.y.0 > first_y.0);
    assert_ne!(first, second);
}

#[test]
fn unknown_movement_pose_refuses_without_committing() {
    let (mut production, content, _) = movement();
    let OwnedState::Movement(samus) = &mut production.state else {
        panic!("movement scope");
    };
    // Model a running owner reaching a pose with no corresponding imported body.
    samus.0.pose.facing = types::Facing::Screen;
    let missing = samus.0.pose;
    let ProductionContent::Movement(content) = content else {
        panic!("movement content");
    };
    assert!(content.radius(missing).is_none());
    let before = production.clone();
    let saved = production.save().unwrap();
    for elevator_moving in [false, true] {
        assert_eq!(
            production.advance(AdvanceInput::Movement {
                inputs: MovementInputs {
                    elevator_moving,
                    ..stimuli()
                },
                world: &empty_world(),
            }),
            Err(ProductionFault::Unsupported(Unsupported::PoseContent(
                missing
            ))),
        );
        assert_eq!(production.state(), before.state());
        assert_eq!(production.save().unwrap(), saved);
    }
}

#[test]
fn admitted_screen_pose_still_refuses_a_moving_elevator() {
    let (_, content, rules) = movement();
    let mut samus = fixtures::samus();
    samus.pose = data::assets::samus::map::MAP
        .iter()
        .find(|entry| entry.key.facing == types::Facing::Screen)
        .unwrap()
        .key;
    let mut production =
        Production::admit(ProductionSeed::Movement(samus), content, rules).unwrap();
    let before = production.clone();
    assert_eq!(
        production.advance(AdvanceInput::Movement {
            inputs: MovementInputs {
                elevator_moving: true,
                ..stimuli()
            },
            world: &empty_world(),
        }),
        Err(ProductionFault::Unsupported(Unsupported::ElevatorBody)),
    );
    assert_eq!(production, before);
}

#[test]
fn wrong_scope_and_missing_input_leave_movement_and_checkpoint_unchanged() {
    let (mut production, _, _) = movement();
    let saved = production.save().unwrap();
    assert_eq!(
        production.advance(AdvanceInput::Audio {
            offers: &[],
            cadence: fixtures::silent_cadence(),
        }),
        Err(ProductionFault::WrongScope)
    );
    assert_eq!(
        production.advance(AdvanceInput::Missing(Scope::Movement)),
        Err(ProductionFault::Unsupported(Unsupported::MissingInput(
            Scope::Movement
        )))
    );
    assert_eq!(production.save().unwrap(), saved);
}

#[test]
fn restore_refuses_other_rules_content_and_previous_state_schema() {
    let (production, content, rules) = movement();
    assert_eq!(
        Production::restore(
            &production.save().unwrap(),
            ProductionContent::Audio(Box::new(fixtures::audio_content())),
            rules.clone()
        ),
        Err(AdmissionFault::ContentChanged)
    );
    assert_eq!(
        Production::restore(
            &production.save().unwrap(),
            content.clone(),
            ProductionRules::Audio
        ),
        Err(AdmissionFault::RulesChanged)
    );
    let mut corrupted = production.save().unwrap();
    corrupted.state[0] ^= 1;
    assert!(matches!(
        Production::restore(&corrupted, content, rules),
        Err(AdmissionFault::Checkpoint(
            canonical::DecodeError::SchemaMismatch { .. }
        ))
    ));
}

#[test]
fn playback_shape() {
    assert_eq!(
        canonical::model::pin_holds::<crate::state::owned::PlaybackPin>(),
        Ok(())
    );
}
