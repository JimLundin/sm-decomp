//! Runs every vector of `scenarios/vectors/progress.toml` through the model, and checks the
//! catalogue against the vectors. Any key the file holds that this test doesn't know stops it.
#![cfg(test)]

mod support;

use std::collections::BTreeSet;
use std::fmt::Debug;
use std::num::Wrapping;

use clean_model_progress::{
    Beams, BossKinds, CATALOGUE, Condition, Count, Events, Items, PerArea, Pickup, Progress,
    Remembered, Report, RoomEntry, RoomStates, Rules,
};
use clean_types::{Area, Checked, DoorId, Entrance, LocationId, StationId};
use support::names::{
    AREAS, BEAMS, BOSS_KINDS, EVENTS, HELD, ITEMS, MAPS, RESERVE_MODES, SUPPLIES, TANKS, lookup,
    named,
};
use support::toml::{Fields, Table, Value, read_vectors};

const VECTOR_FILE: &str = include_str!("../../../../scenarios/vectors/progress.toml");

/// What a vector checks: a frame of reports, or questions.
enum Check {
    /// The reports of a frame, and the progress at its end.
    Frame {
        reports: Vec<Report>,
        after: Box<Progress>,
    },
    /// The questions other domains ask, and the answers.
    Asks { asks: Asks, answers: Answers },
}

/// One vector of the file.
struct Vector {
    id: String,
    entries: Vec<String>,
    area: Area,
    before: Progress,
    check: Check,
}

/// The questions of an asking vector.
#[derive(Default)]
struct Asks {
    room_states: Option<Vec<Condition>>,
    entered_by: Option<Entrance>,
    percentage: bool,
    emptied: Option<Vec<Option<LocationId>>>,
    opened: Option<Vec<Option<DoorId>>>,
    map_known: Option<Vec<Area>>,
    zebetites: bool,
}

/// The answers to an asking vector's questions, compared whole.
#[derive(Debug, Default, PartialEq, Eq)]
struct Answers {
    room_state: Option<usize>,
    percentage: Option<u16>,
    emptied: Option<Vec<bool>>,
    opened: Option<Vec<bool>>,
    map_known: Option<Vec<bool>>,
    zebetites: Option<u16>,
}

fn list_of<T>(value: &Value, read: impl Fn(&Value) -> T) -> Vec<T> {
    value.array().iter().map(read).collect()
}

fn set_of<T: Ord>(value: &Value, read: impl Fn(&Value) -> T) -> BTreeSet<T> {
    value.array().iter().map(read).collect()
}

fn per_area<T: Default>(value: &Value, read: impl Fn(&Value) -> T) -> PerArea<T> {
    let mut each = PerArea::default();
    for (area, v) in value.table() {
        each[named(&AREAS, area)] = read(v);
    }
    each
}

fn count(value: &Value) -> Count {
    Wrapping(value.u16())
}

fn items(value: &Value) -> Items {
    Items::of(&list_of(value, |i| named(&ITEMS, i.str())))
}

fn beams(value: &Value) -> Beams {
    Beams::of(&list_of(value, |b| named(&BEAMS, b.str())))
}

fn boss_kinds(value: &Value) -> BossKinds {
    BossKinds::of(&list_of(value, |b| named(&BOSS_KINDS, b.str())))
}

/// A progress in the file's layout: every key left out at its start-of-game value.
fn progress(value: &Value) -> Progress {
    let mut progress = Progress::new_file();
    for (key, v) in value.table() {
        set_field(&mut progress, key, v);
    }
    progress
}

fn set_field(progress: &mut Progress, key: &str, v: &Value) {
    match key {
        "events" => progress.events = Events::of(&list_of(v, |e| named(&EVENTS, e.str()))),
        "zebetites" => progress.zebetites = count(v),
        "defeated" => progress.defeated = per_area(v, boss_kinds),
        "emptied" => progress.emptied = Remembered::of(list_of(v, |n| LocationId(n.u16()))),
        "opened" => progress.opened = Remembered::of(list_of(v, |n| DoorId(n.u16()))),
        "maps" => progress.maps = per_area(v, |m| named(&MAPS, m.str())),
        "stations" => progress.stations = per_area(v, |s| set_of(s, |n| StationId(n.u16()))),
        "items_placed" => progress.placements = count(v),
        other => set_inventory_field(progress, other, v),
    }
}

fn set_inventory_field(progress: &mut Progress, key: &str, v: &Value) {
    let inventory = &mut progress.inventory;
    if let Some(supply) = lookup(&HELD, key) {
        inventory.held[supply] = count(v);
        return;
    }
    match key {
        "items" => inventory.items = items(v),
        "equipped" => inventory.equipped = items(v),
        "beams" => inventory.beams = beams(v),
        "beams_equipped" => inventory.beams_equipped = beams(v),
        "reserve_mode" => inventory.reserve_mode = named(&RESERVE_MODES, v.str()),
        "capacity" => {
            for (supply, n) in v.table() {
                inventory.capacity[named(&SUPPLIES, supply)] = count(n);
            }
        }
        other => panic!("unknown progress key {other:?}"),
    }
}

