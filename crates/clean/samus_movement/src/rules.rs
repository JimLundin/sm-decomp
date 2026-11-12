//! The tuning parameters the model reads. `Rules::REFERENCE` holds the spec's vanilla numbers.

use Gait::{Falling, Moonwalking, Rolling, Running, SpinJump, StraightJump, Turning};
use Lowers::{Crouch, Morph, Nothing};
use core::num::Wrapping;
use core::ops::{Range, RangeInclusive};
use types::{Fixed, Item, Parts, Px, Set, ZERO, fixed, part_below};

/// `WordwiseTopSpeed`: how the top speed is tested.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpeedCap {
    /// Vanilla: part by part, on the speed's size.
    Wordwise,
    /// The smaller of the two.
    Exact,
}

/// `SpeedBuildsAndBrakes`: what she is doing, as its table's "She is" names it, which gives her
/// speed's step and top.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Gait {
    /// She runs on the ground.
    Running,
    /// She rolls as a ball on the ground.
    Rolling,
    /// She walks backwards, looking the other way.
    Moonwalking,
    /// She is in a straight jump or the spring ball's hop.
    StraightJump,
    /// She is in a spin jump or a wall jump.
    SpinJump,
    /// She falls, upright or as a ball.
    Falling,
    /// She turns round.
    Turning,
}

/// One row of `SpeedBuildsAndBrakes`'s table: what she speeds up by a frame, and her top speed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    /// What she speeds up by a frame.
    pub step: Fixed,
    /// Her top speed.
    pub top: Fixed,
}

/// `SpeedBuildsAndBrakes`'s table: a row for each thing she can be doing, a `Gait`'s each.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Speeds {
    /// The row for `Gait::Running`.
    pub running: Limits,
    /// The row for `Gait::Rolling`.
    pub rolling: Limits,
    /// The row for `Gait::Moonwalking`.
    pub moonwalking: Limits,
    /// The row for `Gait::StraightJump`.
    pub straight_jump: Limits,
    /// The row for `Gait::SpinJump`.
    pub spin_jump: Limits,
    /// The row for `Gait::Falling`.
    pub falling: Limits,
    /// The row for `Gait::Turning`.
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

/// A value that an item switched on changes (§1.3): `Jumping`'s rise speeds with the hi-jump
/// boots, `Dash`'s cap with the speed booster.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WithItem {
    /// The item.
    pub item: Item,
    /// The value without it.
    pub without: Fixed,
    /// The value with it.
    pub with: Fixed,
}

impl WithItem {
    /// The value the items switched on give.
    #[must_use]
    pub fn of(self, items: &Set<Item>) -> Fixed {
        if items[self.item] {
            self.with
        } else {
            self.without
        }
    }
}

/// `Landing`: how many times the ball has bounced since it last rested.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Bounces {
    /// It hasn't bounced since it last rested.
    None,
    /// It has bounced once: the next landing bounces it at any speed.
    Once,
    /// It has bounced twice: the next landing rests it.
    Twice,
}

/// `Landing`: one of the ball's bounces: the landing that makes it, the speed it rises at, and the
/// bounce that comes after it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bounce {
    /// The bounces she has had when the landing that makes it comes.
    pub at: Bounces,
    /// The slowest landing that makes it: the pixel part of her vertical speed after that frame's
    /// pull.
    pub from_px: Px,
    /// How much slower than the bounce speed (`Rules::bounce`) it rises, so that an adjusted
    /// bounce speed moves it too.
    pub slower: Fixed,
    /// The bounces she has had after it: the next landing makes the bounce at them, if any.
    pub then: Bounces,
}

