//! §8: what her moves meet. The world is data; each move asks what is at her leading edge, and
//! reports the cells it reaches (`TouchedCells`).

use crate::rules::Rules;
use clean_types::{
    AirKind, Axis, Block, Cell, Fixed, Half, Halves, Heading, Parity, Position, Quarters, Radius,
    Side, Slope, SlopeFaces, Touch, World,
};
use std::cell::RefCell;

/// Which way along the axis: left or up, right or down.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Toward {
    /// Left or up.
    Less,
    /// Right or down.
    More,
}

impl Toward {
    /// The way along x for a side.
    pub const fn side(side: Side) -> Self {
        match side {
            Side::Left => Self::Less,
            Side::Right => Self::More,
        }
    }
}

/// What stopped a move.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Stop {
    /// `BlocksStopFlush`: a solid block.
    Block,
    /// `BlocksStopFlush`: a stepped block's solid quarter.
    Stepped,
    /// `SolidEnemies`: a solid enemy.
    Enemy,
    /// `SlopesCarry`: a sloped surface at her centre column.
    Slope,
}

/// Where a move leaves her, and what stopped it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Moved {
    /// Her position after the move.
    pub to: Position,
    /// What stopped it, if anything.
    pub stop: Option<Stop>,
}

/// What a frame's moves meet: the room's blocks and solid enemies, and the frame's parity, which
/// orders a vertical move's stops (`BlocksStopFlush`); and the cells the moves reach.
pub struct Room<'a, W> {
    /// The room.
    pub world: &'a W,
    /// The frame's parity.
    pub parity: Parity,
    /// `TouchedCells`: the cells her moves reach, in order.
    pub reached: RefCell<Vec<Touch>>,
}

/// `OtherBlockKinds`: what a cell is to her motion.
enum Shape {
    Air,
    Solid,
    Stepped(Quarters),
    Slope(Slope),
}

/// `OtherBlockKinds`: a cell's kind and the cell it is reported as. A cell that stands for
/// another is that cell's kind, followed to the chain's end; one that names nothing, or a chain
/// that comes back on itself, is air.
pub fn resolve(world: &impl World, at: Cell) -> (Block, Cell) {
    let mut seen = Vec::new();
    let mut cell = at;
    loop {
        match world.block(cell) {
            Block::StandsFor(axis, by) if by != 0 && !seen.contains(&cell) => {
                seen.push(cell);
                let shift = |n: u16| n.wrapping_add_signed(by);
                cell = match axis {
                    Axis::X => Cell {
                        column: shift(cell.column),
                        ..cell
                    },
                    Axis::Y => Cell {
                        row: shift(cell.row),
                        ..cell
                    },
                };
            }
            Block::StandsFor(..) => return (Block::Air(AirKind::Plain), cell),
            block @ (Block::Air(_)
            | Block::Solid(_)
            | Block::Answered(_)
            | Block::Slope(_)
            | Block::Stepped(_)) => return (block, cell),
        }
    }
}

/// `OtherBlockKinds`: how the cell at `at` meets a move heading `heading`, and the cell reported.
fn meet(world: &impl World, at: Cell, heading: Heading) -> (Shape, Cell) {
    let (block, named) = resolve(world, at);
    let shape = match block {
        Block::Answered(_) if world.lets_through(named, heading) => Shape::Air,
        Block::Solid(_) | Block::Answered(_) => Shape::Solid,
        Block::Stepped(quarters) => Shape::Stepped(quarters),
        Block::Slope(slope) => Shape::Slope(slope),
        Block::Air(_) | Block::StandsFor(..) => Shape::Air,
    };
    (shape, named)
}

/// `TouchedCells`: a move's direction.
const fn heading((axis, toward): (Axis, Toward)) -> Heading {
    match (axis, toward) {
        (Axis::X, Toward::Less) => Heading::Left,
        (Axis::X, Toward::More) => Heading::Right,
        (Axis::Y, Toward::Less) => Heading::Up,
        (Axis::Y, Toward::More) => Heading::Down,
    }
}

/// `TouchedCells`: the cells she is inside, from her position at the frame's start: her centre
/// column at her bottom edge's row; at her centre's row, when it differs; at her top edge's row,
/// when it differs from her bottom edge's and the three rows combined by exclusive-or are not 0.
pub fn inside(position: Position, radius: Radius, world: &impl World, rules: &Rules) -> Vec<Cell> {
    let column = cell(position.x.pixel(), rules);
    let y = position.y.pixel();
    let bottom = cell(edge(y, radius.y, Toward::More), rules);
    let centre = cell(y, rules);
    let top = cell(edge(y, radius.y, Toward::Less), rules);
    let mut rows = vec![bottom];
    if centre != bottom {
        rows.push(centre);
    }
    if top != bottom && bottom ^ centre ^ top != 0 {
        rows.push(top);
    }
    rows.into_iter()
        .map(|row| resolve(world, Cell { column, row }).1)
        .collect()
}

