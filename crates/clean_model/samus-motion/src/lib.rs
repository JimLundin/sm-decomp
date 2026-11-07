//! The samus-motion model, built from its behaviour spec alone: a character controller for
//! Samus, stepped one 60 Hz frame at a time, her motion in fixed-point subpixels.
//!
//! Every rule names the spec entry it comes from. A frame (`PoseFitAfterMoves`): she moves as
//! the pose at its start says (`MovesAsShePosed`), horizontally, then by the ground pull or her
//! vertical move, then by the aim change's step; what those moves met is known; the pose is
//! chosen (`PoseFollows`); pose fit or the shrink places her; then the change's effects act.

pub mod catalogue;
pub mod chart;
mod collide;
pub mod rules;
pub mod situation;

pub use catalogue::{Behaviour, BehaviourId, CATALOGUE};
pub use chart::{Choice, Effect, Met, Moment};
pub use rules::{Boots, Limits, Rules, SpeedCap};
pub use situation::{Doing, Situation, classify};

use Carry::{CarriesOn, FollowsPad};
use Effect::{
    AsRunning, CarrySpin, EndSkid, Halt, LoseDash, Lower, Rise, StartDrift, TakeOff, TurnRound,
};
use Met::{AimStep, Ceiling, Floor, NoFloor, Wall, WallKick};
use Probe::{Blocks, Enemies, Look};
use Seat::{Seated, Unseated};
use Spin::{CarriedOn, First, Off, Second};
use Stride::{Both, Dash, DashThenHalt, Nothing};
use Vertical::{Falls, Ground, Moonfall, Rises, Spent};
use WallTest::{Kicked, Neither, Readied};
use clean_types::Action::{AgainstWall, Running, Still};
use clean_types::Body::{Crouched, SpringBall, Upright};
use clean_types::Heading::{Down, Left, Right, Up};
use clean_types::{
    Aim, Cell, Equipment, Facing, Fixed, Gun, Heading, Legs, Pad, Parity, Pose, Position, Radius,
    Touch, World, below,
};
use collide::{Moved, Mover, Probe, Stop};
use situation::{Drift, Gait, Moves};

/// Her vertical motion.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Vertical {
    /// On the ground.
    Ground,
    /// Rising at this upward speed; at 0 she rises by 0 for one more frame.
    Rises(Fixed),
    /// `RisingAndFalling`: the pull took the upward speed past zero, so it points downward by
    /// this; the rise ends at the start of the next frame.
    Spent(Fixed),
    /// Falling at this speed; 0 is a fall just started.
    Falls(Fixed),
    /// `Moonfall`: falling at this speed with no falling limit and no jump, until she lands.
    Moonfall(Fixed),
}

/// `Landing`: how many times the ball has bounced since it last rested.
#[derive(Clone, Copy)]
pub enum Bounces {
    /// None.
    None,
    /// One.
    Once,
    /// Two.
    Twice,
}

/// §1.2 a spin carried on: how long she has been in this spin.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Spin {
    /// Not spinning.
    Off,
    /// Its first frame.
    First,
    /// Its second frame.
    Second,
    /// In it two frames earlier, or it followed a wall jump.
    CarriedOn,
}

/// §1.2 carrying on: how her horizontal motion goes.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Carry {
    /// It follows the pad.
    FollowsPad,
    /// It carries on by itself: it brakes on the ground and keeps building in the air; against
    /// her facing, a skid.
    CarriesOn,
}

/// §1.2 her dash.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DashIs {
    /// Hers to keep.
    Hers,
    /// Left on her by a landing: gone the next frame she moves.
    ToLose,
}

/// `SlopesCarry`: whether she is seated on a slope, until her next vertical move of more than 0
/// starts.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Seat {
    /// Not seated.
    Unseated,
    /// Seated on a slope: her ground pull is 1 px.
    Seated,
}

/// What one frame brings from the player and the world (§1.3), and from what is out of scope
/// and feeds us (the header): an animation's end from its timing.
#[derive(Clone, Copy)]
pub struct Frame<'a> {
    /// The buttons held as the frame starts.
    pub pad: Pad,
    /// The buttons just pressed this frame.
    pub pressed: Pad,
    /// Her equipment.
    pub equipment: Equipment,
    /// `WallJump`: the spin's sprite has turned far enough to kick off a wall.
    pub spin_turned_far: bool,
    /// `RidingTheElevator`: the elevator is moving.
    pub elevator_moving: bool,
    /// `BlocksStopFlush`: the frame count's parity.
    pub parity: Parity,
    /// `AnimationEnds`: her pose's animation ends this frame.
    pub animation_ends: bool,
    /// §1.3 body size: each pose's half-width and half-height, the game's data.
    pub body: &'a dyn Fn(Pose) -> Radius,
}

