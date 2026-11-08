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
mod fit;
mod motion;
pub mod rows;
pub mod rules;
pub mod situation;
mod tick;

pub use catalogue::{Behaviour, BehaviourId, CATALOGUE};
pub use chart::{Choice, Moment};
pub use motion::{Carry, DashIs, Run};
pub use rows::{Effect, Met};
pub use rules::{Rules, SpeedCap};
pub use situation::classify;
pub use tick::Frame;

use Carry::{CarriesOn, FollowsPad};
use Drawn::{OtherSuit, Plain, ScrewAttack, SpaceJump};
use Effect::{AsRunning, EndSkid, Halt, Lift, LoseDash, Rise, TurnRound};
use Met::{Ceiling, Floor, NoFloor, Wall, WallKick};
use Probe::{Look, Move};
use Seat::{Seated, Unseated};
use Spin::{CarriedOn, First, Off, Second};
use Standing::{Plays, Restarts};
use Stride::{Both, DashThenHalt, Nothing};
use Vertical::{Falling, Ground, Moonfall, Rising};
use WallTest::{Kicked, Neither, Readied};
use clean_types::Action::{AgainstWall, Crouching, Jumping, Morphing, Still, WallJumping};
use clean_types::Body::{SpringBall, Upright};
use clean_types::Heading::{self, Down, Left, Right, Up};
use clean_types::button::{JUMP, LEFT, SHOOT, SIDES};
use clean_types::{
    Action, Aim, Buttons, Cell, Equipment, Facing, Fixed, Gun, Parts, Pose, Position, Radius,
    Touch, World, ZERO, fixed,
};
use collide::{Bounds, Moved, Probe, Stop};
use fit::{Change, fit};
use motion::Stride;
use rows::Doing::{self, Ball, Hop, Running, Shaping, Spinning, StraightJump, TakingOff, Turning};
use rows::{Moves, Situation};
use tick::Tick;

/// Her vertical motion's phase; her vertical speed itself is `Samus::vy`.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Vertical {
    /// On the ground, keeping the vertical speed she brought with her (`FollowingTheGround`).
    Ground,
    /// Rising (`RisingAndFalling`); at the start of a frame after the pull took her upward
    /// speed past zero, the rise ends.
    Rising,
    /// Falling.
    Falling,
    /// `Moonfall`: falling with no falling limit and no jump, until she lands.
    Moonfall,
}

/// `Landing`: how many times the ball has bounced since it last rested.
#[derive(Clone, Copy, PartialEq, Eq)]
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

/// `SlopesCarry`: whether she is seated on a slope, until her next vertical move of more than 0
/// starts.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Seat {
    /// Not seated.
    Unseated,
    /// Seated on a slope: her ground pull is 1 px.
    Seated,
}

/// `WallJump`'s output: the near-wall test's outcome this frame.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum WallTest {
    /// No test, no wall found, or a wall near her once the spin has turned far, without jump
    /// pressed.
    #[default]
    Neither,
    /// A wall near her while the spin hasn't turned far: the spin's animation skips ahead.
    Readied,
    /// She kicked off the wall.
    Kicked,
}

/// `AimingAndShooting`: the standing animation.
pub enum Standing {
    /// It plays on.
    Plays,
    /// It restarts.
    Restarts,
}

/// `HowSheIsDrawn`: what her sprite shows beyond her pose's parts.
pub enum Drawn {
    /// Plain, in her first suit.
    Plain,
    /// The spin drawn as the screw attack.
    ScrewAttack,
    /// The spin drawn as the space jump.
    SpaceJump,
    /// Facing the screen in the other suit's sprite.
    OtherSuit,
}

/// What a frame shows beyond her quantities: its outputs, for the caller to act on.
pub struct Outcome {
    /// `WallJump`: the near-wall test's outcome.
    pub wall_test: WallTest,
    /// `AimingAndShooting`: the standing animation.
    pub standing: Standing,
    /// `HowSheIsDrawn`: what her sprite shows beyond her pose.
    pub drawn: Drawn,
    /// `TouchedCells`: the cells her moves and looks reached, in the order they happen.
    pub reached: Vec<Touch>,
    /// `TouchedCells`: the cells she was inside at the frame's start.
    pub inside: Vec<Cell>,
    /// §1, `PoseFitAfterMoves`, `ShrinkingLowersHer`: where the camera follows her from, a y.
    pub camera_from: Fixed,
}

