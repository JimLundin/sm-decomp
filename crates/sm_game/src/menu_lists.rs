//! Pause and file-select menu word tables through the imported `sm_data`
//! records.
//!
//! Each accessor serves the record while the loaded image is the imported
//! allocation; any other image, and indices past a table, keep the original
//! indexed load with its failures.
#![forbid(unsafe_code)]
use crate::imported::{ImportedWords, load};
use crate::records::Records;
use sm_data::cartridge::RomImage;
use sm_data::menu_lists::*;

#[derive(Default)]
pub struct ImportedMenuLists {
    area_select_colours: ImportedWords<AREA_SELECT_COLOURS, AREA_SELECT_COLOUR_WORDS>,
    binding_destinations: ImportedWords<BINDING_DESTINATIONS, BINDING_ROWS>,
    binding_tilemaps: ImportedWords<BINDING_BUTTON_TILEMAPS, BINDING_ROWS>,
    equipment_bits: ImportedWords<EQUIPMENT_BITS, EQUIPMENT_BIT_WORDS>,
    reserve_destinations: ImportedWords<RESERVE_TANK_DESTINATIONS, RESERVE_TANKS>,
    reserve_tilemaps: ImportedWords<RESERVE_TANK_TILEMAPS, RESERVE_TANKS>,
    equipment_tilemaps: ImportedWords<EQUIPMENT_TILEMAPS, EQUIPMENT_TILEMAP_WORDS>,
    hyper_beam_tilemaps: ImportedWords<HYPER_BEAM_TILEMAPS, HYPER_BEAM_WEAPONS>,
    wireframe_suits: ImportedWords<WIREFRAME_SUITS, WIREFRAMES>,
    wireframe_tilemaps: ImportedWords<WIREFRAME_TILEMAPS, WIREFRAMES>,
    timer_digits: ImportedWords<TIMER_DIGIT_SPRITEMAPS, DIGITS>,
    elevator_doors: ImportedWords<ELEVATOR_DOOR_TABLES, ELEVATOR_AREAS>,
    area_maps: ImportedWords<AREA_MAP_RECORDS, MAP_AREAS>,
    area_map_bits: ImportedWords<AREA_MAP_BIT_OFFSETS, MAP_AREAS>,
    menu_spritemaps: ImportedWords<MENU_SPRITEMAPS, MENU_SPRITES>,
    option_settings: ImportedWords<OPTION_SETTING_ADDRESSES, OPTION_SETTING_ROWS>,
    options_objects: ImportedWords<OPTIONS_SCREEN_OBJECTS, OPTIONS_SCREENS>,
    binding_addresses: ImportedWords<BINDING_ADDRESSES, BINDING_ROWS>,
    beam_spritemaps: ImportedWords<BEAM_SPRITEMAPS, BEAM_FRAMES>,
}

pub fn import(image: &RomImage) -> ImportedMenuLists {
    ImportedMenuLists {
        area_select_colours: ImportedWords::import(image),
        binding_destinations: ImportedWords::import(image),
        binding_tilemaps: ImportedWords::import(image),
        equipment_bits: ImportedWords::import(image),
        reserve_destinations: ImportedWords::import(image),
        reserve_tilemaps: ImportedWords::import(image),
        equipment_tilemaps: ImportedWords::import(image),
        hyper_beam_tilemaps: ImportedWords::import(image),
        wireframe_suits: ImportedWords::import(image),
        wireframe_tilemaps: ImportedWords::import(image),
        timer_digits: ImportedWords::import(image),
        elevator_doors: ImportedWords::import(image),
        area_maps: ImportedWords::import(image),
        area_map_bits: ImportedWords::import(image),
        menu_spritemaps: ImportedWords::import(image),
        option_settings: ImportedWords::import(image),
        options_objects: ImportedWords::import(image),
        binding_addresses: ImportedWords::import(image),
        beam_spritemaps: ImportedWords::import(image),
    }
}

fn lists(game: &(impl Records + ?Sized)) -> &ImportedMenuLists {
    &game.records().menu_lists
}

/// $81:A40E: one area select colour.
pub fn area_select_colour(game: &(impl Records + ?Sized), index: isize) -> u16 {
    lists(game)
        .area_select_colours
        .word(game.loaded_image(), index)
}

/// $82:F639: the tilemap destination of a binding row.
pub fn binding_destination(game: &(impl Records + ?Sized), row: isize) -> u16 {
    lists(game)
        .binding_destinations
        .word(game.loaded_image(), row)
}

