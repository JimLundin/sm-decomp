//! One frame of her motion, in the order `PoseFitAfterMoves` gives: she moves as the pose at
//! its start says (`MovesAsShePosed`), horizontally, then by the ground pull or her vertical
//! move, then by the aim change's step; what those moves met is known; the pose is chosen
//! (`PoseFollows`); pose fit or the shrink places her; then the change's effects act.

use crate::chart::{Choice, Moment};
use crate::collide::Probe::{Blocks, Look, Move, Walk};
use crate::collide::Stop;
use crate::motion::Carry::FollowsPad;
use crate::motion::DashIs::ToLose;
use crate::motion::Stride::{Both, DashThenHalt, Nothing};
use crate::rows::Doing::{
    self, Ball, Hop, Running, Shaping, Spinning, StraightJump, TakingOff, Turning,
};
use crate::rows::Effect::{
    self, AsRunning, EndSkid, Halt, Launch, Lift, LoseDash, Rise, SpinOn, TurnRound,
};
use crate::rows::Met::{self, Ceiling, Floor, NoFloor, StepBlocked, Wall, WallKick};
use crate::rows::Situation;
use crate::rules::Rules;
use crate::samus::Seat::{Seated, Unseated};
use crate::samus::WallTest::{Kicked, Neither, Readied};
use crate::samus::{Bounces, Drawn, Outcome, Samus, Spin, Standing, Vertical, WallTest};
use crate::situation::{ENTERING, classify};
use crate::tick::{Frame, Tick};
use clean_types::Action::{self, Still, WallJumping};
use clean_types::Body::{SpringBall, Upright};
use clean_types::Heading::{self, Down, Left, Right};
use clean_types::button::{JUMP, LEFT, SHOOT, SIDES};
use clean_types::{Aim, Buttons, Equipment, Facing, Fixed, Parts, Pose, World, ZERO, px};

/// `WhatHerMovesMet`: what a frame's moves met, and the near-wall test's outcome (`WallJump`).
#[derive(Default)]
struct Meeting {
    /// What her moves met.
    met: Option<Met>,
    /// The near-wall test's outcome.
    wall_test: WallTest,
}

/// `FollowingTheGround`, `WallsAndCeilings`: what a walk did: the speed the ground pull reads,
/// from before any stop, and a wall stop while running.
struct Walked {
    /// Her horizontal speed and dash.
    pull: Fixed,
    /// A wall stop while running (`WhatHerMovesMet`).
    wall: Option<Met>,
}

/// No effects: a change refused, or crouched `Still` taken instead (`PoseFitAfterMoves`).
const NONE: &[Effect] = &[];

/// `HowSheIsDrawn`: the spin (and a wall jump's) drawn as the screw attack with screw attack,
/// else as the space jump with space jump; facing the screen, the other suit's sprite with it.
const fn drawn(pose: Pose, e: Equipment) -> Drawn {
    let spin = matches!(pose.action, Action::Spinning | WallJumping);
    if spin && e.screw_attack {
        Drawn::ScrewAttack
    } else if spin && e.space_jump {
        Drawn::SpaceJump
    } else if matches!(pose.facing, Facing::Screen) && e.other_suit {
        Drawn::OtherSuit
    } else {
        Drawn::Plain
    }
}

/// `AimingAndShooting`: standing, unaimed and not landing, holding shoot restarts the standing
/// animation.
const fn standing(p: Pose, pad: Buttons) -> Standing {
    let unaimed = matches!((p.body, p.action, p.aim), (Upright, Still, Aim::Forward));
    if unaimed && pad.any(SHOOT) {
        Standing::Restarts
    } else {
        Standing::Plays
    }
}

