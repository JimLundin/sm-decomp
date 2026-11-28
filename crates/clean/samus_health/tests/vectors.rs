//! Every vector played through the whole-frame step, and save-and-restore checks.

use samus_health::{
    BehaviourId, EnergyAmount, EnergyLoss, Impact, PlayedFrame, ProtectionFrames, Rules, State,
    Warning, WarningSound, WholeEnergy, behaviour,
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
    protection_drawn: Option<ProtectionFrames>,
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
    /// The worth of the frame's eligible hit, as its hitter gives it; omitted, no hit.
    impact_worth: Option<WholeEnergy>,
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

fn vectors() -> Result<Vec<Vector>, Box<dyn std::error::Error>> {
    Ok(toml::from_str::<Vectors>(&vector_file::read("samus-health")?)?.vector)
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

impl Vector {
    /// The frame's impact: an eligible hit worth `impact_worth`, or Absent where it is omitted.
    fn impact(&self) -> Impact {
        self.inputs
            .impact_worth
            .map_or(Impact::Absent, |worth| Impact::EligibleHit { worth })
    }

    fn play(&self, start: State) -> PlayedFrame {
        start.play_frame(self.impact(), &self.rules())
    }
}

/// Plays the vector's frame from `start` (which must be its before) and from a saved and
/// restored copy of it, checks both against the vector, and gives the State it leaves.
fn verify_frame(vector: &Vector, start: State) -> Result<State, Box<dyn std::error::Error>> {
    let id = &vector.id;
    assert_eq!(start, State::from(vector.before), "{id} before");
    let restored_start = restored(start)?;
    assert_eq!(restored_start, start, "{id} restored before");

    let played = vector.play(start);
    assert_eq!(vector.play(restored_start), played, "{id} restored frame");
    let expected = State::from(vector.after.ok_or("admitted vector lacks after")?);
    assert_eq!(played.state, expected, "{id} after");
    assert_eq!(Some(played.warning_sound), vector.sound, "{id} sound");
    if let Some(drawn) = vector.protection_drawn {
        assert_eq!(played.protection_drawn, drawn, "{id} protection drawn");
    }
    assert_eq!(restored(played.state)?, played.state, "{id} restored after");

    // The loss is consumed once: a following frame without a hit neither spends it again
    // nor requests a sound.
    let next = played.state.play_frame(Impact::Absent, &vector.rules());
    assert_eq!(
        (
            next.state.energy,
            next.state.pending_loss,
            next.warning_sound
        ),
        (played.state.energy, EnergyLoss::ZERO, WarningSound::None),
        "{id} following frame"
    );
    Ok(played.state)
}

#[test]
fn samus_health_admitted_vectors() -> TestResult {
    let data = vectors()?;
    let admitted: Vec<_> = data.iter().filter(|v| v.evidence_kind.is_none()).collect();
    // Every vector of the file is either played here or is integration admission evidence.
    assert_eq!(admitted.len() + 3, data.len());
    assert_eq!(admitted.len(), 29);
    let recorded: Vec<_> = admitted
        .iter()
        .filter(|v| v.source == Source::Recorded)
        .map(|v| v.id.as_str())
        .collect();
    assert_eq!(
        recorded,
        [
            "R-maintenance-current",
            "R-tail-hit",
            "R-tail-protected",
            "R-steam-hit"
        ]
    );
    assert_eq!(
        admitted
            .iter()
            .filter(|v| v.source == Source::Constructed)
            .count(),
        25
    );
    for vector in &admitted {
        assert_eq!(vector.inputs.time, Time::Running);
        assert_eq!(vector.inputs.suit, Suit::Neither);
        assert_eq!(vector.unsupported, None);
        verify_frame(vector, State::from(vector.before))?;
        if let Some(previous_id) = &vector.continues {
            // The chain: the previous frame's State carries into this one, as it is and
            // through a save and restore.
            let previous = admitted
                .iter()
                .find(|v| &v.id == previous_id)
                .ok_or("unknown continuation")?;
            assert_eq!(previous.source, vector.source);
            let carried = verify_frame(previous, State::from(previous.before))?;
            let fresh = restored(carried)?;
            assert_eq!(fresh, carried);
            assert_eq!(verify_frame(vector, carried)?, verify_frame(vector, fresh)?);
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
        assert!(vector.inputs.impact_worth.is_none());
        assert!(vector.protection_drawn.is_none());
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
        fresh = restored(state)?;
        let played = state.play_frame(Impact::Absent, &rules);
        assert_eq!(fresh.play_frame(Impact::Absent, &rules), played);
        assert_eq!(played.state.pending_loss, EnergyLoss::ZERO);
        assert_eq!(restored(played.state)?, played.state);
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
fn excess_fractional_loss_clears_nonzero_starting_fraction() -> TestResult {
    let state = State {
        energy: EnergyAmount {
            whole: WholeEnergy::ZERO,
            fraction: 1,
        },
        pending_loss: EnergyLoss {
            whole: WholeEnergy::ZERO,
            fraction: 2,
        },
        warning: Warning::Off,
        protection: ProtectionFrames::HIT_GRANT,
    };

    // An admitted Running/NeitherSuit frame without an impact. Its expected result is
    // independent of another model call.
    let played = state.play_frame(Impact::Absent, &Rules::REFERENCE);
    assert_eq!(
        played.state,
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
            protection: ProtectionFrames::new(95).ok_or("95 frames is in range")?,
        }
    );
    assert_eq!(played.warning_sound, WarningSound::Start);
    assert_eq!(played.protection_drawn, ProtectionFrames::HIT_GRANT);
    Ok(())
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
