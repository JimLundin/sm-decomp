use super::runtime_helpers::{admitted, advance_until, restore_next};
use super::*;
use game::production::timing::{Event, EventKind};
use game::{
    production::{
        AdmissionFault, AdvanceInput, CompletedFrameSaveFault, ProductionFault,
        SavedCompletedFrame, Unsupported,
    },
    state::owned::OwnedState,
};

#[test]
#[ignore = "requires accepted retained liquid and actual first-frame owners"]
fn actual_animation_consumes_after_countdown_and_restores_next_result() {
    let mut admitted = admitted();
    let InputReader::Frame(inputs) = &admitted.inputs else {
        panic!("frame inputs")
    };
    let budget = inputs.budget;
    advance_until(&mut admitted, budget, |state| {
        state
            .health
            .as_ref()
            .is_some_and(|health| health.actor().is_some())
    });
    advance_until(&mut admitted, budget, |state| {
        matches!(
            state.caller,
            frame::Caller::PlayerContinuation {
                post_down: frame::post_down_caller::Stage::Complete(_),
                ..
            }
        )
    });
    advance_until(&mut admitted, budget, |state| state.hud_map.is_complete());
    advance_until(&mut admitted, budget, |state| {
        matches!(
            state.countdown,
            frame::countdown_caller::Stage::Complete { .. }
        )
    });
    let OwnedState::Frame(state) = admitted.production.state() else {
        panic!("frame state")
    };
    let countdown = state.countdown.clone();
    let delivered = state.countdown_delivered.clone();
    let before_animation = state.as_ref().clone();
    let ProductionContent::Frame(content) = &admitted.content else {
        panic!("frame content")
    };
    let ProductionRules::Frame(rules) = &admitted.rules else {
        panic!("frame rules")
    };
    reject_animation_history_changes(&before_animation, &delivered, content, rules, budget);
    restore_next(&mut admitted, budget).unwrap();
    advance_to_animation_return(&mut admitted, budget);
    let OwnedState::Frame(state) = admitted.production.state() else {
        panic!("frame state")
    };
    assert!(state.countdown.eq(&countdown));
    assert!(state.countdown_delivered.eq(&delivered));
    let frame::animation_caller::Stage::Complete(completion) = &state.animation else {
        panic!("animation")
    };
    assert_observation_eq!(
        completion.work,
        frame::appearance_caller::Kind::Animation.identity()
    );
    assert!(
        state.appearance == completion.receipt.result,
        "actual animation result consumed"
    );
    assert!(
        completion
            .receipt
            .before
            .model
            .0
            .state
            .ne(&completion.receipt.result.model.0.state)
    );
    reject_completed_animation_changes(&admitted, state, completion, &delivered, budget);
    let before = admitted.production.state().clone();
    assert!(matches!(
        admitted.production.advance(AdvanceInput::Frame { budget }),
        Err(ProductionFault::Unsupported(Unsupported::Frame(
            frame::Dependency::InterruptionPreparation
        )))
    ));
    assert_observation_eq!(admitted.production.state(), &before);
    assert!(restore_next(&mut admitted, budget).is_err());
}

fn reject_completed_animation_changes(
    admitted: &AdmittedCase,
    state: &frame::State,
    completion: &frame::appearance_caller::Completion,
    delivered: &[Model<frame::countdown_caller::AnswerPin>],
    budget: data::timing::MasterTicks,
) {
    let current = state;
    let ProductionContent::Frame(content) = &admitted.content else {
        panic!("frame content")
    };
    let ProductionRules::Frame(rules) = &admitted.rules else {
        panic!("frame rules")
    };
    for mutation in 0..3 {
        let mut changed = current.clone();
        match mutation {
            0 => changed.appearance = completion.receipt.before.clone(),
            1 => changed.animation = frame::animation_caller::Stage::Pending,
            2 => changed
                .countdown_delivered
                .extend(delivered.iter().copied()),
            _ => panic!("mutation"),
        }
        let before = changed.clone();
        assert!(changed.advance(content, rules, budget).is_err());
        assert!(changed.eq(&before));
        assert!(
            Production::admit(
                ProductionSeed::Frame(Box::new(changed)),
                admitted.content.clone(),
                admitted.rules.clone()
            )
            .is_err()
        );
    }
}

fn advance_to_animation_return(admitted: &mut AdmittedCase, budget: data::timing::MasterTicks) {
    for _ in 0..100_000 {
        let OwnedState::Frame(state) = admitted.production.state() else {
            panic!("frame state")
        };
        if matches!(state.animation, frame::animation_caller::Stage::Complete(_)) {
            break;
        }
        let previous = state.timing.hardware.now;
        admitted
            .production
            .advance(AdvanceInput::Frame { budget })
            .unwrap();
        let OwnedState::Frame(state) = admitted.production.state() else {
            panic!("frame state")
        };
        if state.timing.hardware.now == previous
            && state.timing.work.is_empty()
            && !matches!(state.animation, frame::animation_caller::Stage::Complete(_))
        {
            panic!("actual selected animation has no physical progress");
        }
    }
}

fn reject_animation_history_changes(
    before_animation: &frame::State,
    delivered: &[Model<frame::countdown_caller::AnswerPin>],
    content: &frame::Content,
    rules: &frame::Rules,
    budget: data::timing::MasterTicks,
) {
    for mutation in 0..3 {
        let mut changed = before_animation.clone();
        match mutation {
            0 => changed.liquid_history = None,
            1 => changed
                .liquid_history
                .as_mut()
                .unwrap()
                .write(frame::liquid_history::Meaning::Air),
            2 => changed.liquid_history.as_mut().unwrap().invalidate(),
            _ => panic!("mutation"),
        }
        let before = changed.clone();
        assert!(matches!(
            changed.advance(content, rules, budget),
            Err(ProductionFault::Unsupported(Unsupported::Frame(
                frame::Dependency::AnimationLiquidHistory
            )))
        ));
        assert!(changed.eq(&before));
        assert!(changed.countdown_delivered.as_slice().eq(delivered));
    }
}

