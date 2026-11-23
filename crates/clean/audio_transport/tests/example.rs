//! A complete caller example using only the public interface: frames played with their cadence,
//! the State saved and restored between them, and the published replies read back.
#![expect(
    clippy::expect_used,
    reason = "Invented example values and saved state must be valid"
)]
use audio_transport::frame::{self, EffectCadence, FrameCadence, TurnCount};
use audio_transport::*;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::num::NonZeroU8;

/// One effect turn after the sender service, and one music turn.
const EVERY_FRAME: FrameCadence = FrameCadence {
    effects: EffectCadence::AroundService {
        before: TurnCount::ZERO,
        after: TurnCount::ONE,
    },
    music: TurnCount::ONE,
};
const fn positive(value: u8) -> NonZeroU8 {
    NonZeroU8::new(value).expect("positive invented duration")
}
const fn mp(value: u8) -> MusicalPitch {
    MusicalPitch::new(value).expect("admitted musical pitch")
}
const fn ep(value: u8) -> EffectPitch {
    EffectPitch::new(value).expect("admitted effect pitch")
}
fn invented_content() -> Content {
    let ordinary = EffectClip {
        parts: vec![EffectPartContent {
            spans: vec![EffectSpan {
                duration: EffectTurns(positive(20)),
                pitch: ep(24),
                timbre: TimbreId(0),
                motion: None,
            }],
            loop_start: None,
        }],
    };
    let door = EffectClip {
        parts: vec![
            EffectPartContent {
                spans: vec![EffectSpan {
                    duration: EffectTurns(positive(255)),
                    pitch: ep(13),
                    timbre: TimbreId(0),
                    motion: Some(PitchMotion {
                        speed: positive(200),
                        target: ep(14)
                    })
                }],
                loop_start: None
            };
            2
        ],
    };
    let effects = BTreeMap::from([
        (CueId::Footstep, ordinary.clone()),
        (CueId::Spin, ordinary.clone()),
        (CueId::EndSpin, ordinary.clone()),
        (CueId::Land, ordinary.clone()),
        (CueId::Beam, ordinary.clone()),
        (CueId::SamusSilence, ordinary.clone()),
        (CueId::BackgroundSilence, ordinary),
        (CueId::Door, door),
    ]);
    let track = TrackContent {
        phrases: vec![
            vec![
                MusicEvent::Timing {
                    duration: MusicTurns(positive(4)),
                    articulation: Some(Articulation {
                        ratio: 128,
                        strength: 64,
                    }),
                },
                MusicEvent::Tone(mp(24)),
                MusicEvent::Repeat {
                    phrase: 1,
                    count: positive(2),
                },
                MusicEvent::Complete,
            ],
            vec![
                MusicEvent::Ramp {
                    turns: positive(3),
                    target: 10,
                },
                MusicEvent::Sustain,
                MusicEvent::Complete,
            ],
        ],
    };
    Content {
        effects,
        tracks: vec![track.clone(), track],
    }
}
fn invented_song() -> SongState {
    let track = TrackState {
        position: TrackPosition::Root { cursor: 0 },
        note: MusicTurns(positive(1)),
        release: 0,
        duration: MusicTurns(positive(4)),
        articulation: 128,
        pitch: mp(24),
        loudness: Loudness(2560),
        ramp: None,
        timbre: TimbreId(0),
        offset: PitchOffset::Zero,
        strength: 64,
    };
    SongState {
        tracks: vec![track.clone(), track],
        mix: SongMix {
            master: Loudness(25600),
            percussion: 0,
            echo_voices: BTreeSet::new(),
            echo_left: 0,
            echo_right: 0,
            echo_delay: 2,
            feedback: 0,
            filter: 0,
            echo_enabled: false,
        },
    }
}
const fn quiet_group(history: ReleaseHistory) -> GroupState {
    GroupState {
        history,
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
/// The caller builds every part of the State, here at the start of play.
fn new_audio() -> State {
    let resident = ResidentMusicReply {
        identity: "ordinary caller arrangement".into(),
        original_byte: 42,
    };
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
        // The caller supplies every admitted history even though nothing plays.
        groups: enum_map::enum_map! {
            SoundGroup::Action => quiet_group(vec![ReleaseMemory::Inactive]),
            SoundGroup::Ambient => quiet_group(vec![ReleaseMemory::Inactive]),
            SoundGroup::Contact => quiet_group(vec![ReleaseMemory::Inactive, ReleaseMemory::Active(positive(2))]),
        },
        song: invented_song(),
        music_reply: resident.clone(),
        resident_music_reply: resident,
        reservations: BTreeSet::new(),
    }
}
fn save(audio: &State) -> toml::Value {
    toml::Value::try_from(audio).expect("save the whole State")
}
#[test]
fn complete_public_caller_example() {
    // The importer owns the validated content; the producer owns its offer and permission.
    let content = invented_content();
    let spin = Offer {
        cue: CueId::Spin,
        allowance: QueueAllowance::Three,
        permission: OfferPermission::Enabled,
    };
    let mut audio = new_audio();
    frame::play(&mut audio, &content, &[spin], EVERY_FRAME);
    assert_eq!(
        audio.request_lines[SoundGroup::Action],
        Request::Cue(CueId::Spin)
    );
    assert_eq!(
        audio.senders[SoundGroup::Action].progress,
        SenderProgress::AwaitCue(CueId::Spin)
    );
    // Restoring needs no initialization and replays nothing.
    let mut restored: State = save(&audio).try_into().expect("restore the whole State");
    assert_eq!(restored, audio);
    // The reply reaches the sender, which then clears the line and sees it acknowledged.
    for _ in 0..6 {
        frame::play(&mut restored, &content, &[], EVERY_FRAME);
        frame::play(&mut audio, &content, &[], EVERY_FRAME);
    }
    assert_eq!(restored, audio);
    assert_eq!(
        restored.senders[SoundGroup::Action].progress,
        SenderProgress::Ready
    );
    assert_eq!(restored.groups[SoundGroup::Action].reply, Request::Absent);
    // The spin still sounds after its request is cleared.
    assert_eq!(
        restored.groups[SoundGroup::Action].playback.owned_voices(),
        BTreeSet::from([EffectVoiceId::V7])
    );
    // The music line carries the resident reply; selection absence leaves it unchanged.
    assert_eq!(restored.music_reply, restored.resident_music_reply);
    assert_eq!(restored.music_reply.original_byte, 42);
}
#[test]
fn a_frame_completes_at_most_two_turns_of_a_kind() {
    assert_eq!(TurnCount::new(2), Some(TurnCount::TWO));
    assert_eq!(TurnCount::new(3), None);
}
#[test]
fn restore_refuses_an_overfull_queue() {
    let content = invented_content();
    let mut audio = new_audio();
    audio.senders[SoundGroup::Action].queue =
        CueQueue::try_from(VecDeque::from(vec![CueId::Spin; QUEUE_CAPACITY])).expect("full queue");
    let mut saved = save(&audio);
    let restored: State = saved.clone().try_into().expect("a full queue restores");
    assert_eq!(restored, audio);
    saved["senders"]["Action"]["queue"]
        .as_array_mut()
        .expect("saved queue")
        .push(toml::Value::String("Spin".into()));
    assert!(saved.try_into::<State>().is_err());
    // Even the largest allowance admits no cue to a full queue.
    let offer = Offer {
        cue: CueId::EndSpin,
        allowance: QueueAllowance::Fifteen,
        permission: OfferPermission::Enabled,
    };
    let quiet = FrameCadence {
        effects: EffectCadence::WithoutService(TurnCount::ZERO),
        music: TurnCount::ZERO,
    };
    frame::play(&mut audio, &content, &[offer], quiet);
    assert_eq!(audio, restored);
}

// Edit one saved field at a time, so neither a different invalid quantity nor a constructor can
// account for a refused restore.
fn restore_pitch<T>(
    original: &T,
    field: fn(&mut toml::Value) -> &mut toml::Value,
    pitch: u8,
    invalid: &[toml::Value],
) -> T
where
    T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + std::fmt::Debug,
{
    let mut saved = toml::Value::try_from(original).expect("save pitch fixture");
    assert_eq!(*field(&mut saved), toml::Value::Integer(i64::from(pitch)));
    let restored: T = saved.clone().try_into().expect("restore admitted endpoint");
    assert_eq!(&restored, original);
    assert_eq!(toml::Value::try_from(&restored).expect("resave"), saved);
    for bad in invalid {
        *field(&mut saved) = bad.clone();
        assert!(saved.clone().try_into::<T>().is_err(), "restored {bad}");
    }
    restored
}
#[test]
fn restore_musical_pitch_at_saved_boundaries() {
    let invalid = [
        toml::Value::Integer(18),
        toml::Value::Integer(52),
        toml::Value::Float(19.5),
    ];
    let one_music_turn = FrameCadence {
        effects: EffectCadence::WithoutService(TurnCount::ZERO),
        music: TurnCount::ONE,
    };
    for pitch in [19, 51] {
        let mut content = invented_content();
        // The first track sustains its saved pitch while the second sounds a saved tone.
        content.tracks[0].phrases[0][1] = MusicEvent::Sustain;
        content.tracks[1].phrases[0][1] = MusicEvent::Tone(mp(pitch));
        let content = restore_pitch(
            &content,
            |saved| &mut saved["tracks"][1]["phrases"][0][1]["Tone"],
            pitch,
            &invalid,
        );
        let mut audio = new_audio();
        audio.song.tracks[0].pitch = mp(pitch);
        frame::play(&mut audio, &content, &[], one_music_turn);
        assert_eq!(audio.song.tracks[0].pitch, mp(pitch));
        assert_eq!(audio.song.tracks[1].pitch, mp(pitch));
        let mut restored = restore_pitch(
            &audio,
            |saved| &mut saved["song"]["tracks"][0]["pitch"],
            pitch,
            &invalid,
        );
        frame::play(&mut restored, &content, &[], one_music_turn);
        frame::play(&mut audio, &content, &[], one_music_turn);
        assert_eq!(restored, audio);
    }
}
#[test]
fn restore_effect_pitch_at_saved_boundaries() {
    let invalid = [toml::Value::Integer(-1), toml::Value::Integer(72)];
    let one_effect_turn = FrameCadence {
        effects: EffectCadence::WithoutService(TurnCount::ONE),
        music: TurnCount::ZERO,
    };
    for pitch in [0, 71] {
        let mut content = invented_content();
        content
            .effects
            .get_mut(&CueId::Spin)
            .expect("spin clip")
            .parts[0]
            .spans[0]
            .pitch = ep(pitch);
        let content = restore_pitch(
            &content,
            |saved| &mut saved["effects"]["Spin"]["parts"][0]["spans"][0]["pitch"],
            pitch,
            &invalid,
        );
        let mut audio = new_audio();
        audio.request_lines[SoundGroup::Action] = Request::Cue(CueId::Spin);
        let action = &mut audio.groups[SoundGroup::Action];
        action.staged = Request::Cue(CueId::Spin);
        action.handled = action.staged;
        // The pitch holds through the release while the zero countdown wraps.
        action.history[0] = ReleaseMemory::Active(positive(2));
        action.playback = Playback::Active {
            cue: CueId::Spin,
            parts: vec![PartState::Live(LivePart {
                voice: EffectVoiceId::V7,
                cursor: 1,
                instruction: 0,
                pitch: ep(pitch),
                fraction: 31,
                motion: None,
            })],
        };
        frame::play(&mut audio, &content, &[], one_effect_turn);
        let mut restored = restore_pitch(
            &audio,
            |saved| &mut saved["groups"]["Action"]["playback"]["Active"]["parts"][0]["Live"]["pitch"],
            pitch,
            &invalid,
        );
        frame::play(&mut restored, &content, &[], one_effect_turn);
        frame::play(&mut audio, &content, &[], one_effect_turn);
        assert_eq!(restored, audio);
    }
}
