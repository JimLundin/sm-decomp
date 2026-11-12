//! Her run: her horizontal motion as a value of its own (§1.2 carrying on, her dash, dashing).
//!
//! How her speed builds, brakes and halts as her situation moves her (`SpeedBuildsAndBrakes`,
//! `Dash`, `DriftInTheAir`, `Jumping`, `TurningRound`), and what a turn, a landing, a wall's
//! kick or the pad chart's let-go rows do to it. Samus holds one; her moves read it.

use crate::rows::Drift::{self, Air, BallAir, Ground, Spin, WallJump};
use crate::rows::{FORWARD, Moves, seen};
use crate::rules::{Gait, Limits, Rules};
use crate::tick::Tick;
use crate::vertical::Contact;
use Carry::{CarriesOn, FollowsPad};
use Stride::{DashAlone, DashThenHalt, Nothing, SpeedAndDash};
use types::button::{RUN, SIDES};
use types::{Buttons, Facing, Fixed, Parts, Side, ZERO, part_below};

/// §1.2 carrying on: how her horizontal motion goes.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Carry {
    /// It follows the pad.
    FollowsPad,
    /// It carries on by itself: it brakes on the ground and keeps building in the air; against
    /// her facing, a skid.
    CarriesOn,
}

/// `Dashing` (§1, §1.2): whether she is dashing.
///
/// She dashes from a run with run held until a halt, a turn or the pad's let-go ends it, in any
/// pose and with any dash: crouched she goes on dashing with a dash of 0, so her dash can't tell
/// it. It is also what keeps her dash hers: a dash she has while not dashing is lost the next
/// frame she moves (`Dash`). State a caller gives and reads back; samus-appearance reads it for her
/// run's pace and its stages towards boosting.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Dashing {
    /// She is not dashing.
    Off,
    /// She is dashing.
    On,
}

/// `SpeedBuildsAndBrakes`: what a frame's horizontal move is made of.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Stride {
    /// She makes no horizontal move.
    Nothing,
    /// She moves by her dash alone.
    DashAlone,
    /// She moves by her dash alone, then halts.
    DashThenHalt,
    /// She moves by her speed and her dash together.
    SpeedAndDash,
}

/// `Dash`, `Dashing`: running with run held: she gains dash, and she dashes.
pub fn running_with_run_held(gait: Gait, pad: Buttons) -> bool {
    gait == Gait::Running && pad.any(RUN)
}

/// Her horizontal motion: her speed's size and the way it points, whether it carries on, her
/// dash, added the way she moves, and whether she is dashing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Run {
    /// Her horizontal speed's size.
    pub speed: Fixed,
    /// The side her horizontal speed points to.
    pub way: Side,
    /// §1.2 carrying on: whether her motion follows the pad or carries on by itself.
    pub carry: Carry,
    /// `Dash`: the extra speed from holding run, added the way she moves.
    pub dash: Fixed,
    /// `Dashing`: on from a run with run held until a halt, a turn or the pad's let-go ends it;
    /// while it is off, a dash on her is lost the next frame she moves.
    pub dashing: Dashing,
}

impl Run {
    /// `SpeedBuildsAndBrakes`, `WallsAndCeilings`, `PadChart`: a halt: horizontal speed 0, dash
    /// 0, following the pad.
    pub(crate) const fn halt(&mut self) {
        self.stop();
        self.dash = ZERO;
    }

    /// `Landing`, `WallJump`, `DriftInTheAir`: horizontal speed 0, following the pad.
    const fn stop(&mut self) {
        self.speed = ZERO;
        self.carry = FollowsPad;
    }

    /// `TurningRound` (§1.2 skidding): she carries on against her facing, at any speed, 0
    /// included.
    pub(crate) fn skids(&self, facing: Facing) -> bool {
        self.carry == CarriesOn && self.way != Side::of(facing)
    }

    /// `TurningRound`, `PadChart`'s "as running": the dash joins her speed (dash 0) and she
    /// carries on.
    fn join_dash(&mut self) {
        self.speed += self.dash;
        self.dash = ZERO;
        self.carry = CarriesOn;
    }

    /// `TurningRound`: her dash joins her speed and she travels against her new facing
    /// (`facing`), a skid.
    pub(crate) fn turn_round(&mut self, facing: Facing) {
        self.way = Side::of(facing.back());
        self.join_dash();
    }

    /// `PadChart`: she follows the pad again, a skid ending along her new facing (`facing`).
    pub(crate) const fn end_skid(&mut self, facing: Facing) {
        self.carry = FollowsPad;
        self.way = Side::of(facing);
    }

    /// `PadChart`'s "as running": her horizontal speed 0 at the frame's start (`start`) or
    /// after the frame's move, she follows the pad again, keeping the speed the move left her;
    /// else the dash joins her speed and she carries on, braking.
    pub(crate) fn as_running(&mut self, start: Fixed) {
        if start == ZERO || self.speed == ZERO {
            self.carry = FollowsPad;
        } else {
            self.join_dash();
        }
    }

    /// `Landing`, `WallJump`: what her moves' contact does to her run: a wall's kick stops
    /// it, following the pad; a landing too, the dash keeping its value and her dashing going on
    /// until the landing's own moves (`Dashing`).
    pub(crate) const fn meet(&mut self, contact: Contact) {
        match contact {
            Contact::WallKick | Contact::Land => self.stop(),
            Contact::Hop | Contact::Bounce(_) => {}
        }
    }

