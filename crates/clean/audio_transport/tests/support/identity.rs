//! Spec: EffectResources and SongControls. Echo membership is a selection over all eight voice
//! identities, separate from effect ownership and reservations.
use super::*;
fn find<'a>(root: &'a Table, family: &str, id: &str) -> &'a Table {
    array(root, family)
        .iter()
        .map(table)
        .find(|r| text(r, "id") == id)
        .expect("fixture vector")
}
#[test]
fn every_echo_subset_leaves_effect_ownership_alone() {
    let root: Table =
        toml::from_str(&vector_file::read("audio-transport").unwrap()).expect("vectors");
    let mix = &root["fixture_mix"];
    let identities = [
        VoiceId::FirstMusic,
        VoiceId::SecondMusic,
        VoiceId::Effect(EffectVoiceId::V2),
        VoiceId::Effect(EffectVoiceId::V3),
        VoiceId::Effect(EffectVoiceId::V4),
        VoiceId::Effect(EffectVoiceId::V5),
        VoiceId::Effect(EffectVoiceId::V6),
        VoiceId::Effect(EffectVoiceId::V7),
    ];
    let (song, mut tracks) = song(find(&root, "control", "M-echo-all-voices"), mix);
    let fixture = effect_fixture(find(&root, "effect", "E-contact-occupied"));
    let reservations = BTreeSet::from([EffectVoiceId::V2, EffectVoiceId::V5]);
    for mask in 0u16..256 {
        let selected: BTreeSet<_> = identities
            .iter()
            .enumerate()
            .filter(|(i, _)| mask & (1 << i) != 0)
            .map(|(_, v)| *v)
            .collect();
        let mut state = effect_state(&fixture, mix);
        state.song = song.clone();
        state.song.mix.echo_voices.clone_from(&selected);
        state.reservations.clone_from(&reservations);
        play(&mut state, &fixture.content, &[], ONE_EFFECT_TURN);
        let owned = BTreeSet::from([EffectVoiceId::V7]);
        assert_eq!(state.groups[fixture.group].playback.owned_voices(), owned);
        let mut remaining = selected.clone();
        remaining.remove(&VoiceId::Effect(EffectVoiceId::V7));
        assert_eq!(state.song.mix.echo_voices, remaining);
        let choice = &mut tracks[0].phrases[0][0];
        let voices = if let MusicEvent::EchoMix { voices, .. } = choice {
            Some(voices)
        } else {
            None
        };
        voices.expect("an echo choice").clone_from(&selected);
        let music = content(fixture.content.effects.clone(), tracks.clone());
        play(&mut state, &music, &[], ONE_MUSIC_TURN);
        assert_eq!(state.song.mix.echo_voices, selected);
        assert_eq!(state.reservations, reservations);
        assert_eq!(state.groups[fixture.group].playback.owned_voices(), owned);
    }
}
