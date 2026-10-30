//! Immutable authored topology and validated compatibility bindings.
//! Construction validates every reference before an owned Game is opened.
use super::content::{AssetRef, AssetSource, DoorList};
use crate::room_entry::AssetSlot;
pub use sm_data::world::*;
use std::{
    collections::BTreeMap,
    hash::{DefaultHasher, Hash, Hasher},
    sync::Arc,
};
#[derive(Clone, Debug, PartialEq)]
pub struct WorldBinding {
    tile_tables: Option<Arc<PreparedTileTables>>,
    room_assets: Option<Arc<PreparedRoomAssets>>,
    actor_catalog: Arc<crate::actors::content::ActorCatalog>,
    actor_media: Arc<crate::actors::media::ActorMediaCatalog>,
    pub revision: WorldRevision,
    pub content: BoundContent,
    pub keys: CompatibilityKeys,
    pub starts_by_key: BTreeMap<LegacyStartSite, StartId>,
    pub rooms_by_key: BTreeMap<u16, RoomId>,
    pub states_by_key: BTreeMap<u16, StateId>,
    pub exits_by_key: BTreeMap<u16, ExitId>,
    pub door_lists_by_key: BTreeMap<u16, DoorListId>,
    pub resources_by_key: BTreeMap<(ResourceKind, ResourceKey), ResourceId>,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum BoundContent {
    CanonicalLegacy,
    Prepared(WorldDefinition),
}

impl WorldBinding {
    pub fn canonical_legacy() -> Arc<Self> {
        Arc::new(Self {
            tile_tables: None,
            room_assets: None,
            actor_catalog: Arc::new(crate::actors::content::ActorCatalog::LegacyBound),
            actor_media: Arc::new(crate::actors::media::ActorMediaCatalog::LegacyBound),
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
        for (lane, bytes) in revision.as_chunks_mut::<8>().0.iter_mut().enumerate() {
            let mut hasher = DefaultHasher::new();
            ("room-entry-world-v1", lane as u64, &world, &keys).hash(&mut hasher);
            bytes.copy_from_slice(&hasher.finish().to_le_bytes());
        }
        Ok(Arc::new(Self {
            tile_tables: None,
            room_assets: None,
            actor_catalog: Arc::new(crate::actors::content::ActorCatalog::LegacyBound),
            actor_media: Arc::new(crate::actors::media::ActorMediaCatalog::LegacyBound),
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
    pub fn with_tables(
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
        for (lane, bytes) in owned
            .revision
            .0
            .as_chunks_mut::<8>()
            .0
            .iter_mut()
            .enumerate()
        {
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
    pub fn compatibility_keys(&self) -> &CompatibilityKeys {
        &self.keys
    }
    pub fn prepared_room_assets(&self) -> Option<Arc<PreparedRoomAssets>> {
        self.room_assets.clone()
    }
    pub fn prepared_table(&self, source: &AssetSource) -> Option<Arc<[u8]>> {
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
    pub fn start_at(&self, area: u16, index: u16) -> StartId {
        self.starts_by_key[&LegacyStartSite { area, index }]
    }
    pub fn room_identity(&self, key: u16) -> Identity<RoomId> {
        identity(&self.rooms_by_key, key)
    }
    pub fn state_identity(&self, key: u16) -> Identity<StateId> {
        identity(&self.states_by_key, key)
    }
    pub fn exit_identity(&self, key: u16) -> Identity<ExitId> {
        identity(&self.exits_by_key, key)
    }
    pub fn room_key(&self, value: &Identity<RoomId>) -> u16 {
        key(&self.keys.rooms, value)
    }
    pub fn state_key(&self, value: &Identity<StateId>) -> u16 {
        key(&self.keys.states, value)
    }
    pub fn exit_key(&self, value: &Identity<ExitId>) -> u16 {
        key(&self.keys.exits, value)
    }
    pub fn door_list(&self, key: u16) -> DoorList {
        self.door_lists_by_key
            .get(&key)
            .map_or(DoorList::Legacy(key), |&id| DoorList::Prepared(id))
    }
    pub fn door_list_key(&self, value: &DoorList) -> u16 {
        match value {
            DoorList::Legacy(key) => *key,
            DoorList::Prepared(id) => self.keys.door_lists[id],
        }
    }
    pub fn asset(&self, slot: AssetSlot, bytes: [u8; 3]) -> AssetRef {
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
    pub fn asset_bytes(&self, value: &AssetRef) -> [u8; 3] {
        match value {
            AssetRef::Legacy(bytes) => *bytes,
            AssetRef::Prepared(id) => match self.resource_key(*id) {
                ResourceKey::Long(LegacyAddress24(bytes)) => bytes,
                ResourceKey::Word { .. } => panic!("word resource used as a room asset"),
            },
        }
    }
    pub fn resource_key(&self, id: ResourceId) -> ResourceKey {
        self.keys.resources[&id]
    }
    pub fn resource_kind(&self, id: ResourceId) -> ResourceKind {
        self.definition().expect("prepared resource").resources[&id]
    }
    pub fn resource_word(&self, id: ResourceId, kind: ResourceKind, bank: u8) -> u16 {
        assert_eq!(self.resource_kind(id), kind, "resource kind mismatch");
        match self.resource_key(id) {
            ResourceKey::Word {
                bank: actual,
                offset,
            } if actual == bank => offset,
            _ => panic!("resource is not a word in the required bank"),
        }
    }
    pub fn resource_address(&self, id: ResourceId, kind: ResourceKind) -> u32 {
        assert_eq!(self.resource_kind(id), kind, "resource kind mismatch");
        match self.resource_key(id) {
            ResourceKey::Long(LegacyAddress24([lo, hi, bank])) => {
                u32::from_le_bytes([lo, hi, bank, 0])
            }
            ResourceKey::Word { bank, offset } => u32::from(bank) << 16 | u32::from(offset),
        }
    }
    pub fn skips_cre_upload(&self, exit: &Identity<ExitId>) -> bool {
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
// Public constructor vocabulary, reachable without exposing Session or actor storage.
pub use crate::actors::content::{
    ActorCallbackId, ActorCompatibilityKeys, ActorContentError, ActorDefinitions,
    ActorProgramDefinition, ActorProgramId, ActorTemplateId, BehaviorId, CollisionBox,
    CollisionLayout, CollisionLayoutId, CollisionPart, GameplayRevision, PartRoleId,
    PopulationDefinition, PopulationEntry, PopulationEntryId, PopulationId, PoseDefinition, PoseId,
    PosePart, SpeciesDefinition, SpeciesId,
};
pub use crate::actors::media::{
    ActorMaterialDefinition, ActorMediaDefinitions, ActorVisualResource, AppearanceDefinition,
    AppearanceId, FrameDefinition, FrameId, MaterialId, MediaRevision, VisualPieceDefinition,
    VisualResourceId,
};
pub use crate::geometry::Point;
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
        let actors =
            crate::actors::content::ActorCatalog::prepared(actors, actor_keys, &binding, &media)?;
        let media = crate::actors::media::ActorMediaCatalog::prepared(media, &actors, &binding)?;
        let owned = Arc::get_mut(&mut binding).expect("new unshared world binding");
        owned.actor_catalog = Arc::new(actors);
        owned.actor_media = Arc::new(media);
        Ok(binding)
    }
    pub fn actors(&self) -> &crate::actors::content::ActorCatalog {
        &self.actor_catalog
    }
    pub fn actor_media(&self) -> &crate::actors::media::ActorMediaCatalog {
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
