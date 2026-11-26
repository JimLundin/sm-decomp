use super::{GetawayInputs, GetawayPass, TurnContent};
use crate::production::played::{Content, Fault, Rules, tests};
use crate::production::world::{Boundary, MovementWorld};
use core::num::Wrapping;
use samus_movement::{Getaway, Outcome};
use types::{Action, Block, Direction, Parts, Side, ZERO, button, fixed};

const NO_PASS: GetawayInputs = GetawayInputs {
    view_left: Wrapping(0),
    pass: None,
    slowdown: samus_movement::Slowdown::NONE,
    carry: ZERO,
};

fn turn<'a>(content: &'a Content, rules: &'a Rules) -> TurnContent<'a> {
    TurnContent {
        movement: &content.movement,
        appearance: &content.appearance,
        movement_rules: &rules.movement,
        appearance_rules: &rules.appearance,
    }
}

#[test]
fn travel_interval_shape_remains_pinned() {
    assert_eq!(canonical::model::pin_holds::<super::TravelPin>(), Ok(()));
}

#[test]
fn the_pass_keeps_ordinary_travel_and_the_next_turn_reads_the_completed_position() {
    let mut her = tests::state(Vec::new()).samus;
    her.movement.0.position.x = fixed(Wrapping(127), 0x8000);
    her.movement.0.run.speed = types::px(Wrapping(3));
    let inputs = tests::inputs(button::RIGHT);
    let content = tests::content();
    let rules = tests::rules();
    let turn = turn(&content, &rules);
    let mut ordinary = her.clone();
    let expected = ordinary.step_getaway(&inputs, NO_PASS, &[], turn).unwrap();
    let arm = GetawayInputs {
        pass: Some(GetawayPass),
        ..NO_PASS
    };
    let armed = her.step_getaway(&inputs, arm, &[], turn).unwrap();
    assert_eq!(armed, expected);
    assert_eq!(her.travelled.0.travels, armed.travelled);
    assert!(armed.travelled[Direction::Right].0 > ZERO);
    assert!(her.movement.0.position.x.pixel().0 >= 128);
    assert_eq!(her.movement.0.getaway, Getaway::Pending);
    let state = her.movement.0;
    let start = her
        .step_getaway(&tests::inputs(button::LEFT), NO_PASS, &[], turn)
        .unwrap();
    assert_eq!(her.movement.0.position.x, state.position.x);
    assert_eq!(her.movement.0.getaway, Getaway::Travelling(Side::Right));
    assert_eq!(start.travelled, types::DirectionalTravels::default());
    assert_eq!(her.travelled.0.travels, start.travelled);
    assert_eq!(her.figure.0.pose.action, Action::Recoiling);

    let adjusted_view = GetawayInputs {
        view_left: Wrapping(8),
        ..NO_PASS
    };
    GetawayPass.arm(&mut ordinary.movement.0);
    ordinary
        .step_getaway(&inputs, adjusted_view, &[], turn)
        .unwrap();
    assert_eq!(ordinary.movement.0.getaway, Getaway::Travelling(Side::Left));
}

fn floor_and_wall() -> MovementWorld {
    let width = 16;
    let cells = (0..128)
        .map(|cell| {
            if cell / width >= 6 || cell % width == 4 {
                Block::Solid
            } else {
                Block::Air
            }
        })
        .collect();
    MovementWorld::new(width, cells, vec![false; 128], Vec::new(), Boundary::Air).unwrap()
}

