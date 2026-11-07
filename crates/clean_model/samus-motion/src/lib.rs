//! The samus-motion model, built from its behaviour spec alone.
//!
//! Every rule names the spec entry it comes from. A frame moves her horizontally, then the ground
//! pull or her vertical move, then the aim's step, then takes the pose the frame chose.

pub mod catalogue;
mod collide;
pub mod rules;

pub use catalogue::{Behaviour, BehaviourId, CATALOGUE};
pub use rules::{Activity, JumpKind, Limits, Rules, SpeedCap};

use clean_types::{
    Action, Aim, Axis, Body, Button, Equipment, Facing, Fixed, Pad, Parity, Pose, Position, Radius,
    Side, Signed, World, below,
};
use collide::{Moved, Room, Stop, Toward, travel};

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
    /// Falling at this speed; 0 is a fall just started.
    Falling(Fixed),
    /// On the ground.
    Ground,
    /// Rising at this upward speed; at 0 she rises by 0 for one more frame.
    Rising(Fixed),
    /// `RisingAndFalling`: the pull took the upward speed past zero, so it points downward by
    /// this; the rise ends at the start of the next frame.
    Spent(Fixed),
}

/// `SlopesCarry`: whether she is on a slope.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Footing {
    /// Not on a slope.
    Free,
    /// Seated on a slope, until her next vertical move starts.
    Slope,
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
    /// `BlocksStopFlush`: the frame count's parity.
    pub parity: Parity,
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
    /// How long she has been in a spin.
    pub spin: Spin,
    /// `Moonfall`: in one, until she lands.
    pub moonfall: bool,
    /// `SlopesCarry`: whether she is on a slope.
    pub footing: Footing,
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

