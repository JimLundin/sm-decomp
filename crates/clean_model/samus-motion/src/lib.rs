//! The samus-motion model, built from its behaviour spec alone.
//!
//! Every rule names the spec entry it comes from. A frame moves her horizontally, then the ground
//! pull or her vertical move, then takes the pose the frame chose.

pub mod catalogue;
mod collide;
pub mod rules;

pub use catalogue::{Behaviour, BehaviourId, CATALOGUE};
pub use rules::{Activity, Limits, Rules, SpeedCap};

use clean_types::{
    Action, Aim, Body, Button, Equipment, Facing, Fixed, Pad, Pose, Position, Side, Signed, World,
    below,
};
use collide::{Axis, Toward, travel};

/// Her horizontal speed: a size and the way it points.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct HSpeed {
    /// The size.
    pub size: Fixed,
    /// The way it points.
    pub way: Side,
}

/// §1.2 carrying on: whether her horizontal motion follows the pad or carries on by itself.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Drive {
    /// It builds while she holds a side, and in the air stops when she lets go.
    FollowPad,
    /// It brakes on the ground and keeps building in the air; against her facing, a skid.
    CarryOn,
}

/// §1.2 her dash: whether it is hers to keep or to lose the next frame she moves.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DashHold {
    /// Hers, through jumps and into the ball.
    Hers,
    /// Left on her by a halt: gone the next frame she moves.
    ToLose,
}

/// Her dash.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Dash {
    /// The size, added the way she moves.
    pub size: Fixed,
    /// Whether she keeps it.
    pub hold: DashHold,
}

/// Her vertical motion.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Vertical {
    /// On the ground.
    Ground,
    /// Rising at this speed.
    Rising(Fixed),
    /// Falling at this speed; 0 is a fall just started.
    Falling(Fixed),
}

/// `Landing`: how many times the ball has bounced since it last rested.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Bounces {
    /// None.
    None,
    /// One.
    Once,
    /// Two.
    Twice,
}

/// §1.2 a spin carried on: how long she has been in this spin.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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

impl Spin {
    const fn advance(self, spinning: bool) -> Self {
        match (spinning, self) {
            (false, _) => Self::Off,
            (true, Self::Off) => Self::First,
            (true, Self::First) => Self::Second,
            (true, Self::Second | Self::CarriedOn) => Self::CarriedOn,
        }
    }
}

/// What one frame brings from the player and the world (§1.3).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Frame {
    /// The buttons held as the frame starts.
    pub pad: Pad,
    /// Jump just pressed.
    pub jump_pressed: bool,
    /// Hi-jump and space jump.
    pub equipment: Equipment,
    /// `WallJump`: the spin's sprite has turned far enough to kick off a wall.
    pub spin_turned_far: bool,
    /// `RidingTheElevator`: the elevator is moving.
    pub elevator_moving: bool,
    /// §7: the pose chosen during the frame, when it changes.
    pub next_pose: Option<Pose>,
}

/// What a frame shows beyond her quantities.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Outcome {
    /// `AimingAndShooting`: the standing animation restarts.
    pub standing_restarts: bool,
}

/// Samus as the player can tell her, and what she remembers.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Samus {
    /// Her body's centre.
    pub position: Position,
    /// Her horizontal speed.
    pub speed_x: HSpeed,
    /// Whether her horizontal motion follows the pad.
    pub drive: Drive,
    /// Her dash.
    pub dash: Dash,
    /// Her vertical motion.
    pub vertical: Vertical,
    /// The ball's bounces.
    pub bounces: Bounces,
    /// Her pose.
    pub pose: Pose,
    /// Her arm's direction, for `AimingAndShooting`.
    pub aim: Aim,
    /// How long she has been in a spin.
    pub spin: Spin,
    /// `Moonfall`: in one, until she lands.
    pub moonfall: bool,
}

