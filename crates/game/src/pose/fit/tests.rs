use std::marker::PhantomData;

use data::level::{BlockKind, Bts};
use data::pose::{MovementType, Poses};

use super::*;
use crate::collision::solid_enemy::one_enemy;
use crate::collision::{Room, ScanOrder};
use crate::state::room::{Block, BlockColumns, Enemy, EnemyBody, EnemyCommon};
use crate::state::samus::{Fixed, PoseRecord};
use crate::state::slots::{Slot, SlotIndex};

/// The morph ball's landing in `item-morph-ball`: from a pose of Y radius $13 to $A5, $15.
const FALLING: PoseId = PoseId(0x29);
const LANDING: PoseId = PoseId(0xA5);

/// A pose of Y radius 7, one of 8 and one of $10.
const MORPH: PoseId = PoseId(0x41);
const SHORT: PoseId = PoseId(0x42);
const TALL: PoseId = PoseId(0x43);

/// The crouches' Y radius.
const CROUCH: u8 = 0x0A;

fn content() -> Content {
    let radii = [
        (FALLING, 0x13),
        (LANDING, 0x15),
        (MORPH, 7),
        (SHORT, 8),
        (TALL, 0x10),
        (CROUCH_RIGHT, CROUCH),
        (CROUCH_LEFT, CROUCH),
        (PoseId(0), 0x18),
    ];
    let bytes: Vec<u8> = (0..0xB0_u16)
        .flat_map(|pose| {
            let radius = radii
                .iter()
                .find(|&&(id, _)| id == PoseId(pose))
                .map_or(0, |&(_, radius)| radius);
            [8, 0, 0xFF, 2, 0, 0, radius, 0]
        })
        .collect();
    Content {
        poses: Poses::decode(&bytes).unwrap(),
        ..Content::default()
    }
}

