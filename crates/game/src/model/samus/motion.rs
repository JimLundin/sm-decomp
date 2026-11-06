//! Samus's motion: how her speeds change and carry her, on the ground and in the air.
//!
//! Each way of moving is a row of the motion table: what moves her sideways, what moves her
//! up or down, and what happens after. The speeds come from `data`, the one switch from
//! [`SpeedCap`], and every move goes through [`Collision`]. A frame runs three entry points in
//! order: [`State::step`]; [`State::arm_pump_step`] once pose selection has settled its
//! request; and [`State::pose_writes`] when the new pose starts.

mod pose;

pub use pose::{PoseChange, PoseCommand, PoseStart};

use crate::model::collision::{Body, Collision, Enemies, Heading};
use crate::model::num::Fixed;
use crate::session::SpeedCap;
use data::movement::{AirMotion, GroundMotion, HorizontalSpeeds, SpeedEntry, VerticalSpeeds};
use data::pose::MovementType;

/// Where Samus's centre is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Position {
    /// Rightward.
    pub x: Fixed,
    /// Downward.
    pub y: Fixed,
}

/// Half her size each way, in pixels, as her pose gives it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Radius {
    /// Half her width.
    pub x: u16,
    /// Half her height.
    pub y: u16,
}

/// What the base speed is doing; pose selection sets it when a pose starts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SpeedMode {
    /// Speeding up to the row's top speed.
    #[default]
    Accelerating,
    /// Braking through a turn, still sliding the old way.
    TurningAround,
    /// Braking (on the ground; in the air it still speeds up).
    Decelerating,
}

/// Which way she moves vertically; anything but `None` is airborne.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum VerticalMotion {
    /// On the ground.
    #[default]
    None,
    /// Rising.
    Up,
    /// Falling.
    Down,
}

/// How many times the ball has bounced since it landed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Bounce {
    /// Not bouncing.
    #[default]
    None,
    /// In the first bounce.
    First,
    /// In the second.
    Second,
}

/// Samus's motion State: the quantities a later frame reads, one set for every way of moving.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct State {
    /// Her centre.
    pub position: Position,
    /// The row's own speed, never negative.
    pub base_speed: Fixed,
    /// The speed running adds, never negative.
    pub extra_run: Fixed,
    /// Whether the run speed carries into rows that don't run.
    pub momentum: bool,
    /// What the base speed is doing.
    pub mode: SpeedMode,
    /// The vertical speed's magnitude.
    pub vertical_speed: Fixed,
    /// Its direction.
    pub vertical: VerticalMotion,
    /// The ball's bounces.
    pub bounce: Bounce,
}

/// The side her pose faces.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Facing {
    /// Left.
    Left,
    /// Right.
    Right,
    /// The screen; wherever a side is picked, it counts as right.
    Forward,
}

/// The way she moves this frame, from her pose at the frame's start (`LocomotionAtFrameStart`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Locomotion {
    /// Standing, facing the screen.
    FacingForward,
    /// Standing; `plain` without aim and not landing.
    Standing {
        /// One of the plain standing poses.
        plain: bool,
    },
    /// Running, at any aim.
    Running,
    /// The morph or spring ball on the ground.
    Ball {
        /// The spring ball.
        spring: bool,
        /// The still pose, not a rolling one.
        at_rest: bool,
    },
    /// Crouching.
    Crouching,
    /// The on-ground turn.
    TurningOnGround,
    /// A crouch, stand, morph or unmorph transition.
    Transition,
    /// Moonwalking.
    Moonwalking,
    /// Ran into a wall.
    RanIntoWall,
    /// The normal jump; `take_off` in its first poses.
    Jumping {
        /// One of the take-off poses.
        take_off: bool,
    },
    /// The spin, space and screw-attack jumps.
    SpinJumping,
    /// Falling.
    Falling,
    /// The ball in the air; the morph ball is always `falling`.
    BallInAir {
        /// The spring ball.
        spring: bool,
        /// One of the falling poses.
        falling: bool,
    },
    /// The wall jump.
    WallJumping,
    /// Turning while jumping or falling.
    TurningInAir,
}