/// How a pose moves her horizontally.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Motion {
    /// `SpeedBuildsAndBrakes`: the pad doesn't move her; after her dash's move she halts or not.
    DashOnly { halts: bool },
    /// `Jumping`: taking off.
    TakeOff,
    /// `SpeedBuildsAndBrakes`: the ball at rest on the ground.
    BallRest,
    /// `SpeedBuildsAndBrakes`: builds towards a top speed in this style.
    Build(Activity, Style),
}

/// How a building motion follows the pad.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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

const fn motion(pose: Pose, airborne: bool) -> Motion {
    use Activity as Act;
    match pose.action {
        Action::Still if pose.body.is_ball() => {
            if airborne {
                Motion::Build(Act::Falling, Style::Air)
            } else {
                Motion::BallRest
            }
        }
        Action::Still | Action::AgainstWall => Motion::DashOnly { halts: true },
        Action::Crouching | Action::StandingUp | Action::Morphing | Action::Unmorphing => {
            Motion::DashOnly { halts: false }
        }
        Action::TakingOff => Motion::TakeOff,
        Action::Running => Motion::Build(Act::Running, Style::Ground),
        Action::Rolling if airborne => Motion::Build(Act::Falling, Style::Air),
        Action::Rolling => Motion::Build(Act::Rolling, Style::Ground),
        Action::Moonwalking => Motion::Build(Act::Moonwalking, Style::Ground),
        Action::Turning => Motion::Build(Act::Turning, Style::Ground),
        Action::Jumping => Motion::Build(Act::StraightJump, Style::Air),
        Action::Falling => Motion::Build(Act::Falling, Style::Air),
        Action::Spinning => Motion::Build(Act::SpinJump, Style::Spin),
        Action::WallJumping => Motion::Build(Act::SpinJump, Style::WallJump),
    }
}

/// What the horizontal move left for the rest of the frame.
struct Horizontal {
    /// `WallsAndCeilings`: the speed from before a stop, for the ground pull.
    speed: Fixed,
    /// `WallsAndCeilings`: a wall stopped her.
    walled: bool,
}

impl Samus {
    /// Samus at a vector's start (§1.2): her motion follows the pad unless she is skidding, her
    /// dash is hers, and a vertical speed of 0 means she stands on the ground, except in a jump, a
    /// spin, a fall or a wall jump, where it means she has just started to fall.
    #[must_use]
    pub const fn start(
        position: Position,
        (speed_x, dash, speed_y): (Signed, Fixed, Signed),
        bounces: Bounces,
        pose: Pose,
        aim: Aim,
    ) -> Self {
        let facing = pose.facing.side();
        let moving = !speed_x.size.is_zero();
        let way = match (moving, speed_x.negative) {
            (false, _) => facing,
            (true, true) => Side::Left,
            (true, false) => Side::Right,
        };
        let skidding = moving
            && !matches!(
                (way, facing),
                (Side::Left, Side::Left) | (Side::Right, Side::Right)
            );
        let in_air = matches!(
            pose.action,
            Action::Jumping | Action::Spinning | Action::Falling | Action::WallJumping
        );
        let vertical = match (speed_y.size.is_zero(), speed_y.negative) {
            (true, _) if in_air => Vertical::Falling(Fixed::ZERO),
            (true, _) => Vertical::Ground,
            (false, true) => Vertical::Rising(speed_y.size),
            (false, false) => Vertical::Falling(speed_y.size),
        };
        Self {
            position,
            speed_x: HSpeed {
                size: speed_x.size,
                way,
            },
            drive: if skidding {
                Drive::CarryOn
            } else {
                Drive::FollowPad
            },
            dash: Dash {
                size: dash,
                hold: DashHold::Hers,
            },
            vertical,
            bounces,
            pose,
            aim,
            spin: Spin::Off,
            moonfall: false,
        }
    }

    /// Her horizontal speed as the player reads it, rightward positive.
    #[must_use]
    pub const fn speed_x_signed(&self) -> Signed {
        Signed {
            negative: matches!(self.speed_x.way, Side::Left),
            size: self.speed_x.size,
        }
    }