/// `SlopesSlowTheWalk`: a horizontal move of `distance` towards `side`, shortened by the walk
/// factor of each sloped floor in her leading edge's column at the unshortened target, in each
/// row her body spans, top row first.
pub fn walk_shortened(
    position: Position,
    radius: Radius,
    side: Side,
    distance: Fixed,
    world: &impl World,
    rules: &Rules,
) -> Fixed {
    let toward = Toward::side(side);
    let target = match toward {
        Toward::Less => position.x.minus(distance),
        Toward::More => position.x.plus(distance),
    };
    let column = cell(edge(target.pixel(), radius.x, toward), rules);
    let her = Line::of(position.x, position.y, radius, Axis::X);
    let (lo, hi) = her.across_span();
    (cell(lo, rules)..=cell(hi, rules)).fold(distance, |d, row| {
        match resolve(world, Cell { column, row }).0 {
            Block::Slope(slope) if slope.faces == SlopeFaces::Floor => {
                shorten(d, toward, slope.factor)
            }
            Block::Air(_)
            | Block::Solid(_)
            | Block::Answered(_)
            | Block::Slope(_)
            | Block::Stepped(_)
            | Block::StandsFor(..) => d,
        }
    })
}

/// `SlopesSlowTheWalk`: the move cut to a multiple of 1/256 px towards minus infinity, its pixel
/// part kept mod 256, its length multiplied by the factor exactly, its direction put back.
fn shorten(distance: Fixed, toward: Toward, factor: u16) -> Fixed {
    // Read as 8.8 (the pixel part's low byte, the subpixel's high byte), the signed move's bits
    // are the cut, rounded down.
    let cut = match toward {
        Toward::Less => Fixed::ZERO.minus(distance).as_8_8().wrapping_neg(),
        Toward::More => distance.as_8_8(),
    };
    Fixed::from_bits(u32::from(cut).wrapping_mul(u32::from(factor)))
}

/// `BlocksStopFlush`: the subpixel a flush stop leaves: FFFF right or down, 0 left or up.
const fn flush_sub(toward: Toward) -> u16 {
    match toward {
        Toward::More => 0xFFFF,
        Toward::Less => 0,
    }
}

/// Where a stopped move leaves her. A face is the first pixel that is not free.
enum End {
    /// `SolidEnemies`: one she already touches: no move, y subpixel 0.
    Touch,
    /// `SolidEnemies`, `SlopesCarry`: her centre at this pixel, her subpixel kept.
    Pixels(u16),
    /// `BlocksStopFlush`: flush against a face, its subpixel FFFF or 0, never backwards.
    Flush(u16),
    /// `SlopesCarry`: a sloped ceiling: no move at all, her subpixel kept.
    Stay,
}

/// One axis of a box: its centre and half-size along the axis, and across it.
struct Line {
    along: Fixed,
    radius: u16,
    across: u16,
    across_radius: u16,
}

impl Line {
    const fn of(x: Fixed, y: Fixed, radius: Radius, axis: Axis) -> Self {
        let (along, across, r_along, r_across) = match axis {
            Axis::X => (x, y.pixel(), radius.x, radius.y),
            Axis::Y => (y, x.pixel(), radius.y, radius.x),
        };
        Self {
            along,
            radius: r_along,
            across,
            across_radius: r_across,
        }
    }

    /// The first and last pixels across that the box covers.
    const fn across_span(&self) -> (u16, u16) {
        let lo = self.across.wrapping_sub(self.across_radius);
        let hi = self.across.wrapping_add(self.across_radius).wrapping_sub(1);
        (lo, hi)
    }
}

/// Whether pixel `a` lies beyond pixel `b` the way `toward` points (positions wrap mod 2^16).
const fn beyond(a: u16, b: u16, toward: Toward) -> bool {
    let d = a.wrapping_sub(b).cast_signed();
    match toward {
        Toward::More => d > 0,
        Toward::Less => d < 0,
    }
}

/// `SolidEnemies`: whether two centres are closer than `reach` (positions wrap mod 2^16).
const fn closer(a: u16, b: u16, reach: u16) -> bool {
    a.wrapping_sub(b).cast_signed().unsigned_abs() < reach
}

