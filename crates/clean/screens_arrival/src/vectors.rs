//! Runs every vector of `scenarios/vectors/screens-arrival.toml` through the model. The file's
//! header gives its format; any key it doesn't name fails the read.

use core::num::Wrapping;

use crate::catalogue::{BehaviourId, catalogue};
use crate::{Preparation, Report, Rules, SUIT_COLOURS, Start, SuitColours, step};
use serde::Deserialize;
use types::arrival::{ArrivalStage, ArrivalState, FadeFrames};
use types::brightness::{Blanking, Brightness, Level};
use types::{Checked, Colour, Facing, PALETTE_SLOTS, Pair, Palette};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct VectorFile {
    vector: Vec<Vector>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Vector {
    id: String,
    source: Source,
    entries: Vec<BehaviourName>,
    rules: Option<RulesRow>,
    before: StateRow,
    inputs: Option<InputsRow>,
    after: StateRow,
    output: Option<OutputRow>,
    continues: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum Source {
    Recorded,
    Constructed,
}

/// A behaviour's name as the vectors write it: each the name of a `BehaviourId`.
#[derive(Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
enum BehaviourName {
    StartsInTheDark,
    ArrivesAtTheTop,
    PictureBrightens,
    PlayBegins,
}

impl BehaviourName {
    const fn id(self) -> BehaviourId {
        match self {
            Self::StartsInTheDark => BehaviourId::StartsInTheDark,
            Self::ArrivesAtTheTop => BehaviourId::ArrivesAtTheTop,
            Self::PictureBrightens => BehaviourId::PictureBrightens,
            Self::PlayBegins => BehaviourId::PlayBegins,
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RulesRow {
    rhythm: FadeFrames,
}

#[derive(Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(deny_unknown_fields)]
struct StateRow {
    stage: ArrivalStage,
    level: Level,
    screen: Blanking,
    count: FadeFrames,
}

impl StateRow {
    const fn state(self) -> ArrivalState {
        ArrivalState {
            stage: self.stage,
            brightness: Brightness {
                level: self.level,
                blanking: self.screen,
            },
            count: self.count,
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct InputsRow {
    start_ready: bool,
    arrival_x: String,
    suit: Option<Vec<String>>,
    colours: Option<GoalColoursRow>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GoalColoursRow {
    default: GoalRow,
    at: Vec<IndexedGoal>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GoalRow {
    goal: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct IndexedGoal {
    index: usize,
    goal: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OutputRow {
    arrives: Option<[String; 2]>,
    pad: Option<[String; 2]>,
    map_shown: Option<bool>,
    play_begins: Option<bool>,
    others: Option<Others>,
    goal: Option<GoalAfter>,
    now: Option<Vec<IndexedNow>>,
}

#[derive(Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum Others {
    Goal,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum GoalAfter {
    Kept,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct IndexedNow {
    index: usize,
    now: String,
}

/// The base the vector file writes its numbers in: hexadecimal.
const VECTOR_NUMBER_BASE: u32 = 16;

/// A number as the vector file writes it, in hexadecimal.
fn hex(text: &str) -> u16 {
    u16::from_str_radix(text, VECTOR_NUMBER_BASE).unwrap()
}

fn colour(text: &str) -> Colour {
    Colour::new(hex(text)).unwrap()
}

fn place([x, y]: &[String; 2]) -> Pair<Wrapping<u16>> {
    Pair {
        x: Wrapping(hex(x)),
        y: Wrapping(hex(y)),
    }
}

/// Black stands in for any colour a vector doesn't give: such a vector checks no colour.
fn suit(row: Option<&Vec<String>>) -> SuitColours {
    let mut suit = SuitColours([Colour::BLACK; SUIT_COLOURS]);
    for (slot, text) in suit.0.iter_mut().zip(row.into_iter().flatten()) {
        *slot = colour(text);
    }
    suit
}

fn goal(row: Option<&GoalColoursRow>) -> Palette {
    let Some(row) = row else {
        return Palette([Colour::BLACK; PALETTE_SLOTS]);
    };
    let mut goal = Palette([colour(&row.default.goal); PALETTE_SLOTS]);
    for listed in &row.at {
        goal.0[listed.index] = colour(&listed.goal);
    }
    goal
}

/// The start the vector gives, if it is ready this frame.
fn ready_start(inputs: Option<&InputsRow>) -> Option<Start> {
    inputs.filter(|row| row.start_ready).map(|row| Start {
        x: Wrapping(hex(&row.arrival_x)),
        suit: suit(row.suit.as_ref()),
        goal: goal(row.colours.as_ref()),
    })
}

/// What differs between the vector's output and the report, one line each.
fn output_mismatches(output: Option<&OutputRow>, report: &Report, goal: &Palette) -> Vec<String> {
    let mut wrong = Vec::new();
    let arrives = output.and_then(|o| o.arrives.as_ref()).map(place);
    let pad = output.and_then(|o| o.pad.as_ref()).map(place);
    let map_shown = output.and_then(|o| o.map_shown).unwrap_or(false);
    let play_begins = output.and_then(|o| o.play_begins).unwrap_or(false);
    let place_now = report.arrival.as_ref().map(|a| a.place);
    if arrives != place_now {
        wrong.push(format!("arrives: expected {arrives:?}, got {place_now:?}"));
    }
    if pad != place_now {
        wrong.push(format!("pad: expected {pad:?}, got {place_now:?}"));
    }
    if map_shown != report.arrival.is_some() {
        wrong.push(format!("map_shown: expected {map_shown}"));
    }
    if play_begins != report.play_begins {
        wrong.push(format!("play_begins: expected {play_begins}"));
    }
    if report
        .arrival
        .as_ref()
        .is_some_and(|a| a.facing != Facing::Screen)
    {
        wrong.push("facing: expected the screen".to_owned());
    }
    if let Some(row) = output {
        wrong.extend(colour_mismatches(row, report, goal));
    }
    wrong
}

/// The screen's colours the vector lists, then every other one against its goal.
fn colour_mismatches(row: &OutputRow, report: &Report, goal: &Palette) -> Vec<String> {
    let Some(now) = &row.now else {
        return Vec::new();
    };
    let Some(arrival) = &report.arrival else {
        return vec!["colours: expected, but she didn't arrive".to_owned()];
    };
    let mut expected = goal.clone();
    assert!(row.others == Some(Others::Goal), "others must be \"goal\"");
    for listed in now {
        expected.0[listed.index] = colour(&listed.now);
    }
    (0..PALETTE_SLOTS)
        .filter(|&i| expected.0[i] != arrival.colours.0[i])
        .map(|i| {
            format!(
                "colour {i}: expected {:04X}, got {:04X}",
                expected.0[i].bits(),
                arrival.colours.0[i].bits()
            )
        })
        .collect()
}

/// Runs one vector; what it gets wrong, one line each.
fn run(vector: &Vector) -> Vec<String> {
    let rules = vector
        .rules
        .as_ref()
        .map_or(Rules::REFERENCE, |row| Rules { rhythm: row.rhythm });
    let start = ready_start(vector.inputs.as_ref());
    let preparation = start
        .as_ref()
        .map_or(Preparation::Underway, Preparation::Ready);
    let goal_before = start
        .as_ref()
        .map_or(Palette([Colour::BLACK; PALETTE_SLOTS]), |s| s.goal.clone());
    let mut state = vector.before.state();
    let report = step(&mut state, preparation, rules);
    let mut wrong = Vec::new();
    let expected = vector.after.state();
    if state != expected {
        wrong.push(format!("after: expected {expected:?}, got {state:?}"));
    }
    if report.brightness != expected.brightness {
        wrong.push(format!("brightness: got {:?}", report.brightness));
    }
    if let Some(start) = &start
        && vector.output.as_ref().is_some_and(|o| o.goal.is_some())
        && start.goal != goal_before
    {
        wrong.push("goal: expected kept".to_owned());
    }
    wrong.extend(output_mismatches(
        vector.output.as_ref(),
        &report,
        &goal_before,
    ));
    wrong
}

fn vectors() -> Vec<Vector> {
    toml::from_str::<VectorFile>(&vector_file::read("screens-arrival").unwrap())
        .unwrap()
        .vector
}

#[test]
fn every_vector_passes() {
    let vectors = vectors();
    assert!(!vectors.is_empty());
    let tables = vector_file::read("screens-arrival")
        .unwrap()
        .lines()
        .filter(|line| line.starts_with("[[vector]]"))
        .count();
    assert_eq!(vectors.len(), tables);
    let failures: Vec<String> = vectors
        .iter()
        .flat_map(|v| {
            run(v)
                .into_iter()
                .map(move |line| format!("{}: {line}", v.id))
        })
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn a_continued_vector_starts_where_its_previous_frame_ended() {
    let vectors = vectors();
    for vector in &vectors {
        let Some(previous) = &vector.continues else {
            continue;
        };
        let previous = vectors.iter().find(|v| &v.id == previous).unwrap();
        assert_eq!(previous.after, vector.before, "{}", vector.id);
    }
}

#[test]
fn every_vector_has_a_source_and_an_entry() {
    for vector in vectors() {
        assert!(!vector.entries.is_empty(), "{}", vector.id);
        assert!(matches!(
            vector.source,
            Source::Recorded | Source::Constructed
        ));
    }
}

#[test]
fn every_behaviour_checked_by_vectors_has_one() {
    let vectors = vectors();
    for (id, behaviour) in catalogue() {
        assert_eq!(behaviour.id, id);
        if behaviour.checked == Checked::ByVectors {
            let exercised = vectors
                .iter()
                .any(|v| v.entries.iter().any(|name| name.id() == id));
            assert!(exercised, "{id:?} has no vector");
        }
    }
}

#[test]
fn the_state_saves_and_restores_as_it_is() {
    for vector in vectors() {
        let state = vector.after.state();
        let saved = toml::to_string(&state).unwrap();
        assert_eq!(toml::from_str::<ArrivalState>(&saved).unwrap(), state);
    }
}
