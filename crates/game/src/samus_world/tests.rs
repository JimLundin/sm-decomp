use clean_types::{
    AirKind, AnsweredKind, Axis, Block, Cell, Heading, SlopeFaces, SolidKind, World,
};
use data::level::slope::{SHAPES, Slopes};
use data::level::{BlockKind, Bts, Offset, PlmChoice, Slope};

use super::{Reaction, SamusWorld};
use crate::collision::Room;
use crate::collision::solid_enemy::SolidEnemies;
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

const fn at(column: u16, row: u16) -> Cell {
    Cell { column, row }
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
    assert_eq!(world.block(at(0, 0)), Block::Solid(SolidKind::Plain));
    assert_eq!(world.block(at(1, 0)), Block::Air(AirKind::Reacting));
    assert_eq!(
        world.block(at(2, 0)),
        Block::Answered(AnsweredKind::Reacting)
    );
    assert!(world.lets_through(at(2, 0), Heading::Down));
    assert!(!world.lets_through(at(3, 0), Heading::Down));
    assert_eq!(world.block(at(0, 1)), Block::StandsFor(Axis::X, -1));
    assert_eq!(world.block(at(1, 1)), Block::StandsFor(Axis::Y, 2));
    assert_eq!(world.block(at(3, 3)), Block::Air(AirKind::Plain));
    // Past the level: air.
    assert_eq!(world.block(at(0, 9)), Block::Air(AirKind::Plain));
    assert_eq!(
        world.reaction(at(1, 0)),
        Some(Reaction::Scroll(BlockIndex(1)))
    );
    assert_eq!(
        world.reaction(at(2, 0)),
        Some(Reaction::Item(BlockIndex(2)))
    );
    assert_eq!(world.reaction(at(3, 0)), None);
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
    assert_eq!(top_left.0, [[true, false], [false, false]]);
    let Block::Stepped(bottom_right) = world.block(at(1, 0)) else {
        panic!("flipped square slope");
    };
    assert_eq!(bottom_right.0, [[false, false], [false, true]]);
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
    assert_eq!(floor.faces, SlopeFaces::Floor);
    assert_eq!((floor.surface[0], floor.surface[15]), (15, 0));
    assert_eq!(floor.factor, 0xc0);
    let Block::Slope(ceiling) = world.block(at(1, 0)) else {
        panic!("ceiling slope");
    };
    assert_eq!(ceiling.faces, SlopeFaces::Ceiling);
    assert_eq!((ceiling.surface[0], ceiling.surface[15]), (15, 0));
}
