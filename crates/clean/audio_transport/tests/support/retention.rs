//! Spec: `EffectResources` and `EffectTiming`. Complete admitted histories and echo results.
use super::*;

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
    let r = value.as_table().expect("span");
    keys(r, "duration pitch timbre motion");
    EffectSpan {
        duration: EffectTurns(nz(byte(r, "duration"))),
        pitch: ep(byte(r, "pitch")),
        timbre: TimbreId(u16::try_from(integer(r, "timbre")).expect("timbre")),
        motion: r.get("motion").map(|v| {
            let m = v.as_table().expect("motion");
            keys(m, "speed target");
            PitchMotion {
                speed: nz(byte(m, "speed")),
                target: ep(byte(m, "target")),
            }
        }),
    }
}
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
fn initial(root: &Table, retained: ReleaseHistory) -> State {
    let (song, _) = song(
        array(root, "progress")[0].as_table().expect("song"),
        &root["fixture_mix"],
    );
    let mut state = state_fixture(song, 0, 0);
    state.groups[SoundGroup::Contact] = GroupState {
        history: retained,
        handled: Request::Cue(CueId::Footstep),
        staged: Request::Cue(CueId::Footstep),
        reply: Request::Absent,
        playback: Playback::Prepared {
            cue: CueId::Footstep,
        },
    };
    state
}
#[test]
fn initialization_and_restore_preserve_caller_supplied_dormant_history() {
    let root: Table = toml::from_str(SOURCE).expect("vectors");
    for count in 0u8..=255 {
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
            let mut state = initial(&root, vec![ReleaseMemory::Inactive, retained]);
            state.groups[SoundGroup::Contact].playback = playback;
            state.groups[SoundGroup::Action].history = vec![retained];
            state.groups[SoundGroup::Ambient].history = vec![retained];
            let groups = state.groups.clone();
            transport::initialize(&mut state);
            assert_eq!(state.groups, groups);
            assert_eq!(roundtrip(&state), state);
        }
    }
}
fn completion(result: &effects::EffectResult) -> CompletionResult {
    CompletionResult::Effect {
        group: SoundGroup::Contact,
        result: result.group.clone(),
        echo_voices: result.echo_voices.clone(),
    }
}
// Start through the actual dispatcher, then stop and restore on both sides of publication.
fn publish_turn(state: &mut State, content: &Content, expected: i64) -> effects::EffectResult {
    let before = state.groups[SoundGroup::Contact].clone();
    let mix = state.song.mix.clone();
    let reservations = state.reservations.clone();
    let result = effects::turn(
        SoundGroup::Contact,
        &before,
        content,
        &reservations,
        &mix.echo_voices,
    );
    assert_eq!(result.work.0.to_i64().expect("work"), expected);
    let anchor = state.now.clone();
    state.phase = AudioPhase::Sample {
        sample: StableSample {
            line: ReplyLine::Group(SoundGroup::Ambient),
            anchor: anchor.clone(),
            first: Some(Request::Absent),
        },
        next: Continuation::Effect(SoundGroup::Contact),
    };
    let initial = roundtrip(state);
    let mut split_output = transport::advance(state, content, time::halves(19), &[]);
    let at = time::add(&anchor, time::whole(result.work.0.clone()));
    assert_eq!(
        state.phase,
        AudioPhase::Work {
            at: at.clone(),
            result: completion(&result),
        }
    );
    *state = roundtrip(state);
    assert_eq!(
        transport::advance(state, content, time::cycles(0), &[]),
        Vec::new()
    );
    let remaining = time::subtract(time::subtract(&at, &state.now), time::halves(1));
    assert_eq!(
        transport::advance(state, content, remaining, &[]),
        Vec::new()
    );
    assert_eq!(state.groups[SoundGroup::Contact], before);
    assert_eq!(state.song.mix, mix);
    *state = roundtrip(state);
    let output = transport::advance(state, content, time::halves(1), &[]);
    assert_eq!(
        output,
        vec![transport::Output::Publication {
            at,
            line: ReplyLine::Group(SoundGroup::Contact),
            reply: Reply::Effect(result.group.reply),
            result: completion(&result),
        }]
    );
    split_output.extend(output);
    let mut direct = initial;
    assert_eq!(
        transport::advance(
            &mut direct,
            content,
            time::whole(result.work.0.clone()),
            &[]
        ),
        split_output
    );
    assert_eq!(*state, direct);
    assert_eq!(state.groups[SoundGroup::Contact], result.group);
    let mut completed_mix = mix;
    completed_mix.echo_voices.clone_from(&result.echo_voices);
    assert_eq!(state.song.mix, completed_mix);
    assert_eq!(state.reservations, reservations);
    assert_eq!(
        transport::advance(state, content, time::cycles(0), &[]),
        Vec::new()
    );
    result
}
pub fn history_vector(row: &Table, root: &Table) {
    keys(
        row,
        "id initial_history prior_parts steps work history owned prior_span one_span door_spans door_external_voice wrong_final_allocation_work",
    );
    let prior = span(&row["prior_span"]);
    let content = Content {
        effects: BTreeMap::from([
            (
                CueId::Footstep,
                clip(vec![
                    prior;
                    usize::try_from(integer(row, "prior_parts"))
                        .expect("parts")
                ]),
            ),
            (CueId::Land, clip(vec![span(&row["one_span"])])),
            (
                CueId::Door,
                clip(array(row, "door_spans").iter().map(span).collect()),
            ),
        ]),
        tracks: Vec::new(),
    };
    let mut state = initial(root, history(array(row, "initial_history")));
    let steps = array(row, "steps");
    assert_eq!(steps.len(), array(row, "work").len());
    assert_eq!(steps.len(), array(row, "history").len());
    assert_eq!(steps.len(), array(row, "owned").len());
    for (i, step) in steps.iter().enumerate() {
        match step.as_str().expect("step") {
            "AllocatePrior" => assert_eq!(i, 0),
            "Advance" => {}
            "PrepareOne" => state.groups[SoundGroup::Contact].staged = Request::Cue(CueId::Land),
            "PrepareDoor" => {
                state.groups[SoundGroup::Contact].staged = Request::Cue(CueId::Door);
                state
                    .reservations
                    .insert(effect_voice(integer(row, "door_external_voice")));
            }
            other => assert_eq!(other, "known step"),
        }
        let saved = &state.groups[SoundGroup::Contact];
        if matches!(saved.playback, Playback::Prepared { cue: CueId::Door }) {
            let mut lost = saved.clone();
            lost.history[1] = ReleaseMemory::Inactive;
            let wrong = effects::turn(
                SoundGroup::Contact,
                &lost,
                &content,
                &state.reservations,
                &state.song.mix.echo_voices,
            );
            assert_eq!(
                wrong.work,
                AudioCycles(unsigned(row, "wrong_final_allocation_work"))
            );
            assert_ne!(
                (wrong.work, wrong.group.history),
                (
                    AudioCycles(u64::try_from(ints(row, "work")[i]).expect("work")),
                    history(array(row, "history")[i].as_array().expect("history"))
                )
            );
        }
        publish_turn(&mut state, &content, ints(row, "work")[i]);
        assert_eq!(
            state.groups[SoundGroup::Contact].history,
            history(array(row, "history")[i].as_array().expect("history")),
            "{} step {i}",
            text(row, "id")
        );
        assert_eq!(
            effects::owned(&state.groups[SoundGroup::Contact].playback).len(),
            usize::try_from(ints(row, "owned")[i]).expect("owned")
        );
    }
}
pub fn echo_vector(row: &Table, root: &Table) {
    keys(
        row,
        "id initial_echo external_voices parts span release_history work after_echo after_voices reselect_echo",
    );
    let mut state = initial(root, history(array(row, "release_history")));
    state.song.mix.echo_voices = ints(row, "initial_echo")
        .into_iter()
        .map(super::super::voice)
        .collect();
    state.reservations = ints(row, "external_voices")
        .into_iter()
        .map(effect_voice)
        .collect();
    let mut content = Content {
        effects: BTreeMap::from([(
            CueId::Footstep,
            clip(vec![
                span(&row["span"]);
                usize::try_from(integer(row, "parts")).expect("parts")
            ]),
        )]),
        tracks: Vec::new(),
    };
    for (i, work) in ints(row, "work").into_iter().enumerate() {
        publish_turn(&mut state, &content, work);
        assert_eq!(
            state.song.mix.echo_voices,
            ints(row, "after_echo")
                .into_iter()
                .map(super::super::voice)
                .collect()
        );
        if i == 0 {
            let Playback::Active { parts, .. } = &state.groups[SoundGroup::Contact].playback else {
                assert!(false);
                return;
            };
            let voices: Vec<_> = parts
                .iter()
                .filter_map(|p| match p {
                    PartState::Live(l) => Some(l.voice),
                    PartState::Finished => None,
                })
                .collect();
            assert_eq!(
                voices,
                ints(row, "after_voices")
                    .into_iter()
                    .map(effect_voice)
                    .collect::<Vec<_>>()
            );
        }
    }
    assert!(effects::owned(&state.groups[SoundGroup::Contact].playback).is_empty());
    reselect_echo(row, &mut state, &mut content);
}
fn reselect_echo(row: &Table, state: &mut State, content: &mut Content) {
    let reservations = state.reservations.clone();
    let groups = state.groups.clone();
    let selected = ints(row, "reselect_echo")
        .into_iter()
        .map(super::super::voice)
        .collect();
    content.tracks = vec![
        TrackContent {
            phrases: vec![vec![
                MusicEvent::EchoMix {
                    voices: selected,
                    left: 1,
                    right: 1
                },
                MusicEvent::Tone(mp(24)),
                MusicEvent::Complete
            ]]
        };
        2
    ];
    for track in &mut state.song.tracks {
        track.note = MusicTurns(nz(1));
        track.position = TrackPosition::Root { cursor: 0 };
    }
    state.music_due = true;
    state.phase = AudioPhase::MusicStart {
        at: state.now.clone(),
    };
    let result = music::turn(&state.song, &content.tracks);
    transport::advance(state, content, time::whole(result.work.0), &[]);
    assert_eq!(
        state.song.mix.echo_voices,
        ints(row, "reselect_echo")
            .into_iter()
            .map(super::super::voice)
            .collect()
    );
    assert_eq!(state.reservations, reservations);
    assert_eq!(state.groups, groups);
}