/// `WallJump`'s output: the near-wall test's outcome this frame.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum WallTest {
    /// No test, no wall found, or a wall near her once the spin has turned far, without jump
    /// pressed.
    Neither,
    /// A wall near her while the spin hasn't turned far: the spin's animation skips ahead.
    Readied,
    /// She kicked off the wall.
    Kicked,
}

/// `PoseFitAfterMoves`: a push, in whole pixels.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Push {
    /// The way she is pushed.
    pub way: Heading,
    /// How far.
    pub px: u16,
}

/// `PoseFitAfterMoves`: how the pose the frame ends in came about.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Change {
    /// The chosen pose, with pose fit's push, if any.
    Chosen(Option<Push>),
    /// The change refused: she keeps the pose she had.
    Refused,
    /// Crouched `Still` taken instead.
    Crouched,
}

/// `AimingAndShooting`: the standing animation.
#[derive(Clone, Copy)]
pub enum Standing {
    /// It plays on.
    Plays,
    /// It restarts.
    Restarts,
}

/// What a frame shows beyond her quantities: events for the caller to act on.
pub struct Outcome {
    /// `PoseFitAfterMoves`: how her pose came about.
    pub change: Change,
    /// `WhatHerMovesMet`: what her moves met.
    pub met: Option<Met>,
    /// `WallJump`: the near-wall test's outcome.
    pub wall_test: WallTest,
    /// `AimingAndShooting`: the standing animation.
    pub standing: Standing,
    /// `TouchedCells`: the cells her moves and looks reached, in the order they happen.
    pub reached: Vec<Touch>,
    /// `TouchedCells`: the cells she was inside at the frame's start.
    pub inside: Vec<Cell>,
}

/// Samus as the player can tell her, and what she remembers.
#[derive(Clone, Copy)]
pub struct Samus {
    /// Her body's centre.
    pub position: Position,
    /// Her horizontal speed's size.
    pub speed: Fixed,
    /// The way her horizontal speed points, left or right.
    pub way: Heading,
    /// §1.2 carrying on.
    pub carry: Carry,
    /// Her dash, added the way she moves.
    pub dash: Fixed,
    /// §1.2 her dash.
    pub dash_is: DashIs,
    /// Her vertical motion.
    pub vertical: Vertical,
    /// The ball's bounces.
    pub bounces: Bounces,
    /// Her pose.
    pub pose: Pose,
    /// Her body's half-width and half-height.
    pub radius: Radius,
    /// How long she has been in a spin.
    pub spin: Spin,
    /// `SlopesCarry`: seated on a slope.
    pub seat: Seat,
}

/// What a frame's horizontal move is made of (`SpeedBuildsAndBrakes`).
#[derive(Clone, Copy)]
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

/// What a walk did: the speed the ground pull reads (`FollowingTheGround`), and what stopped
/// her.
struct Walk {
    /// Her horizontal speed and dash, from before any stop.
    pull: Fixed,
    /// What stopped the move.
    stop: Option<Stop>,
}

/// `PoseFitAfterMoves`: with something found above her alone (up, down), the way she is pushed
/// away from it and the room on its side; with something below alone, the same upward.
const fn alone(found: (Option<u16>, Option<u16>)) -> Option<(Heading, u16)> {
    match found {
        (Some(room), None) => Some((Down, room)),
        (None, Some(room)) => Some((Up, room)),
        _ => None,
    }
}

/// `PoseFitAfterMoves`: crouched `Still`, aim `Forward`, facing as before the change.
const fn crouched(p: Pose) -> Pose {
    Pose {
        body: Crouched,
        action: Still,
        aim: Aim::Forward,
        gun: Gun::Stowed,
        legs: Legs::Together,
        ..p
    }
}