/// `WhatHerMovesMet`: what a frame's moves met, and the near-wall test's outcome (`WallJump`).
#[derive(Default)]
struct Meeting {
    /// What her moves met.
    met: Option<Met>,
    /// The near-wall test's outcome.
    wall_test: WallTest,
}

/// `FollowingTheGround`, `WallsAndCeilings`: what a walk did: the speed the ground pull reads,
/// and what stopped her.
struct Walk {
    /// Her horizontal speed and dash, from before any stop.
    pull: Fixed,
    /// What stopped the move.
    stop: Option<Stop>,
}

/// Samus as the player can tell her, and what she remembers.
#[derive(Clone, Copy)]
pub struct Samus {
    /// Her body's centre.
    pub position: Position,
    /// Her horizontal motion: speed, the way it points, carrying on, her dash.
    pub run: Run,
    /// Her vertical speed, signed, downward positive.
    pub vy: Fixed,
    /// Her vertical motion's phase.
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

/// `HowSheIsDrawn`: the spin (and a wall jump's) drawn as the screw attack with screw attack,
/// else as the space jump with space jump; facing the screen, the other suit's sprite with it.
const fn drawn(pose: Pose, e: Equipment) -> Drawn {
    let spin = matches!(pose.action, Action::Spinning | WallJumping);
    if spin && e.screw_attack {
        ScrewAttack
    } else if spin && e.space_jump {
        SpaceJump
    } else if matches!(pose.facing, Facing::Screen) && e.other_suit {
        OtherSuit
    } else {
        Plain
    }
}

/// `AimingAndShooting`: standing, unaimed and not landing, holding shoot restarts the standing
/// animation.
const fn standing(p: Pose, pad: Buttons) -> Standing {
    let unaimed = matches!((p.body, p.action, p.aim), (Upright, Still, Aim::Forward));
    if unaimed && pad.any(SHOOT) {
        Restarts
    } else {
        Plays
    }
}

impl Samus {
    /// One frame (`PoseFitAfterMoves`): her moves as the pose at its start says
    /// (`MovesAsShePosed`) and what they met; the pose chosen (`PoseFollows`); pose fit or the
    /// shrink; then the change's effects.
    pub fn step(&mut self, frame: &Frame, world: &dyn World, rules: &Rules) -> Outcome {
        let mut t = Tick {
            frame,
            world,
            rules,
            reached: Vec::new(),
        };
        // TouchedCells: the cells she is inside, before her moves.
        let inside = t.inside(self.bounds());
        let start = *self;
        let situation = classify(start.pose, start.vertical == Ground);
        self.begin(situation.doing, frame.pad);
        let meeting = self.moves(situation, &mut t);
        // PoseFitAfterMoves: the speeds her moves left her, for a change that doesn't come.
        let moved = *self;
        self.meet(meeting.met, situation.doing, &t);
        let mut chosen = Moment {
            situation,
            pose: start.pose,
            moving: start.run.speed != ZERO,
            airborne: self.vertical != Ground,
            pad: frame.pad,
            pressed: frame.pressed,
            equipment: frame.equipment,
            met: meeting.met,
        }
        .choose(frame.animation_ends);
        if chosen.steps {
            self.aim_step(&mut chosen, &mut t);
        }
        let camera_from = self.take(chosen, &start, moved, &mut t);
        Outcome {
            wall_test: meeting.wall_test,
            standing: standing(start.pose, frame.pad),
            drawn: drawn(self.pose, frame.equipment),
            reached: t.reached,
            inside,
            camera_from,
        }
    }

    /// §8, `BlocksStopFlush`: her box, the one her moves test.
    const fn bounds(&self) -> Bounds {
        Bounds {
            at: self.position,
            r: self.radius,
        }
    }

    /// `RisingAndFalling`, `Jumping`, `WallJump`: she starts rising at this speed.
    fn rise(&mut self, v: Fixed) {
        self.vertical = Rising;
        self.vy = -v;
    }

