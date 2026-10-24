//! Compatibility entry adapters and retained level/graphics backend leaves.
use crate::game::decompression::decompress_to_ram;
use crate::game::room_entry::{
    content::{LoadMode, SelectorCursor, SelectorKind},
    installation, selection,
};
use crate::game::session_access::SessionAccess;
use crate::game::session_runtime::Runtime;
use crate::game::sm_80::CopyToVramNow;
use crate::game_state::Game;
use crate::ram_map::checked_word;
use crate::rtl::rom_bank_record;
use crate::types::{EnemyDef, EnemyTileset};
use core::mem::offset_of;

pub(crate) fn select_entry(address: u32, cursor: u16, game: &mut Game) -> u16 {
    let mut runtime = Runtime::new(game);
    match selection::select_entry(
        &mut runtime,
        SelectorKind::from_legacy(address),
        SelectorCursor::Legacy(cursor),
    ) {
        Some(SelectorCursor::Legacy(next)) => next,
        None => 0,
        Some(SelectorCursor::Prepared(_)) => {
            unreachable!("legacy selector returned authored cursor")
        }
    }
}

pub(crate) fn select(room: u16, game: &mut Game) {
    let mut runtime = Runtime::new(game);
    let room = runtime.session().world.room_identity(room);
    selection::select(&mut runtime, room);
}

pub(crate) fn load_header(game: &mut Game) {
    installation::load_header(&mut Runtime::new(game));
}

pub(crate) fn load_state(game: &mut Game) {
    installation::load_state(&mut Runtime::new(game));
}

pub(crate) fn load_destination_cre(game: &mut Game) {
    installation::load_destination_cre(&mut Runtime::new(game));
}

pub(crate) fn load_door(game: &mut Game) {
    installation::load_door(&mut Runtime::new(game));
}

pub(crate) fn save_departing_map(game: &mut Game) {
    installation::save_departing_map(&mut Runtime::new(game));
}

pub(crate) fn load_destination_map(game: &mut Game) {
    installation::load_destination_map(&mut Runtime::new(game));
}

pub(crate) fn run_door_setup(game: &mut Game) {
    installation::run_door_setup(&mut Runtime::new(game));
}

pub(crate) fn run_room_setup(game: &mut Game) {
    installation::run_room_setup(&mut Runtime::new(game));
}

pub(crate) fn initialize_door_objects(game: &mut Game) {
    installation::initialize_door_objects(&mut Runtime::new(game));
}

pub(crate) fn load_tiles_and_palette(game: &mut Game) {
    installation::load_tiles_and_palette(&mut Runtime::new(game));
}

pub(crate) fn load_level<const INITIAL: bool>(game: &mut Game) {
    let mode = if INITIAL {
        LoadMode::Initial
    } else {
        LoadMode::Door
    };
    installation::load_level(&mut Runtime::new(game), mode);
}

#[forbid(unsafe_code)]
pub(crate) fn decompress_level(source: u32, game: &mut Game) {
    decompress_to_ram(source, 0x10000, game);
    let size = usize::from(checked_word!(game.ram7F_start));
    for (source, destination, count) in [
        (0x10002 + size + size / 2, 0x19602, size),
        (0x10002 + size, 0x16402, size / 2),
    ] {
        // Valid room arrays can overlap. Preserve their pre-copy contents;
        // malformed extents retain the checked forward loop's write prefix.
        if source < destination + count
            && destination < source + count
            && source + count <= game.g_ram.len()
            && destination + count <= game.g_ram.len()
        {
            crate::verify_probe::hit(if destination == 0x19602 {
                "room.level.layer2_overlap"
            } else {
                "room.level.bts_overlap"
            });
            game.g_ram.copy_within(source..source + count, destination);
        } else {
            for index in 0..count {
                game.g_ram[destination + index] = game.g_ram[source + index];
            }
        }
    }
}

pub(crate) unsafe fn load_enemy_graphics(game: &mut Game) {
    unsafe {
        let mut cursor = i32::from(checked_word!(game.room_enemy_tilesets_ptr));
        if cursor == 0 {
            return;
        }
        let mut next_tile = 0x7000u16;
        loop {
            // Reacquire each bank address: this list wraps, unlike initial staging.
            let enemy = rom_bank_record::<0xb4>(cursor as u16, game)
                .word(offset_of!(EnemyTileset, enemy_def));
            if enemy == 0xffff {
                return;
            }
            let definition = rom_bank_record::<0xa0>(enemy, game);
            let size = definition.word(offset_of!(EnemyDef, tile_data_size));
            let destination = if size & 0x8000 != 0 {
                let tileset = rom_bank_record::<0xb4>(cursor as u16, game);
                (tileset.word(offset_of!(EnemyTileset, vram_dst)) & 0xf000) >> 4 | 0x6000
            } else {
                let destination = next_tile;
                next_tile = next_tile.wrapping_add(size >> 1);
                destination
            };
            CopyToVramNow(
                destination,
                definition.load24(offset_of!(EnemyDef, tile_data)),
                size & 0x7fff,
                game,
            );
            cursor += 4;
        }
    }
}

#[forbid(unsafe_code)]
pub(crate) fn load_sprite_palettes(game: &mut Game) {
    word_transfers!(game, descending, 16; le(target_palettes, 223) <- rom(0x9a8000, 223));
    word_transfers!(game, descending, 16; native(0xc3de) <- native(0xc1de));
    word_transfers!(game, descending, 16; native(0xc31e) <- rom(0x9afc00, 15));
}

#[path = "../verification/room_level_copy.rs"]
mod level_copy_checks;
pub(crate) use level_copy_checks::check_level_copy;
