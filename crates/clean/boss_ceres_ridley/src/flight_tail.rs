//! The seven linked segments of Ridley's tail in flight, from base to tip.

use core::num;

use types::{Pair, Px};

use crate::her::Her;
use crate::ridley_phase::RidleyPhase;
use crate::tail::{TailCurl, place_at_rest};
use crate::tail_aim::CeresAim;

/// Number of linked segments, including the base and tip.
pub const TAIL_SEGMENTS: usize = 7;
const PIXEL: u16 = 256;
const QUARTER_TURN: u16 = 64;
const REST_ANGLE: u16 = 16_384;
const INITIAL_ANGLE_STEP: u16 = 16;
const INITIAL_LAG: u8 = 17;
const MAX_LAG: u8 = 30;
const LAST_SWISH_RUN: u8 = 15;
const FORWARD_LIMIT: TailAngle = TailAngle(16_368);
const BACK_LIMIT: TailAngle = TailAngle(16_448);
const RECOVERY_X: u16 = 112;
const GROWTH: u16 = 240;
const SHRINKAGE: u16 = 128;
const STRETCH_THRESHOLD: u16 = 3_072;
// Lengths in 1/256 px, in base-to-tip order.
const REST_LENGTHS: [u16; TAIL_SEGMENTS] = [512, 2048, 2048, 2048, 2048, 2048, 1280];
const MAX_LENGTH: u16 = 6144;
const CAPS: [u16; TAIL_SEGMENTS] = [512, MAX_LENGTH, MAX_LENGTH, 5632, 5632, 4608, 1280];

/// `TailSwings`: angle in 256ths of a turn, wrapping at 16 bits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TailAngle(pub u16);

/// `TailStretches`: length in 1/256 px, up to the longest segment's cap.
/// The importer checks the length against the particular segment's cap.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TailLength(u16);

impl TailLength {
    /// A length up to 24 px, expressed in 1/256 px.
    #[must_use]
    pub const fn new(subpixels: u16) -> Option<Self> {
        if subpixels <= MAX_LENGTH {
            Some(Self(subpixels))
        } else {
            None
        }
    }

    /// The length in 1/256 px.
    #[must_use]
    pub const fn subpixels(self) -> u16 {
        self.0
    }
}

/// `TailFollows`: angle units per run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SwingPace {
    /// One angle unit.
    Slow,
    /// Two angle units.
    Quick,
    /// Eight angle units, during a whip.
    Whip,
}

impl SwingPace {
    const fn units(self) -> u8 {
        match self {
            Self::Slow => 1,
            Self::Quick => 2,
            Self::Whip => 8,
        }
    }
}

/// `TailFollows`: which way the swing travels around the resting angle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SwingWay {
    /// Toward the forward limit or mark.
    Ahead,
    /// Toward the back limit.
    Back,
}

/// `TailFollows`: elapsed angle units before a segment propagates its swing, at most 30.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LagCount(u8);

impl LagCount {
    /// A lag count within the chain's range.
    #[must_use]
    pub const fn new(count: u8) -> Option<Self> {
        if count <= MAX_LAG {
            Some(Self(count))
        } else {
            None
        }
    }

    /// Elapsed angle units.
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }
}

/// `TailFollows`: whether a segment has already passed this swing to its neighbour.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SwingLag {
    /// Waiting to pass on the swing.
    Counting(LagCount),
    /// Already passed it on.
    Passed,
}

/// `TailSwishes`: runs at whip pace since the last swish, 0 to 15.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SwishRuns(u8);

impl SwishRuns {
    /// A count within the swish period.
    #[must_use]
    pub const fn new(count: u8) -> Option<Self> {
        if count <= LAST_SWISH_RUN {
            Some(Self(count))
        } else {
            None
        }
    }

    /// Runs elapsed in the period.
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }

    /// `TailSwishes`: measure the starting pace, before aiming or recovering.
    fn tick(&mut self, pace: SwingPace) -> Option<TailSwish> {
        let swish = pace == SwingPace::Whip && self.0 == LAST_SWISH_RUN;
        self.0 = if pace == SwingPace::Whip && !swish {
            self.0.saturating_add(1)
        } else {
            0
        };
        swish.then_some(TailSwish)
    }
}

/// `TailSwishes`: a request for the whip sound.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TailSwish;

impl TailSwish {
    /// Effects channel for the request.
    pub const CHANNEL: u8 = 3;
    /// Limit of the sound queue for this request.
    pub const QUEUE_LIMIT: u8 = 6;
}

