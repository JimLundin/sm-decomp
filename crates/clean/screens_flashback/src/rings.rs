//! The rings: two pairs form at set places on a schedule, then drift diagonally across the
//! room; the first pair's ring toward Samus can push her.

use core::num::Wrapping;
use core::ops::Range;

use enum_map::EnumMap;
use types::flashback::{
    Animation, DrawRank, EncounterState, HoldBeats, PairSchedule, Ring, RingPair, RingPath,
    RingStep,
};
use types::{Fixed, Pair, Parts, Px, Side, px};

use crate::clip;
use crate::ranks;

/// Spec: `EncounterStarts`. The initial ring pair wait, 74 beats after the encounter's first beat.
const FIRST_PAIR_WAIT_BEATS: u16 = 74;
/// Spec: `RingsAppear`. The second pair's wait, 128 beats after the first pair appears.
const SECOND_PAIR_WAIT_BEATS: u16 = 128;
/// Spec: `RingsAppear`, `RingsSweep`. The beats each ring picture holds, 10.
const RING_IMAGE_HOLD_BEATS: u16 = 10;

/// Spec: `EncounterStarts`. The first pair, due after the initial ring pair wait.
pub const FIRST_PAIR_DUE: PairSchedule = PairSchedule {
    pair: RingPair::First,
    hold: HoldBeats::of::<FIRST_PAIR_WAIT_BEATS>(),
};

/// Spec: `RingsAppear`. The second pair, due after its wait once the first appears.
const SECOND_PAIR_DUE: PairSchedule = PairSchedule {
    pair: RingPair::Second,
    hold: HoldBeats::of::<SECOND_PAIR_WAIT_BEATS>(),
};

/// Spec: `RingsAppear`, `RingsSweep`. Each ring picture's full hold.
const RING_IMAGE_HOLD: HoldBeats = HoldBeats::of::<RING_IMAGE_HOLD_BEATS>();

/// Spec: `RingsSweep`. The ring that can hit reaches 8 px right of its centre.
const RING_REACH: i32 = 8;
/// Spec: `RingsSweep`. It tests against 5 px left of Samus's whole X.
const SAMUS_EDGE: i32 = 5;
/// Spec: `RingsSweep`. The first whole room Y the other rings may hold, 16.
const RING_ROWS_TOP: u16 = 16;
/// Spec: `RingsSweep`. The whole room Y just past the other rings' rows, 208.
const RING_ROWS_END: u16 = 208;
/// Spec: `RingsSweep`. The other rings stay within whole room Y 16 inclusive to 208 exclusive.
const RING_ROWS: Range<u16> = RING_ROWS_TOP..RING_ROWS_END;
/// Spec: `RingsSweep`. The protection and push holds both last 11 beats.
const HIT_TIMER_BEATS: u8 = 11;

/// Spec: ring travel, +0.5 px per beat (`0000.8000`).
const FORWARD_HALF: Fixed = Wrapping(0x0000_8000);
/// Spec: ring travel, -0.5 px per beat (`FFFF.8000`).
const BACK_HALF: Fixed = Wrapping(0xFFFF_8000);

/// Spec: `RingsSweep`. How a moving ring's sweep can end.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RingEnd {
    /// It can hit Samus, regardless of its Y.
    MeetsSamus,
    /// It disappears once its whole Y leaves the permitted rows.
    LeavesRows,
}

/// Spec: ring starts and travel. Where a ring spawns, how far it travels each beat and how its
/// sweep ends.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RingStart {
    /// Its spawn point, whole pixels.
    spawn: Pair<Px>,
    /// Its travel per beat.
    travel: Pair<Fixed>,
    /// How its sweep ends.
    end: RingEnd,
}