impl Samus {
    /// One frame (`PoseFitAfterMoves`): her moves as the pose at its start says
    /// (`MovesAsShePosed`) and what they met; the pose chosen (`PoseFollows`); pose fit or the
    /// shrink; then the change's effects.
    pub fn step(&mut self, frame: &Frame, world: &dyn World, rules: &Rules) -> Outcome {
        let mut m = Mover {
            world,
            rules,
            parity: frame.parity,
            reached: Vec::new(),
        };
        // TouchedCells: the cells she is inside, before her moves.
        let inside = m.inside(self.position, self.radius);
        let start = self.pose;
        let situation = classify(start, self.vertical == Ground);
        self.begin(situation.doing, frame.pad);
        let (met, wall_test) = self.moves(situation, frame, &mut m);
        // PoseFitAfterMoves: her speeds as her moves left them, for a change that doesn't come.
        let moved = Self {
            vertical: if met == Some(Floor) {
                self.pulled(rules)
            } else {
                self.vertical
            },
            ..*self
        };
        self.meet(met, situation.doing, frame, rules);
        let mut moment = Moment {
            situation,
            pose: start,
            airborne: self.vertical != Ground,
            pad: frame.pad,
            pressed: frame.pressed,
            equipment: frame.equipment,
            met,
        };
        let by_moves = self.aim_step(&mut moment, &mut m);
        let chosen = if frame.animation_ends {
            moment.handed_over()
        } else {
            None
        }
        .unwrap_or(by_moves);
        let change = self.fit(chosen.pose, frame, &mut m);
        self.take(change, chosen, moved, frame, &mut m);
        // AimingAndShooting: standing, unaimed and not landing, holding shoot restarts the
        // standing animation.
        let restarts = start.body == Upright
            && start.action == Still
            && start.aim == Aim::Forward
            && frame.pad.shoot;
        Outcome {
            change,
            met: moment.met,
            wall_test,
            standing: if restarts {
                Standing::Restarts
            } else {
                Standing::Plays
            },
            reached: m.reached,
            inside,
        }
    }

    /// The frame's start: §1.2's spin count, and `RisingAndFalling`: the rise ends at the start
    /// of the frame after it is spent, and in a straight jump, a spin or the hop at the start of
    /// any frame with jump not held.
    fn begin(&mut self, doing: Doing, pad: Pad) {
        self.spin = if doing == Doing::Spinning {
            match self.spin {
                Off => First,
                First => Second,
                Second | CarriedOn => CarriedOn,
            }
        } else {
            Off
        };
        let lets_go =
            !pad.jump && matches!(doing, Doing::StraightJump | Doing::Spinning | Doing::Hop);
        if matches!(self.vertical, Spent(_)) || lets_go && matches!(self.vertical, Rises(_)) {
            self.vertical = Falls(Fixed(0));
        }
    }

    /// Her moves, as the pose at the frame's start says: what they met, and the near-wall
    /// test's outcome.
    fn moves(&mut self, s: &Situation, frame: &Frame, m: &mut Mover) -> (Option<Met>, WallTest) {
        if self.pose.facing == Facing::Screen {
            // SpeedBuildsAndBrakes: facing the screen she doesn't move; RidingTheElevator: on a
            // moving elevator she moves down 1 px, a move stopped as any downward move.
            if frame.elevator_moving {
                let elevator = m.rules.elevator;
                self.position = m
                    .go(self.position, self.radius, Down, elevator, Probe::Move)
                    .to;
            }
            return (None, Neither);
        }
        let walk = self.walk(s.moves, frame.pad, m);
        let test = self.wall_test(frame, m);
        let met = if test == Kicked {
            // WallJump: kicking off she doesn't move vertically that frame.
            Some(WallKick)
        } else if s.doing == Doing::TakingOff {
            // Jumping: taking off she doesn't move vertically and the pull doesn't act.
            None
        } else if self.vertical == Ground {
            self.pull(s.doing, walk.pull, m)
        } else {
            self.rise_or_fall(s.doing, frame, m)
        };
        // WhatHerMovesMet: a wall stop while running wins over walking off a ledge.
        if walk.stop.is_some() && s.doing == Doing::Running {
            return (Some(Wall), test);
        }
        (met, test)
    }

