use super::runtime_helpers::{admitted, advance_until, restore_next};
use super::*;
use frame::countdown_scene::{Choice, Current};
use game::{
    production::{AdvanceInput, ProductionFault, Unsupported},
    state::owned::OwnedState,
};

#[test]
#[ignore = "requires reviewed scene projection, explicitly borrowed case and player ROM"]
fn actual_scene_owner_offer_is_current_distinct_and_atomic_at_timer() {
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
    let OwnedState::Frame(actual) = admitted.production.state() else {
        panic!("frame state")
    };
    let ProductionContent::Frame(content) = &admitted.content else {
        panic!("frame content")
    };
    let ProductionRules::Frame(rules) = &admitted.rules else {
        panic!("frame rules")
    };
    let before = actual.as_ref().clone();
    assert_observation_eq!(
        actual
            .countdown_scene_offer(content, rules)
            .unwrap()
            .choice(),
        Choice::MaintainCountdown
    );
    assert!(actual.as_ref().eq(&before));
    reject_countdown_scene_writes(&before, content, rules, budget);
    let mut unrelated = before.clone();
    unrelated.scene = Some(frame::borrowed_gameplay::SceneController::Other);
    assert_observation_eq!(
        unrelated
            .countdown_scene_offer(content, rules)
            .unwrap()
            .choice(),
        Choice::MaintainCountdown
    );
    let next = restore_next(&mut admitted, budget);
    assert!(matches!(
        next,
        Err(ProductionFault::Unsupported(Unsupported::Frame(
            frame::Dependency::CountdownPreparation
        )))
    ));
    assert!(
        admitted
            .production
            .state()
            .eq(&OwnedState::Frame(Box::new(before)))
    );
    let again = admitted.production.advance(AdvanceInput::Frame { budget });
    assert!(matches!(
        again,
        Err(ProductionFault::Unsupported(Unsupported::Frame(
            frame::Dependency::CountdownPreparation
        )))
    ));
}

fn reject_countdown_scene_writes(
    before: &frame::State,
    content: &frame::Content,
    rules: &frame::Rules,
    budget: data::timing::MasterTicks,
) {
    for mutation in 0..5 {
        let mut changed = before.clone();
        match mutation {
            0 => changed.scene_countdown = None,
            1 => changed
                .scene_countdown
                .as_mut()
                .unwrap()
                .0
                .write(Choice::MaintainCountdown),
            2 => changed
                .scene_countdown
                .as_mut()
                .unwrap()
                .0
                .write(Choice::PresentCountdownOnly),
            3 => changed.scene_countdown.as_mut().unwrap().0.invalidate(),
            4 => {
                changed.scene_countdown =
                    Some(Model(Current::from_origin(Choice::PresentCountdownOnly)));
            }
            _ => panic!("mutation"),
        }
        let saved = changed.clone();
        assert!(
            matches!(
                changed.advance(content, rules, budget),
                Err(ProductionFault::Unsupported(Unsupported::Frame(
                    frame::Dependency::CountdownSceneScheduling
                )))
            ),
            "scene mutation {mutation}"
        );
        assert!(changed.eq(&saved));
    }
}

#[test]
#[ignore = "requires explicitly borrowed scene case"]
fn missing_scene_borrowed_declaration_does_not_admit_source_member() {
    let path = std::env::var_os("SM_FRAME_DIAGNOSTIC_CASE").unwrap();
    let mut case: Case = toml::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    case.borrowed
        .retain(|value| value.dependency != "frame.scene-countdown-scheduling");
    let sources = Sources {
        rom: std::env::var_os("SM_ROM").unwrap().into(),
        ..Sources::default()
    };
    assert!(admit(&case, &sources).is_err());
}
