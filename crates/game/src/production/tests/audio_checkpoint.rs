use super::{
    AdmissionFault, Production, ProductionContent, ProductionRules, ProductionSeed, fixtures,
};
use audio_transport::{
    CueId, EffectPitch, EffectVoiceId, LivePart, MusicalPitch, PartState, Playback, ReleaseMemory,
    Request, SoundGroup, State,
};
use std::num::NonZeroU8;

fn music_history(pitch: u8) -> State {
    let mut state = fixtures::transport();
    state.song.tracks[0].pitch = MusicalPitch::new(pitch).unwrap();
    state
}

fn effect_history(pitch: u8) -> (State, ProductionContent) {
    let mut state = fixtures::transport();
    let group = SoundGroup::Action;
    let cue = CueId::Beam;
    let request = Request::Cue(cue);
    let saved = &mut state.groups[group];
    saved.history[0] = ReleaseMemory::Active(NonZeroU8::MIN);
    saved.handled = request;
    saved.staged = request;
    saved.reply = request;
    saved.playback = Playback::Active {
        cue,
        parts: vec![PartState::Live(LivePart {
            voice: EffectVoiceId::V7,
            cursor: 1,
            instruction: 1,
            pitch: EffectPitch::new(pitch).unwrap(),
            fraction: 0,
            motion: None,
        })],
    };
    state.request_lines[group] = request;
    let content = fixtures::audio_content();
    (state, ProductionContent::Audio(Box::new(content)))
}

fn music_content() -> ProductionContent {
    ProductionContent::Audio(Box::new(fixtures::audio_content()))
}

fn admit(state: State, content: ProductionContent) -> Production {
    Production::admit(
        ProductionSeed::Audio(Box::new(state)),
        content,
        ProductionRules::Audio,
    )
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
fn restore_refuses_music_pitch_below_minimum() {
    check_rejected_pitch(
        music_history(19),
        music_history(20),
        music_content(),
        19,
        20,
        18,
    );
}

#[test]
fn restore_refuses_music_pitch_above_maximum() {
    check_rejected_pitch(
        music_history(51),
        music_history(50),
        music_content(),
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

#[test]
fn portable_audio_refuses_another_schema_or_scope_tag() {
    use crate::production::PortableAudio;
    use crate::state::owned::AudioPin;
    use canonical::model::Model;
    use canonical::{Canonical, DecodeError, Schema};

    let state = music_history(24);
    let content = music_content();
    let mut saved = admit(state.clone(), content.clone()).save().unwrap();
    let (movement, _, _) = super::movement();
    saved.state = movement.save().unwrap().state;
    assert!(matches!(
        Production::restore(&saved, content.clone(), ProductionRules::Audio),
        Err(AdmissionFault::Checkpoint(
            DecodeError::SchemaMismatch { .. }
        )),
    ));

    let mut wrong_scope = canonical::to_bytes(&Schema::<PortableAudio>::new());
    // A valid Audio payload under the Movement tag must still be refused.
    1_u8.encode(&mut wrong_scope);
    Model::<AudioPin>(Box::new(state)).encode(&mut wrong_scope);
    saved.state = wrong_scope;
    assert!(matches!(
        Production::restore(&saved, content, ProductionRules::Audio),
        Err(AdmissionFault::Checkpoint(DecodeError::UnknownVariant {
            tag: 1,
            ..
        })),
    ));
}
