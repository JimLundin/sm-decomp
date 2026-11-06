use data::pose::animation::AnimationDelays;
use data::pose::transition::{Control, Controls, PoseTransitions, Transition};
use data::pose::{MovementType, PoseFacing, PoseId, Poses};

use super::*;
use crate::state::samus::Horizontal;
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
        visor_colours: data::palette::VisorColours::default(),
        face_block_glow: data::enemy::face_block::GlowColours::default(),
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
        facing: PoseFacing::Right,
        previous: PoseRecord {
            pose: PoseId(1),
            facing: PoseFacing::Right,
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
    change(&mut samus, requests, Collisions::default(), &content);
    assert_eq!(samus.pose, PoseId(2));
    assert_eq!(samus.movement.kind(), MovementType::Running);
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
        movement: Movement::from(MovementType::Running),
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
        movement: Movement::Running {
            speed: Horizontal {
                base: Speed::new(1, 0),
                ..Horizontal::default()
            },
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
        movement: Movement::from(MovementType::Running),
        ..standing()
    };
    // Pose 2's definition names no next pose, so she keeps it; the command is a stop.
    assert_eq!(
        input(&samus, &Controller::default(), &content).command,
        Some(PoseCommand::Stop)
    );
}

#[test]
fn letting_go_folds_the_run_speed_into_the_base_speed_and_decelerates() {
    let mut speed = Horizontal {
        base: Speed::new(2, 0xC000),
        extra_run: Speed::new(1, 0x2000),
        acceleration: Acceleration::Accelerating,
        momentum: true,
    };
    apply(PoseCommand::Decelerate, &mut speed);
    assert_eq!(
        speed,
        Horizontal {
            base: Speed::new(3, 0xE000),
            extra_run: Speed::ZERO,
            acceleration: Acceleration::Decelerating,
            momentum: false,
        }
    );
}

#[test]
fn turning_on_the_ground_takes_the_run_speed_into_the_turn() {
    let mut movement = Movement::TurningAround {
        place: Turn::OnGround,
        speed: Horizontal {
            base: Speed::new(2, 0xC000),
            extra_run: Speed::new(0, 0x4000),
            ..Horizontal::default()
        },
    };
    initialise(PoseRecord::default(), Facing::Left, &mut movement);
    assert_eq!(
        movement.horizontal(),
        Some(Horizontal {
            base: Speed::new(3, 0),
            extra_run: Speed::ZERO,
            acceleration: Acceleration::TurningAround,
            momentum: false,
        })
    );
}

#[test]
fn a_crouching_transition_takes_no_input() {
    let content = content();
    let samus = Samus {
        movement: Movement::from(MovementType::CrouchingTransition),
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

/// The log's initial jump speeds (`$90:9EB9` on, the air column).
fn jump_content() -> Content {
    Content {
        vertical: VerticalSpeeds {
            jump: Speed::new(4, 0xE000),
            wall_jump: Speed::new(4, 0xA000),
            ..VerticalSpeeds::default()
        },
        ..Content::default()
    }
}

fn in_jump(pose: u16, jump: Jump, old: PoseRecord) -> Samus {
    Samus {
        pose: PoseId(pose),
        movement: Movement::from(match jump {
            Jump::Normal => MovementType::NormalJumping,
            Jump::Spin => MovementType::SpinJumping,
        }),
        previous: old,
        position: crate::state::samus::Point {
            y: crate::state::samus::Fixed {
                pixels: 0x0100,
                subpixels: 0,
            },
            ..crate::state::samus::Point::default()
        },
        ..Samus::default()
    }
}

const fn rising(speed: Speed) -> Vertical {
    Vertical {
        speed,
        direction: YDirection::Up,
    }
}

#[test]
fn a_take_off_from_a_crouch_starts_the_jump_10_pixels_higher() {
    let crouching = PoseRecord {
        pose: PoseId(0x27),
        movement: MovementType::Crouching,
        ..PoseRecord::default()
    };
    let mut samus = in_jump(0x4B, Jump::Normal, crouching);
    jump_transition(&mut samus, &jump_content());
    assert_eq!(samus.position.y.pixels, 0x00F6);
    assert_eq!(
        samus.movement.vertical(),
        Some(rising(Speed::new(4, 0xE000)))
    );
    // A normal jumping pose past the take-off starts nothing.
    let mut samus = in_jump(0x4D, Jump::Normal, crouching);
    jump_transition(&mut samus, &jump_content());
    assert_eq!(samus.position.y.pixels, 0x0100);
    assert_eq!(samus.movement.vertical(), Some(Vertical::default()));
}

#[test]
fn a_spin_jump_starts_from_the_ground_but_not_from_a_spin_or_wall_jump() {
    let running = PoseRecord {
        movement: MovementType::Running,
        ..PoseRecord::default()
    };
    let mut samus = in_jump(0x19, Jump::Spin, running);
    jump_transition(&mut samus, &jump_content());
    assert_eq!(
        samus.movement.vertical(),
        Some(rising(Speed::new(4, 0xE000)))
    );
    for movement in [MovementType::SpinJumping, MovementType::WallJumping] {
        let old = PoseRecord {
            movement,
            ..PoseRecord::default()
        };
        let mut samus = in_jump(0x19, Jump::Spin, old);
        jump_transition(&mut samus, &jump_content());
        assert_eq!(samus.movement.vertical(), Some(Vertical::default()));
    }
}

#[test]
fn a_spin_jump_turned_around_from_a_spin_turns_its_momentum() {
    let old = PoseRecord {
        facing: Facing::Left,
        movement: MovementType::SpinJumping,
        ..PoseRecord::default()
    };
    let mut movement = Movement::from(MovementType::SpinJumping);
    if let Some(speed) = movement.horizontal_mut() {
        speed.base = Speed::new(1, 0);
        speed.extra_run = Speed::new(0, 0x8000);
        speed.momentum = true;
    }
    initialise(old, Facing::Right, &mut movement);
    assert_eq!(
        movement.horizontal(),
        Some(Horizontal {
            base: Speed::new(1, 0x8000),
            extra_run: Speed::ZERO,
            acceleration: Acceleration::TurningAround,
            momentum: false,
        })
    );
}

#[test]
fn a_normal_jump_with_run_speed_decelerates() {
    let mut movement = Movement::from(MovementType::NormalJumping);
    if let Some(speed) = movement.horizontal_mut() {
        speed.extra_run = Speed::new(0, 0x1000);
    }
    initialise(PoseRecord::default(), Facing::Right, &mut movement);
    assert_eq!(
        movement.horizontal().map(|speed| speed.acceleration),
        Some(Acceleration::Decelerating)
    );
}
