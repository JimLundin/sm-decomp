//! Spec: IndependentCadence, StableRequests, ClockObservation and ReplyCadence.
use crate::{
    AudioCycles, AudioInstant, AudioPhase, CadencePhase, CompletionResult, Content, Continuation,
    ReplyLine, Request, SoundGroup, StableSample, State,
};
use crate::{effects, music, time};
use num_traits::ToPrimitive;
use std::ops::{Div, Mul, Rem};
const POLL: u64 = 8;
const COUNT: u8 = 16;
const EFFECT: u16 = 8;
const MUSIC: u16 = 256;
const MUSIC_RATE: u16 = 32;
const DISTRIBUTE_HALF: u64 = 53;
const FIRST_HALF: u64 = 9;
const SECOND_HALF: u64 = 19;
const RETRY: u64 = 14;
const AFTER_EFFECT: u64 = 57;
const WITHOUT_EFFECT: u64 = 29;
const IDLE: u64 = 874;
const AFTER_MUSIC: u64 = 501;
/// Spec: IndependentCadence. Full groups of eight are discarded from each observed batch.
#[must_use]
pub fn distribute(phase: &mut CadencePhase, batch: u8) -> (bool, bool) {
    let remainder = u16::from(batch).checked_rem(EFFECT).unwrap_or(0);
    let effect = u16::from(phase.effect).saturating_add(remainder);
    let music = u16::from(phase.music).saturating_add(MUSIC_RATE.saturating_mul(remainder));
    phase.effect = u8::try_from(effect.checked_rem(EFFECT).unwrap_or(0)).unwrap_or(0);
    phase.music = u8::try_from(music.checked_rem(MUSIC).unwrap_or(0)).unwrap_or(0);
    (effect >= EFFECT, music >= MUSIC)
}
/// Spec: VisibleCompletion. Ordered observable milestones with exact instants.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Output {
    /// Clock count read and cleared.
    Clock {
        /// Observation instant.
        at: AudioInstant,
        /// Four-bit count.
        batch: u8,
    },
    /// Request read.
    Observation {
        /// Observation instant.
        at: AudioInstant,
        /// Request line.
        line: ReplyLine,
        /// Observed request.
        request: Request,
    },
    /// Equal pair accepted.
    Staged {
        /// Second observation instant.
        at: AudioInstant,
        /// Request line.
        line: ReplyLine,
        /// Accepted request.
        request: Request,
    },
    /// Completed work committed and published.
    Publication {
        /// Publication instant.
        at: AudioInstant,
        /// Reply line.
        line: ReplyLine,
        /// Published reply.
        reply: crate::Reply,
        /// Completed semantic result.
        result: CompletionResult,
    },
}
/// Spec: VisibleCompletion. Environment-owned ordered presentation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Input {
    /// Exact presentation instant.
    pub at: AudioInstant,
    group: SoundGroup,
    request: Request,
}
impl Input {
    /// Spec: StagedRequests. Cue determines its group.
    #[must_use]
    pub const fn cue(at: AudioInstant, cue: crate::CueId) -> Self {
        Self {
            at,
            group: effects::group(cue),
            request: Request::Cue(cue),
        }
    }
    /// Spec: StagedRequests. Absence releases a named line.
    #[must_use]
    pub const fn absence(at: AudioInstant, group: SoundGroup) -> Self {
        Self {
            at,
            group,
            request: Request::Absent,
        }
    }
}
/// Spec: VisibleCompletion. Resume saved state and process every milestone through the stop.
///
/// Inputs are ordered within this interval; updates at a milestone precede observation.
/// All arithmetic is exact and unbounded. Future inputs remain the caller's responsibility.
pub fn advance(
    state: &mut State,
    content: &Content,
    elapsed: AudioInstant,
    inputs: &[Input],
) -> Vec<Output> {
    let stop = time::add(&state.now, elapsed);
    let mut outputs = Vec::new();
    let mut inputs = inputs.iter().peekable();
    loop {
        let due = deadline(&state.phase);
        let next = inputs
            .peek()
            .map_or(&due, |input| std::cmp::min(&input.at, &due))
            .clone();
        if next > stop {
            break;
        }
        accrue(state, next.clone());
        while inputs.peek().is_some_and(|input| input.at <= next) {
            if let Some(input) = inputs.next() {
                state.requests[input.group] = input.request;
            }
        }
        if due <= next {
            milestone(state, content, &mut outputs);
        }
    }
    accrue(state, stop);
    outputs
}
fn deadline(phase: &AudioPhase) -> AudioInstant {
    match phase {
        AudioPhase::ObserveClock { at }
        | AudioPhase::Distribute { at, .. }
        | AudioPhase::Work { at, .. }
        | AudioPhase::MusicStart { at }
        | AudioPhase::Idle { at } => at.clone(),
        AudioPhase::Sample { sample, .. } => time::add(
            &sample.anchor,
            time::halves(if sample.first.is_some() {
                SECOND_HALF
            } else {
                FIRST_HALF
            }),
        ),
    }
}
fn accrue(state: &mut State, at: AudioInstant) {
    let (phase, periods) = state.clock.phase.accrue(&time::subtract(&at, &state.now));
    state.clock.phase = phase;
    let added = (&periods).rem(u64::from(COUNT)).to_u8().unwrap_or(0);
    state.clock.unread = state
        .clock
        .unread
        .wrapping_add(added)
        .checked_rem(COUNT)
        .unwrap_or(0);
    state.now = at;
}
fn milestone(state: &mut State, content: &Content, outputs: &mut Vec<Output>) {
    match state.phase.clone() {
        AudioPhase::ObserveClock { .. } => observe_clock(state, outputs),
        AudioPhase::Distribute { batch, .. } => cadence(state, content, batch),
        AudioPhase::Work { result, .. } => publish(state, result, outputs),
        AudioPhase::Sample { sample, next } => sample_pair(state, content, sample, next, outputs),
        AudioPhase::MusicStart { .. } => start_music(state, content),
        AudioPhase::Idle { .. } => {
            state.clock.lattice.clone_from(&state.now);
            observe_clock(state, outputs);
        }
    }
}
fn observe_clock(state: &mut State, outputs: &mut Vec<Output>) {
    let batch = state.clock.unread;
    state.clock.unread = 0;
    outputs.push(Output::Clock {
        at: state.now.clone(),
        batch,
    });
    state.phase = if batch == 0 {
        AudioPhase::ObserveClock {
            at: time::add(&state.now, time::cycles(POLL)),
        }
    } else {
        AudioPhase::Distribute {
            at: time::add(&state.now, time::halves(DISTRIBUTE_HALF)),
            batch,
        }
    };
}
fn cadence(state: &mut State, content: &Content, batch: u8) {
    let (effect, music) = distribute(&mut state.cadence, batch);
    state.music_due = music;
    if effect {
        start_effect(state, content, SoundGroup::Action, state.now.clone());
    } else {
        state.phase = AudioPhase::MusicStart {
            at: time::add(&state.now, time::cycles(WITHOUT_EFFECT)),
        };
    }
}
fn start_effect(state: &mut State, content: &Content, group: SoundGroup, anchor: AudioInstant) {
    let mut occupied = state.reservations.clone();
    for (other, saved) in &state.groups {
        if other != group {
            occupied.extend(effects::owned(&saved.playback));
        }
    }
    let result = effects::turn(
        group,
        &state.groups[group],
        content,
        &occupied,
        &state.song.mix.echo_voices,
    );
    let at = time::add(anchor, time::whole(result.work.0)).max(state.now.clone());
    state.phase = AudioPhase::Work {
        at,
        result: CompletionResult::Effect {
            group,
            result: result.group,
            echo_voices: result.echo_voices,
        },
    };
}
fn publish(state: &mut State, result: CompletionResult, outputs: &mut Vec<Output>) {
    let (line, reply, next) = match &result {
        CompletionResult::Effect {
            group,
            result,
            echo_voices,
        } => {
            state.groups[*group] = result.clone();
            state.song.mix.echo_voices.clone_from(echo_voices);
            let next = match group {
                SoundGroup::Action => Continuation::Effect(SoundGroup::Ambient),
                SoundGroup::Ambient => Continuation::Effect(SoundGroup::Contact),
                SoundGroup::Contact => Continuation::AfterEffects,
            };
            (
                ReplyLine::Group(*group),
                crate::Reply::Effect(result.reply),
                next,
            )
        }
        CompletionResult::Music(song) => {
            state.song = song.clone();
            (
                ReplyLine::Music,
                crate::Reply::Music(state.resident_music_reply.clone()),
                Continuation::AfterMusic,
            )
        }
    };
    outputs.push(Output::Publication {
        at: state.now.clone(),
        line,
        reply,
        result,
    });
    state.phase = AudioPhase::Sample {
        sample: StableSample {
            line,
            anchor: state.now.clone(),
            first: None,
        },
        next,
    };
}
fn sample_pair(
    state: &mut State,
    content: &Content,
    mut sample: StableSample,
    next: Continuation,
    outputs: &mut Vec<Output>,
) {
    let value = match sample.line {
        ReplyLine::Group(group) => state.requests[group],
        ReplyLine::Music => Request::Absent,
    };
    outputs.push(Output::Observation {
        at: state.now.clone(),
        line: sample.line,
        request: value,
    });
    let Some(first) = sample.first else {
        sample.first = Some(value);
        state.phase = AudioPhase::Sample { sample, next };
        return;
    };
    if first != value {
        sample.first = None;
        sample.anchor = time::add(&sample.anchor, time::cycles(RETRY));
        state.phase = AudioPhase::Sample { sample, next };
        return;
    }
    if let ReplyLine::Group(group) = sample.line {
        state.groups[group].staged = value;
    }
    outputs.push(Output::Staged {
        at: state.now.clone(),
        line: sample.line,
        request: value,
    });
    continue_after(state, content, sample.anchor, &next);
}
fn continue_after(state: &mut State, content: &Content, anchor: AudioInstant, next: &Continuation) {
    match next {
        Continuation::Effect(group) => start_effect(state, content, *group, anchor),
        Continuation::AfterEffects => {
            state.phase = AudioPhase::MusicStart {
                at: time::add(anchor, time::cycles(AFTER_EFFECT)),
            }
        }
        Continuation::AfterMusic => poll_after(state, time::add(anchor, time::cycles(AFTER_MUSIC))),
    }
}
fn start_music(state: &mut State, content: &Content) {
    if !state.music_due {
        state.phase = AudioPhase::Idle {
            at: time::add(
                &state.now,
                time::subtract(time::cycles(IDLE), time::halves(1)),
            ),
        };
        return;
    }
    let result = music::turn(&state.song, &content.tracks);
    state.music_due = false;
    state.phase = AudioPhase::Work {
        at: time::add(&state.now, time::whole(result.work.0)),
        result: CompletionResult::Music(result.song),
    };
}
fn poll_after(state: &mut State, completion: AudioInstant) {
    let at = time::subtract(completion, time::halves(1));
    state.clock.lattice.clone_from(&at);
    state.phase = AudioPhase::ObserveClock { at };
}
/// Spec: ClockObservation. Select the first opportunity on the caller's initial episode lattice.
/// Restoration instead retains the phase and does not call initialization.
pub fn initialize(state: &mut State) {
    let delta = time::subtract(&state.now, &state.clock.lattice);
    let steps = time::floor(delta).div(POLL);
    let candidate = time::add(&state.clock.lattice, time::whole(steps.mul(POLL)));
    let at = if candidate < state.now {
        time::add(candidate, time::cycles(POLL))
    } else {
        candidate
    };
    state.phase = AudioPhase::ObserveClock { at };
}
/// Spec: VisibleCompletion. Integral work as an exact elapsed interval, never CPU conversion.
#[must_use]
pub fn elapsed(work: AudioCycles) -> AudioInstant {
    time::whole(work.0)
}