#[test]
#[ignore = "requires accepted V2 health origin and explicit finite context scope"]
fn actual_live_health_origin_actor_irq_and_full_restoration_before_animation() {
    let mut admitted = admitted();
    let InputReader::Frame(inputs) = &admitted.inputs else {
        panic!("frame inputs")
    };
    let budget = inputs.budget;
    let ProductionContent::Frame(content) = &admitted.content else {
        panic!("frame content")
    };
    let origin = content.health.as_ref().unwrap().initial.0;
    let OwnedState::Frame(initial) = admitted.production.state() else {
        panic!("frame state")
    };
    assert_observation_eq!(initial.health.as_ref().unwrap().state(), &origin);
    assert!(initial.health.as_ref().unwrap().actor().is_none());
    let initial = initial.as_ref().clone();
    restore_next(&mut admitted, budget).unwrap();
    advance_until(&mut admitted, budget, |state| {
        state
            .health
            .as_ref()
            .is_some_and(|health| health.actor().is_some())
    });
    restore_next(&mut admitted, budget).unwrap();
    advance_until(&mut admitted, budget, |state| {
        matches!(
            state.caller,
            frame::Caller::PlayerContinuation {
                post_down: frame::post_down_caller::Stage::Complete(_),
                ..
            }
        )
    });
    advance_until(&mut admitted, budget, |state| state.hud_map.is_complete());
    restore_next(&mut admitted, budget).unwrap();
    advance_until(&mut admitted, budget, |state| {
        matches!(
            state.countdown,
            frame::countdown_caller::Stage::Complete { .. }
        )
    });
    let OwnedState::Frame(actual) = admitted.production.state() else {
        panic!("frame state")
    };
    let actual = actual.as_ref().clone();
    let ProductionContent::Frame(content) = &admitted.content else {
        panic!("frame content")
    };
    let ProductionRules::Frame(rules) = &admitted.rules else {
        panic!("frame rules")
    };
    let frame::Caller::PlayerContinuation { remaining, .. } = &actual.caller else {
        panic!("frame caller")
    };
    assert_observation_eq!(actual.health.as_ref().unwrap().state(), &origin);
    assert_observation_eq!(
        actual.health.as_ref().unwrap().at_animation(
            content.health.as_ref().unwrap(),
            remaining.0.actors.as_ref().unwrap()
        ),
        Ok(origin.energy.whole.get())
    );
    let observed_countdown_receiver = actual.countdown_delivered.clone();
    for mutation in 0..5 {
        let mut changed = actual.clone();
        match mutation {
            0 => changed.health = None,
            1 => changed.health.as_mut().unwrap().write(origin),
            2 => changed
                .health
                .as_mut()
                .unwrap()
                .write_context(content.health.as_ref().unwrap().context),
            3 => changed.health.as_mut().unwrap().invalidate(),
            4 => changed.drawing_origins.as_mut().unwrap().invincibility.0 = 1,
            _ => panic!("mutation"),
        }
        let before = changed.clone();
        assert!(matches!(
            changed.advance(content, rules, budget),
            Err(ProductionFault::Unsupported(Unsupported::Frame(
                frame::Dependency::HealthCurrentInputs
            )))
        ));
        assert!(changed.eq(&before));
        assert!(changed.countdown_delivered.eq(&observed_countdown_receiver));
    }
    synthetic_health_restores_complete_loss(initial, content, &admitted.rules);
}

fn synthetic_health_restores_complete_loss(
    initial: frame::State,
    content: &frame::Content,
    rules: &ProductionRules,
) {
    // Construction and read-only energy qualification must not consume pending loss.
    let mut synthetic_origin = content.health.as_ref().unwrap().clone();
    synthetic_origin.initial.0.energy.fraction = 123;
    synthetic_origin.initial.0.pending_loss.whole = samus_health::WholeEnergy::new(2).unwrap();
    synthetic_origin.initial.0.pending_loss.fraction = 456;
    synthetic_origin.initial.0.warning = samus_health::Warning::On;
    let mut synthetic_state = initial;
    synthetic_state.health =
        Some(frame::health::Health::from_origin(synthetic_origin.clone()).unwrap());
    let mut synthetic_content = content.clone();
    synthetic_content.health = Some(synthetic_origin.clone());
    // Use the production codec, not a source TOML or a State-only health codec.
    let synthetic_content = ProductionContent::Frame(Box::new(synthetic_content));
    let synthetic = Production::admit(
        ProductionSeed::Frame(Box::new(synthetic_state)),
        synthetic_content.clone(),
        rules.clone(),
    )
    .unwrap();
    let saved = synthetic.save().unwrap();
    let fresh = Production::restore(&saved, synthetic_content, rules.clone());
    assert!(fresh.is_ok());
    let fresh = fresh.unwrap();
    let OwnedState::Frame(restored) = fresh.state() else {
        panic!("frame state")
    };
    assert_observation_eq!(
        restored.health.as_ref().unwrap().state(),
        &synthetic_origin.initial.0
    );
    assert_observation_eq!(
        restored.health.as_ref().unwrap().state().pending_loss,
        synthetic_origin.initial.0.pending_loss
    );
}

#[test]
#[ignore = "requires authenticated inactive interruption lifetime and full first-frame content"]
fn actual_interruption_preserves_movement_and_restores_its_return() {
    use frame::interruption_caller::Stage;

    let mut admitted = admitted();
    let InputReader::Frame(inputs) = &admitted.inputs else {
        panic!("frame inputs")
    };
    let budget = inputs.budget;
    for _ in 0..200_000 {
        let OwnedState::Frame(state) = admitted.production.state() else {
            panic!("frame state")
        };
        if matches!(state.animation, frame::animation_caller::Stage::Complete(_)) {
            break;
        }
        admitted
            .production
            .advance(AdvanceInput::Frame { budget })
            .unwrap();
    }
    let OwnedState::Frame(state) = admitted.production.state() else {
        panic!("frame state")
    };
    assert!(matches!(
        state.animation,
        frame::animation_caller::Stage::Complete(_)
    ));
    let movement = state.movement.clone();
    let ProductionContent::Frame(content) = &admitted.content else {
        panic!("frame content")
    };
    let ProductionRules::Frame(rules) = &admitted.rules else {
        panic!("frame rules")
    };
    let mut bad = state.as_ref().clone();
    bad.liquid_history
        .as_mut()
        .unwrap()
        .invalidate_interruption();
    let before = bad.clone();
    assert!(matches!(
        bad.advance(content, rules, budget),
        Err(ProductionFault::Unsupported(Unsupported::Frame(
            frame::Dependency::InterruptionPreparation
        )))
    ));
    assert!(bad.eq(&before));
    restore_next(&mut admitted, budget).unwrap();
    advance_until(&mut admitted, budget, |state| {
        matches!(state.interruption, Stage::Charging(_))
    });
    restore_next(&mut admitted, budget).unwrap();
    advance_until(&mut admitted, budget, |state| {
        matches!(state.interruption, Stage::Complete(_))
    });
    let OwnedState::Frame(state) = admitted.production.state() else {
        panic!("frame state")
    };
    assert!(state.movement.eq(&movement));
    assert_observation_eq!(
        restore_next(&mut admitted, budget),
        Err(ProductionFault::Unsupported(Unsupported::Frame(
            frame::Dependency::AimPreparation
        )))
    );
}

