//! One authored topology fixture shared by actual Runtime/GameOwner checks.
//! Keys are compatibility bindings, never the identities or map positions.
#![forbid(unsafe_code)]
use sm_game::room_entry::world::*;
use std::{collections::BTreeMap, sync::Arc};

pub(crate) const ROOM_A: RoomId = RoomId(0x1200_0041);
pub(crate) const ROOM_B: RoomId = RoomId(0x0300_0082);
pub(crate) const ROOM_A_KEY: u16 = 0x9000;
pub(crate) const ROOM_B_KEY: u16 = 0x9200;
pub(crate) const EXIT_A: ExitId = ExitId(0x2400_0051);
pub(crate) const EXIT_B: ExitId = ExitId(0x0500_0092);
pub(crate) const EXIT_A_KEY: u16 = 0xa000;
pub(crate) const EXIT_B_KEY: u16 = 0xa200;
pub(crate) const START_A: StartId = StartId(0x3600_0061);
pub(crate) const START_B: StartId = StartId(0x0700_0062);
pub(crate) const START_A_SITE: LegacyStartSite = LegacyStartSite { area: 0, index: 0 };
pub(crate) const START_B_SITE: LegacyStartSite = LegacyStartSite { area: 1, index: 2 };
pub(crate) const STATE_A: StateId = StateId(0x4700_0071);
pub(crate) const STATE_A_EVENT: StateId = StateId(0x0800_0032);
pub(crate) const STATE_B: StateId = StateId(0x1900_0093);
pub(crate) const STATE_A_KEY: u16 = 0xb000;
pub(crate) const STATE_A_EVENT_KEY: u16 = 0xb100;
pub(crate) const STATE_B_KEY: u16 = 0xb200;
pub(crate) const TILESET: TilesetId = TilesetId(0x5100_0007);
pub(crate) const LEVEL_A: ResourceId = ResourceId(0x6200_0001);
pub(crate) const LEVEL_A_EVENT: ResourceId = ResourceId(0x6200_0002);
pub(crate) const LEVEL_B: ResourceId = ResourceId(0x6200_0003);
pub(crate) const TILE_TABLE: ResourceId = ResourceId(0x6200_0004);
pub(crate) const TILES: ResourceId = ResourceId(0x6200_0005);
pub(crate) const PALETTE: ResourceId = ResourceId(0x6200_0006);
pub(crate) const ENEMIES: ResourceId = ResourceId(0x6200_0007);
pub(crate) const ENEMY_TILES: ResourceId = ResourceId(0x6200_0008);
const DOORS_A: DoorListId = DoorListId(0x7300_0001);
const DOORS_B: DoorListId = DoorListId(0x0400_0002);

pub(crate) const LEVEL_A_BYTES: [u8; 3] = [0x00, 0x80, 0xc0];
pub(crate) const LEVEL_A_EVENT_BYTES: [u8; 3] = [0x00, 0x81, 0xc0];
pub(crate) const LEVEL_B_BYTES: [u8; 3] = [0x00, 0x82, 0xc0];
pub(crate) const TILE_TABLE_BYTES: [u8; 3] = [0x00, 0x83, 0xc0];
pub(crate) const TILES_BYTES: [u8; 3] = [0x00, 0x84, 0xc0];
pub(crate) const PALETTE_BYTES: [u8; 3] = [0x00, 0x85, 0xc0];
pub(crate) const ROOM_SCROLLS: u8 = 6;

fn state(level: ResourceId, music: u8) -> StateDefinition {
    StateDefinition {
        tileset: TILESET,
        level,
        music_data: music,
        music_track: if music == 0 { 0 } else { 5 },
        fx: None,
        enemy_population: ENEMIES,
        enemy_tilesets: ENEMY_TILES,
        layer2_policy: [0, 0],
        main_code: None,
        scrolls: ScrollDefinition::Generated(0),
        objects: Vec::new(),
        background: Vec::new(),
        setup: None,
    }
}

