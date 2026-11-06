use data::movement::{ExtraRun, MOVEMENT_TYPES, SpeedTable};
use data::pose::{MovementType, PoseFacing};

use super::*;
use crate::state::slots::FlagSet;
use crate::state::{Button, Controller};

/// The bank $90 log's normal speed table rows for running (1) and turning on the ground
/// ($E), the rest zero, and the air column of the extra run constants.
fn content() -> Content {
    let mut words = vec![0_u16; MOVEMENT_TYPES * 6];
    let rows: [(usize, [u16; 6]); 2] = [
        (1, [0x0000, 0x3000, 0x0002, 0xC000, 0x0000, 0x8000]),
        (0xE, [0x0000, 0xC000, 0x0000, 0x0000, 0x0000, 0x8000]),
    ];
    for (movement, row) in rows {
        if let Some(entry) = words.as_chunks_mut::<6>().0.get_mut(movement) {
            entry.copy_from_slice(&row);
        }
    }
    let bytes: Vec<u8> = words.iter().flat_map(|word| word.to_le_bytes()).collect();
    Content {
        speeds: HorizontalSpeeds {
            normal: SpeedTable::decode(&bytes),
            extra_run: ExtraRun {
                acceleration: Speed::new(0, 0x1000),
                booster_max: Speed::new(7, 0),
                max: Speed::new(2, 0),
            },
        },
        ..Content::default()
    }
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

fn running_left() -> Samus {
    Samus {
        pose: PoseId(0x0A),
        facing: PoseFacing::Left,
        movement: Movement::from(MovementType::Running),
        position: crate::state::samus::Point {
            x: Fixed {
                pixels: 0x0500,
                subpixels: 0,
            },
            ..crate::state::samus::Point::default()
        },
        ..Samus::default()
    }
}

#[test]
fn running_accelerates_from_rest_to_the_cap() {
    let content = content();
    let controller = holding(&[Button::Left, Button::B]);
    let mut samus = running_left();
    // Worked by hand from the table: the base speed gains 0.3000 a frame and the extra run
    // speed 0.1000, and she moves left by their sum.
    run(&mut samus, &controller, &content);
    let speed = samus.movement.horizontal().unwrap();
    assert_eq!(
        (speed.base, speed.extra_run),
        (Speed::new(0, 0x3000), Speed::new(0, 0x1000))
    );
    assert!(speed.momentum);
    assert_eq!(
        (samus.position.x.pixels, samus.position.x.subpixels),
        (0x04FF, 0xC000)
    );
    for _ in 1..10 {
        run(&mut samus, &controller, &content);
    }
    let speed = samus.movement.horizontal().unwrap();
    assert_eq!(speed.base, Speed::new(1, 0xE000));
    // 0.3000 more would be 2.1000: its subpixel word compares above the cap's C000 (the
    // difference is $5000), so the cap holds her at 2.C000 a frame early.
    run(&mut samus, &controller, &content);
    let speed = samus.movement.horizontal().unwrap();
    assert_eq!(speed.base, Speed::new(2, 0xC000));
    assert_eq!(speed.extra_run, Speed::new(0, 0xB000));
    // The extra run speed climbs on to its cap of 2.0 and stays.
    for _ in 0..30 {
        run(&mut samus, &controller, &content);
    }
    let speed = samus.movement.horizontal().unwrap();
    assert_eq!(
        (speed.base, speed.extra_run),
        (Speed::new(2, 0xC000), Speed::new(2, 0))
    );
}

#[test]
fn without_the_run_button_there_is_no_extra_run_speed() {
    let content = content();
    let mut samus = running_left();
    run(&mut samus, &holding(&[Button::Left]), &content);
    let speed = samus.movement.horizontal().unwrap();
    assert_eq!(
        (speed.base, speed.extra_run, speed.momentum),
        (Speed::new(0, 0x3000), Speed::ZERO, false)
    );
}

#[test]
fn decelerating_slows_by_the_table_then_accelerates_again() {
    let content = content();
    let mut samus = Samus {
        movement: Movement::Running {
            speed: Horizontal {
                base: Speed::new(1, 0),
                acceleration: Acceleration::Decelerating,
                ..Horizontal::default()
            },
        },
        ..running_left()
    };
    run(&mut samus, &Controller::default(), &content);
    assert_eq!(
        samus.movement.horizontal().unwrap().base,
        Speed::new(0, 0x8000)
    );
    assert_eq!(samus.position.x.pixels, 0x04FF);
    run(&mut samus, &Controller::default(), &content);
    run(&mut samus, &Controller::default(), &content);
    let speed = samus.movement.horizontal().unwrap();
    assert_eq!(
        (speed.base, speed.acceleration),
        (Speed::ZERO, Acceleration::Accelerating)
    );
}

#[test]
fn a_turn_from_rest_moves_on_alternate_frames_under_its_cap_of_zero() {
    let content = content();
    let mut samus = Samus {
        pose: PoseId(0x25),
        movement: Movement::TurningAround {
            place: Turn::OnGround,
            speed: Horizontal {
                acceleration: Acceleration::TurningAround,
                ..Horizontal::default()
            },
        },
        ..running_left()
    };
    // Turning at rest: the deceleration takes her below zero, so she accelerates again.
    run(&mut samus, &Controller::default(), &content);
    assert_eq!(
        samus.movement.horizontal().unwrap().acceleration,
        Acceleration::Accelerating
    );
    // 0.C000 compares below the cap of 0.0 (the difference is $C000, negative); 1.8000
    // doesn't, so the cap takes her back to zero.
    run(&mut samus, &Controller::default(), &content);
    assert_eq!(
        (samus.position.x.pixels, samus.position.x.subpixels),
        (0x04FF, 0x4000)
    );
    run(&mut samus, &Controller::default(), &content);
    assert_eq!(samus.movement.horizontal().unwrap().base, Speed::ZERO);
    assert_eq!(samus.position.x.subpixels, 0x4000);
}

#[test]
fn standing_spends_the_extra_run_speed_then_stops() {
    let content = content();
    let mut samus = Samus {
        pose: PoseId(0xA5),
        movement: Movement::Standing {
            speed: Horizontal {
                extra_run: Speed::new(1, 0x2000),
                momentum: true,
                ..Horizontal::default()
            },
        },
        ..running_left()
    };
    run(&mut samus, &Controller::default(), &content);
    assert_eq!(
        (samus.position.x.pixels, samus.position.x.subpixels),
        (0x04FE, 0xE000)
    );
    assert_eq!(samus.movement.horizontal(), Some(Horizontal::default()));
}

#[test]
fn the_displacement_is_held_to_fifteen_pixels() {
    assert_eq!(
        clamp(Speed::new(0x0020, 0x1234).0),
        Speed::new(0x000F, 0x1234).0
    );
    assert_eq!(
        clamp(Speed::new(0xFFE0, 0x1234).0),
        Speed::new(0xFFF1, 0x1234).0
    );
    assert_eq!(clamp(Speed::new(0x0003, 0).0), Speed::new(0x0003, 0).0);
}
