//! The catalogue: one behaviour for each entry of the spec, with its kind, its switch and how it
//! is checked.

use enum_map::{Enum, EnumMap};
use types::{Checked, Kind, Switch, by_member};

/// A behaviour of the spec, by its entry id.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum)]
pub enum BehaviourId {
    /// A door counts its wait off by one each run, and steps when it runs out.
    Waiting,
    /// A door sees her when she was within the reach on both axes.
    SheIsNear,
    /// A placed door shows itself, then hides and chooses to stand open or closed.
    RoomStart,
    /// A closed or open door looks for her every few frames.
    Polling,
    /// A closed door that sees her hisses and slides open.
    Opening,
    /// An open door that no longer sees her slides shut.
    Closing,
    /// Once Ridley has fled, the shaft's door is an unseen wall.
    ShaftWall,
    /// Which phases block her.
    Solid,
    /// Which phases are drawn.
    Shown,
    /// During the escape, the doors keep the station shaking.
    EscapeShakes,
    /// The doors' palette takes the calm or the escape colours.
    TheDoorsColours,
    /// The arrival puts the pad under her and the cover below it; she is held.
    PadPlaced,
    /// The pad holds her still for a second before it lowers her.
    PadWaits,
    /// The pad lowers her a pixel a run to the shaft's floor, then lets her go.
    PadCarries,
    /// The pad's two figures take turns.
    PadFlickers,
    /// The cover hides the shaft's floor until she stands on it.
    CoverHides,
    /// Before the escape, the beacon steps through its colour sets.
    BeaconCycles,
    /// Before the escape, the platform shows its two looks in turn.
    PlatformTurns,
    /// The door of Ridley's room stands open behind her a moment.
    RidleyRoomStart,
    /// Then it slams shut over four figures.
    SlamShut,
    /// The shut door stays locked while Ridley lives.
    LockedIn,
    /// While the door of Ridley's room is shut, Ridley's drawing draws it.
    DrawnWithRidley,
    /// During Ridley's getaway two walls stand at his room's sides.
    GetawayWalls,
    /// As she enters his room Ridley is placed in the dark, the baby in his hold.
    RidleyPlaced,
    /// Once the passage into his room is over he waits a fixed while, then wakes.
    RidleyWaits,
    /// The baby stays in his hold.
    BabyHeld,
    /// The baby fidgets through its figures.
    BabyFidgets,
    /// At the end of each fidget the baby, one time in two, cries and glows.
    BabyCries,
    /// Once awake his eye lights up out of the dark, a step each run.
    EyeFadesIn,
    /// Then his body fades in, a set every few runs; with the last he turns tangible.
    BodyFadesIn,
    /// Tangible, he holds still a moment before he roars.
    ReadyToRoar,
    /// He roars, shuts his jaws and sits; then he takes off.
    Roars,
    /// From his moving until his take-off, his tail hangs still behind him.
    TailAtRest,
    /// Now and then, at random, his tail's curl changes.
    TailCurls,
    /// Tangible, his tail's tip hits her within its reach once her protection has run out.
    TailHits,
    /// He crouches, hops up and to the left twice, then springs into the air.
    TakesOff,
    /// He rises faster and faster until he is high enough, then starts to slow.
    Rises,
    /// Rising, he slows until he is high enough, then hovers.
    SlowsDown,
    /// He hovers until she is nearly out of energy, or he reaches his point or tires and attacks.
    Hovers,
    /// Each run he hovers, his speed is pulled to his point.
    PulledToHisPoint,
    /// He moves by his speed within the room's bounds, shaking it as he meets its left side fast.
    Flies,
}

/// A catalogue entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Behaviour {
    /// Its spec entry.
    pub id: BehaviourId,
    /// What the player sees.
    pub summary: &'static str,
    /// What kind of behaviour it is.
    pub kind: Kind,
    /// How it can be adjusted.
    pub switch: Switch,
    /// How the spec checks it.
    pub checked: Checked,
}

/// How many entries the spec has: the doors' eleven, the shaft's seven, Ridley's room's ten,
/// his waking's four, his tail's three and his flight's six.
const ENTRIES: usize = 41;

