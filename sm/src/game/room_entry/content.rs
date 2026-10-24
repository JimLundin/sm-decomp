//! Retained live content captures and typed room descriptors.
#![forbid(unsafe_code)]
use super::{
    api::StartSite,
    effects::{AssetRead, AssetSlot, StagingSource, VramSlot},
    world::{
        BackgroundTarget, DoorListId, ExitId, Identity, ResourceId, RoomId, StartId, StateId,
        TilesetId, WorldBinding,
    },
};
use crate::rom::RomImage;
use std::sync::Arc;
/// P itself is the prepared position; there is no second discriminant identifying its type.
/// Legacy identity getters use this pinned immutable world and live captured allocation.
#[derive(Clone)]
pub enum Capture<P> {
    Legacy {
        image: RomImage,
        base: usize,
        access: RecordAccess,
        world: Arc<WorldBinding>,
    },
    Prepared {
        world: Arc<WorldBinding>,
        position: P,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecordAccess {
    Native,
    LittleEndian,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SelectorPosition {
    pub room: RoomId,
    pub index: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackgroundPart {
    Opcode,
    Operand,
    MatchedTransfer,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BackgroundPosition {
    pub state: StateId,
    pub index: usize,
    pub part: BackgroundPart,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClosingHeaderPosition {
    pub direction: u16,
}
pub type HeaderCapture = Capture<RoomId>;
pub type StateCapture = Capture<StateId>;
pub type DoorCapture = Capture<ExitId>;
pub type StationCapture = Capture<StartId>;
pub type TilesetCapture = Capture<TilesetId>;
pub type OperandCapture = Capture<SelectorPosition>;
pub type BackgroundCapture = Capture<BackgroundPosition>;
pub type ClosingHeaderCapture = Capture<ClosingHeaderPosition>;
/// Normal door and closing-door acquisitions remain distinct Host methods/policies.
pub type ClosingDoorCapture = Capture<ExitId>;
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DoorList {
    Legacy(u16),
    Prepared(DoorListId),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SelectorCursor {
    Legacy(u16),
    Prepared(SelectorPosition),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ObjectCursor {
    LegacyInitial(u16),
    LegacyDoor(i32),
    Prepared { state: StateId, index: usize },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BackgroundCursor {
    Legacy(u16),
    Prepared(BackgroundPosition),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScrollCursor {
    Legacy(u16),
    Prepared { state: StateId, index: usize },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TilesetSelection {
    Legacy(u8),
    Prepared(TilesetId),
}
/// Persistent room asset identity: exactly three projected bytes, no transfer variants.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AssetRef {
    Legacy([u8; 3]),
    Prepared(ResourceId),
}
/// Transient transfer operand. It never appears as a persistent RoomEntry field.
#[derive(Clone, Debug, PartialEq)]
pub enum AssetSource {
    Bound(AssetRef),
    Shared(SharedAsset),
    Staging(StagingSource),
    FxTilemap(u16),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BackgroundDestination {
    Legacy(u16),
    Prepared(BackgroundTarget),
}
#[derive(Clone, Debug, PartialEq)]
pub enum VramDestination {
    Legacy(u16),
    Slot(VramSlot),
    Prepared(BackgroundTarget),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SetupAction {
    Legacy { kind: SetupKind, code: u16 },
    Prepared(ResourceId),
}
#[derive(Clone, Debug, PartialEq)]
pub enum LoadMode {
    Initial,
    Door,
}
#[derive(Clone, Debug, PartialEq)]
pub enum Axis {
    X,
    Y,
}
#[derive(Clone, Debug, PartialEq)]
pub enum SelectorKind {
    Default,
    Event,
    Boss,
    BossOne,
    MorphAndMissiles,
    PowerBombs,
    Unknown(u32),
}
#[derive(Clone, Debug, PartialEq)]
pub enum SelectorLayout {
    ConditionAndTarget,
    TargetOnly,
}
#[derive(Clone, Debug, PartialEq)]
pub enum HeaderScalar {
    Index,
    Area,
    MapX,
    MapY,
    Width,
    Height,
    UpScroll,
    DownScroll,
    Cre,
}
#[derive(Clone, Debug, PartialEq)]
pub enum StateScalar {
    MusicData,
    MusicTrack,
    Fx,
    EnemyPopulation,
    EnemyTilesets,
    Layer2Policy,
    MainCode,
}
#[derive(Clone, Debug, PartialEq)]
pub enum DoorScalar {
    Properties,
    Direction,
    ArrivalX,
    ArrivalY,
    Distance,
    MapFlags,
    CloseX,
    CloseY,
}
#[derive(Clone, Debug, PartialEq)]
pub enum StationScalar {
    CameraX,
    CameraY,
    SamusXOffset,
    SamusYOffset,
}
#[derive(Clone, Debug, PartialEq)]
pub enum TilesetAsset {
    TileTable,
    Tiles,
    Palette,
}
#[derive(Clone, Debug, PartialEq)]
pub enum SharedAsset {
    CreTiles,
    CreTileTable,
    BackgroundBlankLow,
    BackgroundBlankHigh,
    FxBlankLow,
    FxBlankHigh,
    TransitionBackground,
}
#[derive(Clone, Debug, PartialEq)]
pub enum BackgroundWord {
    Opcode,
    SourceLow,
    SourceBankWord,
    ByteCount,
}
#[derive(Clone, Debug, PartialEq)]
pub enum BackgroundWidth {
    Opcode,
    Transfer,
    Decompress,
    MatchAndTransfer,
}
#[derive(Clone, Debug, PartialEq)]
pub enum ScrollDefinition {
    Generated(u8),
    Records(ScrollCursor),
}

/// Only acquisition, diagnostics and current-session observations cross this seam.
/// Pure retained-token reads and cursor arithmetic are inherent helpers (signature ledger).
pub trait EntryContent {
    fn contact_bts(&mut self) -> u8;
    fn contact_exit(&mut self, list: &DoorList, bts: u8) -> Identity<ExitId>;
    fn contact_has_destination(&mut self, exit: &Identity<ExitId>) -> bool;
    fn capture_station(&mut self, site: StartSite) -> StationCapture;
    fn capture_header(&mut self, room: &Identity<RoomId>) -> HeaderCapture;
    fn capture_start_header(&mut self, room: &Identity<RoomId>) -> HeaderCapture;
    fn selector(&mut self, cursor: &SelectorCursor) -> (SelectorKind, SelectorCursor);
    fn capture_operand(&mut self, cursor: &SelectorCursor) -> OperandCapture;
    fn direct_target(&mut self, cursor: &SelectorCursor) -> Identity<StateId>;
    fn invalid_selector(&mut self, address: u32) -> Option<SelectorCursor>;
    fn capture_state(&mut self, state: &Identity<StateId>) -> StateCapture;
    fn capture_tileset(&mut self, selection: TilesetSelection) -> TilesetCapture;
    fn capture_door(&mut self, exit: &Identity<ExitId>) -> DoorCapture;
    fn scroll_pair(&mut self, cursor: &ScrollCursor) -> [u8; 2];
    fn object_exists(&mut self, cursor: &ObjectCursor) -> bool;
    fn selected_background(
        &mut self,
        state: &Identity<StateId>,
        mode: LoadMode,
    ) -> Option<BackgroundCursor>;
    fn capture_background(
        &mut self,
        cursor: &BackgroundCursor,
        mode: LoadMode,
    ) -> BackgroundCapture;
    fn live_asset(&self, slot: AssetSlot, read: AssetRead) -> AssetSource;
    fn capture_closing_door(&mut self, exit: &Identity<ExitId>) -> ClosingDoorCapture;
    fn capture_closing_header(&mut self, direction: u16) -> ClosingHeaderCapture;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SetupKind {
    Door,
    Room,
}
pub(crate) use super::content_background::SpawnObject;

impl<P> Capture<P> {
    pub(super) fn raw_byte(&self, offset: usize) -> u8 {
        match self {
            Self::Legacy { image, base, .. } => image.read(
                base.checked_add(offset)
                    .expect("ROM record offset overflow"),
            ),
            Self::Prepared { .. } => panic!("raw read of prepared content"),
        }
    }
    pub(super) fn raw_bytes<const N: usize>(&self, offset: usize) -> [u8; N] {
        core::array::from_fn(|i| self.raw_byte(offset + i))
    }
    pub(super) fn raw_word(&self, offset: usize) -> u16 {
        let bytes = self.raw_bytes(offset);
        match self {
            Self::Legacy {
                access: RecordAccess::LittleEndian,
                ..
            } => u16::from_le_bytes(bytes),
            _ => u16::from_ne_bytes(bytes),
        }
    }
    pub(super) fn world(&self) -> &WorldBinding {
        match self {
            Self::Legacy { world, .. } | Self::Prepared { world, .. } => world,
        }
    }
}
impl Capture<StartId> {
    pub(crate) fn room(&self) -> Identity<RoomId> {
        match self {
            Self::Legacy { world, .. } => world.room_identity(self.raw_word(0)),
            Self::Prepared { world, position } => {
                Identity::Known(world.definition().unwrap().starts[position].room)
            }
        }
    }
    pub(crate) fn exit(&self) -> Identity<ExitId> {
        match self {
            Self::Legacy { world, .. } => world.exit_identity(self.raw_word(2)),
            Self::Prepared { world, position } => {
                Identity::Known(world.definition().unwrap().starts[position].exit)
            }
        }
    }
    pub(crate) fn scalar(&self, field: StationScalar) -> u16 {
        match self {
            Self::Legacy { .. } => self.raw_word(match field {
                StationScalar::CameraX => 6,
                StationScalar::CameraY => 8,
                StationScalar::SamusYOffset => 10,
                StationScalar::SamusXOffset => 12,
            }),
            Self::Prepared { world, position } => {
                let station = &world.definition().unwrap().starts[position];
                match field {
                    StationScalar::CameraX => station.camera_x,
                    StationScalar::CameraY => station.camera_y,
                    StationScalar::SamusYOffset => station.samus_y_offset,
                    StationScalar::SamusXOffset => station.samus_x_offset,
                }
            }
        }
    }
}
impl Capture<RoomId> {
    pub(crate) fn scalar(&self, field: HeaderScalar) -> u16 {
        u16::from(match self {
            Self::Legacy { .. } => self.raw_byte(match field {
                HeaderScalar::Index => 0,
                HeaderScalar::Area => 1,
                HeaderScalar::MapX => 2,
                HeaderScalar::MapY => 3,
                HeaderScalar::Width => 4,
                HeaderScalar::Height => 5,
                HeaderScalar::UpScroll => 6,
                HeaderScalar::DownScroll => 7,
                HeaderScalar::Cre => 8,
            }),
            Self::Prepared { world, position } => {
                let room = &world.definition().unwrap().rooms[position];
                match field {
                    HeaderScalar::Index => room.index,
                    HeaderScalar::Area => room.area,
                    HeaderScalar::MapX => room.map_x,
                    HeaderScalar::MapY => room.map_y,
                    HeaderScalar::Width => room.width,
                    HeaderScalar::Height => room.height,
                    HeaderScalar::UpScroll => room.up_scroll,
                    HeaderScalar::DownScroll => room.down_scroll,
                    HeaderScalar::Cre => room.cre,
                }
            }
        })
    }
    pub(crate) fn doors(&self) -> DoorList {
        match self {
            Self::Legacy { world, .. } => world.door_list(self.raw_word(9)),
            Self::Prepared { world, position } => {
                DoorList::Prepared(world.definition().unwrap().rooms[position].doors)
            }
        }
    }
}
impl Capture<SelectorPosition> {
    pub(crate) fn condition_id(&self) -> u8 {
        match self {
            Self::Legacy { .. } => self.raw_byte(0),
            Self::Prepared { world, position } => {
                match world.definition().unwrap().rooms[&position.room].selectors[position.index]
                    .condition
                {
                    super::world::StateCondition::Event(id)
                    | super::world::StateCondition::Boss(id) => id,
                    _ => panic!("selector has no condition operand"),
                }
            }
        }
    }
    pub(crate) fn target(&self) -> Identity<StateId> {
        match self {
            Self::Legacy { world, .. } => world.state_identity(self.raw_word(1)),
            Self::Prepared { world, position } => Identity::Known(
                world.definition().unwrap().rooms[&position.room].selectors[position.index].state,
            ),
        }
    }
}
impl Capture<ExitId> {
    pub(crate) fn room(&self) -> Identity<RoomId> {
        match self {
            Self::Legacy { world, .. } => world.room_identity(self.raw_word(0)),
            Self::Prepared { world, position } => {
                Identity::Known(world.definition().unwrap().exits[position].destination)
            }
        }
    }
    pub(crate) fn scalar(&self, field: DoorScalar) -> u16 {
        match self {
            Self::Legacy { .. } => match field {
                DoorScalar::Properties => u16::from_le_bytes(self.raw_bytes(2)),
                DoorScalar::Distance => self.raw_word(8),
                _ => u16::from(self.raw_byte(match field {
                    DoorScalar::Direction => 3,
                    DoorScalar::ArrivalX => 6,
                    DoorScalar::ArrivalY => 7,
                    DoorScalar::MapFlags => 2,
                    DoorScalar::CloseX => 4,
                    DoorScalar::CloseY => 5,
                    _ => unreachable!(),
                })),
            },
            Self::Prepared { world, position } => {
                let arrival = &world.definition().unwrap().exits[position].arrival;
                match field {
                    DoorScalar::Properties => {
                        u16::from_le_bytes([arrival.flags, arrival.direction])
                    }
                    DoorScalar::Distance => arrival.distance as u16,
                    DoorScalar::Direction => arrival.direction as u16,
                    DoorScalar::ArrivalX => arrival.x as u16,
                    DoorScalar::ArrivalY => arrival.y as u16,
                    DoorScalar::MapFlags => arrival.flags as u16,
                    DoorScalar::CloseX => arrival.close_x as u16,
                    DoorScalar::CloseY => arrival.close_y as u16,
                }
            }
        }
    }
    pub(crate) fn setup(&self) -> Option<SetupAction> {
        match self {
            Self::Legacy { .. } => {
                let code = self.raw_word(10);
                (code != 0).then_some(SetupAction::Legacy {
                    kind: SetupKind::Door,
                    code,
                })
            }
            Self::Prepared { world, position } => world.definition().unwrap().exits[position]
                .setup
                .map(SetupAction::Prepared),
        }
    }
    pub(crate) fn closing_position(&self, axis: Axis) -> u8 {
        self.scalar(match axis {
            Axis::X => DoorScalar::CloseX,
            Axis::Y => DoorScalar::CloseY,
        }) as u8
    }
    pub(crate) fn closing_coordinates_native(&self) -> u16 {
        match self {
            Self::Legacy { .. } => u16::from_ne_bytes(self.raw_bytes(4)),
            Self::Prepared { world, position } => {
                let arrival = &world.definition().unwrap().exits[position].arrival;
                u16::from_ne_bytes([arrival.close_x, arrival.close_y])
            }
        }
    }
}
impl SelectorKind {
    pub(crate) fn from_legacy(address: u32) -> Self {
        match address {
            9430502 => Self::Default,
            9430546 => Self::Event,
            9430569 => Self::Boss,
            9430527 => Self::BossOne,
            9430610 => Self::MorphAndMissiles,
            9430633 => Self::PowerBombs,
            other => Self::Unknown(other),
        }
    }
}
impl SelectorCursor {
    pub(crate) fn for_room(room: &Identity<RoomId>, world: &WorldBinding) -> Self {
        match room {
            Identity::Known(room) => Self::Prepared(SelectorPosition {
                room: *room,
                index: 0,
            }),
            Identity::LegacyUnresolved(_) => Self::Legacy(world.room_key(room).wrapping_add(11)),
        }
    }
    pub(crate) fn inline_state(&self, world: &WorldBinding) -> Identity<StateId> {
        match self {
            Self::Legacy(cursor) => world.state_identity(*cursor),
            Self::Prepared(position) => Identity::Known(
                world.definition().unwrap().rooms[&position.room].selectors[position.index].state,
            ),
        }
    }
    pub(crate) fn next(self, layout: SelectorLayout, world: &WorldBinding) -> Option<Self> {
        match self {
            Self::Legacy(cursor) => {
                let next = cursor.wrapping_add(match layout {
                    SelectorLayout::ConditionAndTarget => 3,
                    SelectorLayout::TargetOnly => 2,
                });
                (next != 0).then_some(Self::Legacy(next))
            }
            Self::Prepared(position) => {
                let index = position.index + 1;
                (index
                    < world.definition().unwrap().rooms[&position.room]
                        .selectors
                        .len())
                .then_some(Self::Prepared(SelectorPosition { index, ..position }))
            }
        }
    }
}
impl ScrollCursor {
    pub(crate) fn next(self) -> Self {
        match self {
            Self::Legacy(cursor) => Self::Legacy(cursor.wrapping_add(2)),
            Self::Prepared { state, index } => Self::Prepared {
                state,
                index: index + 2,
            },
        }
    }
}
impl ObjectCursor {
    pub(crate) fn next(self, mode: LoadMode) -> Self {
        match self {
            Self::LegacyInitial(cursor) => Self::LegacyInitial(cursor.wrapping_add(6)),
            Self::LegacyDoor(cursor) => match mode {
                LoadMode::Initial => Self::LegacyInitial((cursor as u16).wrapping_add(6)),
                LoadMode::Door => Self::LegacyDoor(cursor + 6),
            },
            Self::Prepared { state, index } => Self::Prepared {
                state,
                index: index + 1,
            },
        }
    }
}