#[test]
fn leftward_travel_survives_the_floor_and_restores_through_the_side_stop() {
    let mut her = tests::state(Vec::new()).samus;
    her.movement.0.position.y = fixed(Wrapping(74), 0x8000);
    let inputs = crate::production::connected::inputs::Inputs {
        geometry: floor_and_wall(),
        ..tests::inputs(button::RIGHT)
    };
    let previous = her.movement.0.pose;
    let mut content = tests::content();
    content.movement = crate::production::MovementContent::new(
        data::assets::samus::map::MAP
            .iter()
            .map(|entry| crate::production::PoseContent {
                pose: entry.key,
                radius: types::Radius {
                    x: Wrapping(5),
                    y: Wrapping(if entry.key == previous { 19 } else { 21 }),
                },
                cannon_heading: None,
            })
            .collect(),
    )
    .unwrap();
    let rules = tests::rules();
    let advance = |her: &mut super::State| -> Outcome {
        her.step_getaway(&inputs, NO_PASS, &[], turn(&content, &rules))
            .unwrap()
    };
    let at_arm = crate::production::connected::inputs::Inputs {
        held: types::Buttons::default(),
        pressed: types::Buttons::default(),
        ..inputs.clone()
    };
    let cue = GetawayInputs {
        pass: Some(GetawayPass),
        ..NO_PASS
    };
    her.step_getaway(&at_arm, cue, &[], turn(&content, &rules))
        .unwrap();
    assert_eq!(her.movement.0.getaway, Getaway::Pending);

    let start = advance(&mut her);
    assert_eq!(her.movement.0.position.y, fixed(Wrapping(72), 0x8000));
    let backward_leap = data::assets::samus::map::MAP
        .iter()
        .find(|entry| entry.key.action == Action::Leaping)
        .unwrap()
        .key;
    let mut blast = Some(());
    let mut deferred = Some((previous, ()));
    start.deferred_motion.discard_blast_lift(&mut blast);
    start.deferred_motion.discard_pose(&mut deferred);
    assert!(blast.is_none() && deferred.is_none());

    let first = advance(&mut her);
    assert_eq!(her.movement.0.getaway, Getaway::Travelling(Side::Left));
    assert_eq!(her.movement.0.position.y.pixel(), Wrapping(75));
    assert!(first.travelled[Direction::Down].0 < types::px(Wrapping(5)));
    assert!(first.floor_response.is_none());
    let mut blast = Some(());
    let mut leap = Some((backward_leap, ()));
    let mut other = Some((previous, ()));
    first.deferred_motion.discard_blast_lift(&mut blast);
    first.deferred_motion.discard_pose(&mut leap);
    first.deferred_motion.discard_pose(&mut other);
    assert!(blast.is_some() && leap.is_none() && other.is_some());
    let floor_y = her.movement.0.position.y;
    let saved = canonical::to_bytes(&her);
    let mut resumed = canonical::from_bytes(&saved).unwrap();

    let mut adjusted = her.clone();
    let mut adjusted_rules = rules.clone();
    adjusted_rules.movement.push.step = types::px(Wrapping(3));
    adjusted
        .step_getaway(&inputs, NO_PASS, &[], turn(&content, &adjusted_rules))
        .unwrap();

    let next = advance(&mut her);
    assert_eq!(advance(&mut resumed), next);
    assert_eq!(resumed, her);
    assert_eq!(her.movement.0.getaway, Getaway::Travelling(Side::Left));
    assert_eq!(her.movement.0.position.y, floor_y);
    assert!(next.travelled[Direction::Left].0.0.cast_signed() < 0);
    assert_eq!(next.travelled[Direction::Down].0, ZERO);
    assert!(adjusted.movement.0.position.x < her.movement.0.position.x);

    for _ in 0..5 {
        let direct = advance(&mut her);
        assert_eq!(advance(&mut resumed), direct);
        assert_eq!(resumed, her);
        if her.movement.0.getaway == Getaway::Idle {
            assert_eq!(her.movement.0.position.x.pixel(), Wrapping(85));
            assert_eq!(her.movement.0.position.y, floor_y);
            assert_eq!(her.movement.0.run.speed, ZERO);
            assert_eq!(her.movement.0.vertical.speed, ZERO);
            assert_eq!(direct.travelled[Direction::Down].0, ZERO);
            assert!(direct.travelled[Direction::Left].0.0.cast_signed() < 0);
            assert_eq!(her.travelled.0.travels, direct.travelled);
            assert_ne!(advance(&mut her).deferred_motion, direct.deferred_motion);
            return;
        }
    }
    panic!("the solid wall must release the shove");
}

