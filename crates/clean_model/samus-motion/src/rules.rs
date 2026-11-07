//! The tuning parameters the model reads. `Rules::REFERENCE` holds the spec's vanilla numbers.

use crate::situation::Gait;
use clean_types::{Fixed, below};
use core::ops::Range;

/// `WordwiseTopSpeed`: how the top speed is tested.
#[derive(Clone, Copy)]
pub enum SpeedCap {
    /// Vanilla: part by part, on the speed's size.
    Wordwise,
    /// The smaller of the two.
    Exact,
}

/// One row of `SpeedBuildsAndBrakes`'s table.
#[derive(Clone, Copy)]
pub struct Limits {
    /// What she speeds up by a frame.
    pub step: Fixed,
    /// Her top speed.
    pub top: Fixed,
}

/// `Jumping`: a rise speed without and with hi-jump.
#[derive(Clone, Copy)]
pub struct Boots {
    /// Without hi-jump.
    pub plain: Fixed,
    /// With hi-jump.
    pub hi_jump: Fixed,
}

impl Boots {
    /// The speed for her equipment.
    #[must_use]
    pub const fn of(self, hi_jump: bool) -> Fixed {
        if hi_jump { self.hi_jump } else { self.plain }
    }
}

/// The values the model reads.
#[derive(Clone)]
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
    /// `SpeedBuildsAndBrakes`: falling, upright or as a ball.
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
    pub jump: Boots,
    /// `Jumping`: a wall jump.
    pub wall_jump: Boots,
    /// `Jumping`: a ball bounce.
    pub bounce: Fixed,
    /// `Jumping`: the crouch lift.
    pub crouch_lift: Fixed,
    /// `Jumping`: the space jump's fall speeds, read as 8.8.
    pub space_jump: Range<u16>,
    /// `RisingAndFalling`: the pull a frame.
    pub gravity: Fixed,
    /// `RisingAndFalling`: the pixel part at which a fall stops growing.
    pub fall_settle_px: u16,
    /// `Landing`: the slowest landing that bounces the ball, in px.
    pub bounce_from_px: u16,
    /// `Landing`: the second bounce is the bounce speed less this.
    pub second_bounce_less: Fixed,
    /// `FollowingTheGround` and `SlopesCarry`: the pull beyond her speed, and on a slope.
    pub ground_pull: Fixed,
    /// `WallJump`: how near a wall must be, in px.
    pub wall_reach: u16,
    /// `BlocksStopFlush`: the longest horizontal move, in px.
    pub move_limit_px: u16,
    /// `AimingAndShooting`: the step forward.
    pub aim_step: Fixed,
    /// `RidingTheElevator`: the descent a frame.
    pub elevator: Fixed,
    /// `PoseFitAfterMoves`: a block look this long or longer is stopped as this long a look is.
    pub look_px: u16,
    /// `PoseFitAfterMoves`: an old body under this half-height can't crouch instead.
    pub small_body_px: u16,
}

impl Rules {
    /// The vanilla game, with the spec's numbers.
    pub const REFERENCE: Self = Self {
        speed_cap: SpeedCap::Wordwise,
        running: Limits {
            step: Fixed(0x3000),
            top: Fixed(0x2_C000),
        },
        rolling: Limits {
            step: Fixed(0xC000),
            top: Fixed(0x3_4000),
        },
        moonwalking: Limits {
            step: Fixed(0xC000),
            top: Fixed(0x8000),
        },
        straight_jump: Limits {
            step: Fixed(0xC000),
            top: Fixed(0x1_4000),
        },
        spin_jump: Limits {
            step: Fixed(0xC000),
            top: Fixed(0x1_6000),
        },
        falling: Limits {
            step: Fixed(0xC000),
            top: Fixed(0x1_0000),
        },
        turning: Limits {
            step: Fixed(0xC000),
            top: Fixed(0),
        },
        brake: Fixed(0x8000),
        dash_step: Fixed(0x1000),
        dash_cap: Fixed(0x2_0000),
        jump: Boots {
            plain: Fixed(0x4_E000),
            hi_jump: Fixed(0x6_0000),
        },
        wall_jump: Boots {
            plain: Fixed(0x4_A000),
            hi_jump: Fixed(0x5_8000),
        },
        bounce: Fixed(0x1_0000),
        crouch_lift: Fixed(0xA_0000),
        space_jump: 0x280..0x500,
        gravity: Fixed(0x1C00),
        fall_settle_px: 5,
        bounce_from_px: 3,
        second_bounce_less: Fixed(0x1_0000),
        ground_pull: Fixed(0x1_0000),
        wall_reach: 8,
        move_limit_px: 15,
        aim_step: Fixed(0x1_0000),
        elevator: Fixed(0x1_0000),
        look_px: 8,
        small_body_px: 8,
    };

    /// `SpeedBuildsAndBrakes`: the row for a gait.
    #[must_use]
    pub const fn limits(&self, gait: Gait) -> Limits {
        match gait {
            Gait::Running => self.running,
            Gait::Rolling => self.rolling,
            Gait::Moonwalking => self.moonwalking,
            Gait::StraightJump => self.straight_jump,
            Gait::SpinJump => self.spin_jump,
            Gait::Falling => self.falling,
            Gait::Turning => self.turning,
        }
    }

    /// `WordwiseTopSpeed`: the speed after the top is tested.
    #[must_use]
    pub fn cap(&self, speed: Fixed, top: Fixed) -> Fixed {
        let keep = match self.speed_cap {
            // `Wordwise`: pixel parts first, then subpixels when they are equal.
            SpeedCap::Wordwise => {
                below(speed.pixel(), top.pixel())
                    || speed.pixel() == top.pixel()
                        && (speed.sub() == top.sub() || below(speed.sub(), top.sub()))
            }
            SpeedCap::Exact => speed <= top,
        };
        if keep { speed } else { top }
    }
}
