//! Spec: MusicalProgress, FinitePhrases, WholeArrangement and MusicTiming.
//!
//! Musical work accumulates exactly in unbounded whole AudioCycles; bounded loudness wraps.
use crate::ExactAudioCycles;
use crate::{
    AudioCycles, Loudness, LoudnessRamp, MusicEvent, MusicTurns, SongMix, SongState, TrackContent,
    TrackPosition, TrackState,
};
use num_bigint::BigUint;
use num_traits::Zero;
use std::num::NonZeroU8;
use std::ops::{Add, AddAssign, Sub};
const FRACTIONS: u16 = 256;
const OCTAVE: u8 = 12;
const OCTAVES: u64 = 6;
const OCTAVE_WORK: u64 = 14;
const COMMON: u64 = 651;
const PROGRESS: u64 = 159;
const RELEASED: u64 = 144;
const PREVIEW_DURATION: u8 = 3;
const PASSAGE: u64 = 23;
const ROOT_WORK: u64 = 923;
const ROOT_RETURN_WORK: u64 = 1025;
const PREVIEW_TONE: u64 = 70;
const PREVIEW_TIMED_TONE: u64 = 58;
const PREVIEW_SUSTAIN: u64 = 18;
const PREVIEW_TIMED_SUSTAIN: u64 = 6;
const ENTER: u64 = 233;
const PREVIEW_ENTER: u64 = 59;
const PREVIEW_ROOT: u64 = 61;
const AGAIN: u64 = 80;
const PREVIEW_AGAIN: u64 = 52;
const RETURN: u64 = 102;
const PREVIEW_RETURN: u64 = 50;
const IMMEDIATE: u64 = 130;
const TIMBRE: u64 = 334;
const OFFSET: u64 = 122;
const MASTER: u64 = 123;
const PERCUSSION: u64 = 120;
const ECHO_MIX: u64 = 198;
const ECHO_SPACE: u64 = 599;

