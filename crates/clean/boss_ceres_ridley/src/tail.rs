//! His tail from the moment he starts moving until he takes off: it hangs at rest behind him,
//! its curl changes now and then at random, and once he is tangible its tip hits her when her
//! body comes within reach.

use core::num::Wrapping;

use types::{Hit, Pair, PushFrames, Px, Radius, Side};

/// `TailAtRest`: the tail's place is 32 px right of him (he faces left).
const TIP_RIGHT: Px = Wrapping(32);
/// `TailAtRest`: and 16 px down.
const TIP_DOWN: Px = Wrapping(16);
/// `TailCurls`: the curl as he is placed.
const CURL_PLACED: u8 = 16;
/// `TailCurls`: the least curl.
const CURL_LEAST: u8 = 8;
/// `TailCurls`: the most curl.
const CURL_MOST: u8 = 23;
/// `TailCurls`: the curl changes at a random number of 65,280 or more.
const CURL_CHANGES_FROM: u16 = 65_280;
/// `TailCurls`: the new curl is 8 + the random number mod 16.
const CURL_SPREAD: u16 = 16;
/// `TailHits`: the push time of a hit, in frames.
const PUSH_FRAMES: u8 = 5;
/// `TailHits`: the push time of a hit, 5 frames, within the longest push the shared type allows
/// (checked as the crate compiles).
const PUSH: PushFrames = match PushFrames::new(PUSH_FRAMES) {
    Some(push) => push,
    None => panic!("the tail's push is within the longest push"),
};

/// How tightly his tail curls once it moves (his next part): 8 to 23.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TailCurl(u8);

impl TailCurl {
    /// `TailCurls`: the curl as he is placed, 16.
    pub const PLACED: Self = Self(CURL_PLACED);

    /// This curl, or none outside 8 to 23.
    #[must_use]
    pub const fn new(curl: u8) -> Option<Self> {
        if curl >= CURL_LEAST && curl <= CURL_MOST {
            Some(Self(curl))
        } else {
            None
        }
    }

    /// The curl's count.
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }

    /// `TailCurls`: a run with his tail at rest, reading the random number `n` the baby reads:
    /// at 65,280 or more the curl turns `8 + (n mod 16)`; under it, the curl stays.
    pub(crate) fn curled(self, random: u16) -> Self {
        if random < CURL_CHANGES_FROM {
            return self;
        }
        u8::try_from(random % CURL_SPREAD)
            .ok()
            .and_then(|spread| CURL_LEAST.checked_add(spread))
            .map_or(self, Self)
    }
}

/// What his tail reads of her on a run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Her {
    /// Her position, in whole pixels, as her moves this frame leave it, before her reaction to
    /// a hit (samus-movement).
    pub position: Pair<Px>,
    /// The half-width and half-height of the body she started the frame in, from her
    /// movement's content.
    pub body: Radius,
    /// Her protection as the frame starts, in frames (samus-health).
    pub protection: u16,
}

impl Her {
    /// `TailHits`: whether her protection had run out as the frame started.
    const fn unprotected(&self) -> bool {
        self.protection == 0
    }

    /// `TailHits`: the frame's hit of a tail whose tip is at `tip`, with the tail's reach `t`:
    /// her protection run out, and on each axis the gap from the tip to her, read signed, under
    /// her half-size plus `t`. Each push carries her for 5 frames; she goes `Right` when her x is
    /// at the tip's or right of it, else `Left`.
    pub(crate) fn hit_by(&self, tip: Pair<Px>, reach: u8) -> Option<Hit> {
        let gap = Pair {
            x: (self.position.x - tip.x).0.cast_signed(),
            y: (self.position.y - tip.y).0.cast_signed(),
        };
        let in_reach = within(gap.x, self.body.x, reach) && within(gap.y, self.body.y, reach);
        (self.unprotected() && in_reach).then_some(Hit {
            side: if gap.x >= 0 { Side::Right } else { Side::Left },
            push: PUSH,
        })
    }
}

/// `TailHits`: whether a signed gap on one axis is under her half-size there plus the reach.
fn within(gap: i16, half: Px, reach: u8) -> bool {
    u32::from(gap.unsigned_abs()) < u32::from(half.0).saturating_add(u32::from(reach))
}

/// `TailAtRest`: the tail's place while it rests, the tip and all seven of its segments at one
/// place: 32 px right of his position and 16 px down, each mod 2^16.
pub fn place_at_rest(ridley: Pair<Px>) -> Pair<Px> {
    Pair {
        x: ridley.x + TIP_RIGHT,
        y: ridley.y + TIP_DOWN,
    }
}
