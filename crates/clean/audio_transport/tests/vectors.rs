//! Every audio-transport vector, run by sender operation or by frame, with carried state saved
//! and restored before services and frames; and the catalogue checked against the spec.
#![expect(
    clippy::expect_used,
    reason = "Malformed invented vectors must fail loudly"
)]
#![expect(
    clippy::indexing_slicing,
    reason = "Fixture bounds are asserted by vector decoding"
)]
#![expect(
    clippy::doc_markdown,
    reason = "Spec entry identifiers are plain catalogue labels"
)]
mod support;
use audio_transport::frame::{self, EffectCadence, FrameCadence, TurnCount};
use audio_transport::*;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::num::NonZeroU8;
use toml::{Table, Value};
const VECTORS: &str = include_str!("../../../../scenarios/vectors/audio-transport.toml");
const SPEC: &str = include_str!("../../../../docs/behaviours/audio-transport.md");
/// Every vector family the file holds, each run below.
const FAMILIES: [&str; 15] = [
    "sender",
    "offer",
    "phrase",
    "ramp",
    "restart",
    "progress",
    "staged",
    "control",
    "effect",
    "inactive_ramp",
    "content_admission",
    "resident_reply",
    "effect_history",
    "effect_echo",
    "frame",
];
/// A frame with one effect turn after its sender service: the most common cadence.
const ONE_EFFECT_TURN: FrameCadence = FrameCadence {
    effects: EffectCadence::AroundService {
        before: TurnCount::ZERO,
        after: TurnCount::ONE,
    },
    music: TurnCount::ZERO,
};
/// A frame that runs only its sender service.
const SERVICE_ONLY: FrameCadence = FrameCadence {
    effects: EffectCadence::AroundService {
        before: TurnCount::ZERO,
        after: TurnCount::ZERO,
    },
    music: TurnCount::ZERO,
};
/// A frame that runs out of time before its service and completes no audio turn.
const NO_TURN: FrameCadence = FrameCadence {
    effects: EffectCadence::WithoutService(TurnCount::ZERO),
    music: TurnCount::ZERO,
};
/// A frame that completes one music turn and nothing else.
const ONE_MUSIC_TURN: FrameCadence = FrameCadence {
    effects: EffectCadence::WithoutService(TurnCount::ZERO),
    music: TurnCount::ONE,
};
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
fn unsigned(row: &Table, key: &str) -> u64 {
    u64::try_from(integer(row, key)).expect(key)
}
fn byte(row: &Table, key: &str) -> u8 {
    u8::try_from(integer(row, key)).expect(key)
}
fn index(row: &Table, key: &str) -> usize {
    usize::try_from(integer(row, key)).expect(key)
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
fn table(value: &Value) -> &Table {
    value.as_table().expect("vector record")
}
fn ints(row: &Table, key: &str) -> Vec<i64> {
    array(row, key)
        .iter()
        .map(|v| v.as_integer().expect(key))
        .collect()
}
/// Refuse any key the test does not know.
fn keys(row: &Table, admitted: &str) {
    for key in row.keys() {
        assert!(
            admitted.split_whitespace().any(|known| known == key),
            "unknown vector key {key}"
        );
    }
}
/// A typed value from its name, such as a cue or a sound group.
fn named<T: serde::de::DeserializeOwned>(name: &str) -> T {
    Value::String(name.into()).try_into().expect(name)
}
fn request_named(name: &str) -> Request {
    if name == "Absent" {
        Request::Absent
    } else {
        Request::Cue(named(name))
    }
}
/// Sender and offer aliases carry cue identity only.
fn alias_cue(value: i64) -> CueId {
    match value {
        7 => CueId::Spin,
        8 => CueId::ShaftExplosion,
        other => {
            assert_eq!(other, 9, "cue alias");
            CueId::EndSpin
        }
    }
}
/// Alias 0 is absence.
fn alias_request(value: i64) -> Request {
    if value == 0 {
        Request::Absent
    } else {
        Request::Cue(alias_cue(value))
    }
}
fn queue_of(cues: impl IntoIterator<Item = CueId>) -> CueQueue {
    CueQueue::try_from(cues.into_iter().collect::<VecDeque<_>>()).expect("at most fifteen cues")
}
fn effect_voice(value: i64) -> EffectVoiceId {
    match value {
        7 => EffectVoiceId::V7,
        6 => EffectVoiceId::V6,
        5 => EffectVoiceId::V5,
        4 => EffectVoiceId::V4,
        3 => EffectVoiceId::V3,
        _ => {
            assert_eq!(value, 2, "effect voice alias");
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
fn voices(row: &Table, key: &str) -> BTreeSet<VoiceId> {
    ints(row, key).into_iter().map(voice).collect()
}
fn effect_voices(row: &Table, key: &str) -> BTreeSet<EffectVoiceId> {
    ints(row, key).into_iter().map(effect_voice).collect()
}
fn decode_event(v: &Value) -> MusicEvent {
    let row = table(v);
    keys(
        row,
        "kind duration articulation pitch value turns target identity fraction voices left right delay feedback filter phrase count",
    );
    match text(row, "kind") {
        "Timing" => MusicEvent::Timing {
            duration: MusicTurns(nz(byte(row, "duration"))),
            articulation: row.get("articulation").map(|value| {
                let r = table(value);
                keys(r, "ratio strength");
                Articulation {
                    ratio: byte(r, "ratio"),
                    strength: byte(r, "strength"),
                }
            }),
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
            voices: voices(row, "voices"),
            left: byte(row, "left"),
            right: byte(row, "right"),
        },
        "EchoSpace" => MusicEvent::EchoSpace {
            delay: byte(row, "delay"),
            feedback: byte(row, "feedback"),
            filter: u16::try_from(integer(row, "filter")).expect("filter"),
        },
        "RepeatPhrase" => MusicEvent::Repeat {
            phrase: index(row, "phrase"),
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
fn mix_fixture(row: &Table, fixture_mix: &Value) -> MixFixture {
    row.get("mix")
        .unwrap_or(fixture_mix)
        .clone()
        .try_into()
        .expect("complete fixture mix")
}
fn song_mix(mix: &MixFixture) -> SongMix {
    SongMix {
        master: Loudness(mix.master_loudness),
        percussion: mix.percussion_set,
        echo_voices: mix
            .echo_voices
            .iter()
            .map(|v| voice(i64::from(*v)))
            .collect(),
        echo_left: mix.echo_left,
        echo_right: mix.echo_right,
        echo_delay: mix.echo_delay,
        feedback: mix.feedback,
        filter: mix.filter_set,
        echo_enabled: mix.echo_writes_enabled,
    }
}
fn track(f: &TrackFixture, mix: &MixFixture, index: usize) -> (TrackState, TrackContent) {
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
    let state = TrackState {
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
    };
    let graph = TrackContent {
        phrases: f
            .phrases
            .iter()
            .map(|p| p.iter().map(decode_event).collect())
            .collect(),
    };
    (state, graph)
}
/// The vector's song and its tracks' musical content.
fn song(row: &Table, fixture_mix: &Value) -> (SongState, Vec<TrackContent>) {
    let mix = mix_fixture(row, fixture_mix);
    let (tracks, graphs) = array(row, "tracks")
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let fixture = value.clone().try_into().expect("complete track fixture");
            track(&fixture, &mix, index)
        })
        .unzip();
    (
        SongState {
            tracks,
            mix: song_mix(&mix),
        },
        graphs,
    )
}
/// A song with no tracks, for vectors that run no music turn.
fn silent_song(fixture_mix: &Value) -> SongState {
    SongState {
        tracks: Vec::new(),
        mix: song_mix(&fixture_mix.clone().try_into().expect("fixture mix")),
    }
}
fn music_reply(identity: &str, original_byte: u8) -> ResidentMusicReply {
    ResidentMusicReply {
        identity: identity.into(),
        original_byte,
    }
}
fn idle_group(parts: usize) -> GroupState {
    GroupState {
        history: vec![ReleaseMemory::Inactive; parts],
        handled: Request::Absent,
        staged: Request::Absent,
        reply: Request::Absent,
        playback: Playback::Empty,
    }
}
fn ready() -> SenderState {
    SenderState {
        queue: CueQueue::default(),
        progress: SenderProgress::Ready,
    }
}
/// Every group ready with an empty queue, absent lines, free voices and complete inactive
/// histories; invented music replies of alias zero.
fn fresh_state(song: SongState) -> State {
    State {
        senders: enum_map::enum_map! {
            SoundGroup::Action => ready(),
            SoundGroup::Ambient => ready(),
            SoundGroup::Contact => ready(),
        },
        request_lines: enum_map::enum_map! {
            SoundGroup::Action => Request::Absent,
            SoundGroup::Ambient => Request::Absent,
            SoundGroup::Contact => Request::Absent,
        },
        groups: enum_map::enum_map! {
            SoundGroup::Action => idle_group(1),
            SoundGroup::Ambient => idle_group(1),
            SoundGroup::Contact => idle_group(2),
        },
        song,
        music_reply: music_reply("invented earlier music reply", 0),
        resident_music_reply: music_reply("invented resident arrangement", 0),
        reservations: BTreeSet::new(),
    }
}
const fn content(effects: BTreeMap<CueId, EffectClip>, tracks: Vec<TrackContent>) -> Content {
    Content { effects, tracks }
}
fn roundtrip<T: serde::Serialize + serde::de::DeserializeOwned>(state: &T) -> T {
    let saved = Value::try_from(state).expect("State serialization");
    saved.try_into().expect("State restoration")
}
/// Play one frame from the State, and again from a saved and restored copy of it; the two must
/// end the frame alike.
fn play(state: &mut State, content: &Content, offers: &[Offer], cadence: FrameCadence) {
    let mut restored = roundtrip(state);
    assert_eq!(&restored, state, "restoring changes nothing");
    frame::play(state, content, offers, cadence);
    frame::play(&mut restored, content, offers, cadence);
    assert_eq!(&restored, state, "a restored State plays the same frame");
}
fn sender_progress(kind: &str, value: i64) -> SenderProgress {
    let turns = || nz(u8::try_from(value).expect("delay"));
    match kind {
        "Ready" => SenderProgress::Ready,
        "AwaitCue" => SenderProgress::AwaitCue(alias_cue(value)),
        "ClearDelay" => SenderProgress::ClearDelay(turns()),
        "AwaitClear" => SenderProgress::AwaitClear,
        other => {
            assert_eq!(other, "ResetDelay");
            SenderProgress::ResetDelay(turns())
        }
    }
}
fn expected_group(row: &Table) -> SoundGroup {
    row.get("group").map_or(SoundGroup::Action, |value| {
        named(value.as_str().expect("group"))
    })
}
/// One service of a carried sender and request line, with a caller-supplied reply.
fn sender_vector(row: &Table) {
    keys(
        row,
        "id group state value queue reply after_state after_value after_queue present",
    );
    let id = text(row, "id");
    let group = expected_group(row);
    let mut sender = SenderState {
        queue: queue_of(ints(row, "queue").into_iter().map(alias_cue)),
        progress: sender_progress(text(row, "state"), integer(row, "value")),
    };
    for cue in sender.queue.cues() {
        assert_eq!(cue.group(), group, "{id}: queued cue group");
    }
    let reply = alias_request(integer(row, "reply"));
    if let Request::Cue(cue) = reply {
        assert_eq!(cue.group(), group, "{id}: reply group");
    }
    // Seed the line with a value no vector presents, to detect writes on a no-presentation cut.
    let carried_line = Request::Cue(match group {
        SoundGroup::Action => CueId::SamusSilence,
        SoundGroup::Ambient => CueId::BackgroundSilence,
        SoundGroup::Contact => CueId::Footstep,
    });
    let mut line = carried_line;
    let (mut restored_sender, mut restored_line) = roundtrip(&(sender.clone(), line));
    assert_eq!((&restored_sender, restored_line), (&sender, line), "{id}");
    serve(&mut sender, &mut line, reply);
    serve(&mut restored_sender, &mut restored_line, reply);
    assert_eq!((&restored_sender, restored_line), (&sender, line), "{id}");
    assert_eq!(
        sender.progress,
        sender_progress(text(row, "after_state"), integer(row, "after_value")),
        "{id}"
    );
    assert_eq!(
        sender.queue,
        queue_of(ints(row, "after_queue").into_iter().map(alias_cue)),
        "{id}"
    );
    let presented = match integer(row, "present") {
        -1 => carried_line,
        value => alias_request(value),
    };
    assert_eq!(line, presented, "{id}");
}
/// Apply one offer to the carried queue, without running a sender service.
fn offer_vector(row: &Table) {
    keys(
        row,
        "id group queue cue allowance enabled demo exploding after_queue",
    );
    let id = text(row, "id");
    let cue = alias_cue(integer(row, "cue"));
    assert_eq!(cue.group(), expected_group(row), "{id}: offered cue group");
    let mut queue = queue_of(ints(row, "queue").into_iter().map(alias_cue));
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
    offer(
        &mut queue,
        Offer {
            cue,
            allowance,
            permission,
        },
    );
    assert_eq!(
        queue,
        queue_of(ints(row, "after_queue").into_iter().map(alias_cue)),
        "{id}"
    );
}
fn musical_keys(family: &str) -> &'static str {
    match family {
        "progress" => "id note gate duration ratio after_note after_gate tracks mix",
        "phrase" => {
            "id remaining after_remaining after_position after_phrase after_cursor after_note after_release after_pitch tracks mix"
        }
        "ramp" => "id volume target length delta after_volume after_remaining tracks mix",
        "restart" => {
            "id root_completed other_note_remaining other_repeat_remaining other_ramp_remaining master_loudness after_other_repeat_remaining after_other_ramp_remaining after_master_loudness intro_first_duration intro_second_duration intro_first_ramp_length intro_first_ramp_target first_volume second_volume after_first_note after_second_note after_first_volume after_second_volume after_first_ramp_remaining tracks mix"
        }
        _ => {
            assert_eq!(family, "control");
            "id kind value after_value duration ratio voices left right after_left after_right after_enabled delay feedback filter after_delay after_feedback after_filter ramp_remaining ramp_target ramp_delta after_ramp_remaining reserved_voices after_reserved_voices tracks mix"
        }
    }
}
/// One music turn of the vector's song.
fn musical_vector(row: &Table, family: &str, mix: &Value) {
    keys(row, musical_keys(family));
    let (before, graphs) = song(row, mix);
    check_song_input_fields(row, family, &before, &graphs);
    let mut state = fresh_state(before.clone());
    if row.contains_key("reserved_voices") {
        state.reservations = effect_voices(row, "reserved_voices");
    }
    play(
        &mut state,
        &content(BTreeMap::new(), graphs),
        &[],
        ONE_MUSIC_TURN,
    );
    let after = &state.song;
    match family {
        "progress" => check_music_progress(row, &after.tracks[0]),
        "phrase" => check_phrase_progress(row, &after.tracks[0]),
        "ramp" => check_loudness_ramp(row, &before.tracks[0], &after.tracks[0]),
        "restart" => check_restart(row, after),
        _ => check_control(row, after),
    }
    if row.contains_key("after_reserved_voices") {
        assert_eq!(
            state.reservations,
            effect_voices(row, "after_reserved_voices")
        );
    }
    assert_eq!(state.music_reply, state.resident_music_reply);
}
fn ramp_turns(track: &TrackState) -> u8 {
    track.ramp.as_ref().map_or(0, |r| r.remaining.get())
}
const fn repeats(track: &TrackState) -> u8 {
    match track.position {
        TrackPosition::Root { .. } => 0,
        TrackPosition::Repeated { remaining, .. } => remaining.get(),
    }
}
fn check_music_progress(row: &Table, track: &TrackState) {
    assert_eq!(track.note.0.get(), byte(row, "after_note"));
    assert_eq!(track.release, byte(row, "after_gate"));
}
fn check_phrase_progress(row: &Table, track: &TrackState) {
    let (phrase, cursor) = match track.position {
        TrackPosition::Root { cursor } => (0, cursor),
        TrackPosition::Repeated { phrase, cursor, .. } => (phrase, cursor),
    };
    assert_eq!(phrase, index(row, "after_phrase"));
    assert_eq!(cursor, index(row, "after_cursor"));
    assert_eq!(repeats(track), byte(row, "after_remaining"));
    let position = if repeats(track) == 0 {
        "root-return"
    } else {
        "repeat-start"
    };
    assert_eq!(text(row, "after_position"), position);
    assert_eq!(track.note.0.get(), byte(row, "after_note"));
    assert_eq!(track.release, byte(row, "after_release"));
    assert_eq!(track.pitch.get(), byte(row, "after_pitch"));
}
fn check_loudness_ramp(row: &Table, before: &TrackState, track: &TrackState) {
    let ramp = ramp_from(
        before.loudness,
        nz(byte(row, "length")),
        byte(row, "target"),
    );
    assert_eq!(i64::from(ramp.increment), integer(row, "delta"));
    assert_eq!(u64::from(track.loudness.0), unsigned(row, "after_volume"));
    assert_eq!(ramp_turns(track), byte(row, "after_remaining"));
    if let Some(active) = &track.ramp {
        assert_eq!(active.increment, ramp.increment);
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
    assert_eq!(ramp_turns(first), byte(row, "after_first_ramp_remaining"));
    assert_eq!(ramp_turns(second), byte(row, "after_other_ramp_remaining"));
    assert_eq!(repeats(second), byte(row, "after_other_repeat_remaining"));
    assert!(matches!(second.position, TrackPosition::Root { .. }));
    assert_eq!(
        u64::from(song.mix.master.0),
        unsigned(row, "after_master_loudness")
    );
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
            assert_eq!(song.mix.echo_voices, voices(row, "voices"));
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
            assert_eq!(ramp_turns(track), byte(row, "after_ramp_remaining"));
        }
    }
}
/// The vector's input fields must describe the song it supplies.
fn check_song_input_fields(row: &Table, family: &str, song: &SongState, graphs: &[TrackContent]) {
    let track = &song.tracks[0];
    let fields = match family {
        "progress" => vec![
            ("note", i64::from(track.note.0.get())),
            ("gate", i64::from(track.release)),
            ("duration", i64::from(track.duration.0.get())),
            ("ratio", i64::from(track.articulation)),
        ],
        "phrase" => vec![("remaining", i64::from(repeats(track)))],
        "ramp" => vec![("volume", i64::from(track.loudness.0))],
        "restart" => vec![
            ("root_completed", 0),
            (
                "other_note_remaining",
                i64::from(song.tracks[1].note.0.get()),
            ),
            (
                "other_repeat_remaining",
                i64::from(repeats(&song.tracks[1])),
            ),
            (
                "other_ramp_remaining",
                i64::from(ramp_turns(&song.tracks[1])),
            ),
            ("master_loudness", i64::from(song.mix.master.0)),
            ("first_volume", i64::from(track.loudness.0)),
            ("second_volume", i64::from(song.tracks[1].loudness.0)),
        ],
        _ => vec![("ramp_remaining", i64::from(ramp_turns(track)))],
    };
    for (key, actual) in fields {
        if row.contains_key(key) {
            assert_eq!(integer(row, key), actual, "{} {key}", text(row, "id"));
        }
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
        } => {
            let mut values = vec![("duration", i64::from(duration.0.get()))];
            if let Some(choice) = articulation {
                values.extend([
                    ("ratio", i64::from(choice.ratio)),
                    ("value", i64::from(choice.strength)),
                ]);
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
            voices: selected,
            left,
            right,
        } => {
            assert_eq!(*selected, voices(row, "voices"));
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
/// Two songs that differ only in a finished ramp's configuration: an inactive ramp keeps none, so
/// both start alike and stay alike through every music turn.
fn inactive_vector(row: &Table, mix: &Value) {
    keys(
        row,
        "id first_target first_delta second_target second_delta music_turns tracks mix",
    );
    assert_ne!(
        unsigned(row, "first_target"),
        unsigned(row, "second_target")
    );
    assert_ne!(integer(row, "first_delta"), integer(row, "second_delta"));
    let (song, graphs) = song(row, mix);
    assert!(song.tracks.iter().all(|t| t.ramp.is_none()));
    let content = content(BTreeMap::new(), graphs);
    let mut first = fresh_state(song.clone());
    let mut second = fresh_state(song);
    let mut ramps_seen = false;
    for _ in 0..unsigned(row, "music_turns") {
        play(&mut first, &content, &[], ONE_MUSIC_TURN);
        play(&mut second, &content, &[], ONE_MUSIC_TURN);
        assert_eq!(first.song, second.song);
        ramps_seen |= first.song.tracks.iter().any(|t| t.ramp.is_some());
    }
    assert!(ramps_seen, "the songs start and finish ramps along the way");
}
/// Importer-side check of a root's introduction; the model does not validate content.
fn admission_vector(row: &Table) {
    keys(row, "id root repeated admitted");
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
fn run(family: &str, row: &Table, root: &Table) {
    let mix = &root["fixture_mix"];
    match family {
        "sender" => sender_vector(row),
        "offer" => offer_vector(row),
        "staged" => support::staged_vector(row, mix),
        "effect" => support::effect_vector(row, mix),
        "inactive_ramp" => inactive_vector(row, mix),
        "content_admission" => admission_vector(row),
        "resident_reply" => support::resident_vector(row, mix),
        "effect_history" => support::history_vector(row, root),
        "effect_echo" => support::echo_vector(row, root),
        "frame" => support::frame_vector(row, mix),
        _ => musical_vector(row, family, mix),
    }
}
#[test]
fn every_vector_runs_through_the_model() {
    let root: Table = toml::from_str(VECTORS).expect("decode all supplied vectors");
    keys(&root, &format!("schema fixture_mix {}", FAMILIES.join(" ")));
    assert_eq!(text(&root, "schema"), "audio-transport-frames-v1");
    let mut seen = BTreeSet::new();
    for family in FAMILIES {
        for value in array(&root, family) {
            let row = table(value);
            assert!(
                seen.insert(text(row, "id").to_owned()),
                "duplicate vector id"
            );
            run(family, row, &root);
        }
    }
    assert_eq!(seen.len(), 82);
}
/// Each spec entry's id and kind, and the vectors its `checked:` line cites.
fn spec_entries() -> Vec<(String, String, Vec<String>)> {
    let mut entries = Vec::new();
    let mut lines = SPEC.lines();
    while let Some(line) = lines.next() {
        let Some(id) = line.strip_prefix("- id: `") else {
            continue;
        };
        let id = id.trim_end_matches("`.").to_owned();
        let kind = lines
            .next()
            .and_then(|l| l.strip_prefix("- kind: "))
            .expect("kind line");
        let checked = lines
            .find(|l| l.starts_with("- checked: "))
            .expect("checked line");
        let cited = checked
            .split(|c: char| !(c.is_ascii_alphanumeric() || c == '-'))
            .filter(|w| {
                w.len() > 2 && w.as_bytes()[1] == b'-' && w.as_bytes()[0].is_ascii_uppercase()
            })
            .map(str::to_owned)
            .collect();
        entries.push((id, kind.trim_end_matches('.').to_owned(), cited));
    }
    entries
}
#[test]
fn catalogue_matches_the_spec_one_for_one() {
    let root: Table = toml::from_str(VECTORS).expect("vectors");
    let family_of: BTreeMap<String, &str> = FAMILIES
        .iter()
        .flat_map(|family| {
            array(&root, family)
                .iter()
                .map(|v| (text(table(v), "id").to_owned(), *family))
        })
        .collect();
    let entries = spec_entries();
    let catalogue = audio_transport::catalogue::catalogue();
    assert_eq!(entries.len(), 13);
    assert_eq!(catalogue.len(), entries.len());
    for (id, behaviour) in &catalogue {
        let name = format!("{id:?}");
        let (_, kind, cited) = entries
            .iter()
            .find(|(spec_id, _, _)| *spec_id == name)
            .expect("every catalogue entry is a spec entry");
        assert_eq!(&format!("{:?}", behaviour.kind), kind, "{name} kind");
        assert_eq!(behaviour.switch, catalogue::Switch::Fixed, "{name} switch");
        for vector in cited {
            let family = family_of
                .get(vector)
                .expect("every cited vector is supplied");
            assert!(
                behaviour.checked_by.contains(family),
                "{name} is checked by {family}"
            );
        }
        for family in behaviour.checked_by {
            assert!(FAMILIES.contains(family), "{name} names a supplied family");
            assert_ne!(array(&root, family).len(), 0, "{name} family has vectors");
        }
    }
}