/// `pixel` moved `by` pixels the way `toward` points.
const fn shift(pixel: u16, by: u16, toward: Toward) -> u16 {
    match toward {
        Toward::More => pixel.wrapping_add(by),
        Toward::Less => pixel.wrapping_sub(by),
    }
}

/// `BlocksStopFlush`: her leading edge with her centre at `pixel`.
const fn edge(pixel: u16, radius: u16, toward: Toward) -> u16 {
    match toward {
        Toward::More => pixel.wrapping_add(radius).wrapping_sub(1),
        Toward::Less => pixel.wrapping_sub(radius),
    }
}

/// §8: her centre pixel when her leading edge ends just before `face`.
const fn against(face: u16, radius: u16, toward: Toward) -> u16 {
    match toward {
        Toward::More => face.wrapping_sub(radius),
        Toward::Less => face.wrapping_add(1).wrapping_add(radius),
    }
}

/// Where a move of `distance` along `axis` from `position` leaves her, stopping as §8 says; the
/// cells it reaches go to the room's report (`TouchedCells`).
pub fn move_until_met(
    position: Position,
    radius: Radius,
    (axis, toward): (Axis, Toward),
    distance: Fixed,
    room: &Room<impl World>,
    rules: &Rules,
) -> Moved {
    // BlocksStopFlush, SolidEnemies, TouchedCells: nothing stops a move of exactly 0, and it
    // reaches no cells.
    if distance.is_zero() {
        return Moved {
            to: position,
            stop: None,
        };
    }
    // BlocksStopFlush: a horizontal move is limited to 15 px either way, its subpixel kept.
    let distance = if axis == Axis::X && distance.pixel() > rules.move_limit_px {
        distance.with_pixel(rules.move_limit_px)
    } else {
        distance
    };
    let her = Line::of(position.x, position.y, radius, axis);
    let target = match toward {
        Toward::Less => her.along.minus(distance),
        Toward::More => her.along.plus(distance),
    };
    // SolidEnemies: an enemy in the path decides the move alone; blocks stop her only on a path
    // free of them.
    let way = (axis, toward);
    let stop = enemy_stop(&her, distance, target, way, room.world)
        .map(|end| (Stop::Enemy, end))
        .or_else(|| block_stop(position, &her, target, way, room, rules));
    let Some((stop, end)) = stop else {
        return Moved {
            to: along(position, axis, target),
            stop: None,
        };
    };
    let start = her.along.pixel();
    let to = match end {
        // SolidEnemies: one she touches stops the move with no distance, y subpixel 0.
        End::Touch => Position {
            y: position.y.with_sub(0),
            ..position
        },
        End::Stay => position,
        End::Pixels(centre) => along(position, axis, her.along.with_pixel(centre)),
        End::Flush(face) => {
            // BlocksStopFlush: a stop never moves her backwards; only the subpixel is written.
            let flush = against(face, her.radius, toward);
            let centre = if beyond(start, flush, toward) {
                start
            } else {
                flush
            };
            along(position, axis, Fixed::from_parts(centre, flush_sub(toward)))
        }
    };
    Moved {
        to,
        stop: Some(stop),
    }
}

/// `position` with its `axis` part set to `value`.
const fn along(position: Position, axis: Axis, value: Fixed) -> Position {
    match axis {
        Axis::X => Position {
            x: value,
            ..position
        },
        Axis::Y => Position {
            y: value,
            ..position
        },
    }
}

/// `SolidEnemies`: the first solid enemy her reach meets that stops her. Her reach is her box at
/// the move's end pushed 1 px further on: moving left or up unless the end's subpixel is 0,
/// moving right or down unless both her start's subpixel and the move's are 0. An enemy she
/// reaches is judged by the gap from where she starts: 0 is touching, a larger gap stops her by
/// that many whole pixels, a negative one doesn't stop her.
fn enemy_stop(
    her: &Line,
    distance: Fixed,
    target: Fixed,
    (axis, toward): (Axis, Toward),
    world: &impl World,
) -> Option<End> {
    let pushes = match toward {
        Toward::Less => target.sub() != 0,
        Toward::More => her.along.sub() != 0 || distance.sub() != 0,
    };
    let reach = shift(target.pixel(), u16::from(pushes), toward);
    let start = her.along.pixel();
    world.solid_enemies().iter().find_map(|enemy| {
        let it = Line::of(Fixed::px(enemy.x), Fixed::px(enemy.y), enemy.radius, axis);
        let centre = it.along.pixel();
        let reached = closer(reach, centre, her.radius.wrapping_add(it.radius))
            && closer(
                her.across,
                it.across,
                her.across_radius.wrapping_add(it.across_radius),
            );
        // The enemy's near side less her near side, less 1.
        let (ahead, behind) = match toward {
            Toward::More => (
                centre.wrapping_sub(it.radius),
                start.wrapping_add(her.radius),
            ),
            Toward::Less => (
                start.wrapping_sub(her.radius),
                centre.wrapping_add(it.radius),
            ),
        };
        let gap = ahead.wrapping_sub(behind).wrapping_sub(1).cast_signed();
        if !reached || gap < 0 {
            None
        } else if gap == 0 {
            Some(End::Touch)
        } else {
            Some(End::Pixels(shift(start, gap.unsigned_abs(), toward)))
        }
    })
}

