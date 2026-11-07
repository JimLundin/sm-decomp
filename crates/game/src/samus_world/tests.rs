use clean_types::{
    AirKind, AnsweredKind, Block, Cell, Fixed, Heading, Position, Radius, SlopeFaces, SolidKind,
    Surface, World,
};
use data::level::slope::{SHAPES, Slopes};
use data::level::{BlockKind, Bts, Offset, PlmChoice, Slope};

use super::SamusWorld;
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

const fn at(column: u16, row: u16) -> Cell {
    Cell { column, row }
}

/// A surface column as the vectors write it: its row, 16 just past, 17 far past.
const fn row(surface: Surface) -> u8 {
    match surface {
        Surface::Row(row) => row,
        Surface::JustPast => 16,
        Surface::FarPast => 17,
    }
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
    assert!(matches!(
        world.block(at(0, 0)),
        Block::Solid(SolidKind::Plain)
    ));
    assert!(matches!(
        world.block(at(1, 0)),
        Block::Air(AirKind::Reacting)
    ));
    assert!(matches!(
        world.block(at(2, 0)),
        Block::Answered(AnsweredKind::Reacting)
    ));
    assert!(world.lets_through(at(2, 0), Heading::Down));
    assert!(!world.lets_through(at(3, 0), Heading::Down));
    // One block back along the level from (0, 1) is the row above's last.
    assert!(matches!(
        world.block(at(0, 1)),
        Block::StandsFor(Some(Cell { column: 3, row: 0 }))
    ));
    assert!(matches!(
        world.block(at(1, 1)),
        Block::StandsFor(Some(Cell { column: 1, row: 3 }))
    ));
    assert!(matches!(world.block(at(3, 3)), Block::Air(AirKind::Plain)));
    // Past the level: air.
    assert!(matches!(world.block(at(0, 9)), Block::Air(AirKind::Plain)));
    assert_eq!(world.reaction(at(1, 0)), Some(Touch::Scroll(BlockIndex(1))));
    assert_eq!(world.reaction(at(2, 0)), Some(Touch::Item(BlockIndex(2))));
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
    assert_eq!((row(floor.surface[0]), row(floor.surface[15])), (15, 0));
    assert_eq!(floor.factor, 0xc0);
    let Block::Slope(ceiling) = world.block(at(1, 0)) else {
        panic!("ceiling slope");
    };
    assert!(ceiling.faces == SlopeFaces::Ceiling);
    assert_eq!((row(ceiling.surface[0]), row(ceiling.surface[15])), (15, 0));
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
    assert_eq!((row(floor.surface[0]), row(floor.surface[1])), (16, 17));
    let Block::Slope(ceiling) = world.block(at(1, 0)) else {
        panic!("ceiling slope");
    };
    assert_eq!((row(ceiling.surface[0]), row(ceiling.surface[1])), (16, 17));
}

#[test]
fn the_cells_her_moves_reached_become_the_touches_in_order() {
    let slopes = slopes();
    let blocks = level(&[
        ((1, 0), BlockKind::SpecialAir(Bts(0x46))),
        ((2, 0), BlockKind::Special(PlmChoice::Common(0x45))),
    ]);
    let world = world(&blocks, &slopes);
    let reached = |column, heading| clean_types::Touch {
        cell: at(column, 0),
        heading,
    };
    let touches = world.touches(&[
        reached(2, Heading::Right),
        reached(0, Heading::Right),
        reached(1, Heading::Up),
    ]);
    assert_eq!(
        touches.touched(),
        [Touch::Item(BlockIndex(2)), Touch::Scroll(BlockIndex(1))]
    );
}

#[test]
fn only_her_centres_scroll_trigger_reacts_when_it_is_in_another_row() {
    let slopes = slopes();
    let blocks = level(&[
        ((1, 1), BlockKind::SpecialAir(Bts(0x46))),
        ((1, 2), BlockKind::Special(PlmChoice::Common(0x45))),
    ]);
    let world = world(&blocks, &slopes);
    let radius = Radius { x: 5, y: 0x10 };
    // Centre on row 1, bottom edge on row 2.
    let position = Position {
        x: Fixed::px(0x18),
        y: Fixed::px(0x18),
    };
    let inside = [at(1, 2), at(1, 1), at(1, 0)];
    assert_eq!(
        world.inside_touch(&inside, position, radius),
        Some(Touch::Scroll(BlockIndex(5)))
    );
    // The item's block below reacts to no inside point.
    assert_eq!(
        world.inside_touch(&[at(1, 2), at(1, 2)], position, radius),
        None
    );
    // Centre and bottom edge on one row: the second cell is her top's.
    let short = Radius { x: 5, y: 4 };
    let low = Position {
        x: Fixed::px(0x18),
        y: Fixed::px(0x1A),
    };
    assert_eq!(world.inside_touch(&[at(1, 1), at(1, 1)], low, short), None);
}
