//! Compatibility entry adapters and retained level/graphics backend leaves.
use crate::game::decompression::decompress_to_ram;
use crate::game::display_refresh::CopyToVramNow;
use crate::game::room_entry::{
    content::{LoadMode, SelectorCursor, SelectorKind},
    installation, selection,
};
use crate::game::session_access::SessionAccess;
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;
use crate::ram_map::{checked_word, word, words as ram};
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

pub(crate) fn pause_draygon(game: &mut Game) {
    word!(game.irqhandler_next_handler = 4);
}

pub(crate) fn unpause_draygon(game: &mut Game) -> crate::types::CoroutineRet {
    if word!(game.hdma_object_channels_bitmask[1]) == 8 {
        word!(game.irqhandler_next_handler = 12);
    }
    0
}

pub(crate) fn setup_script(address: u32, game: &mut Game) {
    use crate::game::room_mechanisms::spawn_hardcoded;
    use crate::game::visual_runtime::initialize_room_haze;
    use crate::game::{animated_tiles, constants::bank_87::*, scrolling_sky};
    use crate::types::SpawnHardcodedPlmArgs;
    match address {
        9408916 => {
            spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x3d, 0xb, 0xbb30), game);
            word! {game; earthquake_type = 24; earthquake_timer = u16::MAX; }
        }
        9408937 => spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x10, 0x87, 0xb964), game),
        9408946 => spawn_hardcoded(SpawnHardcodedPlmArgs::new(0xf, 0xa, 0xb9ed), game),
        9408957 => {
            word! {game; earthquake_type = 6; earthquake_timer = u16::MAX; }
            scrolling_sky::initialize_land(game);
        }
        9408969 => scrolling_sky::initialize_land(game),
        9408974 => scrolling_sky::initialize_ocean(game),
        9408983 => {
            for animation in [
                addr_kAnimtiles_TourianStatue_Kraid,
                addr_kAnimtiles_TourianStatue_Phantoon,
                addr_kAnimtiles_TourianStatue_Draygon,
                addr_kAnimtiles_TourianStatue_Ridley,
            ] {
                animated_tiles::spawn(animation as u16, game);
            }
        }
        9423048 => {
            crate::game::enemy_projectiles::SpawnEprojWithRoomGfx(
                crate::game::constants::bank_86::addr_kEproj_PrePhantomRoom as u16,
                0xc8c8,
                game,
            );
        }
        9423059 => spawn_hardcoded(SpawnHardcodedPlmArgs::new(0, 0, 0xb8eb), game),
        9423069 => {
            // LongPtr stores its address in native order, then the bank byte.
            for (at, hook) in [(0x601, 0xc8f6), (0x604, 0xc8fc)] {
                word_transfers!(game, ascending, 1; native(at) <- value(hook));
                game.ram().write_byte(at + 2, 0x8f);
            }
        }
        9423114 => {
            let at = 0xd908 + usize::from(word!(game.area_index));
            let collected =
                u16::from_ne_bytes([game.ram().read_byte(at), game.ram().read_byte(at + 1)]) | 1;
            word_transfers!(game, ascending, 1; native(at) <- value(collected));
            word!(game.has_area_map = collected);
        }
        9423135 => {
            crate::game::gameplay_flag_runtime::SetEventHappened(0xe, game);
            word! {game; earthquake_type = 18; earthquake_timer = u16::MAX; }
        }
        9423155 | 9423187 => {
            let kind = if address == 9423187 {
                spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x10, 0x10, 0xb968), game);
                21
            } else {
                18
            };
            word!(game.earthquake_type = kind);
            word_transfers!(game, ascending, 1;
                    native(0x7e3) <- value(kind), native(0x7e1) <- value(0));
            word!(game.earthquake_timer = u16::MAX);
        }
        9423174 => {
            word! {game; earthquake_type = 21; earthquake_timer = u16::MAX; }
        }
        9423214 => {
            spawn_hardcoded(SpawnHardcodedPlmArgs::new(0xf, 0x26, 0xba48), game);
            initialize_room_haze(game);
        }
        9423222 => initialize_room_haze(game),
        9423227 => {
            game.ram().write_byte(0x5d, 102);
            initialize_room_haze(game);
            word!(game.hdma_data_table_in_ceres = 9);
        }
        9408955 | 9408956 | 9408979 | 9408980 | 9408981 | 9408982 | 9409012 | 9409013 | 9409014
        | 9409015 | 9423047 | 9423056 | 9423057 | 9423058 | 9423068 | 9423134 => {}
        _ => {
            crate::rtl::Unreachable(game);
        }
    }
}

