use super::*;
use core::{cell::Cell, num::Wrapping};
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

struct World {
    queries: Cell<u32>,
    blocked: Cell<bool>,
}
impl types::World for World {
    fn block(&self, _: types::Cell) -> types::Block {
        self.queries.set(self.queries.get().checked_add(1).unwrap());
        if self.blocked.get() {
            types::Block::Solid
        } else {
            types::Block::Air
        }
    }
    fn lets_through(&self, _: types::Cell, _: types::Way) -> bool {
        false
    }
    fn solid_enemies(&self) -> &[types::Hitbox] {
        self.queries.set(self.queries.get().checked_add(1).unwrap());
        &[]
    }
}

fn restore<T: canonical::Canonical>(value: T) -> T {
    let saved = canonical::to_bytes(&value);
    drop(value);
    canonical::from_bytes(&saved).unwrap()
}

#[test]
fn whole_inside_query_retains_answer_through_physical_hold_and_restore() {
    let content = content();
    let rules = samus_movement::Rules::default();
    let frame = frame();
    let world = World {
        queries: Cell::new(0),
        blocked: Cell::new(false),
    };
    let inputs = Inputs {
        content: &content,
        rules: &rules,
        frame: &frame,
        world: &world,
    };
    let mut movement = owner();
    movement.begin().unwrap();
    let mut stage = Stage::default();
    let receipt = stage
        .choose(Group::Inside, &mut movement, &inputs)
        .unwrap()
        .clone();
    assert!(matches!(
        receipt.decisions.0.first(),
        Some(MovementDecision::InsideRequest(_))
    ));
    assert!(matches!(
        receipt.decisions.0.last(),
        Some(MovementDecision::InsideAnswer(_))
    ));
    let query_count = world.queries.get();
    assert!(query_count > 0);
    stage.bind(WorkId(12)).unwrap();
    let mut stage = restore(stage);
    let mut movement = restore(movement);
    world.blocked.set(true);
    let held = movement.clone();
    assert_eq!(
        stage.complete(WorkId(13), &mut movement, &inputs),
        Err(Fault::Completion)
    );
    assert_eq!(movement, held);
    assert_eq!(stage.receipt(), Some(&receipt));
    stage.complete(WorkId(12), &mut movement, &inputs).unwrap();
    assert_eq!(world.queries.get(), query_count);
    assert!(matches!(
        movement.offer(&content, &rules, &frame, &world).unwrap(),
        Some(MovementDecision::Local {
            rule: LocalRule::RiseEnds,
            ..
        })
    ));
    assert_eq!(
        stage.complete(WorkId(12), &mut movement, &inputs),
        Err(Fault::Order)
    );
}

