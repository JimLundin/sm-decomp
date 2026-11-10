//! One entry per spec entry: what the player sees, its kind and its switch.

use clean_types::{Checked, Kind, Switch};

/// A spec entry's id.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Behaviour {
    /// Its id.
    pub id: BehaviourId,
    /// The spec entry's summary: what the player sees.
    pub summary: &'static str,
    /// The kind the spec entry names first: mechanic, quirk, timing, output or encoding.
    pub kind: Kind,
    /// As the spec entry's `switch:` says: `none` is fixed, `parameter` a parameter (its numbers
    /// in `Rules`), a named rule a toggle.
    pub switch: Switch,
    /// How the spec checks it: by vectors, or on the original alone.
    pub checked: Checked,
}

/// The catalogue, in spec order.
pub const CATALOGUE: [Behaviour; 29] = [
    Behaviour {
        id: BehaviourId::HowSheIsDrawn,
        summary: "what her sprite shows beyond her pose's parts, from the items switched on.",
        kind: Kind::Output,
        switch: Switch::Fixed,
        checked: Checked::OriginalOnly,
    },
    Behaviour {
        id: BehaviourId::SpeedBuildsAndBrakes,
        summary: "how her horizontal speed builds towards a top speed, brakes and halts, by what she is doing.",
        kind: Kind::Mechanic,
        switch: Switch::Parameter,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::Dash,
        summary: "the extra speed she builds running with run held, and its cap, higher with the speed booster.",
        kind: Kind::Mechanic,
        switch: Switch::Parameter,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::TurningRound,
        summary: "turning round, or reversing a spin, makes her skid the old way and brake before she goes the new way.",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::AimingAndShooting,
        summary: "changing into another running pose steps her 1 px forward.",
        kind: Kind::Quirk,
        switch: Switch::Parameter,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::RidingTheElevator,
        summary: "facing the screen on a moving elevator, she goes down with it.",
        kind: Kind::Mechanic,
        switch: Switch::Parameter,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::Jumping,
        summary: "how she takes off, starts a spin or a hop, jumps again with space jump, and takes off again with jump held through a landing.",
        kind: Kind::Mechanic,
        switch: Switch::Parameter,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::FollowingTheGround,
        summary: "on the ground she is pulled down onto the floor after each horizontal move.",
        kind: Kind::Mechanic,
        switch: Switch::Parameter,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::MovesAsShePosed,
        summary: "a frame's moves follow the pose she started it in.",
        kind: Kind::Timing,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::PoseFitAfterMoves,
        summary: "when her new pose is taller, she is pushed clear of what is above or below her, or the change is refused or becomes a crouch.",
        kind: Kind::Timing,
        switch: Switch::Parameter,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::ShrinkingLowersHer,
        summary: "the frame she starts to crouch or morph, she is lowered.",
        kind: Kind::Mechanic,
        switch: Switch::Parameter,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::RisingAndFalling,
        summary: "gravity, the fall's limit, and when a rise ends.",
        kind: Kind::Mechanic,
        switch: Switch::Parameter,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::DriftInTheAir,
        summary: "how she steers, keeps or loses her drift in the air.",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::Moonfall,
        summary: "a spin entered as a turn ends falls without the limit and can't jump.",
        kind: Kind::Quirk,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::WallJump,
        summary: "a spin near a wall readies, then kicks off it.",
        kind: Kind::Mechanic,
        switch: Switch::Parameter,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::Landing,
        summary: "what touching down does to her speeds, by what she is doing, and the ball's bounces.",
        kind: Kind::Mechanic,
        switch: Switch::Parameter,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::PoseFollows,
        summary: "what decides her next pose, and when the change acts.",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::PadChart,
        summary: "the pad's rows, by what she is doing, and how a row is chosen.",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::Aiming,
        summary: "the aim a next pose takes from the held buttons.",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::WhatHerMovesMet,
        summary: "the poses that what her moves met chooses over the pad.",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::AnimationEnds,
        summary: "the poses that last as long as their animation, and what their end hands over.",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::BlocksStopFlush,
        summary: "a move into a solid block stops her flush against its face.",
        kind: Kind::Mechanic,
        switch: Switch::Parameter,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::WallsAndCeilings,
        summary: "a wall halts her; a ceiling ends her rise.",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::SolidEnemies,
        summary: "how solid enemies stop her, judged by the gap between the boxes.",
        kind: Kind::Quirk,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::SlopesCarry,
        summary: "how sloped floors and ceilings seat her, lift her, push her down and stop her.",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::SlopesSlowTheWalk,
        summary: "a sloped floor shortens the move she makes along it.",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::TouchedCells,
        summary: "which cells her moves and looks reach, and which she is inside, in order.",
        kind: Kind::Output,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::OtherBlockKinds,
        summary: "how the remaining kinds of cell meet her motion.",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::WordwiseTopSpeed,
        summary: "the top speed is tested part by part.",
        kind: Kind::Quirk,
        switch: Switch::Toggle,
        checked: Checked::ByVectors,
    },
];
