use clean_model_samus_motion::{Rules, Spin, Vertical};
use clean_types::{Action, Fixed as Model, Heading};
use data::level::{BlockKind, Bts};

use super::{X_RADIUS, clean_of, pose_id, write_back};
use crate::collision::Room;
use crate::collision::solid_enemy::SolidEnemies;
use crate::model::num::Fixed;
use crate::model::samus::motion::{Bounce, Position, SpeedMode, State as Motion, VerticalMotion};
use crate::pose::{Content, PoseRequests};
use crate::samus_world::SamusWorld;
use crate::samus_world::frame::FrameSource;
use crate::state::room::{Block, BlockColumns, ElevatorRide};
use crate::state::samus::{Item, MovementType, PoseId, PoseRecord, Samus};
use crate::state::slots::FlagSet;
use crate::state::{Button, Controller};

/// Running right (pose 9), facing right.
const RUNNING_RIGHT: PoseId = PoseId(0x09);
/// Spin jumping right.
const SPIN_RIGHT: PoseId = PoseId(0x19);
/// Standing facing right.
const STANDING_RIGHT: PoseId = PoseId(0x01);

fn samus(pose: PoseId, motion: Motion) -> Samus {
    Samus {
        pose,
        motion,
        y_radius: 0x15,
        ..Samus::default()
    }
}

fn motion(mode: SpeedMode, vertical: VerticalMotion, speed: u32) -> Motion {
    Motion {
        position: Position {
            x: Fixed::new(0x30, 0x1234),
            y: Fixed::new(0x2B, 0xFFFF),
        },
        base_speed: Fixed::new(1, 0x8000),
        extra_run: Fixed::new(0, 0x3000),
        momentum: false,
        mode,
        vertical_speed: Fixed(speed),
        vertical,
        bounce: Bounce::First,
    }
}

/// Every motion the game's words can hold for her comes back as it went in.
#[test]
fn the_games_words_come_back_as_they_went() {
    let cases = [
        (
            RUNNING_RIGHT,
            SpeedMode::Accelerating,
            VerticalMotion::None,
            0,
        ),
        (
            RUNNING_RIGHT,
            SpeedMode::Decelerating,
            VerticalMotion::None,
            0,
        ),
        (
            RUNNING_RIGHT,
            SpeedMode::TurningAround,
            VerticalMotion::None,
            0,
        ),
        (
            SPIN_RIGHT,
            SpeedMode::Decelerating,
            VerticalMotion::Up,
            0x2_0000,
        ),
        (
            SPIN_RIGHT,
            SpeedMode::Accelerating,
            VerticalMotion::Up,
            0xFFFF_E400,
        ),
        (
            SPIN_RIGHT,
            SpeedMode::Accelerating,
            VerticalMotion::Down,
            0x3_1C00,
        ),
        (
            SPIN_RIGHT,
            SpeedMode::Accelerating,
            VerticalMotion::None,
            0x1_3800,
        ),
        (SPIN_RIGHT, SpeedMode::Accelerating, VerticalMotion::Down, 0),
    ];
    for (pose, mode, vertical, speed) in cases {
        for (momentum, seated) in [(false, false), (true, true)] {
            let before = Samus {
                slope_adjusted: seated,
                ..samus(
                    pose,
                    Motion {
                        momentum,
                        ..motion(mode, vertical, speed)
                    },
                )
            };
            let Some(model) = clean_of(&before) else {
                panic!("pose {pose:?} is hers");
            };
            let mut after = Samus {
                motion: Motion {
                    momentum,
                    ..Motion::default()
                },
                slope_adjusted: !seated,
                ..before
            };
            write_back(&model, &mut after);
            assert_eq!(after, before, "{pose:?} {mode:?} {vertical:?} {speed:#x}");
        }
    }
}

