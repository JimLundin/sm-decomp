//! The behaviour catalogue (MODEL §3): every named behaviour, what the player sees, the one
//! bank-log reference it was observed at, its kind and how it may be switched.
//!
//! Entries are transcribed from the behaviour specs in `docs/behaviours/`, one per spec entry;
//! the specs are their authority. This is the only file in `game` that cites the original.

/// A named behaviour.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BehaviourId {
    /// On the frame she changes pose she still moves as her old pose did.
    LocomotionAtFrameStart,
    /// Each way of moving has its own acceleration, top speed and braking.
    SpeedTable,
    /// She speeds up by the row's acceleration to its top speed, and brakes by its deceleration.
    BaseSpeed,
    /// Speed overshoots, or snaps up, to the top speed.
    SpeedCap,
    /// Turning from standing slides 0.C000 px every other frame along her facing.
    TurnCreep,
    /// Holding run while running adds speed, up to 2 px; momentum carries it elsewhere.
    ExtraRunSpeed,
    /// (invisible in vanilla) the run-speed cap is tested on the old speed, word by word.
    ExtraRunCap,
    /// The run speed's momentum is lost.
    CancelSpeedBoost,
    /// She moves sideways by her total speed; a zero move still meets a solid enemy.
    HorizontalStep,
    /// During a turn, she keeps sliding the way she was going.
    HorizontalDirection,
    /// She never moves more than 15 px sideways in a frame.
    DisplacementClamp,
    /// A stopped sideways move, on the ground or in the air, stops her speeds dead.
    WallStop,
    /// On the ground she sticks to the floor beneath her.
    StepDown,
    /// The still ball coasts and stops, unless it's in the air.
    BallAtRest,
    /// A turning Samus sinks onto lower ground instead of falling.
    TurnIgnoresFall,
    /// Landing during a crouch or morph transition ends the vertical motion.
    TransitionLanding,
    /// A vertical move tells pose selection whether she landed, fell or hit the ceiling.
    LandingContact,
    /// On a moving elevator, Samus facing the screen rides it down.
    ElevatorRide,
    /// The elevator's ride down tests the side above her.
    ElevatorHeadsUp,
    /// Shooting while standing plainly restarts the standing animation.
    StandingShotRestartsAnimation,
    /// (invisible) a block stop leaves the subpixel FFFF moving right or down, 0 moving left or up.
    FloorFlushSubpixel,
    /// (invisible) a solid enemy she already touches clears her vertical subpixel.
    EnemyTouchZeroesYSubpixel,
    /// A sideways move ends seated on the slope or under the sloped ceiling it reached.
    SlopeSeat,
    /// Changing aim while running moves her 1 px forward.
    ArmPumpStep,
    /// Letting go of jump ends the rise at once.
    JumpRelease,
    /// A rise whose speed has gone negative turns into a fall.
    StartedFalling,
    /// She rises and falls under gravity, moving by the speed she had before it.
    VerticalStep,
    /// The fall speed stops growing when its pixel word is exactly 5.
    FallCap,
    /// In the air, letting go of left and right stops her drift, unless she is braking or turning.
    NoInputStops,
    /// A spin jump drifts only while the side she faces is held.
    SpinJumpDrift,
    /// A wall jump always drifts.
    WallJumpDrift,
    /// A falling ball with neither left nor right held stops dead.
    BallDropStops,
    /// Pressing away from a wall within 8 px arms the wall jump, and a new jump press fires it.
    WallJumpCheck,
    /// (invisible) the wall jump's wall test writes the horizontal subpixel as a stop would, without moving her.
    WallProbeSubpixel,
    /// With space jump, a new jump press while falling in a spin jump jumps again.
    SpaceJump,
    /// A jump starts at the jump speed, or the hi-jump's.
    Jump,
    /// Landing as a spring ball with jump held jumps again.
    SpringBallJump,
    /// A ball landing fast bounces twice before it rests.
    BallBounce,
    /// A new pose's start sets her speeds: a turn folds them, a jump sets its speed.
    PoseStartWrites,
    /// Pose selection's commands brake, stop or kill her speeds.
    PoseCommandWrites,
    /// The frame's vertical result, passed back, stops a landing, starts a fall or ends a rise.
    VerticalResultWrites,
    /// A new pose's start comes before its command, and a replaced pose skips the command.
    PoseWriteOrder,
    /// A pose an animation hands over to makes no jump, and its command ends the bounce.
    AnimationEndPose,
}
/// What sort of behaviour an entry is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    /// A deliberate game rule.
    Mechanic,
    /// Unintended behaviour that is reproduced.
    Quirk,
    /// Slowdown, loading time, or when in a frame something takes effect.
    Timing,
    /// A quirk of the output's own format.
    Encoding,
}

