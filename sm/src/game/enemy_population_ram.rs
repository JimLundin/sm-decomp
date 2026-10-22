//! Transitional population encoding; initializer callbacks observe committed slots.
#![deny(unsafe_op_in_unsafe_fn)]
use crate::game::constants::{bank_a0::*, bank_a4::*};
use crate::game::enemy_population::{
    Creation, EnemyShape, PopulationEntry, SearchResult, SpawnSearch, SpawnSequence,
};
use crate::game::sm_a0::CallEnemyAi;
use crate::game::sprite_objects::clear_sprites;
use crate::game_state::Game;
use crate::ram_map::checked_word as word;
use crate::rom::RomRecord as Record;
use crate::rtl::rom_record;
use crate::types::{EnemyData, EnemyDef, EnemySpawnData, EnemyTileset};
use core::mem::offset_of;

#[forbid(unsafe_code)]
fn read(game: &Game, address: usize) -> u16 {
    u16::from_ne_bytes([game.g_ram[address], game.g_ram[address + 1]])
}
#[forbid(unsafe_code)]
fn write(game: &mut Game, address: usize, value: u16) {
    game.g_ram[address..address + 2].copy_from_slice(&value.to_ne_bytes());
}
fn slot(index: u16) -> usize {
    0xf78 + usize::from(index)
}
fn spawn(index: u16) -> usize {
    0x7000 + usize::from(index)
}

impl Record {
    fn population(&self) -> PopulationEntry {
        PopulationEntry {
            definition: self.word(0),
            x: self.word(2),
            y: self.word(4),
            instruction: self.word(6),
            properties: self.word(8),
            extra_properties: self.word(10),
            parameters: [self.word(12), self.word(14)],
        }
    }
    fn shape(&self) -> EnemyShape {
        EnemyShape {
            width: self.word(offset_of!(EnemyDef, x_radius)),
            height: self.word(offset_of!(EnemyDef, y_radius)),
            health: self.word(offset_of!(EnemyDef, health)),
            layer: self.byte(offset_of!(EnemyDef, layer)),
            bank: self.byte(offset_of!(EnemyDef, bank)),
            hurt_time: self.byte(offset_of!(EnemyDef, hurt_ai_time)),
        }
    }
}

fn configure(index: u16, shape: EnemyShape, game: &mut Game) {
    for (offset, value) in [
        (offset_of!(EnemyData, x_width), shape.width),
        (offset_of!(EnemyData, y_height), shape.height),
        (offset_of!(EnemyData, health), shape.health),
        (offset_of!(EnemyData, layer), u16::from(shape.layer)),
        (
            offset_of!(EnemyData, bank),
            u16::from_ne_bytes([shape.bank, shape.hurt_time]),
        ),
    ] {
        write(game, slot(index) + offset, value);
    }
}
fn place(index: u16, entry: PopulationEntry, creation: Creation, game: &mut Game) {
    for (offset, value) in [
        (offset_of!(EnemyData, enemy_ptr), entry.definition),
        (offset_of!(EnemyData, x_pos), entry.x),
        (offset_of!(EnemyData, y_pos), entry.y),
        (
            offset_of!(EnemyData, current_instruction),
            entry.instruction,
        ),
        (offset_of!(EnemyData, properties), entry.properties),
        (
            offset_of!(EnemyData, extra_properties),
            entry.extra_properties,
        ),
        (offset_of!(EnemyData, parameter_1), entry.parameters[0]),
        (offset_of!(EnemyData, parameter_2), entry.parameters[1]),
    ] {
        write(game, slot(index) + offset, value);
    }
    write(game, slot(index) + offset_of!(EnemyData, frame_counter), 0);
    write(game, slot(index) + offset_of!(EnemyData, timer), 0);
    if creation.clear_script_variables() {
        game.g_ram[slot(index) + offset_of!(EnemyData, ai_var_A)
            ..slot(index) + offset_of!(EnemyData, parameter_1)]
            .fill(0);
    }
    write(
        game,
        slot(index) + offset_of!(EnemyData, instruction_timer),
        1,
    );
    write(game, slot(index) + offset_of!(EnemyData, frame_counter), 0);
}
unsafe fn finish_initialization(
    index: u16,
    definition: &Record,
    creation: Creation,
    game: &mut Game,
) {
    unsafe {
        record_spawn(index, game);
    }
    word!(game.cur_enemy_index = index);
    let function = definition.word(offset_of!(EnemyDef, ai_init));
    if creation.run_initializer(function) {
        let address =
            u32::from(definition.byte(offset_of!(EnemyDef, bank))) << 16 | u32::from(function);
        unsafe {
            CallEnemyAi(address, game);
        }
    }
    let properties = read(game, slot(index) + offset_of!(EnemyData, properties));
    let extra = read(game, slot(index) + offset_of!(EnemyData, extra_properties));
    let empty = match creation {
        Creation::Room => addr_kSpritemap_Nothing_A4,
        Creation::Dynamic => addr_kSpritemap_Nothing_A0,
    } as u16;
    if let Some(map) = creation.initial_spritemap(
        properties,
        extra,
        empty,
        addr_kExtendedSpritemap_Nothing_A4 as u16,
    ) {
        write(
            game,
            slot(index) + offset_of!(EnemyData, spritemap_pointer),
            map,
        );
    }
}

