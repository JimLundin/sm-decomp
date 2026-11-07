use clean_types::Parity;
use data::assets::samus::import::map;
use data::assets::samus::key::{Dress, SpinSprite, Suit};

use super::{FrameSource, body};
use crate::pose::{Content, PoseRequests};
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
        requests: PoseRequests::default(),
        content: &content,
    };
    let plain = Dress {
        spin: SpinSprite::Plain,
        suit: Suit::First,
    };
    let sized = |pose| body(&content, plain, pose);
    let frame = source.frame(&sized);
    let pad = frame.pad;
    assert_eq!(
        [pad.jump, pad.run, pad.right, pad.aim_up],
        [true, true, true, true]
    );
    assert_eq!(
        [pad.shoot, pad.left, pad.up, pad.down, pad.aim_down],
        [false; 5]
    );
    let pressed = frame.pressed;
    assert!(pressed.jump && !pressed.run && !pressed.right && !pressed.aim_up);
    assert!(frame.equipment.hi_jump);
    assert!(!frame.equipment.space_jump);
    assert!(frame.elevator_moving);
    assert!(frame.parity == Parity::Odd);
    // A pose the empty pose table doesn't hold has no half-height.
    let standing = map::key_of(PoseId(0x01)).map(frame.body);
    assert!(standing.is_some_and(|radius| (radius.x, radius.y) == (5, 0)));
    assert!(!frame.animation_ends);

    let still = FrameSource {
        completed: 8,
        ride: ElevatorRide::default(),
        ..source
    }
    .frame(&sized);
    assert!(still.parity == Parity::Even);
    assert!(!still.elevator_moving);
}
