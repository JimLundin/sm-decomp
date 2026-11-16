//! Invented-content importer fixtures and complete ordered vector checks.
#![expect(
    clippy::expect_used,
    reason = "Malformed invented vectors must fail loudly"
)]
#![expect(
    clippy::indexing_slicing,
    reason = "Fixture bounds are asserted by vector decoding"
)]
#![expect(
    clippy::assertions_on_constants,
    reason = "Explicit fixture decoding failure, without a panic macro"
)]
#![expect(
    clippy::arithmetic_side_effects,
    reason = "Tests compare independent exact arithmetic formulas"
)]
mod support;
use audio_transport::*;
use audio_transport::{effects, music, sender, time, transport};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::num::{NonZeroU8, NonZeroU64};
use support::{effect_vector, staged_vector};
use toml::{Table, Value};
const SOURCE: &str = include_str!("../../../../scenarios/vectors/audio-transport.toml");
const fn nz(value: u8) -> NonZeroU8 {
    NonZeroU8::new(value).expect("positive fixture count")
}
const fn mp(value: u8) -> MusicalPitch {
    MusicalPitch::new(value).expect("admitted musical pitch")
}
const fn ep(value: u8) -> EffectPitch {
    EffectPitch::new(value).expect("admitted effect pitch")
}
fn integer(row: &Table, key: &str) -> i64 {
    row.get(key).and_then(Value::as_integer).expect(key)
}
fn optional(row: &Table, key: &str, fallback: i64) -> i64 {
    row.get(key).and_then(Value::as_integer).unwrap_or(fallback)
}
fn unsigned(row: &Table, key: &str) -> u64 {
    u64::try_from(integer(row, key)).expect(key)
}
fn byte(row: &Table, key: &str) -> u8 {
    u8::try_from(integer(row, key)).expect(key)
}
fn text<'a>(row: &'a Table, key: &str) -> &'a str {
    row.get(key).and_then(Value::as_str).expect(key)
}
fn flag(row: &Table, key: &str) -> bool {
    row.get(key).and_then(Value::as_bool).expect(key)
}
fn array<'a>(row: &'a Table, key: &str) -> &'a [Value] {
    row.get(key).and_then(Value::as_array).expect(key)
}
fn ints(row: &Table, key: &str) -> Vec<i64> {
    array(row, key)
        .iter()
        .map(|v| v.as_integer().expect(key))
        .collect()
}
fn keys(row: &Table, admitted: &str) {
    for key in row.keys() {
        assert!(
            admitted.split_whitespace().any(|known| known == key),
            "unknown vector key {key}"
        );
    }
}
fn cue(value: i64) -> CueId {
    match value {
        7 => CueId::Spin,
        9 => CueId::EndSpin,
        _ => {
            assert_eq!(value, 0);
            CueId::SamusSilence
        }
    }
}
fn request(value: i64) -> Request {
    if value == 0 {
        Request::Absent
    } else {
        Request::Cue(cue(value))
    }
}
fn cue_name(value: &str) -> Request {
    match value {
        "Absent" => Request::Absent,
        "Footstep" => Request::Cue(CueId::Footstep),
        "Door" => Request::Cue(CueId::Door),
        _ => {
            assert_eq!(value, "Absent");
            Request::Absent
        }
    }
}
fn group_name(value: &str) -> SoundGroup {
    match value {
        "Action" => SoundGroup::Action,
        "Ambient" => SoundGroup::Ambient,
        _ => {
            assert_eq!(value, "Contact");
            SoundGroup::Contact
        }
    }
}
fn effect_voice(value: i64) -> EffectVoiceId {
    match value {
        7 => EffectVoiceId::V7,
        6 => EffectVoiceId::V6,
        5 => EffectVoiceId::V5,
        4 => EffectVoiceId::V4,
        3 => EffectVoiceId::V3,
        _ => {
            assert_eq!(value, 2);
            EffectVoiceId::V2
        }
    }
}
fn voice(value: i64) -> VoiceId {
    match value {
        0 => VoiceId::FirstMusic,
        1 => VoiceId::SecondMusic,
        _ => VoiceId::Effect(effect_voice(value)),
    }
}
fn decode_event(v: &Value) -> MusicEvent {
    let row = v.as_table().expect("event");
    keys(
        row,
        "kind duration extra_handling_cycles articulation pitch value turns target identity fraction voices left right delay feedback filter phrase count",
    );
    match text(row, "kind") {
        "Timing" => MusicEvent::Timing {
            duration: MusicTurns(nz(byte(row, "duration"))),
            articulation: row.get("articulation").map(|value| {
                let r = value.as_table().expect("articulation");
                keys(r, "ratio strength");
                Articulation {
                    ratio: byte(r, "ratio"),
                    strength: byte(r, "strength"),
                }
            }),
            extra: ExtraHandlingWork::new(
                u64::try_from(optional(row, "extra_handling_cycles", 0)).expect("extra work"),
            ),
        },
        "Tone" => MusicEvent::Tone(mp(byte(row, "pitch"))),
        "Sustain" => MusicEvent::Sustain,
        "Loudness" => MusicEvent::Loudness(byte(row, "value")),
        "Ramp" => MusicEvent::Ramp {
            turns: nz(byte(row, "turns")),
            target: byte(row, "target"),
        },
        "MasterLoudness" => MusicEvent::MasterLoudness(byte(row, "value")),
        "Timbre" => MusicEvent::Timbre(TimbreId(
            u16::try_from(integer(row, "identity")).expect("timbre"),
        )),
        "PitchOffset" => {
            assert_eq!(integer(row, "fraction"), 0);
            MusicEvent::Offset(PitchOffset::Zero)
        }
        "Percussion" => {
            MusicEvent::Percussion(u16::try_from(integer(row, "identity")).expect("percussion"))
        }
        "EchoMix" => MusicEvent::EchoMix {
            voices: ints(row, "voices").into_iter().map(voice).collect(),
            left: byte(row, "left"),
            right: byte(row, "right"),
        },
        "EchoSpace" => MusicEvent::EchoSpace {
            delay: byte(row, "delay"),
            feedback: byte(row, "feedback"),
            filter: u16::try_from(integer(row, "filter")).expect("filter"),
        },
        "RepeatPhrase" => MusicEvent::Repeat {
            phrase: usize::try_from(integer(row, "phrase")).expect("phrase"),
            count: nz(byte(row, "count")),
        },
        other => {
            assert_eq!(other, "CompletePhrase");
            MusicEvent::Complete
        }
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TrackFixture {
    note: u8,
    release: u8,
    duration: u8,
    articulation: u8,
    pitch: u8,
    loudness: u16,
    position: PositionFixture,
    repeat: RepeatFixture,
    ramp: RampFixture,
    phrases: Vec<Vec<Value>>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PositionFixture {
    phrase: usize,
    cursor: usize,
}
#[derive(Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
enum RepeatFixture {
    Inactive,
    Active { count: u8, return_cursor: usize },
}
#[derive(Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
enum RampFixture {
    Inactive,
    Active {
        remaining: u8,
        target: u8,
        delta: i16,
    },
}
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct MixFixture {
    percussion_set: u16,
    master_loudness: u16,
    echo_voices: BTreeSet<u8>,
    echo_left: u16,
    echo_right: u16,
    echo_delay: u8,
    feedback: u8,
    filter_set: u16,
    echo_writes_enabled: bool,
    timbres: Vec<u16>,
    tone_offsets: Vec<i32>,
    note_strengths: Vec<u8>,
}
fn song(row: &Table, fixture_mix: &Value) -> (SongState, Vec<TrackContent>) {
    let mix: MixFixture = row
        .get("mix")
        .unwrap_or(fixture_mix)
        .clone()
        .try_into()
        .expect("complete fixture mix");
    let mut tracks = Vec::new();
    let mut graphs = Vec::new();
    for (index, value) in array(row, "tracks").iter().enumerate() {
        let f: TrackFixture = value.clone().try_into().expect("complete track fixture");
        let position = match f.repeat {
            RepeatFixture::Inactive => {
                assert_eq!(f.position.phrase, 0);
                TrackPosition::Root {
                    cursor: f.position.cursor,
                }
            }
            RepeatFixture::Active {
                count,
                return_cursor,
            } => TrackPosition::Repeated {
                phrase: f.position.phrase,
                cursor: f.position.cursor,
                remaining: nz(count),
                continuation: return_cursor,
            },
        };
        let ramp = match f.ramp {
            RampFixture::Inactive => None,
            RampFixture::Active {
                remaining,
                target,
                delta,
            } => Some(LoudnessRamp {
                remaining: nz(remaining),
                target,
                increment: delta,
            }),
        };
        assert_eq!(mix.tone_offsets[index], 0);
        tracks.push(TrackState {
            position,
            note: MusicTurns(nz(f.note)),
            release: f.release,
            duration: MusicTurns(nz(f.duration)),
            articulation: f.articulation,
            pitch: mp(f.pitch),
            loudness: Loudness(f.loudness),
            ramp,
            timbre: TimbreId(mix.timbres[index]),
            offset: PitchOffset::Zero,
            strength: mix.note_strengths[index],
        });
        graphs.push(TrackContent {
            phrases: f
                .phrases
                .iter()
                .map(|p| p.iter().map(decode_event).collect())
                .collect(),
        });
    }
    (
        SongState {
            tracks,
            mix: SongMix {
                master: Loudness(mix.master_loudness),
                percussion: mix.percussion_set,
                echo_voices: mix
                    .echo_voices
                    .into_iter()
                    .map(|v| voice(i64::from(v)))
                    .collect(),
                echo_left: mix.echo_left,
                echo_right: mix.echo_right,
                echo_delay: mix.echo_delay,
                feedback: mix.feedback,
                filter: mix.filter_set,
                echo_enabled: mix.echo_writes_enabled,
            },
        },
        graphs,
    )
}
fn sender_progress(kind: &str, value: i64) -> SenderProgress {
    match kind {
        "Ready" => SenderProgress::Ready,
        "AwaitCue" => SenderProgress::AwaitCue(cue(value)),
        "ClearDelay" => SenderProgress::ClearDelay(nz(u8::try_from(value).expect("delay"))),
        "AwaitClear" => SenderProgress::AwaitClear,
        other => {
            assert_eq!(other, "ResetDelay");
            SenderProgress::ResetDelay(nz(u8::try_from(value).expect("reset")))
        }
    }
}
fn sender_vector(row: &Table) {
    keys(
        row,
        "id state value queue reply after_state after_value after_queue present work_master_ticks read_master_ticks present_master_ticks",
    );
    let mut state = SenderState {
        queue: ints(row, "queue").into_iter().map(cue).collect(),
        progress: sender_progress(text(row, "state"), integer(row, "value")),
        reply: request(integer(row, "reply")),
        now: CpuMasterTicks(0),
        pending: None,
    };
    sender::begin(&mut state, sender::SenderProfile::FastBus);
    let saved = state.clone();
    let work = unsigned(row, "work_master_ticks");
    let mut outputs = sender::advance(&mut state, CpuMasterTicks(work.saturating_sub(1)));
    assert!(state.pending.is_some());
    let mut restored = state.clone();
    assert_eq!(
        sender::advance(&mut restored, CpuMasterTicks(0)),
        Vec::new()
    );
    outputs.extend(sender::advance(&mut restored, CpuMasterTicks(1)));
    let mut direct = saved;
    assert_eq!(outputs, sender::advance(&mut direct, CpuMasterTicks(work)));
    assert_eq!(restored, direct);
    assert_eq!(
        direct.progress,
        sender_progress(text(row, "after_state"), integer(row, "after_value"))
    );
    assert_eq!(
        direct.queue,
        ints(row, "after_queue")
            .into_iter()
            .map(cue)
            .collect::<VecDeque<_>>()
    );
    let mut expected = Vec::new();
    if integer(row, "read_master_ticks") >= 0 {
        expected.push(sender::SenderOutput::Observation(
            CpuMasterTicks(unsigned(row, "read_master_ticks")),
            request(integer(row, "reply")),
        ));
    }
    if integer(row, "present_master_ticks") >= 0 {
        expected.push(sender::SenderOutput::Presentation(
            CpuMasterTicks(unsigned(row, "present_master_ticks")),
            request(integer(row, "present")),
        ));
    }
    expected.push(sender::SenderOutput::Completed(CpuMasterTicks(work)));
    assert_eq!(outputs, expected, "{}", text(row, "id"));
}
fn offer_vector(row: &Table) {
    keys(
        row,
        "id queue cue allowance enabled demo exploding after_queue",
    );
    let mut state = SenderState {
        queue: ints(row, "queue").into_iter().map(cue).collect(),
        progress: SenderProgress::Ready,
        reply: Request::Absent,
        now: CpuMasterTicks(0),
        pending: None,
    };
    let permission = if !flag(row, "enabled") {
        OfferPermission::Disabled
    } else if flag(row, "demo") {
        OfferPermission::Demo
    } else if flag(row, "exploding") {
        OfferPermission::Explosion
    } else {
        OfferPermission::Enabled
    };
    let allowance = match integer(row, "allowance") {
        1 => QueueAllowance::One,
        3 => QueueAllowance::Three,
        6 => QueueAllowance::Six,
        9 => QueueAllowance::Nine,
        other => {
            assert_eq!(other, 15);
            QueueAllowance::Fifteen
        }
    };
    sender::offer(&mut state, cue(integer(row, "cue")), allowance, permission);
    assert_eq!(
        state.queue,
        ints(row, "after_queue")
            .into_iter()
            .map(cue)
            .collect::<VecDeque<_>>()
    );
}
fn cadence_vector(row: &Table) {
    keys(
        row,
        "id effect_phase music_phase overflows after_effect_phase after_music_phase effect_due music_due",
    );
    let mut phase = CadencePhase {
        effect: byte(row, "effect_phase"),
        music: byte(row, "music_phase"),
    };
    let due = transport::distribute(&mut phase, byte(row, "overflows"));
    assert_eq!(due, (flag(row, "effect_due"), flag(row, "music_due")));
    assert_eq!(
        phase,
        CadencePhase {
            effect: byte(row, "after_effect_phase"),
            music: byte(row, "after_music_phase")
        }
    );
}
fn empty_group(parts: usize) -> GroupState {
    GroupState {
        history: vec![ReleaseMemory::Inactive; parts],
        handled: Request::Absent,
        staged: Request::Absent,
        reply: Request::Absent,
        playback: Playback::Empty,
    }
}
fn state_fixture(song: SongState, effect: u8, music: u8) -> State {
    State {
        now: time::cycles(0),
        clock: ClockState {
            phase: AudioCyclePhase::try_from(time::cycles(531)).expect("phase inside timer period"),
            unread: 0,
            lattice: time::halves(811),
        },
        cadence: CadencePhase { effect, music },
        music_due: false,
        groups: enum_map::enum_map! { SoundGroup::Action => empty_group(1), SoundGroup::Ambient => empty_group(1), SoundGroup::Contact => empty_group(2) },
        song,
        resident_music_reply: ResidentMusicReply {
            identity: "invented legacy arrangement".into(),
            original_byte: 0,
        },
        reservations: BTreeSet::new(),
        requests: enum_map::enum_map! { SoundGroup::Action => Request::Absent, SoundGroup::Ambient => Request::Absent, SoundGroup::Contact => Request::Absent },
        phase: AudioPhase::ObserveClock {
            at: time::halves(811),
        },
    }
}
fn roundtrip(state: &State) -> State {
    let value = Value::try_from(state).expect("State serialization");
    value.try_into().expect("State restoration")
}
fn musical_vector(row: &Table, family: &str, mix: &Value) {
    keys(
        row,
        "id kind tracks mix remaining after_remaining after_position after_phrase after_cursor after_note after_release after_pitch volume target length delta after_volume after_remaining note gate duration ratio after_gate root_completed other_note_remaining other_repeat_remaining other_ramp_remaining master_loudness after_other_repeat_remaining after_other_ramp_remaining after_master_loudness intro_first_duration intro_second_duration intro_first_ramp_length intro_first_ramp_target first_volume second_volume after_first_note after_second_note after_first_volume after_second_volume after_first_ramp_remaining value after_value voices left right after_left after_right after_enabled delay feedback filter after_delay after_feedback after_filter ramp_remaining ramp_target ramp_delta after_ramp_remaining strength extra work reserved_voices after_reserved_voices",
    );
    let (before, graph) = song(row, mix);
    check_song_input_fields(row, family, &before, &graph);
    let result = music::turn(&before, &graph);
    let track = &result.song.tracks[0];
    match family {
        "music_work" => check_music_work(row, &result.work),
        "progress" => check_music_progress(row, track),
        "phrase" => check_phrase_progress(row, track),
        "ramp" => check_loudness_ramp(row, &before.tracks[0], track),
        "restart" => check_restart(row, &result.song),
        other => {
            assert_eq!(other, "control");
            check_control(row, &result.song);
        }
    }
    // Actual entry point pending result, full serialization, no early commitment.
    let content = Content {
        effects: BTreeMap::new(),
        tracks: graph,
    };
    let mut state = state_fixture(before.clone(), 0, 0);
    if row.contains_key("reserved_voices") {
        state.reservations = ints(row, "reserved_voices")
            .into_iter()
            .map(effect_voice)
            .collect();
    }
    state.phase = AudioPhase::Work {
        at: time::whole(result.work.0.clone()),
        result: CompletionResult::Music(result.song.clone()),
    };
    let outputs = transport::advance(
        &mut state,
        &content,
        time::whole(&result.work.0 - 1u64),
        &[],
    );
    assert!(
        !outputs
            .iter()
            .any(|o| matches!(o, transport::Output::Publication { .. }))
    );
    assert_eq!(state.song, before);
    let mut restored = roundtrip(&state);
    assert_eq!(
        transport::advance(&mut restored, &content, time::cycles(0), &[]),
        Vec::new()
    );
    let final_output = transport::advance(&mut restored, &content, time::cycles(1), &[]);
    assert_eq!(
        final_output,
        vec![transport::Output::Publication {
            at: time::whole(result.work.0.clone()),
            line: ReplyLine::Music,
            reply: Reply::Music(restored.resident_music_reply.clone()),
            result: CompletionResult::Music(result.song.clone())
        }]
    );
    assert_eq!(restored.song, result.song);
    if row.contains_key("after_reserved_voices") {
        assert_eq!(
            restored.reservations,
            ints(row, "after_reserved_voices")
                .into_iter()
                .map(effect_voice)
                .collect()
        );
    }
}
fn check_music_work(row: &Table, work: &ExactAudioCycles) {
    assert_eq!(
        *work,
        AudioCycles(unsigned(row, "work")),
        "{}",
        text(row, "id")
    );
}
fn check_music_progress(row: &Table, track: &TrackState) {
    assert_eq!(track.note.0.get(), byte(row, "after_note"));
    assert_eq!(track.release, byte(row, "after_gate"));
}
fn check_phrase_progress(row: &Table, track: &TrackState) {
    let (phrase, cursor, remaining) = match track.position {
        TrackPosition::Root { cursor } => (0, cursor, 0),
        TrackPosition::Repeated {
            phrase,
            cursor,
            remaining,
            ..
        } => (phrase, cursor, remaining.get()),
    };
    assert_eq!(
        phrase,
        usize::try_from(integer(row, "after_phrase")).expect("phrase")
    );
    assert_eq!(
        cursor,
        usize::try_from(integer(row, "after_cursor")).expect("cursor")
    );
    assert_eq!(remaining, byte(row, "after_remaining"));
    assert_eq!(
        text(row, "after_position"),
        if remaining == 0 {
            "root-return"
        } else {
            "repeat-start"
        }
    );
    assert_eq!(track.note.0.get(), byte(row, "after_note"));
    assert_eq!(track.release, byte(row, "after_release"));
    assert_eq!(track.pitch.get(), byte(row, "after_pitch"));
}
fn check_loudness_ramp(row: &Table, before: &TrackState, track: &TrackState) {
    let mut start = before.clone();
    music::start_ramp(&mut start, nz(byte(row, "length")), byte(row, "target"));
    assert_eq!(
        i64::from(start.ramp.expect("active ramp").increment),
        integer(row, "delta")
    );
    assert_eq!(u64::from(track.loudness.0), unsigned(row, "after_volume"));
    assert_eq!(
        track.ramp.as_ref().map_or(0, |r| r.remaining.get()),
        byte(row, "after_remaining")
    );
}
fn check_song_input_fields(row: &Table, family: &str, song: &SongState, graphs: &[TrackContent]) {
    let track = &song.tracks[0];
    let repeat = |track: &TrackState| match track.position {
        TrackPosition::Root { .. } => 0,
        TrackPosition::Repeated { remaining, .. } => remaining.get(),
    };
    let ramp = |track: &TrackState| track.ramp.as_ref().map_or(0, |r| r.remaining.get());
    let fields = match family {
        "progress" => vec![
            ("note", i64::from(track.note.0.get())),
            ("gate", i64::from(track.release)),
            ("duration", i64::from(track.duration.0.get())),
            ("ratio", i64::from(track.articulation)),
        ],
        "phrase" => vec![("remaining", i64::from(repeat(track)))],
        "ramp" => vec![("volume", i64::from(track.loudness.0))],
        "restart" => vec![
            ("root_completed", 0),
            (
                "other_note_remaining",
                i64::from(song.tracks[1].note.0.get()),
            ),
            ("other_repeat_remaining", i64::from(repeat(&song.tracks[1]))),
            ("other_ramp_remaining", i64::from(ramp(&song.tracks[1]))),
            ("master_loudness", i64::from(song.mix.master.0)),
            ("first_volume", i64::from(track.loudness.0)),
            ("second_volume", i64::from(song.tracks[1].loudness.0)),
        ],
        _ => Vec::new(),
    };
    for (key, actual) in fields {
        assert_eq!(integer(row, key), actual, "{} {key}", text(row, "id"));
    }
    for event in graphs[0].phrases[0].iter().take_while(|e| {
        !matches!(
            e,
            MusicEvent::Tone(_)
                | MusicEvent::Sustain
                | MusicEvent::Repeat { .. }
                | MusicEvent::Complete
        )
    }) {
        check_choice_fields(row, event);
    }
    if family == "restart" {
        let (duration, ramp_turns, ramp_target) = intro_choices(&graphs[0]);
        assert_eq!(duration, byte(row, "intro_first_duration"));
        assert_eq!(ramp_turns, byte(row, "intro_first_ramp_length"));
        assert_eq!(ramp_target, byte(row, "intro_first_ramp_target"));
        assert_eq!(
            intro_choices(&graphs[1]).0,
            byte(row, "intro_second_duration")
        );
    }
}
fn intro_choices(graph: &TrackContent) -> (u8, u8, u8) {
    let mut result = (0, 0, 0);
    for event in &graph.phrases[0] {
        match event {
            MusicEvent::Timing { duration, .. } => result.0 = duration.0.get(),
            MusicEvent::Ramp { turns, target } => {
                result.1 = turns.get();
                result.2 = *target;
            }
            MusicEvent::Tone(_) | MusicEvent::Sustain => break,
            MusicEvent::Loudness(_)
            | MusicEvent::MasterLoudness(_)
            | MusicEvent::Timbre(_)
            | MusicEvent::Offset(_)
            | MusicEvent::Percussion(_)
            | MusicEvent::EchoMix { .. }
            | MusicEvent::EchoSpace { .. }
            | MusicEvent::Repeat { .. }
            | MusicEvent::Complete => {}
        }
    }
    result
}
fn check_choice_fields(row: &Table, event: &MusicEvent) {
    let fields = match event {
        MusicEvent::Timing {
            duration,
            articulation,
            extra,
        } => {
            let mut values = vec![("duration", i64::from(duration.0.get()))];
            if let Some(choice) = articulation {
                values.extend([
                    ("ratio", i64::from(choice.ratio)),
                    ("strength", i64::from(choice.strength)),
                ]);
            }
            if row.contains_key("extra") {
                assert_eq!(*extra, ExtraHandlingWork::new(unsigned(row, "extra")));
            }
            values
        }
        MusicEvent::Ramp { turns, target } => vec![
            ("target", i64::from(*target)),
            ("length", i64::from(turns.get())),
        ],
        MusicEvent::Loudness(v) | MusicEvent::MasterLoudness(v) => vec![("value", i64::from(*v))],
        MusicEvent::Timbre(v) => vec![("value", i64::from(v.0))],
        MusicEvent::Offset(_) => vec![("value", 0)],
        MusicEvent::Percussion(v) => vec![("value", i64::from(*v))],
        MusicEvent::EchoMix {
            voices,
            left,
            right,
        } => {
            if row.contains_key("voices") {
                assert_eq!(
                    *voices,
                    ints(row, "voices").into_iter().map(voice).collect()
                );
            }
            vec![("left", i64::from(*left)), ("right", i64::from(*right))]
        }
        MusicEvent::EchoSpace {
            delay,
            feedback,
            filter,
        } => vec![
            ("delay", i64::from(*delay)),
            ("feedback", i64::from(*feedback)),
            ("filter", i64::from(*filter)),
        ],
        MusicEvent::Tone(_)
        | MusicEvent::Sustain
        | MusicEvent::Repeat { .. }
        | MusicEvent::Complete => Vec::new(),
    };
    for (key, actual) in fields {
        if row.contains_key(key) {
            assert_eq!(integer(row, key), actual, "{} {key}", text(row, "id"));
        }
    }
}
fn completion_music_phase(row: &Table) -> u8 {
    if matches!(text(row, "profile"), "both" | "music") {
        224
    } else {
        0
    }
}
fn completion_vector(row: &Table, mix: &Value) {
    keys(
        row,
        "id profile time_numerator time_denominator publications note staged_action pending_work next_observe_numerator tracks mix",
    );
    let (song, tracks) = song(row, mix);
    let effect = matches!(text(row, "profile"), "both" | "effects");
    let mut state = state_fixture(
        song,
        if effect { 7 } else { 0 },
        completion_music_phase(row),
    );
    transport::initialize(&mut state);
    let content = Content {
        effects: BTreeMap::from([(
            CueId::Spin,
            EffectClip {
                parts: vec![EffectPartContent {
                    spans: vec![EffectSpan {
                        duration: EffectTurns(nz(20)),
                        pitch: ep(24),
                        timbre: TimbreId(0),
                        motion: None,
                    }],
                    loop_start: None,
                }],
            },
        )]),
        tracks,
    };
    let stop = time::instant(
        unsigned(row, "time_numerator"),
        NonZeroU64::new(unsigned(row, "time_denominator")).expect("denominator"),
    );
    let inputs = if effect && time::compare(&stop, time::halves(3217)).is_ge() {
        vec![transport::Input::cue(time::halves(3217), CueId::Spin)]
    } else {
        Vec::new()
    };
    let initial = state.clone();
    let output = transport::advance(&mut state, &content, stop.clone(), &inputs);
    let pubs: Vec<_> = output
        .iter()
        .filter(|o| matches!(o, transport::Output::Publication { .. }))
        .collect();
    assert_eq!(
        u64::try_from(pubs.len()).expect("publication count"),
        unsigned(row, "publications"),
        "{}",
        text(row, "id")
    );
    assert_eq!(state.song.tracks[0].note.0.get(), byte(row, "note"));
    assert_eq!(
        state.groups[SoundGroup::Action].staged,
        request(integer(row, "staged_action"))
    );
    let pending = match state.phase {
        AudioPhase::Work {
            result: CompletionResult::Effect { .. },
            ..
        } => "Effect",
        AudioPhase::Work {
            result: CompletionResult::Music(_),
            ..
        } => "Music",
        AudioPhase::ObserveClock { .. }
        | AudioPhase::Distribute { .. }
        | AudioPhase::Sample { .. }
        | AudioPhase::MusicStart { .. }
        | AudioPhase::Idle { .. } => "None",
    };
    assert_eq!(pending, text(row, "pending_work"));
    assert_eq!(state.now, stop);
    let mut split = initial;
    let cut = time::subtract(&stop, time::halves(1));
    let prior_inputs: Vec<_> = inputs
        .iter()
        .filter(|i| time::compare(&i.at, &cut).is_le())
        .cloned()
        .collect();
    let later_inputs: Vec<_> = inputs
        .iter()
        .filter(|i| time::compare(&i.at, &cut).is_gt())
        .cloned()
        .collect();
    let mut split_outputs = transport::advance(&mut split, &content, cut, &prior_inputs);
    split = roundtrip(&split);
    assert_eq!(
        transport::advance(&mut split, &content, time::cycles(0), &[]),
        Vec::new()
    );
    split_outputs.extend(transport::advance(
        &mut split,
        &content,
        time::halves(1),
        &later_inputs,
    ));
    assert_eq!(split_outputs, output);
    assert_eq!(split, state);
    check_completion_continuation(row, &state, effect, &pubs);
}
fn check_completion_continuation(
    row: &Table,
    state: &State,
    effect: bool,
    pubs: &[&transport::Output],
) {
    // Exact ordered publications are computed from the named work/anchor entries.
    let expected_times: Vec<_> = if effect {
        [1604, 1690, 1778, 2804]
            .into_iter()
            .take(pubs.len())
            .map(time::cycles)
            .collect()
    } else {
        (0..pubs.len()).map(|_| time::cycles(2542)).collect()
    };
    let actual_times: Vec<_> = pubs
        .iter()
        .filter_map(|o| match o {
            transport::Output::Publication { at, .. } => Some(at.clone()),
            transport::Output::Clock { .. }
            | transport::Output::Observation { .. }
            | transport::Output::Staged { .. } => None,
        })
        .collect();
    assert_eq!(actual_times, expected_times);
    if unsigned_optional(row, "next_observe_numerator") > 0 {
        let expected = time::halves(unsigned(row, "next_observe_numerator"));
        let mut resumed = roundtrip(state);
        let content = Content {
            effects: BTreeMap::new(),
            tracks: Vec::new(),
        };
        let output = transport::advance(
            &mut resumed,
            &content,
            time::subtract(&expected, &state.now),
            &[],
        );
        let immediate_empty = time::halves(if effect { 5433 } else { 4909 });
        assert_eq!(
            output.first(),
            Some(&transport::Output::Clock {
                at: immediate_empty,
                batch: 0
            })
        );
        let nonempty: Vec<_> = output
            .iter()
            .filter_map(|out| match out {
                transport::Output::Clock { at, batch } => {
                    if *batch > 0 {
                        Some(at.clone())
                    } else {
                        None
                    }
                }
                transport::Output::Publication { .. }
                | transport::Output::Observation { .. }
                | transport::Output::Staged { .. } => None,
            })
            .collect();
        assert_eq!(
            nonempty,
            vec![expected],
            "{} next_observe_numerator",
            text(row, "id")
        );
    }
}
fn check_effect_pending(
    group: SoundGroup,
    before: &GroupState,
    result: &effects::EffectResult,
    content: &Content,
    occupied: &BTreeSet<EffectVoiceId>,
) {
    let root: Table = toml::from_str(SOURCE).expect("fixture vectors");
    let (song, _) = song(
        array(&root, "progress")[0]
            .as_table()
            .expect("song fixture"),
        &root["fixture_mix"],
    );
    let mut state = state_fixture(song, 0, 0);
    state.groups[group] = before.clone();
    state.reservations.clone_from(occupied);
    state.phase = AudioPhase::Work {
        at: time::whole(result.work.0.clone()),
        result: CompletionResult::Effect {
            group,
            result: result.group.clone(),
            echo_voices: result.echo_voices.clone(),
        },
    };
    assert_eq!(
        transport::advance(&mut state, content, time::whole(&result.work.0 - 1u64), &[]),
        Vec::new()
    );
    assert_eq!(&state.groups[group], before);
    let mut restored = roundtrip(&state);
    assert_eq!(
        transport::advance(&mut restored, content, time::cycles(0), &[]),
        Vec::new()
    );
    let actual = transport::advance(&mut restored, content, time::cycles(1), &[]);
    assert_eq!(
        actual,
        vec![transport::Output::Publication {
            at: time::whole(result.work.0.clone()),
            line: ReplyLine::Group(group),
            reply: Reply::Effect(result.group.reply),
            result: CompletionResult::Effect {
                group,
                result: result.group.clone(),
                echo_voices: result.echo_voices.clone(),
            }
        }]
    );
    assert_eq!(restored.groups[group], result.group);
}
fn unsigned_optional(row: &Table, key: &str) -> u64 {
    u64::try_from(optional(row, key, 0)).expect("optional unsigned")
}
fn sample_vector(row: &Table, mix: &Value) {
    keys(row, "id first second accept retry_shift");
    let root: Table = toml::from_str(SOURCE).expect("vectors");
    let fixture = array(&root, "progress")[0]
        .as_table()
        .expect("song fixture");
    let (song, tracks) = song(fixture, mix);
    let mut state = state_fixture(song, 0, 0);
    let first = if integer(row, "first") == 7 {
        CueId::Footstep
    } else {
        CueId::Land
    };
    let second = if integer(row, "second") == 7 {
        CueId::Footstep
    } else {
        CueId::Land
    };
    state.requests[SoundGroup::Contact] = Request::Cue(first);
    state.phase = AudioPhase::Sample {
        sample: StableSample {
            line: ReplyLine::Group(SoundGroup::Contact),
            anchor: time::cycles(0),
            first: None,
        },
        next: Continuation::AfterEffects,
    };
    let content = Content {
        effects: BTreeMap::new(),
        tracks,
    };
    let first_output = transport::advance(
        &mut state,
        &content,
        time::cycles(5),
        &[transport::Input::cue(time::cycles(5), second)],
    );
    assert_eq!(
        first_output,
        vec![transport::Output::Observation {
            at: time::halves(9),
            line: ReplyLine::Group(SoundGroup::Contact),
            request: Request::Cue(first)
        }]
    );
    let mut restored = roundtrip(&state);
    let second_output = transport::advance(&mut restored, &content, time::halves(9), &[]);
    let accepted = second_output
        .iter()
        .any(|o| matches!(o, transport::Output::Staged { .. }));
    assert_eq!(accepted, flag(row, "accept"));
    if accepted {
        assert_eq!(
            restored.groups[SoundGroup::Contact].staged,
            Request::Cue(second)
        );
    } else {
        let AudioPhase::Sample { sample, .. } = restored.phase else {
            assert!(false);
            return;
        };
        assert_eq!(sample.anchor, time::cycles(unsigned(row, "retry_shift")));
        assert!(sample.first.is_none());
    }
}
fn clock_vector(row: &Table, mix: &Value) {
    keys(row, "id phase overflows observed after_observed");
    let root: Table = toml::from_str(SOURCE).expect("vectors");
    let (song, tracks) = song(
        array(&root, "progress")[0].as_table().expect("fixture"),
        mix,
    );
    let mut state = state_fixture(song, 0, 0);
    let content = Content {
        effects: BTreeMap::new(),
        tracks,
    };
    state.now = time::cycles(0);
    state.clock.phase =
        AudioCyclePhase::try_from(time::cycles(unsigned(row, "phase"))).expect("timer phase");
    let stop = time::cycles(unsigned(row, "overflows").saturating_mul(2048));
    state.phase = AudioPhase::ObserveClock { at: stop.clone() };
    let output = transport::advance(&mut state, &content, time::add(&stop, time::halves(1)), &[]);
    assert_eq!(
        output,
        vec![transport::Output::Clock {
            at: stop,
            batch: byte(row, "observed")
        }]
    );
    assert_eq!(state.clock.unread, byte(row, "after_observed"));
}
fn admission_vector(row: &Table) {
    keys(row, "id root repeated admitted");
    // Importer-side assertion only: the model does not validate supplied content.
    let root: Vec<_> = array(row, "root").iter().map(decode_event).collect();
    let admitted = root
        .iter()
        .find(|e| {
            matches!(
                e,
                MusicEvent::Tone(_)
                    | MusicEvent::Sustain
                    | MusicEvent::Repeat { .. }
                    | MusicEvent::Complete
            )
        })
        .is_some_and(|e| matches!(e, MusicEvent::Tone(_) | MusicEvent::Sustain));
    assert_eq!(admitted, flag(row, "admitted"));
}
fn inactive_vector(row: &Table, mix: &Value) {
    keys(
        row,
        "id first_target first_delta second_target second_delta elapsed check_interval tracks mix effect_phase music_phase",
    );
    let (song, tracks) = song(row, mix);
    assert!(song.tracks.iter().all(|t| t.ramp.is_none()));
    let content = Content {
        effects: BTreeMap::new(),
        tracks,
    };
    let initial = state_fixture(song, byte(row, "effect_phase"), byte(row, "music_phase"));
    let mut direct = initial.clone();
    let all = transport::advance(
        &mut direct,
        &content,
        time::cycles(unsigned(row, "elapsed")),
        &[],
    );
    let mut split = initial;
    let mut split_output = Vec::new();
    let mut remaining = unsigned(row, "elapsed");
    while remaining > 0 {
        let interval = remaining.min(unsigned(row, "check_interval"));
        split_output.extend(transport::advance(
            &mut split,
            &content,
            time::cycles(interval),
            &[],
        ));
        split = roundtrip(&split);
        remaining = remaining.saturating_sub(interval);
    }
    assert_eq!(split, direct);
    assert_eq!(split_output, all);
    assert_ne!(
        unsigned(row, "first_target"),
        unsigned(row, "second_target")
    );
    assert_ne!(integer(row, "first_delta"), integer(row, "second_delta"));
}
#[test]
fn all_invented_vectors() {
    let root: Table = toml::from_str(SOURCE).expect("decode all supplied vectors");
    keys(
        &root,
        "schema completion_clock fixture_mix sender phrase ramp restart offer cadence sample progress staged control effect music_work completion inactive_ramp clock content_admission resident_reply effect_history effect_echo",
    );
    assert_eq!(text(&root, "schema"), "audio-transport-draft-v1");
    let clock = root["completion_clock"].as_table().expect("clock fixture");
    keys(
        clock,
        "now timer_phase unread_overflows polling_origin_numerator polling_origin_denominator",
    );
    assert_eq!(integer(clock, "now"), 0);
    assert_eq!(integer(clock, "timer_phase"), 531);
    assert_eq!(integer(clock, "unread_overflows"), 0);
    assert_eq!(integer(clock, "polling_origin_numerator"), 811);
    assert_eq!(integer(clock, "polling_origin_denominator"), 2);
    let mix = &root["fixture_mix"];
    let mut seen = BTreeSet::new();
    for family in [
        "sender",
        "offer",
        "cadence",
        "sample",
        "phrase",
        "ramp",
        "restart",
        "progress",
        "staged",
        "control",
        "effect",
        "music_work",
        "completion",
        "inactive_ramp",
        "clock",
        "content_admission",
        "resident_reply",
        "effect_history",
        "effect_echo",
    ] {
        for value in array(&root, family) {
            let row = value.as_table().expect("vector record");
            assert!(
                seen.insert(text(row, "id").to_owned()),
                "duplicate vector id"
            );
            match family {
                "sender" => sender_vector(row),
                "offer" => offer_vector(row),
                "cadence" => cadence_vector(row),
                "sample" => sample_vector(row, mix),
                "staged" => staged_vector(row),
                "effect" => effect_vector(row),
                "completion" => completion_vector(row, mix),
                "resident_reply" => support::resident_vector(row, mix),
                "effect_history" => support::history_vector(row, &root),
                "effect_echo" => support::echo_vector(row, &root),
                "clock" => clock_vector(row, mix),
                "content_admission" => admission_vector(row),
                "inactive_ramp" => inactive_vector(row, mix),
                _ => musical_vector(row, family, mix),
            }
        }
    }
    assert_eq!(seen.len(), 112);
    let catalogue = audio_transport::catalogue::catalogue();
    assert_eq!(catalogue.len(), 19);
    for (_, behaviour) in &catalogue {
        for family in behaviour.checked_by {
            assert!(
                !array(&root, family).is_empty(),
                "catalogue entry lacks actual supplied checks"
            );
        }
    }
}
fn check_restart(row: &Table, song: &SongState) {
    let first = &song.tracks[0];
    let second = &song.tracks[1];
    assert_eq!(first.note.0.get(), byte(row, "after_first_note"));
    assert_eq!(second.note.0.get(), byte(row, "after_second_note"));
    assert_eq!(
        u64::from(first.loudness.0),
        unsigned(row, "after_first_volume")
    );
    assert_eq!(
        u64::from(second.loudness.0),
        unsigned(row, "after_second_volume")
    );
    assert_eq!(
        first.ramp.as_ref().map_or(0, |r| r.remaining.get()),
        byte(row, "after_first_ramp_remaining")
    );
    assert!(second.ramp.is_none());
    assert_eq!(
        second.ramp.as_ref().map_or(0, |r| r.remaining.get()),
        byte(row, "after_other_ramp_remaining")
    );
    let second_repeat = match second.position {
        TrackPosition::Root { .. } => 0,
        TrackPosition::Repeated { remaining, .. } => remaining.get(),
    };
    assert_eq!(second_repeat, byte(row, "after_other_repeat_remaining"));
    assert!(matches!(second.position, TrackPosition::Root { .. }));
    assert_eq!(
        u64::from(song.mix.master.0),
        unsigned(row, "after_master_loudness")
    );
}
#[test]
fn constructed_retry_wrap_and_large_interval() {
    let root: Table = toml::from_str(SOURCE).expect("vectors");
    let (song, tracks) = song(
        array(&root, "progress")[0]
            .as_table()
            .expect("song fixture"),
        &root["fixture_mix"],
    );
    let mut initial = state_fixture(song, 0, 0);
    initial.phase = AudioPhase::Sample {
        sample: StableSample {
            line: ReplyLine::Group(SoundGroup::Contact),
            anchor: time::cycles(0),
            first: None,
        },
        next: Continuation::AfterEffects,
    };
    let content = Content {
        effects: BTreeMap::new(),
        tracks,
    };
    let inputs: Vec<_> = (0_u64..2393)
        .map(|index| {
            let at = time::halves(19_u64.saturating_add(index.saturating_mul(28)));
            if index.checked_rem(2) == Some(0) {
                transport::Input::cue(at, CueId::Footstep)
            } else {
                transport::Input::absence(at, SoundGroup::Contact)
            }
        })
        .collect();
    let mut direct = initial.clone();
    let all = transport::advance(&mut direct, &content, time::cycles(33500), &inputs);
    assert!(
        all.iter()
            .all(|out| matches!(out, transport::Output::Observation { .. }))
    );
    assert_eq!(direct.clock.unread, 0);
    assert_eq!(direct.clock.phase.instant(), &time::cycles(1263));
    assert_eq!(direct.cadence, initial.cadence);
    let mut split = initial;
    let early: Vec<_> = inputs
        .iter()
        .filter(|input| time::compare(&input.at, time::cycles(2048)).is_le())
        .cloned()
        .collect();
    let later: Vec<_> = inputs
        .iter()
        .filter(|input| time::compare(&input.at, time::cycles(2048)).is_gt())
        .cloned()
        .collect();
    let mut resumed = transport::advance(&mut split, &content, time::cycles(2048), &early);
    split = roundtrip(&split);
    assert_eq!(
        transport::advance(&mut split, &content, time::cycles(0), &[]),
        Vec::new()
    );
    resumed.extend(transport::advance(
        &mut split,
        &content,
        time::cycles(31452),
        &later,
    ));
    assert_eq!(resumed, all);
    assert_eq!(split, direct);
}
#[test]
fn constructed_sender_observes_latest_at_read() {
    let mut state = SenderState {
        queue: VecDeque::from([CueId::EndSpin]),
        progress: SenderProgress::AwaitCue(CueId::Spin),
        reply: Request::Absent,
        now: CpuMasterTicks(0),
        pending: None,
    };
    sender::begin(&mut state, sender::SenderProfile::FastBus);
    let initial = state.clone();
    assert_eq!(sender::advance(&mut state, CpuMasterTicks(57)), Vec::new());
    let serialized = Value::try_from(&state).expect("save pending observation");
    let mut restored: SenderState = serialized.try_into().expect("restore pending observation");
    let updates = [
        sender::ReplyInput {
            at: CpuMasterTicks(58),
            reply: Request::Cue(CueId::EndSpin),
        },
        sender::ReplyInput {
            at: CpuMasterTicks(58),
            reply: Request::Cue(CueId::Spin),
        },
        sender::ReplyInput {
            at: CpuMasterTicks(100),
            reply: Request::Absent,
        },
    ];
    let outputs = sender::advance_with_inputs(&mut restored, CpuMasterTicks(153), &updates);
    assert_eq!(
        outputs,
        vec![
            sender::SenderOutput::Observation(CpuMasterTicks(58), Request::Cue(CueId::Spin)),
            sender::SenderOutput::Completed(CpuMasterTicks(210))
        ]
    );
    assert_eq!(restored.progress, SenderProgress::ClearDelay(nz(2)));
    let mut direct = initial;
    assert_eq!(
        sender::advance_with_inputs(&mut direct, CpuMasterTicks(210), &updates),
        outputs
    );
    assert_eq!(direct, restored);
}
#[test]
fn constructed_lost_root_continuation_is_detected() {
    let root: Table = toml::from_str(SOURCE).expect("vectors");
    let row = array(&root, "phrase")[1]
        .as_table()
        .expect("return fixture");
    let (song, tracks) = song(row, &root["fixture_mix"]);
    let expected = music::turn(&song, &tracks);
    let mut lost = song;
    let TrackPosition::Repeated { continuation, .. } = &mut lost.tracks[0].position else {
        assert!(false);
        return;
    };
    *continuation = 0;
    let actual = music::turn(&lost, &tracks);
    assert_eq!(
        expected.song.tracks[0].position,
        TrackPosition::Root { cursor: 3 }
    );
    assert_eq!(expected.song.tracks[0].pitch, mp(25));
    assert_eq!(
        actual.song.tracks[0].position,
        TrackPosition::Root { cursor: 1 }
    );
    assert_eq!(actual.song.tracks[0].pitch, mp(24));
    assert_ne!(actual, expected);
}
#[test]
fn constructed_wrong_timing_is_detected() {
    let root: Table = toml::from_str(SOURCE).expect("vectors");
    let row = array(&root, "music_work")
        .iter()
        .map(|v| v.as_table().expect("work vector"))
        .find(|r| text(r, "id") == "W-extra-three")
        .expect("extra work case");
    let (song, mut tracks) = song(row, &root["fixture_mix"]);
    let expected = music::turn(&song, &tracks);
    assert_eq!(expected.work, AudioCycles(unsigned(row, "work")));
    let MusicEvent::Timing { extra, .. } = &mut tracks[0].phrases[0][0] else {
        assert!(false);
        return;
    };
    extra.0 += 1u64;
    let actual = music::turn(&song, &tracks);
    assert_eq!(actual.song, expected.song);
    assert_eq!(actual.work.0, &expected.work.0 + 1u64);
    assert_ne!(actual.work, expected.work);
}
#[test]
fn constructed_missing_publication_is_detected() {
    let root: Table = toml::from_str(SOURCE).expect("vectors");
    let (song, tracks) = song(
        array(&root, "progress")[0]
            .as_table()
            .expect("track fixture"),
        &root["fixture_mix"],
    );
    let content = Content {
        effects: BTreeMap::new(),
        tracks,
    };
    let result = music::turn(&song, &content.tracks);
    let mut state = state_fixture(song, 0, 0);
    state.phase = AudioPhase::Work {
        at: time::whole(result.work.0.clone()),
        result: CompletionResult::Music(result.song.clone()),
    };
    let mut actual = transport::advance(
        &mut state,
        &content,
        time::whole(result.work.0.clone()),
        &[],
    );
    let complete = actual.clone();
    actual.retain(|output| !matches!(output, transport::Output::Publication { .. }));
    let expected = vec![transport::Output::Publication {
        at: time::whole(result.work.0.clone()),
        line: ReplyLine::Music,
        reply: Reply::Music(state.resident_music_reply.clone()),
        result: CompletionResult::Music(result.song),
    }];
    assert_eq!(complete, expected);
    assert_eq!(actual, Vec::new());
    assert_eq!(expected.len(), 1);
    assert_ne!(actual, expected);
}
#[test]
fn constructed_lost_repeat_is_detected() {
    let root: Table = toml::from_str(SOURCE).expect("vectors");
    let row = array(&root, "phrase")[0]
        .as_table()
        .expect("repeat fixture");
    let (song, tracks) = song(row, &root["fixture_mix"]);
    let expected = music::turn(&song, &tracks);
    let mut lost = song;
    let TrackPosition::Repeated { remaining, .. } = &mut lost.tracks[0].position else {
        assert!(false);
        return;
    };
    *remaining = nz(1);
    let actual = music::turn(&lost, &tracks);
    assert_eq!(
        expected.song.tracks[0].position,
        TrackPosition::Repeated {
            phrase: 1,
            cursor: 1,
            remaining: nz(2),
            continuation: 2,
        }
    );
    assert_eq!(
        actual.song.tracks[0].position,
        TrackPosition::Root { cursor: 3 }
    );
    assert_eq!(expected.song.tracks[0].pitch, mp(24));
    assert_eq!(actual.song.tracks[0].pitch, mp(25));
    assert_ne!(actual, expected);
}
fn check_control(row: &Table, song: &SongState) {
    let track = &song.tracks[0];
    match text(row, "kind") {
        "master-loudness" => assert_eq!(u64::from(song.mix.master.0), unsigned(row, "after_value")),
        "timbre" => assert_eq!(u64::from(track.timbre.0), unsigned(row, "after_value")),
        "pitch-offset" => {
            assert_eq!(track.offset, PitchOffset::Zero);
            assert_eq!(integer(row, "after_value"), 0);
        }
        "articulation" => {
            assert_eq!(track.strength, byte(row, "after_value"));
            assert_eq!(track.duration.0.get(), byte(row, "duration"));
            assert_eq!(track.articulation, byte(row, "ratio"));
        }
        "percussion-choice" => {
            assert_eq!(u64::from(song.mix.percussion), unsigned(row, "after_value"));
        }
        "echo-mix" => {
            assert_eq!(
                song.mix.echo_voices,
                ints(row, "voices").into_iter().map(voice).collect()
            );
            assert_eq!(u64::from(song.mix.echo_left), unsigned(row, "after_left"));
            assert_eq!(u64::from(song.mix.echo_right), unsigned(row, "after_right"));
            assert_eq!(song.mix.echo_enabled, flag(row, "after_enabled"));
        }
        "echo-space" => {
            assert_eq!(song.mix.echo_delay, byte(row, "after_delay"));
            assert_eq!(song.mix.feedback, byte(row, "after_feedback"));
            assert_eq!(u64::from(song.mix.filter), unsigned(row, "after_filter"));
        }
        other => {
            assert_eq!(other, "loudness");
            assert_eq!(u64::from(track.loudness.0), unsigned(row, "after_value"));
            assert_eq!(
                track.ramp.as_ref().map_or(0, |r| r.remaining.get()),
                byte(row, "after_ramp_remaining")
            );
        }
    }
}
