use super::runtime_helpers::{admitted, advance_until, restore_next};
use super::*;
use data::timing::MasterTicks;
use frame::countdown_caller::Stage;
use game::{
    production::{AdvanceInput, ProductionFault, Unsupported},
    state::owned::OwnedState,
};
use interface_hud::Observation;

#[test]
fn countdown_delivery_shape_remains_pinned() {
    assert_observation_eq!(
        canonical::model::pin_holds::<frame::countdown_caller::AnswerPin>(),
        Ok(())
    );
}

#[test]
#[ignore = "requires independently qualified countdown origin and whole Timer source"]
fn actual_countdown_return_restores_and_consumes_once_against_retained_receiver() {
    let mut admitted = admitted();
    let InputReader::Frame(inputs) = &admitted.inputs else {
        panic!("frame inputs")
    };
    let budget = inputs.budget;
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
    let OwnedState::Frame(before) = admitted.production.state() else {
        panic!("frame state")
    };
    let before = before.as_ref().clone();
    assert_observation_eq!(before.countdown_delivered.len(), 0);
    assert!(before.hud.countdown_answer().is_none());
    restore_next(&mut admitted, budget).unwrap();
    let OwnedState::Frame(chosen) = admitted.production.state() else {
        panic!("frame state")
    };
    assert!(matches!(chosen.countdown, Stage::Charging { .. }));
    assert!(matches!(
        chosen.hud.countdown_answer(),
        Some(Observation::Chosen(_))
    ));
    assert_observation_eq!(chosen.countdown_delivered.len(), 0);
    assert!(chosen.hud.state().eq(before.hud.state()));
    let chosen = chosen.as_ref().clone();
    let ProductionContent::Frame(content) = &admitted.content else {
        panic!("frame content")
    };
    let ProductionRules::Frame(rules) = &admitted.rules else {
        panic!("frame rules")
    };
    reject_countdown_origin_changes(&before, &chosen, content, rules, budget);
    // Compare every actual in-flight next step from a fresh complete checkpoint.
    for _ in 0..100_000 {
        let OwnedState::Frame(current) = admitted.production.state() else {
            panic!("frame state")
        };
        if matches!(current.countdown, Stage::Complete { .. }) {
            break;
        }
        restore_next(&mut admitted, budget).unwrap();
    }
    let OwnedState::Frame(completed) = admitted.production.state() else {
        panic!("frame state")
    };
    assert!(matches!(completed.countdown, Stage::Complete { .. }));
    // Observe actual completed progress before any subsequent completion attempt.
    assert!(matches!(
        completed.hud.countdown_answer(),
        Some(Observation::Completed(_))
    ));
    assert!(completed.hud.state().eq(before.hud.state()));
    assert_observation_eq!(completed.countdown_delivered.len(), 1);
    let completed = completed.as_ref().clone();
    // Receiver evidence survives independently of every damaged producer checkpoint below.
    let observed_receiver_prefix = completed.countdown_delivered.clone();
    let ProductionContent::Frame(content) = &admitted.content else {
        panic!("frame content")
    };
    let ProductionRules::Frame(rules) = &admitted.rules else {
        panic!("frame rules")
    };
    reject_countdown_producer_changes(
        &completed,
        &chosen,
        &observed_receiver_prefix,
        content,
        rules,
        budget,
    );
    compare_countdown_after_return(&mut admitted, &completed, &observed_receiver_prefix, budget);
}

fn compare_countdown_after_return(
    admitted: &mut AdmittedCase,
    completed: &frame::State,
    observed_receiver_prefix: &[Model<frame::countdown_caller::AnswerPin>],
    budget: MasterTicks,
) {
    let saved = admitted.production.save().unwrap();
    let mut fresh =
        Production::restore(&saved, admitted.content.clone(), admitted.rules.clone()).unwrap();
    let OwnedState::Frame(restored) = fresh.state() else {
        panic!("frame state")
    };
    assert!(matches!(
        restored.hud.countdown_answer(),
        Some(Observation::Completed(_))
    ));
    assert!(
        restored
            .countdown_delivered
            .as_slice()
            .eq(observed_receiver_prefix)
    );
    assert!(restored.as_ref().eq(completed));
    let next = fresh.advance(AdvanceInput::Frame { budget });
    let original_next = admitted.production.advance(AdvanceInput::Frame { budget });
    assert!(next.eq(&original_next));
    assert!(fresh.state().eq(admitted.production.state()));
    assert!(matches!(
        next,
        Err(ProductionFault::Unsupported(Unsupported::Frame(
            frame::Dependency::AnimationPreparation
        )))
    ));
    let again = admitted.production.advance(AdvanceInput::Frame { budget });
    assert!(matches!(
        again,
        Err(ProductionFault::Unsupported(Unsupported::Frame(
            frame::Dependency::AnimationPreparation
        )))
    ));
    let OwnedState::Frame(after) = admitted.production.state() else {
        panic!("frame state")
    };
    assert!(after.as_ref().eq(completed));
    assert!(
        after
            .countdown_delivered
            .as_slice()
            .eq(observed_receiver_prefix)
    );
    assert!(matches!(
        after.hud.countdown_answer(),
        Some(Observation::Completed(_))
    ));
}

