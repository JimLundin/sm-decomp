//! The samus-motion model, built from its behaviour spec alone.
//!
//! Every rule names the spec entry it comes from. A frame (`PoseFitAfterMoves`) moves her
//! horizontally, then by the ground pull or her vertical move, then by the aim change's step;
//! then comes what those moves met, the pose chart (§7) and the pose change's effects.

pub mod catalogue;
mod chart;
mod collide;
pub mod rules;

pub use catalogue::{Behaviour, BehaviourId, CATALOGUE};
pub use chart::{Choice, Met, Situation, next_pose, pad_chart};
pub use rules::{Limits, Rules, SpeedCap};

use Vertical::{Falls, Ground, Rises, Spent};
use chart::changes_shape;
use chart::table::{ASRUN, ENDSKID, HALT, LOSEDASH, MOONFALL, RISE, TAKEOFF, TURN};
use clean_types::Action::{
    AgainstWall, Falling, Jumping, Landing, Moonwalking, Rolling, Running, Spinning, Still,
    TakingOff, Turning, WallJumping,
};
use clean_types::Body::{Ball, Crouched, SpringBall, Upright};
use clean_types::Heading::{self, Down, Left, Right, Up};
use clean_types::{
    Aim, Cell, Equipment, Facing, Fixed, Pad, Parity, Pose, Position, Radius, Signed, Touch, World,
    below,
};
use collide::{Mover, Stop};

/// Her vertical motion.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Vertical {
    /// Falling at this speed; 0 is a fall just started.
    Falls(Fixed),
    /// On the ground.
    Ground,
    /// Rising at this upward speed; at 0 she rises by 0 for one more frame.
    Rises(Fixed),
    /// `RisingAndFalling`: the pull took the upward speed past zero, so it points downward by
    /// this; the rise ends at the start of the next frame.
    Spent(Fixed),
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

/// What one frame brings from the player and the world (§1.3), and from what is out of scope
/// and feeds us (the header): her body's size from pose fit, an animation's end from its timing.
#[derive(Clone, Copy)]
pub struct Frame {
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
    /// Her body's half-width and half-height this frame.
    pub radius: Radius,
    /// `PoseFitAfterMoves`: pose fit's push this frame (out of scope, an input; negative is
    /// upward): when the new pose's body is taller, it pushes her clear of what is above or
    /// below her. On a floor with nothing above her it keeps her feet where they were, as when
    /// she stands at once from a crouch (`PadChart`).
    pub push: Signed,
}

/// What a frame shows beyond her quantities.
pub struct Outcome {
    /// `AimingAndShooting`: the standing animation restarts.
    pub standing_restarts: bool,
    /// `WhatHerMovesMet`: what her moves met.
    pub met: Option<Met>,
    /// `TouchedCells`: the cells her moves reached, in the order they happen.
    pub reached: Vec<Touch>,
    /// `TouchedCells`: the cells she was inside at the frame's start.
    pub inside: Vec<Cell>,
}

/// Samus as the player can tell her, and what she remembers.
#[expect(clippy::struct_excessive_bools)]
pub struct Samus {
    /// Her body's centre.
    pub position: Position,
    /// Her horizontal speed's size.
    pub speed: Fixed,
    /// The way her horizontal speed points, left or right.
    pub way: Heading,
    /// §1.2 carrying on: her horizontal motion carries on by itself (it brakes on the ground and
    /// keeps building in the air; against her facing, a skid), rather than following the pad.
    pub carry: bool,
    /// Her dash, added the way she moves.
    pub dash: Fixed,
    /// §1.2 her dash: left on her by a halt, it is gone the next frame she moves.
    pub losing_dash: bool,
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
    /// `Moonfall`: in one, until she lands.
    pub moonfall: bool,
    /// `SlopesCarry`: seated on a slope, until her next vertical move of more than 0 starts.
    pub seated: bool,
}

