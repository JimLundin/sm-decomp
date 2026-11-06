//! Samus as orthogonal facts: where she is, how she moves, what she has, how she's drawn.

use canonical::Canonical;

use super::slots::{Cursor, FlagSet, Frames};

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

pub use data::pose::{MovementType, PoseFacing as Facing, PoseId};

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

/// Samus's movement state: one variant per movement type of the original's movement
/// handler (bank $90), each with the data only it uses.
///
/// The landing frames are not a movement type: the landing poses (bank $91 log, poses $A4
/// to $A7, $E0 to $E5) are standing ones, so behaviour that keys on landing reads
/// [`Samus::pose`].
// open: the data each variant carries joins with movement (C3): the speeds the original
// keeps in its X and Y speed words, and the jump's rise.
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
    /// Rolled into the morph ball.
    MorphBall {
        /// Horizontal speed.
        speed: Fixed,
        /// Whether the ball is falling.
        airborne: bool,
    },
    /// The spring ball.
    SpringBall {
        /// Where the ball is.
        phase: SpringBall,
    },
    /// Knocked back, or ending a crystal flash.
    KnockbackOrCrystalFlashEnding,
    /// Turning around.
    TurningAround {
        /// Where she turns.
        place: Turn,
    },
    /// Crouching, standing, morphing or unmorphing.
    Transition,
    /// Moonwalking.
    Moonwalking,
    /// Wall jumping.
    WallJumping,
    /// Ran into a wall.
    RanIntoWall,
    /// Grappling.
    Grappling,
    /// A damage boost.
    DamageBoost,
    /// Grabbed by Draygon.
    GrabbedByDraygon,
    /// Shinesparking, crystal flashing, drained by a metroid or hit by Mother Brain.
    Shinespark,
    /// A movement type only unused poses have.
    Unused(UnusedMovement),
}

/// The movement types only unused poses have (7, 9, $B, $C, $D).
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum UnusedMovement {
    /// Type 7.
    #[default]
    Unused07,
    /// Type 9.
    Unused09,
    /// Type $B.
    Unused0B,
    /// Type $C.
    Unused0C,
    /// Type $D.
    Unused0D,
}

/// Where a spring ball is.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SpringBall {
    /// On the ground.
    #[default]
    OnGround,
    /// In the air.
    InAir,
    /// Falling.
    Falling,
}

/// Where Samus turns around.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Turn {
    /// On the ground.
    #[default]
    OnGround,
    /// In a jump.
    Jumping,
    /// Falling.
    Falling,
}

impl From<MovementType> for Movement {
    /// The movement a type starts, its data at rest.
    fn from(kind: MovementType) -> Self {
        let jumping = |jump| Self::Jumping {
            jump,
            vertical: Fixed::default(),
            rising: false,
        };
        let morph_ball = |airborne| Self::MorphBall {
            speed: Fixed::default(),
            airborne,
        };
        match kind {
            MovementType::Standing => Self::Standing,
            MovementType::Running => Self::Running {
                speed: Fixed::default(),
            },
            MovementType::NormalJumping => jumping(Jump::Normal),
            MovementType::SpinJumping => jumping(Jump::Spin),
            MovementType::MorphBallOnGround => morph_ball(false),
            MovementType::MorphBallFalling => morph_ball(true),
            MovementType::Crouching => Self::Crouching,
            MovementType::Falling => Self::Falling {
                vertical: Fixed::default(),
            },
            MovementType::KnockbackOrCrystalFlashEnding => Self::KnockbackOrCrystalFlashEnding,
            MovementType::TurningAroundOnGround => Self::TurningAround {
                place: Turn::OnGround,
            },
            MovementType::TurningAroundJumping => Self::TurningAround {
                place: Turn::Jumping,
            },
            MovementType::TurningAroundFalling => Self::TurningAround {
                place: Turn::Falling,
            },
            MovementType::CrouchingTransition => Self::Transition,
            MovementType::Moonwalking => Self::Moonwalking,
            MovementType::SpringBallOnGround => Self::SpringBall {
                phase: SpringBall::OnGround,
            },
            MovementType::SpringBallInAir => Self::SpringBall {
                phase: SpringBall::InAir,
            },
            MovementType::SpringBallFalling => Self::SpringBall {
                phase: SpringBall::Falling,
            },
            MovementType::WallJumping => Self::WallJumping,
            MovementType::RanIntoWall => Self::RanIntoWall,
            MovementType::Grappling => Self::Grappling,
            MovementType::DamageBoost => Self::DamageBoost,
            MovementType::GrabbedByDraygon => Self::GrabbedByDraygon,
            MovementType::Shinespark => Self::Shinespark,
            MovementType::Unused07 => Self::Unused(UnusedMovement::Unused07),
            MovementType::Unused09 => Self::Unused(UnusedMovement::Unused09),
            MovementType::Unused0B => Self::Unused(UnusedMovement::Unused0B),
            MovementType::Unused0C => Self::Unused(UnusedMovement::Unused0C),
            MovementType::Unused0D => Self::Unused(UnusedMovement::Unused0D),
        }
    }
}