/// Which falling pose a fall starts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FallKind {
    /// The plain fall.
    Airborne,
    /// The morph ball's.
    MorphBall,
    /// The spring ball's.
    SpringBall,
    /// No pose change.
    NoChange,
}

/// Which pose a landing starts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LandKind {
    /// On her feet.
    Grounded,
    /// The morph ball's.
    MorphBall,
    /// The spring ball's.
    SpringBall,
    /// No pose change.
    NoChange,
}

impl Locomotion {
    // SpeedTable and LandingContact: whose speeds the row reads, and which fall and landing it
    // starts (spec §2).
    const fn kinds(self) -> (MovementType, FallKind, LandKind) {
        use {FallKind as F, LandKind as L, MovementType as M};
        match self {
            Self::FacingForward | Self::Standing { .. } => (M::Standing, F::Airborne, L::NoChange),
            Self::Running => (M::Running, F::Airborne, L::NoChange),
            Self::Ball { spring: false, .. } => (M::MorphBallOnGround, F::MorphBall, L::NoChange),
            Self::Ball { spring: true, .. } => (M::SpringBallOnGround, F::SpringBall, L::NoChange),
            Self::Crouching => (M::Crouching, F::Airborne, L::NoChange),
            Self::TurningOnGround => (M::TurningAroundOnGround, F::NoChange, L::NoChange),
            Self::Transition => (M::CrouchingTransition, F::NoChange, L::NoChange),
            Self::Moonwalking => (M::Moonwalking, F::Airborne, L::NoChange),
            Self::RanIntoWall => (M::RanIntoWall, F::Airborne, L::NoChange),
            Self::Jumping { .. } => (M::NormalJumping, F::NoChange, L::Grounded),
            Self::SpinJumping => (M::SpinJumping, F::NoChange, L::Grounded),
            Self::Falling => (M::Falling, F::NoChange, L::Grounded),
            Self::BallInAir { spring: false, .. } => {
                (M::MorphBallFalling, F::NoChange, L::MorphBall)
            }
            Self::BallInAir { falling: true, .. } => {
                (M::SpringBallFalling, F::NoChange, L::SpringBall)
            }
            Self::BallInAir { falling: false, .. } => {
                (M::SpringBallInAir, F::NoChange, L::SpringBall)
            }
            Self::WallJumping => (M::WallJumping, F::NoChange, L::Grounded),
            Self::TurningInAir => (M::TurningAroundJumping, F::NoChange, L::NoChange),
        }
    }
}

/// The vertical result of the frame.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum VerticalResult {
    /// Nothing to report.
    #[default]
    None,
    /// She landed.
    Landed(LandKind),
    /// The ground is gone; the fall to start.
    Falling(FallKind),
    /// Her head hit the ceiling.
    HitCeiling,
    /// The wall jump fired.
    WallJump,
}

/// The jump button.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum JumpButton {
    /// Not held.
    #[default]
    Released,
    /// Held since an earlier frame.
    Held,
    /// Pressed this frame.
    Pressed,
}

/// Left and right on the pad; both may be held.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Sideways {
    /// Left is held.
    pub left: bool,
    /// Right is held.
    pub right: bool,
}

/// The buttons motion reads.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Pad {
    /// Run is held.
    pub run: bool,
    /// Shoot is held.
    pub shoot: bool,
    /// Jump.
    pub jump: JumpButton,
    /// Left and right.
    pub sideways: Sideways,
}

/// Where the spin jump's animation and pose history stand, as the wall jump reads them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Spin {
    /// The spin animation is at its wall-jump frames.
    pub ready: bool,
    /// Two pose updates back she was spin jumping or wall jumping.
    pub was_spinning: bool,
}

