use std::marker::PhantomData;

use super::{Contact, Direction, Move, Obstacle, detect, target};
use crate::collision::Body;
use crate::state::room::{EnemyCommon, EnemyProperty};
use crate::state::samus::{Fixed, Point, Speed};
use crate::state::slots::{Frames, SlotIndex};

/// Samus's X radius in every pose.
const X_RADIUS: u16 = 5;

fn point(x: u16, y: u16, y_subpixels: u16) -> Point {
    Point {
        x: Fixed {
            pixels: x,
            subpixels: 0,
        },
        y: Fixed {
            pixels: y,
            subpixels: y_subpixels,
        },
    }
}

fn body(y_radius: u16) -> Body {
    Body {
        x_radius: X_RADIUS,
        y_radius,
        still_vertically: false,
    }
}

fn slot(index: u8) -> SlotIndex<crate::state::room::Enemy> {
    SlotIndex(index, PhantomData)
}

/// A solid enemy at (`x`, `y`) with radii 8 by 8, as the Blue Brinstar face block is.
fn block(x: u16, y: u16) -> Obstacle {
    Obstacle {
        position: point(x, y, 0),
        radius_x: 8,
        radius_y: 8,
        solid: true,
        frozen: false,
    }
}

/// The face block `item-morph-ball` lands Samus on at frame 797, in slot 5.
fn face_block() -> (SlotIndex<crate::state::room::Enemy>, Obstacle) {
    (slot(5), block(0x488, 0x2B8))
}

/// About 4.98 pixels a frame, her fall at frame 797.
const FALL: Speed = Speed::new(4, 0xFAE1);

fn falling(y: u16, y_radius: u16) -> Move {
    Move {
        direction: Direction::Down,
        position: point(0x494, y, 0),
        body: body(y_radius),
        distance: FALL,
    }
}

/// Where a downward move cut by `contact` leaves her Y pixel.
fn landed(y: u16, contact: Contact) -> Option<u16> {
    contact
        .distance()
        .map(|distance| y.wrapping_add(distance.pixels()))
}

#[test]
fn frame_797_fall_is_cut_to_the_face_blocks_top() {
    // Target Y $299 + 4 + 1 = $29E; |$2B8 - $29E| - 8 = $12 < $13 overlaps. The gap from her
    // bottom ($299 + $13) to the block's top ($2B0) is 4.
    let contact = detect(&falling(0x299, 0x13), [face_block()]);
    assert_eq!(
        contact,
        Contact::Blocked {
            slot: slot(5),
            distance: 4
        }
    );
    // Her bottom boundary, $29D + $13 - 1, rests one pixel above the block's top.
    assert_eq!(landed(0x299, contact), Some(0x29D));
}

#[test]
fn the_oracles_resting_y_is_the_one_a_y_radius_of_0x15_reaches() {
    // The oracle rests her at Y $29B, whose bottom boundary is $2AF only with a radius of $15.
    // open: whether her landing pose takes the radius from $13 to $15 and lifts her by the
    // difference, or the frame-797 radius was $15 all along, the wiring settles.
    let contact = detect(&falling(0x297, 0x15), [face_block()]);
    assert_eq!(landed(0x297, contact), Some(0x29B));
}

#[test]
fn touching_the_block_cuts_the_move_to_nothing() {
    let contact = detect(&falling(0x29D, 0x13), [face_block()]);
    assert_eq!(contact, Contact::Touching { slot: slot(5) });
    assert_eq!(contact.distance(), Some(Speed::ZERO));
    assert_eq!(contact.slot(), Some(slot(5)));
}

#[test]
fn a_fall_that_stops_short_of_the_block_is_clear() {
    // Target $28A + 5 = $28F: |$2B8 - $28F| - 8 = $21, not under $13.
    assert_eq!(
        detect(&falling(0x28A, 0x13), [face_block()]),
        Contact::Clear
    );
}

#[test]
fn an_enemy_off_to_the_side_is_clear() {
    // |$488 - $4A0| - 8 = $10, not under her X radius.
    let mut movement = falling(0x299, 0x13);
    movement.position.x.pixels = 0x4A0;
    assert_eq!(detect(&movement, [face_block()]), Contact::Clear);
}

#[test]
fn no_interactive_enemies_is_clear() {
    assert_eq!(detect(&falling(0x299, 0x13), []), Contact::Clear);
}