    /// `RisingAndFalling`, `FollowingTheGround`, `WallsAndCeilings`: she falls from rest.
    const fn fall_from_rest(&mut self) {
        self.vertical = Falling;
        self.vy = ZERO;
    }

    /// The frame's start: `WallJump` (§1.2): how long she has been in this spin;
    /// `RisingAndFalling`: the rise ends at the start of the frame after it is spent, and in a
    /// straight jump, a spin or the hop at the start of any frame with jump not held.
    fn begin(&mut self, doing: Doing, pad: Buttons) {
        let next = match self.spin {
            Off => First,
            First => Second,
            Second | CarriedOn => CarriedOn,
        };
        self.spin = if doing == Spinning { next } else { Off };
        let lets_go = !pad.any(JUMP) && matches!(doing, StraightJump | Spinning | Hop);
        let spent = !self.vy.negative() && self.vy != ZERO;
        if self.vertical == Rising && (lets_go || spent) {
            self.fall_from_rest();
        }
    }

    /// `MovesAsShePosed`: her moves, as the pose at the frame's start says, and what they met.
    fn moves(&mut self, s: &Situation, t: &mut Tick) -> Meeting {
        if self.pose.facing == Facing::Screen {
            // SpeedBuildsAndBrakes: facing the screen she doesn't move; RidingTheElevator: on a
            // moving elevator she moves down 1 px, a move stopped as any downward move.
            if t.frame.elevator_moving {
                self.position = t.go(self.bounds(), Down, t.rules.elevator, Move).to;
            }
            return Meeting::default();
        }
        let (walk, doing) = (self.walk(s.moves, t), s.doing);
        let wall_test = self.wall_test(t);
        // FollowingTheGround: on the ground she is pulled onto the floor, unless as the ball or
        // while her body changes shape she has a vertical speed to move by; Moonfall: a turn in
        // the moonfall is pulled as a turn on the ground over a drop.
        let pulled = self.vertical == Ground
            && (self.vy == ZERO || !matches!(doing, Shaping | Ball))
            || self.vertical == Moonfall && doing == Turning;
        self.space_jump(doing, t);
        let met = if wall_test == Kicked {
            // WallJump: kicking off she doesn't move vertically that frame.
            Some(WallKick)
        } else if doing == TakingOff {
            // Jumping: taking off she doesn't move vertically and the pull doesn't act.
            None
        } else {
            self.drop(doing, pulled.then_some(walk.pull), t)
        };
        // WhatHerMovesMet: a wall stop while running wins over walking off a ledge.
        let stopped = walk.stop.is_some() && doing == Running;
        let met = if stopped { Some(Wall) } else { met };
        Meeting { met, wall_test }
    }

    /// `SpeedBuildsAndBrakes`, `Dash`, `DriftInTheAir`, `WallsAndCeilings`: her horizontal move
    /// by her speed and dash, as her situation moves her.
    fn walk(&mut self, moves: Moves, t: &mut Tick) -> Walk {
        let facing = self.pose.facing;
        let stride = self.run.stride(moves, facing, self.vy == ZERO, t);
        let pull = self.run.speed + self.run.dash;
        let by = if stride == Both { pull } else { self.run.dash };
        let stop = if stride == Nothing {
            None
        } else {
            self.walk_and_seat(self.run.heading(facing), by, t)
        };
        // WallsAndCeilings, SpeedBuildsAndBrakes: a halt.
        if stop.is_some() || stride == DashThenHalt {
            self.run.halt();
        }
        Walk { pull, stop }
    }

    /// `BlocksStopFlush`, `SlopesSlowTheWalk`: a horizontal move, which a sloped floor at her
    /// leading edge shortens when it is made with her vertical speed exactly 0.
    fn hmove(&self, way: Heading, d: Fixed, t: &mut Tick) -> Moved {
        let d = if self.vy == ZERO {
            t.walk_shortened(self.bounds(), way, d)
        } else {
            d
        };
        t.go(self.bounds(), way, d, Move)
    }