/// The items motion reads.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Equipment {
    /// The hi-jump boots.
    pub hi_jump: bool,
    /// Space jump.
    pub space_jump: bool,
}

/// What other domains give motion this frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Inputs {
    /// Her way of moving at the frame's start.
    pub locomotion: Locomotion,
    /// Her pose's facing at the frame's start.
    pub facing: Facing,
    /// Her pose's radius.
    pub radius: Radius,
    /// The buttons.
    pub pad: Pad,
    /// The spin jump's state.
    pub spin: Spin,
    /// Her items.
    pub equipment: Equipment,
    /// The elevator under her is moving.
    pub elevator_moving: bool,
    /// Collision seated her on a slope.
    pub on_slope: bool,
}

/// What pose selection settled on, as the arm-pump step reads it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PoseRequest {
    /// A forced pose; the step does nothing.
    Forced,
    /// No new pose.
    None,
    /// A new pose, `running` or not.
    Pose {
        /// A running pose.
        running: bool,
    },
}

/// What the frame hands other domains.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Contacts {
    /// The horizontal step was stopped (`WallStop`).
    pub wall_stop: bool,
    /// The vertical result, after the row's After.
    pub vertical: VerticalResult,
    /// The wall jump is armed (`WallJumpCheck`).
    pub wall_jump_armed: bool,
    /// The standing animation restarts (`StandingShotRestartsAnimation`).
    pub restart_standing_animation: bool,
}

/// The values motion reads: the speed tables, the declared distances and the speed cap.
#[derive(Clone, Copy, Debug)]
pub struct MotionRules<'a> {
    /// The horizontal speed tables.
    pub speeds: &'a HorizontalSpeeds,
    /// Gravity and the jumps' speeds.
    pub vertical: VerticalSpeeds,
    /// The ground's distances.
    pub ground: GroundMotion,
    /// The air's values.
    pub air: AirMotion,
    /// The `SpeedCap` switch.
    pub speed_cap: SpeedCap,
}

/// The speed table has no entry for this movement type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MissingSpeeds(pub MovementType);

#[derive(Clone, Copy)]
enum Horizontal {
    Ground,
    Coast,
    None,
    TakeOff,
    Air(Air),
}

#[derive(Clone, Copy)]
struct Air {
    release: bool,
    run: bool,
    drop_stops: bool,
    drift: Drift,
}

#[derive(Clone, Copy)]
enum Drift {
    StopWithoutInput { still_moves: bool },
    Spin,
    WallJump,
}

#[derive(Clone, Copy)]
enum Vertical {
    StepDown,
    AirOrStepDown,
    Elevator,
    Air { started_falling: bool },
    Nothing,
}

#[derive(Clone, Copy)]
enum After {
    Keep,
    Stop { cancel: bool },
    StopUnlessAirborne,
    EndTurn,
    EndTransition,
    NoVertical,
}

#[derive(Clone, Copy)]
struct Row(Horizontal, Vertical, After);

