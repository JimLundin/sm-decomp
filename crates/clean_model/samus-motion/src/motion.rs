//! Her moves in a frame, as the pose at its start says (`MovesAsShePosed`): horizontally by her
//! speed and dash, then by the ground pull or her vertical speed, then the aim change's step;
//! and what those moves met.

use crate::collide::{Moved, Mover, Probe, Stop};
use crate::rows::{Doing, Drift, FORWARD, Met, Moves, Situation};
use crate::rules::{Gait, Limits, Rules};
use crate::{Bounces, Carry, Choice, DashIs, Frame, Samus, Seat, Spin, Vertical, WallTest};
use Bounces::{Once, Twice};
use Carry::{CarriesOn, FollowsPad};
use DashIs::{Hers, ToLose};
use Doing::{Ball, Hop, Running, Shaping, Spinning, StraightJump, TakingOff, Turning};
use Met::{Ceiling, Floor, NoFloor, Wall, WallKick};
use Probe::{Look, Move};
use Seat::{Seated, Unseated};
use Spin::{CarriedOn, First, Off, Second};
use Stride::{Both, Dash, DashThenHalt, Nothing};
use Vertical::{Falling, Ground, Moonfall, Rising};
use WallTest::{Kicked, Neither, Readied};
use clean_types::Body::SpringBall;
use clean_types::Heading::{self, Down, Left, Right, Up};
use clean_types::button::{JUMP, LEFT, RIGHT, RUN};
use clean_types::{Action, Buttons, Facing, Fixed, Gun, Position, ZERO, below};

/// `DriftInTheAir`, `WallJump`: left or right.
const SIDES: Buttons = LEFT.and(RIGHT);

/// `SpeedBuildsAndBrakes`: what a frame's horizontal move is made of.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Stride {
    /// No move.
    Nothing,
    /// Her dash alone.
    Dash,
    /// Her dash alone, then a halt.
    DashThenHalt,
    /// Her speed and her dash.
    Both,
}

/// `FollowingTheGround`, `WallsAndCeilings`: what a walk did: the speed the ground pull reads,
/// and what stopped her.
struct Walk {
    /// Her horizontal speed and dash, from before any stop.
    pull: Fixed,
    /// What stopped the move.
    stop: Option<Stop>,
}

/// `WhatHerMovesMet`: what a frame's moves met, and the near-wall test's outcome (`WallJump`).
#[derive(Default)]
pub struct Meeting {
    /// What her moves met.
    pub met: Option<Met>,
    /// The near-wall test's outcome.
    pub wall_test: WallTest,
}

impl Samus {
    /// The frame's start: `WallJump` (§1.2): how long she has been in this spin;
    /// `RisingAndFalling`: the rise ends at the start of the frame after it is spent, and in a
    /// straight jump, a spin or the hop at the start of any frame with jump not held.
    pub(crate) fn begin(&mut self, doing: Doing, pad: Buttons) {
        let next = match self.spin {
            Off => First,
            First => Second,
            Second | CarriedOn => CarriedOn,
        };
        self.spin = if doing == Spinning { next } else { Off };
        let lets_go = !pad.any(JUMP) && matches!(doing, StraightJump | Spinning | Hop);
        let spent = !self.vy.negative() && !self.vy.is_zero();
        if self.vertical == Rising && (lets_go || spent) {
            self.fall_from_rest();
        }
    }

    /// `MovesAsShePosed`: her moves, as the pose at the frame's start says, and what they met.
    pub(crate) fn moves(&mut self, s: &Situation, frame: &Frame, m: &mut Mover) -> Meeting {
        if self.pose.facing == Facing::Screen {
            // SpeedBuildsAndBrakes: facing the screen she doesn't move; RidingTheElevator: on a
            // moving elevator she moves down 1 px, a move stopped as any downward move.
            if frame.elevator_moving {
                self.position = m.go(self.bounds(), Down, m.rules.elevator, Move).to;
            }
            return Meeting::default();
        }
        let (walk, doing) = (self.walk(s.moves, frame.pad, m), s.doing);
        let wall_test = self.wall_test(frame, m);
        // FollowingTheGround: on the ground she is pulled onto the floor, unless as the ball or
        // while her body changes shape she has a vertical speed to move by.
        let pulled =
            self.vertical == Ground && (self.vy.is_zero() || !matches!(doing, Shaping | Ball));
        let met = if wall_test == Kicked {
            // WallJump: kicking off she doesn't move vertically that frame.
            Some(WallKick)
        } else if doing == TakingOff {
            // Jumping: taking off she doesn't move vertically and the pull doesn't act.
            None
        } else if pulled {
            self.pull(doing, walk.pull, m)
        } else {
            self.fall(doing, frame, m)
        };
        // WhatHerMovesMet: a wall stop while running wins over walking off a ledge.
        let stopped = walk.stop.is_some() && doing == Running;
        let met = if stopped { Some(Wall) } else { met };
        Meeting { met, wall_test }
    }