pub(crate) unsafe fn door_script(address: u32, game: &mut Game) {
    use crate::game::room_mechanisms::spawn_hardcoded;
    use crate::game::session_runtime::player_command;
    use crate::game::{animated_tiles, constants::bank_87::*};
    use crate::types::SpawnHardcodedPlmArgs;
    unsafe {
        match address {
            9419121 | 9429464 => {
                let (animation, mechanism) = if address == 9419121 {
                    (addr_kAnimtiles_WreckedShipTradmillRight, 0xb64b)
                } else {
                    (addr_kAnimtiles_WreckedShipTradmillLeft, 0xb64f)
                };
                animated_tiles::spawn(animation as u16, game);
                spawn_hardcoded(SpawnHardcodedPlmArgs::new(4, 9, mechanism), game);
            }
            9429612 | 9429649 => {
                if address == 9429612 {
                    word_transfers!(game, ascending, 1;
                        native(0x7e5) <- value((-256i16) as u16),
                        native(0x7e3) <- value(2496),
                        native(0x7e7) <- value((-32i16) as u16));
                } else {
                    // The original strings write little-endian bytes on every host.
                    word_transfers!(game, ascending, 1;
                        native(0x7e5) <- value(256),
                        little(0x7e3) <- value(64), little(0x7e7) <- value(32));
                }
                player_command(0, game);
                spawn_hardcoded(SpawnHardcodedPlmArgs::new(1, 0, 0xb8f9), game);
            }
            9429761 | 9429769 => {
                if address == 9429769 {
                    word_transfers!(game, ascending, 1; native(0xcd20) <- value(514));
                }
                player_command(1, game);
            }
            9430240 => {
                crate::rtl::write_register_and_mirror(
                    crate::snes::registers::BGMODE as u16,
                    7,
                    0x56,
                    true,
                    game,
                );
                word! { game, 256; reg_M7A, reg_M7D }
                word! { game, 0; reg_M7B, reg_M7C }
                word! {game; reg_M7X = 128; reg_M7Y = 1008; irq_enable_mode7 = 1; }
                word_transfers!(game, ascending, 1;
                    little(0x7e1) <- value(34), little(0x7e3) <- value(60));
            }
            9430291 => {
                game.ram().write_byte(0x56, 9);
                word!(game.irq_enable_mode7 = 0);
            }
            _ => {
                if !crate::game::room_scrolls::apply(address, game) {
                    crate::rtl::Unreachable(game);
                }
            }
        }
    }
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
    copy_level_planes(game);
}

