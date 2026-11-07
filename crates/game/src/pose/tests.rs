use data::pose::animation::AnimationDelays;
use data::pose::transition::{Control, Controls, PoseTransitions, Transition};
use data::pose::{MovementType, PoseFacing as Facing, PoseId, Poses};

use super::*;
use crate::collision::Touches;
use crate::state::slots::FlagSet;

/// Pose definition bytes: facing, movement type, next pose, shot, then four more.
fn definition(facing: u8, movement: u8, next: u8) -> [u8; 8] {
    [facing, movement, next, 2, 0, 0, 0x15, 0]
}

/// Poses 0 to 2: 1 stands facing right and turns into pose 0 by itself; 2 runs right.
fn content() -> Content {
    let bytes: Vec<u8> = [
        definition(0, 0, 0xFF),
        definition(8, 0, 0),
        definition(8, 1, 0xFF),
    ]
    .concat();
    let run_right = Transition {
        pressed: Controls::NONE,
        held: Controls::of(&[Control::Right]),
        pose: PoseId(2),
    };
    Content {
        poses: Poses::decode(&bytes).unwrap(),
        transitions: PoseTransitions::new(vec![Vec::new(), vec![run_right], vec![run_right]]),
        speeds: HorizontalSpeeds::default(),
        vertical: VerticalSpeeds::default(),
        delays: AnimationDelays::default(),
        slopes: data::level::slope::Slopes::default(),
        enemy_programs: data::enemy::program::EnemyPrograms::default(),
        eye_lists: data::enemy::eye::EyeLists::default(),
        face_block_lists: data::enemy::face_block::FaceBlockLists::default(),
        samus_spritemaps: data::pose::spritemap::SamusSpritemaps::default(),
        samus_tiles: data::pose::tiles::SamusTiles::default(),
        graphics: data::pose::tiles::Graphics::default(),
        visor_colours: data::palette::VisorColours::default(),
        face_block_glow: data::enemy::face_block::GlowColours::default(),
        room_plms: crate::plm::RoomPlms::default(),
        area_maps: data::map::AreaMaps::default(),
        tile_tables: data::tileset::TileTables::default(),
        message_boxes: data::message::MessageBoxes::default(),
    }
}

fn pad(buttons: &[Button]) -> Buttons {
    FlagSet {
        bits: buttons.iter().fold(0, |bits, &button| bits | button.bit()),
        ..FlagSet::default()
    }
}

fn standing() -> Samus {
    Samus {
        pose: PoseId(1),
        facing: Facing::Right,
        previous: PoseRecord {
            pose: PoseId(1),
            facing: Facing::Right,
            movement: MovementType::Standing,
        },
        ..Samus::default()
    }
}

#[test]
fn holding_right_while_standing_runs() {
    let content = content();
    let mut samus = standing();
    let controller = Controller {
        held: pad(&[Button::Right]),
        ..Controller::default()
    };
    let requests = input(&samus, &controller, &content);
    assert_eq!(requests.ordinary, Some(PoseId(2)));
    change(
        &mut samus,
        requests,
        still(&content),
        &mut Touches::default(),
        &content,
    );
    assert_eq!(samus.pose, PoseId(2));
    assert_eq!(samus.movement, MovementType::Running);
    assert_eq!(samus.previous.pose, PoseId(2));
    assert_eq!(samus.last_different.pose, PoseId(1));
}

#[test]
fn without_a_matching_entry_the_pose_takes_its_next() {
    let content = content();
    let samus = standing();
    let controller = Controller {
        held: pad(&[Button::Left]),
        ..Controller::default()
    };
    assert_eq!(
        input(&samus, &controller, &content).ordinary,
        Some(PoseId(0))
    );
    assert_eq!(
        input(&samus, &Controller::default(), &content).ordinary,
        Some(PoseId(0))
    );
}

#[test]
fn a_matching_entry_for_the_same_pose_asks_for_nothing() {
    let content = content();
    let samus = Samus {
        pose: PoseId(2),
        movement: MovementType::Running,
        ..standing()
    };
    let controller = Controller {
        held: pad(&[Button::Right]),
        ..Controller::default()
    };
    assert_eq!(input(&samus, &controller, &content).ordinary, None);
}

