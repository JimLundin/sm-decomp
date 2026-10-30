//! Save-slot serialization over the original SRAM format and ordered RAM projections.
use super::prelude::*;
use crate::imported::Imported;
use crate::ram::checked_word as word;
use crate::ram::word as game_word;
use crate::save_file::{self, SaveHost};
use sm_data::cartridge::{RomImage, lorom_offset};
use sm_data::save_slots::{ADDRESS, BYTES, SaveSlots};

pub type ImportedSlots = Imported<SaveSlots>;

pub fn import(image: &RomImage) -> Option<ImportedSlots> {
    Imported::import(image, ADDRESS, BYTES, SaveSlots::import)
}

/// Save file `slot`'s SRAM offset in the loaded image. The record serves the
/// three slots of the imported allocation; anything else keeps the original
/// indexed load with its failures.
pub fn sram_offset<H: Host + ?Sized>(game: &H, slot: isize) -> u16 {
    let rom = game.loaded_image();
    usize::try_from(slot)
        .ok()
        .and_then(|slot| Imported::from(game.save_slot_offsets(), rom)?.offset(slot))
        .unwrap_or_else(|| rom.table_word(lorom_offset(ADDRESS) as usize, slot))
}

/// The save format's view of work RAM, SRAM and the slot offsets.
struct SaveFile<'a, H: ?Sized>(&'a mut H);

impl<H: Host + ?Sized> SaveHost for SaveFile<'_, H> {
    fn ram_word(&self, at: usize) -> u16 {
        u16::from_ne_bytes([self.0.ram().read_byte(at), self.0.ram().read_byte(at + 1)])
    }
    fn set_ram_word(&mut self, at: usize, value: u16) {
        self.0.ram().copy_in(at, &value.to_ne_bytes());
    }
    fn sram_word(&self, at: usize) -> u16 {
        self.0.sram().word(at)
    }
    fn set_sram_word(&mut self, at: usize, value: u16) {
        self.0.sram().set_word(at, value);
    }
    fn slot_offset(&self, slot: isize) -> u16 {
        sram_offset(self.0, slot)
    }
}

/// Accept the stored menu selection only when its range and inverse-bit check agree.
pub fn last_selected<H: Host + ?Sized>(game: &mut H) -> u16 {
    save_file::last_selected(&SaveFile(game))
}

#[derive(Clone, Copy)]
enum Direction {
    Save,
    Load,
}

pub fn save<H: Host + ?Sized>(slot: u16, game: &mut H) {
    game.observe_flags_save(0);
    crate::word_transfers!(game, descending, 48; le(PLAYER_DATA_SAVED, 47) <- native(0xa00));
    game.observe_flags_save(1);
    crate::word_transfers!(game, ascending, 128;
        le(EXPLORED_MAP_TILES_SAVED, crate::room_view::current(game).area.wrapping_mul(256) >> 1)
            <- native(0x7f7));
    game.observe_flags_save(2);
    transfer_map(Direction::Save, game);
    game.observe_flags_save(3);
    word!(game.SRAM_SAVE_STATION_INDEX = word!(game.LOAD_STATION_INDEX));
    word!(game.SRAM_AREA_INDEX = crate::room_view::current(game).area);
    game.observe_flags_save(4);
    let checksum = save_file::save(&mut SaveFile(game), slot);
    game.observe_flags_save(5);
    save_file::write_checksum(&mut SaveFile(game), slot, checksum);
    game.observe_flags_save(6);
    game.flush_sram();
}

pub fn load<H: Host + ?Sized>(slot: u16, game: &mut H) -> u8 {
    let checksum = save_file::load(&mut SaveFile(game), slot);
    if save_file::checksum_matches(&SaveFile(game), slot, checksum) {
        crate::word_transfers!(game, descending, 48; native(0xa00) <- le(PLAYER_DATA_SAVED, 47));
        transfer_map(Direction::Load, game);
        word!(game.LOAD_STATION_INDEX = word!(game.SRAM_SAVE_STATION_INDEX));
        crate::room_view::set(
            game,
            crate::room_view::Word::Area,
            word!(game.SRAM_AREA_INDEX),
        );
        0
    } else {
        save_file::clear(&mut SaveFile(game), slot);
        word!(game.LOAD_STATION_INDEX = 0);
        crate::room_view::set(game, crate::room_view::Word::Area, 0);
        1
    }
}

fn transfer_map<H: Host + ?Sized>(direction: Direction, game: &mut H) {
    if matches!(direction, Direction::Load) {
        crate::word_transfers!(game, descending, 897; le(EXPLORED_MAP_TILES_SAVED, 896) <- value(0));
    }
    for area in 0..6 {
        let mut remaining = i32::from(
            game.native_image()
                .read(sm_data::cartridge::lorom_offset(0x818131) as usize + area),
        );
        let cursor = crate::menu_lists::area_map_record(game, area as isize);
        // Preserve the original diagnostic before capturing its native allocation.
        let map = super::menu_tilemap::native_record(0x810000 | u32::from(cursor), game);
        let mut packed = usize::from(crate::menu_lists::area_map_bit_offset(game, area as isize));
        let mut record = 0;
        loop {
            let unpacked = 0xcd52 + area * 256 + usize::from(map.byte(record));
            let packed_at = 0xd91c + packed;
            let (source, destination) = match direction {
                Direction::Save => (unpacked, packed_at),
                Direction::Load => (packed_at, unpacked),
            };
            let value = game.ram().read_byte(source);
            game.ram().write_byte(destination, value);
            record += 1;
            packed += 1;
            remaining -= 1;
            if remaining == 0 {
                break;
            }
        }
    }
}