/// The motion table (spec §4).
const fn row(locomotion: Locomotion, mode: SpeedMode, bounce: Bounce) -> Row {
    use {After as A, Horizontal as H, Locomotion as L, Vertical as V};
    const fn air(release: bool, run: bool, drop_stops: bool, drift: Drift) -> H {
        H::Air(Air {
            release,
            run,
            drop_stops,
            drift,
        })
    }
    let (halts, coasts) = (
        Drift::StopWithoutInput { still_moves: false },
        Drift::StopWithoutInput { still_moves: true },
    );
    let (rising, falling) = (
        V::Air {
            started_falling: false,
        },
        V::Air {
            started_falling: true,
        },
    );
    match locomotion {
        L::FacingForward => Row(H::None, V::Elevator, A::NoVertical),
        L::Standing { .. } | L::RanIntoWall => Row(H::Coast, V::StepDown, A::Stop { cancel: true }),
        L::Running | L::Moonwalking => Row(H::Ground, V::StepDown, A::Keep),
        // BallAtRest: only an accelerating ball holds still.
        L::Ball { at_rest: true, .. } if matches!(mode, SpeedMode::Accelerating) => {
            Row(H::Coast, V::AirOrStepDown, A::StopUnlessAirborne)
        }
        L::Ball { .. } => Row(H::Ground, V::AirOrStepDown, A::Keep),
        L::Crouching => Row(H::Coast, V::StepDown, A::Stop { cancel: false }),
        L::TurningOnGround => Row(H::Ground, V::StepDown, A::EndTurn),
        L::Transition => Row(H::Coast, V::AirOrStepDown, A::EndTransition),
        L::Jumping { take_off: true } => Row(H::TakeOff, V::Nothing, A::Keep),
        L::Jumping { take_off: false } => Row(air(true, true, false, halts), rising, A::Keep),
        L::BallInAir { falling: false, .. } if matches!(bounce, Bounce::None) => {
            Row(air(true, true, false, halts), rising, A::Keep)
        }
        L::WallJumping => Row(air(true, true, false, Drift::WallJump), rising, A::Keep),
        L::SpinJumping => Row(air(true, true, false, Drift::Spin), rising, A::Keep),
        L::Falling => Row(air(false, true, false, halts), falling, A::Keep),
        L::BallInAir { falling: true, .. } => {
            Row(air(false, false, true, coasts), falling, A::Keep)
        }
        L::BallInAir { falling: false, .. } => {
            Row(air(false, false, false, coasts), falling, A::Keep)
        }
        L::TurningInAir => Row(H::Ground, V::AirOrStepDown, A::EndTurn),
    }
}

impl State {
    /// One frame of motion.
    ///
    /// # Errors
    ///
    /// [`MissingSpeeds`] when the row's movement type has no speed table entry.
    pub fn step(
        &mut self,
        input: &Inputs,
        rules: &MotionRules,
        world: &mut impl Collision,
    ) -> Result<Contacts, MissingSpeeds> {
        // LocomotionAtFrameStart: the row, entry and kinds come from the frame-start pose.
        let (movement, fall, land) = input.locomotion.kinds();
        let entry = rules
            .speeds
            .normal
            .of(movement)
            .ok_or(MissingSpeeds(movement))?;
        let Row(horizontal, vertical, after) = row(input.locomotion, self.mode, self.bounce);
        let mut contacts = Contacts::default();
        let total = match horizontal {
            Horizontal::None => None,
            // HorizontalStep with the base speed taken as 0.
            Horizontal::Coast => Some(self.extra_run),
            Horizontal::TakeOff => {
                self.extra_run_speed(input, rules);
                self.mode = SpeedMode::Accelerating;
                Some(self.extra_run)
            }
            Horizontal::Ground => {
                self.extra_run_speed(input, rules);
                self.base_speed(entry, true, rules.speed_cap);
                Some(self.base_speed.plus(self.extra_run))
            }
            Horizontal::Air(air) => self.drift(air, entry, input, rules),
        };
        if let Some(total) = total {
            contacts.wall_stop = self.slide(total, input, rules, world);
        }
        let wall_jump = matches!(
            horizontal,
            Horizontal::Air(Air {
                drift: Drift::Spin,
                ..
            })
        ) && self.wall_jump_check(input, rules, world, &mut contacts);
        let airborne = !matches!(self.vertical, VerticalMotion::None);
        let contact = |(down, stopped): (bool, bool)| match (down, stopped) {
            (true, true) => VerticalResult::Landed(land),
            (true, false) => VerticalResult::Falling(fall),
            (false, true) => VerticalResult::HitCeiling,
            (false, false) => VerticalResult::None,
        };
        contacts.vertical = match vertical {
            // LandingContact: a fired wall jump skips the vertical step.
            _ if wall_jump => VerticalResult::WallJump,
            Vertical::Nothing => VerticalResult::None,
            Vertical::Elevator => {
                // ElevatorRide, ElevatorHeadsUp
                if input.elevator_moving {
                    let step = rules.ground.elevator_step.into();
                    let moved = world.move_down(self.body(input), Heading::Up, step, Enemies::Skip);
                    moved.apply(&mut self.position, Heading::Up);
                }
                VerticalResult::None
            }
            Vertical::AirOrStepDown if airborne => {
                contact(self.air_step(true, input, rules, world))
            }
            Vertical::StepDown | Vertical::AirOrStepDown => {
                // StepDown reads this frame's total, from before any WallStop.
                let extra = Fixed::from(rules.ground.step_down_extra);
                let distance = match (input.on_slope, total) {
                    (false, Some(total)) => total.plus(extra),
                    (true, _) | (_, None) => extra,
                };
                let moved =
                    world.move_down(self.body(input), Heading::Down, distance, Enemies::Test);
                moved.apply(&mut self.position, Heading::Down);
                contact((true, moved.stop.is_some()))
            }
            Vertical::Air { started_falling } => {
                contact(self.air_step(started_falling, input, rules, world))
            }
        };
        match after {
            After::Keep => {}
            After::Stop { cancel } => self.stop(cancel),
            After::StopUnlessAirborne => {
                if !airborne {
                    self.stop(true);
                }
            }
            After::EndTurn => {
                self.momentum = false; // CancelSpeedBoost
                self.extra_run = Fixed::ZERO;
                contacts.vertical = VerticalResult::None; // TurnIgnoresFall
            }
            After::EndTransition => {
                // TransitionLanding
                if matches!(contacts.vertical, VerticalResult::Landed(_)) {
                    self.end_vertical();
                }
                contacts.vertical = VerticalResult::None;
            }
            After::NoVertical => contacts.vertical = VerticalResult::None,
        }
        contacts.restart_standing_animation =
            matches!(input.locomotion, Locomotion::Standing { plain: true }) && input.pad.shoot;
        Ok(contacts)
    }