/// Expose construction inputs so validation checks can alter one definition.
/// All callers still invoke the actual WorldBinding::prepared constructor.
pub(crate) fn definition(swapped: bool) -> (WorldDefinition, CompatibilityKeys) {
    let room = |index, area, map_x, map_y, doors, selectors| RoomDefinition {
        index,
        area,
        map_x,
        map_y,
        width: ROOM_SCROLLS,
        height: ROOM_SCROLLS,
        up_scroll: 2,
        down_scroll: 3,
        cre: 0,
        doors,
        selectors,
    };
    let exit = |destination, direction, x, y| ExitDefinition {
        destination,
        arrival: ArrivalDefinition {
            flags: 0,
            direction,
            x,
            y,
            close_x: 0,
            close_y: 0,
            distance: -1,
        },
        setup: None,
        skip_cre_upload: false,
    };
    let world = WorldDefinition {
        rooms: BTreeMap::from([
            (
                ROOM_A,
                room(
                    7,
                    0,
                    3,
                    4,
                    DOORS_A,
                    vec![
                        StateSelector {
                            condition: StateCondition::Event(6),
                            state: STATE_A_EVENT,
                        },
                        StateSelector {
                            condition: StateCondition::Default,
                            state: STATE_A,
                        },
                    ],
                ),
            ),
            (
                ROOM_B,
                room(
                    9,
                    1,
                    5,
                    6,
                    DOORS_B,
                    vec![StateSelector {
                        condition: StateCondition::Default,
                        state: STATE_B,
                    }],
                ),
            ),
        ]),
        states: BTreeMap::from([
            (STATE_A, state(LEVEL_A, 0)),
            (STATE_A_EVENT, state(LEVEL_A_EVENT, 3)),
            (STATE_B, state(LEVEL_B, 6)),
        ]),
        exits: BTreeMap::from([
            (EXIT_A, exit(if swapped { ROOM_A } else { ROOM_B }, 0, 2, 3)),
            (EXIT_B, exit(if swapped { ROOM_B } else { ROOM_A }, 2, 4, 5)),
        ]),
        starts: BTreeMap::from([
            (
                START_A,
                StartDefinition {
                    room: ROOM_A,
                    exit: EXIT_B,
                    camera_x: 0x100,
                    camera_y: 0x200,
                    samus_x_offset: 0x20,
                    samus_y_offset: 0x30,
                },
            ),
            (
                START_B,
                StartDefinition {
                    room: ROOM_B,
                    exit: EXIT_A,
                    camera_x: 0x300,
                    camera_y: 0x400,
                    samus_x_offset: 0x40,
                    samus_y_offset: 0x50,
                },
            ),
        ]),
        door_lists: BTreeMap::from([(DOORS_A, vec![EXIT_A]), (DOORS_B, vec![EXIT_B])]),
        tilesets: BTreeMap::from([(
            TILESET,
            TilesetDefinition {
                tile_table: TILE_TABLE,
                tiles: TILES,
                palette: PALETTE,
            },
        )]),
        resources: BTreeMap::from([
            (LEVEL_A, ResourceKind::Level),
            (LEVEL_A_EVENT, ResourceKind::Level),
            (LEVEL_B, ResourceKind::Level),
            (TILE_TABLE, ResourceKind::TileTable),
            (TILES, ResourceKind::Tiles),
            (PALETTE, ResourceKind::Palette),
            (ENEMIES, ResourceKind::EnemyPopulation),
            (ENEMY_TILES, ResourceKind::EnemyTilesets),
        ]),
        closing_doors: (0..4).map(|direction| (direction, None)).collect(),
    };
    let long = |bytes| ResourceKey::Long(LegacyAddress24(bytes));
    let keys = CompatibilityKeys {
        starts: BTreeMap::from([(START_A, START_A_SITE), (START_B, START_B_SITE)]),
        rooms: BTreeMap::from([(ROOM_A, ROOM_A_KEY), (ROOM_B, ROOM_B_KEY)]),
        states: BTreeMap::from([
            (STATE_A, STATE_A_KEY),
            (STATE_A_EVENT, STATE_A_EVENT_KEY),
            (STATE_B, STATE_B_KEY),
        ]),
        exits: BTreeMap::from([(EXIT_A, EXIT_A_KEY), (EXIT_B, EXIT_B_KEY)]),
        door_lists: BTreeMap::from([(DOORS_A, 0xc000), (DOORS_B, 0xc200)]),
        resources: BTreeMap::from([
            (LEVEL_A, long(LEVEL_A_BYTES)),
            (LEVEL_A_EVENT, long(LEVEL_A_EVENT_BYTES)),
            (LEVEL_B, long(LEVEL_B_BYTES)),
            (TILE_TABLE, long(TILE_TABLE_BYTES)),
            (TILES, long(TILES_BYTES)),
            (PALETTE, long(PALETTE_BYTES)),
            (
                ENEMIES,
                ResourceKey::Word {
                    bank: 0xa1,
                    offset: 0x9000,
                },
            ),
            (
                ENEMY_TILES,
                ResourceKey::Word {
                    bank: 0xb4,
                    offset: 0x9000,
                },
            ),
        ]),
    };
    (world, keys)
}

pub(crate) fn binding(swapped: bool) -> Arc<WorldBinding> {
    let (world, keys) = definition(swapped);
    WorldBinding::prepared(world, keys).expect("valid synthetic two-room fixture")
}

/// Only full application packs add the actual CurrentLoadStation opening sites.
#[cfg(sm_verify_dll)]
pub(crate) fn definition_with_application_starts(
    swapped: bool,
) -> (WorldDefinition, CompatibilityKeys) {
    let (mut world, mut keys) = definition(swapped);
    for (id, original, site) in [
        (
            StartId(0x3600_0063),
            START_A,
            LegacyStartSite { area: 6, index: 0 },
        ),
        (
            StartId(0x0700_0064),
            START_B,
            LegacyStartSite { area: 0, index: 18 },
        ),
    ] {
        world.starts.insert(id, world.starts[&original]);
        keys.starts.insert(id, site);
    }
    (world, keys)
}
