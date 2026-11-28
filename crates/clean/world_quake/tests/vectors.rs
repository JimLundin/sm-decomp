//! Run the supplied world-quake vectors against the public model.

use core::num;
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;

use enum_map::{Array, Enum};
use serde::Deserialize;
use world_quake::{
    BehaviourId, CATALOGUE, Check, Effects, Offset, Opportunity, Outcome, QuakeDuration, QuakeKind,
    QuakePhase, QuakeSize, Shake, ShakeOrder, State, Time, drawn_offset, start_or_stop, step,
};

type TestResult = Result<(), Box<dyn Error>>;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Vectors {
    vector: Vec<Vector>,
    admission: Vec<Admission>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
enum Source {
    Recorded,
    Constructed,
}

/// The vector file's State: remaining, phase, and the kind exactly while remaining is over 0.
#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    remaining: u16,
    phase: u16,
    kind: Option<QuakeKind>,
}

impl Record {
    fn state(self) -> Result<State, String> {
        let shake = match (num::NonZeroU16::new(self.remaining), self.kind) {
            (Some(remaining), Some(kind)) => Some(Shake { kind, remaining }),
            (None, None) => None,
            (Some(_), None) | (None, Some(_)) => {
                return Err(format!(
                    "a kind is present exactly while frames are left: {self:?}"
                ));
            }
        };
        Ok(State {
            phase: QuakePhase(self.phase),
            shake,
        })
    }
}

#[derive(Clone, Copy, Deserialize)]
#[serde(deny_unknown_fields)]
struct StartRecord {
    kind: QuakeKind,
    frames: u16,
}

#[derive(Clone, Copy, Deserialize)]
enum OpportunityRecord {
    Absent,
    Unestablished,
}

impl OpportunityRecord {
    const fn opportunity(record: Option<Self>) -> Opportunity {
        match record {
            None => Opportunity::Ordinary,
            Some(Self::Absent) => Opportunity::Absent,
            Some(Self::Unestablished) => Opportunity::Unestablished,
        }
    }
}

#[derive(Clone, Copy, Deserialize)]
enum TimeRecord {
    Held,
}

#[derive(Clone, Copy, Deserialize)]
#[serde(deny_unknown_fields)]
struct EffectsRecord {
    front: Offset,
    back: Offset,
    actors: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Vector {
    id: String,
    source: Source,
    entries: Vec<String>,
    continues: Option<String>,
    before: Record,
    start: Option<StartRecord>,
    stop: Option<bool>,
    opportunity: Option<OpportunityRecord>,
    time: Option<TimeRecord>,
    drawn: Option<Offset>,
    effects: Option<EffectsRecord>,
    after: Record,
}

impl Vector {
    /// The frame's starts and stops, in the order the file states them.
    fn orders(&self) -> Vec<ShakeOrder> {
        let start = self.start.map(|start| ShakeOrder::Start {
            kind: start.kind,
            frames: QuakeDuration(start.frames),
        });
        let stop = (self.stop == Some(true)).then_some(ShakeOrder::Stop);
        start.into_iter().chain(stop).collect()
    }

    const fn time(&self) -> Time {
        match self.time {
            None => Time::Running,
            Some(TimeRecord::Held) => Time::Held,
        }
    }

    fn expected_outcome(&self) -> Outcome {
        self.effects.map_or(Outcome::NoOpportunity, |effects| {
            Outcome::Updated(Effects {
                front: effects.front,
                back: effects.back,
                actors_asked: effects.actors,
            })
        })
    }

