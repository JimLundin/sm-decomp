//! Additive complete readiness histories and fresh-restoration checks.
use super::{behaviour, read_file, state, suspended_vectors::make_work};
use samus_appearance::{
    CannonDelay, CannonObservation, ControlEligibility, ItemControlAnswer, Look, SelectedItem,
    SuspendedAppearance,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CannonHistory {
    id: String,
    source: String,
    pub(super) entries: Vec<String>,
    initial_delay: u8,
    restoration: String,
    steps: Vec<HistoryStep>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum Operation {
    Controls,
    ChooseControls,
    CompleteControls,
    Initialize,
    Draw,
    Skip,
    OutsideSelection,
    Unrelated,
    SaveRestore,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
enum UnrelatedEvent {
    PoseChange,
    ClipChange,
    PassageWithoutInitialization,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct HistoryStep {
    kind: Operation,
    item_before: Option<SelectedItem>,
    item_after: Option<SelectedItem>,
    eligible: Option<bool>,
    one_shot_before: Option<bool>,
    one_shot_after: Option<bool>,
    switched: Option<bool>,
    world_paused: Option<bool>,
    event: Option<UnrelatedEvent>,
    expected_delay: u8,
    expected_may_follow: bool,
}

impl HistoryStep {
    fn answer(&self) -> ItemControlAnswer {
        ItemControlAnswer {
            before: self.item_before.expect("actual starting identity"),
            after: self.item_after.expect("actual chosen identity"),
            eligibility: if self.eligible.expect("actual eligibility") {
                ControlEligibility::Eligible
            } else {
                ControlEligibility::Ineligible
            },
        }
    }

    /// Known fields cannot silently acquire meaning at the wrong opportunity.
    fn validate(&self) {
        let controls = matches!(self.kind, Operation::Controls | Operation::ChooseControls);
        let identities = controls || matches!(self.kind, Operation::OutsideSelection);
        assert_eq!(self.item_before.is_some(), identities);
        assert_eq!(self.item_after.is_some(), identities);
        assert_eq!(self.eligible.is_some(), controls);
        assert_eq!(
            self.event.is_some(),
            matches!(self.kind, Operation::Unrelated)
        );
        if !controls {
            assert!(self.one_shot_before.is_none());
            assert!(self.one_shot_after.is_none());
            assert!(self.switched.is_none());
            assert!(self.world_paused.is_none());
        }
        assert!(CannonDelay::new(self.expected_delay).is_some());
    }
}

/// Complete owner plus actual-control, query, provider and ordered output progress.
/// External delivery is caller-owned, and is not model completion.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct HistoryRun {
    look: Look,
    observation: Option<CannonObservation>,
    next_step: usize,
    provider: Vec<HistoryStep>,
    reads: Vec<bool>,
    external_delivered: usize,
    appearance: SuspendedAppearance,
}

fn fresh(run: HistoryRun) -> HistoryRun {
    let text = toml::to_string(&run).expect("complete readiness and work serialize");
    drop(run);
    let restored = toml::from_str(&text).expect("fresh complete owners deserialize");
    assert_eq!(
        toml::to_string(&restored).expect("complete restored owner"),
        text
    );
    restored
}

fn checkpoint(run: HistoryRun, restore: bool) -> HistoryRun {
    if restore { fresh(run) } else { run }
}

fn choose(mut run: HistoryRun, step: &HistoryStep, restore: bool) -> HistoryRun {
    assert!(!matches!(
        run.observation,
        Some(CannonObservation::Chosen(_))
    ));
    run.observation = Some(CannonObservation::Unmade);
    run = checkpoint(run, restore);
    let before = run.look;
    let answer = step.answer();
    assert!(run.observation.as_mut().expect("unmade").choose(answer));
    assert_eq!(run.look, before, "a chosen answer must not age readiness");
    run = checkpoint(run, restore);
    let different = ItemControlAnswer {
        before: SelectedItem::Grapple,
        after: SelectedItem::None,
        eligibility: ControlEligibility::Ineligible,
    };
    assert!(!run.observation.as_mut().expect("chosen").choose(different));
    assert_eq!(run.observation, Some(CannonObservation::Chosen(answer)));
    run
}

fn complete(mut run: HistoryRun, restore: bool) -> HistoryRun {
    let before = run.look.animation;
    assert!(
        run.observation
            .as_mut()
            .expect("chosen answer")
            .complete(&mut run.look)
    );
    assert_eq!(run.look.animation, before);
    run = checkpoint(run, restore);
    let after = run.look;
    assert!(
        !run.observation
            .as_mut()
            .expect("completed")
            .complete(&mut run.look)
    );
    assert_eq!(run.look, after, "completed effects must not run twice");
    assert_eq!(
        run.external_delivered, 0,
        "model completion is not delivery"
    );
    run
}

fn perform(mut run: HistoryRun, step: &HistoryStep, restore: bool) -> HistoryRun {
    match step.kind {
        Operation::Controls => {
            run = choose(run, step, restore);
            let mut ordinary = run.look.cannon;
            ordinary.controls_completed(step.answer());
            run = complete(run, restore);
            assert_eq!(
                run.look.cannon, ordinary,
                "ordinary and retained rules agree"
            );
        }
        Operation::ChooseControls => run = choose(run, step, restore),
        Operation::CompleteControls => run = complete(run, restore),
        Operation::Initialize => run.look.cannon.initialize(),
        Operation::Draw => run.reads.push(run.look.cannon.may_follow()),
        Operation::SaveRestore => run = fresh(run),
        Operation::Skip | Operation::OutsideSelection | Operation::Unrelated => {}
    }
    run.next_step = run.next_step.saturating_add(1);
    assert_eq!(run.look.cannon.remaining(), step.expected_delay);
    assert_eq!(run.look.cannon.may_follow(), step.expected_may_follow);
    checkpoint(run, restore)
}

fn run_history(v: &CannonHistory, restore: bool) -> (HistoryRun, Vec<String>) {
    let (file, _, _) = read_file();
    // Retain a real completed contact/selection composition and all its outputs
    // alongside readiness. Readiness must neither advance nor republish that work.
    let mut appearance = make_work(&file, "Work", "C-contact-and-start");
    appearance.finish();
    let animation = appearance.state;
    let mut run = HistoryRun {
        look: Look {
            animation,
            cannon: CannonDelay::new(v.initial_delay).expect("bounded delay"),
        },
        observation: None,
        next_step: 0,
        provider: v.steps.clone(),
        reads: Vec::new(),
        external_delivered: 0,
        appearance,
    };
    run = checkpoint(run, restore);
    let mut complete_states = vec![toml::to_string(&run).expect("complete starting owners")];
    while let Some(step) = run.provider.get(run.next_step).cloned() {
        step.validate();
        let outputs = toml::to_string(&run.appearance).expect("complete old work");
        run = perform(run, &step, restore);
        assert_eq!(run.look.animation, animation);
        assert_eq!(toml::to_string(&run.appearance).expect("old work"), outputs);
        complete_states.push(toml::to_string(&run).expect("complete operation result"));
    }
    (run, complete_states)
}

#[test]
#[expect(clippy::print_stdout, reason = "report additive history count")]
fn every_cannon_history() {
    let (file, _, _) = read_file();
    for v in &file.cannon_readiness {
        assert_ne!(v.source, "");
        assert_eq!(v.restoration, "SerializeDropDeserializeAtEveryBoundary");
        for entry in &v.entries {
            behaviour(entry);
        }
        assert_eq!(
            run_history(v, false),
            run_history(v, true),
            "{}: complete readiness, progress and ordered effects",
            v.id
        );
    }
    println!(
        "{} cannon readiness histories passed",
        file.cannon_readiness.len()
    );
}

#[test]
fn strict_cannon_parser_rejects_unknown_fields_and_names() {
    let text = "id='test'\nsource='constructed'\nentries=[]\ninitial_delay=0\nrestoration='SerializeDropDeserializeAtEveryBoundary'\nsteps=[{kind='draw',expected_delay=0,expected_may_follow=true}]";
    assert!(toml::from_str::<CannonHistory>(text).is_ok());
    assert!(toml::from_str::<CannonHistory>(&format!("{text}\nunknown=1")).is_err());
    assert!(
        toml::from_str::<CannonHistory>(&text.replace("kind='draw'", "kind='draw',unknown=1"))
            .is_err()
    );
    assert!(
        toml::from_str::<CannonHistory>(&text.replace("kind='draw'", "kind='redraw'")).is_err()
    );
    assert!(
        toml::from_str::<CannonHistory>(&text.replace(
            "kind='draw'",
            "kind='outside_selection',item_before='Missile',item_after='None'"
        ))
        .is_err()
    );
    assert!(
        toml::from_str::<super::VectorFile>(&format!("{}\nunknown=1", super::VECTORS)).is_err()
    );
}

#[test]
fn bounded_construction_and_unmade_completion() {
    for n in 0..=2 {
        let delay = CannonDelay::new(n).expect("every bounded value");
        assert_eq!(delay.remaining(), n);
        assert_eq!(delay.may_follow(), n == 0);
        let mut look = Look {
            animation: state(3, 7, 2, 1),
            cannon: delay,
        };
        let before = look;
        let mut unmade = CannonObservation::Unmade;
        assert!(!unmade.complete(&mut look));
        assert_eq!(look, before);
        assert_eq!(unmade, CannonObservation::Unmade);
    }
    assert_eq!(CannonDelay::new(3), None);
    assert_eq!(CannonDelay::new(u8::MAX), None);
    let valid = Look {
        animation: state(3, 7, 2, 1),
        cannon: CannonDelay::INITIAL,
    };
    let text = toml::to_string(&valid).expect("complete look");
    assert_eq!(toml::from_str::<Look>(&text).expect("valid look"), valid);
    assert!(text.contains("Two"));
    assert!(toml::from_str::<Look>(&text.replace("Two", "Three")).is_err());
}

#[test]
fn retained_eligibility_and_consumed_progress_are_discriminating() {
    let (file, _, _) = read_file();
    let answers = [
        ItemControlAnswer {
            before: SelectedItem::None,
            after: SelectedItem::None,
            eligibility: ControlEligibility::Eligible,
        },
        ItemControlAnswer {
            before: SelectedItem::None,
            after: SelectedItem::Missiles,
            eligibility: ControlEligibility::Eligible,
        },
        ItemControlAnswer {
            before: SelectedItem::None,
            after: SelectedItem::Missiles,
            eligibility: ControlEligibility::Ineligible,
        },
    ];
    for (answer, remaining) in answers.into_iter().zip([1, 1, 2]) {
        let appearance = make_work(&file, "Work", "C-contact-and-start");
        let run = HistoryRun {
            look: Look {
                animation: appearance.state,
                cannon: CannonDelay::INITIAL,
            },
            observation: Some(CannonObservation::Unmade),
            next_step: 0,
            provider: Vec::new(),
            reads: Vec::new(),
            external_delivered: 1,
            appearance,
        };
        let mut run = fresh(run);
        assert!(run.observation.as_mut().expect("unmade").choose(answer));
        let mut run = fresh(run);
        assert_eq!(
            run.look.cannon,
            CannonDelay::INITIAL,
            "choice has no effects"
        );
        let later = ItemControlAnswer {
            eligibility: ControlEligibility::Eligible,
            after: SelectedItem::Grapple,
            ..answer
        };
        assert!(!run.observation.as_mut().expect("chosen").choose(later));
        assert!(
            run.observation
                .as_mut()
                .expect("chosen")
                .complete(&mut run.look)
        );
        let mut run = fresh(run);
        assert_eq!(run.look.cannon.remaining(), remaining);
        assert_eq!(run.observation, Some(CannonObservation::Completed(answer)));
        assert!(
            !run.observation
                .as_mut()
                .expect("completed")
                .complete(&mut run.look)
        );
        assert_eq!(
            run.look.cannon.remaining(),
            remaining,
            "repeated unchanged completion would prematurely reach zero"
        );
        assert_eq!(run.external_delivered, 1);
    }
}

fn animation_checkpoint(work: SuspendedAppearance, delay: u8) -> SuspendedAppearance {
    let run = HistoryRun {
        look: Look {
            animation: work.state,
            cannon: CannonDelay::new(delay).expect("bounded"),
        },
        observation: Some(CannonObservation::Chosen(ItemControlAnswer {
            before: SelectedItem::None,
            after: SelectedItem::Missiles,
            eligibility: ControlEligibility::Eligible,
        })),
        next_step: 0,
        provider: Vec::new(),
        reads: Vec::new(),
        external_delivered: 0,
        appearance: work,
    };
    let restored = fresh(run);
    assert_eq!(restored.look.cannon.remaining(), delay);
    assert_eq!(restored.look.animation, restored.appearance.state);
    restored.appearance
}

#[test]
fn readiness_survives_every_existing_animation_boundary() {
    let (file, _, _) = read_file();
    for delay in 0..=2 {
        for v in &file.suspended_update {
            // The existing strict tests still check their full base results and
            // observation subsequences. This adds complete-look preservation.
            let mut work = make_work(&file, &v.base_kind, &v.base);
            let mut expected = work.clone();
            expected.finish();
            loop {
                let _ = work.offer();
                work = animation_checkpoint(work, delay);
                if !work.complete() {
                    break;
                }
                work = animation_checkpoint(work, delay);
            }
            assert_eq!(work, expected);
        }
    }
}