/// Every entry's behaviour, as the spec gives it, one row for each entry.
const BEHAVIOURS: [Behaviour; ENTRIES] = [
    Behaviour {
        id: BehaviourId::Waiting,
        summary: "a door counts its wait off by one each run, and steps when it runs out",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::SheIsNear,
        summary: "a door sees her when she was within a set distance of its position on \
                          both axes at the end of the last frame",
        kind: Kind::Mechanic,
        switch: Switch::Parameter,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::RoomStart,
        summary: "a door placed by a room's start is shown and solid with no figure for \
                          its first frame, then hidden and passable until it chooses, two frames \
                          later, to stand open or closed",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::Polling,
        summary: "a door that is closed or open looks for her every few frames",
        kind: Kind::Mechanic,
        switch: Switch::Parameter,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::Opening,
        summary: "a closed door that sees her hisses and slides open over four figures, \
                          staying solid until it is open; then it is hidden and she can pass",
        kind: Kind::Mechanic,
        switch: Switch::Parameter,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::Closing,
        summary: "an open door that no longer sees her shows itself again and slides shut \
                          over four figures, without a sound and without looking for her until \
                          it is closed",
        kind: Kind::Mechanic,
        switch: Switch::Parameter,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::ShaftWall,
        summary: "once Ridley has fled, the shaft's door is an unseen wall",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::Solid,
        summary: "a door blocks in the phases that show it, and as the wall; hidden doors \
                          that are open or starting let her through",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::Shown,
        summary: "a door is shown in the phases it is solid in, except the wall; it is \
                          drawn with its phase's figure",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::EscapeShakes,
        summary: "during the escape, each door keeps the station shaking: whenever the \
                          screen is still, a door's run starts a small shake, and now and then a \
                          strong one",
        kind: Kind::Mechanic,
        switch: Switch::Parameter,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::TheDoorsColours,
        summary: "when a room with doors starts, the doors' palette takes the doors' \
                          colours, another set during the escape",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::PadPlaced,
        summary: "the arrival puts the pad under her and the cover below it, and from \
                          then on she is held",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::PadWaits,
        summary: "the pad holds her still for a second before it starts to lower her",
        kind: Kind::Mechanic,
        switch: Switch::Parameter,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::PadCarries,
        summary: "each run once its wait is over, the pad lowers her one pixel and is \
                          drawn 28 px below where she stood, until she stands on the shaft's \
                          floor; then it lets her go",
        kind: Kind::Mechanic,
        switch: Switch::Parameter,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::PadFlickers,
        summary: "the pad's two figures take turns, one each run",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::CoverHides,
        summary: "the cover hides the shaft's floor below the pad until she stands on \
                          the floor",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::BeaconCycles,
        summary: "before the escape, the shaft's beacon steps through its cycle of eight \
                          colour sets, one step every few frames, except once a door's fade has \
                          taken a step",
        kind: Kind::Mechanic,
        switch: Switch::Parameter,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::PlatformTurns,
        summary: "before the escape, the platform below the shaft shows its two looks in \
                          turn, two frames each",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::RidleyRoomStart,
        summary: "the door of Ridley's room, placed as the room starts, stands open behind \
                          her a moment",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::SlamShut,
        summary: "then it slams shut over four figures, quickly and silently, near her or \
                          not",
        kind: Kind::Mechanic,
        switch: Switch::Parameter,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::LockedIn,
        summary: "the shut door stays locked while Ridley lives, looking every 2 frames \
                          whether he is beaten; then it is a closed door like any other, which \
                          opens as she comes near",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::DrawnWithRidley,
        summary: "while the door of Ridley's room is shut, Ridley's drawing draws it",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::GetawayWalls,
        summary: "during Ridley's getaway two walls stand at his room's sides; they look \
                          for nobody",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::RidleyPlaced,
        summary: "as she enters his room he is placed at his place, in the dark, the baby \
                          in his hold; the mini-map goes blank, the music stops and the station \
                          stops shaking",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::RidleyWaits,
        summary: "once the door's passage into his room is over he waits a fixed while in \
                          the dark, then wakes",
        kind: Kind::Mechanic,
        switch: Switch::Parameter,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::BabyHeld,
        summary: "the baby stays in his hold",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::BabyFidgets,
        summary: "the baby fidgets through its figures, ten runs each",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::BabyCries,
        summary: "at the end of each fidget the baby, one time in two, cries and glows",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::EyeFadesIn,
        summary: "once awake his eye lights up out of the dark, one step of its fade each run",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::BodyFadesIn,
        summary: "then his body fades in from black, one colour set every few runs; with \
                          its last set he turns tangible and the fight's music starts",
        kind: Kind::Mechanic,
        switch: Switch::Parameter,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::ReadyToRoar,
        summary: "tangible, he holds still a moment before he roars",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::Roars,
        summary: "he opens his jaws and roars, shuts them, and sits until his roar's wait \
                          runs out; then he takes off",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::TailAtRest,
        summary: "from the moment he starts moving until he takes off, his tail hangs \
                          still: its tip and all its segments at one place behind him",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::TailCurls,
        summary: "now and then, at random, the tail's curl changes; it shows only once \
                          the tail moves",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::TailHits,
        summary: "tangible, he hits her when her body comes within reach of his tail's \
                          tip and her protection has run out; she is pushed away from the tip",
        kind: Kind::Mechanic,
        switch: Switch::Parameter,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::TakesOff,
        summary: "he crouches, hops up and to the left twice, then springs into the air",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::Rises,
        summary: "he rises faster and faster until he is high enough, then starts to slow",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::SlowsDown,
        summary: "rising, he slows until he is high enough, then hovers",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::Hovers,
        summary: "he hovers, drifting to his point; once she is nearly out of energy he gives \
                  up, and once he reaches his point, or has hovered a while, he attacks",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::PulledToHisPoint,
        summary: "each run he hovers, his speed on each axis is pulled to his point: harder \
                  the farther he is, three times as hard while he moves away from it",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::Flies,
        summary: "he moves by his speed, kept within the room's bounds; meeting the left side \
                  fast, he shakes the room",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
];

/// The catalogue: every entry's behaviour, keyed by its id. Each row names its entry; the
/// vector test checks that every entry has its own row.
#[must_use]
pub fn catalogue() -> EnumMap<BehaviourId, Behaviour> {
    by_member(BEHAVIOURS, |behaviour| behaviour.id)
}
