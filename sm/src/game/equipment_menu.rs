//! Equipment selection, item toggling and mutually exclusive beam controls.
#![forbid(unsafe_code)]
use crate::game::imported::{Imported, load, word as table_word};
use crate::game::{
    constants::buttons::{kButton_A, kButton_Down, kButton_Left, kButton_Right, kButton_Up},
    menu_tilemap,
    music::QueueSfx1_Max6,
    reserve_tanks,
};
use crate::game_state::Game;
use crate::ram_map::word;
use sm_data::equipment_screen::{
    BEAM_BITMASK_ADDRESS, BEAM_TILEMAP_OFFSET_ADDRESS, BEAMS, BOOT_BITMASK_ADDRESS, BOOTS,
    BeamBitmasks, BeamTilemapOffsets, BootBitmasks, CATEGORY_ADDRESS, CATEGORY_BYTES, Categories,
    Category, ITEM_BITMASK_ADDRESS, ITEMS, ItemBitmasks, import_words,
};

#[derive(Default)]
pub(crate) struct ImportedTables {
    categories: Option<Imported<Categories>>,
    beam_bitmasks: Option<Imported<BeamBitmasks>>,
    item_bitmasks: Option<Imported<ItemBitmasks>>,
    boot_bitmasks: Option<Imported<BootBitmasks>>,
    beam_tilemap_offsets: Option<Imported<BeamTilemapOffsets>>,
}

