//! The gameplay words a finished demo clears ($7E:077C..$7E:0997), by the
//! owners that hold their bytes.
//!
//! The original clears the block a word at a time, from $0996 down to
//! $077C. Most of its owners are odd-aligned words, so most of the cleared
//! words straddle two owners: [`session_word`] reads each byte from its own
//! owner, and the projection's [`crate::projection::Field::DemoClear`] key
//! names each word.
use crate::session::Session;

/// The words the block holds.
pub const WORDS: usize = 270;

/// Where the block starts.
pub const FIRST: usize = 0x77c;

/// An owner of the block's bytes, in address order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Owner {
    /// The HUD's item tilemap palette bits ($077C), which only an owned
    /// display holds.
    HudPalette,
    MessageDelay,
    UnpauseQueueTail,
    /// The NMI's coroutine completion byte ($0782), which only an owned
    /// display holds.
    NmiAcknowledgment,
    Mode7Irq,
    /// Words no Session owner holds: the byte path stores their RAM.
    Unowned(usize),
    MapUnread,
    HasAreaMap,
    LoadStation,
    DoorExit,
    DoorDirection,
    ElevatorProperties,
    ElevatorInterlock,
    EnemyInterlock,
    ElevatorDirection,
    RoomHeader,
    RoomIndex,
    Area,
    MapX,
    MapY,
    Width,
    Height,
    ScrollWidth,
    ScrollHeight,
    ScrollUp,
    ScrollDown,
    PreviousCre,
    CurrentCre,
    DoorList,
    SizeBytes,
    RoomState,
    /// The room's asset long pointers ($07BD..$07C8): the level data, the
    /// tile table, the tiles and the palette.
    LevelData,
    TileTable,
    Tiles,
    Palette,
    MusicTrack,
    MusicData,
    FxKey,
    EnemyPopulation,
    EnemyTilesets,
    MainCode,
    MainVariables,
    ScrollingHook,
    /// The Ceres HDMA table word's low byte ($07EB).
    CeresHdmaLow,
    /// $07EC: an owned display's committed fake mode, which overlays the
    /// Ceres HDMA table word's high byte; otherwise that high byte.
    CeresHdmaHigh,
    AudioDataIndex,
    AudioTrackIndex,
    ExploredMap,
    /// The camera's block coordinates and positions ($08F7..$091A).
    CameraBlocks,
    CameraPositions,
    Layer2ScrollX,
    Layer2ScrollY,
    /// BG1's offsets and BG2's scrolls ($091D..$0924).
    BackgroundScrolls,
    /// The door transition words ($0925..$0932).
    DoorTransition,
    CeresStatus,
    CameraDistance,
    TimerStatus,
    /// The escape countdown's digits and position ($0945..$094B).
    Countdown,
    MapAreaIndex,
    SelectedSlot,
    NonemptySlots,
    BackgroundSize,
    /// The scrolling tilemaps' source and destination blocks ($0990..$0997).
    TilemapBlocks,
}

use Owner::*;

/// Every owner, in address order from [`FIRST`].
const OWNERS: &[Owner] = &[
    HudPalette,
    MessageDelay,
    UnpauseQueueTail,
    NmiAcknowledgment,
    Mode7Irq,
    // The Mode 7 rotation angle ($0785).
    Unowned(2),
    MapUnread,
    HasAreaMap,
    LoadStation,
    DoorExit,
    // The removed door block type ($078F).
    Unowned(2),
    DoorDirection,
    ElevatorProperties,
    ElevatorInterlock,
    EnemyInterlock,
    ElevatorDirection,
    RoomHeader,
    RoomIndex,
    Area,
    MapX,
    MapY,
    Width,
    Height,
    ScrollWidth,
    ScrollHeight,
    ScrollUp,
    ScrollDown,
    PreviousCre,
    CurrentCre,
    DoorList,
    // The removed event pointer ($07B7).
    Unowned(2),
    SizeBytes,
    RoomState,
    LevelData,
    TileTable,
    Tiles,
    Palette,
    MusicTrack,
    MusicData,
    FxKey,
    EnemyPopulation,
    EnemyTilesets,
    // $07D3..$07DE.
    Unowned(12),
    MainCode,
    MainVariables,
    ScrollingHook,
    CeresHdmaLow,
    CeresHdmaHigh,
    // $07ED..$07F2.
    Unowned(6),
    AudioDataIndex,
    AudioTrackIndex,
    ExploredMap,
    CameraBlocks,
    CameraPositions,
    Layer2ScrollX,
    Layer2ScrollY,
    BackgroundScrolls,
    DoorTransition,
    // The removed words $0933..$093E.
    Unowned(12),
    CeresStatus,
    CameraDistance,
    TimerStatus,
    Countdown,
    // $094C..$094F.
    Unowned(4),
    MapAreaIndex,
    SelectedSlot,
    NonemptySlots,
    // The BG1 and BG2 tilemap update requests ($0956..$098D), whose numeric
    // stores still need their semantic counterparts.
    Unowned(56),
    BackgroundSize,
    TilemapBlocks,
];

