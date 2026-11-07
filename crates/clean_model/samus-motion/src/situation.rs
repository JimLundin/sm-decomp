//! Her situation, classified once a frame from her pose at its start (`MovesAsShePosed`).
//!
//! A situation says what she is doing, how she moves, and the pad chart's rows for it. Each is a
//! state of the pose chart's state machine (§7); its rows are the moves out of it
//! (`chart::table`).

use crate::chart::{Row, table};
use clean_types::Action::{
    AgainstWall, Crouching, Falling, Jumping, Landing, Moonwalking, Morphing, Rolling, Running,
    Spinning, StandingUp, Still, TakingOff, Turning, Unmorphing, WallJumping,
};
use clean_types::Body::{Ball, Crouched, SpringBall, Upright};
use clean_types::{Aim, Pose, Turn};

/// What she is doing, as the spec's tables name it ("She is", "She was").
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Doing {
    /// Standing still, or landing.
    Standing,
    /// Running.
    Running,
    /// Taking off: the first poses of a straight jump.
    TakingOff,
    /// In a straight jump, upright.
    StraightJump,
    /// Spinning.
    Spinning,
    /// Falling, upright.
    Falling,
    /// Wall jumping.
    WallJumping,
    /// Moonwalking.
    Moonwalking,
    /// Against the wall.
    AgainstWall,
    /// Crouched.
    Crouched,
    /// A ball, on the ground or in the air.
    Ball,
    /// The spring ball's hop.
    Hop,
    /// Turning, on the ground or in the air.
    Turning,
    /// Her body changing shape: crouching, standing up, morphing or unmorphing.
    Shaping,
}

/// `SpeedBuildsAndBrakes`: the row of its table whose speeds she builds by.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Gait {
    /// Running.
    Running,
    /// Rolling as a ball.
    Rolling,
    /// Moonwalking.
    Moonwalking,
    /// In a straight jump or the spring ball's hop.
    StraightJump,
    /// In a spin jump or a wall jump.
    SpinJump,
    /// Falling, upright or as a ball.
    Falling,
    /// Turning round.
    Turning,
}

/// How her speed builds as she moves by it.
#[derive(Clone, Copy)]
pub enum Drift {
    /// `SpeedBuildsAndBrakes`: following the pad she builds; carrying on she brakes.
    Ground,
    /// `DriftInTheAir`: a straight jump or a fall; let go, she stops drifting.
    Air,
    /// `DriftInTheAir`: the ball in the air; let go, it halts but still moves by its dash.
    BallAir,
    /// `DriftInTheAir`: a spin jump, drifting only while she holds forward, skids or tops out.
    Spin,
    /// `DriftInTheAir`: a wall jump, always drifting.
    WallJump,
}

/// How she moves horizontally in her situation (`SpeedBuildsAndBrakes`).
#[derive(Clone, Copy)]
pub enum Moves {
    /// Her speed builds by a gait's limits as the drift says, and she moves by it and her dash.
    Build(Gait, Drift),
    /// Standing, against the wall, crouched: her dash only, then a halt.
    DashThenHalt,
    /// The ball at rest on the ground: braking or skidding it rolls on, keeping its dash; else
    /// as standing.
    Rest,
    /// `Jumping`: taking off, her dash only, a dash to lose gone first, following the pad.
    TakeOff,
    /// While her body changes shape: her dash only, keeping both speeds.
    Dash,
}

/// A state of the pose chart: what she is doing, how she moves, and its rows.
pub struct Situation {
    /// What she is doing.
    pub doing: Doing,
    /// How she moves.
    pub moves: Moves,
    /// Its rows of the pad chart (`PadChart`).
    pub rows: &'static [Row],
}

