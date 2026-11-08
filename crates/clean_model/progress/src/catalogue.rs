//! The catalogue: one behaviour for each entry of the progress spec.

use clean_types::{Checked, Kind, Switch};

/// A behaviour of the spec, one per entry, named as the spec names it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BehaviourId {
    /// Spec: EventsHappen.
    EventsHappen,
    /// Spec: ZebetitesCounted.
    ZebetitesCounted,
    /// Spec: BossesDefeated.
    BossesDefeated,
    /// Spec: EmptiedLocations.
    EmptiedLocations,
    /// Spec: DoorsStayOpen.
    DoorsStayOpen,
    /// Spec: MapsByArea.
    MapsByArea,
    /// Spec: MapOnArrival.
    MapOnArrival,
    /// Spec: StationsSavedAt.
    StationsSavedAt,
    /// Spec: ItemsPlacedCount.
    ItemsPlacedCount,
    /// Spec: TakingItems.
    TakingItems,
    /// Spec: TakingBeams.
    TakingBeams,
    /// Spec: Tanks.
    Tanks,
    /// Spec: RoomStates.
    RoomStates,
    /// Spec: ItemPercentage.
    ItemPercentage,
}

/// One behaviour of the catalogue.
pub struct Behaviour {
    /// Which spec entry it is.
    pub id: BehaviourId,
    /// What the player sees.
    pub summary: &'static str,
    /// Whether it is a rule of the game, a quirk, or another kind.
    pub kind: Kind,
    /// Whether `Rules` holds its values (`Switch::Parameter`) or it is fixed.
    pub switch: Switch,
    /// Whether vectors check it, or the original game only.
    pub checked: Checked,
}

/// Every behaviour of the progress spec, in the spec's order.
pub const CATALOGUE: &[Behaviour] = &[
    Behaviour {
        id: BehaviourId::EventsHappen,
        summary: "an event, once it happens, has happened for the rest of the run",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::ZebetitesCounted,
        summary: "the game counts the zebetites she has destroyed, so the ones she broke stay broken",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::BossesDefeated,
        summary: "a defeated boss is remembered by its kind in its area, so it stays gone",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::EmptiedLocations,
        summary: "an item she takes is gone from its place for good",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::DoorsStayOpen,
        summary: "a door she opens for good stays open",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::MapsByArea,
        summary: "a map station gives her the map of its area",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::MapOnArrival,
        summary: "Tourian, which has no map station, gives its map when she enters its first room; \
                  the map counts as downloaded only once she has left the area",
        kind: Kind::Quirk,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::StationsSavedAt,
        summary: "the save stations she has saved at are remembered by area, for the maps to show",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::ItemsPlacedCount,
        summary: "the game counts every item it places as rooms load, taken or not, over the whole \
                  run; nothing in play reads the count, but the file keeps it",
        kind: Kind::Quirk,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::TakingItems,
        summary: "an item she takes is hers and switched on",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::TakingBeams,
        summary: "a beam she takes is hers and switched on; spazer and plasma don't fire together, \
                  so taking one switches the other off",
        kind: Kind::Mechanic,
        switch: Switch::Parameter,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::Tanks,
        summary: "a tank raises a capacity by its amount: an energy tank fills her energy to the new \
                  capacity, a reserve tank leaves the reserves as they were and, the first time, \
                  makes them refill her automatically, and a missile, super missile or power bomb \
                  tank adds its amount to what she holds without filling her up",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::RoomStates,
        summary: "a room has versions, each with a condition on the run's progress; it loads the \
                  first whose condition holds, so it changes as the run goes on",
        kind: Kind::Mechanic,
        switch: Switch::Fixed,
        checked: Checked::ByVectors,
    },
    Behaviour {
        id: BehaviourId::ItemPercentage,
        summary: "the ending shows how much of the game's equipment she found, as a percentage",
        kind: Kind::Mechanic,
        switch: Switch::Parameter,
        checked: Checked::ByVectors,
    },
];
