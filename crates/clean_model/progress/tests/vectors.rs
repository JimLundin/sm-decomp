//! Runs every vector of `scenarios/vectors/progress.toml` through the model, and checks the
//! catalogue against the vectors. The file is read with `toml` into the shape `support`
//! declares, which refuses any key or name this test doesn't know.
#![cfg(test)]

mod support;

use std::fmt::Debug;

use clean_model_progress::{
    Condition, EnteredRoom, Progress, RoomStates, Rules, Version, catalogue,
};
use clean_types::{Area, Checked, Entrance};
use support::{Answers, Asks, Check, Named, State, Vector, VectorFile};

const VECTOR_FILE: &str = include_str!("../../../../scenarios/vectors/progress.toml");

/// The file's text, read into its shape.
fn read(text: &str) -> Result<VectorFile, toml::de::Error> {
    toml::from_str(text)
}

fn vectors() -> Vec<Vector> {
    read(VECTOR_FILE)
        .unwrap_or_else(|error| panic!("the vector file doesn't read: {error}"))
        .vector
}

/// The model's answers to `asks`, about `progress` in a room of `area`, under `rules`.
fn answer(asks: &Asks, progress: &Progress, area: Area, rules: &Rules) -> Answers {
    let room = EnteredRoom {
        area,
        entered_by: asks.entered_by.map(Entrance),
    };
    // The file numbers the default version after the conditional ones.
    let room_state = |conditional: Vec<Condition>| {
        let states = RoomStates {
            conditional: &conditional,
        };
        match states.choose(progress, room) {
            Version::Conditional(index) => index,
            Version::Default => conditional.len(),
        }
    };
    Answers {
        room_state: asks.room_states().map(room_state),
        percentage: asks
            .percentage
            .then(|| progress.inventory.item_percentage(rules).0),
        emptied: asks.emptied().map(|q| {
            q.into_iter()
                .map(|l| progress.emptied.contains(l))
                .collect()
        }),
        opened: asks
            .opened()
            .map(|q| q.into_iter().map(|d| progress.opened.contains(d)).collect()),
        map_known: asks.map_known.as_ref().map(|q| {
            q.iter()
                .map(|&Named(a)| progress.maps[a].is_known())
                .collect()
        }),
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
    differ(o, "equipped", &a.items_switched_on, &e.items_switched_on);
    differ(o, "beams", &a.beams, &e.beams);
    differ(
        o,
        "beams_equipped",
        &a.beams_switched_on,
        &e.beams_switched_on,
    );
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
    differ(&mut out, "items_placed", &a.items_placed, &e.items_placed);
    if out.is_empty() {
        differ(&mut out, "progress", actual, expected);
    }
    out
}

/// What fails of one vector; nothing when it passes.
fn run(vector: &Vector) -> Vec<String> {
    let (before, rules) = (vector.before.progress(), vector.rules());
    match vector.check() {
        Check::Frame { reports, after } => {
            let mut progress = before;
            progress.apply(&rules, &reports);
            differences(&progress, &after)
        }
        Check::Asks { asks, answers } => {
            let mut out = Vec::new();
            let Named(area) = vector.area;
            let answers_given = answer(asks, &before, area, &rules);
            differ(&mut out, "answers", &answers_given, answers);
            out
        }
    }
}

#[test]
fn every_vector_passes() {
    let vectors = vectors();
    assert!(!vectors.is_empty(), "the vector file holds no vectors");
    let failures: Vec<String> = vectors
        .iter()
        .flat_map(|v| {
            let at = format!("{} ({})", v.id, v.source);
            run(v).into_iter().map(move |f| format!("{at}: {f}"))
        })
        .collect();
    assert!(
        failures.is_empty(),
        "{} of {} vectors' fields fail:\n{}",
        failures.len(),
        vectors.len(),
        failures.join("\n")
    );
}

/// Spec §1.1, The start of a run: the model's new file is the start the vector file's header
/// lists, which fills every field a vector leaves out.
#[test]
fn a_new_file_is_the_headers_start() {
    let header = State::default().progress();
    let failures = differences(&Progress::default(), &header);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// A vector of one frame whose `before` and reports are given, the rest the least the file allows.
fn frame(before: &str, reports: &str) -> String {
    format!(
        "[[vector]]\nid = \"x\"\nsource = \"constructed\"\nentries = []\narea = \"Crateria\"\n\
         before = {before}\nreports = [{reports}]\nafter = {{}}\n"
    )
}

/// A vector of one frame, with no reports, run under the `rules` given.
fn ruled(rules: &str) -> String {
    format!(
        "[[vector]]\nid = \"x\"\nsource = \"constructed\"\nentries = []\nrules = {rules}\n\
         area = \"Crateria\"\nbefore = {{}}\nreports = []\nafter = {{}}\n"
    )
}

/// Divisors with every key, the game's own.
const DIVISORS: &str =
    "divisors = { energy = 100, reserve = 100, missiles = 5, super_missiles = 5, power_bombs = 5 }";

/// A vector of questions whose asks and answers are given.
fn questions(asks: &str, answers: &str) -> String {
    format!(
        "[[vector]]\nid = \"x\"\nsource = \"constructed\"\nentries = []\narea = \"Crateria\"\n\
         before = {{}}\nasks = {asks}\nanswers = {answers}\n"
    )
}

#[test]
fn unknown_keys_and_names_fail_in_every_table() {
    let known = [
        frame("{}", "{ report = \"ZebetiteDestroyed\" }"),
        frame(
            "{ capacity = { energy = 1 }, maps = { Tourian = \"OnArrival\" } }",
            "",
        ),
        frame(
            "{}",
            "{ report = \"Taken\", item = \"MissileTank\", amount = 5 }",
        ),
        frame(
            "{}",
            "{ report = \"LeftArea\", area = \"Tourian\" }, { report = \"Defeated\", \
             boss = \"Torizo\", area = \"Norfair\" }, { report = \"Saved\", station = 0, \
             area = \"Ceres\" }, { report = \"MapStation\", area = \"Maridia\" }, \
             { report = \"MapOnArrival\", area = \"WreckedShip\" }",
        ),
        questions(
            "{ room_states = [{ when = \"Always\" }] }",
            "{ room_state = 0 }",
        ),
        questions("{ emptied = [1], opened = [0, 2] }", "{}"),
        ruled("{}"),
        ruled(&format!(
            "{{ switches_off = {{ Ice = [\"Charge\", \"Wave\"] }}, {DIVISORS} }}"
        )),
    ];
    for text in &known {
        assert!(read(text).is_ok(), "a known vector fails to read:\n{text}");
    }
    let unknown = [
        format!("{}extra = 1\n", known[0]),
        frame("{ extra = 1 }", ""),
        frame("{ capacity = { extra = 1 } }", ""),
        frame("{ maps = { Zebes = \"Downloaded\" } }", ""),
        frame("{ maps = { Tourian = \"Unmapped\" } }", ""),
        frame("{ defeated = { Ceres = [\"Ridley\"] } }", ""),
        frame("{ items = [\"Boots\"] }", ""),
        frame("{ reserve_mode = \"NoTank\" }", ""),
        frame("{}", "{ report = \"ZebetiteDestroyed\", extra = 1 }"),
        frame("{}", "{ report = \"Exploded\" }"),
        frame("{}", "{ report = \"Taken\", item = \"Boots\" }"),
        frame("{}", "{ report = \"Taken\", item = \"Bombs\", extra = 1 }"),
        frame("{}", "{ report = \"Happened\", event = \"PlanetAsleep\" }"),
        frame("{}", "{ report = \"LeftArea\", area = \"Zebes\" }"),
        frame(
            "{}",
            "{ report = \"ZebetiteDestroyed\", area = \"Tourian\" }",
        ),
        frame(
            "{}",
            "{ report = \"ItemsPlaced\", count = 1, area = \"Norfair\" }",
        ),
        questions("{ extra = true }", "{}"),
        questions("{}", "{ extra = 0 }"),
        questions("{ room_states = [{ when = \"Always\", extra = 1 }] }", "{}"),
        questions("{ room_states = [{ when = \"Sometimes\" }] }", "{}"),
        questions("{ emptied = [{ location = 1 }] }", "{}"),
        questions("{ opened = [{ door = 1 }] }", "{}"),
        questions("{ map_known = [\"Zebes\"] }", "{}"),
        ruled("{ extra = 1 }"),
        ruled("{ switches_off = { Hyper = [\"Charge\"] } }"),
        ruled("{ switches_off = { Ice = [\"Hyper\"] } }"),
        ruled(&format!(
            "{{ {} }}",
            DIVISORS.replace(" }", ", extra = 1 }")
        )),
        ruled(&format!("{{ {} }}", DIVISORS.replace("energy", "health"))),
        ruled(&format!("{{ {} }}", DIVISORS.replace("100,", "0,"))),
    ];
    for text in &unknown {
        assert!(read(text).is_err(), "an unknown key or name reads:\n{text}");
    }
}

#[test]
fn every_vector_names_catalogue_entries() {
    let catalogue = catalogue();
    for vector in vectors() {
        for entry in &vector.entries {
            assert!(
                catalogue.iter().any(|(id, _)| format!("{id:?}") == *entry),
                "{}: {entry} is no catalogue entry",
                vector.id
            );
        }
    }
}

#[test]
fn every_entry_checked_by_vectors_has_one() {
    let vectors = vectors();
    let catalogue = catalogue();
    for (id, _) in catalogue
        .iter()
        .filter(|(_, b)| b.checked == Checked::ByVectors)
    {
        let name = format!("{id:?}");
        assert!(
            vectors.iter().any(|v| v.entries.contains(&name)),
            "{name} is checked by vectors, but no vector names it"
        );
    }
}
