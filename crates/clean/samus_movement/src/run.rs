//! Her run: her horizontal motion as a value of its own (§1.2 carrying on, her dash, dashing).
//!
//! How her speed builds, brakes and halts as her situation moves her (`SpeedBuildsAndBrakes`,
//! `Dash`, `DriftInTheAir`, `Jumping`, `TurningRound`), and what a turn, a landing, a wall's
//! kick or the pad chart's let-go rows do to it. Samus holds one; her moves read it.

use crate::rows::{Drift, FORWARD, Moves, seen};
use crate::rules::{Gait, Limits, Rules};
use crate::tick::Tick;
use crate::vertical::Contact;
use types::button::{RUN, SIDES};
use types::{Buttons, Facing, Fixed, Parts, Side, ZERO, part_below};

/// §1.2 carrying on: how her horizontal motion goes.
#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
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
#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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

    /// `Landing`, `WallJump`, `DriftInTheAir`, `PushMoves`: horizontal speed 0, following the pad.
    pub(crate) const fn stop(&mut self) {
        self.speed = ZERO;
        self.carry = Carry::FollowsPad;
    }

    /// `TurningRound` (§1.2 skidding): she carries on against her facing, at any speed, 0
    /// included.
    pub(crate) fn skids(&self, facing: Facing) -> bool {
        self.carry == Carry::CarriesOn && self.way != Side::of(facing)
    }

    /// `TurningRound`, `PadChart`'s "as running": the dash joins her speed (dash 0) and she
    /// carries on.
    fn join_dash(&mut self) {
        self.speed += self.dash;
        self.dash = ZERO;
        self.carry = Carry::CarriesOn;
    }

    /// `TurningRound`: her dash joins her speed and she travels against her new facing
    /// (`facing`), a skid.
    pub(crate) fn turn_round(&mut self, facing: Facing) {
        self.way = Side::of(facing.back());
        self.join_dash();
    }

    /// `PadChart`: she follows the pad again, a skid ending along her new facing (`facing`).
    pub(crate) const fn end_skid(&mut self, facing: Facing) {
        self.carry = Carry::FollowsPad;
        self.way = Side::of(facing);
    }

    /// `BackwardLeap`'s let-go: she follows the pad again, her speed and dash as they are.
    pub(crate) const fn follow_the_pad(&mut self) {
        self.carry = Carry::FollowsPad;
    }

    /// `PadChart`'s "as running": her horizontal speed 0 at the frame's start (`start`) or
    /// after the frame's move, she follows the pad again, keeping the speed the move left her;
    /// else the dash joins her speed and she carries on, braking.
    pub(crate) fn as_running(&mut self, start: Fixed) {
        if start == ZERO || self.speed == ZERO {
            self.carry = Carry::FollowsPad;
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
        self.carry = Carry::FollowsPad;
        if self.dash != ZERO {
            self.way = Side::of(facing);
            self.carry = Carry::CarriesOn;
        }
    }

    /// `SpeedBuildsAndBrakes`: the side she moves to: the way her speed points, her facing's when
    /// it is 0 (§1). A skid braked to exactly 0 this frame still carries on, so its move of 0
    /// goes the old way, against her facing (a skid of 0 at the frame's start braked below zero
    /// and follows the pad, so it moves the new way).
    pub(crate) fn moving(&self, facing: Facing) -> Side {
        if self.speed == ZERO && self.carry == Carry::FollowsPad {
            Side::of(facing)
        } else {
            self.way
        }
    }

    /// `Dash`, `Dashing`: her dash, before her speed builds. Running with run held she is
    /// dashing from her moves on and her dash grows to its cap; in any other activity whose
    /// speed builds, the backward leap's included (`BackwardLeap`), a dash she has while not
    /// dashing is gone. A push neither builds nor loses it (`PushMoves`).
    pub(crate) fn dash_for_moves(&mut self, moves: Moves, t: &Tick) {
        match moves {
            Moves::Build(gait, _) if running_with_run_held(gait, t.frame.pad) => {
                self.dashing = Dashing::On;
                self.grow_dash(t.rules.dash_cap.of(&t.frame.items), t.rules);
            }
            Moves::Build(..) | Moves::Leap => self.lose_dash(),
            Moves::HaltsAfterDash
            | Moves::Rest
            | Moves::TakeOff
            | Moves::DashKeepingSpeeds
            | Moves::Pushed(_)
            | Moves::DropsInPlace => {}
        }
    }

    /// `SpeedBuildsAndBrakes`, `Dash`, `Jumping`: what her speed and dash become as her
    /// situation moves her (`moves`, facing `facing`), and what she moves by; `falls` is a
    /// vertical motion on the ground (a vertical speed, or falling at 0). The ball at rest
    /// braking or skidding rolls like running and keeps its dash; with a vertical motion it moves
    /// by its dash only, keeping both speeds. Taking off she moves by her dash only, a dash she is
    /// to lose gone first, and follows the pad. A push moves her by its speed and her dash; the
    /// frame after it she makes no horizontal move, keeping her speed; the leap moves as its pad
    /// says (`leap`).
    pub(crate) fn stride(&mut self, moves: Moves, facing: Facing, falls: bool, t: &Tick) -> Stride {
        match moves {
            Moves::Build(gait, drift) => self.drift(gait, drift, facing, t),
            Moves::Rest if self.carry == Carry::CarriesOn => self.brake(Drift::Ground, facing, t),
            Moves::Rest if falls => Stride::DashAlone,
            Moves::Rest | Moves::HaltsAfterDash => Stride::DashThenHalt,
            Moves::TakeOff => {
                self.lose_dash();
                self.carry = Carry::FollowsPad;
                Stride::DashAlone
            }
            Moves::DashKeepingSpeeds => Stride::DashAlone,
            Moves::Pushed(_) => {
                self.pushed(t.rules);
                Stride::SpeedAndDash
            }
            Moves::DropsInPlace => Stride::Nothing,
            Moves::Leap => self.leap(t),
        }
    }

    /// `PushMoves`: her speed's size builds by the push's gain to its top (`WordwiseTopSpeed`),
    /// or, where her motion carries on or skids, brakes by the push's brake, to no lower than 0.
    pub(crate) fn pushed(&mut self, r: &Rules) {
        if self.carry == Carry::CarriesOn {
            self.speed -= self.speed.min(r.push_brake);
        } else {
            self.gain(r.push, r);
        }
    }

    /// `BackwardLeap`: holding either side her speed's size builds by the leap's gain to its top
    /// (`WordwiseTopSpeed`), and she moves by it and her dash; holding neither her speed is 0 and
    /// she makes no horizontal move.
    fn leap(&mut self, t: &Tick) -> Stride {
        if t.frame.pad.any(SIDES) {
            self.gain(t.rules.leap, t.rules);
            Stride::SpeedAndDash
        } else {
            self.speed = ZERO;
            Stride::Nothing
        }
    }

    /// `SpeedBuildsAndBrakes`, `DriftInTheAir`: her speed building towards her gait's top, as her
    /// drift says. Carrying on along the ground she brakes, and a skid in the air brakes as on
    /// the ground, but not through a wall jump. Following the pad with neither side held, a
    /// straight jump or a fall stops drifting, and the falling ball halts but still moves by its
    /// dash. Else she builds; carrying on in the air she doesn't brake. A spin drifts as
    /// `spin_drift` says; a wall jump always carries on along her facing.
    fn drift(&mut self, gait: Gait, drift: Drift, facing: Facing, t: &Tick) -> Stride {
        let (r, pad) = (t.rules, t.frame.pad);
        let (ahead, carries) = (Side::of(facing), self.carry == Carry::CarriesOn);
        if carries && (drift == Drift::Ground || self.way != ahead && drift != Drift::WallJump) {
            return self.brake(drift, facing, t);
        }
        let let_go = !carries && !pad.any(SIDES);
        if let_go && drift == Drift::Air {
            self.speed = ZERO;
            return Stride::Nothing;
        }
        if let_go && drift == Drift::BallAir {
            return Stride::DashThenHalt;
        }
        let topped = self.gain(r.limits(gait), r);
        self.way = ahead;
        if drift == Drift::Spin {
            return self.spin_drift(topped, facing, t);
        }
        if drift == Drift::WallJump {
            self.carry = Carry::CarriesOn;
        }
        Stride::SpeedAndDash
    }

    /// `DriftInTheAir`: a spin not skidding: holding the side she faces (left and right together
    /// read so), or having reached her top speed this frame (`topped`), she carries on along her
    /// facing and moves; otherwise she stops drifting: horizontal speed 0, following the pad, and
    /// she moves by her dash alone.
    const fn spin_drift(&mut self, topped: bool, facing: Facing, t: &Tick) -> Stride {
        if topped || seen(t.frame.pad, facing).any(FORWARD) {
            self.way = Side::of(facing);
            self.carry = Carry::CarriesOn;
            Stride::SpeedAndDash
        } else {
            self.stop();
            Stride::DashAlone
        }
    }

    /// `SpeedBuildsAndBrakes`: her speed's size gains the step up to the top
    /// (`WordwiseTopSpeed`). Returns whether she took the top speed: the step was cut to it, so a
    /// step that lands exactly on it hasn't reached it (`DriftInTheAir`).
    fn gain(&mut self, limits: Limits, r: &Rules) -> bool {
        let sum = self.speed + limits.step;
        self.speed = r.cap(sum, limits.top);
        self.speed != sum
    }

    /// `SpeedBuildsAndBrakes`: carrying on along the ground, or skidding in the air (`drift`
    /// says where), she loses the brake and moves by her speed and dash. Where braking would take
    /// her below zero she stops at horizontal speed 0, with no speed-up step that frame: she
    /// follows the pad again, or, in a spin (`DriftInTheAir`), her skid ends there and she drifts
    /// as a spin not skidding does (`spin_drift`).
    fn brake(&mut self, drift: Drift, facing: Facing, t: &Tick) -> Stride {
        if self.speed >= t.rules.brake {
            self.speed -= t.rules.brake;
            return Stride::SpeedAndDash;
        }
        self.speed = ZERO;
        if drift == Drift::Spin {
            return self.spin_drift(false, facing, t);
        }
        self.carry = Carry::FollowsPad;
        Stride::SpeedAndDash
    }

    /// `Dash`: grows if, before the step, its pixel part is below the cap's or its subpixel part
    /// is below the cap's (`part_below`); else it becomes the cap. The cap is the items' (higher
    /// with the speed booster).
    fn grow_dash(&mut self, cap: Fixed, r: &Rules) {
        let ((p, s), (cp, cs)) = (self.dash.words(), cap.words());
        self.dash = if part_below(p, cp) || part_below(s, cs) {
            self.dash + r.dash_gain
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
