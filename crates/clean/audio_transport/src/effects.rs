//! Spec: EffectResources, ConnectedPitchMotion and EffectTiming.
use crate::ReleaseMemory;
use crate::{
    AudioCycles, Content, CueId, EffectPartContent, EffectVoiceId, GroupState, LivePart, PartState,
    Playback, Request, SoundGroup,
};
use std::collections::BTreeSet;
use std::num::NonZeroU8;
const FRACTIONS: u16 = 256;
const OCTAVE: u8 = 12;
const OCTAVE_WORK: u64 = 14;
const SECOND: u64 = 6;
const AMBIENT: u64 = 12;
const SILENCE: u64 = 8;
const PART: u64 = 300;
const VISIT: u64 = 16;
const ALIVE: u64 = 19;
const RELEASE_START: u64 = 64;
const RELEASE_CONTINUE: u64 = 34;
const FRACTIONAL: u64 = 464;
const TERMINAL: u64 = 489;
const NONTERMINAL: u64 = 479;
const POOL: [EffectVoiceId; 6] = [
    EffectVoiceId::V7,
    EffectVoiceId::V6,
    EffectVoiceId::V5,
    EffectVoiceId::V4,
    EffectVoiceId::V3,
    EffectVoiceId::V2,
];
/// Spec: EffectTiming. Group work profile.
#[derive(Clone, Copy)]
struct Profile {
    base: u64,
    prepare: u64,
    replace: u64,
    play: u64,
    allocate: u64,
}
const fn profile(group: SoundGroup) -> Profile {
    match group {
        SoundGroup::Action => Profile {
            base: 60,
            prepare: 188,
            replace: 1433,
            play: 59,
            allocate: 494,
        },
        SoundGroup::Ambient => Profile {
            base: 86,
            prepare: 170,
            replace: 719,
            play: 37,
            allocate: 278,
        },
        SoundGroup::Contact => Profile {
            base: 88,
            prepare: 197,
            replace: 719,
            play: 37,
            allocate: 278,
        },
    }
}
/// Spec: StagedRequests. A cue's family is fixed.
#[must_use]
pub const fn group(cue: CueId) -> SoundGroup {
    match cue {
        CueId::Spin | CueId::EndSpin | CueId::Beam | CueId::SamusSilence => SoundGroup::Action,
        CueId::BackgroundSilence => SoundGroup::Ambient,
        CueId::Footstep | CueId::Land | CueId::Door => SoundGroup::Contact,
    }
}
/// Spec: EffectResources. Live ownership only.
#[must_use]
pub fn owned(playback: &Playback) -> BTreeSet<EffectVoiceId> {
    let Playback::Active { parts, .. } = playback else {
        return BTreeSet::new();
    };
    parts
        .iter()
        .filter_map(|part| match part {
            PartState::Live(live) => Some(live.voice),
            PartState::Finished => None,
        })
        .collect()
}
/// Spec: EffectTiming. Completed group and exact work, for deferred publication.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EffectResult {
    /// Completed group.
    pub group: GroupState,
    /// Audio instruction work.
    pub work: AudioCycles,
    /// Spec: EffectResources. Complete echo membership, published with the completed group.
    pub echo_voices: BTreeSet<crate::VoiceId>,
}
/// Spec: StagedRequests, EffectResources. Changed requests prepare; absence does not stop sound.
///
/// The caller supplies complete group history and current semantic echo membership independently
/// of ownership. The result is pending data: do not commit its history or membership before work
/// completes. Allocated voices leave membership; releasing voices never restores it.
#[must_use]
pub fn turn(
    group: SoundGroup,
    state: &GroupState,
    content: &Content,
    occupied: &BTreeSet<EffectVoiceId>,
    echo_voices: &BTreeSet<crate::VoiceId>,
) -> EffectResult {
    let mut result = state.clone();
    let rules = profile(group);
    let mut work = rules.base;
    if state.staged != state.handled {
        result.handled = state.staged;
        if let Request::Cue(cue) = state.staged
            && !protected(&state.playback)
        {
            if !owned(&state.playback).is_empty() {
                work = work.saturating_add(rules.replace);
            }
            work =
                work.saturating_add(rules.prepare.saturating_sub(if cue == CueId::SamusSilence {
                    SILENCE
                } else {
                    0
                }));
            result.playback = Playback::Prepared { cue };
            result.reply = state.staged;
            return EffectResult {
                group: result,
                work: AudioCycles(work),
                echo_voices: echo_voices.clone(),
            };
        }
        let handling = match state.staged {
            Request::Absent => 8,
            Request::Cue(_) => 36,
        };
        work = work.saturating_add(handling);
    }
    let mut echo_voices = echo_voices.clone();
    let (playback, playback_work) = play(
        group,
        &result.playback,
        &mut result.history,
        content,
        occupied,
        &mut echo_voices,
    );
    result.playback = playback;
    result.reply = state.staged;
    EffectResult {
        group: result,
        work: AudioCycles(work.saturating_add(playback_work)),
        echo_voices,
    }
}
fn protected(playback: &Playback) -> bool {
    matches!(
        playback,
        Playback::Prepared {
            cue: CueId::Door,
            ..
        } | Playback::Active {
            cue: CueId::Door,
            ..
        }
    ) && !matches!(playback, Playback::Active { parts, .. } if parts.iter().all(|part| matches!(part, PartState::Finished)))
}
fn play(
    group: SoundGroup,
    playback: &Playback,
    history: &mut [ReleaseMemory],
    content: &Content,
    occupied: &BTreeSet<EffectVoiceId>,
    echo_voices: &mut BTreeSet<crate::VoiceId>,
) -> (Playback, u64) {
    let (cue, mut parts, allocation) = match playback {
        Playback::Empty => return (Playback::Empty, 0),
        Playback::Prepared { cue } => {
            let Some(clip) = content.effects.get(cue) else {
                return (playback.clone(), 0);
            };
            let (parts, visits) = allocate(clip.parts.len(), occupied, echo_voices);
            let cost = profile(group)
                .allocate
                .saturating_add(PART.saturating_mul(u64::try_from(parts.len()).unwrap_or(0)))
                .saturating_add(VISIT.saturating_mul(visits));
            (*cue, parts, cost)
        }
        Playback::Active { cue, parts } => (*cue, parts.clone(), 0),
    };
    let alive = parts
        .iter()
        .filter(|part| matches!(part, PartState::Live(_)))
        .count();
    if alive == 0 {
        return (Playback::Active { cue, parts }, 0);
    }
    let mut work = allocation
        .saturating_add(profile(group).play)
        .saturating_add(ALIVE.saturating_mul(u64::try_from(alive).unwrap_or(0)));
    let last = parts
        .iter()
        .rposition(|part| matches!(part, PartState::Live(_)));
    let Some(clip) = content.effects.get(&cue) else {
        return (playback.clone(), work);
    };
    for (identity, ((part, graph), release)) in
        parts.iter_mut().zip(&clip.parts).zip(history).enumerate()
    {
        work = work.saturating_add(advance_part(
            part,
            release,
            graph,
            group,
            identity,
            Some(identity) == last,
        ));
    }
    (Playback::Active { cue, parts }, work)
}
fn allocate(
    count: usize,
    occupied: &BTreeSet<EffectVoiceId>,
    echo_voices: &mut BTreeSet<crate::VoiceId>,
) -> (Vec<PartState>, u64) {
    let mut parts = Vec::new();
    let mut visits: u64 = 0;
    for voice in POOL {
        if occupied.contains(&voice) {
            visits = visits.saturating_add(1);
            continue;
        }
        if parts.len() == count {
            break;
        }
        echo_voices.remove(&crate::VoiceId::Effect(voice));
        parts.push(PartState::Live(LivePart {
            voice,
            cursor: 0,
            instruction: 1,
            pitch: crate::EffectPitch::PRENOTE,
            fraction: 0,
            motion: None,
        }));
    }
    (parts, visits)
}
fn advance_part(
    part: &mut PartState,
    release: &mut ReleaseMemory,
    graph: &EffectPartContent,
    group: SoundGroup,
    identity: usize,
    last: bool,
) -> u64 {
    let PartState::Live(live) = part else {
        return 0;
    };
    live.instruction = live.instruction.wrapping_sub(1);
    if live.instruction != 0 {
        let slide_work = motion(live);
        return slide_work.saturating_add(if group == SoundGroup::Ambient {
            AMBIENT
        } else {
            0
        });
    }
    let connected = live.motion.is_some();
    if !connected {
        match *release {
            ReleaseMemory::Inactive => {
                *release = ReleaseMemory::Active(NonZeroU8::MIN);
                live.instruction = 1;
                return RELEASE_START;
            }
            ReleaseMemory::Active(n) if n.get() > 1 => {
                *release = ReleaseMemory::Active(
                    NonZeroU8::new(n.get().saturating_sub(1)).unwrap_or(NonZeroU8::MIN),
                );
                return RELEASE_CONTINUE;
            }
            ReleaseMemory::Active(_) => {}
        }
    }
    *release = ReleaseMemory::Inactive;
    let Some(span) = next_span(live, graph) else {
        const FINISH: u64 = 463;
        const LAST: u64 = 15;
        const CONNECTED: u64 = 66;
        *part = PartState::Finished;
        return FINISH
            .saturating_add(if last { LAST } else { 0 })
            .saturating_sub(if connected { CONNECTED } else { 0 });
    };
    start_span(live, span, graph, group, identity != 0)
}
fn next_span<'a>(live: &LivePart, graph: &'a EffectPartContent) -> Option<&'a crate::EffectSpan> {
    let cursor = if live.cursor >= graph.spans.len() {
        graph.loop_start?
    } else {
        live.cursor
    };
    graph.spans.get(cursor)
}
fn start_span(
    live: &mut LivePart,
    span: &crate::EffectSpan,
    graph: &EffectPartContent,
    group: SoundGroup,
    second: bool,
) -> u64 {
    const START: u64 = 1298;
    const CONNECTION: u64 = 102;
    const ENTRY: u64 = 94;
    const RETURN: u64 = 44;
    let returning = live.cursor >= graph.spans.len();
    if returning {
        live.cursor = graph.loop_start.unwrap_or(0);
    }
    let mut work = START
        .saturating_sub(
            OCTAVE_WORK
                .saturating_mul(u64::from(span.pitch.get().checked_div(OCTAVE).unwrap_or(0))),
        )
        .saturating_sub(if second { SECOND } else { 0 });
    if graph.loop_start == Some(live.cursor) {
        work = work.saturating_add(if returning { RETURN } else { ENTRY });
    }
    live.instruction = span.duration.0.get();
    live.pitch = span.pitch;
    live.fraction = 0;
    live.motion.clone_from(&span.motion);
    live.cursor = live.cursor.saturating_add(1);
    if span.motion.is_some() {
        work = work.saturating_add(CONNECTION).saturating_add(motion(live));
    }
    if group == SoundGroup::Ambient {
        work = work.saturating_add(AMBIENT);
    }
    work
}
/// Spec: ConnectedPitchMotion. Fractional carry and terminal carry charge work once.
pub fn motion(live: &mut LivePart) -> u64 {
    let Some(slide) = &live.motion else {
        return 0;
    };
    let sum = u16::from(live.fraction).saturating_add(u16::from(slide.speed.get()));
    live.fraction = u8::try_from(sum.checked_rem(FRACTIONS).unwrap_or(0)).unwrap_or(0);
    let carry = sum.checked_div(FRACTIONS).unwrap_or(0);
    live.pitch = live.pitch.carry(u8::try_from(carry).unwrap_or(0));
    let terminal = live.pitch.get() >= slide.target.get();
    let base = if carry == 0 {
        FRACTIONAL
    } else if terminal {
        TERMINAL
    } else {
        NONTERMINAL
    };
    if terminal {
        live.motion = None;
    }
    base.saturating_sub(
        OCTAVE_WORK.saturating_mul(u64::from(live.pitch.get().checked_div(OCTAVE).unwrap_or(0))),
    )
}
