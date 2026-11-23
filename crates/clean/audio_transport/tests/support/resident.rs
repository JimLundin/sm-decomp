//! Spec: PublishedReplies. Each music turn publishes the resident reply, taken from its input
//! identity, never from the expected aliases.
use super::*;
/// Whether this turn restarts the fixture's arrangement: its first track ends its note at the
/// root's completion.
fn restarts(before: &SongState) -> bool {
    before.tracks[0].note.0.get() == 1
        && before.tracks[0].position == TrackPosition::Root { cursor: 1 }
}
pub fn resident_vector(row: &Table, mix: &Value) {
    keys(
        row,
        "id resident_identity resident_reply_alias tracks mix music_turns expected_reply_aliases after_restarts inputs",
    );
    assert!(array(row, "inputs").is_empty(), "no new selection request");
    let (song, tracks) = song(row, mix);
    let content = content(BTreeMap::new(), tracks);
    let resident = music_reply(
        text(row, "resident_identity"),
        byte(row, "resident_reply_alias"),
    );
    let mut state = fresh_state(song);
    state.resident_music_reply = resident.clone();
    let mut aliases = Vec::new();
    let mut restarted = 0u64;
    for _ in 0..unsigned(row, "music_turns") {
        restarted = restarted.saturating_add(u64::from(restarts(&state.song)));
        play(&mut state, &content, &[], ONE_MUSIC_TURN);
        assert_eq!(state.music_reply, resident);
        assert_eq!(state.resident_music_reply, resident);
        aliases.push(i64::from(state.music_reply.original_byte));
    }
    assert_eq!(aliases, ints(row, "expected_reply_aliases"));
    assert_eq!(restarted, unsigned(row, "after_restarts"));
}
#[test]
fn every_original_byte_is_published_as_supplied() {
    let root: Table = toml::from_str(VECTORS).expect("vectors");
    let row = table(&array(&root, "resident_reply")[0]);
    let (song, tracks) = song(row, &root["fixture_mix"]);
    let content = content(BTreeMap::new(), tracks);
    for original_byte in 0..=u8::MAX {
        let resident = music_reply("independent caller identity", original_byte);
        let mut state = fresh_state(song.clone());
        state.resident_music_reply = resident.clone();
        play(&mut state, &content, &[], ONE_MUSIC_TURN);
        assert_eq!(state.music_reply, resident);
        assert_eq!(state.resident_music_reply, resident);
        let mut quiet = roundtrip(&state);
        play(&mut quiet, &content, &[], NO_TURN);
        assert_eq!(
            quiet, state,
            "a frame without music turns publishes nothing"
        );
    }
}