    /// Her vertical speed as the player reads it, downward positive.
    #[must_use]
    pub const fn speed_y_signed(&self) -> Signed {
        match self.vertical {
            Vertical::Ground => Signed {
                negative: false,
                size: Fixed::ZERO,
            },
            Vertical::Rising(size) => Signed {
                negative: true,
                size,
            },
            Vertical::Falling(size) => Signed {
                negative: false,
                size,
            },
        }
    }

    /// One frame.
    pub fn step(&mut self, frame: &Frame, world: &impl World, rules: &Rules) -> Outcome {
        // MovesAsShePosed: the whole frame moves as the pose at its start says.
        let pose = self.pose;
        let was_grounded = self.vertical == Vertical::Ground;
        self.spin = self.spin.advance(pose.action == Action::Spinning);
        if pose.facing == Facing::Screen {
            // SpeedBuildsAndBrakes: facing the screen she doesn't move and keeps her speeds.
            if frame.elevator_moving {
                // RidingTheElevator: down 1 px a frame.
                self.position.y = self.position.y.plus(rules.elevator);
            }
        } else {
            let moved = self.horizontal(pose, frame, world, rules);
            self.change_aim(pose, frame.pad, moved.walled, world, rules);
            let kicked = self.wall_jump(pose, frame, world, rules);
            match self.vertical {
                Vertical::Ground => self.follow_ground(pose, moved.speed, world, rules),
                // Jumping: taking off, and the wall jump's kick, don't move her vertically.
                Vertical::Rising(_) | Vertical::Falling(_)
                    if kicked || pose.action == Action::TakingOff => {}
                Vertical::Rising(_) | Vertical::Falling(_) => self.air(pose, frame, world, rules),
            }
        }
        // AimingAndShooting: standing, unaimed and not landing, shoot restarts the animation.
        let standing_restarts = pose.body == Body::Upright
            && pose.action == Action::Still
            && was_grounded
            && self.aim == Aim::Forward
            && frame.pad.held(Button::Shoot);
        if let Some(next) = frame.next_pose {
            self.change_pose(next, frame, rules);
        }
        Outcome { standing_restarts }
    }

    /// `SpeedBuildsAndBrakes`, `Dash`, `TurningRound`, `DriftInTheAir`, `WallsAndCeilings`.
    fn horizontal(
        &mut self,
        pose: Pose,
        frame: &Frame,
        world: &impl World,
        rules: &Rules,
    ) -> Horizontal {
        let facing = pose.facing.side();
        let run = frame.pad.held(Button::Run);
        let skidding = self.drive == Drive::CarryOn && self.speed_x.way != facing;
        let mut halts_after = false;
        let distance = match motion(pose, self.vertical != Vertical::Ground) {
            Motion::DashOnly { halts } => {
                // SpeedBuildsAndBrakes: standing, crouched and against the wall halt after it.
                halts_after = halts;
                Some(self.dash.size)
            }
            Motion::TakeOff => {
                // Jumping: she moves by her dash only, follows the pad and still gains dash.
                self.drive = Drive::FollowPad;
                if run {
                    self.grow_dash(rules);
                }
                Some(self.dash.size)
            }
            Motion::BallRest => {
                if self.drive == Drive::CarryOn {
                    // SpeedBuildsAndBrakes: braking or skidding it rolls and keeps its dash.
                    self.brake(rules);
                    Some(self.speed_x.size.plus(self.dash.size))
                } else {
                    // SpeedBuildsAndBrakes: following the pad it moves by its dash, then halts.
                    halts_after = true;
                    Some(self.dash.size)
                }
            }
            Motion::Build(activity, style) => {
                if activity == Activity::Running && run {
                    // Dash: running with run held she gains dash.
                    self.grow_dash(rules);
                } else if self.dash.hold == DashHold::ToLose {
                    // Dash: in any other moving activity a dash she is to lose is gone.
                    self.dash = Dash {
                        size: Fixed::ZERO,
                        hold: DashHold::Hers,
                    };
                }
                let limits = rules.limits(activity);
                self.build_or_brake(
                    limits,
                    style,
                    (facing, skidding),
                    frame.pad,
                    rules,
                    &mut halts_after,
                )
            }
        };
        let speed = self.speed_x.size;
        // §1: the dash taken the way the speed points, her facing when it is 0.
        let heading = if speed.is_zero() {
            facing
        } else {
            self.speed_x.way
        };
        let toward = (Axis::X, Toward::side(heading));
        let walled = distance.is_some_and(|distance| {
            travel(
                &mut self.position,
                pose.radius,
                toward,
                distance,
                world,
                rules,
            )
        });
        if walled || halts_after {
            // WallsAndCeilings, SpeedBuildsAndBrakes: a halt: speed 0, dash 0, following the pad.
            self.halt();
        }
        Horizontal { speed, walled }
    }