    /// `Jumping`: starting a straight jump or a fall with a dash she carries on along her
    /// facing; without a dash she follows the pad.
    pub(crate) fn launch(&mut self, facing: Facing) {
        self.carry = FollowsPad;
        if self.dash != ZERO {
            self.way = Side::of(facing);
            self.carry = CarriesOn;
        }
    }

    /// `SpeedBuildsAndBrakes`: the side she moves to: the way her speed points, her facing's when
    /// it is 0 (§1).
    pub(crate) fn moving(&self, facing: Facing) -> Side {
        if self.speed == ZERO {
            Side::of(facing)
        } else {
            self.way
        }
    }

    /// `SpeedBuildsAndBrakes`, `Dash`, `Jumping`: what her speed and dash become as her
    /// situation moves her (`moves`, facing `facing`), and what she moves by; `falls` is a
    /// vertical motion on the ground (a vertical speed, or falling at 0). The ball at rest
    /// braking or skidding rolls like running and keeps its dash; with a vertical motion it moves
    /// by its dash only, keeping both speeds. Taking off she moves by her dash only, a dash she is
    /// to lose gone first, and follows the pad.
    pub(crate) fn stride(&mut self, moves: Moves, facing: Facing, falls: bool, t: &Tick) -> Stride {
        match moves {
            Moves::Build(gait, drift) => self.drift(gait, drift, facing, t),
            Moves::Rest if self.carry == CarriesOn => {
                self.brake(t.rules);
                SpeedAndDash
            }
            Moves::Rest if falls => DashAlone,
            Moves::Rest | Moves::HaltsAfterDash => DashThenHalt,
            Moves::TakeOff => {
                self.lose_dash();
                self.carry = FollowsPad;
                DashAlone
            }
            Moves::DashKeepingSpeeds => DashAlone,
        }
    }

    /// `SpeedBuildsAndBrakes`, `DriftInTheAir`, `Dash`, `Dashing`: her speed building towards her
    /// gait's top, as her drift says. Running with run held she is dashing from her moves on and
    /// gains dash; in any other moving activity a dash she is to lose is gone. Carrying on along
    /// the ground she brakes, and a skid in the air brakes as on the ground, but not through a
    /// wall jump. Following the pad with neither side held, a straight jump or a fall stops
    /// drifting, and the falling ball halts but still moves by its dash. Else she builds;
    /// carrying on in the air she doesn't brake. A spin drifts only while she holds the side she
    /// faces (left and right together read so) or reached her top speed this frame, else she
    /// stops drifting and moves by her dash alone; a spin drifting, and a wall jump always, carry
    /// on along her facing.
    fn drift(&mut self, gait: Gait, drift: Drift, facing: Facing, t: &Tick) -> Stride {
        let (r, pad) = (t.rules, t.frame.pad);
        if running_with_run_held(gait, pad) {
            self.dashing = Dashing::On;
            self.grow_dash(r.dash_cap.of(&t.frame.items), r);
        } else {
            self.lose_dash();
        }
        let (ahead, carries) = (Side::of(facing), self.carry == CarriesOn);
        if carries && (drift == Ground || self.way != ahead && drift != WallJump) {
            self.brake(r);
            return SpeedAndDash;
        }
        let let_go = !carries && !pad.any(SIDES);
        if let_go && drift == Air {
            self.speed = ZERO;
            return Nothing;
        }
        if let_go && drift == BallAir {
            return DashThenHalt;
        }
        let topped = self.build(r.limits(gait), ahead, r);
        if drift == Spin && !topped && !seen(pad, facing).any(FORWARD) {
            self.stop();
            return DashAlone;
        }
        if matches!(drift, Spin | WallJump) {
            self.carry = CarriesOn;
        }
        SpeedAndDash
    }

    /// `SpeedBuildsAndBrakes`: gains the step up to the top (`WordwiseTopSpeed`), along her
    /// facing. Returns whether she took the top speed: the step was cut to it, so a step that
    /// lands exactly on it hasn't reached it (`DriftInTheAir`).
    fn build(&mut self, limits: Limits, ahead: Side, r: &Rules) -> bool {
        let sum = self.speed + limits.step;
        self.speed = r.cap(sum, limits.top);
        self.way = ahead;
        self.speed != sum
    }

    /// `SpeedBuildsAndBrakes`: carrying on she loses the brake; below zero she stops and follows
    /// the pad again.
    fn brake(&mut self, r: &Rules) {
        if self.speed < r.brake {
            self.speed = ZERO;
            self.carry = FollowsPad;
        } else {
            self.speed -= r.brake;
        }
    }

    /// `Dash`: grows if, before the step, its pixel part is below the cap's or its subpixel part
    /// is below the cap's (`part_below`); else it becomes the cap. The cap is the items' (higher
    /// with the speed booster).
    fn grow_dash(&mut self, cap: Fixed, r: &Rules) {
        let ((p, s), (cp, cs)) = (self.dash.words(), cap.words());
        self.dash = if part_below(p, cp) || part_below(s, cs) {
            self.dash + r.dash_step
        } else {
            cap
        };
    }

    /// `Dash` (§1.2): a dash she has while not dashing is the dash she is to lose, gone (0)
    /// before the move of the next frame she moves; dashing, she keeps it.
    fn lose_dash(&mut self) {
        if self.dashing == Dashing::Off {
            self.dash = ZERO;
        }
    }
}
