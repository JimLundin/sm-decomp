//! The behaviour catalogue (MODEL §3): every named behaviour, what the player sees, the one
//! bank-log reference it was observed at, its kind and how it may be switched.
//!
//! Entries are transcribed from the behaviour specs in `docs/behaviours/`, one per spec entry;
//! the specs are their authority. This is the only file in `game` that cites the original.

/// A named behaviour.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BehaviourId {
    /// On the frame Samus changes pose, she still moves the way her old pose moved.
    LocomotionAtFrameStart,
    /// Each way of moving has its own acceleration, top speed and braking.
    SpeedTableRow,
    /// Samus speeds up by the row's acceleration each frame, up to its top speed.
    BaseSpeedAccelerate,
    /// Speed briefly overshoots, or snaps *up* to, the top speed.
    SpeedCap,
    /// During the on-ground turn from standing, Samus slides 0.C000 px every other frame.
    TurnCreep,
    /// When Samus brakes or turns, her speed falls by the row's deceleration.
    BaseSpeedDecelerate,
    /// Holding run while running adds speed, up to 2 px a frame; a kept momentum carries it into other rows.
    ExtraRunSpeed,
    /// (invisible in vanilla play) the run-speed cap is tested before the add, word by word.
    ExtraRunCap,
    /// Run speed carries through a jump and is lost when she stops.
    CancelSpeedBoost,
    /// Samus moves sideways by her total speed.
    HorizontalStep,
    /// During a turn, Samus keeps sliding the way she was going.
    HorizontalDirection,
    /// Samus never moves more than 15 px sideways in a frame.
    DisplacementClamp,
    /// Running into a wall stops her dead, flush against it.
    WallStop,
    /// Walking off a ledge or down a slope, Samus sticks to the ground beneath her.
    StepDown,
    /// The still morph ball sheds any speed it has, unless it's in the air.
    BallAtRest,
    /// A turning Samus sinks onto lower ground instead of falling.
    TurnIgnoresFall,
    /// Landing during a crouch or morph transition ends the vertical motion.
    TransitionLanding,
    /// Falling off the ground tells pose selection which fall to start.
    LandingContact,
    /// On a moving elevator, Samus facing the screen rides it down.
    ElevatorRide,
    /// Shooting while standing plainly restarts the standing animation.
    StandingShotRestartsAnimation,
    /// (invisible) flush against a floor her vertical subpixel is FFFF; against a wall, FFFF or 0 by side.
    FloorFlushSubpixel,
    /// Changing aim while running moves Samus an extra pixel forward.
    ArmPumpStep,
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

/// Samus on the ground (`docs/behaviours/samus-ground.md`).
pub const SAMUS_GROUND: [Behaviour; 22] = [
    Behaviour {
        id: BehaviourId::LocomotionAtFrameStart,
        summary: "on the frame Samus changes pose, she still moves the way her old pose moved.",
        source: Source::bank(0x90, 0xA337),
        kind: Kind::Timing,
        switch: Switch::Fixed,
    },
    Behaviour {
        id: BehaviourId::SpeedTableRow,
        summary: "each way of moving has its own acceleration, top speed and braking.",
        source: Source::bank(0x90, 0x9BD1),
        kind: Kind::Mechanic,
        switch: Switch::Parameter,
    },
    Behaviour {
        id: BehaviourId::BaseSpeedAccelerate,
        summary: "Samus speeds up by the row's acceleration each frame, up to its top speed.",
        source: Source::bank(0x90, 0x9A7E),
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
    },
    Behaviour {
        id: BehaviourId::SpeedCap,
        summary: "speed briefly overshoots, or snaps *up* to, the top speed.",
        source: Source::bank(0x90, 0x9A99),
        kind: Kind::Quirk,
        switch: Switch::Toggle,
    },
    Behaviour {
        id: BehaviourId::TurnCreep,
        summary: "during the on-ground turn from standing, Samus slides 0.C000 px every other frame.",
        source: Source::bank(0x90, 0xA67C),
        kind: Kind::Quirk,
        switch: Switch::Follows(BehaviourId::SpeedCap),
    },
    Behaviour {
        id: BehaviourId::BaseSpeedDecelerate,
        summary: "when Samus brakes or turns, her speed falls by the row's deceleration.",
        source: Source::bank(0x90, 0x9ABA),
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
    },
    Behaviour {
        id: BehaviourId::ExtraRunSpeed,
        summary: "holding run while running adds speed, up to 2 px a frame; a kept momentum carries it into other rows.",
        source: Source::bank(0x90, 0x973E),
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
    },
    Behaviour {
        id: BehaviourId::ExtraRunCap,
        summary: "(invisible in vanilla play) the run-speed cap is tested before the add, word by word.",
        source: Source::bank(0x90, 0x97D5),
        kind: Kind::Quirk,
        switch: Switch::Fixed,
    },
    Behaviour {
        id: BehaviourId::CancelSpeedBoost,
        summary: "run speed carries through a jump and is lost when she stops.",
        source: Source::bank(0x91, 0xDE53),
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
    },
    Behaviour {
        id: BehaviourId::HorizontalStep,
        summary: "Samus moves sideways by her total speed.",
        source: Source::bank(0x90, 0x8EA9),
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
    },
    Behaviour {
        id: BehaviourId::HorizontalDirection,
        summary: "during a turn, Samus keeps sliding the way she was going.",
        source: Source::bank(0x90, 0x8EA9),
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
    },
    Behaviour {
        id: BehaviourId::DisplacementClamp,
        summary: "Samus never moves more than 15 px sideways in a frame.",
        source: Source::bank(0x90, 0xE4AD),
        kind: Kind::Mechanic,
        switch: Switch::Parameter,
    },
    Behaviour {
        id: BehaviourId::WallStop,
        summary: "running into a wall stops her dead, flush against it.",
        source: Source::bank(0x90, 0xE5CE),
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
    },
    Behaviour {
        id: BehaviourId::StepDown,
        summary: "walking off a ledge or down a slope, Samus sticks to the ground beneath her.",
        source: Source::bank(0x90, 0x923F),
        kind: Kind::Mechanic,
        switch: Switch::Parameter,
    },
    Behaviour {
        id: BehaviourId::BallAtRest,
        summary: "the still morph ball sheds any speed it has, unless it's in the air.",
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
        summary: "falling off the ground tells pose selection which fall to start.",
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
        id: BehaviourId::StandingShotRestartsAnimation,
        summary: "shooting while standing plainly restarts the standing animation.",
        source: Source::bank(0x90, 0xA3AD),
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
    },
    Behaviour {
        id: BehaviourId::FloorFlushSubpixel,
        summary: "(invisible) flush against a floor her vertical subpixel is FFFF; against a wall, FFFF or 0 by side.",
        source: Source::bank(0x94, 0x8F82),
        kind: Kind::Quirk,
        switch: Switch::Fixed,
    },
    Behaviour {
        id: BehaviourId::ArmPumpStep,
        summary: "changing aim while running moves Samus an extra pixel forward.",
        source: Source::bank(0x91, 0xEADE),
        kind: Kind::Quirk,
        switch: Switch::Fixed,
    },
];
