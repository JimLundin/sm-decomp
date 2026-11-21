use super::{ClockRelation, Content, Fault, Origin, State};
use crate::production::tests::fixtures::{audio_content, pending_audio};
use crate::production::{
    AdmissionFault, Checkpoint, Production, ProductionContent, ProductionRules, ProductionSeed,
    Senders,
};
use audio_transport::{
    AudioPhase, CpuMasterTicks, CueId, EffectPitch, EffectTurns, MusicEvent, Request,
    SenderProgress, SenderState, effects, time,
};
use canonical::{Schema, model::Model};
use data::assets::audio::{
    CueAlias, EffectClip, EffectPartContent, EffectSpan, Envelope, Instrument, ResidentAudio,
    SampleId, SampleSource, TimbreId,
};
use std::num::NonZeroU8;

fn opened_context() -> (Content, State) {
    let (transport, _) = pending_audio();
    let mut graph = audio_content();
    for track in &mut graph.tracks {
        track.phrases[0].insert(0, MusicEvent::Sustain);
    }
    let clip = EffectClip {
        parts: vec![EffectPartContent {
            spans: vec![EffectSpan {
                duration: EffectTurns(NonZeroU8::MIN),
                pitch: EffectPitch::PRENOTE,
                timbre: TimbreId(0),
                motion: None,
            }],
            loop_start: None,
        }],
    };
    let mut aliases = Vec::new();
    for (byte, cue) in (1..=8).zip([
        CueId::Footstep,
        CueId::Spin,
        CueId::EndSpin,
        CueId::Land,
        CueId::Beam,
        CueId::SamusSilence,
        CueId::BackgroundSilence,
        CueId::Door,
    ]) {
        let mut effect = clip.clone();
        if cue == CueId::Door {
            effect.parts.push(clip.parts[0].clone());
        }
        graph.effects.insert(cue, effect);
        aliases.push(CueAlias {
            group: effects::group(cue),
            byte,
            cue,
        });
    }
    let bank = ResidentAudio {
        content: graph,
        music_reply: transport.resident_music_reply.clone(),
        echo_delay: transport.song.mix.echo_delay,
        instruments: vec![Instrument {
            sample: SampleId(0),
            envelope: Envelope {
                attack_decay: 0,
                sustain: 0,
                gain: 0,
            },
            pitch_scale: 256,
        }],
        samples: vec![SampleSource {
            frames: 16,
            loop_from: None,
            source_sha256: [19; 32],
        }],
        aliases,
    };
    let origin =
        Origin::opening_ceres([3; 32], ClockRelation::OPENING_CERES, &time::halves(811)).unwrap();
    let content = Content::new(bank, origin).unwrap();
    let sender = SenderState {
        queue: std::collections::VecDeque::new(),
        progress: SenderProgress::Ready,
        reply: Request::Absent,
        now: CpuMasterTicks(0),
        pending: None,
    };
    let senders = Senders::new(sender.clone(), sender.clone(), sender).unwrap();
    let state = State {
        transport: Model(Box::new(transport)),
        senders,
        origin,
        content_identity: content.identity,
    };
    (content, state)
}

#[test]
fn joint_checkpoint_rejects_well_formed_but_inconsistent_saved_state() {
    let (content, state) = opened_context();
    let content = ProductionContent::ResidentAudio(Box::new(content));
    let direct = Production::admit(
        ProductionSeed::ResidentAudio(Box::new(state.clone())),
        content.clone(),
        ProductionRules::Audio,
    )
    .unwrap();
    let saved = direct.save().unwrap();
    let restore = |saved: &crate::production::SavedProduction| {
        Production::restore(saved, content.clone(), ProductionRules::Audio)
    };
    assert_eq!(restore(&saved).unwrap(), direct);
    let check = |state| {
        let mut corrupted = saved.clone();
        corrupted.state = canonical::to_bytes(&Checkpoint {
            schema: Schema::new(),
            state: Box::new(state),
        });
        restore(&corrupted)
    };

    let mut invalid = state.clone();
    invalid.content_identity[0] ^= 1;
    assert_eq!(
        check(invalid),
        Err(AdmissionFault::ResidentAudio(Fault::ContentIdentity))
    );

    let mut invalid = state.clone();
    invalid.origin.seed_identity[0] ^= 1;
    assert_eq!(
        check(invalid),
        Err(AdmissionFault::ResidentAudio(Fault::Origin))
    );

    let mut invalid = state;
    invalid.transport.0.phase = AudioPhase::ObserveClock {
        at: time::cycles(24),
    };
    assert!(matches!(
        check(invalid),
        Err(AdmissionFault::ResidentAudio(Fault::Invalid(_)))
    ));
}

/// Both group slots remain in clip order even when a slot has finished.
fn contact_playback(
    parts: [audio_transport::PartState; 2],
    state: &State,
) -> audio_transport::GroupState {
    let mut group = state.transport.0.groups[audio_transport::SoundGroup::Contact].clone();
    group.history = vec![
        audio_transport::ReleaseMemory::Active(NonZeroU8::MIN),
        audio_transport::ReleaseMemory::Active(NonZeroU8::new(3).unwrap()),
    ];
    group.playback = audio_transport::Playback::Active {
        cue: CueId::Door,
        parts: parts.into(),
    };
    group
}

