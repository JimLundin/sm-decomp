//! One frame of her motion, in the order `PoseFitAfterMoves` gives: she moves as the pose at
//! its start says (`MovesAsShePosed`), horizontally, then by the ground pull or her vertical
//! move; what those moves met is known; the pose is chosen (`PoseFollows`), and the aim change's
//! step made; pose fit or the shrink places her; then what she met and the change's effects act,
//! where the change is taken.

use crate::chart::table::STEP_BLOCKED;
use crate::chart::{Choice, Moment};
use crate::collide::Probe::{Blocks, Look, Move, Walk};
use crate::collide::Stop;
use crate::fit::{Fitted, crouched};
use crate::motion::Carry::FollowsPad;
use crate::motion::DashIs::ToLose;
use crate::motion::Stride::{Both, DashThenHalt, Nothing};
use crate::rows::Doing::{
    self, Ball, Hop, Running, Shaping, Spinning, StraightJump, TakingOff, Turning,
};
use crate::rows::Effect::{
    self, AsRunning, EndSkid, Halt, Launch, Lift, LoseDash, OnTheGround, Rise, SpinOn, TurnRound,
};
use crate::rows::Met::{self, Ceiling, Floor, NoFloor, Wall, WallKick};
use crate::rows::Situation;
use crate::rules::{Bounce, Lowers, Rules};
use crate::samus::Seat::{Seated, Unseated};
use crate::samus::WallTest::{Kicked, Neither, Readied};
use crate::samus::{Bounces, Drawn, Outcome, Samus, Spin, Standing, Vertical, WallTest};
use crate::situation::{ENTERING, classify};
use crate::tick::{Frame, Tick};
use clean_types::Action;
use clean_types::Body::{SpringBall, Upright};
use clean_types::button::{JUMP, LEFT, SHOOT, SIDES};
use clean_types::{Aim, Buttons, Facing, Fixed, Item, Parts, Pose, Set, Side, World, ZERO, px};

/// `Landing`, `WallJump`: what what her moves met does to her motion, once the change it comes
/// with is taken (`PoseFitAfterMoves`: a change refused, or the crouch taken instead, brings
/// none of it). The pose it chooses is the chart's (`WhatHerMovesMet`).
#[derive(Clone, Copy)]
enum Touchdown {
    /// `WallJump`'s kick: she rises at the wall-jump speed, x speed 0, following the pad; it
    /// ends a moonfall.
    Kick,
    /// `Landing`: the spring ball landing with jump held hops again at the jump speed, bounces 0.
    Hop,
    /// `Landing`: the ball bounces.
    Bounce(Bounce),
    /// `Landing` while her body changes shape: only her vertical motion stops; both speeds, and
    /// whether she carries on, are kept.
    Settle,
    /// `Landing`: vertical speed 0, bounces 0, horizontal speed 0, following the pad; the dash
    /// keeps its value, to lose (§1.2).
    Land,
}

impl Touchdown {
    /// `RisingAndFalling`: whether it starts her rising, so that she is off the ground once her
    /// moves are made.
    const fn rises(self) -> bool {
        matches!(self, Self::Kick | Self::Hop | Self::Bounce(_))
    }
}

/// `FollowingTheGround`, `RisingAndFalling`, `Moonfall`: how her vertical move goes this frame.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Fall {
    /// The ground pull: on the ground, with no vertical motion she moves by (a turn in the
    /// moonfall too), she is pulled down by that frame's speed plus 1 px, 1 px seated on a slope.
    /// The floor stops her flush and never lands her; finding none she walks off a ledge, unless
    /// turning (she sinks) or changing shape (no move).
    Pulled,
    /// The ball on the ground with a vertical motion: it moves by its vertical speed under the
    /// pull; the floor stops it flush and never lands it, and finding none it walks off a ledge.
    Grounded,
    /// In the air, or while her body changes shape with a vertical motion: she moves by her
    /// vertical speed under the pull; a floor lands her, and a block above ends the rise.
    Free,
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
/// from before any stop, and a wall stop while running.
struct Walked {
    /// Her horizontal speed and dash.
    pull: Fixed,
    /// A wall stop while running (`WhatHerMovesMet`).
    wall: Option<Met>,
}