/// Spec: MusicTiming. Work result, committed only by the scheduler at completion.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MusicResult {
    /// Completed retained song.
    pub song: SongState,
    /// Work in audio cycles.
    pub work: AudioCycles,
}
/// Spec: WholeArrangement. First root completion abandons volume and restarts once.
#[must_use]
pub fn turn(song: &SongState, content: &[TrackContent]) -> MusicResult {
    let mut result = song.clone();
    let mut work = BigUint::from(COMMON);
    let mut tones = Vec::new();
    let mut restart = false;
    for (track, graph) in result.tracks.iter_mut().zip(content) {
        let passage = progress(track, graph, &mut result.mix);
        work.add_assign(passage.work);
        tones.push(passage.tone);
        if passage.root {
            restart = true;
            break;
        }
    }
    if restart {
        tones.clear();
        for (track, graph) in result.tracks.iter_mut().zip(content) {
            track.position = TrackPosition::Root { cursor: 0 };
            track.ramp = None;
            let passage = consume(track, graph, &mut result.mix);
            work.add_assign(BigUint::from(PROGRESS).add(passage.work));
            tones.push(passage.tone);
        }
    }
    for (track, tone) in result.tracks.iter_mut().zip(tones) {
        work.add_assign(volume(track, tone));
    }
    MusicResult {
        song: result,
        work: ExactAudioCycles(work),
    }
}
struct Passage {
    work: BigUint,
    tone: bool,
    root: bool,
}
fn progress(track: &mut TrackState, graph: &TrackContent, mix: &mut SongMix) -> Passage {
    if track.note.0.get() == 1 {
        let mut passage = consume(track, graph, mix);
        if !passage.root {
            passage.work.add_assign(PROGRESS);
        }
        return passage;
    }
    let old_release = track.release;
    let old_note = track.note.0.get();
    track.note = MusicTurns(NonZeroU8::new(old_note.saturating_sub(1)).unwrap_or(NonZeroU8::MIN));
    track.release = old_release.saturating_sub(1);
    let mut work = BigUint::from(if old_release == 0 { RELEASED } else { PROGRESS });
    if old_release == 1 || (old_release > 0 && old_note == PREVIEW_DURATION) {
        const EXPIRES: u64 = 15;
        const THREE: u64 = 21;
        work.add_assign(
            BigUint::from(if old_release == 1 { EXPIRES } else { THREE })
                .add(preview(track, graph).0),
        );
    }
    Passage {
        work,
        tone: false,
        root: false,
    }
}
/// Spec: MusicTiming. Preview walks the semantic path on a copied cursor only.
#[must_use]
pub fn preview(track: &TrackState, graph: &TrackContent) -> AudioCycles {
    let mut position = track.position.clone();
    let mut work = BigUint::zero();
    let mut timing = false;
    loop {
        let Some(event) = event_at(graph, &position) else {
            return ExactAudioCycles(work);
        };
        match event {
            MusicEvent::Tone(_) => {
                return ExactAudioCycles(work.add(if timing {
                    PREVIEW_TIMED_TONE
                } else {
                    PREVIEW_TONE
                }));
            }
            MusicEvent::Sustain => {
                return ExactAudioCycles(work.add(if timing {
                    PREVIEW_TIMED_SUSTAIN
                } else {
                    PREVIEW_SUSTAIN
                }));
            }
            MusicEvent::Complete | MusicEvent::Repeat { .. } => {
                let boundary = boundary(&mut position, event);
                work.add_assign(boundary.preview);
                if boundary.root {
                    return ExactAudioCycles(work);
                }
            }
            MusicEvent::Timing { articulation, .. } => {
                const READ: u64 = 22;
                const ARTICULATION: u64 = 14;
                timing = true;
                work.add_assign(READ);
                work.add_assign(if articulation.is_some() {
                    ARTICULATION
                } else {
                    0
                });
                bump(&mut position);
            }
            MusicEvent::Loudness(_)
            | MusicEvent::Ramp { .. }
            | MusicEvent::MasterLoudness(_)
            | MusicEvent::Timbre(_)
            | MusicEvent::Offset(_)
            | MusicEvent::Percussion(_)
            | MusicEvent::EchoMix { .. }
            | MusicEvent::EchoSpace { .. } => {
                const SCALAR: u64 = 45;
                work.add_assign(SCALAR);
                bump(&mut position);
            }
        }
    }
}
fn event_at<'a>(graph: &'a TrackContent, position: &TrackPosition) -> Option<&'a MusicEvent> {
    let (phrase, cursor) = match position {
        TrackPosition::Root { cursor } => (0, *cursor),
        TrackPosition::Repeated { phrase, cursor, .. } => (*phrase, *cursor),
    };
    graph
        .phrases
        .get(phrase)
        .and_then(|events| events.get(cursor))
}
const fn bump(position: &mut TrackPosition) {
    match position {
        TrackPosition::Root { cursor } | TrackPosition::Repeated { cursor, .. } => {
            *cursor = cursor.saturating_add(1);
        }
    }
}
struct Boundary {
    work: u64,
    preview: u64,
    root: bool,
    returned: bool,
}
fn boundary(position: &mut TrackPosition, event: &MusicEvent) -> Boundary {
    if let MusicEvent::Repeat { phrase, count } = event {
        let continuation = match position {
            TrackPosition::Root { cursor } | TrackPosition::Repeated { cursor, .. } => {
                cursor.saturating_add(1)
            }
        };
        *position = TrackPosition::Repeated {
            phrase: *phrase,
            cursor: 0,
            remaining: *count,
            continuation,
        };
        return Boundary {
            work: ENTER,
            preview: PREVIEW_ENTER,
            root: false,
            returned: false,
        };
    }
    let TrackPosition::Repeated {
        remaining,
        cursor,
        continuation,
        ..
    } = position
    else {
        return Boundary {
            work: ROOT_WORK,
            preview: PREVIEW_ROOT,
            root: true,
            returned: false,
        };
    };
    if remaining.get() > 1 {
        *remaining = NonZeroU8::new(remaining.get().saturating_sub(1)).unwrap_or(NonZeroU8::MIN);
        *cursor = 0;
        Boundary {
            work: AGAIN,
            preview: PREVIEW_AGAIN,
            root: false,
            returned: false,
        }
    } else {
        *position = TrackPosition::Root {
            cursor: *continuation,
        };
        Boundary {
            work: RETURN,
            preview: PREVIEW_RETURN,
            root: false,
            returned: true,
        }
    }
}
fn consume(track: &mut TrackState, graph: &TrackContent, mix: &mut SongMix) -> Passage {
    let mut work = BigUint::from(PASSAGE);
    let mut returned = false;
    loop {
        let Some(event) = event_at(graph, &track.position) else {
            return Passage {
                work: BigUint::from(ROOT_WORK),
                tone: false,
                root: true,
            };
        };
        if matches!(event, MusicEvent::Complete | MusicEvent::Repeat { .. }) {
            let step = boundary(&mut track.position, event);
            if step.root {
                let ending = if returned {
                    work.sub(PASSAGE).sub(RETURN).add(ROOT_RETURN_WORK)
                } else {
                    work.sub(PASSAGE).add(ROOT_WORK)
                };
                return Passage {
                    work: ending,
                    tone: false,
                    root: true,
                };
            }
            work.add_assign(step.work);
            returned = step.returned;
            continue;
        }
        work.add_assign(apply(track, mix, event));
        if let MusicEvent::Timing { extra, .. } = event {
            work.add_assign(&extra.0);
        }
        bump(&mut track.position);
        let tone = matches!(event, MusicEvent::Tone(_));
        if tone || matches!(event, MusicEvent::Sustain) {
            track.note = track.duration;
            track.release = u8::try_from(
                u16::from(track.duration.0.get())
                    .saturating_mul(u16::from(track.articulation))
                    .checked_div(FRACTIONS)
                    .unwrap_or(0)
                    .max(1),
            )
            .unwrap_or(u8::MAX);
            return Passage {
                work,
                tone,
                root: false,
            };
        }
        returned = false;
    }
}
fn apply(track: &mut TrackState, mix: &mut SongMix, event: &MusicEvent) -> u64 {
    match event {
        MusicEvent::Timing {
            duration,
            articulation,
            ..
        } => timing(track, *duration, articulation.as_ref()),
        MusicEvent::Tone(pitch) => {
            const TONE: u64 = 467;
            track.pitch = *pitch;
            TONE.saturating_add(OCTAVE_WORK.saturating_mul(
                OCTAVES.saturating_sub(u64::from(pitch.get().checked_div(OCTAVE).unwrap_or(0))),
            ))
        }
        MusicEvent::Sustain | MusicEvent::Complete | MusicEvent::Repeat { .. } => 0,
        MusicEvent::Loudness(value) => {
            track.loudness = Loudness(u16::from(*value).saturating_mul(FRACTIONS));
            IMMEDIATE
        }
        MusicEvent::Ramp { turns, target } => start_ramp(track, *turns, *target),
        MusicEvent::Timbre(value) => {
            track.timbre = *value;
            TIMBRE
        }
        MusicEvent::Offset(value) => {
            track.offset = *value;
            OFFSET
        }
        MusicEvent::MasterLoudness(_)
        | MusicEvent::Percussion(_)
        | MusicEvent::EchoMix { .. }
        | MusicEvent::EchoSpace { .. } => mix_choice(mix, event),
    }
}
const fn timing(
    track: &mut TrackState,
    duration: MusicTurns,
    articulation: Option<&crate::Articulation>,
) -> u64 {
    const TIMING: u64 = 38;
    const ARTICULATION: u64 = 71;
    track.duration = duration;
    let Some(choice) = articulation else {
        return TIMING;
    };
    track.articulation = choice.ratio;
    track.strength = choice.strength;
    TIMING.saturating_add(ARTICULATION)
}
/// Spec: LoudnessProgress. Signed truncation and sixteen-bit wrapping are intentional.
pub fn start_ramp(track: &mut TrackState, turns: NonZeroU8, target: u8) -> u64 {
    const DOWN: u64 = 271;
    const UP: u64 = 256;
    let whole = track.loudness.0.checked_div(FRACTIONS).unwrap_or(0);
    let raw = i32::from(target)
        .saturating_sub(i32::from(whole))
        .saturating_mul(i32::from(FRACTIONS))
        .checked_div(i32::from(turns.get()))
        .unwrap_or(0);
    let bytes = raw.to_le_bytes();
    let increment = i16::from_le_bytes([
        bytes.first().copied().unwrap_or(0),
        bytes.get(1).copied().unwrap_or(0),
    ]);
    track.ramp = Some(LoudnessRamp {
        remaining: turns,
        target,
        increment,
    });
    if u16::from(target) < whole { DOWN } else { UP }
}
/// Spec: LoudnessProgress. Only final-pass volume advances; inactive has no payload.
pub fn volume(track: &mut TrackState, tone: bool) -> u64 {
    const ACTIVE: u64 = 357;
    const LAST: u64 = 344;
    const TONE: u64 = 310;
    let Some(mut ramp) = track.ramp.take() else {
        return if tone { TONE } else { 0 };
    };
    if ramp.remaining.get() == 1 {
        track.loudness = Loudness(u16::from(ramp.target).saturating_mul(FRACTIONS));
        return LAST;
    }
    ramp.remaining =
        NonZeroU8::new(ramp.remaining.get().saturating_sub(1)).unwrap_or(NonZeroU8::MIN);
    track.loudness = Loudness(track.loudness.0.wrapping_add_signed(ramp.increment));
    track.ramp = Some(ramp);
    ACTIVE
}
fn mix_choice(mix: &mut SongMix, event: &MusicEvent) -> u64 {
    match event {
        MusicEvent::MasterLoudness(value) => {
            mix.master = Loudness(u16::from(*value).saturating_mul(FRACTIONS));
            MASTER
        }
        MusicEvent::Percussion(value) => {
            mix.percussion = *value;
            PERCUSSION
        }
        MusicEvent::EchoMix {
            voices,
            left,
            right,
        } => {
            mix.echo_voices.clone_from(voices);
            mix.echo_left = u16::from(*left).saturating_mul(FRACTIONS);
            mix.echo_right = u16::from(*right).saturating_mul(FRACTIONS);
            mix.echo_enabled = true;
            ECHO_MIX
        }
        MusicEvent::EchoSpace {
            delay,
            feedback,
            filter,
        } => {
            mix.echo_delay = *delay;
            mix.feedback = *feedback;
            mix.filter = *filter;
            ECHO_SPACE
        }
        MusicEvent::Timing { .. }
        | MusicEvent::Tone(_)
        | MusicEvent::Sustain
        | MusicEvent::Loudness(_)
        | MusicEvent::Ramp { .. }
        | MusicEvent::Timbre(_)
        | MusicEvent::Offset(_)
        | MusicEvent::Repeat { .. }
        | MusicEvent::Complete => 0,
    }
}