    /// `SpeedBuildsAndBrakes`, `SlopesCarry`: a horizontal move, then the checks that run after
    /// every horizontal move, whether or not it met solid enemies, and whose lift or push seats
    /// her. Returns what stopped the move.
    fn walk_and_seat(&mut self, way: Heading, d: Fixed, t: &mut Tick) -> Option<Stop> {
        let moved = self.hmove(way, d, t);
        self.position = moved.to;
        if let Some(at) = t.seat(self.bounds()) {
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
    fn aim_step(&mut self, chosen: &mut Choice, t: &mut Tick) {
        let moved = self.hmove(self.pose.facing.side(), t.rules.aim_step, t);
        self.position.y = moved.to.y;
        if moved.stop != Some(Stop::Enemy) {
            self.position.x = moved.to.x;
        }
        if moved.stop.is_some() && chosen.pose.action == Action::Running {
            chosen.pose.action = AgainstWall;
            chosen.pose.gun = Gun::Stowed;
            chosen.effects = &[];
        }
    }

    /// `WallJump`: the near-wall test, in a spin carried on with a side held, looking 8 px away
    /// from it (left held: a wall on her right; else right held: on her left) from where her
    /// move left her. What stops it writes to her as a stop would, without moving her: a block
    /// her x subpixel, a solid enemy she touches her y subpixel. A wall near her readies the
    /// wall jump; once the spin has turned far, pressing jump kicks her off.
    fn wall_test(&mut self, t: &mut Tick) -> WallTest {
        let f = t.frame;
        if self.spin != CarriedOn || !f.pad.any(SIDES) {
            return Neither;
        }
        let wall = if f.pad.any(LEFT) { Right } else { Left };
        let seen = t.go(self.bounds(), wall, t.rules.wall_reach, Look);
        self.position = Position {
            x: self.position.x.with_sub(seen.to.x.sub()),
            y: seen.to.y,
        };
        if seen.stop.is_none() {
            Neither
        } else if !f.spin_turned_far {
            Readied
        } else if f.pressed.any(JUMP) {
            Kicked
        } else {
            Neither
        }
    }

    /// `BlocksStopFlush`: a vertical move by `v`, signed, downward positive. `SlopesCarry`: as a
    /// move of more than 0 starts she leaves any slope.
    fn vmove(&mut self, v: Fixed, t: &mut Tick) -> Moved {
        if v != ZERO {
            self.seat = Unseated;
        }
        let way = if v.negative() { Up } else { Down };
        t.go(self.bounds(), way, v.size(), Move)
    }

    /// `SlopesCarry`: a downward move or look that a stepped block stops seats her.
    fn seat_on(&mut self, stop: Option<Stop>) {
        if stop == Some(Stop::Stepped) {
            self.seat = Seated;
        }
    }

    /// Her vertical move, and what it met. `FollowingTheGround`, pulled (`pull`, the frame's
    /// speed): down by that speed (her horizontal speed plus her dash, subpixel part included)
    /// plus 1 px (`SlopesCarry`: 1 px on a slope), a move she doesn't move by her vertical speed.
    /// `RisingAndFalling`, not pulled: by her vertical speed, which then grows by the pull. On the
    /// ground (pulled, or the ball moving by its vertical speed) the floor stops her flush and
    /// never lands her, and finding no floor she walks off a ledge, moving the whole move, unless
    /// turning (`TurningRound`, `Moonfall`: she sinks instead) or changing shape (no move). In the
    /// air a floor is a landing, and an upward move stopped is a ceiling (`WallsAndCeilings`): the
    /// rise ends with no pull that frame.
    fn drop(&mut self, doing: Doing, pull: Option<Fixed>, t: &mut Tick) -> Option<Met> {
        let r = t.rules;
        let grounded = pull.is_some() || self.vertical == Ground && doing == Ball;
        let by = match pull {
            Some(_) if self.seat == Seated => r.ground_pull,
            Some(speed) => speed + r.ground_pull,
            None => self.vy,
        };
        let moved = self.vmove(by, t);
        let floor = moved.stop.is_some();
        if pull.is_some() && !floor && doing == Shaping {
            return None;
        }
        self.position = moved.to;
        if by.negative() && floor {
            self.fall_from_rest();
            return Some(Ceiling);
        }
        self.seat_on(moved.stop);
        if pull.is_none() {
            self.vy = self.pulled(r);
        }
        if grounded && !floor && doing != Turning {
            // FollowingTheGround: walking off a ledge she falls from rest, bounces 0.
            self.fall_from_rest();
            self.bounces = Bounces::None;
            return Some(NoFloor);
        }
        (floor && !grounded).then_some(Floor)
    }

    /// `Jumping`: with space jump, pressing jump while falling in a spin with her fall speed in
    /// the window jumps again, rising that same frame; not in a moonfall (`Moonfall`).
    fn space_jump(&mut self, doing: Doing, t: &Tick) {
        let (r, f) = (t.rules, t.frame);
        if self.vertical == Falling
            && doing == Spinning
            && f.pressed.any(JUMP)
            && f.equipment.space_jump
            && r.space_jump.contains(&self.vy.to_8_8())
        {
            self.rise(r.jump.of(f.equipment.hi_jump));
        }
    }

    /// `RisingAndFalling`: her vertical speed after the pull, which stops growing at pixel part
    /// 5; `Moonfall`: with no limit.
    fn pulled(&self, r: &Rules) -> Fixed {
        if self.vertical != Moonfall && self.vy.pixel() == r.fall_settle_px {
            self.vy
        } else {
            self.vy + r.gravity
        }
    }

    /// What her moves met acts on her. `WallJump`'s kick starts the wall jump rising at the
    /// wall-jump speed, x speed 0, following the pad; it ends a moonfall. `Landing`, a move by
    /// her vertical speed meeting a floor: turning, only the move stops, her speed going on
    /// growing by the pull. The spring ball with jump held hops again. The ball's landings each
    /// by their own rule, reading her vertical speed after the frame's pull: with no bounce yet
    /// it bounces at 3 px or more; after bounce 1 it bounces again at any speed, 1 px slower;
    /// after bounce 2 it comes to rest. While her body changes shape only her vertical motion
    /// stops. Else she lands: vertical speed 0, bounces 0, horizontal speed 0, following the
    /// pad; the dash keeps its value, to lose (§1.2).
    fn meet(&mut self, met: Option<Met>, doing: Doing, t: &Tick) {
        let (r, f) = (t.rules, t.frame);
        if met == Some(WallKick) {
            self.rise(r.wall_jump.of(f.equipment.hi_jump));
            self.run.speed = ZERO;
            self.run.carry = FollowsPad;
        }
        let ball = matches!(doing, Ball | Hop);
        let hops = ball && self.pose.body == SpringBall && f.pad.any(JUMP);
        let hop = hops.then_some((r.jump.of(f.equipment.hi_jump), Bounces::None));
        let bounce = match self.bounces {
            Bounces::None if self.vy.pixel() >= r.bounce_from_px => Some((r.bounce, Bounces::Once)),
            Bounces::Once => Some((r.bounce - r.second_bounce_less, Bounces::Twice)),
            Bounces::None | Bounces::Twice => None,
        }
        .filter(|_| ball);
        if met != Some(Floor) || doing == Turning {
            return;
        }
        if let Some((speed, bounces)) = hop.or(bounce) {
            self.rise(speed);
            self.bounces = bounces;
            return;
        }
        self.vertical = Ground;
        self.vy = ZERO;
        if doing == Shaping {
            return;
        }
        self.bounces = Bounces::None;
        self.run.speed = ZERO;
        self.run.carry = FollowsPad;
        self.run.dash_is = DashIs::ToLose;
    }

    /// `PoseFitAfterMoves`, `ShrinkingLowersHer`: she takes the pose the change gives, placed by
    /// pose fit. A change refused, or crouched `Still` taken instead, leaves her speeds and dash
    /// as her moves left them (`moved`), with her position and seat as her looks left them. Only
    /// the chosen pose brings its effects, in order: after pose fit's push or the shrink, the new
    /// pose's speeds, then the crouch take-off's lift. Returns where the camera follows her from
    /// (§1): after pose fit's push or a change of shape, her y pixel there with her y subpixel at
    /// the frame's start; else her y at the frame's start.
    fn take(&mut self, chosen: Choice, start: &Self, moved: Self, t: &mut Tick) -> Fixed {
        let (r, was) = (t.rules, self.pose);
        let fitted = fit(t, self.bounds(), was, chosen.pose);
        let (pushed, pose) = (fitted.at.y.pixel() != self.position.y.pixel(), fitted.pose);
        let effects = if fitted.change == Change::Chosen {
            chosen.effects
        } else {
            *self = moved;
            NONE
        };
        self.position = fitted.at;
        self.seat = if fitted.seated { Seated } else { self.seat };
        self.pose = pose;
        self.radius = t.body(pose);
        // ShrinkingLowersHer: the frame she starts to crouch or to morph she is lowered; standing
        // up or unmorphing she is lowered 0 px.
        let begins = pose.action != was.action;
        let shaped = begins && classify(pose, true).doing == Shaping;
        let shrink = if pose.action == Crouching {
            r.crouch_lower
        } else if pose.action == Morphing {
            r.morph_lower
        } else {
            ZERO
        };
        if shaped {
            self.lower(shrink, t);
        }
        let camera = if pushed || shaped {
            fixed(self.position.y.pixel(), start.position.y.sub())
        } else {
            start.position.y
        };
        for effect in effects {
            self.apply(*effect, start, t);
        }
        // The change of action's own effects: Jumping, starting a straight jump or a fall she
        // carries on along her facing if she has a dash, else she follows the pad; WallJump
        // (§1.2), a spin that followed a wall jump is carried on.
        let launches =
            pose.action == Action::Falling || pose.action == Jumping && pose.body == Upright;
        if begins && launches {
            self.run.launch(pose.facing);
        }
        if begins && was.action == WallJumping && pose.action == Action::Spinning {
            self.spin = CarriedOn;
        }
        camera
    }

    /// `ShrinkingLowersHer`: she looks down `by` at her new size, a block look (solid enemies
    /// don't stop it), and she is lowered as far as it got; what stops it writes to her as a
    /// stop would, and a stepped block seats her.
    fn lower(&mut self, by: Fixed, t: &mut Tick) {
        let seen = t.look(self.bounds(), by, Probe::Blocks);
        self.position = seen.to;
        self.seat_on(seen.stop);
    }

    /// `PadChart`, `AnimationEnds`: one effect of the change on her speeds, dash and position,
    /// from how she was at the frame's start (`start`).
    fn apply(&mut self, effect: Effect, start: &Self, t: &Tick) {
        let jump = t.rules.jump.of(t.frame.equipment.hi_jump);
        let turns = self.pose.facing != start.pose.facing;
        match effect {
            Rise => self.rise(jump),
            // Jumping: changing into a take-off from a crouch without aim she is also lifted.
            Lift if start.pose.aim == Aim::Forward => self.position.y -= t.rules.crouch_lift,
            // TurningRound: a turn, or a pose that faces her the other way (a spin reversed, the
            // ball turning round): her dash joins her speed and she travels the old way, now
            // against her new facing: a skid. A spin reversed while skidding turns her travel
            // round. The ball held to the side it faces doesn't turn.
            TurnRound if turns || matches!(self.pose.action, Action::Turning(_)) => {
                self.run.way = start.pose.facing.side();
                self.run.join_dash();
            }
            EndSkid if self.run.carry == CarriesOn => {
                self.run.carry = FollowsPad;
                self.run.way = self.pose.facing.side();
            }
            // PadChart: as running, her horizontal speed 0 at the frame's start or after the
            // frame's move: she follows the pad again, keeping the speed the move left her.
            AsRunning if start.run.speed == ZERO || self.run.speed == ZERO => {
                self.run.carry = FollowsPad;
            }
            AsRunning => self.run.join_dash(),
            LoseDash => self.run.dash = ZERO,
            Halt => self.run.halt(),
            // Moonfall: a spin she enters when a turn ends, with jump not just pressed, starts
            // with no rise: her vertical speed is 0 and she drops under gravity.
            Effect::Moonfall if t.frame.pressed.any(JUMP) => self.rise(jump),
            Effect::Moonfall => {
                self.fall_from_rest();
                self.vertical = Moonfall;
            }
            Lift | TurnRound | EndSkid => {}
        }
    }
}

/// No effects: a change refused, or crouched `Still` taken instead (`PoseFitAfterMoves`).
const NONE: &[Effect] = &[];
