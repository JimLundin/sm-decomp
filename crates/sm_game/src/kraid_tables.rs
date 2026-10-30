//! Kraid's bank $A7 tables through the imported `sm_data` records.
//!
//! Each accessor serves the record while the loaded image is the imported
//! allocation; any other image, and rows past a table, keep the original
//! indexed loads with their failures.
use crate::imported::Imported;
use crate::records::Records;
use sm_data::cartridge::{RomImage, lorom_offset};
use sm_data::kraid::{
    BG_PALETTE_ADDRESS, BG_PALETTE_BYTES, BODY_HITBOX_ADDRESS, BODY_HITBOX_BYTES, BgPalettes,
    BodyHitbox, BodyHitboxBand, FINGERNAIL_BOUNDARY_ADDRESS, FINGERNAIL_BOUNDARY_BYTES,
    FOOT_TARGET_ADDRESS, FOOT_TARGET_BYTES, FingernailBoundaries, FingernailBoundary, FootTarget,
    FootTargets, SPRITE_PALETTE_ADDRESS, SPRITE_PALETTE_BYTES, SpritePalettes, import_bg_palettes,
    import_body_hitbox, import_fingernail_boundaries, import_foot_targets, import_sprite_palettes,
};

#[derive(Default)]
pub struct ImportedKraid {
    body_hitbox: Option<Imported<BodyHitbox>>,
    fingernail_boundaries: Option<Imported<FingernailBoundaries>>,
    foot_targets: Option<Imported<FootTargets>>,
    bg_palettes: Option<Imported<BgPalettes>>,
    sprite_palettes: Option<Imported<SpritePalettes>>,
}

pub fn import(image: &RomImage) -> ImportedKraid {
    ImportedKraid {
        body_hitbox: Imported::import(
            image,
            BODY_HITBOX_ADDRESS,
            BODY_HITBOX_BYTES,
            import_body_hitbox,
        ),
        fingernail_boundaries: Imported::import(
            image,
            FINGERNAIL_BOUNDARY_ADDRESS,
            FINGERNAIL_BOUNDARY_BYTES,
            import_fingernail_boundaries,
        ),
        foot_targets: Imported::import(
            image,
            FOOT_TARGET_ADDRESS,
            FOOT_TARGET_BYTES,
            import_foot_targets,
        ),
        bg_palettes: Imported::import(
            image,
            BG_PALETTE_ADDRESS,
            BG_PALETTE_BYTES,
            import_bg_palettes,
        ),
        sprite_palettes: Imported::import(
            image,
            SPRITE_PALETTE_ADDRESS,
            SPRITE_PALETTE_BYTES,
            import_sprite_palettes,
        ),
    }
}

fn rom(game: &(impl Records + ?Sized)) -> &RomImage {
    game.loaded_image()
}

/// The original load: the word at word `index` from `address`.
fn load(rom: &RomImage, address: u32, index: usize) -> u16 {
    rom.table_word(lorom_offset(address) as usize, index as isize)
}

/// Band `row` of the body hitbox at $A7:B161.
pub fn body_hitbox(game: &(impl Records + ?Sized), row: usize) -> BodyHitboxBand {
    let rom = rom(game);
    Imported::from(&game.records().kraid_tables.body_hitbox, rom)
        .and_then(|table| table.row(row))
        .unwrap_or_else(|| BodyHitboxBand {
            bottom: load(rom, BODY_HITBOX_ADDRESS, 2 * row),
            left: load(rom, BODY_HITBOX_ADDRESS, 2 * row + 1),
        })
}

/// Step `row` of the fingernail boundary at $A7:BF1D.
pub fn fingernail_boundary(game: &(impl Records + ?Sized), row: usize) -> FingernailBoundary {
    let rom = rom(game);
    Imported::from(&game.records().kraid_tables.fingernail_boundaries, rom)
        .and_then(|table| table.row(row))
        .unwrap_or_else(|| FingernailBoundary {
            left: load(rom, FINGERNAIL_BOUNDARY_ADDRESS, 2 * row),
            top: load(rom, FINGERNAIL_BOUNDARY_ADDRESS, 2 * row + 1),
        })
}

/// Foot target `row` at $A7:BA7D.
pub fn foot_target(game: &(impl Records + ?Sized), row: usize) -> FootTarget {
    let rom = rom(game);
    Imported::from(&game.records().kraid_tables.foot_targets, rom)
        .and_then(|table| table.row(row))
        .unwrap_or_else(|| FootTarget {
            x_position: load(rom, FOOT_TARGET_ADDRESS, 2 * row),
            movement: load(rom, FOOT_TARGET_ADDRESS, 2 * row + 1),
        })
}

/// The colour at word `index` from BG palette 7's rows at $A7:B3D3.
pub fn bg_color(game: &(impl Records + ?Sized), index: usize) -> u16 {
    let rom = rom(game);
    Imported::from(&game.records().kraid_tables.bg_palettes, rom)
        .and_then(|table| table.row(index / 16))
        .map(|palette| palette.colors[index % 16])
        .unwrap_or_else(|| load(rom, BG_PALETTE_ADDRESS, index))
}

/// The colour at word `index` from sprite palette 7's rows at $A7:B513.
pub fn sprite_color(game: &(impl Records + ?Sized), index: usize) -> u16 {
    let rom = rom(game);
    Imported::from(&game.records().kraid_tables.sprite_palettes, rom)
        .and_then(|table| table.row(index / 16))
        .map(|palette| palette.colors[index % 16])
        .unwrap_or_else(|| load(rom, SPRITE_PALETTE_ADDRESS, index))
}