pub fn initialize<H: Host + ?Sized>(game: &mut H) {
    inventory! { game, 99; MaxHealth, Health }
    inventory! { game, 0;
        MaxMissiles, Missiles, MaxSuperMissiles, SuperMissiles, MaxPowerBombs, PowerBombs,
    }
    game_word!(game.HUD_ITEM_INDEX = 0);
    inventory! { game, 0;
        CollectedBeams, EquippedBeams, CollectedItems, EquippedItems, ReserveMode,
        MaxReserveHealth, ReserveHealth, ReserveMissiles,
    }
    game_word! {game; BUTTON_CONFIG_UP = K_BUTTON_UP as u16; BUTTON_CONFIG_DOWN = K_BUTTON_DOWN as u16;
    BUTTON_CONFIG_LEFT = K_BUTTON_LEFT as u16; BUTTON_CONFIG_RIGHT = K_BUTTON_RIGHT as u16;
    BUTTON_CONFIG_JUMP_A = K_BUTTON_A as u16; BUTTON_CONFIG_RUN_B = K_BUTTON_B as u16;
    BUTTON_CONFIG_SHOOT_X = K_BUTTON_X as u16; BUTTON_CONFIG_ITEMCANCEL_Y = K_BUTTON_Y as u16;
    BUTTON_CONFIG_ITEMSWITCH = K_BUTTON_SELECT as u16;
    BUTTON_CONFIG_AIM_UP_R = K_BUTTON_R as u16; BUTTON_CONFIG_AIM_DOWN_L = K_BUTTON_L as u16; }
    game_word! { game, 0;
        GAME_TIME_FRAMES, GAME_TIME_SECONDS, GAME_TIME_MINUTES, GAME_TIME_HOURS, JAPANESE_TEXT_FLAG,
        MOONWALK_FLAG, HUD_AUTO_CANCEL_FLAG,
    }
    game_word! { game, 1; DEBUG_FLAG, UNUSED_WORD_7E09E8 }
    crate::word_transfers!(game, ascending, 4;
        native(0xd830) <- value(0),
        native(0xd870) <- value(0),
        native(0xd870) <- value(0),
        native(0xd8b0) <- value(0),
        le(UNUSED_WORD_7ED8F0, 0) <- value(0),
        native(0xd908) <- value(0),
        native(0xd8f8) <- value(0),
        native(0xd900) <- value(0),
    );
    crate::word_transfers!(game, ascending, 28;
        native(0xd838) <- value(0),
        native(0xd878) <- value(0),
        native(0xd878) <- value(0),
        native(0xd8b8) <- value(0),
    );
    crate::word_transfers!(game, ascending, 896; le(EXPLORED_MAP_TILES_SAVED, 0) <- value(0));
}

pub fn record_completion<H: Host + ?Sized>(game: &mut H) {
    game.sram().write_bytes(0x1fe0, b"supermetroid");
    game.flush_sram();
}
pub fn verify_boot<H: Host + ?Sized>(game: &mut H) {
    crate::flow::set(game, crate::flow::Word::NumDemoSets, 3);
    if (0..3).all(|slot| load(slot, game) != 0) {
        game.sram().write_bytes(0x1fe0, b"madadameyohn");
        game.flush_sram();
    } else if game.sram().bytes(crate::save_file::header::SIGNATURE) == b"supermetroid" {
        crate::flow::set(game, crate::flow::Word::NumDemoSets, 4);
    }
}

fn area_map_words(area: u16) -> impl Iterator<Item = (usize, u16)> {
    let first = area.swap_bytes();
    (0u16..128)
        .map(move |index| first.wrapping_add(index * 2) >> 1)
        .enumerate()
}

pub fn load_area_map<H: Host + ?Sized>(game: &mut H) {
    for (active, saved) in area_map_words(crate::room_view::current(game).area) {
        crate::word_transfers!(game, ascending, 1; native(0x7f7 + active * 2) <- le(EXPLORED_MAP_TILES_SAVED, saved));
    }
    word!(
        game.HAS_AREA_MAP = game.ram().read_byte(
            crate::ram::bytes::MAP_STATION_BYTE_ARRAY
                + usize::from(crate::room_view::current(game).area)
        ) as u16
    );
}

pub fn save_area_map<H: Host + ?Sized>(game: &mut H) {
    for (active, saved) in area_map_words(crate::room_view::current(game).area) {
        crate::word_transfers!(game, ascending, 1; le(EXPLORED_MAP_TILES_SAVED, saved) <- native(0x7f7 + active * 2));
    }
    if word!(game.HAS_AREA_MAP) != 0 {
        let flag = 0xd908 + usize::from(crate::room_view::current(game).area);
        word!(game, flag; = word!(game, flag;) | 0xff);
    }
}

pub fn mark_elevator<H: Host + ?Sized>(game: &mut H) {
    let table =
        crate::menu_lists::elevator_door_table(game, crate::room_view::current(game).area as isize);
    let displacement = 4 * ((crate::room_view::effects(game).elevator_properties & 15) as i32 - 1);
    let cursor = (i32::from(table) + displacement) as u16;
    // Retain the diagnostic before capturing its native allocation.
    if cursor & 0x8000 == 0 {
        game.native_rom_diagnostic(0x800000 | u32::from(cursor));
    }
    let image = game.native_image();
    let base = sm_data::cartridge::lorom_offset(0x800000 | u32::from(cursor)) as usize;
    for record in [0, 2] {
        let destination = 0xd8f8 + usize::from(image.read(base + record));
        let previous = game.ram().read_byte(destination);
        game.ram()
            .write_byte(destination, previous | image.read(base + record + 1));
    }
}
