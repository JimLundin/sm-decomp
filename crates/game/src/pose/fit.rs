//! "Handle collision due to change of pose" (bank $91 log, `$91:FDAE`): a taller pose checks
//! the height it gains above and below her for solid enemies, then for blocks, and moves her
//! to fit, crouches her, or keeps the old pose (<https://patrickjohnston.org/bank/91>).
//!
//! A pose no taller than the last, and a change to a pose facing the screen, check nothing.
//! The checks leave her Y radius (`$0B00`) at the old pose's: the next frame's "set Samus
//! radius" gives her the new one.

use data::pose::PoseId;

use super::Content;
use crate::collision::solid_enemy::{Contact, Direction, SolidEnemies};
use crate::collision::{self, Body, Heading, Surroundings, Touches};
use crate::movement::{FACING_FORWARD, body};
use crate::state::samus::{Facing, Point, Samus, Speed};

#[cfg(test)]
mod tests;

/// Facing right, crouching: the pose a squeeze between two surfaces gives her (`$91:FFA7`).
pub(super) const CROUCH_RIGHT: PoseId = PoseId(0x27);

/// Facing left, crouching.
pub(super) const CROUCH_LEFT: PoseId = PoseId(0x28);

/// The smallest old Y radius a squeeze crouches from; below it, a morph ball's, the new pose
/// is refused (`$91:FFAA`).
const CROUCH_FROM: u16 = 8;

/// Half a block: the block check takes the nearer 8 pixels first when the distance reaches
/// them (`$94:96AB`).
const HALF_BLOCK: u16 = 8;

/// Where a check found something in the way: the collision flags, 1 above and 2 below
/// (`$0A34` solid enemies, `$0A36` blocks), which index the routines' tables.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Sides {
    /// Nothing either way.
    Neither,
    /// Above her.
    Above,
    /// Below her.
    Below,
    /// Both ways.
    Both,
}

impl Sides {
    const fn of(above: bool, below: bool) -> Self {
        match (above, below) {
            (false, false) => Self::Neither,
            (true, false) => Self::Above,
            (false, true) => Self::Below,
            (true, true) => Self::Both,
        }
    }

    const fn above(self) -> bool {
        matches!(self, Self::Above | Self::Both)
    }

    const fn below(self) -> bool {
        matches!(self, Self::Below | Self::Both)
    }
}

/// The pixels each way a pair of checks left: the distance each was cut to, or the whole
/// height gained when nothing was in the way (`$0A3E`/`$0A40` solid enemies, `$0A38`/`$0A3C`
/// blocks).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Space {
    up: u16,
    down: u16,
}

/// Whether the new pose has room.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Fit {
    /// It keeps the new pose, or the crouch a squeeze gave it (carry clear).
    Fits,
    /// It goes back to the old pose (carry set).
    NoSpace,
}

/// What a block check found: whether a block was in the way, and the distance's magnitude as
/// the check left it (`$12`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Found {
    collided: bool,
    space: u16,
}

/// The checks' shared inputs: where they look, the touches they reach and the height gained.
struct Checks<'s, 't> {
    surroundings: Surroundings<'s>,
    touches: &'t mut Touches,
    difference: u16,
}

/// A pose's Y radius (`$91:B62F`); a pose past the table has none.
fn radius(pose: PoseId, content: &Content) -> u16 {
    content
        .poses
        .get(pose)
        .map_or(0, |definition| u16::from(definition.y_radius))
}

/// "Handle collision due to change of pose" (`$91:FDAE`), as the pose change starts: a new
/// pose that has no room gives way to the old one, and a squeeze crouches her.
pub(super) fn fit(
    samus: &mut Samus,
    surroundings: Surroundings<'_>,
    touches: &mut Touches,
    content: &Content,
) {
    if FACING_FORWARD.contains(&samus.pose) {
        return;
    }
    let old = radius(samus.previous.pose, content);
    let Some(difference) = radius(samus.pose, content)
        .checked_sub(old)
        .filter(|&difference| difference != 0)
    else {
        return;
    };
    samus.y_radius = old;
    let mut checks = Checks {
        surroundings,
        touches,
        difference,
    };
    if settle(samus, &mut checks, content) == Fit::NoSpace {
        samus.pose = samus.previous.pose;
    }
}

