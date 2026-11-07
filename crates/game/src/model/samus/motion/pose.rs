//! Pose selection's writes to the motion State when a new pose starts (spec §7).

use super::{
    Bounce, Equipment, FallKind, LandKind, MotionRules, SpeedMode, State, VerticalMotion,
    VerticalResult,
};
use crate::model::num::Fixed;

/// What a new pose's start writes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PoseStart {
    /// A turn, on the ground or in the air.
    Turn,
    /// A ball pose facing the other way.
    BallReverse,
    /// A spin jump after a spin or wall jump, facing the other way.
    SpinReverse,
    /// A normal jump or falling pose.
    JumpOrFall,
    /// The normal jump's first pose.
    TakeOff {
        /// From the crouch without aim.
        from_plain_crouch: bool,
    },
    /// A spin jump not after a spin or wall jump.
    SpinStart,
    /// The spring ball in the air, from the spring ball on the ground.
    SpringLaunch,
    /// A wall jump not after a wall jump.
    WallJumpStart,
}

/// The command pose selection gives with the new pose.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PoseCommand {
    /// Brake from the current speed.
    Decelerate,
    /// Stop accelerating.
    Stop,
    /// Lose the base speed.
    KillXSpeed,
    /// Lose the run speed.
    KillRunSpeed,
    /// An animation ended and handed over to this pose (`AnimationEndPose`); the arm-pump step
    /// sees [`PoseRequest::Forced`](super::PoseRequest::Forced).
    AnimationEnd,
    /// The frame's vertical result, passed back.
    Vertical(VerticalResult),
    /// Nothing.
    None,
}

/// A new pose's start and command.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PoseChange {
    /// What the start writes, if anything.
    pub start: Option<PoseStart>,
    /// The start replaced the pose with another, so the command is skipped (every turn does).
    pub replaced: bool,
    /// The command.
    pub command: PoseCommand,
    /// The new pose is a fall and the old one wasn't (`KillYIfStartedFalling`).
    pub started_falling: bool,
}

impl State {
    /// Pose selection's writes when a new pose starts: its start, then its command unless the
    /// start replaced the pose (`PoseWriteOrder`).
    pub fn pose_writes(
        &mut self,
        change: PoseChange,
        jump_held: bool,
        equipment: Equipment,
        rules: &MotionRules,
    ) {
        use {FallKind as F, LandKind as K, PoseCommand as C, VerticalResult as V};
        // PoseStartWrites
        match change.start {
            None => {}
            Some(PoseStart::Turn) => self.turn(false),
            Some(PoseStart::BallReverse | PoseStart::SpinReverse) => self.turn(true),
            Some(PoseStart::JumpOrFall) => {
                self.mode = if self.extra_run == Fixed::ZERO {
                    SpeedMode::Accelerating
                } else {
                    SpeedMode::Decelerating
                };
            }
            // AnimationEndPose: a hand-over makes no jump.
            Some(
                PoseStart::TakeOff { .. }
                | PoseStart::SpinStart
                | PoseStart::SpringLaunch
                | PoseStart::WallJumpStart,
            ) if change.command == C::AnimationEnd => {}
            Some(PoseStart::TakeOff { from_plain_crouch }) => {
                self.jump(rules, equipment);
                if from_plain_crouch {
                    let lift = Fixed::new(rules.air.crouch_jump_lift, 0);
                    self.position.y = self.position.y.minus(lift);
                }
            }
            Some(PoseStart::SpinStart | PoseStart::SpringLaunch) => self.jump(rules, equipment),
            Some(PoseStart::WallJumpStart) => {
                let speeds = rules.vertical;
                self.rise((speeds.wall_jump, speeds.wall_hi_jump), equipment);
            }
        }
        // AnimationEndPose: its command runs over a replaced pose too.
        if change.command == C::AnimationEnd {
            self.bounce = Bounce::None;
            return;
        }
        if change.replaced || matches!(change.start, Some(PoseStart::Turn)) {
            return;
        }
        // PoseCommandWrites and VerticalResultWrites
        match change.command {
            C::Decelerate if self.base_speed != Fixed::ZERO => {
                self.fold();
                self.mode = SpeedMode::Decelerating;
                self.momentum = false;
                self.kill_y(change.started_falling);
            }
            C::Decelerate | C::Stop => {
                self.mode = SpeedMode::Accelerating;
                self.momentum = false;
            }
            C::KillXSpeed => {
                self.halt();
                self.kill_y(change.started_falling);
            }
            C::KillRunSpeed => {
                self.stop_running();
                self.kill_y(change.started_falling);
            }
            C::Vertical(V::HitCeiling) => self.fall(),
            C::Vertical(V::Falling(F::Airborne | F::MorphBall | F::SpringBall)) => {
                if self.vertical != VerticalMotion::Up {
                    self.bounce = Bounce::None;
                    self.fall();
                }
            }
            C::Vertical(V::Landed(K::Grounded)) => self.land(),
            // SpringBallJump
            C::Vertical(V::Landed(K::SpringBall)) if jump_held => {
                self.bounce = Bounce::None;
                self.jump(rules, equipment);
            }
            // BallBounce
            C::Vertical(V::Landed(K::MorphBall | K::SpringBall)) => {
                let (air, first) = (rules.air, Fixed::from(rules.air.bounce));
                let fast = !Fixed::below(self.vertical_speed.pixel(), air.bounce_min_pixel);
                let (bounce, speed) = match self.bounce {
                    Bounce::None if fast => (Bounce::First, first),
                    Bounce::First => (Bounce::Second, first.minus(air.bounce_decay.into())),
                    Bounce::None | Bounce::Second => return self.land(),
                };
                self.bounce = bounce;
                self.vertical_speed = speed;
                self.vertical = VerticalMotion::Up;
            }
            C::Vertical(V::WallJump) => self.halt(),
            C::Vertical(V::None | V::Falling(F::NoChange) | V::Landed(K::NoChange))
            | C::AnimationEnd
            | C::None => {}
        }
    }

    // Fold: the run speed joins the base speed.
    const fn fold(&mut self) {
        self.base_speed = self.base_speed.plus(self.extra_run);
        self.extra_run = Fixed::ZERO;
    }

    // A turn start: Fold, then TurningAround; `cancel` is CancelSpeedBoost.
    const fn turn(&mut self, cancel: bool) {
        self.fold();
        self.momentum &= !cancel;
        self.mode = SpeedMode::TurningAround;
    }

    // KillYIfStartedFalling
    const fn kill_y(&mut self, started_falling: bool) {
        if started_falling {
            self.fall();
        }
    }

    // LandingStop, then EndVertical; extra_run is kept.
    const fn land(&mut self) {
        self.halt();
        self.end_vertical();
    }
}