#[test]
#[ignore = "requires the complete actual late movement fixture"]
fn actual_late_movement_finishes_with_selected_playing_figure() {
    use frame::late_movement::Stage;
    use samus_movement::suspended::{LocalRule, MovementDecision};

    let mut admitted = admitted();
    let InputReader::Frame(inputs) = &admitted.inputs else {
        panic!("frame inputs")
    };
    let budget = inputs.budget;
    for _ in 0..200_000 {
        let OwnedState::Frame(state) = admitted.production.state() else {
            panic!("frame state")
        };
        if matches!(
            state.interruption,
            frame::interruption_caller::Stage::Complete(_)
        ) {
            break;
        }
        admitted
            .production
            .advance(AdvanceInput::Frame { budget })
            .unwrap();
    }
    restore_next(&mut admitted, budget).unwrap();
    advance_until(&mut admitted, budget, |state| {
        matches!(state.late_movement, Stage::AimReady)
    });
    restore_next(&mut admitted, budget).unwrap();
    advance_until(&mut admitted, budget, |state| {
        matches!(state.late_movement, Stage::Aim(_))
    });
    restore_next(&mut admitted, budget).unwrap();
    advance_until(&mut admitted, budget, |state| {
        matches!(state.late_movement, Stage::FitReady(_))
    });
    restore_next(&mut admitted, budget).unwrap();
    advance_until(&mut admitted, budget, |state| {
        matches!(state.late_movement, Stage::Fit(_))
    });
    restore_next(&mut admitted, budget).unwrap();
    advance_until(&mut admitted, budget, |state| {
        matches!(state.late_movement, Stage::Complete(_))
    });
    let OwnedState::Frame(state) = admitted.production.state() else {
        panic!("frame state")
    };
    let (samus, outcome) = state.movement.finished().unwrap();
    assert_observation_eq!(samus.pose, state.playing_figure.0.pose);
    assert_observation_eq!(outcome.sprite, state.playing_figure.0.sprite);
    assert!(matches!(
        state.movement.decisions().last(),
        Some(MovementDecision::Local {
            rule: LocalRule::Finish,
            ..
        })
    ));
    assert_observation_eq!(
        restore_next(&mut admitted, budget),
        Err(ProductionFault::Unsupported(Unsupported::Frame(
            frame::Dependency::SamusPalettePreparation
        )))
    );
}

#[test]
#[ignore = "requires the full original frame, native expected publications and completion owners"]
fn actual_native_frame_completes_and_matches_from_the_retained_checkpoint() {
    check_complete_frame_from(|state| {
        matches!(
            state.interruption,
            frame::interruption_caller::Stage::Complete(_)
        )
    });
}

#[test]
#[ignore = "requires the full original frame and the actual final NMI admission receipt"]
fn actual_native_admission_restores_complete_return() {
    check_complete_frame_from(|state| {
        matches!(
            state.return_progress,
            frame::frame_return::Stage::Ready {
                suffix: frame::suffix_caller::Suffix::Entering { .. },
                ..
            }
        )
    });
}

#[test]
#[ignore = "requires the retained native frame and checked player ROM"]
fn actual_native_late_publication_failure_rolls_back_production() {
    use game::production::{ProductionOutputs, timing::Stage};

    let (_, mut admitted) = super::runtime_helpers::retained_return();
    let InputReader::Frame(inputs) = &admitted.inputs else {
        panic!("frame inputs")
    };
    let budget = inputs.budget;
    advance_until(&mut admitted, budget, |state| {
        state.timing.work.last().is_some_and(|work| {
            work.cursor.stage == Stage::Publish
                && usize::try_from(work.cursor.run)
                    .ok()
                    .and_then(|run| work.profile.runs.get(run))
                    .is_some_and(|run| {
                        matches!(run.exposure.effect, data::timing::DeviceEffect::Publish(_))
                    })
        })
    });

    let OwnedState::Frame(state) = admitted.production.state() else {
        panic!("frame state")
    };
    let mut blocked = state.clone();
    // The retained stream is ordered, but its last timestamp blocks the next actual write.
    blocked.publications.events.push(Event {
        at: data::timing::MasterTicks(state.timing.hardware.now.0.checked_add(1).unwrap()),
        kind: EventKind::ControllerCommitted,
    });

    let ProductionOutputs::Frame(output) = admitted
        .production
        .advance(AdvanceInput::Frame { budget })
        .unwrap()
    else {
        panic!("frame output")
    };
    assert!(
        output
            .events
            .iter()
            .any(|event| { matches!(event.kind, EventKind::Publication(_)) })
    );
    let mut production = Production::admit(
        ProductionSeed::Frame(blocked),
        admitted.content.clone(),
        admitted.rules.clone(),
    )
    .unwrap();
    let before = production.save().unwrap();
    let mut restored = Production::restore(&before, admitted.content, admitted.rules).unwrap();
    for production in [&mut production, &mut restored] {
        assert!(matches!(
            production.advance(AdvanceInput::Frame { budget }),
            Err(ProductionFault::FramePublications(
                frame::PublicationFault::Order
            ))
        ));
        // The checkpoint covers the hardware cursor, output prefix and delivery cursor.
        assert_observation_eq!(production.save().unwrap(), before);
        assert_incomplete_frame(production);
    }
}

#[test]
#[ignore = "requires the retained native frame and checked player ROM"]
fn actual_native_completed_animation_binds_historical_health() {
    use data::timing::MasterTicks;

    let (_, mut admitted) = super::runtime_helpers::retained_return();
    let budget = MasterTicks(1);
    advance_to_countdown_return(&mut admitted, budget);
    let altered = complete_counterfactual_animation(&mut admitted, budget);
    assert_animation_checkpoint_restores(&admitted);
    reject_animation_energy_checkpoint(&admitted, altered);

    let (_, mut with_loss) = super::runtime_helpers::retained_return();
    add_pending_loss(&mut with_loss);
    advance_to_countdown_return(&mut with_loss, budget);
    advance_to_animation_return(&mut with_loss, budget);
    let before = current_whole_energy(&with_loss);
    advance_until(&mut with_loss, budget, |state| {
        state.maintenance.energy().is_some()
    });
    assert_observation_eq!(
        Some(before),
        current_whole_energy(&with_loss).checked_add(1)
    );
    assert_animation_checkpoint_restores(&with_loss);
}