fn at(x: u16, y: u16, y_subpixels: u16) -> Point {
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

/// Samus changing from `old` to `new` at `position`, facing `facing`.
fn changing(old: PoseId, new: PoseId, position: Point, facing: Facing) -> Samus {
    Samus {
        position,
        pose: new,
        facing,
        previous: PoseRecord {
            pose: old,
            facing,
            movement: MovementType::Falling,
        },
        y_radius: 0x99,
        ..Samus::default()
    }
}

/// Solid enemies 8 pixels each way around each of `centres`, in list order.
fn enemies(centres: &[(u16, u16)]) -> Vec<Slot<EnemyCommon, EnemyBody>> {
    centres
        .iter()
        .flat_map(|&(x, y)| one_enemy(x, y, true))
        .collect()
}

const LIST: [SlotIndex<Enemy>; 2] = [SlotIndex(0, PhantomData), SlotIndex(1, PhantomData)];

/// Four blocks across, four down.
const WIDTH: u16 = 4;

/// A level of air with solid blocks across `rows`.
fn level(rows: &[u16]) -> Vec<Block> {
    let mut blocks = vec![Block::default(); usize::from(WIDTH * WIDTH)];
    for (index, block) in blocks.iter_mut().enumerate() {
        let row = u16::try_from(index).unwrap() / WIDTH;
        if rows.contains(&row) {
            block.kind = BlockKind::Solid(Bts(0));
        }
    }
    blocks
}

fn run(samus: &mut Samus, blocks: &[Block], slots: &[Slot<EnemyCommon, EnemyBody>]) -> Touches {
    let surroundings = Surroundings {
        room: Room {
            blocks,
            width: BlockColumns(WIDTH),
            slopes: None,
        },
        order: ScanOrder::LeftToRight,
        enemies: SolidEnemies {
            interactive: &LIST[..slots.len()],
            slots,
        },
    };
    let mut touches = Touches::default();
    fit(samus, surroundings, &mut touches, &content());
    touches
}

#[test]
fn landing_on_the_face_block_lifts_her_by_the_radius_gained() {
    // `item-morph-ball` frame 797: her bottom touches the face block's top at $2B0, so the
    // check down zeroes her Y subposition and the 2 pixels gained go up.
    let mut samus = changing(FALLING, LANDING, at(0x488, 0x29D, 0x2BFF), Facing::Right);
    run(&mut samus, &[], &enemies(&[(0x488, 0x2B8)]));
    assert_eq!(
        samus.position.y,
        Fixed {
            pixels: 0x29B,
            subpixels: 0,
        }
    );
    assert_eq!(samus.previous_position.y.pixels, 0x29B);
    assert_eq!(samus.y_radius, 0x13);
    assert_eq!(samus.pose, LANDING);
}

#[test]
fn a_shorter_pose_checks_nothing() {
    let mut samus = changing(LANDING, FALLING, at(0x488, 0x29D, 0x2BFF), Facing::Right);
    run(&mut samus, &[], &enemies(&[(0x488, 0x2B8)]));
    assert_eq!(samus.position, at(0x488, 0x29D, 0x2BFF));
    assert_eq!(samus.y_radius, 0x99);
    assert_eq!(samus.pose, FALLING);
}

#[test]
fn a_pose_facing_the_screen_checks_nothing() {
    let mut samus = changing(SHORT, PoseId(0), at(0x18, 0x18, 0), Facing::Forward);
    run(&mut samus, &level(&[0, 2]), &[]);
    assert_eq!(samus.position, at(0x18, 0x18, 0));
    assert_eq!(samus.y_radius, 0x99);
    assert_eq!(samus.pose, PoseId(0));
}

#[test]
fn with_room_both_ways_she_stays_and_keeps_the_old_radius() {
    let mut samus = changing(SHORT, TALL, at(0x18, 0x18, 0x1234), Facing::Right);
    run(&mut samus, &[], &[]);
    assert_eq!(samus.position, at(0x18, 0x18, 0x1234));
    assert_eq!(samus.y_radius, 8);
    assert_eq!(samus.pose, TALL);
}

#[test]
fn an_enemy_touching_her_top_pushes_her_down() {
    let mut samus = changing(FALLING, LANDING, at(0x100, 0x100, 0x8000), Facing::Right);
    // Her top is at $100 - $13 = $ED: the enemy's bottom.
    run(&mut samus, &[], &enemies(&[(0x100, 0xE5)]));
    assert_eq!(
        samus.position.y,
        Fixed {
            pixels: 0x102,
            subpixels: 0,
        }
    );
    assert_eq!(samus.pose, LANDING);
}

#[test]
fn an_enemy_a_pixel_above_her_cuts_the_push() {
    // A gap of 1 leaves 1 pixel up; the other pixel gained pushes her down.
    let mut samus = changing(FALLING, LANDING, at(0x100, 0x100, 0), Facing::Right);
    run(&mut samus, &[], &enemies(&[(0x100, 0xE4)]));
    assert_eq!(samus.position.y.pixels, 0x101);
    assert_eq!(samus.pose, LANDING);
}

#[test]
fn an_enemy_pushing_her_into_another_keeps_the_old_pose() {
    // Touching one above; the check down with the new radius reaches the one below.
    let mut samus = changing(FALLING, LANDING, at(0x100, 0x100, 0), Facing::Right);
    // Above: bottom at $ED. Below: out of reach at the old radius, touching at the new.
    run(&mut samus, &[], &enemies(&[(0x100, 0xE5), (0x100, 0x11D)]));
    assert_eq!(samus.pose, FALLING);
    assert_eq!(samus.position.y.pixels, 0x100);
}

#[test]
fn a_floor_lifts_her_flush_with_it() {
    // Radius 8 to $10 with the floor at $30: room below at $18, the 8 pixels gained go up
    // when it touches her bottom at $28.
    let mut samus = changing(SHORT, TALL, at(0x18, 0x18, 0), Facing::Right);
    run(&mut samus, &level(&[3]), &[]);
    assert_eq!(samus.position.y.pixels, 0x18);
    let mut samus = changing(SHORT, TALL, at(0x18, 0x28, 0), Facing::Right);
    run(&mut samus, &level(&[3]), &[]);
    // The check down puts her subpixels next to the floor.
    assert_eq!(
        samus.position.y,
        Fixed {
            pixels: 0x20,
            subpixels: 0xFFFF,
        }
    );
    assert_eq!(samus.previous_position.y.pixels, 0x20);
    assert_eq!(samus.pose, TALL);
}

#[test]
fn a_ceiling_pushes_her_down_flush_with_it() {
    // Radius 8 to $10 under the ceiling ending at $1F: the 8 pixels gained go down.
    let mut samus = changing(SHORT, TALL, at(0x18, 0x28, 0x4000), Facing::Right);
    run(&mut samus, &level(&[1]), &[]);
    assert_eq!(
        samus.position.y,
        Fixed {
            pixels: 0x30,
            subpixels: 0,
        }
    );
    assert_eq!(samus.pose, TALL);
}

#[test]
fn squeezed_between_blocks_she_crouches_her_bottom_kept() {
    // Radius 8 to $10 between a ceiling ending at $0F and a floor at $20, facing left: the
    // crouch, radius $0A, keeps her bottom at $1F.
    let mut samus = changing(SHORT, TALL, at(0x18, 0x18, 0), Facing::Left);
    run(&mut samus, &level(&[0, 2]), &[]);
    assert_eq!(samus.pose, CROUCH_LEFT);
    assert_eq!(samus.position.y.pixels, 0x16);
    let mut samus = changing(SHORT, TALL, at(0x18, 0x18, 0), Facing::Right);
    run(&mut samus, &level(&[0, 2]), &[]);
    assert_eq!(samus.pose, CROUCH_RIGHT);
}

#[test]
fn a_morph_ball_squeezed_keeps_the_old_pose() {
    // Radius 7 is too small to crouch from.
    let mut samus = changing(MORPH, TALL, at(0x18, 0x18, 0), Facing::Left);
    run(&mut samus, &level(&[0, 2]), &[]);
    assert_eq!(samus.pose, MORPH);
    assert_eq!(samus.position.y.pixels, 0x18);
}

#[test]
fn an_enemy_below_and_a_ceiling_squeeze_her() {
    // A ceiling ending at $1F and an enemy touching her bottom at $38.
    let mut samus = changing(SHORT, TALL, at(0x18, 0x28, 0), Facing::Right);
    run(&mut samus, &level(&[1]), &enemies(&[(0x18, 0x38)]));
    assert_eq!(samus.pose, CROUCH_RIGHT);
    assert_eq!(samus.position.y.pixels, 0x26);
}