pub(crate) fn import(image: &crate::rom::RomImage) -> ImportedTables {
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
fn category(game: &Game, index: isize) -> Category {
    Imported::read(
        &game.equipment_screen.categories,
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

pub(crate) fn update(game: &mut Game) {
    let category = word!(game.pausemenu_equipment_category_item) as u8;
    const NAVIGATE: [fn(&mut Game); 4] = [reserves, beams, suits, boots];
    NAVIGATE[usize::from(category)](game);
    match category {
        0 => menu_tilemap::update_reserve_arrow(game),
        1 => {
            let previous = word!(game.equipped_beams);
            if word!(game.collected_beams) != 0
                && word!(game.pausemenu_equipment_category_item) as u8 == 1
            {
                toggle(10, game);
                exclude_other_beam(previous, game);
            }
        }
        _ => {
            if word!(game.pausemenu_equipment_category_item) as u8 == category {
                toggle(18, game);
            }
        }
    }
    crate::game::map_menu::draw_equipment_selector(game);
    menu_tilemap::reserve_tanks(game);
    menu_tilemap::queue_equipment_wireframe(game);
}

fn reserves(game: &mut Game) {
    const ACTIONS: [fn(&mut Game); 2] =
        [reserve_tanks::toggle_mode, reserve_tanks::transfer_manual];
    ACTIONS[usize::from(game.ram().read_byte(0x756))](game);
    let previous = word!(game.pausemenu_equipment_category_item);
    if word!(game.joypad1_newkeys) & kButton_Right as u16 != 0 {
        if word!(game.joypad1_newkeys) & kButton_Down as u16 != 0
            || scan(2, 0, false, None, game) == 0xffff
        {
            scan(3, 0, false, Some(previous), game);
        }
    } else if word!(game.joypad1_newkeys) & kButton_Up as u16 != 0 {
        if word!(game.pausemenu_equipment_category_item) & 0xff00 != 0 {
            QueueSfx1_Max6(0x37, game);
            word!(game.pausemenu_equipment_category_item -= 256);
        }
    } else if word!(game.joypad1_newkeys) & kButton_Down as u16 != 0 {
        if word!(game.pausemenu_equipment_category_item) & 0xff00 == 256
            || word!(game.reserve_health_mode) == 1
            || {
                word!(game.pausemenu_equipment_category_item += 256);
                word!(game.samus_reserve_health) == 0
            }
        {
            scan(1, 0, false, Some(previous), game);
        } else {
            QueueSfx1_Max6(0x37, game);
        }
    }
}

fn beams(game: &mut Game) {
    let previous = word!(game.pausemenu_equipment_category_item);
    if word!(game.joypad1_newkeys) & kButton_Right as u16 != 0 {
        if word!(game.joypad1_newkeys) & kButton_Up as u16 != 0 {
            scan(2, 0, false, None, game);
        } else if scan(2, 4, false, None, game) != 0 {
            scan(3, 0, false, Some(previous), game);
        }
    } else if word!(game.joypad1_newkeys) & kButton_Down as u16 != 0 {
        if word!(game.hyper_beam_flag) == 0 && word!(game.pausemenu_equipment_category_item) != 1025
        {
            scan(1, shift(256, game), false, Some(previous), game);
        }
    } else if word!(game.joypad1_newkeys) & kButton_Up as u16 != 0
        && (word!(game.pausemenu_equipment_category_item) & 0xff00 == 0
            || scan(1, shift(-256, game), true, None, game) == 0xffff)
        && select_reserves(game) == 0
    {
        word!(game.pausemenu_equipment_category_item = previous);
    }
}

fn suits(game: &mut Game) {
    let previous = word!(game.pausemenu_equipment_category_item);
    if word!(game.joypad1_newkeys) & kButton_Left as u16 != 0 {
        if word!(game.joypad1_newkeys) & kButton_Down as u16 == 0 {
            if select_reserves(game) != 0 {
                return;
            }
            word!(game.pausemenu_equipment_category_item = previous);
        }
        scan(1, 0, false, Some(previous), game);
    } else if word!(game.joypad1_newkeys) & kButton_Up as u16 != 0 {
        if word!(game.pausemenu_equipment_category_item) & 0xff00 != 0 {
            scan(2, shift(-256, game), true, Some(previous), game);
        }
    } else if word!(game.joypad1_newkeys) & kButton_Down as u16 != 0
        && (word!(game.pausemenu_equipment_category_item) & 0xff00 == 1280
            || scan(2, shift(256, game), false, None, game) == 0xffff)
    {
        scan(3, 0, false, Some(previous), game);
    }
}

fn boots(game: &mut Game) {
    let previous = word!(game.pausemenu_equipment_category_item);
    if word!(game.joypad1_newkeys) & kButton_Left as u16 != 0 {
        if (word!(game.joypad1_newkeys) & kButton_Up as u16 != 0
            || scan(1, 8, true, None, game) == 0xffff)
            && select_reserves(game) == 0
        {
            word!(game.pausemenu_equipment_category_item = previous);
        }
    } else if word!(game.joypad1_newkeys) & kButton_Down as u16 != 0 {
        if word!(game.pausemenu_equipment_category_item) != 515 {
            scan(3, shift(256, game), false, Some(previous), game);
        }
    } else if word!(game.joypad1_newkeys) & kButton_Up as u16 != 0
        && (word!(game.pausemenu_equipment_category_item) & 0xff00 == 0
            || scan(3, shift(-256, game), true, None, game) == 0xffff)
    {
        scan(2, 10, true, Some(previous), game);
    }
}

fn shift(amount: i16, game: &mut Game) -> u16 {
    word!(game.pausemenu_equipment_category_item += amount as u16);
    // The original selector reads this physical byte, independently of host endian.
    2 * u16::from(game.ram().read_byte(0x756))
}

fn select_reserves(game: &mut Game) -> u16 {
    let maximum = word!(game.samus_max_reserve_health);
    if word!(game.samus_max_reserve_health) != 0 {
        word!(game.pausemenu_equipment_category_item = 0);
        QueueSfx1_Max6(0x37, game);
        1
    } else {
        maximum
    }
}

// Scan original word offsets, including signed wraparound and the suit end gate.
fn scan(
    category: u16,
    mut offset: u16,
    upward: bool,
    fallback: Option<u16>,
    game: &mut Game,
) -> u16 {
    let limit = match category {
        1 => 10,
        2 => 10,
        3 => 6,
        _ => unreachable!(),
    };
    let mut found = category != 1 || word!(game.hyper_beam_flag) == 0;
    while found {
        let index = (offset >> 1) as isize;
        let tables = &game.equipment_screen;
        let mask = match category {
            1 => table_word(&tables.beam_bitmasks, game, BEAM_BITMASK_ADDRESS, index),
            2 => table_word(&tables.item_bitmasks, game, ITEM_BITMASK_ADDRESS, index),
            _ => table_word(&tables.boot_bitmasks, game, BOOT_BITMASK_ADDRESS, index),
        };
        let collected = if category == 1 {
            word!(game.collected_beams)
        } else {
            word!(game.collected_items)
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
            word!(game.pausemenu_equipment_category_item = previous);
        }
        return 0xffff;
    }
    let selection = ((offset >> 1) << 8) | category;
    // Downward beam selection stores before sound; all other paths store after it.
    if category == 1 && !upward {
        word!(game.pausemenu_equipment_category_item = selection);
        QueueSfx1_Max6(0x37, game);
    } else {
        QueueSfx1_Max6(0x37, game);
        word!(game.pausemenu_equipment_category_item = selection);
    }
    if category == 2 && !upward {
        0
    } else {
        selection
    }
}

fn exclude_other_beam(previous: u16, game: &mut Game) {
    let added = word!(game.equipped_beams) & !previous;
    let (remove, tile) = if added & 4 != 0 {
        (8, 4)
    } else if added & 8 != 0 {
        (4, 3)
    } else {
        return;
    };
    if word!(game.equipped_beams) & remove != 0 {
        word!(game.equipped_beams &= !remove);
        menu_tilemap::recolor_native(
            table_word(
                &game.equipment_screen.beam_tilemap_offsets,
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

fn toggle(bytes: u16, game: &mut Game) {
    if word!(game.joypad1_newkeys) & kButton_A as u16 == 0 {
        return;
    }
    QueueSfx1_Max6(0x38, game);
    let item = u16::from(game.ram().read_byte(0x756));
    let category = word!(game.pausemenu_equipment_category_item) as u8 as isize;
    let tables = self::category(game, category);
    let target = list_entry(tables.tilemap_offsets, item, game) as usize;
    let address = tables.equipped_address;
    if address >= 0x2000 {
        crate::game::session_runtime::legacy_assertion(true);
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
fn list_entry(list: u16, item: u16, game: &mut Game) -> u16 {
    let address = list.wrapping_add(2 * item);
    menu_tilemap::native_record(0x820000 | u32::from(address), game).word(0)
}
