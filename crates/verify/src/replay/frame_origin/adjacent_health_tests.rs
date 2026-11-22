use super::*;
use canonical::model::Model;
use game::production::frame::health::HealthPin;

pub(super) fn check_animation_energy(admitted: &AdmittedCase, direct: &Production) {
    let ProductionContent::Frame(content) = &admitted.content else {
        panic!("frame content")
    };
    let first = current(&admitted.production);
    let maintained = first.health.as_ref().unwrap();
    let restored = restore(admitted, direct);
    for production in [direct, &restored] {
        let state = current(production);
        let health = state.health.as_ref().unwrap();
        assert_observation_eq!(health.state(), maintained.state());
        assert_observation_eq!(
            state
                .fresh_animation_inputs(content)
                .map(|inputs| inputs.energy),
            Ok(maintained.state().energy.whole.get())
        );
    }
    check_carried_amount(admitted, &restored);
    let state = current(&restored);
    let mut stale_actor = state.clone();
    let Caller::PlayerContinuation { remaining, .. } = &mut stale_actor.caller else {
        panic!("adjacent player continuation")
    };
    remaining.0.actors = maintained.actor().cloned();
    let mut written = state.clone();
    let health = written.health.as_mut().unwrap();
    health.write(*health.state());
    let mut no_context = content.as_ref().clone();
    no_context.adjacent_health_context = None;
    let mut rearmed = state.clone();
    rearmed.drawing.as_mut().unwrap().recurring_due = Some(DrawingRequest::End);
    for (state, content) in [
        (&stale_actor, content.as_ref()),
        (&written, content),
        (&rearmed, content),
        (state, &no_context),
    ] {
        assert!(matches!(
            state.fresh_animation_inputs(content),
            Err(ProductionFault::Unsupported(Unsupported::Frame(
                Dependency::HealthCurrentInputs
            )))
        ));
    }
}

pub(super) fn first_animation_bill(
    first: &State,
    content: &frame::Content,
) -> data::timing::Profile {
    let frame::animation_caller::Stage::Complete(done) = &first.animation else {
        panic!("completed first Animation")
    };
    let mut history = first.clone();
    history.appearance = done.receipt.chosen.clone();
    history.playing_figure = done.receipt.playing;
    content
        .animation
        .as_ref()
        .unwrap()
        .qualify(
            &frame::appearance_caller::Stage::Offered(done.receipt.clone()),
            &history,
            &content.movement,
            &content.appearance,
        )
        .unwrap()
}

fn check_carried_amount(admitted: &AdmittedCase, production: &Production) {
    let state = current(production);
    let carried = *state.health.as_ref().unwrap().state();
    // A different origin can produce the same carried state after the first loss.
    // The checkpoint must still name the origin that actually produced it.
    let mut changed_origin = admitted.content.clone();
    let ProductionContent::Frame(content) = &mut changed_origin else {
        panic!("frame content");
    };
    let origin = &mut content.health.as_mut().unwrap().initial.0;
    let one = samus_health::WholeEnergy::new(1).unwrap();
    origin.energy.whole =
        samus_health::WholeEnergy::new(carried.energy.whole.get().checked_add(1).unwrap()).unwrap();
    origin.pending_loss.whole = one;
    let mut maintained = *origin;
    assert_eq!(
        maintained.health_opportunity(&samus_health::Rules::REFERENCE),
        samus_health::WarningSound::None
    );
    assert_observation_eq!(maintained, carried);
    assert!(matches!(
        Production::admit(
            ProductionSeed::Frame(Box::new(state.clone())),
            changed_origin,
            admitted.rules.clone(),
        ),
        Err(game::production::AdmissionFault::Frame(fault))
            if *fault == frame::Fault::InvalidPlayerEntry
    ));

    // Change the saved quantity without calling a writer that would itself mark
    // the owner stale. Admission must check the entire first Maintenance result.
    let mut amounts = [carried; 5];
    let [whole, fraction, pending_whole, pending_fraction, warning] = &mut amounts;
    whole.energy.whole = one;
    fraction.energy.fraction ^= 1;
    pending_whole.pending_loss.whole = one;
    pending_fraction.pending_loss.fraction ^= 1;
    warning.warning = samus_health::Warning::On;
    for altered in amounts {
        assert!(super::super::rejects_changed_frame(
            admitted,
            production,
            |frame| {
                let health = frame.health.as_mut().unwrap();
                let original = canonical::to_bytes(health);
                let canonical_state = canonical::to_bytes(&Model::<HealthPin>(carried));
                let rest = original.strip_prefix(canonical_state.as_slice()).unwrap();
                let mut changed = canonical::to_bytes(&Model::<HealthPin>(altered));
                changed.extend(rest);
                *health = canonical::from_bytes(&changed).unwrap();
                assert_observation_eq!(health.state(), &altered);
            }
        ));
    }
}
