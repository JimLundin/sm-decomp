//! The shape of the vector file, declared once: each table of the file is a type here that the
//! reader (`toml`, with `serde`) fills, and each turns into the model's own values. Every table
//! refuses a key it doesn't know, and every name a member it doesn't know, so anything in the
//! file this test doesn't know stops it.

use std::collections::BTreeMap;
use std::num::Wrapping;

use clean_model_progress::{
    Condition, Count, Inventory, MapStatus, Pickup, Progress, Remembered, Report, ReserveMode,
    Supply,
};
use clean_types::{
    Area, Beam, BossKind, DoorId, Entrance, Event, Item, LocationId, Set, StationId,
};
use enum_map::{Enum, EnumMap};
use serde::{Deserialize, Deserializer};

/// The vector file: its `[[vector]]`s.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct File {
    pub vector: Vec<Vector>,
}

/// One `[[vector]]`: a frame of reports (`reports`, `after`) or questions (`asks`, `answers`),
/// both from the progress `before`, in a frame whose area is `area`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Vector {
    pub id: String,
    pub source: String,
    pub entries: Vec<String>,
    pub area: Named<Area>,
    pub before: State,
    reports: Option<Vec<ReportRow>>,
    after: Option<State>,
    asks: Option<Asks>,
    answers: Option<Answers>,
}

/// What a vector checks, in the model's values.
pub enum Check<'a> {
    /// The reports of a frame, and the progress at its end.
    Frame {
        reports: Vec<Report>,
        after: Box<Progress>,
    },
    /// The questions other domains ask, and their answers.
    Asks {
        asks: &'a Asks,
        answers: &'a Answers,
    },
}

impl Vector {
    /// What the vector checks: a frame or questions, never both or a part of one.
    pub fn check(&self) -> Check<'_> {
        let Named(area) = self.area;
        match (&self.reports, &self.after, &self.asks, &self.answers) {
            (Some(reports), Some(after), None, None) => Check::Frame {
                reports: reports.iter().map(|r| r.report(area)).collect(),
                after: Box::new(after.progress()),
            },
            (None, None, Some(asks), Some(answers)) => Check::Asks { asks, answers },
            _ => panic!(
                "{}: a vector is reports and after, or asks and answers",
                self.id
            ),
        }
    }
}

/// A member of the model's enum `T`, read from the name the file gives it.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Named<T>(pub T);

/// Reads `Named<$model>` by `$names`, a copy of `$model` giving each member the name the file
/// writes. Serde's `remote` builds the model's own members from it, so the copy can name only
/// real members, and a name it doesn't hold fails the read.
macro_rules! named_by {
    ($model:ty, $names:ident) => {
        impl<'de> Deserialize<'de> for Named<$model> {
            fn deserialize<D: Deserializer<'de>>(reader: D) -> Result<Self, D::Error> {
                $names::deserialize(reader).map(Named)
            }
        }
    };
}

#[derive(Deserialize)]
#[serde(remote = "Area")]
enum AreaNames {
    Brinstar,
    Ceres,
    Crateria,
    Maridia,
    Norfair,
    Tourian,
    WreckedShip,
}
named_by!(Area, AreaNames);

#[derive(Deserialize)]
#[serde(remote = "Item")]
enum ItemNames {
    Bombs,
    GrappleBeam,
    GravitySuit,
    HiJumpBoots,
    MorphBall,
    ScrewAttack,
    SpaceJump,
    SpeedBooster,
    SpringBall,
    VariaSuit,
    XRayScope,
}
named_by!(Item, ItemNames);

#[derive(Deserialize)]
#[serde(remote = "Beam")]
enum BeamNames {
    Charge,
    Ice,
    Plasma,
    Spazer,
    Wave,
}
named_by!(Beam, BeamNames);

#[derive(Deserialize)]
#[serde(remote = "Event")]
enum EventNames {
    AcidReceded,
    AnimalsFreed,
    DraygonStatueGrey,
    GlassTunnelBroken,
    KraidStatueGrey,
    LavaQuakeOutrun,
    MetroidRoom1Empty,
    MetroidRoom2Empty,
    MetroidRoom3Empty,
    MetroidRoom4Empty,
    MotherBrainTankBroken,
    PhantoonStatueGrey,
    PlanetAwake,
    RidleyStatueGrey,
    ShaktoolPath,
    TimeBombSet,
    TourianOpen,
    TourianRoomsUnset,
}
named_by!(Event, EventNames);

#[derive(Deserialize)]
#[serde(remote = "BossKind")]
enum BossKindNames {
    MainBoss,
    MiniBoss,
    Torizo,
}
named_by!(BossKind, BossKindNames);

/// The tanks, by the supply each raises.
#[derive(Deserialize)]
#[serde(remote = "Supply")]
enum TankNames {
    #[serde(rename = "EnergyTank")]
    Energy,
    #[serde(rename = "ReserveTank")]
    Reserve,
    #[serde(rename = "MissileTank")]
    Missiles,
    #[serde(rename = "SuperMissileTank")]
    SuperMissiles,
    #[serde(rename = "PowerBombTank")]
    PowerBombs,
}
named_by!(Supply, TankNames);

