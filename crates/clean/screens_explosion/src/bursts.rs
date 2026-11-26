//! Spec: `StationBursts`. Three bands of bursts cross the station before the final explosion.
//!
//! A burst is launched at the screen anchor (the pivot less the plane's floored scroll) plus
//! its offset. From its launch frame on it drifts every finished frame and plays its group's
//! track, hidden while its first picture is delayed, and goes when its track finishes.

use core::num;

use enum_map::EnumMap;
use types::Pair;
use types::explosion::{
    Burst, BurstGroup, BurstId, BurstTrack, ExplosionState, PictureAge, SampleAge, SampleLook,
    ScreenPoint, Subpixels,
};

/// Where a burst is launched from the screen anchor, and after how many updates its first
/// picture shows.
#[derive(Clone, Copy, Debug)]
pub struct Placement {
    /// The offset from the screen anchor, in screen pixels.
    offset: ScreenPoint,
    /// Updates until the first picture shows, from 1: a delay of 1 shows it on the launch frame.
    first_picture_delay: u8,
}

/// A placement at these offsets, in whole pixels, with this first picture delay.
const fn placed(x: i32, y: i32, first_picture_delay: u8) -> Placement {
    Placement {
        offset: Pair {
            x: Subpixels::whole(x),
            y: Subpixels::whole(y),
        },
        first_picture_delay,
    }
}

/// Spec: `StationBursts`. A first picture delay that shows the picture on the launch frame.
const AT_ONCE: u8 = 1;

/// Spec: `StationBursts`. The first group's placements, in launch order.
const FIRST_GROUP: [Placement; 5] = [
    placed(16, -16, 1),
    placed(-16, 16, 16),
    placed(16, 16, 32),
    placed(-16, -16, 48),
    placed(0, 0, 64),
];

/// Spec: `StationBursts`. The middle group's placements, in launch order.
const MIDDLE_GROUP: [Placement; 6] = [
    placed(14, -8, AT_ONCE),
    placed(8, 12, AT_ONCE),
    placed(-16, 12, AT_ONCE),
    placed(-8, -14, AT_ONCE),
    placed(0, 0, AT_ONCE),
    placed(16, 14, AT_ONCE),
];

/// Spec: `StationBursts`. The last group's placements, in launch order.
const LAST_GROUP: [Placement; 4] = [
    placed(8, -4, 1),
    placed(12, 8, 4),
    placed(-8, -10, 8),
    placed(-12, 12, 16),
];

/// Spec: `StationBreaks`. The final burst, launched at the flash's centre.
const FINAL_BURST: Placement = placed(0, 0, AT_ONCE);

/// Spec: `StationBursts`. Each live burst drifts 1/4 px right and 1/16 px up every finished
/// frame, shown or not.
const BURST_DRIFT: ScreenPoint = Pair {
    x: Subpixels::fraction(1, 4),
    y: Subpixels::fraction(-1, 16),
};

/// How a group's bursts are launched.
#[derive(Clone, Copy, Debug)]
enum Cadence {
    /// All of them on the group's start frame.
    Together,
    /// One on the start frame, then one every `frames` frames, until the age `until` cuts the
    /// schedule.
    Every {
        /// The frames between launches.
        frames: u32,
        /// The age from which no more are launched.
        until: PictureAge,
    },
}

/// A group's launch schedule: when it starts, at what cadence, and where its bursts go.
#[derive(Clone, Copy, Debug)]
struct Wave {
    /// The group launched.
    group: BurstGroup,
    /// The picture age of its first launch.
    starts: PictureAge,
    /// How its launches follow.
    cadence: Cadence,
    /// Its placements, in launch order.
    placements: &'static [Placement],
}

/// Spec: `StationBursts`. The age the last group starts, which also cuts the middle schedule.
const LAST_GROUP_AGE: PictureAge = PictureAge(273);

/// Spec: `StationBursts`. The three scheduled groups.
const WAVES: [Wave; 3] = [
    Wave {
        group: BurstGroup::First,
        starts: PictureAge(129),
        cadence: Cadence::Together,
        placements: &FIRST_GROUP,
    },
    Wave {
        group: BurstGroup::Middle,
        starts: PictureAge(210),
        cadence: Cadence::Every {
            frames: 12,
            until: LAST_GROUP_AGE,
        },
        placements: &MIDDLE_GROUP,
    },
    Wave {
        group: BurstGroup::Last,
        starts: LAST_GROUP_AGE,
        cadence: Cadence::Together,
        placements: &LAST_GROUP,
    },
];