    /// `SpeedBuildsAndBrakes`, `Dash`, `DriftInTheAir`, `WallsAndCeilings`: her horizontal move
    /// by her speed and dash, as her situation moves her.
    fn walk(&mut self, moves: Moves, pad: Buttons, m: &mut Mover) -> Walk {
        let stride = self.stride(moves, pad, m.rules);
        // §1: the dash is taken the way the speed points, her facing when it is 0.
        let way = if self.speed.is_zero() {
            self.pose.facing.side()
        } else {
            self.way
        };
        let pull = self.speed.plus(self.dash);
        let by = if stride == Both { pull } else { self.dash };
        let stop = if stride == Nothing {
            None
        } else {
            self.walk_and_seat(way, by, m)
        };
        // WallsAndCeilings, SpeedBuildsAndBrakes: a halt.
        if stop.is_some() || stride == DashThenHalt {
            self.halt();
        }
        Walk { pull, stop }
    }

    /// `SpeedBuildsAndBrakes`, `Dash`, `DriftInTheAir`, `Jumping`: what her speed and dash
    /// become as her situation moves her, and what she moves by. Left and right held together
    /// read as the side she faces held.
    fn stride(&mut self, moves: Moves, pad: Buttons, r: &Rules) -> Stride {
        let carries = self.carry == CarriesOn;
        let (gait, drift) = match moves {
            Moves::Build(gait, drift) => (gait, drift),
            // The ball at rest braking or skidding rolls like running, and keeps its dash.
            Moves::Rest if carries => {
                self.brake(r);
                return Both;
            }
            // The ball at rest with a vertical speed moves by its dash only, keeping both speeds.
            Moves::Rest if !self.vy.is_zero() => return Dash,
            Moves::Rest | Moves::DashThenHalt => return DashThenHalt,
            // Jumping: taking off she moves by her dash only, a dash she is to lose gone first,
            // and follows the pad.
            Moves::TakeOff => {
                self.lose_dash();
                self.carry = FollowsPad;
                return Dash;
            }
            Moves::Dash => return Dash,
        };
        // Dash: running with run held she gains dash; in any other moving activity a dash she is
        // to lose is gone.
        if gait == Gait::Running && pad.any(RUN) {
            self.grow_dash(r);
        } else {
            self.lose_dash();
        }
        // SpeedBuildsAndBrakes: carrying on along the ground she brakes; DriftInTheAir: a skid in
        // the air brakes, as on the ground, but not through a wall jump.
        let facing = self.pose.facing.side();
        if carries && (drift == Drift::Ground || self.way != facing && drift != Drift::WallJump) {
            self.brake(r);
            return Both;
        }
        // DriftInTheAir: following the pad with neither side held she stops drifting; the
        // falling ball instead halts, but still moves by its dash.
        let let_go = !carries && !pad.any(SIDES);
        if let_go && drift == Drift::Air {
            self.speed = ZERO;
            return Nothing;
        }
        if let_go && drift == Drift::BallAir {
            return DashThenHalt;
        }
        // SpeedBuildsAndBrakes: following the pad she builds; DriftInTheAir: carrying on in the
        // air she doesn't brake. A spin drifts only while she holds the side she faces or reached
        // her top speed this frame, else she stops drifting and moves by her dash alone; a spin
        // drifting, and a wall jump always, carry on along her facing.
        let topped = self.build(r.limits(gait), facing, r);
        if drift == Drift::Spin && !topped && !pad.seen(self.pose.facing).any(FORWARD) {
            self.speed = ZERO;
            self.carry = FollowsPad;
            return Dash;
        }
        if matches!(drift, Drift::Spin | Drift::WallJump) {
            self.carry = CarriesOn;
        }
        Both
    }

    /// `SpeedBuildsAndBrakes`: gains the step up to the top (`WordwiseTopSpeed`), along her
    /// facing. Returns whether she took the top speed.
    fn build(&mut self, limits: Limits, facing: Heading, r: &Rules) -> bool {
        let sum = self.speed.plus(limits.step);
        self.speed = r.cap(sum, limits.top);
        self.way = facing;
        self.speed != sum
    }

    /// `SpeedBuildsAndBrakes`: carrying on she loses the brake; below zero she stops and follows
    /// the pad again.
    fn brake(&mut self, r: &Rules) {
        if self.speed < r.brake {
            self.speed = ZERO;
            self.carry = FollowsPad;
        } else {
            self.speed = self.speed.minus(r.brake);
        }
    }

