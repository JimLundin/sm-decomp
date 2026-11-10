use clean_types::{Block, Cell, Heading, SlopeFaces, Way, World};
use core::num::Wrapping;
use data::level::slope::{SHAPES, Slopes};
use data::level::{BlockKind, Bts, Offset, PlmChoice, Slope};

use super::{SamusFault, SamusWorld};
use crate::collision::solid_enemy::SolidEnemies;
use crate::collision::{Room, Touch};
use crate::state::room::{Block as LevelBlock, BlockColumns, BlockIndex};

/// Four blocks across.
const WIDTH: u16 = 4;

/// Square shape 1 solid only top left; surface shape `$12` a ramp, factor `$C0`.
fn slopes() -> Slopes {
    let squares = [0x00, 0x00, 0x00, 0x00, 0x80, 0x00, 0x00, 0x00];
    let mut surfaces = vec![0; SHAPES * 16];
    let mut factors = vec![0; SHAPES * 4];
    for (column, top) in surfaces[0x12 * 16..0x13 * 16].iter_mut().enumerate() {
        *top = u8::try_from(column).unwrap_or(0);
    }
    factors[0x12 * 4 + 2] = 0xc0;
    Slopes::decode(&squares, &surfaces, &factors)
}

fn level(kinds: &[((u16, u16), BlockKind)]) -> Vec<LevelBlock> {
    let mut blocks = vec![LevelBlock::default(); usize::from(WIDTH * WIDTH)];
    for &((column, row), kind) in kinds {
        blocks[usize::from(row.wrapping_mul(WIDTH).wrapping_add(column))].kind = kind;
    }
    blocks
}

fn world<'a>(blocks: &'a [LevelBlock], slopes: &'a Slopes) -> SamusWorld<'a> {
    let room = Room {
        blocks,
        width: BlockColumns(WIDTH),
        slopes: Some(slopes),
    };
    SamusWorld::new(room, SolidEnemies::default())
}

fn slope(bts: u8) -> Slope {
    let BlockKind::Slope(slope) = BlockKind::new(1, bts) else {
        panic!("a slope's kind");
    };
    slope
}

const fn at(x: u16, y: u16) -> Cell {
    Cell { x, y }
}

/// A surface column's depth from the block's open side: its row, 16 just past, 17 far past.
const fn row(depth: clean_types::Px) -> u16 {
    depth.0
}

/// A stand-in chain that loops, which the original follows without end, is cut where the model
/// meets it and refused; one that leads into a loop from outside it alike.
#[test]
fn a_stand_in_chain_that_loops_is_refused() {
    let slopes = slopes();
    let blocks = level(&[
        ((0, 2), BlockKind::HorizontalExtension(Offset(1))),
        ((1, 2), BlockKind::HorizontalExtension(Offset(-1))),
        ((2, 2), BlockKind::VerticalExtension(Offset(-2))),
        ((2, 0), BlockKind::HorizontalExtension(Offset(-2))),
        ((0, 0), BlockKind::VerticalExtension(Offset(2))),
    ]);
    let world = world(&blocks, &slopes);
    assert_eq!(world.refused(), Ok(()));
    // (2, 2) leads to (2, 0), then (0, 0), then into the loop of (0, 2) and (1, 2).
    assert!(matches!(world.block(at(2, 2)), Block::StandsFor(None)));
    assert_eq!(world.refused(), Err(SamusFault::StandInLoop(at(2, 2))));
    // The first cell refused is the one reported.
    assert!(matches!(world.block(at(0, 2)), Block::StandsFor(None)));
    assert_eq!(world.refused(), Err(SamusFault::StandInLoop(at(2, 2))));
}

#[test]
fn each_kind_meets_her_as_its_block_kind_says() {
    let slopes = slopes();
    let blocks = level(&[
        ((0, 0), BlockKind::Solid(Bts(0))),
        ((1, 0), BlockKind::SpecialAir(Bts(0x46))),
        ((2, 0), BlockKind::Special(PlmChoice::Common(0x45))),
        ((3, 0), BlockKind::Special(PlmChoice::Common(0x01))),
        ((0, 1), BlockKind::HorizontalExtension(Offset(-1))),
        ((1, 1), BlockKind::VerticalExtension(Offset(2))),
    ]);
    let world = world(&blocks, &slopes);
    assert!(matches!(world.block(at(0, 0)), Block::Solid));
    assert!(matches!(world.block(at(1, 0)), Block::Air));
    assert!(matches!(world.block(at(2, 0)), Block::Answered));
    // An item's block lets a move and a look through alike.
    assert!(world.lets_through(at(2, 0), Way::Move(Heading::Down)));
    assert!(world.lets_through(at(2, 0), Way::Look));
    assert!(!world.lets_through(at(3, 0), Way::Move(Heading::Down)));
    // One block back along the level from (0, 1) is the row above's last.
    assert!(matches!(
        world.block(at(0, 1)),
        Block::StandsFor(Some(Cell { x: 3, y: 0 }))
    ));
    assert!(matches!(
        world.block(at(1, 1)),
        Block::StandsFor(Some(Cell { x: 1, y: 3 }))
    ));
    assert!(matches!(world.block(at(3, 3)), Block::Air));
    // Past the level: air.
    assert!(matches!(world.block(at(0, 9)), Block::Air));
    // Chains that end are no fault.
    assert_eq!(world.refused(), Ok(()));
    let down = |cell| clean_types::Touch {
        cell,
        way: Way::Move(Heading::Down),
    };
    assert_eq!(
        world.reaction(down(at(1, 0))),
        Some(Touch::Blocks {
            cell: at(1, 0),
            way: Way::Move(Heading::Down),
        })
    );
    assert_eq!(
        world.reaction(down(at(2, 0))),
        Some(Touch::Item(BlockIndex(2)))
    );
    assert_eq!(
        world.reaction(down(at(3, 0))),
        Some(Touch::Blocks {
            cell: at(3, 0),
            way: Way::Move(Heading::Down),
        })
    );
    assert_eq!(world.reaction(down(at(0, 0))), None);
}

