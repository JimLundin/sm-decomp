//! A complete compiling caller example using only public model interfaces.
#![expect(
    clippy::expect_used,
    reason = "Invented example values and saved state must be valid"
)]
use audio_transport::*;
use audio_transport::{sender, time, transport};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::num::NonZeroU8;

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
                    extra: ExtraHandlingWork::new(3),
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
const fn inactive_group(history: ReleaseHistory) -> GroupState {
    GroupState {
        history,
        handled: Request::Absent,
        staged: Request::Absent,
        reply: Request::Absent,
        playback: Playback::Empty,
    }
}
fn initialized_audio() -> State {
    let mut state = State {
        now: time::cycles(0),
        clock: ClockState {
            phase: AudioCyclePhase::try_from(time::cycles(531)).expect("phase inside timer period"),
            unread: 0,
            lattice: time::halves(811),
        },
        cadence: CadencePhase {
            effect: 7,
            music: 224,
        },
        music_due: false,
        // The caller supplies all admitted histories even though playback is empty.
        groups: enum_map::enum_map! {
            SoundGroup::Action => inactive_group(vec![ReleaseMemory::Inactive]),
            SoundGroup::Ambient => inactive_group(vec![ReleaseMemory::Inactive]),
            SoundGroup::Contact => inactive_group(vec![ReleaseMemory::Inactive, ReleaseMemory::Active(positive(2))])
        },
        song: invented_song(),
        resident_music_reply: ResidentMusicReply {
            identity: "ordinary caller arrangement".into(),
            original_byte: 42,
        },
        reservations: BTreeSet::new(),
        requests: enum_map::enum_map! { SoundGroup::Action => Request::Absent, SoundGroup::Ambient => Request::Absent, SoundGroup::Contact => Request::Absent },
        phase: AudioPhase::ObserveClock {
            at: time::cycles(0),
        },
    };
    // Initialization chooses the first polling opportunity. Restoration does not call this.
    transport::initialize(&mut state);
    state
}
#[test]
fn complete_public_caller_example() {
    // Importer owns the validated musical graph, resolved timbres and effect resources.
    let content = invented_content();
    // Request producer owns permissions, offers and the queue; platform owns CPU time.
    let mut sender = SenderState {
        queue: VecDeque::new(),
        progress: SenderProgress::Ready,
        reply: Request::Absent,
        now: CpuMasterTicks(0),
        pending: None,
    };
    sender::offer(
        &mut sender,
        CueId::Spin,
        QueueAllowance::Three,
        OfferPermission::Enabled,
    );
    sender::begin(&mut sender, sender::SenderProfile::FastBus);
    let cpu_publications = sender::advance(&mut sender, CpuMasterTicks(398));
    assert_eq!(
        cpu_publications,
        vec![
            sender::SenderOutput::Presentation(CpuMasterTicks(212), Request::Cue(CueId::Spin)),
            sender::SenderOutput::Completed(CpuMasterTicks(398))
        ]
    );
    // The platform, not the model, maps sender presentations into ordered audio instants.
    // No conversion from CPU ticks to audio cycles is implicit.
    let inputs = [
        transport::Input::cue(time::halves(3217), CueId::Spin),
        transport::Input::cue(time::cycles(1800), CueId::Door),
    ];
    let mut audio = initialized_audio();
    let first = transport::advance(&mut audio, &content, time::cycles(1603), &[]);
    assert!(
        !first
            .iter()
            .any(|out| matches!(out, transport::Output::Publication { .. }))
    );
    // Saving full retained state includes the pending result, prior committed song,
    // exact time, timer/count phases, observation anchor and already visible inputs.
    let saved = toml::Value::try_from(&audio).expect("save complete retained state");
    let mut restored: State = saved.try_into().expect("restore without initialization");
    assert_eq!(
        transport::advance(&mut restored, &content, time::cycles(0), &[]),
        Vec::new()
    );
    let outputs = transport::advance(&mut restored, &content, time::cycles(4000), &inputs);
    // The caller consumes ordered publications and observations, never side effects.
    assert!(
        outputs
            .iter()
            .any(|output| matches!(output, transport::Output::Publication { .. }))
    );
    // Music-selection absence does not erase the supplied resident identity or encoding.
    assert!(outputs.iter().any(|output| matches!(output,
        transport::Output::Publication { reply: Reply::Music(identity), .. }
        if identity == &audio.resident_music_reply
    )));
    assert_eq!(restored.resident_music_reply, audio.resident_music_reply);
    let mut uninterrupted = initialized_audio();
    let mut expected = transport::advance(&mut uninterrupted, &content, time::cycles(1603), &[]);
    expected.extend(transport::advance(
        &mut uninterrupted,
        &content,
        time::cycles(4000),
        &inputs,
    ));
    let mut resumed = first;
    resumed.extend(outputs);
    assert_eq!(resumed, expected);
    assert_eq!(restored, uninterrupted);
}

