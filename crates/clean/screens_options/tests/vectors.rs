//! Runs every vector of the options menu's file through the model, and checks the catalogue
//! against the spec's entries.
#![cfg(test)]

use enum_map::EnumMap;
use screens_options::{
    BehaviourId, ChosenFile, FrameEnd, MenuAction, MenuSound, Pressed, behaviour, step,
};
use serde::Deserialize;
use toml::{Table, Value};
use types::Checked;
use types::options::{
    BlockSize, DissolveCursor, Dissolving, FileBegins, Language, LoadStep, OptionsItem,
    OptionsStage, OptionsState, Settings, SpecialItem, WayOut,
};
use types::power_on::{Level, ScreenLight};

/// The keys `after` may check, besides `requests` and `unsupported`.
const STATE_KEYS: [&str; 9] = [
    "stage",
    "load_step",
    "screen",
    "cursor",
    "mosaic",
    "begins",
    "japanese",
    "moonwalk",
    "weapon_reset",
];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct VectorFile {
    vector: Vec<Vector>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Vector {
    id: String,
    source: String,
    entries: Vec<String>,
    before: Before,
    finished: u8,
    #[serde(default)]
    pressed: Vec<Vec<String>>,
    #[serde(default)]
    input: Input,
    after: Table,
}

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct Input {
    begins: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Before {
    stage: String,
    load_step: Option<u8>,
    screen: (String, u8),
    cursor: Option<String>,
    mosaic: Option<u8>,
    begins: Option<String>,
    japanese: bool,
    moonwalk: bool,
    weapon_reset: bool,
}

fn vectors() -> Vec<Vector> {
    toml::from_str::<VectorFile>(&vector_file::read("screens-options").unwrap())
        .expect("the vector file parses")
        .vector
}

fn level(number: u8) -> Level {
    Level::new(number).expect("a level 0 to 15")
}

fn screen((light, number): &(String, u8)) -> ScreenLight {
    match light.as_str() {
        "blank" => ScreenLight::Blank(level(*number)),
        "lit" => ScreenLight::Lit(level(*number)),
        other => panic!("unknown screen {other}"),
    }
}

fn options_item(name: &str) -> OptionsItem {
    match name {
        "start-game" => OptionsItem::StartGame,
        "english" => OptionsItem::EnglishText,
        "japanese" => OptionsItem::JapaneseText,
        "controls" => OptionsItem::ControllerSettings,
        "special" => OptionsItem::SpecialSettings,
        other => panic!("unknown options item {other}"),
    }
}

const fn options_item_name(item: OptionsItem) -> &'static str {
    match item {
        OptionsItem::StartGame => "start-game",
        OptionsItem::EnglishText => "english",
        OptionsItem::JapaneseText => "japanese",
        OptionsItem::ControllerSettings => "controls",
        OptionsItem::SpecialSettings => "special",
    }
}

fn special_item(name: &str) -> SpecialItem {
    match name {
        "weapon-reset" => SpecialItem::WeaponReset,
        "moonwalk" => SpecialItem::Moonwalk,
        "end" => SpecialItem::End,
        other => panic!("unknown special item {other}"),
    }
}

const fn special_item_name(item: SpecialItem) -> &'static str {
    match item {
        SpecialItem::WeaponReset => "weapon-reset",
        SpecialItem::Moonwalk => "moonwalk",
        SpecialItem::End => "end",
    }
}

fn dissolve_cursor(name: &str) -> DissolveCursor {
    match name {
        "start-game" => DissolveCursor::StartGame,
        "special" => DissolveCursor::SpecialSettings,
        "controls" => DissolveCursor::ControllerSettings,
        other => panic!("unknown dissolve destination {other}"),
    }
}

const fn dissolve_cursor_name(cursor: DissolveCursor) -> &'static str {
    match cursor {
        DissolveCursor::StartGame => "start-game",
        DissolveCursor::SpecialSettings => "special",
        DissolveCursor::ControllerSettings => "controls",
    }
}

fn file_begins(name: &str) -> FileBegins {
    match name {
        "intro" => FileBegins::Intro,
        "ceres-escape" => FileBegins::Escape,
        other => panic!("unknown file beginning {other}"),
    }
}

const fn file_begins_name(begins: FileBegins) -> &'static str {
    match begins {
        FileBegins::Intro => "intro",
        FileBegins::Escape => "ceres-escape",
    }
}

fn way_out(name: &str) -> WayOut {
    match name {
        "intro-begins" => WayOut::IntroBegins,
        "escape-begins" => WayOut::EscapeBegins,
        "play-continues" => WayOut::PlayContinues,
        "file-select-again" => WayOut::FileSelectAgain,
        "controls-begin" => WayOut::ControlsBegin,
        other => panic!("unknown stage {other}"),
    }
}

