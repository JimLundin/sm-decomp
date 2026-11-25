//! Spec: `BurstsPulse`. Once Mother Brain is defeated, bursts of two sizes start at their own
//! delays around her and pulse until the return dissolve ends.

use core::num::Wrapping;

use enum_map::EnumMap;
use types::Pair;
use types::flashback::{Animation, Burst, BurstPlace, BurstStep, EncounterState, HoldBeats};

use crate::clip;
use crate::pictures::BRAIN_ORIGIN;
use crate::ranks;

/// Spec: `BurstsPulse`. The size of a burst, which sets its pictures' hold.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BurstSize {
    /// Small: each picture holds 3 beats.
    Small,
    /// Large: each picture holds 6 beats.
    Large,
}

/// Spec: burst starts. A place's size, its offset from Mother Brain in pixels and its delay
/// after the decisive impact.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct BurstStart {
    /// Its size.
    size: BurstSize,
    /// Its offset from Mother Brain's origin.
    offset: Pair<i16>,
    /// The beats it waits before its first picture.
    delay: HoldBeats,
}

/// Spec: burst starts. The near burst's delay, 1 beat after the decisive impact.
const NEAR_DELAY_BEATS: u16 = 1;
/// Spec: burst starts. The low burst's delay, 8 beats.
const LOW_DELAY_BEATS: u16 = 8;
/// Spec: burst starts. The high burst's delay, 16 beats.
const HIGH_DELAY_BEATS: u16 = 16;
/// Spec: burst starts. The centre burst's delay, 1 beat.
const CENTRE_DELAY_BEATS: u16 = 1;
/// Spec: burst starts. The upper right burst's delay, 16 beats.
const UPPER_RIGHT_DELAY_BEATS: u16 = 16;
/// Spec: burst starts. The lower left burst's delay, 32 beats.
const LOWER_LEFT_DELAY_BEATS: u16 = 32;
/// Spec: burst starts. The upper left burst's delay, 48 beats.
const UPPER_LEFT_DELAY_BEATS: u16 = 48;
/// Spec: burst starts. The lower right burst's delay, 64 beats.
const LOWER_RIGHT_DELAY_BEATS: u16 = 64;

/// Spec: burst starts. The near burst's offset from Mother Brain, 16 px right.
const NEAR: Pair<i16> = Pair { x: 16, y: 0 };
/// Spec: burst starts. The low burst's offset, 16 px left and 4 px down.
const LOW: Pair<i16> = Pair { x: -16, y: 4 };
/// Spec: burst starts. The high burst's offset, 16 px left and 8 px up.
const HIGH: Pair<i16> = Pair { x: -16, y: -8 };
/// Spec: burst starts. The centre burst's offset, on Mother Brain's origin.
const CENTRE: Pair<i16> = Pair { x: 0, y: 0 };
/// Spec: burst starts. The upper right burst's offset, 16 px right and 16 px up.
const UPPER_RIGHT: Pair<i16> = Pair { x: 16, y: -16 };
/// Spec: burst starts. The lower left burst's offset, 16 px left and 8 px down.
const LOWER_LEFT: Pair<i16> = Pair { x: -16, y: 8 };
/// Spec: burst starts. The upper left burst's offset, 8 px left and 16 px up.
const UPPER_LEFT: Pair<i16> = Pair { x: -8, y: -16 };
/// Spec: burst starts. The lower right burst's offset, 8 px right and 8 px down.
const LOWER_RIGHT: Pair<i16> = Pair { x: 8, y: 8 };

/// Spec: `BurstsPulse`. The beats each small picture holds, 3.
const SMALL_IMAGE_HOLD_BEATS: u16 = 3;
/// Spec: `BurstsPulse`. The beats each large picture holds, 6.
const LARGE_IMAGE_HOLD_BEATS: u16 = 6;
/// Spec: `BurstsPulse`. The blank beats between rounds, 16.
const BLANK_HOLD_BEATS: u16 = 16;

