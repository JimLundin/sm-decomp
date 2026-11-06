use data::level::{BlockKind, Bts};
use data::pose::Poses;

use super::*;
use crate::collision::{Contacts, Room, Touches};
use crate::pose::{Collisions, change};
use crate::state::room::{Block, BlockColumns};
use crate::state::samus::{Fixed, Horizontal, Jump, Movement, PoseRecord};
use crate::state::slots::FlagSet;
use crate::state::{Button, Buttons};

/// The bank $91 log's pose definitions (`$91:B629`) for the poses these cases use: facing,
/// movement type, next pose and shot; the Y radius $15. The rest stand facing right.
fn content() -> Content {
    let used: [(u16, [u8; 4]); 14] = [
        (0x09, [8, 1, 0x01, 2]),    // Moving right, not aiming.
        (0x0B, [8, 1, 0x01, 2]),    // Moving right, gun extended.
        (0x1A, [4, 3, 0xFF, 0xFF]), // Facing left, spin jump.
        (0x29, [8, 6, 0xFF, 2]),    // Facing right, falling.
        (0x4D, [8, 2, 0xFF, 2]),    // Facing right, normal jump, not aiming.
        (0x6F, [8, 6, 0x29, 3]),    // Facing right, falling, aiming down-right.
        (0x89, [8, 0x15, 0xFF, 2]), // Facing right, ran into a wall.
        (0xA4, [8, 0, 0xFF, 2]),    // Facing right, landing from a normal jump.
        (0xA7, [4, 0, 0xFF, 7]),    // Facing left, landing from a spin jump.
        (0xE4, [8, 0, 0xFF, 3]),    // Facing right, landing aiming down-right.
        (0xE6, [8, 0, 0xFF, 2]),    // Facing right, landing firing.
        (0x01, [8, 0, 0xFF, 2]),    // Facing right, standing.
        (0x15, [8, 2, 0x51, 0]),    // Facing right, normal jump, aiming up.
        (0x51, [8, 2, 0xFF, 2]),    // Facing right, normal jump, moving forward.
    ];
    let mut bytes = [8, 0, 0xFF, 2, 0, 0, 0x15, 0].repeat(0xFD);
    for (pose, head) in used {
        if let Some(definition) = bytes.as_chunks_mut::<8>().0.get_mut(usize::from(pose)) {
            definition[..4].copy_from_slice(&head);
        }
    }
    Content {
        poses: Poses::decode(&bytes).unwrap(),
        ..Content::default()
    }
}

fn pad(buttons: &[Button]) -> Buttons {
    FlagSet {
        bits: buttons.iter().fold(0, |bits, &button| bits | button.bit()),
        ..FlagSet::default()
    }
}

/// Samus in `pose`, as its definition gives, having been in it last frame too.
fn in_pose(pose: u16, movement: Movement, content: &Content) -> Samus {
    let facing = content.poses.get(PoseId(pose)).unwrap().facing;
    Samus {
        pose: PoseId(pose),
        facing,
        previous: PoseRecord {
            pose: PoseId(pose),
            facing,
            movement: movement.kind(),
        },
        movement,
        ..Samus::default()
    }
}

/// Falling, a frame's speeds: a base speed, an extra run speed and a fall.
fn falling(jump: Option<Jump>) -> Movement {
    let speed = Horizontal {
        base: Speed::new(1, 0x8000),
        extra_run: Speed::new(0, 0x4000),
        acceleration: Acceleration::Decelerating,
        momentum: false,
    };
    let vertical = Vertical {
        speed: Speed::new(4, 0x2000),
        direction: YDirection::Down,
    };
    jump.map_or(Movement::Falling { speed, vertical }, |jump| {
        Movement::Jumping {
            jump,
            speed,
            vertical,
        }
    })
}

fn landed() -> Collisions<'static> {
    Collisions {
        contacts: Contacts {
            vertical: VerticalContact::Landed,
            ..Contacts::default()
        },
        ..Collisions::default()
    }
}

#[test]
fn a_spin_jump_lands_in_the_spin_landing_with_no_base_speed() {
    let content = content();
    let mut samus = in_pose(0x1A, falling(Some(Jump::Spin)), &content);
    let collisions = landed();
    let requests = request(
        &samus,
        collisions.contacts.vertical,
        &Controller::default(),
        &content,
        PoseRequests::default(),
    );
    assert_eq!(
        requests,
        PoseRequests {
            ordinary: Some(PoseId(0xA7)),
            command: Some(PoseCommand::SolidVertical(SolidVertical::Landed(
                Grounding::Grounded
            ))),
        }
    );
    change(
        &mut samus,
        requests,
        collisions,
        &mut Touches::default(),
        &content,
    );
    assert_eq!(samus.pose, PoseId(0xA7));
    // Standing carries her speeds over; the landing stops the base speed and keeps the
    // extra run speed, which moves her one more frame.
    assert_eq!(
        samus.movement,
        Movement::Standing {
            speed: Horizontal {
                base: Speed::ZERO,
                extra_run: Speed::new(0, 0x4000),
                acceleration: Acceleration::Accelerating,
                momentum: false,
            },
        }
    );
}

#[test]
fn a_fall_lands_by_the_direction_its_pose_aims() {
    let content = content();
    let controller = Controller::default();
    let samus = in_pose(0x29, falling(None), &content);
    let landing = |samus: &Samus, controller: &Controller| {
        request(
            samus,
            VerticalContact::Landed,
            controller,
            &content,
            PoseRequests::default(),
        )
        .ordinary
    };
    // Pose $29 shoots right: the plain landing, or the firing one with shoot held.
    assert_eq!(landing(&samus, &controller), Some(PoseId(0xA4)));
    let shooting = Controller {
        held: pad(&[Button::X]),
        ..Controller::default()
    };
    assert_eq!(landing(&samus, &shooting), Some(PoseId(0xE6)));
    // Pose $6F aims down-right, and shooting doesn't change that.
    let aiming = in_pose(0x6F, falling(None), &content);
    assert_eq!(landing(&aiming, &shooting), Some(PoseId(0xE4)));
}

