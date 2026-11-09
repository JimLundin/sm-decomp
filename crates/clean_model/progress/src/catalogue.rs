//! The catalogue: one behaviour for each entry of the progress spec.

use clean_types::{Checked, Kind, Switch};
use enum_map::{Enum, EnumMap};

/// A behaviour of the spec, one per entry, named as the spec and the vector file name it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum)]
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

/// The catalogue: every behaviour of the progress spec, each by its id.
#[must_use]
pub fn catalogue() -> EnumMap<BehaviourId, Behaviour> {
    EnumMap::from_fn(BehaviourId::behaviour)
}

impl BehaviourId {
    /// This behaviour's row of the catalogue. Every entry of this spec is checked by vectors.
    #[must_use]
    pub const fn behaviour(self) -> Behaviour {
        Behaviour {
            id: self,
            summary: self.summary(),
            kind: self.kind(),
            switch: self.switch(),
            checked: Checked::ByVectors,
        }
    }

    /// The two quirks, `MapOnArrival` and `ItemsPlacedCount`; every other entry is a rule of the
    /// game.
    const fn kind(self) -> Kind {
        match self {
            Self::MapOnArrival | Self::ItemsPlacedCount => Kind::Quirk,
            Self::EventsHappen
            | Self::ZebetitesCounted
            | Self::BossesDefeated
            | Self::EmptiedLocations
            | Self::DoorsStayOpen
            | Self::MapsByArea
            | Self::StationsSavedAt
            | Self::TakingItems
            | Self::TakingBeams
            | Self::Tanks
            | Self::RoomStates
            | Self::ItemPercentage => Kind::Mechanic,
        }
    }

    /// The two entries whose values `Rules` holds, the beam each beam taken switches off and the
    /// percentage's divisors; every other entry is fixed.
    const fn switch(self) -> Switch {
        match self {
            Self::TakingBeams | Self::ItemPercentage => Switch::Parameter,
            Self::EventsHappen
            | Self::ZebetitesCounted
            | Self::BossesDefeated
            | Self::EmptiedLocations
            | Self::DoorsStayOpen
            | Self::MapsByArea
            | Self::MapOnArrival
            | Self::StationsSavedAt
            | Self::ItemsPlacedCount
            | Self::TakingItems
            | Self::Tanks
            | Self::RoomStates => Switch::Fixed,
        }
    }

    /// What the player sees, the entry's summary.
    const fn summary(self) -> &'static str {
        match self {
            Self::EventsHappen => "an event, once it happens, has happened for the rest of the run",
            Self::ZebetitesCounted => {
                "the game counts the zebetites she has destroyed, so the ones she broke stay broken"
            }
            Self::BossesDefeated => {
                "a defeated boss is remembered by its kind in its area, so it stays gone"
            }
            Self::EmptiedLocations => "an item she takes is gone from its place for good",
            Self::DoorsStayOpen => "a door she opens for good stays open",
            Self::MapsByArea => "a map station gives her the map of its area",
            Self::MapOnArrival => {
                "Tourian, which has no map station, gives its map when she enters its first room; \
                 the map counts as downloaded only once she has left the area"
            }
            Self::StationsSavedAt => {
                "the save stations she has saved at are remembered by area, for the maps to show"
            }
            Self::ItemsPlacedCount => {
                "the game counts every item it places as rooms load, taken or not, over the whole \
                 run; nothing in play reads the count, but the file keeps it"
            }
            Self::TakingItems => "an item she takes is hers and switched on",
            Self::TakingBeams => {
                "a beam she takes is hers and switched on; spazer and plasma don't fire together, \
                 so taking one switches the other off"
            }
            Self::Tanks => {
                "a tank raises a capacity by its amount: an energy tank fills her energy to the new \
                 capacity, a reserve tank leaves the reserves as they were and, the first time, \
                 makes them refill her automatically, and a missile, super missile or power bomb \
                 tank adds its amount to what she holds without filling her up"
            }
            Self::RoomStates => {
                "a room has versions, each with a condition on the run's progress; it loads the \
                 first whose condition holds, so it changes as the run goes on"
            }
            Self::ItemPercentage => {
                "the ending shows how much of the game's equipment she found, as a percentage"
            }
        }
    }
}
