use super::*;
use core::num::Wrapping;
use types::{Pair, Set};

fn owner() -> Movement {
    Movement::new(samus_movement::Samus {
        position: Pair {
            x: Wrapping(100 * 65536),
            y: Wrapping(100 * 65536),
        },
        pose: types::Pose {
            body: types::Body::Upright,
            action: types::Action::Running,
            facing: types::Facing::Right,
            aim: types::Aim::Forward,
            gun: types::Gun::Stowed,
            legs: types::Legs::Together,
        },
        run: samus_movement::Run {
            speed: Wrapping(0),
            way: types::Side::Right,
            carry: samus_movement::Carry::FollowsPad,
            dash: Wrapping(0),
            dashing: samus_movement::Dashing::Off,
        },
        vertical: samus_movement::Vertical {
            speed: Wrapping(0),
            phase: samus_movement::Phase::Ground,
            bounces: samus_movement::Bounces::None,
        },
        spin: samus_movement::Spin::Off,
        seat: samus_movement::Seat::Unseated,
        jump_held: samus_movement::JumpHeld::NONE,
    })
}

fn content() -> MovementContent {
    MovementContent::new(
        data::assets::samus::map::MAP
            .iter()
            .map(|entry| crate::production::PoseContent {
                pose: entry.key,
                radius: Pair {
                    x: Wrapping(5),
                    y: Wrapping(21),
                },
                cannon_heading: None,
            })
            .collect(),
    )
    .unwrap()
}

fn frame() -> samus_movement::Frame<'static> {
    samus_movement::Frame {
        sprite_before_movement: types::Sprite::Plain,
        cinematic_active: false,
        pad: types::Buttons::default(),
        pressed: types::Buttons::default(),
        items: Set::default(),
        spin_turned_far: false,
        elevator_moving: false,
        parity: types::Parity::Even,
        animation_ends: None,
        body: &|_| Pair {
            x: Wrapping(5),
            y: Wrapping(21),
        },
        cannon_heading: &|_| None,
    }
}

fn world() -> InsideWorld {
    InsideWorld::new(16, vec![Block::Air; 256], Boundary::Solid).unwrap()
}

#[test]
fn inside_resolves_actual_room_stand_ins_and_retains_answer_once() {
    let mut cells = vec![Block::Air; 256];
    cells[7 * 16 + 6] = Block::StandsFor(Some(Cell { x: 2, y: 3 }));
    cells[3 * 16 + 2] = Block::Answered;
    let world = InsideWorld::new(16, cells, Boundary::Solid).unwrap();
    let content = content();
    let rules = samus_movement::Rules::default();
    let frame = frame();
    let mut movement = owner();
    movement.begin().unwrap();
    let mut stage = movement_stage::Stage::Idle;
    let receipt = world
        .choose(&mut stage, &mut movement, &content, &rules, &frame)
        .unwrap()
        .clone();
    let Some(MovementDecision::InsideAnswer(cells)) = receipt.decisions.0.last() else {
        panic!();
    };
    assert_eq!(cells.first(), Some(&Cell { x: 2, y: 3 }));
    stage.bind(WorkId(73)).unwrap();
    let bytes = canonical::to_bytes(&movement);
    movement = canonical::from_bytes(&bytes).unwrap();
    world
        .complete(
            &mut stage,
            WorkId(73),
            &mut movement,
            &content,
            &rules,
            &frame,
        )
        .unwrap();
    assert_eq!(movement.outputs().unwrap().inside, cells.clone());
    let before = movement.clone();
    let stage_before = stage.clone();
    assert_eq!(
        world.choose(&mut stage, &mut movement, &content, &rules, &frame),
        Err(Fault::Opportunity)
    );
    assert_eq!(movement, before);
    assert_eq!(stage, stage_before);
    assert_eq!(
        world.complete(
            &mut stage,
            WorkId(73),
            &mut movement,
            &content,
            &rules,
            &frame
        ),
        Err(Fault::Opportunity)
    );
    assert_eq!(movement, before);
    assert_eq!(stage, stage_before);
}

