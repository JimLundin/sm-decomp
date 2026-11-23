//! Spec: StagedRequests, EffectResources and ConnectedPitchMotion. One group's effect turn up to
//! its publication: it handles the staged request, then plays its effect parts.
use std::collections::BTreeSet;
use std::num::NonZeroU8;
use types::audio::{
    Content, CueId, EffectPartContent, EffectSpan, EffectVoiceId, GroupState, LivePart, PartState,
    Playback, ReleaseMemory, Request, VoiceId, count_down,
};
/// Spec: EffectResources. The six-voice effect pool in the order allocation searches it.
const POOL: [EffectVoiceId; 6] = [
    EffectVoiceId::V7,
    EffectVoiceId::V6,
    EffectVoiceId::V5,
    EffectVoiceId::V4,
    EffectVoiceId::V3,
    EffectVoiceId::V2,
];
/// Spec: StagedRequests and EffectResources. Handle the group's staged request, then play.
///
/// A changed cue replaces the group's playback with the prepared cue, which acquires voices on a
/// later turn; absence, an unchanged request and a cue dropped by door protection all let the
/// current sound play on. `occupied` holds the voices other owners hold; `echo_voices` is the
/// song's echo membership, from which allocation removes the voices it acquires.
pub fn turn(
    group: &mut GroupState,
    content: &Content,
    occupied: &BTreeSet<EffectVoiceId>,
    echo_voices: &mut BTreeSet<VoiceId>,
) {
    if group.staged != group.handled {
        group.handled = group.staged;
        if let Request::Cue(cue) = group.staged
            && !protected(&group.playback)
        {
            // Replacing the playback releases the voices the old one owned.
            group.playback = Playback::Prepared { cue };
            return;
        }
    }
    play(
        &mut group.playback,
        &mut group.history,
        content,
        occupied,
        echo_voices,
    );
}
/// Spec: StagedRequests. Door playback is protected against another cue in its group until its
/// last part finishes.
fn protected(playback: &Playback) -> bool {
    match playback {
        Playback::Prepared { cue } => *cue == CueId::Door,
        Playback::Active { cue, parts } => {
            *cue == CueId::Door && parts.iter().any(|part| matches!(part, PartState::Live(_)))
        }
        Playback::Empty => false,
    }
}
/// Spec: EffectResources. A prepared clip acquires its voices; then every part of the playing clip
/// advances once, reading and updating its own release-history entry by part order.
fn play(
    playback: &mut Playback,
    history: &mut [ReleaseMemory],
    content: &Content,
    occupied: &BTreeSet<EffectVoiceId>,
    echo_voices: &mut BTreeSet<VoiceId>,
) {
    if let Playback::Prepared { cue } = *playback
        && let Some(clip) = content.effects.get(&cue)
    {
        let parts = allocate(clip.parts.len(), occupied, echo_voices);
        *playback = Playback::Active { cue, parts };
    }
    let Playback::Active { cue, parts } = playback else {
        return;
    };
    let Some(clip) = content.effects.get(cue) else {
        return;
    };
    for ((part, graph), release) in parts.iter_mut().zip(&clip.parts).zip(history) {
        advance_part(part, release, graph);
    }
}
/// Spec: EffectResources. Give a clip's ordered parts the highest free voices and take exactly
/// those voices out of echo membership. Each part starts before its first span, with a countdown
/// that expires on the turn it is allocated.
fn allocate(
    count: usize,
    occupied: &BTreeSet<EffectVoiceId>,
    echo_voices: &mut BTreeSet<VoiceId>,
) -> Vec<PartState> {
    let voices: Vec<_> = POOL
        .into_iter()
        .filter(|voice| !occupied.contains(voice))
        .take(count)
        .collect();
    for voice in &voices {
        echo_voices.remove(&VoiceId::Effect(*voice));
    }
    voices
        .into_iter()
        .map(|voice| {
            PartState::Live(LivePart {
                voice,
                cursor: 0,
                instruction: 1,
                pitch: crate::EffectPitch::PRENOTE,
                fraction: 0,
                motion: None,
            })
        })
        .collect()
}
/// Spec: EffectResources. One turn of one part: count its instruction down with eight-bit
/// wrapping, and when the countdown expires, release, start its next span or finish.
fn advance_part(part: &mut PartState, release: &mut ReleaseMemory, graph: &EffectPartContent) {
    let PartState::Live(live) = part else {
        return;
    };
    // A retained zero countdown wraps to 255 here.
    live.instruction = live.instruction.wrapping_sub(1);
    if live.instruction != 0 {
        motion(live);
        return;
    }
    // With an active connected slide the release lasts no turn: the next span starts at once.
    if live.motion.is_none() && hold_release(live, release) {
        return;
    }
    *release = ReleaseMemory::Inactive;
    match next_span(live, graph) {
        Some((index, span)) => start_span(live, index, span),
        // Finishing releases the part's voice: a finished part owns none.
        None => *part = PartState::Finished,
    }
}
/// Spec: EffectResources. An expired countdown with no active slide: begin a release delay, or
/// count an existing one down. Whether the part still waits on its release this turn.
fn hold_release(live: &mut LivePart, release: &mut ReleaseMemory) -> bool {
    match *release {
        ReleaseMemory::Inactive => {
            *release = ReleaseMemory::Active(NonZeroU8::MIN);
            live.instruction = 1;
            true
        }
        // The countdown stays at zero, so it wraps before the next release observation.
        ReleaseMemory::Active(count) => count_down(count).is_some_and(|left| {
            *release = ReleaseMemory::Active(left);
            true
        }),
    }
}
/// Spec: EffectResources. The part's next span and its position: the one after the last started,
/// or the loop span once a repeated sequence reaches its end; none once a finite clip has ended.
fn next_span<'a>(live: &LivePart, graph: &'a EffectPartContent) -> Option<(usize, &'a EffectSpan)> {
    let index = if live.cursor < graph.spans.len() {
        live.cursor
    } else {
        graph.loop_start?
    };
    graph.spans.get(index).map(|span| (index, span))
}
/// Spec: EffectResources. Start a span: its duration, its pitch with the fraction reset, and its
/// connected pitch motion, which advances once in the same turn.
fn start_span(live: &mut LivePart, index: usize, span: &EffectSpan) {
    live.cursor = index.saturating_add(1);
    live.instruction = span.duration.0.get();
    live.pitch = span.pitch;
    live.fraction = 0;
    live.motion.clone_from(&span.motion);
    motion(live);
}
/// Spec: ConnectedPitchMotion. An active slide adds its speed to the fraction, carries whole
/// semitones into the pitch, and ends on reaching its target, keeping its final fraction.
fn motion(live: &mut LivePart) {
    let Some(slide) = &live.motion else {
        return;
    };
    let sum = u16::from(live.fraction).saturating_add(u16::from(slide.speed.get()));
    // The low byte is `sum mod 256`, the high byte `floor(sum / 256)` semitones.
    let [fraction, semitones] = sum.to_le_bytes();
    live.fraction = fraction;
    live.pitch = live.pitch.carry(semitones);
    if live.pitch.get() >= slide.target.get() {
        live.motion = None;
    }
}
