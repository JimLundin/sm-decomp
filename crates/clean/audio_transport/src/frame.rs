//! Spec: FrameCadence and PublishedReplies. One frame of audio transport.
//!
//! A frame runs in the spec's order: its offers, the effect turns before its sender service, the
//! service, the effect turns after it, and its music turns.
use crate::sender::{self, Offer};
use crate::{effects, music};
use std::collections::BTreeSet;
use types::audio::{Content, EffectVoiceId, SoundGroup, State};
/// Spec: FrameCadence. The most audio turns of one kind a frame completes.
const MOST_TURNS: u8 = 2;
/// Spec: FrameCadence. How many turns of one kind a frame completes: zero, one or two.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TurnCount(u8);
impl TurnCount {
    /// No turn.
    pub const ZERO: Self = Self(0);
    /// One turn.
    pub const ONE: Self = Self(1);
    /// Two turns.
    pub const TWO: Self = Self(MOST_TURNS);
    /// The count of `turns` turns, if a frame can complete that many.
    #[must_use]
    pub const fn new(turns: u8) -> Option<Self> {
        if turns <= MOST_TURNS {
            Some(Self(turns))
        } else {
            None
        }
    }
    /// How many turns.
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }
}
/// Spec: FrameCadence. Where a frame's effect turns fall around its sender service.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EffectCadence {
    /// The frame runs its sender service between these effect turns.
    AroundService {
        /// Effect turns completed before the service.
        before: TurnCount,
        /// Effect turns completed after it.
        after: TurnCount,
    },
    /// The frame runs out of time before its service: it runs only its effect turns.
    WithoutService(TurnCount),
}
/// Spec: FrameCadence. A frame's audio cadence, borrowed from the caller until the audio side's
/// own clock is modeled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameCadence {
    /// The frame's effect turns and whether its sender service runs.
    pub effects: EffectCadence,
    /// The music turns the frame completes, independent of its effect turns and service.
    pub music: TurnCount,
}
/// Spec: FrameCadence. Play one frame of audio transport.
///
/// Apply the frame's queue offers, complete the effect turns before its sender service, run the
/// service if the frame has one, complete its remaining effect turns, then advance the song once
/// for each of its music turns.
///
/// The service reads each group's reply as last published, in this frame's earlier effect turns or
/// an earlier frame. The published replies are read back from `state`.
pub fn play(state: &mut State, content: &Content, offers: &[Offer], cadence: FrameCadence) {
    for offer in offers {
        sender::offer(&mut state.senders[offer.cue.group()].queue, *offer);
    }
    match cadence.effects {
        EffectCadence::AroundService { before, after } => {
            effect_turns(state, content, before);
            serve(state);
            effect_turns(state, content, after);
        }
        EffectCadence::WithoutService(turns) => effect_turns(state, content, turns),
    }
    for _ in 0..cadence.music.get() {
        music_turn(state, content);
    }
}
fn effect_turns(state: &mut State, content: &Content, turns: TurnCount) {
    for _ in 0..turns.get() {
        effect_turn(state, content);
    }
}
/// Spec: PublishedReplies and StagedRequests. One effect turn serves each sound group once, in
/// order: the group handles its staged request and plays, publishes the request it handled as its
/// reply, then stages its request line's current value for its next turn.
fn effect_turn(state: &mut State, content: &Content) {
    for group in SoundGroup::IN_TURN_ORDER {
        let occupied = occupied_beyond(state, group);
        let audio = &mut state.groups[group];
        effects::turn(audio, content, &occupied, &mut state.song.mix.echo_voices);
        audio.reply = audio.handled;
        audio.staged = state.request_lines[group];
    }
}
/// Spec: EffectResources. The effect voices held by external reservations and the other groups.
fn occupied_beyond(state: &State, group: SoundGroup) -> BTreeSet<EffectVoiceId> {
    let others = state
        .groups
        .iter()
        .filter(|(other, _)| *other != group)
        .flat_map(|(_, audio)| audio.playback.owned_voices());
    state.reservations.iter().copied().chain(others).collect()
}
/// Spec: RequestDelivery. The sender service serves each group's queue once, in order, reading the
/// group's last published reply.
fn serve(state: &mut State) {
    for group in SoundGroup::IN_TURN_ORDER {
        sender::serve(
            &mut state.senders[group],
            &mut state.request_lines[group],
            state.groups[group].reply,
        );
    }
}
/// Spec: PublishedReplies. A music turn advances the song, then publishes the resident reply on
/// the music line.
fn music_turn(state: &mut State, content: &Content) {
    music::turn(&mut state.song, &content.tracks);
    state.music_reply.clone_from(&state.resident_music_reply);
}