#[test]
fn a_fall_landing_stops_her_vertically() {
    let content = content();
    let mut samus = in_pose(0x29, falling(None), &content);
    let collisions = landed();
    let requests = request(
        &samus,
        VerticalContact::Landed,
        &Controller::default(),
        &content,
        PoseRequests::default(),
    );
    change(
        &mut samus,
        requests,
        collisions,
        &mut Touches::default(),
        &content,
    );
    assert_eq!(samus.pose, PoseId(0xA4));
    assert_eq!(samus.movement.kind(), MovementType::Standing);
    assert_eq!(
        samus.movement.horizontal().map(|speed| speed.base),
        Some(Speed::ZERO)
    );
    assert_eq!(samus.previous.pose, PoseId(0xA4));
}

#[test]
fn a_ceiling_keeps_the_pose_and_stops_the_rise() {
    let content = content();
    let mut movement = falling(Some(Jump::Normal));
    if let Some(vertical) = movement.vertical_mut() {
        vertical.direction = YDirection::Up;
    }
    let mut samus = in_pose(0x4D, movement, &content);
    let collisions = Collisions {
        contacts: Contacts {
            vertical: VerticalContact::HitCeiling,
            ..Contacts::default()
        },
        ..Collisions::default()
    };
    let requests = request(
        &samus,
        VerticalContact::HitCeiling,
        &Controller::default(),
        &content,
        PoseRequests::default(),
    );
    assert_eq!(requests.ordinary, Some(PoseId(0x4D)));
    change(
        &mut samus,
        requests,
        collisions,
        &mut Touches::default(),
        &content,
    );
    assert_eq!(samus.pose, PoseId(0x4D));
    assert_eq!(
        samus.movement.vertical(),
        Some(Vertical {
            speed: Speed::ZERO,
            direction: YDirection::Down,
        })
    );
}

#[test]
fn walking_off_a_ledge_falls_from_rest() {
    let content = content();
    let samus = in_pose(0x01, Movement::default(), &content);
    let requests = request(
        &samus,
        VerticalContact::Falling,
        &Controller::default(),
        &content,
        PoseRequests::default(),
    );
    assert_eq!(
        requests,
        PoseRequests {
            ordinary: Some(PoseId(0x29)),
            command: Some(PoseCommand::SolidVertical(SolidVertical::Falling)),
        }
    );
    // A jump already falling keeps the input's request.
    let jumping = in_pose(0x4D, falling(Some(Jump::Normal)), &content);
    let input = PoseRequests {
        ordinary: Some(PoseId(0x15)),
        command: None,
    };
    assert_eq!(
        request(
            &jumping,
            VerticalContact::Falling,
            &Controller::default(),
            &content,
            input
        ),
        input
    );
}

#[test]
fn running_into_a_wall_takes_the_ran_into_a_wall_pose() {
    let content = content();
    let mut samus = in_pose(0x09, Movement::from(MovementType::Running), &content);
    let collisions = Collisions {
        contacts: Contacts {
            wall: Some(Side::Right),
            ..Contacts::default()
        },
        ..Collisions::default()
    };
    // No request from input: the wall alone changes the pose.
    change(
        &mut samus,
        PoseRequests::default(),
        collisions,
        &mut Touches::default(),
        &content,
    );
    assert_eq!(samus.pose, PoseId(0x89));
    assert_eq!(samus.movement, Movement::RanIntoWall);
}

/// Eight blocks across and down, column 2 solid: its left edge is at X $20.
fn wall() -> Vec<Block> {
    let mut blocks = vec![Block::default(); 64];
    for block in blocks.iter_mut().skip(2).step_by(8) {
        block.kind = BlockKind::Solid(Bts(0));
    }
    blocks
}

fn beside(blocks: &[Block]) -> Collisions<'_> {
    Collisions {
        surroundings: Surroundings {
            room: Room {
                blocks,
                width: BlockColumns(8),
                slopes: None,
            },
            ..Surroundings::default()
        },
        ..Collisions::default()
    }
}

#[test]
fn a_running_pose_moves_her_a_pixel_forward_and_stops_at_a_block() {
    let content = content();
    let blocks = wall();
    let run_to = |x: u16| {
        let mut samus = in_pose(0x09, Movement::from(MovementType::Running), &content);
        samus.position.x = Fixed {
            pixels: x,
            subpixels: 0,
        };
        samus.position.y.pixels = 0x40;
        let requests = PoseRequests {
            ordinary: Some(PoseId(0x0B)),
            command: None,
        };
        change(
            &mut samus,
            requests,
            beside(&blocks),
            &mut Touches::default(),
            &content,
        );
        (samus.pose, samus.position.x)
    };
    // Her right side, X + 5 - 1, is $1E from $1A: the pixel takes her to $1B.
    assert_eq!(
        run_to(0x1A),
        (
            PoseId(0x0B),
            Fixed {
                pixels: 0x1B,
                subpixels: 0
            }
        )
    );
    // From $1B the pixel reaches the block: she stays beside it and the pose she asked for
    // runs into the wall.
    assert_eq!(
        run_to(0x1B),
        (
            PoseId(0x89),
            Fixed {
                pixels: 0x1B,
                subpixels: 0xFFFF
            }
        )
    );
}
