//! Equipment selection, item toggling and mutually exclusive beam controls.
use super::menu_tilemap;
use super::prelude::*;
use crate::imported::{Imported, load, word as table_word};
use sm_data::equipment_screen::{
    BEAM_BITMASK_ADDRESS, BEAM_TILEMAP_OFFSET_ADDRESS, BEAMS, BOOT_BITMASK_ADDRESS, BOOTS,
    BeamBitmasks, BeamTilemapOffsets, BootBitmasks, CATEGORY_ADDRESS, CATEGORY_BYTES, Categories,
    Category, ITEM_BITMASK_ADDRESS, ITEMS, ItemBitmasks, import_words,
};

#[derive(Default)]
pub struct ImportedTables {
    categories: Option<Imported<Categories>>,
    beam_bitmasks: Option<Imported<BeamBitmasks>>,
    item_bitmasks: Option<Imported<ItemBitmasks>>,
    boot_bitmasks: Option<Imported<BootBitmasks>>,
    beam_tilemap_offsets: Option<Imported<BeamTilemapOffsets>>,
}

pub fn import(image: &sm_data::cartridge::RomImage) -> ImportedTables {
    ImportedTables {
        categories: Imported::import(image, CATEGORY_ADDRESS, CATEGORY_BYTES, Categories::import),
        beam_bitmasks: Imported::import(image, BEAM_BITMASK_ADDRESS, 2 * BEAMS, import_words),
        item_bitmasks: Imported::import(image, ITEM_BITMASK_ADDRESS, 2 * ITEMS, import_words),
        boot_bitmasks: Imported::import(image, BOOT_BITMASK_ADDRESS, 2 * BOOTS, import_words),
        beam_tilemap_offsets: Imported::import(
            image,
            BEAM_TILEMAP_OFFSET_ADDRESS,
            2 * BEAMS,
            import_words,
        ),
    }
}

/// Category `index`'s tables; other images and indices run the four original
/// indexed loads.
fn category<H: Host + ?Sized>(game: &H, index: isize) -> Category {
    Imported::read(
        &game.equipment_screen_tables().categories,
        game,
        |table| table.category(usize::try_from(index).ok()?),
        |rom| Category {
            tilemap_offsets: load(rom, CATEGORY_ADDRESS, index),
            bitmasks: load(rom, 0x82c034, index),
            equipped_address: load(rom, 0x82c03c, index),
            tilemaps: load(rom, 0x82c044, index),
        },
    )
}

pub fn update<H: Host + ?Sized>(game: &mut H) {
    let category = crate::menu!(game.PausemenuEquipmentCategoryItem) as u8;
    let navigate: [fn(&mut H); 4] = [reserves, beams, suits, boots];
    navigate[usize::from(category)](game);
    match category {
        0 => menu_tilemap::update_reserve_arrow(game),
        1 => {
            let previous = inventory!(game.EquippedBeams);
            if inventory!(game.CollectedBeams) != 0
                && crate::menu!(game.PausemenuEquipmentCategoryItem) as u8 == 1
            {
                toggle(10, game);
                exclude_other_beam(previous, game);
            }
        }
        _ => {
            if crate::menu!(game.PausemenuEquipmentCategoryItem) as u8 == category {
                toggle(18, game);
            }
        }
    }
    super::map_menu::draw_equipment_selector(game);
    menu_tilemap::reserve_tanks(game);
    menu_tilemap::queue_equipment_wireframe(game);
}

fn reserves<H: Host + ?Sized>(game: &mut H) {
    let actions: [fn(&mut H); 2] = [
        |game| super::reserve_tanks::toggle_mode(game),
        |game| super::reserve_tanks::transfer_manual(game),
    ];
    actions[usize::from(crate::menu!(game.PausemenuEquipmentCategoryItem) >> 8)](game);
    let previous = crate::menu!(game.PausemenuEquipmentCategoryItem);
    if game.observed_inputs().pressed & K_BUTTON_RIGHT as u16 != 0 {
        if game.observed_inputs().pressed & K_BUTTON_DOWN as u16 != 0
            || scan(2, 0, false, None, game) == 0xffff
        {
            scan(3, 0, false, Some(previous), game);
        }
    } else if game.observed_inputs().pressed & K_BUTTON_UP as u16 != 0 {
        if crate::menu!(game.PausemenuEquipmentCategoryItem) & 0xff00 != 0 {
            game.queue_sound(0, 0x37_u8, 6);
            crate::menu!(game.PausemenuEquipmentCategoryItem -= 256);
        }
    } else if game.observed_inputs().pressed & K_BUTTON_DOWN as u16 != 0 {
        if crate::menu!(game.PausemenuEquipmentCategoryItem) & 0xff00 == 256
            || inventory!(game.ReserveMode) == 1
            || {
                crate::menu!(game.PausemenuEquipmentCategoryItem += 256);
                inventory!(game.ReserveHealth) == 0
            }
        {
            scan(1, 0, false, Some(previous), game);
        } else {
            game.queue_sound(0, 0x37_u8, 6);
        }
    }
}

