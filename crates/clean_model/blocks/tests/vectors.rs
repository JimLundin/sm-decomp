//! The spec's vectors (`scenarios/vectors/blocks.toml`) run through the model, and the catalogue
//! checked against them. The file's shape is declared once below; every table refuses a key it
//! doesn't name, and every name is read by a table of the names the format allows.

#![cfg(test)]

use std::collections::BTreeMap;
use std::fmt::Debug;

use clean_model_blocks::{
    Answer, Becomes, Blocks, CATALOGUE, CellKind, Frames, Grid, Inside, Mark, PatternCell, Place,
    Placement, Places, Reacting, Reaction, ReactionName, Reactions, Report, Request, Rules, Run,
    Running, ScreenChange, ScrollReaction, SoundRequest, StandIn, State, Step, Tile, Touched,
};
use clean_types::{
    Cell, Checked, Family, Flip, Heading, HitBy, Look, MetAs, Motion, Pair, Picture, Point,
    ScreenKind, Script, Set, Table, Touch, Way,
};
use serde::de::Error as _;
use serde::{Deserialize, Deserializer};

/// The vector file.
const VECTORS: &str = include_str!("../../../../scenarios/vectors/blocks.toml");

/// A look's picture is written in hex.
const HEX: u32 = 16;

// The file's shape.

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    reaction: BTreeMap<String, ReactionRow>,
    vector: Vec<Vector>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ReactionRow {
    family: String,
    look: Option<String>,
    first_wait: u16,
    broken: Option<Vec<StepRow>>,
    touched: Option<Vec<StepRow>>,
    revealed: Option<Vec<StepRow>>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum StepRow {
    Sound(SoundRow),
    Show(ShowRow),
    Regrow(RegrowRow),
    End(EndRow),
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SoundRow {
    sound: String,
    queue_limit: u16,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ShowRow {
    show: Vec<(i16, i16, String, String)>,
    frames: u16,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RegrowRow {
    #[serde(rename = "regrow")]
    _regrow: True,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EndRow {
    #[serde(rename = "end")]
    _end: True,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Vector {
    id: String,
    source: String,
    entries: Vec<String>,
    before: Before,
    frame: Option<FrameRow>,
    after: After,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Before {
    cells: Vec<CellRow>,
    places: Vec<PlaceRow>,
    held: Option<Vec<usize>>,
    held_until_run: Option<Vec<usize>>,
    screens: Option<Vec<String>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct After {
    cells: Vec<CellRow>,
    places: Vec<PlaceRow>,
    held: Option<Vec<usize>>,
    held_until_run: Option<Vec<usize>>,
    screens: Option<Vec<String>>,
    answers: Option<Vec<String>>,
    redrawn: Option<Vec<(u16, u16)>>,
    sounds: Option<Vec<SoundRow>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CellRow {
    at: (u16, u16),
    kind: String,
    look: String,
    reaction: Option<String>,
    mark: Option<String>,
    met_as: Option<String>,
    reach: Option<i16>,
    inside: Option<String>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum PlaceRow {
    Running(RunningRow),
    Scroll(ScrollRow),
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RunningRow {
    place: usize,
    reaction: String,
    script: String,
    cell: (u16, u16),
    next: usize,
    wait: u16,
    remembers: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ScrollRow {
    place: usize,
    scroll: Vec<ChangeRow>,
    cell: (u16, u16),
    touched: Option<bool>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ChangeRow {
    screen: (u16, u16),
    becomes: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FrameRow {
    she: Option<SheRow>,
    requests: Option<Vec<RequestRow>>,
    paused: Option<bool>,
    placements: Option<Vec<PlacementRow>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SheRow {
    dashing: Option<bool>,
    shinesparking: Option<bool>,
    screw_attacking: Option<bool>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum RequestRow {
    Inside(InsideRow),
    Touch(TouchRow),
    Hit(HitRow),
    Other(OtherRow),
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct InsideRow {
    inside: (u16, u16),
    point: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TouchRow {
    touch: (u16, u16),
    way: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct HitRow {
    hit: (u16, u16),
    by: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OtherRow {
    #[serde(rename = "other")]
    _other: True,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum PlacementRow {
    Scroll(ScrollPlacementRow),
    StandIn(StandInRow),
    Other(OtherRow),
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ScrollPlacementRow {
    scroll: Vec<ChangeRow>,
    cell: (u16, u16),
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StandInRow {
    stand_in: String,
    reach: i16,
    cell: (u16, u16),
}

/// A key the format says is always `true`: a reader refuses `false`.
struct True;

impl<'de> Deserialize<'de> for True {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        if bool::deserialize(deserializer)? {
            Ok(Self)
        } else {
            Err(D::Error::custom("this key is always true"))
        }
    }
}

fn parse() -> File {
    toml::from_str(VECTORS).expect("the vector file reads as the format its header gives")
}

// The names the format allows.

fn family(name: &str) -> Family {
    match name {
        "Bomb" => Family::Bomb,
        "Crumble" => Family::Crumble,
        "PowerBomb" => Family::PowerBomb,
        "Shot" => Family::Shot,
        "Speed" => Family::Speed,
        "SuperMissile" => Family::SuperMissile,
        other => panic!("unknown family {other}"),
    }
}

fn met_as(name: &str) -> MetAs {
    match name {
        "Bomb" => MetAs::Bomb,
        "Crumble" => MetAs::Crumble,
        "Shot" => MetAs::Shot,
        other => panic!("unknown met_as {other}"),
    }
}

fn script(name: &str) -> Script {
    match name {
        "broken" => Script::Broken,
        "revealed" => Script::Revealed,
        "touched" => Script::Touched,
        other => panic!("unknown script {other}"),
    }
}

fn kind(name: &str) -> CellKind {
    match name {
        "Air" => CellKind::Air,
        "Block" => CellKind::Block,
        "ReactingAir" => CellKind::ReactingAir,
        "Solid" => CellKind::Solid,
        "StandInAcross" => CellKind::StandInAcross,
        "StandInDown" => CellKind::StandInDown,
        other => panic!("unknown kind {other}"),
    }
}

fn becomes(name: &str) -> Becomes {
    match name {
        "Air" => Becomes::Air,
        "Itself" => Becomes::Itself,
        "StandInAcross" => Becomes::StandInAcross,
        "StandInDown" => Becomes::StandInDown,
        other => panic!("unknown pattern cell {other}"),
    }
}

fn reacting(name: &str) -> Reacting {
    match name {
        "Block" => Reacting::Block,
        "ReactingAir" => Reacting::Air,
        other => panic!("unknown remembers {other}"),
    }
}

fn screen_kind(name: char) -> ScreenKind {
    match name {
        'C' => ScreenKind::Closed,
        'O' => ScreenKind::Open,
        'S' => ScreenKind::Short,
        other => panic!("unknown screen {other}"),
    }
}

fn screen_kind_named(name: &str) -> ScreenKind {
    match name {
        "Closed" => ScreenKind::Closed,
        "Open" => ScreenKind::Open,
        "Short" => ScreenKind::Short,
        other => panic!("unknown screen kind {other}"),
    }
}

fn point(name: &str) -> Point {
    match name {
        "Top" => Point::Top,
        "Centre" => Point::Centre,
        "Bottom" => Point::Bottom,
        other => panic!("unknown point {other}"),
    }
}

fn way(name: &str) -> Way {
    match name {
        "Down" => Way::Move(Heading::Down),
        "Left" => Way::Move(Heading::Left),
        "Look" => Way::Look,
        "Right" => Way::Move(Heading::Right),
        "Up" => Way::Move(Heading::Up),
        other => panic!("unknown way {other}"),
    }
}

fn hit_by(name: &str) -> HitBy {
    match name {
        "Beam" => HitBy::Beam,
        "Bomb" => HitBy::Bomb,
        "Grapple" => HitBy::Grapple,
        "Missile" => HitBy::Missile,
        "PowerBomb" => HitBy::PowerBomb,
        "SuperMissile" => HitBy::SuperMissile,
        other => panic!("unknown hit {other}"),
    }
}

fn answer(name: &str) -> Answer {
    match name {
        "Air" => Answer::Air,
        "Solid" => Answer::Solid,
        "Other" => Answer::Other,
        other => panic!("unknown answer {other}"),
    }
}

fn stand_in(name: &str) -> StandIn {
    match name {
        "Across" => StandIn::Across,
        "Down" => StandIn::Down,
        other => panic!("unknown stand-in {other}"),
    }
}

fn look(text: &str) -> Look {
    let (index, flip) = text.split_once(':').unwrap_or((text, ""));
    let flip = match flip {
        "" => Flip::None,
        "x" => Flip::Across,
        "y" => Flip::Down,
        "xy" => Flip::Both,
        other => panic!("unknown flip {other}"),
    };
    let index = u16::from_str_radix(index, HEX).expect("a look's picture is hex");
    let picture = Picture::new(index).expect("a picture of the tileset");
    Look { picture, flip }
}

const fn cell((x, y): (u16, u16)) -> Cell {
    Pair { x, y }
}

fn frames(count: u16) -> Frames {
    Frames::new(count).expect("frames the spec allows")
}

// The reactions.

fn sound(row: &SoundRow) -> SoundRequest {
    SoundRequest {
        sound: row.sound.clone(),
        queue_limit: row.queue_limit,
    }
}

fn step(row: &StepRow) -> Step {
    match row {
        StepRow::Sound(row) => Step::Sound(sound(row)),
        StepRow::Show(row) => Step::Show {
            pattern: row
                .show
                .iter()
                .map(|(x, y, to, seen)| PatternCell {
                    offset: Pair { x: *x, y: *y },
                    becomes: becomes(to),
                    look: look(seen),
                })
                .collect(),
            frames: frames(row.frames),
        },
        StepRow::Regrow(_) => Step::Regrow,
        StepRow::End(_) => Step::End,
    }
}

fn steps(rows: Option<&Vec<StepRow>>) -> Vec<Step> {
    rows.map(|rows| rows.iter().map(step).collect())
        .unwrap_or_default()
}

fn reactions(file: &File) -> Reactions {
    file.reaction
        .iter()
        .map(|(name, row)| {
            let mut reaction = Reaction {
                family: family(&row.family),
                whole_look: row.look.as_deref().map(look),
                first_wait: frames(row.first_wait),
                scripts: Table::default(),
            };
            reaction.scripts[Script::Broken] = steps(row.broken.as_ref());
            reaction.scripts[Script::Revealed] = steps(row.revealed.as_ref());
            reaction.scripts[Script::Touched] = steps(row.touched.as_ref());
            (ReactionName(name.clone()), reaction)
        })
        .collect()
}

// The state.

fn reaction_name(reactions: &Reactions, name: &str) -> ReactionName {
    let name = ReactionName(name.to_owned());
    assert!(reactions.contains_key(&name), "unknown reaction {}", name.0);
    name
}

fn mark(reactions: &Reactions, row: &CellRow) -> Mark {
    let given = (
        row.reaction.as_deref(),
        row.mark.as_deref(),
        row.met_as.as_deref(),
        row.reach,
    );
    match given {
        (Some(name), None, None, None) => Mark::Reaction(reaction_name(reactions, name)),
        (None, Some("Scroll"), None, None) => Mark::Scroll,
        (None, Some("Nothing"), Some(family), None) => Mark::Nothing(met_as(family)),
        (None, Some("Inert"), None, None) => Mark::Inert,
        (None, Some("Other"), family, None) => Mark::Other(family.map(met_as)),
        (None, None, None, Some(reach)) => Mark::Reach(reach),
        (None, None, None, None) => Mark::Unmarked,
        _ => panic!("a cell's mark the format doesn't give at {:?}", row.at),
    }
}

fn tile(reactions: &Reactions, row: &CellRow) -> Tile {
    Tile {
        kind: kind(&row.kind),
        mark: mark(reactions, row),
        look: look(&row.look),
        inside: match row.inside.as_deref() {
            None => Inside::Passes,
            Some("Nothing") => Inside::HoldsPlace,
            Some(other) => panic!("unknown inside {other}"),
        },
    }
}

/// A cell no vector lists: plain air.
fn plain() -> Tile {
    Tile {
        kind: CellKind::Air,
        mark: Mark::Unmarked,
        look: look("000"),
        inside: Inside::Passes,
    }
}

/// The room's cells, sized to reach every cell the vector lists.
fn tiles(reactions: &Reactions, rows: &[CellRow], size: Pair<usize>) -> Grid<Tile> {
    let mut grid = Grid {
        rows: vec![vec![plain(); size.x]; size.y],
    };
    for row in rows {
        *grid.get_mut(cell(row.at)).expect("in the room") = tile(reactions, row);
    }
    grid
}

fn room_size(vector: &Vector) -> Pair<usize> {
    let cells = vector.before.cells.iter().chain(&vector.after.cells);
    let (x, y) = cells.fold((0, 0), |(x, y), row| (x.max(row.at.0), y.max(row.at.1)));
    Pair {
        x: usize::from(x).saturating_add(1),
        y: usize::from(y).saturating_add(1),
    }
}

fn changes(rows: &[ChangeRow]) -> Vec<ScreenChange> {
    rows.iter()
        .map(|row| ScreenChange {
            screen: cell(row.screen),
            becomes: screen_kind_named(&row.becomes),
        })
        .collect()
}

fn running(reactions: &Reactions, row: &RunningRow) -> Place {
    Place::Running(Running {
        reaction: reaction_name(reactions, &row.reaction),
        script: script(&row.script),
        cell: cell(row.cell),
        next: row.next,
        wait: frames(row.wait),
        grows_back_as: row.remembers.as_deref().map(reacting),
    })
}

fn scroll(row: &ScrollRow) -> Place {
    Place::Scroll(ScrollReaction {
        cell: cell(row.cell),
        changes: changes(&row.scroll),
        touched: if row.touched.unwrap_or(false) {
            Touched::Yes
        } else {
            Touched::No
        },
    })
}

fn places(
    reactions: &Reactions,
    rows: &[PlaceRow],
    held: Option<&Vec<usize>>,
    held_until_run: Option<&Vec<usize>>,
) -> Places {
    let mut places = Places::default();
    let mut put = |number: usize, place: Place| {
        let slot = &mut places.0[number];
        assert_eq!(*slot, Place::Free, "place {number} given twice");
        *slot = place;
    };
    for row in rows {
        match row {
            PlaceRow::Running(row) => put(row.place, running(reactions, row)),
            PlaceRow::Scroll(row) => put(row.place, scroll(row)),
        }
    }
    for &number in held.into_iter().flatten() {
        put(number, Place::OtherPart);
    }
    for &number in held_until_run.into_iter().flatten() {
        put(number, Place::HeldUntilRun);
    }
    places
}

fn screens(rows: Option<&Vec<String>>) -> Grid<ScreenKind> {
    Grid {
        rows: rows
            .into_iter()
            .flatten()
            .map(|row| row.chars().map(screen_kind).collect())
            .collect(),
    }
}

fn before(reactions: &Reactions, vector: &Vector) -> State {
    let side = &vector.before;
    State {
        tiles: tiles(reactions, &side.cells, room_size(vector)),
        places: places(
            reactions,
            &side.places,
            side.held.as_ref(),
            side.held_until_run.as_ref(),
        ),
        screens: screens(side.screens.as_ref()),
    }
}

fn after(reactions: &Reactions, vector: &Vector) -> (State, Report) {
    let side = &vector.after;
    let state = State {
        tiles: tiles(reactions, &side.cells, room_size(vector)),
        places: places(
            reactions,
            &side.places,
            side.held.as_ref(),
            side.held_until_run.as_ref(),
        ),
        screens: screens(side.screens.as_ref()),
    };
    let report = Report {
        answers: side.answers.iter().flatten().map(|a| answer(a)).collect(),
        redrawn: side.redrawn.iter().flatten().map(|&at| cell(at)).collect(),
        sounds: side.sounds.iter().flatten().map(sound).collect(),
    };
    (state, report)
}

// The frame.

fn request(row: &RequestRow) -> Request {
    match row {
        RequestRow::Inside(row) => Request::Inside {
            cell: cell(row.inside),
            point: point(&row.point),
        },
        RequestRow::Touch(row) => Request::Touch(Touch {
            cell: cell(row.touch),
            way: way(&row.way),
        }),
        RequestRow::Hit(row) => Request::Hit {
            cell: cell(row.hit),
            by: hit_by(&row.by),
        },
        RequestRow::Other(_) => Request::OtherPart,
    }
}

fn placement(row: &PlacementRow) -> Placement {
    match row {
        PlacementRow::Scroll(row) => Placement::Scroll {
            cell: cell(row.cell),
            changes: changes(&row.scroll),
        },
        PlacementRow::StandIn(row) => Placement::StandIn {
            cell: cell(row.cell),
            way: stand_in(&row.stand_in),
            reach: row.reach,
        },
        PlacementRow::Other(_) => Placement::OtherPart,
    }
}

fn moving(she: Option<&SheRow>) -> Set<Motion> {
    let mut moving = Set::default();
    if let Some(she) = she {
        moving[Motion::Dashing] = she.dashing.unwrap_or(false);
        moving[Motion::Shinesparking] = she.shinesparking.unwrap_or(false);
        moving[Motion::ScrewAttacking] = she.screw_attacking.unwrap_or(false);
    }
    moving
}

/// Runs one vector's frame through the model's step; a vector without a frame is a frame with no
/// placements and no requests, whose run isn't paused.
fn run_frame(blocks: &Blocks, state: &mut State, frame: Option<&FrameRow>) -> Report {
    let Some(frame) = frame else {
        return blocks.step(state, &[], &[], Set::default(), Run::Runs);
    };
    let placements: Vec<Placement> = frame.placements.iter().flatten().map(placement).collect();
    let requests: Vec<Request> = frame.requests.iter().flatten().map(request).collect();
    let scripts = if frame.paused.unwrap_or(false) {
        Run::Paused
    } else {
        Run::Runs
    };
    blocks.step(
        state,
        &placements,
        &requests,
        moving(frame.she.as_ref()),
        scripts,
    )
}

// The comparison.

fn differ<T: PartialEq + Debug>(field: &str, actual: &T, expected: &T, out: &mut Vec<String>) {
    if actual != expected {
        out.push(format!(
            "  {field}: expected {expected:?}\n  {field}:   actual {actual:?}"
        ));
    }
}

fn differ_grid<T: PartialEq + Debug>(
    field: &str,
    actual: &Grid<T>,
    expected: &Grid<T>,
    out: &mut Vec<String>,
) {
    if actual.rows.len() != expected.rows.len() {
        differ(field, &actual.rows, &expected.rows, out);
        return;
    }
    for (y, (got, want)) in actual.rows.iter().zip(&expected.rows).enumerate() {
        for (x, (got, want)) in got.iter().zip(want).enumerate() {
            differ(&format!("{field} ({x}, {y})"), got, want, out);
        }
    }
}

fn check(blocks: &Blocks, vector: &Vector) -> Vec<String> {
    let mut state = before(&blocks.reactions, vector);
    let report = run_frame(blocks, &mut state, vector.frame.as_ref());
    let (expected, expected_report) = after(&blocks.reactions, vector);
    let mut out = Vec::new();
    differ_grid("cell", &state.tiles, &expected.tiles, &mut out);
    for (number, (got, want)) in state.places.0.iter().zip(&expected.places.0).enumerate() {
        differ(&format!("place {number}"), got, want, &mut out);
    }
    differ_grid("screen", &state.screens, &expected.screens, &mut out);
    differ(
        "answers",
        &report.answers,
        &expected_report.answers,
        &mut out,
    );
    differ(
        "redrawn",
        &report.redrawn,
        &expected_report.redrawn,
        &mut out,
    );
    differ("sounds", &report.sounds, &expected_report.sounds, &mut out);
    out
}

fn known_entry(name: &str) -> bool {
    CATALOGUE
        .iter()
        .any(|behaviour| format!("{:?}", behaviour.id) == name)
}

#[test]
fn every_vector_passes() {
    let file = parse();
    let blocks = Blocks {
        rules: Rules::REFERENCE,
        reactions: reactions(&file),
    };
    let mut failures = Vec::new();
    for vector in &file.vector {
        assert_eq!(
            vector.source, "constructed",
            "{}: unknown source",
            vector.id
        );
        for entry in &vector.entries {
            assert!(known_entry(entry), "{}: unknown entry {entry}", vector.id);
        }
        let differences = check(&blocks, vector);
        if !differences.is_empty() {
            failures.push(format!("{}\n{}", vector.id, differences.join("\n")));
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} vectors fail:\n{}",
        failures.len(),
        file.vector.len(),
        failures.join("\n")
    );
}

#[test]
fn every_entry_checked_by_vectors_has_one() {
    let file = parse();
    for behaviour in CATALOGUE {
        let name = format!("{:?}", behaviour.id);
        let entries = CATALOGUE.iter().filter(|other| other.id == behaviour.id);
        assert_eq!(entries.count(), 1, "{name} is catalogued once");
        if behaviour.checked == Checked::ByVectors {
            let checked = file
                .vector
                .iter()
                .any(|vector| vector.entries.contains(&name));
            assert!(checked, "{name} is checked by vectors but none names it");
        }
    }
}
