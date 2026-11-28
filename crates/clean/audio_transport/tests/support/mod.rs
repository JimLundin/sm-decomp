//! Effect fixtures and the families that play one sound group's effect turns.
mod frames;
mod identity;
mod resident;
mod retention;
use super::*;
pub use frames::frame_vector;
pub use resident::resident_vector;
pub use retention::{echo_vector, history_vector};
/// A group's effect playback and its content, as an effect vector declares them.
struct EffectFixture {
    group: SoundGroup,
    cue: CueId,
    audio: GroupState,
    content: Content,
    occupied: BTreeSet<EffectVoiceId>,
}
const fn fixture_part(cursor: usize, instruction: u8, voice: EffectVoiceId) -> PartState {
    PartState::Live(LivePart {
        voice,
        cursor,
        instruction,
        pitch: ep(0),
        fraction: 0,
        motion: None,
    })
}
fn effect_fixture(row: &Table) -> EffectFixture {
    let group: SoundGroup = named(text(row, "group"));
    let motion = row.get("slide_speed").map(|_| PitchMotion {
        speed: nz(byte(row, "slide_speed")),
        target: ep(byte(row, "slide_target")),
    });
    let cue = if motion.is_some() {
        CueId::Door
    } else {
        match group {
            SoundGroup::Action => CueId::Spin,
            SoundGroup::Ambient => CueId::BackgroundSilence,
            SoundGroup::Contact => CueId::Footstep,
        }
    };
    let intro = row
        .get("intro_parts")
        .map_or(0, |_| index(row, "intro_parts"));
    let loops = row
        .get("loop_parts")
        .map_or(0, |_| index(row, "loop_parts"));
    let part = EffectPartContent {
        spans: vec![
            EffectSpan {
                duration: EffectTurns(nz(byte(row, "duration"))),
                pitch: ep(byte(row, "pitch")),
                timbre: TimbreId(0),
                motion: motion.clone(),
            };
            intro.saturating_add(loops).max(1)
        ],
        loop_start: (loops > 0).then_some(intro),
    };
    let first_finished = row.contains_key("first_finished");
    let parts = if motion.is_some() || first_finished {
        2
    } else {
        1
    };
    let mut history = release_fixture(row, if group == SoundGroup::Contact { 2 } else { 1 });
    let playback = if row.contains_key("prepared_remaining") {
        let remaining = byte(row, "prepared_remaining");
        Playback::Active {
            cue,
            parts: vec![
                fixture_part(0, remaining, EffectVoiceId::V7),
                fixture_part(0, remaining, EffectVoiceId::V6),
            ],
        }
    } else if first_finished {
        assert!(flag(row, "first_finished"));
        history[1] = ReleaseMemory::Active(nz(1));
        Playback::Active {
            cue,
            parts: vec![PartState::Finished, fixture_part(0, 1, EffectVoiceId::V6)],
        }
    } else {
        Playback::Prepared { cue }
    };
    let occupied = row
        .get("occupied")
        .map_or_else(BTreeSet::new, |_| effect_voices(row, "occupied"));
    EffectFixture {
        group,
        cue,
        audio: GroupState {
            history,
            handled: Request::Cue(cue),
            staged: Request::Cue(cue),
            reply: Request::Absent,
            playback,
        },
        content: content(
            BTreeMap::from([(
                cue,
                EffectClip {
                    parts: vec![part; parts],
                },
            )]),
            Vec::new(),
        ),
        occupied,
    }
}
fn release_fixture(row: &Table, count: usize) -> ReleaseHistory {
    let declared = row
        .get("release")
        .map_or_else(Vec::new, |_| ints(row, "release"));
    declared
        .into_iter()
        .map(|v| {
            NonZeroU8::new(u8::try_from(v).expect("release"))
                .map_or(ReleaseMemory::Inactive, ReleaseMemory::Active)
        })
        .chain(std::iter::repeat(ReleaseMemory::Inactive))
        .take(count)
        .collect()
}
/// A State holding the fixture's group, its cue steady on the group's request line, and the
/// fixture's occupied voices as external reservations.
fn effect_state(fixture: &EffectFixture, mix: &Value) -> State {
    let mut state = fresh_state(silent_song(mix));
    state.groups[fixture.group] = fixture.audio.clone();
    state.request_lines[fixture.group] = Request::Cue(fixture.cue);
    state.reservations.clone_from(&fixture.occupied);
    state
}
fn live_parts(playback: &Playback) -> Vec<(usize, &LivePart)> {
    let Playback::Active { parts, .. } = playback else {
        return Vec::new();
    };
    parts
        .iter()
        .enumerate()
        .filter_map(|(identity, part)| match part {
            PartState::Live(live) => Some((identity, live)),
            PartState::Finished => None,
        })
        .collect()
}
/// `turns` frames of one effect turn each.
pub fn effect_vector(row: &Table, mix: &Value) {
    keys(
        row,
        "id group pitch duration slide_speed slide_target release turns after_pitch after_fraction after_instruction after_release prepared_remaining first_finished intro_parts loop_parts after_span occupied after_voices after_owned",
    );
    let fixture = effect_fixture(row);
    let mut state = effect_state(&fixture, mix);
    for _ in 0..unsigned(row, "turns") {
        play(&mut state, &fixture.content, &[], ONE_EFFECT_TURN);
    }
    let audio = &state.groups[fixture.group];
    assert!(
        matches!(audio.playback, Playback::Active { .. }),
        "allocation has completed"
    );
    let id = text(row, "id");
    for (identity, live) in live_parts(&audio.playback) {
        for (key, actual) in [
            ("after_pitch", u64::from(live.pitch.get())),
            ("after_fraction", u64::from(live.fraction)),
            ("after_instruction", u64::from(live.instruction)),
            (
                "after_release",
                u64::from(audio.history[identity].remaining()),
            ),
            (
                "after_span",
                u64::try_from(live.cursor.saturating_sub(1)).expect("span"),
            ),
        ] {
            if row.contains_key(key) {
                assert_eq!(actual, unsigned(row, key), "{id} {key}");
            }
        }
    }
    if row.contains_key("after_voices") {
        let voices: Vec<_> = live_parts(&audio.playback)
            .into_iter()
            .map(|(_, live)| live.voice)
            .collect();
        let expected: Vec<_> = ints(row, "after_voices")
            .into_iter()
            .map(effect_voice)
            .collect();
        assert_eq!(voices, expected, "{id}");
    }
    if row.contains_key("after_owned") {
        assert_eq!(
            u64::try_from(audio.playback.owned_voices().len()).expect("ownership"),
            unsigned(row, "after_owned"),
            "{id}"
        );
    }
}
#[test]
fn shaft_explosion_plays_one_finite_ambient_part() {
    let root: Table =
        toml::from_str(&vector_file::read("audio-transport").unwrap()).expect("fixture mix");
    let mut state = fresh_state(silent_song(&root["fixture_mix"]));
    let cue = CueId::ShaftExplosion;
    let clip = EffectClip {
        parts: vec![EffectPartContent {
            spans: vec![EffectSpan {
                duration: EffectTurns(nz(1)),
                pitch: ep(24),
                timbre: TimbreId(0),
                motion: None,
            }],
            loop_start: None,
        }],
    };
    let content = content(BTreeMap::from([(cue, clip)]), Vec::new());
    let offer = Offer {
        cue,
        allowance: QueueAllowance::Six,
        permission: OfferPermission::Enabled,
    };
    let group = SoundGroup::Ambient;
    play(&mut state, &content, &[offer], ONE_EFFECT_TURN);
    play(&mut state, &content, &[], ONE_EFFECT_TURN);
    assert_eq!(state.groups[group].reply, Request::Cue(cue));
    assert_eq!(state.groups[group].playback, Playback::Prepared { cue });

    play(&mut state, &content, &[], ONE_EFFECT_TURN);
    assert_eq!(
        state.groups[group].playback.owned_voices(),
        BTreeSet::from([EffectVoiceId::V7])
    );
    // The next three turns start the span, enter ordinary release, then finish the clip.
    for _ in 0..3 {
        play(&mut state, &content, &[], ONE_EFFECT_TURN);
    }
    assert_eq!(
        state.groups[group].playback,
        Playback::Active {
            cue,
            parts: vec![PartState::Finished],
        }
    );
}
/// One Contact effect turn: it handles the staged request, publishes the request it handled, and
/// stages the request line.
pub fn staged_vector(row: &Table, mix: &Value) {
    keys(
        row,
        "id handled staged protected decision after_handled after_reply",
    );
    let group = SoundGroup::Contact;
    let playing = if flag(row, "protected") {
        CueId::Door
    } else {
        CueId::Footstep
    };
    let clip = EffectClip {
        parts: vec![EffectPartContent {
            spans: vec![EffectSpan {
                duration: EffectTurns(nz(10)),
                pitch: ep(24),
                timbre: TimbreId(0),
                motion: None,
            }],
            loop_start: None,
        }],
    };
    let content = content(
        BTreeMap::from([(CueId::Footstep, clip.clone()), (CueId::Door, clip)]),
        Vec::new(),
    );
    let staged = request_named(text(row, "staged"));
    let mut state = fresh_state(silent_song(mix));
    state.groups[group] = GroupState {
        history: vec![ReleaseMemory::Inactive; 2],
        handled: request_named(text(row, "handled")),
        staged,
        reply: Request::Absent,
        playback: Playback::Active {
            cue: playing,
            parts: vec![PartState::Live(LivePart {
                voice: EffectVoiceId::V7,
                cursor: 1,
                instruction: 10,
                pitch: ep(24),
                fraction: 0,
                motion: None,
            })],
        },
    };
    // The line still carries the staged request.
    state.request_lines[group] = staged;
    play(&mut state, &content, &[], ONE_EFFECT_TURN);
    let audio = &state.groups[group];
    let id = text(row, "id");
    assert_eq!(
        audio.handled,
        request_named(text(row, "after_handled")),
        "{id}"
    );
    assert_eq!(audio.reply, request_named(text(row, "after_reply")), "{id}");
    assert_eq!(audio.staged, state.request_lines[group], "{id}");
    let decision = if matches!(audio.playback, Playback::Prepared { .. }) {
        "Prepare"
    } else {
        "Continue"
    };
    assert_eq!(text(row, "decision"), decision, "{id}");
}
