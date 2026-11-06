use data::movement::{ExtraRun, MOVEMENT_TYPES, SpeedTable};
use data::pose::{MovementType, PoseFacing};

use super::*;
use crate::collision::{Surroundings, Touch, Touches};
use crate::state::samus::Fixed;
use crate::state::slots::FlagSet;
use crate::state::{Button, Controller};

/// The bank $90 log's normal speed table rows for running (1), the morph ball on the ground (4) and turning on the ground
/// ($E), the rest zero, and the air column of the extra run constants.
fn content() -> Content {
    let mut words = vec![0_u16; MOVEMENT_TYPES * 6];
    let rows: [(usize, [u16; 6]); 3] = [
        (1, [0x0000, 0x3000, 0x0002, 0xC000, 0x0000, 0x8000]),
        (4, [0x0000, 0x2000, 0x0001, 0x8000, 0x0000, 0x4000]),
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
    run(
        &mut samus,
        &controller,
        &content,
        Surroundings::default(),
        &mut Touches::default(),
    );
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
        run(
            &mut samus,
            &controller,
            &content,
            Surroundings::default(),
            &mut Touches::default(),
        );
    }
    let speed = samus.movement.horizontal().unwrap();
    assert_eq!(speed.base, Speed::new(1, 0xE000));
    // 0.3000 more would be 2.1000: its subpixel word compares above the cap's C000 (the
    // difference is $5000), so the cap holds her at 2.C000 a frame early.
    run(
        &mut samus,
        &controller,
        &content,
        Surroundings::default(),
        &mut Touches::default(),
    );
    let speed = samus.movement.horizontal().unwrap();
    assert_eq!(speed.base, Speed::new(2, 0xC000));
    assert_eq!(speed.extra_run, Speed::new(0, 0xB000));
    // The extra run speed climbs on to its cap of 2.0 and stays.
    for _ in 0..30 {
        run(
            &mut samus,
            &controller,
            &content,
            Surroundings::default(),
            &mut Touches::default(),
        );
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
    run(
        &mut samus,
        &holding(&[Button::Left]),
        &content,
        Surroundings::default(),
        &mut Touches::default(),
    );
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
    run(
        &mut samus,
        &Controller::default(),
        &content,
        Surroundings::default(),
        &mut Touches::default(),
    );
    assert_eq!(
        samus.movement.horizontal().unwrap().base,
        Speed::new(0, 0x8000)
    );
    assert_eq!(samus.position.x.pixels, 0x04FF);
    run(
        &mut samus,
        &Controller::default(),
        &content,
        Surroundings::default(),
        &mut Touches::default(),
    );
    run(
        &mut samus,
        &Controller::default(),
        &content,
        Surroundings::default(),
        &mut Touches::default(),
    );
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
    run(
        &mut samus,
        &Controller::default(),
        &content,
        Surroundings::default(),
        &mut Touches::default(),
    );
    assert_eq!(
        samus.movement.horizontal().unwrap().acceleration,
        Acceleration::Accelerating
    );
    // 0.C000 compares below the cap of 0.0 (the difference is $C000, negative); 1.8000
    // doesn't, so the cap takes her back to zero.
    run(
        &mut samus,
        &Controller::default(),
        &content,
        Surroundings::default(),
        &mut Touches::default(),
    );
    assert_eq!(
        (samus.position.x.pixels, samus.position.x.subpixels),
        (0x04FF, 0x4000)
    );
    run(
        &mut samus,
        &Controller::default(),
        &content,
        Surroundings::default(),
        &mut Touches::default(),
    );
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
    run(
        &mut samus,
        &Controller::default(),
        &content,
        Surroundings::default(),
        &mut Touches::default(),
    );
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

#[test]
fn facing_forward_on_a_ride_steps_down_a_pixel_and_not_without_one() {
    let mut ride = ElevatorRide {
        status: ElevatorStatus::EnteringRoom,
        ..ElevatorRide::default()
    };
    let mut samus = Samus {
        pose: PoseId(0),
        movement: Movement::Standing {
            speed: Horizontal::default(),
        },
        ..Samus::default()
    };
    samus.position.y = Fixed {
        pixels: 0x02,
        subpixels: 0,
    };
    super::ride(
        &mut samus,
        ride,
        Surroundings::default(),
        &mut Touches::default(),
    );
    assert_eq!(samus.position.y.pixels, 0x03);
    ride.status = ElevatorStatus::Inactive;
    super::ride(
        &mut samus,
        ride,
        Surroundings::default(),
        &mut Touches::default(),
    );
    assert_eq!(samus.position.y.pixels, 0x03);
}

/// A level eight blocks across of air, with `solid` blocks at their (column, row).
fn level(solid: &[(u16, u16)]) -> Vec<crate::state::room::Block> {
    use data::level::{BlockKind, Bts};
    let mut blocks = vec![crate::state::room::Block::default(); 64];
    for &(column, row) in solid {
        if let Some(block) = blocks.get_mut(usize::from(row.wrapping_mul(8).wrapping_add(column))) {
            block.kind = BlockKind::Solid(Bts(0));
        }
    }
    blocks
}

fn surroundings(blocks: &[crate::state::room::Block]) -> Surroundings<'_> {
    Surroundings {
        room: crate::collision::Room {
            blocks,
            width: crate::state::room::BlockColumns(8),
            slopes: None,
        },
        ..Surroundings::default()
    }
}

#[test]
fn running_into_a_wall_stops_her_and_kills_her_speed() {
    let content = content();
    let controller = holding(&[Button::Left, Button::B]);
    let mut samus = running_left();
    samus.position.x = Fixed {
        pixels: 0x0015,
        subpixels: 0,
    };
    samus.position.y.pixels = 0x0008;
    if let Some(speed) = samus.movement.horizontal_mut() {
        speed.base = Speed::new(1, 0);
    }
    // The wall's right edge is 15; her left side, 21 - 5, may come to 16 and no further.
    let blocks = level(&[(0, 0)]);
    let contacts = run(
        &mut samus,
        &controller,
        &content,
        surroundings(&blocks),
        &mut Touches::default(),
    );
    assert_eq!(contacts.wall, Some(crate::collision::Side::Left));
    assert_eq!(
        (samus.position.x.pixels, samus.position.x.subpixels),
        (0x0015, 0)
    );
    assert_eq!(samus.movement.horizontal(), Some(Horizontal::default()));
}

#[test]
fn a_ride_steps_with_the_radius_she_called_it_with() {
    // `area-brinstar` frame 314: the ride keeps the standing radius 15h ($0B00), not the
    // facing-forward pose's 18h, so 3 pixels above the floor the step still goes through.
    let ride = ElevatorRide {
        status: ElevatorStatus::EnteringRoom,
        ..ElevatorRide::default()
    };
    let blocks = level(&[(1, 7)]);
    let step = |y_radius| {
        let mut samus = Samus {
            pose: PoseId(0),
            movement: Movement::Standing {
                speed: Horizontal::default(),
            },
            y_radius,
            ..Samus::default()
        };
        samus.position.x.pixels = 0x0018;
        samus.position.y.pixels = 0x0058;
        super::ride(
            &mut samus,
            ride,
            surroundings(&blocks),
            &mut Touches::default(),
        );
        (samus.position.y.pixels, samus.position.y.subpixels)
    };
    assert_eq!(step(0x15), (0x0059, 0));
    assert_eq!(step(0x18), (0x0058, 0xFFFF));
}

#[test]
fn a_ride_stops_its_step_on_a_block_under_her() {
    let ride = ElevatorRide {
        status: ElevatorStatus::EnteringRoom,
        ..ElevatorRide::default()
    };
    let mut samus = Samus {
        pose: PoseId(0),
        movement: Movement::Standing {
            speed: Horizontal::default(),
        },
        ..Samus::default()
    };
    // With no Y radius her bottom is her Y less one: at 0x10 she stands on the floor at 0x10.
    samus.position.x.pixels = 0x0018;
    samus.position.y.pixels = 0x0010;
    let blocks = level(&[(1, 1)]);
    super::ride(
        &mut samus,
        ride,
        surroundings(&blocks),
        &mut Touches::default(),
    );
    assert_eq!(
        (samus.position.y.pixels, samus.position.y.subpixels),
        (0x0010, 0xFFFF)
    );
}

#[test]
fn a_ride_step_into_a_scroll_trigger_changes_the_scroll_area_on_that_frames_plm_pass() {
    use data::level::ScrollArea;
    use data::plm::{PlmEntry, PlmKind, PlmPopulation, ScrollChange, ScrollChangesId};
    use data::rom::RomAddress;

    use crate::state::room::{BlockColumns, BlockIndex, RoomScoped};

    // `area-brinstar` frame 92: the ride's step reaches a scroll PLM's trigger under her.
    let population = PlmPopulation {
        entries: vec![PlmEntry {
            kind: PlmKind::Scroll(ScrollChangesId(0)),
            x: 1,
            y: 4,
        }],
        scroll_changes: vec![(
            RomAddress::new(0x8f, 0x9f0b),
            vec![ScrollChange {
                screen: 0,
                area: ScrollArea::Green,
            }],
        )],
    };
    let mut room = RoomScoped {
        blocks: level(&[]),
        width: BlockColumns(8),
        scroll_areas: vec![ScrollArea::Red],
        ..RoomScoped::default()
    };
    crate::plm::spawn_population(&mut room, &population);
    let ride = ElevatorRide {
        status: ElevatorStatus::EnteringRoom,
        ..ElevatorRide::default()
    };
    let mut samus = Samus {
        pose: PoseId(0),
        movement: Movement::Standing {
            speed: Horizontal::default(),
        },
        y_radius: 0x15,
        ..Samus::default()
    };
    samus.position.x.pixels = 0x0018;
    // Her bottom, 2Bh + 15h - 1, is the pixel above the trigger's row.
    samus.position.y.pixels = 0x002B;
    let mut touches = Touches::default();
    super::ride(&mut samus, ride, surroundings(&room.blocks), &mut touches);
    assert_eq!(samus.position.y.pixels, 0x002C);
    assert_eq!(touches.touched(), [Touch::Scroll(BlockIndex(4 * 8 + 1))]);

    touches.apply(&mut room).unwrap();
    assert_eq!(room.scroll_areas, [ScrollArea::Red]);
    crate::plm::process_unseen(
        &mut room,
        &mut crate::state::CollectedItems::default(),
        &population,
    );
    assert_eq!(room.scroll_areas, [ScrollArea::Green]);
}

#[test]
fn running_into_a_solid_enemy_cuts_the_move_and_kills_her_speed() {
    use crate::collision::solid_enemy::{FIRST, SolidEnemies, one_enemy};
    let content = content();
    let controller = holding(&[Button::Left, Button::B]);
    // The enemy's right edge, $4F0 + 8 = $4F8, is 3 pixels from her left side at $500 - 5.
    let slots = one_enemy(0x04F0, 0x0010, true);
    let surroundings = Surroundings {
        enemies: SolidEnemies {
            interactive: &FIRST,
            slots: &slots,
        },
        ..Surroundings::default()
    };
    let move_from = |x: u16, y_subpixels: u16| {
        let mut samus = running_left();
        samus.position.x.pixels = x;
        samus.position.y = Fixed {
            pixels: 0x0010,
            subpixels: y_subpixels,
        };
        if let Some(speed) = samus.movement.horizontal_mut() {
            speed.base = Speed::new(5, 0);
        }
        let contacts = run(
            &mut samus,
            &controller,
            &content,
            surroundings,
            &mut Touches::default(),
        );
        (contacts.wall, samus)
    };
    let (wall, samus) = move_from(0x0500, 0x8000);
    assert_eq!(wall, Some(crate::collision::Side::Left));
    assert_eq!(
        (samus.position.x.pixels, samus.position.x.subpixels),
        (0x04FD, 0)
    );
    // The cut leaves her Y subpixels; the step down after it, her total X speed (the capped
    // 2.C000 and the extra 0.1000) and a pixel, carries them from $8000 to $5000.
    assert_eq!(samus.position.y.subpixels, 0x5000);
    assert_eq!(samus.movement.horizontal(), Some(Horizontal::default()));
    // Already touching it she stays, and the touch zeroes her Y subpixels before the step.
    let (wall, samus) = move_from(0x04FD, 0x8000);
    assert_eq!(wall, Some(crate::collision::Side::Left));
    assert_eq!(samus.position.x.pixels, 0x04FD);
    assert_eq!(samus.position.y.subpixels, 0xD000);
}

#[test]
fn turning_on_the_ground_steps_down_by_its_x_speed_plus_a_pixel_and_reports_nothing() {
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
    samus.position.y.pixels = 0x02A8;
    let turn = |samus: &mut Samus| {
        run(
            samus,
            &Controller::default(),
            &content,
            Surroundings::default(),
            &mut Touches::default(),
        )
    };
    // At rest her total X speed is 0.0: a pixel down, and the turn clears the result
    // (`$90:A68F`).
    let contacts = turn(&mut samus);
    assert_eq!(contacts.vertical, VerticalContact::NoChange);
    assert_eq!(
        (samus.position.y.pixels, samus.position.y.subpixels),
        (0x02A9, 0)
    );
    // Then 0.C000 sideways and 1.C000 down.
    turn(&mut samus);
    assert_eq!(
        (samus.position.x.pixels, samus.position.x.subpixels),
        (0x04FF, 0x4000)
    );
    assert_eq!(
        (samus.position.y.pixels, samus.position.y.subpixels),
        (0x02AA, 0xC000)
    );
}

#[test]
fn standing_keeps_to_the_floor_and_falls_off_its_edge() {
    let content = content();
    let mut samus = Samus {
        pose: PoseId(0xA5),
        movement: Movement::Standing {
            speed: Horizontal::default(),
        },
        y_radius: 0x15,
        ..running_left()
    };
    samus.position.x.pixels = 0x0018;
    // Her bottom, 43 + 21 - 1, is on the floor's top, 64.
    samus.position.y = Fixed {
        pixels: 43,
        subpixels: 0xFFFF,
    };
    let floor = level(&[(0, 4), (1, 4), (2, 4)]);
    let contacts = run(
        &mut samus,
        &Controller::default(),
        &content,
        surroundings(&floor),
        &mut Touches::default(),
    );
    assert_eq!(contacts.vertical, VerticalContact::Landed);
    assert_eq!(
        (samus.position.y.pixels, samus.position.y.subpixels),
        (43, 0xFFFF)
    );
    let contacts = run(
        &mut samus,
        &Controller::default(),
        &content,
        Surroundings::default(),
        &mut Touches::default(),
    );
    assert_eq!(contacts.vertical, VerticalContact::Falling);
    assert_eq!(
        (samus.position.y.pixels, samus.position.y.subpixels),
        (44, 0xFFFF)
    );
}

#[test]
fn after_a_slope_the_step_down_is_a_pixel_alone_and_clears_the_flag() {
    let content = content();
    let step = |slope_adjusted| {
        let mut samus = Samus {
            slope_adjusted,
            ..running_left()
        };
        if let Some(speed) = samus.movement.horizontal_mut() {
            speed.base = Speed::new(1, 0);
        }
        run(
            &mut samus,
            &Controller::default(),
            &content,
            Surroundings::default(),
            &mut Touches::default(),
        );
        (samus.position.y, samus.slope_adjusted)
    };
    // The base speed accelerates to 1.3000 first: the step is 2.3000 without the flag.
    let without = Fixed {
        pixels: 2,
        subpixels: 0x3000,
    };
    assert_eq!(step(false), (without, false));
    let alone = Fixed {
        pixels: 1,
        subpixels: 0,
    };
    assert_eq!(step(true), (alone, false));
}

fn ball(pose: u16, facing: PoseFacing, speed: Horizontal) -> Samus {
    Samus {
        pose: PoseId(pose),
        facing,
        movement: Movement::MorphBall {
            speed,
            airborne: false,
        },
        ..running_left()
    }
}

#[test]
fn a_ball_in_motion_moves_like_a_runner_by_the_ball_row() {
    let content = content();
    // Pose $1E is the rolling ball, not the resting $1D.
    let mut samus = ball(0x1E, PoseFacing::Right, Horizontal::default());
    run(
        &mut samus,
        &Controller::default(),
        &content,
        Surroundings::default(),
        &mut Touches::default(),
    );
    let speed = samus.movement.horizontal().unwrap();
    // It gains the row's 0.2000 and moves right by it; the resting ball's speeds do not end.
    assert_eq!(speed.base, Speed::new(0, 0x2000));
    assert_eq!(
        (samus.position.x.pixels, samus.position.x.subpixels),
        (0x0500, 0x2000)
    );
}

#[test]
fn a_ball_at_rest_spends_its_extra_run_speed_then_ends_its_speeds() {
    let content = content();
    let moving = Horizontal {
        extra_run: Speed::new(1, 0x2000),
        momentum: true,
        ..Horizontal::default()
    };
    // Resting in the pose for the way it faces: right in $1D, left in $41.
    for (pose, facing) in [(0x1D, PoseFacing::Right), (0x41, PoseFacing::Left)] {
        let mut samus = ball(pose, facing, moving);
        run(
            &mut samus,
            &Controller::default(),
            &content,
            Surroundings::default(),
            &mut Touches::default(),
        );
        assert_eq!(samus.movement.horizontal(), Some(Horizontal::default()));
    }
    // The wrong pose for its facing is not at rest, so it keeps accelerating.
    let mut samus = ball(0x41, PoseFacing::Right, Horizontal::default());
    run(
        &mut samus,
        &Controller::default(),
        &content,
        Surroundings::default(),
        &mut Touches::default(),
    );
    let speed = samus.movement.horizontal().unwrap();
    assert_eq!(speed.base, Speed::new(0, 0x2000));
}

#[test]
fn a_decelerating_ball_in_its_resting_pose_still_decelerates() {
    let content = content();
    let mut samus = ball(
        0x1D,
        PoseFacing::Right,
        Horizontal {
            base: Speed::new(0, 0x5000),
            acceleration: Acceleration::Decelerating,
            ..Horizontal::default()
        },
    );
    run(
        &mut samus,
        &Controller::default(),
        &content,
        Surroundings::default(),
        &mut Touches::default(),
    );
    let speed = samus.movement.horizontal().unwrap();
    assert_eq!(speed.base, Speed::new(0, 0x1000));
}
