use data::level::slope::{SHAPES, Slopes};
use data::level::{Bts, Slope};

use super::*;

/// Four blocks across, four down.
const WIDTH: u16 = 4;

/// The four square shapes' quadrants and every surface but the triangle's empty, with the
/// bank log's triangle (`$94:8B2B` row 12h, factor $C0).
fn slopes() -> Slopes {
    let squares = [0x00, 0x00, 0x80, 0x80];
    let mut surfaces = vec![0; SHAPES * 16];
    let mut factors = vec![0; SHAPES * 4];
    for (column, top) in surfaces[0x12 * 16..0x13 * 16].iter_mut().enumerate() {
        *top = 0x10_u8.wrapping_sub(u8::try_from(column).unwrap());
    }
    factors[0x12 * 4 + 2] = 0xc0;
    Slopes::decode(&squares, &surfaces, &factors)
}

/// A level of air with `kinds` at their (column, row).
fn level(kinds: &[((u16, u16), BlockKind)]) -> Vec<Block> {
    let mut blocks = vec![Block::default(); usize::from(WIDTH * WIDTH)];
    for &((column, row), kind) in kinds {
        blocks[usize::from(row.wrapping_mul(WIDTH).wrapping_add(column))].kind = kind;
    }
    blocks
}

fn room<'a>(blocks: &'a [Block], slopes: &'a Slopes) -> Room<'a> {
    Room {
        blocks,
        width: BlockColumns(WIDTH),
        slopes: Some(slopes),
    }
}

const BODY: Body = Body {
    x_radius: 5,
    y_radius: 8,
    still_vertically: true,
};

fn at(x: u16, y: u16) -> Point {
    Point {
        x: Fixed {
            pixels: x,
            subpixels: 0,
        },
        y: Fixed {
            pixels: y,
            subpixels: 0,
        },
    }
}

const fn solid() -> BlockKind {
    BlockKind::Solid(Bts(0))
}

fn slope(shape: u8) -> BlockKind {
    BlockKind::Slope(Slope {
        shape,
        ..Slope::default()
    })
}

fn pixels(distance: i16) -> Speed {
    Speed::new(distance.cast_unsigned(), 0)
}

#[test]
fn a_floor_lands_her_on_its_top_at_the_last_subpixel() {
    let blocks = level(&[((1, 2), solid()), ((2, 2), solid())]);
    let slopes = slopes();
    let mut position = at(24, 20);
    let moved = move_y(
        room(&blocks, &slopes),
        BODY,
        &mut position,
        pixels(8),
        Heading::Down,
        ScanOrder::LeftToRight,
        &mut Touches::default(),
    );
    // Her bottom, 20 + 8 - 1, stops at 31, above the floor's 32.
    assert_eq!(
        moved,
        Moved {
            distance: pixels(4),
            collided: true
        }
    );
    assert_eq!((position.y.pixels, position.y.subpixels), (24, 0xffff));
}

#[test]
fn a_ceiling_stops_her_at_its_bottom_at_subpixel_zero() {
    let blocks = level(&[((1, 0), solid())]);
    let slopes = slopes();
    let mut position = at(24, 30);
    position.y.subpixels = 0x8000;
    let moved = move_y(
        room(&blocks, &slopes),
        BODY,
        &mut position,
        pixels(-8),
        Heading::Up,
        ScanOrder::RightToLeft,
        &mut Touches::default(),
    );
    // Her top, 30 - 8, stops at 16, under the ceiling's 15.
    assert_eq!(moved.distance, pixels(-6));
    assert!(moved.collided);
    assert_eq!((position.y.pixels, position.y.subpixels), (24, 0));
}

#[test]
fn a_wall_stops_her_either_way() {
    let blocks = level(&[((0, 0), solid()), ((2, 1), solid())]);
    let slopes = slopes();
    let mut position = at(20, 20);
    // Rightwards: her right side, 20 + 5 - 1, stops at 31; the wall is in her lower row.
    let moved = move_x(
        room(&blocks, &slopes),
        BODY,
        &mut position,
        pixels(10),
        &mut Touches::default(),
    );
    assert_eq!(moved.distance, pixels(7));
    assert!(moved.collided);
    assert_eq!((position.x.pixels, position.x.subpixels), (27, 0xffff));
    // Leftwards from 30: her left side stops at 16, beside the wall's 15.
    let mut position = at(30, 20);
    let moved = move_x(
        room(&blocks, &slopes),
        BODY,
        &mut position,
        pixels(-10),
        &mut Touches::default(),
    );
    assert_eq!(moved.distance, pixels(-9));
    assert_eq!((position.x.pixels, position.x.subpixels), (21, 0));
}

#[test]
fn air_lets_her_through_by_the_whole_distance() {
    let blocks = level(&[]);
    let slopes = slopes();
    let mut position = at(20, 20);
    let distance = Speed::new(3, 0x8000);
    let moved = move_x(
        room(&blocks, &slopes),
        BODY,
        &mut position,
        distance,
        &mut Touches::default(),
    );
    assert_eq!(
        moved,
        Moved {
            distance,
            collided: false
        }
    );
    assert_eq!((position.x.pixels, position.x.subpixels), (23, 0x8000));
}