    /// `SpeedBuildsAndBrakes` and `DriftInTheAir`: the speed for a building motion, and how far it
    /// moves her (`None`: no move).
    fn build_or_brake(
        &mut self,
        limits: Limits,
        style: Style,
        (facing, skidding): (Side, bool),
        pad: Pad,
        rules: &Rules,
        halts_after: &mut bool,
    ) -> Option<Fixed> {
        let moves = |s: &Self| Some(s.speed_x.size.plus(s.dash.size));
        match style {
            // SpeedBuildsAndBrakes: carrying on along the ground she brakes.
            Style::Ground if self.drive == Drive::CarryOn => self.brake(rules),
            // DriftInTheAir: a skid in the air brakes, as on the ground.
            Style::Air | Style::Spin if skidding => self.brake(rules),
            // DriftInTheAir: following the pad with neither side held she stops drifting.
            Style::Air
                if self.drive == Drive::FollowPad
                    && !pad.holds(Side::Left)
                    && !pad.holds(Side::Right) =>
            {
                if self.pose.body.is_ball() {
                    // DriftInTheAir: the falling ball halts but still moves by its dash.
                    *halts_after = true;
                    return Some(self.dash.size);
                }
                self.speed_x.size = Fixed::ZERO;
                return None;
            }
            // SpeedBuildsAndBrakes: following the pad she builds; DriftInTheAir: carrying on in
            // the air doesn't brake.
            Style::Ground | Style::Air => {
                self.build(limits, facing, rules);
            }
            Style::Spin => {
                // DriftInTheAir: a spin drifts only holding the side she faces or at its top.
                let topped = self.build(limits, facing, rules);
                if !(pad.holds(facing) || topped) {
                    self.speed_x.size = Fixed::ZERO;
                    self.drive = Drive::FollowPad;
                    return None;
                }
                self.drive = Drive::CarryOn;
            }
            Style::WallJump => {
                // DriftInTheAir: through a wall jump she always drifts, along her facing.
                self.build(limits, facing, rules);
                self.drive = Drive::CarryOn;
            }
        }
        moves(self)
    }

    /// `SpeedBuildsAndBrakes`: gains the step up to the top (`WordwiseTopSpeed`), along her
    /// facing. Returns whether she took the top speed.
    fn build(&mut self, limits: Limits, facing: Side, rules: &Rules) -> bool {
        let (size, topped) = rules
            .speed_cap
            .apply(self.speed_x.size.plus(limits.step), limits.top);
        self.speed_x = HSpeed { size, way: facing };
        topped
    }

    /// `SpeedBuildsAndBrakes`: loses the brake; below zero she stops and follows the pad again.
    fn brake(&mut self, rules: &Rules) {
        if self.speed_x.size < rules.brake {
            self.speed_x.size = Fixed::ZERO;
            self.drive = Drive::FollowPad;
        } else {
            self.speed_x.size = self.speed_x.size.minus(rules.brake);
        }
    }