/// How a pose builds her horizontal speed.
#[derive(Clone, Copy)]
enum Style {
    /// `SpeedBuildsAndBrakes`: following the pad she builds; carrying on she brakes.
    Ground,
    /// `DriftInTheAir`: a straight jump, a fall, the ball in the air.
    Air,
    /// `DriftInTheAir`: a spin jump.
    Spin,
    /// `DriftInTheAir`: a wall jump.
    WallJump,
}

/// `Jumping`: a jump speed, without or with hi-jump.
const fn hi(speeds: (Fixed, Fixed), hi_jump: bool) -> Fixed {
    if hi_jump { speeds.1 } else { speeds.0 }
}

impl Samus {
    /// Samus at a vector's start (§1.2): her motion follows the pad unless she is skidding, her
    /// dash is hers, and a vertical speed of 0 means she stands on the ground, except in a jump, a
    /// spin, a fall or a wall jump, where it means she has just started to fall.
    #[must_use]
    pub const fn start(
        position: Position,
        speed_x: Signed,
        dash: Fixed,
        speed_y: Signed,
        bounces: Bounces,
        pose: Pose,
        radius: Radius,
    ) -> Self {
        let facing = pose.facing.side();
        let way = if speed_x.size.is_zero() {
            facing
        } else if speed_x.negative {
            Left
        } else {
            Right
        };
        let v = speed_y.size;
        let vertical = if !v.is_zero() {
            if speed_y.negative { Rises(v) } else { Falls(v) }
        } else if matches!(pose.action, Jumping | Spinning | Falling | WallJumping) {
            Falls(v)
        } else {
            Ground
        };
        Self {
            position,
            speed: speed_x.size,
            way,
            carry: !matches!((way, facing), (Left, Left) | (Right, Right)),
            dash,
            losing_dash: false,
            vertical,
            bounces,
            pose,
            radius,
            spin: Spin::Off,
            moonfall: false,
            seated: false,
        }
    }

    /// Her horizontal speed as the player reads it, rightward positive.
    #[must_use]
    pub const fn speed_x_signed(&self) -> Signed {
        Signed {
            negative: matches!(self.way, Left),
            size: self.speed,
        }
    }

    /// Her vertical speed as the player reads it, downward positive.
    #[must_use]
    pub const fn speed_y_signed(&self) -> Signed {
        let (negative, size) = match self.vertical {
            Ground => (false, Fixed(0)),
            Rises(v) => (true, v),
            Falls(v) | Spent(v) => (false, v),
        };
        Signed { negative, size }
    }