fn advance_to_countdown_return(admitted: &mut AdmittedCase, budget: data::timing::MasterTicks) {
    advance_until(admitted, budget, |state| {
        matches!(
            state.countdown,
            frame::countdown_caller::Stage::Complete { .. }
        )
    });
}

fn current_whole_energy(admitted: &AdmittedCase) -> u16 {
    let OwnedState::Frame(state) = admitted.production.state() else {
        panic!("frame state");
    };
    state.health.as_ref().unwrap().state().energy.whole.get()
}

fn add_pending_loss(admitted: &mut AdmittedCase) {
    let OwnedState::Frame(initial) = admitted.production.state() else {
        panic!("frame state");
    };
    let ProductionContent::Frame(content) = &mut admitted.content else {
        panic!("frame content");
    };
    let health = content.health.as_mut().unwrap();
    health.initial.0.pending_loss.whole = samus_health::WholeEnergy::new(1).unwrap();
    let mut initial = initial.clone();
    initial.health = Some(frame::health::Health::from_origin(health.clone()).unwrap());
    admitted.production = Production::admit(
        ProductionSeed::Frame(initial),
        admitted.content.clone(),
        admitted.rules.clone(),
    )
    .unwrap();
}

fn complete_counterfactual_animation(
    admitted: &mut AdmittedCase,
    budget: data::timing::MasterTicks,
) -> Box<frame::State> {
    use frame::appearance_caller::{Kind, Stage};

    let OwnedState::Frame(state) = admitted.production.state() else {
        panic!("frame state");
    };
    let ProductionContent::Frame(content) = &admitted.content else {
        panic!("frame content");
    };
    let ProductionRules::Frame(rules) = &admitted.rules else {
        panic!("frame rules");
    };
    let mut replacement = state.as_ref().clone();
    let mut inputs = state.fresh_animation_inputs(content).unwrap();
    let actual_energy = inputs.energy;
    let substituted_energy = actual_energy ^ 1;
    inputs.energy = substituted_energy;
    let mut selection = Stage::Idle;
    selection
        .animation(
            &mut replacement,
            &content.appearance,
            inputs,
            rules.appearance.clone(),
        )
        .unwrap();
    content
        .animation
        .as_ref()
        .unwrap()
        .qualify(
            &selection,
            &replacement,
            &content.movement,
            &content.appearance,
        )
        .unwrap();
    let work = Kind::Animation.identity();
    selection.bind(work).unwrap();
    let (completion, _) = selection.complete(&mut replacement, work).unwrap();
    advance_to_animation_return(admitted, budget);
    let OwnedState::Frame(state) = admitted.production.state() else {
        panic!("frame state");
    };
    let frame::animation_caller::Stage::Complete(original) = &state.animation else {
        panic!("completed Animation");
    };
    let original_answer = original.receipt.chosen.model.0.pending.as_ref().unwrap();
    let substituted_answer = completion.receipt.chosen.model.0.pending.as_ref().unwrap();
    assert_eq!(original_answer.answer, samus_appearance::Answer::Advance);
    assert_eq!(substituted_answer.answer, original_answer.answer);
    assert_observation_eq!(original_answer.inputs.energy, actual_energy);
    assert_observation_eq!(substituted_answer.inputs.energy, substituted_energy);
    assert_observation_eq!(
        original.receipt.result.model.0.state,
        completion.receipt.result.model.0.state
    );
    assert_observation_eq!(
        state.health.as_ref().unwrap().state().energy.whole.get(),
        actual_energy
    );
    assert!(state.maintenance.energy().is_none());
    let mut altered = state.clone();
    altered.appearance = replacement.appearance;
    altered.animation = frame::animation_caller::Stage::Complete(completion);
    altered
}

fn assert_animation_checkpoint_restores(admitted: &AdmittedCase) {
    let restored = Production::restore(
        &admitted.production.save().unwrap(),
        admitted.content.clone(),
        admitted.rules.clone(),
    )
    .unwrap();
    assert_observation_eq!(restored.state(), admitted.production.state());
}

fn reject_animation_energy_checkpoint(admitted: &AdmittedCase, altered: Box<frame::State>) {
    use game::production::AdmissionFault;

    altered.validate().unwrap();
    let attempted = Production::admit(
        ProductionSeed::Frame(altered),
        admitted.content.clone(),
        admitted.rules.clone(),
    )
    .and_then(|production| {
        Production::restore(
            &production.save().unwrap(),
            admitted.content.clone(),
            admitted.rules.clone(),
        )
    });
    assert!(matches!(
        attempted,
        Err(AdmissionFault::Frame(fault)) if *fault == frame::Fault::InvalidPlayerEntry
    ));
}

