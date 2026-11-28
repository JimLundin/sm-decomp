//! Runs every vector of `scenarios/vectors/screens-flight.toml` through the model, reading the
//! file's format as its header gives it; a key the format doesn't name fails the read.
#![cfg(test)]

use core::num::Wrapping;
use std::num::NonZeroU16;

use enum_map::EnumMap;
use screens_flight::{BehaviourId, MusicQueue, Rules, State, catalogue, step};
use serde::Deserialize;
use types::flight::{
    Angle, Brightness, BrightnessLevel, Drift, Drifter, DriftingSprite, FadeFrames, FlightSprites,
    FlightStage, Keystroke, LabelPlace, LabelProgress, LabelStep, MusicRequest, Plane, RushSpeed,
    Scene, SceneSetup, SceneTurn, SpritePlacement, SpriteWrap, StarRush, TuneDelay, TypedLetter,
    Visibility, Zoom,
};
use types::{Checked, Colour, Fixed, Pair, fixed};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct VectorFile {
    vector: Vec<Vector>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "lowercase")]
enum Source {
    Recorded,
    Constructed,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Vector {
    id: String,
    #[expect(
        dead_code,
        reason = "read so the format is checked; the run is the same for both"
    )]
    source: Source,
    entries: Vec<String>,
    rules: Option<RulesRow>,
    before: StateRow,
    inputs: Option<InputsRow>,
    after: StateRow,
    output: Option<OutputRow>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RulesRow {
    rhythm: FadeFrames,
}

/// A place as the file writes it: `[[x, "xf"], [y, "yf"]]`.
type PlaceRow = [(i16, String); 2];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StateRow {
    stage: String,
    angle: String,
    zoom: Zoom,
    plane: PlaceRow,
    sprites: SpritesRow,
    flash: [u16; 3],
    label: Option<LabelRow>,
    level: BrightnessLevel,
    screen: Visibility,
    count: FadeFrames,
}

