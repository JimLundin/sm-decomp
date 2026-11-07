use clean_types::{Button as ModelButton, Pad, Parity, Radius};

use super::FrameSource;
use crate::pose::{Content, PoseRequests};
use crate::state::room::{ElevatorRide, ElevatorStatus};
use crate::state::samus::{Item, Samus};
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
    let frame = source.frame();
    let held = Pad::default()
        .with(ModelButton::Jump)
        .with(ModelButton::Run)
        .with(ModelButton::Right)
        .with(ModelButton::AimUp);
    assert_eq!(frame.pad, held);
    assert_eq!(frame.pressed, Pad::default().with(ModelButton::Jump));
    assert!(frame.equipment.hi_jump);
    assert!(!frame.equipment.space_jump);
    assert!(frame.elevator_moving);
    assert_eq!(frame.parity, Parity::Odd);
    assert_eq!(frame.radius, Radius { x: 5, y: 0x15 });
    assert!(!frame.animation_ends);

    let still = FrameSource {
        completed: 8,
        ride: ElevatorRide::default(),
        ..source
    }
    .frame();
    assert_eq!(still.parity, Parity::Even);
    assert!(!still.elevator_moving);
}