fn check_complete_frame_from(reached: impl Fn(&frame::State) -> bool) {
    let (case, mut admitted) = super::runtime_helpers::retained_return();
    assert_incomplete_frame(&admitted.production);
    let InputReader::Frame(inputs) = &admitted.inputs else {
        panic!("frame inputs")
    };
    let budget = inputs.budget;
    let cut = case.end.cut_inclusive;
    let mut checkpoint = None;
    for _ in 0..500_000 {
        let OwnedState::Frame(state) = admitted.production.state() else {
            panic!("frame state")
        };
        if reached(state) {
            checkpoint = Some(admitted.production.save().unwrap());
            break;
        }
        let game::production::ProductionOutputs::Frame(output) = admitted
            .production
            .advance(AdvanceInput::Frame { budget })
            .unwrap()
        else {
            panic!("frame output")
        };
        assert_observation_eq!(output.progress, frame::Progress::Advanced);
        let super::super::readers::ActualReader::Frame(actual) = &mut admitted.actual else {
            panic!("frame receiver")
        };
        actual.events.extend(output.events);
    }
    let checkpoint = checkpoint.expect("actual returned-work checkpoint");
    let mut restored = Production::restore(
        &checkpoint,
        admitted.content.clone(),
        admitted.rules.clone(),
    )
    .unwrap();
    assert!(restored.state().eq(admitted.production.state()));
    assert_incomplete_frame(&restored);
    let mut restored_actual = admitted.actual.clone();
    for (production, actual) in [
        (&mut admitted.production, &mut admitted.actual),
        (&mut restored, &mut restored_actual),
    ] {
        let result = super::super::runner::execute(
            &case,
            &admitted.inputs,
            &admitted.expected,
            production,
            actual,
            cut,
        );
        let compared = result.unwrap();
        assert_eq!(compared.semantic_count, case.expectations.fields.len());
        let OwnedState::Frame(frame) = production.state() else {
            panic!("frame state")
        };
        assert_observation_eq!(frame.timing.hardware.now.0, 359_922);
        assert_observation_eq!(actual.publication_count(), 2_295);
    }
    assert_observation_eq!(restored.state(), admitted.production.state());
    let saved = admitted.production.save_completed_frame().unwrap();
    assert_observation_eq!(restored.save_completed_frame().unwrap(), saved);
    let mut reopened = Production::restore_completed_frame(
        &saved,
        admitted.content.clone(),
        admitted.rules.clone(),
    )
    .unwrap();
    assert_observation_eq!(reopened.state(), admitted.production.state());
    assert_observation_eq!(reopened.save_completed_frame().unwrap(), saved);
    let OwnedState::Frame(original) = admitted.production.state() else {
        panic!("frame state")
    };
    reject_inconsistent_return_history(original);
    reject_other_completed_frame_context(&admitted, &saved, original);

    assert!(matches!(
        reopened.advance(AdvanceInput::Missing(reopened.scope())),
        Err(ProductionFault::Unsupported(Unsupported::MissingInput(_)))
    ));
    assert_observation_eq!(reopened.save_completed_frame().unwrap(), saved);
    check_completed_frame_pad_scheduling(&reopened);
    start_next_opportunity(&mut admitted, &mut reopened);
}

fn reject_other_completed_frame_context(
    admitted: &AdmittedCase,
    saved: &SavedCompletedFrame,
    original: &frame::State,
) {
    let readmitted = Production::admit(
        ProductionSeed::Frame(Box::new(original.clone())),
        admitted.content.clone(),
        admitted.rules.clone(),
    )
    .unwrap();
    assert_incomplete_frame(&readmitted);

    let mut other_content = admitted.content.clone();
    let ProductionContent::Frame(content) = &mut other_content else {
        panic!("frame content")
    };
    content.return_work = None;
    assert!(
        Production::admit(
            ProductionSeed::Frame(Box::new(original.clone())),
            other_content.clone(),
            admitted.rules.clone(),
        )
        .is_ok()
    );
    assert!(matches!(
        Production::restore_completed_frame(saved, other_content, admitted.rules.clone()),
        Err(AdmissionFault::ContentChanged)
    ));
    let mut other_rules = admitted.rules.clone();
    let ProductionRules::Frame(rules) = &mut other_rules else {
        panic!("frame rules")
    };
    rules.movement.booster_lifts_jumps = !rules.movement.booster_lifts_jumps;
    assert!(matches!(
        Production::restore_completed_frame(saved, admitted.content.clone(), other_rules),
        Err(AdmissionFault::RulesChanged)
    ));
}

fn check_completed_frame_pad_scheduling(production: &Production) {
    use game::production::timing::{PadChange, PhysicalPad};

    let mut production = production.clone();
    let OwnedState::Frame(frame) = production.state() else {
        panic!("frame state")
    };
    let mut change = PadChange {
        at: data::timing::MasterTicks(frame.timing.hardware.now.0.checked_sub(1).unwrap()),
        pad: PhysicalPad::default(),
    };
    let saved = production.save_completed_frame().unwrap();
    assert!(production.schedule_frame_pad_changes(&[change]).is_err());
    production.schedule_frame_pad_changes(&[]).unwrap();
    assert_observation_eq!(production.save_completed_frame().unwrap(), saved);

    change.at = data::timing::MasterTicks(u64::MAX);
    production.schedule_frame_pad_changes(&[change]).unwrap();
    assert_incomplete_frame(&production);
}

#[track_caller]
fn assert_incomplete_frame(production: &Production) {
    assert!(matches!(
        production.save_completed_frame(),
        Err(CompletedFrameSaveFault::Incomplete)
    ));
}

fn start_next_opportunity(admitted: &mut AdmittedCase, restored_return: &mut Production) {
    use data::timing::MasterTicks;

    let OwnedState::Frame(returned) = admitted.production.state() else {
        panic!("frame state")
    };
    let controller = returned.frame_history.as_ref().unwrap().controller();
    let health = *returned.health.as_ref().unwrap().state();
    let play_time = returned.play_time.current();
    let room_quake = returned.room_quake.current();
    let movement = *returned.movement.state();
    let hud = returned.hud.state().clone();
    let buffers = returned.content.regions();
    let haze = returned.haze.clone();
    let now = returned.timing.hardware.now;
    let prefix = returned.publications.clone();
    let music = returned.music.unwrap();
    let tick = || AdvanceInput::Frame {
        budget: MasterTicks(1),
    };
    let game::production::ProductionOutputs::Frame(first) =
        admitted.production.advance(tick()).unwrap()
    else {
        panic!("frame output");
    };
    assert_observation_eq!(
        restored_return.advance(tick()).unwrap(),
        game::production::ProductionOutputs::Frame(first.clone())
    );
    assert_observation_eq!(restored_return.state(), admitted.production.state());
    assert_incomplete_frame(&admitted.production);
    assert_incomplete_frame(restored_return);
    let mut delivered = first.events;
    for _ in 0..100 {
        let OwnedState::Frame(state) = admitted.production.state() else {
            panic!("frame state")
        };
        if state.timing.hardware.now > now {
            break;
        }
        let game::production::ProductionOutputs::Frame(output) =
            admitted.production.advance(tick()).unwrap()
        else {
            panic!("frame output");
        };
        delivered.extend(output.events);
    }
    let OwnedState::Frame(started) = admitted.production.state() else {
        panic!("frame state")
    };
    assert!(matches!(
        started.caller,
        frame::Caller::Acknowledgment {
            prelude: frame::PreludeSource::Recurring,
            ..
        }
    ));
    assert!(started.timing.hardware.now > now);
    let saved = admitted.production.save().unwrap();
    let mut restored =
        Production::restore(&saved, admitted.content.clone(), admitted.rules.clone()).unwrap();

    for (production, budget) in [
        (&mut admitted.production, MasterTicks(10_000)),
        (&mut restored, MasterTicks(1)),
    ] {
        let mut events = delivered.clone();
        events.extend(complete_recurring_prelude(production, budget, now));
        let OwnedState::Frame(frame) = production.state() else {
            panic!("frame state")
        };
        assert_observation_eq!(frame.music, Some(music));
        assert_observation_eq!(frame.health.as_ref().unwrap().state(), &health);
        assert_observation_eq!(frame.play_time.current(), play_time);
        assert_observation_eq!(frame.room_quake.current(), room_quake);
        assert_observation_eq!(frame.movement.state(), &movement);
        assert_observation_eq!(frame.hud.state(), &hud);
        assert_observation_eq!(frame.content.regions(), buffers);
        assert_observation_eq!(frame.haze, haze);
        assert!(matches!(
            frame.return_progress,
            frame::frame_return::Stage::Pending
        ));
        assert!(matches!(frame.late_hud, frame::late_hud::Stage::Pending));
        assert!(!frame.hud_map.is_complete());
        assert!(!frame.play_time.is_complete());
        assert_observation_eq!(
            frame.frame_history.as_ref().unwrap().controller(),
            controller
        );
        assert!(frame.publications.events.starts_with(&prefix.events));
        assert_observation_eq!(frame.publications.events[prefix.events.len()..], events);
        let before = production.save().unwrap();
        assert_observation_eq!(
            production.advance(tick()),
            Err(ProductionFault::Unsupported(Unsupported::Frame(
                frame::Dependency::HazePreparation
            )))
        );
        assert_observation_eq!(production.save().unwrap(), before);
    }
    assert_observation_eq!(admitted.production.state(), restored.state());
}

