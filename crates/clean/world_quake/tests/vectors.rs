//! Run the supplied world-quake vectors against the public model.

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;

use enum_map::{Array, Enum};
use serde::Deserialize;
use types::{Pair, ZERO};
use world_quake::{
    Behaviour, BehaviourId, CATALOGUE, Check, Opportunity, Outcome, QuakeDuration, QuakePhase,
    QuietEffects, State, UnsupportedReason, displacement, step,
};

const VECTORS: &str = include_str!("../../../../scenarios/vectors/world-quake.toml");
const QUIET_DISPLACEMENT: Pair<types::Fixed> = Pair { x: ZERO, y: ZERO };
const QUIET_OUTCOME: Outcome = Outcome::Quiet(QuietEffects {
    displacement: QUIET_DISPLACEMENT,
    actor_shake_requested: false,
});

type TestResult = Result<(), Box<dyn Error>>;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Vectors {
    vector: Vec<Vector>,
    admission: Vec<Admission>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum Source {
    Recorded,
    Constructed,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Vector {
    id: String,
    source: Source,
    entries: Vec<String>,
    before: State,
    after: State,
    continues: Option<String>,
}

#[derive(Deserialize)]
enum AdmissionOutcome {
    Unsupported,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Admission {
    id: String,
    before: State,
    outcome: AdmissionOutcome,
    after: State,
}

fn restore(state: State) -> Result<State, Box<dyn Error>> {
    Ok(toml::from_str(&toml::to_string(&state)?)?)
}

fn check_quiet_step(state: &mut State, vector: &Vector) {
    let id = &vector.id;
    assert_eq!(*state, vector.before, "{id}: before");
    assert!(!state.is_active(), "{id}: quiet before update");
    assert_eq!(
        displacement(state),
        Ok(QUIET_DISPLACEMENT),
        "{id}: before drawing"
    );
    assert_eq!(
        step(state, Opportunity::Ordinary),
        QUIET_OUTCOME,
        "{id}: effects"
    );
    assert_eq!(*state, vector.after, "{id}: after");
    assert_eq!(
        displacement(state),
        Ok(QUIET_DISPLACEMENT),
        "{id}: after drawing"
    );
}

fn check_entries(
    vector: &Vector,
    catalogue: &BTreeMap<&str, &Behaviour>,
    exercised: &mut BTreeSet<BehaviourId>,
) -> TestResult {
    assert!(!vector.entries.is_empty(), "{}: missing entries", vector.id);
    for entry in &vector.entries {
        let behaviour = catalogue
            .get(entry.as_str())
            .ok_or_else(|| format!("{}: unknown entry {entry}", vector.id))?;
        let Check::OracleWithConstructedStartsAndWrap { recorded } = behaviour.checked;
        let expected_source = if vector.id == recorded {
            Source::Recorded
        } else {
            Source::Constructed
        };
        assert_eq!(vector.source, expected_source, "{}: source", vector.id);
        exercised.insert(behaviour.id);
    }
    Ok(())
}

fn check_admission(admission: &Admission) -> TestResult {
    let id = &admission.id;
    let mut state = restore(admission.before)?;
    assert_eq!(state, admission.before, "{id}: restored state");
    assert!(state.is_active(), "{id}: active before update");
    assert_eq!(
        displacement(&state),
        Err(UnsupportedReason::ActiveQuake),
        "{id}: drawing"
    );
    let expected = match admission.outcome {
        AdmissionOutcome::Unsupported => Outcome::Unsupported(UnsupportedReason::ActiveQuake),
    };
    assert_eq!(
        step(&mut state, Opportunity::Ordinary),
        expected,
        "{id}: admission"
    );
    assert_eq!(state, admission.after, "{id}: after");
    Ok(())
}

#[test]
fn world_quake_vectors() -> TestResult {
    let vectors: Vectors = toml::from_str(VECTORS)?;
    let catalogue: BTreeMap<_, _> = CATALOGUE.iter().map(|row| (row.id.as_str(), row)).collect();
    let mut exercised = BTreeSet::new();
    let mut completed = BTreeMap::new();
    for vector in &vectors.vector {
        let mut state = vector.before;
        if let Some(previous) = &vector.continues {
            let prior = completed.get(previous).ok_or_else(|| {
                format!(
                    "{}: unknown or not yet completed vector {previous}",
                    vector.id
                )
            })?;
            state = restore(*prior)?;
            let mut uninterrupted = *prior;
            check_quiet_step(&mut uninterrupted, vector);
        }
        check_quiet_step(&mut state, vector);
        assert!(
            completed.insert(vector.id.clone(), state).is_none(),
            "duplicate id {}",
            vector.id
        );
        check_entries(vector, &catalogue, &mut exercised)?;
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
    assert_eq!(exercised, CATALOGUE.iter().map(|row| row.id).collect());
    for row in CATALOGUE {
        let Check::OracleWithConstructedStartsAndWrap { recorded } = row.checked;
        assert!(
            completed.contains_key(recorded),
            "missing oracle {recorded}"
        );
    }
    Ok(())
}

#[test]
fn scheduler_boundary() {
    const START: State = State {
        phase: QuakePhase(u16::MAX),
        remaining: QuakeDuration(0),
    };
    let mut state = START;
    assert_eq!(
        step(&mut state, Opportunity::Absent),
        Outcome::NoOpportunity
    );
    assert_eq!(state, START);
    assert_eq!(
        step(&mut state, Opportunity::Unestablished),
        Outcome::Unsupported(UnsupportedReason::UnestablishedOpportunity),
    );
    assert_eq!(state, START);
    assert_eq!(displacement(&state), Ok(QUIET_DISPLACEMENT));
}
