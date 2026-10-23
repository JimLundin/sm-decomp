//! Standalone compilable declaration contract; no gameplay bodies.
#![forbid(unsafe_code)]
use crate::game::{external_inputs::ExternalInputs, session::Session};
use core::num::NonZeroU8;

#[derive(Clone, Copy)]
pub struct ProgramCursor(pub u16);
#[derive(Clone, Copy)]
pub struct DrawProgramId(pub u16);
#[derive(Clone, Copy)]
pub struct DrawCursor(pub u16);
#[derive(Clone, Copy)]
pub struct ContentAddress {
    pub bank: u8,
    pub offset: u16,
}
#[derive(Clone, Copy)]
pub enum Pass {
    Instructions,
    DisplayMessage,
    Invalid(u8),
}
pub enum RunResult {
    Complete,
    Yield(NonZeroU8),
}
// Pass is caller-local and NEVER encoded during ordinary Host effects. Runtime
// writes continuation1 ONLY when display actually returns a yield. In particular
// nested callbacks during initial display see the previous contextual byte.
pub enum CallerContext {
    Gameplay,
    DoorTransition,
}
pub enum SpawnSource {
    Collision {
        header: u16,
    },
    Hardcoded {
        x: u8,
        y: u8,
        header: u16,
    },
    Room {
        entry: ProgramCursor,
    },
    Prepared {
        header: u16,
        block: u16,
        before: u16,
        argument: u16,
    },
}
pub enum Contact<'a> {
    Absent,
    Local(&'a mut i32),
    Imported(ContactIdentity),
}
pub struct ContactIdentity(pub(crate) usize);
pub enum Axis {
    X,
    Y,
}
pub enum ByteOrder {
    Little,
    Native,
}
pub enum StoreOrder {
    PrefixLittle,
    SliceLittle,
    SliceNative,
}
// Used ONLY outside normal0..39 slot accesses, never by the ordinary rule path.
pub enum SlotMember {
    Header,
    Block,
    PreAction,
    Program,
    Counter,
    RoomArgument,
    Work0,
    InstructionDelay,
    DrawProgram,
    ReturnProgram,
    Work1,
}
pub trait ImportedAliases {
    fn mechanism_word(&self, member: SlotMember, signed_slot: isize) -> u16;
    fn mechanism_byte(&self, member: SlotMember, signed_slot: isize, lane: usize) -> u8;
    fn set_mechanism_word(
        &mut self,
        member: SlotMember,
        signed_slot: isize,
        value: u16,
        order: StoreOrder,
    );
    fn set_mechanism_byte(
        &mut self,
        member: SlotMember,
        signed_slot: isize,
        lane: usize,
        value: u8,
    );
    // Signed slot -1 is NOT represented by wrapping to raw handle FFFE.
    fn projectile_kind(&self, signed_slot: isize, order: ByteOrder) -> u16;
    fn saved_graphics_word(&self, index: usize) -> u16;
    fn set_saved_graphics_word(&mut self, index: usize, value: u16, order: StoreOrder);
    fn terrain_word(&self, byte_block: usize, order: ByteOrder) -> u16;
    fn set_terrain_word(&mut self, byte_block: usize, value: u16, order: ByteOrder);
    fn terrain_high(&self, unrounded_byte_block: u16) -> u8;
    fn set_terrain_high(&mut self, unrounded_byte_block: u16, value: u8);
    fn behavior_byte(&self, byte_index: usize) -> u8;
    fn set_behavior_byte(&mut self, byte_index: usize, value: u8);
    fn set_behavior_pair(&mut self, byte_index: usize, value: u16);
    fn contact_displacement(&self, token: &ContactIdentity) -> i32;
    fn set_contact_displacement(&mut self, token: &ContactIdentity, value: i32);
    fn scroll_pair(&mut self, byte_index: usize, value: u16);
    fn set_scroll(&mut self, byte_index: usize, value: u8);
    fn signal_turret(&mut self, association: u16);
}
// No arbitrary address read/write methods. Checked range/endian/prefix handling
// lives in the single runtime projection. Normal12800 cells/1024 tile definitions
// are directly owned; exceptional selectors can alias another owned region.
pub trait InstructionRecord {
    fn byte(&self, byte_offset: isize) -> u8;
    fn word(&self, byte_offset: isize) -> u16;
}
pub trait Content {
    type Record: InstructionRecord;
    type TerrainProgram;
    fn loaded_record(&mut self, address: ContentAddress) -> Self::Record;
    fn native_record(&mut self, address: ContentAddress) -> Self::Record;
    fn definition_word(&mut self, address: ContentAddress, byte_offset: usize) -> u16;
    fn table_word(&self, table: ContentTable, index: isize) -> u16;
    fn capture_terrain_program(&mut self, id: DrawProgramId) -> Self::TerrainProgram;
    fn terrain_program_word(&self, program: &Self::TerrainProgram, byte_offset: usize) -> u16;
    fn drawing_word(&mut self, cursor: DrawCursor) -> u16;
    fn drawing_byte(&mut self, cursor: DrawCursor) -> u8;
}
pub enum ContentTable {
    Gate { up: bool, byte_offset: usize },
    MetroidWatcher,
    ItemGraphicDestination,
    ItemTileOffset,
    ItemTileNumber,
    ItemReveal,
}
#[derive(Clone, Copy)]
pub struct TilemapDestination(pub u16);
pub enum DrawLayout {
    VerticalPair,
    HorizontalPair,
    HorizontalSplit,
}
// Identities survive aggregate drop/reimport; they resolve CURRENT buffer at NMI,
// never retain a borrow/Arc snapshot of a discarded Session allocation.
pub struct DrawTarget(pub(crate) usize);
pub struct DrawTargets {
    pub first: DrawTarget,
    pub second: DrawTarget,
}
pub struct TileDefinitionCorner {
    pub block: u16,
    pub corner: u16,
}
pub enum GraphicsTransferKind {
    Item { destination: TilemapDestination },
    CopyScript,
}
pub trait Graphics {
    fn queue_graphics_transfer<R: InstructionRecord>(
        &mut self,
        record: &R,
        kind: GraphicsTransferKind,
    );
    // Owns tail reads, destination write BEFORE live draw_cursor read, layout's
    // ordered packed stores/rereads and tail update. Never precompute source.
    fn queue_draw_transfers(
        &mut self,
        destination: TilemapDestination,
        count: u16,
        layout: DrawLayout,
    ) -> Option<DrawTargets>;
    // Source tile is read INSIDE this operation immediately before its write.
    fn write_draw_tile(&mut self, target: &DrawTarget, offset: u16, source: TileDefinitionCorner);
    fn set_item_tile_definition(&mut self, byte_offset: usize, value: u16);
    fn reveal_block(&mut self, block: u16, x: u16, y: u16);
}
pub(crate) use crate::game::item_pickups::VisualUpdate;
pub enum SoundChannel {
    One,
    Two,
    Three,
}
pub enum Diagnostic {
    MissingDisplacement,
    MissingScroll,
    InvalidRamDraw(u16),
    UnsupportedSetup,
    Unreachable,
}
pub enum SuitPickup {
    Varia,
    Gravity,
    Unknown(u32),
}
pub trait Effects {
    fn sound(&mut self, channel: SoundChannel, maximum: u8, cue: u16);
    fn music(&mut self, track: u16);
    fn clear_music_queue(&mut self);
    fn delayed_room_music(&mut self, frames: u16);
    fn display_message(&mut self, request: u16) -> u8;
    fn poll_message(&mut self, request: u16) -> i32;
    fn player_control(&mut self, operation: u16);
    fn face_forward(&mut self);
    fn suit_pickup(&mut self, pickup: SuitPickup);
    fn pickup_visual(&mut self, visual: VisualUpdate);
    fn save(&mut self, slot: u16);
    fn enemy_projectile(&mut self, definition: u16, parameter: u16);
    fn wake_projectile_for_block(&mut self, block: u16);
    fn event(&mut self, event: u16) -> bool;
    fn mark_event(&mut self, event: u16);
    fn boss(&mut self, mask: u16) -> u16;
    fn door_opened(&mut self, door: u16) -> bool;
    fn mark_door_opened(&mut self, door: u16);
    fn item_collected(&mut self, item: u16) -> bool;
    fn mark_item_collected(&mut self, item: u16);
    fn room_object_used(&mut self, object: u16) -> bool;
    fn mark_room_object_used(&mut self, object: u16);
    fn open_door_and_read_adjacent_flags(&mut self, door: u16) -> u16;
    fn map_used(&self) -> bool;
    fn mark_map_used(&mut self);
    fn set_has_area_map(&mut self, value: u16);
    fn mark_save_station(&mut self, station: u16);
    fn selected_save_slot(&self) -> u16;
    fn statue_signal(&mut self);
    fn statue_wait_actor(&mut self, actor: u16, waiting: bool);
    fn diagnostic(&mut self, diagnostic: Diagnostic) -> u8;
    fn probe(&self, name: &'static str);
}
pub trait Host: ImportedAliases + Content + Graphics + Effects {
    fn session(&self) -> &Session;
    fn session_mut(&mut self) -> &mut Session;
    fn inputs(&self) -> &ExternalInputs;
}
