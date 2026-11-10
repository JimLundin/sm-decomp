//! Her run: her horizontal motion as a value of its own (§1.2 carrying on, her dash).
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
use DashIs::{Hers, ToLose};
use Stride::{Both, Dash, DashThenHalt, Nothing};
use clean_types::button::{RUN, SIDES};
use clean_types::{Facing, Fixed, Parts, Side, ZERO, part_below};

/// §1.2 carrying on: how her horizontal motion goes.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Carry {
    /// It follows the pad.
    FollowsPad,
    /// It carries on by itself: it brakes on the ground and keeps building in the air; against
    /// her facing, a skid.
    CarriesOn,
}

/// §1.2 her dash.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DashIs {
    /// Hers to keep.
    Hers,
    /// Left on her by a landing: gone the next frame she moves.
    ToLose,
}

/// `SpeedBuildsAndBrakes`: what a frame's horizontal move is made of.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Stride {
    /// No move.
    Nothing,
    /// Her dash alone.
    Dash,
    /// Her dash alone, then a halt.
    DashThenHalt,
    /// Her speed and her dash.
    Both,
}

/// Her horizontal motion: her speed's size and the way it points, whether it carries on, and
/// her dash, added the way she moves.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Run {
    /// Her horizontal speed's size.
    pub speed: Fixed,
    /// The side it points to.
    pub way: Side,
    /// §1.2 carrying on.
    pub carry: Carry,
    /// Her dash.
    pub dash: Fixed,
    /// §1.2 her dash.
    pub dash_is: DashIs,
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
    /// it, following the pad; a landing too, the dash keeping its value to lose (§1.2).
    pub(crate) const fn meet(&mut self, contact: Contact) {
        match contact {
            Contact::WallKick => self.stop(),
            Contact::Land => {
                self.stop();
                self.dash_is = ToLose;
            }
            Contact::Hop | Contact::Bounce(_) | Contact::Settle => {}
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
                Both
            }
            Moves::Rest if falls => Dash,
            Moves::Rest | Moves::DashThenHalt => DashThenHalt,
            Moves::TakeOff => {
                self.lose_dash();
                self.carry = FollowsPad;
                Dash
            }
            Moves::Dash => Dash,
        }
    }

    /// `SpeedBuildsAndBrakes`, `DriftInTheAir`, `Dash`: her speed building towards her gait's
    /// top, as her drift says. Running with run held she gains dash; in any other moving
    /// activity a dash she is to lose is gone. Carrying on along the ground she brakes, and a skid
    /// in the air brakes as on the ground, but not through a wall jump. Following the pad with
    /// neither side held, a straight jump or a fall stops drifting, and the falling ball halts but
    /// still moves by its dash. Else she builds; carrying on in the air she doesn't brake. A spin
    /// drifts only while she holds the side she faces (left and right together read so) or
    /// reached her top speed this frame, else she stops drifting and moves by her dash alone; a
    /// spin drifting, and a wall jump always, carry on along her facing.
    fn drift(&mut self, gait: Gait, drift: Drift, facing: Facing, t: &Tick) -> Stride {
        let (r, pad) = (t.rules, t.frame.pad);
        if gait == Gait::Running && pad.any(RUN) {
            self.grow_dash(r.dash_cap.of(&t.frame.items), r);
        } else {
            self.lose_dash();
        }
        let (ahead, carries) = (Side::of(facing), self.carry == CarriesOn);
        if carries && (drift == Ground || self.way != ahead && drift != WallJump) {
            self.brake(r);
            return Both;
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
            return Dash;
        }
        if matches!(drift, Spin | WallJump) {
            self.carry = CarriesOn;
        }
        Both
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
        self.dash_is = Hers;
    }

    /// `Dash` (§1.2): a dash she is to lose is gone the next frame she moves.
    fn lose_dash(&mut self) {
        if self.dash_is == ToLose {
            self.dash = ZERO;
        }
    }
}
