//! Compatibility entry adapters and retained level/graphics backend leaves.
use crate::game::display_refresh::CopyToVramNow;
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;
use crate::ram_map::{checked_word, word, words as ram};
use crate::rtl::rom_bank_record;
use crate::types::{EnemyDef, EnemyTileset};
use core::mem::offset_of;
use sm_game::decompression::decompress_to_ram;
use sm_game::room_entry::{
    content::{LoadMode, SelectorCursor, SelectorKind},
    installation, selection,
};

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

pub(crate) fn load_door(game: &mut Game) {
    installation::load_door(&mut Runtime::new(game));
}

pub(crate) fn pause_draygon(game: &mut Game) {
    word!(game.IRQHANDLER_NEXT_HANDLER = 4);
}

pub(crate) fn unpause_draygon(game: &mut Game) -> crate::types::CoroutineRet {
    if word!(game.HDMA_OBJECT_CHANNELS_BITMASK[1]) == 8 {
        word!(game.IRQHANDLER_NEXT_HANDLER = 12);
    }
    0
}

pub(crate) fn setup_script(address: u32, game: &mut Game) {
    use crate::game::room_mechanisms::spawn_hardcoded;
    use crate::game::visual_runtime::initialize_room_haze;
    use crate::game::{animated_tiles, constants::bank_87::*};
    use crate::types::SpawnHardcodedPlmArgs;
    use sm_game::scrolling_sky;
    match address {
        9408916 => {
            spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x3d, 0xb, 0xbb30), game);
            word! {game; EARTHQUAKE_TYPE = 24; EARTHQUAKE_TIMER = u16::MAX; }
        }
        9408937 => spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x10, 0x87, 0xb964), game),
        9408946 => spawn_hardcoded(SpawnHardcodedPlmArgs::new(0xf, 0xa, 0xb9ed), game),
        9408957 => {
            word! {game; EARTHQUAKE_TYPE = 6; EARTHQUAKE_TIMER = u16::MAX; }
            scrolling_sky::initialize_land(game);
        }
        9408969 => scrolling_sky::initialize_land(game),
        9408974 => scrolling_sky::initialize_ocean(game),
        9408983 => {
            for animation in [
                ADDR_K_ANIMTILES_TOURIAN_STATUE_KRAID,
                ADDR_K_ANIMTILES_TOURIAN_STATUE_PHANTOON,
                ADDR_K_ANIMTILES_TOURIAN_STATUE_DRAYGON,
                ADDR_K_ANIMTILES_TOURIAN_STATUE_RIDLEY,
            ] {
                animated_tiles::spawn(animation as u16, game);
            }
        }
        9423048 => {
            crate::game::enemy_projectiles::spawn_with_room_gfx(
                crate::game::constants::bank_86::ADDR_K_EPROJ_PRE_PHANTOM_ROOM as u16,
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
            let at = 0xd908 + usize::from(word!(game.AREA_INDEX));
            let collected =
                u16::from_ne_bytes([game.ram().read_byte(at), game.ram().read_byte(at + 1)]) | 1;
            word_transfers!(game, ascending, 1; native(at) <- value(collected));
            word!(game.HAS_AREA_MAP = collected);
        }
        9423135 => {
            crate::game::gameplay_flag_runtime::mark_event(0xe, game);
            word! {game; EARTHQUAKE_TYPE = 18; EARTHQUAKE_TIMER = u16::MAX; }
        }
        9423155 | 9423187 => {
            let kind = if address == 9423187 {
                spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x10, 0x10, 0xb968), game);
                21
            } else {
                18
            };
            word!(game.EARTHQUAKE_TYPE = kind);
            {
                let mut runtime = Runtime::new(game);
                let room = &mut runtime.session_mut().room;
                room.set_main_native(1, kind);
                room.set_main_native(0, 0);
            }
            word!(game.EARTHQUAKE_TIMER = u16::MAX);
        }
        9423174 => {
            word! {game; EARTHQUAKE_TYPE = 21; EARTHQUAKE_TIMER = u16::MAX; }
        }
        9423214 => {
            spawn_hardcoded(SpawnHardcodedPlmArgs::new(0xf, 0x26, 0xba48), game);
            initialize_room_haze(game);
        }
        9423222 => initialize_room_haze(game),
        9423227 => {
            game.ram().write_byte(0x5d, 102);
            initialize_room_haze(game);
            word!(game.HDMA_DATA_TABLE_IN_CERES = 9);
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
                    (ADDR_K_ANIMTILES_WRECKED_SHIP_TRADMILL_RIGHT, 0xb64b)
                } else {
                    (ADDR_K_ANIMTILES_WRECKED_SHIP_TRADMILL_LEFT, 0xb64f)
                };
                animated_tiles::spawn(animation as u16, game);
                spawn_hardcoded(SpawnHardcodedPlmArgs::new(4, 9, mechanism), game);
            }
            9429612 | 9429649 => {
                {
                    let mut runtime = Runtime::new(game);
                    let room = &mut runtime.session_mut().room;
                    if address == 9429612 {
                        room.set_main_native(2, (-256i16) as u16);
                        room.set_main_native(1, 2496);
                        room.set_main_native(3, (-32i16) as u16);
                    } else {
                        // The original strings write little-endian bytes on every host.
                        room.set_main_native(2, 256);
                        room.main_variables[1] = 64;
                        room.main_variables[3] = 32;
                    }
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
                word! { game, 256; REG_M7A, REG_M7D }
                word! { game, 0; REG_M7B, REG_M7C }
                word! {game; REG_M7X = 128; REG_M7Y = 1008; IRQ_ENABLE_MODE7 = 1; }
                {
                    let mut runtime = Runtime::new(game);
                    let room = &mut runtime.session_mut().room;
                    room.main_variables[0] = 34;
                    room.main_variables[1] = 60;
                }
            }
            9430291 => {
                game.ram().write_byte(0x56, 9);
                word!(game.IRQ_ENABLE_MODE7 = 0);
            }
            _ => {
                if !crate::game::room_scrolls::apply(address, &mut Runtime::new(game)) {
                    crate::rtl::Unreachable(game);
                }
            }
        }
    }
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

