use data::assets::samus::map;
use data::pose::Poses;
use types::Parity;
use types::button::{AIM_DOWN, AIM_UP, DOWN, JUMP, LEFT, RIGHT, RUN, SHOOT, UP};

use super::{FrameSource, body};
use crate::pose::Content;
use crate::state::room::{ElevatorRide, ElevatorStatus};
use crate::state::samus::{Item, PoseId, Samus};
use crate::state::slots::FlagSet;
use crate::state::{Button, Buttons, Controller};

fn buttons(held: &[Button]) -> Buttons {
    FlagSet {
        bits: held.iter().fold(0, |bits, &button| bits | button.bit()),
        ..FlagSet::default()
    }
}

#[test]
fn the_pad_items_elevator_parity_and_radius_reach_the_model() {
    let mut samus = Samus {
        y_radius: 0x15,
        ..Samus::default()
    };
    samus.equipment.equipped.insert(Item::HiJumpBoots);
    let controller = Controller {
        held: buttons(&[Button::A, Button::B, Button::Right, Button::R, Button::Y]),
        pressed: buttons(&[Button::A]),
        ..Controller::default()
    };
    let content = Content::default();
    let source = FrameSource {
        samus: &samus,
        controller: &controller,
        ride: ElevatorRide {
            status: ElevatorStatus::LeavingRoom,
            ..ElevatorRide::default()
        },
        completed: 7,
        content: &content,
    };
    let sized = |pose| body(&content, &samus, pose);
    let frame = source.frame(&sized);
    let pad = frame.pad;
    let held = [JUMP, RUN, RIGHT, AIM_UP];
    assert!(held.iter().all(|&set| pad.any(set)));
    let free = [SHOOT, LEFT, UP, DOWN, AIM_DOWN];
    assert!(free.iter().all(|&set| !pad.any(set)));
    let pressed = frame.pressed;
    assert!(pressed.any(JUMP) && !pressed.any(types::Buttons(RUN.0 | RIGHT.0 | AIM_UP.0)));
    assert!(frame.items[types::Item::HiJumpBoots]);
    assert!(!frame.items[types::Item::SpaceJump]);
    assert!(frame.elevator_moving);
    assert_eq!(frame.parity, Parity::Odd);
    // A pose the empty pose table doesn't hold has no half-height.
    let standing = map::key_of(PoseId(0x01)).map(frame.body);
    assert!(standing.is_some_and(|radius| (radius.x.0, radius.y.0) == (5, 0)));
    assert_eq!(frame.animation_ends, None);

    let still = FrameSource {
        completed: 8,
        ride: ElevatorRide::default(),
        ..source
    }
    .frame(&sized);
    assert_eq!(still.parity, Parity::Even);
    assert!(!still.elevator_moving);
}

/// §1.3 body size: the pose she is in has her radius word, not the table's; a pose she isn't in
/// has the table's for the pose her dress shows it by.
#[test]
fn the_pose_she_is_in_has_her_radius_word() {
    // An invented table in which each pose's half-height is its number.
    let bytes: Vec<u8> = (0..=0x9B_u8)
        .flat_map(|number| [0, 0, 0xFF, 0, 0, 0, number, 0])
        .collect();
    let Ok(poses) = Poses::decode(&bytes) else {
        panic!("the invented table decodes");
    };
    let content = Content {
        poses,
        ..Content::default()
    };
    let mut samus = Samus {
        pose: PoseId(0x00),
        y_radius: 0x15,
        ..Samus::default()
    };
    samus.equipment.equipped.insert(Item::VariaSuit);
    let Some(forward) = map::key_of(PoseId(0x00)) else {
        panic!("pose 00h faces the screen");
    };
    assert_eq!(body(&content, &samus, forward).y.0, 0x15);
    samus.pose = PoseId(0x01);
    assert_eq!(body(&content, &samus, forward).y.0, 0x9B);
}
