//! The baby Metroid's bank $AD palette tables through the imported `sm_data` records.
//!
//! Each accessor serves the record while the loaded image is the imported
//! allocation; any other image, and indices outside a table, keep the original
//! indexed loads with their failures.
#![forbid(unsafe_code)]
use crate::imported::{Imported, load, rom, row};
use crate::records::Records;
use sm_data::baby_metroid::*;
use sm_data::cartridge::RomImage;

#[derive(Default)]
pub struct ImportedBabyMetroid {
    shell_palettes: Option<Imported<HealthPalettes>>,
    innards_palettes: Option<Imported<HealthPalettes>>,
    fade_palettes: Option<Imported<FadePalettes>>,
}

pub fn import(image: &RomImage) -> ImportedBabyMetroid {
    ImportedBabyMetroid {
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
pub fn shell_palette(game: &(impl Records + ?Sized), index: isize) -> Palette {
    let rom = rom(game);
    row(
        &game.records().baby_metroid_tables.shell_palettes,
        rom,
        index,
    )
    .unwrap_or_else(|| Palette {
        colors: load(rom, SHELL_PALETTE_ADDRESS, index),
    })
}

/// Health-based innards palette `index` at $AD:E882.
pub fn innards_palette(game: &(impl Records + ?Sized), index: isize) -> Palette {
    let rom = rom(game);
    row(
        &game.records().baby_metroid_tables.innards_palettes,
        rom,
        index,
    )
    .unwrap_or_else(|| Palette {
        colors: load(rom, INNARDS_PALETTE_ADDRESS, index),
    })
}

/// Fade-to-black palette `index` at $AD:E8E2.
pub fn fade_palette(game: &(impl Records + ?Sized), index: isize) -> Palette {
    let rom = rom(game);
    row(
        &game.records().baby_metroid_tables.fade_palettes,
        rom,
        index,
    )
    .unwrap_or_else(|| Palette {
        colors: load(rom, FADE_PALETTE_ADDRESS, index),
    })
}