fn beams<H: Host + ?Sized>(game: &mut H) {
    let previous = crate::menu!(game.PausemenuEquipmentCategoryItem);
    if game.observed_inputs().pressed & K_BUTTON_RIGHT as u16 != 0 {
        if game.observed_inputs().pressed & K_BUTTON_UP as u16 != 0 {
            scan(2, 0, false, None, game);
        } else if scan(2, 4, false, None, game) != 0 {
            scan(3, 0, false, Some(previous), game);
        }
    } else if game.observed_inputs().pressed & K_BUTTON_DOWN as u16 != 0 {
        if inventory!(game.HyperBeam) == 0
            && crate::menu!(game.PausemenuEquipmentCategoryItem) != 1025
        {
            scan(1, shift(256, game), false, Some(previous), game);
        }
    } else if game.observed_inputs().pressed & K_BUTTON_UP as u16 != 0
        && (crate::menu!(game.PausemenuEquipmentCategoryItem) & 0xff00 == 0
            || scan(1, shift(-256, game), true, None, game) == 0xffff)
        && select_reserves(game) == 0
    {
        crate::menu!(game.PausemenuEquipmentCategoryItem = previous);
    }
}

fn suits<H: Host + ?Sized>(game: &mut H) {
    let previous = crate::menu!(game.PausemenuEquipmentCategoryItem);
    if game.observed_inputs().pressed & K_BUTTON_LEFT as u16 != 0 {
        if game.observed_inputs().pressed & K_BUTTON_DOWN as u16 == 0 {
            if select_reserves(game) != 0 {
                return;
            }
            crate::menu!(game.PausemenuEquipmentCategoryItem = previous);
        }
        scan(1, 0, false, Some(previous), game);
    } else if game.observed_inputs().pressed & K_BUTTON_UP as u16 != 0 {
        if crate::menu!(game.PausemenuEquipmentCategoryItem) & 0xff00 != 0 {
            scan(2, shift(-256, game), true, Some(previous), game);
        }
    } else if game.observed_inputs().pressed & K_BUTTON_DOWN as u16 != 0
        && (crate::menu!(game.PausemenuEquipmentCategoryItem) & 0xff00 == 1280
            || scan(2, shift(256, game), false, None, game) == 0xffff)
    {
        scan(3, 0, false, Some(previous), game);
    }
}

fn boots<H: Host + ?Sized>(game: &mut H) {
    let previous = crate::menu!(game.PausemenuEquipmentCategoryItem);
    if game.observed_inputs().pressed & K_BUTTON_LEFT as u16 != 0 {
        if (game.observed_inputs().pressed & K_BUTTON_UP as u16 != 0
            || scan(1, 8, true, None, game) == 0xffff)
            && select_reserves(game) == 0
        {
            crate::menu!(game.PausemenuEquipmentCategoryItem = previous);
        }
    } else if game.observed_inputs().pressed & K_BUTTON_DOWN as u16 != 0 {
        if crate::menu!(game.PausemenuEquipmentCategoryItem) != 515 {
            scan(3, shift(256, game), false, Some(previous), game);
        }
    } else if game.observed_inputs().pressed & K_BUTTON_UP as u16 != 0
        && (crate::menu!(game.PausemenuEquipmentCategoryItem) & 0xff00 == 0
            || scan(3, shift(-256, game), true, None, game) == 0xffff)
    {
        scan(2, 10, true, Some(previous), game);
    }
}

fn shift<H: Host + ?Sized>(amount: i16, game: &mut H) -> u16 {
    crate::menu!(game.PausemenuEquipmentCategoryItem += amount as u16);
    // The selected item is the word's high byte.
    2 * (crate::menu!(game.PausemenuEquipmentCategoryItem) >> 8)
}

fn select_reserves<H: Host + ?Sized>(game: &mut H) -> u16 {
    let maximum = inventory!(game.MaxReserveHealth);
    if inventory!(game.MaxReserveHealth) != 0 {
        crate::menu!(game.PausemenuEquipmentCategoryItem = 0);
        game.queue_sound(0, 0x37_u8, 6);
        1
    } else {
        maximum
    }
}