/// The solid enemy checks, then the block checks, and what their flags do (`$91:FDBD`).
fn settle(samus: &mut Samus, checks: &mut Checks<'_, '_>, content: &Content) -> Fit {
    let difference = checks.difference;
    let enemies = checks.surroundings.enemies;
    let gained = Speed::new(difference, 0);
    let above = enemies.check(Direction::Up, body(samus), gained, &mut samus.position);
    let below = enemies.check(Direction::Down, body(samus), gained, &mut samus.position);
    let enemy = Sides::of(above != Contact::Clear, below != Contact::Clear);
    let mut enemy_space = Space {
        up: left_by(above, difference),
        down: left_by(below, difference),
    };
    // The solid enemy table (`$91:FE8A`): an enemy one way checks the other way, with the
    // new pose's radius, for the distance she must move away from it.
    let tall = Body {
        y_radius: radius(samus.pose, content),
        ..body(samus)
    };
    match enemy {
        // `$91:FE9A`, `$91:FE9C`.
        Sides::Neither | Sides::Both => {}
        // `$91:FE9E`.
        Sides::Above => {
            let away = difference.wrapping_sub(enemy_space.up);
            match recheck(enemies, Direction::Down, tall, away, &mut samus.position) {
                Some(away) => enemy_space.up = away,
                None => return Fit::NoSpace,
            }
        }
        // `$91:FEDF`.
        Sides::Below => {
            let away = difference.wrapping_sub(enemy_space.down);
            match recheck(enemies, Direction::Up, tall, away, &mut samus.position) {
                Some(away) => enemy_space.down = away,
                None => return Fit::NoSpace,
            }
        }
    }
    let up = blocks(samus, checks, difference.wrapping_neg());
    let down = blocks(samus, checks, difference);
    let block_space = Space {
        up: up.space,
        down: down.space,
    };
    // The block table (`$91:FE92`).
    match Sides::of(up.collided, down.collided) {
        // `$91:FF76`, by the solid enemy flags (its table at `$91:FF7F`).
        Sides::Neither => match enemy {
            // `$91:FF87`.
            Sides::Neither => Fit::Fits,
            // `$91:FF89`: down, away from the enemy above.
            Sides::Above => {
                shift(samus, enemy_space.up);
                Fit::Fits
            }
            // `$91:FF98`: up, away from the enemy below.
            Sides::Below => {
                shift(samus, enemy_space.down.wrapping_neg());
                Fit::Fits
            }
            Sides::Both => squeeze(samus, content),
        },
        // `$91:FF20`: her top boundary flush with the block above.
        Sides::Above => {
            let found = blocks(samus, checks, difference.wrapping_sub(block_space.up));
            if found.collided {
                Fit::NoSpace
            } else if enemy.below() {
                squeeze(samus, content)
            } else {
                shift(samus, found.space);
                Fit::Fits
            }
        }
        // `$91:FF49`: her bottom boundary flush with the block below.
        Sides::Below => {
            let found = blocks(samus, checks, block_space.down.wrapping_sub(difference));
            if found.collided {
                Fit::NoSpace
            } else if enemy.above() {
                squeeze(samus, content)
            } else {
                shift(samus, found.space.wrapping_neg());
                Fit::Fits
            }
        }
        Sides::Both => squeeze(samus, content),
    }
}

/// The pixels a solid enemy check left: what it cut the move to, or all of `difference`.
fn left_by(contact: Contact, difference: u16) -> u16 {
    contact.distance().map_or(difference, Speed::pixels)
}

/// A solid enemy check away from the enemy found, by the new pose's `body`: the distance it
/// leaves when nothing is in the way, none when something is.
fn recheck(
    enemies: SolidEnemies<'_>,
    direction: Direction,
    body: Body,
    distance: u16,
    position: &mut Point,
) -> Option<u16> {
    let contact = enemies.check(direction, body, Speed::new(distance, 0), position);
    (contact == Contact::Clear).then_some(distance)
}

/// "Block collision detection due to change of pose" (`$94:96AB`): the block `distance`
/// (signed) above or below her, and when that is half a block or more the nearer half block
/// first.
fn blocks(samus: &mut Samus, checks: &mut Checks<'_, '_>, distance: u16) -> Found {
    let magnitude = if distance.cast_signed() < 0 {
        distance.wrapping_neg()
    } else {
        distance
    };
    if magnitude < HALF_BLOCK {
        return block(samus, checks, distance);
    }
    let nearer = (distance & !(HALF_BLOCK - 1)) | HALF_BLOCK;
    let found = block(samus, checks, nearer);
    if found.collided {
        found
    } else {
        block(samus, checks, distance)
    }
}

/// "Block collision detection due to change of pose - single block" (`$94:96E3`): the
/// vertical block collision at `distance`, which leaves her where she is.
///
/// It sets the collision direction to $F, so a non-square slope reads it as down and the
/// special air and special block setups that test it do nothing.
// open: the quicksand flag (`$1E71`) it clears joins with quicksand.
fn block(samus: &mut Samus, checks: &mut Checks<'_, '_>, distance: u16) -> Found {
    let moved = collision::scan_y(
        checks.surroundings.room,
        body(samus),
        &mut samus.position,
        Speed::new(distance, 0),
        Heading::Down,
        checks.surroundings.order,
        checks.touches,
    );
    Found {
        collided: moved.collided,
        space: Speed(moved.distance.0.cast_signed().unsigned_abs()).pixels(),
    }
}

/// "Handle collision from both sides due to pose change" (`$91:FFA7`): with an old Y radius
/// of 8 or more she crouches, her bottom boundary kept where it was if the crouch is
/// shorter; with less there is no room.
fn squeeze(samus: &mut Samus, content: &Content) -> Fit {
    let old = samus.y_radius;
    if old < CROUCH_FROM {
        return Fit::NoSpace;
    }
    samus.pose = match samus.facing {
        Facing::Left => CROUCH_LEFT,
        Facing::Right | Facing::Forward | Facing::Unknown(_) => CROUCH_RIGHT,
    };
    let crouch = radius(samus.pose, content);
    if old < crouch {
        shift(samus, old.wrapping_sub(crouch));
    }
    Fit::Fits
}

/// Move her down by `pixels` (signed), her previous Y position (`$0B14`) with her.
const fn shift(samus: &mut Samus, pixels: u16) {
    let y = samus.position.y.pixels.wrapping_add(pixels);
    samus.position.y.pixels = y;
    samus.previous_position.y.pixels = y;
}