/// Spec: burst starts, one row per place.
const fn burst_start(place: BurstPlace) -> BurstStart {
    match place {
        BurstPlace::Near => BurstStart {
            size: BurstSize::Small,
            offset: NEAR,
            delay: HoldBeats::of::<NEAR_DELAY_BEATS>(),
        },
        BurstPlace::Low => BurstStart {
            size: BurstSize::Small,
            offset: LOW,
            delay: HoldBeats::of::<LOW_DELAY_BEATS>(),
        },
        BurstPlace::High => BurstStart {
            size: BurstSize::Small,
            offset: HIGH,
            delay: HoldBeats::of::<HIGH_DELAY_BEATS>(),
        },
        BurstPlace::Centre => BurstStart {
            size: BurstSize::Large,
            offset: CENTRE,
            delay: HoldBeats::of::<CENTRE_DELAY_BEATS>(),
        },
        BurstPlace::UpperRight => BurstStart {
            size: BurstSize::Large,
            offset: UPPER_RIGHT,
            delay: HoldBeats::of::<UPPER_RIGHT_DELAY_BEATS>(),
        },
        BurstPlace::LowerLeft => BurstStart {
            size: BurstSize::Large,
            offset: LOWER_LEFT,
            delay: HoldBeats::of::<LOWER_LEFT_DELAY_BEATS>(),
        },
        BurstPlace::UpperLeft => BurstStart {
            size: BurstSize::Large,
            offset: UPPER_LEFT,
            delay: HoldBeats::of::<UPPER_LEFT_DELAY_BEATS>(),
        },
        BurstPlace::LowerRight => BurstStart {
            size: BurstSize::Large,
            offset: LOWER_RIGHT,
            delay: HoldBeats::of::<LOWER_RIGHT_DELAY_BEATS>(),
        },
    }
}

/// Spec: `BurstsPulse`. Each picture's hold, by size.
const fn image_hold(size: BurstSize) -> HoldBeats {
    match size {
        BurstSize::Small => HoldBeats::of::<SMALL_IMAGE_HOLD_BEATS>(),
        BurstSize::Large => HoldBeats::of::<LARGE_IMAGE_HOLD_BEATS>(),
    }
}

/// Spec: `BurstsPulse`. The blank interval between rounds, for its full hold.
const BLANK_HOLD: HoldBeats = HoldBeats::of::<BLANK_HOLD_BEATS>();

/// Spec: `BurstsPulse`. The step after each: six pictures, the blank interval, then the
/// pictures again.
const fn next_burst_step(step: BurstStep) -> BurstStep {
    match step {
        BurstStep::Waiting | BurstStep::Blank => BurstStep::First,
        BurstStep::First => BurstStep::Second,
        BurstStep::Second => BurstStep::Third,
        BurstStep::Third => BurstStep::Fourth,
        BurstStep::Fourth => BurstStep::Fifth,
        BurstStep::Fifth => BurstStep::Sixth,
        BurstStep::Sixth => BurstStep::Blank,
    }
}

/// Spec: `BurstsPulse`. The next step of a burst of `size`, for its full hold.
fn burst_after(step: BurstStep, size: BurstSize) -> Animation<BurstStep> {
    let next = next_burst_step(step);
    let hold = if next == BurstStep::Blank {
        BLANK_HOLD
    } else {
        image_hold(size)
    };
    Animation { step: next, hold }
}

/// Spec: `BurstsPulse`. One beat of each burst: a waiting burst spends a beat of its delay and
/// shows its first picture when it runs out; a showing one runs its clip.
pub fn pulse(bursts: &mut EnumMap<BurstPlace, Option<Burst>>) {
    for (place, slot) in bursts.iter_mut() {
        if let Some(burst) = slot {
            let size = burst_start(place).size;
            burst.clip = clip::advanced(burst.clip, |step| burst_after(step, size));
        }
    }
}

/// Spec: `BurstsPulse`, `FightPictures`. A defeat reserves every burst place, waiting out its
/// delay with no picture yet, each taking the foremost vacant rank in burst-table order.
pub fn reserve(state: &mut EncounterState) {
    let mut taken = ranks::taken(state);
    for (place, slot) in &mut state.bursts {
        let start = burst_start(place);
        *slot = Some(Burst {
            at: Pair {
                x: BRAIN_ORIGIN.x + Wrapping(start.offset.x.cast_unsigned()),
                y: BRAIN_ORIGIN.y + Wrapping(start.offset.y.cast_unsigned()),
            },
            clip: Animation {
                step: BurstStep::Waiting,
                hold: start.delay,
            },
            rank: ranks::claim(&mut taken),
        });
    }
}
