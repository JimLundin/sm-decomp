//! Samus's solid enemy collision: whether a move would run her into an enemy she can't pass.
//!
//! Bank $A0 log (<https://patrickjohnston.org/bank/A0>), "Samus / solid enemy collision
//! detection" (`$A0:A8F0`). The move's target position (`$A0:A919` left, `$A0:A945` right,
//! `$A0:A96C` up, `$A0:A993` down) is her position moved by the distance and one pixel more
//! whenever a fraction is involved. Each interactive enemy (`$17EC`) that is frozen or whose
//! hitbox is solid to her is tested for overlap with her box at the target (`$A0:A9EC`), and
//! the first that overlaps decides by the gap between her leading edge and its facing edge
//! (`$A0:AA2F`): none, and she is already touching it (`$A0:AAC8`); a positive gap, and the
//! move is cut to it (`$A0:AB22`); a negative one, she is already inside it, and the next
//! enemy is tried.
//!
//! The extra pixel lets her reach one pixel further into an enemy than into a block, which is
//! the zebetite skip's frozen rinka.

use crate::collision::Body;
use crate::state::room::{Enemy, EnemyBody, EnemyCommon, EnemyProperty};
use crate::state::samus::{Point, Speed};
use crate::state::slots::{Frames, Slot, SlotIndex};

/// The way a move goes (the original's collision direction).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    /// Leftward.
    Left,
    /// Rightward.
    Right,
    /// Upward.
    Up,
    /// Downward.
    Down,
}

/// Samus's move as the check sees it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Move {
    /// The way she moves.
    pub direction: Direction,
    /// Where she is before the move.
    pub position: Point,
    /// Her radii; only the X and Y radius are read.
    pub body: Body,
    /// How far she moves, never negative: the callers negate a leftward or upward move first.
    pub distance: Speed,
}

/// An interactive enemy as the check sees it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Obstacle {
    /// Its centre; the subpixels play no part.
    pub position: Point,
    /// Half its width.
    pub radius_x: u16,
    /// Half its height.
    pub radius_y: u16,
    /// Its hitbox is solid to Samus.
    pub solid: bool,
    /// It is frozen, which makes any enemy solid.
    pub frozen: bool,
}

impl From<&EnemyCommon> for Obstacle {
    fn from(enemy: &EnemyCommon) -> Self {
        Self {
            position: enemy.position,
            radius_x: enemy.radius_x,
            radius_y: enemy.radius_y,
            solid: enemy.properties.contains(EnemyProperty::SolidToSamus),
            frozen: enemy.frozen != Frames(0),
        }
    }
}

/// The interactive `slots` (in list order) as obstacles, from the enemies' table.
pub fn obstacles<'a, B>(
    slots: &'a [SlotIndex<Enemy>],
    enemies: &'a [Slot<EnemyCommon, B>],
) -> impl Iterator<Item = (SlotIndex<Enemy>, Obstacle)> + 'a {
    slots.iter().filter_map(|&slot| {
        enemies
            .get(usize::from(slot.0))
            .map(|enemy| (slot, Obstacle::from(&enemy.common)))
    })
}

/// The enemies a move can run into: the frame's interactive list (`$17EC`) and the enemies'
/// table it indexes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SolidEnemies<'a> {
    /// The interactive slots, in list order.
    pub interactive: &'a [SlotIndex<Enemy>],
    /// The enemies' table.
    pub slots: &'a [Slot<EnemyCommon, EnemyBody>],
}

impl SolidEnemies<'_> {
    /// The check (`$A0:A8F0`) for Samus at `position` moving `distance` the `direction` way,
    /// with what it does to her: [`Contact::Touching`] zeroes her Y subposition.
    pub fn check(
        &self,
        direction: Direction,
        body: Body,
        distance: Speed,
        position: &mut Point,
    ) -> Contact {
        let movement = Move {
            direction,
            position: *position,
            body,
            distance,
        };
        let contact = detect(&movement, obstacles(self.interactive, self.slots));
        if let Contact::Touching { .. } = contact {
            position.y.subpixels = 0;
        }
        contact
    }
}

/// What the check found.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Contact {
    /// No solid enemy is in the way; the block collision moves her.
    Clear,
    /// She already touches `slot`'s facing edge: the move is cut to nothing, and her Y
    /// subposition is zeroed (`$A0:AAC8`), whichever way she moves.
    Touching {
        /// The enemy she touches.
        slot: SlotIndex<Enemy>,
    },
    /// `slot` is `distance` whole pixels ahead of her leading edge: the move is cut to that,
    /// with no fraction (`$A0:AB22`).
    Blocked {
        /// The enemy in the way.
        slot: SlotIndex<Enemy>,
        /// The pixels she can still move.
        distance: u16,
    },
}

impl Contact {
    /// The enemy that stopped her, if any.
    #[must_use]
    pub const fn slot(self) -> Option<SlotIndex<Enemy>> {
        match self {
            Self::Clear => None,
            Self::Touching { slot } | Self::Blocked { slot, .. } => Some(slot),
        }
    }

    /// The distance the move is cut to, if an enemy stopped her.
    #[must_use]
    pub const fn distance(self) -> Option<Speed> {
        match self {
            Self::Clear => None,
            Self::Touching { .. } => Some(Speed::ZERO),
            Self::Blocked { distance, .. } => Some(Speed::new(distance, 0)),
        }
    }
}

/// Whether a 16-bit difference is positive when read signed: the original's `BEQ` then `BPL`.
const fn positive(word: u16) -> bool {
    word != 0 && (word & 0x8000) == 0
}

/// The magnitude of a 16-bit difference read signed (`BPL`, else `EOR #$FFFF : INC`).
const fn magnitude(word: u16) -> u16 {
    if (word & 0x8000) == 0 {
        word
    } else {
        word.wrapping_neg()
    }
}