/// How a frame's horizontal step moves her.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Move {
    /// By her dash alone.
    Dash,
    /// By her dash alone, then she halts.
    DashThenHalt,
    /// By her speed plus her dash.
    SpeedAndDash,
    /// Not at all.
    Stay,
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
            spin: Spin::Off,
            moonfall: false,
            footing: Footing::Free,
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
            Vertical::Falling(size) | Vertical::Spent(size) => Signed {
                negative: false,
                size,
            },
        }
    }

    /// One frame.
    pub fn step(&mut self, frame: &Frame, world: &impl World, rules: &Rules) -> Outcome {
        let room = Room {
            world,
            parity: frame.parity,
        };
        // MovesAsShePosed: the whole frame moves as the pose at its start says.
        let pose = self.pose;
        let was_grounded = self.vertical == Vertical::Ground;
        self.spin = self.spin.advance(pose.action == Action::Spinning);
        let mut against_wall = false;
        if pose.facing == Facing::Screen {
            // SpeedBuildsAndBrakes: facing the screen she doesn't move and keeps her speeds.
            if frame.elevator_moving {
                self.move_down_with_elevator(pose.radius, &room, rules);
            }
        } else {
            let moved = self.move_horizontally(pose, frame, &room, rules);
            let near = self.move_by_wall_test(pose, frame.pad, &room, rules);
            let kicked = near && self.kick_off_wall(frame, rules);
            match self.vertical {
                Vertical::Ground => self.move_by_ground_pull(pose, moved.speed, &room, rules),
                // Jumping: taking off, and the wall jump's kick, don't move her vertically.
                Vertical::Falling(_) | Vertical::Rising(_) | Vertical::Spent(_)
                    if kicked || pose.action == Action::TakingOff => {}
                Vertical::Falling(_) | Vertical::Rising(_) | Vertical::Spent(_) => {
                    self.move_in_air(pose, frame, &room, rules);
                }
            }
            if let Some(next) = frame.next_pose {
                // AimingAndShooting: the step comes after that frame's moves and the ground pull.
                against_wall = self.step_forward_on_aim(pose, next, moved.walled, &room, rules);
            }
        }
        // AimingAndShooting: standing, unaimed and not landing, shoot restarts the animation.
        let standing_restarts = pose.body == Body::Upright
            && pose.action == Action::Still
            && was_grounded
            && pose.aim == Aim::Forward
            && frame.pad.held(Button::Shoot);
        if let Some(next) = frame.next_pose {
            // AimingAndShooting: a step into the way puts her against the wall instead.
            let action = if against_wall {
                Action::AgainstWall
            } else {
                next.action
            };
            // Jumping: changing into the take-off from a crouch without aim lifts her.
            if action == Action::TakingOff
                && pose.action != Action::TakingOff
                && pose.body == Body::Crouched
                && pose.aim == Aim::Forward
            {
                self.move_up_by_crouch_lift(rules);
            }
            self.change_pose(Pose { action, ..next }, frame, rules);
        }
        Outcome { standing_restarts }
    }

    /// `SpeedBuildsAndBrakes`, `Dash`, `TurningRound`, `DriftInTheAir`, `WallsAndCeilings`,
    /// `SlopesCarry`.
    fn move_horizontally(
        &mut self,
        pose: Pose,
        frame: &Frame,
        room: &Room<impl World>,
        rules: &Rules,
    ) -> Horizontal {
        let facing = pose.facing.side();
        let run = frame.pad.held(Button::Run);
        let how = match motion(pose, self.vertical != Vertical::Ground) {
            Motion::BallRest if self.drive == Drive::CarryOn => {
                // SpeedBuildsAndBrakes: braking or skidding it rolls and keeps its dash.
                self.brake(rules);
                Move::SpeedAndDash
            }
            // SpeedBuildsAndBrakes: standing, crouched and against the wall halt after it; the
            // ball at rest following the pad moves by its dash, then halts.
            Motion::DashOnly { halts: true } | Motion::BallRest => Move::DashThenHalt,
            // SpeedBuildsAndBrakes: while her body changes shape she keeps both speeds.
            Motion::DashOnly { halts: false } => Move::Dash,
            Motion::TakeOff => {
                // Jumping: she moves by her dash only, follows the pad and still gains dash.
                self.drive = Drive::FollowPad;
                if run {
                    self.grow_dash(rules);
                }
                Move::Dash
            }
            Motion::Build(activity, style) => {
                if activity == Activity::Running && run {
                    // Dash: running with run held she gains dash.
                    self.grow_dash(rules);
                } else if self.dash.hold == DashHold::ToLose {
                    // Dash: in any other moving activity a dash she is to lose is gone.
                    self.lose_dash();
                }
                self.build_or_brake(rules.limits(activity), style, pose, frame.pad, rules)
            }
        };
        let speed = self.speed_x.size;
        let distance = match how {
            Move::Dash | Move::DashThenHalt => Some(self.dash.size),
            Move::SpeedAndDash => Some(speed.plus(self.dash.size)),
            Move::Stay => None,
        };
        // §1: the dash taken the way the speed points, her facing when it is 0.
        let heading = if speed.is_zero() {
            facing
        } else {
            self.speed_x.way
        };
        let walled = distance.is_some_and(|d| self.move_x(pose.radius, heading, d, room, rules));
        if walled || how == Move::DashThenHalt {
            // WallsAndCeilings, SpeedBuildsAndBrakes: a halt.
            self.halt();
        } else if pose.action == Action::Turning
            && frame.pad.none_held()
            && self.drive == Drive::CarryOn
        {
            // TurningRound: with no button held at all her skid ends after the move: her speed
            // now follows the pad and points along her new facing. Holding any button she keeps
            // skidding until braking would take her below zero.
            self.drive = Drive::FollowPad;
            self.speed_x.way = facing;
        }
        Horizontal { speed, walled }
    }

    /// Every move ends here: she takes the position it leaves her at.
    const fn move_to(&mut self, to: Position) {
        self.position = to;
    }

    /// A horizontal move towards `side`, then `SlopesCarry`'s two checks, which run after every
    /// horizontal move whether or not it met solid enemies. Returns whether the move was stopped.
    fn move_x(
        &mut self,
        radius: Radius,
        side: Side,
        distance: Fixed,
        room: &Room<impl World>,
        rules: &Rules,
    ) -> bool {
        let way = (Axis::X, Toward::side(side));
        let moved = travel(self.position, radius, way, distance, room, rules);
        self.move_to(moved.to);
        if let Some(seated) = collide::seat(self.position, radius, room.world, rules) {
            // SlopesCarry: the lift or the push seats her.
            self.move_to(seated);
            self.footing = Footing::Slope;
        }
        moved.stop.is_some()
    }

    /// Where a vertical move would leave her. `SlopesCarry`: as it starts she leaves any slope.
    fn test_y(
        &mut self,
        radius: Radius,
        toward: Toward,
        distance: Fixed,
        room: &Room<impl World>,
        rules: &Rules,
    ) -> Moved {
        self.footing = Footing::Free;
        travel(
            self.position,
            radius,
            (Axis::Y, toward),
            distance,
            room,
            rules,
        )
    }

    /// `RidingTheElevator`: facing the screen on a moving elevator she moves down 1 px, a move a
    /// solid or stepped block stops as any downward move does. Nothing else of hers changes.
    fn move_down_with_elevator(&mut self, radius: Radius, room: &Room<impl World>, rules: &Rules) {
        let way = (Axis::Y, Toward::More);
        let moved = travel(self.position, radius, way, rules.elevator, room, rules);
        self.move_to(moved.to);
    }

    /// Takes a downward move. `SlopesCarry`: one stopped by a stepped block seats her; a sloped
    /// floor's stop doesn't.
    const fn move_down_to(&mut self, moved: Moved) -> Option<Stop> {
        self.move_to(moved.to);
        if matches!(moved.stop, Some(Stop::Stepped)) {
            self.footing = Footing::Slope;
        }
        moved.stop
    }

    /// `SpeedBuildsAndBrakes` and `DriftInTheAir`: builds or brakes the speed of a building
    /// motion, and says how she moves.
    fn build_or_brake(
        &mut self,
        limits: Limits,
        style: Style,
        pose: Pose,
        pad: Pad,
        rules: &Rules,
    ) -> Move {
        let facing = pose.facing.side();
        let skidding = self.drive == Drive::CarryOn && self.speed_x.way != facing;
        match style {
            // SpeedBuildsAndBrakes: carrying on along the ground she brakes.
            Style::Ground if self.drive == Drive::CarryOn => self.brake(rules),
            // DriftInTheAir: a skid in the air brakes, as on the ground; a spin's skid moves.
            Style::Air | Style::Spin if skidding => self.brake(rules),
            // DriftInTheAir: following the pad with neither side held she stops drifting; the
            // falling ball halts but still moves by its dash.
            Style::Air if self.drive == Drive::FollowPad && pad.steers_neither_way() => {
                if pose.body.is_ball() {
                    return Move::DashThenHalt;
                }
                self.speed_x.size = Fixed::ZERO;
                return Move::Stay;
            }
            // SpeedBuildsAndBrakes: following the pad she builds; DriftInTheAir: carrying on in
            // the air doesn't brake.
            Style::Ground | Style::Air => {
                self.build(limits, facing, rules);
            }
            Style::Spin => {
                // DriftInTheAir: a spin drifts only holding the side she faces or at its top;
                // otherwise she stops drifting and moves by her dash alone.
                let topped = self.build(limits, facing, rules);
                if !(pad.holds(facing) || topped) {
                    self.speed_x.size = Fixed::ZERO;
                    self.drive = Drive::FollowPad;
                    return Move::Dash;
                }
                self.drive = Drive::CarryOn;
            }
            Style::WallJump => {
                // DriftInTheAir: through a wall jump she always drifts, along her facing.
                self.build(limits, facing, rules);
                self.drive = Drive::CarryOn;
            }
        }
        Move::SpeedAndDash
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

    /// `Dash`: she loses her dash (0).
    const fn lose_dash(&mut self) {
        self.dash = Dash {
            size: Fixed::ZERO,
            hold: DashHold::Hers,
        };
    }

    /// A halt: horizontal speed 0, dash 0, following the pad.
    const fn halt(&mut self) {
        self.speed_x.size = Fixed::ZERO;
        self.lose_dash();
        self.drive = Drive::FollowPad;
    }

    /// `AimingAndShooting`: changing aim while running steps her 1 px forward, a step a block
    /// stops as any horizontal move, with no `SlopesCarry` checks after it; a solid enemy within
    /// its reach stops it with no move at all, one she touches still clearing her y subpixel.
    /// Returns whether a block or a solid enemy in the way, or a wall stop, puts her against the
    /// wall instead.
    fn step_forward_on_aim(
        &mut self,
        pose: Pose,
        next: Pose,
        walled: bool,
        room: &Room<impl World>,
        rules: &Rules,
    ) -> bool {
        if next.aim == pose.aim || pose.action != Action::Running {
            return false;
        }
        let way = (Axis::X, Toward::side(pose.facing.side()));
        let step = travel(self.position, pose.radius, way, rules.aim_step, room, rules);
        let to = match step.stop {
            Some(Stop::Enemy) => Position {
                x: self.position.x,
                ..step.to
            },
            Some(Stop::Block | Stop::Stepped | Stop::Slope) | None => step.to,
        };
        self.move_to(to);
        walled || step.stop.is_some()
    }

    /// `WallJump`: in a spin carried on, holding away from a wall, the test for a wall near her,
    /// which can write her x or y subpixel. Returns whether a wall is near, readying the wall jump.
    fn move_by_wall_test(
        &mut self,
        pose: Pose,
        pad: Pad,
        room: &Room<impl World>,
        rules: &Rules,
    ) -> bool {
        if pose.action != Action::Spinning || self.spin != Spin::CarriedOn {
            return false;
        }
        let wall = if pad.holds(Side::Left) {
            Side::Right
        } else if pad.holds(Side::Right) {
            Side::Left
        } else {
            return false;
        };
        let reach = rules.wall_reach;
        let test = collide::wall_test(self.position, pose.radius, wall, reach, room, rules);
        self.move_to(test.to);
        test.stop.is_some()
    }

    /// `WallJump`: with a wall near her and the spin turned far, pressing jump kicks her off;
    /// `Moonfall`: in one, pressing jump does nothing. Returns whether she kicked off.
    const fn kick_off_wall(&mut self, frame: &Frame, rules: &Rules) -> bool {
        if !(frame.spin_turned_far && frame.jump_pressed) || self.moonfall {
            return false;
        }
        let speed = rules.jump_speed(JumpKind::WallJump, frame.equipment.hi_jump);
        self.vertical = Vertical::Rising(speed);
        self.speed_x.size = Fixed::ZERO;
        self.drive = Drive::FollowPad;
        true
    }

    /// `FollowingTheGround`: the pull down after her horizontal move.
    fn move_by_ground_pull(
        &mut self,
        pose: Pose,
        speed: Fixed,
        room: &Room<impl World>,
        rules: &Rules,
    ) {
        let pull = match self.footing {
            // SlopesCarry: on a slope the ground pull is 1 px.
            Footing::Slope => rules.ground_pull,
            Footing::Free => Fixed::px(speed.pixel()).plus(rules.ground_pull),
        };
        let pulled = self.test_y(pose.radius, Toward::More, pull, room, rules);
        if pulled.stop.is_some() || pose.action == Action::Turning {
            // FollowingTheGround: the floor stops her flush; TurningRound: over a drop she sinks
            // instead of falling.
            self.move_down_to(pulled);
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

    /// `RisingAndFalling`: a fall's speed after the pull, which stops growing only at pixel part
    /// 5; `Moonfall`: with no limit.
    const fn fall_pull(&self, speed: Fixed, rules: &Rules) -> Vertical {
        if speed.pixel() == rules.fall_settle_px && !self.moonfall {
            Vertical::Falling(speed)
        } else {
            Vertical::Falling(speed.plus(rules.gravity))
        }
    }

    /// `RisingAndFalling`, `Jumping`'s space jump, `Landing`, `WallsAndCeilings`'s ceiling.
    fn move_in_air(&mut self, pose: Pose, frame: &Frame, room: &Room<impl World>, rules: &Rules) {
        let jump = rules.jump_speed(JumpKind::Jump, frame.equipment.hi_jump);
        let held = frame.pad.held(Button::Jump);
        if let Vertical::Falling(speed) = self.vertical
            && pose.action == Action::Spinning
            && frame.jump_pressed
            && frame.equipment.space_jump
            && !self.moonfall
            && (rules.space_jump_window.0..rules.space_jump_window.1).contains(&speed.as_8_8())
        {
            // Jumping: space jump; she rises at the new speed that same frame.
            self.vertical = Vertical::Rising(jump);
        }
        // RisingAndFalling: a normal or spin jump or the hop ends its rise at the start of any
        // frame with jump not held.
        let lets_go =
            matches!(pose.action, Action::Jumping | Action::Spinning) && !held && !self.moonfall;
        match self.vertical {
            // RisingAndFalling: the rise ends at the start of the frame after it is spent, or
            // when let go: falling from 0 with no vertical move; the pull acts as in a fall.
            Vertical::Spent(_) => self.vertical = self.fall_pull(Fixed::ZERO, rules),
            Vertical::Rising(_) if lets_go => self.vertical = self.fall_pull(Fixed::ZERO, rules),
            Vertical::Rising(speed) => {
                let rise = self.test_y(pose.radius, Toward::Less, speed, room, rules);
                self.move_to(rise.to);
                self.vertical = if rise.stop.is_some() {
                    // WallsAndCeilings, SlopesCarry: a ceiling ends the rise, with no pull.
                    Vertical::Falling(Fixed::ZERO)
                } else if speed < rules.gravity {
                    // RisingAndFalling: the pull took it past zero: spent.
                    Vertical::Spent(rules.gravity.minus(speed))
                } else {
                    // RisingAndFalling: the pull takes 0.1C00 off; exactly 0 is not spent.
                    Vertical::Rising(speed.minus(rules.gravity))
                };
            }
            Vertical::Falling(speed) => {
                let fall = self.test_y(pose.radius, Toward::More, speed, room, rules);
                if self.move_down_to(fall).is_some() {
                    self.land(pose, speed, held.then_some(jump), rules);
                } else {
                    self.vertical = self.fall_pull(speed, rules);
                }
            }
            Vertical::Ground => {}
        }
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

    /// `Jumping`: changing into the take-off from a crouch without aim, she is lifted 10 px.
    const fn move_up_by_crouch_lift(&mut self, rules: &Rules) {
        self.move_to(Position {
            y: self.position.y.minus(rules.crouch_lift),
            ..self.position
        });
    }

    /// §7: the pose the frame chose, and what the change does to what she remembers.
    fn change_pose(&mut self, next: Pose, frame: &Frame, rules: &Rules) {
        let prev = self.pose;
        let turned = prev.facing.side() != next.facing.side();
        let reversed_spin = prev.action == Action::Spinning
            && next.action == Action::Spinning
            && self.spin == Spin::CarriedOn;
        let ball_turns = prev.body.is_ball() && next.body.is_ball();
        if turned && (next.action == Action::Turning || reversed_spin || ball_turns) {
            // TurningRound: she adds her dash to her speed and keeps travelling the old way.
            if self.speed_x.size.is_zero() {
                self.speed_x.way = prev.facing.side();
            }
            self.speed_x.size = self.speed_x.size.plus(self.dash.size);
            self.lose_dash();
            self.drive = Drive::CarryOn;
        }
        if prev.action == Action::Turning && next.action != Action::Turning {
            // TurningRound, Dash: when the turn ends she loses her dash.
            self.lose_dash();
            if next.action == Action::Spinning && !frame.jump_pressed {
                // Moonfall: the spin starts with no rise.
                self.moonfall = true;
                if self.vertical == Vertical::Ground {
                    self.vertical = Vertical::Falling(Fixed::ZERO);
                }
            }
        }
        let jump = rules.jump_speed(JumpKind::Jump, frame.equipment.hi_jump);
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
                // Jumping: on the first take-off pose she starts rising.
                if grounded {
                    self.vertical = Vertical::Rising(jump);
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