fn complete_recurring_prelude(
    production: &mut Production,
    budget: data::timing::MasterTicks,
    now: data::timing::MasterTicks,
) -> Vec<Event> {
    let mut events = Vec::new();
    let mut acknowledged = false;
    for _ in 0..10_000 {
        match production.advance(AdvanceInput::Frame { budget }) {
            Ok(game::production::ProductionOutputs::Frame(output)) => {
                assert_observation_eq!(output.progress, frame::Progress::Advanced);
                events.extend(output.events);
                let OwnedState::Frame(frame) = production.state() else {
                    panic!("frame state")
                };
                if matches!(
                    frame.caller,
                    frame::Caller::Music {
                        prelude: frame::PreludeSource::Recurring,
                        ..
                    }
                ) {
                    assert_observation_eq!(
                        frame.timing.hardware.now.0.checked_sub(now.0),
                        Some(214)
                    );
                    acknowledged = true;
                }
            }
            Err(ProductionFault::Unsupported(Unsupported::Frame(
                frame::Dependency::HazePreparation,
            ))) => break,
            _ => panic!("unexpected next-opportunity result"),
        }
    }
    let OwnedState::Frame(frame) = production.state() else {
        panic!("frame state")
    };
    assert!(matches!(
        frame.caller,
        frame::Caller::Haze {
            prelude: frame::PreludeSource::Recurring,
            ..
        }
    ));
    assert!(acknowledged);
    assert_observation_eq!(frame.timing.hardware.now.0.checked_sub(now.0), Some(720));
    events
}
fn reject_inconsistent_return_history(original: &frame::State) {
    let native = |event: &game::production::timing::Event| {
        event.kind == EventKind::Interrupt(data::timing::Interrupt::Nmi)
    };
    let mut mismatched = original.clone();
    let event = mismatched
        .publications
        .events
        .iter_mut()
        .find(|event| native(event))
        .unwrap();
    event.kind = EventKind::ControllerCommitted;
    let reopened: frame::State = canonical::from_bytes(&canonical::to_bytes(&mismatched)).unwrap();
    assert!(reopened.validate().is_err());

    let mut absent_sender = original.clone();
    absent_sender.sender_history = None;
    let reopened: frame::State =
        canonical::from_bytes(&canonical::to_bytes(&absent_sender)).unwrap();
    assert!(reopened.validate().is_err());

    let mut wrong_prefix = original.clone();
    wrong_prefix.publications.events[0].kind = invalidating_wram_write();
    let reopened: frame::State =
        canonical::from_bytes(&canonical::to_bytes(&wrong_prefix)).unwrap();
    assert!(reopened.validate().is_err());
}

#[test]
#[ignore = "requires the original palette, Health, Camera and Drawing case"]
fn actual_palette_health_camera_drawing_from_one_origin() {
    let mut admitted = admitted();
    let InputReader::Frame(inputs) = &admitted.inputs else {
        panic!("frame inputs")
    };
    let budget = inputs.budget;
    for _ in 0..200_000 {
        let OwnedState::Frame(state) = admitted.production.state() else {
            panic!("frame state")
        };
        if matches!(
            state.late_movement,
            frame::late_movement::Stage::Complete(_)
        ) {
            break;
        }
        admitted
            .production
            .advance(AdvanceInput::Frame { budget })
            .unwrap();
    }
    let OwnedState::Frame(prepared) = admitted.production.state() else {
        panic!("frame state")
    };
    let initial_banks = prepared.samus_palette.as_ref().unwrap().palettes().unwrap();
    let ProductionContent::Frame(content) = &admitted.content else {
        panic!("frame content")
    };
    let ProductionRules::Frame(rules) = &admitted.rules else {
        panic!("frame rules")
    };
    let mut invalid = prepared.as_ref().clone();
    invalid.samus_palette.as_mut().unwrap().invalidate();
    let before = invalid.clone();
    assert_observation_eq!(
        invalid.advance(content, rules, budget),
        Err(ProductionFault::Unsupported(Unsupported::Frame(
            frame::Dependency::SamusPalettePreparation
        )))
    );
    assert!(invalid.eq(&before));
    let is_palette = |state: &frame::State| {
        state
            .samus_palette
            .as_ref()
            .and_then(frame::samus_palette::Palette::completed)
            .is_some()
    };
    restore_next(&mut admitted, budget).unwrap();
    advance_until(&mut admitted, budget, is_palette);
    let OwnedState::Frame(state) = admitted.production.state() else {
        panic!("frame state")
    };
    assert_observation_eq!(
        state.samus_palette.as_ref().unwrap().palettes().unwrap(),
        initial_banks
    );
    assert_observation_eq!(
        state.samus_palette.as_ref().unwrap().completed(),
        Some(samus_appearance::ColourSequences {
            visor_delay: samus_appearance::VisorDelay(1),
            visor_step: samus_appearance::VisorStep(3),
            charge_glow_position: samus_appearance::ChargeGlowPosition(0),
        })
    );
    exercise_drawing_and_native_preparation(&mut admitted, budget);
}

