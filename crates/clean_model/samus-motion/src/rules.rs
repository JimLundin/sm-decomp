//! The tuning parameters the model reads. `Rules::REFERENCE` holds the spec's vanilla numbers.

use clean_types::{Fixed, below};
use core::ops::Range;

/// `WordwiseTopSpeed`: how the top speed is tested.
pub enum SpeedCap {
    /// Vanilla: part by part, on the speed's size.
    Wordwise,
    /// The smaller of the two.
    Exact,
}

/// `SpeedBuildsAndBrakes`: what she is doing, as its table's "She is" names it.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum Gait {
    /// Running.
    #[default]
    Running,
    /// Rolling as a ball.
    Rolling,
    /// Moonwalking.
    Moonwalking,
    /// In a straight jump or the spring ball's hop.
    StraightJump,
    /// In a spin jump or a wall jump.
    SpinJump,
    /// Falling, upright or as a ball.
    Falling,
    /// Turning round.
    Turning,
}

/// One row of `SpeedBuildsAndBrakes`'s table.
#[derive(Clone, Copy, Default)]
pub struct Limits {
    /// What she is doing.
    pub gait: Gait,
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
    /// `Jumping`: the speed for her equipment.
    #[must_use]
    pub const fn of(self, hi_jump: bool) -> Fixed {
        if hi_jump { self.hi_jump } else { self.plain }
    }
}

/// The values the model reads.
pub struct Rules {
    /// `WordwiseTopSpeed`.
    pub speed_cap: SpeedCap,
    /// `SpeedBuildsAndBrakes`: its table, a row for each thing she can be doing.
    pub speeds: [Limits; 7],
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
    /// `WallJump`: the near-wall test's reach.
    pub wall_reach: Fixed,
    /// `ShrinkingLowersHer`: the crouch's look down.
    pub crouch_lower: Fixed,
    /// `ShrinkingLowersHer`: the morph's look down.
    pub morph_lower: Fixed,
    /// `BlocksStopFlush`: the longest horizontal move, in px.
    pub move_limit_px: u16,
    /// `AimingAndShooting`: the step forward.
    pub aim_step: Fixed,
    /// `RidingTheElevator`: the descent a frame.
    pub elevator: Fixed,
    /// `PoseFitAfterMoves`: a block look this long or longer is stopped as this long a look is.
    pub look: Fixed,
    /// `PoseFitAfterMoves`: an old body under this half-height can't crouch instead.
    pub small_body_px: u16,
}

impl Rules {
    /// The vanilla game, with the spec's numbers.
    pub const REFERENCE: Self = Self {
        speed_cap: SpeedCap::Wordwise,
        speeds: [
            Limits {
                gait: Gait::Running,
                step: Fixed(0x3000),
                top: Fixed(0x2_C000),
            },
            Limits {
                gait: Gait::Rolling,
                step: Fixed(0xC000),
                top: Fixed(0x3_4000),
            },
            Limits {
                gait: Gait::Moonwalking,
                step: Fixed(0xC000),
                top: Fixed(0x8000),
            },
            Limits {
                gait: Gait::StraightJump,
                step: Fixed(0xC000),
                top: Fixed(0x1_4000),
            },
            Limits {
                gait: Gait::SpinJump,
                step: Fixed(0xC000),
                top: Fixed(0x1_6000),
            },
            Limits {
                gait: Gait::Falling,
                step: Fixed(0xC000),
                top: Fixed(0x1_0000),
            },
            Limits {
                gait: Gait::Turning,
                step: Fixed(0xC000),
                top: Fixed(0),
            },
        ],
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
        wall_reach: Fixed(0x8_0000),
        crouch_lower: Fixed(0x5_0000),
        morph_lower: Fixed(0x9_0000),
        move_limit_px: 15,
        aim_step: Fixed(0x1_0000),
        elevator: Fixed(0x1_0000),
        look: Fixed(0x8_0000),
        small_body_px: 8,
    };

    /// `SpeedBuildsAndBrakes`: the table's row for what she is doing.
    #[must_use]
    pub fn limits(&self, gait: Gait) -> Limits {
        self.speeds
            .into_iter()
            .find(|l| l.gait == gait)
            .unwrap_or_default()
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