#[derive(Deserialize)]
#[serde(remote = "ReserveMode")]
enum ReserveModeNames {
    #[serde(rename = "None")]
    NoTank,
    Automatic,
    Manual,
}
named_by!(ReserveMode, ReserveModeNames);

/// The maps the file names; an area it doesn't name is `Unmapped`.
#[derive(Deserialize)]
#[serde(remote = "MapStatus")]
enum MapNames {
    Downloaded,
    OnArrival,
}
named_by!(MapStatus, MapNames);

/// The set of the members `names` names.
fn set<K: Enum + Copy>(names: &[Named<K>]) -> Set<K> {
    names.iter().map(|&Named(member)| (member, true)).collect()
}

/// A value for each area: `value` of the row the file gives it, and the default for any other.
fn per_area<T, V: Default>(
    rows: &BTreeMap<Named<Area>, T>,
    value: impl Fn(&T) -> V,
) -> EnumMap<Area, V> {
    rows.iter()
        .map(|(&Named(area), row)| (area, value(row)))
        .collect()
}

/// The supplies a table gives, each left out at its value in `start`.
fn supplies(
    given: EnumMap<Supply, Option<u16>>,
    start: EnumMap<Supply, Count>,
) -> EnumMap<Supply, Count> {
    given.map(|supply, n| n.map_or(start[supply], Wrapping))
}

/// A progress (`before`, `after`): every key left out at its value in a new file.
#[derive(Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct State {
    items: Vec<Named<Item>>,
    equipped: Vec<Named<Item>>,
    beams: Vec<Named<Beam>>,
    beams_equipped: Vec<Named<Beam>>,
    capacity: Capacity,
    energy: Option<u16>,
    reserve_energy: Option<u16>,
    missiles: Option<u16>,
    super_missiles: Option<u16>,
    power_bombs: Option<u16>,
    reserve_mode: Option<Named<ReserveMode>>,
    events: Vec<Named<Event>>,
    zebetites: u16,
    defeated: BTreeMap<Named<Area>, Vec<Named<BossKind>>>,
    emptied: Vec<u16>,
    opened: Vec<u16>,
    maps: BTreeMap<Named<Area>, Named<MapStatus>>,
    stations: BTreeMap<Named<Area>, Vec<u16>>,
    items_placed: u16,
}

/// A progress's `capacity`: each left out at its value in a new file.
#[derive(Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
struct Capacity {
    energy: Option<u16>,
    reserve: Option<u16>,
    missiles: Option<u16>,
    super_missiles: Option<u16>,
    power_bombs: Option<u16>,
}

impl State {
    /// The progress this state gives.
    pub fn progress(&self) -> Progress {
        let new = Inventory::default();
        let capacity = &self.capacity;
        let given_capacity = EnumMap::from_fn(|supply| match supply {
            Supply::Energy => capacity.energy,
            Supply::Reserve => capacity.reserve,
            Supply::Missiles => capacity.missiles,
            Supply::SuperMissiles => capacity.super_missiles,
            Supply::PowerBombs => capacity.power_bombs,
        });
        let given_held = EnumMap::from_fn(|supply| match supply {
            Supply::Energy => self.energy,
            Supply::Reserve => self.reserve_energy,
            Supply::Missiles => self.missiles,
            Supply::SuperMissiles => self.super_missiles,
            Supply::PowerBombs => self.power_bombs,
        });
        Progress {
            inventory: Inventory {
                items: set(&self.items),
                items_switched_on: set(&self.equipped),
                beams: set(&self.beams),
                beams_switched_on: set(&self.beams_equipped),
                capacity: supplies(given_capacity, new.capacity),
                held: supplies(given_held, new.held),
                reserve_mode: self
                    .reserve_mode
                    .map_or(new.reserve_mode, |Named(mode)| mode),
            },
            events: set(&self.events),
            zebetites: Wrapping(self.zebetites),
            defeated: per_area(&self.defeated, |kinds| set(kinds)),
            emptied: Remembered::of(self.emptied.iter().map(|&id| LocationId(id))),
            opened: Remembered::of(self.opened.iter().map(|&id| DoorId(id))),
            maps: per_area(&self.maps, |&Named(map)| map),
            stations: per_area(&self.stations, |ids| {
                ids.iter().map(|&id| StationId(id)).collect()
            }),
            items_placed: Wrapping(self.items_placed),
        }
    }
}

/// A report, by its `report` key; one that concerns an area is the frame's.
#[derive(Deserialize)]
#[serde(tag = "report", deny_unknown_fields)]
enum ReportRow {
    Happened {
        event: Named<Event>,
    },
    ZebetiteDestroyed {},
    Defeated {
        boss: Named<BossKind>,
    },
    Taken {
        item: Taken,
        location: Option<u16>,
        amount: Option<u16>,
    },
    DoorOpened {
        door: Option<u16>,
    },
    MapStation {},
    MapOnArrival {},
    LeftArea {},
    Saved {
        station: u16,
    },
    ItemsPlaced {
        count: u16,
    },
}