#[test]
fn current_run_inputs_and_retained_choice_survive_hold_and_restore() {
    let content = content();
    let rules = samus_movement::Rules::default();
    let mut frame = frame();
    let world = World {
        queries: Cell::new(0),
        blocked: Cell::new(false),
    };
    let mut movement = owner();
    movement.begin().unwrap();
    let mut stage = Stage::default();
    let inputs = Inputs {
        content: &content,
        rules: &rules,
        frame: &frame,
        world: &world,
    };
    stage.choose(Group::Inside, &mut movement, &inputs).unwrap();
    stage.bind(WorkId(7)).unwrap();
    stage.complete(WorkId(7), &mut movement, &inputs).unwrap();
    let queries_before_run = world.queries.get();
    frame.pad = types::Buttons(types::button::RIGHT.0 | types::button::RUN.0);
    let inputs = Inputs {
        content: &content,
        rules: &rules,
        frame: &frame,
        world: &world,
    };
    let receipt = stage
        .choose(Group::Run, &mut movement, &inputs)
        .unwrap()
        .clone();
    assert!(
        matches!(receipt.decisions.0.get(2), Some(MovementDecision::Local {
        rule: LocalRule::DashBuilding { .. }, inputs, ..
    }) if inputs.pad == frame.pad)
    );
    assert!(
        matches!(receipt.decisions.0.first(), Some(MovementDecision::Local {
        rule: LocalRule::RiseEnds, before, answer: after, ..
    }) if before == after)
    );
    assert!(
        matches!(receipt.decisions.0.get(1), Some(MovementDecision::Local {
        rule: LocalRule::FrameStart, before, answer: after, ..
    }) if before == after)
    );
    assert!(movement.state().run.speed > Wrapping(0));
    let held = movement.clone();
    assert_eq!(
        stage.choose(Group::Run, &mut movement, &inputs),
        Err(Fault::Order)
    );
    assert_eq!(movement, held);
    stage.bind(WorkId(8)).unwrap();
    let mut stage = restore(stage);
    let mut movement = restore(movement);
    assert_eq!(stage.receipt(), Some(&receipt));
    assert_eq!(
        stage.complete(WorkId(9), &mut movement, &inputs),
        Err(Fault::Completion)
    );
    frame.pad = types::Buttons::default();
    let inputs = Inputs {
        content: &content,
        rules: &rules,
        frame: &frame,
        world: &world,
    };
    stage.complete(WorkId(8), &mut movement, &inputs).unwrap();
    assert_eq!(world.queries.get(), queries_before_run);
    assert_eq!(movement.state().run, held.state().run);
    assert_eq!(
        stage.complete(WorkId(8), &mut movement, &inputs),
        Err(Fault::Order)
    );
    let next = movement.offer(&content, &rules, &frame, &world).unwrap();
    assert!(matches!(next, Some(MovementDecision::MotionRequest(_))));
}

#[test]
fn restored_request_queries_once_and_retained_answer_never_requeries() {
    let content = content();
    let rules = samus_movement::Rules::default();
    let frame = frame();
    let world = World {
        queries: Cell::new(0),
        blocked: Cell::new(false),
    };
    let inputs = Inputs {
        content: &content,
        rules: &rules,
        frame: &frame,
        world: &world,
    };
    let mut movement = owner();
    movement.begin().unwrap();
    let mut stage = Stage::default();
    loop {
        let receipt = stage
            .choose(Group::Decision, &mut movement, &inputs)
            .unwrap();
        let request = matches!(
            receipt.decisions.0.last(),
            Some(MovementDecision::MotionRequest(_))
        );
        stage.bind(WorkId(10)).unwrap();
        if request {
            break;
        }
        let before_completion = movement.clone();
        stage.complete(WorkId(10), &mut movement, &inputs).unwrap();
        assert_ne!(movement, before_completion);
    }
    let mut earlier_world = movement.clone();
    inputs.complete(&mut earlier_world).unwrap();
    world.blocked.set(true);
    let before = world.queries.get();
    let mut stage = restore(stage);
    let mut movement = restore(movement);
    stage.complete(WorkId(10), &mut movement, &inputs).unwrap();
    assert!(world.queries.get() > before);
    let Movement::Work(earlier_work) = earlier_world else {
        panic!("active earlier request");
    };
    let Movement::Work(fresh_work) = &movement else {
        panic!("active fresh request");
    };
    assert_ne!(fresh_work.model.0.offered, earlier_work.model.0.offered);
    let receipt = stage
        .choose(Group::Decision, &mut movement, &inputs)
        .unwrap();
    assert!(matches!(
        receipt.decisions.0.last(),
        Some(MovementDecision::MotionAnswer { .. })
    ));
    stage.bind(WorkId(11)).unwrap();
    let mut stage = restore(stage);
    let mut movement = restore(movement);
    world.blocked.set(false);
    let before = world.queries.get();
    stage.complete(WorkId(11), &mut movement, &inputs).unwrap();
    assert_eq!(world.queries.get(), before);
    assert_eq!(
        stage.complete(WorkId(11), &mut movement, &inputs),
        Err(Fault::Order)
    );
}
