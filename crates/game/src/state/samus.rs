//! Samus as orthogonal facts: where she is, how she moves, what she has, how she's drawn.

use canonical::Canonical;

use super::slots::{FlagSet, Frames};

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

pub use data::movement::Speed;
pub use data::pose::{MovementType, PoseFacing as Facing, PoseId};

/// How Samus's base speed changes each frame and which way it moves her (bank $90 log,
/// `$90:9A7E` "Calculate Samus X base speed", `$90:8EA9` "Move Samus horizontally").
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Acceleration {
    /// Speeding up to the movement type's cap, the way she faces.
    #[default]
    Accelerating,
    /// Slowing down against the way she faces, the momentum of a turn.
    TurningAround,
    /// Slowing down the way she faces, after letting go.
    Decelerating,
}

/// Samus's horizontal motion: the speeds the X movement routines (bank $90 log, `$90:8E64`
/// "Samus X movement") read and keep from frame to frame.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Horizontal {
    /// The speed the movement type's acceleration builds, capped by its speed table entry.
    pub base: Speed,
    /// The extra speed holding the run button adds while running (`$90:973E`).
    pub extra_run: Speed,
    /// How the base speed changes.
    pub acceleration: Acceleration,
    /// Whether she has started running with the run button held since the last speed boost
    /// cancel: the extra run speed survives leaving the running movement only then.
    pub momentum: bool,
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

