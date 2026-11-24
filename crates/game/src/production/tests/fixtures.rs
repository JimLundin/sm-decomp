use audio_transport::frame::{EffectCadence, FrameCadence, TurnCount};
use audio_transport::*;
use core::num::Wrapping;
use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroU8;
use types::{Pair, Pose};

pub(in crate::production) fn samus() -> samus_movement::Samus {
    samus_movement::Samus {
        position: Pair {
            x: Wrapping(100 * 65536),
            y: Wrapping(100 * 65536),
        },
        pose: Pose {
            body: types::Body::Upright,
            action: types::Action::Falling,
            facing: types::Facing::Right,
            aim: types::Aim::Forward,
            gun: types::Gun::Stowed,
            legs: types::Legs::Together,
        },
        run: samus_movement::Run {
            speed: Wrapping(0),
            way: types::Side::Right,
            carry: samus_movement::Carry::FollowsPad,
            dash: Wrapping(0),
            dashing: samus_movement::Dashing::Off,
        },
        vertical: samus_movement::Vertical {
            speed: Wrapping(0),
            phase: samus_movement::Phase::Falling,
            bounces: samus_movement::Bounces::None,
        },
        spin: samus_movement::Spin::Off,
        seat: samus_movement::Seat::Unseated,
        jump_held: samus_movement::JumpHeld::NONE,
    }
}

/// An invented two-track song and one-span clips for Beam and Land, checked against one timbre
/// and echo delay two.
pub(in crate::production) fn audio_content() -> crate::production::audio::Content {
    let clip = EffectClip {
        parts: vec![EffectPartContent {
            spans: vec![EffectSpan {
                duration: EffectTurns(NonZeroU8::MIN),
                pitch: EffectPitch::new(71).unwrap(),
                timbre: TimbreId(0),
                motion: None,
            }],
            loop_start: None,
        }],
    };
    crate::production::audio::Content::checked(
        Content {
            effects: BTreeMap::from([(CueId::Beam, clip.clone()), (CueId::Land, clip)]),
            tracks: vec![
                TrackContent {
                    phrases: vec![vec![MusicEvent::Sustain, MusicEvent::Complete]]
                };
                2
            ],
        },
        1,
        2,
    )
    .unwrap()
}

/// A frame whose cadence runs no audio turn and no sender service.
pub(in crate::production) const fn silent_cadence() -> FrameCadence {
    FrameCadence {
        effects: EffectCadence::WithoutService(TurnCount::ZERO),
        music: TurnCount::ZERO,
    }
}

/// A frame with one effect turn on each side of its sender service and one music turn.
pub(in crate::production) const fn busy_cadence() -> FrameCadence {
    FrameCadence {
        effects: EffectCadence::AroundService {
            before: TurnCount::ONE,
            after: TurnCount::ONE,
        },
        music: TurnCount::ONE,
    }
}

pub(in crate::production) fn song(master: u16) -> SongState {
    let one = NonZeroU8::new(1).unwrap();
    let track = TrackState {
        position: TrackPosition::Root { cursor: 0 },
        note: MusicTurns(one),
        release: 0,
        duration: MusicTurns(one),
        articulation: 128,
        pitch: MusicalPitch::new(24).unwrap(),
        loudness: Loudness(2560),
        ramp: None,
        timbre: TimbreId(0),
        offset: PitchOffset::Zero,
        strength: 64,
    };
    SongState {
        tracks: vec![track; 2],
        mix: SongMix {
            master: Loudness(master),
            percussion: 0,
            echo_voices: BTreeSet::new(),
            echo_left: 0,
            echo_right: 0,
            echo_delay: 2,
            feedback: 0,
            filter: 0,
            echo_enabled: false,
        },
    }
}

fn group(parts: usize) -> GroupState {
    GroupState {
        history: vec![ReleaseMemory::Inactive; parts],
        handled: Request::Absent,
        staged: Request::Absent,
        reply: Request::Absent,
        playback: Playback::Empty,
    }
}

/// An invented transport between frames: every sender ready, every line and reply absent.
pub(in crate::production) fn transport() -> State {
    let reply = ResidentMusicReply {
        identity: "invented arrangement".into(),
        original_byte: 42,
    };
    State {
        senders: enum_map::EnumMap::from_fn(|_| SenderState {
            queue: CueQueue::default(),
            progress: SenderProgress::Ready,
        }),
        request_lines: enum_map::EnumMap::from_fn(|_| Request::Absent),
        groups: enum_map::enum_map! { SoundGroup::Action => group(1), SoundGroup::Ambient => group(1), SoundGroup::Contact => group(2) },
        song: song(1000),
        music_reply: reply.clone(),
        resident_music_reply: reply,
        reservations: BTreeSet::new(),
    }
}