// Spec: Saved quantities. Edit one saved field at a time so neither a different invalid
// quantity nor a constructor can account for a failed restore.
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

fn resume_pending_pitch(mut restored: State, mut direct: State, content: &Content) {
    let deadline = if let AudioPhase::Work { at, .. } = &direct.phase {
        Some(at.clone())
    } else {
        None
    };
    let at = deadline.expect("pending pitch work");
    assert_eq!(restored.now, time::cycles(1603));
    assert_eq!(
        transport::advance(&mut restored, content, time::cycles(0), &[]),
        []
    );
    assert_eq!(restored, direct);

    let inputs = [transport::Input::cue(time::cycles(1608), CueId::Beam)];
    let before = time::subtract(time::subtract(&at, &restored.now), time::halves(1));
    assert_eq!(
        transport::advance(&mut restored, content, before, &inputs),
        []
    );
    assert_eq!(
        restored.requests[SoundGroup::Action],
        Request::Cue(CueId::Beam)
    );
    let output = transport::advance(&mut restored, content, time::halves(1), &[]);
    assert!(
        matches!(output.as_slice(), [transport::Output::Publication { at: instant, .. }] if instant == &at)
    );
    let elapsed = time::subtract(&at, &direct.now);
    assert_eq!(
        output,
        transport::advance(&mut direct, content, elapsed, &inputs)
    );
    assert_eq!(restored, direct);
    assert_eq!(
        transport::advance(&mut restored, content, time::cycles(0), &[]),
        []
    );
}

#[test]
fn restore_musical_pitch_at_saved_boundaries() {
    let invalid = [
        toml::Value::Integer(18),
        toml::Value::Integer(52),
        toml::Value::Float(19.5),
    ];
    for pitch in [19, 51] {
        let mut content = invented_content();
        // The first track sustains its saved pitch while the second reads a saved tone.
        content.tracks[0].phrases[0][1] = MusicEvent::Sustain;
        content.tracks[1].phrases[0][1] = MusicEvent::Tone(mp(pitch));
        let content = restore_pitch(
            &content,
            |saved| &mut saved["tracks"][1]["phrases"][0][1]["Tone"],
            pitch,
            &invalid,
        );
        let mut audio = initialized_audio();
        audio.song.tracks[0].pitch = mp(pitch);
        audio.cadence.effect = 0;
        transport::advance(&mut audio, &content, time::cycles(1603), &[]);
        let restored = restore_pitch(
            &audio,
            |saved| &mut saved["song"]["tracks"][0]["pitch"],
            pitch,
            &invalid,
        );
        let restored = restore_pitch(
            &restored,
            |saved| &mut saved["phase"]["Work"]["result"]["Music"]["tracks"][0]["pitch"],
            pitch,
            &invalid,
        );
        resume_pending_pitch(restored, audio, &content);
    }
}

#[test]
fn restore_effect_pitch_at_saved_boundaries() {
    let invalid = [toml::Value::Integer(-1), toml::Value::Integer(72)];
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
        let mut audio = initialized_audio();
        let action = &mut audio.groups[SoundGroup::Action];
        action.staged = Request::Cue(CueId::Spin);
        action.handled = action.staged;
        // Pitch is retained during release while the instruction countdown wraps.
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
        transport::advance(&mut audio, &content, time::cycles(1603), &[]);
        let restored = restore_pitch(
            &audio,
            |saved| &mut saved["groups"]["Action"]["playback"]["Active"]["parts"][0]["Live"]["pitch"],
            pitch,
            &invalid,
        );
        let restored = restore_pitch(
            &restored,
            |saved| {
                &mut saved["phase"]["Work"]["result"]["Effect"]["result"]["playback"]["Active"]["parts"]
                    [0]["Live"]["pitch"]
            },
            pitch,
            &invalid,
        );
        resume_pending_pitch(restored, audio, &content);
    }
}