#[test]
fn only_a_solid_or_frozen_enemy_stops_her() {
    let (index, solid) = face_block();
    let passable = Obstacle {
        solid: false,
        ..solid
    };
    assert_eq!(
        detect(&falling(0x299, 0x13), [(index, passable)]),
        Contact::Clear
    );
    let frozen = Obstacle {
        frozen: true,
        ..passable
    };
    assert!(matches!(
        detect(&falling(0x299, 0x13), [(index, frozen)]),
        Contact::Blocked { distance: 4, .. }
    ));
}

#[test]
fn an_enemy_she_is_already_inside_gives_way_to_the_next() {
    // The first block's top ($2A0) is above her bottom ($2AC): a negative gap.
    let inside = (slot(1), block(0x488, 0x2A8));
    let contact = detect(&falling(0x299, 0x13), [inside, face_block()]);
    assert_eq!(contact.slot(), Some(slot(5)));
}

#[test]
fn the_first_listed_enemy_in_the_way_decides() {
    let other = (slot(2), block(0x490, 0x2B7));
    let contact = detect(&falling(0x299, 0x13), [other, face_block()]);
    assert_eq!(
        contact,
        Contact::Blocked {
            slot: slot(2),
            distance: 3
        }
    );
}

#[test]
fn the_obstacle_reads_the_slots_flags() {
    let mut enemy = EnemyCommon::default();
    assert!(!Obstacle::from(&enemy).solid);
    enemy.properties.insert(EnemyProperty::SolidToSamus);
    enemy.frozen = Frames(3);
    let obstacle = Obstacle::from(&enemy);
    assert!(obstacle.solid && obstacle.frozen);
}

#[test]
fn the_target_takes_an_extra_pixel_with_any_fraction() {
    // Down, `$A0:A993`: whole pixels and no subpixels reach exactly.
    assert_eq!(target(0x100, 0, Speed::new(3, 0), true), 0x103);
    // A fraction in the distance or her position adds a pixel.
    assert_eq!(target(0x100, 0, Speed::new(3, 1), true), 0x104);
    assert_eq!(target(0x100, 0x8000, Speed::new(3, 0), true), 0x104);
    // A carry steps the pixel, and the stepped word decides: nonzero, so one more.
    assert_eq!(target(0x100, 0x8000, Speed::new(3, 0x8000), true), 0x105);
    // Wrapping to 0 on the carry skips the extra pixel.
    assert_eq!(target(0xFFFC, 0x8000, Speed::new(3, 0x8000), true), 0);
}

#[test]
fn the_target_backwards_mirrors_it() {
    // Left or up, `$A0:A919`: no borrow and no fraction reaches exactly.
    assert_eq!(target(0x100, 0x4000, Speed::new(3, 0x4000), false), 0xFD);
    assert_eq!(target(0x100, 0x4000, Speed::new(3, 0), false), 0xFC);
    // A borrow steps back once, then the nonzero word once more.
    assert_eq!(target(0x100, 0, Speed::new(3, 1), false), 0xFB);
    // Borrowing to 0 skips the extra pixel.
    assert_eq!(target(4, 0, Speed::new(3, 1), false), 0);
}

#[test]
fn horizontal_and_upward_moves_measure_from_their_leading_edges() {
    let enemy = (slot(7), block(0x120, 0x100));
    // Right: her right edge $10A + 5 to the enemy's left $118, a gap of 9.
    let right = Move {
        direction: Direction::Right,
        position: point(0x10A, 0x100, 0x1234),
        body: body(0x10),
        distance: Speed::new(0xA, 0),
    };
    assert_eq!(
        detect(&right, [enemy]),
        Contact::Blocked {
            slot: slot(7),
            distance: 9
        }
    );
    // Left: her left edge $13A - 5 to the enemy's right $128, a gap of $D.
    let left = Move {
        direction: Direction::Left,
        position: point(0x13A, 0x100, 0),
        distance: Speed::new(0xE, 0),
        ..right
    };
    assert_eq!(
        detect(&left, [enemy]),
        Contact::Blocked {
            slot: slot(7),
            distance: 0xD
        }
    );
    // Up: her top $118 - $10 to the enemy's bottom $108, touching.
    let up = Move {
        direction: Direction::Up,
        position: point(0x120, 0x118, 0),
        distance: Speed::new(2, 0),
        ..right
    };
    assert_eq!(detect(&up, [enemy]), Contact::Touching { slot: slot(7) });
}
