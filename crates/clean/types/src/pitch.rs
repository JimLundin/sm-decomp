//! Spec: MusicalProgress and ConnectedPitchMotion. Separate admitted pitch domains.
use serde::{Deserialize, Serialize};
const MUSIC_MIN: u8 = 19;
const MUSIC_MAX: u8 = 51;
const EFFECT_MAX: u8 = 71;
/// Semitone in the admitted non-noise musical range.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MusicalPitch(u8);
impl MusicalPitch {
    /// Refuse pitches outside the admitted musical range.
    #[must_use]
    pub const fn new(value: u8) -> Option<Self> {
        if value >= MUSIC_MIN && value <= MUSIC_MAX {
            Some(Self(value))
        } else {
            None
        }
    }
    /// Semitone quantity, not a loudness or a turn count.
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }
}
/// Semitone in the admitted effect range.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EffectPitch(u8);
impl EffectPitch {
    /// Refuse pitches above the admitted effect bound.
    #[must_use]
    pub const fn new(value: u8) -> Option<Self> {
        if value <= EFFECT_MAX {
            Some(Self(value))
        } else {
            None
        }
    }
    /// Effect semitone, not a musical duration.
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }
    /// Zero semitones before a first span is consumed.
    pub const PRENOTE: Self = Self(0);
    /// Spec: `ConnectedPitchMotion`. Admitted slide carries stay within their target.
    ///
    /// As a public operation, arbitrary carries saturate the u8 sum and clamp to 71;
    /// that clamping is a representation safeguard, not an additional slide rule.
    #[must_use]
    pub fn carry(self, value: u8) -> Self {
        Self(self.0.saturating_add(value).min(EFFECT_MAX))
    }
}
