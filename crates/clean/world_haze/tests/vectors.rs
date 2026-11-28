//! Recorded and constructed Ceres haze vectors through the public interface.

use std::{collections::BTreeMap, error::Error};

use enum_map::EnumMap;
use serde::Deserialize;
use types::Colour;
use world_haze::{
    BandIntent, BehaviourId, DisplayIntent, HazeBands, HazeColour, HazeFade, HazeLevel, HazePeak,
    HazeState, HazeStep, Kind, Switch, ThirdBackground, catalogue, ordinary_opportunity,
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Vectors {
    vector: Vec<Vector>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Vector {
    id: String,
    source: Source,
    entries: Vec<Entry>,
    before: Snapshot,
    after: Snapshot,
    output: Output,
    continues: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Source {
    Recorded,
    Constructed,
}

#[derive(Deserialize)]
enum Entry {
    #[serde(rename = "HazeBrightens")]
    Brightens,
    #[serde(rename = "HazeSettles")]
    Settles,
    #[serde(rename = "HazeHolds")]
    Holds,
}

impl From<Entry> for BehaviourId {
    fn from(entry: Entry) -> Self {
        match entry {
            Entry::Brightens => Self::HazeBrightens,
            Entry::Settles => Self::HazeSettles,
            Entry::Holds => Self::HazeHolds,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum FadeName {
    Brightening,
    Steady,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Snapshot {
    colour: HazeColour,
    fade: FadeName,
    bands: HazeBands,
    peak: Option<HazePeak>,
}

impl TryFrom<Snapshot> for HazeState {
    type Error = &'static str;

    fn try_from(snapshot: Snapshot) -> Result<Self, Self::Error> {
        let fade = match (snapshot.fade, snapshot.peak) {
            (FadeName::Brightening, Some(peak)) => HazeFade::Brightening(peak),
            (FadeName::Steady, None) => HazeFade::Steady,
            (FadeName::Brightening, None) => return Err("brightening requires a peak"),
            (FadeName::Steady, Some(_)) => return Err("steady must not carry a peak"),
        };
        Ok(Self {
            colour: snapshot.colour,
            fade,
            bands: snapshot.bands,
        })
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum BandsEffect {
    Replace,
    Retain,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum BackdropName {
    Black,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Output {
    bands: BandsEffect,
    backdrop: BackdropName,
    excludes_third_background: bool,
}

impl Output {
    fn check(&self, actual: DisplayIntent, id: &str) {
        let bands = match self.bands {
            BandsEffect::Replace => BandIntent::Prepare,
            BandsEffect::Retain => BandIntent::Retain,
        };
        let backdrop = match self.backdrop {
            BackdropName::Black => Colour::BLACK,
        };
        assert_eq!(actual.bands, bands, "{id}: output.bands");
        assert_eq!(actual.backdrop, backdrop, "{id}: output.backdrop");
        assert_eq!(
            actual.third_background == ThirdBackground::Excluded,
            self.excludes_third_background,
            "{id}: output.excludes_third_background"
        );
    }
}

fn restored(state: HazeState) -> Result<HazeState, Box<dyn Error>> {
    Ok(toml::from_str(&toml::to_string(&state)?)?)
}

fn run_vector(
    vector: Vector,
    completed: &mut BTreeMap<String, HazeState>,
) -> Result<BehaviourId, Box<dyn Error>> {
    let Vector {
        id,
        source,
        entries,
        before,
        after,
        output,
        continues,
    } = vector;
    let before = HazeState::try_from(before)?;
    let expected = HazeState::try_from(after)?;
    let step = ordinary_opportunity(before);
    assert_eq!(step.state, expected, "{id} ({source:?}): after");
    output.check(step.display, &id);
    let entries: Vec<_> = entries.into_iter().map(BehaviourId::from).collect();
    assert_eq!(entries, [step.behaviour], "{id}: entries");
    check_restore(before, step, &id)?;

    if let Some(previous_id) = continues {
        let previous = *completed
            .get(&previous_id)
            .ok_or_else(|| format!("{id}: continuation from missing result {previous_id}"))?;
        assert_eq!(previous, before, "{id}: carried state from {previous_id}");
        assert_eq!(ordinary_opportunity(previous), step, "{id}: continuation");
        check_restore(previous, step, &id)?;
    }
    assert_eq!(restored(step.state)?, step.state, "{id}: save result");
    assert!(
        completed.insert(id.clone(), step.state).is_none(),
        "duplicate vector {id}"
    );
    Ok(step.behaviour)
}

fn check_restore(before: HazeState, step: HazeStep, id: &str) -> Result<(), Box<dyn Error>> {
    let fresh = restored(before)?;
    assert_eq!(fresh, before, "{id}: restore state");
    assert_eq!(
        ordinary_opportunity(fresh),
        step,
        "{id}: restored opportunity"
    );
    Ok(())
}

#[test]
fn world_haze_vectors() -> Result<(), Box<dyn Error>> {
    let vectors: Vectors = toml::from_str(&vector_file::read("world-haze")?)?;
    let mut completed = BTreeMap::new();
    let mut exercised = EnumMap::default();
    for vector in vectors.vector {
        exercised[run_vector(vector, &mut completed)?] = true;
    }
    for (id, entry) in catalogue() {
        assert_eq!(entry.id, id);
        assert_eq!(entry.kind, Kind::Mechanic);
        assert_eq!(entry.switch, Switch::Fixed);
        assert!(exercised[id], "no vector checks {id:?}");
    }
    Ok(())
}

#[test]
fn peak_and_band_ranges() {
    assert!(HazePeak::new(0).is_some());
    assert!(HazePeak::new(16).is_some());
    assert!(HazePeak::new(17).is_none());
    assert!(HazeLevel::new(0).is_some());
    assert!(HazeLevel::new(15).is_some());
    assert!(HazeLevel::new(16).is_none());
}
