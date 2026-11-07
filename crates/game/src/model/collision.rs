//! The collision domain as motion asks it: a move by a distance on a heading, and what stopped
//! it; a look ahead for a solid enemy; and a probe for a wall.

use crate::model::num::Fixed;
use crate::model::samus::motion::{Position, Radius};

/// Her extent for a collision request.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Body {
    /// Her centre.
    pub position: Position,
    /// Her radius.
    pub radius: Radius,
}

/// The side a request tests. The distance's sign, not the heading, decides where she moves.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Heading {
    /// Her left.
    Left,
    /// Her right.
    Right,
    /// Above her.
    Up,
    /// Below her.
    Down,
}

/// Whether a move tests solid enemies.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Enemies {
    /// Solid enemies stop her.
    Test,
    /// They don't.
    Skip,
}

/// What ended a move short.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Blocker {
    /// A solid block; the move's axis takes `subpixel` before the distance (`FloorFlushSubpixel`).
    Block {
        /// The subpixel written.
        subpixel: u16,
    },
    /// A solid enemy; one she already touched also clears her Y subpixel
    /// (`EnemyTouchZeroesYSubpixel`).
    Enemy {
        /// She was already touching it.
        touching: bool,
    },
    /// A slope.
    Slope,
}

/// Why a move ended short.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Stop {
    /// The side tested.
    pub side: Heading,
    /// What stopped her.
    pub blocker: Blocker,
}

/// A move's result.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Moved {
    /// The signed displacement made.
    pub distance: Fixed,
    /// Set when she was stopped.
    pub stop: Option<Stop>,
}

/// A solid enemy found ahead.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct EnemyHit {
    /// She already touches it.
    pub touching: bool,
}

/// A wall found by a probe.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WallHit {
    /// How far it is, in pixels.
    pub distance_px: u16,
    /// The X subpixel a block writes without moving her (`WallProbeSubpixel`).
    pub subpixel: Option<u16>,
}

impl Moved {
    /// Applies the move on `heading`'s axis to `position`; true when she was stopped.
    pub(crate) fn apply(self, position: &mut Position, heading: Heading) -> bool {
        let axis = match heading {
            Heading::Left | Heading::Right => &mut position.x,
            Heading::Up | Heading::Down => &mut position.y,
        };
        let blocker = self.stop.map(|stop| stop.blocker);
        if let Some(Blocker::Block { subpixel }) = blocker {
            *axis = axis.with_sub(subpixel);
        }
        *axis = axis.plus(self.distance);
        if blocker == Some(Blocker::Enemy { touching: true }) {
            position.y = position.y.with_sub(0);
        }
        blocker.is_some()
    }
}

/// The collision domain, as motion asks it.
pub trait Collision {
    /// Moves `body` sideways by the signed `distance`, testing the `heading` side.
    fn move_horizontal(
        &mut self,
        body: Body,
        heading: Heading,
        distance: Fixed,
        enemies: Enemies,
    ) -> Moved;
    /// Moves `body` down by `distance`, testing the `heading` side.
    fn move_down(
        &mut self,
        body: Body,
        heading: Heading,
        distance: Fixed,
        enemies: Enemies,
    ) -> Moved;
    /// Moves `body` up by `distance`, testing solid enemies; the result's distance is at most 0.
    fn move_up(&mut self, body: Body, distance: Fixed) -> Moved;
    /// A solid enemy within `distance` on `heading`.
    fn enemy_ahead(&mut self, body: Body, heading: Heading, distance: Fixed) -> Option<EnemyHit>;
    /// A solid enemy, then a block, within `reach` pixels on `heading`; nothing moves.
    fn probe_wall(&mut self, body: Body, heading: Heading, reach: u16) -> Option<WallHit>;
}