fn exercise_drawing_and_native_preparation(
    admitted: &mut AdmittedCase,
    budget: data::timing::MasterTicks,
) {
    restore_next(admitted, budget).unwrap();
    advance_until(admitted, budget, |state| {
        state.maintenance.energy().is_some()
    });
    restore_next(admitted, budget).unwrap();
    advance_until(admitted, budget, |state| {
        state.s6_camera.following().is_some()
    });
    restore_next(admitted, budget).unwrap();
    advance_until(admitted, budget, |state| {
        state.s6_drawing.builder().is_some()
    });
    let ProductionContent::Frame(content) = &admitted.content else {
        panic!("frame content")
    };
    if content.late_hud.is_none() {
        assert_observation_eq!(
            restore_next(admitted, budget),
            Err(ProductionFault::Unsupported(Unsupported::Frame(
                frame::Dependency::LateHudPreparation
            )))
        );
        return;
    }
    restore_next(admitted, budget).unwrap();
    advance_until(admitted, budget, |state| {
        matches!(
            state.late_hud,
            frame::late_hud::Stage::Completed(frame::suffix_caller::HudQueue::Queued(_))
        )
    });
    restore_next(admitted, budget).unwrap();
    advance_until(admitted, budget, |state| {
        matches!(
            state.background,
            frame::suffix_caller::BackgroundScroll::Complete(_)
        )
    });
    let ProductionContent::Frame(content) = &admitted.content else {
        panic!("frame content")
    };
    if content.room_tail.is_none() {
        assert_observation_eq!(
            restore_next(admitted, budget),
            Err(ProductionFault::Unsupported(Unsupported::Frame(
                frame::Dependency::RoomPreparation
            )))
        );
        return;
    }
    restore_next(admitted, budget).unwrap();
    advance_until(admitted, budget, |state| {
        matches!(state.room_tail, frame::room_tail::Stage::Completed(_))
    });
    let OwnedState::Frame(state) = admitted.production.state() else {
        panic!("frame state")
    };
    assert!(state.frame_history.as_ref().unwrap().drawn());
    assert_observation_eq!(
        restore_next(admitted, budget),
        Err(ProductionFault::Unsupported(Unsupported::Frame(
            frame::Dependency::PlayTimePreparation
        )))
    );
}

#[test]
#[ignore = "requires original room and qualified play-time opportunity"]
fn actual_play_time_returns_once_and_survives_checkpoint() {
    use super::super::frame_history_inputs::play_time;
    use data::timing::MasterTicks;
    use frame::{play_time::Stage, room_tail};

    let mut admitted = admitted();
    let InputReader::Frame(inputs) = &admitted.inputs else {
        panic!("frame inputs")
    };
    let budget = inputs.budget;
    advance_until(&mut admitted, budget, |state| {
        matches!(state.room_tail, room_tail::Stage::Completed(_))
    });
    let OwnedState::Frame(state) = admitted.production.state() else {
        panic!("frame state")
    };
    let room = state.as_ref().clone();
    let original = room.play_time.current().unwrap();

    restore_next(&mut admitted, MasterTicks(1)).unwrap();
    let OwnedState::Frame(state) = admitted.production.state() else {
        panic!("frame state")
    };
    assert!(state.play_time.is_charging());
    assert_observation_eq!(state.play_time.current(), Some(original));
    restore_next(&mut admitted, MasterTicks(1)).unwrap();
    advance_until(&mut admitted, budget, |state| state.play_time.is_complete());
    let OwnedState::Frame(state) = admitted.production.state() else {
        panic!("frame state")
    };
    let retained = state.play_time.current().unwrap();
    assert!(
        retained
            .frames
            .get()
            .eq(&original.frames.get().checked_add(1).unwrap())
    );
    assert!((retained.hours, retained.minutes, retained.seconds).eq(&(
        original.hours,
        original.minutes,
        original.seconds
    )));
    assert!(state.health.eq(&room.health));
    assert_observation_eq!(
        restore_next(&mut admitted, budget),
        Err(ProductionFault::Unsupported(Unsupported::Frame(
            frame::Dependency::RoomShakePreparation
        )))
    );

    let ProductionContent::Frame(content) = &admitted.content else {
        panic!("frame content")
    };
    let ProductionRules::Frame(rules) = &admitted.rules else {
        panic!("frame rules")
    };
    check_play_time_counters(&room, content, rules, budget);
    reject_play_time_activity_and_forged_return(&room, content, rules, budget);

    let mut ceiling = room.clone();
    ceiling.play_time = Stage::from_origin(play_time([99, 59, 59, 59]).unwrap());
    let mut invalid_health = room.clone();
    invalid_health.health.as_mut().unwrap().invalidate();
    let mut missing = room;
    missing.play_time = Stage::default();
    for mut rejected in [
        Box::new(ceiling),
        Box::new(invalid_health),
        Box::new(missing),
    ] {
        let before = canonical::to_bytes(&rejected);
        assert!(matches!(
            rejected.advance(content, rules, budget),
            Err(ProductionFault::Unsupported(Unsupported::Frame(
                frame::Dependency::PlayTimePreparation
            )))
        ));
        assert_observation_eq!(canonical::to_bytes(&rejected), before);
    }
}

fn reject_play_time_activity_and_forged_return(
    room: &frame::State,
    content: &frame::Content,
    rules: &frame::Rules,
    budget: data::timing::MasterTicks,
) {
    for kind in [
        EventKind::Interrupt(data::timing::Interrupt::Nmi),
        invalidating_wram_write(),
    ] {
        let mut invalidated = room.clone();
        invalidated
            .publications
            .record(vec![Event {
                at: room.timing.hardware.now,
                kind,
            }])
            .unwrap();
        let before = canonical::to_bytes(&invalidated);
        assert!(matches!(
            invalidated.advance(content, rules, budget),
            Err(ProductionFault::Unsupported(Unsupported::Frame(
                frame::Dependency::CountdownSceneScheduling
            )))
        ));
        assert_observation_eq!(canonical::to_bytes(&invalidated), before);
    }
    let mut forged = room.clone();
    forged.frame_tail = frame::frame_tail::Stage::Complete;
    let before = canonical::to_bytes(&forged);
    let reopened: frame::State = canonical::from_bytes(&before).unwrap();
    assert!(reopened.validate().is_err());
    assert!(matches!(
        forged.advance(content, rules, budget),
        Err(ProductionFault::Unsupported(Unsupported::Frame(
            frame::Dependency::NativeReturnPreparation
        )))
    ));
    assert_observation_eq!(canonical::to_bytes(&forged), before);
}

