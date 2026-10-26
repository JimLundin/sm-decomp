//! Save-slot serialization over the original SRAM format and ordered RAM projections.
use crate::game::constants::buttons::*;
use crate::game_state::Game;
use crate::ram_map::checked_word as word;
use crate::ram_map::word as game_word;
use crate::rtl::rom_table_word;

/// Accept the stored menu selection only when its range and inverse-bit check agree.
pub(crate) fn last_selected(game: &Game) -> u16 {
    let slot = game.g_sram.word(0x1fec);
    if slot < 3 && slot & game.g_sram.word(0x1fee) == 0 {
        slot
    } else {
        0
    }
}

#[derive(Clone, Copy)]
enum Direction {
    Save,
    Load,
}

fn transfer_slot(slot: u16, direction: Direction, game: &mut Game) -> u16 {
    let mut cursor = rom_table_word!(game, 0x81812b, (slot & 3) as isize);
    let mut checksum = 0u16;
    for at in (0xd7c0..0xde1c).step_by(2) {
        let sram = usize::from(cursor & !1);
        let value = match direction {
            Direction::Save => u16::from_ne_bytes([game.g_ram[at], game.g_ram[at + 1]]),
            Direction::Load => game.g_sram.word(sram),
        };
        match direction {
            Direction::Save => game.g_sram.set_word(sram, value),
            Direction::Load => game.g_ram[at..at + 2].copy_from_slice(&value.to_ne_bytes()),
        }
        checksum = checksum.wrapping_add(value);
        cursor = cursor.wrapping_add(2);
    }
    checksum
}

pub(crate) fn save(slot: u16, game: &mut Game) {
    #[cfg(sm_verify_dll)]
    crate::flags_save_observer::observe(0, game);
    word_transfers!(game, descending, 48; le(player_data_saved, 47) <- native(0xa00));
    #[cfg(sm_verify_dll)]
    crate::flags_save_observer::observe(1, game);
    word_transfers!(game, ascending, 128;
        le(explored_map_tiles_saved, word!(game.area_index).wrapping_mul(256) >> 1)
            <- native(0x7f7));
    #[cfg(sm_verify_dll)]
    crate::flags_save_observer::observe(2, game);
    transfer_map(Direction::Save, game);
    #[cfg(sm_verify_dll)]
    crate::flags_save_observer::observe(3, game);
    word!(game.sram_save_station_index = word!(game.load_station_index));
    word!(game.sram_area_index = word!(game.area_index));
    #[cfg(sm_verify_dll)]
    crate::flags_save_observer::observe(4, game);
    let checksum = transfer_slot(slot, Direction::Save, game);
    #[cfg(sm_verify_dll)]
    crate::flags_save_observer::observe(5, game);
    let header = 2 * usize::from(slot & 3);
    game.g_sram.set_word(header, checksum);
    game.g_sram.set_word(header + 0x1ff0, checksum);
    game.g_sram.set_word(header + 8, !checksum);
    game.g_sram.set_word(header + 0x1ff8, !checksum);
    #[cfg(sm_verify_dll)]
    crate::flags_save_observer::observe(6, game);
    unsafe { crate::rtl::RtlWriteSram(game) };
}

pub(crate) fn load(slot: u16, game: &mut Game) -> u8 {
    let checksum = transfer_slot(slot, Direction::Load, game);
    let header = 2 * usize::from(slot & 3);
    if checksum == game.g_sram.word(header) && !checksum == game.g_sram.word(header + 8)
        || checksum == game.g_sram.word(header + 0x1ff0)
            && !checksum == game.g_sram.word(header + 0x1ff8)
    {
        word_transfers!(game, descending, 48; native(0xa00) <- le(player_data_saved, 47));
        transfer_map(Direction::Load, game);
        word!(game.load_station_index = word!(game.sram_save_station_index));
        word!(game.area_index = word!(game.sram_area_index));
        0
    } else {
        let mut cursor = rom_table_word!(game, 0x81812b, (slot & 3) as isize);
        for _ in (0xd7c0..0xde1c).step_by(2) {
            game.g_sram.set_word(usize::from(cursor & !1), 0);
            cursor = cursor.wrapping_add(2);
        }
        word!(game.load_station_index = 0);
        word!(game.area_index = 0);
        1
    }
}

