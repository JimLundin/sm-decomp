//! Bank $A7 Kraid, Phantoon and Dachora word tables through the imported
//! `sm_data` records.
//!
//! Each accessor serves the record while the loaded image is the imported
//! allocation; any other image, and indices past a table, keep the original
//! indexed load with its failures.
#![forbid(unsafe_code)]
use crate::imported::ImportedWords;
use crate::records::Records;
use sm_data::boss_lists::*;
use sm_data::cartridge::RomImage;

#[derive(Default)]
pub struct ImportedBossLists {
    kraid_rocks: ImportedWords<KRAID_ROCK_X_POSITIONS, KRAID_ROCKS>,
    kraid_rock_spits: ImportedWords<KRAID_ROCK_SPIT_X_VELOCITIES, KRAID_ROCK_SPITS>,
    kraid_nails_up: ImportedWords<KRAID_NAIL_UPWARD_VELOCITIES, KRAID_NAIL_CHOICES>,
    kraid_nails_down: ImportedWords<KRAID_NAIL_DOWNWARD_VELOCITIES, KRAID_NAIL_CHOICES>,
    fade_out_target: ImportedWords<FADE_OUT_TARGET_PALETTE, FADE_OUT_TARGET_COLOURS>,
    wrecked_ship_powered_on:
        ImportedWords<WRECKED_SHIP_POWERED_ON_PALETTES, WRECKED_SHIP_POWERED_ON_COLOURS>,
    phantoon_sounds: ImportedWords<PHANTOON_MATERIALISATION_SOUNDS, PHANTOON_MATERIALISATION_STEPS>,
    phantoon_flames: ImportedWords<PHANTOON_FLAME_INSTRUCTION_LISTS, PHANTOON_FLAME_KINDS>,
    phantoon_lists: ImportedWords<PHANTOON_INSTRUCTION_LISTS, PHANTOON_PATTERNS>,
    dachora_charge: ImportedWords<DACHORA_CHARGE_PALETTES, DACHORA_PALETTE_STEPS>,
    dachora_shine: ImportedWords<DACHORA_SHINE_PALETTES, DACHORA_PALETTE_STEPS>,
}

pub fn import(image: &RomImage) -> ImportedBossLists {
    ImportedBossLists {
        kraid_rocks: ImportedWords::import(image),
        kraid_rock_spits: ImportedWords::import(image),
        kraid_nails_up: ImportedWords::import(image),
        kraid_nails_down: ImportedWords::import(image),
        fade_out_target: ImportedWords::import(image),
        wrecked_ship_powered_on: ImportedWords::import(image),
        phantoon_sounds: ImportedWords::import(image),
        phantoon_flames: ImportedWords::import(image),
        phantoon_lists: ImportedWords::import(image),
        dachora_charge: ImportedWords::import(image),
        dachora_shine: ImportedWords::import(image),
    }
}

fn lists(game: &(impl Records + ?Sized)) -> &ImportedBossLists {
    &game.records().boss_lists
}

/// $A7:ACB3: a falling rock's X position.
pub fn kraid_rock_x(game: &(impl Records + ?Sized), rock: isize) -> u16 {
    lists(game).kraid_rocks.word(game.loaded_image(), rock)
}

/// $A7:BC65: a rock spit X velocity.
pub fn kraid_rock_spit_x_velocity(game: &(impl Records + ?Sized), choice: isize) -> u16 {
    lists(game)
        .kraid_rock_spits
        .word(game.loaded_image(), choice)
}

/// $A7:BE3E (`upward`) or $A7:BE46: a fingernail velocity record address.
pub fn kraid_nail_velocity(game: &(impl Records + ?Sized), upward: bool, choice: isize) -> u16 {
    let lists = lists(game);
    if upward {
        lists.kraid_nails_up.word(game.loaded_image(), choice)
    } else {
        lists.kraid_nails_down.word(game.loaded_image(), choice)
    }
}

/// $A7:CA41: one fade-out target colour.
pub fn fade_out_target_colour(game: &(impl Records + ?Sized), index: isize) -> u16 {
    lists(game).fade_out_target.word(game.loaded_image(), index)
}

/// $A7:CA61: one Wrecked Ship powered-on target colour.
pub fn wrecked_ship_powered_on_colour(game: &(impl Records + ?Sized), index: isize) -> u16 {
    lists(game)
        .wrecked_ship_powered_on
        .word(game.loaded_image(), index)
}

/// $A7:CDED: the materialisation sound effect for a step.
pub fn phantoon_materialisation_sound(game: &(impl Records + ?Sized), step: isize) -> u16 {
    lists(game).phantoon_sounds.word(game.loaded_image(), step)
}

/// $A7:CE8E: a flame's instruction list.
pub fn phantoon_flame_instruction_list(game: &(impl Records + ?Sized), kind: isize) -> u16 {
    lists(game).phantoon_flames.word(game.loaded_image(), kind)
}

/// $A7:D40D: Phantoon's body instruction list for a pattern.
pub fn phantoon_instruction_list(game: &(impl Records + ?Sized), pattern: isize) -> u16 {
    lists(game)
        .phantoon_lists
        .word(game.loaded_image(), pattern)
}

/// $A7:F787 (`shining` false) or $A7:F92D: a Dachora palette address.
pub fn dachora_palette(game: &(impl Records + ?Sized), shining: bool, step: isize) -> u16 {
    let lists = lists(game);
    if shining {
        lists.dachora_shine.word(game.loaded_image(), step)
    } else {
        lists.dachora_charge.word(game.loaded_image(), step)
    }
}