    /// `Dash`: grows if, before the step, its pixel part is below the cap's or its subpixel part
    /// is below the cap's; else it becomes the cap.
    const fn grow_dash(&mut self, rules: &Rules) {
        let (dash, cap) = (self.dash.size, rules.dash_cap);
        let grows = below(dash.pixel(), cap.pixel()) || below(dash.sub(), cap.sub());
        let size = if grows {
            dash.plus(rules.dash_step)
        } else {
            cap
        };
        self.dash = Dash {
            size,
            hold: DashHold::Hers,
        };
    }

    /// A halt: horizontal speed 0, dash 0, following the pad.
    const fn halt(&mut self) {
        self.speed_x.size = Fixed::ZERO;
        self.dash = Dash {
            size: Fixed::ZERO,
            hold: DashHold::Hers,
        };
        self.drive = Drive::FollowPad;
    }

    /// `AimingAndShooting`: changing aim while running moves her 1 px forward, or puts her
    /// against the wall.
    fn change_aim(
        &mut self,
        pose: Pose,
        pad: Pad,
        walled: bool,
        world: &impl World,
        rules: &Rules,
    ) {
        let aim = pad.aim();
        let changed = aim != self.aim;
        self.aim = aim;
        if !(changed && pose.action == Action::Running) {
            return;
        }
        let mut nudged = self.position;
        let toward = (Axis::X, Toward::side(pose.facing.side()));
        if walled
            || travel(
                &mut nudged,
                pose.radius,
                toward,
                rules.aim_nudge,
                world,
                rules,
            )
        {
            self.pose.action = Action::AgainstWall;
        } else {
            self.position = nudged;
        }
    }

    /// `WallJump`: a spin carried on, holding away from a wall near her, readies the wall jump;
    /// with the spin turned far, jump kicks her off. Returns whether she kicked off.
    fn wall_jump(&mut self, pose: Pose, frame: &Frame, world: &impl World, rules: &Rules) -> bool {
        if pose.action != Action::Spinning || self.spin != Spin::CarriedOn {
            return false;
        }
        let wall = if frame.pad.holds(Side::Left) {
            Side::Right
        } else if frame.pad.holds(Side::Right) {
            Side::Left
        } else {
            return false;
        };
        let near = collide::wall_near(
            &mut self.position,
            pose.radius,
            wall,
            rules.wall_reach,
            world,
            rules,
        );
        if !(near && frame.spin_turned_far && frame.jump_pressed) {
            return false;
        }
        let speed = if frame.equipment.hi_jump {
            rules.hi_wall_jump
        } else {
            rules.wall_jump
        };
        self.vertical = Vertical::Rising(speed);
        self.speed_x.size = Fixed::ZERO;
        self.drive = Drive::FollowPad;
        true
    }

    /// `FollowingTheGround`: the pull down after her horizontal move.
    fn follow_ground(&mut self, pose: Pose, speed: Fixed, world: &impl World, rules: &Rules) {
        let pull = if collide::on_slope(self.position, pose.radius, world, rules) {
            // SlopesCarry: on a slope the ground pull is 1 px.
            rules.ground_pull
        } else {
            Fixed::px(speed.pixel()).plus(rules.ground_pull)
        };
        let mut pulled = self.position;
        if travel(
            &mut pulled,
            pose.radius,
            (Axis::Y, Toward::More),
            pull,
            world,
            rules,
        ) {
            // FollowingTheGround: the floor stops her flush.
            self.position = pulled;
        } else if pose.action == Action::Turning {
            // TurningRound: over a drop she sinks instead of falling.
            self.position = pulled;
        } else if !pose.action.changes_shape() {
            // FollowingTheGround: she falls from rest.
            self.vertical = Vertical::Falling(Fixed::ZERO);
            self.bounces = Bounces::None;
            self.start_air_drift(pose.facing.side());
        }
    }