impl Wave {
    /// The placements this wave launches at `age`, each with its index in launch order.
    fn due_at(&self, age: PictureAge) -> Vec<(usize, Placement)> {
        let indexed = self.placements.iter().copied().enumerate();
        match self.cadence {
            Cadence::Together => indexed.filter(|_| age == self.starts).collect(),
            Cadence::Every { frames, until } => {
                let launch = age
                    .0
                    .checked_sub(self.starts.0)
                    .filter(|since| age < until && since.checked_rem(frames) == Some(0))
                    .and_then(|since| since.checked_div(frames))
                    .and_then(|index| usize::try_from(index).ok());
                indexed
                    .filter(|&(index, _)| Some(index) == launch)
                    .collect()
            }
        }
    }
}

/// Spec: `StationBursts`, `StationBreaks`. The screen anchor: the pivot less the plane's scroll
/// rounded down to whole pixels.
#[must_use]
pub fn screen_anchor(pivot: Pair<i16>, scroll: ScreenPoint) -> ScreenPoint {
    ScreenPoint::at_pixels(pivot).minus(scroll.floor())
}

/// Adds a burst at `anchor` plus its offset, its age counting toward zero through its delay.
/// The frame's burst update then gives it its first drift and animation update.
fn launch(
    bursts: &mut Vec<Burst>,
    group: BurstGroup,
    index: usize,
    placement: Placement,
    anchor: ScreenPoint,
) {
    let occurrence = u8::try_from(index)
        .ok()
        .and_then(|index| num::NonZeroU8::MIN.checked_add(index));
    if let Some(occurrence) = occurrence {
        bursts.push(Burst {
            id: BurstId { group, occurrence },
            age: SampleAge(i16::from(placement.first_picture_delay).wrapping_neg()),
            position: anchor.plus(placement.offset),
        });
    }
}

/// Spec: `StationBreaks`. Launches the final burst at the flash's centre; the frame's burst
/// update then applies its first drift, so the drift locates the burst one drift past the
/// centre.
pub fn launch_final(bursts: &mut Vec<Burst>, center: ScreenPoint) {
    launch(bursts, BurstGroup::Final, 0, FINAL_BURST, center);
}

/// Spec: `StationBursts`. Launches the bursts the schedule makes due at the current picture
/// age, from the anchor of the plane's scroll after this frame's drift.
pub fn launch_scheduled(state: &mut ExplosionState, pivot: Pair<i16>) {
    let anchor = screen_anchor(pivot, state.pose.scroll);
    for wave in &WAVES {
        for (index, placement) in wave.due_at(state.age) {
            launch(&mut state.bursts, wave.group, index, placement, anchor);
        }
    }
}

/// Spec: `StationBursts`. One finished frame for every live burst: it drifts, its track
/// advances a frame, and it goes once its track finishes. The rest stay front to back: by
/// group, then in launch order.
pub fn update(bursts: &mut Vec<Burst>, tracks: &EnumMap<BurstGroup, BurstTrack>) {
    for burst in bursts.iter_mut() {
        burst.position = burst.position.plus(BURST_DRIFT);
        burst.age = SampleAge(burst.age.0.saturating_add(1));
    }
    bursts.retain(|burst| !finished(&tracks[burst.id.group], burst.age));
    bursts.sort_by_key(|burst| burst.id);
}

/// The frames of one pass of a track.
fn pass_frames(track: &BurstTrack) -> u32 {
    track
        .samples
        .iter()
        .map(|sample| u32::from(sample.frames.get()))
        .sum()
}

/// Whether a burst of this age has played all its track's passes.
fn finished(track: &BurstTrack, age: SampleAge) -> bool {
    let track_frames = pass_frames(track).saturating_mul(u32::from(track.passes.get()));
    u32::try_from(age.0).is_ok_and(|played| played >= track_frames)
}

/// What a burst shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BurstFrame {
    /// Nothing: its first picture is delayed, or its sample is blank.
    Hidden,
    /// The track's sample at this index of a pass, 0 the first.
    Showing(usize),
}

/// Spec: `StationBursts`. What a burst of this age shows: hidden through its delay, then each
/// sample of its track's pass for its frames, pass after pass.
#[must_use]
pub fn frame(track: &BurstTrack, age: SampleAge) -> BurstFrame {
    let Some(in_pass) = u32::try_from(age.0)
        .ok()
        .and_then(|played| played.checked_rem(pass_frames(track)))
    else {
        return BurstFrame::Hidden;
    };
    let sample_ends = track.samples.iter().scan(0_u32, |end, sample| {
        *end = end.saturating_add(u32::from(sample.frames.get()));
        Some(*end)
    });
    let Some(index) = sample_ends.into_iter().position(|end| in_pass < end) else {
        return BurstFrame::Hidden;
    };
    match track.samples.get(index).map(|sample| &sample.look) {
        Some(SampleLook::Drawing(_)) => BurstFrame::Showing(index),
        Some(SampleLook::Blank) | None => BurstFrame::Hidden,
    }
}