impl Samus {
    /// `PoseFollows`: one frame, in the order it runs: her moves as the pose at its start says
    /// (`MovesAsShePosed`) and what they met; the pose chosen; pose fit or the shrink
    /// (`PoseFitAfterMoves`); then the change's effects. `TouchedCells`: the cells she is inside
    /// come first, from where she starts.
    pub fn step(&mut self, frame: &Frame, world: &dyn World, rules: &Rules) -> Outcome {
        let mut t = Tick {
            frame,
            world,
            rules,
            reached: Vec::new(),
        };
        let inside = t.inside(self.bounds());
        let start = *self;
        let situation = classify(start.pose);
        self.begin(situation.doing, frame.pad);
        let meeting = self.moves(situation, &mut t);
        let moved = *self;
        self.meet(meeting.met, situation.doing, &t);
        let moment = Moment {
            situation,
            pose: start.pose,
            moving: start.run.speed != ZERO,
            airborne: !self.vertical.on_ground(),
            frame,
            met: meeting.met,
        };
        let mut chosen = moment.choose();
        if chosen.steps && self.aim_step(&mut t) {
            let blocked = Moment {
                pose: chosen.pose,
                met: Some(StepBlocked),
                ..moment
            };
            chosen = blocked.met().unwrap_or(chosen);
        }
        let camera_from = self.take(chosen, &start, moved, &mut t);
        Outcome {
            wall_test: meeting.wall_test,
            standing: standing(start.pose, frame.pad),
            drawn: drawn(self.pose, frame.equipment),
            skidding: self.run.skids(self.pose.facing),
            reached: t.reached,
            inside,
            camera_from,
        }
    }

    /// `WallJump` (§1.2): how long she has been in this spin. `RisingAndFalling`: the rise ends
    /// at the start of the frame after it is spent, and in a straight jump, a spin or the hop at
    /// the start of any frame with jump not held.
    fn begin(&mut self, doing: Doing, pad: Buttons) {
        self.spin = self.spin.then(doing == Spinning);
        let lets_go = !pad.any(JUMP) && matches!(doing, StraightJump | Spinning | Hop);
        let spent = !self.vy.negative() && self.vy != ZERO;
        if self.vertical == Vertical::Rising && (lets_go || spent) {
            self.fall_from_rest();
        }
    }

    /// `MovesAsShePosed`: her moves, as the pose at the frame's start says, and what they met.
    /// `SpeedBuildsAndBrakes`: facing the screen she doesn't move; `RidingTheElevator`: on a
    /// moving elevator she moves down 1 px, a move stopped as any downward move. `WallJump`:
    /// kicking off she doesn't move vertically that frame; `Jumping`: nor taking off.
    /// `WhatHerMovesMet`: a wall stop while running wins over walking off a ledge.
    fn moves(&mut self, s: &Situation, t: &mut Tick) -> Meeting {
        if self.pose.facing == Facing::Screen {
            if t.frame.elevator_moving {
                self.position = t.go(self.bounds(), Down, t.rules.elevator, Move).to;
            }
            return Meeting::default();
        }
        let walk = self.walk(s, t);
        let wall_test = self.wall_test(t);
        self.space_jump(s.doing, t);
        let met = if wall_test == Kicked {
            Some(WallKick)
        } else if s.doing == TakingOff {
            None
        } else {
            self.drop(s.doing, walk.pull, t)
        };
        Meeting {
            met: walk.wall.or(met),
            wall_test,
        }
    }

    /// `SpeedBuildsAndBrakes`, `Dash`, `DriftInTheAir`, `WallsAndCeilings`: her horizontal move
    /// by her speed and dash, as her situation moves her, and the halt that follows a wall stop
    /// or a move by her dash alone.
    fn walk(&mut self, s: &Situation, t: &mut Tick) -> Walked {
        let facing = self.pose.facing;
        let stride = self.run.stride(s.moves, facing, self.vertical_motion(), t);
        let pull = self.run.speed + self.run.dash;
        if stride == Nothing {
            return Walked { pull, wall: None };
        }
        let by = if stride == Both { pull } else { self.run.dash };
        let stop = self.hmove(self.run.heading(facing), by, t);
        if let Some(y) = t.seat(self.bounds()) {
            self.position.y = y;
            self.seat = Seated;
        }
        if stop.is_some() || stride == DashThenHalt {
            self.run.halt();
        }
        Walked {
            pull,
            wall: (stop.is_some() && s.doing == Running).then_some(Wall),
        }
    }