/// One link of the tail, with the geometry and swing carried from the last run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TailSegment {
    /// Whether this link participates in the swing yet.
    pub swinging: bool,
    /// Whether it has passed the swing on.
    pub lag: SwingLag,
    /// Direction of its swing.
    pub way: SwingWay,
    /// Its angle, before adding the preceding link's angle for bending.
    pub angle: TailAngle,
    /// Its length in subpixels.
    pub length: TailLength,
    /// Whether it is extending.
    pub stretching: bool,
    /// Whole-pixel displacement from its parent (the anchor for the base).
    pub offset: Pair<i16>,
    /// Whole-pixel position last reported to drawing.
    pub place: Pair<Px>,
}

/// `TailWhips`: an aim mark in one of the two permitted arcs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TailMark(TailAngle);

impl TailMark {
    /// Construct a mark in the forward or chance arc.
    #[must_use]
    pub fn new(angle: TailAngle) -> Option<Self> {
        CeresAim::valid_mark(angle.0).then_some(Self(angle))
    }

    /// Angle of the mark.
    #[must_use]
    pub const fn angle(self) -> TailAngle {
        self.0
    }
}

/// The caller's in-flight shape. Curl is shared with the resting tail.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FlightTail {
    /// How fast the chain swings.
    pub pace: SwingPace,
    /// Where a whip aims.
    pub mark: Option<TailMark>,
    /// Progress toward the next swish.
    pub swish: SwishRuns,
    /// Shape in base-to-tip order.
    pub segments: [TailSegment; TAIL_SEGMENTS],
}

impl FlightTail {
    /// `TailComesLoose`: give every link its starting length and lag at the resting place.
    #[must_use]
    pub fn comes_loose(position: Pair<Px>) -> Self {
        let mut angle = REST_ANGLE;
        Self {
            pace: SwingPace::Slow,
            mark: None,
            swish: SwishRuns(0),
            segments: REST_LENGTHS.map(|length| {
                let segment = TailSegment {
                    swinging: true,
                    lag: SwingLag::Counting(LagCount(INITIAL_LAG)),
                    way: SwingWay::Ahead,
                    angle: TailAngle(angle),
                    length: TailLength(length),
                    stretching: false,
                    offset: Pair { x: 0, y: 0 },
                    place: place_at_rest(position),
                };
                angle = angle.wrapping_add(INITIAL_ANGLE_STEP);
                segment
            }),
        }
    }

    /// `TailBends`: places for drawing, base first and tip last.
    #[must_use]
    pub fn places(&self) -> [Pair<Px>; TAIL_SEGMENTS] {
        self.segments.each_ref().map(|segment| segment.place)
    }

    /// Aim from the previous position of the first link, then pass the swing along the chain.
    /// Each link is positioned for drawing before it is resized, so its new length affects
    /// its position on the next run.
    pub(crate) fn run(
        &mut self,
        position: Pair<Px>,
        curl: TailCurl,
        aim: CeresAim,
        her: Option<&Her>,
    ) -> Option<TailSwish> {
        let swish = self.swish.tick(self.pace);
        // `GetawayBackdrop`: keep the stopped tail's swing state, but still relink and resize.
        let swings = aim.phase != RidleyPhase::GetawayReady;
        if swings {
            self.aim_or_recover(position, aim, her);
        }
        let mut preceding = None;
        let mut passed_way = None;
        for ((segment, rest), cap) in self.segments.iter_mut().zip(REST_LENGTHS).zip(CAPS) {
            if let Some(way) = passed_way.take() {
                segment.swinging = true;
                segment.way = way;
            }
            if swings
                && (!segment.swinging
                    || (segment.ready(curl, self.pace, &mut passed_way)
                        && segment.swing(self.pace, self.mark, preceding)))
            {
                segment.bend(preceding);
            }
            segment.link(preceding.map_or_else(|| place_at_rest(position), |parent| parent.place));
            if preceding.is_some() {
                segment.resize(rest, cap);
            }
            preceding = Some(*segment);
        }
        swish
    }

    /// `TailWhips`, `TailRecovers`: retarget a fully swinging chain, or restart a fully held one.
    fn aim_or_recover(&mut self, position: Pair<Px>, aim: CeresAim, her: Option<&Her>) {
        let base = &mut self.segments;
        if base.iter().all(|segment| !segment.swinging) {
            base[0].swinging = true;
            self.mark = None;
            self.pace = if her.is_some_and(|her| her.position.x.0 >= RECOVERY_X) {
                SwingPace::Quick
            } else {
                SwingPace::Slow
            };
        } else if base.iter().all(|segment| segment.swinging)
            && let Some(mark) = her.and_then(|her| aim.mark(position, base[0].place, her.position))
            && mark < base[0].angle.0
        {
            self.mark = TailMark::new(TailAngle(mark));
            self.pace = SwingPace::Whip;
        }
    }
}

