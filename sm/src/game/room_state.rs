//! Room and door metadata, ordered state selection and setup dispatch.
//! Checked records retain their cartridge allocation; RAM owns the live state.
use crate::game::decompression::decompress_to_ram;
use crate::game::plm_spawning::SpawnRoomPLM;
use crate::game::progress_flags::{CheckBossBitForCurArea, CheckEventHappened};
use crate::game::sm_80::{
    CopyToVramNow, DecompressToVRAM, LoadMirrorOfExploredMapTiles, SaveExploredMapTilesToSaved,
    SetElevatorsAsUsed,
};
use crate::game::sm_8f::{CallDoorDefSetupCode, CallRoomSetupCode};
use crate::game_state::Game;
use crate::ram_map::checked_word;
use crate::rtl::{rom_bank_record, Mult8x8, Unreachable, WriteRegWord};
use crate::snes::registers::{VMADDL, VMAIN};
use crate::types::{DoorDef as Door, RoomDefHeader as Header, RoomDefRoomstate as State};
use crate::types::{EnemyDef, EnemyTileset};
use core::mem::offset_of;

const room_record: unsafe fn(u16, &mut Game) -> crate::rom::RomRecord =
    crate::rtl::rom_bank_record::<0x8f>;

const door_record: unsafe fn(u16, &mut Game) -> crate::rom::RomRecord =
    crate::rtl::rom_bank_record::<0x83>;

pub(crate) unsafe fn select_entry(address: u32, cursor: u16, game: &mut Game) -> u16 {
    unsafe {
        let state = match address {
            9430502 => cursor, // The default state follows this opcode inline.
            9430546 | 9430569 => {
                let record = room_record(cursor, game);
                let selected = if address == 9430546 {
                    CheckEventHappened(u16::from(record.byte(0)), game) != 0
                } else {
                    CheckBossBitForCurArea(u16::from(record.byte(0)), game) & 1 != 0
                };
                if !selected {
                    return cursor.wrapping_add(3);
                }
                record.word(1)
            }
            9430527 | 9430610 | 9430633 => {
                let selected = match address {
                    9430527 => CheckBossBitForCurArea(1, game) & 1 != 0,
                    9430610 => {
                        checked_word!(game.collected_items) & 4 != 0
                            && checked_word!(game.samus_max_missiles) != 0
                    }
                    _ => checked_word!(game.samus_max_power_bombs) != 0,
                };
                if !selected {
                    return cursor.wrapping_add(2);
                }
                room_record(cursor, game).word(0)
            }
            _ => return Unreachable(game) as u16,
        };
        checked_word!(game.roomdefroomstate_ptr = state);
        0
    }
}

pub(crate) unsafe fn select(room: u16, game: &mut Game) {
    unsafe {
        let mut cursor = room.wrapping_add(11);
        loop {
            let record = room_record(cursor, game);
            let opcode = record.word(0);
            cursor = select_entry(0x8f0000 | u32::from(opcode), cursor.wrapping_add(2), game);
            if cursor == 0 {
                return;
            }
        }
    }
}

