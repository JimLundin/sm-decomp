//! Spec: MusicalProgress and ConnectedPitchMotion. Separate admitted pitch domains.
use serde::{Deserialize, Serialize};
const MUSIC_MIN: u8 = 19;
const MUSIC_MAX: u8 = 51;
const EFFECT_MAX: u8 = 71;
// Spec: Saved quantities. Keep each saved newtype's name and integer representation,
// but pass the restored semitone through the same check as a newly constructed pitch.
#[derive(Deserialize)]
#[serde(rename = "MusicalPitch")]
struct SavedMusicalPitch(u8);

/// Semitone in the admitted non-noise musical range.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "SavedMusicalPitch")]
pub struct MusicalPitch(u8);

impl TryFrom<SavedMusicalPitch> for MusicalPitch {
    type Error = &'static str;

    fn try_from(saved: SavedMusicalPitch) -> Result<Self, Self::Error> {
        Self::new(saved.0).ok_or("musical pitch must be in 19..=51 semitones")
    }
}

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
#[derive(Deserialize)]
#[serde(rename = "EffectPitch")]
struct SavedEffectPitch(u8);

/// Semitone in the admitted effect range.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "SavedEffectPitch")]
pub struct EffectPitch(u8);

impl TryFrom<SavedEffectPitch> for EffectPitch {
    type Error = &'static str;

    fn try_from(saved: SavedEffectPitch) -> Result<Self, Self::Error> {
        Self::new(saved.0).ok_or("effect pitch must be in 0..=71 semitones")
    }
}

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