/// Samus's movement state: one variant per movement type of the original's movement
/// handler (bank $90), each with the data only it uses.
///
/// The landing frames are not a movement type: the landing poses (bank $91 log, poses $A4
/// to $A7, $E0 to $E5) are standing ones, so behaviour that keys on landing reads
/// [`Samus::pose`].
// open: the jumps, falls and wall jumps keep `Horizontal` too, and the jump's rise; they
// join with C3b.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Movement {
    /// On the ground, still: a landing's extra run speed still moves her one frame.
    Standing {
        /// Horizontal motion.
        speed: Horizontal,
    },
    /// On the ground, moving.
    Running {
        /// Horizontal motion.
        speed: Horizontal,
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
        /// Horizontal motion.
        speed: Horizontal,
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

impl Default for Movement {
    fn default() -> Self {
        Self::Standing {
            speed: Horizontal::default(),
        }
    }
}

impl From<MovementType> for Movement {
    /// The movement a type starts, its data at rest.
    fn from(kind: MovementType) -> Self {
        let jumping = |jump| Self::Jumping {
            jump,
            vertical: Fixed::default(),
            rising: false,
        };
        let turning = |place| Self::TurningAround {
            place,
            speed: Horizontal::default(),
        };
        let morph_ball = |airborne| Self::MorphBall {
            speed: Fixed::default(),
            airborne,
        };
        match kind {
            MovementType::Standing => Self::default(),
            MovementType::Running => Self::Running {
                speed: Horizontal::default(),
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
            MovementType::TurningAroundOnGround => turning(Turn::OnGround),
            MovementType::TurningAroundJumping => turning(Turn::Jumping),
            MovementType::TurningAroundFalling => turning(Turn::Falling),
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
    /// The horizontal motion this movement keeps, if it keeps one.
    #[must_use]
    pub const fn horizontal(self) -> Option<Horizontal> {
        match self {
            Self::Standing { speed }
            | Self::Running { speed }
            | Self::TurningAround { speed, .. } => Some(speed),
            Self::Crouching
            | Self::Jumping { .. }
            | Self::Falling { .. }
            | Self::MorphBall { .. }
            | Self::SpringBall { .. }
            | Self::KnockbackOrCrystalFlashEnding
            | Self::Transition
            | Self::Moonwalking
            | Self::WallJumping
            | Self::RanIntoWall
            | Self::Grappling
            | Self::DamageBoost
            | Self::GrabbedByDraygon
            | Self::Shinespark
            | Self::Unused(_) => None,
        }
    }

    /// The horizontal motion this movement keeps, to change, if it keeps one.
    pub const fn horizontal_mut(&mut self) -> Option<&mut Horizontal> {
        match self {
            Self::Standing { speed }
            | Self::Running { speed }
            | Self::TurningAround { speed, .. } => Some(speed),
            Self::Crouching
            | Self::Jumping { .. }
            | Self::Falling { .. }
            | Self::MorphBall { .. }
            | Self::SpringBall { .. }
            | Self::KnockbackOrCrystalFlashEnding
            | Self::Transition
            | Self::Moonwalking
            | Self::WallJumping
            | Self::RanIntoWall
            | Self::Grappling
            | Self::DamageBoost
            | Self::GrabbedByDraygon
            | Self::Shinespark
            | Self::Unused(_) => None,
        }
    }

    /// The movement type the original keeps for this movement, byte-exact.
    #[must_use]
    pub const fn kind(self) -> MovementType {
        match self {
            Self::Standing { .. } => MovementType::Standing,
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
                ..
            } => MovementType::TurningAroundOnGround,
            Self::TurningAround {
                place: Turn::Jumping,
                ..
            } => MovementType::TurningAroundJumping,
            Self::TurningAround {
                place: Turn::Falling,
                ..
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

impl Item {
    /// The item's bit in an [`Items`] set: its place in declaration order.
    #[must_use]
    pub const fn bit(self) -> u32 {
        let place = match self {
            Self::VariaSuit => 0,
            Self::SpringBall => 1,
            Self::MorphBall => 2,
            Self::ScrewAttack => 3,
            Self::GravitySuit => 4,
            Self::HiJumpBoots => 5,
            Self::SpaceJump => 6,
            Self::Bombs => 7,
            Self::SpeedBooster => 8,
            Self::GrappleBeam => 9,
            Self::XRayScope => 10,
        };
        1 << place
    }
}

impl Items {
    /// Whether the set holds `item`.
    #[must_use]
    pub const fn contains(&self, item: Item) -> bool {
        (self.bits & item.bit()) != 0
    }
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
    /// The reserve tanks' energy.
    pub reserve: Gauge,
    /// Missiles.
    pub missiles: Gauge,
    /// Super missiles.
    pub super_missiles: Gauge,
    /// Power bombs.
    pub power_bombs: Gauge,
    /// How the reserve tanks refill energy.
    pub reserve_mode: ReserveMode,
}

/// How the reserve tanks refill energy (bank `$80` log, `$80:9B4E`: the HUD draws AUTO only
/// in the automatic mode).
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ReserveMode {
    /// No reserve tank collected yet.
    #[default]
    None,
    /// They refill energy when it runs out.
    Automatic,
    /// The player refills from the equipment screen.
    Manual,
}

pub use data::pose::animation::AnimationFrame;

/// The frames an animation frame still shows, as the original counts them: a word that
/// goes negative, since a decrement from 0 gives $FFFF and the frame then advances
/// (bank $90 log, `$90:8000`).
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AnimationTimer(pub u16);

impl AnimationTimer {
    /// Whether the frame still shows: the timer is above zero, read as signed.
    #[must_use]
    pub const fn running(self) -> bool {
        self.0 != 0 && (self.0 & 0x8000) == 0
    }
}

/// The frame a new pose's animation starts on (bank $91 log, `$91:FB08`).
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum NewPoseFrame {
    /// From this frame.
    From(AnimationFrame),
    /// Keep the frame and its timer: running from running, a ball from a ball (the
    /// original's $8000).
    Preserve,
}

impl Default for NewPoseFrame {
    fn default() -> Self {
        Self::From(AnimationFrame::default())
    }
}

/// Samus's animation: where her pose's delay list is and how long its frame still shows.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Animation {
    /// The frame: a byte offset into the pose's delay list.
    pub frame: AnimationFrame,
    /// How long the frame still shows.
    pub timer: AnimationTimer,
    /// The frame the next pose change starts the animation on.
    pub new_pose_frame: NewPoseFrame,
}

/// What the pose update does after it takes the super special prospective pose (bank $91
/// log, the super special prospective pose change commands).
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SuperSpecialCommand {
    /// Nothing (0).
    #[default]
    None,
    /// 1 (`$91:F31D`): knockback finished.
    KnockbackFinished,
    /// 2 (`$91:F34E`): shinespark finished.
    ShinesparkFinished,
    /// 3 (`$91:F36E`): the transition animation finished.
    TransitionAnimationFinished,
    /// 4 (`$91:F37C`): Samus to the screen's centre.
    CentreOnScreen,
    /// 5 (`$91:F397`): Samus 5 pixels down.
    MoveDown,
    /// 6 (`$91:F3A5`): start the grapple wall jump.
    StartGrappleWallJump,
    /// 7 (`$91:F3AA`): start the release from the grapple swing.
    StartReleaseFromGrappleSwing,
    /// 8 (`$91:F3FD`): knockback and the transition animation both finished.
    KnockbackAndTransitionAnimationFinished,
}

/// The super special prospective pose request the frame made.
///
/// The pose update takes it before every other (bank $91 log, `$91:EB88`). The original clears it at the start
/// of every frame (`$90:E695`) and it keeps the last frame's until then.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SuperSpecial {
    /// The pose asked for.
    pub pose: Option<PoseId>,
    /// What the update does after taking it.
    pub command: SuperSpecialCommand,
}

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
    /// Her animation.
    pub animation: Animation,
    /// The super special prospective pose request.
    pub super_special: SuperSpecial,
}
