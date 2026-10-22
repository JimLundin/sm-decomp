//! Room and door metadata, ordered state selection and setup dispatch.
//! Checked records retain their cartridge allocation; RAM owns the live state.
use crate::game::progress_flags::{CheckBossBitForCurArea, CheckEventHappened};
use crate::game::sm_80::{
    LoadMirrorOfExploredMapTiles, SaveExploredMapTilesToSaved, SetElevatorsAsUsed,
};
use crate::game::sm_8f::{CallDoorDefSetupCode, CallRoomSetupCode};
use crate::game_state::Game;
use crate::ram_map::checked_word;
use crate::rtl::{Mult8x8, Unreachable};
use crate::types::{DoorDef as Door, RoomDefHeader as Header, RoomDefRoomstate as State};
use core::mem::offset_of;

const room_record: unsafe fn(u16, &mut Game) -> crate::rom::RomRecord =
    crate::rtl::rom_bank_record::<0x8f>;

const door_record: unsafe fn(u16, &mut Game) -> crate::rom::RomRecord =
    crate::rtl::rom_bank_record::<0x83>;

pub(crate) unsafe fn select_entry(address: u32, cursor: u16, game: &mut Game) -> u16 {
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

pub(crate) unsafe fn select(room: u16, game: &mut Game) {
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

pub(crate) unsafe fn load_header(game: &mut Game) {
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
pub(crate) unsafe fn load_state(game: &mut Game) {
    let room_state = room_record(game.roomdefroomstate_ptr(), game);
    let graphics_set = usize::from(room_state.byte(offset_of!(State, graphics_set)));
    let tileset_ptr = game.read_rom_word(0x8fe7a7, 2 * graphics_set);
    let tileset = room_record(tileset_ptr, game);
    game.set_tileset_tile_table_pointer(tileset.long(0));
    game.set_tileset_tiles_pointer(tileset.long(3));
    game.set_tileset_compr_palette_ptr(tileset.long(6));
    game.set_room_compr_level_data_ptr(room_state.long(offset_of!(State, compressed_room_map_ptr)));
    game.set_room_music_data_index(u16::from(room_state.byte(offset_of!(State, music_track))));
    game.set_room_music_track_index(u16::from(room_state.byte(offset_of!(State, music_control))));
    game.set_room_layer3_asm_ptr(room_state.word(offset_of!(State, room_layer3_fx_ptr)));
    game.set_room_enemy_population_ptr(room_state.word(offset_of!(State, enemy_population_ptr_)));
    game.set_room_enemy_tilesets_ptr(room_state.word(offset_of!(State, enemy_tilesets_ptr)));
    let [scroll_x, scroll_y] = room_state
        .word(offset_of!(State, vertical_screen_nudge_limit))
        .to_le_bytes();
    game.set_layer2_scroll_x_and_y(scroll_x, scroll_y);
    game.set_room_main_code_ptr(room_state.word(offset_of!(State, main_code_ptr)));
}

pub(crate) unsafe fn load_destination_cre(game: &mut Game) {
    let destination =
        door_record(game.door_def_ptr(), game).word(offset_of!(Door, room_definition_ptr));
    game.set_previous_cre_bitset(game.cre_bitset());
    let room = room_record(destination, game);
    game.set_cre_bitset(u16::from(room.byte(offset_of!(Header, cre_bitset_))));
}

pub(crate) unsafe fn load_door(game: &mut Game) {
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

pub(crate) unsafe fn save_departing_map(game: &mut Game) {
    if game.elevator_door_properties_orientation() & 0x000f != 0 {
        SetElevatorsAsUsed(game);
    }
    let door = door_record(game.door_def_ptr(), game);
    if door.byte(offset_of!(Door, door_bitflags)) & 0x40 != 0 {
        SaveExploredMapTilesToSaved(game);
    }
}

pub(crate) unsafe fn load_destination_map(game: &mut Game) {
    let door = door_record(game.door_def_ptr(), game);
    if door.byte(offset_of!(Door, door_bitflags)) & 0x40 != 0 {
        LoadMirrorOfExploredMapTiles(game);
    }
}

pub(crate) unsafe fn run_door_setup(game: &mut Game) {
    let code = door_record(game.door_def_ptr(), game).word(offset_of!(Door, door_setup_code));
    if code != 0 {
        CallDoorDefSetupCode(0x8f0000 | u32::from(code), game);
    }
}

pub(crate) unsafe fn run_room_setup(game: &mut Game) {
    let code =
        room_record(game.roomdefroomstate_ptr(), game).word(offset_of!(State, room_setup_code));
    if code != 0 {
        CallRoomSetupCode(0x8f0000 | u32::from(code), game);
    }
}