#[test]
fn support_and_absent_work_are_rejected_before_touching_world_or_owners() {
    let world = world();
    let content = content();
    let rules = samus_movement::Rules::default();
    let frame = frame();
    let mut movement = owner();
    movement.begin().unwrap();
    let offered = movement
        .offer(
            &content,
            &rules,
            &frame,
            &Blocks {
                room: &world,
                fault: FaultCell::new(None),
            },
        )
        .unwrap()
        .unwrap();
    let MovementDecision::InsideRequest(bounds) = offered else {
        panic!();
    };
    let Movement::Work(work) = &mut movement else {
        panic!();
    };
    // A support request requires a later capability even with an empty consumed history.
    work.model.0.offered = Some(MovementDecision::SupportRequest {
        bounds,
        surface: types::SupportSurface::Floor,
    });
    let before = movement.clone();
    let mut stage = movement_stage::Stage::Idle;
    assert_eq!(
        world.choose(&mut stage, &mut movement, &content, &rules, &frame),
        Err(Fault::Opportunity)
    );
    assert_eq!(movement, before);
    assert_eq!(stage, movement_stage::Stage::Idle);
    let mut absent = owner();
    let before = absent.clone();
    assert_eq!(
        world.choose(&mut stage, &mut absent, &content, &rules, &frame),
        Err(Fault::Opportunity)
    );
    assert_eq!(absent, before);
}

#[test]
fn room_shape_and_stand_in_cycles_are_rejected() {
    assert_eq!(
        InsideWorld::new(0, vec![Block::Air], Boundary::Air),
        Err(WorldFault::Shape)
    );
    assert_eq!(
        InsideWorld::new(2, vec![Block::Air], Boundary::Air),
        Err(WorldFault::Shape)
    );
    let cell = Cell { x: 0, y: 0 };
    assert_eq!(
        InsideWorld::new(1, vec![Block::StandsFor(Some(cell))], Boundary::Air),
        Err(WorldFault::StandInLoop(cell))
    );
    let outside = Cell { x: 1, y: 0 };
    assert_eq!(
        InsideWorld::new(1, vec![Block::StandsFor(Some(outside))], Boundary::Air),
        Err(WorldFault::StandInOutside(outside))
    );
}

#[test]
fn unavailable_actor_and_reaction_queries_discard_both_candidate_owners() {
    let world = world();
    for expected in [Fault::Actors, Fault::Reaction] {
        let mut movement = owner();
        movement.begin().unwrap();
        let before = movement.clone();
        let mut stage = movement_stage::Stage::Idle;
        let result = world.transaction(&mut stage, &mut movement, |stage, movement, blocks| {
            match expected {
                Fault::Actors => {
                    let _ = blocks.solid_enemies();
                }
                Fault::Reaction => {
                    let _ = blocks.lets_through(Cell { x: 0, y: 0 }, types::Way::Look);
                }
                Fault::Opportunity | Fault::Stage(_) => panic!(),
            }
            let Movement::Work(work) = movement else {
                panic!();
            };
            work.model.0.state.position.x = Wrapping(0);
            *stage = movement_stage::Stage::Offered(movement_stage::Receipt {
                group: movement_stage::Group::Inside,
                decisions: canonical::model::Model(Vec::new()),
            });
            Ok(())
        });
        assert_eq!(result, Err(expected));
        assert_eq!(movement, before);
        assert_eq!(stage, movement_stage::Stage::Idle);
    }
}

#[test]
fn trace_actual_room_block_shape_and_restore_complete_content() {
    let shape = canonical::model::shape::<InsideWorld>().unwrap();
    assert_eq!(shape, <InsideWorldPin as canonical::model::Pinned>::SHAPE);
    assert_eq!(canonical::model::pin_holds::<InsideWorldPin>(), Ok(()));
    let current = canonical::model::Model::<InsideWorldPin>(world());
    let restored = canonical::from_bytes::<canonical::model::Model<InsideWorldPin>>(
        &canonical::to_bytes(&current),
    )
    .unwrap();
    assert_eq!(restored, current);
}
