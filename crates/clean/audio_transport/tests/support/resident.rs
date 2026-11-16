//! Resident identity is initialized from input fields, never expected publication aliases.
use super::*;

pub fn resident_vector(row: &Table, mix: &Value) {
    keys(
        row,
        "id resident_identity resident_reply_alias clock effect_phase music_phase tracks mix music_publications expected_reply_aliases after_restarts inputs",
    );
    let (song, tracks) = song(row, mix);
    let mut initial = state_fixture(song, byte(row, "effect_phase"), byte(row, "music_phase"));
    initial.resident_music_reply = ResidentMusicReply {
        identity: text(row, "resident_identity").into(),
        original_byte: byte(row, "resident_reply_alias"),
    };
    let clock = row["clock"].as_table().expect("resident clock");
    keys(
        clock,
        "now timer_phase unread_overflows polling_origin_numerator polling_origin_denominator",
    );
    initial.now = time::cycles(unsigned(clock, "now"));
    initial.clock.phase = AudioCyclePhase::try_from(time::cycles(unsigned(clock, "timer_phase")))
        .expect("admitted timer phase");
    initial.clock.unread = byte(clock, "unread_overflows");
    initial.clock.lattice = time::instant(
        unsigned(clock, "polling_origin_numerator"),
        NonZeroU64::new(unsigned(clock, "polling_origin_denominator"))
            .expect("positive denominator"),
    );
    assert_eq!(array(row, "inputs"), &Vec::<Value>::new());
    transport::initialize(&mut initial);
    let content = Content {
        effects: BTreeMap::new(),
        tracks,
    };
    let mut resumed = initial.clone();
    let mut outputs = Vec::new();
    let mut aliases = Vec::new();
    let mut restarts = 0u64;
    while aliases.len()
        < usize::try_from(unsigned(row, "music_publications")).expect("publication count")
    {
        let at = next_milestone(&resumed.phase);
        let before = resumed.song.clone();
        let elapsed = time::subtract(&at, &resumed.now);
        let step = transport::advance(&mut resumed, &content, elapsed, &[]);
        for output in &step {
            if let Some((alias, restarted)) =
                inspect_publication(output, &before, &initial.resident_music_reply)
            {
                aliases.push(alias);
                restarts += u64::from(restarted);
            }
        }
        outputs.extend(step);
        resumed = roundtrip(&resumed);
        assert_eq!(resumed.resident_music_reply, initial.resident_music_reply);
        assert_eq!(
            transport::advance(&mut resumed, &content, time::cycles(0), &[]),
            Vec::new()
        );
    }
    assert_eq!(aliases, ints(row, "expected_reply_aliases"));
    assert_eq!(restarts, unsigned(row, "after_restarts"));
    let mut direct = initial;
    let elapsed = time::subtract(&resumed.now, &direct.now);
    assert_eq!(
        transport::advance(&mut direct, &content, elapsed, &[]),
        outputs
    );
    assert_eq!(direct, resumed);
}
fn inspect_publication(
    output: &transport::Output,
    before: &SongState,
    resident: &ResidentMusicReply,
) -> Option<(i64, bool)> {
    let transport::Output::Publication {
        line: ReplyLine::Music,
        reply,
        result,
        ..
    } = output
    else {
        return None;
    };
    assert_eq!(reply, &Reply::Music(resident.clone()));
    let Reply::Music(identity) = reply else {
        assert!(false);
        return None;
    };
    assert_eq!(identity.identity, resident.identity);
    let CompletionResult::Music(after) = result else {
        assert!(false);
        return None;
    };
    let restarted = before.tracks[0].note.0.get() == 1
        && before.tracks[0].position == (TrackPosition::Root { cursor: 1 });
    if restarted {
        assert_eq!(after.tracks[0].position, TrackPosition::Root { cursor: 1 });
    }
    Some((i64::from(identity.original_byte), restarted))
}
#[test]
fn resident_encoding_is_not_the_semantic_identity() {
    let root: Table = toml::from_str(SOURCE).expect("vectors");
    let row = array(&root, "resident_reply")[0]
        .as_table()
        .expect("resident content");
    let (song, tracks) = song(row, &root["fixture_mix"]);
    let content = Content {
        effects: BTreeMap::new(),
        tracks,
    };
    for original_byte in 0..=u8::MAX {
        let resident = ResidentMusicReply {
            identity: "independent caller identity".into(),
            original_byte,
        };
        let mut state = state_fixture(song.clone(), 0, 0);
        state.resident_music_reply = resident.clone();
        state.phase = AudioPhase::Work {
            at: time::cycles(1),
            result: CompletionResult::Music(song.clone()),
        };
        state = roundtrip(&state);
        let output = transport::advance(&mut state, &content, time::cycles(1), &[]);
        assert_eq!(
            output,
            vec![transport::Output::Publication {
                at: time::cycles(1),
                line: ReplyLine::Music,
                reply: Reply::Music(resident.clone()),
                result: CompletionResult::Music(song.clone())
            }]
        );
        assert_eq!(state.resident_music_reply, resident);
        assert_eq!(roundtrip(&state), state);
    }
}
fn next_milestone(phase: &AudioPhase) -> AudioInstant {
    match phase {
        AudioPhase::ObserveClock { at }
        | AudioPhase::Distribute { at, .. }
        | AudioPhase::Work { at, .. }
        | AudioPhase::MusicStart { at }
        | AudioPhase::Idle { at } => at.clone(),
        AudioPhase::Sample { sample, .. } => time::add(
            &sample.anchor,
            time::halves(if sample.first.is_some() { 19 } else { 9 }),
        ),
    }
}