pub unsafe fn record_spawn(index: u16, game: &mut Game) {
    for (target, source) in [
        (
            offset_of!(EnemySpawnData, id),
            offset_of!(EnemyData, enemy_ptr),
        ),
        (
            offset_of!(EnemySpawnData, x_pos),
            offset_of!(EnemyData, x_pos),
        ),
        (
            offset_of!(EnemySpawnData, y_pos),
            offset_of!(EnemyData, y_pos),
        ),
        (
            offset_of!(EnemySpawnData, init_param),
            offset_of!(EnemyData, current_instruction),
        ),
        (
            offset_of!(EnemySpawnData, properties),
            offset_of!(EnemyData, properties),
        ),
        (
            offset_of!(EnemySpawnData, extra_properties),
            offset_of!(EnemyData, extra_properties),
        ),
        (
            offset_of!(EnemySpawnData, param_1),
            offset_of!(EnemyData, parameter_1),
        ),
        (
            offset_of!(EnemySpawnData, param_2),
            offset_of!(EnemyData, parameter_2),
        ),
    ] {
        write(
            game,
            spawn(index) + target,
            read(game, slot(index) + source),
        );
    }
    let definition = unsafe { rom_record(0xa0, read(game, slot(index)), game) };
    let name = definition.word(offset_of!(EnemyDef, name_ptr));
    let mut words = [0; 6];
    if name != 0 {
        let text = unsafe { rom_record(0xb4, name, game) };
        for (value, offset) in words.iter_mut().zip([0, 2, 4, 6, 8, 12]) {
            *value = text.word(offset);
        }
    }
    for (n, value) in words.into_iter().enumerate() {
        write(
            game,
            spawn(index) + offset_of!(EnemySpawnData, name) + 2 * n,
            value,
        );
    }
}

pub unsafe fn load_graphics(population: u16, index: u16, game: &mut Game) {
    let mut address = word!(game.room_enemy_tilesets_ptr);
    let mut tiles = 0u16;
    loop {
        let tileset = unsafe { rom_record(0xb4, address, game) };
        let population = unsafe { rom_record(0xa1, population, game) };
        let found = population.word(0) == tileset.word(0);
        if found || tileset.word(0) == 0xffff {
            let (palette, tiles) = if found {
                ((tileset.word(2) & 15) << 9, tiles)
            } else {
                (2560, 0)
            };
            let fields = if found {
                [
                    (
                        offset_of!(EnemySpawnData, palette_index),
                        offset_of!(EnemyData, palette_index),
                        palette,
                    ),
                    (
                        offset_of!(EnemySpawnData, vram_tiles_index),
                        offset_of!(EnemyData, vram_tiles_index),
                        tiles,
                    ),
                ]
            } else {
                [
                    (
                        offset_of!(EnemySpawnData, vram_tiles_index),
                        offset_of!(EnemyData, vram_tiles_index),
                        tiles,
                    ),
                    (
                        offset_of!(EnemySpawnData, palette_index),
                        offset_of!(EnemyData, palette_index),
                        palette,
                    ),
                ]
            };
            for (save_field, live_field, value) in fields {
                write(game, spawn(index) + save_field, value);
                write(
                    game,
                    slot(index) + live_field,
                    read(game, spawn(index) + save_field),
                );
            }
            return;
        }
        let definition = unsafe { rom_record(0xa0, tileset.word(0), game) };
        tiles = tiles.wrapping_add(definition.word(offset_of!(EnemyDef, tile_data_size)) >> 5);
        address = address.wrapping_add(4);
    }
}

