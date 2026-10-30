//! Ordered room installation with live captures and shared state.
use super::{
    content::{
        AssetSource, DoorScalar, HeaderScalar, LoadMode, ScrollDefinition, SharedAsset,
        StateCapture, StateScalar, TilesetAsset, VramDestination,
    },
    effects::{
        AssetRead, AssetSlot, BufferClear, DeviceWord, EntryHost, InstallBuffer, LeafEffect,
        VramDecodeTarget, VramSlot,
    },
    selection,
};

pub fn load_header(h: &mut impl EntryHost) {
    let room = h.session().room.entry.room;
    let header = h.capture_header(&room);
    h.session_mut().room.entry.room_index = header.scalar(HeaderScalar::Index);
    h.session_mut().room.area = header.scalar(HeaderScalar::Area);
    h.session_mut().room.entry.map_x = header.scalar(HeaderScalar::MapX);
    h.session_mut().room.entry.map_y = header.scalar(HeaderScalar::MapY);
    h.session_mut().room.scroll_width = header.scalar(HeaderScalar::Width);
    h.session_mut().room.width = h.session().room.scroll_width.wrapping_mul(16);
    h.session_mut().room.scroll_height = header.scalar(HeaderScalar::Height);
    h.session_mut().room.height = h.session().room.scroll_height.wrapping_mul(16);
    h.session_mut().room.entry.scroll_up = header.scalar(HeaderScalar::UpScroll);
    h.session_mut().room.entry.scroll_down = header.scalar(HeaderScalar::DownScroll);
    h.session_mut().room.entry.doors = header.doors();
    let room = h.session().room.entry.room;
    selection::select(h, room);
    let product =
        u16::from(h.session().room.width as u8) * u16::from(h.session().room.height as u8);
    h.session_mut().room.size_bytes = product.wrapping_mul(2);
}

pub fn load_state(h: &mut impl EntryHost) {
    let state = h.session().room.entry.state;
    let record = h.capture_state(&state);
    let tileset = h.capture_tileset(record.tileset());
    h.session_mut().room.entry.tile_table = tileset.asset(TilesetAsset::TileTable);
    h.session_mut().room.entry.tiles = tileset.asset(TilesetAsset::Tiles);
    h.session_mut().room.entry.palette = tileset.asset(TilesetAsset::Palette);
    h.session_mut().room.entry.level = record.level();
    h.session_mut().room.entry.music_data = record.scalar(StateScalar::MusicData);
    h.session_mut().room.entry.music_track = record.scalar(StateScalar::MusicTrack);
    h.session_mut().room.entry.fx_key = record.scalar(StateScalar::Fx);
    h.session_mut().room.entry.enemy_population_key = record.scalar(StateScalar::EnemyPopulation);
    h.session_mut().room.entry.enemy_tilesets_key = record.scalar(StateScalar::EnemyTilesets);
    let [x, y] = record.scalar(StateScalar::Layer2Policy).to_le_bytes();
    h.session_mut().room.entry.layer2_x = x;
    h.session_mut().room.entry.layer2_y = y;
    h.session_mut().room.entry.main_code_key = record.scalar(StateScalar::MainCode);
}

pub fn load_destination_cre(h: &mut impl EntryHost) {
    let exit = h.session().room.entry.exit;
    let destination = h.capture_door(&exit).room();
    h.session_mut().room.entry.previous_cre = h.session().room.entry.cre;
    let header = h.capture_header(&destination);
    h.session_mut().room.entry.cre = header.scalar(HeaderScalar::Cre);
}

pub fn load_door(h: &mut impl EntryHost) {
    let exit = h.session().room.entry.exit;
    let door = h.capture_door(&exit);
    h.session_mut().room.entry.room = door.room();
    h.session_mut().room.entry.elevator_properties = door.scalar(DoorScalar::Properties);
    h.session_mut().room.entry.elevator_flags = h.session().room.entry.elevator_properties & 0x80;
    h.session_mut().room.entry.direction = door.scalar(DoorScalar::Direction);
    h.session_mut().room.entry.arrival_x = door.scalar(DoorScalar::ArrivalX) << 8;
    h.session_mut().room.entry.arrival_y = door.scalar(DoorScalar::ArrivalY) << 8;
    let mut distance = door.scalar(DoorScalar::Distance) as i16;
    if distance < 0 {
        distance = if h.session().room.entry.direction & 2 != 0 {
            384
        } else {
            200
        };
    }
    let speed = (i32::from(distance) << 8) as u32;
    h.session_mut().room.entry.speed_fraction = speed as u16;
    h.session_mut().room.entry.speed_whole = (speed >> 16) as u16;
}

pub fn save_departing_map(h: &mut impl EntryHost) {
    if h.session().room.entry.elevator_properties & 15 != 0 {
        h.call(LeafEffect::MarkElevatorsUsed);
    }
    let exit = h.session().room.entry.exit;
    let door = h.capture_door(&exit);
    if door.scalar(DoorScalar::MapFlags) & 0x40 != 0 {
        h.call(LeafEffect::SaveExploredMap);
    }
}