/// §8: the 16 px cell holding a pixel.
const fn cell(pixel: u16, rules: &Rules) -> u16 {
    match pixel.checked_div(rules.block_px) {
        Some(cell) => cell,
        None => 0,
    }
}

/// `BlocksStopFlush`: the half of its block an offset within the block lies in.
const fn half(offset: u16, rules: &Rules) -> Half {
    if offset < rules.quarter_px {
        Half::First
    } else {
        Half::Second
    }
}

/// `BlocksStopFlush`: the face of a span of `len` px from `from`, on the side she meets it.
const fn face(from: u16, len: u16, toward: Toward) -> u16 {
    match toward {
        Toward::More => from,
        Toward::Less => from.wrapping_add(len).wrapping_sub(1),
    }
}

/// `BlocksStopFlush`: the halves of the block at cell `across` that the pixels `lo..=hi` cover.
const fn halves(lo: u16, hi: u16, across: u16, rules: &Rules) -> Halves {
    let start = across.wrapping_mul(rules.block_px);
    let end = start.wrapping_add(rules.block_px).wrapping_sub(1);
    let first = if lo > start { lo } else { start };
    let last = if hi < end { hi } else { end };
    Halves {
        first: first.wrapping_sub(start) < rules.quarter_px,
        second: last.wrapping_sub(start) >= rules.quarter_px,
    }
}

/// `BlocksStopFlush`, `SlopesCarry`: what stops a move in the row or column of her leading edge
/// at the target, in every block her body spans across it: a solid block at its face; a stepped
/// block at the face of the half her leading edge reaches; and, on a vertical move, a sloped
/// surface at her centre column. The cells are met top to bottom on a horizontal move, and left
/// to right (an even frame) or right to left (an odd one) on a vertical move; the first that
/// stops her decides, and `TouchedCells` reports them up to it.
fn block_stop(
    position: Position,
    her: &Line,
    target: Fixed,
    way: (Axis, Toward),
    room: &Room<impl World>,
    rules: &Rules,
) -> Option<(Stop, End)> {
    let (axis, toward) = way;
    let lead_px = edge(target.pixel(), her.radius, toward);
    let lead = cell(lead_px, rules);
    let start = lead.wrapping_mul(rules.block_px);
    let reached = half(lead_px.wrapping_sub(start), rules);
    let half_start = match reached {
        Half::First => start,
        Half::Second => start.wrapping_add(rules.quarter_px),
    };
    let (lo, hi) = her.across_span();
    let centre = position.x.pixel();
    let mut line: Vec<u16> = (cell(lo, rules)..=cell(hi, rules)).collect();
    if (axis, room.parity) == (Axis::Y, Parity::Odd) {
        line.reverse();
    }
    line.into_iter().find_map(|across| {
        let at = match axis {
            Axis::X => Cell {
                column: lead,
                row: across,
            },
            Axis::Y => Cell {
                column: across,
                row: lead,
            },
        };
        let (shape, named) = meet(room.world, at, heading(way));
        room.reached.borrow_mut().push(Touch {
            cell: named,
            heading: heading(way),
        });
        match shape {
            Shape::Solid => Some((Stop::Block, End::Flush(face(start, rules.block_px, toward)))),
            Shape::Stepped(quarters) => quarters
                .stops(axis, reached, halves(lo, hi, across, rules))
                .then_some((
                    Stop::Stepped,
                    End::Flush(face(half_start, rules.quarter_px, toward)),
                )),
            // SlopesCarry: a sloped surface never stops a horizontal move, and a vertical one
            // only at her centre column.
            Shape::Slope(slope) if axis == Axis::Y && across == cell(centre, rules) => {
                slope_stop(slope, (centre, start), lead_px, her.radius, toward, rules)
            }
            Shape::Air | Shape::Slope(_) => None,
        }
    })
}

