//! §8: what her moves meet. The world is data; each move asks what is at her leading edge.

use crate::rules::Rules;
use clean_types::{Block, Fixed, Half, Position, Radius, Side, SlopeFaces, World};

/// The axis of a move.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Axis {
    /// Horizontal.
    X,
    /// Vertical.
    Y,
}

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

/// `BlocksStopFlush`: the subpixel a flush stop leaves: FFFF right or down, 0 left or up.
const fn flush_sub(toward: Toward) -> u16 {
    match toward {
        Toward::More => 0xFFFF,
        Toward::Less => 0,
    }
}

/// Where a stopped move leaves her. A face is the first pixel that is not free.
enum End {
    /// `SolidEnemies`: one she already touches: no distance, y subpixel 0.
    Touch,
    /// `SolidEnemies`: at its face by whole pixels, her subpixel kept.
    Pixels(u16),
    /// `BlocksStopFlush`: flush against a face, its subpixel FFFF or 0.
    Flush(u16),
    /// `SlopesCarry`: a sloped ceiling: no move at all, her subpixel kept.
    Stay,
}

/// One axis of a box: its centre along the axis, and its span across it.
struct Line {
    along: Fixed,
    radius: u16,
    across_lo: u16,
    across_hi: u16,
}

const fn line(x: Fixed, y: Fixed, radius: Radius, axis: Axis) -> Line {
    let (along, across, r_along, r_across) = match axis {
        Axis::X => (x, y.pixel(), radius.x, radius.y),
        Axis::Y => (y, x.pixel(), radius.y, radius.x),
    };
    Line {
        along,
        radius: r_along,
        across_lo: across.wrapping_sub(r_across),
        across_hi: across.wrapping_add(r_across).wrapping_sub(1),
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

/// Where a move of `distance` along `axis` from `position` leaves her, stopping as §8 says.
pub fn travel(
    position: Position,
    radius: Radius,
    (axis, toward): (Axis, Toward),
    distance: Fixed,
    world: &impl World,
    rules: &Rules,
) -> Moved {
    // BlocksStopFlush: a horizontal move is limited to 15 px either way, its subpixel kept.
    let distance = if axis == Axis::X && distance.pixel() > rules.move_limit_px {
        distance.with_pixel(rules.move_limit_px)
    } else {
        distance
    };
    let her = line(position.x, position.y, radius, axis);
    let target = match toward {
        Toward::Less => her.along.minus(distance),
        Toward::More => her.along.plus(distance),
    };
    // SolidEnemies: an enemy in the path decides the move alone, even on a move of 0; blocks
    // stop her only on a path free of them.
    let way = (axis, toward);
    let stop = enemy_stop(&her, target, way, world)
        .map(|end| (Stop::Enemy, end))
        .or_else(|| slope_stop(position, radius, target, way, world, rules))
        .or_else(|| block_stop(&her, target, way, world, rules));
    let Some((stop, end)) = stop else {
        return Moved {
            to: along(position, axis, target),
            stop: None,
        };
    };
    let to = match end {
        // SolidEnemies: one she touches stops the move with no distance, y subpixel 0.
        End::Touch => Position {
            y: position.y.with_sub(0),
            ..position
        },
        End::Stay => position,
        End::Pixels(face) => along(
            position,
            axis,
            Fixed::from_parts(against(face, her.radius, toward), her.along.sub()),
        ),
        End::Flush(face) => along(
            position,
            axis,
            Fixed::from_parts(against(face, her.radius, toward), flush_sub(toward)),
        ),
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

/// `SolidEnemies`: the first solid enemy across her path that she touches or reaches.
fn enemy_stop(
    her: &Line,
    target: Fixed,
    (axis, toward): (Axis, Toward),
    world: &impl World,
) -> Option<End> {
    let now = edge(her.along.pixel(), her.radius, toward);
    let then = edge(target.pixel(), her.radius, toward);
    world.solid_enemies().iter().find_map(|enemy| {
        let it = line(Fixed::px(enemy.x), Fixed::px(enemy.y), enemy.radius, axis);
        if it.across_hi < her.across_lo || her.across_hi < it.across_lo {
            return None;
        }
        let lo = it.along.pixel().wrapping_sub(it.radius);
        let hi = it.along.pixel().wrapping_add(it.radius).wrapping_sub(1);
        // The pixel just past her edge on the tested side, and the enemy's near face.
        let (next, near) = match toward {
            Toward::More => (now.wrapping_add(1), lo),
            Toward::Less => (now.wrapping_sub(1), hi),
        };
        let reached = match toward {
            Toward::More => then >= near && now < near,
            Toward::Less => then <= near && now > near,
        };
        if (lo..=hi).contains(&next) {
            Some(End::Touch)
        } else if reached {
            Some(End::Pixels(near))
        } else {
            None
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

/// `BlocksStopFlush`: tests only the row or column of her leading edge at the target, in every
/// block her body spans across it. A solid block stops her at its face; a stepped block at the
/// face of its solid quarter that her leading edge reaches. The nearest face found decides.
fn block_stop(
    her: &Line,
    target: Fixed,
    (axis, toward): (Axis, Toward),
    world: &impl World,
    rules: &Rules,
) -> Option<(Stop, End)> {
    let (size, quarter) = (rules.block_px, rules.quarter_px);
    let lead_px = edge(target.pixel(), her.radius, toward);
    let lead = cell(lead_px, rules);
    let start = lead.wrapping_mul(size);
    // The face of a span of `len` px from `from`, on the side she meets it.
    let face = |from: u16, len: u16| match toward {
        Toward::More => from,
        Toward::Less => from.wrapping_add(len).wrapping_sub(1),
    };
    let (near, far_start) = match toward {
        Toward::More => (Half::First, start.wrapping_add(quarter)),
        Toward::Less => (Half::Second, start),
    };
    let faces = (cell(her.across_lo, rules)..=cell(her.across_hi, rules)).filter_map(|across| {
        let block = match axis {
            Axis::X => world.block(lead, across),
            Axis::Y => world.block(across, lead),
        };
        let quarters = match block {
            Block::Solid => return Some((Stop::Block, face(start, size))),
            Block::Stepped(quarters) => quarters,
            Block::Air | Block::Slope(_) => return None,
        };
        // The halves across that her body spans within this block.
        let across_start = across.wrapping_mul(size);
        let first = her.across_lo.max(across_start).wrapping_sub(across_start);
        let last = her
            .across_hi
            .min(across_start.wrapping_add(size).wrapping_sub(1))
            .wrapping_sub(across_start);
        let spans = [
            (Half::First, first < quarter),
            (Half::Second, last >= quarter),
        ];
        let solid = |along: Half| {
            spans.iter().any(|&(across, spanned)| {
                spanned
                    && match axis {
                        Axis::X => quarters.solid(across, along),
                        Axis::Y => quarters.solid(along, across),
                    }
            })
        };
        if !solid(half(lead_px.wrapping_sub(start), rules)) {
            None
        } else if solid(near) {
            Some((Stop::Stepped, face(start, size)))
        } else {
            Some((Stop::Stepped, face(far_start, quarter)))
        }
    });
    faces
        .min_by_key(|&(_, face)| match toward {
            Toward::More => face,
            Toward::Less => u16::MAX.wrapping_sub(face),
        })
        .map(|(stop, face)| (stop, End::Flush(face)))
}

/// `SlopesCarry`: the sloped surface holding a pixel, and its surface's y at that pixel's column.
fn slope_at(world: &impl World, x: u16, y: u16, rules: &Rules) -> Option<(SlopeFaces, u16)> {
    let Block::Slope(slope) = world.block(cell(x, rules), cell(y, rules)) else {
        return None;
    };
    let column = x.checked_rem(rules.block_px).unwrap_or_default();
    let within = slope.surface.get(usize::from(column)).copied()?;
    let top = cell(y, rules).wrapping_mul(rules.block_px);
    Some((slope.faces, top.wrapping_add(u16::from(within))))
}

/// `SlopesCarry`: a vertical move meets a sloped surface only at her centre column. A sloped
/// floor stops a downward move with her bottom edge on the last air row above it, by whole
/// pixels; a sloped ceiling's solid part stops a rise before it moves.
fn slope_stop(
    position: Position,
    radius: Radius,
    target: Fixed,
    (axis, toward): (Axis, Toward),
    world: &impl World,
    rules: &Rules,
) -> Option<(Stop, End)> {
    if axis == Axis::X {
        // SlopesCarry: a sloped surface never stops a horizontal move.
        return None;
    }
    let lead = edge(target.pixel(), radius.y, toward);
    let (faces, surface) = slope_at(world, position.x.pixel(), lead, rules)?;
    match (toward, faces) {
        (Toward::More, SlopeFaces::Floor) if lead >= surface => {
            Some((Stop::Slope, End::Pixels(surface)))
        }
        (Toward::Less, SlopeFaces::Ceiling) if lead <= surface => Some((Stop::Slope, End::Stay)),
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
/// already touches clears her y subpixel (`SolidEnemies`), one ahead writes nothing.
pub fn wall_test(
    position: Position,
    radius: Radius,
    side: Side,
    reach: Fixed,
    world: &impl World,
    rules: &Rules,
) -> Moved {
    let toward = Toward::side(side);
    let test = travel(position, radius, (Axis::X, toward), reach, world, rules);
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
