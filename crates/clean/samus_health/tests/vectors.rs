//! Model vectors and complete owner-state restoration tests.

use samus_health::{
    BehaviourId, EnergyAmount, EnergyLoss, ProtectionFrames, Rules, State, TailImpact, Warning,
    WarningSound, WholeEnergy, behaviour,
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
    before_maintenance_whole: Option<WholeEnergy>,
    drawing: Option<Drawing>,
    rules: Option<RuleOverrides>,
    continues: Option<String>,
    evidence_kind: Option<EvidenceKind>,
    unsupported: Option<bool>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RuleOverrides {
    warning_threshold: WholeEnergy,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Drawing {
    energy_whole: WholeEnergy,
    protection: ProtectionFrames,
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
    protection: ProtectionFrames,
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
            protection: value.protection,
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Context {
    time: Time,
    suit: Suit,
    tail_impacts: Option<Vec<u16>>,
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

impl Vector {
    fn rules(&self) -> Rules {
        self.rules.as_ref().map_or(Rules::REFERENCE, |rules| Rules {
            warning_threshold: rules.warning_threshold,
        })
    }
}

fn checkpoint(state: State, continued: &mut State, label: &str) -> TestResult {
    assert_eq!(*continued, state, "{label}: restored path differs");
    *continued = restored(state)?;
    assert_eq!(*continued, state, "{label}: snapshot differs");
    Ok(())
}

fn verify_contact(vector: &Vector, actual: &mut State, fresh: &mut State) -> TestResult {
    let Some(impacts) = &vector.inputs.tail_impacts else {
        assert!(vector.before_maintenance_whole.is_none());
        assert!(vector.drawing.is_none());
        return Ok(());
    };
    let before = *actual;
    if impacts.is_empty() {
        actual.tail_contact(TailImpact::Absent);
        fresh.tail_contact(TailImpact::Absent);
        assert_eq!(*actual, before, "{} absent impact", vector.id);
        checkpoint(*actual, fresh, "absent contact")?;
    }
    for impact in impacts {
        assert_eq!(*impact, 15, "{} unknown tail impact", vector.id);
        let before_contact = *actual;
        actual.tail_contact(TailImpact::EligibleCeresTail);
        fresh.tail_contact(TailImpact::EligibleCeresTail);
        if before_contact.protection != ProtectionFrames::ZERO {
            assert_eq!(*actual, before_contact, "{} blocked contact", vector.id);
        }
        checkpoint(*actual, fresh, "eligible candidate")?;
    }
    assert_eq!(
        Some(actual.energy.whole),
        vector.before_maintenance_whole,
        "{} before Maintenance",
        vector.id
    );
    assert_eq!(
        actual.energy.fraction, before.energy.fraction,
        "{} contact fraction",
        vector.id
    );
    assert_eq!(
        actual.pending_loss, before.pending_loss,
        "{} contact pending loss",
        vector.id
    );
    assert_eq!(
        actual.warning, before.warning,
        "{} contact warning",
        vector.id
    );
    Ok(())
}

fn verify_drawing(vector: &Vector, actual: State, fresh: &mut State) -> TestResult {
    let Some(expected) = vector.drawing else {
        assert!(vector.inputs.tail_impacts.is_none());
        return Ok(());
    };
    assert!(vector.inputs.tail_impacts.is_some());
    // Drawing reads the pair live and owns its history. The later gauge reads
    // the same authority, not that history. Neither read mutates health.
    let drawing = Drawing {
        energy_whole: actual.energy.whole,
        protection: actual.protection,
    };
    assert_eq!(drawing, expected, "{} Drawing", vector.id);
    assert_eq!(
        (fresh.energy, fresh.protection),
        (actual.energy, actual.protection)
    );
    checkpoint(actual, fresh, "after Drawing")?;
    assert_eq!(
        (actual.energy.whole, actual.protection),
        (expected.energy_whole, expected.protection),
        "{} late gauge",
        vector.id
    );
    assert_eq!(fresh.energy.whole, expected.energy_whole);
    checkpoint(actual, fresh, "after gauge")
}

fn verify_opportunity(
    vector: &Vector,
    mut actual: State,
) -> Result<State, Box<dyn std::error::Error>> {
    let expected = State::from(vector.after.ok_or("admitted vector lacks after")?);
    let rules = vector.rules();
    assert_eq!(
        actual,
        State::from(vector.before),
        "{} carried input",
        vector.id
    );
    let mut fresh = restored(actual)?;
    assert_eq!(fresh, actual, "{} restored input", vector.id);
    verify_contact(vector, &mut actual, &mut fresh)?;

    let protection = actual.protection;
    let sound = actual.health_opportunity(&rules);
    assert_eq!(Some(sound), vector.sound, "{} request", vector.id);
    assert_eq!(
        fresh.health_opportunity(&rules),
        sound,
        "{} restored request",
        vector.id
    );
    assert_eq!(
        actual.protection, protection,
        "{} Maintenance does not age protection",
        vector.id
    );
    checkpoint(actual, &mut fresh, "after Maintenance")?;

    verify_drawing(vector, actual, &mut fresh)?;
    if vector.inputs.tail_impacts.is_some() {
        actual.close_active_opportunity();
        fresh.close_active_opportunity();
    }
    assert_eq!(actual, expected, "{} completed State", vector.id);
    checkpoint(actual, &mut fresh, "completed opportunity")?;
    assert_eq!(
        fresh.health_opportunity(&rules),
        WarningSound::None,
        "{} later Maintenance request",
        vector.id
    );
    assert_eq!(fresh, actual, "{} loss consumed once", vector.id);
    Ok(actual)
}

#[test]
fn samus_health_admitted_vectors() -> TestResult {
    let data = vectors()?;
    let admitted: Vec<_> = data.iter().filter(|v| v.evidence_kind.is_none()).collect();
    assert_eq!(admitted.len(), 24);
    let recorded: Vec<_> = admitted
        .iter()
        .filter(|v| v.source == Source::Recorded)
        .map(|v| v.id.as_str())
        .collect();
    assert_eq!(
        recorded,
        ["R-maintenance-current", "R-tail-hit", "R-tail-protected"]
    );
    assert_eq!(
        admitted
            .iter()
            .filter(|v| v.source == Source::Constructed)
            .count(),
        21
    );
    for vector in &admitted {
        assert_eq!(vector.inputs.time, Time::Running);
        assert_eq!(vector.inputs.suit, Suit::Neither);
        assert_eq!(vector.unsupported, None);
        verify_opportunity(vector, State::from(vector.before))?;
        if let Some(previous_id) = &vector.continues {
            let previous = admitted
                .iter()
                .find(|v| &v.id == previous_id)
                .ok_or("unknown continuation")?;
            assert_eq!(previous.source, vector.source);
            let carried = verify_opportunity(previous, State::from(previous.before))?;
            let fresh = restored(carried)?;
            assert_eq!(fresh, carried);
            assert_eq!(
                verify_opportunity(vector, carried)?,
                verify_opportunity(vector, fresh)?
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
        assert!(vector.inputs.tail_impacts.is_none());
        assert!(vector.before_maintenance_whole.is_none());
        assert!(vector.drawing.is_none());
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
        protection: ProtectionFrames::ZERO,
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
        protection: ProtectionFrames::TAIL_GRANT,
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
            protection: ProtectionFrames::TAIL_GRANT,
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
    let tail = behaviour(BehaviourId::TailHit);
    assert_eq!(
        (tail.kind, tail.switch, tail.checked),
        (Kind::Mechanic, Switch::Fixed, Checked::ByVectors)
    );
    let timing = behaviour(BehaviourId::ProtectionTime);
    assert_eq!(
        (timing.kind, timing.switch, timing.checked),
        (Kind::Timing, Switch::Fixed, Checked::ByVectors)
    );
    assert_eq!(Rules::REFERENCE.warning_threshold.get(), 30);
    assert!(WholeEnergy::new(32768).is_none());
    assert!(toml::from_str::<Rules>("warning_threshold = 32768").is_err());
    assert!(toml::from_str::<Rules>("warning_threshold = -1").is_err());
    let state = independent_state(32767, 65535, Warning::On)?;
    let text = toml::to_string(&state)?;
    assert!(toml::from_str::<State>(&text.replace("32767", "32768")).is_err());
    assert!(ProtectionFrames::new(96).is_some());
    assert!(ProtectionFrames::new(97).is_none());
    assert!(toml::from_str::<State>(&text.replace("protection = 0", "protection = 97")).is_err());
    assert!(toml::from_str::<State>(&text.replace("protection = 0", "protection = -1")).is_err());
    assert!(toml::from_str::<State>(&text.replace("protection = 0", "")).is_err());
    Ok(())
}
