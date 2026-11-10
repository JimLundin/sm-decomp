//! The spec's vectors (`scenarios/vectors/blocks.toml`) run through the model, and the catalogue
//! checked against the vectors. The file's shape is declared once below; every table refuses a
//! key it doesn't name, and every name is read by a table of the names the format allows.

#![cfg(test)]

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Debug;
use std::sync::Arc;

use clean_model_blocks::{
    Answer, Becomes, CATALOGUE, CellKind, Frames, Grid, Inside, Mark, PatternCell, Place,
    Placement, Places, ReactingBody, Reaction, ReactionName, Report, Request, Rules, Run, Running,
    ScreenChange, ScrollReaction, SoundRequest, StandIn, State, Step, Tile, Touched,
};
use clean_types::{
    Cell, Checked, Family, Flip, Heading, HitBy, Look, MetAs, Motion, Pair, Picture, Point,
    ScreenKind, Script, Set, Touch, Way,
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
struct VectorFile {
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
    other_parts: Option<Vec<usize>>,
    held_until_run: Option<Vec<usize>>,
    screens: Option<Vec<String>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct After {
    cells: Vec<CellRow>,
    places: Vec<PlaceRow>,
    other_parts: Option<Vec<usize>>,
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
    paused: Option<True>,
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

/// The vector file, refusing a duplicate vector id.
fn parse() -> VectorFile {
    let file: VectorFile =
        toml::from_str(VECTORS).expect("the vector file reads as the format its header gives");
    let mut ids = BTreeSet::new();
    for vector in &file.vector {
        assert!(ids.insert(&vector.id), "duplicate vector id {}", vector.id);
    }
    file
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
        "Block" => CellKind::Reacting(ReactingBody::Block),
        "ReactingAir" => CellKind::Reacting(ReactingBody::Air),
        "Solid" => CellKind::Solid,
        "StandInAcross" => CellKind::StandIn(StandIn::Across),
        "StandInDown" => CellKind::StandIn(StandIn::Down),
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

fn remembers(name: &str) -> ReactingBody {
    match name {
        "Block" => ReactingBody::Block,
        "ReactingAir" => ReactingBody::Air,
        other => panic!("unknown remembers {other}"),
    }
}

fn inside(name: Option<&str>) -> Inside {
    match name {
        None => Inside::Passes,
        Some("HoldsPlace") => Inside::HoldsPlace,
        Some(other) => panic!("unknown inside {other}"),
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

/// A picture's index in hex; a flip is refused.
fn picture(text: &str) -> Picture {
    let index = u16::from_str_radix(text, HEX).expect("a picture's index is hex, never flipped");
    Picture::new(index).expect("a picture of the tileset")
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
    Look {
        picture: picture(index),
        flip,
    }
}

const fn cell((x, y): (u16, u16)) -> Cell {
    Pair { x, y }
}

/// Frames the spec allows, 1 to 32,767; any other count is refused.
fn frames(count: u16) -> Frames {
    Frames::new(count).expect("frames the spec allows, 1 to 32,767")
}

// The reactions.

/// The room's reactions, by name.
type Reactions = BTreeMap<String, Arc<Reaction>>;

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

fn reactions(file: &VectorFile) -> Reactions {
    file.reaction
        .iter()
        .map(|(name, row)| {
            let mut reaction = Reaction {
                name: ReactionName(name.clone()),
                family: family(&row.family),
                whole_look: row.look.as_deref().map(picture),
                first_wait: frames(row.first_wait),
                scripts: enum_map::EnumMap::default(),
            };
            reaction.scripts[Script::Broken] = steps(row.broken.as_ref());
            reaction.scripts[Script::Revealed] = steps(row.revealed.as_ref());
            reaction.scripts[Script::Touched] = steps(row.touched.as_ref());
            (name.clone(), Arc::new(reaction))
        })
        .collect()
}

// The state.

/// The reaction a mark or a place names: one of the room's, as the room's loading guarantees.
fn named(reactions: &Reactions, name: &str) -> Arc<Reaction> {
    Arc::clone(
        reactions
            .get(name)
            .unwrap_or_else(|| panic!("unknown reaction {name}")),
    )
}

fn mark(reactions: &Reactions, row: &CellRow) -> Mark {
    let given = (
        row.reaction.as_deref(),
        row.mark.as_deref(),
        row.met_as.as_deref(),
        row.reach,
    );
    match given {
        (Some(name), None, None, None) => Mark::Reaction(named(reactions, name)),
        (None, Some("Scroll"), None, None) => Mark::Scroll,
        (None, Some("Nothing"), Some(family), None) => Mark::Nothing(met_as(family)),
        (None, Some("Inert"), None, None) => Mark::Inert,
        (None, Some("Other"), None, None) => Mark::Other(None),
        (None, Some("Other"), Some("Crumble"), None) => Mark::Other(Some(MetAs::Crumble)),
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
        inside: inside(row.inside.as_deref()),
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
    let mut grid = Grid::new(vec![vec![plain(); size.x]; size.y]).expect("rows of one length");
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
    let running = Running::new(
        named(reactions, &row.reaction),
        script(&row.script),
        row.remembers.as_deref().map(remembers),
        cell(row.cell),
        row.next,
        frames(row.wait),
    );
    Place::Running(running.unwrap_or_else(|| {
        panic!(
            "place {}: remembers is given exactly when the script has a Regrow step",
            row.place
        )
    }))
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
    other_parts: Option<&Vec<usize>>,
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
    for &number in other_parts.into_iter().flatten() {
        put(number, Place::OtherPart);
    }
    for &number in held_until_run.into_iter().flatten() {
        put(number, Place::HeldUntilRun);
    }
    places
}

/// The room's screens, refusing rows of unequal length.
fn screens(rows: Option<&Vec<String>>) -> Grid<ScreenKind> {
    let rows = rows
        .into_iter()
        .flatten()
        .map(|row| row.chars().map(screen_kind).collect())
        .collect();
    Grid::new(rows).expect("screens rows of equal length")
}

fn before(reactions: &Reactions, vector: &Vector) -> State {
    let side = &vector.before;
    State {
        tiles: tiles(reactions, &side.cells, room_size(vector)),
        places: places(
            reactions,
            &side.places,
            side.other_parts.as_ref(),
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
            side.other_parts.as_ref(),
            side.held_until_run.as_ref(),
        ),
        screens: screens(side.screens.as_ref()),
    };
    let report = Report {
        answers: side.answers.iter().flatten().map(|a| answer(a)).collect(),
        written: side.redrawn.iter().flatten().map(|&at| cell(at)).collect(),
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

/// Runs one vector's frame through the model: the room's start first if it has placements, then
/// the frame's step. A vector without a frame is a frame with no requests, whose run isn't
/// paused.
fn run_frame(rules: &Rules, state: &mut State, frame: Option<&FrameRow>) -> Report {
    let Some(frame) = frame else {
        return state
            .step(rules, &[], Set::default(), Run::Runs)
            .expect("no request");
    };
    let placements: Vec<Placement> = frame.placements.iter().flatten().map(placement).collect();
    state
        .start_room(&placements)
        .expect("placements in the room");
    let requests: Vec<Request> = frame.requests.iter().flatten().map(request).collect();
    let run = if frame.paused.is_some() {
        Run::Paused
    } else {
        Run::Runs
    };
    state
        .step(rules, &requests, moving(frame.she.as_ref()), run)
        .expect("requests in the room")
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
    if actual.rows().len() != expected.rows().len() {
        differ(field, &actual.rows(), &expected.rows(), out);
        return;
    }
    for (y, (got, want)) in actual.rows().iter().zip(expected.rows()).enumerate() {
        for (x, (got, want)) in got.iter().zip(want).enumerate() {
            differ(&format!("{field} ({x}, {y})"), got, want, out);
        }
    }
}

fn check(rules: &Rules, reactions: &Reactions, vector: &Vector) -> Vec<String> {
    let mut state = before(reactions, vector);
    let report = run_frame(rules, &mut state, vector.frame.as_ref());
    let (expected, expected_report) = after(reactions, vector);
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
        &report.written,
        &expected_report.written,
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
    let rules = Rules::reference();
    let reactions = reactions(&file);
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
        let differences = check(&rules, &reactions, vector);
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