fn transfer_map(direction: Direction, game: &mut Game) {
    if matches!(direction, Direction::Load) {
        word_transfers!(game, descending, 897; le(explored_map_tiles_saved, 896) <- value(0));
    }
    for area in 0..6 {
        let mut remaining = i32::from(
            game.capture_native_rom()
                .read(crate::rtl::lorom_offset(0x818131) as usize + area),
        );
        let cursor = rom_table_word!(game, 0x8182d6, area as isize);
        // Preserve the original diagnostic before capturing its native allocation.
        let map = crate::game::menu_tilemap::native_record(0x810000 | u32::from(cursor), game);
        let mut packed = usize::from(rom_table_word!(game, 0x818138, area as isize));
        let mut record = 0;
        loop {
            let unpacked = 0xcd52 + area * 256 + usize::from(map.byte(record));
            let packed_at = 0xd91c + packed;
            let (source, destination) = match direction {
                Direction::Save => (unpacked, packed_at),
                Direction::Load => (packed_at, unpacked),
            };
            game.g_ram[destination] = game.g_ram[source];
            record += 1;
            packed += 1;
            remaining -= 1;
            if remaining == 0 {
                break;
            }
        }
    }
}

pub(crate) fn initialize(game: &mut Game) {
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
    word_transfers!(game, ascending, 4;
        native(0xd830) <- value(0),
        native(0xd870) <- value(0),
        native(0xd870) <- value(0),
        native(0xd8b0) <- value(0),
        le(UNUSED_word_7ED8F0, 0) <- value(0),
        native(0xd908) <- value(0),
        native(0xd8f8) <- value(0),
        native(0xd900) <- value(0),
    );
    word_transfers!(game, ascending, 28;
        native(0xd838) <- value(0),
        native(0xd878) <- value(0),
        native(0xd878) <- value(0),
        native(0xd8b8) <- value(0),
    );
    word_transfers!(game, ascending, 896; le(explored_map_tiles_saved, 0) <- value(0));
}

pub(crate) unsafe fn record_completion(game: &mut Game) {
    unsafe {
        game.g_sram.write_bytes(0x1fe0, b"supermetroid");
        crate::rtl::RtlWriteSram(game);
    }
}
pub(crate) unsafe fn verify_boot(game: &mut Game) {
    unsafe {
        game_word!(game.num_demo_sets = 3);
        if (0..3).all(|slot| load(slot, game) != 0) {
            game.g_sram.write_bytes(0x1fe0, b"madadameyohn");
            crate::rtl::RtlWriteSram(game);
        } else if game.g_sram.bytes(0x1fe0..0x1fec) == b"supermetroid" {
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

pub(crate) fn load_area_map(game: &mut Game) {
    for (active, saved) in area_map_words(word!(game.area_index)) {
        word_transfers!(game, ascending, 1; native(0x7f7 + active * 2) <- le(explored_map_tiles_saved, saved));
    }
    word!(game.has_area_map = game.g_ram[0xd908 + usize::from(word!(game.area_index))] as u16);
}

pub(crate) fn save_area_map(game: &mut Game) {
    for (active, saved) in area_map_words(word!(game.area_index)) {
        word_transfers!(game, ascending, 1; le(explored_map_tiles_saved, saved) <- native(0x7f7 + active * 2));
    }
    if word!(game.has_area_map) != 0 {
        let flag = 0xd908 + usize::from(word!(game.area_index));
        word!(game, flag; = word!(game, flag;) | 0xff);
    }
}

pub(crate) fn mark_elevator(game: &mut Game) {
    let table = rom_table_word!(game, 0x80cd46, word!(game.area_index) as isize);
    let displacement = 4 * ((word!(game.elevator_door_properties_orientation) & 15) as i32 - 1);
    let cursor = (i32::from(table) + displacement) as u16;
    // Retain the diagnostic before capturing its native allocation.
    if cursor & 0x8000 == 0 {
        crate::game::session_runtime::native_rom_diagnostic(0x800000 | u32::from(cursor), game);
    }
    let image = game.capture_native_rom();
    let base = crate::rtl::lorom_offset(0x800000 | u32::from(cursor)) as usize;
    for record in [0, 2] {
        let destination = 0xd8f8 + usize::from(image.read(base + record));
        let previous = game.g_ram[destination];
        game.g_ram[destination] = previous | image.read(base + record + 1);
    }
}
