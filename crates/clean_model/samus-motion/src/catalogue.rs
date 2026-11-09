//! One entry per spec entry: what the player sees, its kind and its switch.

use clean_types::{Checked, Kind, Switch};

/// A spec entry's id.
#[derive(Debug)]
pub enum BehaviourId {
    /// §2.
    HowSheIsDrawn,
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
    /// §4.
    ShrinkingLowersHer,
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
pub struct Behaviour {
    /// Its id.
    pub id: BehaviourId,
    /// What the player sees.
    pub summary: &'static str,
    /// Mechanic, quirk, timing or encoding.
    pub kind: Kind,
    /// Fixed, toggle or parameter.
    pub switch: Switch,
    /// How the spec checks it: by vectors, or on the original alone.
    pub checked: Checked,
}

/// The catalogue, in spec order.
pub const CATALOGUE: [Behaviour; 29] = [
    Behaviour {
        id: BehaviourId::HowSheIsDrawn,
        summary: "Her spin is drawn as the screw attack or the space jump, and facing the screen she wears her suit",
        kind: Kind::Output,
        switch: Switch::Fixed,
        checked: Checked::OriginalOnly,
    },
    Behaviour {
        id: BehaviourId::SpeedBuildsAndBrakes,
        summary: "Her speed builds to a top by what she is doing, and brakes when she carries on along the ground",
        kind: Kind::Mechanic,
        switch: Switch::Parameter,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::Dash,
        summary: "Running with run held builds a dash up to 2 px a frame, carried into jumps and the ball",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::TurningRound,
        summary: "Turning round she skids the old way with her dash added, then creeps the new way",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::AimingAndShooting,
        summary: "Changing into another running pose, or running on let go, moves her 1 px forward; shooting standing restarts the animation",
        kind: Kind::Quirk,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::RidingTheElevator,
        summary: "Facing the screen on a moving elevator she moves down 1 px a frame, stopped as any downward move",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::Jumping,
        summary: "Jumps, wall jumps, ball bounces and space jumps start her rising at their speeds, and jump held through a landing or a turn's end takes her off again",
        kind: Kind::Mechanic,
        switch: Switch::Parameter,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::FollowingTheGround,
        summary: "On the ground she is pulled down onto the floor, keeps a vertical speed she brought, and walks off a ledge into a fall",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::MovesAsShePosed,
        summary: "On the frame her pose changes she moves as the pose at the frame's start",
        kind: Kind::Timing,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::PoseFitAfterMoves,
        summary: "A frame ends: her moves, what they met, then the pose change: pose fit's push or refusal, which places her, speeds, the lift",
        kind: Kind::Timing,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::ShrinkingLowersHer,
        summary: "Starting to crouch or to morph lowers her 5 or 9 px, or only to a floor nearer under her, and any change of shape places her",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::RisingAndFalling,
        summary: "She rises and falls under gravity, settling at 5.1400 a frame, a let-go jump ends the rise, and each frame says whether she is rising",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::DriftInTheAir,
        summary: "In the air she drifts towards a top speed, and stops drifting when the pad is let go",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::Moonfall,
        summary: "A spin entered as a turn ends drops with no rise and no falling limit until she lands",
        kind: Kind::Quirk,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::WallJump,
        summary: "A spin carried on near a wall readies a wall jump, and jump kicks her off it",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::Landing,
        summary: "Falling onto a floor she lands; the ball bounces twice before it rests",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::PoseFollows,
        summary: "Her pose changes at most once a frame: an animation's end, then what she met, then the pad",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::PadChart,
        summary: "The pad chart's fitting row with a just-pressed button, then the most buttons, wins",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::Aiming,
        summary: "The held buttons give a next pose's aim by the rule its row names",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::WhatHerMovesMet,
        summary: "A wall, the aim step's block, a kick, a floor, no floor or a ceiling chooses her pose over the pad",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::AnimationEnds,
        summary: "A pose that lasts as long as its animation hands over the next pose at its end",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::BlocksStopFlush,
        summary: "A move into a solid block stops her with her leading edge against its face",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::WallsAndCeilings,
        summary: "A wall halts her; a ceiling ends her rise",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::SolidEnemies,
        summary: "A solid enemy she reaches decides a move before blocks; one she touches clears her y subpixel",
        kind: Kind::Quirk,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::SlopesCarry,
        summary: "A slope lifts her onto it after a horizontal move, and seated on one her ground pull is 1 px",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::SlopesSlowTheWalk,
        summary: "With no vertical speed, a sloped floor at her leading edge shortens her move by its factor",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::TouchedCells,
        summary: "A frame reports the cells her moves reach, in order, and the cells she is inside",
        kind: Kind::Output,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::OtherBlockKinds,
        summary: "Doors, spikes, shot, bomb and reacting cells meet her as solid, as air, or as they answer",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::WordwiseTopSpeed,
        summary: "The top speed is tested part by part, so some speeds snap up and some are kept above it",
        kind: Kind::Quirk,
        switch: Switch::Toggle,
        checked: Checked::ByVectors,
    },
];
