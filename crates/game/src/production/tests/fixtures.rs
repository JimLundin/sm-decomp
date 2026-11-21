use audio_transport::*;
use core::num::Wrapping;
use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroU8;
use types::{Pair, Pose};

pub(super) fn samus() -> samus_movement::Samus {
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

pub(in crate::production) fn audio_content() -> Content {
    Content {
        effects: BTreeMap::new(),
        tracks: vec![
            TrackContent {
                phrases: vec![vec![MusicEvent::Complete]]
            };
            2
        ],
    }
}

fn song(master: u16) -> SongState {
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

pub(in crate::production) fn pending_audio() -> (State, SongState) {
    let pending = song(1999);
    let state = State {
        now: time::cycles(10),
        clock: ClockState {
            phase: AudioCyclePhase::try_from(time::cycles(10)).unwrap(),
            unread: 0,
            lattice: time::cycles(0),
        },
        cadence: CadencePhase {
            effect: 0,
            music: 0,
        },
        music_due: false,
        groups: enum_map::enum_map! { SoundGroup::Action => group(1), SoundGroup::Ambient => group(1), SoundGroup::Contact => group(2) },
        song: song(1000),
        resident_music_reply: ResidentMusicReply {
            identity: "invented arrangement".into(),
            original_byte: 42,
        },
        reservations: BTreeSet::new(),
        requests: enum_map::enum_map! { SoundGroup::Action => Request::Absent, SoundGroup::Ambient => Request::Absent, SoundGroup::Contact => Request::Absent },
        phase: AudioPhase::Work {
            at: time::cycles(100),
            result: CompletionResult::Music(pending.clone()),
        },
    };
    (state, pending)
}