/// `HowSheIsDrawn`: the spin (and a wall jump's) drawn as the screw attack with screw attack,
/// else as the space jump with space jump; facing the screen, the other suit's sprite with varia
/// or gravity.
fn drawn(pose: Pose, items: &Set<Item>) -> Drawn {
    let spin = matches!(pose.action, Action::Spinning | Action::WallJumping);
    let suit = items[Item::VariaSuit] || items[Item::GravitySuit];
    if spin && items[Item::ScrewAttack] {
        Drawn::ScrewAttack
    } else if spin && items[Item::SpaceJump] {
        Drawn::SpaceJump
    } else if pose.facing == Facing::Screen && suit {
        Drawn::OtherSuit
    } else {
        Drawn::Plain
    }
}

/// `PoseFollows`: a change into `Morphing` without the morph ball is refused, as pose fit
/// refuses one: she ends the frame in the pose she had, aim included, the pad's other rows don't
/// step in, and none of the change's effects come, no lowering and no placement.
fn morph_refused(chosen: Pose, items: &Set<Item>) -> bool {
    chosen.action == Action::Morphing && !items[Item::MorphBall]
}

/// `AimingAndShooting`: standing, unaimed and not landing, holding shoot restarts the standing
/// animation.
const fn standing(p: Pose, pad: Buttons) -> Standing {
    let unaimed = matches!(
        (p.body, p.action, p.aim),
        (Upright, Action::Still, Aim::Forward)
    );
    if unaimed && pad.any(SHOOT) {
        Standing::Restarts
    } else {
        Standing::Plays
    }
}

