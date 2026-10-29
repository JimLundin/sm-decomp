//! The Shitroid's bank $AD palette tables through the imported `sm_data` records.
//!
//! Each accessor serves the record while the loaded image is the imported
//! allocation; any other image, and indices outside a table, keep the original
//! indexed loads with their failures.
#![forbid(unsafe_code)]
use crate::game::imported::{Imported, load, rom, row};
use crate::game_state::Game;
use crate::rom::RomImage;
use sm_data::shitroid::*;

#[derive(Default)]
pub(crate) struct ImportedShitroid {
    shell_palettes: Option<Imported<HealthPalettes>>,
    innards_palettes: Option<Imported<HealthPalettes>>,
    fade_palettes: Option<Imported<FadePalettes>>,
}

pub(crate) fn import(image: &RomImage) -> ImportedShitroid {
    ImportedShitroid {
        shell_palettes: Imported::import(
            image,
            SHELL_PALETTE_ADDRESS,
            HEALTH_PALETTE_BYTES,
            import_health_palettes,
        ),
        innards_palettes: Imported::import(
            image,
            INNARDS_PALETTE_ADDRESS,
            HEALTH_PALETTE_BYTES,
            import_health_palettes,
        ),
        fade_palettes: Imported::import(
            image,
            FADE_PALETTE_ADDRESS,
            FADE_PALETTE_BYTES,
            import_fade_palettes,
        ),
    }
}

/// Health-based shell palette `index` at $AD:E7E2.
pub(crate) fn shell_palette(game: &Game, index: isize) -> Palette {
    let rom = rom(game);
    row(&game.shitroid_tables.shell_palettes, rom, index).unwrap_or_else(|| Palette {
        colors: load(rom, SHELL_PALETTE_ADDRESS, index),
    })
}

/// Health-based innards palette `index` at $AD:E882.
pub(crate) fn innards_palette(game: &Game, index: isize) -> Palette {
    let rom = rom(game);
    row(&game.shitroid_tables.innards_palettes, rom, index).unwrap_or_else(|| Palette {
        colors: load(rom, INNARDS_PALETTE_ADDRESS, index),
    })
}

/// Fade-to-black palette `index` at $AD:E8E2.
pub(crate) fn fade_palette(game: &Game, index: isize) -> Palette {
    let rom = rom(game);
    row(&game.shitroid_tables.fade_palettes, rom, index).unwrap_or_else(|| Palette {
        colors: load(rom, FADE_PALETTE_ADDRESS, index),
    })
}
