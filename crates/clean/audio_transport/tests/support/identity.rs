//! Shared echo identity membership is independent of effect allocation and restoration.
use super::*;
#[test]
fn every_echo_subset_preserves_effect_pool_ownership() {
    let root: Table = toml::from_str(SOURCE).expect("vectors");
    let row = array(&root, "control")
        .iter()
        .map(|v| v.as_table().expect("control"))
        .find(|r| text(r, "id") == "M-echo-all-voices")
        .expect("shared identity fixture");
    let (song, mut tracks) = song(row, &root["fixture_mix"]);
    let identities = [
        audio_transport::VoiceId::FirstMusic,
        audio_transport::VoiceId::SecondMusic,
        audio_transport::VoiceId::Effect(VoiceId::V2),
        audio_transport::VoiceId::Effect(VoiceId::V3),
        audio_transport::VoiceId::Effect(VoiceId::V4),
        audio_transport::VoiceId::Effect(VoiceId::V5),
        audio_transport::VoiceId::Effect(VoiceId::V6),
        audio_transport::VoiceId::Effect(VoiceId::V7),
    ];
    let effect_row = array(&root, "effect")
        .iter()
        .map(|v| v.as_table().expect("effect"))
        .find(|r| text(r, "id") == "E-contact-occupied")
        .expect("allocation fixture");
    let (group, saved, mut content, _) = effect_fixture(effect_row);
    let reservations = BTreeSet::from([VoiceId::V2, VoiceId::V5]);
    let allocation = effects::turn(
        group,
        &saved,
        &content,
        &reservations,
        &song.mix.echo_voices,
    );
    assert_eq!(
        effects::owned(&allocation.group.playback),
        BTreeSet::from([VoiceId::V7])
    );
    for mask in 0u16..256 {
        let voices: BTreeSet<_> = identities
            .iter()
            .enumerate()
            .filter(|(i, _)| mask & (1 << i) != 0)
            .map(|(_, v)| *v)
            .collect();
        let allocated = effects::turn(group, &saved, &content, &reservations, &voices);
        let mut remaining = voices.clone();
        remaining.remove(&audio_transport::VoiceId::Effect(VoiceId::V7));
        assert_eq!(allocated.echo_voices, remaining);
        assert_eq!(allocated.work, allocation.work);
        assert_eq!(allocated.group, allocation.group);
        let MusicEvent::EchoMix {
            voices: selected, ..
        } = &mut tracks[0].phrases[0][0]
        else {
            assert!(false);
            return;
        };
        selected.clone_from(&voices);
        content.tracks.clone_from(&tracks);
        let result = music::turn(&song, &content.tracks);
        assert_eq!(result.song.mix.echo_voices, voices);
        let mut state = state_fixture(song.clone(), 0, 0);
        state.reservations.clone_from(&reservations);
        state.groups[group] = allocation.group.clone();
        state.music_due = true;
        state.phase = AudioPhase::MusicStart {
            at: time::cycles(0),
        };
        assert_eq!(
            transport::advance(&mut state, &content, time::cycles(0), &[]),
            Vec::new()
        );
        state = roundtrip(&state);
        let publication = transport::advance(&mut state, &content, time::whole(result.work.0), &[]);
        assert_eq!(publication.len(), 1);
        assert_eq!(state.song, result.song);
        assert_eq!(state.song.mix.echo_voices, voices);
        assert_eq!(state.reservations, reservations);
        assert_eq!(
            effects::owned(&state.groups[group].playback),
            BTreeSet::from([VoiceId::V7])
        );
        assert_eq!(roundtrip(&state), state);
    }
}