    /// The arm-pump step (`ArmPumpStep`), after `step` once pose selection has settled
    /// `request`: true when pose selection should start the ran-into-a-wall pose.
    pub fn arm_pump_step(
        &mut self,
        input: &Inputs,
        wall_stop: bool,
        request: PoseRequest,
        rules: &MotionRules,
        world: &mut impl Collision,
    ) -> bool {
        let running = matches!(input.locomotion, Locomotion::Running);
        match request {
            PoseRequest::Forced => false,
            _ if wall_stop && running => true,
            PoseRequest::Pose { running: true } => {
                let step = Fixed::from(rules.ground.arm_pump_step);
                let (heading, signed) = match input.facing {
                    Facing::Left => (Heading::Left, Fixed::ZERO.minus(step)),
                    Facing::Right | Facing::Forward => (Heading::Right, step),
                };
                if let Some(hit) = world.enemy_ahead(self.body(input), heading, step) {
                    if hit.touching {
                        self.position.y = self.position.y.with_sub(0); // EnemyTouchZeroesYSubpixel
                    }
                    return true;
                }
                let moved = world.move_horizontal(self.body(input), heading, signed, Enemies::Skip);
                moved.apply(&mut self.position, heading);
                moved.stop.is_some()
            }
            PoseRequest::None | PoseRequest::Pose { running: false } => false,
        }
    }

    const fn body(&self, input: &Inputs) -> Body {
        Body {
            position: self.position,
            radius: input.radius,
        }
    }

