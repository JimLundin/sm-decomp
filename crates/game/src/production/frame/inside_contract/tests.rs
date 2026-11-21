use super::super::{inside_world::InsideWorld, movement_stage};
use super::*;
use crate::production::world::Boundary;
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

fn selected() -> (
    Movement,
    movement_stage::Stage,
    MovementContent,
    InsideWorld,
) {
    let content = content();
    let room = InsideWorld::new(16, vec![types::Block::Air; 256], Boundary::Solid).unwrap();
    let mut movement = owner();
    movement.begin().unwrap();
    let mut stage = movement_stage::Stage::Idle;
    room.choose(
        &mut stage,
        &mut movement,
        &content,
        &samus_movement::Rules::default(),
        &frame(),
    )
    .unwrap();
    (movement, stage, content, room)
}

fn template() -> Template {
    use data::timing::{
        Cycle, DIVIDEND_LOW_PORT, ExposureKind, MULTIPLICAND_PORT, MULTIPLIER_PORT, TemplateRun,
    };
    let mut runs = Vec::new();
    for _ in 0..3 {
        for effect in [
            TemplateEffect::Publish(MULTIPLICAND_PORT),
            TemplateEffect::Publish(MULTIPLIER_PORT),
            TemplateEffect::Publish(DIVIDEND_LOW_PORT),
            TemplateEffect::Sample(Sample::ProductLow),
            TemplateEffect::Sample(Sample::ProductHigh),
        ] {
            runs.push(TemplateRun {
                count: 1,
                cycle: Cycle::Six,
                kind: match effect {
                    TemplateEffect::Publish(_) => ExposureKind::Write,
                    TemplateEffect::Sample(_) => ExposureKind::Read,
                    TemplateEffect::None => ExposureKind::Internal,
                },
                poll_before: false,
                effect,
            });
        }
    }
    Template { runs }
}

fn completed_pose_input(
    pose: types::Pose,
) -> super::super::borrowed_player_preparation::PoseInputCompletion {
    use super::super::borrowed_player_preparation::*;
    let binding = PhysicalButtons(0);
    let mut prospective = pose;
    prospective.facing = types::Facing::Left;
    PoseInputCompletion {
        dependency: PoseInputOrigin {
            pose,
            action: pose.action,
            held: binding,
            pressed: binding,
            bindings: InputBindings {
                shoot: binding,
                jump: binding,
                run: binding,
                cancel: binding,
                aim_up: binding,
                aim_down: binding,
            },
        },
        terminal: PoseInputTerminal {
            pressed_actions: types::Buttons::default(),
            held_actions: types::Buttons::default(),
            prospective,
            special: None,
            super_special: None,
            commands: [PoseCommand::None; 3],
            radius: Pair {
                x: Wrapping(5),
                y: Wrapping(21),
            },
            bomb_jump: BombJumpDirection::None,
        },
    }
}

#[test]
fn actual_bounds_and_complete_current_dependencies_admit_one_whole_owner() {
    let (movement, stage, content, room) = selected();
    let movement_stage::Stage::Offered(receipt) = stage else {
        panic!();
    };
    let pose_input = completed_pose_input(movement.state().pose);
    let required = RequiredDependencies {
        pixels: [100, 100],
        pose: movement.state().pose,
        radius: [5, 21],
        room: Box::new(room.clone()),
    };
    let contract = PhysicalContract::new(required.clone(), template());
    assert!(
        contract
            .physical(
                &receipt,
                &movement,
                &content,
                &room,
                &pose_input,
                GeometryMatch::Origin
            )
            .is_ok()
    );
    assert_eq!(
        Requests::from_receipt(&receipt, 16).unwrap(),
        Requests {
            rows: [7, 6, 4],
            width: 16
        }
    );
    let mut wrong = required.clone();
    wrong.pixels[0] = 101;
    assert_eq!(
        PhysicalContract::new(wrong, template()).physical(
            &receipt,
            &movement,
            &content,
            &room,
            &pose_input,
            GeometryMatch::Origin,
        ),
        Err(Fault::Dependencies)
    );
    let mut wrong = required;
    wrong.radius = [5, 20];
    assert_eq!(
        PhysicalContract::new(wrong, template()).physical(
            &receipt,
            &movement,
            &content,
            &room,
            &pose_input,
            GeometryMatch::Origin,
        ),
        Err(Fault::Dependencies)
    );
    let other = InsideWorld::new(16, vec![types::Block::Solid; 256], Boundary::Solid).unwrap();
    assert_eq!(
        contract.physical(
            &receipt,
            &movement,
            &content,
            &other,
            &pose_input,
            GeometryMatch::Origin
        ),
        Err(Fault::Dependencies)
    );
    let mut stale_body = pose_input;
    stale_body.terminal.radius.y = Wrapping(20);
    assert_eq!(
        contract.physical(
            &receipt,
            &movement,
            &content,
            &room,
            &stale_body,
            GeometryMatch::Origin
        ),
        Err(Fault::Dependencies)
    );
    let mut incomplete = template();
    incomplete.runs.truncate(5);
    assert_eq!(
        Requests {
            rows: [7, 6, 4],
            width: 16
        }
        .bind(&incomplete),
        Err(Fault::Profile)
    );
}

