//! The collision domain as motion asks it: a move by a distance, and what stopped it.

use crate::model::num::Fixed;
use crate::model::samus::ground::{Position, Radius};

/// Her extent for a collision request.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Body {
    /// Her centre.
    pub position: Position,
    /// Her radius.
    pub radius: Radius,
}

/// Whether a move tests solid enemies.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Enemies {
    /// Solid enemies stop her.
    Test,
    /// They don't.
    Skip,
}

/// The side she was stopped on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Side {
    /// Her left.
    Left,
    /// Her right.
    Right,
    /// Below her.
    Below,
}

/// Why a move ended short.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Stop {
    /// The side.
    pub side: Side,
    /// The subpixel the move's axis takes before the distance is added (`FloorFlushSubpixel`).
    pub subpixel: Option<u16>,
}

/// A move's result.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Moved {
    /// The signed displacement made.
    pub distance: Fixed,
    /// Set when she was stopped.
    pub stop: Option<Stop>,
}

impl Moved {
    pub(crate) fn applied(self, coordinate: Fixed) -> Fixed {
        let flush = self.stop.and_then(|stop| stop.subpixel);
        flush
            .map_or(coordinate, |sub| Fixed::new(coordinate.pixel(), sub))
            .plus(self.distance)
    }
}

/// The collision domain, as motion asks it.
pub trait Collision {
    /// Moves `body` sideways by the signed `distance`.
    fn move_horizontal(&mut self, body: Body, distance: Fixed, enemies: Enemies) -> Moved;
    /// Moves `body` down by `distance`.
    fn move_down(&mut self, body: Body, distance: Fixed, enemies: Enemies) -> Moved;
    /// Whether a solid enemy is within `distance` on `side` of `body`.
    fn enemy_blocks(&self, body: Body, side: Side, distance: Fixed) -> bool;
}
