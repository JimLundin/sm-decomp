//! Physical fixture construction and ordered effect checks.
mod arithmetic;
mod identity;
mod resident;
mod retention;
use super::effect_voice as voice;
use super::*;
use audio_transport::EffectVoiceId as VoiceId;
use num_traits::ToPrimitive;
pub use resident::resident_vector;
pub use retention::{echo_vector, history_vector};
fn effect_fixture(row: &Table) -> (SoundGroup, GroupState, Content, BTreeSet<VoiceId>) {
    let group = group_name(text(row, "group"));
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
    let count = if motion.is_some() { 2 } else { 1 };
    let intro = usize::try_from(optional(row, "intro_parts", 0)).expect("intro");
    let loops = usize::try_from(optional(row, "loop_parts", 0)).expect("loop");
    let spans = if loops == 0 {
        1
    } else {
        intro.saturating_add(loops)
    };
    let part = EffectPartContent {
        spans: (0..spans)
            .map(|_| EffectSpan {
                duration: EffectTurns(nz(byte(row, "duration"))),
                pitch: ep(byte(row, "pitch")),
                timbre: TimbreId(0),
                motion: motion.clone(),
            })
            .collect(),
        loop_start: (loops > 0).then_some(intro),
    };
    let clip = EffectClip {
        parts: vec![part; count],
    };
    let mut releases = release_fixture(row, if group == SoundGroup::Contact { 2 } else { 1 });
    if row.contains_key("first_finished") {
        releases[1] = ReleaseMemory::Active(nz(1));
    }
    let playback = if row.get("prepared_remaining").is_some() {
        Playback::Active {
            cue,
            parts: [VoiceId::V7, VoiceId::V6]
                .into_iter()
                .map(|voice| {
                    PartState::Live(LivePart {
                        voice,
                        cursor: 0,
                        instruction: byte(row, "prepared_remaining"),
                        pitch: ep(0),
                        fraction: 0,
                        motion: None,
                    })
                })
                .collect(),
        }
    } else if row.get("first_finished").is_some() {
        assert!(flag(row, "first_finished"));
        Playback::Active {
            cue,
            parts: vec![
                PartState::Finished,
                PartState::Live(LivePart {
                    voice: VoiceId::V6,
                    cursor: 0,
                    instruction: 1,
                    pitch: ep(0),
                    fraction: 0,
                    motion: None,
                }),
            ],
        }
    } else {
        Playback::Prepared { cue }
    };
    let mut clip = clip;
    if row.get("first_finished").is_some() {
        clip.parts.push(clip.parts[0].clone());
    }
    let content = Content {
        effects: BTreeMap::from([(cue, clip)]),
        tracks: Vec::new(),
    };
    let occupied = row.get("occupied").map_or_else(BTreeSet::new, |_| {
        ints(row, "occupied").into_iter().map(voice).collect()
    });
    (
        group,
        GroupState {
            history: releases,
            handled: Request::Cue(cue),
            staged: Request::Cue(cue),
            reply: Request::Absent,
            playback,
        },
        content,
        occupied,
    )
}
fn release_fixture(row: &Table, count: usize) -> Vec<ReleaseMemory> {
    row.get("release").map_or_else(
        || vec![ReleaseMemory::Inactive; count],
        |_| {
            ints(row, "release")
                .iter()
                .map(|v| {
                    NonZeroU8::new(u8::try_from(*v).expect("release"))
                        .map_or(ReleaseMemory::Inactive, ReleaseMemory::Active)
                })
                .chain(std::iter::repeat(ReleaseMemory::Inactive))
                .take(count)
                .collect()
        },
    )
}
pub fn effect_vector(row: &Table) {
    keys(
        row,
        "id group pitch duration slide_speed slide_target release work after_pitch after_fraction after_instruction after_release prepared_remaining first_finished intro_parts loop_parts after_span occupied after_voices after_owned",
    );
    let (group, mut saved, content, occupied) = effect_fixture(row);
    let mut work = Vec::new();
    for expected in ints(row, "work") {
        let result = effects::turn(group, &saved, &content, &occupied, &BTreeSet::new());
        work.push(result.work.0.to_i64().expect("work"));
        assert_eq!(
            result.work.0.to_i64().expect("work"),
            expected,
            "{} turn {}",
            text(row, "id"),
            work.len()
        );
        check_effect_pending(group, &saved, &result, &content, &occupied);
        let serialized = Value::try_from(&result.group).expect("group serialization");
        saved = serialized.try_into().expect("group restoration");
    }
    assert_eq!(work, ints(row, "work"));
    let parts = match &saved.playback {
        Playback::Active { parts, .. } => parts,
        Playback::Empty | Playback::Prepared { .. } => {
            assert!(false, "allocation must have completed");
            return;
        }
    };
    for (identity, live) in parts
        .iter()
        .enumerate()
        .filter_map(|(identity, part)| match part {
            PartState::Live(live) => Some((identity, live)),
            PartState::Finished => None,
        })
    {
        for (key, actual) in [
            ("after_pitch", u64::from(live.pitch.get())),
            ("after_fraction", u64::from(live.fraction)),
            ("after_instruction", u64::from(live.instruction)),
            (
                "after_release",
                u64::from(saved.history[identity].remaining()),
            ),
            (
                "after_span",
                u64::try_from(live.cursor.saturating_sub(1)).expect("span"),
            ),
        ] {
            if row.contains_key(key) {
                assert_eq!(actual, unsigned(row, key), "{} {key}", text(row, "id"));
            }
        }
    }
    if row.contains_key("after_voices") {
        let voices: Vec<_> = parts
            .iter()
            .filter_map(|part| match part {
                PartState::Live(live) => Some(live.voice),
                PartState::Finished => None,
            })
            .collect();
        assert_eq!(
            voices,
            ints(row, "after_voices")
                .into_iter()
                .map(voice)
                .collect::<Vec<_>>()
        );
    }
    if row.contains_key("after_owned") {
        assert_eq!(
            u64::try_from(effects::owned(&saved.playback).len()).expect("ownership"),
            unsigned(row, "after_owned")
        );
    }
}
pub fn staged_vector(row: &Table) {
    keys(row, "id handled staged protected decision after_handled");
    let protected = flag(row, "protected");
    let cue = if protected {
        CueId::Door
    } else {
        CueId::Footstep
    };
    let saved = GroupState {
        history: vec![ReleaseMemory::Inactive; 2],
        handled: cue_name(text(row, "handled")),
        staged: cue_name(text(row, "staged")),
        reply: Request::Absent,
        playback: Playback::Active {
            cue,
            parts: vec![PartState::Live(LivePart {
                voice: VoiceId::V7,
                cursor: 1,
                instruction: 10,
                pitch: ep(24),
                fraction: 0,
                motion: None,
            })],
        },
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
    let content = Content {
        effects: BTreeMap::from([(CueId::Footstep, clip.clone()), (CueId::Door, clip)]),
        tracks: Vec::new(),
    };
    let result = effects::turn(
        SoundGroup::Contact,
        &saved,
        &content,
        &BTreeSet::new(),
        &BTreeSet::new(),
    );
    assert_eq!(result.group.handled, cue_name(text(row, "after_handled")));
    assert_eq!(
        text(row, "decision"),
        if matches!(result.group.playback, Playback::Prepared { .. }) {
            "Prepare"
        } else {
            "Continue"
        }
    );
}
