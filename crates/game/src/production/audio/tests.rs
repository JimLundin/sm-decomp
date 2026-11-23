use crate::production::tests::fixtures;
use crate::production::{
    AdmissionFault, AdvanceInput, Production, ProductionContent, ProductionOutputs,
    ProductionRules, ProductionSeed,
};
use crate::state::owned::OwnedState;
use audio_transport::{CueId, OfferPermission, QueueAllowance, Request, SoundGroup, State};

fn admit(state: State) -> Result<Production, AdmissionFault> {
    Production::admit(
        ProductionSeed::Audio(Box::new(state)),
        ProductionContent::Audio(Box::new(fixtures::audio_content())),
        ProductionRules::Audio,
    )
}

fn transport(production: &Production) -> &State {
    let OwnedState::Audio(state) = production.state() else {
        panic!("audio scope");
    };
    &state.0
}

fn offer(cue: CueId) -> audio_transport::Offer {
    audio_transport::Offer {
        cue,
        allowance: QueueAllowance::Six,
        permission: OfferPermission::Enabled,
    }
}

#[test]
fn a_frame_offers_then_serves_and_the_service_presents_the_offered_cue() {
    let mut production = admit(fixtures::transport()).unwrap();
    assert_eq!(
        production.advance(AdvanceInput::Audio {
            offers: &[offer(CueId::Land)],
            cadence: fixtures::busy_cadence(),
        }),
        Ok(ProductionOutputs::Audio)
    );
    let state = transport(&production);
    assert_eq!(
        state.request_lines[SoundGroup::Contact],
        Request::Cue(CueId::Land)
    );
    assert!(state.senders[SoundGroup::Contact].queue.is_empty());
}

#[test]
fn a_frame_without_turns_or_service_only_queues_its_offers() {
    let mut production = admit(fixtures::transport()).unwrap();
    production
        .advance(AdvanceInput::Audio {
            offers: &[offer(CueId::Land), offer(CueId::Door)],
            cadence: fixtures::silent_cadence(),
        })
        .unwrap();
    let state = transport(&production);
    assert_eq!(
        state.senders[SoundGroup::Contact]
            .queue
            .cues()
            .iter()
            .copied()
            .collect::<Vec<_>>(),
        [CueId::Land, CueId::Door]
    );
    assert_eq!(state.request_lines[SoundGroup::Contact], Request::Absent);
}

#[test]
fn a_restored_transport_plays_its_next_frame_as_the_carried_one() {
    let mut carried = admit(fixtures::transport()).unwrap();
    carried
        .advance(AdvanceInput::Audio {
            offers: &[offer(CueId::Beam)],
            cadence: fixtures::busy_cadence(),
        })
        .unwrap();
    let saved = carried.save().unwrap();
    let mut restored = Production::restore(
        &saved,
        ProductionContent::Audio(Box::new(fixtures::audio_content())),
        ProductionRules::Audio,
    )
    .unwrap();
    assert_eq!(restored.state(), carried.state());
    for production in [&mut carried, &mut restored] {
        production
            .advance(AdvanceInput::Audio {
                offers: &[],
                cadence: fixtures::busy_cadence(),
            })
            .unwrap();
    }
    assert_eq!(restored.state(), carried.state());
    assert_eq!(restored.save().unwrap(), carried.save().unwrap());
}

#[test]
fn admission_refuses_a_queued_cue_of_another_group() {
    let mut state = fixtures::transport();
    state.senders[SoundGroup::Action].queue.push(CueId::Land);
    assert!(matches!(admit(state), Err(AdmissionFault::Audio(_))));
}