    /// One frame (`PoseFitAfterMoves`): her moves as the pose at its start says
    /// (`MovesAsShePosed`), what they met, then the pose chosen (`PoseFollows`) and its effects.
    pub fn step(&mut self, frame: &Frame, world: &dyn World, rules: &Rules) -> Outcome {
        let mut m = Mover {
            world,
            rules,
            parity: frame.parity,
            reached: Vec::new(),
        };
        self.radius = frame.radius;
        // TouchedCells: the cells she is inside, before her moves.
        let inside = m.inside(self.position, self.radius);
        let (pose, pad) = (self.pose, frame.pad);
        self.spin = if pose.action != Spinning {
            Spin::Off
        } else if self.spin == Spin::Off {
            Spin::First
        } else if self.spin == Spin::First {
            Spin::Second
        } else {
            Spin::CarriedOn
        };
        // RisingAndFalling: the rise ends at the start of the frame after it is spent, and in a
        // jump, a spin or the hop at the start of any frame with jump not held (not in a
        // moonfall, which has no rise).
        let ended = matches!(self.vertical, Spent(_))
            || matches!(self.vertical, Rises(_))
                && matches!(pose.action, Jumping | Spinning)
                && !pad.jump
                && !self.moonfall;
        if ended {
            self.vertical = Falls(Fixed(0));
        }
        let (mut met, mut walled) = (None, false);
        if pose.facing == Facing::Screen {
            // SpeedBuildsAndBrakes: facing the screen she doesn't move; RidingTheElevator: on a
            // moving elevator she moves down 1 px, as any downward move.
            if frame.elevator_moving {
                self.position = m.go(self.position, self.radius, Down, rules.elevator).0;
            }
        } else {
            let speed;
            (speed, walled) = self.walk(pose, pad, &mut m);
            met = if self.wall_kick(pose, frame, &mut m) {
                Some(Met::WallKick)
            } else if self.vertical == Ground {
                self.pull(pose, speed, &mut m)
            } else if pose.action == TakingOff {
                // Jumping: taking off she doesn't move vertically and the pull doesn't act.
                None
            } else {
                self.rise_or_fall(pose, frame, ended, &mut m)
            };
        }
        let s = Situation {
            pose,
            airborne: self.vertical != Ground,
            pad,
            pressed: frame.pressed,
            equipment: frame.equipment,
        };
        let chart = pad_chart(&s);
        // AimingAndShooting: changing aim while running steps her forward after the frame's
        // moves; a block or solid enemy in the way puts her against the wall.
        if pose.action == Running
            && chart.0.action == Running
            && chart.0.aim != pose.aim
            && self.aim_step(pose, &mut m)
        {
            met = Some(Met::AimStep);
        }
        // WhatHerMovesMet: a wall stop while running wins over the rest.
        if walled && pose.action == Running {
            met = Some(Met::Wall);
        }
        // AimingAndShooting: standing, unaimed and not landing, holding shoot restarts the
        // standing animation.
        let standing_restarts =
            pose.body == Upright && pose.action == Still && pose.aim == Aim::Forward && pad.shoot;
        let (next, fx) = next_pose(&s, chart, met, frame.animation_ends);
        self.take(pose, next, fx, frame, rules);
        Outcome {
            standing_restarts,
            met,
            reached: m.reached,
            inside,
        }
    }

    /// `SpeedBuildsAndBrakes`, `Dash`, `DriftInTheAir`, `WallsAndCeilings`: her move by her speed
    /// and dash. Returns the speed from before a stop (the ground pull's) and whether a wall
    /// stopped her.
    fn walk(&mut self, pose: Pose, pad: Pad, m: &mut Mover) -> (Fixed, bool) {
        let r = m.rules;
        let ball = matches!(pose.body, Ball | SpringBall);
        let build = match pose.action {
            Running => Some((r.running, Style::Ground)),
            Rolling | Still if ball && self.vertical != Ground => Some((r.falling, Style::Air)),
            Rolling => Some((r.rolling, Style::Ground)),
            Moonwalking => Some((r.moonwalking, Style::Ground)),
            Turning(_) => Some((r.turning, Style::Ground)),
            Jumping => Some((r.straight_jump, Style::Air)),
            Falling => Some((r.falling, Style::Air)),
            Spinning => Some((r.spin_jump, Style::Spin)),
            WallJumping => Some((r.spin_jump, Style::WallJump)),
            Still
            | AgainstWall
            | Landing(_)
            | TakingOff
            | clean_types::Action::Crouching
            | clean_types::Action::StandingUp
            | clean_types::Action::Morphing
            | clean_types::Action::Unmorphing => None,
        };
        // What she moves by (her speed and dash, her dash alone, or nothing), and whether she
        // halts after the move.
        let (moves, halts) = match build {
            Some((limits, style)) => {
                // Dash: running with run held she gains dash; in any other moving activity a
                // dash she is to lose is gone.
                if pose.action == Running && pad.run {
                    self.grow_dash(r);
                } else if self.losing_dash {
                    self.dash = Fixed(0);
                }
                self.build_or_brake(limits, style, pose, pad, r)
            }
            // SpeedBuildsAndBrakes: the ball at rest braking or skidding rolls like running, and
            // keeps its dash.
            None if pose.action == Still && ball && self.carry => {
                self.brake(r);
                (Some(true), false)
            }
            // SpeedBuildsAndBrakes: standing, crouched, against the wall and the ball at rest
            // move by their dash only, then halt.
            None if matches!(pose.action, Still | AgainstWall | Landing(_)) => (Some(false), true),
            // Jumping: taking off she moves by her dash only, follows the pad and still gains
            // dash; SpeedBuildsAndBrakes: while her body changes shape she keeps both speeds.
            None => {
                if pose.action == TakingOff {
                    self.carry = false;
                    if pad.run {
                        self.grow_dash(r);
                    }
                }
                (Some(false), false)
            }
        };
        let speed = self.speed;
        // §1: the dash taken the way the speed points, her facing when it is 0.
        let way = if speed.is_zero() {
            pose.facing.side()
        } else {
            self.way
        };
        let d = moves.map(|both| {
            if both {
                speed.plus(self.dash)
            } else {
                self.dash
            }
        });
        let walled = d.is_some_and(|d| self.walk_and_seat(way, d, m));
        // WallsAndCeilings, SpeedBuildsAndBrakes: a halt.
        if walled || halts {
            self.halt();
        }
        (speed, walled)
    }

