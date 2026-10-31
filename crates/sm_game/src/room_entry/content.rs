//! Retained live content captures and typed room descriptors.
use super::{
    api::StartSite,
    effects::{AssetRead, AssetSlot, StagingSource, VramSlot},
    world::{
        BackgroundTarget, DoorListId, ExitId, Identity, ResourceId, RoomId, StartId, StateId,
        TilesetId, WorldBinding,
    },
};
use sm_data::cartridge::RomImage;
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
/// A prepared world answers from the Session; the required methods acquire the
/// original's records and RAM for the canonical one.
pub trait EntryContent: crate::session_access::SessionAccess {
    fn contact_bts(&mut self) -> u8;
    /// The word at `bank:address`, after the original's RomPtr diagnostic.
    fn legacy_word(&mut self, bank: u8, address: u16) -> u16;
    /// The load station index ($078B).
    fn load_station_index(&self) -> u16;
    /// The current load station's record in the original's table ($80:C4B5).
    fn legacy_station(&mut self) -> StationCapture;
    /// A room record read through the original's room_record policy.
    fn room_capture<P>(&mut self, bank: u8, address: u16) -> Capture<P>;
    /// A room record read through the RomPtr policy of load stations.
    fn native_room_capture<P>(&mut self, bank: u8, address: u16) -> Capture<P>;
    /// A record read through the PLM loaded_record policy.
    fn closing_capture<P>(&mut self, bank: u8, address: u16) -> Capture<P>;
    /// A background record in bank $8F.
    fn background_capture<P>(&mut self, address: u16, mode: LoadMode) -> Capture<P>;
    fn invalid_selector(&mut self, address: u32) -> Option<SelectorCursor>;
    /// The tileset record the original's table ($8F:E7A7) names for `index`.
    fn legacy_tileset(&mut self, index: u8) -> TilesetCapture;
    /// The asset slot's three projected bytes.
    fn asset_bytes(&self, slot: AssetSlot, read: AssetRead) -> [u8; 3];