pub(crate) unsafe fn load_header(game: &mut Game) {
    unsafe {
        let room = room_record(game.room_ptr(), game);
        game.set_room_index(u16::from(
            room.byte(offset_of!(Header, semiunique_room_number)),
        ));
        game.set_area_index(u16::from(room.byte(offset_of!(Header, area_index_))));
        game.set_room_x_coordinate_on_map(u16::from(
            room.byte(offset_of!(Header, x_coordinate_on_map)),
        ));
        game.set_room_y_coordinate_on_map(u16::from(
            room.byte(offset_of!(Header, y_coordinate_on_map)),
        ));
        game.set_room_width_in_scrolls(u16::from(room.byte(offset_of!(Header, width))));
        game.set_room_width_in_blocks(game.room_width_in_scrolls().wrapping_mul(16));
        game.set_room_height_in_scrolls(u16::from(room.byte(offset_of!(Header, height))));
        game.set_room_height_in_blocks(game.room_height_in_scrolls().wrapping_mul(16));
        game.set_up_scroller(u16::from(room.byte(offset_of!(Header, up_scroller_))));
        game.set_down_scroller(u16::from(room.byte(offset_of!(Header, down_scroller_))));
        game.set_door_list_pointer(room.word(offset_of!(Header, ptr_to_doorout)));
        select(game.room_ptr(), game);
        let product = Mult8x8(
            game.room_width_in_blocks() as u8,
            game.room_height_in_blocks() as u8,
        );
        game.set_room_size_in_blocks(product.wrapping_mul(2));
    }
}
pub(crate) unsafe fn load_state(game: &mut Game) {
    unsafe {
        let room_state = room_record(game.roomdefroomstate_ptr(), game);
        let graphics_set = usize::from(room_state.byte(offset_of!(State, graphics_set)));
        let tileset_ptr = game.read_rom_word(0x8fe7a7, 2 * graphics_set);
        let tileset = room_record(tileset_ptr, game);
        game.set_tileset_tile_table_pointer(tileset.long(0));
        game.set_tileset_tiles_pointer(tileset.long(3));
        game.set_tileset_compr_palette_ptr(tileset.long(6));
        game.set_room_compr_level_data_ptr(
            room_state.long(offset_of!(State, compressed_room_map_ptr)),
        );
        game.set_room_music_data_index(u16::from(room_state.byte(offset_of!(State, music_track))));
        game.set_room_music_track_index(u16::from(
            room_state.byte(offset_of!(State, music_control)),
        ));
        game.set_room_layer3_asm_ptr(room_state.word(offset_of!(State, room_layer3_fx_ptr)));
        game.set_room_enemy_population_ptr(
            room_state.word(offset_of!(State, enemy_population_ptr_)),
        );
        game.set_room_enemy_tilesets_ptr(room_state.word(offset_of!(State, enemy_tilesets_ptr)));
        let [scroll_x, scroll_y] = room_state
            .word(offset_of!(State, vertical_screen_nudge_limit))
            .to_le_bytes();
        game.set_layer2_scroll_x_and_y(scroll_x, scroll_y);
        game.set_room_main_code_ptr(room_state.word(offset_of!(State, main_code_ptr)));
    }
}

pub(crate) unsafe fn load_destination_cre(game: &mut Game) {
    unsafe {
        let destination =
            door_record(game.door_def_ptr(), game).word(offset_of!(Door, room_definition_ptr));
        game.set_previous_cre_bitset(game.cre_bitset());
        let room = room_record(destination, game);
        game.set_cre_bitset(u16::from(room.byte(offset_of!(Header, cre_bitset_))));
    }
}

pub(crate) unsafe fn load_door(game: &mut Game) {
    unsafe {
        let door = door_record(game.door_def_ptr(), game);
        game.set_room_ptr(door.word(offset_of!(Door, room_definition_ptr)));
        // Properties explicitly combine little-endian bytes, unlike native ROM words.
        let properties = u16::from_le_bytes([
            door.byte(offset_of!(Door, door_bitflags)),
            door.byte(offset_of!(Door, door_orientation)),
        ]);
        game.set_elevator_door_properties_orientation(properties);
        game.set_elevator_flags(game.elevator_door_properties_orientation() & 0x0080);
        game.set_door_direction(u16::from(door.byte(offset_of!(Door, door_orientation))));
        game.set_door_destination_x_pos(u16::from(door.byte(offset_of!(Door, x_pos_in_room))) << 8);
        game.set_door_destination_y_pos(u16::from(door.byte(offset_of!(Door, y_pos_in_room))) << 8);
        let mut distance = door.word(offset_of!(Door, samus_distance_from_door)) as i16;
        if distance < 0 {
            distance = if game.door_direction() & 2 != 0 {
                384
            } else {
                200
            };
        }
        let speed = (i32::from(distance) << 8) as u32;
        game.set_samus_door_transition_subspeed(speed as u16);
        game.set_samus_door_transition_speed((speed >> 16) as u16);
    }
}

pub(crate) unsafe fn save_departing_map(game: &mut Game) {
    unsafe {
        if game.elevator_door_properties_orientation() & 0x000f != 0 {
            SetElevatorsAsUsed(game);
        }
        let door = door_record(game.door_def_ptr(), game);
        if door.byte(offset_of!(Door, door_bitflags)) & 0x40 != 0 {
            SaveExploredMapTilesToSaved(game);
        }
    }
}

pub(crate) unsafe fn load_destination_map(game: &mut Game) {
    unsafe {
        let door = door_record(game.door_def_ptr(), game);
        if door.byte(offset_of!(Door, door_bitflags)) & 0x40 != 0 {
            LoadMirrorOfExploredMapTiles(game);
        }
    }
}