    /// `SpeedBuildsAndBrakes`, `DriftInTheAir`: builds or brakes a building pose's speed; returns
    /// what she moves by (`Some(true)` her speed and dash, `Some(false)` her dash alone) and
    /// whether she halts after.
    fn build_or_brake(
        &mut self,
        limits: Limits,
        style: Style,
        pose: Pose,
        pad: Pad,
        r: &Rules,
    ) -> (Option<bool>, bool) {
        let facing = pose.facing.side();
        let carry = self.carry;
        match style {
            // SpeedBuildsAndBrakes: carrying on along the ground she brakes.
            Style::Ground if carry => self.brake(r),
            // DriftInTheAir: a skid in the air brakes, as on the ground; a spin's skid moves.
            Style::Air | Style::Spin if carry && self.way != facing => self.brake(r),
            // DriftInTheAir: following the pad with neither side held she stops drifting; the
            // falling ball instead halts, but still moves by its dash.
            Style::Air if !carry && !pad.left && !pad.right => {
                if matches!(pose.body, Ball | SpringBall) {
                    return (Some(false), true);
                }
                self.speed = Fixed(0);
                return (None, false);
            }
            // SpeedBuildsAndBrakes: following the pad she builds; DriftInTheAir: carrying on in
            // the air doesn't brake.
            Style::Ground | Style::Air => {
                self.build(limits, facing, r);
            }
            // DriftInTheAir: a spin drifts only while she holds the side she faces or reached
            // her top speed this frame; else she stops drifting and moves by her dash alone.
            Style::Spin => {
                let topped = self.build(limits, facing, r);
                if !(topped || if facing == Left { pad.left } else { pad.right }) {
                    self.speed = Fixed(0);
                    self.carry = false;
                    return (Some(false), false);
                }
                self.carry = true;
            }
            // DriftInTheAir: through a wall jump she always drifts, carrying on along her facing.
            Style::WallJump => {
                self.build(limits, facing, r);
                self.carry = true;
            }
        }
        (Some(true), false)
    }

    /// `SpeedBuildsAndBrakes`: gains the step up to the top (`WordwiseTopSpeed`), along her
    /// facing. Returns whether she took the top speed.
    fn build(&mut self, (step, top): Limits, facing: Heading, r: &Rules) -> bool {
        let topped;
        (self.speed, topped) = r.cap(self.speed.plus(step), top);
        self.way = facing;
        topped
    }