    /// Run the vector's frame on `state`: its starts and stops, Drawing's query, the update.
    fn run(&self, state: &mut State) -> TestResult {
        let id = &self.id;
        assert_eq!(*state, self.before.state()?, "{id}: before");
        for order in self.orders() {
            start_or_stop(state, order);
        }
        if let Some(drawn) = self.drawn {
            assert_eq!(drawn_offset(*state, self.time()), drawn, "{id}: drawn");
        }
        let outcome = step(
            state,
            OpportunityRecord::opportunity(self.opportunity),
            self.time(),
        );
        assert_eq!(outcome, self.expected_outcome(), "{id}: effects");
        assert_eq!(*state, self.after.state()?, "{id}: after");
        Ok(())
    }
}

#[derive(Deserialize)]
enum AdmissionOutcome {
    Unsupported,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Admission {
    id: String,
    opportunity: OpportunityRecord,
    before: Record,
    outcome: AdmissionOutcome,
    after: Record,
}

fn restore(state: State) -> Result<State, Box<dyn Error>> {
    Ok(toml::from_str(&toml::to_string(&state)?)?)
}

fn check_admission(admission: &Admission) -> TestResult {
    let id = &admission.id;
    let mut state = restore(admission.before.state()?)?;
    let expected = match admission.outcome {
        AdmissionOutcome::Unsupported => Outcome::Unsupported,
    };
    let opportunity = OpportunityRecord::opportunity(Some(admission.opportunity));
    assert_eq!(
        step(&mut state, opportunity, Time::Running),
        expected,
        "{id}: admission"
    );
    assert_eq!(state, admission.after.state()?, "{id}: after");
    Ok(())
}

/// The State a vector starts from: its own, restored, or the full result of the vector it
/// continues, restored; a continuation also runs on the uninterrupted result.
fn starting_state(
    vector: &Vector,
    completed: &BTreeMap<String, State>,
) -> Result<State, Box<dyn Error>> {
    let Some(previous) = &vector.continues else {
        return restore(vector.before.state()?);
    };
    let prior = completed.get(previous).ok_or_else(|| {
        format!(
            "{}: unknown or not yet completed vector {previous}",
            vector.id
        )
    })?;
    let mut uninterrupted = *prior;
    vector.run(&mut uninterrupted)?;
    restore(*prior)
}

#[test]
fn world_quake_vectors() -> TestResult {
    let vectors: Vectors = toml::from_str(&vector_file::read("world-quake")?)?;
    let catalogue: BTreeMap<_, _> = CATALOGUE
        .iter()
        .map(|row| (row.id.as_str(), row.id))
        .collect();
    let mut sources: BTreeMap<BehaviourId, BTreeSet<Source>> = BTreeMap::new();
    let mut completed = BTreeMap::new();
    for vector in &vectors.vector {
        let mut state = starting_state(vector, &completed)?;
        vector.run(&mut state)?;
        assert!(
            completed.insert(vector.id.clone(), state).is_none(),
            "duplicate id {}",
            vector.id
        );
        assert!(!vector.entries.is_empty(), "{}: missing entries", vector.id);
        for entry in &vector.entries {
            let id = catalogue
                .get(entry.as_str())
                .ok_or_else(|| format!("{}: unknown entry {entry}", vector.id))?;
            sources.entry(*id).or_default().insert(vector.source);
        }
    }
    for admission in &vectors.admission {
        check_admission(admission)?;
    }
    assert_eq!(catalogue.len(), CATALOGUE.len(), "duplicate catalogue id");
    assert_eq!(
        catalogue.len(),
        <BehaviourId as Enum>::Array::<()>::LENGTH,
        "incomplete catalogue"
    );
    for row in CATALOGUE {
        let Check::OracleAndConstructed { .. } = row.checked;
        let both = BTreeSet::from([Source::Recorded, Source::Constructed]);
        assert_eq!(
            sources.get(&row.id),
            Some(&both),
            "{}: evidence",
            row.id.as_str()
        );
    }
    Ok(())
}

#[test]
fn sizes_are_one_to_three_pixels() {
    assert_eq!(QuakeSize::new(0), None);
    assert_eq!(QuakeSize::new(1).map(QuakeSize::pixels), Some(1));
    assert_eq!(QuakeSize::new(3).map(QuakeSize::pixels), Some(3));
    assert_eq!(QuakeSize::new(4), None);
}
