//! Samus on the ground.
//!
//! Standing, running, turning, crouching, moonwalking, the pose transitions and the morph and
//! spring ball. Each way of moving is a row of the motion table; the speeds
//! come from `data`, the switches from `Switches`, and every move goes through [`Collision`].

use crate::model::collision::{Body, Collision, Enemies, Side};
use crate::model::num::Fixed;
use crate::session::{SpeedCap, Switches};
use data::movement::{ExtraRun, GroundMotion, HorizontalSpeeds, SpeedEntry};
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
    /// Braking.
    Decelerating,
}

/// Samus's ground-motion State: the quantities a later frame reads.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Ground {
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
        /// One of the two plain standing poses.
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
}

impl Locomotion {
    // SpeedTableRow: whose speeds the row reads.
    const fn movement(self) -> MovementType {
        match self {
            Self::FacingForward | Self::Standing { .. } => MovementType::Standing,
            Self::Running => MovementType::Running,
            Self::Ball { spring: false, .. } => MovementType::MorphBallOnGround,
            Self::Ball { spring: true, .. } => MovementType::SpringBallOnGround,
            Self::Crouching => MovementType::Crouching,
            Self::TurningOnGround => MovementType::TurningAroundOnGround,
            Self::Transition => MovementType::CrouchingTransition,
            Self::Moonwalking => MovementType::Moonwalking,
            Self::RanIntoWall => MovementType::RanIntoWall,
        }
    }
    // LandingContact: the fall pose selection starts.
    const fn fall(self) -> FallKind {
        match self {
            Self::Ball { spring: false, .. } => FallKind::MorphBall,
            Self::Ball { spring: true, .. } => FallKind::SpringBall,
            Self::TurningOnGround | Self::Transition => FallKind::NoChange,
            Self::FacingForward
            | Self::Standing { .. }
            | Self::Running
            | Self::Crouching
            | Self::Moonwalking
            | Self::RanIntoWall => FallKind::Airborne,
        }
    }
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

/// The vertical result of the frame.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Vertical {
    /// Nothing to report.
    #[default]
    None,
    /// She is on the ground.
    Landed,
    /// The ground is gone; the fall to start.
    Falling(FallKind),
    /// Her head hit the ceiling (only from the air domain).
    HitCeiling,
}

/// The buttons ground motion reads.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Pad {
    /// Run is held.
    pub run: bool,
    /// Shoot is held.
    pub shoot: bool,
}

/// What other domains give ground motion this frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Inputs {
    /// Her way of moving at the frame's start.
    pub locomotion: Locomotion,
    /// Her pose's facing at the frame's start.
    pub facing: Facing,
    /// Her pose's radius.
    pub radius: Radius,
    /// The buttons held.
    pub pad: Pad,
    /// Pose selection started a running pose this frame.
    pub new_running_pose: bool,
    /// The air domain's vertical result, when she is airborne.
    pub air: Option<Vertical>,
    /// The elevator under her is moving.
    pub elevator_moving: bool,
    /// Collision placed her by a slope.
    pub on_slope: bool,
}

/// What the frame hands other domains.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Contacts {
    /// The vertical result, after the row's After.
    pub vertical: Vertical,
    /// The horizontal step was stopped (`WallStop`).
    pub wall_stop: bool,
    /// Pose selection should start the ran-into-a-wall pose (`ArmPumpStep`).
    pub ran_into_wall: bool,
    /// Requests to the air and animation domains.
    pub requests: Requests,
}

/// Requests for the air and animation domains.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Requests {
    /// The air domain resets her vertical motion (`TransitionLanding`).
    pub end_vertical_motion: bool,
    /// The standing animation restarts (`StandingShotRestartsAnimation`).
    pub restart_standing_animation: bool,
}

/// The speed table has no entry for this movement type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MissingSpeeds(pub MovementType);

#[derive(Clone, Copy)]
enum Horizontal {
    Speed,
    Coast,
    None,
}