pub unsafe fn initialize_room(game: &mut Game) {
    game.g_ram[0x7000..0x9800].fill(0);
    word!(game.num_enemies_in_room = 0);
    word!(game.num_enemies_killed_in_room = 0);
    word!(game.flag_process_all_enemies = 0);
    for i in (0..144).rev() {
        word!(game.eproj_flags[i] = 0);
    }
    for i in (0..18).rev() {
        word!(game.eproj_killed_enemy_index[i] = 0xffff);
    }
    let mut population = word!(game.room_enemy_population_ptr);
    if unsafe { rom_record(0xa1, word!(game.room_enemy_population_ptr), game) }.word(0) == 0xffff {
        return;
    }
    word!(game.UNUSED_word_7E0E48 = 0);
    let mut index = 0u16;
    loop {
        unsafe {
            load_graphics(population, index, game);
        }
        let entry = unsafe { rom_record(0xa1, population, game) };
        let definition = unsafe { rom_record(0xa0, entry.word(0), game) };
        configure(index, definition.shape(), game);
        let boss = definition.word(offset_of!(EnemyDef, boss_fight_value));
        if boss != 0 {
            word!(game.boss_id = boss);
        }
        place(index, entry.population(), Creation::Room, game);
        unsafe {
            finish_initialization(index, &definition, Creation::Room, game);
        }
        index = index.wrapping_add(64);
        population = population.wrapping_add(16);
        if unsafe { rom_record(0xa1, population, game) }.word(0) == 0xffff {
            break;
        }
    }
    word!(game.first_free_enemy_index = index);
    word!(game.num_enemies_in_room = index >> 6);
    let deaths = unsafe { rom_record(0xa1, population, game) }.byte(2);
    word!(game.num_enemy_deaths_left_to_clear = u16::from(deaths));
}

pub unsafe fn spawn_enemy(bank: u8, population: u16, game: &mut Game) -> u16 {
    let saved = word!(game.cur_enemy_index);
    let entry = unsafe { rom_record(bank, population, game) };
    let definition = unsafe { rom_record(0xa0, entry.word(0), game) };
    let parts = definition.word(offset_of!(EnemyDef, num_parts));
    let mut search = SpawnSearch::new(parts);
    let index = loop {
        match search.observe(read(game, slot(search.probe)) != 0) {
            SearchResult::Continue => {}
            SearchResult::Found(index) => break index,
            SearchResult::Full => return 0xffff,
        }
    };
    let mut sequence = SpawnSequence::new(index, population, parts);
    loop {
        let entry = unsafe { rom_record(bank, sequence.population, game) };
        let graphics = (0..4).find(|&i| entry.word(0) == word!(game.enemy_def_ptr[i]));
        let (tiles, palette) = graphics
            .map(|i| {
                (
                    word!(game.enemy_gfxdata_tiles_index[i]),
                    word!(game.enemy_gfxdata_vram_ptr[i])
                        .swap_bytes()
                        .wrapping_mul(2),
                )
            })
            .unwrap_or((0, 0));
        write(
            game,
            slot(sequence.slot) + offset_of!(EnemyData, vram_tiles_index),
            tiles,
        );
        write(
            game,
            slot(sequence.slot) + offset_of!(EnemyData, palette_index),
            palette,
        );
        let definition = unsafe { rom_record(0xa0, entry.word(0), game) };
        configure(sequence.slot, definition.shape(), game);
        place(sequence.slot, entry.population(), Creation::Dynamic, game);
        unsafe {
            finish_initialization(sequence.slot, &definition, Creation::Dynamic, game);
        }
        if !sequence.advance() {
            word!(game.cur_enemy_index = saved);
            return sequence.slot;
        }
    }
}

pub unsafe fn delete_connected(game: &mut Game) {
    let definition =
        unsafe { rom_record(0xa0, read(game, slot(word!(game.cur_enemy_index))), game) };
    let parts = definition.word(offset_of!(EnemyDef, num_parts)).max(1);
    for _ in 0..parts {
        write(game, slot(word!(game.cur_enemy_index)), 0);
        word!(game.cur_enemy_index += 64);
    }
}

// Packed compatibility load entries: size word, source long, RAM-offset word.
// The fifth entry overlaps the live write position; do not normalize these aliases.
const LOAD_BYTES: usize = 7;
const LOAD_SOURCE: usize = 2;
const LOAD_DESTINATION: usize = 5;

#[forbid(unsafe_code)]
fn copy_record<const N: usize>(
    record: &Record,
    source: usize,
    destination: usize,
    game: &mut Game,
) {
    let bytes: [u8; N] = core::array::from_fn(|i| record.byte(source + i));
    game.g_ram[destination..destination + N].copy_from_slice(&bytes);
}