/// $82:F647: the button tilemap for a binding.
pub fn binding_button_tilemap(game: &(impl Records + ?Sized), binding: isize) -> u16 {
    lists(game)
        .binding_tilemaps
        .word(game.loaded_image(), binding)
}

/// Word `index` of a slice starting at `address` inside a record at `start`;
/// slices that start outside it keep the original load.
fn slice<const START: u32, const N: usize>(
    table: &ImportedWords<START, N>,
    rom: &RomImage,
    address: u32,
    index: isize,
) -> u16 {
    match address.checked_sub(START) {
        Some(offset) if offset % 2 == 0 && offset >> 16 == 0 => {
            table.word(rom, (offset / 2) as isize + index)
        }
        _ => load(rom, address, index),
    }
}

/// Item bit `index` of the category whose bits start at `address`.
pub fn equipment_bit(game: &(impl Records + ?Sized), address: u32, index: isize) -> u16 {
    slice(
        &lists(game).equipment_bits,
        game.loaded_image(),
        address,
        index,
    )
}

/// Icon tilemap `index` of the category whose tilemaps start at `address`.
pub fn equipment_tilemap(game: &(impl Records + ?Sized), address: u32, index: isize) -> u16 {
    slice(
        &lists(game).equipment_tilemaps,
        game.loaded_image(),
        address,
        index,
    )
}

/// $82:C068: a reserve tank's tilemap destination.
pub fn reserve_tank_destination(game: &(impl Records + ?Sized), tank: isize) -> u16 {
    lists(game)
        .reserve_destinations
        .word(game.loaded_image(), tank)
}

/// $82:C088: a reserve tank's tilemap.
pub fn reserve_tank_tilemap(game: &(impl Records + ?Sized), tank: isize) -> u16 {
    lists(game).reserve_tilemaps.word(game.loaded_image(), tank)
}

/// $82:C0A8: a weapon's tilemap with the hyper beam.
pub fn hyper_beam_tilemap(game: &(impl Records + ?Sized), weapon: isize) -> u16 {
    lists(game)
        .hyper_beam_tilemaps
        .word(game.loaded_image(), weapon)
}

/// $82:B257: the equipped-suit bits of a wireframe.
pub fn wireframe_suit(game: &(impl Records + ?Sized), wireframe: isize) -> u16 {
    lists(game)
        .wireframe_suits
        .word(game.loaded_image(), wireframe)
}

/// $82:B25F: the tilemap of a wireframe.
pub fn wireframe_tilemap(game: &(impl Records + ?Sized), wireframe: isize) -> u16 {
    lists(game)
        .wireframe_tilemaps
        .word(game.loaded_image(), wireframe)
}

/// $80:9FD4: the escape timer spritemap for a digit.
pub fn timer_digit_spritemap(game: &(impl Records + ?Sized), digit: isize) -> u16 {
    lists(game).timer_digits.word(game.loaded_image(), digit)
}

/// $80:CD46: the map's elevator door table for an area.
pub fn elevator_door_table(game: &(impl Records + ?Sized), area: isize) -> u16 {
    lists(game).elevator_doors.word(game.loaded_image(), area)
}

/// $81:82D6: an area's map tilemap record.
pub fn area_map_record(game: &(impl Records + ?Sized), area: isize) -> u16 {
    lists(game).area_maps.word(game.loaded_image(), area)
}

/// $81:8138: an area's explored-map bit offset.
pub fn area_map_bit_offset(game: &(impl Records + ?Sized), area: isize) -> u16 {
    lists(game).area_map_bits.word(game.loaded_image(), area)
}

/// $82:C569: a menu spritemap.
pub fn menu_spritemap(game: &(impl Records + ?Sized), sprite: isize) -> u16 {
    lists(game)
        .menu_spritemaps
        .word(game.loaded_image(), sprite)
}

/// $82:F0AE: an option row's setting address.
pub fn option_setting_address(game: &(impl Records + ?Sized), row: isize) -> u16 {
    lists(game).option_settings.word(game.loaded_image(), row)
}

/// $82:F2ED: an options screen's object table.
pub fn options_screen_objects(game: &(impl Records + ?Sized), screen: isize) -> u16 {
    lists(game)
        .options_objects
        .word(game.loaded_image(), screen)
}

/// $82:F54A: a controller binding's RAM address.
pub fn binding_address(game: &(impl Records + ?Sized), binding: isize) -> u16 {
    lists(game)
        .binding_addresses
        .word(game.loaded_image(), binding)
}

/// $93:A1A1: a beam icon spritemap.
pub fn beam_spritemap(game: &(impl Records + ?Sized), frame: isize) -> u16 {
    lists(game).beam_spritemaps.word(game.loaded_image(), frame)
}