    // ExtraRunSpeed, with ExtraRunCap tested on the old value before the add.
    fn extra_run_speed(&mut self, input: &Inputs, rules: &MotionRules) {
        let extra = rules.speeds.extra_run;
        if matches!(input.locomotion, Locomotion::Running) && input.pad.run {
            self.momentum = true;
            let (old, cap) = (self.extra_run, Fixed::from(extra.max));
            let grows =
                Fixed::below(old.pixel(), cap.pixel()) || Fixed::below(old.sub(), cap.sub());
            self.extra_run = if grows {
                old.plus(extra.acceleration.into())
            } else {
                cap
            };
        } else if !self.momentum {
            self.extra_run = Fixed::ZERO;
        }
    }

    // BaseSpeed, with SpeedCap: true when the cap snapped the speed.
    fn base_speed(&mut self, entry: &SpeedEntry, brake: bool, cap: SpeedCap) -> bool {
        let braking = match self.mode {
            SpeedMode::TurningAround => true,
            SpeedMode::Decelerating => brake,
            SpeedMode::Accelerating => false,
        };
        if braking {
            self.base_speed = self.base_speed.minus(entry.deceleration.into());
            if self.base_speed.negative() {
                self.base_speed = Fixed::ZERO;
                self.mode = SpeedMode::Accelerating;
            }
            return false;
        }
        let raised = self.base_speed.plus(entry.acceleration.into());
        self.base_speed = raised.capped(entry.max.into(), cap);
        self.base_speed != raised
    }

    // The air's horizontal part: the total to move by, or None when the drift stops her.
    fn drift(
        &mut self,
        air: Air,
        entry: &SpeedEntry,
        input: &Inputs,
        rules: &MotionRules,
    ) -> Option<Fixed> {
        let pad = input.pad;
        let accelerating = |mode| matches!(mode, SpeedMode::Accelerating);
        let still = !pad.sideways.left && !pad.sideways.right;
        if air.drop_stops && still && accelerating(self.mode) {
            self.stop(true); // BallDropStops
        }
        if matches!(air.drift, Drift::Spin) {
            // SpaceJump: the speed's middle bytes, as 8.8 pixels.
            let [_, sub_high, pixel_low, _] = self.vertical_speed.0.to_le_bytes();
            let (from, below) = rules.air.space_jump_window;
            let window = (from..below).contains(&u16::from_le_bytes([sub_high, pixel_low]));
            let falling = matches!(self.vertical, VerticalMotion::Down);
            if input.equipment.space_jump
                && falling
                && window
                && matches!(pad.jump, JumpButton::Pressed)
            {
                self.jump(rules, input.equipment);
            }
        }
        if air.run {
            self.extra_run_speed(input, rules);
        }
        if air.release
            && matches!(self.vertical, VerticalMotion::Up)
            && (matches!(pad.jump, JumpButton::Released) || self.vertical_speed.negative())
        {
            self.fall(); // JumpRelease
        }
        let capped = self.base_speed(entry, false, rules.speed_cap);
        let moves = match air.drift {
            // NoInputStops
            Drift::StopWithoutInput { still_moves } if still && accelerating(self.mode) => {
                self.base_speed = Fixed::ZERO;
                still_moves
            }
            Drift::StopWithoutInput { .. } => true,
            // SpinJumpDrift
            Drift::Spin => {
                let facing_held = if matches!(input.facing, Facing::Left) {
                    pad.sideways.left
                } else {
                    pad.sideways.right
                };
                let drifts = capped || matches!(self.mode, SpeedMode::TurningAround) || facing_held;
                if !drifts {
                    self.base_speed = Fixed::ZERO;
                    self.mode = SpeedMode::Accelerating;
                } else if accelerating(self.mode) {
                    self.mode = SpeedMode::Decelerating;
                }
                drifts
            }
            // WallJumpDrift
            Drift::WallJump => {
                if accelerating(self.mode) {
                    self.mode = SpeedMode::Decelerating;
                }
                true
            }
        };
        moves.then(|| self.base_speed.plus(self.extra_run))
    }

