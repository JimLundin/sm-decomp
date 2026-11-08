//! The tuning parameters the model reads. `Rules::REFERENCE` holds the spec's vanilla numbers.

use Gait::{Falling, Moonwalking, Rolling, Running, SpinJump, StraightJump, Turning};
use Lowers::{Crouch, Morph, Nothing};
use clean_types::{Fixed, Parts, Px, ZERO, below};
use core::num::Wrapping;
use core::ops::Range;

/// `WordwiseTopSpeed`: how the top speed is tested.
pub enum SpeedCap {
    /// Vanilla: part by part, on the speed's size.
    Wordwise,
    /// The smaller of the two.
    Exact,
}

/// `SpeedBuildsAndBrakes`: what she is doing, as its table's "She is" names it.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Gait {
    /// Running.
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

/// One row of `SpeedBuildsAndBrakes`'s table: what she speeds up by a frame, and her top speed.
#[derive(Clone, Copy)]
pub struct Limits {
    /// What she speeds up by a frame.
    pub step: Fixed,
    /// Her top speed.
    pub top: Fixed,
}

/// `SpeedBuildsAndBrakes`'s table: a row for each thing she can be doing.
pub struct Speeds {
    /// Running.
    pub running: Limits,
    /// Rolling as a ball.
    pub rolling: Limits,
    /// Moonwalking.
    pub moonwalking: Limits,
    /// In a straight jump or the spring ball's hop.
    pub straight_jump: Limits,
    /// In a spin jump or a wall jump.
    pub spin_jump: Limits,
    /// Falling, upright or as a ball.
    pub falling: Limits,
    /// Turning round.
    pub turning: Limits,
}

/// `ShrinkingLowersHer`: the distance a change of shape lowers her by, as its row names it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lowers {
    /// Starting to crouch: the crouch's own distance.
    Crouch,
    /// Starting to morph: the morph's own distance.
    Morph,
    /// Standing up or unmorphing: 0 px.
    Nothing,
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
    /// `SpeedBuildsAndBrakes`: its table.
    pub speeds: Speeds,
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
    /// `Jumping`: a ball bounce. An adjusted one is never below its vanilla value, 1.0000: the
    /// rise after bounce 1 is it less 1 px (`Landing`).
    pub bounce: Fixed,
    /// `Jumping`: the crouch lift.
    pub crouch_lift: Fixed,
    /// `Jumping`: the space jump's fall speeds, read as 8.8.
    pub space_jump: Range<u16>,
    /// `RisingAndFalling`: the pull a frame.
    pub gravity: Fixed,
    /// `RisingAndFalling`: the pixel part at which a fall stops growing.
    pub fall_settle_px: Px,
    /// `Landing`: the slowest landing that bounces the ball, in px.
    pub bounce_from_px: Px,
    /// `Landing`: the second bounce is the bounce speed less this.
    pub second_bounce_less: Fixed,
    /// `FollowingTheGround` and `SlopesCarry`: the pull beyond her speed, and on a slope.
    pub ground_pull: Fixed,
    /// `WallJump`: the near-wall test's reach.
    pub wall_reach: Fixed,
    /// `BlocksStopFlush`: the longest horizontal move, in px.
    pub move_limit_px: Px,
    /// `AimingAndShooting`: the step forward.
    pub aim_step: Fixed,
    /// `RidingTheElevator`: the descent a frame.
    pub elevator: Fixed,
    /// `PoseFitAfterMoves`: a block look this long or longer is stopped as this long a look is.
    pub look: Fixed,
    /// `PoseFitAfterMoves`: an old body under this half-height can't crouch instead.
    pub small_body_px: Px,
    /// `ShrinkingLowersHer`: the crouch's own distance.
    pub crouch_lowers: Fixed,
    /// `ShrinkingLowersHer`: the morph's own distance.
    pub morph_lowers: Fixed,
}

