//! The values the model reads. `Rules::REFERENCE` holds the spec's vanilla numbers.

use clean_types::{Fixed, below};

/// What she is doing, as `SpeedBuildsAndBrakes`'s table rows.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Activity {
    /// Running.
    Running,
    /// Rolling as a ball.
    Rolling,
    /// Moonwalking.
    Moonwalking,
    /// A straight jump or the spring ball's hop.
    StraightJump,
    /// A spin jump or a wall jump.
    SpinJump,
    /// Falling, upright or as a ball.
    Falling,
    /// Turning round.
    Turning,
}

/// `Jumping`: which rise a jump speed is for.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum JumpKind {
    /// A jump, a spin jump, the spring ball's hop or a space jump.
    Jump,
    /// A wall jump.
    WallJump,
}

/// One row of `SpeedBuildsAndBrakes`'s table.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Limits {
    /// Speeds up by this a frame.
    pub step: Fixed,
    /// Top speed.
    pub top: Fixed,
}

/// How the top speed is tested (`WordwiseTopSpeed`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SpeedCap {
    /// Vanilla: part by part.
    Wordwise,
    /// The smaller of the two.
    Exact,
}

impl SpeedCap {
    /// The speed after the top is tested, and whether it took the top.
    #[must_use]
    pub fn apply(self, speed: Fixed, top: Fixed) -> (Fixed, bool) {
        let keep = match self {
            // WordwiseTopSpeed: pixel parts first, then subpixels when they are equal.
            Self::Wordwise => {
                below(speed.pixel(), top.pixel())
                    || (speed.pixel() == top.pixel()
                        && (speed.sub() == top.sub() || below(speed.sub(), top.sub())))
            }
            // WordwiseTopSpeed, `Exact`: the smaller.
            Self::Exact => speed <= top,
        };
        if keep { (speed, false) } else { (top, true) }
    }
}

/// The values the model reads.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Rules {
    /// `WordwiseTopSpeed`.
    pub speed_cap: SpeedCap,
    /// `SpeedBuildsAndBrakes`: running.
    pub running: Limits,
    /// `SpeedBuildsAndBrakes`: rolling as a ball.
    pub rolling: Limits,
    /// `SpeedBuildsAndBrakes`: moonwalking.
    pub moonwalking: Limits,
    /// `SpeedBuildsAndBrakes`: a straight jump or the spring ball's hop.
    pub straight_jump: Limits,
    /// `SpeedBuildsAndBrakes`: a spin jump or a wall jump.
    pub spin_jump: Limits,
    /// `SpeedBuildsAndBrakes`: falling.
    pub falling: Limits,
    /// `SpeedBuildsAndBrakes`: turning round.
    pub turning: Limits,
    /// `SpeedBuildsAndBrakes`: lost a frame carrying on along the ground.
    pub brake: Fixed,
    /// `Dash`: gained a frame.
    pub dash_step: Fixed,
    /// `Dash`: its cap.
    pub dash_cap: Fixed,
    /// `Jumping`: a jump.
    pub jump: Fixed,
    /// `Jumping`: a jump with hi-jump.
    pub hi_jump: Fixed,
    /// `Jumping`: a wall jump.
    pub wall_jump: Fixed,
    /// `Jumping`: a wall jump with hi-jump.
    pub hi_wall_jump: Fixed,
    /// `Jumping`: a ball bounce.
    pub bounce: Fixed,
    /// `Jumping`: the crouch lift.
    pub crouch_lift: Fixed,
    /// `PadChart`: crouched, she stands at once this far up.
    pub stand_up: Fixed,
    /// `Jumping`: the space jump's window of fall speeds read as 8.8, from and up to.
    pub space_jump_window: (u16, u16),
    /// `RisingAndFalling`: gravity.
    pub gravity: Fixed,
    /// `RisingAndFalling`: the pixel part at which a fall stops growing.
    pub fall_settle_px: u16,
    /// `Landing`: the slowest landing that bounces the ball, in px.
    pub bounce_from_px: u16,
    /// `Landing`: the second bounce is the bounce speed less this.
    pub second_bounce_less: Fixed,
    /// `FollowingTheGround` and `SlopesCarry`: the pull beyond her speed, and on a slope.
    pub ground_pull: Fixed,
    /// `WallJump`: how near a wall must be.
    pub wall_reach: Fixed,
    /// `BlocksStopFlush`: the longest horizontal move, in px.
    pub move_limit_px: u16,
    /// §8: a block's size, in px.
    pub block_px: u16,
    /// §8: a stepped block's quarter's size, in px.
    pub quarter_px: u16,
    /// `AimingAndShooting`: the step forward.
    pub aim_step: Fixed,
    /// `RidingTheElevator`: the descent a frame.
    pub elevator: Fixed,
}