    /// `Jumping`: starting a straight jump or a fall she carries on along her facing if she has a
    /// dash, else she follows the pad.
    const fn start_air_drift(&mut self, facing: Side) {
        if self.dash.size.is_zero() {
            self.drive = Drive::FollowPad;
        } else {
            self.drive = Drive::CarryOn;
            self.speed_x.way = facing;
        }
    }

    /// `RisingAndFalling`, `Jumping`'s space jump, `Landing`, `WallsAndCeilings`'s ceiling.
    fn air(&mut self, pose: Pose, frame: &Frame, world: &impl World, rules: &Rules) {
        let jump = if frame.equipment.hi_jump {
            rules.hi_jump
        } else {
            rules.jump
        };
        let held = frame.pad.held(Button::Jump);
        if let Vertical::Falling(speed) = self.vertical {
            let (from, to) = rules.space_jump_window;
            let read = speed.as_8_8();
            if pose.action == Action::Spinning
                && frame.jump_pressed
                && frame.equipment.space_jump
                && !self.moonfall
                && (from..to).contains(&read)
            {
                // Jumping: space jump; she rises at the new speed that same frame.
                self.vertical = Vertical::Rising(jump);
            }
        }
        // RisingAndFalling: a jump, a spin or the hop ends its rise as soon as jump is let go.
        let lets_go =
            matches!(pose.action, Action::Jumping | Action::Spinning) && !held && !self.moonfall;
        match self.vertical {
            Vertical::Rising(_) if lets_go => self.vertical = Vertical::Falling(Fixed::ZERO),
            Vertical::Rising(speed) => {
                if travel(
                    &mut self.position,
                    pose.radius,
                    (Axis::Y, Toward::Less),
                    speed,
                    world,
                    rules,
                ) {
                    // WallsAndCeilings: a ceiling stops her flush and ends the rise.
                    self.vertical = Vertical::Falling(Fixed::ZERO);
                }
            }
            Vertical::Falling(speed) => {
                if travel(
                    &mut self.position,
                    pose.radius,
                    (Axis::Y, Toward::More),
                    speed,
                    world,
                    rules,
                ) {
                    self.land(pose, speed, held.then_some(jump), rules);
                    return;
                }
            }
            Vertical::Ground => return,
        }
        self.vertical = match self.vertical {
            // RisingAndFalling: once the upward speed is spent she starts to fall.
            Vertical::Rising(speed) if speed > rules.gravity => {
                Vertical::Rising(speed.minus(rules.gravity))
            }
            Vertical::Rising(_) => Vertical::Falling(Fixed::ZERO),
            // RisingAndFalling: a fall stops growing only at pixel part 5; Moonfall: no limit.
            Vertical::Falling(speed) if speed.pixel() == rules.fall_settle_px && !self.moonfall => {
                Vertical::Falling(speed)
            }
            Vertical::Falling(speed) => Vertical::Falling(speed.plus(rules.gravity)),
            Vertical::Ground => Vertical::Ground,
        };
    }

    /// `Landing`: falling onto a floor at `speed`; `hop` is the jump speed when jump is held.
    fn land(&mut self, pose: Pose, speed: Fixed, hop: Option<Fixed>, rules: &Rules) {
        // Moonfall: until she lands.
        self.moonfall = false;
        if pose.action.changes_shape() {
            // Landing: while her body changes shape it stops only her vertical motion.
            self.vertical = Vertical::Ground;
            return;
        }
        if pose.body.is_ball() {
            if let Some(jump) = hop.filter(|_| pose.body == Body::SpringBall) {
                // Landing: the spring ball with jump held hops again.
                self.vertical = Vertical::Rising(jump);
                self.bounces = Bounces::None;
                return;
            }
            let bounce = if speed.pixel() < rules.bounce_from_px {
                None
            } else {
                match self.bounces {
                    Bounces::None => Some((rules.bounce, Bounces::Once)),
                    Bounces::Once => {
                        Some((rules.bounce.minus(rules.second_bounce_less), Bounces::Twice))
                    }
                    Bounces::Twice => None,
                }
            };
            if let Some((rise, bounces)) = bounce {
                // Landing: the ball bounces twice before it rests.
                self.vertical = Vertical::Rising(rise);
                self.bounces = bounces;
                return;
            }
        }
        // Landing: vertical speed 0, bounces 0, horizontal speed 0, following the pad; the dash
        // keeps its value, to lose (§1.2).
        self.vertical = Vertical::Ground;
        self.bounces = Bounces::None;
        self.speed_x.size = Fixed::ZERO;
        self.drive = Drive::FollowPad;
        self.dash.hold = DashHold::ToLose;
    }

