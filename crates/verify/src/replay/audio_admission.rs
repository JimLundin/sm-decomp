//! Validate the independent fixture's content/state relations before the model runs.
//! Serde checks representation; private pitch wrappers and Vec lengths do not prove admission.

use super::audio::ProviderFault;
use audio_transport::{
    AudioPhase, Content, MusicEvent, Request, State, TrackContent, TrackPosition,
};

fn valid_pitch(value: u8) -> bool {
    (19..=51).contains(&value)
}

fn valid_event(event: &MusicEvent, timbres: usize, echo_delay: u8) -> bool {
    match event {
        MusicEvent::Tone(pitch) => valid_pitch(pitch.get()),
        MusicEvent::Timbre(timbre) => usize::from(timbre.0) < timbres,
        MusicEvent::EchoMix { left, right, .. } => *left <= 127 && *right <= 127,
        MusicEvent::EchoSpace {
            delay, feedback, ..
        } => *delay == echo_delay && *feedback <= 127,
        MusicEvent::Timing { .. }
        | MusicEvent::Sustain
        | MusicEvent::Loudness(_)
        | MusicEvent::Ramp { .. }
        | MusicEvent::MasterLoudness(_)
        | MusicEvent::Offset(_)
        | MusicEvent::Percussion(_)
        | MusicEvent::Repeat { .. }
        | MusicEvent::Complete => true,
    }
}

fn valid_track(content: &TrackContent, timbres: usize, echo_delay: u8) -> bool {
    let Some(root) = content.phrases.first() else {
        return false;
    };
    let first_boundary = root.iter().find(|event| {
        matches!(
            event,
            MusicEvent::Tone(_)
                | MusicEvent::Sustain
                | MusicEvent::Repeat { .. }
                | MusicEvent::Complete
        )
    });
    if !matches!(
        first_boundary,
        Some(MusicEvent::Tone(_) | MusicEvent::Sustain)
    ) {
        return false;
    }
    content.phrases.iter().enumerate().all(|(phrase, events)| {
        if !matches!(events.last(), Some(MusicEvent::Complete)) {
            return false;
        }
        events.iter().enumerate().all(|(cursor, event)| {
            if !valid_event(event, timbres, echo_delay) {
                return false;
            }
            match event {
                MusicEvent::Complete => cursor.saturating_add(1) == events.len(),
                MusicEvent::Timing { .. } => matches!(
                    events.get(cursor.saturating_add(1)),
                    Some(MusicEvent::Tone(_) | MusicEvent::Sustain)
                ),
                MusicEvent::Repeat { phrase: target, .. } => {
                    phrase == 0 && *target > 0 && *target < content.phrases.len()
                }
                MusicEvent::Tone(_)
                | MusicEvent::Sustain
                | MusicEvent::Loudness(_)
                | MusicEvent::Ramp { .. }
                | MusicEvent::MasterLoudness(_)
                | MusicEvent::Timbre(_)
                | MusicEvent::Offset(_)
                | MusicEvent::Percussion(_)
                | MusicEvent::EchoMix { .. }
                | MusicEvent::EchoSpace { .. } => true,
            }
        })
    })
}

fn valid_position(position: &TrackPosition, graph: &TrackContent) -> bool {
    match position {
        TrackPosition::Root { cursor } => graph.phrases.first().and_then(|p| p.get(*cursor)).is_some(),
        TrackPosition::Repeated { phrase, cursor, continuation, .. } => {
            *phrase > 0 && graph.phrases.get(*phrase).and_then(|p| p.get(*cursor)).is_some()
                && continuation.checked_sub(1).and_then(|previous| graph.phrases.first()
                    .and_then(|root| root.get(previous))).is_some_and(|event| {
                        matches!(event, MusicEvent::Repeat { phrase: target, .. } if target == phrase)
                    })
                && graph.phrases.first().and_then(|p| p.get(*continuation)).is_some()
        }
    }
}

/// Admit the two-track loaded native seed and every resource/graph relation it consumes.
///
/// # Errors
/// Unsupported content or malformed loaded state is a fixture defect, never a successful skip.
pub fn validate(seed: &State, content: &Content, timbres: usize) -> Result<(), ProviderFault> {
    validate_for(seed, content, timbres, Admission::Loaded)
}

