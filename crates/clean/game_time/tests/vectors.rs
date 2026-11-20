//! Runs the game-time vectors through the public clock model.

use std::collections::{HashMap, HashSet};
use std::error::Error;

use enum_map::enum_len;
use game_time::{BEHAVIOURS, BehaviourId, Clock, PlayTime, QualifiedOpportunity};
use serde::Deserialize;
use types::{Checked, Kind, Switch};

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
    before: PlayTime,
    after: PlayTime,
    continues: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Source {
    Recorded,
    Constructed,
}

#[test]
fn play_time_vectors() -> Result<(), Box<dyn Error>> {
    let vectors: Vectors =
        toml::from_str(include_str!("../../../../scenarios/vectors/game-time.toml"))?;
    let mut completed = HashMap::new();
    let mut covered = HashSet::new();

    for vector in vectors.vector {
        let id = &vector.id;
        let [entry] = vector.entries.as_slice() else {
            return Err(format!("{id}: expected one behaviour entry").into());
        };
        if entry != "PlayTimeAdvance" {
            return Err(format!("{id}: unknown behaviour {entry}").into());
        }
        let mut clock = start_clock(&vector, &completed)?;
        assert_eq!(clock.time(), vector.before, "{id}: starting time");
        assert_eq!(
            clock.advance(QualifiedOpportunity),
            vector.after,
            "{id}: advance result"
        );
        assert_eq!(clock.time(), vector.after, "{id}: retained time");
        assert!(
            completed.insert(vector.id, clock.time()).is_none(),
            "duplicate vector id"
        );
        covered.insert(BehaviourId::PlayTimeAdvance);
        if vector.source == Source::Recorded {
            assert_eq!(
                vector.before.frames.get().saturating_add(1),
                vector.after.frames.get()
            );
        }
    }

    assert_catalogue(&covered);
    Ok(())
}

fn start_clock(
    vector: &Vector,
    completed: &HashMap<String, PlayTime>,
) -> Result<Clock, Box<dyn Error>> {
    if let Some(previous) = &vector.continues {
        let snapshot = completed
            .get(previous)
            .ok_or_else(|| format!("{}: no completed vector {previous}", vector.id))?;
        let saved = toml::to_string(snapshot)?;
        return Ok(Clock::restore(toml::from_str(&saved)?));
    }

    // Set independent times explicitly, starting from a different time in these vectors.
    let mut clock = Clock::restore(vector.after);
    clock.set(vector.before);
    Ok(clock)
}

fn assert_catalogue(covered: &HashSet<BehaviourId>) {
    assert_eq!(BEHAVIOURS.len(), enum_len::<BehaviourId>());
    let catalogued: HashSet<_> = BEHAVIOURS.iter().map(|entry| entry.id).collect();
    assert_eq!(catalogued.len(), BEHAVIOURS.len());
    assert_eq!(&catalogued, covered);
    for behaviour in BEHAVIOURS {
        assert_eq!(behaviour.kind, Kind::Mechanic);
        assert_eq!(behaviour.switch, Switch::Fixed);
        assert_eq!(behaviour.checked, Checked::ByVectors);
    }
}