pub fn load_destination_map(h: &mut impl EntryHost) {
    let exit = h.session().room.entry.exit;
    let door = h.capture_door(&exit);
    if door.scalar(DoorScalar::MapFlags) & 0x40 != 0 {
        h.call(LeafEffect::LoadExploredMap);
    }
}

pub fn run_door_setup(h: &mut impl EntryHost) {
    let exit = h.session().room.entry.exit;
    if let Some(action) = h.capture_door(&exit).setup() {
        h.setup(action);
    }
}

pub fn run_room_setup(h: &mut impl EntryHost) {
    let state = h.session().room.entry.state;
    if let Some(action) = h.capture_state(&state).setup() {
        h.setup(action);
    }
}

fn generate_scrolls(h: &mut impl EntryHost, bottom_color: u8, mode: LoadMode) {
    let bottom_row = (h.session().room.scroll_height as u8).wrapping_sub(1);
    let (mut row, mut cell, mut color) = (0i8, 0u8, 2u8);
    loop {
        if i32::from(row) == i32::from(bottom_row) {
            color = bottom_color.wrapping_add(1);
        }
        let mut column = 0i8;
        loop {
            h.write_scroll_byte(cell, color);
            cell = cell.wrapping_add(1);
            column += 1;
            if i32::from(column) == i32::from(h.session().room.scroll_width as u8) {
                break;
            }
        }
        let next_row = if mode == LoadMode::Initial {
            row += 1;
            i32::from(row)
        } else {
            let previous = row;
            row = row.wrapping_add(1);
            i32::from(previous) + 1
        };
        if next_row == i32::from(h.session().room.scroll_height as u8) {
            return;
        }
    }
}

fn load_scrolls(h: &mut impl EntryHost, mode: LoadMode) -> StateCapture {
    let state = h.session().room.entry.state;
    let record = h.capture_state(&state);
    match record.scrolls() {
        ScrollDefinition::Generated(color) => generate_scrolls(h, color, mode),
        ScrollDefinition::Records(mut cursor) => {
            for offset in (0..50).step_by(2) {
                let pair = h.scroll_pair(&cursor);
                h.write_scroll_pair(offset, pair);
                cursor = cursor.next();
            }
        }
    }
    record
}

/// Initial objects use the state capture retained across scroll installation.
pub fn load_level(h: &mut impl EntryHost, mode: LoadMode) {
    h.clear_buffer(if mode == LoadMode::Initial {
        BufferClear::InitialLevel
    } else {
        BufferClear::DoorLevel
    });
    let level = h.live_asset(AssetSlot::Level, AssetRead::FourNativeMasked);
    h.decode_level_and_copy_planes(level);
    if h.session().room.area == 6 {
        let table = h.live_asset(AssetSlot::TileTable, AssetRead::FourNativeMasked);
        h.decompress(table, InstallBuffer::TileTablePrimary);
    } else {
        if mode == LoadMode::Initial || h.session().room.entry.cre & 2 != 0 {
            h.decompress(
                AssetSource::Shared(SharedAsset::CreTileTable),
                InstallBuffer::TileTablePrimary,
            );
        }
        let table = h.live_asset(AssetSlot::TileTable, AssetRead::FourNativeMasked);
        h.decompress(table, InstallBuffer::TileTableAfterCre);
    }
    let record = load_scrolls(h, mode.clone());
    if mode == LoadMode::Initial {
        initialize_objects(h, record, mode);
    }
    crate::probe::hit("room.level.loaded");
}

pub fn initialize_door_objects(h: &mut impl EntryHost) {
    let state = h.session().room.entry.state;
    let record = h.capture_state(&state);
    initialize_objects(h, record, LoadMode::Door);
}

fn initialize_objects(h: &mut impl EntryHost, record: StateCapture, mode: LoadMode) {
    if let Some(mut cursor) = record.objects(mode.clone()) {
        while h.object_exists(&cursor) {
            h.spawn_room_object(&cursor);
            cursor = cursor.next(mode.clone());
        }
    }
    run_door_setup(h);
    run_room_setup(h);
    if h.session().room.entry.elevator_flags != 0 {
        h.session_mut().room.entry.elevator_status = 2;
    }
}

pub fn load_tiles_and_palette(h: &mut impl EntryHost) {
    h.session_mut().room.entry.elevator_flags = 0;
    h.device_word(DeviceWord::VramIncrement, 0x80);
    h.device_vram_address(VramDestination::Slot(VramSlot::CreFirst));
    h.decompress_vram(
        AssetSource::Shared(SharedAsset::CreTiles),
        VramDecodeTarget::Cre,
    );
    h.device_vram_address(VramDestination::Slot(VramSlot::MainTiles));
    let tiles = h.live_asset(AssetSlot::Tiles, AssetRead::FourNativeMasked);
    h.decompress_vram(tiles, VramDecodeTarget::MainTiles);
    let palette = h.live_asset(AssetSlot::Palette, AssetRead::FourNativeMasked);
    h.decompress(palette, InstallBuffer::Palette);
}