fn check_play_time_counters(
    room: &frame::State,
    content: &frame::Content,
    rules: &frame::Rules,
    budget: data::timing::MasterTicks,
) {
    use super::super::frame_history_inputs::play_time;
    use data::timing::MasterTicks;
    use frame::play_time::Stage;

    for (before, after) in [
        ([2, 3, 4, 5], [2, 3, 4, 6]),
        ([99, 59, 59, 58], [99, 59, 59, 59]),
    ] {
        let current = play_time(before).unwrap();
        let expected = play_time(after).unwrap();
        let mut candidate = room.clone();
        candidate.play_time = Stage::from_origin(current);
        candidate.advance(content, rules, MasterTicks(1)).unwrap();
        assert!(candidate.play_time.is_charging());
        assert_observation_eq!(candidate.play_time.current(), Some(current));
        let mut revived: frame::State =
            canonical::from_bytes(&canonical::to_bytes(&candidate)).unwrap();
        revived.validate().unwrap();
        for _ in 0..500 {
            assert!(
                candidate
                    .advance(content, rules, budget)
                    .eq(&revived.advance(content, rules, budget))
            );
            assert!(candidate.eq(&revived));
            if candidate.play_time.is_complete() {
                break;
            }
        }
        assert!(candidate.play_time.is_complete());
        assert_observation_eq!(candidate.play_time.current(), Some(expected));
        assert!(candidate.health.eq(&room.health));
        let returned = canonical::to_bytes(&candidate);
        assert!(matches!(
            candidate.advance(content, rules, budget),
            Err(ProductionFault::Unsupported(Unsupported::Frame(
                frame::Dependency::RoomShakePreparation
            )))
        ));
        assert_observation_eq!(canonical::to_bytes(&candidate), returned);
    }
}

#[test]
#[ignore = "requires original clock and quiet room-quake opportunities"]
fn actual_quake_returns_once_after_clock_and_survives_checkpoint() {
    use data::timing::MasterTicks;
    use frame::room_quake::Stage;
    use world_quake::{QuakeDuration, QuakePhase, State as Quake};

    let mut admitted = admitted();
    let InputReader::Frame(inputs) = &admitted.inputs else {
        panic!("frame inputs")
    };
    let budget = inputs.budget;
    advance_until(&mut admitted, budget, |state| state.play_time.is_complete());
    let OwnedState::Frame(state) = admitted.production.state() else {
        panic!("frame state")
    };
    let clock_return = state.as_ref().clone();
    let original = clock_return.room_quake.current().unwrap();
    assert!(!clock_return.room_quake.is_charging());

    restore_next(&mut admitted, MasterTicks(1)).unwrap();
    let OwnedState::Frame(state) = admitted.production.state() else {
        panic!("frame state")
    };
    assert!(state.room_quake.is_charging());
    assert_observation_eq!(state.room_quake.current(), Some(original));
    restore_next(&mut admitted, MasterTicks(1)).unwrap();
    advance_until(&mut admitted, budget, |state| {
        state.room_quake.is_complete()
    });
    let OwnedState::Frame(state) = admitted.production.state() else {
        panic!("frame state")
    };
    let retained = state.room_quake.current().unwrap();
    assert_observation_eq!(retained.phase.0, original.phase.0.wrapping_add(1));
    assert_observation_eq!(retained.remaining, original.remaining);
    assert_observation_eq!(state.play_time, clock_return.play_time);
    restore_next(&mut admitted, MasterTicks(1)).unwrap();
    let OwnedState::Frame(state) = admitted.production.state() else {
        panic!("frame state")
    };
    assert_observation_eq!(state.room_quake.current(), Some(retained));

    let ProductionContent::Frame(content) = &admitted.content else {
        panic!("frame content")
    };
    let ProductionRules::Frame(rules) = &admitted.rules else {
        panic!("frame rules")
    };
    let mut wrap = clock_return.clone();
    wrap.room_quake = Stage::from_origin(Quake {
        remaining: QuakeDuration(0),
        phase: QuakePhase(u16::MAX),
    });
    wrap.advance(content, rules, MasterTicks(1)).unwrap();
    assert!(wrap.room_quake.is_charging());
    let mut revived: frame::State = canonical::from_bytes(&canonical::to_bytes(&wrap)).unwrap();
    revived.validate().unwrap();
    for _ in 0..500 {
        assert!(
            wrap.advance(content, rules, budget)
                .eq(&revived.advance(content, rules, budget))
        );
        assert!(wrap.eq(&revived));
        if wrap.room_quake.is_complete() {
            break;
        }
    }
    assert!(wrap.room_quake.is_complete());
    assert_observation_eq!(
        wrap.room_quake.current(),
        Some(Quake {
            remaining: QuakeDuration(0),
            phase: QuakePhase(0),
        })
    );

    let mut active = clock_return.clone();
    active.room_quake = Stage::from_origin(Quake {
        remaining: QuakeDuration(1),
        phase: QuakePhase(u16::MAX),
    });
    let mut missing = clock_return;
    missing.room_quake = Stage::default();
    for mut rejected in [Box::new(active), Box::new(missing)] {
        let before = canonical::to_bytes(&rejected);
        assert!(matches!(
            rejected.advance(content, rules, budget),
            Err(ProductionFault::Unsupported(Unsupported::Frame(
                frame::Dependency::RoomShakePreparation
            )))
        ));
        assert_observation_eq!(canonical::to_bytes(&rejected), before);
    }
}

fn invalidating_wram_write() -> game::production::timing::EventKind {
    use data::timing::{DeviceWrite, Publication, WRAM_DATA_PORT};
    // Inject an invalidating history receipt without asking timing to execute the unadmitted write.
    game::production::timing::EventKind::Publication(DeviceWrite {
        port: WRAM_DATA_PORT,
        value: 0,
        effect: Publication::Downstream,
    })
}