#[derive(Clone, Copy)]
enum VerticalStep {
    StepDown,
    StepDownUnlessAirborne,
    Elevator,
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
struct Row(Horizontal, VerticalStep, After);

/// The motion table.
const fn row(locomotion: Locomotion, mode: SpeedMode) -> Row {
    use {After as A, Horizontal as H, VerticalStep as V};
    match locomotion {
        Locomotion::FacingForward => Row(H::None, V::Elevator, A::NoVertical),
        Locomotion::Standing { .. } | Locomotion::RanIntoWall => {
            Row(H::Coast, V::StepDown, A::Stop { cancel: true })
        }
        Locomotion::Running | Locomotion::Moonwalking => Row(H::Speed, V::StepDown, A::Keep),
        // BallAtRest: only an accelerating ball holds still.
        Locomotion::Ball { at_rest: true, .. } if matches!(mode, SpeedMode::Accelerating) => {
            Row(H::Coast, V::StepDownUnlessAirborne, A::StopUnlessAirborne)
        }
        Locomotion::Ball { .. } => Row(H::Speed, V::StepDownUnlessAirborne, A::Keep),
        Locomotion::Crouching => Row(H::Coast, V::StepDown, A::Stop { cancel: false }),
        Locomotion::TurningOnGround => Row(H::Speed, V::StepDown, A::EndTurn),
        Locomotion::Transition => Row(H::Coast, V::StepDownUnlessAirborne, A::EndTransition),
    }
}

impl Ground {
    /// One frame of ground motion.
    ///
    /// # Errors
    ///
    /// [`MissingSpeeds`] when a speed row's movement type is past the end of `speeds`.
    pub fn step(
        &mut self,
        input: &Inputs,
        switches: Switches,
        speeds: &HorizontalSpeeds,
        motion: &GroundMotion,
        world: &mut impl Collision,
    ) -> Result<Contacts, MissingSpeeds> {
        // LocomotionAtFrameStart: the row comes from the pose before this frame's pose update.
        let Row(horizontal, vertical, after) = row(input.locomotion, self.mode);
        let mut contacts = Contacts::default();
        let total = match horizontal {
            // ExtraRunSpeed comes before the base speed; the step reads both (SpeedTableRow).
            Horizontal::Speed => {
                let movement = input.locomotion.movement();
                let entry = speeds.normal.of(movement).ok_or(MissingSpeeds(movement))?;
                self.run(input, speeds.extra_run);
                self.change_base_speed(entry, switches.speed_cap);
                Some(self.base_speed.plus(self.extra_run))
            }
            // HorizontalStep with the base speed taken as 0.
            Horizontal::Coast => Some(self.extra_run),
            Horizontal::None => None,
        };
        if let Some(total) = total {
            contacts.wall_stop = self.step_sideways(total, input, motion, world);
        }
        // StepDown reads this frame's total even after a WallStop zeroed the speeds.
        let total = total.unwrap_or(Fixed::ZERO);
        contacts.vertical = match (vertical, input.air) {
            // StepDownUnlessAirborne: in the air, the air domain's result stands.
            (VerticalStep::StepDownUnlessAirborne, Some(air)) => air,
            (VerticalStep::StepDown | VerticalStep::StepDownUnlessAirborne, _) => {
                self.step_down(total, input, motion, world)
            }
            (VerticalStep::Elevator, _) => {
                // ElevatorRide
                if input.elevator_moving {
                    let moved = world.move_down(
                        self.body(input),
                        motion.elevator_step.into(),
                        Enemies::Skip,
                    );
                    self.position.y = moved.applied(self.position.y);
                }
                Vertical::None
            }
        };
        // ArmPumpStep comes after the After, and so after WallStop.
        self.after(after, input, &mut contacts);
        contacts.ran_into_wall = self.arm_pump(contacts, input, motion, world);
        // StandingShotRestartsAnimation
        contacts.requests.restart_standing_animation =
            matches!(input.locomotion, Locomotion::Standing { plain: true }) && input.pad.shoot;
        Ok(contacts)
    }

    const fn body(&self, input: &Inputs) -> Body {
        Body {
            position: self.position,
            radius: input.radius,
        }
    }

