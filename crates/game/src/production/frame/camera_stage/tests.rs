use super::super::movement::{ReceiptsPin, Work, WorkPin};
use super::*;
use types::{Pair, Set};

fn samus() -> samus_movement::Samus {
    samus_movement::Samus {
        pose: types::Pose {
            body: types::Body::Upright,
            action: types::Action::Running,
            facing: types::Facing::Right,
            aim: types::Aim::Forward,
            gun: types::Gun::Stowed,
            legs: types::Legs::Together,
        },
        position: Pair {
            x: types::px(Wrapping(200)),
            y: types::fixed(Wrapping(300), 0x1234),
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
    }
}

struct Air;
impl types::World for Air {
    fn block(&self, _: types::Cell) -> types::Block {
        types::Block::Air
    }
    fn lets_through(&self, _: types::Cell, _: types::Way) -> bool {
        false
    }
    fn solid_enemies(&self) -> &[types::Hitbox] {
        &[]
    }
}

fn finished() -> Movement {
    let frame = samus_movement::Frame {
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
    };
    let mut work = samus_movement::suspended::SuspendedMovement::new(samus()).unwrap();
    let mut decisions = Vec::new();
    while let Some(decision) = work
        .offer(&frame, &Air, &samus_movement::Rules::default())
        .unwrap()
        .cloned()
    {
        decisions.push(decision);
        work.complete(&frame, &Air, &samus_movement::Rules::default())
            .unwrap();
    }
    assert!(work.is_complete());
    // Constructed completed publications distinguish all three Camera motion dependencies.
    work.state.position = Pair {
        x: types::px(Wrapping(220)),
        y: types::fixed(Wrapping(330), 0x5678),
    };
    work.state.pose.facing = types::Facing::Left;
    work.outputs.skidding = true;
    work.outputs.rising = true;
    work.outputs.placed = Some(types::fixed(Wrapping(329), 0x9999));
    Movement::Work(Work {
        model: Model::<WorkPin>(Box::new(work)),
        completed: Model::<ReceiptsPin>(decisions),
    })
}

fn inputs() -> Inputs {
    Inputs {
        room: Room {
            identity: data::id::RoomId(99),
            screens: vec![Screen::Open; 20],
            size: [4, 4],
            heights: [91, 153],
        },
        lead: Lead::Ahead,
        time_stopped: false,
        tuning: Tuning::capture(&world_camera::Rules::REFERENCE),
        grapple: Activity::Inactive,
        knockback: Activity::Inactive,
        hook: Activity::Inactive,
    }
}

fn camera(previous: &Previous) -> Model<CameraPin> {
    Model(world_camera::Camera::restore(
        world_camera::ViewPosition(Pair {
            x: types::fixed(Wrapping(100), 0xabcd),
            y: types::fixed(Wrapping(200), 0xef01),
        }),
        world_camera::RememberedPosition(previous.0.0.position),
        types::VerticalFraction(0x3456),
    ))
}

fn restore<T: Canonical>(value: &T) -> T {
    let bytes = canonical::to_bytes(value);
    canonical::from_bytes(&bytes).unwrap()
}

#[test]
fn complete_follow_publishes_once_and_uses_actual_motion_outputs() {
    let previous = Previous::capture(&Movement::new(samus()));
    let movement = finished();
    let mut camera = camera(&previous);
    let initial = camera;
    let mut stage = Stage::Idle;
    let receipt = stage
        .choose(&camera, &previous, &movement, &inputs())
        .unwrap()
        .clone();
    assert_eq!(receipt.movement, movement);
    assert_eq!(receipt.previous, previous);
    assert_eq!(stage.viewport(&camera), None);
    assert_eq!(camera, initial);
    let work = WorkId(17);
    stage.bind(work).unwrap();
    stage
        .sample(work, data::timing::Sample::ProductLow, 0)
        .unwrap();
    stage = restore(&stage);
    assert_eq!(
        stage.complete(&mut camera, WorkId(18)),
        Err(Fault::Completion)
    );
    assert_eq!(camera, initial);
    assert_eq!(stage.complete(&mut camera, work), Err(Fault::Samples));
    stage
        .sample(work, data::timing::Sample::ProductHigh, 0)
        .unwrap();
    stage
        .sample(work, data::timing::Sample::ProductLow, 0)
        .unwrap();
    stage
        .sample(work, data::timing::Sample::ProductHigh, 0)
        .unwrap();
    stage.complete(&mut camera, work).unwrap();
    assert_eq!(camera.0.seen(), movement.state().position);
    // Placement replaces the remembered pixel only; her new fraction minus the old is retained.
    assert_eq!(
        camera.0.vertical_fraction(),
        types::VerticalFraction(0x4444)
    );
    // Left-facing skid leads right; rising aims at the higher room aim.
    assert_eq!(stage.viewport(&camera), Some([121, 198]));
    let completed = camera;
    assert_eq!(stage.complete(&mut camera, work), Err(Fault::Order));
    assert_eq!(camera, completed);
}

#[test]
fn unfinished_movement_wrong_previous_and_changed_camera_reject_atomically() {
    let previous = Previous::capture(&Movement::new(samus()));
    let mut camera = camera(&previous);
    let initial = camera;
    let mut stage = Stage::Idle;
    assert_eq!(
        stage.choose(&camera, &previous, &Movement::new(samus()), &inputs()),
        Err(Fault::Movement)
    );
    assert_eq!(stage, Stage::Idle);
    let mut wrong = previous.clone();
    wrong.0.0.position.x = types::px(Wrapping(199));
    assert_eq!(
        stage.choose(&camera, &wrong, &finished(), &inputs()),
        Err(Fault::Previous)
    );
    assert_eq!(camera, initial);
    stage
        .choose(&camera, &previous, &finished(), &inputs())
        .unwrap();
    stage.bind(WorkId(19)).unwrap();
    camera.0 = camera.0.with_view(world_camera::ViewPosition(Pair {
        x: types::px(Wrapping(99)),
        y: types::px(Wrapping(200)),
    }));
    let changed = camera;
    assert_eq!(
        stage.complete(&mut camera, WorkId(19)),
        Err(Fault::Completion)
    );
    assert_eq!(camera, changed);
}

#[test]
fn current_room_lead_tuning_and_stop_are_retained_through_suspension() {
    let previous = Previous::capture(&Movement::new(samus()));
    let movement = finished();
    let mut camera = camera(&previous);
    let mut current = inputs();
    current.lead = Lead::KeepLeft;
    current.time_stopped = true;
    current.tuning.extra = 0;
    let mut stage = Stage::Idle;
    let receipt = stage
        .choose(&camera, &previous, &movement, &current)
        .unwrap()
        .clone();
    stage.bind(WorkId(23)).unwrap();
    for device in [
        data::timing::Sample::ProductLow,
        data::timing::Sample::ProductHigh,
        data::timing::Sample::ProductLow,
        data::timing::Sample::ProductHigh,
    ] {
        stage.sample(WorkId(23), device, 0).unwrap();
    }
    current.room.screens.fill(Screen::Closed);
    current.room.heights = [1, 1];
    current.tuning.extra = u32::MAX;
    current.lead = Lead::KeepRight;
    current.time_stopped = false;
    stage = restore(&stage);
    assert_eq!(stage.complete(&mut camera, WorkId(23)).unwrap(), &receipt);
    assert_eq!(stage.viewport(&camera), Some([120, 199]));
}

#[test]
fn live_products_must_match_actual_current_operands_before_publication() {
    let previous = Previous::capture(&Movement::new(samus()));
    let mut camera = camera(&previous);
    camera.0 = camera.0.with_view(world_camera::ViewPosition(Pair {
        x: types::px(Wrapping(100)),
        y: types::px(Wrapping(512)),
    }));
    let initial = camera;
    let mut stage = Stage::Idle;
    let receipt = stage
        .choose(&camera, &previous, &finished(), &inputs())
        .unwrap();
    assert_eq!(
        Requests::from_receipt(receipt).unwrap(),
        Requests { row: 2, width: 4 }
    );
    assert_eq!(Requests::from_receipt(receipt).unwrap().product(), 8);
    stage.bind(WorkId(31)).unwrap();
    assert_eq!(
        stage.sample(WorkId(32), data::timing::Sample::ProductLow, 8),
        Err(Fault::Completion)
    );
    assert_eq!(
        stage.sample(WorkId(31), data::timing::Sample::ProductHigh, 0),
        Err(Fault::Samples)
    );
    stage
        .sample(WorkId(31), data::timing::Sample::ProductLow, 7)
        .unwrap();
    stage = restore(&stage);
    stage
        .sample(WorkId(31), data::timing::Sample::ProductHigh, 0)
        .unwrap();
    stage
        .sample(WorkId(31), data::timing::Sample::ProductLow, 8)
        .unwrap();
    stage
        .sample(WorkId(31), data::timing::Sample::ProductHigh, 0)
        .unwrap();
    assert_eq!(stage.complete(&mut camera, WorkId(31)), Err(Fault::Samples));
    assert_eq!(camera, initial);
    assert_eq!(
        stage.sample(WorkId(31), data::timing::Sample::ProductLow, 8),
        Err(Fault::Samples)
    );
}
