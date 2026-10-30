//! Small gameplay tables through the imported `sm_data` records.
//!
//! Each accessor serves the record while the loaded image is the imported
//! allocation; any other image, and indices past a table, keep the original
//! indexed load with its failures.
use crate::imported::ImportedWords;
use crate::records::Records;
use sm_data::cartridge::RomImage;
use sm_data::gameplay_tables::*;

#[derive(Default)]
pub struct ImportedGameplayTables {
    area_labels: ImportedWords<AREA_LABELS, AREAS>,
    torizo_low: ImportedWords<TORIZO_LOW_PALETTE_BANDS, TORIZO_PALETTE_WORDS>,
    torizo_high: ImportedWords<TORIZO_HIGH_PALETTE_BANDS, TORIZO_PALETTE_WORDS>,
    death: ImportedWords<DEATH_PALETTES, SUITS>,
    death_flash: ImportedWords<DEATH_FLASH_PALETTES, SUITS>,
    death_frames: ImportedWords<DEATH_FRAME_PALETTES, DEATH_FRAMES>,
    sine: ImportedWords<ENEMY_SINE, ANGLES>,
    shutter_timers: ImportedWords<SHUTTER_TIMERS, SHUTTER_TIMER_WORDS>,
    spore_spawn: ImportedWords<SPORE_SPAWN_PALETTES, SPORE_SPAWN_PALETTE_WORDS>,
    pirate: ImportedWords<PIRATE_INSTRUCTION_LISTS, PIRATE_CHOICES>,
    sky_wave: ImportedWords<SKY_SCROLL_WAVE, SKY_SCROLL_WAVE_WORDS>,
    lava_horizontal: ImportedWords<LAVA_HORIZONTAL_WAVE, LAVA_WAVE_SAMPLES>,
    lava_vertical: ImportedWords<LAVA_VERTICAL_WAVE, LAVA_WAVE_SAMPLES>,
    grapple_point_tiles: ImportedWords<GRAPPLE_POINT_TILES, GRAPPLE_POINT_ANGLES>,
}

pub fn import(image: &RomImage) -> ImportedGameplayTables {
    ImportedGameplayTables {
        area_labels: ImportedWords::import(image),
        torizo_low: ImportedWords::import(image),
        torizo_high: ImportedWords::import(image),
        death: ImportedWords::import(image),
        death_flash: ImportedWords::import(image),
        death_frames: ImportedWords::import(image),
        sine: ImportedWords::import(image),
        shutter_timers: ImportedWords::import(image),
        spore_spawn: ImportedWords::import(image),
        pirate: ImportedWords::import(image),
        sky_wave: ImportedWords::import(image),
        lava_horizontal: ImportedWords::import(image),
        lava_vertical: ImportedWords::import(image),
        grapple_point_tiles: ImportedWords::import(image),
    }
}

fn tables(game: &(impl Records + ?Sized)) -> &ImportedGameplayTables {
    &game.records().gameplay_tables
}

/// $82:965F: the area label pointer for `area`.
pub fn area_label(game: &(impl Records + ?Sized), area: isize) -> u16 {
    tables(game).area_labels.word(game.loaded_image(), area)
}

/// $84:8032 (`high` false) or $84:8132: one Torizo palette band colour.
pub fn torizo_palette_colour(game: &(impl Records + ?Sized), high: bool, index: isize) -> u16 {
    let tables = tables(game);
    if high {
        tables.torizo_high.word(game.loaded_image(), index)
    } else {
        tables.torizo_low.word(game.loaded_image(), index)
    }
}

/// $9B:B6D2 (`flashing`) or $9B:B5C8: the death palette pointer for a suit.
pub fn death_palette(game: &(impl Records + ?Sized), flashing: bool, suit: isize) -> u16 {
    let tables = tables(game);
    if flashing {
        tables.death_flash.word(game.loaded_image(), suit)
    } else {
        tables.death.word(game.loaded_image(), suit)
    }
}

/// $9B:B80F: the death sequence's second palette for a frame.
pub fn death_frame_palette(game: &(impl Records + ?Sized), frame: isize) -> u16 {
    tables(game).death_frames.word(game.loaded_image(), frame)
}

/// $A0:B1C3: the enemy sine for an 8-bit angle.
pub fn enemy_sine(game: &(impl Records + ?Sized), angle: isize) -> u16 {
    tables(game).sine.word(game.loaded_image(), angle)
}

/// $A2:EA56: one timed shutter timer word.
pub fn shutter_timer(game: &(impl Records + ?Sized), index: isize) -> u16 {
    tables(game).shutter_timers.word(game.loaded_image(), index)
}

/// $A5:E379: one Spore Spawn health palette colour.
pub fn spore_spawn_colour(game: &(impl Records + ?Sized), index: isize) -> u16 {
    tables(game).spore_spawn.word(game.loaded_image(), index)
}

/// $B2:F959: the space pirate instruction list for a choice.
pub fn pirate_instruction_list(game: &(impl Records + ?Sized), choice: isize) -> u16 {
    tables(game).pirate.word(game.loaded_image(), choice)
}

/// $88:A938: one scrolling sky wave offset.
pub fn sky_scroll_wave(game: &(impl Records + ?Sized), row: isize) -> u16 {
    tables(game).sky_wave.word(game.loaded_image(), row)
}

/// $88:B60A (`vertical`) or $88:B589: one lava wave offset.
pub fn lava_wave(game: &(impl Records + ?Sized), vertical: bool, sample: isize) -> u16 {
    let tables = tables(game);
    if vertical {
        tables.lava_vertical.word(game.loaded_image(), sample)
    } else {
        tables.lava_horizontal.word(game.loaded_image(), sample)
    }
}

/// $9B:C346: the grapple point's tile source for a beam angle.
pub fn grapple_point_tile(game: &(impl Records + ?Sized), angle: isize) -> u16 {
    tables(game)
        .grapple_point_tiles
        .word(game.loaded_image(), angle)
}
