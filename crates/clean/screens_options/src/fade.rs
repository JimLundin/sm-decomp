//! The screen brightening or darkening one level per finished frame, as the menu fades in, as a
//! dissolve darkens and brightens, and as the menu fades out.

use types::power_on::{Level, ScreenLight};

/// Where a fade stands after its step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fade {
    /// Still on its way.
    Partway,
    /// At its end: full, or dark.
    Ended,
}

/// The level a fade starts its step from: a blank screen's is 0, whatever level it keeps.
const fn shown(screen: ScreenLight) -> Level {
    match screen {
        ScreenLight::Blank(_) => Level::DARK,
        ScreenLight::Lit(level) => level,
    }
}

/// Spec: `OptionsBrighten`, `Dissolve`. The screen lit one level brighter; the fade ends when it
/// reaches full.
pub fn brighten(screen: &mut ScreenLight) -> Fade {
    let level = shown(*screen).brighter();
    *screen = ScreenLight::Lit(level);
    if level == Level::FULL {
        Fade::Ended
    } else {
        Fade::Partway
    }
}

/// Spec: `Dissolve`, `OptionsLeave`. The screen lit one level darker; at 0 it is blank, and the
/// fade ends.
pub fn darken(screen: &mut ScreenLight) -> Fade {
    let level = shown(*screen).darker();
    if level == Level::DARK {
        *screen = ScreenLight::Blank(Level::DARK);
        Fade::Ended
    } else {
        *screen = ScreenLight::Lit(level);
        Fade::Partway
    }
}
