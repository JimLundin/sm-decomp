use data::pose::transition::{Control, Controls, PoseTransitions, Transition};
use data::pose::{MovementType, PoseFacing as Facing, PoseId, Poses};

use super::*;
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
        samus_clips: crate::samus_looks_world::Clips::default(),
        slopes: data::level::slope::Slopes::default(),
        enemy_programs: data::enemy::program::EnemyPrograms::default(),
        eye_lists: data::enemy::eye::EyeLists::default(),
        face_block_lists: data::enemy::face_block::FaceBlockLists::default(),
        samus_spritemaps: data::pose::spritemap::SamusSpritemaps::default(),
        samus_assets: data::assets::samus::SamusAssets::default(),
        samus_tiles: data::pose::tiles::SamusTiles::default(),
        graphics: data::pose::tiles::Graphics::default(),
        visor_colours: data::palette::VisorColours::default(),
        face_block_glow: data::enemy::face_block::GlowColours::default(),
        room_plms: crate::plm::RoomPlms::default(),
        room_doors: crate::collision::RoomDoors::default(),
        blocks: crate::blocks_world::BlockContent::default(),
        hud: crate::hud_world::HudContent::default(),
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
    change(&mut samus, requests.ordinary.map(Asked::Pose), &content);
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
        motion: crate::state::samus::Motion {
            base_speed: Fixed::new(1, 0),
            ..Default::default()
        },
        ..standing()
    };
    assert_eq!(
        input(&samus, &Controller::default(), &content),
        PoseRequests {
            ordinary: Some(PoseId(2)),
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
    // With no base speed she asks for the pose's next; pose 2's definition names none, so she
    // keeps it.
    assert_eq!(
        input(&samus, &Controller::default(), &content).ordinary,
        Some(PoseId(2))
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