impl Samus {
    /// `PoseFollows`: one frame, in the order it runs: her moves as the pose at its start says
    /// (`MovesAsShePosed`) and what they met, which tells whether she is off the ground; the pose
    /// chosen, the pad chart reading jump held through as a press (`Jumping`), and the aim
    /// change's step; pose fit or the shrink (`PoseFitAfterMoves`); then, where she takes the
    /// change, what she met and the change's effects; at its end, what she remembers of jump
    /// held. `TouchedCells`: the cells she is inside come first, from where she starts.
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
        let touchdown = self.touchdown(meeting.met, situation.doing, &t);
        let moment = Moment {
            moving: start.run.speed != ZERO,
            airborne: touchdown.map_or_else(|| !self.vertical.on_ground(), Touchdown::rises),
            jump_held_through: start.jump_held.presses(rules),
            frame,
            met: meeting.met,
        };
        let chosen = self.step_forward(moment.choose(start.pose, situation), &mut t);
        let placed = self.take(chosen, touchdown, &start, &mut t);
        self.jump_held = start.jump_held.then(&moment, start.pose, self.pose);
        Outcome {
            wall_test: meeting.wall_test,
            standing: standing(start.pose, frame.pad),
            drawn: drawn(self.pose, &frame.items),
            skidding: self.run.skids(self.pose.facing),
            reached: t.reached,
            inside,
            placed,
            rising: self.rising(),
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
                self.position = t.vmove(self.bounds(), t.rules.elevator, Move).to;
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
        let stop = self.hmove(self.run.moving(facing), by, t);
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
    fn hmove(&mut self, way: Side, d: Fixed, t: &mut Tick) -> Option<Stop> {
        let probe = if self.vy == ZERO { Walk } else { Move };
        let moved = t.go(self.bounds().axis(way.heading(), d, probe));
        self.position = moved.to;
        moved.stop
    }

    /// `AimingAndShooting`: a frame whose chosen pose brings the step moves her 1 px forward, by
    /// her facing at the frame's start, after her moves: a horizontal move with no `SlopesCarry`
    /// checks after it. A solid enemy within its reach stops it with no move at all, one she
    /// touches still clearing her y subpixel. `WhatHerMovesMet`: something in its way puts the
    /// running pose chosen against the wall, with its aim (`STEP_BLOCKED`), and none of its row's
    /// effects come.
    fn step_forward(&mut self, chosen: Choice, t: &mut Tick) -> Choice {
        if !chosen.steps {
            return chosen;
        }
        let x = self.position.x;
        let stop = self.hmove(Side::of(self.pose.facing), t.rules.aim_step, t);
        if stop == Some(Stop::Enemy) {
            self.position.x = x;
        }
        if stop.is_some() && chosen.pose.action == Action::Running {
            Choice::instead(STEP_BLOCKED.pose_from(chosen.pose, None))
        } else {
            chosen
        }
    }

    /// `WallJump`: the near-wall test, in a spin carried on with a side held, looking 8 px away
    /// from it (left held: a wall on her right; else right held: on her left) from where her
    /// move left her. What stops it writes to her as a stop would, without moving her: a block
    /// her x subpixel, a solid enemy she touches her y subpixel. A wall near her readies the
    /// wall jump; once the spin has turned far, pressing jump kicks her off.
    fn wall_test(&mut self, t: &mut Tick) -> WallTest {
        let f = t.frame;
        if self.spin != Spin::CarriedOn || !f.pad.any(SIDES) {
            return Neither;
        }
        let wall = if f.pad.any(LEFT) {
            Side::Right
        } else {
            Side::Left
        };
        let seen = t.go(self.bounds().axis(wall.heading(), t.rules.wall_reach, Look));
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

    /// `FollowingTheGround`, `Moonfall`: how her vertical move goes, as she is doing. On the
    /// ground she is pulled, unless as the ball or while her body changes shape she has a
    /// vertical motion to move by; a turn in the moonfall is pulled as a turn on the ground over a
    /// drop.
    fn fall(&self, doing: Doing) -> Fall {
        let ground = self.vertical.on_ground();
        let moves_by_it = self.vertical_motion() && matches!(doing, Shaping | Ball);
        if ground && !moves_by_it || self.vertical == Vertical::Moonfall && doing == Turning {
            Fall::Pulled
        } else if ground && doing == Ball {
            Fall::Grounded
        } else {
            Fall::Free
        }
    }

    /// Her vertical move (`fall`), and what it met. `FollowingTheGround`: pulled, she moves down
    /// by `speed` (the frame's horizontal speed and dash) plus 1 px (`SlopesCarry`: 1 px seated);
    /// else she moves by her vertical speed, which then grows by the pull (`RisingAndFalling`).
    /// An upward move stopped is a ceiling (`WallsAndCeilings`): the rise ends with no pull that
    /// frame. `SlopesCarry`: a move of more than 0 unseats her as it starts.
    fn drop(&mut self, doing: Doing, speed: Fixed, t: &mut Tick) -> Option<Met> {
        let r = t.rules;
        let fall = self.fall(doing);
        let by = match fall {
            Fall::Pulled if self.seat == Seated => r.ground_pull,
            Fall::Pulled => speed + r.ground_pull,
            Fall::Grounded | Fall::Free => self.vy,
        };
        let moved = t.vmove(self.bounds(), by, Move);
        if by != ZERO {
            self.seat = Unseated;
        }
        let floor = moved.stop.is_some();
        if fall == Fall::Pulled && !floor && doing == Shaping {
            return None;
        }
        self.reach(&moved);
        if by.negative() && floor {
            self.fall_from_rest();
            return Some(Ceiling);
        }
        if fall != Fall::Pulled {
            self.vy = self.pulled(r);
        }
        match (fall, floor) {
            (Fall::Free, true) => Some(Floor),
            (Fall::Pulled, false) if doing == Turning => None,
            (Fall::Pulled | Fall::Grounded, false) => Some(self.walk_off()),
            (Fall::Pulled | Fall::Grounded, true) | (Fall::Free, false) => None,
        }
    }

    /// `FollowingTheGround`: walking off a ledge, she falls from rest, or with an upward speed she
    /// kept on the ground rises on by it; bounces 0.
    fn walk_off(&mut self) -> Met {
        if self.vy.negative() {
            self.vertical = Vertical::Rising;
        } else {
            self.fall_from_rest();
        }
        self.bounces = Bounces::None;
        NoFloor
    }

    /// `Jumping`: with space jump, pressing jump while falling in a spin with her fall speed in
    /// the window jumps again, rising that same frame; not in a moonfall (`Moonfall`).
    fn space_jump(&mut self, doing: Doing, t: &Tick) {
        let (r, f) = (t.rules, t.frame);
        if self.vertical == Vertical::Falling
            && doing == Spinning
            && f.pressed.any(JUMP)
            && f.items[Item::SpaceJump]
            && r.space_jump.contains(&self.vy.to_8_8().0)
        {
            self.jump(t);
        }
    }

    /// `Jumping`: she starts rising at the jump speed her items give.
    fn jump(&mut self, t: &Tick) {
        self.rise(t.rules.jump.of(&t.frame.items));
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

    /// `Landing`, `WallJump`: what her moves met will do to her motion, once she takes the
    /// change: `WallJump`'s kick, or a floor met by a move by her vertical speed.
    fn touchdown(&self, met: Option<Met>, doing: Doing, t: &Tick) -> Option<Touchdown> {
        match met? {
            WallKick => Some(Touchdown::Kick),
            Floor => self.landing(doing, t),
            Wall | NoFloor | Ceiling => None,
        }
    }

    /// `Landing`: what a floor does as she is doing. Turning, only the move stops, her speed
    /// going on growing by the pull. While her body changes shape only her vertical motion stops.
    /// The spring ball with jump held hops again; else the ball bounces, or comes to rest. Anything
    /// else lands.
    fn landing(&self, doing: Doing, t: &Tick) -> Option<Touchdown> {
        match doing {
            Turning => None,
            Shaping => Some(Touchdown::Settle),
            Ball | Hop if self.pose.body == SpringBall && t.frame.pad.any(JUMP) => {
                Some(Touchdown::Hop)
            }
            Ball | Hop => Some(
                self.bounce(t.rules)
                    .map_or(Touchdown::Land, Touchdown::Bounce),
            ),
            Doing::Standing
            | Running
            | TakingOff
            | StraightJump
            | Spinning
            | Doing::Falling
            | Doing::WallJumping
            | Doing::Moonwalking
            | Doing::AgainstWall
            | Doing::Crouched => Some(Touchdown::Land),
        }
    }

    /// `Landing`: the ball's bounce a landing makes, if any: the one at the bounces she has had,
    /// where her vertical speed after the frame's pull reaches it (its pixel part alone counts).
    fn bounce(&self, r: &Rules) -> Option<Bounce> {
        r.bounces
            .iter()
            .find(|b| b.at == self.bounces && self.vy.pixel() >= b.from_px)
            .copied()
    }

    /// `Landing`, `WallJump`: what her moves met acts on her (`Touchdown`); a bounce rises at
    /// the bounce speed less its own cut.
    fn meet(&mut self, touchdown: Touchdown, t: &Tick) {
        match touchdown {
            Touchdown::Kick => {
                self.rise(t.rules.wall_jump.of(&t.frame.items));
                self.run.stop();
            }
            Touchdown::Hop => {
                self.jump(t);
                self.bounces = Bounces::None;
            }
            Touchdown::Bounce(bounce) => {
                self.rise(t.rules.bounce - bounce.slower);
                self.bounces = bounce.then;
            }
            Touchdown::Settle => self.settle(),
            Touchdown::Land => {
                self.settle();
                self.bounces = Bounces::None;
                self.run.stop();
                self.run.dash_is = ToLose;
            }
        }
    }

    /// `PoseFollows`, `PoseFitAfterMoves`: what she becomes. Pose fit (or a morph refused) says
    /// whether she takes the chosen pose with what her moves met (`touchdown`), or ends in the
    /// pose she had or crouched `Still` with neither: her speeds, dash and whether she carries on
    /// or skids stay as her moves left them, her position and seat as her looks left them. What
    /// she becomes brings, in order: after pose fit's push the shrink (`shrink`), then what she
    /// met, then its row's effects, then the change of action's own (`ENTERING`).
    ///
    /// §1, `PoseFitAfterMoves`, `ShrinkingLowersHer`: where pose fit moved her, or a change of
    /// shape starts (a lowering of 0 px included), she is placed at her y pixel just after that,
    /// before the effects, with the subpixel her y had at the frame's start. Returns that y, the
    /// placement, or none on any other frame.
    fn take(
        &mut self,
        chosen: Choice,
        touchdown: Option<Touchdown>,
        start: &Self,
        t: &mut Tick,
    ) -> Option<Fixed> {
        let fitted = if morph_refused(chosen.pose, &t.frame.items) {
            Fitted::Refused
        } else {
            self.fit(t, chosen.pose)
        };
        let (became, touchdown) = match fitted {
            Fitted::Taken | Fitted::Pushed(_) => (chosen, touchdown),
            Fitted::Refused => (Choice::instead(self.pose), None),
            Fitted::Crouched { .. } => (Choice::instead(crouched(self.pose.facing)), None),
        };
        self.pose = became.pose;
        self.radius = t.body(became.pose);
        if let Some(lowers) = became.lower {
            self.shrink(lowers, t);
        }
        let placed = (fitted.moved() || became.lower.is_some())
            .then(|| self.position.y.with_sub(start.position.y.sub()));
        if let Some(touchdown) = touchdown {
            self.meet(touchdown, t);
        }
        let entered = ENTERING
            .iter()
            .filter(|e| e.on(start.pose.action, became.pose));
        for effect in became.effects.iter().chain(entered.map(|e| &e.effect)) {
            self.apply(*effect, start, t);
        }
        placed
    }

    /// `ShrinkingLowersHer`: the frame a change of shape starts, she looks down its distance
    /// (`lowers`) at her new size, a block look, and is lowered as far as it got, after what
    /// stops it writes to her as a stop would.
    fn shrink(&mut self, lowers: Lowers, t: &mut Tick) {
        let seen = t.vmove(self.bounds(), t.rules.lowers(lowers), Blocks);
        self.reach(&seen);
        self.position.y += px(seen.gone);
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
    /// drops under gravity. `AnimationEnds`: the unmorph's end hands over the crouch, a pose on
    /// the ground, in the air too: from the next frame she is on the ground, her vertical speed
    /// kept (`FollowingTheGround`).
    fn apply(&mut self, effect: Effect, start: &Self, t: &Tick) {
        let pose = self.pose;
        match effect {
            Rise => self.jump(t),
            Lift if start.pose.aim == Aim::Forward => self.position.y -= t.rules.crouch_lift,
            TurnRound
                if pose.facing != start.pose.facing
                    || matches!(pose.action, Action::Turning(_)) =>
            {
                self.run.way = Side::of(pose.facing.back());
                self.run.join_dash();
            }
            EndSkid => {
                self.run.carry = FollowsPad;
                self.run.way = Side::of(pose.facing);
            }
            AsRunning if start.run.speed == ZERO || self.run.speed == ZERO => {
                self.run.carry = FollowsPad;
            }
            AsRunning => self.run.join_dash(),
            LoseDash => self.run.dash = ZERO,
            Halt => self.run.halt(),
            Effect::Moonfall if t.frame.pressed.any(JUMP) => self.jump(t),
            Effect::Moonfall => {
                self.fall_from_rest();
                self.vertical = Vertical::Moonfall;
            }
            Launch => self.run.launch(pose.facing),
            SpinOn => self.spin = Spin::CarriedOn,
            OnTheGround if !self.vertical.on_ground() => self.vertical = Vertical::Ground,
            Lift | TurnRound | OnTheGround => {}
        }
    }
}