const fn limits(step: u32, top: u32) -> Limits {
    Limits {
        step: Fixed::from_bits(step),
        top: Fixed::from_bits(top),
    }
}

impl Rules {
    /// The vanilla game, with the spec's numbers.
    pub const REFERENCE: Self = Self {
        speed_cap: SpeedCap::Wordwise,
        running: limits(0x0000_3000, 0x0002_C000),
        rolling: limits(0x0000_C000, 0x0003_4000),
        moonwalking: limits(0x0000_C000, 0x0000_8000),
        straight_jump: limits(0x0000_C000, 0x0001_4000),
        spin_jump: limits(0x0000_C000, 0x0001_6000),
        falling: limits(0x0000_C000, 0x0001_0000),
        turning: limits(0x0000_C000, 0x0000_0000),
        brake: Fixed::from_bits(0x0000_8000),
        dash_step: Fixed::from_bits(0x0000_1000),
        dash_cap: Fixed::from_bits(0x0002_0000),
        jump: Fixed::from_bits(0x0004_E000),
        hi_jump: Fixed::from_bits(0x0006_0000),
        wall_jump: Fixed::from_bits(0x0004_A000),
        hi_wall_jump: Fixed::from_bits(0x0005_8000),
        bounce: Fixed::from_bits(0x0001_0000),
        crouch_lift: Fixed::px(10),
        stand_up: Fixed::px(5),
        space_jump_window: (0x0280, 0x0500),
        gravity: Fixed::from_bits(0x0000_1C00),
        fall_settle_px: 5,
        bounce_from_px: 3,
        second_bounce_less: Fixed::px(1),
        ground_pull: Fixed::px(1),
        wall_reach: Fixed::px(8),
        move_limit_px: 15,
        block_px: 16,
        quarter_px: 8,
        aim_step: Fixed::px(1),
        elevator: Fixed::px(1),
    };

    /// `Jumping`: the speed a jump or a wall jump starts her rising at, with or without hi-jump.
    #[must_use]
    pub const fn jump_speed(&self, kind: JumpKind, hi_jump: bool) -> Fixed {
        match (kind, hi_jump) {
            (JumpKind::Jump, false) => self.jump,
            (JumpKind::Jump, true) => self.hi_jump,
            (JumpKind::WallJump, false) => self.wall_jump,
            (JumpKind::WallJump, true) => self.hi_wall_jump,
        }
    }

    /// The row of `SpeedBuildsAndBrakes`'s table for what she is doing.
    #[must_use]
    pub const fn limits(&self, activity: Activity) -> Limits {
        match activity {
            Activity::Running => self.running,
            Activity::Rolling => self.rolling,
            Activity::Moonwalking => self.moonwalking,
            Activity::StraightJump => self.straight_jump,
            Activity::SpinJump => self.spin_jump,
            Activity::Falling => self.falling,
            Activity::Turning => self.turning,
        }
    }
}

impl Default for Rules {
    fn default() -> Self {
        Self::REFERENCE
    }
}
