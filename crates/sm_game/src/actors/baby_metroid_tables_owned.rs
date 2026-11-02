//! Actual lazy prepared table acquisitions; immutable loaded allocation is retained.
#![forbid(unsafe_code)]
use crate::actors::owned_execution::PreparedActorState;
use sm_data::baby_metroid::*;
pub fn shell_palette(game: &PreparedActorState<'_>, index: isize) -> Palette {
    Palette {
        colors: game.loaded_table_word(SHELL_PALETTE_ADDRESS, index),
    }
}
pub fn innards_palette(game: &PreparedActorState<'_>, index: isize) -> Palette {
    Palette {
        colors: game.loaded_table_word(INNARDS_PALETTE_ADDRESS, index),
    }
}
pub fn fade_palette(game: &PreparedActorState<'_>, index: isize) -> Palette {
    Palette {
        colors: game.loaded_table_word(FADE_PALETTE_ADDRESS, index),
    }
}