#[test]
fn an_extension_stands_for_the_block_it_points_at() {
    let blocks = level(&[
        ((2, 0), BlockKind::HorizontalExtension(Offset(1))),
        ((3, 0), solid()),
        ((2, 1), BlockKind::VerticalExtension(Offset(0))),
    ]);
    let slopes = slopes();
    let mut position = at(20, 20);
    let moved = move_x(
        room(&blocks, &slopes),
        BODY,
        &mut position,
        pixels(10),
        &mut Touches::default(),
    );
    // The extension's own column stops her, not the solid block's.
    assert_eq!(moved.distance, pixels(7));
    assert!(moved.collided);
}

#[test]
fn a_half_height_square_slope_lands_her_on_its_lower_half() {
    let blocks = level(&[((1, 2), slope(0))]);
    let slopes = slopes();
    let mut position = at(24, 20);
    let room = room(&blocks, &slopes);
    let down = |position: &mut Point, distance| {
        move_y(
            room,
            BODY,
            position,
            pixels(distance),
            Heading::Down,
            ScanOrder::LeftToRight,
            &mut Touches::default(),
        )
    };
    // Into the empty top half: through.
    assert!(!down(&mut position, 10).collided);
    assert_eq!(position.y.pixels, 30);
    // Into the solid bottom half: her bottom stops at 39, above its 40.
    let moved = down(&mut position, 4);
    assert_eq!(moved.distance, pixels(2));
    assert!(moved.collided);
    assert_eq!((position.y.pixels, position.y.subpixels), (32, 0xffff));
}

#[test]
fn a_triangle_slope_stops_her_on_its_surface_in_her_column() {
    let blocks = level(&[((1, 2), slope(0x12))]);
    let slopes = slopes();
    let mut position = at(24, 30);
    let moved = move_y(
        room(&blocks, &slopes),
        BODY,
        &mut position,
        pixels(4),
        Heading::Down,
        ScanOrder::RightToLeft,
        &mut Touches::default(),
    );
    // Column 8's surface is 8 below the block's top, 40: her bottom stops at 39.
    assert_eq!(moved.distance, pixels(2));
    assert!(moved.collided);
    assert_eq!(position.y.pixels, 32);
    // A ceiling heading reads it as a floor slope: no collision.
    let mut position = at(24, 30);
    let moved = move_y(
        room(&blocks, &slopes),
        BODY,
        &mut position,
        pixels(4),
        Heading::Up,
        ScanOrder::RightToLeft,
        &mut Touches::default(),
    );
    assert!(!moved.collided);
}

#[test]
fn walking_onto_a_triangle_scales_her_step_by_its_factor() {
    let blocks = level(&[((1, 1), slope(0x12))]);
    let slopes = slopes();
    let mut position = at(20, 20);
    let moved = move_x(
        room(&blocks, &slopes),
        BODY,
        &mut position,
        pixels(1),
        &mut Touches::default(),
    );
    assert_eq!(moved.distance, Speed::new(0, 0xc000));
    assert!(!moved.collided);
    let mut position = at(28, 20);
    let moved = move_x(
        room(&blocks, &slopes),
        BODY,
        &mut position,
        pixels(-1),
        &mut Touches::default(),
    );
    assert_eq!(moved.distance, Speed(0x0000_c000_u32.wrapping_neg()));
    // In the air she keeps the whole step.
    let airborne = Body {
        still_vertically: false,
        ..BODY
    };
    let mut position = at(20, 20);
    let moved = move_x(
        room(&blocks, &slopes),
        airborne,
        &mut position,
        pixels(1),
        &mut Touches::default(),
    );
    assert_eq!(moved.distance, pixels(1));
}

const fn trigger() -> BlockKind {
    BlockKind::SpecialAir(SCROLL_TRIGGER)
}

#[test]
fn scroll_triggers_join_the_touches_in_the_scans_order_and_let_her_through() {
    let blocks = level(&[((0, 2), trigger()), ((1, 2), trigger())]);
    let slopes = slopes();
    let fall = |order| {
        let mut position = at(16, 20);
        let mut touches = Touches::default();
        let moved = move_y(
            room(&blocks, &slopes),
            BODY,
            &mut position,
            pixels(8),
            Heading::Down,
            order,
            &mut touches,
        );
        assert_eq!(
            moved,
            Moved {
                distance: pixels(8),
                collided: false
            }
        );
        touches
    };
    assert_eq!(
        fall(ScanOrder::LeftToRight).blocks(),
        [BlockIndex(8), BlockIndex(9)]
    );
    assert_eq!(
        fall(ScanOrder::RightToLeft).blocks(),
        [BlockIndex(9), BlockIndex(8)]
    );
}

#[test]
fn an_extension_touches_the_trigger_it_points_at() {
    let blocks = level(&[
        ((2, 1), BlockKind::HorizontalExtension(Offset(1))),
        ((3, 1), trigger()),
    ]);
    let slopes = slopes();
    let mut position = at(24, 20);
    let mut touches = Touches::default();
    let moved = move_x(
        room(&blocks, &slopes),
        BODY,
        &mut position,
        pixels(10),
        &mut touches,
    );
    assert!(!moved.collided);
    assert_eq!(position.x.pixels, 34);
    assert_eq!(touches.blocks(), [BlockIndex(7)]);
}
