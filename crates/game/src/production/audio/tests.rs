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
    let (mut transport, _) = pending_audio();
    transport.phase = AudioPhase::Idle {
        at: time::cycles(100),
    };
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
    let (bank, origin) = opened_context();
    let graph = &bank.bank().content;
    let content = ProductionContent::ResidentAudio(Box::new(bank.clone()));
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
        for (mut parts, ordered) in [
            ([live(EffectVoiceId::V7), live(EffectVoiceId::V5)], true),
            ([live(EffectVoiceId::V5), live(EffectVoiceId::V7)], false),
            ([PartState::Finished, live(EffectVoiceId::V5)], true),
            ([live(EffectVoiceId::V7), PartState::Finished], true),
        ] {
            let mut state = origin.clone();
            if pending && !ordered {
                parts.swap(0, 1);
            }
            let committed = contact_playback(parts, &state);
            if pending {
                let turn = effects::turn(
                    SoundGroup::Contact,
                    &committed,
                    graph,
                    &std::collections::BTreeSet::new(),
                    &state.transport.0.song.mix.echo_voices,
                );
                let mut result = turn.group;
                if !ordered {
                    let audio_transport::Playback::Active { parts, .. } = &mut result.playback
                    else {
                        panic!("pending active Contact");
                    };
                    parts.swap(0, 1);
                }
                state.transport.0.phase = AudioPhase::Work {
                    at: state
                        .transport
                        .0
                        .now
                        .sum(&audio_transport::transport::elapsed(turn.work)),
                    result: CompletionResult::Effect {
                        group: SoundGroup::Contact,
                        result,
                        echo_voices: turn.echo_voices,
                    },
                };
            }
            state.transport.0.groups[SoundGroup::Contact] = committed;
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

fn sender_step(
    production: &mut Production,
    input: crate::production::SenderInput<'_>,
) -> Vec<audio_transport::sender::SenderOutput> {
    use crate::production::{AdvanceInput, ProductionOutputs};
    let ProductionOutputs::AudioSender(outputs) = production
        .advance(AdvanceInput::ResidentSender {
            group: audio_transport::SoundGroup::Contact,
            input,
        })
        .unwrap()
    else {
        panic!("contact sender output");
    };
    outputs
}

fn sender_at(
    production: &mut Production,
    stop: u64,
    replies: &[audio_transport::sender::ReplyInput],
) -> Vec<audio_transport::sender::SenderOutput> {
    sender_step(
        production,
        crate::production::SenderInput::Advance {
            stop: CpuMasterTicks(stop),
            replies,
        },
    )
}

fn start_contact(progress: SenderProgress) -> (ProductionContent, Production) {
    use crate::production::SenderInput;
    use audio_transport::sender::SenderProfile;
    let (content, mut state) = opened_context();
    state.senders.contact.0.progress = progress;
    state.senders.contact.0.queue.push_back(CueId::Door);
    let content = ProductionContent::ResidentAudio(Box::new(content));
    let mut production = Production::admit(
        ProductionSeed::ResidentAudio(Box::new(state)),
        content.clone(),
        ProductionRules::Audio,
    )
    .unwrap();
    assert_eq!(
        sender_step(&mut production, SenderInput::Begin(SenderProfile::FastBus)).len(),
        0
    );
    (content, production)
}

fn edit_contact_checkpoint(
    saved: &crate::production::SavedProduction,
    edit: impl FnOnce(&mut SenderState),
) -> crate::production::SavedProduction {
    let mut candidate: Checkpoint<Box<State>> = canonical::from_bytes(&saved.state).unwrap();
    edit(&mut candidate.state.senders.contact.0);
    let mut changed = saved.clone();
    changed.state = canonical::to_bytes(&candidate);
    changed
}

#[test]
fn sender_checkpoint_keeps_only_the_actual_remaining_milestones() {
    use audio_transport::sender::SenderOutput;
    let (content, mut direct) = start_contact(SenderProgress::Ready);
    let cue = Request::Cue(CueId::Door);
    let restore = |saved: &crate::production::SavedProduction| {
        Production::restore(saved, content.clone(), ProductionRules::Audio)
    };
    assert_eq!(sender_at(&mut direct, 211, &[]).len(), 0);
    let before = direct.save().unwrap();
    let mut restored = restore(&before).unwrap();
    assert_eq!(restored.save().unwrap(), before);
    for (presentation, completion) in [(Some(212), 399), (Some(213), 398), (None, 398)] {
        let changed = edit_contact_checkpoint(&before, |sender| {
            let result = sender.pending.as_mut().unwrap().result.as_mut().unwrap();
            result.presentation = presentation.map(|tick| (CpuMasterTicks(tick), cue));
            result.completion = CpuMasterTicks(completion);
        });
        assert!(matches!(
            restore(&changed),
            Err(AdmissionFault::ResidentAudio(Fault::Invalid(_)))
        ));
    }
    for production in [&mut direct, &mut restored] {
        assert_eq!(
            sender_at(production, 212, &[]),
            [SenderOutput::Presentation(CpuMasterTicks(212), cue)]
        );
        let after = production.save().unwrap();
        *production = restore(&after).unwrap();
        assert_eq!(production.save().unwrap(), after);
        assert_eq!(sender_at(production, 397, &[]).len(), 0);
        assert_eq!(
            sender_at(production, 398, &[]),
            [SenderOutput::Completed(CpuMasterTicks(398))]
        );
    }
    assert_eq!(direct, restored);
}

#[test]
fn sender_checkpoint_keeps_the_coalesced_read_result_after_the_latch_changes() {
    use audio_transport::sender::{ReplyInput, SenderOutput};
    let cue = Request::Cue(CueId::Door);
    let held = Request::Cue(CueId::Footstep);
    for (progress, sampled, opposite, presentation, completion) in [
        (
            SenderProgress::AwaitCue(CueId::Door),
            cue,
            Request::Absent,
            None,
            210,
        ),
        (
            SenderProgress::AwaitCue(CueId::Door),
            Request::Absent,
            cue,
            Some((104, cue)),
            162,
        ),
        (
            SenderProgress::AwaitClear,
            Request::Absent,
            held,
            Some((342, cue)),
            528,
        ),
        (
            SenderProgress::AwaitClear,
            held,
            Request::Absent,
            Some((104, Request::Absent)),
            162,
        ),
    ] {
        let (content, mut direct) = start_contact(progress);
        let restore = |saved: &crate::production::SavedProduction| {
            Production::restore(saved, content.clone(), ProductionRules::Audio)
        };
        let simultaneous = [opposite, sampled].map(|reply| ReplyInput {
            at: CpuMasterTicks(58),
            reply,
        });
        assert_eq!(
            sender_at(&mut direct, 58, &simultaneous),
            [SenderOutput::Observation(CpuMasterTicks(58), sampled)]
        );
        let at_read = direct.save().unwrap();
        assert_eq!(restore(&at_read).unwrap().save().unwrap(), at_read);
        let changed = edit_contact_checkpoint(&at_read, |sender| sender.reply = opposite);
        assert!(matches!(
            restore(&changed),
            Err(AdmissionFault::ResidentAudio(Fault::Invalid(_)))
        ));

        assert_eq!(
            sender_at(
                &mut direct,
                59,
                &[ReplyInput {
                    at: CpuMasterTicks(59),
                    reply: opposite
                }]
            )
            .len(),
            0
        );
        let later = direct.save().unwrap();
        let mut restored = restore(&later).unwrap();
        assert_eq!(restored.save().unwrap(), later);
        let expected: Vec<_> = presentation
            .map(|(tick, request)| SenderOutput::Presentation(CpuMasterTicks(tick), request))
            .into_iter()
            .chain([SenderOutput::Completed(CpuMasterTicks(completion))])
            .collect();
        for production in [&mut direct, &mut restored] {
            assert_eq!(sender_at(production, completion, &[]), expected);
        }
        assert_eq!(direct, restored);
    }
}

fn overlong_work_deadline(
    held: &crate::production::SavedProduction,
    deadline: &audio_transport::AudioInstant,
    work_start: &audio_transport::AudioInstant,
) -> crate::production::SavedProduction {
    let mut bad_deadline: Checkpoint<Box<State>> = canonical::from_bytes(&held.state).unwrap();
    let AudioPhase::Work { at, .. } = &mut bad_deadline.state.transport.0.phase else {
        panic!("pending transport deadline");
    };
    // The fresh turn schedules its full work from the actual start.
    let work = deadline.difference(work_start);
    *at = bad_deadline
        .state
        .transport
        .0
        .now
        .sum(&work)
        .sum(&time::cycles(1));
    let mut altered_deadline = held.clone();
    altered_deadline.state = canonical::to_bytes(&bad_deadline);
    altered_deadline
}

fn transport_test_step(
    production: &mut Production,
    stop: audio_transport::AudioInstant,
    presentations: &[audio_transport::transport::Input],
) -> crate::production::ProductionOutputs {
    production
        .advance(crate::production::AdvanceInput::Audio {
            stop,
            presentations,
        })
        .unwrap()
}

fn alter_transport_save(
    saved: &crate::production::SavedProduction,
    edit: impl FnOnce(&mut audio_transport::State),
) -> crate::production::SavedProduction {
    let mut candidate: Checkpoint<Box<State>> = canonical::from_bytes(&saved.state).unwrap();
    edit(&mut candidate.state.transport.0);
    let mut changed = saved.clone();
    changed.state = canonical::to_bytes(&candidate);
    changed
}

fn start_transport_work(effect: bool) -> (ProductionContent, Production) {
    use audio_transport::SoundGroup;
    let (content, mut state) = opened_context();
    let start = if effect {
        time::halves(53)
    } else {
        time::cycles(11)
    };
    state.transport.0.phase = if effect {
        state.transport.0.cadence.effect = 7;
        state.transport.0.cadence.music = 224;
        state.transport.0.groups[SoundGroup::Action].staged = Request::Cue(CueId::Spin);
        AudioPhase::Distribute {
            at: start.clone(),
            batch: 1,
        }
    } else {
        state.transport.0.music_due = true;
        AudioPhase::MusicStart { at: start.clone() }
    };
    let content = ProductionContent::ResidentAudio(Box::new(content));
    let mut direct = Production::admit(
        ProductionSeed::ResidentAudio(Box::new(state)),
        content.clone(),
        ProductionRules::Audio,
    )
    .unwrap();
    assert_eq!(
        transport_test_step(&mut direct, start, &[]),
        crate::production::ProductionOutputs::Audio(Vec::new())
    );
    (content, direct)
}

fn check_pending_transport_turn(effect: bool) {
    use crate::production::ProductionOutputs;
    use audio_transport::{CompletionResult, MusicalPitch, SoundGroup, VoiceId, transport};
    let (content, mut direct) = start_transport_work(effect);
    let restore = |saved: &crate::production::SavedProduction| {
        Production::restore(saved, content.clone(), ProductionRules::Audio)
    };
    let before = direct.save().unwrap();
    let original_clock = direct.transport().unwrap().clock.clone();
    let mut restored = restore(&before).unwrap();
    let work_start = direct.transport().unwrap().now.clone();
    let stop = work_start.sum(&time::cycles(1));
    let input = transport::Input::cue(stop.clone(), CueId::Beam);
    for production in [&mut direct, &mut restored] {
        assert_eq!(
            transport_test_step(production, stop.clone(), std::slice::from_ref(&input)),
            ProductionOutputs::Audio(Vec::new())
        );
        assert_ne!(production.transport().unwrap().clock, original_clock);
        assert_eq!(
            production.transport().unwrap().requests[SoundGroup::Action],
            Request::Cue(CueId::Beam)
        );
    }
    let held = direct.save().unwrap();
    assert_eq!(restored.save().unwrap(), held);
    restored = restore(&held).unwrap();
    let mut fork: Checkpoint<Box<State>> = canonical::from_bytes(&held.state).unwrap();
    let AudioPhase::Work { at, result } = &mut fork.state.transport.0.phase else {
        panic!("pending transport result");
    };
    let deadline = at.clone();
    let expected = result.clone();
    match result {
        CompletionResult::Effect { echo_voices, .. } => {
            assert!(effect && echo_voices.insert(VoiceId::FirstMusic));
        }
        CompletionResult::Music(song) => {
            assert!(!effect);
            let pitch = &mut song.tracks[0].pitch;
            *pitch = MusicalPitch::new(if pitch.get() == 19 { 20 } else { 19 }).unwrap();
        }
    }
    let altered_deadline = overlong_work_deadline(&held, &deadline, &work_start);
    assert!(matches!(
        restore(&altered_deadline),
        Err(AdmissionFault::ResidentAudio(Fault::Invalid(_)))
    ));

    let mut forged = held;
    forged.state = canonical::to_bytes(&fork);
    assert!(matches!(
        restore(&forged),
        Err(AdmissionFault::ResidentAudio(Fault::Invalid(_)))
    ));

    let near_end = deadline.difference(&time::halves(1));
    for production in [&mut direct, &mut restored] {
        assert_eq!(
            transport_test_step(production, near_end.clone(), &[]),
            ProductionOutputs::Audio(Vec::new())
        );
        let saved = production.save().unwrap();
        *production = restore(&saved).unwrap();
        assert_eq!(production.save().unwrap(), saved);
    }
    let output = transport_test_step(&mut direct, deadline.clone(), &[]);
    assert!(matches!(
        &output,
        ProductionOutputs::Audio(outputs)
            if matches!(outputs.as_slice(), [transport::Output::Publication { result, .. }]
                if result == &expected)
    ));
    assert_eq!(transport_test_step(&mut restored, deadline, &[]), output);
    assert_eq!(restored.save().unwrap(), direct.save().unwrap());
}

#[test]
fn pending_music_turn_rejects_substituted_pitch() {
    check_pending_transport_turn(false);
}

#[test]
fn pending_effect_turn_rejects_substituted_echo_membership() {
    check_pending_transport_turn(true);
}

#[test]
fn resident_work_keeps_rational_inputs_and_checks_its_scheduler_fields() {
    use crate::production::ProductionOutputs;
    use audio_transport::{AudioInstant, transport};
    let third = AudioInstant::new(1u64.into(), 3u64.into()).unwrap();
    for effect in [false, true] {
        let (content, mut production) = start_transport_work(effect);
        let stop = production
            .transport()
            .unwrap()
            .now
            .sum(&time::cycles(1).sum(&third));
        let restore = |saved: &crate::production::SavedProduction| {
            Production::restore(saved, content.clone(), ProductionRules::Audio)
        };
        let input = transport::Input::cue(stop.clone(), CueId::Beam);
        assert_eq!(
            transport_test_step(&mut production, stop.clone(), &[input]),
            ProductionOutputs::Audio(Vec::new())
        );
        let held = production.transport().unwrap();
        assert_eq!(held.now, stop);
        assert_eq!(held.clock.phase.instant(), &stop);
        assert_eq!(held.music_due, effect);
        let saved = production.save().unwrap();
        let mut restored = restore(&saved).unwrap();
        assert_eq!(restored.save().unwrap(), saved);

        let AudioPhase::Work { at, .. } = &held.phase else {
            panic!("pending work");
        };
        let deadline = at.clone();
        let with_fractional_deadline = alter_transport_save(&saved, |transport| {
            let AudioPhase::Work { at, .. } = &mut transport.phase else {
                panic!("pending work");
            };
            *at = at.sum(&third);
        });
        assert!(matches!(
            restore(&with_fractional_deadline),
            Err(AdmissionFault::ResidentAudio(Fault::Invalid(_)))
        ));
        let with_fractional_lattice = alter_transport_save(&saved, |transport| {
            transport.clock.lattice = transport.clock.lattice.sum(&third);
        });
        assert!(matches!(
            restore(&with_fractional_lattice),
            Err(AdmissionFault::ResidentAudio(Fault::Invalid(_)))
        ));
        if !effect {
            let with_music_due = alter_transport_save(&saved, |transport| {
                transport.music_due = true;
            });
            assert!(matches!(
                restore(&with_music_due),
                Err(AdmissionFault::ResidentAudio(Fault::Invalid(_)))
            ));
        }

        let output = transport_test_step(&mut production, deadline.clone(), &[]);
        assert_eq!(transport_test_step(&mut restored, deadline, &[]), output);
        let published = production.save().unwrap();
        assert_eq!(restore(&published).unwrap().save().unwrap(), published);
        let with_fractional_anchor = alter_transport_save(&published, |transport| {
            let AudioPhase::Sample { sample, .. } = &mut transport.phase else {
                panic!("stable sampling after publication");
            };
            sample.anchor = sample.anchor.sum(&third);
        });
        assert!(matches!(
            restore(&with_fractional_anchor),
            Err(AdmissionFault::ResidentAudio(Fault::Invalid(_)))
        ));
    }
}

mod phase_tests;
