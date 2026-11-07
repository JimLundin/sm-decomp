//! One entry per spec entry: what the player sees, its kind and its switch.

use clean_types::Kind::{self, Mechanic, Quirk, Timing};
use clean_types::Switch::{self, Fixed, Parameter, Toggle};

/// A spec entry's id.
#[derive(Clone, Copy, Debug)]
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
    /// §4.
    PoseFitAfterMoves,
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
    /// §7.
    PoseFollows,
    /// §7.
    PadChart,
    /// §7.
    Aiming,
    /// §7.
    WhatHerMovesMet,
    /// §7.
    AnimationEnds,
    /// §8.
    BlocksStopFlush,
    /// §8.
    WallsAndCeilings,
    /// §8.
    SolidEnemies,
    /// §8.
    SlopesCarry,
    /// §8.
    SlopesSlowTheWalk,
    /// §8.
    TouchedCells,
    /// §8.
    OtherBlockKinds,
    /// §9.
    WordwiseTopSpeed,
}

/// A catalogue entry.
#[derive(Clone, Copy)]
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

const fn e(id: BehaviourId, summary: &'static str, kind: Kind, switch: Switch) -> Behaviour {
    Behaviour {
        id,
        summary,
        kind,
        switch,
    }
}

#[cfg_attr(not(test), expect(clippy::enum_glob_use))]
use BehaviourId::*;

/// The catalogue, in spec order.
pub const CATALOGUE: [Behaviour; 27] = [
    e(
        SpeedBuildsAndBrakes,
        "Her speed builds to a top by what she is doing, and brakes when she carries on along the ground",
        Mechanic,
        Parameter,
    ),
    e(
        Dash,
        "Running with run held builds a dash up to 2 px a frame, carried into jumps and the ball",
        Mechanic,
        Fixed,
    ),
    e(
        TurningRound,
        "Turning round she skids the old way with her dash added, then creeps the new way",
        Mechanic,
        Fixed,
    ),
    e(
        AimingAndShooting,
        "Changing aim while running moves her 1 px forward; shooting standing restarts the animation",
        Quirk,
        Fixed,
    ),
    e(
        RidingTheElevator,
        "Facing the screen on a moving elevator she moves down 1 px a frame, stopped as any downward move",
        Mechanic,
        Fixed,
    ),
    e(
        Jumping,
        "Jumps, wall jumps, ball bounces and space jumps start her rising at their speeds",
        Mechanic,
        Parameter,
    ),
    e(
        FollowingTheGround,
        "On the ground she is pulled down onto the floor, and walks off a ledge into a fall",
        Mechanic,
        Fixed,
    ),
    e(
        MovesAsShePosed,
        "On the frame her pose changes she moves as the pose at the frame's start",
        Timing,
        Fixed,
    ),
    e(
        PoseFitAfterMoves,
        "A frame ends: her moves, what they met, then the pose change: pose fit, speeds, the lift",
        Timing,
        Fixed,
    ),
    e(
        RisingAndFalling,
        "She rises and falls under gravity, settling at 5.1400 a frame, and a let-go jump ends the rise",
        Mechanic,
        Fixed,
    ),
    e(
        DriftInTheAir,
        "In the air she drifts towards a top speed, and stops drifting when the pad is let go",
        Mechanic,
        Fixed,
    ),
    e(
        Moonfall,
        "A spin entered as a turn ends drops with no rise and no falling limit until she lands",
        Quirk,
        Fixed,
    ),
    e(
        WallJump,
        "A spin carried on near a wall readies a wall jump, and jump kicks her off it",
        Mechanic,
        Fixed,
    ),
    e(
        Landing,
        "Falling onto a floor she lands; the ball bounces twice before it rests",
        Mechanic,
        Fixed,
    ),
    e(
        PoseFollows,
        "Her pose changes at most once a frame: an animation's end, then what she met, then the pad",
        Mechanic,
        Fixed,
    ),
    e(
        PadChart,
        "The pad chart's fitting row with a just-pressed button, then the most buttons, wins",
        Mechanic,
        Fixed,
    ),
    e(
        Aiming,
        "The held buttons give a next pose's aim by the rule its row names",
        Mechanic,
        Fixed,
    ),
    e(
        WhatHerMovesMet,
        "A wall, the aim step's block, a kick, a floor, no floor or a ceiling chooses her pose over the pad",
        Mechanic,
        Fixed,
    ),
    e(
        AnimationEnds,
        "A pose that lasts as long as its animation hands over the next pose at its end",
        Mechanic,
        Fixed,
    ),
    e(
        BlocksStopFlush,
        "A move into a solid block stops her with her leading edge against its face",
        Mechanic,
        Fixed,
    ),
    e(
        WallsAndCeilings,
        "A wall halts her; a ceiling ends her rise",
        Mechanic,
        Fixed,
    ),
    e(
        SolidEnemies,
        "A solid enemy she reaches decides a move before blocks; one she touches clears her y subpixel",
        Quirk,
        Fixed,
    ),
    e(
        SlopesCarry,
        "A slope lifts her onto it after a horizontal move, and seated on one her ground pull is 1 px",
        Mechanic,
        Fixed,
    ),
    e(
        SlopesSlowTheWalk,
        "With no vertical speed, a sloped floor at her leading edge shortens her move by its factor",
        Mechanic,
        Fixed,
    ),
    e(
        TouchedCells,
        "A frame reports the cells her moves reach, in order, and the cells she is inside",
        Mechanic,
        Fixed,
    ),
    e(
        OtherBlockKinds,
        "Doors, spikes, shot, bomb and reacting cells meet her as solid, as air, or as they answer",
        Mechanic,
        Fixed,
    ),
    e(
        WordwiseTopSpeed,
        "The top speed is tested part by part, so some speeds snap up and some are kept above it",
        Quirk,
        Toggle,
    ),
];