pub(super) fn validate_continuation(
    seed: &State,
    content: &Content,
    timbres: usize,
) -> Result<(), ProviderFault> {
    validate_for(seed, content, timbres, Admission::NativeContinuation)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Admission {
    Loaded,
    NativeContinuation,
}

fn validate_for(
    seed: &State,
    content: &Content,
    timbres: usize,
    admission: Admission,
) -> Result<(), ProviderFault> {
    let mut voices = std::collections::BTreeSet::new();
    if seed.song.tracks.len() != 2
        || content.tracks.len() != 2
        || timbres == 0
        || !seed.reservations.is_empty()
        || !content
            .tracks
            .iter()
            .all(|track| valid_track(track, timbres, seed.song.mix.echo_delay))
    {
        return Err(ProviderFault::Admission(
            "resident graph or resource relation",
        ));
    }
    for (track, graph) in seed.song.tracks.iter().zip(&content.tracks) {
        if !valid_pitch(track.pitch.get())
            || usize::from(track.timbre.0) >= timbres
            || !valid_position(&track.position, graph)
        {
            return Err(ProviderFault::Admission("loaded track position or pitch"));
        }
    }
    validate_effect_content(content, timbres)?;
    for (group, saved) in &seed.groups {
        let histories = match group {
            audio_transport::SoundGroup::Action | audio_transport::SoundGroup::Ambient => 1,
            audio_transport::SoundGroup::Contact => 2,
        };
        if saved.history.len() != histories
            || content.effects.iter().any(|(cue, clip)| {
                audio_transport::effects::group(*cue) == group && clip.parts.len() > histories
            })
        {
            return Err(ProviderFault::Admission("complete ordered effect history"));
        }
        for request in [
            saved.handled,
            saved.staged,
            saved.reply,
            seed.requests[group],
        ] {
            if let Request::Cue(cue) = request
                && (audio_transport::effects::group(cue) != group
                    || !content.effects.contains_key(&cue))
            {
                return Err(ProviderFault::Admission("loaded request family"));
            }
        }
        if admission == Admission::Loaded
            && !matches!(saved.playback, audio_transport::Playback::Empty)
        {
            return Err(ProviderFault::Admission(
                "unsupported loaded effect playback",
            ));
        }
        if admission == Admission::NativeContinuation {
            validate_playback(seed, content, group, saved, &mut voices)?;
        }
    }
    if !matches!(&seed.phase, AudioPhase::ObserveClock { at } if at >= &seed.now) {
        return Err(ProviderFault::Admission(
            "unsupported loaded scheduler phase",
        ));
    }
    Ok(())
}

fn validate_effect_content(content: &Content, timbres: usize) -> Result<(), ProviderFault> {
    for clip in content.effects.values() {
        if clip.parts.is_empty()
            || clip.parts.len() > 6
            || clip.parts.iter().any(|part| {
                part.spans.is_empty()
                    || part
                        .loop_start
                        .is_some_and(|start| start >= part.spans.len())
                    || part.spans.iter().any(|span| {
                        span.pitch.get() > 71
                            || usize::from(span.timbre.0) >= timbres
                            || span
                                .motion
                                .as_ref()
                                .is_some_and(|motion| motion.target.get() > 71)
                    })
            })
        {
            return Err(ProviderFault::Admission(
                "effect content or voice resources",
            ));
        }
    }
    Ok(())
}

fn validate_playback(
    seed: &State,
    content: &Content,
    group: audio_transport::SoundGroup,
    saved: &audio_transport::GroupState,
    voices: &mut std::collections::BTreeSet<audio_transport::EffectVoiceId>,
) -> Result<(), ProviderFault> {
    match &saved.playback {
        audio_transport::Playback::Empty => {}
        audio_transport::Playback::Prepared { cue } => {
            if audio_transport::effects::group(*cue) != group || !content.effects.contains_key(cue)
            {
                return Err(ProviderFault::Admission(
                    "continuation prepared cue relation",
                ));
            }
        }
        audio_transport::Playback::Active { cue, parts } => {
            let clip = content
                .effects
                .get(cue)
                .ok_or(ProviderFault::Admission("continuation live cue content"))?;
            if audio_transport::effects::group(*cue) != group
                || parts.len() != clip.parts.len()
                || parts.len() > saved.history.len()
            {
                return Err(ProviderFault::Admission("continuation live part shape"));
            }
            for (state, part) in parts.iter().zip(&clip.parts) {
                match state {
                    audio_transport::PartState::Finished => {}
                    audio_transport::PartState::Live(live) => {
                        if live.cursor > part.spans.len()
                            || live.pitch.get() > 71
                            || live.motion.as_ref().is_some_and(|motion| {
                                motion.target.get() > 71 || live.pitch.get() >= motion.target.get()
                            })
                            || !voices.insert(live.voice)
                            || seed.reservations.contains(&live.voice)
                        {
                            return Err(ProviderFault::Admission(
                                "continuation live cursor pitch or voice relation",
                            ));
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
