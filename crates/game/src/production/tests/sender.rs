use super::super::*;
use crate::state::owned::OwnedState;
use audio_transport::sender::{ReplyInput, SenderOutput, SenderProfile};
use audio_transport::{
    CpuMasterTicks, CueId, OfferPermission, QueueAllowance, Request, SenderProgress, SenderState,
    SoundGroup,
};

fn sender() -> (Production, ProductionContent, ProductionRules) {
    let state = SenderState {
        queue: std::collections::VecDeque::new(),
        progress: SenderProgress::Ready,
        reply: Request::Absent,
        now: CpuMasterTicks(0),
        pending: None,
    };
    let content = ProductionContent::AudioSender(SoundGroup::Action);
    let rules = ProductionRules::AudioSender;
    let production = Production::admit(
        ProductionSeed::AudioSender(Box::new(state)),
        content.clone(),
        rules.clone(),
    )
    .unwrap();
    (production, content, rules)
}

#[test]
fn sender_pending_service_carries_and_restores_each_publication_once() {
    let (mut production, content, rules) = sender();
    for cue in [CueId::Beam, CueId::Spin] {
        assert_eq!(
            production.advance(AdvanceInput::AudioSender(SenderInput::Offer {
                cue,
                allowance: QueueAllowance::Three,
                permission: OfferPermission::Enabled
            })),
            Ok(ProductionOutputs::AudioSender(Vec::new()))
        );
    }
    production
        .advance(AdvanceInput::AudioSender(SenderInput::Begin(
            SenderProfile::FastBus,
        )))
        .unwrap();
    assert_eq!(
        production.advance(AdvanceInput::AudioSender(SenderInput::Advance {
            stop: CpuMasterTicks(211),
            replies: &[]
        })),
        Ok(ProductionOutputs::AudioSender(Vec::new()))
    );
    let saved = production.save().unwrap();
    let OwnedState::AudioSender(state) = production.state() else {
        panic!("sender scope")
    };
    assert_eq!(
        state.0.queue.iter().copied().collect::<Vec<_>>(),
        [CueId::Beam, CueId::Spin]
    );
    assert_eq!(state.0.progress, SenderProgress::Ready);
    assert!(state.0.pending.is_some());
    assert_eq!(
        production.advance(AdvanceInput::AudioSender(SenderInput::Offer {
            cue: CueId::Beam,
            allowance: QueueAllowance::Fifteen,
            permission: OfferPermission::Enabled
        })),
        Err(ProductionFault::Unsupported(Unsupported::SenderService))
    );
    assert_eq!(production.save().unwrap(), saved);
    let mut restored = Production::restore(&saved, content, rules).unwrap();
    let publication = ProductionOutputs::AudioSender(vec![SenderOutput::Presentation(
        CpuMasterTicks(212),
        Request::Cue(CueId::Beam),
    )]);
    for engine in [&mut production, &mut restored] {
        assert_eq!(
            engine.advance(AdvanceInput::AudioSender(SenderInput::Advance {
                stop: CpuMasterTicks(212),
                replies: &[]
            })),
            Ok(publication.clone())
        );
        assert_eq!(
            engine.advance(AdvanceInput::AudioSender(SenderInput::Advance {
                stop: CpuMasterTicks(212),
                replies: &[]
            })),
            Ok(ProductionOutputs::AudioSender(Vec::new()))
        );
        assert_eq!(
            engine.advance(AdvanceInput::AudioSender(SenderInput::Advance {
                stop: CpuMasterTicks(398),
                replies: &[]
            })),
            Ok(ProductionOutputs::AudioSender(vec![
                SenderOutput::Completed(CpuMasterTicks(398))
            ]))
        );
    }
    assert_eq!(production.state(), restored.state());
    let OwnedState::AudioSender(state) = production.state() else {
        panic!("sender scope")
    };
    assert_eq!(
        state.0.queue.iter().copied().collect::<Vec<_>>(),
        [CueId::Spin]
    );
    assert_eq!(state.0.progress, SenderProgress::AwaitCue(CueId::Beam));
}

#[test]
fn sender_reply_is_sampled_at_service_milestone_and_pending_result_restores() {
    let state = SenderState {
        queue: std::collections::VecDeque::from([CueId::Spin]),
        progress: SenderProgress::AwaitCue(CueId::Beam),
        reply: Request::Absent,
        now: CpuMasterTicks(398),
        pending: None,
    };
    let content = ProductionContent::AudioSender(SoundGroup::Action);
    let rules = ProductionRules::AudioSender;
    let mut production = Production::admit(
        ProductionSeed::AudioSender(Box::new(state)),
        content.clone(),
        rules.clone(),
    )
    .unwrap();
    production
        .advance(AdvanceInput::AudioSender(SenderInput::Begin(
            SenderProfile::FastBus,
        )))
        .unwrap();
    assert_eq!(
        production.advance(AdvanceInput::AudioSender(SenderInput::Advance {
            stop: CpuMasterTicks(456),
            replies: &[ReplyInput {
                at: CpuMasterTicks(455),
                reply: Request::Cue(CueId::Beam)
            }]
        })),
        Ok(ProductionOutputs::AudioSender(vec![
            SenderOutput::Observation(CpuMasterTicks(456), Request::Cue(CueId::Beam))
        ]))
    );
    let pending_ack = production.save().unwrap();
    let mut restored_ack = Production::restore(&pending_ack, content, rules).unwrap();
    let completed = production
        .advance(AdvanceInput::AudioSender(SenderInput::Advance {
            stop: CpuMasterTicks(608),
            replies: &[],
        }))
        .unwrap();
    assert_eq!(
        restored_ack.advance(AdvanceInput::AudioSender(SenderInput::Advance {
            stop: CpuMasterTicks(608),
            replies: &[]
        })),
        Ok(completed)
    );
    assert_eq!(production.state(), restored_ack.state());
    let OwnedState::AudioSender(state) = production.state() else {
        panic!("sender scope")
    };
    assert!(matches!(state.0.progress, SenderProgress::ClearDelay(_)));
}

#[test]
fn sender_rejects_other_group_and_invalid_intervals_without_consuming_service() {
    let (mut production, _, _) = sender();
    production
        .advance(AdvanceInput::AudioSender(SenderInput::Begin(
            SenderProfile::FastBus,
        )))
        .unwrap();
    production
        .advance(AdvanceInput::AudioSender(SenderInput::Advance {
            stop: CpuMasterTicks(20),
            replies: &[],
        }))
        .unwrap();
    let saved = production.save().unwrap();
    assert_eq!(
        production.advance(AdvanceInput::AudioSender(SenderInput::Advance {
            stop: CpuMasterTicks(19),
            replies: &[]
        })),
        Err(ProductionFault::TimeReversed)
    );
    assert_eq!(
        production.advance(AdvanceInput::AudioSender(SenderInput::Advance {
            stop: CpuMasterTicks(21),
            replies: &[ReplyInput {
                at: CpuMasterTicks(22),
                reply: Request::Absent
            }]
        })),
        Err(ProductionFault::PresentationOutsideInterval)
    );
    assert_eq!(production.save().unwrap(), saved);
    let (mut offers, _, _) = sender();
    assert_eq!(
        offers.advance(AdvanceInput::AudioSender(SenderInput::Offer {
            cue: CueId::Door,
            allowance: QueueAllowance::One,
            permission: OfferPermission::Enabled
        })),
        Err(ProductionFault::Unsupported(Unsupported::SenderCueGroup))
    );
}