    /// `BlocksStopFlush`, `SlopesSlowTheWalk`: a horizontal move, a walk when it is made with
    /// her vertical speed exactly 0. Returns what stopped it. `SlopesCarry`: the walk's checks
    /// after it are `walk`'s.
    fn hmove(&mut self, way: Heading, d: Fixed, t: &mut Tick) -> Option<Stop> {
        let probe = if self.vy == ZERO { Walk } else { Move };
        let moved = t.go(self.bounds(), way, d, probe);
        self.position = moved.to;
        moved.stop
    }

    /// `AimingAndShooting`: a frame whose chosen pose brings the step moves her 1 px forward, by
    /// her facing at the frame's start, after her moves: a horizontal move with no `SlopesCarry`
    /// checks after it. A solid enemy within its reach stops it with no move at all, one she
    /// touches still clearing her y subpixel. Returns whether something was in the way
    /// (`WhatHerMovesMet`).
    fn aim_step(&mut self, t: &mut Tick) -> bool {
        let x = self.position.x;
        let stop = self.hmove(self.pose.facing.side(), t.rules.aim_step, t);
        if stop == Some(Stop::Enemy) {
            self.position.x = x;
        }
        stop.is_some()
    }

    /// `WallJump`: the near-wall test, in a spin carried on with a side held, looking 8 px away
    /// from it (left held: a wall on her right; else right held: on her left) from where her
    /// move left her. What stops it writes to her as a stop would, without moving her: a block
    /// her x subpixel, a solid enemy she touches her y subpixel. A wall near her readies the
    /// wall jump; once the spin has turned far, pressing jump kicks her off.
    fn wall_test(&mut self, t: &mut Tick) -> WallTest {
        let f = t.frame;
        if !self.spin.carried_on() || !f.pad.any(SIDES) {
            return Neither;
        }
        let wall = if f.pad.any(LEFT) { Right } else { Left };
        let seen = t.go(self.bounds(), wall, t.rules.wall_reach, Look);
        self.position = seen.to;
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

    /// Her vertical move, and what it met. `FollowingTheGround`: on the ground she is pulled
    /// (`speed`, the frame's horizontal speed and dash) down by that speed plus 1 px
    /// (`SlopesCarry`: 1 px on a slope), unless as the ball or while her body changes shape she
    /// has a vertical motion to move by; `Moonfall`: a turn in the moonfall is pulled as a turn
    /// on the ground over a drop. `RisingAndFalling`, not pulled: she moves by her vertical
    /// speed, which then grows by the pull. On the ground (pulled, or the ball moving by its
    /// vertical speed) the floor stops her flush and never lands her, and finding no floor she
    /// walks off a ledge, moving the whole move, unless turning (`TurningRound`: she sinks
    /// instead) or changing shape (no move). In the air a floor is a landing, and an upward move
    /// stopped is a ceiling (`WallsAndCeilings`): the rise ends with no pull that frame.
    /// `SlopesCarry`: a move of more than 0 unseats her as it starts.
    fn drop(&mut self, doing: Doing, speed: Fixed, t: &mut Tick) -> Option<Met> {
        let r = t.rules;
        let ground = self.vertical.on_ground();
        let pulled = ground && !(self.vertical_motion() && matches!(doing, Shaping | Ball))
            || self.vertical == Vertical::Moonfall && doing == Turning;
        let grounded = pulled || ground && doing == Ball;
        let by = if pulled && self.seat == Seated {
            r.ground_pull
        } else if pulled {
            speed + r.ground_pull
        } else {
            self.vy
        };
        let moved = t.vmove(self.bounds(), by, Move);
        if by != ZERO {
            self.seat = Unseated;
        }
        let floor = moved.stop.is_some();
        if pulled && !floor && doing == Shaping {
            return None;
        }
        self.reach(&moved);
        if by.negative() && floor {
            self.fall_from_rest();
            return Some(Ceiling);
        }
        if !pulled {
            self.vy = self.pulled(r);
        }
        if grounded && !floor && doing != Turning {
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
        if self.vertical == Vertical::Falling
            && doing == Spinning
            && f.pressed.any(JUMP)
            && f.equipment.space_jump
            && r.space_jump.contains(&self.vy.to_8_8().0)
        {
            self.rise(r.jump.of(f.equipment.hi_jump));
        }
    }

    /// `RisingAndFalling`: her vertical speed after the pull, which stops growing at pixel part
    /// 5; `Moonfall`: with no limit.
    fn pulled(&self, r: &Rules) -> Fixed {
        if self.vertical != Vertical::Moonfall && self.vy.pixel() == r.fall_settle_px {
            self.vy
        } else {
            self.vy + r.gravity
        }
    }

    /// `Landing`: what her moves met acts on her. `WallJump`'s kick starts the wall jump rising
    /// at the wall-jump speed, x speed 0, following the pad; it ends a moonfall. A move by her
    /// vertical speed meeting a floor: turning, only the move stops, her speed going on growing
    /// by the pull. The spring ball with jump held hops again. The ball's landings each by their
    /// own rule, reading her vertical speed after the frame's pull: with no bounce yet it bounces
    /// at 3 px or more; after bounce 1 it bounces again at any speed, 1 px slower; after bounce
    /// 2 it comes to rest. While her body changes shape only her vertical motion stops. Else she
    /// lands: vertical speed 0, bounces 0, horizontal speed 0, following the pad; the dash keeps
    /// its value, to lose (§1.2).
    fn meet(&mut self, met: Option<Met>, doing: Doing, t: &Tick) {
        let (r, f) = (t.rules, t.frame);
        let hi_jump = f.equipment.hi_jump;
        if met == Some(WallKick) {
            self.rise(r.wall_jump.of(hi_jump));
            self.run.stop();
        }
        if met != Some(Floor) || doing == Turning {
            return;
        }
        let rises = match self.bounces {
            _ if self.pose.body == SpringBall && f.pad.any(JUMP) => {
                Some((r.jump.of(hi_jump), Bounces::None))
            }
            Bounces::None if self.vy.pixel() >= r.bounce_from_px => Some((r.bounce, Bounces::Once)),
            Bounces::Once => Some((r.bounce - r.second_bounce_less, Bounces::Twice)),
            Bounces::None | Bounces::Twice => None,
        };
        if let Some((speed, bounces)) = rises.filter(|_| matches!(doing, Ball | Hop)) {
            self.rise(speed);
            self.bounces = bounces;
            return;
        }
        self.vertical = Vertical::Ground;
        self.vy = ZERO;
        if doing == Shaping {
            return;
        }
        self.bounces = Bounces::None;
        self.run.stop();
        self.run.dash_is = ToLose;
    }

    /// `PoseFollows`: she takes the chosen pose, placed by pose fit (`PoseFitAfterMoves`). A
    /// change refused, or crouched `Still` taken instead, leaves her speeds and dash as her moves
    /// left them (`moved`), with her position and seat as her looks left them, and brings none
    /// of the chosen pose's effects. The chosen pose brings, in order: after pose fit's push the
    /// shrink (`ShrinkingLowersHer`: she looks down its distance at her new size, a block look,
    /// and is lowered as far as it got, after what stops it writes to her as a stop would), then
    /// its row's effects, then the change of action's own (`ENTERING`). Returns where the camera
    /// follows her from (§1).
    fn take(&mut self, chosen: Choice, start: &Self, moved: Self, t: &mut Tick) -> Fixed {
        let chosen = match self.fit(t, chosen.pose) {
            None => chosen,
            Some(pose) => {
                *self = Self {
                    position: self.position,
                    seat: self.seat,
                    ..moved
                };
                Choice {
                    pose,
                    effects: NONE,
                    lower: None,
                    ..chosen
                }
            }
        };
        self.pose = chosen.pose;
        self.radius = t.body(chosen.pose);
        if let Some(by) = chosen.lower {
            let seen = t.vmove(self.bounds(), t.rules.lowers(by), Blocks);
            self.reach(&seen);
            self.position.y += px(seen.gone);
        }
        let camera = self.camera_from(start.position.y, moved.position.y, chosen.lower.is_some());
        let entered = ENTERING
            .iter()
            .filter(|e| e.on(start.pose.action, chosen.pose));
        for effect in chosen.effects.iter().chain(entered.map(|e| &e.effect)) {
            self.apply(*effect, start, t);
        }
        camera
    }

    /// §1, `PoseFitAfterMoves`, `ShrinkingLowersHer`: where the camera follows her from. After
    /// pose fit moved her (her y pixel differs from `moved`'s, after the frame's moves), or the
    /// frame a change of shape starts (`shapes`), her y pixel now with her y subpixel at the
    /// frame's start; else her y at the frame's start, `start`.
    fn camera_from(&self, start: Fixed, moved: Fixed, shapes: bool) -> Fixed {
        if shapes || self.position.y.pixel() != moved.pixel() {
            self.position.y.with_sub(start.sub())
        } else {
            start
        }
    }

    /// `Jumping`, `TurningRound`, `Moonfall`: one effect of the change (`PoseFollows`) on her
    /// speeds, dash and position, from how she was at the frame's start (`start`). `Jumping`:
    /// changing into a take-off from a crouch without aim she is also lifted; starting a straight
    /// jump or a fall she carries on with a dash, else follows the pad. `WallJump`: a spin that
    /// followed a wall jump is carried on. `TurningRound`: a turn, or a pose that faces her the other
    /// way (a spin reversed, the ball turning round): her dash joins her speed and she travels
    /// against her new facing, a skid; the moonwalk's jump, the one turn that keeps her facing,
    /// skids too, and a spin reversed while skidding turns her travel round. The ball held to
    /// the side it faces doesn't turn. `PadChart`: a skid ends along her new facing; as running,
    /// her horizontal speed 0 at the frame's start or after the frame's move, she follows the pad
    /// again, keeping the speed the move left her. `Moonfall`: a spin she enters when a turn
    /// ends, with jump not just pressed, starts with no rise: her vertical speed is 0 and she
    /// drops under gravity.
    fn apply(&mut self, effect: Effect, start: &Self, t: &Tick) {
        let jump = t.rules.jump.of(t.frame.equipment.hi_jump);
        let pose = self.pose;
        match effect {
            Rise => self.rise(jump),
            Lift if start.pose.aim == Aim::Forward => self.position.y -= t.rules.crouch_lift,
            TurnRound
                if pose.facing != start.pose.facing
                    || matches!(pose.action, Action::Turning(_)) =>
            {
                self.run.way = pose.facing.back().side();
                self.run.join_dash();
            }
            EndSkid => {
                self.run.carry = FollowsPad;
                self.run.way = pose.facing.side();
            }
            AsRunning if start.run.speed == ZERO || self.run.speed == ZERO => {
                self.run.carry = FollowsPad;
            }
            AsRunning => self.run.join_dash(),
            LoseDash => self.run.dash = ZERO,
            Halt => self.run.halt(),
            Effect::Moonfall if t.frame.pressed.any(JUMP) => self.rise(jump),
            Effect::Moonfall => {
                self.fall_from_rest();
                self.vertical = Vertical::Moonfall;
            }
            Launch => self.run.launch(pose.facing),
            SpinOn => self.spin = Spin::CARRIED_ON,
            Lift | TurnRound => {}
        }
    }
}