    /// §7: the pose the frame chose, and what the change does to what she remembers.
    fn change_pose(&mut self, next: Pose, frame: &Frame, rules: &Rules) {
        let prev = self.pose;
        let turned = prev.facing.side() != next.facing.side();
        let reversed_spin = prev.action == Action::Spinning
            && next.action == Action::Spinning
            && self.spin == Spin::CarriedOn;
        if turned && (next.action == Action::Turning || reversed_spin) {
            // TurningRound: she adds her dash to her speed and keeps travelling the old way.
            if self.speed_x.size.is_zero() {
                self.speed_x.way = prev.facing.side();
            }
            self.speed_x.size = self.speed_x.size.plus(self.dash.size);
            self.dash = Dash {
                size: Fixed::ZERO,
                hold: DashHold::Hers,
            };
            self.drive = Drive::CarryOn;
        }
        if prev.action == Action::Turning && next.action != Action::Turning {
            // TurningRound, Dash: when the turn ends she loses her dash.
            self.dash = Dash {
                size: Fixed::ZERO,
                hold: DashHold::Hers,
            };
            if next.action == Action::Spinning && !frame.jump_pressed {
                // Moonfall: the spin starts with no rise.
                self.moonfall = true;
                if self.vertical == Vertical::Ground {
                    self.vertical = Vertical::Falling(Fixed::ZERO);
                }
            }
        }
        let jump = if frame.equipment.hi_jump {
            rules.hi_jump
        } else {
            rules.jump
        };
        let grounded = self.vertical == Vertical::Ground;
        let starts = prev.action != next.action;
        match next.action {
            Action::Spinning if starts && prev.action == Action::WallJumping => {
                // §1.2: a spin that followed a wall jump is carried on.
                self.spin = Spin::CarriedOn;
            }
            Action::Spinning if starts && grounded && !self.moonfall => {
                // Jumping: starting a spin jump she rises at the jump speed.
                self.vertical = Vertical::Rising(jump);
            }
            Action::Jumping if starts && next.body == Body::SpringBall && grounded => {
                // Jumping: the spring ball's hop from the ground.
                self.vertical = Vertical::Rising(jump);
            }
            Action::TakingOff if starts => {
                // Jumping: on the first take-off pose she starts rising; from a crouch without
                // aim she is lifted.
                if grounded {
                    self.vertical = Vertical::Rising(jump);
                }
                if prev.body == Body::Crouched && self.aim == Aim::Forward {
                    self.position.y = self.position.y.minus(rules.crouch_lift);
                }
                self.drive = Drive::FollowPad;
            }
            Action::Jumping | Action::Falling if starts => self.start_air_drift(next.facing.side()),
            Action::AgainstWall
            | Action::Crouching
            | Action::Falling
            | Action::Jumping
            | Action::Moonwalking
            | Action::Morphing
            | Action::Rolling
            | Action::Running
            | Action::Spinning
            | Action::StandingUp
            | Action::Still
            | Action::TakingOff
            | Action::Turning
            | Action::Unmorphing
            | Action::WallJumping => {}
        }
        self.pose = next;
    }
}