/// Spec: ring starts and travel, one row per path.
const fn ring_start(path: RingPath) -> RingStart {
    match path {
        RingPath::FirstToward => RingStart {
            spawn: Pair {
                x: Wrapping(112),
                y: Wrapping(72),
            },
            travel: Pair {
                x: FORWARD_HALF,
                y: FORWARD_HALF,
            },
            end: RingEnd::MeetsSamus,
        },
        RingPath::FirstAcross => RingStart {
            spawn: Pair {
                x: Wrapping(192),
                y: Wrapping(56),
            },
            travel: Pair {
                x: BACK_HALF,
                y: FORWARD_HALF,
            },
            end: RingEnd::LeavesRows,
        },
        RingPath::SecondToward => RingStart {
            spawn: Pair {
                x: Wrapping(128),
                y: Wrapping(48),
            },
            travel: Pair {
                x: FORWARD_HALF,
                y: FORWARD_HALF,
            },
            end: RingEnd::LeavesRows,
        },
        RingPath::SecondAcross => RingStart {
            spawn: Pair {
                x: Wrapping(232),
                y: Wrapping(80),
            },
            travel: Pair {
                x: BACK_HALF,
                y: FORWARD_HALF,
            },
            end: RingEnd::LeavesRows,
        },
    }
}

/// Spec: `RingsAppear`. A pair's rings, the toward ring first.
const fn pair_paths(pair: RingPair) -> [RingPath; 2] {
    match pair {
        RingPair::First => [RingPath::FirstToward, RingPath::FirstAcross],
        RingPair::Second => [RingPath::SecondToward, RingPath::SecondAcross],
    }
}

/// Spec: `RingsAppear`. What is due once a pair appears: the second pair after the first, then
/// nothing.
const fn after(pair: RingPair) -> Option<PairSchedule> {
    match pair {
        RingPair::First => Some(SECOND_PAIR_DUE),
        RingPair::Second => None,
    }
}

/// Spec: `RingsAppear`, `RingsSweep`. The step after each: three forming pictures, then the
/// moving clip, which repeats.
const fn next_ring_step(step: RingStep) -> RingStep {
    match step {
        RingStep::FormingFirst => RingStep::FormingSecond,
        RingStep::FormingSecond => RingStep::FormingThird,
        RingStep::FormingThird | RingStep::MovingFourth => RingStep::MovingFirst,
        RingStep::MovingFirst => RingStep::MovingSecond,
        RingStep::MovingSecond => RingStep::MovingThird,
        RingStep::MovingThird => RingStep::MovingFourth,
    }
}

/// Spec: `RingsAppear`. A ring moves on the beats after it is made ready to move.
const fn moving(step: RingStep) -> bool {
    matches!(
        step,
        RingStep::MovingFirst
            | RingStep::MovingSecond
            | RingStep::MovingThird
            | RingStep::MovingFourth
    )
}

/// Spec: `RingsSweep`. What a ring offers Samus's owners when it reaches her: rightward
/// push, with protection and push holds. Energy and ammo stay as they were.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RingKnockback {
    /// The way she is pushed.
    pub direction: Side,
    /// Her protection hold, in beats.
    pub invincible_beats: u8,
    /// Her push hold, in beats.
    pub knockback_beats: u8,
}

/// Spec: `RingsSweep`. The push a ring gives: rightward, with 11-beat protection and push
/// holds.
const RING_KNOCKBACK: RingKnockback = RingKnockback {
    direction: Side::Right,
    invincible_beats: HIT_TIMER_BEATS,
    knockback_beats: HIT_TIMER_BEATS,
};

/// What becomes of a ring on a beat.
enum RingFate {
    /// It stays, as it now is.
    Stays(Ring),
    /// It disappears.
    Leaves,
    /// It reaches Samus and disappears.
    MeetsSamus,
}

/// One beat of the rings: the schedule and new arrivals, then the sweep of the rings already
/// present. `shaking_at_start` tells whether Mother Brain was shaking as the beat began.
pub fn advance(
    state: &mut EncounterState,
    shaking_at_start: bool,
    samus_x: Px,
) -> Option<RingKnockback> {
    let arrivals = arrivals(state);
    let knockback = sweep(&mut state.rings, shaking_at_start, samus_x);
    for (path, ring) in arrivals {
        state.rings[path] = Some(ring);
    }
    knockback
}