#[test]
fn running_with_speed_keeps_its_pose_when_released() {
    let content = content();
    let samus = Samus {
        pose: PoseId(2),
        movement: MovementType::Running,
        motion: crate::model::samus::motion::State {
            base_speed: Fixed::new(1, 0),
            ..Default::default()
        },
        ..standing()
    };
    assert_eq!(
        input(&samus, &Controller::default(), &content),
        PoseRequests {
            ordinary: Some(PoseId(2)),
            command: Some(PoseCommand::Decelerate),
        }
    );
}

#[test]
fn running_without_base_speed_stops_when_released() {
    let content = content();
    let samus = Samus {
        pose: PoseId(2),
        movement: MovementType::Running,
        ..standing()
    };
    // Pose 2's definition names no next pose, so she keeps it; the command is a stop.
    assert_eq!(
        input(&samus, &Controller::default(), &content).command,
        Some(PoseCommand::Stop)
    );
}

#[test]
fn a_crouching_transition_takes_no_input() {
    let content = content();
    let samus = Samus {
        movement: MovementType::CrouchingTransition,
        ..standing()
    };
    let controller = Controller {
        held: pad(&[Button::Right]),
        ..Controller::default()
    };
    assert_eq!(
        input(&samus, &controller, &content),
        PoseRequests::default()
    );
}

/// A pose record in `pose` with `movement`, facing `facing`.
const fn record(pose: u16, movement: MovementType, facing: Facing) -> PoseRecord {
    PoseRecord {
        pose: PoseId(pose),
        facing,
        movement,
    }
}

#[test]
fn a_new_pose_starts_motion_by_its_movement_type_and_the_old_pose() {
    use MovementType as M;
    let right = Facing::Right;
    let cases = [
        // A take-off from the crouch without aim, then one from standing.
        (
            record(0x27, M::Crouching, right),
            record(0x4B, M::NormalJumping, right),
            Some(PoseStart::TakeOff {
                from_plain_crouch: true,
            }),
        ),
        (
            record(0x01, M::Standing, right),
            record(0x4B, M::NormalJumping, right),
            Some(PoseStart::TakeOff {
                from_plain_crouch: false,
            }),
        ),
        // A normal jumping pose past the take-off, and a fall.
        (
            record(0x4B, M::NormalJumping, right),
            record(0x4D, M::NormalJumping, right),
            Some(PoseStart::JumpOrFall),
        ),
        (
            record(0x01, M::Standing, right),
            record(0x29, M::Falling, right),
            Some(PoseStart::JumpOrFall),
        ),
        // A spin jump from the ground starts one; from a spin it doesn't, unless turned.
        (
            record(0x09, M::Running, right),
            record(0x19, M::SpinJumping, right),
            Some(PoseStart::SpinStart),
        ),
        (
            record(0x19, M::SpinJumping, right),
            record(0x1B, M::SpinJumping, right),
            None,
        ),
        (
            record(0x1A, M::SpinJumping, Facing::Left),
            record(0x19, M::SpinJumping, right),
            Some(PoseStart::SpinReverse),
        ),
        (
            record(0x01, M::Standing, right),
            record(0x25, M::TurningAroundOnGround, right),
            Some(PoseStart::Turn),
        ),
        (
            record(0x19, M::SpinJumping, right),
            record(0x83, M::WallJumping, right),
            Some(PoseStart::WallJumpStart),
        ),
        (
            record(0x01, M::Standing, right),
            record(0x09, M::Running, right),
            None,
        ),
    ];
    for (old, new, start) in cases {
        let samus = Samus {
            pose: new.pose,
            facing: new.facing,
            movement: new.movement,
            ..Samus::default()
        };
        assert_eq!(pose_start(old, &samus), start, "{old:?} to {new:?}");
    }
}

/// A frame whose motion didn't run.
fn still(content: &Content) -> Collisions<'_> {
    Collisions {
        motion: Motion {
            inputs: None,
            rules: crate::motion::rules(content, crate::session::SpeedCap::Wordwise),
            contacts: crate::model::samus::motion::Contacts::default(),
        },
        surroundings: Surroundings::default(),
    }
}