impl Owner {
    /// How many bytes of the block the owner holds.
    const fn bytes(self) -> usize {
        match self {
            NmiAcknowledgment | CeresHdmaLow | CeresHdmaHigh | Layer2ScrollX | Layer2ScrollY => 1,
            LevelData | TileTable | Tiles | Palette => 3,
            Countdown => 7,
            MainVariables | BackgroundScrolls | TilemapBlocks => 8,
            DoorTransition => 14,
            CameraBlocks => 24,
            CameraPositions => 12,
            ExploredMap => 256,
            Unowned(bytes) => bytes,
            _ => 2,
        }
    }

    /// The owner's byte `offset` as `session` holds it; `None` where the
    /// Session holds no byte there.
    fn byte(self, session: &Session, offset: usize) -> Option<u8> {
        let (room, entry, world) = (&session.room, &session.room.entry, &session.world);
        let camera = &room.camera;
        let selection = &session.menu_screens.save_selection;
        let lane = |word: u16| word.to_le_bytes()[offset % 2];
        let pair = |words: &[u16]| lane(words[offset / 2]);
        Some(match self {
            HudPalette => lane(session.presentation.owner().ok()?.state.hud.palette),
            MessageDelay => lane(session.messages.delay),
            UnpauseQueueTail => lane(room.unpause_queue_tail),
            NmiAcknowledgment => session.presentation.owner().ok()?.state.nmi.acknowledgment,
            Mode7Irq => lane(room.mode7_irq),
            Unowned(_) => return None,
            MapUnread => lane(selection.map_unread),
            HasAreaMap => lane(selection.has_area_map),
            LoadStation => lane(selection.load_station),
            DoorExit => lane(world.exit_key(&entry.exit)),
            DoorDirection => lane(entry.direction),
            ElevatorProperties => lane(entry.elevator_properties),
            ElevatorInterlock => lane(entry.elevator_interlock),
            EnemyInterlock => lane(entry.enemy_interlock),
            ElevatorDirection => lane(entry.elevator_direction),
            RoomHeader => lane(world.room_key(&entry.room)),
            RoomIndex => lane(entry.room_index),
            Area => lane(room.area),
            MapX => lane(entry.map_x),
            MapY => lane(entry.map_y),
            Width => lane(room.width),
            Height => lane(room.height),
            ScrollWidth => lane(room.scroll_width),
            ScrollHeight => lane(room.scroll_height),
            ScrollUp => lane(entry.scroll_up),
            ScrollDown => lane(entry.scroll_down),
            PreviousCre => lane(entry.previous_cre),
            CurrentCre => lane(entry.cre),
            DoorList => lane(world.door_list_key(&entry.doors)),
            SizeBytes => lane(room.size_bytes),
            RoomState => lane(world.state_key(&entry.state)),
            LevelData => world.asset_bytes(&entry.level)[offset],
            TileTable => world.asset_bytes(&entry.tile_table)[offset],
            Tiles => world.asset_bytes(&entry.tiles)[offset],
            Palette => world.asset_bytes(&entry.palette)[offset],
            MusicTrack => lane(entry.music_track),
            MusicData => lane(entry.music_data),
            FxKey => lane(entry.fx_key),
            EnemyPopulation => lane(entry.enemy_population_key),
            EnemyTilesets => lane(entry.enemy_tilesets_key),
            MainCode => lane(entry.main_code_key),
            MainVariables => pair(&room.main_variables),
            ScrollingHook => lane(entry.scrolling_hook),
            CeresHdmaLow => room.ceres_hdma_table.to_le_bytes()[0],
            CeresHdmaHigh => match session.presentation.owner() {
                Ok(display) => display.state.controls.committed_fake_mode,
                Err(_) => room.ceres_hdma_table.to_le_bytes()[1],
            },
            AudioDataIndex => lane(audio_indices(session)[0]),
            AudioTrackIndex => lane(audio_indices(session)[1]),
            ExploredMap => session.minimap.explored[offset],
            CameraBlocks => pair(&[
                camera.layer1_block[0],
                camera.layer1_block[1],
                camera.layer2_block[0],
                camera.layer2_block[1],
                camera.previous_layer1_block[0],
                camera.previous_layer1_block[1],
                camera.previous_layer2_block[0],
                camera.previous_layer2_block[1],
                camera.bg1_block[0],
                camera.bg1_block[1],
                camera.bg2_block[0],
                camera.bg2_block[1],
            ]),
            CameraPositions => pair(&[
                camera.layer1_fraction[0],
                camera.layer1[0],
                camera.layer1_fraction[1],
                camera.layer1[1],
                camera.layer2[0],
                camera.layer2[1],
            ]),
            Layer2ScrollX => entry.layer2_x,
            Layer2ScrollY => entry.layer2_y,
            BackgroundScrolls => pair(&[
                camera.bg1_offset[0],
                camera.bg1_offset[1],
                camera.bg2_scroll[0],
                camera.bg2_scroll[1],
            ]),
            DoorTransition => pair(&[
                entry.frame,
                entry.arrival_x,
                entry.arrival_y,
                entry.speed_fraction,
                entry.speed_whole,
                entry.elevator_delay,
                entry.completion,
            ]),
            CeresStatus => lane(room.ceres_status),
            CameraDistance => lane(camera.distance_index),
            TimerStatus => lane(session.timing.timer_status),
            Countdown => session.timing.countdown.byte(offset),
            MapAreaIndex => lane(selection.map_area_index),
            SelectedSlot => lane(selection.selected_slot),
            NonemptySlots => lane(selection.nonempty_slots),
            BackgroundSize => lane(room.background_size),
            TilemapBlocks => {
                let blocks = &session.queued_tilemaps;
                pair(&[
                    blocks.source_block[0],
                    blocks.source_block[1],
                    blocks.destination_block[0],
                    blocks.destination_block[1],
                ])
            }
        })
    }
}

