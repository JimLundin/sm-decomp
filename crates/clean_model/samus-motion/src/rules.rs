//! The values the model reads. `Rules::REFERENCE` holds the spec's vanilla numbers.

use clean_types::{Fixed, below};

/// `WordwiseTopSpeed`: how the top speed is tested.
#[derive(Clone, Copy)]
pub enum SpeedCap {
    /// Vanilla: part by part, on the speed's size.
    Wordwise,
    /// The smaller of the two.
    Exact,
}

/// One row of `SpeedBuildsAndBrakes`'s table: what she speeds up by a frame, and her top speed.
pub type Limits = (Fixed, Fixed);

/// The values the model reads.
#[derive(Clone, Copy)]
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
    /// `Jumping`: a jump, without and with hi-jump.
    pub jump: (Fixed, Fixed),
    /// `Jumping`: a wall jump, without and with hi-jump.
    pub wall_jump: (Fixed, Fixed),
    /// `Jumping`: a ball bounce.
    pub bounce: Fixed,
    /// `Jumping`: the crouch lift.
    pub crouch_lift: Fixed,
    /// `Jumping`: the space jump's window of fall speeds read as 8.8, from and up to.
    pub space_jump_window: (u16, u16),
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
    /// `WallJump`: how near a wall must be.
    pub wall_reach: Fixed,
    /// `BlocksStopFlush`: the longest horizontal move, in px.
    pub move_limit_px: u16,
    /// `AimingAndShooting`: the step forward.
    pub aim_step: Fixed,
    /// `RidingTheElevator`: the descent a frame.
    pub elevator: Fixed,
}

impl Rules {
    /// The vanilla game, with the spec's numbers.
    pub const REFERENCE: Self = Self {
        speed_cap: SpeedCap::Wordwise,
        running: (Fixed(0x3000), Fixed(0x2_C000)),
        rolling: (Fixed(0xC000), Fixed(0x3_4000)),
        moonwalking: (Fixed(0xC000), Fixed(0x8000)),
        straight_jump: (Fixed(0xC000), Fixed(0x1_4000)),
        spin_jump: (Fixed(0xC000), Fixed(0x1_6000)),
        falling: (Fixed(0xC000), Fixed(0x1_0000)),
        turning: (Fixed(0xC000), Fixed(0)),
        brake: Fixed(0x8000),
        dash_step: Fixed(0x1000),
        dash_cap: Fixed(0x2_0000),
        jump: (Fixed(0x4_E000), Fixed(0x6_0000)),
        wall_jump: (Fixed(0x4_A000), Fixed(0x5_8000)),
        bounce: Fixed(0x1_0000),
        crouch_lift: Fixed(0xA_0000),
        space_jump_window: (0x280, 0x500),
        gravity: Fixed(0x1C00),
        fall_settle_px: 5,
        bounce_from_px: 3,
        second_bounce_less: Fixed(0x1_0000),
        ground_pull: Fixed(0x1_0000),
        wall_reach: Fixed(0x8_0000),
        move_limit_px: 15,
        aim_step: Fixed(0x1_0000),
        elevator: Fixed(0x1_0000),
    };

    /// `WordwiseTopSpeed`: the speed after the top is tested, and whether it took the top.
    #[must_use]
    pub fn cap(&self, speed: Fixed, top: Fixed) -> (Fixed, bool) {
        let keep = match self.speed_cap {
            // `Wordwise`: pixel parts first, then subpixels when they are equal.
            SpeedCap::Wordwise => {
                below(speed.pixel(), top.pixel())
                    || speed.pixel() == top.pixel()
                        && (speed.sub() == top.sub() || below(speed.sub(), top.sub()))
            }
            SpeedCap::Exact => speed <= top,
        };
        if keep { (speed, false) } else { (top, true) }
    }
}