#[test]
fn a_square_slope_is_stepped_through_its_flips() {
    let slopes = slopes();
    let plain = slope(0x01);
    let flipped = slope(0x01 | 0x40 | 0x80);
    let blocks = level(&[
        ((0, 0), BlockKind::Slope(plain)),
        ((1, 0), BlockKind::Slope(flipped)),
    ]);
    let world = world(&blocks, &slopes);
    let Block::Stepped(top_left) = world.block(at(0, 0)) else {
        panic!("square slope");
    };
    assert_eq!(top_left, [[true, false], [false, false]]);
    let Block::Stepped(bottom_right) = world.block(at(1, 0)) else {
        panic!("flipped square slope");
    };
    assert_eq!(bottom_right, [[false, false], [false, true]]);
}

#[test]
fn a_surface_is_a_floor_or_a_ceiling_by_its_flip() {
    let slopes = slopes();
    let floor = slope(0x12 | 0x40);
    let ceiling = slope(0x12 | 0x80);
    let blocks = level(&[
        ((0, 0), BlockKind::Slope(floor)),
        ((1, 0), BlockKind::Slope(ceiling)),
    ]);
    let world = world(&blocks, &slopes);
    let Block::Slope(floor) = world.block(at(0, 0)) else {
        panic!("floor slope");
    };
    assert!(floor.faces == SlopeFaces::Floor);
    assert_eq!((row(floor.depth[0]), row(floor.depth[15])), (15, 0));
    assert_eq!(floor.factor, Wrapping(0xc0));
    let Block::Slope(ceiling) = world.block(at(1, 0)) else {
        panic!("ceiling slope");
    };
    assert!(ceiling.faces == SlopeFaces::Ceiling);
    // A ceiling counts up from the block's bottom, as the level's offsets do.
    assert_eq!((row(ceiling.depth[0]), row(ceiling.depth[15])), (0, 15));
}

/// Gap 7: a column with no solid pixel (10h) lies just past the block and one four rows past
/// (14h, as shape 1Bh's columns 0 to 7) far past it, for a floor and a ceiling alike.
#[test]
fn an_empty_column_is_just_past_and_a_far_one_far_past_floor_or_ceiling() {
    let squares = [0; 8];
    let mut surfaces = vec![0; SHAPES * 16];
    surfaces[0x13 * 16] = 0x10;
    surfaces[0x13 * 16 + 1] = 0x14;
    let factors = vec![0; SHAPES * 4];
    let slopes = Slopes::decode(&squares, &surfaces, &factors);
    let blocks = level(&[
        ((0, 0), BlockKind::Slope(slope(0x13))),
        ((1, 0), BlockKind::Slope(slope(0x13 | 0x80))),
    ]);
    let world = world(&blocks, &slopes);
    let Block::Slope(floor) = world.block(at(0, 0)) else {
        panic!("floor slope");
    };
    assert_eq!((row(floor.depth[0]), row(floor.depth[1])), (16, 17));
    let Block::Slope(ceiling) = world.block(at(1, 0)) else {
        panic!("ceiling slope");
    };
    assert_eq!((row(ceiling.depth[0]), row(ceiling.depth[1])), (16, 17));
}

#[test]
fn the_cells_her_moves_reached_become_the_touches_in_order() {
    let slopes = slopes();
    let blocks = level(&[
        ((1, 0), BlockKind::SpecialAir(Bts(0x46))),
        ((2, 0), BlockKind::Special(PlmChoice::Common(0x45))),
    ]);
    let world = world(&blocks, &slopes);
    let reached = |column, way| clean_types::Touch {
        cell: at(column, 0),
        way,
    };
    // A look's cells react as a move's do.
    let touches = world.touches(&[
        reached(2, Way::Move(Heading::Right)),
        reached(0, Way::Move(Heading::Right)),
        reached(1, Way::Look),
    ]);
    assert_eq!(
        touches.touched(),
        [
            Touch::Item(BlockIndex(2)),
            Touch::Blocks {
                cell: at(1, 0),
                way: Way::Look,
            },
        ]
    );
}