/// The values the model reads.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rules {
    /// `WordwiseTopSpeed`: how the top speed is tested.
    pub speed_cap: SpeedCap,
    /// `SpeedBuildsAndBrakes`: its table.
    pub speeds: Speeds,
    /// `SpeedBuildsAndBrakes`: lost a frame carrying on along the ground.
    pub brake: Fixed,
    /// `Dash`: gained a frame.
    pub dash_step: Fixed,
    /// `Dash`: its cap, higher with the speed booster.
    pub dash_cap: WithItem,
    /// `Jumping`: a jump, faster with the hi-jump boots.
    pub jump: WithItem,
    /// `Jumping`: a wall jump, faster with the hi-jump boots.
    pub wall_jump: WithItem,
    /// `Jumping`: with the speed booster, a rise at the jump or wall-jump speed starts faster by
    /// part of her dash (`Rules::rise_speed`). Only whether it applies is the game's to set; how
    /// much it adds is fixed.
    pub booster_lifts_jumps: bool,
    /// `Jumping`: a ball bounce. An adjusted one is never below its vanilla value, 1.0000: the
    /// rise after bounce 1 is it less 1 px (`Landing`).
    pub bounce: Fixed,
    /// `Landing`: the ball's bounces, the first and the second; after the second it comes to
    /// rest.
    pub bounces: [Bounce; 2],
    /// `Jumping`: the crouch lift.
    pub crouch_lift: Fixed,
    /// `Jumping`: the space jump's fall speeds, read as 8.8.
    pub space_jump: Range<u16>,
    /// `Jumping`, jump held through: the counts of frames jump has stayed held at which, the
    /// frame after a landing or a turn's or a landing's end, it counts as a press.
    pub held_through: RangeInclusive<u16>,
    /// `Jumping`, jump held through: the long hold, counts of frames jump has stayed held since
    /// its press, 32,777 to 65,535 (about 9 minutes or more), which act as one of
    /// `held_through` does (original only).
    pub held_long: RangeInclusive<u16>,
    /// `RisingAndFalling`: the pull a frame.
    pub gravity: Fixed,
    /// `RisingAndFalling`: the pixel part at which a fall stops growing.
    pub fall_settle_px: Px,
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
    /// `PoseFitAfterMoves`: the long look, 8 px: a block look this long or longer is stopped by
    /// whatever a look of this length would stop.
    pub long_look: Fixed,
    /// `PoseFitAfterMoves`: with something above her and something below, a change from an old
    /// body under this half-height, 8 px (the ball), is refused; from one of this half-height or
    /// more she takes the crouch instead.
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
        dash_cap: WithItem {
            item: Item::SpeedBooster,
            without: Wrapping(0x2_0000),
            with: Wrapping(0x7_0000),
        },
        jump: WithItem {
            item: Item::HiJumpBoots,
            without: Wrapping(0x4_E000),
            with: Wrapping(0x6_0000),
        },
        wall_jump: WithItem {
            item: Item::HiJumpBoots,
            without: Wrapping(0x4_A000),
            with: Wrapping(0x5_8000),
        },
        booster_lifts_jumps: true,
        bounce: Wrapping(0x1_0000),
        bounces: [
            Bounce {
                at: Bounces::None,
                from_px: Wrapping(3),
                slower: Wrapping(0),
                then: Bounces::Once,
            },
            Bounce {
                at: Bounces::Once,
                from_px: Wrapping(0),
                slower: Wrapping(0x1_0000),
                then: Bounces::Twice,
            },
        ],
        crouch_lift: Wrapping(0xA_0000),
        space_jump: 0x280..0x500,
        held_through: 1..=8,
        held_long: 32_777..=65_535,
        gravity: Wrapping(0x1C00),
        fall_settle_px: Wrapping(5),
        ground_pull: Wrapping(0x1_0000),
        wall_reach: Wrapping(0x8_0000),
        move_limit_px: Wrapping(15),
        aim_step: Wrapping(0x1_0000),
        elevator: Wrapping(0x1_0000),
        long_look: Wrapping(0x8_0000),
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
            SpeedCap::Wordwise => part_below(p, tp) || p == tp && (s == ts || part_below(s, ts)),
            SpeedCap::Exact => speed <= top,
        };
        if keep { speed } else { top }
    }

    /// `Jumping`: the speed a rise at the jump or wall-jump speed (`base`) starts at, with the
    /// items switched on and her dash as the frame's moves left it. With the speed booster it is
    /// faster by part of the dash `d.e`: from the speed `p.s` the items give, the pixel part is
    /// `p + ⌊d ÷ 2⌋` and the subpixel part `(s + e) mod 2^16`, the carry out of the subpixel part
    /// lost (so a dash of 1.7000 makes 4.E000 a slower 4.5000).
    #[must_use]
    pub fn rise_speed(&self, base: WithItem, items: &Set<Item>, dash: Fixed) -> Fixed {
        let speed = base.of(items);
        if !(self.booster_lifts_jumps && items[Item::SpeedBooster]) {
            return speed;
        }
        let ((p, s), (d, e)) = (speed.words(), dash.words());
        fixed(Wrapping(p) + Wrapping(d >> 1), s.wrapping_add(e))
    }
}

#[cfg(test)]
mod tests {
    use super::Rules;
    use types::{Parts, fixed};

    /// The reference rules agree with each other. `Landing`: no bounce is slower than the bounce
    /// speed itself, so its speed never wraps below zero. `WallJump`, `BlocksStopFlush`: the
    /// near-wall test's reach is within the longest horizontal move, so the move limit never cuts
    /// the test short.
    #[test]
    fn the_reference_rules_agree() {
        let r = Rules::REFERENCE;
        for bounce in r.bounces {
            assert!(
                bounce.slower <= r.bounce,
                "a bounce rises {:?} slower than the bounce speed {:?}",
                bounce.slower,
                r.bounce
            );
        }
        let limit = fixed(r.move_limit_px, r.wall_reach.sub());
        assert!(
            r.wall_reach <= limit,
            "the near-wall test's reach {:?} is beyond the move limit {limit:?}",
            r.wall_reach
        );
    }
}