// Scan original word offsets, including signed wraparound and the suit end gate.
fn scan<H: Host + ?Sized>(
    category: u16,
    mut offset: u16,
    upward: bool,
    fallback: Option<u16>,
    game: &mut H,
) -> u16 {
    let limit = match category {
        1 => 10,
        2 => 10,
        3 => 6,
        _ => unreachable!(),
    };
    let mut found = category != 1 || inventory!(game.HyperBeam) == 0;
    while found {
        let index = (offset >> 1) as isize;
        let tables = &game.equipment_screen_tables();
        let mask = match category {
            1 => table_word(&tables.beam_bitmasks, game, BEAM_BITMASK_ADDRESS, index),
            2 => table_word(&tables.item_bitmasks, game, ITEM_BITMASK_ADDRESS, index),
            _ => table_word(&tables.boot_bitmasks, game, BOOT_BITMASK_ADDRESS, index),
        };
        let collected = if category == 1 {
            inventory!(game.CollectedBeams)
        } else {
            inventory!(game.CollectedItems)
        };
        if mask & collected != 0 {
            break;
        }
        offset = offset.wrapping_add(if upward { 0xfffe } else { 2 });
        found = if upward {
            offset & 0x8000 == 0
        } else {
            (offset as i16).wrapping_sub(limit) < 0
        };
    }
    if !found {
        if let Some(previous) = fallback {
            crate::menu!(game.PausemenuEquipmentCategoryItem = previous);
        }
        return 0xffff;
    }
    let selection = ((offset >> 1) << 8) | category;
    // Downward beam selection stores before sound; all other paths store after it.
    if category == 1 && !upward {
        crate::menu!(game.PausemenuEquipmentCategoryItem = selection);
        game.queue_sound(0, 0x37_u8, 6);
    } else {
        game.queue_sound(0, 0x37_u8, 6);
        crate::menu!(game.PausemenuEquipmentCategoryItem = selection);
    }
    if category == 2 && !upward {
        0
    } else {
        selection
    }
}

fn exclude_other_beam<H: Host + ?Sized>(previous: u16, game: &mut H) {
    let added = inventory!(game.EquippedBeams) & !previous;
    let (remove, tile) = if added & 4 != 0 {
        (8, 4)
    } else if added & 8 != 0 {
        (4, 3)
    } else {
        return;
    };
    if inventory!(game.EquippedBeams) & remove != 0 {
        inventory!(game.EquippedBeams &= !remove);
        menu_tilemap::recolor_native(
            table_word(
                &game.equipment_screen_tables().beam_tilemap_offsets,
                game,
                BEAM_TILEMAP_OFFSET_ADDRESS,
                tile,
            ) as usize,
            3072,
            5,
            game,
        );
    }
}

fn toggle<H: Host + ?Sized>(bytes: u16, game: &mut H) {
    if game.observed_inputs().pressed & K_BUTTON_A as u16 == 0 {
        return;
    }
    crate::probe::hit("equipment_menu.toggle");
    game.queue_sound(0, 0x38_u8, 6);
    let item = crate::menu!(game.PausemenuEquipmentCategoryItem) >> 8;
    let category = crate::menu!(game.PausemenuEquipmentCategoryItem) as u8 as isize;
    let tables = self::category(game, category);
    let target = list_entry(tables.tilemap_offsets, item, game) as usize;
    let address = tables.equipped_address;
    if address >= 0x2000 {
        game.legacy_assertion(true);
    }
    let address = usize::from(address);
    let mask = list_entry(tables.bitmasks, item, game);
    let value = u16::from_ne_bytes([
        game.ram().read_byte(address),
        game.ram().read_byte(address + 1),
    ]);
    let enabled = value & mask != 0;
    let value = if enabled { value & !mask } else { value | mask };
    game.ram().copy_in(address, &value.to_ne_bytes());
    if enabled {
        menu_tilemap::recolor_native(target, 0xc00, usize::from(bytes >> 1), game);
    } else {
        let source = list_entry(tables.tilemaps, item, game);
        menu_tilemap::copy_native_tiles(target, source, usize::from(bytes), game);
    }
}

/// Entry `item` of a category's bank $82 list, as one original native fetch.
fn list_entry<H: Host + ?Sized>(list: u16, item: u16, game: &mut H) -> u16 {
    let address = list.wrapping_add(2 * item);
    menu_tilemap::native_record(0x820000 | u32::from(address), game).word(0)
}