/// The audio requests' music data and track indexes ($07F3, $07F5), under
/// either control.
fn audio_indices(session: &Session) -> [u16; 2] {
    match &session.control {
        crate::session_owners::Control::Legacy(control) => {
            let music = &control.audio.music;
            [music.data_index, music.track_index]
        }
        crate::session_owners::Control::Owned(control) => {
            let music = &control.requests.music;
            [music.data_index, music.track_index]
        }
    }
}

const fn total_bytes() -> usize {
    let mut total = 0;
    let mut index = 0;
    while index < OWNERS.len() {
        total += OWNERS[index].bytes();
        index += 1;
    }
    total
}

const _: () = assert!(total_bytes() == 2 * WORDS, "the owners cover the block");

/// Byte `offset` of the block as `session` holds it.
fn byte(session: &Session, mut offset: usize) -> Option<u8> {
    for &owner in OWNERS {
        if offset < owner.bytes() {
            return owner.byte(session, offset);
        }
        offset -= owner.bytes();
    }
    None
}

/// Word `index` of the block as `session` holds it: its low byte's owner's
/// byte, then its high byte's. `None` where either owner is not a Session
/// owner.
pub fn session_word(session: &Session, index: usize) -> Option<u16> {
    if index >= WORDS {
        return None;
    }
    let low = byte(session, 2 * index)?;
    let high = byte(session, 2 * index + 1)?;
    Some(u16::from_le_bytes([low, high]))
}