/// Her situation, from her pose at the frame's start and whether she is on the ground.
#[must_use]
pub fn classify(pose: Pose, on_ground: bool) -> &'static Situation {
    match (pose.body, pose.action) {
        (_, Crouching | StandingUp | Morphing | Unmorphing) => &SHAPING,
        (Ball | SpringBall, Jumping) => &HOP,
        (Ball | SpringBall, Rolling) if on_ground => &ROLLING,
        (Ball | SpringBall, Still) if on_ground => &RESTING,
        (Ball | SpringBall, _) => &BALL_IN_AIR,
        (Crouched, Turning(_)) | (Upright, Turning(Turn::Jump | Turn::Fall)) => &TURNING_ON,
        (Upright, Turning(_)) if pose.aim == Aim::UpForward => &TURNING_ON,
        (Upright, Turning(_)) => &TURNING,
        (Crouched, _) => &CROUCHED,
        (Upright, Still | Landing(_) | Rolling) => &STANDING,
        (Upright, Running) => &RUNNING,
        (Upright, TakingOff) => &TAKING_OFF,
        (Upright, Jumping) => &STRAIGHT_JUMP,
        (Upright, Spinning) => &SPINNING,
        (Upright, Falling) => &FALLING,
        (Upright, Moonwalking) => &MOONWALKING,
        (Upright, WallJumping) => &WALL_JUMPING,
        (Upright, AgainstWall) => &AGAINST_WALL,
    }
}

/// Standing still, or landing.
static STANDING: Situation = Situation {
    doing: Doing::Standing,
    moves: Moves::DashThenHalt,
    rows: table::STANDING,
};

/// Running.
static RUNNING: Situation = Situation {
    doing: Doing::Running,
    moves: Moves::Build(Gait::Running, Drift::Ground),
    rows: table::RUNNING,
};

/// Taking off.
static TAKING_OFF: Situation = Situation {
    doing: Doing::TakingOff,
    moves: Moves::TakeOff,
    rows: table::TAKING_OFF,
};

/// In a straight jump.
static STRAIGHT_JUMP: Situation = Situation {
    doing: Doing::StraightJump,
    moves: Moves::Build(Gait::StraightJump, Drift::Air),
    rows: table::STRAIGHT_JUMP,
};

/// Spinning.
static SPINNING: Situation = Situation {
    doing: Doing::Spinning,
    moves: Moves::Build(Gait::SpinJump, Drift::Spin),
    rows: table::SPINNING,
};

/// Falling, upright.
static FALLING: Situation = Situation {
    doing: Doing::Falling,
    moves: Moves::Build(Gait::Falling, Drift::Air),
    rows: table::FALLING,
};

/// Wall jumping.
static WALL_JUMPING: Situation = Situation {
    doing: Doing::WallJumping,
    moves: Moves::Build(Gait::SpinJump, Drift::WallJump),
    rows: table::WALL_JUMPING,
};

/// Moonwalking.
static MOONWALKING: Situation = Situation {
    doing: Doing::Moonwalking,
    moves: Moves::Build(Gait::Moonwalking, Drift::Ground),
    rows: table::MOONWALKING,
};

/// Against the wall.
static AGAINST_WALL: Situation = Situation {
    doing: Doing::AgainstWall,
    moves: Moves::DashThenHalt,
    rows: table::AGAINST_WALL,
};

/// Crouched.
static CROUCHED: Situation = Situation {
    doing: Doing::Crouched,
    moves: Moves::DashThenHalt,
    rows: table::CROUCHED,
};

/// A ball rolling on the ground.
static ROLLING: Situation = Situation {
    doing: Doing::Ball,
    moves: Moves::Build(Gait::Rolling, Drift::Ground),
    rows: table::BALL,
};

/// The ball at rest on the ground.
static RESTING: Situation = Situation {
    doing: Doing::Ball,
    moves: Moves::Rest,
    rows: table::BALL,
};

/// A ball in the air.
static BALL_IN_AIR: Situation = Situation {
    doing: Doing::Ball,
    moves: Moves::Build(Gait::Falling, Drift::BallAir),
    rows: table::BALL,
};

/// The spring ball's hop.
static HOP: Situation = Situation {
    doing: Doing::Hop,
    moves: Moves::Build(Gait::StraightJump, Drift::BallAir),
    rows: table::BALL,
};

/// Turning on the ground or from the moonwalk's jump, not aimed `UpForward`.
static TURNING: Situation = Situation {
    doing: Doing::Turning,
    moves: Moves::Build(Gait::Turning, Drift::Ground),
    rows: table::TURNING,
};

/// Turning in a jump or a fall, crouched turning, or turning on the ground or from the
/// moonwalk's jump aimed `UpForward`.
static TURNING_ON: Situation = Situation {
    doing: Doing::Turning,
    moves: Moves::Build(Gait::Turning, Drift::Ground),
    rows: table::TURNING_ON,
};

/// Her body changing shape.
static SHAPING: Situation = Situation {
    doing: Doing::Shaping,
    moves: Moves::Dash,
    rows: table::SHAPING,
};
