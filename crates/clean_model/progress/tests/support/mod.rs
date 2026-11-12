//! The shape of the vector file, declared once: each table of the file is a type here that the
//! reader (`toml`, with `serde`) fills, and each turns into the model's own values. Every table
//! refuses a key it doesn't know, and every name a member it doesn't know, so anything in the
//! file this test doesn't know stops it.

use std::collections::BTreeMap;
use std::num::{NonZeroU16, Wrapping};

use clean_model_progress::{
    Condition, Inventory, MapStatus, Pickup, Progress, Remembered, Report, ReserveMode, Rules,
    Supplies, Supply,
};
use clean_types::{
    Area, Beam, BossKind, DoorId, Entrance, Event, Item, LocationId, Set, StationId,
};
use enum_map::{Enum, EnumMap};
use serde::{Deserialize, Deserializer};

/// The vector file: its `[[vector]]`s.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VectorFile {
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
    #[serde(default)]
    rules: RulesRow,
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

    /// The rules the vector runs under: the game's, but for the tables its `rules` gives.
    pub fn rules(&self) -> Rules {
        let reference = Rules::reference();
        let RulesRow {
            switches_off,
            divisors,
        } = &self.rules;
        Rules {
            switches_off: switches_off
                .as_ref()
                .map_or(reference.switches_off, |rows| {
                    per_member(rows, |beams| {
                        beams.iter().map(|&Named(beam)| beam).collect()
                    })
                }),
            divisors: divisors
                .as_ref()
                .map_or(reference.divisors, Divisors::table),
        }
    }
}

/// A vector's `rules`: the tables it gives, each left out at the game's.
#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct RulesRow {
    /// For each beam she takes, the beams it switches off; the table given is the whole table,
    /// so a beam it doesn't name switches none off.
    switches_off: Option<BTreeMap<Named<Beam>, Vec<Named<Beam>>>>,
    divisors: Option<Divisors>,
}

/// The divisors of the item percentage; all five are given, each a whole number from 1.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Divisors {
    energy: NonZeroU16,
    reserve: NonZeroU16,
    missiles: NonZeroU16,
    super_missiles: NonZeroU16,
    power_bombs: NonZeroU16,
}

impl Divisors {
    /// The divisors as the model's table.
    fn table(&self) -> EnumMap<Supply, NonZeroU16> {
        EnumMap::from_fn(|supply| match supply {
            Supply::Energy => self.energy,
            Supply::Reserve => self.reserve,
            Supply::Missiles => self.missiles,
            Supply::SuperMissiles => self.super_missiles,
            Supply::PowerBombs => self.power_bombs,
        })
    }
}

/// A member of the model's enum `T`, read from the name the file gives it.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Named<T>(pub T);

