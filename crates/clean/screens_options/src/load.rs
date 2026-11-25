//! Spec: `OptionsLoad`, `OptionsBrighten`. Black while the menu loads, then the menu fading in.

use types::options::{LoadStep, OptionsItem, OptionsStage};
use types::power_on::ScreenLight;

use crate::fade::{self, Fade};

/// Spec: `OptionsLoad`. The load takes two finished frames; then the menu brightens with the
/// cursor on start game.
pub const fn load(step: LoadStep) -> OptionsStage {
    match step {
        LoadStep::First => OptionsStage::Loading(LoadStep::Second),
        LoadStep::Second => OptionsStage::Brightening(OptionsItem::StartGame),
    }
}

/// Spec: `OptionsBrighten`. One level brighter per finished frame; at full the player chooses.
pub fn brighten(screen: &mut ScreenLight, cursor: OptionsItem) -> OptionsStage {
    match fade::brighten(screen) {
        Fade::Partway => OptionsStage::Brightening(cursor),
        Fade::Ended => OptionsStage::Choosing(cursor),
    }
}
