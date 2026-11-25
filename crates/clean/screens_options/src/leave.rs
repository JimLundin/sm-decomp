//! Spec: `OptionsLeave`. Leaving for the game or for file select, the menu darkens.

use types::options::{FileBegins, OptionsStage, WayOut};
use types::power_on::ScreenLight;

use crate::fade::{self, Fade};

/// Spec: `OptionsLeave`. One level darker per finished frame, staying in `stage`; at dark the
/// menu hands on by `way_out`.
pub fn darken(screen: &mut ScreenLight, stage: OptionsStage, way_out: WayOut) -> OptionsStage {
    match fade::darken(screen) {
        Fade::Partway => stage,
        Fade::Ended => OptionsStage::HandedOn(way_out),
    }
}

/// Spec: `OptionsLeave`. The way into the game, by how the chosen file begins.
pub const fn into_game(begins: FileBegins) -> WayOut {
    match begins {
        FileBegins::Intro => WayOut::IntroBegins,
        FileBegins::Escape => WayOut::EscapeBegins,
    }
}