/// Spec: `RingsAppear`, `FightPictures`. A pair due in more than one beat spends a beat; a due
/// pair's rings arrive at their spawn points with their first picture. They take their ranks,
/// the toward ring first, while the schedule and the rings leaving this beat still hold
/// theirs; then the next pair, if any, is scheduled.
fn arrivals(state: &mut EncounterState) -> Vec<(RingPath, Ring)> {
    let Some(schedule) = state.next_pair else {
        return Vec::new();
    };
    if let Some(hold) = schedule.hold.counted_down() {
        state.next_pair = Some(PairSchedule { hold, ..schedule });
        return Vec::new();
    }
    let mut taken = ranks::taken(state);
    let arrivals = pair_paths(schedule.pair)
        .into_iter()
        .map(|path| (path, spawned(path, ranks::claim(&mut taken))))
        .collect();
    state.next_pair = after(schedule.pair);
    arrivals
}

/// Spec: `RingsAppear`. A ring at its spawn point, showing its first picture for a full hold.
fn spawned(path: RingPath, rank: DrawRank) -> Ring {
    let spawn = ring_start(path).spawn;
    Ring {
        position: Pair {
            x: px(spawn.x),
            y: px(spawn.y),
        },
        clip: Animation {
            step: RingStep::FormingFirst,
            hold: RING_IMAGE_HOLD,
        },
        rank,
    }
}

/// Spec: `RingsSweep`. Each ring present as the beat began takes its beat; the push is offered
/// if one reaches Samus.
fn sweep(
    rings: &mut EnumMap<RingPath, Option<Ring>>,
    shaking_at_start: bool,
    samus_x: Px,
) -> Option<RingKnockback> {
    let mut knockback = None;
    for (path, slot) in rings.iter_mut() {
        let Some(ring) = *slot else {
            continue;
        };
        *slot = match fate(path, ring, shaking_at_start, samus_x) {
            RingFate::Stays(ring) => Some(ring),
            RingFate::Leaves => None,
            RingFate::MeetsSamus => {
                knockback = Some(RING_KNOCKBACK);
                None
            }
        };
    }
    knockback
}

/// Spec: `RingsSweep`. A moving ring adds its travel and is checked at its new whole
/// coordinates: the ring that can hit Samus leaves when it reaches her; another leaves when its
/// Y leaves the permitted rows; and a ring that misses leaves if Mother Brain was already
/// shaking. A surviving ring, moving or forming, runs its clip.
fn fate(path: RingPath, ring: Ring, shaking_at_start: bool, samus_x: Px) -> RingFate {
    let clip = clip::advanced(ring.clip, |step| Animation {
        step: next_ring_step(step),
        hold: RING_IMAGE_HOLD,
    });
    if !moving(ring.clip.step) {
        return RingFate::Stays(Ring { clip, ..ring });
    }
    let start = ring_start(path);
    let position = Pair {
        x: ring.position.x + start.travel.x,
        y: ring.position.y + start.travel.y,
    };
    let stays_in_play = match start.end {
        RingEnd::MeetsSamus => {
            if reaches(position.x.pixel(), samus_x) {
                return RingFate::MeetsSamus;
            }
            true
        }
        RingEnd::LeavesRows => RING_ROWS.contains(&position.y.pixel().0),
    };
    if stays_in_play && !shaking_at_start {
        RingFate::Stays(Ring {
            position,
            clip,
            rank: ring.rank,
        })
    } else {
        RingFate::Leaves
    }
}

/// Spec: `RingsSweep`. Whether the ring's reach, 8 px right of its whole X, reaches or passes
/// 5 px left of Samus's whole X.
fn reaches(ring_x: Px, samus_x: Px) -> bool {
    i32::from(ring_x.0).saturating_add(RING_REACH)
        >= i32::from(samus_x.0).saturating_sub(SAMUS_EDGE)
}