/// Reads `Named<$model>` by `$names`, a copy of the model's enum `$model` giving each member
/// the name the file writes (a member's attributes rename it, or keep the file from naming it).
/// The copy is matched against the model's enum both ways, so the compiler checks that it holds
/// every member of the model's and no other: a member the copy lacks fails to build. A name the
/// copy doesn't hold fails the read.
macro_rules! named_by {
    ($model:ident, $names:ident { $($(#[$attr:meta])* $member:ident),* $(,)? }) => {
        #[derive(Deserialize)]
        enum $names {
            $($(#[$attr])* $member),*
        }

        impl From<$names> for $model {
            fn from(name: $names) -> Self {
                match name {
                    $($names::$member => Self::$member),*
                }
            }
        }

        impl From<$model> for $names {
            fn from(member: $model) -> Self {
                match member {
                    $($model::$member => Self::$member),*
                }
            }
        }

        impl<'de> Deserialize<'de> for Named<$model> {
            fn deserialize<D: Deserializer<'de>>(reader: D) -> Result<Self, D::Error> {
                $names::deserialize(reader).map(|name| Named(name.into()))
            }
        }
    };
}

named_by!(
    Area,
    AreaNames {
        Brinstar,
        Ceres,
        Crateria,
        Maridia,
        Norfair,
        Tourian,
        WreckedShip,
    }
);

named_by!(
    Item,
    ItemNames {
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
);

named_by!(
    Beam,
    BeamNames {
        Charge,
        Ice,
        Plasma,
        Spazer,
        Wave,
    }
);

named_by!(
    Event,
    EventNames {
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
);

named_by!(
    BossKind,
    BossKindNames {
        MainBoss,
        MiniBoss,
        Torizo,
    }
);

// The tanks, by the supply each raises.
named_by!(
    Supply,
    TankNames {
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
);

named_by!(
    ReserveMode,
    ReserveModeNames {
        #[serde(rename = "None")]
        NoTank,
        Automatic,
        Manual,
    }
);

// The maps the file names; it never names `Unmapped`, which an area it doesn't name is.
named_by!(
    MapStatus,
    MapNames {
        #[serde(skip_deserializing)]
        Unmapped,
        Downloaded,
        OnArrival,
    }
);

/// The set of the members `names` names.
fn set<K: Enum + Copy>(names: &[Named<K>]) -> Set<K> {
    names.iter().map(|&Named(member)| (member, true)).collect()
}

/// A value for each member (an area, a beam): `value` of the row the file gives it, and the
/// default (none, nothing) for any other.
fn per_member<K: Enum + Copy, T, V: Default>(
    rows: &BTreeMap<Named<K>, T>,
    value: impl Fn(&T) -> V,
) -> EnumMap<K, V> {
    rows.iter()
        .map(|(&Named(member), row)| (member, value(row)))
        .collect()
}

/// The energy, and its capacity, the file's header gives a field left out: 99.
const HEADER_ENERGY: u16 = 99;

/// A progress (`before`, `after`): every key left out at the start-of-game value the file's
/// header lists, never at the model's own start, so the vectors check the model's new file.
#[derive(Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct State {
    items: Vec<Named<Item>>,
    equipped: Vec<Named<Item>>,
    beams: Vec<Named<Beam>>,
    beams_equipped: Vec<Named<Beam>>,
    capacity: Capacity,
    energy: u16,
    reserve_energy: u16,
    missiles: u16,
    super_missiles: u16,
    power_bombs: u16,
    reserve_mode: Named<ReserveMode>,
    events: Vec<Named<Event>>,
    zebetites: u16,
    defeated: BTreeMap<Named<Area>, Vec<Named<BossKind>>>,
    emptied: Vec<u16>,
    opened: Vec<u16>,
    maps: BTreeMap<Named<Area>, Named<MapStatus>>,
    stations: BTreeMap<Named<Area>, Vec<u16>>,
    items_placed: u16,
}

/// The header's list: `items = [], equipped = [], beams = [], beams_equipped = [], capacity =
/// { energy = 99, .. 0 }, energy = 99, reserve_energy = 0, missiles = 0, super_missiles = 0,
/// power_bombs = 0, reserve_mode = "None", events = [], zebetites = 0, defeated = {}, emptied =
/// [], opened = [], maps = {}, stations = {}, items_placed = 0`.
impl Default for State {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            equipped: Vec::new(),
            beams: Vec::new(),
            beams_equipped: Vec::new(),
            capacity: Capacity::default(),
            energy: HEADER_ENERGY,
            reserve_energy: 0,
            missiles: 0,
            super_missiles: 0,
            power_bombs: 0,
            reserve_mode: Named(ReserveMode::NoTank),
            events: Vec::new(),
            zebetites: 0,
            defeated: BTreeMap::new(),
            emptied: Vec::new(),
            opened: Vec::new(),
            maps: BTreeMap::new(),
            stations: BTreeMap::new(),
            items_placed: 0,
        }
    }
}

/// A progress's `capacity`: each left out at the header's value.
#[derive(Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Capacity {
    energy: u16,
    reserve: u16,
    missiles: u16,
    super_missiles: u16,
    power_bombs: u16,
}

/// The header's capacities: `{ energy = 99, reserve = 0, missiles = 0, super_missiles = 0,
/// power_bombs = 0 }`.
impl Default for Capacity {
    fn default() -> Self {
        Self {
            energy: HEADER_ENERGY,
            reserve: 0,
            missiles: 0,
            super_missiles: 0,
            power_bombs: 0,
        }
    }
}

impl State {
    /// The progress this state gives.
    pub fn progress(&self) -> Progress {
        let capacity = &self.capacity;
        let capacity = Supplies(EnumMap::from_fn(|supply| {
            Wrapping(match supply {
                Supply::Energy => capacity.energy,
                Supply::Reserve => capacity.reserve,
                Supply::Missiles => capacity.missiles,
                Supply::SuperMissiles => capacity.super_missiles,
                Supply::PowerBombs => capacity.power_bombs,
            })
        }));
        let held = Supplies(EnumMap::from_fn(|supply| {
            Wrapping(match supply {
                Supply::Energy => self.energy,
                Supply::Reserve => self.reserve_energy,
                Supply::Missiles => self.missiles,
                Supply::SuperMissiles => self.super_missiles,
                Supply::PowerBombs => self.power_bombs,
            })
        }));
        let Named(reserve_mode) = self.reserve_mode;
        Progress {
            inventory: Inventory {
                items: set(&self.items),
                items_switched_on: set(&self.equipped),
                beams: set(&self.beams),
                beams_switched_on: set(&self.beams_equipped),
                capacity,
                held,
                reserve_mode,
            },
            events: set(&self.events),
            zebetites: Wrapping(self.zebetites),
            defeated: per_member(&self.defeated, |kinds| set(kinds)),
            emptied: Remembered(self.emptied.iter().map(|&id| LocationId(id)).collect()),
            opened: Remembered(self.opened.iter().map(|&id| DoorId(id)).collect()),
            maps: per_member(&self.maps, |&Named(map)| map),
            stations: per_member(&self.stations, |ids| {
                ids.iter().map(|&id| StationId(id)).collect()
            }),
            items_placed: Wrapping(self.items_placed),
        }
    }
}

/// A report, by its `report` key. One that concerns an area may name it in `area`; left out, it
/// is the frame's.
#[derive(Deserialize)]
#[serde(tag = "report", deny_unknown_fields)]
enum ReportRow {
    Happened {
        event: Named<Event>,
    },
    ZebetiteDestroyed {},
    Defeated {
        boss: Named<BossKind>,
        area: Option<Named<Area>>,
    },
    Taken {
        item: Taken,
        location: Option<u16>,
        amount: Option<u16>,
    },
    DoorOpened {
        door: Option<u16>,
    },
    MapStation {
        area: Option<Named<Area>>,
    },
    MapOnArrival {
        area: Option<Named<Area>>,
    },
    LeftArea {
        area: Option<Named<Area>>,
    },
    Saved {
        station: u16,
        area: Option<Named<Area>>,
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
    /// The report, made in a frame whose area is `frame_area`.
    fn report(&self, frame_area: Area) -> Report {
        let area = |named: Option<Named<Area>>| named.map_or(frame_area, |Named(area)| area);
        match *self {
            Self::Happened {
                event: Named(event),
            } => Report::Happened(event),
            Self::ZebetiteDestroyed {} => Report::ZebetiteDestroyed,
            Self::Defeated {
                boss: Named(boss),
                area: named,
            } => Report::Defeated {
                boss,
                area: area(named),
            },
            Self::Taken {
                item,
                location,
                amount,
            } => Report::Taken {
                pickup: pickup(item, amount),
                location: location.map(LocationId),
            },
            Self::DoorOpened { door } => Report::DoorOpened(door.map(DoorId)),
            Self::MapStation { area: named } => Report::MapStation(area(named)),
            Self::MapOnArrival { area: named } => Report::MapOnArrival(area(named)),
            Self::LeftArea { area: named } => Report::LeftArea(area(named)),
            Self::Saved {
                station,
                area: named,
            } => Report::Saved {
                area: area(named),
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
    emptied: Option<Vec<u16>>,
    opened: Option<Vec<u16>>,
    pub map_known: Option<Vec<Named<Area>>>,
    pub zebetites: bool,
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

    /// The locations asked about, by id, if any are.
    pub fn emptied(&self) -> Option<Vec<LocationId>> {
        let asked = self.emptied.as_ref()?;
        Some(asked.iter().map(|&id| LocationId(id)).collect())
    }

    /// The doors asked about, by id, if any are.
    pub fn opened(&self) -> Option<Vec<DoorId>> {
        let asked = self.opened.as_ref()?;
        Some(asked.iter().map(|&id| DoorId(id)).collect())
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