/// What a `Taken` report names: an item, a beam or a tank.
#[derive(Deserialize, Clone, Copy)]
#[serde(untagged)]
enum Taken {
    Item(Named<Item>),
    Beam(Named<Beam>),
    Tank(Named<Supply>),
}

/// What she takes; a tank, and only a tank, carries an amount.
fn pickup(taken: Taken, amount: Option<u16>) -> Pickup {
    match (taken, amount) {
        (Taken::Item(Named(item)), None) => Pickup::Item(item),
        (Taken::Beam(Named(beam)), None) => Pickup::Beam(beam),
        (Taken::Tank(Named(supply)), Some(n)) => Pickup::Tank {
            supply,
            amount: Wrapping(n),
        },
        (Taken::Item(_) | Taken::Beam(_), Some(_)) | (Taken::Tank(_), None) => {
            panic!("a tank, and only a tank, carries an amount")
        }
    }
}

impl ReportRow {
    /// The report, made in a frame whose area is `area`.
    fn report(&self, area: Area) -> Report {
        match *self {
            Self::Happened {
                event: Named(event),
            } => Report::Happened(event),
            Self::ZebetiteDestroyed {} => Report::ZebetiteDestroyed,
            Self::Defeated { boss: Named(kind) } => Report::Defeated { kind, area },
            Self::Taken {
                item,
                location,
                amount,
            } => Report::Taken {
                pickup: pickup(item, amount),
                location: location.map(LocationId),
            },
            Self::DoorOpened { door } => Report::DoorOpened(door.map(DoorId)),
            Self::MapStation {} => Report::MapStation(area),
            Self::MapOnArrival {} => Report::MapOnArrival(area),
            Self::LeftArea {} => Report::LeftArea(area),
            Self::Saved { station } => Report::Saved {
                area,
                station: StationId(station),
            },
            Self::ItemsPlaced { count } => Report::ItemsPlaced(Wrapping(count)),
        }
    }
}

/// A room's state, by its `when` key: a condition, or `Always`, the default state.
#[derive(Deserialize)]
#[serde(tag = "when", deny_unknown_fields)]
enum When {
    Always {},
    Happened { event: Named<Event> },
    Defeated { bosses: Vec<Named<BossKind>> },
    Has { item: Named<Item> },
    MorphBallAndMissiles {},
    PowerBombs {},
    EnteredBy { entrance: u16 },
}

impl When {
    /// The condition, or nothing for `Always`.
    fn condition(&self) -> Option<Condition> {
        Some(match self {
            Self::Always {} => return None,
            Self::Happened {
                event: Named(event),
            } => Condition::Happened(*event),
            Self::Defeated { bosses } => Condition::Defeated(set(bosses)),
            Self::Has { item: Named(item) } => Condition::Has(*item),
            Self::MorphBallAndMissiles {} => Condition::MorphBallAndMissiles,
            Self::PowerBombs {} => Condition::PowerBombs,
            Self::EnteredBy { entrance } => Condition::EnteredBy(Entrance(*entrance)),
        })
    }
}

/// The questions of a vector.
#[derive(Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct Asks {
    room_states: Option<Vec<When>>,
    pub entered_by: Option<u16>,
    pub percentage: bool,
    emptied: Option<Vec<AskedLocation>>,
    opened: Option<Vec<AskedDoor>>,
    pub map_known: Option<Vec<Named<Area>>>,
    pub zebetites: bool,
}

/// A question about a location: its id, or none for an item placed without one.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AskedLocation {
    location: Option<u16>,
}

/// A question about a door: its id, or none for a door placed without one.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AskedDoor {
    door: Option<u16>,
}

impl Asks {
    /// The room's conditional versions, in its order, if a room is asked about. The file ends a
    /// room's states with `Always`, its default, which `RoomStates` holds by its type.
    pub fn room_states(&self) -> Option<Vec<Condition>> {
        let states = self.room_states.as_ref()?;
        let (last, conditional) = states.split_last().expect("a room has states");
        assert!(last.condition().is_none(), "a room's last state is Always");
        let condition = |when: &When| {
            when.condition()
                .expect("only a room's last state is Always")
        };
        Some(conditional.iter().map(condition).collect())
    }

    /// The locations asked about, if any are.
    pub fn emptied(&self) -> Option<Vec<Option<LocationId>>> {
        let asked = self.emptied.as_ref()?;
        Some(asked.iter().map(|q| q.location.map(LocationId)).collect())
    }

    /// The doors asked about, if any are.
    pub fn opened(&self) -> Option<Vec<Option<DoorId>>> {
        let asked = self.opened.as_ref()?;
        Some(asked.iter().map(|q| q.door.map(DoorId)).collect())
    }
}

/// The answers to a vector's questions, compared whole: the file's, and the model's.
#[derive(Deserialize, Debug, Default, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct Answers {
    pub room_state: Option<usize>,
    pub percentage: Option<u16>,
    pub emptied: Option<Vec<bool>>,
    pub opened: Option<Vec<bool>>,
    pub map_known: Option<Vec<bool>>,
    pub zebetites: Option<u16>,
}
