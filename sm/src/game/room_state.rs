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
use crate::ram_map::{checked_word, ram_ptr, word, words as ram};
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

use crate::game::visual_host::{Content, Source};
use crate::game::visual_state::{PaletteBuffer, Weather};
use crate::rom::RomRecord;

fn apply_fx(h: &mut Runtime<'_>, definition: &RomRecord, palette: PaletteBuffer) {
    h.session_mut().room.liquid_height = definition.word(2);
    h.session_mut().room.liquid_target = definition.word(4);
    h.session_mut().room.liquid_velocity = definition.word(6);
    h.session_mut().room.liquid_delay = u16::from(definition.byte(8));
    h.write_word_le(
        u16::from(definition.byte(10)),
        ram::fx_layer_blending_config_a,
    );
    h.write_word_le(
        u16::from(definition.byte(11)),
        ram::fx_layer_blending_config_b,
    );
    h.session_mut().room.liquid_options = u16::from(definition.byte(12));
    let blend = definition.byte(15);
    for color in if blend == 0 { 27..28 } else { 25..28 } {
        let value = if blend == 0 {
            0
        } else {
            h.table(Source::Loaded, 0x89aa02)
                .word(usize::from(blend & !1) + 2 * (color - 25))
        };
        let visuals = &mut h.session_mut().visual;
        match palette {
            PaletteBuffer::Current => visuals.colors[color] = value,
            PaletteBuffer::Target => visuals.targets[color] = value,
        }
    }
}

fn fx_definition(address: u16, game: &mut Game) -> RomRecord {
    let address = 0x830000 | u32::from(address);
    unsafe { crate::rtl::RomPtr(address, game) };
    crate::game::player_projectiles::content::captured(game.capture_native_rom(), address)
}

pub(crate) fn load_fx_entry(index: u16, game: &mut Game) {
    let address = checked_word!(game.room_layer3_asm_ptr).wrapping_add(16 * (index & 7));
    let definition = fx_definition(address, game);
    let mut h = Runtime::new(game);
    apply_fx(&mut h, &definition, PaletteBuffer::Current);
}

pub(crate) fn load_fx_header(game: &mut Game) {
    use crate::game::{
        animated_tiles, liquid_effects, palette_effects, scrolling_sky, sm_88, visual_runtime,
    };
    let mut address = checked_word!(game.room_layer3_asm_ptr);
    if address == 0 {
        return;
    }
    loop {
        let door = fx_definition(address, game).word(0);
        if door == 0xffff {
            return;
        }
        if door == 0 || door == checked_word!(game.door_def_ptr) {
            break;
        }
        address = address.wrapping_add(16);
    }
    let definition = fx_definition(address, game);
    let mut h = Runtime::new(game);
    apply_fx(&mut h, &definition, PaletteBuffer::Target);
    let kind = definition.byte(9);
    h.write_word_le(u16::from(kind), ram::fx_type);
    if kind != 0 {
        let tilemap = h
            .table(Source::Loaded, 0x83abf0)
            .word(usize::from(kind & !1));
        h.write_word_le(tilemap, ram::fx_tilemap_ptr);
        let initialize: Option<unsafe fn(&mut Game)> = match kind >> 1 {
            0 | 7..=15 => None,
            1 => Some(liquid_effects::initialize_lava),
            2 => Some(liquid_effects::initialize_acid),
            3 => Some(liquid_effects::initialize_water),
            4..=6 => {
                let weather =
                    [Weather::Spores, Weather::Rain, Weather::Fog][usize::from((kind >> 1) - 4)];
                h.foreign(|game| unsafe { visual_runtime::initialize_weather(weather, game) });
                None
            }
            16 => Some(scrolling_sky::initialize_land),
            17 => Some(scrolling_sky::initialize_banded_sky),
            18 => Some(visual_runtime::initialize_fireflea_lighting),
            19 => Some(visual_runtime::initialize_tourian_statue),
            20 => Some(sm_88::FxTypeFunc_28_CeresRidley),
            21 => Some(sm_88::FxTypeFunc_CeresElevator),
            22 => Some(sm_88::FxTypeFunc_2C_Haze),
            _ => panic!("room FX kind outside callback table"),
        };
        if let Some(initialize) = initialize {
            h.foreign(|game| unsafe { initialize(game) });
        }
    }
    for (field, table, spawn) in [
        (13, 0x83ac46, palette_effects::spawn as fn(u16, &mut Game)),
        (14, 0x83ac56, animated_tiles::spawn as fn(u16, &mut Game)),
    ] {
        let bits = definition.byte(field);
        if bits == 0 {
            continue;
        }
        let table = h.table(Source::Loaded, table);
        let address = table.word(2 * usize::from(h.session().room.area));
        let objects = h.legacy_native_record(0x83, address);
        for index in 0..8 {
            if bits & (1 << index) != 0 {
                let object = objects.word(2 * index);
                h.foreign(|game| spawn(object, game));
            }
        }
    }
}

pub(crate) unsafe fn advance_ceres_elevator(game: &mut Game) {
    unsafe {
        let mut v1: i16 = 0;
        if (word!(game.ceres_status) & 0x8000) != 0 {
            if (112i16).wrapping_sub((word!(game.samus_x_pos)) as i16) < 0
                && (144i16).wrapping_sub((word!(game.samus_x_pos)) as i16) >= 0
                && word!(game.samus_y_pos) as i32 - 128i32 & 0x8000 != 0
                && word!(game.samus_y_pos) as i32 - 75i32 & 0x8000 == 0
                && word!(game.samus_y_speed) == 0
                && word!(game.samus_y_subspeed) == 0
                && word!(game.game_state) as i32 == 8
            {
                crate::game::session_runtime::player_command(2, game);
                word! { game, 0; screen_fade_delay, screen_fade_counter }
                word!(game.game_state = 32);
            }
            crate::update_place! { unaligned_word fresh0 = &raw mut (*(ram_ptr!(game, u16; 0x7e1, 2))) => (u16::from_le(fresh0.read_unaligned())).wrapping_sub(1); }
            if *(ram_ptr!(game, i16; 0x7e1, 2)) < 0 {
                let v0: i32 = ((6u16).wrapping_mul(*(ram_ptr!(game, u16; 0x7e1)))) as i32 >> 1;
                *(ram_ptr!(game, u16; 0x7e1, 2)) =
                    crate::rtl::rom_table_word!(game, 0x89ad5f, v0 as isize);
                word!(
                    game.reg_M7B = crate::rtl::rom_table_word!(game, 0x89ad5f, (v0 + 1) as isize)
                );
                word!(game.reg_M7C = -(word!(game.reg_M7B) as i32) as u16);
                word!(
                    game.reg_M7A = crate::rtl::rom_table_word!(game, 0x89ad5f, (v0 + 2) as isize)
                );
                word!(game.reg_M7D = word!(game.reg_M7A));
                if *(ram_ptr!(game, i16; 0x7e1)) < 0 {
                    v1 = ((*(ram_ptr!(game, u16; 0x7e1))) as i16).wrapping_sub(1);
                    if *(ram_ptr!(game, u16; 0x7e1)) == 0x8001 {
                        v1 = 0;
                    }
                } else {
                    v1 = ((*(ram_ptr!(game, u16; 0x7e1))) as i16).wrapping_add(1);
                    if *(ram_ptr!(game, u16; 0x7e1)) == 67 {
                        v1 = -32700i16;
                    }
                }
                *(ram_ptr!(game, u16; 0x7e1)) = v1 as u16;
            }
        }
    }
}
