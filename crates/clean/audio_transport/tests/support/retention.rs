//! Spec: EffectResources. Complete release histories and echo membership across effect turns.
use super::*;
const GROUP: SoundGroup = SoundGroup::Contact;
fn history(values: &[Value]) -> ReleaseHistory {
    values
        .iter()
        .map(|v| {
            let count = u8::try_from(v.as_integer().expect("history count")).expect("byte");
            NonZeroU8::new(count).map_or(ReleaseMemory::Inactive, ReleaseMemory::Active)
        })
        .collect()
}
fn span(value: &Value) -> EffectSpan {
    let r = table(value);
    keys(r, "duration pitch timbre motion");
    EffectSpan {
        duration: EffectTurns(nz(byte(r, "duration"))),
        pitch: ep(byte(r, "pitch")),
        timbre: TimbreId(u16::try_from(integer(r, "timbre")).expect("timbre")),
        motion: r.get("motion").map(|v| {
            let m = table(v);
            keys(m, "speed target");
            PitchMotion {
                speed: nz(byte(m, "speed")),
                target: ep(byte(m, "target")),
            }
        }),
    }
}
/// A clip of one single-span part per span.
fn clip(spans: Vec<EffectSpan>) -> EffectClip {
    EffectClip {
        parts: spans
            .into_iter()
            .map(|s| EffectPartContent {
                spans: vec![s],
                loop_start: None,
            })
            .collect(),
    }
}
/// The first progress vector's song, with Contact's Footstep prepared and steady on its line.
fn prepared_footstep(root: &Table, retained: ReleaseHistory) -> State {
    let (song, _) = song(table(&array(root, "progress")[0]), &root["fixture_mix"]);
    let mut state = fresh_state(song);
    state.groups[GROUP] = GroupState {
        history: retained,
        handled: Request::Cue(CueId::Footstep),
        staged: Request::Cue(CueId::Footstep),
        reply: Request::Absent,
        playback: Playback::Prepared {
            cue: CueId::Footstep,
        },
    };
    state.request_lines[GROUP] = Request::Cue(CueId::Footstep);
    state
}
/// A changed cue, presented and staged on earlier turns, for the next turn to handle.
fn stage(state: &mut State, cue: CueId) {
    state.request_lines[GROUP] = Request::Cue(cue);
    state.groups[GROUP].staged = Request::Cue(cue);
}
pub fn history_vector(row: &Table, root: &Table) {
    keys(
        row,
        "id initial_history prior_parts steps history owned prior_span one_span door_spans door_external_voice",
    );
    let content = content(
        BTreeMap::from([
            (
                CueId::Footstep,
                clip(vec![span(&row["prior_span"]); index(row, "prior_parts")]),
            ),
            (CueId::Land, clip(vec![span(&row["one_span"])])),
            (
                CueId::Door,
                clip(array(row, "door_spans").iter().map(span).collect()),
            ),
        ]),
        Vec::new(),
    );
    let mut state = prepared_footstep(root, history(array(row, "initial_history")));
    let steps = array(row, "steps");
    let histories = array(row, "history");
    let owned = ints(row, "owned");
    assert_eq!(steps.len(), histories.len());
    assert_eq!(steps.len(), owned.len());
    for (i, step) in steps.iter().enumerate() {
        match step.as_str().expect("step") {
            "AllocatePrior" => assert_eq!(i, 0),
            "Advance" => {}
            "PrepareOne" => stage(&mut state, CueId::Land),
            other => {
                assert_eq!(other, "PrepareDoor");
                stage(&mut state, CueId::Door);
                state
                    .reservations
                    .insert(effect_voice(integer(row, "door_external_voice")));
            }
        }
        play(&mut state, &content, &[], ONE_EFFECT_TURN);
        let audio = &state.groups[GROUP];
        let label = format!("{} step {i}", text(row, "id"));
        assert_eq!(
            audio.history,
            history(histories[i].as_array().expect("history")),
            "{label}"
        );
        assert_eq!(
            i64::try_from(audio.playback.owned_voices().len()).expect("owned"),
            owned[i],
            "{label}"
        );
    }
}
pub fn echo_vector(row: &Table, root: &Table) {
    keys(
        row,
        "id initial_echo external_voices parts span release_history turns after_echo after_voices reselect_echo",
    );
    let mut state = prepared_footstep(root, history(array(row, "release_history")));
    state.song.mix.echo_voices = voices(row, "initial_echo");
    state.reservations = effect_voices(row, "external_voices");
    let effects = BTreeMap::from([(
        CueId::Footstep,
        clip(vec![span(&row["span"]); index(row, "parts")]),
    )]);
    let effect_content = content(effects.clone(), Vec::new());
    for turn in 0..unsigned(row, "turns") {
        play(&mut state, &effect_content, &[], ONE_EFFECT_TURN);
        assert_eq!(state.song.mix.echo_voices, voices(row, "after_echo"));
        if turn == 0 {
            let acquired: Vec<_> = live_parts(&state.groups[GROUP].playback)
                .into_iter()
                .map(|(_, live)| live.voice)
                .collect();
            let expected: Vec<_> = ints(row, "after_voices")
                .into_iter()
                .map(effect_voice)
                .collect();
            assert_eq!(acquired, expected);
        }
    }
    assert!(state.groups[GROUP].playback.owned_voices().is_empty());
    reselect_echo(row, &mut state, effects);
}
/// After playback finishes, a music turn's explicit echo selection restores the membership; it
/// changes no effect ownership or reservation.
fn reselect_echo(row: &Table, state: &mut State, effects: BTreeMap<CueId, EffectClip>) {
    let reservations = state.reservations.clone();
    let groups = state.groups.clone();
    let selecting = TrackContent {
        phrases: vec![vec![
            MusicEvent::EchoMix {
                voices: voices(row, "reselect_echo"),
                left: 1,
                right: 1,
            },
            MusicEvent::Tone(mp(24)),
            MusicEvent::Complete,
        ]],
    };
    for track in &mut state.song.tracks {
        track.note = MusicTurns(nz(1));
        track.position = TrackPosition::Root { cursor: 0 };
    }
    let music_content = content(effects, vec![selecting; 2]);
    play(state, &music_content, &[], ONE_MUSIC_TURN);
    assert_eq!(state.song.mix.echo_voices, voices(row, "reselect_echo"));
    assert_eq!(state.reservations, reservations);
    assert_eq!(state.groups, groups);
}
#[test]
fn restoring_and_idle_frames_keep_caller_supplied_dormant_history() {
    let root: Table =
        toml::from_str(&vector_file::read("audio-transport").unwrap()).expect("vectors");
    let content = content(BTreeMap::new(), Vec::new());
    for count in 0u8..=u8::MAX {
        let retained = NonZeroU8::new(count).map_or(ReleaseMemory::Inactive, ReleaseMemory::Active);
        for playback in [
            Playback::Empty,
            Playback::Prepared {
                cue: CueId::Footstep,
            },
            Playback::Active {
                cue: CueId::Footstep,
                parts: vec![PartState::Finished],
            },
        ] {
            let mut state = prepared_footstep(&root, vec![ReleaseMemory::Inactive, retained]);
            state.groups[GROUP].playback = playback;
            state.groups[SoundGroup::Action].history = vec![retained];
            state.groups[SoundGroup::Ambient].history = vec![retained];
            let groups = state.groups.clone();
            play(&mut state, &content, &[], SERVICE_ONLY);
            assert_eq!(state.groups, groups);
        }
    }
}