pub(crate) unsafe fn run_door_setup(game: &mut Game) {
    unsafe {
        let code = door_record(game.door_def_ptr(), game).word(offset_of!(Door, door_setup_code));
        if code != 0 {
            CallDoorDefSetupCode(0x8f0000 | u32::from(code), game);
        }
    }
}

pub(crate) unsafe fn run_room_setup(game: &mut Game) {
    unsafe {
        let code =
            room_record(game.roomdefroomstate_ptr(), game).word(offset_of!(State, room_setup_code));
        if code != 0 {
            CallRoomSetupCode(0x8f0000 | u32::from(code), game);
        }
    }
}

/// Load scroll data and retain the selected record for the initial PLM list.
/// Door loading reacquires that record later, after its scrolling suspension.
unsafe fn load_scrolls<const INITIAL: bool>(game: &mut Game) -> crate::rom::RomRecord {
    unsafe {
        let state = room_record(game.roomdefroomstate_ptr(), game);
        let mut cursor = state.word(offset_of!(State, rdf_scroll_ptr));
        if cursor & 0x8000 == 0 {
            generate_scrolls::<INITIAL>(cursor as u8, game);
        } else {
            for offset in (0..50).step_by(2) {
                let value = room_record(cursor, game).word(0);
                game.g_ram[0xcd20 + offset..0xcd22 + offset].copy_from_slice(&value.to_ne_bytes());
                cursor = cursor.wrapping_add(2);
            }
        }
        state
    }
}

#[forbid(unsafe_code)]
fn generate_scrolls<const INITIAL: bool>(bottom_color: u8, game: &mut Game) {
    let bottom_row = (game.room_height_in_scrolls() as u8).wrapping_sub(1);
    let (mut row, mut cell, mut color) = (0i8, 0u8, 2u8);
    loop {
        if i32::from(row) == i32::from(bottom_row) {
            color = bottom_color.wrapping_add(1);
        }
        let mut column = 0i8;
        loop {
            game.g_ram[0xcd20 + usize::from(cell)] = color;
            cell = cell.wrapping_add(1);
            column += 1;
            if i32::from(column) == i32::from(game.room_width_in_scrolls() as u8) {
                break;
            }
        }
        // Initial loading tests the wrapped i8; door loading tests the wider sum.
        let next_row = if INITIAL {
            row += 1;
            i32::from(row)
        } else {
            let previous = row;
            row = row.wrapping_add(1);
            i32::from(previous) + 1
        };
        if next_row == i32::from(game.room_height_in_scrolls() as u8) {
            return;
        }
    }
}

// Legacy Load24 reads a native dword and masks its high byte, including the
// adjacent RAM byte in that read. Keep its byte order without an unaligned load.
#[forbid(unsafe_code)]
fn asset_pointer(offset: usize, game: &Game) -> u32 {
    u32::from_ne_bytes(game.g_ram[offset..offset + 4].try_into().unwrap()) & 0xffffff
}

