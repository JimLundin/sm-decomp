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
    crate::word_transfers!(game, descending, 48; le(player_data_saved, 47) <- native(0xa00));
    game.observe_flags_save(1);
    crate::word_transfers!(game, ascending, 128;
        le(explored_map_tiles_saved, word!(game.area_index).wrapping_mul(256) >> 1)
            <- native(0x7f7));
    game.observe_flags_save(2);
    transfer_map(Direction::Save, game);
    game.observe_flags_save(3);
    word!(game.sram_save_station_index = word!(game.load_station_index));
    word!(game.sram_area_index = word!(game.area_index));
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
        crate::word_transfers!(game, descending, 48; native(0xa00) <- le(player_data_saved, 47));
        transfer_map(Direction::Load, game);
        word!(game.load_station_index = word!(game.sram_save_station_index));
        word!(game.area_index = word!(game.sram_area_index));
        0
    } else {
        save_file::clear(&mut SaveFile(game), slot);
        word!(game.load_station_index = 0);
        word!(game.area_index = 0);
        1
    }
}

fn transfer_map<H: Host + ?Sized>(direction: Direction, game: &mut H) {
    if matches!(direction, Direction::Load) {
        crate::word_transfers!(game, descending, 897; le(explored_map_tiles_saved, 896) <- value(0));
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
    game_word! { game, 99; samus_max_health, samus_health }
    game_word! { game, 0;
        samus_max_missiles, samus_missiles, samus_max_super_missiles, samus_super_missiles,
        samus_max_power_bombs, samus_power_bombs, hud_item_index, collected_beams, equipped_beams,
        collected_items, equipped_items, reserve_health_mode, samus_max_reserve_health,
        samus_reserve_health, samus_reserve_missiles,
    }
    game_word! {game; button_config_up = kButton_Up as u16; button_config_down = kButton_Down as u16;
    button_config_left = kButton_Left as u16; button_config_right = kButton_Right as u16;
    button_config_jump_a = kButton_A as u16; button_config_run_b = kButton_B as u16;
    button_config_shoot_x = kButton_X as u16; button_config_itemcancel_y = kButton_Y as u16;
    button_config_itemswitch = kButton_Select as u16;
    button_config_aim_up_R = kButton_R as u16; button_config_aim_down_L = kButton_L as u16; }
    game_word! { game, 0;
        game_time_frames, game_time_seconds, game_time_minutes, game_time_hours, japanese_text_flag,
        moonwalk_flag, hud_auto_cancel_flag,
    }
    game_word! { game, 1; debug_flag, UNUSED_word_7E09E8 }
    crate::word_transfers!(game, ascending, 4;
        native(0xd830) <- value(0),
        native(0xd870) <- value(0),
        native(0xd870) <- value(0),
        native(0xd8b0) <- value(0),
        le(UNUSED_word_7ED8F0, 0) <- value(0),
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
    crate::word_transfers!(game, ascending, 896; le(explored_map_tiles_saved, 0) <- value(0));
}

pub fn record_completion<H: Host + ?Sized>(game: &mut H) {
    {
        game.sram().write_bytes(0x1fe0, b"supermetroid");
        game.flush_sram();
    }
}
pub fn verify_boot<H: Host + ?Sized>(game: &mut H) {
    {
        game_word!(game.num_demo_sets = 3);
        if (0..3).all(|slot| load(slot, game) != 0) {
            game.sram().write_bytes(0x1fe0, b"madadameyohn");
            game.flush_sram();
        } else if game.sram().bytes(0x1fe0..0x1fec) == b"supermetroid" {
            game_word!(game.num_demo_sets = 4);
        }
    }
}

fn area_map_words(area: u16) -> impl Iterator<Item = (usize, u16)> {
    let first = area.swap_bytes();
    (0u16..128)
        .map(move |index| first.wrapping_add(index * 2) >> 1)
        .enumerate()
}

pub fn load_area_map<H: Host + ?Sized>(game: &mut H) {
    for (active, saved) in area_map_words(word!(game.area_index)) {
        crate::word_transfers!(game, ascending, 1; native(0x7f7 + active * 2) <- le(explored_map_tiles_saved, saved));
    }
    word!(
        game.has_area_map =
            game.ram()
                .read_byte(0xd908 + usize::from(word!(game.area_index))) as u16
    );
}

pub fn save_area_map<H: Host + ?Sized>(game: &mut H) {
    for (active, saved) in area_map_words(word!(game.area_index)) {
        crate::word_transfers!(game, ascending, 1; le(explored_map_tiles_saved, saved) <- native(0x7f7 + active * 2));
    }
    if word!(game.has_area_map) != 0 {
        let flag = 0xd908 + usize::from(word!(game.area_index));
        word!(game, flag; = word!(game, flag;) | 0xff);
    }
}

pub fn mark_elevator<H: Host + ?Sized>(game: &mut H) {
    let table = crate::menu_lists::elevator_door_table(game, word!(game.area_index) as isize);
    let displacement = 4 * ((word!(game.elevator_door_properties_orientation) & 15) as i32 - 1);
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