/// `SlopesCarry`: a sloped surface's y at pixel column `x`, in the block whose top row is `top`.
fn surface_y(slope: Slope, x: u16, top: u16, rules: &Rules) -> Option<u16> {
    let column = x.checked_rem(rules.block_px).unwrap_or_default();
    let within = slope.surface.get(usize::from(column)).copied()?;
    Some(top.wrapping_add(u16::from(within)))
}

/// `SlopesCarry`: the sloped surface holding a pixel, and its surface's y at that pixel's column.
fn slope_at(world: &impl World, x: u16, y: u16, rules: &Rules) -> Option<(SlopeFaces, u16)> {
    let at = Cell {
        column: cell(x, rules),
        row: cell(y, rules),
    };
    let (Block::Slope(slope), _) = resolve(world, at) else {
        return None;
    };
    let top = cell(y, rules).wrapping_mul(rules.block_px);
    Some((slope.faces, surface_y(slope, x, top, rules)?))
}

/// `SlopesCarry`: her bottom edge at the target reaching a sloped floor's last air row, or lower,
/// stops a downward move with that edge on it, by whole pixels; her top edge reaching a sloped
/// ceiling's solid part, or the first air row under it, stops a rise before it moves.
/// `(x, top)` is her centre column and the block's top row.
fn slope_stop(
    slope: Slope,
    (x, top): (u16, u16),
    lead: u16,
    radius: u16,
    toward: Toward,
    rules: &Rules,
) -> Option<(Stop, End)> {
    let surface = surface_y(slope, x, top, rules)?;
    match (toward, slope.faces) {
        (Toward::More, SlopeFaces::Floor) if lead >= surface.wrapping_sub(1) => {
            Some((Stop::Slope, End::Pixels(against(surface, radius, toward))))
        }
        (Toward::Less, SlopeFaces::Ceiling) if lead <= surface.wrapping_add(1) => {
            Some((Stop::Slope, End::Stay))
        }
        (Toward::More | Toward::Less, SlopeFaces::Floor | SlopeFaces::Ceiling) => None,
    }
}

/// `SlopesCarry`: the two checks after every horizontal move, each asking one cell at her centre
/// column. A sloped floor at her bottom edge's row, with her bottom edge in its solid part, lifts
/// her by whole pixels onto its last air row; then a sloped ceiling at her top edge's row, with
/// her top edge in its solid part or on the first air row under it, pushes her down by whole
/// pixels onto that row. Either seats her: returns where she is seated, if she is.
pub fn seat(
    position: Position,
    radius: Radius,
    world: &impl World,
    rules: &Rules,
) -> Option<Position> {
    let x = position.x.pixel();
    let mut y = position.y.pixel();
    let mut seated = false;
    let bottom = edge(y, radius.y, Toward::More);
    if let Some((SlopeFaces::Floor, surface)) = slope_at(world, x, bottom, rules)
        && bottom >= surface
    {
        y = y.wrapping_sub(bottom.wrapping_sub(surface).wrapping_add(1));
        seated = true;
    }
    let top = edge(y, radius.y, Toward::Less);
    if let Some((SlopeFaces::Ceiling, surface)) = slope_at(world, x, top, rules)
        && top <= surface.wrapping_add(1)
    {
        y = y.wrapping_add(surface.wrapping_add(1).wrapping_sub(top));
        seated = true;
    }
    seated.then_some(Position {
        y: position.y.with_pixel(y),
        ..position
    })
}

/// `WallJump`: whether a test move of `reach` towards `side` would be stopped (solid enemies
/// first, then blocks), and where the test leaves her. A block wall writes her x subpixel as a
/// stop against it would, without moving her. A solid enemy writes her x nothing: one she
/// already touches clears her y subpixel (`SolidEnemies`), one ahead writes nothing. The cells
/// the test reaches are its own, not the frame's report (`TouchedCells`).
pub fn wall_test(
    position: Position,
    radius: Radius,
    side: Side,
    reach: Fixed,
    room: &Room<impl World>,
    rules: &Rules,
) -> Moved {
    let toward = Toward::side(side);
    let mark = room.reached.borrow().len();
    let test = move_until_met(position, radius, (Axis::X, toward), reach, room, rules);
    room.reached.borrow_mut().truncate(mark);
    let to = match test.stop {
        Some(Stop::Block | Stop::Stepped) => Position {
            x: position.x.with_sub(flush_sub(toward)),
            ..position
        },
        Some(Stop::Enemy) => Position {
            y: test.to.y,
            ..position
        },
        Some(Stop::Slope) | None => position,
    };
    Moved {
        to,
        stop: test.stop,
    }
}
