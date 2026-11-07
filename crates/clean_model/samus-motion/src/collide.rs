//! §8: what her moves meet. The world is data; each move asks what is at her leading edge.

use crate::rules::Rules;
use clean_types::{Block, Fixed, Position, Radius, Side, World};

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
enum Stop {
    /// `SolidEnemies`: one she already touches: no distance, y subpixel 0.
    Touch,
    /// `BlocksStopFlush`: a face, the first pixel that is not free.
    Face(u16),
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

/// Moves her `distance` along `axis`, stopping as §8 says. Returns whether she was stopped.
pub fn travel(
    position: &mut Position,
    radius: Radius,
    (axis, toward): (Axis, Toward),
    distance: Fixed,
    world: &impl World,
    rules: &Rules,
) -> bool {
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
    // SolidEnemies: met first, and on a move of 0 too.
    let stop = enemy_stop(&her, target, (axis, toward), world)
        .or_else(|| block_stop(&her, target, (axis, toward), world, rules));
    let along = match stop {
        None => target,
        Some(Stop::Touch) => {
            position.y = position.y.with_sub(0);
            return true;
        }
        // BlocksStopFlush: right or down ends on subpixel FFFF, left or up on 0.
        Some(Stop::Face(face)) => match toward {
            Toward::More => Fixed::from_parts(face.wrapping_sub(her.radius), 0xFFFF),
            Toward::Less => Fixed::from_parts(face.wrapping_add(1).wrapping_add(her.radius), 0),
        },
    };
    match axis {
        Axis::X => position.x = along,
        Axis::Y => position.y = along,
    }
    stop.is_some()
}

/// `SolidEnemies`: the first solid enemy across her path that she touches or reaches.
fn enemy_stop(
    her: &Line,
    target: Fixed,
    (axis, toward): (Axis, Toward),
    world: &impl World,
) -> Option<Stop> {
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
            Some(Stop::Touch)
        } else if reached {
            Some(Stop::Face(near))
        } else {
            None
        }
    })
}

/// `BlocksStopFlush`: tests only the row or column of her leading edge at the target.
fn block_stop(
    her: &Line,
    target: Fixed,
    (axis, toward): (Axis, Toward),
    world: &impl World,
    rules: &Rules,
) -> Option<Stop> {
    let cell = |pixel: u16| pixel.checked_div(rules.block_px).unwrap_or_default();
    let lead = cell(edge(target.pixel(), her.radius, toward));
    let solid = (cell(her.across_lo)..=cell(her.across_hi)).any(|across| {
        let block = match axis {
            Axis::X => world.block(lead, across),
            Axis::Y => world.block(across, lead),
        };
        block != Block::Air
    });
    let face = lead.wrapping_mul(rules.block_px);
    solid.then_some(Stop::Face(match toward {
        Toward::More => face,
        Toward::Less => face.wrapping_add(rules.block_px).wrapping_sub(1),
    }))
}

/// `WallJump`: whether a wall on `side` is within `reach`; finding one writes her x subpixel as a
/// stop against it would, without moving her.
pub fn wall_near(
    position: &mut Position,
    radius: Radius,
    side: Side,
    reach: Fixed,
    world: &impl World,
    rules: &Rules,
) -> bool {
    let mut probe = *position;
    let found = travel(
        &mut probe,
        radius,
        (Axis::X, Toward::side(side)),
        reach,
        world,
        rules,
    );
    if found {
        position.x = position.x.with_sub(match side {
            Side::Right => 0xFFFF,
            Side::Left => 0,
        });
    }
    found
}

/// `SlopesCarry`: whether the block under her is a slope.
pub fn on_slope(position: Position, radius: Radius, world: &impl World, rules: &Rules) -> bool {
    let cell = |pixel: u16| pixel.checked_div(rules.block_px).unwrap_or_default();
    let below = position.y.pixel().wrapping_add(radius.y);
    world.block(cell(position.x.pixel()), cell(below)) == Block::Slope
}