fn pickup(fields: &mut Fields) -> Pickup {
    let value = fields.need("item");
    let name = value.str();
    if let Some(supply) = lookup(&TANKS, name) {
        let amount = count(&fields.need("amount"));
        return Pickup::Tank { supply, amount };
    }
    lookup(&ITEMS, name).map_or_else(|| Pickup::Beam(named(&BEAMS, name)), Pickup::Item)
}

/// A report of the file, made in a frame whose area is `area`: the reports that concern an area
/// carry it.
fn report(value: &Value, area: Area) -> Report {
    let mut fields = Fields::new(value.table(), "report");
    let kind = fields.need("report");
    let report = match kind.str() {
        "Happened" => Report::Happened(named(&EVENTS, fields.need("event").str())),
        "ZebetiteDestroyed" => Report::ZebetiteDestroyed,
        "Defeated" => Report::Defeated {
            kind: named(&BOSS_KINDS, fields.need("boss").str()),
            area,
        },
        "Taken" => Report::Taken {
            pickup: pickup(&mut fields),
            location: fields.take("location").map(|n| LocationId(n.u16())),
        },
        "DoorOpened" => Report::DoorOpened(fields.take("door").map(|n| DoorId(n.u16()))),
        "MapStation" => Report::MapStation(area),
        "MapOnArrival" => Report::MapOnArrival(area),
        "LeftArea" => Report::LeftArea(area),
        "Saved" => Report::Saved {
            area,
            station: StationId(fields.need("station").u16()),
        },
        "ItemsPlaced" => Report::ItemsPlaced(count(&fields.need("count"))),
        other => panic!("unknown report {other:?}"),
    };
    fields.done();
    report
}

fn condition(value: &Value) -> Condition {
    let mut fields = Fields::new(value.table(), "condition");
    let when = fields.need("when");
    let condition = match when.str() {
        "Always" => Condition::Always,
        "Happened" => Condition::Happened(named(&EVENTS, fields.need("event").str())),
        "Defeated" => Condition::Defeated(boss_kinds(&fields.need("bosses"))),
        "Has" => Condition::Has(named(&ITEMS, fields.need("item").str())),
        "MorphBallAndMissiles" => Condition::MorphBallAndMissiles,
        "PowerBombs" => Condition::PowerBombs,
        "EnteredBy" => Condition::EnteredBy(Entrance(fields.need("entrance").u16())),
        other => panic!("unknown condition {other:?}"),
    };
    fields.done();
    condition
}

fn ids<T>(value: &Value, key: &str, id: impl Fn(u16) -> T) -> Vec<Option<T>> {
    let question = |q: &Value| {
        let mut fields = Fields::new(q.table(), key);
        let asked = fields.take(key).map(|n| id(n.u16()));
        fields.done();
        asked
    };
    value.array().iter().map(question).collect()
}

fn bools(value: &Value) -> Vec<bool> {
    value.array().iter().map(Value::bool).collect()
}

fn asks(value: &Value) -> Asks {
    let mut fields = Fields::new(value.table(), "asks");
    let asks = Asks {
        room_states: fields.take("room_states").map(|s| list_of(&s, condition)),
        entered_by: fields.take("entered_by").map(|n| Entrance(n.u16())),
        percentage: fields.take("percentage").is_some_and(|p| p.bool()),
        emptied: fields
            .take("emptied")
            .map(|q| ids(&q, "location", LocationId)),
        opened: fields.take("opened").map(|q| ids(&q, "door", DoorId)),
        map_known: fields
            .take("map_known")
            .map(|a| a.array().iter().map(|n| named(&AREAS, n.str())).collect()),
        zebetites: fields.take("zebetites").is_some_and(|z| z.bool()),
    };
    fields.done();
    asks
}

fn answers(value: &Value) -> Answers {
    let mut fields = Fields::new(value.table(), "answers");
    let answers = Answers {
        room_state: fields.take("room_state").map(|n| usize::from(n.u16())),
        percentage: fields.take("percentage").map(|n| n.u16()),
        emptied: fields.take("emptied").map(|b| bools(&b)),
        opened: fields.take("opened").map(|b| bools(&b)),
        map_known: fields.take("map_known").map(|b| bools(&b)),
        zebetites: fields.take("zebetites").map(|n| n.u16()),
    };
    fields.done();
    answers
}

fn vector(table: &Table) -> Vector {
    let mut fields = Fields::new(table, "vector");
    let id = fields.need("id").str().to_owned();
    fields.need("source");
    let entries = fields.need("entries");
    let area = named(&AREAS, fields.need("area").str());
    let before = progress(&fields.need("before"));
    let check = match fields.take("asks") {
        Some(asked) => Check::Asks {
            asks: asks(&asked),
            answers: answers(&fields.need("answers")),
        },
        None => Check::Frame {
            reports: list_of(&fields.need("reports"), |r| report(r, area)),
            after: Box::new(progress(&fields.need("after"))),
        },
    };
    fields.done();
    Vector {
        id,
        entries: entries.array().iter().map(|e| e.str().to_owned()).collect(),
        area,
        before,
        check,
    }
}

