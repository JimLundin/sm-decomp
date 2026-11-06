//! The air movements against values worked by hand from the bank $90 log's constants.

use data::movement::{MOVEMENT_TYPES, SpeedTable};
use data::pose::MovementType;

use super::*;
use crate::movement::run;
use crate::state::samus::Point;
use crate::state::slots::FlagSet;

/// The log's air column (`$90:9EA1`, `$90:9EB9` on) and a spin jump row of acceleration
/// 0.1000 and cap 1.0000 (a made-up row, the rest zero).
fn content() -> Content {
    let mut words = vec![0_u16; MOVEMENT_TYPES * 6];
    if let Some(entry) = words.as_chunks_mut::<6>().0.get_mut(3) {
        entry.copy_from_slice(&[0x0000, 0x1000, 0x0001, 0x0000, 0x0000, 0x8000]);
    }
    let bytes: Vec<u8> = words.iter().flat_map(|word| word.to_le_bytes()).collect();
    let mut content = Content {
        vertical: VerticalSpeeds {
            gravity: Speed::new(0, 0x1C00),
            jump: Speed::new(4, 0xE000),
            hi_jump: Speed::new(6, 0),
            wall_jump: Speed::new(4, 0xA000),
            wall_hi_jump: Speed::new(5, 0x8000),
        },
        ..Content::default()
    };
    content.speeds.normal = SpeedTable::decode(&bytes);
    content
}

fn holding(buttons: &[Button]) -> Controller {
    Controller {
        held: FlagSet {
            bits: buttons.iter().fold(0, |bits, &button| bits | button.bit()),
            ..FlagSet::default()
        },
        ..Controller::default()
    }
}

const fn at(pixels: u16, subpixels: u16) -> Fixed {
    Fixed { pixels, subpixels }
}

/// Facing right in a normal jump past its take-off (pose $4D), at `speed` vertically.
fn jumping(speed: Speed, direction: YDirection) -> Samus {
    Samus {
        pose: PoseId(0x4D),
        facing: Facing::Right,
        movement: Movement::Jumping {
            jump: Jump::Normal,
            speed: Horizontal::default(),
            vertical: Vertical { speed, direction },
        },
        position: Point {
            x: at(0x0080, 0),
            y: at(0x0100, 0),
        },
        ..Samus::default()
    }
}

fn vertical(samus: &Samus) -> Vertical {
    samus.movement.vertical().unwrap_or_default()
}

fn equipping(item: Item) -> crate::state::samus::Items {
    FlagSet {
        bits: item.bit(),
        ..FlagSet::default()
    }
}

#[test]
fn a_jump_takes_its_initial_speed_from_the_items() {
    let speeds = content().vertical;
    let mut samus = jumping(Speed::ZERO, YDirection::None);
    make_jump(&mut samus, Takeoff::Jump, &speeds);
    assert_eq!(
        vertical(&samus),
        Vertical {
            speed: Speed::new(4, 0xE000),
            direction: YDirection::Up,
        }
    );
    samus.equipment.equipped = equipping(Item::HiJumpBoots);
    make_jump(&mut samus, Takeoff::WallJump, &speeds);
    assert_eq!(vertical(&samus).speed, Speed::new(5, 0x8000));
}

#[test]
fn the_speed_booster_adds_half_the_run_speed_without_a_carry() {
    let speeds = content().vertical;
    let mut samus = jumping(Speed::ZERO, YDirection::None);
    samus.equipment.equipped = equipping(Item::SpeedBooster);
    if let Some(speed) = samus.movement.horizontal_mut() {
        speed.extra_run = Speed::new(3, 0x2000);
    }
    make_jump(&mut samus, Takeoff::Jump, &speeds);
    // Pixels 4 + 3 / 2 = 5; subpixels E000 + 2000 wrap to 0000 and carry nothing.
    assert_eq!(vertical(&samus).speed, Speed::new(5, 0));
}

#[test]
fn a_rise_moves_by_the_frames_speed_then_gravity_slows_it() {
    let content = content();
    let jump = holding(&[Button::A]);
    let mut samus = jumping(Speed::new(4, 0xE000), YDirection::Up);
    run(&mut samus, &jump, &content);
    // $100.0000 - 4.E000 = $FB.2000; 4.E000 - 0.1C00 = 4.C400.
    assert_eq!(samus.position.y, at(0x00FB, 0x2000));
    assert_eq!(vertical(&samus).speed, Speed::new(4, 0xC400));
    run(&mut samus, &jump, &content);
    assert_eq!(samus.position.y, at(0x00F6, 0x5C00));
    assert_eq!(vertical(&samus).speed, Speed::new(4, 0xA800));
}

#[test]
fn letting_go_of_jump_ends_the_rise() {
    let content = content();
    let mut samus = jumping(Speed::new(4, 0xE000), YDirection::Up);
    run(&mut samus, &Controller::default(), &content);
    // She falls from rest: no move this frame, then gravity's 0.1C00.
    assert_eq!(samus.position.y, at(0x0100, 0));
    assert_eq!(
        vertical(&samus),
        Vertical {
            speed: Speed::new(0, 0x1C00),
            direction: YDirection::Down,
        }
    );
    run(&mut samus, &Controller::default(), &content);
    assert_eq!(samus.position.y, at(0x0100, 0x1C00));
}

