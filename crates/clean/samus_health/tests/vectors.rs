//! Model vectors and complete owner-state restoration tests.

use samus_health::{
    BehaviourId, EnergyAmount, EnergyLoss, Rules, State, Warning, WarningSound, WholeEnergy,
    behaviour,
};
use serde::Deserialize;
use types::{Checked, Kind, Switch};

type TestResult = Result<(), Box<dyn std::error::Error>>;

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
    entries: Vec<BehaviourId>,
    before: Snapshot,
    after: Option<Snapshot>,
    sound: Option<WarningSound>,
    inputs: Context,
    rules: Option<Rules>,
    continues: Option<String>,
    evidence_kind: Option<EvidenceKind>,
    unsupported: Option<bool>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
enum Source {
    #[serde(rename = "recorded")]
    Recorded,
    #[serde(rename = "constructed")]
    Constructed,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
enum EvidenceKind {
    IntegrationAdmission,
}

#[derive(Clone, Copy, Deserialize)]
#[serde(deny_unknown_fields)]
struct Snapshot {
    energy_whole: WholeEnergy,
    energy_fraction: u16,
    pending_whole: WholeEnergy,
    pending_fraction: u16,
    warning: Warning,
}

impl From<Snapshot> for State {
    fn from(value: Snapshot) -> Self {
        Self {
            energy: EnergyAmount {
                whole: value.energy_whole,
                fraction: value.energy_fraction,
            },
            pending_loss: EnergyLoss {
                whole: value.pending_whole,
                fraction: value.pending_fraction,
            },
            warning: value.warning,
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Context {
    time: Time,
    suit: Suit,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
enum Time {
    Running,
    Stopped,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
enum Suit {
    #[serde(rename = "NeitherSuit")]
    Neither,
    #[serde(rename = "VariaSuit")]
    Varia,
    #[serde(rename = "GravitySuit")]
    Gravity,
}

fn vectors() -> Result<Vec<Vector>, toml::de::Error> {
    toml::from_str::<Vectors>(include_str!(
        "../../../../scenarios/vectors/samus-health.toml"
    ))
    .map(|data| data.vector)
}

fn restored(state: State) -> Result<State, Box<dyn std::error::Error>> {
    let serialized = toml::to_string(&state)?;
    Ok(toml::from_str(&serialized)?)
}

fn verify_opportunity(vector: &Vector) -> TestResult {
    let before = State::from(vector.before);
    let expected = State::from(vector.after.ok_or("admitted vector lacks after")?);
    let rules = vector.rules.unwrap_or(Rules::REFERENCE);
    let mut actual = before;
    let mut fresh = restored(before)?;
    assert_eq!(fresh, before, "{} fresh before", vector.id);
    let sound = actual.health_opportunity(&rules);
    assert_eq!(actual, expected, "{} completed State", vector.id);
    assert_eq!(Some(sound), vector.sound, "{} request", vector.id);
    assert_eq!(
        fresh.health_opportunity(&rules),
        sound,
        "{} restored request",
        vector.id
    );
    assert_eq!(fresh, actual, "{} restored next State", vector.id);

    let mut fresh_after = restored(actual)?;
    assert_eq!(fresh_after, actual, "{} fresh after", vector.id);
    let repeat = actual.health_opportunity(&rules);
    assert_eq!(repeat, WarningSound::None, "{} repeated request", vector.id);
    assert_eq!(fresh_after.health_opportunity(&rules), repeat);
    assert_eq!(fresh_after, actual);
    assert_eq!(actual, expected, "{} consumed once", vector.id);
    Ok(())
}

#[test]
fn samus_health_admitted_vectors() -> TestResult {
    let data = vectors()?;
    let admitted: Vec<_> = data.iter().filter(|v| v.evidence_kind.is_none()).collect();
    assert_eq!(admitted.len(), 16);
    assert_eq!(
        admitted
            .iter()
            .filter(|v| v.source == Source::Recorded)
            .count(),
        1
    );
    for vector in &admitted {
        assert_eq!(vector.inputs.time, Time::Running);
        assert_eq!(vector.inputs.suit, Suit::Neither);
        assert_eq!(vector.unsupported, None);
        verify_opportunity(vector)?;
        if let Some(previous_id) = &vector.continues {
            let previous = admitted
                .iter()
                .find(|v| &v.id == previous_id)
                .ok_or("unknown continuation")?;
            let mut carried = State::from(previous.before);
            assert_eq!(
                Some(carried.health_opportunity(&previous.rules.unwrap_or(Rules::REFERENCE))),
                previous.sound
            );
            assert_eq!(carried, State::from(vector.before));
            let sound = carried.health_opportunity(&vector.rules.unwrap_or(Rules::REFERENCE));
            assert_eq!(Some(sound), vector.sound);
            assert_eq!(
                carried,
                State::from(vector.after.ok_or("missing continuation after")?)
            );
        }
    }
    for id in BehaviourId::ALL {
        assert!(admitted.iter().any(|v| v.entries.contains(&id)));
        assert_eq!(behaviour(id).id, id);
    }
    let witness = admitted
        .iter()
        .find(|v| v.source == Source::Recorded)
        .ok_or("missing witness")?;
    assert_eq!(witness.id, "R-maintenance-current");
    assert_eq!(
        State::from(witness.before),
        State::from(witness.after.ok_or("missing witness after")?)
    );
    Ok(())
}

#[test]
fn integration_admission_metadata_is_not_model_coverage() -> TestResult {
    let data = vectors()?;
    let neighbors: Vec<_> = data.iter().filter(|v| v.evidence_kind.is_some()).collect();
    assert_eq!(neighbors.len(), 3);
    for vector in neighbors {
        assert_eq!(
            vector.evidence_kind,
            Some(EvidenceKind::IntegrationAdmission)
        );
        assert_eq!(vector.unsupported, Some(true));
        assert_eq!(vector.entries, Vec::<BehaviourId>::new());
        assert!(vector.after.is_none());
        assert!(vector.sound.is_none());
        assert!(vector.rules.is_none());
        assert!(vector.continues.is_none());
        assert_eq!(vector.source, Source::Constructed);
        // Only validate evidence shape. No qualification implementation, operation
        // call, successful no-op claim or behavior credit is made here.
    }
    Ok(())
}

fn independent_state(whole: u16, fraction: u16, warning: Warning) -> Result<State, &'static str> {
    Ok(State {
        energy: EnergyAmount {
            whole: WholeEnergy::try_from(whole)?,
            fraction,
        },
        pending_loss: EnergyLoss::ZERO,
        warning,
    })
}

fn check_independent_loss(mut state: State, pending: u16, fraction: u16) -> TestResult {
    let energy = state.energy;
    let warning = state.warning;
    state.pending_loss = EnergyLoss {
        whole: WholeEnergy::try_from(pending)?,
        fraction,
    };
    assert_eq!(state.energy, energy);
    assert_eq!(state.warning, warning);
    let mut fresh = restored(state)?;
    assert_eq!(fresh, state);
    for threshold in [0, 30, 32767] {
        let rules = Rules {
            warning_threshold: WholeEnergy::try_from(threshold)?,
        };
        let mut original = state;
        fresh = restored(state)?;
        assert_eq!(
            fresh.health_opportunity(&rules),
            original.health_opportunity(&rules)
        );
        assert_eq!(fresh, original);
        assert_eq!(original.pending_loss, EnergyLoss::ZERO);
        assert_eq!(restored(original)?, original);
    }
    Ok(())
}

#[test]
fn independent_quantities_and_fresh_complete_restoration() -> TestResult {
    for whole in [0, 1, 31, 32767] {
        for fraction in [0, 1, 65535] {
            for warning in [Warning::Off, Warning::On] {
                let state = independent_state(whole, fraction, warning)?;
                for (pending, pending_fraction) in [(0, 0), (0, 65535), (1, 1), (32767, 65535)] {
                    check_independent_loss(state, pending, pending_fraction)?;
                }
            }
        }
    }
    Ok(())
}

/// Spec: `PeriodicLoss`, `LowEnergyWarning`. Constructed model regression, not
/// recorded evidence or integration admission. Fractions are in 1/65536 energy.
#[test]
fn excess_fractional_loss_clears_nonzero_starting_fraction() {
    let mut state = State {
        energy: EnergyAmount {
            whole: WholeEnergy::ZERO,
            fraction: 1,
        },
        pending_loss: EnergyLoss {
            whole: WholeEnergy::ZERO,
            fraction: 2,
        },
        warning: Warning::Off,
    };

    // This public operation receives an already qualified Running/NeitherSuit
    // opportunity. Its expected result is independent of another model call.
    let sound = state.health_opportunity(&Rules::REFERENCE);
    assert_eq!(
        state,
        State {
            energy: EnergyAmount {
                whole: WholeEnergy::ZERO,
                fraction: 0,
            },
            pending_loss: EnergyLoss {
                whole: WholeEnergy::ZERO,
                fraction: 0,
            },
            warning: Warning::On,
        }
    );
    assert_eq!(sound, WarningSound::Start);
}

#[test]
fn catalogue_metadata_and_checked_ranges() -> TestResult {
    let loss = behaviour(BehaviourId::PeriodicLoss);
    assert_eq!(
        (loss.kind, loss.switch, loss.checked),
        (Kind::Mechanic, Switch::Fixed, Checked::ByVectors)
    );
    let warning = behaviour(BehaviourId::LowEnergyWarning);
    assert_eq!(
        (warning.kind, warning.switch, warning.checked),
        (Kind::Output, Switch::Parameter, Checked::ByVectors)
    );
    assert_eq!(Rules::REFERENCE.warning_threshold.get(), 30);
    assert!(WholeEnergy::new(32768).is_none());
    assert!(toml::from_str::<Rules>("warning_threshold = 32768").is_err());
    assert!(toml::from_str::<Rules>("warning_threshold = -1").is_err());
    let state = independent_state(32767, 65535, Warning::On)?;
    let text = toml::to_string(&state)?;
    assert!(toml::from_str::<State>(&text.replace("32767", "32768")).is_err());
    Ok(())
}
