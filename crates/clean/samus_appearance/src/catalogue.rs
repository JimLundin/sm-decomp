//! The catalogue: one behaviour per entry of the spec.

use enum_map::{Enum, EnumMap};
use types::{Checked, Kind, Switch, by_member};

/// One per entry of the spec, by the entry's name. Its order means nothing.
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
    /// A boosted lap that only counts goes on at the clip's own hold.
    CountedLapHold,
    /// The lap on which she starts boosting starts on a stage the sounds waiting name.
    BoostingLap,
    /// Running contact graphics and optional footstep sound.
    FootContact,
    /// Pose selection starts its spin sound.
    SpinStarts,
    /// Prepared frames keep her action; normal drawing ends a kept spin's sound once she left it.
    SpinEnds,
    /// First passage entry quiets Samus before sound hold.
    AtTheDoor,
    /// A changed selected item waits one more frame of eligible item controls.
    CannonItemDelay,
    /// A newly initialized Samus waits two frames of eligible unchanged item controls.
    CannonReadinessReset,
    /// Ordinary dry movement readies the visor and charge glow without changing either palette.
    OrdinaryDryColours,
    /// A fall in energy or a push's start pulses her colours and asks for the hurt sound.
    HurtColours,
    /// The standing body keeps its current picture as the elevator shaft tilts.
    ElevatorStandingPicture,
    /// After push time runs out, protection hides the body on alternate draws.
    ProtectionFlicker,
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

/// The catalogue: every behaviour of the spec, by its id.
#[must_use]
pub fn catalogue() -> EnumMap<BehaviourId, Behaviour> {
    by_member(ROWS, |behaviour| behaviour.id)
}

/// The catalogue's rows, each naming its behaviour, in any order.
const ROWS: [Behaviour; 26] = [
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
                  the start slowdown and by the step slowdown both",
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
        id: BehaviourId::CountedLapHold,
        summary: "a boosted lap that only counts down her laps goes on at the clip's own hold, \
                  not the pace's, until the next shown step",
        kind: Kind::Quirk,
        switch: Switch::Fixed,
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
        summary: "her prepared frames keep her action, and when she has left a kept spin or wall \
                  jump, normal drawing ends its sound, even while her animation is frozen",
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
        id: BehaviourId::CannonItemDelay,
        summary: "after her selected item changes, the cannon waits for one more frame of eligible item controls before it may follow that item",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::CannonReadinessReset,
        summary: "a newly initialized Samus needs two frames of eligible unchanged item controls before her cannon may follow the selection",
        kind: Kind::Mechanic,
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
    // The spec gives this entry two kinds, timing and output; a behaviour has one, the first.
    Behaviour {
        id: BehaviourId::HurtColours,
        summary: "a fall in energy or a push's start pulses her colours three times and asks for \
                  the hurt sound",
        kind: Kind::Timing,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::ElevatorStandingPicture,
        summary: "Samus keeps the standing picture for her current pose and clip step as the shaft tilts behind her",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    // As with HurtColours, use the first of the two kinds the spec gives.
    Behaviour {
        id: BehaviourId::ProtectionFlicker,
        summary: "after a hit's push time runs out, protection hides the body on alternate draws",
        kind: Kind::Timing,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
];
