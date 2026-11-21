//! Validate the relations between an admitted resident bank and saved transport state.

use super::Fault;
use audio_transport::{
    AudioPhase, Content, MusicEvent, Request, SongState, State, TrackContent, TrackPosition,
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
            delay,
            feedback,
            filter,
        } => *delay == echo_delay && *feedback <= 127 && *filter == 0,
        MusicEvent::Timing { extra, .. } => extra.0 == 0_u64.into() || extra.0 == 3_u64.into(),
        MusicEvent::Sustain
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
        if phrase != 0
            && !events
                .iter()
                .any(|event| matches!(event, MusicEvent::Tone(_) | MusicEvent::Sustain))
        {
            return false;
        }
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
        TrackPosition::Repeated { phrase, cursor, continuation, remaining } => {
            *phrase > 0 && graph.phrases.get(*phrase).and_then(|p| p.get(*cursor)).is_some()
                && continuation.checked_sub(1).and_then(|previous| graph.phrases.first()
                    .and_then(|root| root.get(previous))).is_some_and(|event| {
                        matches!(event, MusicEvent::Repeat { phrase: target, count } if target == phrase && remaining <= count)
                    })
                && graph.phrases.first().and_then(|p| p.get(*continuation)).is_some()
        }
    }
}

fn validate_effect_content(content: &Content, timbres: usize) -> Result<(), Fault> {
    for (cue, clip) in &content.effects {
        if clip.parts.is_empty()
            || clip.parts.len() > part_count(audio_transport::effects::group(*cue))
            || clip.parts.iter().any(|part| {
                part.spans.is_empty()
                    || part
                        .loop_start
                        .is_some_and(|start| start >= part.spans.len())
                    || part.spans.iter().any(|span| {
                        span.pitch.get() > 71
                            || usize::from(span.timbre.0) >= timbres
                            || span.motion.as_ref().is_some_and(|motion| {
                                motion.target.get() > 71 || motion.target.get() <= span.pitch.get()
                            })
                    })
            })
        {
            return Err(Fault::Invalid("effect content or voice resources"));
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
) -> Result<(), Fault> {
    match &saved.playback {
        audio_transport::Playback::Empty => {}
        audio_transport::Playback::Prepared { cue } => {
            if audio_transport::effects::group(*cue) != group || !content.effects.contains_key(cue)
            {
                return Err(Fault::Invalid("continuation prepared cue relation"));
            }
        }
        audio_transport::Playback::Active { cue, parts } => {
            let clip = content
                .effects
                .get(cue)
                .ok_or(Fault::Invalid("continuation live cue content"))?;
            if audio_transport::effects::group(*cue) != group
                || parts.len() != clip.parts.len()
                || parts.len() > saved.history.len()
            {
                return Err(Fault::Invalid("continuation live part shape"));
            }
            let mut previous = None;
            for (state, part) in parts.iter().zip(&clip.parts) {
                match state {
                    audio_transport::PartState::Finished => {}
                    audio_transport::PartState::Live(live) => {
                        // Allocations follow the pool's order. Finished slots still divide
                        // the same ordered clip, but release their voice identity.
                        if previous.is_some_and(|voice| voice >= live.voice) {
                            return Err(Fault::Invalid("effect voice allocation order"));
                        }
                        previous = Some(live.voice);
                        if live.cursor > part.spans.len()
                            || live.pitch.get() > 71
                            || live.motion.as_ref().is_some_and(|motion| {
                                motion.target.get() > 71 || live.pitch.get() >= motion.target.get()
                            })
                            || !voices.insert(live.voice)
                            || seed.reservations.contains(&live.voice)
                        {
                            return Err(Fault::Invalid(
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

/// The resident bank admits the whole cue roster and one checked resource per timbre.
pub(super) fn validate_bank(bank: &data::assets::audio::ResidentAudio) -> Result<(), Fault> {
    const CUES: usize = 8;
    if bank.echo_delay > 15
        || bank.content.effects.len() != CUES
        || bank.aliases.len() != CUES
        || bank.instruments.is_empty()
        || bank
            .instruments
            .iter()
            .any(|instrument| bank.samples.get(usize::from(instrument.sample.0)).is_none())
        || bank.samples.iter().any(|sample| {
            sample.frames == 0
                || !sample.frames.is_multiple_of(16)
                || sample
                    .loop_from
                    .is_some_and(|start| start >= sample.frames || !start.is_multiple_of(16))
        })
    {
        return Err(Fault::Invalid("resident resources"));
    }
    for (index, alias) in bank.aliases.iter().enumerate() {
        if alias.byte == 0
            || audio_transport::effects::group(alias.cue) != alias.group
            || !bank.content.effects.contains_key(&alias.cue)
            || bank.aliases.iter().take(index).any(|prior| {
                prior.cue == alias.cue || (prior.group == alias.group && prior.byte == alias.byte)
            })
        {
            return Err(Fault::Invalid("resident cue aliases"));
        }
    }
    validate_graph(&bank.content, bank.instruments.len(), bank.echo_delay)
}

pub(super) fn validate_graph(
    content: &Content,
    timbres: usize,
    echo_delay: u8,
) -> Result<(), Fault> {
    if content.tracks.len() != 2
        || timbres == 0
        || !content
            .tracks
            .iter()
            .all(|track| valid_track(track, timbres, echo_delay))
    {
        return Err(Fault::Invalid("resident song graph"));
    }
    validate_effect_content(content, timbres)
}

pub(super) fn validate_resident_state(
    seed: &State,
    bank: &data::assets::audio::ResidentAudio,
) -> Result<(), Fault> {
    use audio_transport::{CompletionResult, Continuation, ReplyLine, SoundGroup, time};

    if seed.resident_music_reply != bank.music_reply {
        return Err(Fault::ContentIdentity);
    }
    validate_song(
        &seed.song,
        &bank.content,
        bank.instruments.len(),
        bank.echo_delay,
    )?;
    if seed.clock.unread > 15 || seed.cadence.effect > 7 || !seed.reservations.is_empty() {
        return Err(Fault::Invalid("retained timer count or cadence"));
    }
    validate_groups(seed, &bank.content, None)?;
    let deadline = match &seed.phase {
        AudioPhase::ObserveClock { at } => {
            if at < &seed.clock.lattice
                || !std::ops::Div::div(
                    at.difference(&seed.clock.lattice).rational(),
                    num_bigint::BigInt::from(8),
                )
                .is_integer()
                || (seed.clock.lattice > seed.now && at != &seed.clock.lattice)
                || (seed.clock.lattice <= seed.now && at.difference(&seed.now) > time::cycles(8))
            {
                return Err(Fault::Invalid("polling lattice"));
            }
            at.clone()
        }
        AudioPhase::Distribute { at, batch } => {
            if !(1..16).contains(batch) {
                return Err(Fault::Invalid("pending timer batch"));
            }
            at.clone()
        }
        AudioPhase::Work { at, result } => {
            match result {
                CompletionResult::Music(song) => {
                    validate_song(song, &bank.content, bank.instruments.len(), bank.echo_delay)?;
                }
                CompletionResult::Effect { group, result, .. } => {
                    validate_groups(seed, &bank.content, Some((*group, result)))?;
                }
            }
            at.clone()
        }
        AudioPhase::Sample { sample, next } => {
            let expected = match sample.line {
                ReplyLine::Group(SoundGroup::Action) => Continuation::Effect(SoundGroup::Ambient),
                ReplyLine::Group(SoundGroup::Ambient) => Continuation::Effect(SoundGroup::Contact),
                ReplyLine::Group(SoundGroup::Contact) => Continuation::AfterEffects,
                ReplyLine::Music => Continuation::AfterMusic,
            };
            let first = sample.anchor.sum(&time::halves(9));
            if next != &expected
                || sample.first.is_some() != (first <= seed.now)
                || sample
                    .first
                    .is_some_and(|request| match (sample.line, request) {
                        (_, Request::Absent) => false,
                        (ReplyLine::Music, Request::Cue(_)) => true,
                        (ReplyLine::Group(group), Request::Cue(cue)) => {
                            audio_transport::effects::group(cue) != group
                        }
                    })
            {
                return Err(Fault::Invalid("pending sample"));
            }
            if sample.first.is_some() {
                sample.anchor.sum(&time::halves(19))
            } else {
                first
            }
        }
        AudioPhase::MusicStart { at } | AudioPhase::Idle { at } => at.clone(),
    };
    if deadline < seed.now || (deadline == seed.now && seed.now != time::cycles(0)) {
        return Err(Fault::Invalid("past transport deadline"));
    }
    Ok(())
}

pub(super) fn validate_song(
    song: &SongState,
    content: &Content,
    timbres: usize,
    echo_delay: u8,
) -> Result<(), Fault> {
    if song.tracks.len() != content.tracks.len()
        || song.mix.echo_delay != echo_delay
        || song.mix.feedback > 127
        || song.mix.filter != 0
        || song.mix.echo_left > 127 * 256
        || song.mix.echo_right > 127 * 256
        || song
            .tracks
            .iter()
            .zip(&content.tracks)
            .any(|(track, graph)| {
                !valid_pitch(track.pitch.get())
                    || usize::from(track.timbre.0) >= timbres
                    || !valid_position(&track.position, graph)
            })
    {
        return Err(Fault::Invalid("saved track or mix"));
    }
    Ok(())
}

pub(super) fn validate_groups(
    seed: &State,
    content: &Content,
    pending: Option<(audio_transport::SoundGroup, &audio_transport::GroupState)>,
) -> Result<(), Fault> {
    let mut voices = std::collections::BTreeSet::new();
    for (group, committed) in &seed.groups {
        let saved = match pending {
            Some((selected, result)) if selected == group => result,
            Some(_) | None => committed,
        };
        validate_group(content, group, saved, seed.requests[group])?;
        validate_playback(seed, content, group, saved, &mut voices)?;
    }
    Ok(())
}

const fn part_count(group: audio_transport::SoundGroup) -> usize {
    match group {
        audio_transport::SoundGroup::Action | audio_transport::SoundGroup::Ambient => 1,
        audio_transport::SoundGroup::Contact => 2,
    }
}

fn validate_group(
    content: &Content,
    group: audio_transport::SoundGroup,
    saved: &audio_transport::GroupState,
    request: Request,
) -> Result<(), Fault> {
    if saved.history.len() != part_count(group) {
        return Err(Fault::Invalid("complete ordered effect history"));
    }
    for request in [saved.handled, saved.staged, saved.reply, request] {
        if let Request::Cue(cue) = request
            && (audio_transport::effects::group(cue) != group
                || !content.effects.contains_key(&cue))
        {
            return Err(Fault::Invalid("request family"));
        }
    }
    Ok(())
}

pub(super) fn valid_sender_phase(state: &audio_transport::SenderState) -> bool {
    state.pending.as_ref().is_none_or(|pending| {
        pending.origin.0 <= state.now.0
            && match (&pending.observation, &pending.result) {
                (Some(observation), None) => {
                    observation.0 > state.now.0
                        && matches!(
                            state.progress,
                            audio_transport::SenderProgress::AwaitCue(_)
                                | audio_transport::SenderProgress::AwaitClear
                        )
                        && fast_bus_observation(state, &pending.origin).as_ref()
                            == Some(observation)
                }
                (None, Some(result)) => {
                    result.completion.0 > state.now.0
                        && result
                            .presentation
                            .as_ref()
                            .is_none_or(|(at, _)| at.0 > state.now.0 && at.0 < result.completion.0)
                }
                (Some(_), Some(_)) | (None, None) => false,
            }
    })
}

fn fast_bus_observation(
    state: &audio_transport::SenderState,
    origin: &audio_transport::CpuMasterTicks,
) -> Option<audio_transport::CpuMasterTicks> {
    // The waiting phases schedule their observation without using the current reply.
    // Reading the schedule from a separate value leaves the retained service untouched.
    let mut schedule = state.clone();
    schedule.now.clone_from(origin);
    schedule.pending = None;
    audio_transport::sender::begin(
        &mut schedule,
        audio_transport::sender::SenderProfile::FastBus,
    );
    schedule.pending.and_then(|service| service.observation)
}
