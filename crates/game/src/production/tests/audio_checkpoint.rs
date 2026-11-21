use super::{
    AdmissionFault, AdvanceInput, Production, ProductionContent, ProductionOutputs,
    ProductionRules, ProductionSeed, fixtures,
};
use audio_transport::{
    AudioPhase, CompletionResult, CueId, EffectClip, EffectPartContent, EffectPitch, EffectSpan,
    EffectTurns, EffectVoiceId, LivePart, MusicalPitch, PartState, Playback, ReleaseMemory, Reply,
    ReplyLine, Request, SoundGroup, State, TimbreId, time::cycles, transport::Output,
};
use std::{collections::BTreeSet, num::NonZeroU8};

fn music_history(committed: u8, pending: u8) -> State {
    let (mut state, _) = fixtures::pending_audio();
    state.song.tracks[0].pitch = MusicalPitch::new(committed).unwrap();
    let AudioPhase::Work {
        result: CompletionResult::Music(song),
        ..
    } = &mut state.phase
    else {
        panic!("pending music fixture");
    };
    song.tracks[0].pitch = MusicalPitch::new(pending).unwrap();
    state
}

fn effect_history(pending: u8) -> (State, ProductionContent) {
    let (mut state, _) = fixtures::pending_audio();
    let group = SoundGroup::Action;
    let cue = CueId::Beam;
    let request = Request::Cue(cue);
    let before = &mut state.groups[group];
    before.history[0] = ReleaseMemory::Active(NonZeroU8::MIN);
    before.handled = request;
    before.staged = request;
    before.reply = request;
    let part = LivePart {
        voice: EffectVoiceId::V7,
        cursor: 0,
        instruction: 1,
        pitch: EffectPitch::PRENOTE,
        fraction: 0,
        motion: None,
    };
    before.playback = Playback::Active {
        cue,
        parts: vec![PartState::Live(part.clone())],
    };
    let mut result = before.clone();
    result.history[0] = ReleaseMemory::Inactive;
    result.playback = Playback::Active {
        cue,
        parts: vec![PartState::Live(LivePart {
            cursor: 1,
            pitch: EffectPitch::new(pending).unwrap(),
            ..part
        })],
    };
    state.requests[group] = request;
    state.phase = AudioPhase::Work {
        at: cycles(100),
        result: CompletionResult::Effect {
            group,
            result,
            echo_voices: BTreeSet::new(),
        },
    };
    let mut content = fixtures::audio_content();
    content.effects.insert(
        cue,
        EffectClip {
            parts: vec![EffectPartContent {
                spans: vec![EffectSpan {
                    duration: EffectTurns(NonZeroU8::new(1).unwrap()),
                    pitch: EffectPitch::new(71).unwrap(),
                    timbre: TimbreId(0),
                    motion: None,
                }],
                loop_start: None,
            }],
        },
    );
    (state, ProductionContent::Audio(content))
}

fn admit(state: State, content: ProductionContent) -> Production {
    Production::admit(
        ProductionSeed::Audio(Box::new(state)),
        content,
        ProductionRules::Audio,
    )
    .unwrap()
}

fn check_publication(state: State, content: ProductionContent, line: ReplyLine, reply: Reply) {
    let AudioPhase::Work { at, result } = state.phase.clone() else {
        panic!("pending work fixture");
    };
    let mut carried = admit(state, content.clone());
    assert_eq!(advance(&mut carried, 99), ProductionOutputs::Audio(vec![]));
    let saved = carried.save().unwrap();
    let mut restored = Production::restore(&saved, content, ProductionRules::Audio).unwrap();
    assert_eq!(restored.state(), carried.state());
    assert_eq!(restored.save().unwrap(), saved);

    let expected = ProductionOutputs::Audio(vec![Output::Publication {
        at,
        line,
        reply,
        result,
    }]);
    assert_eq!(advance(&mut carried, 100), expected);
    assert_eq!(advance(&mut restored, 100), expected);
    assert_eq!(restored.state(), carried.state());
    assert_eq!(restored.save().unwrap(), carried.save().unwrap());
    assert_eq!(
        advance(&mut restored, 100),
        ProductionOutputs::Audio(vec![])
    );
}

fn advance(production: &mut Production, stop: u64) -> ProductionOutputs {
    production
        .advance(AdvanceInput::Audio {
            stop: cycles(stop),
            presentations: &[],
        })
        .unwrap()
}

fn check_rejected_pitch(
    state: State,
    alternate: State,
    content: ProductionContent,
    boundary: u8,
    neighbour: u8,
    invalid: u8,
) {
    let mut saved = admit(state, content.clone()).save().unwrap();
    let alternate = admit(alternate, content.clone()).save().unwrap();
    assert_eq!(saved.state.len(), alternate.state.len());
    // Find the field through the encoder so this test does not depend on a hard-coded offset.
    let changes: Vec<_> = saved
        .state
        .iter()
        .zip(&alternate.state)
        .enumerate()
        .filter_map(|(index, (&left, &right))| (left != right).then_some((index, left, right)))
        .collect();
    let [(offset, old, new)] = changes.as_slice() else {
        panic!("expected one saved pitch byte to differ");
    };
    assert_eq!((*old, *new), (boundary, neighbour));
    saved.state[*offset] = invalid;
    assert_eq!(
        Production::restore(&saved, content, ProductionRules::Audio).map(|_| ()),
        Err(AdmissionFault::Checkpoint(canonical::DecodeError::Refused)),
        "restore must refuse pitch {invalid}"
    );
}

#[test]
fn music_endpoints_preserve_state_and_publication() {
    let state = music_history(19, 51);
    let reply = Reply::Music(state.resident_music_reply.clone());
    check_publication(
        state,
        ProductionContent::Audio(fixtures::audio_content()),
        ReplyLine::Music,
        reply,
    );
}

#[test]
fn effect_endpoints_preserve_state_and_publication() {
    let (state, content) = effect_history(71);
    check_publication(
        state,
        content,
        ReplyLine::Group(SoundGroup::Action),
        Reply::Effect(Request::Cue(CueId::Beam)),
    );
}

#[test]
fn restore_refuses_music_pitch_below_minimum() {
    check_rejected_pitch(
        music_history(19, 51),
        music_history(20, 51),
        ProductionContent::Audio(fixtures::audio_content()),
        19,
        20,
        18,
    );
}

#[test]
fn restore_refuses_music_pitch_above_maximum() {
    check_rejected_pitch(
        music_history(19, 51),
        music_history(19, 50),
        ProductionContent::Audio(fixtures::audio_content()),
        51,
        50,
        52,
    );
}

#[test]
fn restore_refuses_effect_pitch_above_maximum() {
    let (state, content) = effect_history(71);
    let (alternate, _) = effect_history(70);
    check_rejected_pitch(state, alternate, content, 71, 70, 72);
}