#[test]
fn recurring_inside_qualifies_the_live_pose_at_the_same_room_cells() {
    let (movement, stage, content, room) = selected();
    let movement_stage::Stage::Offered(receipt) = stage else {
        panic!();
    };
    let pose_input = completed_pose_input(movement.state().pose);
    let mut prior_pose = movement.state().pose;
    prior_pose.aim = types::Aim::DownForward;
    let profile = |x| {
        PhysicalContract::new(
            RequiredDependencies {
                pixels: [x, 100],
                pose: prior_pose,
                radius: [5, 21],
                room: Box::new(room.clone()),
            },
            template(),
        )
        .physical(
            &receipt,
            &movement,
            &content,
            &room,
            &pose_input,
            GeometryMatch::Recurring,
        )
    };
    assert!(profile(101).is_ok());
    assert_eq!(profile(116), Err(Fault::Dependencies));
}

#[test]
fn width_row_and_zero_publications_and_six_live_samples_remain_exact() {
    let requests = Requests {
        rows: [7, 6, 4],
        width: 16,
    };
    let bound = requests.bind(&template()).unwrap();
    let writes = bound
        .runs
        .iter()
        .filter_map(|run| match &run.exposure.effect {
            data::timing::DeviceEffect::Publish(write) => Some(write.effect.clone()),
            data::timing::DeviceEffect::None | data::timing::DeviceEffect::Sample(_) => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        writes,
        [
            Publication::Multiplicand(16),
            Publication::Multiplier(7),
            Publication::DividendLow(0),
            Publication::Multiplicand(16),
            Publication::Multiplier(6),
            Publication::DividendLow(0),
            Publication::Multiplicand(16),
            Publication::Multiplier(4),
            Publication::DividendLow(0)
        ]
    );
    let mut pending = Pending::new(WorkId(15), requests);
    assert_eq!(
        pending.sample(WorkId(16), Sample::ProductLow, 112),
        Err(Fault::Work)
    );
    assert_eq!(pending.complete(WorkId(15)), Err(Fault::Samples));
    for (device, byte) in [
        (Sample::ProductLow, 112),
        (Sample::ProductHigh, 0),
        (Sample::ProductLow, 96),
        (Sample::ProductHigh, 0),
        (Sample::ProductLow, 64),
        (Sample::ProductHigh, 0),
    ] {
        pending.sample(WorkId(15), device, byte).unwrap();
        pending = canonical::from_bytes(&canonical::to_bytes(&pending)).unwrap();
    }
    assert_eq!(pending.complete(WorkId(15)), Ok(()));
    assert_eq!(
        pending.sample(WorkId(15), Sample::ProductLow, 0),
        Err(Fault::Samples)
    );
    assert_eq!(pending.complete(WorkId(16)), Err(Fault::Work));
    let mut wrong = Pending::new(WorkId(15), requests);
    for device in [
        Sample::ProductLow,
        Sample::ProductHigh,
        Sample::ProductLow,
        Sample::ProductHigh,
        Sample::ProductLow,
        Sample::ProductHigh,
    ] {
        wrong.sample(WorkId(15), device, 0).unwrap();
    }
    assert_eq!(wrong.complete(WorkId(15)), Err(Fault::Samples));
}
