//! The logo's fade. Spec: `LogoBrightens` and `LogoDims` step the logo's level the same way: by
//! the fade step on each finished frame, lit at each level short of the end it heads for, and at
//! that end the fade is over.

use types::power_on::{FinishedFrames, Level, PowerOnStage, PowerOnState, ScreenLight};

use crate::rules::{FadeStep, Rules};

/// Which way the logo fades.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fade {
    /// Spec: `LogoBrightens`. Toward full; at full it is held.
    Brighten,
    /// Spec: `LogoDims`. Toward dark; at dark the screen blanks and the sound loads.
    Dim,
}

impl Fade {
    /// The State after one finished frame of this fade from `level`, with the frame already
    /// counted in `frames`.
    #[must_use]
    pub const fn step(self, rules: Rules, level: Level, frames: FinishedFrames) -> PowerOnState {
        match short_of_the_ends(self.stepped(level, rules.fade_step)) {
            Some(level) => PowerOnState {
                stage: self.under_way(),
                screen: ScreenLight::Lit(level),
                frames,
            },
            None => self.over(rules, frames),
        }
    }

    /// The level's number one step on: brightening adds the step, dimming takes it away, and a
    /// step past dark or full stops there.
    const fn stepped(self, level: Level, step: FadeStep) -> u8 {
        match self {
            Self::Brighten => level.get().saturating_add(step.get()),
            Self::Dim => level.get().saturating_sub(step.get()),
        }
    }

    /// The stage while the fade goes on.
    const fn under_way(self) -> PowerOnStage {
        match self {
            Self::Brighten => PowerOnStage::Brightening,
            Self::Dim => PowerOnStage::Dimming,
        }
    }

    /// The State once the fade reaches its end. Brightening: lit at full, held with the whole
    /// hold left. Dimming: blank at dark for this frame end, and the sound driver loads.
    const fn over(self, rules: Rules, frames: FinishedFrames) -> PowerOnState {
        match self {
            Self::Brighten => PowerOnState {
                stage: PowerOnStage::Held(rules.hold),
                screen: ScreenLight::Lit(Level::FULL),
                frames,
            },
            Self::Dim => PowerOnState {
                stage: PowerOnStage::SoundLoading,
                screen: ScreenLight::Blank(Level::DARK),
                frames,
            },
        }
    }
}

/// The level a step reaches while its fade goes on: strictly between dark and full. A step that
/// reaches dark or full, or would pass it, ends the fade.
const fn short_of_the_ends(stepped: u8) -> Option<Level> {
    if Level::DARK.get() < stepped && stepped < Level::FULL.get() {
        Level::new(stepped)
    } else {
        None
    }
}