#[test]
fn the_rise_ends_at_its_apex() {
    let content = content();
    let jump = holding(&[Button::A]);
    let mut samus = jumping(Speed::new(0, 0x1000), YDirection::Up);
    run(&mut samus, &jump, &content);
    // 0.1000 - 0.1C00 goes below zero for the rest of the frame...
    assert_eq!(vertical(&samus).speed, Speed(0xFFFF_F400));
    assert_eq!(samus.position.y, at(0x00FF, 0xF000));
    // ... and the next frame's check starts the fall from rest.
    run(&mut samus, &jump, &content);
    assert_eq!(
        vertical(&samus),
        Vertical {
            speed: Speed::new(0, 0x1C00),
            direction: YDirection::Down,
        }
    );
}

#[test]
fn gravity_stops_adding_once_the_pixel_word_is_5() {
    let content = content();
    let mut samus = Samus {
        pose: PoseId(0x29),
        movement: Movement::Falling {
            speed: Horizontal::default(),
            vertical: Vertical {
                speed: Speed::new(4, 0xF000),
                direction: YDirection::Down,
            },
        },
        ..jumping(Speed::ZERO, YDirection::None)
    };
    run(&mut samus, &Controller::default(), &content);
    // 4.F000 + 0.1C00 = 5.0C00: past 5, as the original tests the pixel word alone.
    assert_eq!(samus.position.y, at(0x0104, 0xF000));
    assert_eq!(vertical(&samus).speed, Speed::new(5, 0x0C00));
    // At terminal speed the falling pose's animation goes to its frame 5.
    assert_eq!(samus.animation.frame, TERMINAL_FRAME);
    assert_eq!(samus.animation.timer, TERMINAL_DELAY);
    run(&mut samus, &Controller::default(), &content);
    assert_eq!(vertical(&samus).speed, Speed::new(5, 0x0C00));
    assert_eq!(samus.position.y, at(0x0109, 0xFC00));
}

/// Facing right in a spin jump (pose $19), with no vertical direction, at base speed `base`.
fn spinning(base: Speed) -> Samus {
    Samus {
        pose: PoseId(0x19),
        movement: Movement::Jumping {
            jump: Jump::Spin,
            speed: Horizontal {
                base,
                ..Horizontal::default()
            },
            vertical: Vertical::default(),
        },
        ..jumping(Speed::ZERO, YDirection::None)
    }
}

#[test]
fn a_spin_jump_keeps_its_speed_only_forwards_or_at_the_cap() {
    let content = content();
    // Not steering: she loses her base speed and stays where she is.
    let mut samus = spinning(Speed::new(0, 0x8000));
    run(&mut samus, &Controller::default(), &content);
    let speed = samus.movement.horizontal().unwrap_or_default();
    assert_eq!(
        (speed.base, speed.acceleration),
        (Speed::ZERO, Acceleration::Accelerating)
    );
    assert_eq!(samus.position.x, at(0x0080, 0));
    // Steering forwards: 0.8000 + 0.1000, and the mode turns to decelerating.
    let mut samus = spinning(Speed::new(0, 0x8000));
    run(&mut samus, &holding(&[Button::Right]), &content);
    let speed = samus.movement.horizontal().unwrap_or_default();
    assert_eq!(
        (speed.base, speed.acceleration),
        (Speed::new(0, 0x9000), Acceleration::Decelerating)
    );
    assert_eq!(samus.position.x, at(0x0080, 0x9000));
    // At the cap she keeps moving without steering: 0.F800 + 0.1000 caps to 1.0000.
    let mut samus = spinning(Speed::new(0, 0xF800));
    run(&mut samus, &Controller::default(), &content);
    assert_eq!(samus.position.x, at(0x0081, 0));
    // With no direction the rise rule runs: it moves her by nothing and gravity takes from
    // it (the moonfall glitch's start).
    assert_eq!(vertical(&samus).speed, Speed(0xFFFF_E400));
}

#[test]
fn the_take_off_moves_her_by_her_run_speed_alone() {
    let content = content();
    let mut samus = Samus {
        pose: PoseId(0x4B),
        ..jumping(Speed::new(4, 0xE000), YDirection::Up)
    };
    if let Some(speed) = samus.movement.horizontal_mut() {
        speed.base = Speed::new(1, 0);
        speed.extra_run = Speed::new(0, 0x4000);
        speed.momentum = true;
    }
    run(&mut samus, &holding(&[Button::A]), &content);
    assert_eq!(
        samus.position,
        Point {
            x: at(0x0080, 0x4000),
            y: at(0x0100, 0),
        }
    );
    assert_eq!(vertical(&samus).speed, Speed::new(4, 0xE000));
    assert_eq!(samus.movement.kind(), MovementType::NormalJumping);
}