const fn way_out_name(way_out: WayOut) -> &'static str {
    match way_out {
        WayOut::IntroBegins => "intro-begins",
        WayOut::EscapeBegins => "escape-begins",
        WayOut::PlayContinues => "play-continues",
        WayOut::FileSelectAgain => "file-select-again",
        WayOut::ControlsBegin => "controls-begin",
    }
}

fn chosen_file(input: &Input) -> ChosenFile {
    match input.begins.as_deref() {
        None | Some("intro") => ChosenFile::Begins(FileBegins::Intro),
        Some("play") => ChosenFile::InPlay,
        Some(other) => ChosenFile::Begins(file_begins(other)),
    }
}

/// The action a pad button stands for on the menu; the buttons the menu doesn't act on stand for
/// none.
fn action(button: &str) -> Option<MenuAction> {
    match button {
        "Up" => Some(MenuAction::Up),
        "Down" => Some(MenuAction::Down),
        "B" => Some(MenuAction::Back),
        "A" | "Start" => Some(MenuAction::Choose),
        "Left" | "Right" => Some(MenuAction::Sideways),
        "X" | "Y" | "Select" | "L" | "R" => None,
        other => panic!("unknown button {other}"),
    }
}

fn frame_end(vector: &Vector) -> FrameEnd {
    let finished = usize::from(vector.finished);
    assert!(finished <= 1, "{}: one frame end at a time", vector.id);
    assert!(
        vector.pressed.len() <= finished,
        "{}: a list of buttons for each finished frame",
        vector.id
    );
    if finished == 0 {
        return FrameEnd::Busy;
    }
    let mut pressed = Pressed::default();
    for button in vector.pressed.iter().flatten() {
        if let Some(action) = action(button) {
            pressed[action] = true;
        }
    }
    FrameEnd::Finished(pressed)
}

fn before_state(before: &Before) -> OptionsState {
    OptionsState {
        stage: before_stage(before),
        screen: screen(&before.screen),
        settings: Settings {
            language: if before.japanese {
                Language::Japanese
            } else {
                Language::English
            },
            weapon_reset: before.weapon_reset,
            moonwalk: before.moonwalk,
        },
    }
}

fn before_stage(before: &Before) -> OptionsStage {
    let cursor = || before.cursor.as_deref().expect("a cursor for this stage");
    let dissolving = || Dissolving {
        cursor: dissolve_cursor(cursor()),
        blocks: BlockSize::new(before.mosaic.expect("a block size while dissolving"))
            .expect("a block size 0 to 15"),
    };
    match before.stage.as_str() {
        "loading" => OptionsStage::Loading(match before.load_step.unwrap_or(0) {
            0 => LoadStep::First,
            1 => LoadStep::Second,
            other => panic!("unknown load step {other}"),
        }),
        "brightening" => OptionsStage::Brightening(options_item(cursor())),
        "choosing" => OptionsStage::Choosing(options_item(cursor())),
        "special" => OptionsStage::Special(special_item(cursor())),
        "dissolving-out" => OptionsStage::DissolvingOut(dissolving()),
        "dissolving-in" => OptionsStage::DissolvingIn(dissolving()),
        "leaving" => OptionsStage::Leaving {
            cursor: options_item(cursor()),
            begins: file_begins(before.begins.as_deref().expect("how the file begins")),
        },
        "returning" => OptionsStage::Returning(options_item(cursor())),
        other => OptionsStage::HandedOn(way_out(other)),
    }
}

fn text(value: &str) -> Value {
    Value::String(value.to_owned())
}

fn number(value: u8) -> Value {
    Value::Integer(i64::from(value))
}