fn reject_countdown_producer_changes(
    completed: &frame::State,
    chosen: &frame::State,
    observed_receiver_prefix: &[Model<frame::countdown_caller::AnswerPin>],
    content: &frame::Content,
    rules: &frame::Rules,
    budget: MasterTicks,
) {
    for mutation in 0..4 {
        let mut changed = completed.clone();
        match mutation {
            0 => changed.hud = chosen.hud.clone(),
            1 => changed.countdown = chosen.countdown.clone(),
            2 => changed.countdown_delivered.clear(),
            3 => changed
                .countdown_delivered
                .extend(observed_receiver_prefix.iter().copied()),
            _ => panic!("mutation"),
        }
        let bytes = canonical::to_bytes(&changed);
        drop(changed);
        let mut restored: frame::State = canonical::from_bytes(&bytes).unwrap();
        let retained = restored.clone();
        assert!(restored.validate().is_err());
        assert!(restored.advance(content, rules, budget).is_err());
        assert!(restored.eq(&retained));
        assert_observation_eq!(observed_receiver_prefix.len(), 1);
        if mutation == 0 {
            assert!(
                restored
                    .countdown_delivered
                    .as_slice()
                    .eq(observed_receiver_prefix)
            );
            assert!(matches!(
                restored.hud.countdown_answer(),
                Some(Observation::Chosen(_))
            ));
        }
    }
}

fn reject_countdown_origin_changes(
    before: &frame::State,
    chosen: &frame::State,
    content: &frame::Content,
    rules: &frame::Rules,
    budget: MasterTicks,
) {
    for checkpoint in [before, chosen] {
        for mutation in 0..4 {
            let mut changed = checkpoint.clone();
            match mutation {
                0 => changed.scene_countdown = None,
                1 => changed
                    .scene_countdown
                    .as_mut()
                    .unwrap()
                    .0
                    .write(frame::countdown_scene::Choice::MaintainCountdown),
                2 => changed.scene_countdown.as_mut().unwrap().0.invalidate(),
                3 => {
                    let mut incompatible = content.clone();
                    incompatible
                        .countdown
                        .as_mut()
                        .unwrap()
                        .template
                        .runs
                        .clear();
                    let saved = changed.clone();
                    assert!(changed.advance(&incompatible, rules, budget).is_err());
                    assert!(changed.eq(&saved));
                    continue;
                }
                _ => panic!("mutation"),
            }
            let saved = changed.clone();
            assert!(changed.advance(content, rules, budget).is_err());
            assert!(changed.eq(&saved));
        }
    }
}

#[test]
#[ignore = "requires qualified complete origin and countdown projection"]
fn current_countdown_decoder_rejects_missing_active_and_unknown_input() {
    let original = std::fs::read(std::env::var_os("SM_FRAME_CURRENT_ORIGIN").unwrap()).unwrap();
    let projection: toml::Value = toml::from_str(
        &std::fs::read_to_string(std::env::var_os("SM_COUNTDOWN_PROJECTION").unwrap()).unwrap(),
    )
    .unwrap();
    let at = usize::try_from(
        projection
            .get("private_wram_offset")
            .unwrap()
            .as_integer()
            .unwrap(),
    )
    .unwrap();
    assert_observation_eq!(
        super::super::hud::countdown_origin(&original),
        Ok(interface_hud::Countdown::Dormant)
    );
    assert!(super::super::hud::countdown_origin(&[]).is_err());
    for value in [1_u16, 2, u16::MAX] {
        let mut changed = original.clone();
        changed
            .get_mut(at..at.saturating_add(2))
            .unwrap()
            .copy_from_slice(&value.to_le_bytes());
        assert!(super::super::hud::countdown_origin(&changed).is_err());
    }
}