#[forbid(unsafe_code)]
pub(crate) fn copy_level_planes(game: &mut Game) {
    let size = usize::from(checked_word!(game.ram7F_start));
    for (source, destination, count) in [
        (0x10002 + size + size / 2, 0x19602, size),
        (0x10002 + size, 0x16402, size / 2),
    ] {
        // Valid room arrays can overlap. Preserve their pre-copy contents;
        // malformed extents retain the checked forward loop's write prefix.
        if source < destination + count
            && destination < source + count
            && source + count <= 0x20000
            && destination + count <= 0x20000
        {
            crate::verify_probe::hit(if destination == 0x19602 {
                "room.level.layer2_overlap"
            } else {
                "room.level.bts_overlap"
            });
            game.ram().copy_within(source..source + count, destination);
        } else {
            for index in 0..count {
                game.ram()
                    .write_byte(destination + index, game.ram().read_byte(source + index));
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

fn apply_fx(h: &mut Runtime<'_>, definition: &FxRecord, palette: PaletteBuffer) {
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
            fx_table(h, 0x89aa02).word(usize::from(blend & !1) + 2 * (color - 25))
        };
        let visuals = &mut h.session_mut().visual;
        match palette {
            PaletteBuffer::Current => visuals.colors[color] = value,
            PaletteBuffer::Target => visuals.targets[color] = value,
        }
    }
}

enum FxRecord {
    Legacy(RomRecord),
    Prepared {
        assets: std::sync::Arc<super::room_entry::PreparedRoomAssets>,
        base: usize,
    },
}
impl FxRecord {
    fn byte(&self, offset: usize) -> u8 {
        match self {
            Self::Legacy(record) => record.byte(offset),
            Self::Prepared { assets, base } => assets.rom_byte(base.checked_add(offset).unwrap()),
        }
    }
    fn word(&self, offset: usize) -> u16 {
        u16::from_ne_bytes([self.byte(offset), self.byte(offset + 1)])
    }
}
#[forbid(unsafe_code)]
fn fx_definition(address: u16, game: &mut Game) -> FxRecord {
    let full = 0x830000 | u32::from(address);
    if game.world().prepared_room_assets().is_some() {
        if address & 0x8000 == 0 {
            crate::game::session_runtime::native_rom_diagnostic(full, game);
        }
        FxRecord::Prepared {
            assets: game.world().prepared_room_assets().unwrap(),
            base: crate::rom::lorom_offset(full) as usize,
        }
    } else {
        FxRecord::Legacy(crate::game::menu_tilemap::native_record(full, game))
    }
}
fn fx_table(h: &Runtime<'_>, address: u32) -> FxRecord {
    match h.session().world.prepared_room_assets() {
        Some(assets) => FxRecord::Prepared {
            assets,
            base: crate::rom::lorom_offset(address) as usize,
        },
        None => FxRecord::Legacy(h.table(Source::Loaded, address)),
    }
}

pub(crate) fn load_fx_entry(index: u16, game: &mut Game) {
    let address = checked_word!(game.room_layer3_asm_ptr).wrapping_add(16 * (index & 7));
    let definition = fx_definition(address, game);
    let mut h = Runtime::new(game);
    apply_fx(&mut h, &definition, PaletteBuffer::Current);
}

pub(crate) fn load_fx_header(game: &mut Game) {
    use crate::game::{
        animated_tiles, liquid_effects, palette_effects, scrolling_sky, visual_runtime,
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
        let tilemap = fx_table(&h, 0x83abf0).word(usize::from(kind & !1));
        h.write_word_le(tilemap, ram::fx_tilemap_ptr);
        let initialize: Option<fn(&mut Game)> = match kind >> 1 {
            0 | 7..=15 => None,
            1 => Some(liquid_effects::initialize_lava),
            2 => Some(liquid_effects::initialize_acid),
            3 => Some(liquid_effects::initialize_water),
            4..=6 => {
                let weather =
                    [Weather::Spores, Weather::Rain, Weather::Fog][usize::from((kind >> 1) - 4)];
                h.foreign(|game| visual_runtime::initialize_weather(weather, game));
                None
            }
            16 => Some(scrolling_sky::initialize_land),
            17 => Some(scrolling_sky::initialize_banded_sky),
            18 => Some(visual_runtime::initialize_fireflea_lighting),
            19 => Some(visual_runtime::initialize_tourian_statue),
            20 => Some(visual_runtime::initialize_ceres_ridley),
            21 => Some(visual_runtime::initialize_ceres_elevator),
            22 => Some(visual_runtime::initialize_room_haze),
            _ => panic!("room FX kind outside callback table"),
        };
        if let Some(initialize) = initialize {
            h.foreign(initialize);
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
        let table = fx_table(&h, table);
        let address = table.word(2 * usize::from(h.session().room.area));
        let objects = if h.session().world.prepared_room_assets().is_some() {
            if address & 0x8000 == 0 {
                h.foreign(|game| {
                    crate::game::session_runtime::native_rom_diagnostic(
                        0x830000 | u32::from(address),
                        game,
                    )
                });
            }
            FxRecord::Prepared {
                assets: h.session().world.prepared_room_assets().unwrap(),
                base: crate::rom::lorom_offset(0x830000 | u32::from(address)) as usize,
            }
        } else {
            FxRecord::Legacy(h.legacy_native_record(0x83, address))
        };
        for index in 0..8 {
            if bits & (1 << index) != 0 {
                let object = objects.word(2 * index);
                h.foreign(|game| spawn(object, game));
            }
        }
    }
}

#[forbid(unsafe_code)]
pub(crate) fn advance_ceres_elevator(game: &mut Game) {
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
        let countdown = word!(@read game, 0x7e3);
        word!(@write game, 0x7e3, countdown.wrapping_sub(1));
        if (u16::from_ne_bytes([game.ram().read_byte(0x7e3), game.ram().read_byte(0x7e4)]) as i16)
            < 0
        {
            let v0: i32 = ((6u16).wrapping_mul(u16::from_ne_bytes([
                game.ram().read_byte(0x7e1),
                game.ram().read_byte(0x7e2),
            ]))) as i32
                >> 1;
            let delay = crate::rtl::rom_table_word!(game, 0x89ad5f, v0 as isize);
            word_transfers!(game, ascending, 1; native(0x7e3) <- value(delay));
            word!(game.reg_M7B = crate::rtl::rom_table_word!(game, 0x89ad5f, (v0 + 1) as isize));
            word!(game.reg_M7C = -(word!(game.reg_M7B) as i32) as u16);
            word!(game.reg_M7A = crate::rtl::rom_table_word!(game, 0x89ad5f, (v0 + 2) as isize));
            word!(game.reg_M7D = word!(game.reg_M7A));
            if (u16::from_ne_bytes([game.ram().read_byte(0x7e1), game.ram().read_byte(0x7e2)])
                as i16)
                < 0
            {
                v1 = ((u16::from_ne_bytes([
                    game.ram().read_byte(0x7e1),
                    game.ram().read_byte(0x7e2),
                ])) as i16)
                    .wrapping_sub(1);
                if u16::from_ne_bytes([game.ram().read_byte(0x7e1), game.ram().read_byte(0x7e2)])
                    == 0x8001
                {
                    v1 = 0;
                }
            } else {
                v1 = ((u16::from_ne_bytes([
                    game.ram().read_byte(0x7e1),
                    game.ram().read_byte(0x7e2),
                ])) as i16)
                    .wrapping_add(1);
                if u16::from_ne_bytes([game.ram().read_byte(0x7e1), game.ram().read_byte(0x7e2)])
                    == 67
                {
                    v1 = -32700i16;
                }
            }
            word_transfers!(game, ascending, 1; native(0x7e1) <- value(v1 as u16));
        }
    }
}

pub(crate) unsafe fn advance_room(game: &mut Game) {
    let script = word!(game.room_main_code_ptr);
    if script != 0 {
        unsafe { advance_script(0x8f0000 | u32::from(script), &mut Runtime::new(game)) };
    }
}

unsafe fn random_explosions(mask: u16, check_tile: bool, h: &mut Runtime<'_>) {
    {
        if h.word(ram::time_is_frozen_flag) != 0 || h.word(ram::nmi_frame_counter_word) & mask != 0
        {
            return;
        }
        let random = h.foreign(|game| game.next_random());
        let x = h.word(ram::layer1_x_pos).wrapping_add(random as u8 as u16);
        let y = h
            .word(ram::layer1_y_pos)
            .wrapping_add(u16::from(random.to_ne_bytes()[1]));
        if check_tile {
            let row =
                u16::from((y >> 4) as u8) * u16::from(h.word(ram::room_width_in_blocks) as u8);
            if h.native_word(0x10002 + 2 * (usize::from(row) + usize::from(x >> 4))) & 0x3ff == 255
            {
                return;
            }
        }
        let random = h.foreign(|game| game.next_random()) & 15;
        if random < 8 {
            let sound = [0x24, 0, 0, 0x25, 0, 0, 0, 0][usize::from(random)];
            if sound != 0 {
                h.foreign(|game| crate::game::music::QueueSfx2_Max6(sound, game));
            }
        }
        let sprite = [3, 3, 9, 12, 12, 18, 18, 21][usize::from(random & 7)];
        h.foreign(|game| crate::game::sprite_effect_runtime::create_sprite(x, y, sprite, 0, game));
    }
}

unsafe fn select_earthquake(
    ending: u16,
    threshold: u16,
    starting: u16,
    h: &mut Runtime<'_>,
) -> Option<u16> {
    let timer = h.native_word(0x7e1);
    if timer != 0 {
        let remaining = timer.wrapping_sub(1);
        h.write_slice_native(0x7e1, remaining);
        (remaining == 0).then_some(ending)
    } else if h.foreign(|game| game.next_random()) < threshold {
        h.write_slice_le(0x7e1, 42);
        Some(starting)
    } else {
        None
    }
}

fn advance_elevatube(h: &mut Runtime<'_>) {
    use crate::game::player_collision::operations::{self, Axis, DisplacementRequest};
    h.write_word_le(128, ram::samus_x_pos);
    h.write_word_le(0, ram::samus_x_subpos);
    let amount = i32::from(h.native_word(0x7e5) as i16) << 8;
    let position = u32::from_ne_bytes([h.byte(0x7e1), h.byte(0x7e2), h.byte(0x7e3), h.byte(0x7e4)]);
    let whole = (position.wrapping_add(amount as u32) >> 16) as u16;
    h.write_slice_le(0x7e1, (i32::from(h.word(0x7e1)) + amount) as u16);
    h.write_slice_native(0x7e3, whole);
    operations::displace(
        h,
        DisplacementRequest::TerrainOnly {
            axis: Axis::Y,
            amount,
        },
    );
    if ((i32::from(h.native_word(0x7e7)) + i32::from(h.native_word(0x7e5)) + 3616) as u16) < 0x1c41
    {
        h.write_slice_le(0x7e5, h.word(0x7e5).wrapping_add(h.native_word(0x7e7)));
    }
}

fn shake_crocomire(h: &mut Runtime<'_>) {
    use crate::game::actors::access::{AccessMode::Native, ActorAccess, ActorKey};
    let body = ActorKey::from_legacy(0);
    let legs = ActorKey::from_legacy(64);
    if h.resolve_actor(body, Native).properties & 0x200 != 0 {
        return;
    }
    if h.resolve_actor(body, Native).variables[0] == 64 {
        let shift = h.resolve_actor(legs, Native).variables[3];
        h.write_word_le(
            shift
                .wrapping_add(h.word(ram::bg1_y_offset))
                .wrapping_add(h.word(ram::layer1_y_pos)),
            ram::reg_BG1VOFS,
        );
    } else if h.resolve_actor(body, Native).variables[1] & 0x400 != 0 {
        let timer = h.edit_actor(legs, Native, |actor| {
            actor.variables[3] = actor.variables[3].wrapping_sub(1);
            actor.variables[3]
        });
        let shift = if (i32::from(timer) + 7) & 0x8000 != 0 {
            (i32::from(timer) - 2 * (i32::from(timer) + 7)) as u16
        } else {
            timer
        };
        h.write_word_le(
            h.word(ram::reg_BG1VOFS).wrapping_add(shift),
            ram::reg_BG1VOFS,
        );
        h.write_word_le(shift.wrapping_sub(48), ram::reg_BG2VOFS);
    } else if h.resolve_actor(body, Native).variables[2] == 34
        && h.resolve_actor(body, Native).variables[3] != 0
    {
        let timer = h.edit_actor(body, Native, |actor| {
            actor.variables[3] = actor.variables[3].wrapping_sub(1);
            actor.variables[3]
        });
        let x = h.word(ram::layer1_x_pos);
        h.write_word_le(
            if timer & 1 != 0 {
                x.wrapping_sub(4)
            } else {
                x.wrapping_add(4)
            },
            ram::layer1_x_pos,
        );
    }
}

fn shake_ridley(h: &mut Runtime<'_>) {
    use crate::game::actors::access::{AccessMode::Native, ActorAccess, ActorKey};
    let actor = ActorKey::from_legacy(256);
    let timer = h.resolve_actor(actor, Native).variables[0];
    if timer == 0 {
        return;
    }
    let index = usize::from(timer - 1);
    h.edit_actor(actor, Native, |actor| actor.variables[0] = timer - 1);
    for (table, registers) in [
        (
            [0, 2, 2, 2, 0, -2, -2, -2],
            [ram::reg_BG1HOFS, ram::reg_BG2HOFS],
        ),
        (
            [-2, -2, 0, 2, 2, 2, 0, -2],
            [ram::reg_BG1VOFS, ram::reg_BG2VOFS],
        ),
    ] {
        let shift = table[index] as u16;
        for register in registers {
            h.write_word_le(h.word(register).wrapping_add(shift), register);
        }
    }
}

unsafe fn advance_script(address: u32, h: &mut Runtime<'_>) {
    use crate::game::{constants::bank_86::*, scrolling_sky};
    unsafe {
        match address {
            9421078 => h.foreign(|game| scrolling_sky::queue_land_rows(game)),
            9421083 => h.foreign(|game| scrolling_sky::queue_ocean_rows(game)),
            9421088 | 9421092 => {
                if address == 9421088 {
                    h.foreign(|game| scrolling_sky::queue_land_rows(game));
                }
                random_explosions(1, true, h);
                h.write_word_le(
                    h.word(ram::earthquake_timer) | 0x8000,
                    ram::earthquake_timer,
                );
            }
            9421286 => {
                if h.byte(0xcd2b) == 2
                    && h.word(ram::layer1_y_pos) < 0x500
                    && h.word(ram::layer1_x_pos) < 0x380
                {
                    h.write_word_le(h.word(ram::layer1_x_pos) + 3, ram::layer1_x_pos);
                }
            }
            9429686 => advance_elevatube(h),
            9430303 => h.foreign(|game| advance_ceres_elevator(game)),
            9430308 => {}
            9430309 => {
                if h.word(ram::ceres_status) != 0 {
                    h.write_slice_le(0x7e1, h.word(0x7e1).wrapping_sub(1));
                    if (h.native_word(0x7e1) as i16) < 0 {
                        h.write_slice_le(0x7e1, 8);
                        let definition = if h.word(ram::random_number) & 0x8000 != 0 {
                            addr_stru_869742
                        } else {
                            addr_stru_869734
                        } as u16;
                        let x = [
                            0x50, 0x60, 0x70, 0x80, 0x90, 0xa0, 0xb0, 0xc0, 0xd0, 0xe0, 0xf0,
                            0x110, 0x130, 0x150, 0x170, 0x190,
                        ][usize::from(h.word(ram::random_number) & 15)];
                        h.foreign(|game| {
                            crate::game::enemy_projectiles::SpawnEprojWithRoomGfx(
                                definition, x, game,
                            )
                        });
                    }
                }
            }
            9430385 => {
                if h.word(ram::ceres_status) & 1 != 0 {
                    h.foreign(|game| crate::game::sm_a6::HandleCeresRidleyGetawayCutscene(game));
                }
            }
            9430396 => {
                if let Some(kind) = select_earthquake(18, 0x200, 23, h) {
                    h.write_word_le(kind, ram::earthquake_type);
                }
                random_explosions(3, false, h);
            }
            9430432 => random_explosions(3, false, h),
            9430436 => {
                if let Some(kind) = select_earthquake(21, 0x180, 26, h) {
                    h.write_slice_native(0x7e3, kind);
                }
                h.write_word_le(h.native_word(0x7e3), ram::earthquake_type);
                random_explosions(3, false, h);
            }
            9431245 => shake_crocomire(h),
            9431376 => shake_ridley(h),
            _ => {
                h.foreign(|game| crate::rtl::Unreachable(game));
            }
        }
    }
}
