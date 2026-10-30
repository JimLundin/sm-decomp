//! The dead monsters' bank $A9 tables through the imported `sm_data` records.
//!
//! Each accessor serves the record while the loaded image is the imported
//! allocation; any other image, and indices outside a table, keep the original
//! indexed loads with their failures.
#![forbid(unsafe_code)]
use crate::imported::{Imported, load, rom, row};
use crate::records::Records;
use sm_data::cartridge::RomImage;
use sm_data::corpses::*;

#[derive(Default)]
pub struct ImportedCorpses {
    hops: Option<Imported<Hops>>,
    zoomer: Option<Imported<Variants<ZOOMER_ROWS>>>,
    ripper: Option<Imported<Variants<RIPPER_ROWS>>>,
    skree: Option<Imported<Variants<SKREE_ROWS>>>,
}

pub fn import(image: &RomImage) -> ImportedCorpses {
    ImportedCorpses {
        hops: Imported::import(image, HOP_ADDRESS, HOP_BYTES, import_hops),
        zoomer: Imported::import(image, ZOOMER_ADDRESS, ZOOMER_ROWS * 4, import_variants),
        ripper: Imported::import(image, RIPPER_ADDRESS, RIPPER_ROWS * 4, import_variants),
        skree: Imported::import(image, SKREE_ADDRESS, SKREE_ROWS * 4, import_variants),
    }
}

/// Sidehopper corpse hop `index` at $A9:D951/$A9:D959.
pub fn hop(game: &(impl Records + ?Sized), index: isize) -> Hop {
    let rom = rom(game);
    row(&game.records().corpse_tables.hops, rom, index).unwrap_or_else(|| Hop {
        y_velocity: load(rom, HOP_ADDRESS, index),
        x_velocity: load(rom, HOP_X_VELOCITY_ADDRESS, index),
    })
}

/// A corpse whose variants the tables above choose between.
#[derive(Clone, Copy)]
pub enum Corpse {
    Zoomer,
    Ripper,
    Skree,
}

/// Variant `index` of `corpse`.
pub fn variant(game: &(impl Records + ?Sized), corpse: Corpse, index: isize) -> CorpseVariant {
    let rom = rom(game);
    let tables = &game.records().corpse_tables;
    let (variant, address, rotting) = match corpse {
        Corpse::Zoomer => (
            row(&tables.zoomer, rom, index),
            ZOOMER_ADDRESS,
            ZOOMER_ROTTING_ADDRESS,
        ),
        Corpse::Ripper => (
            row(&tables.ripper, rom, index),
            RIPPER_ADDRESS,
            RIPPER_ROTTING_ADDRESS,
        ),
        Corpse::Skree => (
            row(&tables.skree, rom, index),
            SKREE_ADDRESS,
            SKREE_ROTTING_ADDRESS,
        ),
    };
    variant.unwrap_or_else(|| CorpseVariant {
        instruction_list: load(rom, address, index),
        rotting: load(rom, rotting, index),
    })
}
