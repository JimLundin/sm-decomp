//! Exercise the supplied recorded and constructed draws through the public model.

use std::collections::{BTreeMap, BTreeSet};

use enum_map::EnumMap;
use game_random::{
    BehaviourId, CATALOGUE, Check, Kind, RandomNumber, RandomSequence, Switch, advance_random,
};
use serde::Deserialize;

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
    entries: Vec<String>,
    before: RandomSequence,
    after: RandomSequence,
    drawn: RandomNumber,
    continues: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum Source {
    Recorded,
    Constructed,
}

fn vectors() -> Result<Vec<Vector>, toml::de::Error> {
    let vectors: Vectors = toml::from_str(include_str!(
        "../../../../scenarios/vectors/game-random.toml"
    ))?;
    Ok(vectors.vector)
}

#[test]
fn supplied_draws() -> Result<(), Box<dyn std::error::Error>> {
    let mut completed = BTreeMap::<String, RandomSequence>::new();
    for vector in vectors()? {
        let initial = match &vector.continues {
            Some(previous) => {
                let saved = *completed.get(previous).ok_or_else(|| {
                    format!("{}: continuation {previous} has not completed", vector.id)
                })?;
                // Draw both on the carried state and after a save and restore.
                let mut carried = saved;
                assert_draw(&vector, &mut carried);
                saved
            }
            None => vector.before,
        };
        let mut sequence = toml::from_str(&toml::to_string(&initial)?)?;
        assert_draw(&vector, &mut sequence);
        assert!(
            completed.insert(vector.id, sequence).is_none(),
            "duplicate vector ID"
        );
    }
    Ok(())
}

fn assert_draw(vector: &Vector, sequence: &mut RandomSequence) {
    let id = &vector.id;
    assert_eq!(sequence.number, vector.before.number, "{id}: before.number");
    let drawn = advance_random(sequence);
    assert_eq!(sequence.number, vector.after.number, "{id}: after.number");
    assert_eq!(drawn, vector.drawn, "{id}: drawn");
}

const fn entry_name(id: BehaviourId) -> &'static str {
    match id {
        BehaviourId::AdvanceRandom => "AdvanceRandom",
    }
}

#[test]
fn catalogue_matches_vector_coverage() -> Result<(), toml::de::Error> {
    let vectors = vectors()?;
    let mut catalogued = EnumMap::<BehaviourId, bool>::default();
    let mut entries = BTreeSet::new();
    for behaviour in CATALOGUE {
        let name = entry_name(behaviour.id);
        assert!(
            !catalogued[behaviour.id],
            "duplicate catalogue entry: {name}"
        );
        catalogued[behaviour.id] = true;
        entries.insert(name);
        assert_eq!(behaviour.kind, Kind::Mechanic);
        assert_eq!(behaviour.switch, Switch::Fixed);
        let Check::Oracle { recorded } = behaviour.checked;
        let oracle_ids: Vec<_> = vectors
            .iter()
            .filter(|vector| {
                vector.source == Source::Recorded
                    && vector.entries.iter().any(|entry| entry == name)
            })
            .map(|vector| vector.id.as_str())
            .collect();
        assert!(!oracle_ids.is_empty(), "{name} has no recorded vectors");
        assert_eq!(oracle_ids, recorded, "{name}: recorded oracle vectors");
    }
    for (id, present) in catalogued {
        assert!(present, "missing catalogue entry: {id:?}");
    }
    for vector in &vectors {
        assert!(!vector.entries.is_empty(), "{} has no entries", vector.id);
        for entry in &vector.entries {
            assert!(
                entries.contains(entry.as_str()),
                "{}: unknown entry {entry}",
                vector.id
            );
        }
    }
    Ok(())
}
