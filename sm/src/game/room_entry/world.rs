//! Immutable authored topology and validated compatibility bindings.
//! Construction validates every reference before an owned Game is opened.
#![forbid(unsafe_code)]
use super::{
    content::{AssetRef, AssetSource, DoorList},
    effects::AssetSlot,
};
use std::{
    collections::BTreeMap,
    hash::{DefaultHasher, Hash, Hasher},
    sync::Arc,
};
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
#[derive(Clone, Debug, PartialEq)]
pub struct WorldBinding {
    tile_tables: Option<Arc<PreparedTileTables>>,
    room_assets: Option<Arc<PreparedRoomAssets>>,
    actor_catalog: Arc<crate::game::actors::content::ActorCatalog>,
    actor_media: Arc<crate::game::actors::media::ActorMediaCatalog>,
    pub(super) revision: WorldRevision,
    pub(super) content: BoundContent,
    pub(super) keys: CompatibilityKeys,
    pub(super) starts_by_key: BTreeMap<LegacyStartSite, StartId>,
    pub(super) rooms_by_key: BTreeMap<u16, RoomId>,
    pub(super) states_by_key: BTreeMap<u16, StateId>,
    pub(super) exits_by_key: BTreeMap<u16, ExitId>,
    pub(super) door_lists_by_key: BTreeMap<u16, DoorListId>,
    pub(super) resources_by_key: BTreeMap<(ResourceKind, ResourceKey), ResourceId>,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) enum BoundContent {
    CanonicalLegacy,
    Prepared(WorldDefinition),
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

impl WorldBinding {
    pub fn canonical_legacy() -> Arc<Self> {
        Arc::new(Self {
            tile_tables: None,
            room_assets: None,
            actor_catalog: Arc::new(crate::game::actors::content::ActorCatalog::LegacyBound),
            actor_media: Arc::new(crate::game::actors::media::ActorMediaCatalog::LegacyBound),
            revision: WorldRevision([0; 32]),
            content: BoundContent::CanonicalLegacy,
            keys: CompatibilityKeys {
                starts: BTreeMap::new(),
                rooms: BTreeMap::new(),
                states: BTreeMap::new(),
                exits: BTreeMap::new(),
                door_lists: BTreeMap::new(),
                resources: BTreeMap::new(),
            },
            starts_by_key: BTreeMap::new(),
            rooms_by_key: BTreeMap::new(),
            states_by_key: BTreeMap::new(),
            exits_by_key: BTreeMap::new(),
            door_lists_by_key: BTreeMap::new(),
            resources_by_key: BTreeMap::new(),
        })
    }
    pub fn prepared(
        world: WorldDefinition,
        keys: CompatibilityKeys,
    ) -> Result<Arc<Self>, WorldDefinitionError> {
        validate(&world)?;
        let starts_by_key = reverse(&world.starts, &keys.starts)?;
        let rooms_by_key = reverse(&world.rooms, &keys.rooms)?;
        let states_by_key = reverse(&world.states, &keys.states)?;
        let exits_by_key = reverse(&world.exits, &keys.exits)?;
        let door_lists_by_key = reverse(&world.door_lists, &keys.door_lists)?;
        if world.resources.keys().ne(keys.resources.keys()) {
            return Err(WorldDefinitionError::MissingCompatibilityKey);
        }
        let mut resources_by_key = BTreeMap::new();
        for (&id, &kind) in &world.resources {
            let key = keys.resources[&id];
            validate_resource(id, kind, key)?;
            if resources_by_key.insert((kind, key), id).is_some() {
                return Err(WorldDefinitionError::AmbiguousCompatibilityKey);
            }
        }
        // Process-local revision evidence, not a cryptographic or portable file identifier.
        let mut revision = [0; 32];
        for (lane, bytes) in revision.chunks_exact_mut(8).enumerate() {
            let mut hasher = DefaultHasher::new();
            ("room-entry-world-v1", lane as u64, &world, &keys).hash(&mut hasher);
            bytes.copy_from_slice(&hasher.finish().to_le_bytes());
        }
        Ok(Arc::new(Self {
            tile_tables: None,
            room_assets: None,
            actor_catalog: Arc::new(crate::game::actors::content::ActorCatalog::LegacyBound),
            actor_media: Arc::new(crate::game::actors::media::ActorMediaCatalog::LegacyBound),
            revision: WorldRevision(revision),
            content: BoundContent::Prepared(world),
            keys,
            starts_by_key,
            rooms_by_key,
            states_by_key,
            exits_by_key,
            door_lists_by_key,
            resources_by_key,
        }))
    }
    /// Attach a validated immutable catalog before opening, preserving existing construction.
    pub fn prepared_with_tables(
        world: WorldDefinition,
        keys: CompatibilityKeys,
        tables: PreparedTileTables,
    ) -> Result<Arc<Self>, WorldDefinitionError> {
        Self::with_tables(Self::prepared(world, keys)?, tables)
    }
    pub(crate) fn with_tables(
        mut binding: Arc<Self>,
        tables: PreparedTileTables,
    ) -> Result<Arc<Self>, WorldDefinitionError> {
        let owned = Arc::get_mut(&mut binding).ok_or(WorldDefinitionError::InvalidDefinition)?;
        if owned.tile_tables.is_some() {
            return Err(WorldDefinitionError::TableCatalogMismatch);
        }
        if !tables.matches(owned.definition().unwrap(), &owned.keys) {
            return Err(WorldDefinitionError::TableCatalogMismatch);
        }
        let original = owned.revision;
        for (lane, bytes) in owned.revision.0.chunks_exact_mut(8).enumerate() {
            let mut hasher = DefaultHasher::new();
            (
                "room-entry-tables-v1",
                lane as u64,
                original,
                tables.fingerprint(),
            )
                .hash(&mut hasher);
            bytes.copy_from_slice(&hasher.finish().to_le_bytes());
        }
        owned.tile_tables = Some(Arc::new(tables));
        Ok(binding)
    }
    pub fn prepared_with_room_assets(
        world: WorldDefinition,
        keys: CompatibilityKeys,
        assets: PreparedRoomAssets,
    ) -> Result<Arc<Self>, PreparedRoomError> {
        Self::with_room_assets(
            Self::prepared(world, keys).map_err(PreparedRoomError::World)?,
            assets,
        )
    }
    /// Complete a compatible Actor/table binding before an owner is opened.
    pub fn with_room_assets(
        mut binding: Arc<Self>,
        assets: PreparedRoomAssets,
    ) -> Result<Arc<Self>, PreparedRoomError> {
        let owned = Arc::get_mut(&mut binding).ok_or(PreparedRoomError::World(
            WorldDefinitionError::InvalidDefinition,
        ))?;
        if owned.room_assets.is_some()
            || !assets.matches(
                owned.definition().ok_or(PreparedRoomError::Incompatible)?,
                &owned.keys,
            )
        {
            return Err(PreparedRoomError::Incompatible);
        }
        if owned
            .tile_tables
            .as_ref()
            .is_some_and(|tables| tables.fingerprint() != assets.tables().fingerprint())
        {
            return Err(PreparedRoomError::Incompatible);
        }
        let mut media = (*owned.actor_media).clone();
        media.bind_room_catalog(assets.fingerprint());
        owned.actor_media = Arc::new(media);
        owned.revision = WorldRevision(assets.fingerprint());
        owned.tile_tables = Some(Arc::new(assets.tables().clone()));
        owned.room_assets = Some(Arc::new(assets));
        Ok(binding)
    }
    pub fn prepared_room_source_size(&self) -> Option<usize> {
        self.room_assets.as_ref().map(|assets| assets.source_size())
    }
    pub(crate) fn compatibility_keys(&self) -> &CompatibilityKeys {
        &self.keys
    }
    pub(crate) fn prepared_room_assets(&self) -> Option<Arc<PreparedRoomAssets>> {
        self.room_assets.clone()
    }
    pub(crate) fn prepared_table(&self, source: &AssetSource) -> Option<Arc<[u8]>> {
        self.tile_tables.as_ref()?.stream(source)
    }
    pub fn is_canonical_legacy(&self) -> bool {
        matches!(self.content, BoundContent::CanonicalLegacy)
    }
    pub fn revision(&self) -> WorldRevision {
        self.revision
    }
    pub fn definition(&self) -> Option<&WorldDefinition> {
        match &self.content {
            BoundContent::CanonicalLegacy => None,
            BoundContent::Prepared(world) => Some(world),
        }
    }
    pub(crate) fn start_at(&self, area: u16, index: u16) -> StartId {
        self.starts_by_key[&LegacyStartSite { area, index }]
    }
    pub(crate) fn room_identity(&self, key: u16) -> Identity<RoomId> {
        identity(&self.rooms_by_key, key)
    }
    pub(crate) fn state_identity(&self, key: u16) -> Identity<StateId> {
        identity(&self.states_by_key, key)
    }
    pub(crate) fn exit_identity(&self, key: u16) -> Identity<ExitId> {
        identity(&self.exits_by_key, key)
    }
    pub(crate) fn room_key(&self, value: &Identity<RoomId>) -> u16 {
        key(&self.keys.rooms, value)
    }
    pub(crate) fn state_key(&self, value: &Identity<StateId>) -> u16 {
        key(&self.keys.states, value)
    }
    pub(crate) fn exit_key(&self, value: &Identity<ExitId>) -> u16 {
        key(&self.keys.exits, value)
    }
    pub(crate) fn door_list(&self, key: u16) -> DoorList {
        self.door_lists_by_key
            .get(&key)
            .map_or(DoorList::Legacy(key), |&id| DoorList::Prepared(id))
    }
    pub(crate) fn door_list_key(&self, value: &DoorList) -> u16 {
        match value {
            DoorList::Legacy(key) => *key,
            DoorList::Prepared(id) => self.keys.door_lists[id],
        }
    }
    pub(crate) fn asset(&self, slot: AssetSlot, bytes: [u8; 3]) -> AssetRef {
        let kind = match slot {
            AssetSlot::Level => ResourceKind::Level,
            AssetSlot::TileTable => ResourceKind::TileTable,
            AssetSlot::Tiles => ResourceKind::Tiles,
            AssetSlot::Palette => ResourceKind::Palette,
        };
        self.resources_by_key
            .get(&(kind, ResourceKey::Long(LegacyAddress24(bytes))))
            .map_or(AssetRef::Legacy(bytes), |&id| AssetRef::Prepared(id))
    }
    pub(crate) fn asset_bytes(&self, value: &AssetRef) -> [u8; 3] {
        match value {
            AssetRef::Legacy(bytes) => *bytes,
            AssetRef::Prepared(id) => match self.resource_key(*id) {
                ResourceKey::Long(LegacyAddress24(bytes)) => bytes,
                ResourceKey::Word { .. } => panic!("word resource used as a room asset"),
            },
        }
    }
    pub(crate) fn resource_key(&self, id: ResourceId) -> ResourceKey {
        self.keys.resources[&id]
    }
    pub(crate) fn resource_kind(&self, id: ResourceId) -> ResourceKind {
        self.definition().expect("prepared resource").resources[&id]
    }
    pub(crate) fn resource_word(&self, id: ResourceId, kind: ResourceKind, bank: u8) -> u16 {
        assert_eq!(self.resource_kind(id), kind, "resource kind mismatch");
        match self.resource_key(id) {
            ResourceKey::Word {
                bank: actual,
                offset,
            } if actual == bank => offset,
            _ => panic!("resource is not a word in the required bank"),
        }
    }
    pub(crate) fn resource_address(&self, id: ResourceId, kind: ResourceKind) -> u32 {
        assert_eq!(self.resource_kind(id), kind, "resource kind mismatch");
        match self.resource_key(id) {
            ResourceKey::Long(LegacyAddress24([lo, hi, bank])) => {
                u32::from_le_bytes([lo, hi, bank, 0])
            }
            ResourceKey::Word { bank, offset } => u32::from(bank) << 16 | u32::from(offset),
        }
    }
    pub(crate) fn skips_cre_upload(&self, exit: &Identity<ExitId>) -> bool {
        match exit {
            Identity::LegacyUnresolved(key) => *key == 0x947a,
            Identity::Known(id) => {
                self.definition().expect("prepared exit").exits[id].skip_cre_upload
            }
        }
    }
}
fn identity<I: Copy>(reverse: &BTreeMap<u16, I>, value: u16) -> Identity<I> {
    reverse
        .get(&value)
        .map_or(Identity::LegacyUnresolved(value), |&id| Identity::Known(id))
}
fn key<I: Ord>(keys: &BTreeMap<I, u16>, value: &Identity<I>) -> u16 {
    match value {
        Identity::LegacyUnresolved(key) => *key,
        Identity::Known(id) => keys[id],
    }
}
fn reverse<I: Copy + Ord, K: Copy + Ord, V>(
    definitions: &BTreeMap<I, V>,
    keys: &BTreeMap<I, K>,
) -> Result<BTreeMap<K, I>, WorldDefinitionError> {
    if definitions.keys().ne(keys.keys()) {
        return Err(WorldDefinitionError::MissingCompatibilityKey);
    }
    let mut result = BTreeMap::new();
    for (&id, &key) in keys {
        if result.insert(key, id).is_some() {
            return Err(WorldDefinitionError::AmbiguousCompatibilityKey);
        }
    }
    Ok(result)
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
fn validate(world: &WorldDefinition) -> Result<(), WorldDefinitionError> {
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
fn validate_resource(
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

// Public constructor vocabulary, reachable without exposing Session or actor storage.
pub use crate::game::actors::content::{
    ActorCallbackId, ActorCompatibilityKeys, ActorContentError, ActorDefinitions,
    ActorProgramDefinition, ActorProgramId, ActorTemplateId, BehaviorId, CollisionBox,
    CollisionLayout, CollisionLayoutId, CollisionPart, GameplayRevision, PartRoleId,
    PopulationDefinition, PopulationEntry, PopulationEntryId, PopulationId, PoseDefinition, PoseId,
    PosePart, SpeciesDefinition, SpeciesId,
};
pub use crate::game::actors::media::{
    ActorMaterialDefinition, ActorMediaDefinitions, ActorVisualResource, AppearanceDefinition,
    AppearanceId, FrameDefinition, FrameId, MaterialId, MediaRevision, VisualPieceDefinition,
    VisualResourceId,
};
pub use sm_game::geometry::Point;
impl WorldBinding {
    /// Validates all authored references before a Game can be opened. Later template
    /// parts remain compatible live records; this constructor authors part zero only.
    pub fn prepared_with_actors(
        world: WorldDefinition,
        keys: CompatibilityKeys,
        actors: ActorDefinitions,
        actor_keys: ActorCompatibilityKeys,
        media: ActorMediaDefinitions,
    ) -> Result<Arc<Self>, ActorContentError> {
        let mut binding = Self::prepared(world, keys).map_err(ActorContentError::World)?;
        let actors = crate::game::actors::content::ActorCatalog::prepared(
            actors, actor_keys, &binding, &media,
        )?;
        let media =
            crate::game::actors::media::ActorMediaCatalog::prepared(media, &actors, &binding)?;
        let owned = Arc::get_mut(&mut binding).expect("new unshared world binding");
        owned.actor_catalog = Arc::new(actors);
        owned.actor_media = Arc::new(media);
        Ok(binding)
    }
    pub(crate) fn actors(&self) -> &crate::game::actors::content::ActorCatalog {
        &self.actor_catalog
    }
    pub(crate) fn actor_media(&self) -> &crate::game::actors::media::ActorMediaCatalog {
        &self.actor_media
    }
    pub fn actor_gameplay_revision(&self) -> Option<GameplayRevision> {
        self.actor_catalog.revision()
    }
    pub fn actor_media_revision(&self) -> Option<MediaRevision> {
        self.actor_media.revision()
    }
}

pub use super::prepared_assets::{PreparedRoomAssets, PreparedRoomError};
pub use super::prepared_tables::{PreparedTableError, PreparedTileTables};