/// How a behaviour may be changed from its vanilla form.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Switch {
    /// Always the vanilla behaviour.
    Fixed,
    /// A choice in `Rules` between the vanilla behaviour and its alternatives.
    Toggle,
    /// A value in `Rules` or a `data` table.
    Parameter,
    /// Changes with another behaviour's switch, having none of its own.
    Follows(BehaviourId),
}

/// The bank-log reference a behaviour was observed at: a bank and an offset in it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Source {
    /// The bank.
    pub bank: u8,
    /// The offset in the bank.
    pub offset: u16,
}

impl Source {
    /// The reference to `offset` in `bank`.
    #[must_use]
    pub const fn bank(bank: u8, offset: u16) -> Self {
        Self { bank, offset }
    }
}

/// One catalogue entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Behaviour {
    /// Its name.
    pub id: BehaviourId,
    /// What the player sees.
    pub summary: &'static str,
    /// Where it was observed.
    pub source: Source,
    /// Its kind.
    pub kind: Kind,
    /// How it may be switched.
    pub switch: Switch,
}

/// Samus's motion (`docs/behaviours/samus-motion.md`).
pub const SAMUS_MOTION: [Behaviour; 43] = [
    Behaviour {
        id: BehaviourId::LocomotionAtFrameStart,
        summary: "on the frame she changes pose she still moves as her old pose did.",
        source: Source::bank(0x90, 0xA337),
        kind: Kind::Timing,
        switch: Switch::Fixed,
    },
    Behaviour {
        id: BehaviourId::SpeedTable,
        summary: "each way of moving has its own acceleration, top speed and braking.",
        source: Source::bank(0x90, 0x9BD1),
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
    },
    Behaviour {
        id: BehaviourId::BaseSpeed,
        summary: "she speeds up by the row's acceleration to its top speed, and brakes by its deceleration.",
        source: Source::bank(0x90, 0x9A7E),
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
    },
    Behaviour {
        id: BehaviourId::SpeedCap,
        summary: "speed overshoots, or snaps up, to the top speed.",
        source: Source::bank(0x90, 0x9A99),
        kind: Kind::Quirk,
        switch: Switch::Toggle,
    },
    Behaviour {
        id: BehaviourId::TurnCreep,
        summary: "turning from standing slides 0.C000 px every other frame along her facing.",
        source: Source::bank(0x90, 0xA67C),
        kind: Kind::Quirk,
        switch: Switch::Follows(BehaviourId::SpeedCap),
    },
    Behaviour {
        id: BehaviourId::ExtraRunSpeed,
        summary: "holding run while running adds speed, up to 2 px; momentum carries it elsewhere.",
        source: Source::bank(0x90, 0x973E),
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
    },
    Behaviour {
        id: BehaviourId::ExtraRunCap,
        summary: "(invisible in vanilla) the run-speed cap is tested on the old speed, word by word.",
        source: Source::bank(0x90, 0x97D5),
        kind: Kind::Quirk,
        switch: Switch::Fixed,
    },
    Behaviour {
        id: BehaviourId::CancelSpeedBoost,
        summary: "the run speed's momentum is lost.",
        source: Source::bank(0x91, 0xDE53),
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
    },
    Behaviour {
        id: BehaviourId::HorizontalStep,
        summary: "she moves sideways by her total speed; a zero move still meets a solid enemy.",
        source: Source::bank(0x90, 0x8EA9),
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
    },
    Behaviour {
        id: BehaviourId::HorizontalDirection,
        summary: "during a turn, she keeps sliding the way she was going.",
        source: Source::bank(0x90, 0x8EA9),
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
    },
    Behaviour {
        id: BehaviourId::DisplacementClamp,
        summary: "she never moves more than 15 px sideways in a frame.",
        source: Source::bank(0x90, 0xE4AD),
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
    },
    Behaviour {
        id: BehaviourId::WallStop,
        summary: "a stopped sideways move, on the ground or in the air, stops her speeds dead.",
        source: Source::bank(0x90, 0xE5CE),
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
    },
    Behaviour {
        id: BehaviourId::StepDown,
        summary: "on the ground she sticks to the floor beneath her.",
        source: Source::bank(0x90, 0x923F),
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
    },
    Behaviour {
        id: BehaviourId::BallAtRest,
        summary: "the still ball coasts and stops, unless it's in the air.",
        source: Source::bank(0x90, 0xA521),
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
    },
    Behaviour {
        id: BehaviourId::TurnIgnoresFall,
        summary: "a turning Samus sinks onto lower ground instead of falling.",
        source: Source::bank(0x90, 0xA68F),
        kind: Kind::Quirk,
        switch: Switch::Fixed,
    },
    Behaviour {
        id: BehaviourId::TransitionLanding,
        summary: "landing during a crouch or morph transition ends the vertical motion.",
        source: Source::bank(0x90, 0xA640),
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
    },
    Behaviour {
        id: BehaviourId::LandingContact,
        summary: "a vertical move tells pose selection whether she landed, fell or hit the ceiling.",
        source: Source::bank(0x90, 0xE61B),
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
    },
    Behaviour {
        id: BehaviourId::ElevatorRide,
        summary: "on a moving elevator, Samus facing the screen rides it down.",
        source: Source::bank(0x90, 0xA392),
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
    },
    Behaviour {
        id: BehaviourId::ElevatorHeadsUp,
        summary: "the elevator's ride down tests the side above her.",
        source: Source::bank(0x90, 0xA397),
        kind: Kind::Quirk,
        switch: Switch::Fixed,
    },
    Behaviour {
        id: BehaviourId::StandingShotRestartsAnimation,
        summary: "shooting while standing plainly restarts the standing animation.",
        source: Source::bank(0x90, 0xA3AD),
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
    },
    Behaviour {
        id: BehaviourId::FloorFlushSubpixel,
        summary: "(invisible) a block stop leaves the subpixel FFFF moving right or down, 0 moving left or up.",
        source: Source::bank(0x94, 0x8F82),
        kind: Kind::Quirk,
        switch: Switch::Fixed,
    },
    Behaviour {
        id: BehaviourId::EnemyTouchZeroesYSubpixel,
        summary: "(invisible) a solid enemy she already touches clears her vertical subpixel.",
        source: Source::bank(0xA0, 0xAAC8),
        kind: Kind::Quirk,
        switch: Switch::Fixed,
    },
    Behaviour {
        id: BehaviourId::SlopeSeat,
        summary: "a sideways move ends seated on the slope or under the sloped ceiling it reached.",
        source: Source::bank(0x90, 0x9376),
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
    },
    Behaviour {
        id: BehaviourId::ArmPumpStep,
        summary: "changing aim while running moves her 1 px forward.",
        source: Source::bank(0x91, 0xEADE),
        kind: Kind::Quirk,
        switch: Switch::Fixed,
    },
    Behaviour {
        id: BehaviourId::JumpRelease,
        summary: "letting go of jump ends the rise at once.",
        source: Source::bank(0x90, 0x8FDC),
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
    },
    Behaviour {
        id: BehaviourId::StartedFalling,
        summary: "a rise whose speed has gone negative turns into a fall.",
        source: Source::bank(0x90, 0x90C4),
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
    },
    Behaviour {
        id: BehaviourId::VerticalStep,
        summary: "she rises and falls under gravity, moving by the speed she had before it.",
        source: Source::bank(0x90, 0x90E2),
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
    },
    Behaviour {
        id: BehaviourId::FallCap,
        summary: "the fall speed stops growing when its pixel word is exactly 5.",
        source: Source::bank(0x90, 0x910C),
        kind: Kind::Quirk,
        switch: Switch::Fixed,
    },
    Behaviour {
        id: BehaviourId::NoInputStops,
        summary: "in the air, letting go of left and right stops her drift, unless she is braking or turning.",
        source: Source::bank(0x90, 0x900D),
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
    },
    Behaviour {
        id: BehaviourId::SpinJumpDrift,
        summary: "a spin jump drifts only while the side she faces is held.",
        source: Source::bank(0x90, 0x906C),
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
    },
    Behaviour {
        id: BehaviourId::WallJumpDrift,
        summary: "a wall jump always drifts.",
        source: Source::bank(0x90, 0x902D),
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
    },
    Behaviour {
        id: BehaviourId::BallDropStops,
        summary: "a falling ball with neither left nor right held stops dead.",
        source: Source::bank(0x90, 0xA5CD),
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
    },
    Behaviour {
        id: BehaviourId::WallJumpCheck,
        summary: "pressing away from a wall within 8 px arms the wall jump, and a new jump press fires it.",
        source: Source::bank(0x90, 0x9D35),
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
    },
    Behaviour {
        id: BehaviourId::WallProbeSubpixel,
        summary: "(invisible) the wall jump's wall test writes the horizontal subpixel as a stop would, without moving her.",
        source: Source::bank(0x94, 0x967F),
        kind: Kind::Quirk,
        switch: Switch::Fixed,
    },
    Behaviour {
        id: BehaviourId::SpaceJump,
        summary: "with space jump, a new jump press while falling in a spin jump jumps again.",
        source: Source::bank(0x90, 0xA488),
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
    },
    Behaviour {
        id: BehaviourId::Jump,
        summary: "a jump starts at the jump speed, or the hi-jump's.",
        source: Source::bank(0x90, 0x98BC),
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
    },
    Behaviour {
        id: BehaviourId::SpringBallJump,
        summary: "landing as a spring ball with jump held jumps again.",
        source: Source::bank(0x91, 0xF25E),
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
    },
    Behaviour {
        id: BehaviourId::BallBounce,
        summary: "a ball landing fast bounces twice before it rests.",
        source: Source::bank(0x91, 0xF1FC),
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
    },
    Behaviour {
        id: BehaviourId::PoseStartWrites,
        summary: "a new pose's start sets her speeds: a turn folds them, a jump sets its speed.",
        source: Source::bank(0x91, 0xF543),
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
    },
    Behaviour {
        id: BehaviourId::PoseCommandWrites,
        summary: "pose selection's commands brake, stop or kill her speeds.",
        source: Source::bank(0x91, 0xEC50),
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
    },
    Behaviour {
        id: BehaviourId::VerticalResultWrites,
        summary: "the frame's vertical result, passed back, stops a landing, starts a fall or ends a rise.",
        source: Source::bank(0x91, 0xEFC4),
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
    },
    Behaviour {
        id: BehaviourId::PoseWriteOrder,
        summary: "a new pose's start comes before its command, and a replaced pose skips the command.",
        source: Source::bank(0x91, 0xEB88),
        kind: Kind::Timing,
        switch: Switch::Fixed,
    },
    Behaviour {
        id: BehaviourId::AnimationEndPose,
        summary: "a pose an animation hands over to makes no jump, and its command ends the bounce.",
        source: Source::bank(0x91, 0xEBAF),
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
    },
];
