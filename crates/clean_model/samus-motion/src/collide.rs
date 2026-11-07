//! §8: what her moves meet. The world is data; each move asks what is at her leading edge.

use crate::rules::Rules;
use clean_types::{Block, Fixed, Position, Radius, Side, Slope, SlopeFaces, SlopeHeight, World};

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
    /// `SolidEnemies`: a solid enemy.
    Enemy,
    /// `SlopesCarry`: a slope's surface at her centre column.
    Slope(SlopeHeight),
}

/// Where a stopped move leaves her. A face is the first pixel that is not free.
enum End {
    /// `SolidEnemies`: one she already touches: no distance, y subpixel 0.
    Touch,
    /// `SolidEnemies`: at its face by whole pixels, her subpixel kept.
    Pixels(u16),
    /// `BlocksStopFlush`: flush against a face.
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

/// Moves her `distance` along `axis`, stopping as §8 says. Returns what stopped her.
pub fn travel(
    position: &mut Position,
    radius: Radius,
    (axis, toward): (Axis, Toward),
    distance: Fixed,
    world: &impl World,
    rules: &Rules,
) -> Option<Stop> {
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
        .or_else(|| slope_stop(*position, radius, target, way, world, rules))
        .or_else(|| block_stop(&her, target, way, world, rules).map(|f| (Stop::Block, f)));
    let Some((stop, end)) = stop else {
        set_along(position, axis, target);
        return None;
    };
    let along = match end {
        End::Touch => {
            position.y = position.y.with_sub(0);
            return Some(stop);
        }
        End::Stay => return Some(stop),
        End::Pixels(face) => Fixed::from_parts(against(face, her.radius, toward), her.along.sub()),
        // BlocksStopFlush: right or down ends on subpixel FFFF, left or up on 0.
        End::Flush(face) => Fixed::from_parts(
            against(face, her.radius, toward),
            match toward {
                Toward::More => 0xFFFF,
                Toward::Less => 0,
            },
        ),
    };
    set_along(position, axis, along);
    Some(stop)
}

const fn set_along(position: &mut Position, axis: Axis, along: Fixed) {
    match axis {
        Axis::X => position.x = along,
        Axis::Y => position.y = along,
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

/// `BlocksStopFlush`: tests only the row or column of her leading edge at the target.
fn block_stop(
    her: &Line,
    target: Fixed,
    (axis, toward): (Axis, Toward),
    world: &impl World,
    rules: &Rules,
) -> Option<End> {
    let lead = cell(edge(target.pixel(), her.radius, toward), rules);
    let solid = (cell(her.across_lo, rules)..=cell(her.across_hi, rules)).any(|across| {
        let block = match axis {
            Axis::X => world.block(lead, across),
            Axis::Y => world.block(across, lead),
        };
        block == Block::Solid
    });
    let face = lead.wrapping_mul(rules.block_px);
    solid.then_some(End::Flush(match toward {
        Toward::More => face,
        Toward::Less => face.wrapping_add(rules.block_px).wrapping_sub(1),
    }))
}

/// `SlopesCarry`: the slope holding a pixel, and its surface's y at that pixel's column.
fn slope_at(world: &impl World, x: u16, y: u16, rules: &Rules) -> Option<(Slope, u16)> {
    let Block::Slope(slope) = world.block(cell(x, rules), cell(y, rules)) else {
        return None;
    };
    let column = x.checked_rem(rules.block_px).unwrap_or_default();
    let within = slope.surface.get(usize::from(column)).copied()?;
    let top = cell(y, rules).wrapping_mul(rules.block_px);
    Some((slope, top.wrapping_add(u16::from(within))))
}

/// `SlopesCarry`: a vertical move meets a slope at her centre column: a sloped floor stops a
/// downward move on its surface; a sloped ceiling's solid part stops a rise before it moves.
fn slope_stop(
    position: Position,
    radius: Radius,
    target: Fixed,
    (axis, toward): (Axis, Toward),
    world: &impl World,
    rules: &Rules,
) -> Option<(Stop, End)> {
    if axis == Axis::X {
        return None;
    }
    let lead = edge(target.pixel(), radius.y, toward);
    let (slope, surface) = slope_at(world, position.x.pixel(), lead, rules)?;
    match (toward, slope.faces) {
        (Toward::More, SlopeFaces::Floor) if lead >= surface => {
            Some((Stop::Slope(slope.height), End::Flush(surface)))
        }
        (Toward::Less, SlopeFaces::Ceiling) if lead <= surface => {
            Some((Stop::Slope(slope.height), End::Stay))
        }
        (Toward::More | Toward::Less, SlopeFaces::Floor | SlopeFaces::Ceiling) => None,
    }
}

/// `SlopesCarry`: after a horizontal move, a sloped floor at her centre column and bottom edge's
/// row lifts her onto it by whole pixels when her bottom edge is below its surface; otherwise a
/// sloped ceiling at her top edge's row pushes her down under it. Returns whether she was seated.
pub fn seat(position: &mut Position, radius: Radius, world: &impl World, rules: &Rules) -> bool {
    let (x, y) = (position.x.pixel(), position.y.pixel());
    let bottom = edge(y, radius.y, Toward::More);
    if let Some((slope, surface)) = slope_at(world, x, bottom, rules)
        && slope.faces == SlopeFaces::Floor
        && bottom >= surface
    {
        let lift = bottom.wrapping_sub(surface).wrapping_add(1);
        position.y = position.y.with_pixel(y.wrapping_sub(lift));
        return true;
    }
    let top = edge(y, radius.y, Toward::Less);
    if let Some((slope, surface)) = slope_at(world, x, top, rules)
        && slope.faces == SlopeFaces::Ceiling
        && top <= surface
    {
        let push = surface.wrapping_sub(top).wrapping_add(1);
        position.y = position.y.with_pixel(y.wrapping_add(push));
    }
    false
}

/// `WallJump`: whether a test move of `reach` towards `side` would be stopped (solid enemies
/// first, then blocks). Finding a block wall writes her x subpixel as a stop against it would,
/// without moving her.
pub fn wall_near(
    position: &mut Position,
    radius: Radius,
    side: Side,
    reach: Fixed,
    world: &impl World,
    rules: &Rules,
) -> bool {
    let mut probe = *position;
    let way = (Axis::X, Toward::side(side));
    let found = travel(&mut probe, radius, way, reach, world, rules);
    if found == Some(Stop::Block) {
        position.x = position.x.with_sub(match side {
            Side::Right => 0xFFFF,
            Side::Left => 0,
        });
    }
    found.is_some()
}