#[forbid(unsafe_code)]
fn decompress_level(game: &mut Game) {
    decompress_to_ram(asset_pointer(0x7bd, game), 0x10000, game);
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

/// Initial entry creates objects now; door entry waits for the later setup phase.
pub(crate) unsafe fn load_level<const INITIAL: bool>(game: &mut Game) {
    unsafe {
        if INITIAL {
            word_transfers!(game, descending, 12800; le(level_data, 12799) <- value(0x8000));
        } else {
            word_transfers!(game, descending, 3200;
                le(level_data, 3199) <- value(0x8000),
                native(0x13200) <- value(0x8000),
                native(0x14b00) <- value(0x8000),
                native(0x16400) <- value(0x8000),
            );
        }
        decompress_level(game);
        if checked_word!(game.area_index) == 6 {
            decompress_to_ram(asset_pointer(0x7c0, game), 0xa000, game);
        } else {
            if INITIAL || checked_word!(game.cre_bitset) & 2 != 0 {
                decompress_to_ram(0xb9a09d, 0xa000, game);
            }
            decompress_to_ram(asset_pointer(0x7c0, game), 0xa800, game);
        }
        let state = load_scrolls::<INITIAL>(game);
        if INITIAL {
            initialize_objects::<true>(state, game);
        }
        crate::verify_probe::hit("room.level.loaded");
    }
}

pub(crate) unsafe fn initialize_door_objects(game: &mut Game) {
    unsafe {
        let state = room_record(game.roomdefroomstate_ptr(), game);
        initialize_objects::<false>(state, game);
    }
}

unsafe fn initialize_objects<const INITIAL: bool>(state: crate::rom::RomRecord, game: &mut Game) {
    unsafe {
        let mut cursor = i32::from(state.word(offset_of!(State, room_plm_header_ptr)));
        if cursor != 0 {
            while room_record(cursor as u16, game).word(0) != 0 {
                SpawnRoomPLM(cursor as u16, game);
                cursor = if INITIAL {
                    i32::from((cursor as u16).wrapping_add(6))
                } else {
                    cursor + 6
                };
            }
        }
        run_door_setup(game);
        run_room_setup(game);
        if checked_word!(game.elevator_flags) != 0 {
            checked_word!(game.elevator_status = 2);
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

pub(crate) unsafe fn load_tiles_and_palette(game: &mut Game) {
    unsafe {
        game.set_elevator_flags(0);
        crate::rtl::register_writes! { game;
            word(VMAIN as u16, 0x80); word(VMADDL as u16, 0x2800);
        }
        DecompressToVRAM(0xb98000, 0x5000, game);
        WriteRegWord(VMADDL as u16, 0, game);
        DecompressToVRAM(asset_pointer(0x7c3, game), 0, game);
        decompress_to_ram(asset_pointer(0x7c6, game), 0xc200, game);
    }
}

/// Synthetic regression for the actual decoder and room-array split.
/// Cases: overlapping large room, nonoverlap boundary, checked failure prefix.
#[forbid(unsafe_code)]
pub(crate) fn check_level_copy(case: u32) {
    use crate::rom::RomImage;
    use std::panic::{catch_unwind, AssertUnwindSafe};
    let size = match case {
        0 => 0x5a00usize,
        1 => 0x3c00,
        2 => 0x7000,
        _ => panic!("unknown room-copy case"),
    };
    let pattern =
        |i: usize, salt: u8| (i.wrapping_mul(73) as u8) ^ ((i >> 8) as u8).wrapping_mul(29) ^ salt;
    let mut game = Game::new();
    for (i, b) in game.g_ram.iter_mut().enumerate() {
        *b = pattern(i, 0x5d);
    }
    let mut raw = (size as u16).to_le_bytes().to_vec();
    // The malformed case intentionally decompresses only its two-byte header.
    if case != 2 {
        raw.extend((0..size).map(|i| pattern(i, 0x11)));
        raw.extend((0..size / 2).map(|i| pattern(i, 0x47)));
        raw.extend((0..size).map(|i| pattern(i, 0xa3)));
    }
    let mut compressed = Vec::new();
    for chunk in raw.chunks(1024) {
        let length = chunk.len() - 1;
        compressed.extend_from_slice(&[0xe0 | (length >> 8) as u8, length as u8]);
        compressed.extend_from_slice(chunk);
    }
    compressed.push(0xff);
    let rom = RomImage::zeroed(0x100000);
    for (i, b) in compressed.into_iter().enumerate() {
        rom.write(i, b);
    }
    game.loaded_rom = Some(rom);
    game.g_ram[0x7bd..0x7c1].copy_from_slice(&0x808000u32.to_ne_bytes());
    let mut expected = game.g_ram.to_vec();
    expected[0x10000..0x10000 + raw.len()].copy_from_slice(&raw);
    if case == 2 {
        // Source 1A802 reaches RAM's end after 57FE bytes; destination remains
        // in bounds. Forward copy retains exactly that successful prefix.
        let prefix = expected[0x1a802..].to_vec();
        expected[0x19602..0x19602 + prefix.len()].copy_from_slice(&prefix);
    } else {
        expected[0x19602..0x19602 + size].copy_from_slice(&raw[2 + size + size / 2..]);
        expected[0x16402..0x16402 + size / 2].copy_from_slice(&raw[2 + size..2 + size + size / 2]);
    }
    let outcome = catch_unwind(AssertUnwindSafe(|| decompress_level(&mut game)));
    assert_eq!(
        outcome.is_err(),
        case == 2,
        "room copy case {case} panic status"
    );
    let first = game.g_ram.iter().zip(&expected).position(|(a, b)| a != b);
    assert!(
        first.is_none(),
        "room copy case {case}: first RAM mismatch {first:x?}"
    );
    assert!(!game.g_fail, "room copy case {case}: invalid ROM access");
}
