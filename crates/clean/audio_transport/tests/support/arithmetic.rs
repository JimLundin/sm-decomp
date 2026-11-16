//! Exact arithmetic and actual scheduler boundary proofs beyond machine words.
use super::*;
use num_bigint::{BigInt, BigUint};
use num_traits::One;
fn big(bits: usize) -> BigUint {
    BigUint::one() << bits
}
fn fraction(n: BigUint, d: BigUint) -> AudioInstant {
    AudioInstant::new(n, d).expect("positive exact fraction")
}
fn music_fixture() -> (SongState, Content) {
    let root: Table = toml::from_str(SOURCE).expect("vectors");
    let row = array(&root, "music_work")
        .iter()
        .map(|v| v.as_table().expect("work vector"))
        .find(|r| text(r, "id") == "W-extra-three")
        .expect("timing fixture");
    let (song, tracks) = song(row, &root["fixture_mix"]);
    (
        song,
        Content {
            effects: BTreeMap::new(),
            tracks,
        },
    )
}
#[test]
fn exact_fraction_input_and_restoration() {
    let d = big(131) + 1u64;
    let a = fraction(BigUint::one(), d.clone());
    let b = fraction(BigUint::one(), &d + 1u64);
    let difference = time::subtract(&a, &b);
    assert_eq!(difference, fraction(BigUint::one(), &d * (&d + 1u64)));
    assert_eq!(time::add(&difference, &b), a);
    let large = time::whole(big(256));
    assert!(time::add(&large, &a) > large);
    let raw = InstantFraction {
        numerator: BigInt::from(6),
        denominator: BigInt::from(8),
    };
    let restored: AudioInstant = Value::try_from(&raw)
        .expect("raw fraction")
        .try_into()
        .expect("normalize on restoration");
    assert_eq!(
        restored,
        time::instant(3, NonZeroU64::new(4).expect("positive"))
    );
    assert_eq!(
        InstantFraction::from(restored),
        InstantFraction {
            numerator: BigInt::from(3),
            denominator: BigInt::from(4)
        }
    );
    for (n, d, expected) in [
        (1, 0, FractionError::NonpositiveDenominator),
        (1, -1, FractionError::NonpositiveDenominator),
        (-1, 1, FractionError::NegativeNumerator),
        (-1, -1, FractionError::NonpositiveDenominator),
    ] {
        let raw = InstantFraction {
            numerator: n.into(),
            denominator: d.into(),
        };
        assert_eq!(AudioInstant::try_from(raw.clone()), Err(expected));
        let serialized = Value::try_from(&raw).expect("invalid fraction serialization");
        assert!(serialized.try_into::<AudioInstant>().is_err());
    }
    let n = big(201) + 3u64;
    let d = big(199) + 1u64;
    let raw = InstantFraction {
        numerator: (&n * 6u64).into(),
        denominator: (&d * 6u64).into(),
    };
    let restored: AudioInstant = Value::try_from(&raw)
        .expect("large unreduced fraction")
        .try_into()
        .expect("normalized exact restore");
    assert_eq!(restored, fraction(n, d));
    assert_eq!(
        Value::try_from(&restored).expect("canonical serialization"),
        Value::try_from(InstantFraction::from(restored)).expect("normalized components")
    );
    for instant in [time::cycles(2048), time::whole(big(200))] {
        assert_eq!(
            AudioCyclePhase::try_from(instant.clone()),
            Err(PhaseOutOfRange)
        );
        assert!(
            Value::try_from(&instant)
                .expect("phase input")
                .try_into::<AudioCyclePhase>()
                .is_err()
        );
    }
    let zero = fraction(BigUint::from(0u64), big(256));
    assert_eq!(
        InstantFraction::from(zero),
        InstantFraction {
            numerator: 0.into(),
            denominator: 1.into()
        }
    );
}
#[test]
fn unbounded_work_publication_and_observations_resume_exactly() {
    let (song, mut content) = music_fixture();
    let extra = big(256) + 17u64;
    let MusicEvent::Timing {
        extra: saved_extra, ..
    } = &mut content.tracks[0].phrases[0][0]
    else {
        assert!(false);
        return;
    };
    *saved_extra = ExtraHandlingWork(extra.clone());
    let result = music::turn(&song, &content.tracks);
    // 651 common + 159 + 23 + 38 + 71 + 523 tone + 159 other + 310 volume.
    assert_eq!(result.work, ExactAudioCycles(&extra + 1934u64));
    let origin = time::add(
        time::whole(big(200)),
        fraction(BigUint::one(), big(131) + 1u64),
    );
    let epsilon = fraction(BigUint::one(), big(257) + 3u64);
    let mut state = state_fixture(song.clone(), 0, 0);
    state.now = origin.clone();
    state.clock.phase =
        AudioCyclePhase::try_from(fraction(big(130), big(131) + 1u64)).expect("exact timer phase");
    state.phase = AudioPhase::MusicStart { at: origin.clone() };
    state.music_due = true;
    assert_eq!(
        transport::advance(&mut state, &content, time::cycles(0), &[]),
        Vec::new()
    );
    let at = time::add(&origin, time::whole(result.work.0.clone()));
    assert_eq!(
        state.phase,
        AudioPhase::Work {
            at: at.clone(),
            result: CompletionResult::Music(result.song.clone())
        }
    );
    let initial = roundtrip(&state);
    let before = time::subtract(time::whole(result.work.0.clone()), &epsilon);
    assert_eq!(
        transport::advance(&mut state, &content, before, &[]),
        Vec::new()
    );
    assert_eq!(state.song, song);
    assert_eq!(state.now, time::subtract(&at, &epsilon));
    let phase_fraction = fraction(big(130), big(131) + 1u64);
    assert_eq!(state.clock.unread, 0);
    assert_eq!(
        state.clock.phase.instant(),
        &time::subtract(time::add(time::cycles(1951), &phase_fraction), &epsilon)
    );
    let mut restored = roundtrip(&state);
    assert_eq!(restored, state);
    assert_eq!(
        transport::advance(&mut restored, &content, time::cycles(0), &[]),
        Vec::new()
    );
    let publication = transport::advance(&mut restored, &content, epsilon.clone(), &[]);
    assert_eq!(
        publication,
        vec![transport::Output::Publication {
            at: at.clone(),
            line: ReplyLine::Music,
            reply: Reply::Music(restored.resident_music_reply.clone()),
            result: CompletionResult::Music(result.song.clone())
        }]
    );
    let mut direct = initial;
    assert_eq!(
        transport::advance(&mut direct, &content, time::whole(result.work.0), &[]),
        publication
    );
    assert_eq!(restored, direct);
    assert_eq!(
        restored.clock.phase.instant(),
        &time::add(time::cycles(1951), phase_fraction)
    );
    observe_large_publication(&restored, &content, &at, epsilon);
}
fn observe_large_publication(
    saved: &State,
    content: &Content,
    at: &AudioInstant,
    epsilon: AudioInstant,
) {
    let mut restored = roundtrip(saved);
    let first = time::add(at, time::halves(9));
    let second = time::add(at, time::halves(19));
    assert_eq!(
        transport::advance(&mut restored, content, time::halves(9), &[]),
        vec![transport::Output::Observation {
            at: first,
            line: ReplyLine::Music,
            request: Request::Absent
        }]
    );
    restored = roundtrip(&restored);
    assert_eq!(
        transport::advance(
            &mut restored,
            content,
            time::subtract(time::cycles(5), &epsilon),
            &[]
        ),
        Vec::new()
    );
    restored = roundtrip(&restored);
    let accepted = transport::advance(&mut restored, content, epsilon, &[]);
    assert_eq!(
        accepted,
        vec![
            transport::Output::Observation {
                at: second.clone(),
                line: ReplyLine::Music,
                request: Request::Absent
            },
            transport::Output::Staged {
                at: second,
                line: ReplyLine::Music,
                request: Request::Absent
            }
        ]
    );
    let episode = time::add(at, time::halves(1001));
    assert_eq!(restored.clock.lattice, episode);
    assert_eq!(
        restored.phase,
        AudioPhase::ObserveClock {
            at: episode.clone()
        }
    );
    let continued = transport::advance(&mut restored, content, time::halves(982), &[]);
    assert_eq!(
        continued,
        vec![transport::Output::Clock {
            at: episode,
            batch: 1
        }]
    );
    assert!(&restored.now > at);
}
#[test]
fn exact_effect_publication_and_observation_time_updates() {
    let root: Table = toml::from_str(SOURCE).expect("vectors");
    let row = array(&root, "effect")
        .iter()
        .map(|v| v.as_table().expect("effect"))
        .find(|r| text(r, "id") == "E-door-direct")
        .expect("effect fixture");
    let (group, before, mut content, occupied) = effect_fixture(row);
    let (song, song_content) = music_fixture();
    content.tracks = song_content.tracks;
    let result = effects::turn(group, &before, &content, &occupied, &BTreeSet::new());
    assert_eq!(result.work, AudioCycles(4707));
    let origin = time::add(
        time::whole(big(300)),
        fraction(BigUint::one(), big(149) + 3u64),
    );
    let mut state = state_fixture(song, 0, 0);
    state.groups[group] = before.clone();
    state.now = time::add(&origin, time::halves(19));
    state.phase = AudioPhase::Sample {
        sample: StableSample {
            line: ReplyLine::Group(SoundGroup::Ambient),
            anchor: origin.clone(),
            first: Some(Request::Absent),
        },
        next: Continuation::Effect(group),
    };
    let sampled = transport::advance(&mut state, &content, time::cycles(0), &[]);
    assert_eq!(sampled.len(), 2);
    let at = time::add(&origin, time::cycles(4707));
    assert_eq!(
        state.phase,
        AudioPhase::Work {
            at: at.clone(),
            result: CompletionResult::Effect {
                group,
                result: result.group.clone(),
                echo_voices: result.echo_voices.clone(),
            }
        }
    );
    let epsilon = fraction(BigUint::one(), big(251) + 1u64);
    let remaining = time::subtract(time::subtract(&at, &state.now), &epsilon);
    assert_eq!(
        transport::advance(&mut state, &content, remaining, &[]),
        Vec::new()
    );
    assert_eq!(state.groups[group], before);
    state = roundtrip(&state);
    assert_eq!(
        transport::advance(&mut state, &content, epsilon, &[]),
        vec![transport::Output::Publication {
            at: at.clone(),
            line: ReplyLine::Group(group),
            reply: Reply::Effect(result.group.reply),
            result: CompletionResult::Effect {
                group,
                result: result.group,
                echo_voices: result.echo_voices,
            }
        }]
    );
    observe_large_effect(&mut state, &content, group, &at);
}
fn observe_large_effect(
    state: &mut State,
    content: &Content,
    group: SoundGroup,
    at: &AudioInstant,
) {
    let first = time::add(at, time::halves(9));
    let input = transport::Input::cue(first.clone(), CueId::Footstep);
    let initial = roundtrip(state);
    let output = transport::advance(
        state,
        content,
        time::halves(9),
        std::slice::from_ref(&input),
    );
    assert_eq!(
        output,
        vec![transport::Output::Observation {
            at: first,
            line: ReplyLine::Group(group),
            request: Request::Cue(CueId::Footstep)
        }]
    );
    assert_eq!(state.groups[group].staged, Request::Cue(CueId::Door));
    *state = roundtrip(state);
    let second = time::add(at, time::halves(19));
    let mut split = output;
    split.extend(transport::advance(state, content, time::cycles(5), &[]));
    assert_eq!(state.groups[group].staged, Request::Cue(CueId::Footstep));
    let mut direct = initial;
    assert_eq!(
        transport::advance(&mut direct, content, time::halves(19), &[input]),
        split
    );
    assert_eq!(*state, direct);
    assert_eq!(
        split.last(),
        Some(&transport::Output::Staged {
            at: second,
            line: ReplyLine::Group(group),
            request: Request::Cue(CueId::Footstep)
        })
    );
}
#[test]
fn extra_work_consumed_once_through_repeat_return_and_restart() {
    let (mut song, mut content) = music_fixture();
    let timing = MusicEvent::Timing {
        duration: MusicTurns(nz(8)),
        articulation: None,
        extra: ExtraHandlingWork::new(0),
    };
    content.tracks[0].phrases = vec![
        vec![
            timing.clone(),
            MusicEvent::Tone(mp(24)),
            MusicEvent::Repeat {
                phrase: 1,
                count: nz(2),
            },
            timing.clone(),
            MusicEvent::Sustain,
            MusicEvent::Complete,
        ],
        vec![timing, MusicEvent::Sustain, MusicEvent::Complete],
    ];
    song.tracks[0].note = MusicTurns(nz(1));
    let positions = [
        TrackPosition::Root { cursor: 0 },
        TrackPosition::Repeated {
            phrase: 1,
            cursor: 2,
            remaining: nz(2),
            continuation: 3,
        },
        TrackPosition::Repeated {
            phrase: 1,
            cursor: 2,
            remaining: nz(1),
            continuation: 3,
        },
        TrackPosition::Root { cursor: 5 },
    ];
    let extra = big(300) + 9u64;
    for position in positions {
        song.tracks[0].position = position;
        let ordinary = music::turn(&song, &content.tracks);
        let preview = music::preview(&song.tracks[0], &content.tracks[0]);
        let mut extended = content.clone();
        for event in extended.tracks[0].phrases.iter_mut().flatten() {
            if let MusicEvent::Timing { extra: value, .. } = event {
                *value = ExtraHandlingWork(extra.clone());
            }
        }
        let extended: Content = Value::try_from(&extended)
            .expect("save exact content")
            .try_into()
            .expect("restore exact content");
        let actual = music::turn(&song, &extended.tracks);
        assert_eq!(actual.song, ordinary.song);
        assert_eq!(actual.work.0, &ordinary.work.0 + &extra);
        assert_eq!(
            music::preview(&song.tracks[0], &extended.tracks[0]),
            preview
        );
    }
}
#[test]
fn unbounded_cpu_observation_presentation_and_completion() {
    let origin = big(256) + 1u64;
    let mut state = SenderState {
        queue: VecDeque::from([CueId::EndSpin]),
        progress: SenderProgress::AwaitClear,
        reply: Request::Cue(CueId::Spin),
        now: ExactCpuMasterTicks(origin.clone()),
        pending: None,
    };
    sender::begin(&mut state, sender::SenderProfile::FastBus);
    let initial = state.clone();
    assert_eq!(sender::advance(&mut state, CpuMasterTicks(57)), Vec::new());
    state = Value::try_from(&state)
        .expect("save sender")
        .try_into()
        .expect("restore sender");
    let updates = [sender::ReplyInput {
        at: ExactCpuMasterTicks(&origin + 58u64),
        reply: Request::Absent,
    }];
    let observation = sender::advance_with_inputs(&mut state, CpuMasterTicks(1), &updates);
    assert_eq!(
        observation,
        vec![sender::SenderOutput::Observation(
            ExactCpuMasterTicks(&origin + 58u64),
            Request::Absent
        )]
    );
    assert_eq!(state.progress, SenderProgress::AwaitClear);
    assert_eq!(sender::advance(&mut state, CpuMasterTicks(283)), Vec::new());
    state = Value::try_from(&state)
        .expect("save presentation")
        .try_into()
        .expect("restore presentation");
    let presentation = sender::advance(&mut state, CpuMasterTicks(1));
    assert_eq!(
        presentation,
        vec![sender::SenderOutput::Presentation(
            ExactCpuMasterTicks(&origin + 342u64),
            Request::Cue(CueId::EndSpin)
        )]
    );
    assert_eq!(sender::advance(&mut state, CpuMasterTicks(185)), Vec::new());
    state = Value::try_from(&state)
        .expect("save completion")
        .try_into()
        .expect("restore completion");
    let completion = sender::advance(&mut state, CpuMasterTicks(1));
    assert_eq!(
        completion,
        vec![sender::SenderOutput::Completed(ExactCpuMasterTicks(
            &origin + 528u64
        ))]
    );
    assert_eq!(sender::advance(&mut state, CpuMasterTicks(0)), Vec::new());
    let mut direct = initial;
    let mut expected = observation;
    expected.extend(presentation);
    expected.extend(completion);
    assert_eq!(
        sender::advance_with_inputs(&mut direct, CpuMasterTicks(528), &updates),
        expected
    );
    assert_eq!(direct, state);
    assert_eq!(
        sender::advance(&mut state, ExactCpuMasterTicks(big(300))),
        Vec::new()
    );
    assert_eq!(state.now, ExactCpuMasterTicks(origin + 528u64 + big(300)));
}
