use data::level::{BlockKind, Bts};
use data::pose::Poses;

use super::*;
use crate::collision::{Room, Surroundings, Touches};
use crate::model::num::Fixed;
use crate::model::samus::motion::{Contacts, SpeedMode, State, VerticalMotion};
use crate::motion::Motion;
use crate::pose::{Collisions, change};
use crate::state::room::ElevatorRide;
use crate::state::room::{Block, BlockColumns};
use crate::state::samus::PoseRecord;
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
fn in_pose(pose: u16, motion: State, content: &Content) -> Samus {
    let definition = content.poses.get(PoseId(pose)).unwrap();
    Samus {
        pose: PoseId(pose),
        facing: definition.facing,
        previous: PoseRecord {
            pose: PoseId(pose),
            facing: definition.facing,
            movement: definition.movement,
        },
        movement: definition.movement,
        motion,
        ..Samus::default()
    }
}

/// Falling, a frame's speeds: a base speed, an extra run speed and a fall.
fn falling() -> State {
    State {
        base_speed: Fixed::new(1, 0x8000),
        extra_run: Fixed::new(0, 0x4000),
        mode: SpeedMode::Decelerating,
        vertical_speed: Fixed::new(4, 0x2000),
        vertical: VerticalMotion::Down,
        ..State::default()
    }
}

/// The pose update's view of a frame in which motion met `contacts` in `surroundings`.
fn frame<'a>(
    samus: &Samus,
    contacts: Contacts,
    surroundings: Surroundings<'a>,
    content: &'a Content,
) -> Collisions<'a> {
    Collisions {
        motion: Motion {
            inputs: crate::motion::inputs(samus, &Controller::default(), ElevatorRide::default()),
            rules: crate::motion::rules(content, crate::session::SpeedCap::Wordwise),
            contacts,
        },
        surroundings,
    }
}

fn landed<'a>(samus: &Samus, content: &'a Content) -> Collisions<'a> {
    let contacts = Contacts {
        vertical: VerticalResult::Landed(LandKind::Grounded),
        ..Contacts::default()
    };
    frame(samus, contacts, Surroundings::default(), content)
}

