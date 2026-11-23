//! The catalogue: one behaviour per entry of the spec.

use enum_map::Enum;
use types::{Checked, Kind, Switch};

/// One per entry of the spec, by the entry's name.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Enum)]
pub enum BehaviourId {
    /// The time runs down, and the clip moves on.
    ClipPlays,
    /// An `again` loops the clip, and a `rest` holds it one frame.
    ClipControls,
    /// Standing or crouched still, she breathes, hard once her energy is low.
    Breathing,
    /// A clip that reaches its end reports which end it is.
    ClipEnds,
    /// A wall jump spins by her items, each spin with its sound.
    WallJumpBranch,
    /// A pose an end handed over is slowed twice in a liquid.
    HandOverSlowdown,
    /// Where the clip of the pose she takes starts.
    StartingClip,
    /// Holding shoot standing unaimed starts her standing clip over.
    StandingRestart,
    /// Readying a wall jump turns her spin to its kick step.
    WallJumpReady,
    /// Her spin has turned far enough to kick off a wall.
    SpinTurnedFar,
    /// Moving fast in a fall's pose jumps to its fast fall picture.
    FastFall,
    /// A straight jump holds its rising picture while she rises.
    RisingHold,
    /// Running while dashing, her stride takes the run's pace.
    RunPace,
    /// The speed booster's stages, lap by lap, up to boosting.
    BoostStages,
    /// The lap on which she starts boosting starts on a stage the sounds waiting name.
    BoostingLap,
    /// Running contact graphics and optional footstep sound.
    FootContact,
    /// Pose selection starts its spin sound.
    SpinStarts,
    /// Normal drawing ends a departed spin sound.
    SpinEnds,
    /// First passage entry quiets Samus before sound hold.
    AtTheDoor,
    /// Actual animation answers offered before their effects.
    PerformedAnimationDecisions,
    /// Complete interrupted work restored and continued once.
    SuspendedAppearance,
    /// Selected item identity controls the independent cannon delay.
    CannonItemDelay,
    /// Full initialization starts two eligible observations' wait.
    CannonReadinessReset,
    /// Actual chosen controls complete readiness once and survive restoration.
    RetainedCannonReadiness,
    /// Ordinary dry movement readies the visor and charge glow without changing either palette.
    OrdinaryDryColours,
    /// An admitted palette reset survives interruption and completes once.
    RetainedDryColours,
}

/// A behaviour of the spec: what the player sees, its kind, its switch and how it is checked.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Behaviour {
    /// The spec entry.
    pub id: BehaviourId,
    /// What the player sees.
    pub summary: &'static str,
    /// Its kind.
    pub kind: Kind,
    /// Its switch.
    pub switch: Switch,
    /// How the spec checks it.
    pub checked: Checked,
}

/// Every behaviour of the spec, in its order.
pub const CATALOGUE: [Behaviour; 26] = [
    Behaviour {
        id: BehaviourId::ClipPlays,
        summary: "each frame the step's time runs down by 1; when it runs out the clip moves on, \
                  and a shown step shows for its hold",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::ClipControls,
        summary: "an again loops the clip back to a step, and a rest holds it one frame",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::Breathing,
        summary: "standing or crouched still, she breathes in a loop while her energy is high, \
                  and goes on to breathe hard once it is low",
        kind: Kind::Mechanic,
        switch: Switch::Parameter,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::ClipEnds,
        summary: "a clip that reaches its end reports which end it is, and samus-movement hands \
                  over her next pose; at a turn's or a landing's end it gives way to a jump its \
                  pad chart asks for",
        kind: Kind::Output,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::WallJumpBranch,
        summary: "a wall jump spins as the screw attack with it, else as the space jump with it, \
                  else plainly, and plainly when submerged; each asks for its own spin's sound",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::HandOverSlowdown,
        summary: "in a liquid, the first step of a pose a clip's end handed over is slowed by \
                  the start's slowdown and by the clip's slowdown both",
        kind: Kind::Quirk,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::StartingClip,
        summary: "the clip of the pose she takes starts at its first step, at its second when \
                  she was already doing what the first step shows, and keeps going when her run \
                  or her ball goes on in another pose",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::StandingRestart,
        summary: "holding shoot while she stands unaimed starts her standing clip over",
        kind: Kind::Mechanic,
        switch: Switch::Parameter,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::WallJumpReady,
        summary: "readying a wall jump turns her spin straight to its kick step",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::SpinTurnedFar,
        summary: "her spin has turned far enough to kick off a wall once it is on its kick step \
                  or past it, known as the frame starts",
        kind: Kind::Output,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::FastFall,
        summary: "moving fast in a fall's pose, her fall jumps to its fast fall picture, \
                  whichever way she moves",
        kind: Kind::Mechanic,
        switch: Switch::Parameter,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::RisingHold,
        summary: "a straight jump holds its rising picture for as long as she rises",
        kind: Kind::Mechanic,
        switch: Switch::Parameter,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::RunPace,
        summary: "running while dashing, her stride takes the run's pace, plain without the \
                  speed booster and quickening with its stages; with run held each lap of her \
                  run starts over on the pace",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::BoostStages,
        summary: "with the speed booster, every lap run with run held counts towards the next \
                  stage; four stages on she is boosting, and laps go on at the boosting pace",
        kind: Kind::Mechanic,
        switch: Switch::Parameter,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::BoostingLap,
        summary: "the lap on which she starts boosting starts on the first hold of the stage \
                  numbered by the sounds waiting, usually the slowest, and lasts that stage's \
                  laps; the rest of the lap runs at the boosting pace",
        kind: Kind::Quirk,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::FootContact,
        summary: "the last frame of a running contact picture asks for the stride's graphics, and for its footstep sound unless an active sound suppressor prevents it",
        kind: Kind::Output,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::SpinStarts,
        summary: "the pose she takes starts its spin's sound, while a plain continuing spin stays quiet; this does not add a second animation update",
        kind: Kind::Output,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::SpinEnds,
        summary: "when she leaves a spin or wall jump, normal drawing ends its sound, even while her animation is frozen",
        kind: Kind::Output,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::AtTheDoor,
        summary: "on first entering a passage, Samus stops her spin sound or her quiet uncharged sound before sounds are held",
        kind: Kind::Output,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::PerformedAnimationDecisions,
        summary: "the actual current clip decision and selected clip initialization are observable before their effects",
        kind: Kind::Timing,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::SuspendedAppearance,
        summary: "interrupted animation and appearance opportunities continue once with complete State and ordered outputs",
        kind: Kind::Timing,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::CannonItemDelay,
        summary: "after her selected item changes, the cannon waits for one more eligible control observation before it may follow that item",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::CannonReadinessReset,
        summary: "a newly initialized Samus needs two eligible unchanged item-control observations before her cannon may follow the selection",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::RetainedCannonReadiness,
        summary: "cannon readiness follows one actual completed control answer, survives interruption, and is read without advancing it",
        kind: Kind::Timing,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::OrdinaryDryColours,
        summary: "during ordinary dry movement the visor and charge-glow sequences are readied while the present and target colours stay as they were",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::RetainedDryColours,
        summary: "one actual ordinary palette opportunity survives interruption and applies its reset once",
        kind: Kind::Timing,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
];