impl Movement {
    /// The movement type the original keeps for this movement, byte-exact.
    #[must_use]
    pub const fn kind(self) -> MovementType {
        match self {
            Self::Standing => MovementType::Standing,
            Self::Running { .. } => MovementType::Running,
            Self::Crouching => MovementType::Crouching,
            Self::Jumping {
                jump: Jump::Normal, ..
            } => MovementType::NormalJumping,
            Self::Jumping {
                jump: Jump::Spin, ..
            } => MovementType::SpinJumping,
            Self::Falling { .. } => MovementType::Falling,
            Self::MorphBall {
                airborne: false, ..
            } => MovementType::MorphBallOnGround,
            Self::MorphBall { airborne: true, .. } => MovementType::MorphBallFalling,
            Self::SpringBall {
                phase: SpringBall::OnGround,
            } => MovementType::SpringBallOnGround,
            Self::SpringBall {
                phase: SpringBall::InAir,
            } => MovementType::SpringBallInAir,
            Self::SpringBall {
                phase: SpringBall::Falling,
            } => MovementType::SpringBallFalling,
            Self::KnockbackOrCrystalFlashEnding => MovementType::KnockbackOrCrystalFlashEnding,
            Self::TurningAround {
                place: Turn::OnGround,
            } => MovementType::TurningAroundOnGround,
            Self::TurningAround {
                place: Turn::Jumping,
            } => MovementType::TurningAroundJumping,
            Self::TurningAround {
                place: Turn::Falling,
            } => MovementType::TurningAroundFalling,
            Self::Transition => MovementType::CrouchingTransition,
            Self::Moonwalking => MovementType::Moonwalking,
            Self::WallJumping => MovementType::WallJumping,
            Self::RanIntoWall => MovementType::RanIntoWall,
            Self::Grappling => MovementType::Grappling,
            Self::DamageBoost => MovementType::DamageBoost,
            Self::GrabbedByDraygon => MovementType::GrabbedByDraygon,
            Self::Shinespark => MovementType::Shinespark,
            Self::Unused(UnusedMovement::Unused07) => MovementType::Unused07,
            Self::Unused(UnusedMovement::Unused09) => MovementType::Unused09,
            Self::Unused(UnusedMovement::Unused0B) => MovementType::Unused0B,
            Self::Unused(UnusedMovement::Unused0C) => MovementType::Unused0C,
            Self::Unused(UnusedMovement::Unused0D) => MovementType::Unused0D,
        }
    }
}

/// A pose with the facing and movement type its definition gave, as the original copies
/// them beside the pose word (bank $90 log).
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PoseRecord {
    /// The pose.
    pub pose: PoseId,
    /// Its facing.
    pub facing: Facing,
    /// Its movement type.
    pub movement: MovementType,
}

/// An item Samus can collect and equip.
// open: the beams and the charge beam (the original's beam words), with their combination
// rules, join with weapons.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Item {
    /// The Varia suit.
    VariaSuit,
    /// The spring ball.
    SpringBall,
    /// The morph ball.
    MorphBall,
    /// The screw attack.
    ScrewAttack,
    /// The gravity suit.
    GravitySuit,
    /// The hi-jump boots.
    HiJumpBoots,
    /// The space jump.
    SpaceJump,
    /// Bombs.
    Bombs,
    /// The speed booster.
    SpeedBooster,
    /// The grapple beam.
    GrappleBeam,
    /// The X-ray scope.
    XRayScope,
}

/// A set of items.
pub type Items = FlagSet<Item>;

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
    /// The reserve tanks' energy.
    pub reserve: Gauge,
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
    /// Her pose: the pose definitions table (`data::pose`) gives its facing, movement type,
    /// shot direction and radius, and behaviour that keys on one pose (landing, the
    /// elevator's facing forward) reads it.
    pub pose: PoseId,
    /// The pose before the last pose change, and what it gave.
    pub previous: PoseRecord,
    /// The last pose that differed from the current one, and what it gave.
    pub last_different: PoseRecord,
    /// Which way she faces.
    pub facing: Facing,
    /// Where she aims.
    // open: the aim is the pose's shot direction (`data::pose::Shot`); whether the State
    // keeps it beside the pose or reads it from the table joins with weapons.
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
