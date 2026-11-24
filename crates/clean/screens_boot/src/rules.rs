//! The numbers an adjusted game may change: the logo's fade step and its hold.

use types::power_on::HoldFrames;

/// Spec: `LogoBrightens`. The smallest fade step: 1 level per finished frame.
const SLOWEST_FADE_STEP: u8 = 1;
/// Spec: `LogoBrightens`. The largest fade step: 15 levels per finished frame, dark to full at once.
const FASTEST_FADE_STEP: u8 = 15;
/// Spec: `LogoBrightens`. The reference fade step: 1 level per finished frame.
const REFERENCE_FADE_STEP: FadeStep = FadeStep(1);
/// Spec: `LogoHeld`. The reference hold: 120 finished frames.
const REFERENCE_HOLD: HoldFrames = HoldFrames(120);

/// Spec: `LogoBrightens`, `LogoDims`. The levels the logo brightens or dims by on each finished
/// frame, 1 to 15.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FadeStep(u8);

impl FadeStep {
    /// The fade step of this many levels, if it is one (1 to 15).
    #[must_use]
    pub const fn new(levels: u8) -> Option<Self> {
        if SLOWEST_FADE_STEP <= levels && levels <= FASTEST_FADE_STEP {
            Some(Self(levels))
        } else {
            None
        }
    }

    /// Its levels per finished frame.
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }
}

/// The power-on's adjustable numbers. `REFERENCE` is the game as it ships; any other value is an
/// adjusted game.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rules {
    /// Spec: `LogoBrightens`, `LogoDims`. The levels per finished frame of both fades.
    pub fade_step: FadeStep,
    /// Spec: `LogoHeld`. The finished frames the logo stays at full before it dims.
    pub hold: HoldFrames,
}

impl Rules {
    /// The reference rules: a fade step of 1 and a hold of 120 frames.
    pub const REFERENCE: Self = Self {
        fade_step: REFERENCE_FADE_STEP,
        hold: REFERENCE_HOLD,
    };
}