fn vectors() -> Vec<Vector> {
    read_vectors(VECTOR_FILE).iter().map(vector).collect()
}

fn answer(asks: &Asks, progress: &Progress, area: Area) -> Answers {
    let room = RoomEntry {
        area,
        entered_by: asks.entered_by,
    };
    let room_state = |states: &Vec<Condition>| {
        RoomStates::new(states)
            .expect("a room's states end with Always")
            .choose(progress, room)
    };
    Answers {
        room_state: asks.room_states.as_ref().map(room_state),
        percentage: asks
            .percentage
            .then(|| progress.inventory.item_percentage(&Rules::REFERENCE).0),
        emptied: asks
            .emptied
            .as_ref()
            .map(|q| q.iter().map(|&l| progress.emptied.contains(l)).collect()),
        opened: asks
            .opened
            .as_ref()
            .map(|q| q.iter().map(|&d| progress.opened.contains(d)).collect()),
        map_known: asks
            .map_known
            .as_ref()
            .map(|q| q.iter().map(|&a| progress.maps[a].is_known()).collect()),
        zebetites: asks.zebetites.then_some(progress.zebetites.0),
    }
}

fn differ<T: PartialEq + Debug>(out: &mut Vec<String>, field: &str, actual: &T, expected: &T) {
    if actual != expected {
        out.push(format!("{field}: expected {expected:?}, actual {actual:?}"));
    }
}

/// Each field of the progress that differs, named.
fn differences(actual: &Progress, expected: &Progress) -> Vec<String> {
    let mut out = Vec::new();
    let (a, e) = (&actual.inventory, &expected.inventory);
    let o = &mut out;
    differ(o, "items", &a.items, &e.items);
    differ(o, "equipped", &a.equipped, &e.equipped);
    differ(o, "beams", &a.beams, &e.beams);
    differ(o, "beams_equipped", &a.beams_equipped, &e.beams_equipped);
    differ(&mut out, "capacity", &a.capacity, &e.capacity);
    differ(&mut out, "held", &a.held, &e.held);
    differ(&mut out, "reserve_mode", &a.reserve_mode, &e.reserve_mode);
    let (a, e) = (actual, expected);
    differ(&mut out, "events", &a.events, &e.events);
    differ(&mut out, "zebetites", &a.zebetites, &e.zebetites);
    differ(&mut out, "defeated", &a.defeated, &e.defeated);
    differ(&mut out, "emptied", &a.emptied, &e.emptied);
    differ(&mut out, "opened", &a.opened, &e.opened);
    differ(&mut out, "maps", &a.maps, &e.maps);
    differ(&mut out, "stations", &a.stations, &e.stations);
    differ(&mut out, "items_placed", &a.placements, &e.placements);
    if out.is_empty() {
        differ(&mut out, "progress", actual, expected);
    }
    out
}

/// What fails of one vector; nothing when it passes.
fn run(vector: &Vector) -> Vec<String> {
    match &vector.check {
        Check::Frame { reports, after } => {
            let mut progress = vector.before.clone();
            for &report in reports {
                progress.apply(&Rules::REFERENCE, report);
            }
            differences(&progress, after)
        }
        Check::Asks { asks, answers } => {
            let mut out = Vec::new();
            let actual = answer(asks, &vector.before, vector.area);
            differ(&mut out, "answers", &actual, answers);
            out
        }
    }
}

#[test]
fn every_vector_passes() {
    let vectors = vectors();
    let headers = VECTOR_FILE
        .lines()
        .filter(|line| line.trim() == "[[vector]]")
        .count();
    assert!(!vectors.is_empty(), "the vector file holds no vectors");
    assert_eq!(vectors.len(), headers, "every vector of the file is read");
    let failures: Vec<String> = vectors
        .iter()
        .flat_map(|v| run(v).into_iter().map(|f| format!("{}: {f}", v.id)))
        .collect();
    assert!(
        failures.is_empty(),
        "{} of {} vectors' fields fail:\n{}",
        failures.len(),
        vectors.len(),
        failures.join("\n")
    );
}

#[test]
fn every_vector_names_catalogue_entries() {
    for vector in vectors() {
        for entry in &vector.entries {
            assert!(
                CATALOGUE.iter().any(|b| b.name == entry),
                "{}: {entry} is no catalogue entry",
                vector.id
            );
        }
    }
}

#[test]
fn every_entry_checked_by_vectors_has_one() {
    let vectors = vectors();
    for behaviour in CATALOGUE.iter().filter(|b| b.checked == Checked::ByVectors) {
        let name = behaviour.name;
        assert!(
            vectors.iter().any(|v| v.entries.iter().any(|e| e == name)),
            "{name} is checked by vectors, but no vector names it"
        );
    }
}