/// One axis of the target position: `pixels.subpixels` moved by `distance`, forward or back,
/// keeping only the pixel word.
///
/// Past the move, the original adds or takes one more pixel unless its zero flag is set. Without
/// a carry (forward) or borrow (back) that flag comes from the subpixel sum; with one, from the
/// target pixel word the carry just stepped (`INC`/`DEC $189A`), which is set only when the
/// target wraps to 0. The log reads the intended test as the target's fraction being nonzero.
fn target(pixels: u16, subpixels: u16, distance: Speed, forward: bool) -> u16 {
    let (fraction, carried) = if forward {
        subpixels.overflowing_add(distance.subpixels())
    } else {
        subpixels.overflowing_sub(distance.subpixels())
    };
    let step = |word: u16| {
        if forward {
            word.wrapping_add(1)
        } else {
            word.wrapping_sub(1)
        }
    };
    let moved = if forward {
        pixels.wrapping_add(distance.pixels())
    } else {
        pixels.wrapping_sub(distance.pixels())
    };
    let moved = if carried { step(moved) } else { moved };
    let zero = if carried { moved == 0 } else { fraction == 0 };
    if zero { moved } else { step(moved) }
}

/// Whether two boxes overlap along one axis, as `$A0:A9F5` tests it: the centres' distance
/// below the enemy's radius, or past it by less than Samus's.
const fn overlaps(enemy: u16, samus: u16, enemy_radius: u16, samus_radius: u16) -> bool {
    let apart = magnitude(enemy.wrapping_sub(samus));
    match apart.checked_sub(enemy_radius) {
        None => true,
        Some(beyond) => beyond < samus_radius,
    }
}

/// The gap from her leading edge to the enemy's facing edge before the move (`$A0:AA37`,
/// `$A0:AA5C`, `$A0:AA7E`, `$A0:AAA0`): 0 when they touch.
const fn gap(movement: &Move, enemy: &Obstacle) -> u16 {
    let samus = movement.position;
    let Body {
        x_radius, y_radius, ..
    } = movement.body;
    let (samus_x, samus_y) = (samus.x.pixels, samus.y.pixels);
    let (enemy_x, enemy_y) = (enemy.position.x.pixels, enemy.position.y.pixels);
    match movement.direction {
        Direction::Left => samus_x
            .wrapping_sub(x_radius)
            .wrapping_sub(enemy_x.wrapping_add(enemy.radius_x)),
        Direction::Right => enemy_x
            .wrapping_sub(enemy.radius_x)
            .wrapping_sub(samus_x.wrapping_add(x_radius)),
        Direction::Up => samus_y
            .wrapping_sub(y_radius)
            .wrapping_sub(enemy_y.wrapping_add(enemy.radius_y)),
        Direction::Down => enemy_y
            .wrapping_sub(enemy.radius_y)
            .wrapping_sub(samus_y.wrapping_add(y_radius)),
    }
}

/// "Samus / solid enemy collision detection" (`$A0:A8F0`) over the interactive enemies in
/// list order.
///
/// The caller applies the result: a stopped move goes on by [`Contact::distance`] with no
/// block collision, and [`Contact::Touching`] zeroes her Y subposition first
/// ([`SolidEnemies::check`] does both of the check's own parts).
// open: the per-direction enemy and distance words (`$182C`, `$1834`) and the debug copies
// (`$184A` to `$185A`) the original writes on a hit have no reader in the engine yet.
pub fn detect(
    movement: &Move,
    interactive: impl IntoIterator<Item = (SlotIndex<Enemy>, Obstacle)>,
) -> Contact {
    let position = movement.position;
    let (target_x, target_y) = match movement.direction {
        Direction::Left | Direction::Right => (
            target(
                position.x.pixels,
                position.x.subpixels,
                movement.distance,
                movement.direction == Direction::Right,
            ),
            position.y.pixels,
        ),
        Direction::Up | Direction::Down => (
            position.x.pixels,
            target(
                position.y.pixels,
                position.y.subpixels,
                movement.distance,
                movement.direction == Direction::Down,
            ),
        ),
    };
    for (slot, enemy) in interactive {
        if !(enemy.frozen || enemy.solid)
            || !overlaps(
                enemy.position.x.pixels,
                target_x,
                enemy.radius_x,
                movement.body.x_radius,
            )
            || !overlaps(
                enemy.position.y.pixels,
                target_y,
                enemy.radius_y,
                movement.body.y_radius,
            )
        {
            continue;
        }
        let gap = gap(movement, &enemy);
        if gap == 0 {
            return Contact::Touching { slot };
        }
        if positive(gap) {
            return Contact::Blocked {
                slot,
                distance: gap,
            };
        }
    }
    Contact::Clear
}

/// A table of one slot, 0, holding an enemy 8 pixels each way around (`x`, `y`), solid to
/// Samus when `solid`.
#[cfg(test)]
pub(crate) fn one_enemy(x: u16, y: u16, solid: bool) -> Vec<Slot<EnemyCommon, EnemyBody>> {
    let mut enemy = EnemyCommon {
        radius_x: 8,
        radius_y: 8,
        ..EnemyCommon::default()
    };
    enemy.position.x.pixels = x;
    enemy.position.y.pixels = y;
    if solid {
        enemy.properties.insert(EnemyProperty::SolidToSamus);
    }
    vec![Slot {
        common: enemy,
        ..Slot::default()
    }]
}

/// The interactive list naming slot 0 alone.
#[cfg(test)]
pub(crate) const FIRST: [SlotIndex<Enemy>; 1] = [SlotIndex(0, core::marker::PhantomData)];

#[cfg(test)]
mod tests;