#[test]
fn checkpoint_checks_contact_voice_order_in_committed_and_pending_slots() {
    use audio_transport::{CompletionResult, EffectVoiceId, LivePart, PartState, SoundGroup};
    let (content, origin) = opened_context();
    let content = ProductionContent::ResidentAudio(Box::new(content));
    let good = Production::admit(
        ProductionSeed::ResidentAudio(Box::new(origin.clone())),
        content.clone(),
        ProductionRules::Audio,
    )
    .unwrap();
    let saved = good.save().unwrap();
    let live = |voice| {
        PartState::Live(LivePart {
            voice,
            cursor: 0,
            instruction: 1,
            pitch: EffectPitch::PRENOTE,
            fraction: 0,
            motion: None,
        })
    };
    for pending in [false, true] {
        for (parts, ordered) in [
            ([live(EffectVoiceId::V7), live(EffectVoiceId::V5)], true),
            ([live(EffectVoiceId::V5), live(EffectVoiceId::V7)], false),
            ([PartState::Finished, live(EffectVoiceId::V5)], true),
            ([live(EffectVoiceId::V7), PartState::Finished], true),
        ] {
            let mut state = origin.clone();
            let result = contact_playback(parts, &state);
            if pending {
                state.transport.0.phase = AudioPhase::Work {
                    at: time::cycles(100),
                    result: CompletionResult::Effect {
                        group: SoundGroup::Contact,
                        result,
                        echo_voices: std::collections::BTreeSet::new(),
                    },
                };
            } else {
                state.transport.0.groups[SoundGroup::Contact] = result;
            }
            let mut specimen = saved.clone();
            specimen.state = canonical::to_bytes(&Checkpoint {
                schema: Schema::new(),
                state: Box::new(state),
            });
            let restored = Production::restore(&specimen, content.clone(), ProductionRules::Audio);
            if ordered {
                assert_eq!(restored.unwrap().save().unwrap(), specimen);
            } else {
                assert!(matches!(
                    restored,
                    Err(AdmissionFault::ResidentAudio(Fault::Invalid(_)))
                ));
            }
        }
    }
}

#[test]
fn sender_checkpoint_checks_the_profile_observation_without_consuming_it() {
    use crate::production::{AdvanceInput, ProductionOutputs, SenderInput};
    use audio_transport::SoundGroup;
    use audio_transport::sender::{SenderOutput, SenderProfile};
    let (content, origin) = opened_context();
    let content = ProductionContent::ResidentAudio(Box::new(content));
    let group = SoundGroup::Contact;
    for progress in [
        SenderProgress::AwaitCue(CueId::Door),
        SenderProgress::AwaitClear,
    ] {
        let mut state = origin.clone();
        state.senders.contact.0.now = CpuMasterTicks(31);
        state.senders.contact.0.progress = progress;
        let mut direct = Production::admit(
            ProductionSeed::ResidentAudio(Box::new(state)),
            content.clone(),
            ProductionRules::Audio,
        )
        .unwrap();
        let advance = |production: &mut Production, input| {
            production
                .advance(AdvanceInput::ResidentSender { group, input })
                .unwrap()
        };
        assert_eq!(
            advance(&mut direct, SenderInput::Begin(SenderProfile::FastBus)),
            ProductionOutputs::AudioSender(Vec::new())
        );
        let saved = direct.save().unwrap();
        let restore = |saved: &crate::production::SavedProduction| {
            Production::restore(saved, content.clone(), ProductionRules::Audio)
        };
        let mut candidate: Checkpoint<Box<State>> = canonical::from_bytes(&saved.state).unwrap();
        let wait = candidate.state.senders.contact.0.pending.as_mut().unwrap();
        assert_eq!(wait.observation, Some(CpuMasterTicks(89)));
        wait.observation = Some(CpuMasterTicks(90));
        let mut corrupt = saved.clone();
        corrupt.state = canonical::to_bytes(&candidate);
        assert!(matches!(
            restore(&corrupt),
            Err(AdmissionFault::ResidentAudio(Fault::Invalid(_)))
        ));

        let mut restored = restore(&saved).unwrap();
        assert_eq!(restored.save().unwrap(), saved);
        for production in [&mut direct, &mut restored] {
            assert_eq!(
                advance(
                    production,
                    SenderInput::Advance {
                        stop: CpuMasterTicks(88),
                        replies: &[],
                    }
                ),
                ProductionOutputs::AudioSender(Vec::new())
            );
            assert_eq!(
                advance(
                    production,
                    SenderInput::Advance {
                        stop: CpuMasterTicks(89),
                        replies: &[],
                    }
                ),
                ProductionOutputs::AudioSender(vec![SenderOutput::Observation(
                    CpuMasterTicks(89),
                    Request::Absent
                )])
            );
            let after = production.save().unwrap();
            assert_eq!(restore(&after).unwrap().save().unwrap(), after);
        }
        assert_eq!(direct, restored);
    }
}
