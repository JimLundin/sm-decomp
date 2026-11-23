//! Spec: FrameCadence and PublishedReplies. Successive frames of one sound group.
use super::*;
/// Progress written as `Ready`, `AwaitCue:<cue>`, `ClearDelay:<turns>`, `AwaitClear` or
/// `ResetDelay:<turns>`.
fn progress_named(name: &str) -> SenderProgress {
    let turns = |value: &str| nz(value.parse().expect("delay turns"));
    match name.split_once(':') {
        None if name == "Ready" => SenderProgress::Ready,
        None => {
            assert_eq!(name, "AwaitClear");
            SenderProgress::AwaitClear
        }
        Some(("AwaitCue", cue)) => SenderProgress::AwaitCue(named(cue)),
        Some(("ClearDelay", value)) => SenderProgress::ClearDelay(turns(value)),
        Some((kind, value)) => {
            assert_eq!(kind, "ResetDelay");
            SenderProgress::ResetDelay(turns(value))
        }
    }
}
fn queue_named(row: &Table) -> CueQueue {
    queue_of(
        array(row, "queue")
            .iter()
            .map(|v| named(v.as_str().expect("cue name"))),
    )
}
fn turns(row: &Table, key: &str) -> TurnCount {
    TurnCount::new(row.get(key).map_or(0, |_| byte(row, key))).expect("at most two turns")
}
fn frame_cadence(row: &Table) -> FrameCadence {
    let effects = if flag(row, "service") {
        EffectCadence::AroundService {
            before: turns(row, "before"),
            after: turns(row, "after"),
        }
    } else {
        assert_eq!(
            integer(row, "before"),
            0,
            "no turn falls before a missing service"
        );
        EffectCadence::WithoutService(turns(row, "after"))
    };
    FrameCadence {
        effects,
        music: turns(row, "music"),
    }
}
/// Each cue's clip: one invented part of one 20-turn span at pitch 24, timbre 0.
fn every_cue_clip() -> BTreeMap<CueId, EffectClip> {
    let clip = EffectClip {
        parts: vec![EffectPartContent {
            spans: vec![EffectSpan {
                duration: EffectTurns(nz(20)),
                pitch: ep(24),
                timbre: TimbreId(0),
                motion: None,
            }],
            loop_start: None,
        }],
    };
    [
        CueId::Footstep,
        CueId::Spin,
        CueId::EndSpin,
        CueId::Land,
        CueId::Beam,
        CueId::SamusSilence,
        CueId::BackgroundSilence,
        CueId::Door,
    ]
    .into_iter()
    .map(|cue| (cue, clip.clone()))
    .collect()
}
fn frame_state(row: &Table, group: SoundGroup, mix: &Value) -> (State, Vec<TrackContent>) {
    let (song, tracks) = if row.contains_key("tracks") {
        song(row, mix)
    } else {
        (silent_song(mix), Vec::new())
    };
    let mut state = fresh_state(song);
    state.senders[group] = SenderState {
        queue: queue_named(row),
        progress: progress_named(text(row, "progress")),
    };
    state.request_lines[group] = request_named(text(row, "request"));
    let audio = &mut state.groups[group];
    audio.staged = request_named(text(row, "staged"));
    audio.handled = request_named(text(row, "handled"));
    audio.reply = request_named(text(row, "reply"));
    if row.contains_key("resident_reply_alias") {
        state.resident_music_reply.original_byte = byte(row, "resident_reply_alias");
        state.music_reply.original_byte = byte(row, "music_reply");
    }
    (state, tracks)
}
fn check_frame_cut(expect: &Table, state: &State, group: SoundGroup, label: &str) {
    keys(
        expect,
        "progress queue request staged reply notes music_reply",
    );
    assert_eq!(
        state.senders[group].progress,
        progress_named(text(expect, "progress")),
        "{label} progress"
    );
    assert_eq!(
        state.senders[group].queue,
        queue_named(expect),
        "{label} queue"
    );
    assert_eq!(
        state.request_lines[group],
        request_named(text(expect, "request")),
        "{label} request"
    );
    let audio = &state.groups[group];
    assert_eq!(
        audio.staged,
        request_named(text(expect, "staged")),
        "{label} staged"
    );
    assert_eq!(
        audio.reply,
        request_named(text(expect, "reply")),
        "{label} reply"
    );
    if expect.contains_key("notes") {
        let notes: Vec<_> = state
            .song
            .tracks
            .iter()
            .map(|t| i64::from(t.note.0.get()))
            .collect();
        assert_eq!(notes, ints(expect, "notes"), "{label} notes");
    }
    if expect.contains_key("music_reply") {
        assert_eq!(
            state.music_reply.original_byte,
            byte(expect, "music_reply"),
            "{label} music reply"
        );
    }
}
/// Successive frames from the declared group state; the state at each frame cut as expected.
pub fn frame_vector(row: &Table, mix: &Value) {
    keys(
        row,
        "id source group progress queue request staged handled reply frames expect tracks mix resident_reply_alias music_reply",
    );
    let group: SoundGroup = named(text(row, "group"));
    let (mut state, tracks) = frame_state(row, group, mix);
    let content = content(every_cue_clip(), tracks);
    let frames = array(row, "frames");
    let expected = array(row, "expect");
    assert_eq!(frames.len(), expected.len());
    for (number, (frame, expect)) in frames.iter().zip(expected).enumerate() {
        let frame = table(frame);
        keys(frame, "offers before service after music");
        let offers: Vec<_> = array(frame, "offers")
            .iter()
            .map(|cue| Offer {
                cue: named(cue.as_str().expect("cue name")),
                allowance: QueueAllowance::Three,
                permission: OfferPermission::Enabled,
            })
            .collect();
        play(&mut state, &content, &offers, frame_cadence(frame));
        let label = format!("{} frame {}", text(row, "id"), number.saturating_add(1));
        check_frame_cut(table(expect), &state, group, &label);
    }
}