/// The game's words mean what the decode notes say: a skid points against her facing, a rise
/// past zero is spent, no direction in a spin is a moonfall.
#[test]
fn what_she_remembers_reads_from_the_games_words() {
    let read = |pose, mode, vertical, speed| {
        clean_of(&samus(pose, motion(mode, vertical, speed))).map(|model| {
            (
                model.carry,
                model.way,
                model.vertical,
                model.moonfall,
                model.losing_dash,
            )
        })
    };
    let skid = read(
        RUNNING_RIGHT,
        SpeedMode::TurningAround,
        VerticalMotion::None,
        0,
    );
    assert!(skid.is_some_and(|(carry, way, vertical, ..)| carry
        && way == Heading::Left
        && vertical == Vertical::Ground));
    let spent = read(
        SPIN_RIGHT,
        SpeedMode::Accelerating,
        VerticalMotion::Up,
        0xFFFF_E400,
    );
    assert!(spent.is_some_and(|(.., vertical, _, _)| vertical == Vertical::Spent(Model(0x1C00))));
    let moonfall = read(
        SPIN_RIGHT,
        SpeedMode::Accelerating,
        VerticalMotion::None,
        0x1_3800,
    );
    assert!(
        moonfall.is_some_and(|(.., vertical, moonfall, losing)| moonfall
            && losing
            && vertical == Vertical::Falls(Model(0x1_3800)))
    );
    // A pose outside her domain has no model.
    assert!(clean_of(&samus(PoseId(0xD3), Motion::default())).is_none());
}

/// A spin is carried on when the pose before the last change was a spin or a wall jump.
#[test]
fn a_spin_is_carried_on_by_the_pose_history() {
    let mut spinning = samus(SPIN_RIGHT, Motion::default());
    assert!(clean_of(&spinning).is_some_and(|model| model.spin == Spin::Off));
    spinning.last_different = PoseRecord {
        pose: SPIN_RIGHT,
        movement: MovementType::SpinJumping,
        ..PoseRecord::default()
    };
    assert!(clean_of(&spinning).is_some_and(|model| model.spin == Spin::CarriedOn));
}

/// The engine's pose for the model's: hers while it shows it, else her dress's.
#[test]
fn the_pose_id_keeps_hers_and_dresses_a_new_one() {
    let mut spinning = samus(SPIN_RIGHT, Motion::default());
    let Some(model) = clean_of(&spinning) else {
        panic!("a spin is hers");
    };
    assert_eq!(pose_id(model.pose, &spinning, None), Some(SPIN_RIGHT));
    spinning.equipment.equipped.insert(Item::ScrewAttack);
    spinning.pose = STANDING_RIGHT;
    let screw = pose_id(model.pose, &spinning, None);
    assert!(screw.is_some_and(|id| id != SPIN_RIGHT));
    assert_eq!(
        pose_id(model.pose, &spinning, Some(SPIN_RIGHT)),
        Some(SPIN_RIGHT)
    );
}

/// Four blocks across and four down, the bottom row solid.
fn floor() -> Vec<Block> {
    let mut blocks = vec![Block::default(); 16];
    for block in &mut blocks[12..] {
        block.kind = BlockKind::Solid(Bts(0));
    }
    blocks
}

/// One frame in a test room: running right on a floor holding right, she gains her running
/// step, moves by her speed and dash, and the pull keeps her feet on the floor.
#[test]
fn a_frame_on_the_floor_runs_her_right() {
    let blocks = floor();
    let room = Room {
        blocks: &blocks,
        width: BlockColumns(4),
        slopes: None,
    };
    let world = SamusWorld::new(room, SolidEnemies::default());
    // Her feet on the floor's top (y 30h): centre 30h less her radius.
    let mut engine = samus(
        RUNNING_RIGHT,
        Motion {
            position: Position {
                x: Fixed::new(0x20, 0),
                y: Fixed::new(0x30 - 0x15, 0),
            },
            base_speed: Fixed::new(1, 0),
            momentum: true,
            ..Motion::default()
        },
    );
    let controller = Controller {
        held: FlagSet {
            bits: Button::Right.bit(),
            ..FlagSet::default()
        },
        ..Controller::default()
    };
    let content = Content::default();
    let frame = FrameSource {
        samus: &engine,
        controller: &controller,
        ride: ElevatorRide::default(),
        completed: 0,
        requests: PoseRequests::default(),
        content: &content,
    }
    .frame();
    let Some(mut model) = clean_of(&engine) else {
        panic!("running is hers");
    };
    let outcome = model.step(&frame, &world, &Rules::REFERENCE);
    write_back(&model, &mut engine);
    assert!(outcome.met.is_none());
    assert_eq!(model.pose.action, Action::Running);
    assert_eq!(engine.motion.base_speed, Fixed::new(1, 0x3000));
    assert_eq!(engine.motion.position.x, Fixed::new(0x21, 0x3000));
    assert_eq!(engine.motion.position.y, Fixed::new(0x30 - 0x15, 0xFFFF));
    assert_eq!(engine.motion.vertical, VerticalMotion::None);
    assert_eq!(X_RADIUS, frame.radius.x);
}