/// The State as the vector file writes it.
fn observed(state: OptionsState) -> Table {
    let mut table = Table::new();
    let (light, shown) = match state.screen {
        ScreenLight::Blank(level) => ("blank", level),
        ScreenLight::Lit(level) => ("lit", level),
    };
    table.insert(
        "screen".into(),
        Value::Array(vec![text(light), number(shown.get())]),
    );
    let settings = state.settings;
    table.insert(
        "japanese".into(),
        Value::Boolean(settings.language == Language::Japanese),
    );
    table.insert("moonwalk".into(), Value::Boolean(settings.moonwalk));
    table.insert("weapon_reset".into(), Value::Boolean(settings.weapon_reset));
    let (stage_name, cursor) = match state.stage {
        OptionsStage::Loading(load_step) => {
            let step = match load_step {
                LoadStep::First => 0,
                LoadStep::Second => 1,
            };
            table.insert("load_step".into(), number(step));
            ("loading", None)
        }
        OptionsStage::Brightening(item) => ("brightening", Some(options_item_name(item))),
        OptionsStage::Choosing(item) => ("choosing", Some(options_item_name(item))),
        OptionsStage::Special(item) => ("special", Some(special_item_name(item))),
        OptionsStage::DissolvingOut(dissolving) => {
            table.insert("mosaic".into(), number(dissolving.blocks.get()));
            (
                "dissolving-out",
                Some(dissolve_cursor_name(dissolving.cursor)),
            )
        }
        OptionsStage::DissolvingIn(dissolving) => {
            table.insert("mosaic".into(), number(dissolving.blocks.get()));
            (
                "dissolving-in",
                Some(dissolve_cursor_name(dissolving.cursor)),
            )
        }
        OptionsStage::Leaving { cursor, begins } => {
            table.insert("begins".into(), text(file_begins_name(begins)));
            ("leaving", Some(options_item_name(cursor)))
        }
        OptionsStage::Returning(item) => ("returning", Some(options_item_name(item))),
        OptionsStage::HandedOn(way_out) => (way_out_name(way_out), None),
    };
    table.insert("stage".into(), text(stage_name));
    if let Some(cursor) = cursor {
        table.insert("cursor".into(), text(cursor));
    }
    table
}

fn sound_names(sounds: &[MenuSound]) -> Value {
    Value::Array(
        sounds
            .iter()
            .map(|sound| {
                text(match sound {
                    MenuSound::CursorMoved => "cursor-moved",
                    MenuSound::Selected => "selected",
                })
            })
            .collect(),
    )
}

/// What differs between the vector's `after` and the model's, one line each.
fn differences(vector: &Vector) -> Vec<String> {
    let mut state = before_state(&vector.before);
    let outcome = step(&mut state, frame_end(vector), chosen_file(&vector.input));
    let unsupported = vector.after.get("unsupported") == Some(&Value::Boolean(true));
    let sounds = match (outcome, unsupported) {
        (Err(_), true) => return Vec::new(),
        (Ok(sounds), false) => sounds,
        (outcome, _) => return vec![format!("unsupported {unsupported}, got {outcome:?}")],
    };
    let observed = observed(state);
    let mut differences = Vec::new();
    let expected_sounds = vector
        .after
        .get("requests")
        .cloned()
        .unwrap_or_else(|| Value::Array(Vec::new()));
    let actual_sounds = sound_names(&sounds);
    if expected_sounds != actual_sounds {
        differences.push(format!(
            "requests: expected {expected_sounds}, got {actual_sounds}"
        ));
    }
    for (key, expected) in &vector.after {
        if key == "requests" {
            continue;
        }
        assert!(
            STATE_KEYS.contains(&key.as_str()),
            "{}: unknown key {key}",
            vector.id
        );
        let actual = observed.get(key);
        if actual != Some(expected) {
            differences.push(format!("{key}: expected {expected}, got {actual:?}"));
        }
    }
    differences
}

#[test]
fn every_vector_passes() {
    let vectors = vectors();
    assert!(!vectors.is_empty(), "the file holds vectors");
    let mut failures = Vec::new();
    for vector in &vectors {
        assert!(
            ["recorded", "constructed"].contains(&vector.source.as_str()),
            "{}: unknown source {}",
            vector.id,
            vector.source
        );
        for difference in differences(vector) {
            failures.push(format!("{}: {difference}", vector.id));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

fn every_id() -> Vec<BehaviourId> {
    EnumMap::<BehaviourId, ()>::default()
        .into_iter()
        .map(|(id, ())| id)
        .collect()
}

fn id_named(name: &str) -> BehaviourId {
    every_id()
        .into_iter()
        .find(|id| format!("{id:?}") == name)
        .unwrap_or_else(|| panic!("no catalogue entry {name}"))
}

#[test]
fn every_vector_entry_is_in_the_catalogue() {
    for vector in vectors() {
        assert!(
            !vector.entries.is_empty(),
            "{}: names its entries",
            vector.id
        );
        for entry in &vector.entries {
            id_named(entry);
        }
    }
}

#[test]
fn every_entry_checked_by_vectors_has_one() {
    let vectors = vectors();
    for id in every_id() {
        let entry = behaviour(id);
        assert_eq!(entry.id, id, "the catalogue's row for {id:?} names it");
        let exercised = vectors
            .iter()
            .flat_map(|vector| &vector.entries)
            .any(|name| id_named(name) == id);
        assert_eq!(
            exercised,
            entry.checked == Checked::ByVectors,
            "{id:?}: checked by vectors exactly when a vector exercises it"
        );
    }
}
