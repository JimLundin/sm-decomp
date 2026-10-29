//! The authored world graph: rooms, states, exits, starts and door lists with
//! stable IDs, their compatibility keys and validation. Map Rando and other
//! topologies supply this record; the game binds it to content at open.
use std::collections::BTreeMap;

macro_rules! ids { ($($name:ident),+ $(,)?) => { $(
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct $name(pub u32);
)+ }; }
ids!(
    RoomId, StateId, ExitId, StartId, TilesetId, DoorListId, ResourceId
);
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WorldRevision(pub [u8; 32]);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Identity<I> {
    Known(I),
    LegacyUnresolved(u16),
}
/// IDs name content; compatibility addresses are supplied separately.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct WorldDefinition {
    pub rooms: BTreeMap<RoomId, RoomDefinition>,
    pub states: BTreeMap<StateId, StateDefinition>,
    pub exits: BTreeMap<ExitId, ExitDefinition>,
    pub starts: BTreeMap<StartId, StartDefinition>,
    pub door_lists: BTreeMap<DoorListId, Vec<ExitId>>,
    pub tilesets: BTreeMap<TilesetId, TilesetDefinition>,
    pub resources: BTreeMap<ResourceId, ResourceKind>,
    pub closing_doors: BTreeMap<u16, Option<ResourceId>>,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RoomDefinition {
    pub index: u8,
    pub area: u8,
    pub map_x: u8,
    pub map_y: u8,
    pub width: u8,
    pub height: u8,
    pub up_scroll: u8,
    pub down_scroll: u8,
    pub cre: u8,
    pub doors: DoorListId,
    pub selectors: Vec<StateSelector>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct StateSelector {
    pub condition: StateCondition,
    pub state: StateId,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StateCondition {
    Default,
    Event(u8),
    Boss(u8),
    BossOne,
    MorphAndMissiles,
    PowerBombs,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StateDefinition {
    pub tileset: TilesetId,
    pub level: ResourceId,
    pub music_data: u8,
    pub music_track: u8,
    pub fx: Option<ResourceId>,
    pub enemy_population: ResourceId,
    pub enemy_tilesets: ResourceId,
    pub layer2_policy: [u8; 2],
    pub main_code: Option<ResourceId>,
    pub scrolls: ScrollDefinition,
    pub objects: Vec<RoomObject>,
    pub background: Vec<BackgroundCommand>,
    pub setup: Option<ResourceId>,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ScrollDefinition {
    Generated(u8),
    Cells(Vec<u8>),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TilesetDefinition {
    pub tile_table: ResourceId,
    pub tiles: ResourceId,
    pub palette: ResourceId,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RoomObject {
    pub x: u8,
    pub y: u8,
    pub definition: ResourceId,
    pub argument: u16,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ExitDefinition {
    pub destination: RoomId,
    pub arrival: ArrivalDefinition,
    pub setup: Option<ResourceId>,
    pub skip_cre_upload: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ArrivalDefinition {
    pub flags: u8,
    pub direction: u8,
    pub x: u8,
    pub y: u8,
    pub close_x: u8,
    pub close_y: u8,
    pub distance: i16,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct StartDefinition {
    pub room: RoomId,
    pub exit: ExitId,
    pub camera_x: u16,
    pub camera_y: u16,
    pub samus_x_offset: u16,
    pub samus_y_offset: u16,
}
/// Authored target names, not SNES register values.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BackgroundTarget {
    Main,
    Secondary,
    Fx,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum BackgroundCommand {
    Transfer {
        source: ResourceId,
        destination: BackgroundTarget,
        bytes: u16,
    },
    Decompress {
        source: ResourceId,
        destination: BackgroundTarget,
    },
    ClearFx,
    ClearBackground,
    ClearDualBackground,
    TransferAndSetCopyPolicy {
        source: ResourceId,
        destination: BackgroundTarget,
        bytes: u16,
    },
    MatchExitTransfer {
        exit: ExitId,
        source: ResourceId,
        destination: BackgroundTarget,
        bytes: u16,
    },
}
/// This milestone binds retained compatible resources; it adds no importer or setup interpreter.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ResourceKind {
    Level,
    TileTable,
    Tiles,
    Palette,
    Fx,
    EnemyPopulation,
    EnemyTilesets,
    ActorSprites,
    ActorTilemap,
    RoomObject,
    DoorSetup,
    RoomSetup,
    RoomMain,
    Background,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LegacyAddress24(pub [u8; 3]);
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ResourceKey {
    Word { bank: u8, offset: u16 },
    Long(LegacyAddress24),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LegacyStartSite {
    pub area: u16,
    pub index: u16,
}
/// Construction input only; WorldBinding keeps a private immutable validated copy.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CompatibilityKeys {
    pub starts: BTreeMap<StartId, LegacyStartSite>,
    pub rooms: BTreeMap<RoomId, u16>,
    pub states: BTreeMap<StateId, u16>,
    pub exits: BTreeMap<ExitId, u16>,
    pub door_lists: BTreeMap<DoorListId, u16>,
    pub resources: BTreeMap<ResourceId, ResourceKey>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum WorldDefinitionError {
    MissingRoom(RoomId),
    MissingState(StateId),
    MissingExit(ExitId),
    MissingStart(StartId),
    MissingDoorList(DoorListId),
    MissingTileset(TilesetId),
    MissingResource(ResourceId),
    MissingCompatibilityKey,
    AmbiguousCompatibilityKey,
    WrongResourceKind(ResourceId),
    UnsupportedResource(ResourceId),
    UnsupportedSetup(ResourceId),
    InvalidDefinition,
    TableCatalogMismatch,
}

/// Every reference resolves and every room ends its selectors with Default.
pub fn validate(world: &WorldDefinition) -> Result<(), WorldDefinitionError> {
    use WorldDefinitionError::*;
    for room in world.rooms.values() {
        if room.width == 0
            || room.height == 0
            || room.selectors.last().map(|s| s.condition) != Some(StateCondition::Default)
            || room.selectors[..room.selectors.len().saturating_sub(1)]
                .iter()
                .any(|s| s.condition == StateCondition::Default)
        {
            return Err(InvalidDefinition);
        }
        if !world.door_lists.contains_key(&room.doors) {
            return Err(MissingDoorList(room.doors));
        }
        for selector in &room.selectors {
            let state = world
                .states
                .get(&selector.state)
                .ok_or(MissingState(selector.state))?;
            // The retained generated-scroll loops compare signed eight-bit counters.
            if matches!(state.scrolls, ScrollDefinition::Generated(_))
                && (room.width > 127 || room.height > 127)
            {
                return Err(InvalidDefinition);
            }
        }
    }
    for exits in world.door_lists.values() {
        for id in exits {
            if !world.exits.contains_key(id) {
                return Err(MissingExit(*id));
            }
        }
    }
    for exit in world.exits.values() {
        if !world.rooms.contains_key(&exit.destination) {
            return Err(MissingRoom(exit.destination));
        }
        optional_resource(world, exit.setup, ResourceKind::DoorSetup)?;
        if !world
            .closing_doors
            .contains_key(&u16::from(exit.arrival.direction))
        {
            return Err(InvalidDefinition);
        }
    }
    for start in world.starts.values() {
        if !world.rooms.contains_key(&start.room) {
            return Err(MissingRoom(start.room));
        }
        if !world.exits.contains_key(&start.exit) {
            return Err(MissingExit(start.exit));
        }
    }
    for state in world.states.values() {
        if !world.tilesets.contains_key(&state.tileset) {
            return Err(MissingTileset(state.tileset));
        }
        resource(world, state.level, ResourceKind::Level)?;
        resource(world, state.enemy_population, ResourceKind::EnemyPopulation)?;
        resource(world, state.enemy_tilesets, ResourceKind::EnemyTilesets)?;
        optional_resource(world, state.fx, ResourceKind::Fx)?;
        optional_resource(world, state.main_code, ResourceKind::RoomMain)?;
        optional_resource(world, state.setup, ResourceKind::RoomSetup)?;
        if let ScrollDefinition::Cells(cells) = &state.scrolls {
            if cells.len() != 50 {
                return Err(InvalidDefinition);
            }
        }
        for object in &state.objects {
            resource(world, object.definition, ResourceKind::RoomObject)?;
        }
        for command in &state.background {
            match command {
                BackgroundCommand::Transfer { source, .. }
                | BackgroundCommand::Decompress { source, .. }
                | BackgroundCommand::TransferAndSetCopyPolicy { source, .. } => {
                    resource(world, *source, ResourceKind::Background)?
                }
                BackgroundCommand::MatchExitTransfer { exit, source, .. } => {
                    if !world.exits.contains_key(exit) {
                        return Err(MissingExit(*exit));
                    }
                    resource(world, *source, ResourceKind::Background)?;
                }
                BackgroundCommand::ClearFx
                | BackgroundCommand::ClearBackground
                | BackgroundCommand::ClearDualBackground => {}
            }
        }
    }
    for tileset in world.tilesets.values() {
        resource(world, tileset.tile_table, ResourceKind::TileTable)?;
        resource(world, tileset.tiles, ResourceKind::Tiles)?;
        resource(world, tileset.palette, ResourceKind::Palette)?;
    }
    for &id in world.closing_doors.values() {
        optional_resource(world, id, ResourceKind::RoomObject)?;
    }
    Ok(())
}
pub fn validate_resource(
    id: ResourceId,
    kind: ResourceKind,
    key: ResourceKey,
) -> Result<(), WorldDefinitionError> {
    use ResourceKind::*;
    let bank = match kind {
        Level | TileTable | Tiles | Palette | Background | ActorSprites | ActorTilemap => None,
        Fx => Some(0x83),
        EnemyPopulation => Some(0xa1),
        EnemyTilesets => Some(0xb4),
        RoomObject => Some(0x84),
        DoorSetup | RoomSetup | RoomMain => Some(0x8f),
    };
    let supported = match (bank, key) {
        (None, ResourceKey::Long(LegacyAddress24([_, hi, bank]))) => {
            hi & 0x80 != 0 && !matches!(bank, 0x7e | 0x7f)
        }
        (Some(expected), ResourceKey::Word { bank, offset }) => {
            bank == expected && offset & 0x8000 != 0
        }
        _ => false,
    };
    if !supported {
        return Err(WorldDefinitionError::UnsupportedResource(id));
    }
    if matches!(kind, DoorSetup | RoomSetup | RoomMain) {
        let ResourceKey::Word { offset, .. } = key else {
            unreachable!()
        };
        if !supported_setup(kind, offset) {
            return Err(WorldDefinitionError::UnsupportedSetup(id));
        }
    }
    Ok(())
}

// Supported retained dispatches from sm_8f.rs and room_scrolls.rs (b09 flags baseline).
fn supported_setup(kind: ResourceKind, offset: u16) -> bool {
    match kind {
        ResourceKind::RoomSetup => matches!(
            offset,
            0x9194
                | 0x91a9
                | 0x91b2
                | 0x91bb
                | 0x91bc
                | 0x91bd
                | 0x91c9
                | 0x91ce
                | 0x91d3
                | 0x91d4
                | 0x91d5
                | 0x91d6
                | 0x91d7
                | 0x91f4
                | 0x91f5
                | 0x91f6
                | 0x91f7
                | 0xc8c7
                | 0xc8c8
                | 0xc8d0
                | 0xc8d1
                | 0xc8d2
                | 0xc8d3
                | 0xc8dc
                | 0xc8dd
                | 0xc90a
                | 0xc91e
                | 0xc91f
                | 0xc933
                | 0xc946
                | 0xc953
                | 0xc96e
                | 0xc976
                | 0xc97b
        ),
        ResourceKind::DoorSetup => matches!(
            offset,
            0xb971
                | 0xb981
                | 0xb98c
                | 0xb997
                | 0xb9a2
                | 0xb9b3
                | 0xb9ca
                | 0xb9f1
                | 0xba00
                | 0xba0b
                | 0xba16
                | 0xba21
                | 0xba2c
                | 0xbd07
                | 0xbd16
                | 0xbd25
                | 0xbd30
                | 0xbd3f
                | 0xbd50
                | 0xbd5b
                | 0xbd6c
                | 0xbd77
                | 0xbd8a
                | 0xbd95
                | 0xbda0
                | 0xbdaf
                | 0xbdc0
                | 0xbdd1
                | 0xbde2
                | 0xbdf1
                | 0xbe00
                | 0xbe0b
                | 0xbe1a
                | 0xbe25
                | 0xbe36
                | 0xbf9e
                | 0xbfbb
                | 0xbfda
                | 0xbfe9
                | 0xbffa
                | 0xc016
                | 0xc025
                | 0xc03a
                | 0xc04b
                | 0xc056
                | 0xc067
                | 0xc078
                | 0xc089
                | 0xc0a2
                | 0xc0ad
                | 0xc0c2
                | 0xc0d3
                | 0xc0de
                | 0xc0ef
                | 0xc0fa
                | 0xc10b
                | 0xe1d8
                | 0xe1e8
                | 0xe1f3
                | 0xe1fe
                | 0xe20f
                | 0xe21a
                | 0xe229
                | 0xe26c
                | 0xe291
                | 0xe301
                | 0xe309
                | 0xe318
                | 0xe345
                | 0xe356
                | 0xe367
                | 0xe378
                | 0xe38d
                | 0xe398
                | 0xe3a3
                | 0xe3b9
                | 0xe3c8
                | 0xe3d9
                | 0xe4c0
                | 0xe4cf
                | 0xe4e0
                | 0xe513
        ),
        ResourceKind::RoomMain => matches!(
            offset,
            0xc116
                | 0xc11b
                | 0xc120
                | 0xc124
                | 0xc1e6
                | 0xe2b6
                | 0xe51f
                | 0xe524
                | 0xe525
                | 0xe571
                | 0xe57c
                | 0xe5a0
                | 0xe5a4
                | 0xe8cd
                | 0xe950
        ),
        _ => false,
    }
}

fn resource(
    world: &WorldDefinition,
    id: ResourceId,
    kind: ResourceKind,
) -> Result<(), WorldDefinitionError> {
    match world.resources.get(&id) {
        None => Err(WorldDefinitionError::MissingResource(id)),
        Some(actual) if *actual != kind => Err(WorldDefinitionError::WrongResourceKind(id)),
        Some(_) => Ok(()),
    }
}
fn optional_resource(
    world: &WorldDefinition,
    id: Option<ResourceId>,
    kind: ResourceKind,
) -> Result<(), WorldDefinitionError> {
    match id {
        Some(id) => resource(world, id, kind),
        None => Ok(()),
    }
}