pub(crate) unsafe fn load_room_assets(game: &mut Game) {
    word!(game.debug_time_frozen_for_enemies = 0);
    game.g_ram[0x178e] = 0xa0;
    write(game, 0x178c, 0x804c);
    word!(game.enemy_bg2_tilemap_size = 2048);
    word!(game.UNUSED_word_7E179E = 0);
    word!(game.UNUSED_word_7E17A0 = 0);
    word!(game.boss_id = 0);
    game.g_ram[0xf78..0x1778].fill(0);
    if unsafe { rom_record(0xa1, word!(game.room_enemy_population_ptr), game) }.word(0) != 0xffff {
        unsafe {
            stage_tilesets(game);
        }
    }
    unsafe {
        load_tile_data(game);
    }
    word!(game.enemy_tile_vram_src = 0);
    word!(game.flag_disable_projectile_interaction = 0);
    clear_sprites(game);
}

unsafe fn stage_tilesets(game: &mut Game) {
    word!(game.enemy_tile_load_data_write_pos = 0);
    // Preserve array-major clearing before acquiring the retained tileset list.
    word_transfers!(game, ascending, 4; le(enemy_def_ptr, 0) <- value(0));
    word_transfers!(game, ascending, 4; le(enemy_gfxdata_tiles_index, 0) <- value(0));
    word_transfers!(game, ascending, 4; le(enemy_gfxdata_vram_ptr, 0) <- value(0));
    let mut tileset = unsafe { rom_record(0xb4, word!(game.room_enemy_tilesets_ptr), game) };
    let (mut load, mut graphics, mut tiles, mut next_ram) = (0xe5ausize, 0u16, 0u16, 2048u16);
    while tileset.word(offset_of!(EnemyTileset, enemy_def)) != 0xffff {
        let definition = unsafe { rom_record(0xa0, tileset.word(0), game) };
        // The native low byte selects the palette before the source diagnostic.
        let palette =
            0xc200 + (usize::from(tileset.byte(offset_of!(EnemyTileset, vram_dst))) + 8) * 32;
        let colors = unsafe {
            rom_record(
                definition.byte(offset_of!(EnemyDef, bank)),
                definition.word(offset_of!(EnemyDef, palette_ptr)),
                game,
            )
        };
        copy_record::<32>(&colors, 0, palette, game);
        write(
            game,
            load,
            definition.word(offset_of!(EnemyDef, tile_data_size)) & 0x7fff,
        );
        copy_record::<3>(
            &definition,
            offset_of!(EnemyDef, tile_data),
            load + LOAD_SOURCE,
            game,
        );
        let destination = if definition.word(offset_of!(EnemyDef, tile_data_size)) & 0x8000 != 0 {
            (tileset.word(offset_of!(EnemyTileset, vram_dst)) & 0x3000) >> 3
        } else {
            next_ram
        };
        write(game, load + LOAD_DESTINATION, destination);
        word!(game.enemy_tile_load_data_write_pos += LOAD_BYTES as u16);
        load += LOAD_BYTES;
        let slot = usize::from(graphics >> 1);
        write(game, 0xef64 + 2 * slot, tiles);
        word!(game.enemy_def_ptr[slot] = tileset.word(0));
        word!(game.enemy_gfxdata_vram_ptr[slot] = tileset.word(offset_of!(EnemyTileset, vram_dst)));
        graphics = graphics.wrapping_add(2);
        tiles = tiles.wrapping_add(definition.word(offset_of!(EnemyDef, tile_data_size)) >> 5);
        next_ram = next_ram.wrapping_add(definition.word(offset_of!(EnemyDef, tile_data_size)));
        // This allocation traversal is linear, unlike room_state's bank wrapping.
        tileset.base += core::mem::size_of::<EnemyTileset>();
    }
}

unsafe fn load_tile_data(game: &mut Game) {
    word_transfers!(game, descending, 256; native(0x71fe) <- rom(0x9ad200, 3327));
    if word!(game.enemy_tile_load_data_write_pos) == 0 {
        return;
    }
    let (mut load, mut consumed) = (0xe5ausize, 0u16);
    loop {
        let mut source = read(game, load + LOAD_SOURCE);
        let end = read(game, load).wrapping_add(source);
        let mut destination = read(game, load + LOAD_DESTINATION);
        let bank = game.g_ram[load + LOAD_SOURCE + 2];
        loop {
            let chunk = unsafe { rom_record(bank, source, game) };
            copy_record::<8>(&chunk, 0, 0x7000 + usize::from(destination), game);
            destination = destination.wrapping_add(8);
            source = source.wrapping_add(8);
            if source == end {
                break;
            }
        }
        load += LOAD_BYTES;
        consumed = consumed.wrapping_add(LOAD_BYTES as u16);
        if consumed == word!(game.enemy_tile_load_data_write_pos) {
            break;
        }
    }
    word!(game.enemy_tile_load_data_write_pos = 0);
}
