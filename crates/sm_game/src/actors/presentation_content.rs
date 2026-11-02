//! Authored actor presentation declarations and their admitted immutable content.
//! Tools resolves these declarations; gameplay never discovers IDs from RAM or OAM.
#![forbid(unsafe_code)]
use crate::{
    actors::{
        content::GameplayRevision,
        media::{MediaRevision, VisualResourceId},
    },
    presentation_workspace::Role as WorkspaceRole,
    room_entry::world::{ResourceId, ResourceKind, WorldRevision},
    scene::{
        actor_presentation::{Binding, DataRole},
        hdma_sources::{MajorHdmaRole, SourceBinding},
        records::{AssetRef, ObjectSizeVariants, Revision},
        state::{DisplayError, WordOrder},
    },
    session::Session,
};
use std::{collections::BTreeMap, rc::Rc, sync::Arc};

pub const MANIFEST_SCHEMA: u16 = 2;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AllocationRole {
    Loaded,
    Native,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaptureOffset {
    Linear(usize),
    Wrapping(u16),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Capture {
    pub allocation: AllocationRole,
    pub bank: u8,
    pub address: u16,
    pub offset: CaptureOffset,
}
/// Original cannon program inputs; no selected pose or decoded table is stored.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CannonSourceRole {
    PosePointers,
    UploadPointers,
    BodyHeights,
}
impl CannonSourceRole {
    pub const ALL: [Self; 3] = [Self::PosePointers, Self::UploadPointers, Self::BodyHeights];
    pub fn capture(self) -> Capture {
        let (allocation, bank, address) = match self {
            Self::PosePointers => (AllocationRole::Loaded, 0x90, 0xc7df),
            Self::UploadPointers => (AllocationRole::Loaded, 0x90, 0xc7a5),
            Self::BodyHeights => (AllocationRole::Native, 0x91, 0xb629),
        };
        Capture {
            allocation,
            bank,
            address,
            offset: CaptureOffset::Linear(0),
        }
    }
}
/// Full emitted words preserve palette, priority and flip identity; these are
/// behavior keys, never inferred asset IDs. Custom media supplies the bindings.
pub const CANNON_CHARACTERS: [u16; 4] = [0x281f, 0x681f, 0xa81f, 0xe81f];
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct VariantId(pub u32);
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceDeclaration {
    pub identity: ResourceId,
    pub kind: ResourceKind,
    pub capture: Capture,
    pub order: WordOrder,
    pub parts: Vec<VariantId>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DataDeclaration {
    pub identity: ResourceId,
    pub capture: Capture,
    pub order: WordOrder,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChunkDeclaration {
    pub media: AssetRef,
    pub capture: Capture,
    pub extent: usize,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum QueueDeclaration {
    ActorState {
        address: u16,
    },
    WorkspacePhysical {
        address: usize,
    },
    WorkspaceRole {
        role: WorkspaceRole,
        offset: usize,
    },
    Asset {
        asset: AssetRef,
        capture: Capture,
        length: usize,
        offset: usize,
    },
    Hud {
        offset: usize,
    },
    Objects {
        offset: usize,
    },
    QueuedScroll {
        offset: usize,
    },
    Scroll {
        slot: usize,
        half: usize,
        wrapped: bool,
        offset: usize,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActorManifest {
    pub schema: u16,
    pub world: WorldRevision,
    pub gameplay: GameplayRevision,
    pub actor_media: MediaRevision,
    pub scene_media: Revision,
    pub loaded_source: [u8; 32],
    pub native_source: [u8; 32],
    pub variants: BTreeMap<VariantId, BTreeMap<u8, ObjectSizeVariants>>,
    pub resources: BTreeMap<VisualResourceId, ResourceDeclaration>,
    pub sprites: BTreeMap<(u8, u16), ResourceDeclaration>,
    pub room_sprites: BTreeMap<(u8, u16), ResourceDeclaration>,
    pub chunks: Vec<ChunkDeclaration>,
    pub data: BTreeMap<DataRole, DataDeclaration>,
    pub trails: BTreeMap<(bool, u16), VariantId>,
    pub cannon_parts: BTreeMap<u16, VariantId>,
    pub cannon_sources: BTreeMap<CannonSourceRole, Capture>,
    pub grapple_segment: VariantId,
    pub grapple_end: VariantId,
    pub compatible: BTreeMap<(u8, u16, bool), VisualResourceId>,
    pub queued_sources: BTreeMap<[u8; 3], QueueDeclaration>,
    pub producer_bindings: BTreeMap<MajorHdmaRole, SourceBinding>,
}
/// Supplied independently by the authored content contract, never from a selector.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActorManifestContract(ActorManifest);
impl ActorManifestContract {
    pub fn new(expected: ActorManifest) -> Self {
        Self(expected)
    }
    pub fn expected(&self) -> &ActorManifest {
        &self.0
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActorImportRequest {
    pub supplied: ActorManifest,
    pub contract: ActorManifestContract,
}
impl ActorImportRequest {
    pub fn declarations(&self) -> Result<Vec<u8>, DisplayError> {
        if self.supplied != *self.contract.expected() {
            return Err(DisplayError::Identity);
        }
        self.supplied.declarations()
    }
}
/// Actual observed original allocations. Cloning retains allocation identity.
#[derive(Clone)]
pub struct SourceAllocations {
    pub loaded: crate::rom::RomImage,
    pub native: crate::rom::RomImage,
}
impl SourceAllocations {
    pub fn image(&self, role: AllocationRole) -> &crate::rom::RomImage {
        match role {
            AllocationRole::Loaded => &self.loaded,
            AllocationRole::Native => &self.native,
        }
    }
    pub fn same_allocations(&self, other: &Self) -> bool {
        self.loaded.same_allocation(&other.loaded) && self.native.same_allocation(&other.native)
    }
}
/// One admitted immutable binding; numeric actors, buffers and queues stay in Session.
#[derive(Clone)]
pub struct PreparedActorContent {
    binding: Rc<Binding>,
    manifest: Arc<ActorManifest>,
    sources: SourceAllocations,
    declarations: Arc<[u8]>,
}
impl PreparedActorContent {
    /// Tools calls this only after constructing Binding against independent descriptions.
    pub fn new(
        binding: Binding,
        manifest: ActorManifest,
        sources: SourceAllocations,
    ) -> Result<Self, DisplayError> {
        let declarations = manifest.declarations()?.into();
        Ok(Self {
            binding: Rc::new(binding),
            manifest: Arc::new(manifest),
            sources,
            declarations,
        })
    }
    pub fn binding(&self) -> &Binding {
        &self.binding
    }
    pub fn manifest(&self) -> &ActorManifest {
        &self.manifest
    }
    pub fn declarations(&self) -> &[u8] {
        &self.declarations
    }
    pub fn validate_contract(&self, contract: &ActorManifestContract) -> Result<(), DisplayError> {
        if self.manifest.as_ref() == contract.expected() {
            Ok(())
        } else {
            Err(DisplayError::Identity)
        }
    }
    pub fn validate_request(&self, request: &ActorImportRequest) -> Result<(), DisplayError> {
        if request.supplied != *request.contract.expected() {
            return Err(DisplayError::Identity);
        }
        self.validate_contract(&request.contract)
    }
    pub fn check_sources(&self, sources: &SourceAllocations) -> Result<(), DisplayError> {
        if self.sources.same_allocations(sources) {
            Ok(())
        } else {
            Err(DisplayError::Identity)
        }
    }
    pub fn check(&self, session: &Session) -> Result<(), DisplayError> {
        if session.world.revision() != self.manifest.world
            || session.world.actors().revision() != Some(self.manifest.gameplay)
        {
            return Err(DisplayError::Revision);
        }
        self.binding.check(session)
    }
    pub fn admit(&self, session: &mut Session) -> Result<(), DisplayError> {
        if session.world.revision() != self.manifest.world
            || session.world.actors().revision() != Some(self.manifest.gameplay)
        {
            return Err(DisplayError::Revision);
        }
        self.binding.admit(session)
    }
}

fn count(bytes: &mut Vec<u8>, value: usize) -> Result<(), DisplayError> {
    bytes.extend_from_slice(
        &u64::try_from(value)
            .map_err(|_| DisplayError::Source)?
            .to_le_bytes(),
    );
    Ok(())
}
fn asset(bytes: &mut Vec<u8>, value: AssetRef) {
    bytes.extend_from_slice(&value.id.0.to_le_bytes());
    bytes.extend_from_slice(&value.revision.0);
}
fn capture(bytes: &mut Vec<u8>, value: Capture) -> Result<(), DisplayError> {
    bytes.push(match value.allocation {
        AllocationRole::Loaded => 0,
        AllocationRole::Native => 1,
    });
    bytes.push(value.bank);
    bytes.extend_from_slice(&value.address.to_le_bytes());
    match value.offset {
        CaptureOffset::Linear(offset) => {
            bytes.push(0);
            count(bytes, offset)?;
        }
        CaptureOffset::Wrapping(offset) => {
            bytes.push(1);
            bytes.extend_from_slice(&offset.to_le_bytes());
        }
    }
    Ok(())
}
fn resource(bytes: &mut Vec<u8>, value: &ResourceDeclaration) -> Result<(), DisplayError> {
    bytes.extend_from_slice(&value.identity.0.to_le_bytes());
    use ResourceKind::*;
    bytes.push(match value.kind {
        Level => 0,
        TileTable => 1,
        Tiles => 2,
        Palette => 3,
        Fx => 4,
        EnemyPopulation => 5,
        EnemyTilesets => 6,
        ActorSprites => 7,
        ActorTilemap => 8,
        RoomObject => 9,
        DoorSetup => 10,
        RoomSetup => 11,
        RoomMain => 12,
        Background => 13,
    });
    capture(bytes, value.capture)?;
    bytes.push(match value.order {
        WordOrder::Little => 0,
        WordOrder::Big => 1,
    });
    count(bytes, value.parts.len())?;
    for part in &value.parts {
        bytes.extend_from_slice(&part.0.to_le_bytes());
    }
    Ok(())
}
impl ActorManifest {
    /// Ordered schema bytes for actual checkpoint/start contract comparison.
    /// Allocation pointers stay runtime evidence; original source digests remain explicit.
    pub fn declarations(&self) -> Result<Vec<u8>, DisplayError> {
        let mut b = b"SMACTORCONTENT\x02".to_vec();
        b.extend_from_slice(&self.schema.to_le_bytes());
        for digest in [
            self.world.0,
            self.gameplay.0,
            self.actor_media.0,
            self.scene_media.0,
            self.loaded_source,
            self.native_source,
        ] {
            b.extend_from_slice(&digest);
        }
        count(&mut b, self.variants.len())?;
        for (id, sets) in &self.variants {
            b.extend_from_slice(&id.0.to_le_bytes());
            count(&mut b, sets.len())?;
            for (set, variants) in sets {
                b.push(*set);
                for part in [variants.small, variants.large] {
                    asset(&mut b, part.visual);
                    b.extend_from_slice(&part.frame.to_le_bytes());
                    b.extend_from_slice(&part.part.to_le_bytes());
                }
            }
        }
        count(&mut b, self.resources.len())?;
        for (id, value) in &self.resources {
            b.extend_from_slice(&id.0.to_le_bytes());
            resource(&mut b, value)?;
        }
        for sources in [&self.sprites, &self.room_sprites] {
            count(&mut b, sources.len())?;
            for ((bank, map), value) in sources {
                b.push(*bank);
                b.extend_from_slice(&map.to_le_bytes());
                resource(&mut b, value)?;
            }
        }
        count(&mut b, self.chunks.len())?;
        for value in &self.chunks {
            asset(&mut b, value.media);
            capture(&mut b, value.capture)?;
            count(&mut b, value.extent)?;
        }
        count(&mut b, self.data.len())?;
        for (role, value) in &self.data {
            b.push(match role {
                DataRole::BeamPointers => 0,
                DataRole::EchoPoseHeights => 1,
            });
            b.extend_from_slice(&value.identity.0.to_le_bytes());
            capture(&mut b, value.capture)?;
            b.push(match value.order {
                WordOrder::Little => 0,
                WordOrder::Big => 1,
            });
        }
        count(&mut b, self.trails.len())?;
        for ((right, cursor), id) in &self.trails {
            b.push(u8::from(*right));
            b.extend_from_slice(&cursor.to_le_bytes());
            b.extend_from_slice(&id.0.to_le_bytes());
        }
        count(&mut b, self.cannon_parts.len())?;
        for (character, id) in &self.cannon_parts {
            b.extend_from_slice(&character.to_le_bytes());
            b.extend_from_slice(&id.0.to_le_bytes());
        }
        count(&mut b, self.cannon_sources.len())?;
        for (role, origin) in &self.cannon_sources {
            b.push(match role {
                CannonSourceRole::PosePointers => 0,
                CannonSourceRole::UploadPointers => 1,
                CannonSourceRole::BodyHeights => 2,
            });
            capture(&mut b, *origin)?;
        }
        for id in [self.grapple_segment, self.grapple_end] {
            b.extend_from_slice(&id.0.to_le_bytes());
        }
        count(&mut b, self.compatible.len())?;
        for ((bank, map, tilemap), id) in &self.compatible {
            b.push(*bank);
            b.extend_from_slice(&map.to_le_bytes());
            b.push(u8::from(*tilemap));
            b.extend_from_slice(&id.0.to_le_bytes());
        }
        count(&mut b, self.queued_sources.len())?;
        for (token, source) in &self.queued_sources {
            b.extend_from_slice(token);
            match source {
                QueueDeclaration::ActorState { address } => {
                    b.push(0);
                    b.extend_from_slice(&address.to_le_bytes());
                }
                QueueDeclaration::WorkspacePhysical { address } => {
                    b.push(1);
                    count(&mut b, *address)?;
                }
                QueueDeclaration::WorkspaceRole { role, offset } => {
                    b.push(2);
                    let tag = WorkspaceRole::ALL
                        .iter()
                        .position(|candidate| candidate == role)
                        .ok_or(DisplayError::Identity)?;
                    b.push(u8::try_from(tag).map_err(|_| DisplayError::Identity)?);
                    count(&mut b, *offset)?;
                }
                QueueDeclaration::Asset {
                    asset: value,
                    capture: origin,
                    length,
                    offset,
                } => {
                    b.push(3);
                    asset(&mut b, *value);
                    capture(&mut b, *origin)?;
                    count(&mut b, *length)?;
                    count(&mut b, *offset)?;
                }
                QueueDeclaration::Hud { offset } => {
                    b.push(4);
                    count(&mut b, *offset)?;
                }
                QueueDeclaration::Objects { offset } => {
                    b.push(5);
                    count(&mut b, *offset)?;
                }
                QueueDeclaration::QueuedScroll { offset } => {
                    b.push(6);
                    count(&mut b, *offset)?;
                }
                QueueDeclaration::Scroll {
                    slot,
                    half,
                    wrapped,
                    offset,
                } => {
                    b.push(7);
                    count(&mut b, *slot)?;
                    count(&mut b, *half)?;
                    b.push(u8::from(*wrapped));
                    count(&mut b, *offset)?;
                }
            }
        }
        count(&mut b, self.producer_bindings.len())?;
        for (role, binding) in &self.producer_bindings {
            if *binding != role.declaration() {
                return Err(DisplayError::Identity);
            }
            b.push(match role {
                MajorHdmaRole::MotherBrainHeader => 0,
                MajorHdmaRole::MotherBrainWindows => 1,
                MajorHdmaRole::CrocomireScroll => 2,
            });
            let SourceBinding::LiveOwner { windows, .. } = binding else {
                return Err(DisplayError::Identity);
            };
            count(&mut b, windows.len())?;
            for window in windows.iter() {
                b.push(window.bank);
                b.extend_from_slice(&window.address.to_le_bytes());
                count(&mut b, window.offset)?;
                count(&mut b, window.length)?;
            }
        }
        Ok(b)
    }
}