impl Rules {
    /// The vanilla game, with the spec's numbers.
    pub const REFERENCE: Self = Self {
        speed_cap: SpeedCap::Wordwise,
        speeds: Speeds {
            running: Limits {
                step: Wrapping(0x3000),
                top: Wrapping(0x2_C000),
            },
            rolling: Limits {
                step: Wrapping(0xC000),
                top: Wrapping(0x3_4000),
            },
            moonwalking: Limits {
                step: Wrapping(0xC000),
                top: Wrapping(0x8000),
            },
            straight_jump: Limits {
                step: Wrapping(0xC000),
                top: Wrapping(0x1_4000),
            },
            spin_jump: Limits {
                step: Wrapping(0xC000),
                top: Wrapping(0x1_6000),
            },
            falling: Limits {
                step: Wrapping(0xC000),
                top: Wrapping(0x1_0000),
            },
            turning: Limits {
                step: Wrapping(0xC000),
                top: Wrapping(0),
            },
        },
        brake: Wrapping(0x8000),
        dash_step: Wrapping(0x1000),
        dash_cap: Wrapping(0x2_0000),
        jump: Boots {
            plain: Wrapping(0x4_E000),
            hi_jump: Wrapping(0x6_0000),
        },
        wall_jump: Boots {
            plain: Wrapping(0x4_A000),
            hi_jump: Wrapping(0x5_8000),
        },
        bounce: Wrapping(0x1_0000),
        crouch_lift: Wrapping(0xA_0000),
        space_jump: 0x280..0x500,
        gravity: Wrapping(0x1C00),
        fall_settle_px: Wrapping(5),
        bounce_from_px: Wrapping(3),
        second_bounce_less: Wrapping(0x1_0000),
        ground_pull: Wrapping(0x1_0000),
        wall_reach: Wrapping(0x8_0000),
        move_limit_px: Wrapping(15),
        aim_step: Wrapping(0x1_0000),
        elevator: Wrapping(0x1_0000),
        look: Wrapping(0x8_0000),
        small_body_px: Wrapping(8),
        crouch_lowers: Wrapping(0x5_0000),
        morph_lowers: Wrapping(0x9_0000),
    };

    /// `SpeedBuildsAndBrakes`: the table's row for what she is doing.
    #[must_use]
    pub const fn limits(&self, gait: Gait) -> Limits {
        let s = &self.speeds;
        match gait {
            Running => s.running,
            Rolling => s.rolling,
            Moonwalking => s.moonwalking,
            StraightJump => s.straight_jump,
            SpinJump => s.spin_jump,
            Falling => s.falling,
            Turning => s.turning,
        }
    }

    /// `ShrinkingLowersHer`: how far the change of shape a row names looks down to lower her.
    #[must_use]
    pub const fn lowers(&self, by: Lowers) -> Fixed {
        match by {
            Crouch => self.crouch_lowers,
            Morph => self.morph_lowers,
            Nothing => ZERO,
        }
    }

    /// `WordwiseTopSpeed`: the speed after the top is tested, on the speed's size. `Wordwise`:
    /// pixel parts first, then subpixels when they are equal.
    #[must_use]
    pub fn cap(&self, speed: Fixed, top: Fixed) -> Fixed {
        let ((p, s), (tp, ts)) = (speed.words(), top.words());
        let keep = match self.speed_cap {
            SpeedCap::Wordwise => below(p, tp) || p == tp && (s == ts || below(s, ts)),
            SpeedCap::Exact => speed <= top,
        };
        if keep { speed } else { top }
    }
}

#[cfg(test)]
mod tests {
    use super::Rules;
    use clean_types::{Parts, fixed};

    /// The reference rules agree with each other. `Landing`: the ball's bounce speed is at least
    /// what a bounce after the first takes off, its bounce speed less the second bounce's cut,
    /// so that cut never wraps below zero. `WallJump`, `BlocksStopFlush`: the near-wall test's
    /// reach is within the longest horizontal move, so the move limit never cuts the test short.
    #[test]
    fn the_reference_rules_agree() {
        let r = Rules::REFERENCE;
        let after_the_first = r.bounce - r.second_bounce_less;
        assert!(
            after_the_first <= r.bounce,
            "a bounce after the first takes off at {after_the_first:?}, above the bounce speed {:?}",
            r.bounce
        );
        let limit = fixed(r.move_limit_px, r.wall_reach.sub());
        assert!(
            r.wall_reach <= limit,
            "the near-wall test's reach {:?} is beyond the move limit {limit:?}",
            r.wall_reach
        );
    }
}
