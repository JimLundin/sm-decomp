//! The cinematic, credits and HDMA tables visual code reads, imported from the loaded image.
use crate::imported::Imported;
use sm_data::credits::{
    BEAM_BIT_ADDRESS, BEAM_DESTINATION_ADDRESS, BEAM_FRAMES, BEAM_SOURCE_ADDRESS, BEAMS, BeamBits,
    BeamTransfers, CAPACITIES, CAPACITY_ADDRESS, CAPACITY_DIVISOR_ADDRESS, Capacities,
    DIGIT_TILE_ADDRESS, DIGITS, DigitTiles, ICON_PALETTE_ADDRESS, ICON_PALETTE_BYTES,
    ICON_TRANSFER_ADDRESS, ICON_TRANSFER_BYTES, ITEM_BIT_ADDRESS, ITEMS, IconPaletteSteps,
    IconTransfers, ItemBits,
};
use sm_data::hdma_objects::{BG3_X_VELOCITIES, BG3_X_VELOCITY_ADDRESS, Bg3XVelocities};
use sm_data::shooting_stars::{
    TILE_ADDRESS as STAR_TILE_ADDRESS, TILE_STEPS as STAR_TILE_STEPS, Tiles as StarTiles,
};
use sm_data::title_screen::{GRADIENT_ADDRESS, GRADIENTS, Gradients};

/// The cinematic and HDMA tables this module reads, imported from the loaded image.
#[derive(Default)]
pub struct ImportedTables {
    pub bg3_x_velocities: Option<Imported<Bg3XVelocities>>,
    pub title_gradients: Option<Imported<Gradients>>,
    pub star_tiles: Option<Imported<StarTiles>>,
    pub icon_palettes: Option<Imported<IconPaletteSteps>>,
    pub capacities: Option<Imported<Capacities>>,
    pub capacity_divisors: Option<Imported<Capacities>>,
    pub item_bits: Option<Imported<ItemBits>>,
    pub beam_bits: Option<Imported<BeamBits>>,
    pub digit_tiles: Option<Imported<DigitTiles>>,
    pub icon_transfers: Option<Imported<IconTransfers>>,
    pub beam_sources: Option<Imported<BeamTransfers>>,
    pub beam_destinations: Option<Imported<BeamTransfers>>,
}

pub fn import(image: &sm_data::cartridge::RomImage) -> ImportedTables {
    use sm_data::credits::{
        import_digit_tiles, import_icon_palettes, import_icon_transfers, import_words,
    };
    ImportedTables {
        bg3_x_velocities: Imported::import(
            image,
            BG3_X_VELOCITY_ADDRESS,
            2 * BG3_X_VELOCITIES,
            sm_data::hdma_objects::import_bg3_x_velocities,
        ),
        title_gradients: Imported::import(
            image,
            GRADIENT_ADDRESS,
            2 * GRADIENTS,
            sm_data::title_screen::import_gradients,
        ),
        star_tiles: Imported::import(
            image,
            STAR_TILE_ADDRESS,
            2 * STAR_TILE_STEPS,
            sm_data::shooting_stars::import_tiles,
        ),
        icon_palettes: Imported::import(
            image,
            ICON_PALETTE_ADDRESS,
            ICON_PALETTE_BYTES,
            import_icon_palettes,
        ),
        capacities: Imported::import(image, CAPACITY_ADDRESS, 2 * CAPACITIES, import_words),
        capacity_divisors: Imported::import(
            image,
            CAPACITY_DIVISOR_ADDRESS,
            2 * CAPACITIES,
            import_words,
        ),
        item_bits: Imported::import(image, ITEM_BIT_ADDRESS, 2 * ITEMS, import_words),
        beam_bits: Imported::import(image, BEAM_BIT_ADDRESS, 2 * BEAMS, import_words),
        digit_tiles: Imported::import(image, DIGIT_TILE_ADDRESS, 4 * DIGITS, import_digit_tiles),
        icon_transfers: Imported::import(
            image,
            ICON_TRANSFER_ADDRESS,
            ICON_TRANSFER_BYTES,
            import_icon_transfers,
        ),
        beam_sources: Imported::import(image, BEAM_SOURCE_ADDRESS, 2 * BEAM_FRAMES, import_words),
        beam_destinations: Imported::import(
            image,
            BEAM_DESTINATION_ADDRESS,
            2 * BEAM_FRAMES,
            import_words,
        ),
    }
}
