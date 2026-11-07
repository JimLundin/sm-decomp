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
use data::movement::{
    AirMotion, GroundMotion, HorizontalSpeeds, Speed, SpeedEntry, VerticalSpeeds,
};
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

// SpeedTable and LandingContact: whose speeds the row reads, and which fall and landing it
// starts (spec §2).
const fn kinds(locomotion: Locomotion) -> (MovementType, FallKind, LandKind) {
    use {FallKind as F, LandKind as K, Locomotion as L, MovementType as M};
    match locomotion {
        L::FacingForward | L::Standing { .. } => (M::Standing, F::Airborne, K::NoChange),
        L::Running => (M::Running, F::Airborne, K::NoChange),
        L::Ball { spring: false, .. } => (M::MorphBallOnGround, F::MorphBall, K::NoChange),
        L::Ball { spring: true, .. } => (M::SpringBallOnGround, F::SpringBall, K::NoChange),
        L::Crouching => (M::Crouching, F::Airborne, K::NoChange),
        L::TurningOnGround => (M::TurningAroundOnGround, F::NoChange, K::NoChange),
        L::Transition => (M::CrouchingTransition, F::NoChange, K::NoChange),
        L::Moonwalking => (M::Moonwalking, F::Airborne, K::NoChange),
        L::RanIntoWall => (M::RanIntoWall, F::Airborne, K::NoChange),
        L::Jumping { .. } => (M::NormalJumping, F::NoChange, K::Grounded),
        L::SpinJumping => (M::SpinJumping, F::NoChange, K::Grounded),
        L::Falling => (M::Falling, F::NoChange, K::Grounded),
        L::BallInAir { spring: false, .. } => (M::MorphBallFalling, F::NoChange, K::MorphBall),
        L::BallInAir { falling: true, .. } => (M::SpringBallFalling, F::NoChange, K::SpringBall),
        L::BallInAir { falling: false, .. } => (M::SpringBallInAir, F::NoChange, K::SpringBall),
        L::WallJumping => (M::WallJumping, F::NoChange, K::Grounded),
        L::TurningInAir => (M::TurningAroundJumping, F::NoChange, K::NoChange),
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

// What moves her sideways.
enum Horizontal {
    // HorizontalStep.
    Ground,
    // HorizontalStep with the base speed taken as 0.
    Coast,
    None,
    // ExtraRunSpeed, then the run speed alone.
    TakeOff,
    Air(Drift),
}

// How the air moves her sideways.
#[derive(Clone, Copy)]
enum Drift {
    // NoInputStops halts her; ExtraRunSpeed runs.
    Halts,
    // NoInputStops zeroes the base speed but she still moves; no ExtraRunSpeed.
    Rolls,
    // Rolls, after BallDropStops.
    Drops,
    // SpinJumpDrift, SpaceJump and WallJumpCheck.
    Spin,
    // WallJumpDrift.
    WallJump,
}

// What moves her up or down.
#[derive(Clone, Copy)]
enum Vertical {
    StepDown,
    // The air step when airborne (with StartedFalling), else StepDown.
    AirOrStepDown,
    Elevator,
    Air(Rise),
    Nothing,
}

// What ends a rise in the air.
#[derive(Clone, Copy)]
enum Rise {
    // JumpRelease, in the drift.
    Released,
    // StartedFalling, in the air step.
    Spent,
}

// What happens after both steps.
enum After {
    Keep,
    // The speeds stop; `cancel` is CancelSpeedBoost.
    Stop { cancel: bool },
    StopUnlessAirborne,
    // CancelSpeedBoost, the run speed lost, and TurnIgnoresFall.
    EndTurn,
    // TransitionLanding.
    EndTransition,
}

struct Row(Horizontal, Vertical, After);

/// The motion table (spec §4).
const fn row(locomotion: Locomotion, mode: SpeedMode, bounce: Bounce) -> Row {
    use {After as A, Drift as D, Horizontal as H, Locomotion as L, Rise as R, Vertical as V};
    match locomotion {
        L::FacingForward => Row(H::None, V::Elevator, A::Keep),
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
        L::Jumping { take_off: false } => Row(H::Air(D::Halts), V::Air(R::Released), A::Keep),
        L::BallInAir { falling: false, .. } if matches!(bounce, Bounce::None) => {
            Row(H::Air(D::Halts), V::Air(R::Released), A::Keep)
        }
        L::WallJumping => Row(H::Air(D::WallJump), V::Air(R::Released), A::Keep),
        L::SpinJumping => Row(H::Air(D::Spin), V::Air(R::Released), A::Keep),
        L::Falling => Row(H::Air(D::Halts), V::Air(R::Spent), A::Keep),
        L::BallInAir { falling: true, .. } => Row(H::Air(D::Drops), V::Air(R::Spent), A::Keep),
        L::BallInAir { falling: false, .. } => Row(H::Air(D::Rolls), V::Air(R::Spent), A::Keep),
        L::TurningInAir => Row(H::Ground, V::AirOrStepDown, A::EndTurn),
    }
}

// One frame's inputs, values and collision, as every step reads them.
struct Frame<'a, W> {
    input: &'a Inputs,
    rules: &'a MotionRules<'a>,
    world: &'a mut W,
}

// HorizontalDirection: the heading and signed distance for `distance` one way or the other.
const fn sideways(leftward: bool, distance: Fixed) -> (Heading, Fixed) {
    if leftward {
        (Heading::Left, Fixed::ZERO.minus(distance))
    } else {
        (Heading::Right, distance)
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
        let (kind, fall, land) = kinds(input.locomotion);
        let entry = rules.speeds.normal.of(kind).ok_or(MissingSpeeds(kind))?;
        let Row(horizontal, vertical, after) = row(input.locomotion, self.mode, self.bounce);
        let f = &mut Frame {
            input,
            rules,
            world,
        };
        let total = match horizontal {
            Horizontal::None => None,
            Horizontal::Coast => Some(self.extra_run),
            Horizontal::TakeOff => {
                self.extra_run_speed(f);
                self.mode = SpeedMode::Accelerating;
                Some(self.extra_run)
            }
            Horizontal::Ground => {
                self.extra_run_speed(f);
                self.base_speed(entry, true, rules.speed_cap);
                Some(self.base_speed.plus(self.extra_run))
            }
            Horizontal::Air(drift) => self.drift(f, drift, vertical, entry),
        };
        let wall_stop = total.is_some_and(|total| self.slide(f, total));
        let (wall_jump_armed, wall_jump) = self.wall_jump_check(f, &horizontal);
        let airborne = self.vertical != VerticalMotion::None;
        let contact = |(down, stopped)| match (down, stopped) {
            (true, true) => VerticalResult::Landed(land),
            (true, false) => VerticalResult::Falling(fall),
            (false, true) => VerticalResult::HitCeiling,
            (false, false) => VerticalResult::None,
        };
        let body = self.body(input);
        let mut vertical = match vertical {
            // LandingContact: a fired wall jump skips the vertical step.
            _ if wall_jump => VerticalResult::WallJump,
            // ElevatorRide, ElevatorHeadsUp
            Vertical::Elevator if input.elevator_moving => {
                let step = rules.ground.elevator_step.into();
                let moved = f.world.move_down(body, Heading::Up, step, Enemies::Skip);
                moved.apply(&mut self.position, Heading::Up);
                VerticalResult::None
            }
            Vertical::Nothing | Vertical::Elevator => VerticalResult::None,
            Vertical::AirOrStepDown if airborne => contact(self.air_step(f, Rise::Spent)),
            Vertical::StepDown | Vertical::AirOrStepDown => {
                // StepDown reads this frame's total, from before any WallStop.
                let extra = Fixed::from(rules.ground.step_down_extra);
                let depth = match (input.on_slope, total) {
                    (false, Some(total)) => total.plus(extra),
                    (true, _) | (_, None) => extra,
                };
                let moved = f.world.move_down(body, Heading::Down, depth, Enemies::Test);
                contact((true, moved.apply(&mut self.position, Heading::Down)))
            }
            Vertical::Air(rise) => contact(self.air_step(f, rise)),
        };
        match after {
            After::Stop { cancel } => self.stop(cancel),
            After::StopUnlessAirborne if !airborne => self.stop(true),
            After::EndTurn => self.stop_running(),
            After::EndTransition if matches!(vertical, VerticalResult::Landed(_)) => {
                self.end_vertical();
            }
            After::Keep | After::StopUnlessAirborne | After::EndTransition => {}
        }
        // TurnIgnoresFall, TransitionLanding
        if matches!(after, After::EndTurn | After::EndTransition) {
            vertical = VerticalResult::None;
        }
        Ok(Contacts {
            wall_stop,
            vertical,
            wall_jump_armed,
            restart_standing_animation: input.pad.shoot
                && input.locomotion == Locomotion::Standing { plain: true },
        })
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
        match request {
            PoseRequest::Forced => false,
            _ if wall_stop && input.locomotion == Locomotion::Running => true,
            PoseRequest::Pose { running: true } => {
                let step = Fixed::from(rules.ground.arm_pump_step);
                let (heading, signed) = sideways(input.facing == Facing::Left, step);
                if let Some(hit) = world.enemy_ahead(self.body(input), heading, step) {
                    if hit.touching {
                        self.position.y = self.position.y.with_sub(0); // EnemyTouchZeroesYSubpixel
                    }
                    return true;
                }
                let moved = world.move_horizontal(self.body(input), heading, signed, Enemies::Skip);
                moved.apply(&mut self.position, heading)
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
    fn extra_run_speed<W>(&mut self, f: &Frame<W>) {
        let extra = f.rules.speeds.extra_run;
        if f.input.locomotion == Locomotion::Running && f.input.pad.run {
            self.momentum = true;
            let (old, cap) = (self.extra_run, Fixed::from(extra.max));
            let grows =
                Fixed::below(old.pixel(), cap.pixel()) || Fixed::below(old.sub(), cap.sub());
            let raised = old.plus(extra.acceleration.into());
            self.extra_run = if grows { raised } else { cap };
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
                self.halt();
            }
            return false;
        }
        let raised = self.base_speed.plus(entry.acceleration.into());
        self.base_speed = raised.capped(entry.max.into(), cap);
        self.base_speed != raised
    }

    // The air's horizontal part: the total to move by, or None when the drift stops her.
    fn drift<W>(
        &mut self,
        f: &Frame<W>,
        drift: Drift,
        vertical: Vertical,
        entry: &SpeedEntry,
    ) -> Option<Fixed> {
        let (pad, equipment) = (f.input.pad, f.input.equipment);
        let still = !pad.sideways.left && !pad.sideways.right;
        if matches!(drift, Drift::Drops) && still && self.mode == SpeedMode::Accelerating {
            self.stop(true); // BallDropStops
        }
        // SpaceJump: the speed's middle bytes, as 8.8 pixels.
        let [_, sub_high, pixel_low, _] = self.vertical_speed.0.to_le_bytes();
        let (from, below) = f.rules.air.space_jump_window;
        if matches!(drift, Drift::Spin)
            && equipment.space_jump
            && self.vertical == VerticalMotion::Down
            && (from..below).contains(&u16::from_le_bytes([sub_high, pixel_low]))
            && pad.jump == JumpButton::Pressed
        {
            self.jump(f.rules, equipment);
        }
        if !matches!(drift, Drift::Rolls | Drift::Drops) {
            self.extra_run_speed(f);
        }
        if matches!(vertical, Vertical::Air(Rise::Released))
            && self.vertical == VerticalMotion::Up
            && (pad.jump == JumpButton::Released || self.vertical_speed.negative())
        {
            self.fall(); // JumpRelease
        }
        let capped = self.base_speed(entry, false, f.rules.speed_cap);
        let accelerating = self.mode == SpeedMode::Accelerating;
        let facing_held = match f.input.facing {
            Facing::Left => pad.sideways.left,
            Facing::Right | Facing::Forward => pad.sideways.right,
        };
        let moves = match drift {
            // NoInputStops
            Drift::Halts | Drift::Rolls | Drift::Drops if still && accelerating => {
                self.base_speed = Fixed::ZERO;
                !matches!(drift, Drift::Halts)
            }
            Drift::Halts | Drift::Rolls | Drift::Drops => true,
            // SpinJumpDrift
            Drift::Spin if !(capped || self.mode == SpeedMode::TurningAround || facing_held) => {
                self.halt();
                false
            }
            // SpinJumpDrift and WallJumpDrift: a drift that carries on brakes.
            Drift::Spin | Drift::WallJump => {
                if accelerating {
                    self.mode = SpeedMode::Decelerating;
                }
                true
            }
        };
        moves.then(|| self.base_speed.plus(self.extra_run))
    }

    // HorizontalStep in the HorizontalDirection, then DisplacementClamp; true on a WallStop.
    fn slide(&mut self, f: &mut Frame<impl Collision>, total: Fixed) -> bool {
        let leftward = (self.mode == SpeedMode::TurningAround) != (f.input.facing == Facing::Left);
        let (heading, signed) = sideways(leftward, total);
        let by = signed.clamped(f.rules.ground.max_displacement);
        let body = self.body(f.input);
        let moved = f.world.move_horizontal(body, heading, by, Enemies::Test);
        let stopped = moved.apply(&mut self.position, heading);
        if stopped {
            self.stop(true); // WallStop
        }
        stopped
    }

    // WallJumpCheck, in the spin jump's row, and WallProbeSubpixel: whether the wall jump is
    // armed, and whether it fires.
    fn wall_jump_check(&mut self, f: &mut Frame<impl Collision>, row: &Horizontal) -> (bool, bool) {
        let (input, reach) = (f.input, f.rules.air.wall_jump_reach);
        let heading = match (input.pad.sideways.left, input.pad.sideways.right) {
            _ if !matches!(row, Horizontal::Air(Drift::Spin)) || !input.spin.was_spinning => {
                return (false, false);
            }
            (true, _) => Heading::Right,
            (false, true) => Heading::Left,
            (false, false) => return (false, false),
        };
        let Some(hit) = f.world.probe_wall(self.body(input), heading, reach) else {
            return (false, false);
        };
        if let Some(sub) = hit.subpixel {
            self.position.x = self.position.x.with_sub(sub);
        }
        let ready = input.spin.ready;
        let fires =
            ready && Fixed::below(hit.distance_px, reach) && input.pad.jump == JumpButton::Pressed;
        (!ready, fires)
    }

    // The Rise's end, then VerticalStep with FallCap: whether she moved down, and whether she
    // was stopped.
    fn air_step(&mut self, f: &mut Frame<impl Collision>, rise: Rise) -> (bool, bool) {
        if matches!(rise, Rise::Spent)
            && self.vertical == VerticalMotion::Up
            && self.vertical_speed.negative()
        {
            self.fall(); // StartedFalling
        }
        let (speed, body) = (self.vertical_speed, self.body(f.input));
        let gravity = Fixed::from(f.rules.vertical.gravity);
        let down = self.vertical == VerticalMotion::Down;
        let moved = if down {
            if self.vertical_speed.pixel() != f.rules.air.fall_cap_pixel {
                self.vertical_speed = self.vertical_speed.plus(gravity);
            }
            f.world.move_down(body, Heading::Down, speed, Enemies::Test)
        } else {
            self.vertical_speed = self.vertical_speed.minus(gravity);
            f.world.move_up(body, speed)
        };
        (down, moved.apply(&mut self.position, Heading::Down))
    }

    // The base speed stops; the next frame accelerates.
    const fn halt(&mut self) {
        self.base_speed = Fixed::ZERO;
        self.mode = SpeedMode::Accelerating;
    }

    // CancelSpeedBoost, and the run speed is lost.
    const fn stop_running(&mut self) {
        self.momentum = false;
        self.extra_run = Fixed::ZERO;
    }

    // The speeds stop; `cancel` is CancelSpeedBoost.
    const fn stop(&mut self, cancel: bool) {
        self.momentum &= !cancel;
        self.extra_run = Fixed::ZERO;
        self.halt();
    }

    // The rise ends: she falls from a standstill.
    const fn fall(&mut self) {
        self.vertical_speed = Fixed::ZERO;
        self.vertical = VerticalMotion::Down;
    }

    // Jump.
    fn jump(&mut self, rules: &MotionRules, equipment: Equipment) {
        self.rise((rules.vertical.jump, rules.vertical.hi_jump), equipment);
    }

    // A rise at the first speed, or the second with the hi-jump boots.
    fn rise(&mut self, (normal, hi): (Speed, Speed), equipment: Equipment) {
        self.vertical_speed = Fixed::from(if equipment.hi_jump { hi } else { normal });
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
