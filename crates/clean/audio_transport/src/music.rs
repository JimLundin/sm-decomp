//! Spec: MusicalProgress, FinitePhrases, WholeArrangement, LoudnessProgress and SongControls.
//! One music turn of the resident two-track arrangement.
use std::num::NonZeroU8;
use types::audio::{
    Loudness, LoudnessRamp, MusicEvent, MusicTurns, SongMix, SongState, TrackContent,
    TrackPosition, TrackState, count_down,
};
/// Spec: LoudnessProgress. Fractional units in one whole unit of loudness or echo gain.
const FRACTIONS: u16 = 256;
/// Where a track's pass through its events ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Pass {
    /// A note sounds on.
    Sounding,
    /// The track completed its root.
    RootCompleted,
}
/// Spec: WholeArrangement and LoudnessProgress. Advance the song by one music turn.
///
/// The tracks progress in arrangement order. The first track to complete its root restarts the
/// whole arrangement at once: later tracks skip their pass, and the abandoned pass advances no
/// volume. The restarted introductions are consumed in this same turn, and loudness ramps then
/// advance once, after the final pass of both tracks.
pub fn turn(song: &mut SongState, content: &[TrackContent]) {
    let SongState { tracks, mix } = song;
    // `any` stops at the first completed root, so no later track runs the abandoned pass.
    let restarts = tracks
        .iter_mut()
        .zip(content)
        .any(|(track, graph)| progress(track, graph, mix) == Pass::RootCompleted);
    if restarts {
        restart(tracks, content, mix);
    }
    tracks.iter_mut().for_each(advance_ramp);
}
/// Spec: WholeArrangement. Restart both tracks at their roots, without finite-repeat or ramp
/// progress, and consume their introductions; scalar controls stay until an introduction
/// replaces them.
fn restart(tracks: &mut [TrackState], content: &[TrackContent], mix: &mut SongMix) {
    for (track, graph) in tracks.iter_mut().zip(content) {
        track.position = TrackPosition::Root { cursor: 0 };
        track.ramp = None;
        // An admitted root introduction ends in a timed event, so this pass cannot complete it.
        consume(track, graph, mix);
    }
}
/// Spec: MusicalProgress. Count the sounding note down, or start the next timed event when its
/// last turn is consumed.
fn progress(track: &mut TrackState, graph: &TrackContent, mix: &mut SongMix) -> Pass {
    let Some(left) = count_down(track.note.0) else {
        return consume(track, graph, mix);
    };
    track.note = MusicTurns(left);
    track.release = track.release.saturating_sub(1);
    Pass::Sounding
}
/// Spec: MusicalProgress. Take the track's events from its position through the next timed event,
/// or until its root completes.
fn consume(track: &mut TrackState, graph: &TrackContent, mix: &mut SongMix) -> Pass {
    while let Some(event) = event_at(graph, &track.position) {
        if let Some(end) = take(track, mix, event) {
            return end;
        }
    }
    // A position past its phrase's events has nothing left to play: the root is complete.
    Pass::RootCompleted
}
/// Spec: MusicalProgress. Take one event: where the pass ends with it, or none if it goes on.
fn take(track: &mut TrackState, mix: &mut SongMix, event: &MusicEvent) -> Option<Pass> {
    match event {
        MusicEvent::Repeat { phrase, count } => {
            enter_phrase(&mut track.position, *phrase, *count);
            None
        }
        MusicEvent::Complete => complete_phrase(&mut track.position),
        MusicEvent::Tone(_) | MusicEvent::Sustain => {
            apply(track, mix, event);
            next_event(&mut track.position);
            start_note(track);
            Some(Pass::Sounding)
        }
        MusicEvent::Timing { .. }
        | MusicEvent::Loudness(_)
        | MusicEvent::Ramp { .. }
        | MusicEvent::MasterLoudness(_)
        | MusicEvent::Timbre(_)
        | MusicEvent::Offset(_)
        | MusicEvent::Percussion(_)
        | MusicEvent::EchoMix { .. }
        | MusicEvent::EchoSpace { .. } => {
            apply(track, mix, event);
            next_event(&mut track.position);
            None
        }
    }
}
/// Spec: MusicalProgress. A timed event sounds for the retained duration, released after
/// `max(1, floor(duration * articulation / 256))` turns.
fn start_note(track: &mut TrackState) {
    track.note = track.duration;
    let scaled = u16::from(track.duration.0.get()).saturating_mul(u16::from(track.articulation));
    // The high byte is `floor(scaled / 256)`.
    let [release, _] = scaled.to_be_bytes();
    track.release = release.max(1);
}
/// The event at a track's position, if its phrase has one there.
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
/// Move past the event just taken, within its phrase.
const fn next_event(position: &mut TrackPosition) {
    match position {
        TrackPosition::Root { cursor } | TrackPosition::Repeated { cursor, .. } => {
            *cursor = cursor.saturating_add(1);
        }
    }
}
/// Spec: FinitePhrases. Enter a repeated phrase, keeping the root continuation after its entry.
const fn enter_phrase(position: &mut TrackPosition, phrase: usize, count: NonZeroU8) {
    let continuation = match position {
        TrackPosition::Root { cursor } | TrackPosition::Repeated { cursor, .. } => {
            cursor.saturating_add(1)
        }
    };
    *position = TrackPosition::Repeated {
        phrase,
        cursor: 0,
        remaining: count,
        continuation,
    };
}
/// Spec: FinitePhrases and WholeArrangement. Finish a phrase: repeat it while repeats remain,
/// return to the root continuation after its last repeat, or complete the root.
const fn complete_phrase(position: &mut TrackPosition) -> Option<Pass> {
    let TrackPosition::Repeated {
        remaining,
        cursor,
        continuation,
        ..
    } = position
    else {
        return Some(Pass::RootCompleted);
    };
    if let Some(left) = count_down(*remaining) {
        *remaining = left;
        *cursor = 0;
    } else {
        *position = TrackPosition::Root {
            cursor: *continuation,
        };
    }
    None
}
/// Spec: SongControls and LoudnessProgress. A consumed choice replaces what the track or the song
/// retained; a tone replaces the pitch, which a sustain keeps.
fn apply(track: &mut TrackState, mix: &mut SongMix, event: &MusicEvent) {
    match event {
        MusicEvent::Timing {
            duration,
            articulation,
        } => {
            track.duration = *duration;
            if let Some(choice) = articulation {
                track.articulation = choice.ratio;
                track.strength = choice.strength;
            }
        }
        MusicEvent::Tone(pitch) => track.pitch = *pitch,
        MusicEvent::Loudness(value) => track.loudness = Loudness(fractional(*value)),
        MusicEvent::Ramp { turns, target } => {
            track.ramp = Some(ramp_from(track.loudness, *turns, *target));
        }
        MusicEvent::Timbre(timbre) => track.timbre = *timbre,
        MusicEvent::Offset(offset) => track.offset = *offset,
        MusicEvent::MasterLoudness(value) => mix.master = Loudness(fractional(*value)),
        MusicEvent::Percussion(set) => mix.percussion = *set,
        MusicEvent::EchoMix {
            voices,
            left,
            right,
        } => {
            mix.echo_voices.clone_from(voices);
            mix.echo_left = fractional(*left);
            mix.echo_right = fractional(*right);
            mix.echo_enabled = true;
        }
        MusicEvent::EchoSpace {
            delay,
            feedback,
            filter,
        } => {
            mix.echo_delay = *delay;
            mix.feedback = *feedback;
            mix.filter = *filter;
        }
        MusicEvent::Sustain | MusicEvent::Repeat { .. } | MusicEvent::Complete => {}
    }
}
/// Spec: LoudnessProgress. A whole-unit value in fractional units: 256 times it.
fn fractional(whole: u8) -> u16 {
    u16::from(whole).saturating_mul(FRACTIONS)
}
/// Spec: LoudnessProgress. A ramp of `turns` turns from `loudness` to the whole-unit `target`.
///
/// Its increment is `256 * (target - floor(loudness / 256)) / turns`, truncated toward zero and
/// then wrapped to signed sixteen bits.
#[must_use]
pub fn ramp_from(loudness: Loudness, turns: NonZeroU8, target: u8) -> LoudnessRamp {
    let [whole, _] = loudness.0.to_be_bytes();
    let step = i32::from(target)
        .saturating_sub(i32::from(whole))
        .saturating_mul(i32::from(FRACTIONS))
        .checked_div(i32::from(turns.get()))
        .unwrap_or(0);
    // The low two bytes are the step wrapped to sixteen bits.
    let [low, high, ..] = step.to_le_bytes();
    LoudnessRamp {
        remaining: turns,
        target,
        increment: i16::from_le_bytes([low, high]),
    }
}
/// Spec: LoudnessProgress. Once a turn, after the final pass: add the ramp's increment with
/// sixteen-bit wrapping, or on its last turn land exactly on its target. A finished ramp keeps no
/// configuration.
fn advance_ramp(track: &mut TrackState) {
    let Some(ramp) = track.ramp.as_mut() else {
        return;
    };
    if let Some(left) = count_down(ramp.remaining) {
        ramp.remaining = left;
        track.loudness = Loudness(track.loudness.0.wrapping_add_signed(ramp.increment));
    } else {
        track.loudness = Loudness(fractional(ramp.target));
        track.ramp = None;
    }
}
