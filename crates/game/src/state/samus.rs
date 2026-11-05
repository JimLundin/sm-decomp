//! Samus as orthogonal facts: where she is, how she moves, what she has, how she's drawn.

use canonical::Canonical;

use super::slots::{Cursor, Frames};

/// A pixel position or speed with its subpixel fraction, as the original keeps them.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Fixed {
    /// Whole pixels.
    pub pixels: u16,
    /// 1/65536ths of a pixel.
    pub subpixels: u16,
}

/// A room position.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Point {
    /// Rightward from the room's left edge.
    pub x: Fixed,
    /// Downward from the room's top edge.
    pub y: Fixed,
}

/// Which way Samus faces.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Facing {
    /// Toward the left edge.
    Left,
    /// Toward the right edge.
    #[default]
    Right,
}

/// Where Samus aims her arm cannon.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Aim {
    /// Straight ahead.
    #[default]
    Forward,
    /// Up and ahead.
    UpForward,
    /// Straight up.
    Up,
    /// Down and ahead.
    DownForward,
    /// Straight down.
    Down,
}

/// How a jump began, which decides its rise and what it can turn into.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Jump {
    /// Upright, aiming.
    #[default]
    Normal,
    /// Somersaulting, from a running jump.
    Spin,
}

/// Samus's movement state, each with the data only it uses.
// open: the full set. The original's pose table (bank $91 "Samus pose definitions") gives
// each pose a movement type, a direction and an aim; the decoder's pose table decides
// whether every pose maps onto (movement, facing, aim) or some need a variant of their own
// (transitions, grabbed, elevator, knockback, wall jump, crystal flash, ...).
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Movement {
    /// On the ground, still.
    #[default]
    Standing,
    /// On the ground, moving.
    Running {
        /// Horizontal speed.
        speed: Fixed,
    },
    /// Crouched.
    Crouching,
    /// In the air, rising or falling from a jump.
    Jumping {
        /// How it began.
        jump: Jump,
        /// Vertical speed.
        vertical: Fixed,
        /// Whether the rise is still under the jump button's control.
        rising: bool,
    },
    /// In the air without a jump.
    Falling {
        /// Vertical speed.
        vertical: Fixed,
    },
    /// Touching down: the landing frames before Samus can act.
    Landing {
        /// Frames of the landing left.
        left: Frames,
    },
    /// Rolled into the morph ball.
    MorphBall {
        /// Horizontal speed.
        speed: Fixed,
        /// Whether the ball is in the air.
        airborne: bool,
    },
}

/// The items Samus can collect and equip, as a typed flag set.
// open: the full set (the original's equipment and beam words); slice 0 needs only the
// morph ball. Charge beam, beams and their combination rules join with weapons.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Items {
    /// The morph ball.
    pub morph_ball: bool,
    /// Bombs.
    pub bombs: bool,
}

/// A quantity with a ceiling: energy, a reserve, an ammunition type.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Gauge {
    /// What's left.
    pub current: u16,
    /// The ceiling.
    pub max: u16,
}

/// What Samus has.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Equipment {
    /// Collected.
    pub collected: Items,
    /// Collected and switched on.
    pub equipped: Items,
    /// Energy.
    pub energy: Gauge,
    /// Missiles.
    pub missiles: Gauge,
}

/// The family marker for Samus's animation delay lists, a single-cursor consumer (J14).
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SamusAnimation;

/// Samus.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Samus {
    /// Where she is.
    pub position: Point,
    /// Which way she faces.
    pub facing: Facing,
    /// Where she aims.
    pub aim: Aim,
    /// How she moves.
    pub movement: Movement,
    /// What she has.
    pub equipment: Equipment,
    /// Frames of invincibility left after a hit.
    pub invincibility: Frames,
    /// Her animation delay list.
    pub animation: Cursor<SamusAnimation>,
}