#[test]
fn a_missing_recoil_clip_leaves_the_pending_turn_and_its_appearance_intact() {
    let mut her = tests::state(Vec::new()).samus;
    GetawayPass.arm(&mut her.movement.0);
    let saved = her.clone();
    let inputs = tests::inputs(button::LEFT);
    let content = tests::content();
    let rules = tests::rules();
    let figure = her.figure.0;
    let appearance = crate::production::AppearanceContent::new(
        vec![crate::production::FigureClip {
            figure,
            clip: content.appearance.get(figure).unwrap().clone(),
        }],
        samus_appearance::Paces {
            plain: samus_appearance::Pace(Vec::new()),
            boosted: enum_map::EnumMap::from_fn(|_| samus_appearance::Pace(Vec::new())),
        },
        crate::production::pulse_colours(&data::assets::samus::palettes::SamusPalettes::default())
            .unwrap(),
    )
    .unwrap();
    let fault = her.step_getaway(
        &inputs,
        NO_PASS,
        &[],
        TurnContent {
            appearance: &appearance,
            ..turn(&content, &rules)
        },
    );
    assert!(matches!(fault, Err(Fault::Clip(figure)) if figure.pose.action == Action::Recoiling));
    assert_eq!(her, saved);
    assert!(
        her.step_getaway(&inputs, NO_PASS, &[], turn(&content, &rules))
            .is_ok()
    );
    assert_eq!(her.movement.0.getaway, Getaway::Travelling(Side::Left));
}

#[test]
fn slowdown_precedes_signed_room_carry_and_the_move_can_reverse() {
    let mut her = tests::state(Vec::new()).samus;
    her.movement.0.run.dash = fixed(Wrapping(0), 0x8000);
    GetawayPass.arm(&mut her.movement.0);
    let inputs = tests::inputs(types::Buttons::default());
    let content = tests::content();
    let rules = tests::rules();
    let turn = turn(&content, &rules);
    her.step_getaway(&inputs, NO_PASS, &[], turn).unwrap();
    let x_before = her.movement.0.position.x;
    let mut at_block = her.clone();
    at_block.movement.0.position.x = fixed(Wrapping(107), 0x8000);
    let with_room_carry = GetawayInputs {
        slowdown: samus_movement::Slowdown::new(1).unwrap(),
        carry: fixed(Wrapping(2), 0x4000),
        ..NO_PASS
    };
    let outcome = her
        .step_getaway(&inputs, with_room_carry, &[], turn)
        .unwrap();

    // The recoil gains 1.5, half a pixel of dash makes 2, and the slowdown
    // leaves 1 pixel to the left. The room moves her 2.25 pixels to the right.
    let right = fixed(Wrapping(1), 0x4000);
    assert_eq!(her.movement.0.position.x, x_before + right);
    assert_eq!(her.movement.0.getaway, Getaway::Travelling(Side::Left));
    assert_eq!(outcome.travelled[Direction::Right].0, right);
    assert_eq!(outcome.travelled[Direction::Left].0, ZERO);

    let width = 16;
    let cells = (0..128)
        .map(|cell| {
            if cell % width == 7 {
                Block::Solid
            } else {
                Block::Air
            }
        })
        .collect();
    let blocked = crate::production::connected::inputs::Inputs {
        geometry: MovementWorld::new(width, cells, vec![false; 128], Vec::new(), Boundary::Air)
            .unwrap(),
        ..inputs
    };
    let output = at_block
        .step_getaway(&blocked, with_room_carry, &[], turn)
        .unwrap();
    assert_eq!(
        at_block.movement.0.position.x,
        fixed(Wrapping(107), u16::MAX)
    );
    assert_eq!(at_block.movement.0.getaway, Getaway::Travelling(Side::Left));
    assert_eq!(output.travelled[Direction::Right].0, ZERO);
    assert_eq!(output.travelled[Direction::Down].0, types::px(Wrapping(5)));
}