    fn contact_exit(&mut self, list: &DoorList, bts: u8) -> Identity<ExitId> {
        match list {
            DoorList::Prepared(id) => Identity::Known(
                self.session()
                    .world
                    .definition()
                    .expect("prepared door list")
                    .door_lists[id][usize::from(bts & 0x7f)],
            ),
            DoorList::Legacy(list) => {
                let key = self.legacy_word(0x8f, list.wrapping_add(u16::from(bts & 0x7f) * 2));
                self.session().world.exit_identity(key)
            }
        }
    }
    fn contact_has_destination(&mut self, exit: &Identity<ExitId>) -> bool {
        match exit {
            Identity::Known(id) => {
                let world = self.session().world.definition().expect("prepared exit");
                let _ = &world.rooms[&world.exits[id].destination];
                true
            }
            Identity::LegacyUnresolved(key) => self.legacy_word(0x83, *key) & 0x8000 != 0,
        }
    }
    fn capture_station(&mut self, site: StartSite) -> StationCapture {
        let world = self.session().world.clone();
        match site {
            StartSite::Authored(position) => Capture::Prepared { world, position },
            StartSite::CurrentLoadStation if !world.is_canonical_legacy() => {
                let position = world.start_at(self.session().room.area, self.load_station_index());
                Capture::Prepared { world, position }
            }
            StartSite::CurrentLoadStation => self.legacy_station(),
        }
    }
    fn capture_header(&mut self, room: &Identity<RoomId>) -> HeaderCapture {
        match room {
            Identity::Known(position) => self.prepared(*position),
            Identity::LegacyUnresolved(key) => self.room_capture(0x8f, *key),
        }
    }
    fn capture_start_header(&mut self, room: &Identity<RoomId>) -> HeaderCapture {
        match room {
            Identity::Known(position) => self.prepared(*position),
            Identity::LegacyUnresolved(key) => self.native_room_capture(0x8f, *key),
        }
    }
    fn selector(&mut self, cursor: &SelectorCursor) -> (SelectorKind, SelectorCursor) {
        match cursor {
            SelectorCursor::Legacy(address) => {
                let token: OperandCapture = self.room_capture(0x8f, *address);
                let opcode = 0x8f0000 | u32::from(token.raw_word(0));
                let kind = SelectorKind::from_legacy(opcode);
                (kind, SelectorCursor::Legacy(address.wrapping_add(2)))
            }
            SelectorCursor::Prepared(position) => {
                use super::world::StateCondition;
                let world = self
                    .session()
                    .world
                    .definition()
                    .expect("prepared selector");
                let kind = match world.rooms[&position.room].selectors[position.index].condition {
                    StateCondition::Default => SelectorKind::Default,
                    StateCondition::Event(_) => SelectorKind::Event,
                    StateCondition::Boss(_) => SelectorKind::Boss,
                    StateCondition::BossOne => SelectorKind::BossOne,
                    StateCondition::MorphAndMissiles => SelectorKind::MorphAndMissiles,
                    StateCondition::PowerBombs => SelectorKind::PowerBombs,
                };
                (kind, cursor.clone())
            }
        }
    }
    fn capture_operand(&mut self, cursor: &SelectorCursor) -> OperandCapture {
        match cursor {
            SelectorCursor::Legacy(address) => self.room_capture(0x8f, *address),
            SelectorCursor::Prepared(position) => self.prepared(*position),
        }
    }
    fn direct_target(&mut self, cursor: &SelectorCursor) -> Identity<StateId> {
        match cursor {
            SelectorCursor::Legacy(address) => {
                let token: OperandCapture = self.room_capture(0x8f, *address);
                self.session().world.state_identity(token.raw_word(0))
            }
            SelectorCursor::Prepared(position) => Identity::Known(
                self.session()
                    .world
                    .definition()
                    .expect("prepared selector")
                    .rooms[&position.room]
                    .selectors[position.index]
                    .state,
            ),
        }
    }
    fn capture_state(&mut self, state: &Identity<StateId>) -> StateCapture {
        match state {
            Identity::Known(position) => self.prepared(*position),
            Identity::LegacyUnresolved(key) => self.room_capture(0x8f, *key),
        }
    }
    fn capture_tileset(&mut self, selection: TilesetSelection) -> TilesetCapture {
        match selection {
            TilesetSelection::Prepared(position) => self.prepared(position),
            TilesetSelection::Legacy(index) => self.legacy_tileset(index),
        }
    }
    fn capture_door(&mut self, exit: &Identity<ExitId>) -> DoorCapture {
        match exit {
            Identity::Known(position) => self.prepared(*position),
            Identity::LegacyUnresolved(key) => self.room_capture(0x83, *key),
        }
    }
    fn scroll_pair(&mut self, cursor: &ScrollCursor) -> [u8; 2] {
        match cursor {
            ScrollCursor::Legacy(address) => {
                let record: StateCapture = self.room_capture(0x8f, *address);
                record.raw_word(0).to_ne_bytes()
            }
            ScrollCursor::Prepared { state, index } => {
                let world = self.session().world.definition().expect("prepared scrolls");
                match &world.states[state].scrolls {
                    super::world::ScrollDefinition::Cells(bytes) => {
                        [bytes[*index], bytes[*index + 1]]
                    }
                    _ => panic!("record cursor for generated scrolls"),
                }
            }
        }
    }
    fn object_exists(&mut self, cursor: &ObjectCursor) -> bool {
        let address = match cursor {
            ObjectCursor::LegacyInitial(address) => *address,
            ObjectCursor::LegacyDoor(address) => *address as u16,
            ObjectCursor::Prepared { state, index } => {
                let world = self.session().world.definition().expect("prepared objects");
                return *index < world.states[state].objects.len();
            }
        };
        let record: StateCapture = self.room_capture(0x8f, address);
        record.raw_word(0) != 0
    }
    fn selected_background(
        &mut self,
        state: &Identity<StateId>,
        mode: LoadMode,
    ) -> Option<BackgroundCursor> {
        match state {
            Identity::Known(state) => {
                let world = self
                    .session()
                    .world
                    .definition()
                    .expect("prepared background");
                (!world.states[state].background.is_empty()).then_some(BackgroundCursor::Prepared(
                    BackgroundPosition {
                        state: *state,
                        index: 0,
                        part: BackgroundPart::Opcode,
                    },
                ))
            }
            Identity::LegacyUnresolved(address) => {
                /// The room state's library background pointer ($8F state header +16h).
                const BACKGROUND_POINTER: usize = 0x16;
                let record: StateCapture = self.background_capture(*address, mode);
                let cursor = record.background_word(BACKGROUND_POINTER);
                (cursor & 0x8000 != 0).then_some(BackgroundCursor::Legacy(cursor))
            }
        }
    }
    fn capture_background(
        &mut self,
        cursor: &BackgroundCursor,
        mode: LoadMode,
    ) -> BackgroundCapture {
        match cursor {
            BackgroundCursor::Legacy(address) => self.background_capture(*address, mode),
            BackgroundCursor::Prepared(position) => self.prepared(*position),
        }
    }
    fn live_asset(&self, slot: AssetSlot, read: AssetRead) -> AssetSource {
        AssetSource::Bound(
            self.session()
                .world
                .asset(slot, self.asset_bytes(slot, read)),
        )
    }
    fn capture_closing_door(&mut self, exit: &Identity<ExitId>) -> ClosingDoorCapture {
        match exit {
            Identity::Known(position) => self.prepared(*position),
            Identity::LegacyUnresolved(key) => self.closing_capture(0x83, *key),
        }
    }
    fn capture_closing_header(&mut self, direction: u16) -> ClosingHeaderCapture {
        if self.session().world.is_canonical_legacy() {
            self.closing_capture(0x8f, direction.wrapping_mul(2).wrapping_add(0xe68a))
        } else {
            self.prepared(ClosingHeaderPosition { direction })
        }
    }
    /// The prepared world's record at `position`.
    fn prepared<P>(&self, position: P) -> Capture<P> {
        Capture::Prepared {
            world: self.session().world.clone(),
            position,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SetupKind {
    Door,
    Room,
}
pub use super::content_background::SpawnObject;

impl<P> Capture<P> {
    pub fn raw_byte(&self, offset: usize) -> u8 {
        match self {
            Self::Legacy { image, base, .. } => image.read(
                base.checked_add(offset)
                    .expect("ROM record offset overflow"),
            ),
            Self::Prepared { .. } => panic!("raw read of prepared content"),
        }
    }
    pub fn raw_bytes<const N: usize>(&self, offset: usize) -> [u8; N] {
        core::array::from_fn(|i| self.raw_byte(offset + i))
    }
    pub fn raw_word(&self, offset: usize) -> u16 {
        let bytes = self.raw_bytes(offset);
        match self {
            Self::Legacy {
                access: RecordAccess::LittleEndian,
                ..
            } => u16::from_le_bytes(bytes),
            _ => u16::from_ne_bytes(bytes),
        }
    }
    pub fn world(&self) -> &WorldBinding {
        match self {
            Self::Legacy { world, .. } | Self::Prepared { world, .. } => world,
        }
    }
}
impl Capture<StartId> {
    pub fn room(&self) -> Identity<RoomId> {
        match self {
            Self::Legacy { world, .. } => world.room_identity(self.raw_word(0)),
            Self::Prepared { world, position } => {
                Identity::Known(world.definition().unwrap().starts[position].room)
            }
        }
    }
    pub fn exit(&self) -> Identity<ExitId> {
        match self {
            Self::Legacy { world, .. } => world.exit_identity(self.raw_word(2)),
            Self::Prepared { world, position } => {
                Identity::Known(world.definition().unwrap().starts[position].exit)
            }
        }
    }
    pub fn scalar(&self, field: StationScalar) -> u16 {
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
    pub fn scalar(&self, field: HeaderScalar) -> u16 {
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
    pub fn doors(&self) -> DoorList {
        match self {
            Self::Legacy { world, .. } => world.door_list(self.raw_word(9)),
            Self::Prepared { world, position } => {
                DoorList::Prepared(world.definition().unwrap().rooms[position].doors)
            }
        }
    }
}
impl Capture<SelectorPosition> {
    pub fn condition_id(&self) -> u8 {
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
    pub fn target(&self) -> Identity<StateId> {
        match self {
            Self::Legacy { world, .. } => world.state_identity(self.raw_word(1)),
            Self::Prepared { world, position } => Identity::Known(
                world.definition().unwrap().rooms[&position.room].selectors[position.index].state,
            ),
        }
    }
}
impl Capture<ExitId> {
    pub fn room(&self) -> Identity<RoomId> {
        match self {
            Self::Legacy { world, .. } => world.room_identity(self.raw_word(0)),
            Self::Prepared { world, position } => {
                Identity::Known(world.definition().unwrap().exits[position].destination)
            }
        }
    }
    pub fn scalar(&self, field: DoorScalar) -> u16 {
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
    pub fn setup(&self) -> Option<SetupAction> {
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
    pub fn closing_position(&self, axis: Axis) -> u8 {
        self.scalar(match axis {
            Axis::X => DoorScalar::CloseX,
            Axis::Y => DoorScalar::CloseY,
        }) as u8
    }
    pub fn closing_coordinates_native(&self) -> u16 {
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
    pub fn from_legacy(address: u32) -> Self {
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
    pub fn for_room(room: &Identity<RoomId>, world: &WorldBinding) -> Self {
        match room {
            Identity::Known(room) => Self::Prepared(SelectorPosition {
                room: *room,
                index: 0,
            }),
            Identity::LegacyUnresolved(_) => Self::Legacy(world.room_key(room).wrapping_add(11)),
        }
    }
    pub fn inline_state(&self, world: &WorldBinding) -> Identity<StateId> {
        match self {
            Self::Legacy(cursor) => world.state_identity(*cursor),
            Self::Prepared(position) => Identity::Known(
                world.definition().unwrap().rooms[&position.room].selectors[position.index].state,
            ),
        }
    }
    pub fn next(self, layout: SelectorLayout, world: &WorldBinding) -> Option<Self> {
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
    pub fn next(self) -> Self {
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
    pub fn next(self, mode: LoadMode) -> Self {
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