/// A drifting sprite as the file writes it: `[x, "xf", y, "yf", drift, wraps]`.
type DrifterRow = (i16, String, i16, String, i32, bool);

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SpritesRow {
    #[serde(rename = "RushingStars")]
    rushing_stars: Option<(i16, String, i16, String, i16)>,
    #[serde(rename = "Station")]
    station: Option<DrifterRow>,
    #[serde(rename = "SmallAsteroids")]
    small_asteroids: Option<DrifterRow>,
    #[serde(rename = "LargeAsteroids")]
    large_asteroids: Option<DrifterRow>,
    #[serde(rename = "Vortex")]
    vortex: Option<DrifterRow>,
    #[serde(rename = "DistantStars")]
    distant_stars: Option<DrifterRow>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LabelRow {
    step: usize,
    hold: u16,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct InputsRow {
    track_waits: Option<bool>,
    scene: Option<SceneRow>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SceneRow {
    ready: Option<ReadyRow>,
    turn: Option<TurnRow>,
    label: Option<Vec<StepRow>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ReadyRow {
    angle: String,
    zoom: Zoom,
    plane: [i16; 2],
    stars: [i16; 2],
    speed: i16,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TurnRow {
    angle: String,
    plane: [i16; 2],
    sprites: TurnSpritesRow,
}

/// A sprite placed at the turn as the file writes it: `[x, y, drift, wraps]`.
type PlacementRow = (i16, i16, i32, bool);

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TurnSpritesRow {
    #[serde(rename = "Station")]
    station: Option<PlacementRow>,
    #[serde(rename = "SmallAsteroids")]
    small_asteroids: Option<PlacementRow>,
    #[serde(rename = "LargeAsteroids")]
    large_asteroids: Option<PlacementRow>,
    #[serde(rename = "Vortex")]
    vortex: Option<PlacementRow>,
    #[serde(rename = "DistantStars")]
    distant_stars: Option<PlacementRow>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StepRow {
    letter: Option<char>,
    column: Option<u16>,
    row: Option<u16>,
    hold: NonZeroU16,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OutputRow {
    transform: Option<[i16; 4]>,
    scroll: Option<[i16; 2]>,
    typed: Option<Vec<TypedRow>>,
    done: Option<bool>,
    music: Option<Vec<MusicRow>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MusicRow {
    request: String,
    delay: u16,
}

fn music_request(row: &MusicRow) -> MusicRequest {
    let delay = TuneDelay(row.delay);
    match row.request.as_str() {
        "LoadCeresFlightTune" => MusicRequest::LoadCeresFlightTune(delay),
        "PlayCeresFlightTune" => MusicRequest::PlayCeresFlightTune(delay),
        other => panic!("unknown music request {other}"),
    }
}

/// The music requests a vector expects, an absent field (or an absent output) expecting none,
/// against those the model reported.
fn music_difference(output: Option<&OutputRow>, got: &[MusicRequest]) -> Option<String> {
    let wanted: Vec<MusicRequest> = output
        .and_then(|output| output.music.as_ref())
        .map_or_else(Vec::new, |rows| rows.iter().map(music_request).collect());
    (wanted != got).then(|| format!("music: expected {wanted:?}, got {got:?}"))
}

#[derive(Deserialize, Clone, Copy)]
#[serde(deny_unknown_fields)]
struct TypedRow {
    letter: char,
    column: u16,
    row: u16,
}

fn angle(hex: &str) -> Angle {
    Angle(u8::from_str_radix(hex, 16).expect("an angle is a hex byte"))
}

fn part(whole: i16, fraction: &str) -> Fixed {
    let fraction = u16::from_str_radix(fraction, 16).expect("a fraction is 4 hex digits");
    fixed(Wrapping(whole.cast_unsigned()), fraction)
}

fn place(row: &PlaceRow) -> Pair<Fixed> {
    let [(x, xf), (y, yf)] = row;
    Pair {
        x: part(*x, xf),
        y: part(*y, yf),
    }
}

fn whole(row: [i16; 2]) -> Pair<i16> {
    Pair {
        x: row[0],
        y: row[1],
    }
}

fn wrap(wraps: bool) -> SpriteWrap {
    if wraps {
        SpriteWrap::Wraps
    } else {
        SpriteWrap::RunsOn
    }
}

fn drifting((x, xf, y, yf, drift, wraps): &DrifterRow) -> DriftingSprite {
    DriftingSprite {
        place: Pair {
            x: part(*x, xf),
            y: part(*y, yf),
        },
        drift: Drift(*drift),
        wrap: wrap(*wraps),
    }
}

fn by_kind<R, T>(
    rows: [(Drifter, Option<&R>); 5],
    each: fn(&R) -> T,
) -> EnumMap<Drifter, Option<T>> {
    let mut map = EnumMap::default();
    for (kind, row) in rows {
        map[kind] = row.map(each);
    }
    map
}

fn sprites(row: &SpritesRow) -> FlightSprites {
    FlightSprites {
        rushing_stars: row
            .rushing_stars
            .as_ref()
            .map(|(x, xf, y, yf, speed)| StarRush {
                place: Pair {
                    x: part(*x, xf),
                    y: part(*y, yf),
                },
                speed: RushSpeed(*speed),
            }),
        drifting: by_kind(
            [
                (Drifter::Station, row.station.as_ref()),
                (Drifter::SmallAsteroids, row.small_asteroids.as_ref()),
                (Drifter::LargeAsteroids, row.large_asteroids.as_ref()),
                (Drifter::Vortex, row.vortex.as_ref()),
                (Drifter::DistantStars, row.distant_stars.as_ref()),
            ],
            drifting,
        ),
    }
}

fn stage(row: &StateRow) -> FlightStage {
    let label = row.label.as_ref().map(|label| LabelProgress {
        step: label.step,
        hold_left: label.hold,
    });
    match (row.stage.as_str(), label) {
        ("Starting", None) => FlightStage::Starting,
        ("WaitingForMusic", None) => FlightStage::WaitingForMusic,
        ("RushingIn", None) => FlightStage::RushingIn,
        ("FlyingAway", None) => FlightStage::FlyingAway,
        ("Label", Some(progress)) => FlightStage::Label(progress),
        ("FadingOut", None) => FlightStage::FadingOut,
        ("Done", None) => FlightStage::Done,
        (other, _) => panic!("unknown stage {other}, or a label outside the label stage"),
    }
}

fn state(row: &StateRow) -> State {
    State {
        stage: stage(row),
        plane: Plane {
            angle: angle(&row.angle),
            zoom: row.zoom,
            position: place(&row.plane),
        },
        sprites: sprites(&row.sprites),
        flash: Colour::from_channels(row.flash),
        brightness: Brightness {
            level: row.level,
            visibility: row.screen,
        },
        fade_count: row.count,
    }
}

fn placement(&(x, y, drift, wraps): &PlacementRow) -> SpritePlacement {
    SpritePlacement {
        place: Pair { x, y },
        drift: Drift(drift),
        wrap: wrap(wraps),
    }
}

fn turn(row: &TurnRow) -> SceneTurn {
    let sprites = &row.sprites;
    SceneTurn {
        angle: angle(&row.angle),
        plane: whole(row.plane),
        sprites: by_kind(
            [
                (Drifter::Station, sprites.station.as_ref()),
                (Drifter::SmallAsteroids, sprites.small_asteroids.as_ref()),
                (Drifter::LargeAsteroids, sprites.large_asteroids.as_ref()),
                (Drifter::Vortex, sprites.vortex.as_ref()),
                (Drifter::DistantStars, sprites.distant_stars.as_ref()),
            ],
            placement,
        ),
    }
}

fn label_step(row: &StepRow) -> LabelStep {
    let keystroke = match (row.letter, row.column, row.row) {
        (Some(letter), Some(column), Some(row)) => Keystroke::Letter(TypedLetter {
            letter,
            place: LabelPlace { column, row },
        }),
        (None, None, None) => Keystroke::Pause,
        _ => panic!("a step's letter comes with its column and row"),
    };
    LabelStep {
        keystroke,
        hold: row.hold,
    }
}

/// A stand-in for the scene's setup where a vector supplies none: no vector that reads it
/// leaves it out.
const STAND_IN_SETUP: SceneSetup = SceneSetup {
    angle: Angle(0),
    zoom: Zoom::NONE,
    plane: Pair { x: 0, y: 0 },
    stars: Pair { x: 0, y: 0 },
    speed: RushSpeed(0),
};

/// The scene a vector gives, its parts it leaves out stood in for: no turn's sprites, and no
/// label's steps.
fn scene(row: Option<&SceneRow>) -> Scene {
    let setup = row
        .and_then(|row| row.ready.as_ref())
        .map_or(STAND_IN_SETUP, |ready| SceneSetup {
            angle: angle(&ready.angle),
            zoom: ready.zoom,
            plane: whole(ready.plane),
            stars: whole(ready.stars),
            speed: RushSpeed(ready.speed),
        });
    let turn = row.and_then(|row| row.turn.as_ref()).map_or_else(
        || SceneTurn {
            angle: Angle(0),
            plane: Pair { x: 0, y: 0 },
            sprites: EnumMap::default(),
        },
        turn,
    );
    let label = row
        .and_then(|row| row.label.as_ref())
        .map_or_else(Vec::new, |steps| steps.iter().map(label_step).collect());
    Scene { setup, turn, label }
}

fn entry(name: &str) -> BehaviourId {
    match name {
        "FlightIsMadeReady" => BehaviourId::FlightIsMadeReady,
        "PictureAppears" => BehaviourId::PictureAppears,
        "GunshipRushesIn" => BehaviourId::GunshipRushesIn,
        "StarsRushPast" => BehaviourId::StarsRushPast,
        "ViewTurnsAway" => BehaviourId::ViewTurnsAway,
        "GunshipFliesAway" => BehaviourId::GunshipFliesAway,
        "FlashFades" => BehaviourId::FlashFades,
        "SpritesDrift" => BehaviourId::SpritesDrift,
        "StationComesIntoView" => BehaviourId::StationComesIntoView,
        "LabelIsTyped" => BehaviourId::LabelIsTyped,
        "PictureDarkens" => BehaviourId::PictureDarkens,
        "FlightEnds" => BehaviourId::FlightEnds,
        "PlaneTransforms" => BehaviourId::PlaneTransforms,
        other => panic!("unknown entry {other}"),
    }
}

fn vectors() -> Vec<Vector> {
    toml::from_str::<VectorFile>(&vector_file::read("screens-flight").unwrap())
        .expect("the vector file reads in its format")
        .vector
}

/// The differences between what a vector expects and what the model gave, each named.
fn run(vector: &Vector) -> Vec<String> {
    let rules = vector
        .rules
        .as_ref()
        .map_or(Rules::REFERENCE, |rules| Rules {
            rhythm: rules.rhythm,
        });
    let inputs = vector.inputs.as_ref();
    let music = if inputs.and_then(|inputs| inputs.track_waits) == Some(true) {
        MusicQueue::TrackWaits
    } else {
        MusicQueue::Clear
    };
    let scene = scene(inputs.and_then(|inputs| inputs.scene.as_ref()));
    let mut actual = state(&vector.before);
    let report = step(&mut actual, &scene, music, rules);
    assert_eq!(screens_flight::transform(&actual.plane), report.transform);
    let expected = state(&vector.after);
    let mut differences = Vec::new();
    if actual != expected {
        differences.push(format!("state: expected {expected:?}, got {actual:?}"));
    }
    differences.extend(music_difference(vector.output.as_ref(), &report.music));
    let Some(output) = vector.output.as_ref() else {
        return differences;
    };
    let transform = report.transform;
    let got_transform = [transform.a, transform.b, transform.c, transform.d];
    if output
        .transform
        .is_some_and(|wanted| wanted != got_transform)
    {
        differences.push(format!(
            "transform: expected {:?}, got {got_transform:?}",
            output.transform
        ));
    }
    if output
        .scroll
        .is_some_and(|wanted| whole(wanted) != report.scroll)
    {
        differences.push(format!(
            "scroll: expected {:?}, got {:?}",
            output.scroll, report.scroll
        ));
    }
    if output.done.is_some_and(|wanted| wanted != report.done) {
        differences.push(format!(
            "done: expected {:?}, got {}",
            output.done, report.done
        ));
    }
    if let Some(typed) = output.typed.as_ref() {
        let wanted: Vec<TypedLetter> = typed
            .iter()
            .map(|row| TypedLetter {
                letter: row.letter,
                place: LabelPlace {
                    column: row.column,
                    row: row.row,
                },
            })
            .collect();
        let got: Vec<TypedLetter> = report.typed.into_iter().collect();
        if wanted != got {
            differences.push(format!("typed: expected {wanted:?}, got {got:?}"));
        }
    }
    differences
}

#[test]
fn every_vector_passes() {
    let vectors = vectors();
    assert!(!vectors.is_empty(), "the vector file holds vectors");
    let failures: Vec<String> = vectors
        .iter()
        .flat_map(|vector| {
            run(vector)
                .into_iter()
                .map(move |difference| format!("{}: {difference}", vector.id))
        })
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn every_entry_checked_by_vectors_has_one() {
    let vectors = vectors();
    for (id, behaviour) in catalogue() {
        let exercised = vectors
            .iter()
            .any(|vector| vector.entries.iter().any(|name| entry(name) == id));
        assert!(
            exercised || behaviour.checked != Checked::ByVectors,
            "{id:?} has no vector"
        );
    }
}

#[test]
fn every_vector_names_known_entries() {
    for vector in vectors() {
        for name in &vector.entries {
            entry(name);
        }
    }
}