    /// `Dash`: grows if, before the step, its pixel part is below the cap's or its subpixel part
    /// is below the cap's; else it becomes the cap.
    const fn grow_dash(&mut self, r: &Rules) {
        let (d, cap) = (self.dash, r.dash_cap);
        self.dash = if below(d.pixel(), cap.pixel()) || below(d.sub(), cap.sub()) {
            d.plus(r.dash_step)
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

    /// `BlocksStopFlush`, `SlopesSlowTheWalk`: a horizontal move, which a sloped floor at her
    /// leading edge shortens when it is made with her vertical speed exactly 0.
    fn hmove(&self, way: Heading, d: Fixed, m: &mut Mover) -> Moved {
        let d = if self.vy.is_zero() {
            m.walk_shortened(self.bounds(), way, d)
        } else {
            d
        };
        m.go(self.bounds(), way, d, Move)
    }

    /// `SpeedBuildsAndBrakes`, `SlopesCarry`: a horizontal move, then the checks that run after
    /// every horizontal move, whether or not it met solid enemies, and whose lift or push seats
    /// her. Returns what stopped the move.
    fn walk_and_seat(&mut self, way: Heading, d: Fixed, m: &mut Mover) -> Option<Stop> {
        let moved = self.hmove(way, d, m);
        self.position = moved.to;
        if let Some(at) = m.seat(self.bounds()) {
            self.position = at;
            self.seat = Seated;
        }
        moved.stop
    }

    /// `AimingAndShooting`: a frame whose chosen pose brings the step moves her 1 px forward, by
    /// her facing at the frame's start, after her moves: a horizontal move with no `SlopesCarry`
    /// checks after it. A solid enemy within its reach stops it with no move at all, one she
    /// touches still clearing her y subpixel. Something in the way puts her against the wall
    /// instead, with the running pose's aim (`WhatHerMovesMet`), unless an animation's end
    /// handed over another pose.
    pub(crate) fn aim_step(&mut self, chosen: &mut Choice, m: &mut Mover) {
        let moved = self.hmove(self.pose.facing.side(), m.rules.aim_step, m);
        self.position.y = moved.to.y;
        if moved.stop != Some(Stop::Enemy) {
            self.position.x = moved.to.x;
        }
        if moved.stop.is_some() && chosen.pose.action == Action::Running {
            chosen.pose.action = Action::AgainstWall;
            chosen.pose.gun = Gun::Stowed;
            chosen.effects = &[];
        }
    }

    /// `WallJump`: the near-wall test, in a spin carried on with a side held, looking 8 px away
    /// from it (left held: a wall on her right; else right held: on her left) from where her
    /// move left her. What stops it writes to her as a stop would, without moving her: a block
    /// her x subpixel, a solid enemy she touches her y subpixel. A wall near her readies the
    /// wall jump; once the spin has turned far, pressing jump kicks her off.
    fn wall_test(&mut self, frame: &Frame, m: &mut Mover) -> WallTest {
        if self.spin != CarriedOn || !frame.pad.any(SIDES) {
            return Neither;
        }
        let wall = if frame.pad.any(LEFT) { Right } else { Left };
        let seen = m.go(self.bounds(), wall, m.rules.wall_reach, Look);
        self.position = Position {
            x: self.position.x.with_sub(seen.to.x.sub()),
            y: seen.to.y,
        };
        if seen.stop.is_none() {
            Neither
        } else if !frame.spin_turned_far {
            Readied
        } else if frame.pressed.any(JUMP) {
            Kicked
        } else {
            Neither
        }
    }

    /// `BlocksStopFlush`: a vertical move by `v`, signed, downward positive. `SlopesCarry`: as a
    /// move of more than 0 starts she leaves any slope.
    fn vmove(&mut self, v: Fixed, m: &mut Mover) -> Moved {
        if !v.is_zero() {
            self.seat = Unseated;
        }
        let way = if v.negative() { Up } else { Down };
        m.go(self.bounds(), way, v.size(), Move)
    }

    /// `SlopesCarry`: a downward move or look that a stepped block stops seats her.
    pub(crate) fn seat_on(&mut self, stop: Option<Stop>) {
        if stop == Some(Stop::Stepped) {
            self.seat = Seated;
        }
    }

    /// `FollowingTheGround`: the pull down after her horizontal move, by that frame's speed (her
    /// horizontal speed plus her dash, subpixel part included) plus 1 px (`SlopesCarry`: 1 px on
    /// a slope); the floor stops her flush and never lands her. Walking off a ledge, unless
    /// turning (`TurningRound`: she sinks instead) or changing shape, she falls from rest.
    fn pull(&mut self, doing: Doing, speed: Fixed, m: &mut Mover) -> Option<Met> {
        let r = m.rules;
        let by = if self.seat == Seated {
            r.ground_pull
        } else {
            speed.plus(r.ground_pull)
        };
        let moved = self.vmove(by, m);
        if moved.stop.is_some() || doing == Turning {
            self.position = moved.to;
            self.seat_on(moved.stop);
            return None;
        }
        if doing == Shaping {
            return None;
        }
        self.walk_off();
        Some(NoFloor)
    }

    /// `FollowingTheGround`: walking off a ledge she falls from rest, bounces 0.
    const fn walk_off(&mut self) {
        self.fall_from_rest();
        self.bounces = Bounces::None;
    }

    /// `RisingAndFalling`, `WallsAndCeilings`, `FollowingTheGround`, `Jumping`'s space jump: her
    /// move by her vertical speed, which then grows by the pull. An upward move stopped is a
    /// ceiling: the rise ends with no pull that frame. A downward one stopped is a floor, but
    /// the ball on the ground meets it without landing, and a move of hers by its speed that
    /// finds no floor is a walk off a ledge.
    fn fall(&mut self, doing: Doing, frame: &Frame, m: &mut Mover) -> Option<Met> {
        let r = m.rules;
        // Jumping: with space jump, pressing jump while falling in a spin with her fall speed in
        // the window jumps again, rising that same frame; not in a moonfall (`Moonfall`).
        if self.vertical == Falling
            && doing == Spinning
            && frame.pressed.any(JUMP)
            && frame.equipment.space_jump
            && r.space_jump.contains(&self.vy.as_8_8())
        {
            self.rise(r.jump.of(frame.equipment.hi_jump));
        }
        let up = self.vy.negative();
        let moved = self.vmove(self.vy, m);
        self.position = moved.to;
        if up && moved.stop.is_some() {
            self.fall_from_rest();
            return Some(Ceiling);
        }
        self.seat_on(moved.stop);
        self.vy = self.pulled(r);
        let on_ground = self.vertical == Ground && doing == Ball;
        if on_ground && moved.stop.is_none() {
            self.walk_off();
            return Some(NoFloor);
        }
        moved.stop.filter(|_| !on_ground).map(|_| Floor)
    }

    /// `RisingAndFalling`: her vertical speed after the pull, which stops growing at pixel part
    /// 5; `Moonfall`: with no limit.
    fn pulled(&self, r: &Rules) -> Fixed {
        if self.vertical != Moonfall && self.vy.pixel() == r.fall_settle_px {
            self.vy
        } else {
            self.vy.plus(r.gravity)
        }
    }

    /// `Landing`, `WallJump`'s kick: what her moves met acts on her.
    pub(crate) fn meet(&mut self, met: Option<Met>, doing: Doing, frame: &Frame, r: &Rules) {
        match met {
            Some(Floor) => self.land(doing, frame, r),
            // WallJump: the kick starts the wall jump rising at the wall-jump speed, x speed 0,
            // following the pad; it ends a moonfall (`Moonfall`).
            Some(WallKick) => {
                self.rise(r.wall_jump.of(frame.equipment.hi_jump));
                self.speed = ZERO;
                self.carry = FollowsPad;
            }
            Some(Wall | Met::AimStep | NoFloor | Ceiling) | None => {}
        }
    }

    /// `Landing`: a move by her vertical speed met a floor. Turning, only the move stops: her
    /// speed goes on growing by the pull. The spring ball with jump held hops again. The ball
    /// landing at 3 px or more bounces, twice before it rests, the second time 1 px slower: the
    /// speed it met the floor at is hers less this frame's pull (§9), or held at the fall's limit,
    /// which is past 3 px either way. While her body changes shape only her vertical motion
    /// stops. Else she lands: vertical speed 0, bounces 0, horizontal speed 0, following the
    /// pad; the dash keeps its value, to lose (§1.2).
    fn land(&mut self, doing: Doing, frame: &Frame, r: &Rules) {
        let ball = matches!(doing, Ball | Hop);
        let fast = self.vy.minus(r.gravity).pixel() >= r.bounce_from_px;
        if doing == Turning {
            return;
        }
        if ball && self.pose.body == SpringBall && frame.pad.any(JUMP) {
            self.rise(r.jump.of(frame.equipment.hi_jump));
            self.bounces = Bounces::None;
            return;
        }
        if ball && fast && self.bounces == Bounces::None {
            self.rise(r.bounce);
            self.bounces = Once;
            return;
        }
        if ball && fast && self.bounces == Once {
            self.rise(r.bounce.minus(r.second_bounce_less));
            self.bounces = Twice;
            return;
        }
        self.vertical = Ground;
        self.vy = ZERO;
        if doing == Shaping {
            return;
        }
        self.bounces = Bounces::None;
        self.speed = ZERO;
        self.carry = FollowsPad;
        self.dash_is = ToLose;
    }
}