impl TailSegment {
    /// `TailFollows`: a link passes its current way before its own swing can turn it.
    const fn ready(
        &mut self,
        curl: TailCurl,
        pace: SwingPace,
        passed_way: &mut Option<SwingWay>,
    ) -> bool {
        let SwingLag::Counting(count) = self.lag else {
            return true;
        };
        if count.get() < curl.get() {
            self.lag = SwingLag::Counting(LagCount(count.get().saturating_add(pace.units())));
            return false;
        }
        self.lag = SwingLag::Passed;
        *passed_way = Some(self.way);
        true
    }

    /// `TailSwings`: advance toward the current limit. Settling holds the old offset.
    fn swing(&mut self, pace: SwingPace, mark: Option<TailMark>, preceding: Option<Self>) -> bool {
        let pace = u16::from(pace.units());
        if self.way == SwingWay::Back {
            let next = self.angle.0.wrapping_add(pace);
            self.angle = if next.wrapping_sub(BACK_LIMIT.0).cast_signed() < 0 {
                TailAngle(next)
            } else {
                self.way = SwingWay::Ahead;
                BACK_LIMIT
            };
            return true;
        }
        let limit = mark.map_or(FORWARD_LIMIT, TailMark::angle);
        self.stretching |= mark.is_some();
        let next = self.angle.0.wrapping_sub(pace);
        if next.wrapping_sub(1).wrapping_sub(limit.0).cast_signed() >= 0 {
            self.angle = TailAngle(next);
            return true;
        }
        self.way = SwingWay::Back;
        if mark.is_some() && !preceding.is_some_and(|parent| parent.swinging) {
            self.angle = limit;
            self.swinging = false;
            self.lag = SwingLag::Counting(LagCount(0));
            return false;
        }
        self.angle = FORWARD_LIMIT;
        true
    }

    /// `TailBends`: the preceding link's new angle contributes to this offset.
    fn bend(&mut self, preceding: Option<Self>) {
        let angle = self
            .angle
            .0
            .wrapping_add(preceding.map_or(0, |parent| parent.angle.0));
        let length = self.length.0 / PIXEL;
        self.offset = Pair {
            x: projected(angle, length),
            y: projected(angle.wrapping_add(QUARTER_TURN), length),
        };
    }

    /// `TailBends`: chain the offsets even when a link did not bend.
    const fn link(&mut self, parent: Pair<Px>) {
        self.place = Pair {
            x: num::Wrapping(parent.x.0.wrapping_add_signed(self.offset.x)),
            y: num::Wrapping(parent.y.0.wrapping_add_signed(self.offset.y)),
        };
    }

    /// `TailStretches`: a final growth step still applies past the threshold; shrinkage can
    /// undershoot the resting length.
    fn resize(&mut self, rest: u16, cap: u16) {
        if self.stretching {
            self.stretching = self.length.0 <= STRETCH_THRESHOLD;
            self.length.0 = self.length.0.saturating_add(GROWTH).min(cap);
        } else if self.length.0 > rest {
            self.length.0 = self.length.0.saturating_sub(SHRINKAGE);
        }
    }
}

/// `TailBends`: the first quarter wave of trunc(256 × sin(2πk / 256)), endpoints included.
/// Reflect and negate it for the other quadrants, keeping the projection integer-only.
const QUARTER_WAVE: [i16; 65] = [
    0, 6, 12, 18, 25, 31, 37, 43, 49, 56, 62, 68, 74, 80, 86, 92, 97, 103, 109, 115, 120, 126, 131,
    136, 142, 147, 152, 157, 162, 167, 171, 176, 181, 185, 189, 193, 197, 201, 205, 209, 212, 216,
    219, 222, 225, 228, 231, 234, 236, 238, 241, 243, 244, 246, 248, 249, 251, 252, 253, 254, 254,
    255, 255, 255, 256,
];
const HALF_TURN: u16 = 128;

/// `TailBends`: truncate the sine first, then round the scaled offset down (also for negatives).
fn projected(angle: u16, length: u16) -> i16 {
    let half = angle % HALF_TURN;
    let index = half.min(HALF_TURN.saturating_sub(half));
    let magnitude = i32::from(QUARTER_WAVE.get(usize::from(index)).copied().unwrap_or(0));
    let sine = if angle % PIXEL < HALF_TURN {
        magnitude
    } else {
        magnitude.saturating_neg()
    };
    let offset = sine
        .saturating_mul(i32::from(length))
        .div_euclid(i32::from(PIXEL));
    i16::try_from(offset).unwrap_or(0)
}
