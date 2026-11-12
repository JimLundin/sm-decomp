//! One entry per spec entry: what the player sees, its kind and its switch.

use types::{Checked, Kind, Switch};

/// A spec entry's id, one per entry, named as the spec names it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BehaviourId {
    /// What her sprite shows beyond her pose (§2).
    HowSheIsDrawn,
    /// How her horizontal speed builds, brakes and halts (§3).
    SpeedBuildsAndBrakes,
    /// The extra speed she builds running with run held (§3).
    Dash,
    /// Whether she is dashing, from a run with run held until something ends it (§3).
    Dashing,
    /// How turning round or reversing a spin makes her skid (§3).
    TurningRound,
    /// The 1 px step forward into another running pose (§3).
    AimingAndShooting,
    /// Her descent facing the screen on a moving elevator (§3).
    RidingTheElevator,
    /// How she takes off, spins, hops and jumps again (§4).
    Jumping,
    /// The pull that holds her on the floor after each horizontal move (§4).
    FollowingTheGround,
    /// A frame's moves follow the pose she started it in (§4).
    MovesAsShePosed,
    /// How a taller new pose is fitted between what is above and below her (§4).
    PoseFitAfterMoves,
    /// How starting to crouch or morph lowers her, and a change of shape ends a bounce (§4).
    ShrinkingLowersHer,
    /// Gravity, the fall's limit, and when a rise ends (§5).
    RisingAndFalling,
    /// How she steers, keeps or loses her drift in the air (§5).
    DriftInTheAir,
    /// A spin entered as a turn ends, which falls without the limit (§5).
    Moonfall,
    /// How a spin near a wall readies, then kicks off it (§6).
    WallJump,
    /// What touching down does to her speeds, and the ball's bounces (§6).
    Landing,
    /// What decides her next pose, and when the change acts (§7).
    PoseFollows,
    /// The pad's rows, and how a row is chosen (§7).
    PadChart,
    /// The aim a next pose takes from the held buttons (§7).
    Aiming,
    /// The poses that what her moves met chooses over the pad (§7).
    WhatHerMovesMet,
    /// The poses that last as long as their animation, and what their end hands over (§7).
    AnimationEnds,
    /// How a move into a solid block stops her flush against its face (§8).
    BlocksStopFlush,
    /// How a wall halts her and a ceiling ends her rise, but for a change of shape's (§8).
    WallsAndCeilings,
    /// How solid enemies stop her, by the gap between the boxes (§8).
    SolidEnemies,
    /// How sloped floors and ceilings seat, lift, push and stop her (§8).
    SlopesCarry,
    /// How a sloped floor shortens the move she makes along it (§8).
    SlopesSlowTheWalk,
    /// Which cells her moves and looks reach, and which she is inside (§8).
    TouchedCells,
    /// How the remaining kinds of cell meet her motion (§8).
    OtherBlockKinds,
    /// How the top speed is tested part by part (§9).
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
pub const CATALOGUE: [Behaviour; 30] = [
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
        id: BehaviourId::Dashing,
        summary: "she dashes from a run with run held until a halt, a turn or the pad's let-go ends it; her dash's size doesn't say whether she does.",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
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
        summary: "the frame she starts to crouch or morph, she is lowered; a change of shape started from a bouncing ball ends its bounce.",
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
        summary: "a wall halts her; a ceiling ends her rise, except while her body changes shape.",
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
