//! The encounter's adjustable numbers. The reference profile holds the spec's vanilla values; a
//! different `Rules` is an adjusted game.

/// Spec: `BrainEndures`. The reference contact line, room X 84.
const REFERENCE_CONTACT_LINE: u8 = 84;
/// Spec: `BrainEndures`. The reference hit threshold, 4 impacts.
const REFERENCE_HITS: u8 = 4;
/// Spec: `BackgroundBobs`. The reference shake travel, 4 px per beat.
const REFERENCE_TRAVEL: u8 = 4;
/// Spec: `BackgroundBobs`. The longest shake travel, 16 px per beat.
const LONGEST_TRAVEL: u8 = 16;
/// Spec: `ReturnToPage`. The reference shake hold, 128 beats.
const REFERENCE_SHAKE_HOLD: u16 = 128;

/// Spec: `BrainEndures`. The contact line: a whole room X in pixels, 0 to 255. A missile whose
/// centre is strictly left of it hits Mother Brain.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ContactLine(pub u8);

/// Spec: `BrainEndures`. The impacts needed to start the bursts: 1 to 255.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HitThreshold(u8);

impl HitThreshold {
    /// The threshold of this many impacts, if it is 1 to 255.
    #[must_use]
    pub const fn new(hits: u8) -> Option<Self> {
        if hits >= 1 { Some(Self(hits)) } else { None }
    }

    /// Its impacts.
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }
}

/// Spec: `BackgroundBobs`. The background's vertical travel per shaking beat: 0 to 16 whole
/// pixels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShakeTravel(u8);

impl ShakeTravel {
    /// The travel of this many pixels, if it is 0 to 16.
    #[must_use]
    pub const fn new(pixels: u8) -> Option<Self> {
        if pixels <= LONGEST_TRAVEL {
            Some(Self(pixels))
        } else {
            None
        }
    }

    /// Its pixels.
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }
}

/// Spec: `ReturnToPage`. The encounter beats completed after the decisive impact before page 2
/// is cued: 1 to 65,535.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShakeHold(u16);

impl ShakeHold {
    /// The hold of this many beats, if it is 1 to 65,535.
    #[must_use]
    pub const fn new(beats: u16) -> Option<Self> {
        if beats >= 1 { Some(Self(beats)) } else { None }
    }

    /// Its beats.
    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }
}

/// Spec: `BrainEndures`. When a missile hits Mother Brain, and how many hits defeat her.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ContactRules {
    /// The contact line.
    pub limit: ContactLine,
    /// The impacts that start the bursts.
    pub hits: HitThreshold,
}

/// The encounter's adjustable numbers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rules {
    /// Spec: `BrainEndures`. Missile contact.
    pub contact: ContactRules,
    /// Spec: `BackgroundBobs`. The background's travel per shaking beat.
    pub shake_travel: ShakeTravel,
    /// Spec: `ReturnToPage`. The shaking beats before page 2 is cued.
    pub shake_hold: ShakeHold,
}

impl Rules {
    /// The reference game: contact line 84, 4 hits, 4 px of shake travel, 128 shaking beats.
    pub const REFERENCE: Self = Self {
        contact: ContactRules {
            limit: ContactLine(REFERENCE_CONTACT_LINE),
            hits: HitThreshold(REFERENCE_HITS),
        },
        shake_travel: ShakeTravel(REFERENCE_TRAVEL),
        shake_hold: ShakeHold(REFERENCE_SHAKE_HOLD),
    };
}