    /// `SpeedBuildsAndBrakes`, `Dash`, `DriftInTheAir`, `WallsAndCeilings`: her horizontal move
    /// by her speed and dash, as her situation moves her.
    fn walk(&mut self, moves: Moves, pad: Pad, m: &mut Mover) -> Walk {
        let r = m.rules;
        let stride = match moves {
            Moves::Build(gait, drift) => {
                // Dash: running with run held she gains dash; in any other moving activity a
                // dash she is to lose is gone.
                if gait == Gait::Running && pad.run {
                    self.grow_dash(r);
                } else {
                    self.lose_dash();
                }
                self.drift(r.limits(gait), drift, pad, r)
            }
            // SpeedBuildsAndBrakes: the ball at rest braking or skidding rolls like running, and
            // keeps its dash.
            Moves::Rest if self.carry == CarriesOn => {
                self.brake(r);
                Both
            }
            Moves::Rest | Moves::DashThenHalt => DashThenHalt,
            // Jumping: taking off she moves by her dash only, a dash she is to lose gone first,
            // and follows the pad.
            Moves::TakeOff => {
                self.lose_dash();
                self.carry = FollowsPad;
                Dash
            }
            Moves::Dash => Dash,
        };
        // §1: the dash is taken the way the speed points, her facing when it is 0.
        let way = if self.speed.is_zero() {
            self.pose.facing.side()
        } else {
            self.way
        };
        let (dash, pull) = (self.dash, self.speed.plus(self.dash));
        let stop = match stride {
            Nothing => None,
            Dash | DashThenHalt => self.walk_and_seat(way, dash, m),
            Both => self.walk_and_seat(way, pull, m),
        };
        // WallsAndCeilings, SpeedBuildsAndBrakes: a halt.
        if stop.is_some() || matches!(stride, DashThenHalt) {
            self.halt();
        }
        Walk { pull, stop }
    }