#[test]
fn a_spin_jump_lands_in_the_spin_landing_with_no_base_speed() {
    let content = content();
    let mut samus = in_pose(0x1A, falling(), &content);
    let collisions = landed(&samus, &content);
    let requests = request(
        &samus,
        collisions.motion.contacts.vertical,
        &Controller::default(),
        &content,
        PoseRequests::default(),
    );
    assert_eq!(
        requests,
        PoseRequests {
            ordinary: Some(PoseId(0xA7)),
            command: Some(PoseCommand::Vertical(VerticalResult::Landed(
                LandKind::Grounded
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
    // The landing stops the base speed and the fall and keeps the extra run speed, which
    // moves her one more frame.
    assert_eq!(
        samus.motion,
        State {
            extra_run: Fixed::new(0, 0x4000),
            ..State::default()
        }
    );
}

#[test]
fn a_fall_lands_by_the_direction_its_pose_aims() {
    let content = content();
    let controller = Controller::default();
    let samus = in_pose(0x29, falling(), &content);
    let landing = |samus: &Samus, controller: &Controller| {
        request(
            samus,
            VerticalResult::Landed(LandKind::Grounded),
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
    let aiming = in_pose(0x6F, falling(), &content);
    assert_eq!(landing(&aiming, &shooting), Some(PoseId(0xE4)));
}

#[test]
fn a_ceiling_keeps_the_pose_and_stops_the_rise() {
    let content = content();
    let rising = State {
        vertical: VerticalMotion::Up,
        ..falling()
    };
    let mut samus = in_pose(0x4D, rising, &content);
    let contacts = Contacts {
        vertical: VerticalResult::HitCeiling,
        ..Contacts::default()
    };
    let collisions = frame(&samus, contacts, Surroundings::default(), &content);
    let requests = request(
        &samus,
        VerticalResult::HitCeiling,
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
        (samus.motion.vertical_speed, samus.motion.vertical),
        (Fixed::default(), VerticalMotion::Down)
    );
}

#[test]
fn walking_off_a_ledge_falls_from_rest() {
    let content = content();
    let samus = in_pose(0x01, State::default(), &content);
    let off = VerticalResult::Falling(FallKind::Airborne);
    let requests = request(
        &samus,
        off,
        &Controller::default(),
        &content,
        PoseRequests::default(),
    );
    assert_eq!(
        requests,
        PoseRequests {
            ordinary: Some(PoseId(0x29)),
            command: Some(PoseCommand::Vertical(off)),
        }
    );
    // A jump already falling keeps the input's request.
    let jumping = in_pose(0x4D, falling(), &content);
    let input = PoseRequests {
        ordinary: Some(PoseId(0x15)),
        command: None,
    };
    let still_falling = VerticalResult::Falling(FallKind::NoChange);
    assert_eq!(
        request(
            &jumping,
            still_falling,
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
    let mut samus = in_pose(0x09, State::default(), &content);
    let contacts = Contacts {
        wall_stop: true,
        ..Contacts::default()
    };
    let collisions = frame(&samus, contacts, Surroundings::default(), &content);
    // No request from input: the wall alone changes the pose.
    change(
        &mut samus,
        PoseRequests::default(),
        collisions,
        &mut Touches::default(),
        &content,
    );
    assert_eq!(samus.pose, PoseId(0x89));
    assert_eq!(samus.movement, MovementType::RanIntoWall);
}

/// Eight blocks across and down, column 2 solid: its left edge is at X $20.
fn wall() -> Vec<Block> {
    let mut blocks = vec![Block::default(); 64];
    for block in blocks.iter_mut().skip(2).step_by(8) {
        block.kind = BlockKind::Solid(Bts(0));
    }
    blocks
}

/// Running at `x`, $40 down, asking for the running pose $0B in `surroundings`: her pose
/// and X afterwards.
fn run_to(x: u16, surroundings: Surroundings<'_>, content: &Content) -> (PoseId, Fixed) {
    let mut samus = in_pose(0x09, State::default(), content);
    samus.motion.position.x = Fixed::new(x, 0);
    samus.motion.position.y = Fixed::new(0x40, 0);
    let requests = PoseRequests {
        ordinary: Some(PoseId(0x0B)),
        command: None,
    };
    let collisions = frame(&samus, Contacts::default(), surroundings, content);
    change(
        &mut samus,
        requests,
        collisions,
        &mut Touches::default(),
        content,
    );
    (samus.pose, samus.motion.position.x)
}

#[test]
fn a_running_pose_moves_her_a_pixel_forward_and_stops_at_a_block() {
    let content = content();
    let blocks = wall();
    let beside = Surroundings {
        room: Room {
            blocks: &blocks,
            width: BlockColumns(8),
            slopes: None,
        },
        ..Surroundings::default()
    };
    // Her right side, X + 5 - 1, is $1E from $1A: the pixel takes her to $1B.
    assert_eq!(
        run_to(0x1A, beside, &content),
        (PoseId(0x0B), Fixed::new(0x1B, 0))
    );
    // From $1B the pixel reaches the block: she stays beside it and the pose she asked for
    // runs into the wall.
    assert_eq!(
        run_to(0x1B, beside, &content),
        (PoseId(0x89), Fixed::new(0x1B, 0xFFFF))
    );
}

#[test]
fn a_solid_enemy_a_pixel_ahead_runs_the_pose_into_the_wall_without_the_move() {
    use crate::collision::solid_enemy::{FIRST, SolidEnemies, one_enemy};
    let content = content();
    // Facing right, her right side is X + 5; the enemy's left edge is $50 - 8 = $48.
    let slots = one_enemy(0x50, 0x40, true);
    let ahead = Surroundings {
        enemies: SolidEnemies {
            interactive: &FIRST,
            slots: &slots,
        },
        ..Surroundings::default()
    };
    // From $42 the pixel's target, $43, is clear of the enemy, and the pixel moves her.
    assert_eq!(
        run_to(0x42, ahead, &content),
        (PoseId(0x0B), Fixed::new(0x43, 0))
    );
    // From $43 she touches it: the pose runs into it and she stays.
    assert_eq!(
        run_to(0x43, ahead, &content),
        (PoseId(0x89), Fixed::new(0x43, 0))
    );
}