    // HorizontalStep in the HorizontalDirection, then DisplacementClamp; true on a WallStop.
    fn slide(
        &mut self,
        total: Fixed,
        input: &Inputs,
        rules: &MotionRules,
        world: &mut impl Collision,
    ) -> bool {
        let leftward =
            matches!(self.mode, SpeedMode::TurningAround) != matches!(input.facing, Facing::Left);
        let (heading, signed) = if leftward {
            (Heading::Left, Fixed::ZERO.minus(total))
        } else {
            (Heading::Right, total)
        };
        let distance = signed.clamped(rules.ground.max_displacement);
        let moved = world.move_horizontal(self.body(input), heading, distance, Enemies::Test);
        moved.apply(&mut self.position, heading);
        let stopped = moved.stop.is_some();
        if stopped {
            self.stop(true); // WallStop
        }
        stopped
    }

    // WallJumpCheck and WallProbeSubpixel: true when the wall jump fires.
    fn wall_jump_check(
        &mut self,
        input: &Inputs,
        rules: &MotionRules,
        world: &mut impl Collision,
        contacts: &mut Contacts,
    ) -> bool {
        let pad = input.pad;
        let heading = match (pad.sideways.left, pad.sideways.right) {
            _ if !input.spin.was_spinning => return false,
            (true, _) => Heading::Right,
            (false, true) => Heading::Left,
            (false, false) => return false,
        };
        let reach = rules.air.wall_jump_reach;
        let Some(hit) = world.probe_wall(self.body(input), heading, reach) else {
            return false;
        };
        if let Some(sub) = hit.subpixel {
            self.position.x = self.position.x.with_sub(sub);
        }
        contacts.wall_jump_armed = !input.spin.ready;
        input.spin.ready
            && Fixed::below(hit.distance_px, reach)
            && matches!(pad.jump, JumpButton::Pressed)
    }

    // StartedFalling if asked, then VerticalStep with FallCap: whether she moved down, and whether she was stopped.
    fn air_step(
        &mut self,
        started_falling: bool,
        input: &Inputs,
        rules: &MotionRules,
        world: &mut impl Collision,
    ) -> (bool, bool) {
        if started_falling
            && matches!(self.vertical, VerticalMotion::Up)
            && self.vertical_speed.negative()
        {
            self.fall();
        }
        let distance = self.vertical_speed;
        let gravity = Fixed::from(rules.vertical.gravity);
        let down = matches!(self.vertical, VerticalMotion::Down);
        let moved = if down {
            if self.vertical_speed.pixel() != rules.air.fall_cap_pixel {
                self.vertical_speed = self.vertical_speed.plus(gravity);
            }
            world.move_down(self.body(input), Heading::Down, distance, Enemies::Test)
        } else {
            self.vertical_speed = self.vertical_speed.minus(gravity);
            world.move_up(self.body(input), distance)
        };
        moved.apply(&mut self.position, Heading::Down);
        (down, moved.stop.is_some())
    }

    // The speeds stop; `cancel` is CancelSpeedBoost.
    const fn stop(&mut self, cancel: bool) {
        self.momentum &= !cancel;
        self.extra_run = Fixed::ZERO;
        self.base_speed = Fixed::ZERO;
        self.mode = SpeedMode::Accelerating;
    }

    // The rise ends: she falls from a standstill.
    const fn fall(&mut self) {
        self.vertical_speed = Fixed::ZERO;
        self.vertical = VerticalMotion::Down;
    }

    // Jump, with the hi-jump's speed if equipped.
    fn jump(&mut self, rules: &MotionRules, equipment: Equipment) {
        let speeds = rules.vertical;
        self.vertical_speed = (if equipment.hi_jump {
            speeds.hi_jump
        } else {
            speeds.jump
        })
        .into();
        self.vertical = VerticalMotion::Up;
    }

    // EndVertical
    const fn end_vertical(&mut self) {
        self.vertical_speed = Fixed::ZERO;
        self.vertical = VerticalMotion::None;
        self.bounce = Bounce::None;
    }
}

#[cfg(test)]
mod tests;