pub(crate) unsafe fn load_enemy_graphics(game: &mut Game) {
    unsafe {
        let mut cursor = i32::from(checked_word!(game.ROOM_ENEMY_TILESETS_PTR));
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

use sm_game::room_state::advance_ceres_elevator;
pub(crate) use sm_game::room_state::{copy_level_planes, load_sprite_palettes};

impl sm_game::room_state::MainCodeHost for Runtime<'_> {
    fn command_player(&mut self, command: u16) {
        self.foreign(|game| crate::game::session_runtime::player_command(command, game));
    }
    fn ceres_rotation_step(&mut self, word: isize) -> sm_data::ceres_elevator::RotationStep {
        self.foreign(|game| sm_game::ceres_elevator::step(game, word))
    }
}
use sm_game::room_state::{advance_elevatube, shake_crocomire, shake_ridley};

#[path = "../verification/room_level_copy.rs"]
mod level_copy_checks;
pub(crate) use level_copy_checks::check_level_copy;

use crate::rom::RomRecord;
use sm_game::visual_host::{Content, Source};
use sm_game::visual_state::{PaletteBuffer, Weather};

fn apply_fx(h: &mut Runtime<'_>, definition: &FxRecord, palette: PaletteBuffer) {
    h.session_mut().room.liquid_height = definition.word(2);
    h.session_mut().room.liquid_target = definition.word(4);
    h.session_mut().room.liquid_velocity = definition.word(6);
    h.session_mut().room.liquid_delay = u16::from(definition.byte(8));
    h.write_word_le(
        u16::from(definition.byte(10)),
        ram::FX_LAYER_BLENDING_CONFIG_A,
    );
    h.write_word_le(
        u16::from(definition.byte(11)),
        ram::FX_LAYER_BLENDING_CONFIG_B,
    );
    h.session_mut().room.liquid_options = u16::from(definition.byte(12));
    let blend = definition.byte(15);
    for color in if blend == 0 { 27..28 } else { 25..28 } {
        let value = if blend == 0 {
            0
        } else {
            fx_table(h, 0x89aa02).word(usize::from(blend & !1) + 2 * (color - 25))
        };
        let visuals = &mut h.session_mut().presentation.visual_mut();
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
        FxRecord::Legacy(sm_game::menus::menu_tilemap::native_record(full, game))
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
    let address = checked_word!(game.ROOM_LAYER3_ASM_PTR).wrapping_add(16 * (index & 7));
    let definition = fx_definition(address, game);
    let mut h = Runtime::new(game);
    apply_fx(&mut h, &definition, PaletteBuffer::Current);
}

pub(crate) fn load_fx_header(game: &mut Game) {
    use crate::game::{animated_tiles, palette_effects, visual_runtime};
    use sm_game::{liquid_effects, scrolling_sky};
    let mut address = checked_word!(game.ROOM_LAYER3_ASM_PTR);
    if address == 0 {
        return;
    }
    loop {
        let door = fx_definition(address, game).word(0);
        if door == 0xffff {
            return;
        }
        if door == 0 || door == checked_word!(game.DOOR_DEF_PTR) {
            break;
        }
        address = address.wrapping_add(16);
    }
    let definition = fx_definition(address, game);
    let mut h = Runtime::new(game);
    apply_fx(&mut h, &definition, PaletteBuffer::Target);
    let kind = definition.byte(9);
    h.session_mut().room.liquid.kind = u16::from(kind);
    if kind != 0 {
        let tilemap = fx_table(&h, 0x83abf0).word(usize::from(kind & !1));
        h.session_mut().room.liquid.tilemap = tilemap;
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

pub(crate) unsafe fn advance_room(game: &mut Game) {
    let script = word!(game.ROOM_MAIN_CODE_PTR);
    if script != 0 {
        unsafe { advance_script(0x8f0000 | u32::from(script), &mut Runtime::new(game)) };
    }
}

unsafe fn random_explosions(mask: u16, check_tile: bool, h: &mut Runtime<'_>) {
    {
        if h.word(ram::TIME_IS_FROZEN_FLAG) != 0 || h.word(ram::NMI_FRAME_COUNTER_WORD) & mask != 0
        {
            return;
        }
        let random = h.foreign(|game| game.next_random());
        let layer1 = h.session().room.camera.layer1;
        let x = layer1[0].wrapping_add(random as u8 as u16);
        let y = layer1[1].wrapping_add(u16::from(random.to_ne_bytes()[1]));
        // X on entry to $8F:C1A9: 0 from the room main ASM dispatch, or the
        // block index the tile check ($8F:C131) left.
        let mut entry_x = 0;
        if check_tile {
            let row =
                u16::from((y >> 4) as u8) * u16::from(h.word(ram::ROOM_WIDTH_IN_BLOCKS) as u8);
            entry_x = row.wrapping_add(x >> 4).wrapping_mul(2);
            if h.native_word(0x10002 + usize::from(entry_x)) & 0x3ff == 255 {
                return;
            }
        }
        let random = h.foreign(|game| game.next_random()) & 15;
        if random < 8 {
            let sound = [0x24, 0, 0, 0x25, 0, 0, 0, 0][usize::from(random)];
            if sound != 0 {
                h.foreign(|game| crate::game::music::queue_sfx2_max6(sound, game));
            }
        }
        // $8F:C1B5 loads X only for a random value below 8; otherwise the
        // sprite object ID comes from the X it entered with.
        let index = if random < 8 { random } else { entry_x & 7 };
        let sprite = [3, 3, 9, 12, 12, 18, 18, 21][usize::from(index)];
        h.foreign(|game| crate::game::sprite_effect_runtime::create_sprite(x, y, sprite, 0, game));
    }
}

unsafe fn select_earthquake(
    ending: u16,
    threshold: u16,
    starting: u16,
    h: &mut Runtime<'_>,
) -> Option<u16> {
    let timer = h.session().room.main_native(0);
    if timer != 0 {
        let remaining = timer.wrapping_sub(1);
        h.session_mut().room.set_main_native(0, remaining);
        (remaining == 0).then_some(ending)
    } else if h.foreign(|game| game.next_random()) < threshold {
        h.session_mut().room.main_variables[0] = 42;
        Some(starting)
    } else {
        None
    }
}

unsafe fn advance_script(address: u32, h: &mut Runtime<'_>) {
    use crate::game::constants::bank_86::*;
    use sm_game::scrolling_sky;
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
                    h.word(ram::EARTHQUAKE_TIMER) | 0x8000,
                    ram::EARTHQUAKE_TIMER,
                );
            }
            9421286 => {
                if h.byte(0xcd2b) == 2
                    && h.session().room.camera.layer1[1] < 0x500
                    && h.session().room.camera.layer1[0] < 0x380
                {
                    let x = h.session().room.camera.layer1[0] + 3;
                    sm_game::camera::set_layer1(h, 0, x);
                }
            }
            9429686 => advance_elevatube(h),
            9430303 => advance_ceres_elevator(h),
            9430308 => {}
            9430309 => {
                if h.word(ram::CERES_STATUS) != 0 {
                    let room = &mut h.session_mut().room;
                    room.main_variables[0] = room.main_variables[0].wrapping_sub(1);
                    if (room.main_native(0) as i16) < 0 {
                        room.main_variables[0] = 8;
                        let definition = if h.word(ram::RANDOM_NUMBER) & 0x8000 != 0 {
                            ADDR_STRU_869742
                        } else {
                            ADDR_STRU_869734
                        } as u16;
                        let x = [
                            0x50, 0x60, 0x70, 0x80, 0x90, 0xa0, 0xb0, 0xc0, 0xd0, 0xe0, 0xf0,
                            0x110, 0x130, 0x150, 0x170, 0x190,
                        ][usize::from(h.word(ram::RANDOM_NUMBER) & 15)];
                        h.foreign(|game| {
                            crate::game::enemy_projectiles::spawn_with_room_gfx(definition, x, game)
                        });
                    }
                }
            }
            9430385 => {
                if h.word(ram::CERES_STATUS) & 1 != 0 {
                    h.foreign(sm_game::actors::ceres_ridley::handle_getaway_if_active);
                }
            }
            9430396 => {
                if let Some(kind) = select_earthquake(18, 0x200, 23, h) {
                    h.write_word_le(kind, ram::EARTHQUAKE_TYPE);
                }
                random_explosions(3, false, h);
            }
            9430432 => random_explosions(3, false, h),
            9430436 => {
                if let Some(kind) = select_earthquake(21, 0x180, 26, h) {
                    h.session_mut().room.set_main_native(1, kind);
                }
                h.write_word_le(h.session().room.main_native(1), ram::EARTHQUAKE_TYPE);
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