    // ExtraRunSpeed, with ExtraRunCap tested on the old value before the add.
    fn run(&mut self, input: &Inputs, extra: ExtraRun) {
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

    // BaseSpeedAccelerate with SpeedCap, or BaseSpeedDecelerate down to zero.
    const fn change_base_speed(&mut self, entry: &SpeedEntry, cap: SpeedCap) {
        let (accel, max) = (Fixed(entry.acceleration.0), Fixed(entry.max.0));
        match self.mode {
            SpeedMode::Accelerating => {
                self.base_speed = self.base_speed.plus(accel).capped(max, cap);
            }
            SpeedMode::TurningAround | SpeedMode::Decelerating => {
                self.base_speed = self.base_speed.minus(Fixed(entry.deceleration.0));
                if self.base_speed.pixel().cast_signed().is_negative() {
                    self.base_speed = Fixed::ZERO;
                    self.mode = SpeedMode::Accelerating;
                }
            }
        }
    }

    // HorizontalStep in the HorizontalDirection, then DisplacementClamp; true on a WallStop.
    fn step_sideways(
        &mut self,
        total: Fixed,
        input: &Inputs,
        motion: &GroundMotion,
        world: &mut impl Collision,
    ) -> bool {
        let leftward = if matches!(self.mode, SpeedMode::TurningAround) {
            !matches!(input.facing, Facing::Left)
        } else {
            matches!(input.facing, Facing::Left)
        };
        let signed = if leftward {
            Fixed::ZERO.minus(total)
        } else {
            total
        };
        let moved = world.move_horizontal(
            self.body(input),
            signed.clamped(motion.max_displacement),
            Enemies::Test,
        );
        self.position.x = moved.applied(self.position.x);
        let stopped = moved.stop.is_some();
        if stopped {
            self.stop(true); // WallStop
        }
        stopped
    }

    // StepDown, reporting LandingContact.
    fn step_down(
        &mut self,
        total: Fixed,
        input: &Inputs,
        motion: &GroundMotion,
        world: &mut impl Collision,
    ) -> Vertical {
        let extra = Fixed::from(motion.step_down_extra);
        let distance = if input.on_slope {
            extra
        } else {
            total.plus(extra)
        };
        let moved = world.move_down(self.body(input), distance, Enemies::Test);
        self.position.y = moved.applied(self.position.y);
        if moved.stop.is_some() {
            Vertical::Landed
        } else {
            Vertical::Falling(input.locomotion.fall())
        }
    }

    // The speeds stop; `cancel` is CancelSpeedBoost.
    const fn stop(&mut self, cancel: bool) {
        self.momentum &= !cancel;
        self.extra_run = Fixed::ZERO;
        self.base_speed = Fixed::ZERO;
        self.mode = SpeedMode::Accelerating;
    }

    const fn after(&mut self, after: After, input: &Inputs, contacts: &mut Contacts) {
        match after {
            After::Keep => {}
            After::Stop { cancel } => self.stop(cancel),
            After::StopUnlessAirborne => {
                if input.air.is_none() {
                    self.stop(true);
                }
            }
            After::EndTurn => {
                self.momentum = false; // CancelSpeedBoost
                self.extra_run = Fixed::ZERO;
                contacts.vertical = Vertical::None; // TurnIgnoresFall
            }
            After::EndTransition => {
                // TransitionLanding
                contacts.requests.end_vertical_motion =
                    matches!(contacts.vertical, Vertical::Landed);
                contacts.vertical = Vertical::None;
            }
            After::NoVertical => contacts.vertical = Vertical::None,
        }
    }

    // ArmPumpStep: true when pose selection should start the ran-into-a-wall pose.
    fn arm_pump(
        &mut self,
        contacts: Contacts,
        input: &Inputs,
        motion: &GroundMotion,
        world: &mut impl Collision,
    ) -> bool {
        if contacts.wall_stop && matches!(input.locomotion, Locomotion::Running) {
            return true;
        }
        if !input.new_running_pose {
            return false;
        }
        let step = Fixed::from(motion.arm_pump_step);
        let (side, signed) = match input.facing {
            Facing::Left => (Side::Left, Fixed::ZERO.minus(step)),
            Facing::Right | Facing::Forward => (Side::Right, step),
        };
        if world.enemy_blocks(self.body(input), side, step) {
            return true;
        }
        let moved = world.move_horizontal(self.body(input), signed, Enemies::Skip);
        self.position.x = moved.applied(self.position.x);
        moved.stop.is_some()
    }
}

#[cfg(test)]
mod tests;
