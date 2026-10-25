//! Save-slot serialization over the original SRAM format and ordered RAM projections.
use crate::game_state::Game;
use crate::ram_map::checked_word as word;
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
        unsafe { crate::rtl::RomPtr(0x810000 | u32::from(cursor), game) };
        let image = game.capture_native_rom();
        let base = crate::rtl::lorom_offset(0x810000 | u32::from(cursor)) as usize;
        let mut packed = usize::from(rom_table_word!(game, 0x818138, area as isize));
        let mut record = 0;
        loop {
            let unpacked = 0xcd52 + area * 256 + usize::from(image.read(base + record));
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