    /// `SpeedBuildsAndBrakes`, `DriftInTheAir`: builds or brakes her speed as she drifts; returns
    /// what she moves by.
    fn drift(&mut self, limits: Limits, drift: Drift, pad: Pad, r: &Rules) -> Stride {
        let facing = self.pose.facing.side();
        let carries = self.carry == CarriesOn;
        let skids = carries && self.way != facing;
        let side = pad.left || pad.right;
        match drift {
            // SpeedBuildsAndBrakes: carrying on along the ground she brakes.
            Drift::Ground if carries => self.brake(r),
            // DriftInTheAir: a skid in the air brakes, as on the ground.
            Drift::Air | Drift::BallAir | Drift::Spin if skids => self.brake(r),
            // DriftInTheAir: following the pad with neither side held she stops drifting; the
            // falling ball instead halts, but still moves by its dash.
            Drift::Air if !carries && !side => {
                self.speed = Fixed(0);
                return Nothing;
            }
            Drift::BallAir if !carries && !side => return DashThenHalt,
            // SpeedBuildsAndBrakes: following the pad she builds; DriftInTheAir: carrying on in
            // the air doesn't brake.
            Drift::Ground | Drift::Air | Drift::BallAir => {
                self.build(limits, facing, r);
            }
            // DriftInTheAir: a spin drifts only while she holds the side she faces or reached
            // her top speed this frame; else she stops drifting and moves by her dash alone.
            Drift::Spin => {
                let forward = if facing == Left { pad.left } else { pad.right };
                if !(self.build(limits, facing, r) || forward) {
                    self.speed = Fixed(0);
                    self.carry = FollowsPad;
                    return Dash;
                }
                self.carry = CarriesOn;
            }
            // DriftInTheAir: through a wall jump she always drifts, carrying on along her facing.
            Drift::WallJump => {
                self.build(limits, facing, r);
                self.carry = CarriesOn;
            }
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
            self.speed = Fixed(0);
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
        self.dash_is = DashIs::Hers;
    }

    /// §1.2 her dash: a dash she is to lose is gone the next frame she moves.
    fn lose_dash(&mut self) {
        if self.dash_is == DashIs::ToLose {
            self.dash = Fixed(0);
        }
    }

    /// A halt: horizontal speed 0, dash 0, following the pad.
    const fn halt(&mut self) {
        self.speed = Fixed(0);
        self.dash = Fixed(0);
        self.carry = FollowsPad;
    }

    /// `SlopesSlowTheWalk`: a horizontal move made while her vertical speed is exactly 0 is
    /// shortened by the sloped floors at her leading edge.
    fn shortened(&self, way: Heading, d: Fixed, m: &Mover) -> Fixed {
        if matches!(
            self.vertical,
            Ground | Rises(Fixed(0)) | Spent(Fixed(0)) | Falls(Fixed(0)) | Moonfall(Fixed(0))
        ) {
            m.walk_shortened(self.position, self.radius, way, d)
        } else {
            d
        }
    }

    /// A horizontal move, then `SlopesCarry`'s checks, which run after every horizontal move,
    /// whether or not it met solid enemies, and whose lift or push seats her. Returns what
    /// stopped the move.
    fn walk_and_seat(&mut self, way: Heading, d: Fixed, m: &mut Mover) -> Option<Stop> {
        let d = self.shortened(way, d, m);
        let moved = m.go(self.position, self.radius, way, d, Probe::Move);
        self.position = m.seat(moved.to, self.radius).map_or(moved.to, |at| {
            self.seat = Seated;
            at
        });
        moved.stop
    }

    /// `AimingAndShooting`: a frame whose pose is chosen as a running pose other than hers (by
    /// what her moves met or the pad) steps her 1 px forward after her moves: a horizontal move
    /// with no `SlopesCarry` checks after it. A solid enemy within its reach stops it with no
    /// move at all, one she touches still clearing her y subpixel. Something in the way puts
    /// her against the wall instead, with the running pose's aim (`WhatHerMovesMet`).
    fn aim_step(&mut self, moment: &mut Moment, m: &mut Mover) -> Choice {
        let chosen = moment.by_moves();
        if chosen.pose.action != Running || chosen.pose == moment.pose {
            return chosen;
        }
        let side = moment.pose.facing.side();
        let d = self.shortened(side, m.rules.aim_step, m);
        let Moved { to, stop } = m.go(self.position, self.radius, side, d, Probe::Move);
        self.position = if stop == Some(Stop::Enemy) {
            Position {
                y: to.y,
                ..self.position
            }
        } else {
            to
        };
        if stop.is_none() {
            return chosen;
        }
        moment.met = Some(AimStep);
        Choice {
            pose: Pose {
                action: AgainstWall,
                gun: Gun::Stowed,
                ..chosen.pose
            },
            effects: &[],
        }
    }

    /// `WallJump`: the near-wall test, in a spin carried on with a side held, looking 8 px away
    /// from it (left held: a wall on her right; else right held: on her left) from where her
    /// move left her. What stops it writes to her as a stop would, without moving her: a block
    /// her x subpixel, a solid enemy she touches her y subpixel. A wall near her readies the
    /// wall jump; once the spin has turned far, pressing jump kicks her off.
    fn wall_test(&mut self, frame: &Frame, m: &mut Mover) -> WallTest {
        let pad = frame.pad;
        if self.spin != CarriedOn || !(pad.left || pad.right) {
            return Neither;
        }
        let (at, reach) = (self.position, m.rules.wall_reach);
        let wall = if pad.left { Right } else { Left };
        let seen = m.look(at, self.radius, wall, reach, Look);
        self.position = Position {
            x: at.x.with_sub(seen.to.x.sub()),
            y: seen.to.y,
        };
        match (seen.stop, frame.spin_turned_far, frame.pressed.jump) {
            (None, ..) | (Some(_), true, false) => Neither,
            (Some(_), false, _) => Readied,
            (Some(_), true, true) => Kicked,
        }
    }

    /// Where a vertical move would leave her. `SlopesCarry`: as a move of more than 0 starts she
    /// leaves any slope.
    fn vmove(&mut self, heading: Heading, d: Fixed, m: &mut Mover) -> Moved {
        if !d.is_zero() {
            self.seat = Unseated;
        }
        m.go(self.position, self.radius, heading, d, Probe::Move)
    }

    /// `SlopesCarry`: a downward move or look that a stepped block stops seats her.
    fn seat_on(&mut self, stop: Option<Stop>) {
        if stop == Some(Stop::Stepped) {
            self.seat = Seated;
        }
    }

    /// Takes a downward move; returns whether it was stopped.
    fn take_down(&mut self, moved: Moved) -> bool {
        self.position = moved.to;
        self.seat_on(moved.stop);
        moved.stop.is_some()
    }

    /// `FollowingTheGround`: the pull down after her horizontal move, by that frame's speed (her
    /// horizontal speed plus her dash, subpixel part included) plus 1 px (`SlopesCarry`: 1 px on
    /// a slope); the floor stops her flush. Walking off a ledge, unless turning (`TurningRound`:
    /// she sinks instead) or changing shape, she falls from rest.
    fn pull(&mut self, doing: Doing, speed: Fixed, m: &mut Mover) -> Option<Met> {
        let r = m.rules;
        let pull = if self.seat == Seated {
            r.ground_pull
        } else {
            speed.plus(r.ground_pull)
        };
        let moved = self.vmove(Down, pull, m);
        if moved.stop.is_some() || doing == Doing::Turning {
            self.take_down(moved);
            return None;
        }
        if doing == Doing::Shaping {
            return None;
        }
        self.vertical = Falls(Fixed(0));
        self.bounces = Bounces::None;
        self.start_drift();
        Some(NoFloor)
    }

    /// `RisingAndFalling`, `Jumping`'s space jump, `WallsAndCeilings`'s ceiling: her vertical
    /// move in the air, and what it met. A floor met leaves her speed as it was, for the landing
    /// (`meet`) to read.
    fn rise_or_fall(&mut self, doing: Doing, frame: &Frame, m: &mut Mover) -> Option<Met> {
        let r = m.rules;
        // Jumping: with space jump, pressing jump while falling in a spin with her fall speed in
        // the window jumps again, rising that same frame; not in a moonfall (`Moonfall`).
        if let Falls(v) = self.vertical
            && doing == Doing::Spinning
            && frame.pressed.jump
            && frame.equipment.space_jump
            && r.space_jump.contains(&v.as_8_8())
        {
            self.vertical = Rises(r.jump.of(frame.equipment.hi_jump));
        }
        match self.vertical {
            Rises(v) => {
                let moved = self.vmove(Up, v, m);
                self.position = moved.to;
                // WallsAndCeilings: a ceiling ends the rise: falling, with no pull that frame.
                if moved.stop.is_some() {
                    self.vertical = Falls(Fixed(0));
                    return Some(Ceiling);
                }
                // RisingAndFalling: the pull takes 0.1C00 off; past zero the speed is spent, and
                // exactly 0 is not.
                self.vertical = if v < r.gravity {
                    Spent(r.gravity.minus(v))
                } else {
                    Rises(v.minus(r.gravity))
                };
                None
            }
            Falls(v) | Moonfall(v) => {
                let moved = self.vmove(Down, v, m);
                if self.take_down(moved) {
                    return Some(Floor);
                }
                self.vertical = self.pulled(r);
                None
            }
            Ground | Spent(_) => None,
        }
    }

    /// `RisingAndFalling`: her fall's speed after the pull, which stops growing only at pixel
    /// part 5; `Moonfall`: with no limit.
    const fn pulled(&self, r: &Rules) -> Vertical {
        match self.vertical {
            Falls(v) if v.pixel() == r.fall_settle_px => Falls(v),
            Falls(v) => Falls(v.plus(r.gravity)),
            Moonfall(v) => Moonfall(v.plus(r.gravity)),
            v @ (Ground | Rises(_) | Spent(_)) => v,
        }
    }

    /// What her moves met acts on her (`Landing`, `WallJump`'s kick).
    fn meet(&mut self, met: Option<Met>, doing: Doing, frame: &Frame, r: &Rules) {
        match met {
            Some(Floor) => self.land(doing, frame, r),
            // WallJump: the kick starts the wall jump rising at the wall-jump speed, x speed 0,
            // following the pad; it ends a moonfall (`Moonfall`).
            Some(WallKick) => {
                self.vertical = Rises(r.wall_jump.of(frame.equipment.hi_jump));
                self.speed = Fixed(0);
                self.carry = FollowsPad;
            }
            Some(Wall | AimStep | NoFloor | Ceiling) | None => {}
        }
    }

    /// `Landing`: her fall met a floor. Turning, only the move stops: her speed goes on growing
    /// by the pull. While her body changes shape only her vertical motion stops. The ball lands
    /// at 3 px or more bouncing, twice before it rests, the second time 1 px slower, and the
    /// spring ball with jump held hops again. Else she lands: vertical speed 0, bounces 0,
    /// horizontal speed 0, following the pad; the dash keeps its value, to lose (§1.2).
    fn land(&mut self, doing: Doing, frame: &Frame, r: &Rules) {
        let (Falls(v) | Moonfall(v)) = self.vertical else {
            return;
        };
        if doing == Doing::Turning {
            self.vertical = self.pulled(r);
            return;
        }
        self.vertical = Ground;
        if doing == Doing::Shaping {
            return;
        }
        let ball = matches!(doing, Doing::Ball | Doing::Hop);
        let bounce = if ball && self.pose.body == SpringBall && frame.pad.jump {
            Some((r.jump.of(frame.equipment.hi_jump), Bounces::None))
        } else if ball && v.pixel() >= r.bounce_from_px {
            match self.bounces {
                Bounces::None => Some((r.bounce, Bounces::Once)),
                Bounces::Once => Some((r.bounce.minus(r.second_bounce_less), Bounces::Twice)),
                Bounces::Twice => None,
            }
        } else {
            None
        };
        if let Some((rise, bounces)) = bounce {
            self.vertical = Rises(rise);
            self.bounces = bounces;
            return;
        }
        self.bounces = Bounces::None;
        self.speed = Fixed(0);
        self.carry = FollowsPad;
        self.dash_is = DashIs::ToLose;
    }

    /// `Jumping`: starting a straight jump or a fall she carries on along her facing if she has a
    /// dash, else she follows the pad.
    const fn start_drift(&mut self) {
        if self.dash.is_zero() {
            self.carry = FollowsPad;
        } else {
            self.carry = CarriesOn;
            self.way = self.pose.facing.side();
        }
    }

    /// `Jumping`: she starts rising at the jump speed, if she is on the ground.
    fn rise(&mut self, v: Fixed) {
        if self.vertical == Ground {
            self.vertical = Rises(v);
        }
    }

    /// `PoseFitAfterMoves`: when the chosen pose differs from hers, doesn't face the screen and
    /// has a body taller by g px, she looks g px up and down at her old size, from where her
    /// moves left her: solid enemies up and down, then the enemy's far-side look; blocks up and
    /// down, then the block's far-side look. A far-side look stopped refuses the change. With
    /// something on one side only she is pushed away from it by g less the room there (a
    /// block's room before an enemy's on the same side); with something on both, a ball's change
    /// is refused, and any other body takes crouched `Still` instead, rising by the crouch's
    /// growth over her old body, her feet where they were.
    fn fit(&mut self, chosen: Pose, frame: &Frame, m: &mut Mover) -> Change {
        let old = self.radius;
        let fits = chosen != self.pose && chosen.facing != Facing::Screen;
        let Some(g) = (frame.body)(chosen)
            .y
            .checked_sub(old.y)
            .filter(|g| fits && *g > 0)
        else {
            return Change::Chosen(None);
        };
        let new = (frame.body)(chosen);
        let enemies = (
            self.look(m, Up, g, old, Enemies),
            self.look(m, Down, g, old, Enemies),
        );
        if self.far_side(m, enemies, g, new, Enemies) {
            return Change::Refused;
        }
        let blocks = (
            self.look(m, Up, g, old, Blocks),
            self.look(m, Down, g, old, Blocks),
        );
        if self.far_side(m, blocks, g, old, Blocks) {
            return Change::Refused;
        }
        let found = (blocks.0.or(enemies.0), blocks.1.or(enemies.1));
        if let Some((way, room)) = alone(found) {
            // Pushed away by g less the room, in whole px, keeping the subpixel her looks left.
            let px = g.wrapping_sub(room);
            let (y, by) = (self.position.y, Fixed::px(px));
            self.position.y = if way == Up { y.minus(by) } else { y.plus(by) };
            return Change::Chosen(Some(Push { way, px }));
        }
        if found.0.is_none() {
            return Change::Chosen(None);
        }
        if old.y < m.rules.small_body_px {
            return Change::Refused;
        }
        let crouch = (frame.body)(crouched(self.pose));
        let rise = Fixed::px(crouch.y.saturating_sub(old.y));
        self.position.y = self.position.y.minus(rise);
        Change::Crouched
    }

    /// `PoseFitAfterMoves`: one look of `px` at size `r`. It doesn't move her, but what stops it
    /// writes her y subpixel as a stop would, and a stepped block below seats her. Returns the
    /// room it found where something stopped it: how far it got, in whole px.
    fn look(
        &mut self,
        m: &mut Mover,
        heading: Heading,
        px: u16,
        r: Radius,
        probe: Probe,
    ) -> Option<u16> {
        let at = self.position;
        let seen = m.look(at, r, heading, px, probe);
        self.position.y = at.y.with_sub(seen.to.y.sub());
        if heading == Down {
            self.seat_on(seen.stop);
        }
        seen.stop.map(|_| seen.to.y.pixel().abs_diff(at.y.pixel()))
    }

    /// `PoseFitAfterMoves`: with something on one side alone, the far side is looked at once
    /// more for the push's distance. Returns whether that look is stopped.
    fn far_side(
        &mut self,
        m: &mut Mover,
        sides: (Option<u16>, Option<u16>),
        g: u16,
        r: Radius,
        probe: Probe,
    ) -> bool {
        alone(sides)
            .is_some_and(|(way, room)| self.look(m, way, g.wrapping_sub(room), r, probe).is_some())
    }

    /// `ShrinkingLowersHer`: before she is lowered she looks down `px` at her new size, a block
    /// look (solid enemies don't stop it), and she is lowered as far as it got; what stops it
    /// writes to her as a stop would.
    fn lower(&mut self, px: u16, m: &mut Mover) {
        let seen = m.look(self.position, self.radius, Down, px, Blocks);
        self.position = seen.to;
        self.seat_on(seen.stop);
    }

    /// `PoseFitAfterMoves`: she takes the pose the change gives. A change refused, or crouched
    /// `Still` taken instead, leaves her speeds and dash as her moves left them (`moved`), with
    /// her position and seat as her looks left them. Only the chosen pose brings its effects, in
    /// order: after pose fit's push, the new pose's speeds, then the crouch take-off's lift.
    fn take(&mut self, change: Change, chosen: Choice, moved: Self, frame: &Frame, m: &mut Mover) {
        let (start, r) = (self.pose, m.rules);
        if !matches!(change, Change::Chosen(_)) {
            *self = Self {
                position: self.position,
                seat: self.seat,
                ..moved
            };
            if change == Change::Crouched {
                self.pose = crouched(start);
                self.radius = (frame.body)(self.pose);
            }
            return;
        }
        self.pose = chosen.pose;
        self.radius = (frame.body)(chosen.pose);
        let jump = r.jump.of(frame.equipment.hi_jump);
        for effect in chosen.effects {
            match *effect {
                // Jumping: changing into a take-off she starts rising and follows the pad; from
                // a crouch without aim she is lifted.
                TakeOff => {
                    self.rise(jump);
                    self.carry = FollowsPad;
                    if start.body == Crouched && start.aim == Aim::Forward {
                        self.position.y = self.position.y.minus(r.crouch_lift);
                    }
                }
                Rise => self.rise(jump),
                TurnRound => self.turn_round(start),
                EndSkid if self.carry == CarriesOn => {
                    self.carry = FollowsPad;
                    self.way = self.pose.facing.side();
                }
                // PadChart: with a horizontal speed of 0 she only follows the pad again.
                AsRunning if self.speed.is_zero() => self.carry = FollowsPad,
                AsRunning => {
                    self.speed = self.speed.plus(self.dash);
                    self.dash = Fixed(0);
                    self.carry = CarriesOn;
                }
                LoseDash => self.dash = Fixed(0),
                Halt => self.halt(),
                // Moonfall: a spin she enters when a turn ends, with jump not just pressed,
                // starts with no rise: her vertical speed is 0 and she drops under gravity.
                Effect::Moonfall if frame.pressed.jump => self.rise(jump),
                Effect::Moonfall => self.vertical = Moonfall(Fixed(0)),
                Lower(px) => self.lower(px, m),
                StartDrift => self.start_drift(),
                CarrySpin => self.spin = CarriedOn,
                EndSkid => {}
            }
        }
    }

    /// `TurningRound`: turning round, reversing a spin, or the ball turning round, she adds her
    /// dash to her speed (dash 0) and travels the old way, against her new facing: a skid. A
    /// spin reversed while skidding turns her travel round, back along her old facing.
    const fn turn_round(&mut self, start: Pose) {
        self.speed = self.speed.plus(self.dash);
        self.dash = Fixed(0);
        self.way = start.facing.side();
        self.carry = CarriesOn;
    }
}