    /// `SpeedBuildsAndBrakes`: carrying on she loses the brake; below zero she stops and follows
    /// the pad again.
    fn brake(&mut self, r: &Rules) {
        if self.speed < r.brake {
            self.speed = Fixed(0);
            self.carry = false;
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
        self.losing_dash = false;
    }

    /// A halt: horizontal speed 0, dash 0, following the pad.
    const fn halt(&mut self) {
        self.speed = Fixed(0);
        self.dash = Fixed(0);
        self.carry = false;
    }

    /// `SlopesSlowTheWalk`: a horizontal move made while her vertical speed is exactly 0 is
    /// shortened by the sloped floors at her leading edge.
    fn shorten(&self, way: Heading, d: Fixed, m: &Mover) -> Fixed {
        if self.speed_y_signed().size.is_zero() {
            m.walk_shortened(self.position, self.radius, way, d)
        } else {
            d
        }
    }

    /// A horizontal move, then `SlopesCarry`'s checks, which run after every horizontal move,
    /// whether or not it met solid enemies, and whose lift or push seats her. Returns whether
    /// the move was stopped.
    fn walk_and_seat(&mut self, way: Heading, d: Fixed, m: &mut Mover) -> bool {
        let d = self.shorten(way, d, m);
        let (to, stop) = m.go(self.position, self.radius, way, d);
        self.position = m.seat(to, self.radius).map_or(to, |at| {
            self.seated = true;
            at
        });
        stop.is_some()
    }

    /// `AimingAndShooting`: the step forward, a horizontal move with no `SlopesCarry` checks
    /// after it; a solid enemy within its reach stops it with no move at all, one she touches
    /// still clearing her y subpixel. Returns whether something was in the way.
    fn aim_step(&mut self, pose: Pose, m: &mut Mover) -> bool {
        let side = pose.facing.side();
        let d = self.shorten(side, m.rules.aim_step, m);
        let (to, stop) = m.go(self.position, self.radius, side, d);
        self.position = if stop == Some(Stop::Enemy) {
            Position {
                y: to.y,
                ..self.position
            }
        } else {
            to
        };
        stop.is_some()
    }

    /// `WallJump`: in a spin carried on, holding away from a wall near her (left held: a wall on
    /// her right; else right held: on her left), the test for it, which can write her subpixels;
    /// with the spin turned far, pressing jump kicks her off: she doesn't move vertically that
    /// frame, and starts rising at the wall-jump speed, x speed 0, following the pad. The kick
    /// ends a moonfall (`Moonfall`). Returns whether she kicked.
    fn wall_kick(&mut self, pose: Pose, frame: &Frame, m: &mut Mover) -> bool {
        let pad = frame.pad;
        if pose.action != Spinning || self.spin != Spin::CarriedOn || !(pad.left || pad.right) {
            return false;
        }
        let wall = if pad.left { Right } else { Left };
        let (to, near) = m.wall_test(self.position, self.radius, wall, m.rules.wall_reach);
        self.position = to;
        if !(near && frame.spin_turned_far && frame.pressed.jump) {
            return false;
        }
        self.vertical = Rises(hi(m.rules.wall_jump, frame.equipment.hi_jump));
        self.speed = Fixed(0);
        self.carry = false;
        self.moonfall = false;
        true
    }

    /// Where a vertical move would leave her. `SlopesCarry`: as a move of more than 0 starts she
    /// leaves any slope.
    fn vmove(&mut self, h: Heading, d: Fixed, m: &mut Mover) -> (Position, Option<Stop>) {
        if !d.is_zero() {
            self.seated = false;
        }
        m.go(self.position, self.radius, h, d)
    }

    /// Takes a downward move; returns whether it was stopped. `SlopesCarry`: a stop on a stepped
    /// block seats her.
    fn take_down(&mut self, (to, stop): (Position, Option<Stop>)) -> bool {
        self.position = to;
        if stop == Some(Stop::Stepped) {
            self.seated = true;
        }
        stop.is_some()
    }

    /// `FollowingTheGround`: the pull down after her horizontal move, by that frame's whole speed
    /// plus 1 px (`SlopesCarry`: 1 px on a slope); the floor stops her flush. Walking off a
    /// ledge, unless turning (`TurningRound`: she sinks instead) or changing shape, she falls
    /// from rest.
    fn pull(&mut self, pose: Pose, speed: Fixed, m: &mut Mover) -> Option<Met> {
        let r = m.rules;
        let pull = if self.seated {
            r.ground_pull
        } else {
            Fixed::px(speed.pixel()).plus(r.ground_pull)
        };
        let moved = self.vmove(Down, pull, m);
        if moved.1.is_some() || matches!(pose.action, Turning(_)) {
            self.take_down(moved);
            None
        } else if changes_shape(pose.action) {
            None
        } else {
            self.vertical = Falls(Fixed(0));
            self.bounces = Bounces::None;
            self.start_drift(pose.facing.side());
            Some(Met::NoFloor)
        }
    }

    /// `RisingAndFalling`, `Jumping`'s space jump, `Landing`, `WallsAndCeilings`'s ceiling: her
    /// vertical move in the air, and what it met.
    fn rise_or_fall(
        &mut self,
        pose: Pose,
        frame: &Frame,
        ended: bool,
        m: &mut Mover,
    ) -> Option<Met> {
        let r = m.rules;
        // RisingAndFalling: no vertical move the frame a rise ends; the pull acts as in a fall.
        if ended {
            self.vertical = self.fall_pull(Fixed(0), r);
            return None;
        }
        let jump = hi(r.jump, frame.equipment.hi_jump);
        let (from, to) = r.space_jump_window;
        // Jumping: with space jump, pressing jump while falling in a spin with her fall speed in
        // the window jumps again, rising that same frame; not in a moonfall (`Moonfall`).
        if let Falls(v) = self.vertical
            && pose.action == Spinning
            && frame.pressed.jump
            && frame.equipment.space_jump
            && !self.moonfall
            && (from..to).contains(&v.as_8_8())
        {
            self.vertical = Rises(jump);
        }
        match self.vertical {
            Rises(v) => {
                let (at, stop) = self.vmove(Up, v, m);
                self.position = at;
                // WallsAndCeilings: a ceiling ends the rise: falling, with no pull that frame.
                if stop.is_some() {
                    self.vertical = Falls(Fixed(0));
                    return Some(Met::Ceiling);
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
            Falls(v) => {
                let moved = self.vmove(Down, v, m);
                if self.take_down(moved) {
                    self.land(pose, v, frame.pad.jump.then_some(jump), r);
                    return Some(Met::Floor);
                }
                self.vertical = self.fall_pull(v, r);
                None
            }
            Ground | Spent(_) => None,
        }
    }

    /// `RisingAndFalling`: a fall's speed after the pull, which stops growing only at pixel part
    /// 5; `Moonfall`: with no limit.
    const fn fall_pull(&self, v: Fixed, r: &Rules) -> Vertical {
        Falls(if v.pixel() == r.fall_settle_px && !self.moonfall {
            v
        } else {
            v.plus(r.gravity)
        })
    }

    /// `Landing`: falling onto a floor at `v`; `hop` is the jump speed when jump is held.
    fn land(&mut self, pose: Pose, v: Fixed, hop: Option<Fixed>, r: &Rules) {
        // Moonfall: until she lands.
        self.moonfall = false;
        self.vertical = Ground;
        // Landing: while her body changes shape it stops only her vertical motion.
        if changes_shape(pose.action) {
            return;
        }
        // Landing: the spring ball landing with jump held hops again, bounces 0; the ball
        // landing at 3 px or more bounces twice before it rests, the second time 1 px slower.
        let bounce = match (hop, self.bounces) {
            (Some(jump), _) if pose.body == SpringBall => Some((jump, Bounces::None)),
            _ if pose.body != Ball && pose.body != SpringBall || v.pixel() < r.bounce_from_px => {
                None
            }
            (_, Bounces::None) => Some((r.bounce, Bounces::Once)),
            (_, Bounces::Once) => Some((r.bounce.minus(r.second_bounce_less), Bounces::Twice)),
            (_, Bounces::Twice) => None,
        };
        if let Some((rise, bounces)) = bounce {
            self.vertical = Rises(rise);
            self.bounces = bounces;
            return;
        }
        // Landing: bounces 0, horizontal speed 0, following the pad; the dash keeps its value,
        // to lose (§1.2).
        self.bounces = Bounces::None;
        self.speed = Fixed(0);
        self.carry = false;
        self.losing_dash = true;
    }

    /// `Jumping`: starting a straight jump or a fall she carries on along her facing if she has a
    /// dash, else she follows the pad.
    const fn start_drift(&mut self, facing: Heading) {
        if self.dash.is_zero() {
            self.carry = false;
        } else {
            self.carry = true;
            self.way = facing;
        }
    }

    /// `Jumping`: she starts rising at the jump speed, if she is on the ground.
    const fn rise(&mut self, v: Fixed) {
        if matches!(self.vertical, Ground) {
            self.vertical = Rises(v);
        }
    }

    /// `PoseFollows`: she takes the chosen pose, and its effects act on her speeds and dash after
    /// the frame's moves (`PoseFitAfterMoves`: the new pose's speeds, then the crouch lift).
    fn take(&mut self, prev: Pose, next: Pose, fx: u16, frame: &Frame, r: &Rules) {
        let on = |f: u16| fx & f != 0;
        let jump = hi(r.jump, frame.equipment.hi_jump);
        let side = next.facing.side();
        // PoseFitAfterMoves: pose fit's push comes first, from where her moves left her.
        let (y, push) = (self.position.y, frame.push);
        self.position.y = if push.negative {
            y.minus(push.size)
        } else {
            y.plus(push.size)
        };
        // Jumping: changing into a take-off, a spin start or the hop she starts rising.
        if on(TAKEOFF | RISE) {
            self.rise(jump);
        }
        // Jumping: the take-off follows the pad, and from a crouch without aim lifts her 10 px.
        if on(TAKEOFF) {
            self.carry = false;
            if prev.body == Crouched && prev.aim == Aim::Forward {
                self.position.y = self.position.y.minus(r.crouch_lift);
            }
        }
        if on(TURN) {
            self.turn_round(prev, next);
        }
        // TurningRound, PadChart: she follows the pad again; a skid goes on along her new facing.
        if on(ENDSKID) && self.carry {
            self.carry = false;
            self.way = side;
        }
        // PadChart's "as running": the dash joins her speed and she carries on, braking; with a
        // speed of 0 she only follows the pad again.
        if on(ASRUN) {
            if self.speed.is_zero() {
                self.carry = false;
            } else {
                self.speed = self.speed.plus(self.dash);
                self.dash = Fixed(0);
                self.carry = true;
            }
        }
        // Dash: a turn's end takes it.
        if on(LOSEDASH) {
            self.dash = Fixed(0);
        }
        if on(HALT) {
            self.halt();
        }
        // Moonfall: a spin she enters when a turn ends, with jump not just pressed, starts with no
        // rise: she drops under gravity.
        if on(MOONFALL) {
            if frame.pressed.jump {
                self.rise(jump);
            } else {
                self.moonfall = true;
                if self.vertical == Ground {
                    self.vertical = Falls(Fixed(0));
                }
            }
        }
        if prev.action != next.action {
            // Jumping: starting a straight jump or a fall she carries on along her facing if she
            // has a dash (not the hop).
            if matches!(next.action, Jumping | Falling) && !on(RISE) {
                self.start_drift(side);
            }
            // §1.2: a spin that followed a wall jump is carried on.
            if next.action == Spinning && prev.action == WallJumping {
                self.spin = Spin::CarriedOn;
            }
        }
        self.pose = next;
    }

    /// `TurningRound`: turning round, reversing a spin carried on, or the ball turning round, she
    /// adds her dash to her speed (dash 0) and keeps travelling the old way: a skid.
    fn turn_round(&mut self, prev: Pose, next: Pose) {
        let reversed = prev.action == Spinning && next.action == Spinning;
        if prev.facing.side() == next.facing.side() || reversed && self.spin != Spin::CarriedOn {
            return;
        }
        if self.speed.is_zero() {
            self.way = prev.facing.side();
        }
        self.speed = self.speed.plus(self.dash);
        self.dash = Fixed(0);
        self.carry = true;
    }
}
