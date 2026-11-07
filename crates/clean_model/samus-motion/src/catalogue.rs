//! One entry per spec entry: what the player sees, its kind and its switch.

use clean_types::{Kind, Switch};

/// A spec entry's id.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BehaviourId {
    /// §3.
    SpeedBuildsAndBrakes,
    /// §3.
    Dash,
    /// §3.
    TurningRound,
    /// §3.
    AimingAndShooting,
    /// §3.
    RidingTheElevator,
    /// §4.
    Jumping,
    /// §4.
    FollowingTheGround,
    /// §4.
    MovesAsShePosed,
    /// §5.
    RisingAndFalling,
    /// §5.
    DriftInTheAir,
    /// §5.
    Moonfall,
    /// §6.
    WallJump,
    /// §6.
    Landing,
    /// §8.
    BlocksStopFlush,
    /// §8.
    WallsAndCeilings,
    /// §8.
    SolidEnemies,
    /// §8.
    SlopesCarry,
    /// §9.
    WordwiseTopSpeed,
}

/// A catalogue entry.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Behaviour {
    /// Its id.
    pub id: BehaviourId,
    /// What the player sees.
    pub summary: &'static str,
    /// Mechanic, quirk, timing or encoding.
    pub kind: Kind,
    /// Fixed, toggle or parameter.
    pub switch: Switch,
}

const fn entry(id: BehaviourId, summary: &'static str, kind: Kind, switch: Switch) -> Behaviour {
    Behaviour {
        id,
        summary,
        kind,
        switch,
    }
}

/// The catalogue, in spec order.
pub const CATALOGUE: [Behaviour; 18] = [
    entry(
        BehaviourId::SpeedBuildsAndBrakes,
        "Her speed builds to a top by what she is doing, and brakes when she carries on along the ground",
        Kind::Mechanic,
        Switch::Parameter,
    ),
    entry(
        BehaviourId::Dash,
        "Running with run held builds a dash up to 2 px a frame, carried into jumps and the ball",
        Kind::Mechanic,
        Switch::Fixed,
    ),
    entry(
        BehaviourId::TurningRound,
        "Turning round she skids the old way with her dash added, then creeps the new way",
        Kind::Mechanic,
        Switch::Fixed,
    ),
    entry(
        BehaviourId::AimingAndShooting,
        "Changing aim while running moves her 1 px forward; shooting standing restarts the animation",
        Kind::Quirk,
        Switch::Fixed,
    ),
    entry(
        BehaviourId::RidingTheElevator,
        "Facing the screen on a moving elevator she moves down 1 px a frame, stopped as any downward move",
        Kind::Mechanic,
        Switch::Fixed,
    ),
    entry(
        BehaviourId::Jumping,
        "Jumps, wall jumps, ball bounces and space jumps start her rising at their speeds",
        Kind::Mechanic,
        Switch::Parameter,
    ),
    entry(
        BehaviourId::FollowingTheGround,
        "On the ground she is pulled down onto the floor, and walks off a ledge into a fall",
        Kind::Mechanic,
        Switch::Fixed,
    ),
    entry(
        BehaviourId::MovesAsShePosed,
        "On the frame her pose changes she moves as the pose at the frame's start",
        Kind::Timing,
        Switch::Fixed,
    ),
    entry(
        BehaviourId::RisingAndFalling,
        "She rises and falls under gravity, settling at 5.1400 a frame, and a let-go jump ends the rise",
        Kind::Mechanic,
        Switch::Fixed,
    ),
    entry(
        BehaviourId::DriftInTheAir,
        "In the air she drifts towards a top speed, and stops drifting when the pad is let go",
        Kind::Mechanic,
        Switch::Fixed,
    ),
    entry(
        BehaviourId::Moonfall,
        "A spin entered as a turn ends drops with no rise and no falling limit until she lands",
        Kind::Quirk,
        Switch::Fixed,
    ),
    entry(
        BehaviourId::WallJump,
        "A spin carried on near a wall readies a wall jump, and jump kicks her off it",
        Kind::Mechanic,
        Switch::Fixed,
    ),
    entry(
        BehaviourId::Landing,
        "Falling onto a floor she lands; the ball bounces twice before it rests",
        Kind::Mechanic,
        Switch::Fixed,
    ),
    entry(
        BehaviourId::BlocksStopFlush,
        "A move into a solid block stops her with her leading edge against its face",
        Kind::Mechanic,
        Switch::Fixed,
    ),
    entry(
        BehaviourId::WallsAndCeilings,
        "A wall halts her; a ceiling ends her rise",
        Kind::Mechanic,
        Switch::Fixed,
    ),
    entry(
        BehaviourId::SolidEnemies,
        "A solid enemy she reaches decides a move before blocks; one she touches clears her y subpixel",
        Kind::Quirk,
        Switch::Fixed,
    ),
    entry(
        BehaviourId::SlopesCarry,
        "A slope lifts her onto it after a horizontal move, and seated on one her ground pull is 1 px",
        Kind::Mechanic,
        Switch::Fixed,
    ),
    entry(
        BehaviourId::WordwiseTopSpeed,
        "The top speed is tested part by part, so some speeds snap up and some are kept above it",
        Kind::Quirk,
        Switch::Toggle,
    ),
];
