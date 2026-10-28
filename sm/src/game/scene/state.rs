//! Owned display construction and staging; the semantic Scene is committed later.
#![forbid(unsafe_code)]
use std::sync::Arc;

pub use super::atmosphere::{AtmosphericEffects, AtmosphericSlot};
use super::hdma_sources::{HdmaCatalog, HdmaLiveDescription, SourceBinding};
use super::records::ProducedObject;
use super::records::{AssetRef, Revision, SceneAsset, SceneMedia};
pub use crate::game::visual_state::{
    AnimatedTiles, AnimationSlot, PaletteEffects, PaletteSlot, PlayerPalette, Visuals,
};

/// Supplied construction data for the existing crate-private lighting owner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FirefleaDescription {
    pub timer: u16,
    pub flash_index: u16,
    pub compatibility_word: u16,
    pub darkness: u16,
    pub compatibility_limit: u16,
    pub compatibility_shade: u16,
}

/// Construct the existing pure owner from every supplied field, without defaults.
pub fn supplied_visuals(
    fireflea: FirefleaDescription,
    colors: [u16; 256],
    targets: [u16; 256],
    palette_effects: PaletteEffects,
    animated_tiles: AnimatedTiles,
) -> Visuals {
    Visuals {
        fireflea: crate::game::visual_state::FirefleaLighting {
            timer: fireflea.timer,
            flash_index: fireflea.flash_index,
            compatibility_word: fireflea.compatibility_word,
            darkness: fireflea.darkness,
            compatibility_limit: fireflea.compatibility_limit,
            compatibility_shade: fireflea.compatibility_shade,
        },
        colors,
        targets,
        palette_effects,
        animated_tiles,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WordOrder {
    Little,
    Big,
}
impl WordOrder {
    pub(crate) fn encode(self, value: u16) -> [u8; 2] {
        match self {
            Self::Little => value.to_le_bytes(),
            Self::Big => value.to_be_bytes(),
        }
    }
    pub(crate) fn decode(self, bytes: [u8; 2]) -> u16 {
        match self {
            Self::Little => u16::from_le_bytes(bytes),
            Self::Big => u16::from_be_bytes(bytes),
        }
    }
}

pub use super::error::DisplayError;

#[derive(Clone, Debug)]
pub struct SourceSpan {
    pub asset: AssetRef,
    pub bytes: Arc<[u8]>,
    pub offset: usize,
}
impl SourceSpan {
    pub(crate) fn validate(&self, revision: Revision) -> Result<(), DisplayError> {
        if self.asset.id.0 == 0 {
            return Err(DisplayError::Identity);
        }
        if self.asset.revision != revision {
            return Err(DisplayError::Revision);
        }
        if self.offset > self.bytes.len() {
            return Err(DisplayError::Source);
        }
        Ok(())
    }
    pub(crate) fn byte(&self, displacement: usize) -> Result<u8, DisplayError> {
        self.offset
            .checked_add(displacement)
            .and_then(|at| self.bytes.get(at).copied())
            .ok_or(DisplayError::Source)
    }
    fn validate_media(&self, media: &SceneMedia) -> Result<(), DisplayError> {
        let asset = media
            .asset(self.asset)
            .map_err(|_| DisplayError::Identity)?;
        let words: Vec<u16> = match asset {
            SceneAsset::Atlas(atlas) => atlas.data.to_vec(),
            SceneAsset::Map(map) => map.cells.iter().map(|cell| cell.planar_word()).collect(),
            SceneAsset::Palette(palette) => palette.colors.to_vec(),
            _ => return Err(DisplayError::Source),
        };
        if self.bytes.len() != words.len() * 2 {
            return Err(DisplayError::Source);
        }
        for (index, word) in words.into_iter().enumerate() {
            if self.bytes[2 * index..2 * index + 2] != word.to_le_bytes() {
                return Err(DisplayError::Source);
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub enum TransferSource {
    Asset(SourceSpan),
    Hud {
        offset: usize,
    },
    Objects {
        offset: usize,
    },
    ActorTilemap {
        offset: usize,
    },
    Scroll {
        slot: usize,
        half: usize,
        wrapped: bool,
        offset: usize,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ByteLanes {
    Both,
    Low,
    High,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TileDestination {
    pub word: u16,
    pub step: u16,
    pub lanes: ByteLanes,
    pub remap: u8,
    pub increment_on_high: bool,
}
#[derive(Clone, Debug)]
pub struct TileTransfer {
    pub source: TransferSource,
    pub count: u16,
    pub destination: TileDestination,
}
impl TileTransfer {
    pub(crate) fn byte_count(&self) -> usize {
        if self.count == 0 {
            65_536
        } else {
            usize::from(self.count)
        }
    }
    pub(crate) fn validate(&self, revision: Revision) -> Result<(), DisplayError> {
        if self.destination.remap > 3 {
            return Err(DisplayError::Geometry);
        }
        match &self.source {
            TransferSource::Asset(source) => source.validate(revision),
            _ => Ok(()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ObjectStaging {
    /// Exactly the 512 object bytes and 32 high-table bytes; not public Scene data.
    pub bytes: [u8; 544],
    pub cursor: u16,
    pub native_order: WordOrder,
    /// Actual successful authored publications, separate from byte/order identities.
    pub produced: Vec<ProducedObject>,
}
impl ObjectStaging {
    pub(crate) fn byte(&self, offset: usize) -> Result<u8, DisplayError> {
        if let Some(value) = self.bytes.get(offset) {
            return Ok(*value);
        }
        // The cursor immediately follows the high table. Preserve this owned alias.
        if let Some(part) = offset.checked_sub(544).filter(|part| *part < 2) {
            return Ok(self.native_order.encode(self.cursor)[part]);
        }
        Err(DisplayError::Alias { offset })
    }
    pub(crate) fn write_byte(&mut self, offset: usize, value: u8) -> Result<(), DisplayError> {
        if let Some(at) = self.bytes.get_mut(offset) {
            if offset < 512 {
                self.produced
                    .retain(|record| usize::from(record.object.order) != offset / 4);
            }
            *at = value;
            return Ok(());
        }
        if let Some(part) = offset.checked_sub(544).filter(|part| *part < 2) {
            let mut encoded = self.native_order.encode(self.cursor);
            encoded[part] = value;
            self.cursor = self.native_order.decode(encoded);
            return Ok(());
        }
        Err(DisplayError::Alias { offset })
    }
    pub(crate) fn native_word(&self, offset: usize) -> Result<u16, DisplayError> {
        let low = self.byte(offset)?;
        let high = self.byte(offset.checked_add(1).ok_or(DisplayError::Geometry)?)?;
        Ok(self.native_order.decode([low, high]))
    }
    /// Slice-native stores preflight both bytes before either byte changes.
    pub(crate) fn write_native_slice(
        &mut self,
        offset: usize,
        value: u16,
    ) -> Result<(), DisplayError> {
        self.byte(offset)?;
        let second = offset.checked_add(1).ok_or(DisplayError::Geometry)?;
        self.byte(second)?;
        let encoded = self.native_order.encode(value);
        self.write_byte(offset, encoded[0])?;
        self.write_byte(second, encoded[1])
    }
    pub(crate) fn clear_high_table(&mut self) {
        // All 32 high bytes are owned; this host cannot fail at these offsets.
        super::objects::clear_high_table(self).expect("owned object high table");
    }
    pub(crate) fn finish_frame(&mut self) -> Result<(), DisplayError> {
        super::objects::finish_frame(self)
    }
    pub(crate) fn draw(
        &mut self,
        map: &mut impl SpriteGeometry<Error = DisplayError>,
        position: (u16, u16),
        clip: SpriteClip,
        tile_word: impl FnMut(u16) -> Result<u16, DisplayError>,
    ) -> Result<(), DisplayError> {
        super::objects::draw(self, map, position, clip, tile_word)
    }
}

pub use super::objects::Clip as SpriteClip;
pub(crate) use super::objects::Geometry as SpriteGeometry;

impl super::objects::Host for ObjectStaging {
    type Error = DisplayError;
    fn read_native_word(&mut self, offset: usize) -> Result<u16, DisplayError> {
        self.native_word(offset)
    }
    fn read_little_word(&mut self, offset: usize) -> Result<u16, DisplayError> {
        let low = self.byte(offset)?;
        let high = self.byte(offset.checked_add(1).ok_or(DisplayError::Geometry)?)?;
        Ok(u16::from_le_bytes([low, high]))
    }
    fn write_byte(&mut self, offset: usize, value: u8) -> Result<(), DisplayError> {
        ObjectStaging::write_byte(self, offset, value)
    }
    fn write_native_slice(&mut self, offset: usize, value: u16) -> Result<(), DisplayError> {
        ObjectStaging::write_native_slice(self, offset, value)
    }
    fn write_native_bytes(&mut self, offset: usize, value: u16) -> Result<(), DisplayError> {
        let bytes = self.native_order.encode(value);
        self.write_byte(offset, bytes[0])?;
        self.write_byte(
            offset.checked_add(1).ok_or(DisplayError::Geometry)?,
            bytes[1],
        )
    }
    fn started(&mut self, _clip: SpriteClip) -> Result<(), DisplayError> {
        Ok(())
    }
}
#[derive(Clone, Debug)]
pub struct HudState {
    pub tiles: [u16; 96],
    pub health: u16,
    pub maximum_health: u16,
    pub previous_health: u16,
    pub ammunition: [u16; 3],
    pub maximum_ammunition: [u16; 3],
    pub previous_ammunition: [u16; 3],
    pub selection: u16,
    pub previous_selection: u16,
    pub auto_cancel: u16,
    pub reserve_mode: u16,
    pub reserve_health: u16,
    pub palette: u16,
    pub media: AssetRef,
}
impl HudState {
    pub(crate) fn stamp(
        &mut self,
        base: usize,
        columns: usize,
        count: usize,
        mut tile: impl FnMut(usize) -> Result<u16, DisplayError>,
    ) -> Result<(), DisplayError> {
        if columns == 0 {
            return Err(DisplayError::Geometry);
        }
        for index in 0..count {
            let value = tile(index)?;
            let at = base + 32 * (index / columns) + index % columns;
            *self
                .tiles
                .get_mut(at)
                .ok_or(DisplayError::Alias { offset: at })? = value;
        }
        Ok(())
    }
    pub(crate) fn highlight(&mut self, selection: u16, palette: u16) -> Result<(), DisplayError> {
        self.palette = palette;
        let item = (selection as i16).wrapping_sub(1);
        if item < 0 {
            return Ok(());
        }
        let bases = [0x14usize, 0x1c, 0x22, 0x28, 0x2e];
        let base = *bases.get(item as usize).ok_or(DisplayError::Geometry)? >> 1;
        let offsets: &[usize] = if item == 0 {
            &[0, 1, 32, 33, 2, 34]
        } else {
            &[0, 1, 32, 33]
        };
        for &offset in offsets {
            let at = base + offset;
            let tile = self
                .tiles
                .get_mut(at)
                .ok_or(DisplayError::Alias { offset: at })?;
            if *tile != 11279 {
                *tile = self.palette | *tile & 0xe3ff;
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct NmiState {
    pub pending: u8,
    pub frame_byte: u8,
    pub frame_word: u16,
    pub including_lag: u16,
    pub missed: u8,
    pub max_missed: u8,
    pub acknowledgment: u8,
}
impl NmiState {
    pub(crate) fn wait(&mut self, update: bool) -> u8 {
        if self.acknowledgment != 0 {
            self.acknowledgment = 0;
            return 0;
        }
        if update {
            self.pending = 1;
        }
        self.acknowledgment = 1;
        1
    }
}

#[derive(Clone, Debug)]
pub struct SamusHalf {
    pub enabled: u8,
    pub descriptor: AssetRef,
    pub first: TileTransfer,
    pub second: TileTransfer,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollLayout {
    Column,
    Row,
}
#[derive(Clone, Debug)]
pub struct ScrollTransfer {
    pub enabled: u8,
    pub layout: ScrollLayout,
    pub first: [TileTransfer; 2],
    pub wrapped: [TileTransfer; 2],
    pub wrapped_count: u16,
    pub first_bytes: [Vec<u8>; 2],
    pub wrapped_bytes: [Vec<u8>; 2],
}
#[derive(Clone, Debug)]
pub struct QueuedUpload {
    pub cursor: u16,
    pub transfer: TileTransfer,
}
#[derive(Clone, Debug)]
pub enum Mode7Target {
    Tiles(TileDestination),
    Palette { first_color: u8 },
}
#[derive(Clone, Debug)]
pub struct Mode7Upload {
    pub cursor: u16,
    pub source: TransferSource,
    pub count: u16,
    pub target: Mode7Target,
    pub transfer_mode: u8,
}
#[derive(Clone, Debug)]
pub struct Readback {
    pub cursor: u16,
    pub destination: AssetRef,
    pub first_word: u16,
    pub count: u16,
}
#[derive(Clone, Debug)]
pub struct ActorDrawQueue {
    pub size: u16,
    pub cursor: u16,
    pub actors: Vec<u16>,
}
#[derive(Clone, Debug)]
pub struct ActorTileStaging {
    pub pending: u16,
    pub size: u16,
    pub source: SourceSpan,
    pub destination: u16,
    pub transfer_source: u16,
    pub transfer_destination: u16,
    pub tilemap_bytes: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct BackgroundState {
    pub tilemap: AssetRef,
    pub atlas: AssetRef,
    pub palette: AssetRef,
    pub camera: [u16; 2],
    pub scroll: [u16; 2],
    pub screen_layout: u8,
    pub character_layout: u8,
}
#[derive(Clone, Copy, Debug)]
pub struct DisplayControls {
    pub brightness: u8,
    pub object_layout: u8,
    pub mode: u8,
    pub fake_mode: u8,
    pub mosaic: u8,
    pub screens: [u8; 4],
    pub previous_background3_map: u8,
    pub characters: [u8; 2],
    pub affine_selection: u8,
    pub window_selection: [u8; 3],
    pub window_positions: [u8; 4],
    pub window_logic: [u8; 2],
    pub main_layers: u8,
    pub previous_main_layers: u8,
    pub sub_layers: u8,
    pub main_window: u8,
    pub sub_window: u8,
    pub next_color_selection: u8,
    pub color_selection: u8,
    pub previous_color_selection: u8,
    pub next_color_math: u8,
    pub color_math: u8,
    pub previous_color_math: u8,
    pub fixed_color: [u8; 3],
    pub geometry: u8,
    pub scroll_words: [u16; 8],
    pub affine_words: [u16; 6],
    pub nmi_enable: u8,
    pub hdma_enable: u8,
    pub committed_fake_mode: u8,
}
#[derive(Clone, Copy, Debug)]
pub struct HdmaBinding {
    pub enable: u16,
    pub bank_slot: u16,
    pub table: u16,
}
#[derive(Clone, Debug)]
pub struct HdmaObject {
    pub instruction: SourceBinding,
    pub instruction_cursor: u16,
    pub instruction_timer: u16,
    pub pre_instruction: u16,
    pub pre_instruction_bank: u16,
    pub timer: u16,
    pub variables: [u16; 6],
}
#[derive(Clone, Debug)]
pub struct HdmaChannel {
    pub source: SourceBinding,
    pub table: SourceBinding,
    pub active: bool,
    pub terminated: bool,
    pub do_transfer: bool,
    pub repeat_count: u8,
    pub source_address: u16,
    pub table_cursor: u16,
    pub indirect_cursor: u16,
    pub mode: u8,
    pub from_destination: bool,
    pub indirect: bool,
    pub fixed: bool,
    pub decrement: bool,
    pub source_bank: u8,
    pub indirect_bank: u8,
    pub destination: u8,
    pub transfer_cursor: u8,
    pub unused_bit: bool,
    pub unused_byte: u8,
    pub dma_active: bool,
}
#[derive(Clone, Debug)]
pub struct HdmaState {
    pub bindings: [HdmaBinding; 6],
    pub objects: [HdmaObject; 6],
    pub channels: [HdmaChannel; 8],
    pub selected: u16,
    pub flags: u16,
    pub hdma_timer: u16,
    pub dma_timer: u32,
    pub dma_busy: bool,
}
#[derive(Clone, Copy, Debug)]
pub struct InterruptState {
    pub current: u16,
    pub pending: u16,
    /// Effective delivered timers, independent of staged control bytes.
    pub vertical: u16,
    pub horizontal: u16,
    pub previous_cre: u16,
    pub current_cre: u16,
    pub transition_flag: u16,
    pub transition_continuation: u16,
}
impl InterruptState {
    pub(crate) fn consume_pending(&mut self, fallback: u16) -> u16 {
        let next = self.pending;
        self.pending = 0;
        if next == 0 { fallback } else { next }
    }
    pub(crate) fn room_layers(&self) -> u8 {
        if (self.previous_cre | self.current_cre) & 1 != 0 {
            16
        } else {
            17
        }
    }
}
/// Required effective delivery history, supplied without opening-time inference.
/// Refresh requests, staged controls and committed ticks have separate lifetimes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DisplayDeliveryDescription {
    pub horizontal_position: u16,
    pub vertical_position: u16,
    pub beam_frames: u32,
    pub horizontal_irq_enabled: bool,
    pub vertical_irq_enabled: bool,
    pub nmi_enabled: bool,
    pub nmi_wanted: bool,
    pub irq_wanted: bool,
    pub in_nmi: bool,
    pub in_irq: bool,
    pub in_vblank: bool,
    pub automatic_input_read: bool,
    pub automatic_input_timer: u16,
    pub counter_latch_gate: bool,
    pub raster_suppressed: bool,
}
/// Effective values consumed by IRQ output and vblank, not staged controls.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EffectiveDisplayDescription {
    pub background3_map_words: u16,
    pub background3_map_wider: bool,
    pub background3_map_higher: bool,
    pub color_clip: u8,
    pub color_prevent_math: u8,
    pub color_add_subscreen: bool,
    pub direct_color: bool,
    pub color_math_enabled: [bool; 6],
    pub color_subtract: bool,
    pub color_half: bool,
    /// Full effective routing byte, including the native cached unused bits.
    pub main_layers: u8,
    pub forced_blank: bool,
}
#[derive(Clone, Copy, Debug)]
pub struct DisplayLatches {
    pub tile_pointer: u16,
    pub tile_increment: u16,
    pub tile_remap: u8,
    pub tile_read_buffer: u16,
    pub tile_increment_on_high: bool,
    pub tile_padding: [u8; 2],
    pub palette_pointer: u8,
    pub palette_second: bool,
    pub palette_buffer: u8,
    pub object_written_address: u16,
    pub object_written_high_table: bool,
    pub object_address: u16,
    pub object_high_table: bool,
    pub object_second: bool,
    pub object_buffer: u8,
    pub scroll_previous: u8,
    pub scroll_previous_second: u8,
    pub affine_previous: u8,
    pub latched_horizontal: u16,
    pub latched_vertical: u16,
    pub horizontal_second: bool,
    pub vertical_second: bool,
    pub counter_latched: bool,
    pub open_bus: u8,
    pub ppu_open_bus: [u8; 2],
}
#[derive(Clone, Copy, Debug)]
pub struct RasterState {
    pub range_overflow: bool,
    pub time_overflow: bool,
    pub has_objects: bool,
    pub even_frame: bool,
    pub interlaced: bool,
    pub frame_interlaced: bool,
    pub overscan: bool,
    pub frame_overscan: bool,
    pub mosaic_origin: u8,
    pub mosaic_phase: u8,
    pub visible: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisplayPhase {
    Committed,
    Producing,
    Refreshed,
    Settling,
    Failed,
}

#[derive(Clone, Debug)]
pub struct DisplayDescription {
    pub media_revision: Revision,
    pub phase: DisplayPhase,
    pub objects: ObjectStaging,
    pub hud: HudState,
    /// Move these existing owners at the handoff; do not retain active Session copies.
    pub visual: Visuals,
    pub player_palette: PlayerPalette,
    pub atmospheric: AtmosphericEffects,
    /// Required immutable content and persistent IDs supplied by the G start.
    pub atmosphere_catalog: Arc<super::atmosphere::AtmosphereCatalog>,
    /// Exactly one mutable lifecycle counter for all display producer cohorts.
    pub producer_ids: super::records::DisplayProducerIds,
    pub atmospheric_producers: super::atmosphere::AtmosphericProducers,
    pub nmi: NmiState,
    pub samus: [SamusHalf; 2],
    pub backgrounds: [BackgroundState; 3],
    pub scrolling: [ScrollTransfer; 4],
    pub uploads: Vec<QueuedUpload>,
    pub upload_tail: u16,
    pub mode7_uploads: Vec<Mode7Upload>,
    pub mode7_tail: u16,
    pub readbacks: Vec<Readback>,
    pub readback_tail: u16,
    pub actor_queues: [ActorDrawQueue; 8],
    pub actor_tiles: ActorTileStaging,
    pub controls: DisplayControls,
    pub hdma: HdmaState,
    /// The current descriptor/scanline/scratch owners are adopted with this state.
    pub hdma_live: HdmaLiveDescription,
    pub interrupt: InterruptState,
    pub delivery: DisplayDeliveryDescription,
    pub effective: EffectiveDisplayDescription,
    pub latches: DisplayLatches,
    pub raster: RasterState,
    /// Actual resolved animated sources; no source is cleared before successful upload.
    pub animated_sources: [SourceSpan; 6],
}
/// The live owner remains private; the public description is consumed at opening.
pub(crate) type DisplayState = DisplayDescription;
impl DisplayDescription {
    pub(crate) fn revision(&self) -> Revision {
        self.media_revision
    }
    pub(crate) fn validate(&self) -> Result<(), DisplayError> {
        if self.phase != DisplayPhase::Committed {
            return Err(DisplayError::Phase);
        }
        if self.effective.color_clip > 3 || self.effective.color_prevent_math > 3 {
            return Err(DisplayError::Geometry);
        }
        // This is explicit sealed-start admission. Runtime aliases stay with compatibility.
        if self.objects.cursor > 512 {
            return Err(DisplayError::Alias {
                offset: usize::from(self.objects.cursor),
            });
        }
        self.atmospheric_producers
            .validate(&self.atmospheric, &self.producer_ids)?;
        if self.atmosphere_catalog.revision != self.media_revision {
            return Err(DisplayError::Revision);
        }
        let identity = |asset: AssetRef| {
            if asset.id.0 == 0 {
                Err(DisplayError::Identity)
            } else if asset.revision != self.media_revision {
                Err(DisplayError::Revision)
            } else {
                Ok(())
            }
        };
        identity(self.hud.media)?;
        for background in &self.backgrounds {
            identity(background.tilemap)?;
            identity(background.atlas)?;
            identity(background.palette)?;
        }
        for half in &self.samus {
            identity(half.descriptor)?;
            half.first.validate(self.media_revision)?;
            half.second.validate(self.media_revision)?;
        }
        for source in &self.animated_sources {
            source.validate(self.media_revision)?;
        }
        for scroll in &self.scrolling {
            for transfer in scroll.first.iter().chain(scroll.wrapped.iter()) {
                transfer.validate(self.media_revision)?;
            }
        }
        if self.scrolling[0].layout != ScrollLayout::Column
            || self.scrolling[1].layout != ScrollLayout::Column
            || self.scrolling[2].layout != ScrollLayout::Row
            || self.scrolling[3].layout != ScrollLayout::Row
        {
            return Err(DisplayError::Queue);
        }
        for upload in &self.uploads {
            upload.transfer.validate(self.media_revision)?;
        }
        for upload in &self.mode7_uploads {
            if matches!(&upload.target, Mode7Target::Tiles(target) if target.remap > 3) {
                return Err(DisplayError::Geometry);
            }
            if let TransferSource::Asset(source) = &upload.source {
                source.validate(self.media_revision)?;
            }
            if upload.transfer_mode != 0 {
                return Err(DisplayError::Incomplete);
            }
        }
        for readback in &self.readbacks {
            identity(readback.destination)?;
        }
        for queue in &self.actor_queues {
            if queue.size & 1 != 0 || usize::from(queue.size / 2) > queue.actors.len() {
                return Err(DisplayError::Queue);
            }
        }
        self.actor_tiles.source.validate(self.media_revision)?;
        self.hdma_live.validate()?;
        for channel in &self.hdma.channels {
            if channel.mode > 7 {
                return Err(DisplayError::Geometry);
            }
        }
        Ok(())
    }
    pub(crate) fn source_byte(
        &self,
        source: &TransferSource,
        displacement: usize,
    ) -> Result<u8, DisplayError> {
        let add = |offset: usize| offset.checked_add(displacement).ok_or(DisplayError::Source);
        match source {
            TransferSource::Asset(source) => source.byte(displacement),
            TransferSource::Hud { offset } => {
                let at = add(*offset)?;
                let word = self.hud.tiles.get(at >> 1).ok_or(DisplayError::Source)?;
                Ok(word.to_le_bytes()[at & 1])
            }
            TransferSource::Objects { offset } => self.objects.byte(add(*offset)?),
            TransferSource::ActorTilemap { offset } => self
                .actor_tiles
                .tilemap_bytes
                .get(add(*offset)?)
                .copied()
                .ok_or(DisplayError::Source),
            TransferSource::Scroll {
                slot,
                half,
                wrapped,
                offset,
            } => {
                let scroll = self.scrolling.get(*slot).ok_or(DisplayError::Source)?;
                let halves = if *wrapped {
                    &scroll.wrapped_bytes
                } else {
                    &scroll.first_bytes
                };
                halves
                    .get(*half)
                    .and_then(|bytes| bytes.get(add(*offset).ok()?))
                    .copied()
                    .ok_or(DisplayError::Source)
            }
        }
    }
    fn source_available(&self, source: &TransferSource) -> Result<usize, DisplayError> {
        let remaining =
            |length: usize, offset: usize| length.checked_sub(offset).ok_or(DisplayError::Source);
        match source {
            TransferSource::Asset(source) => remaining(source.bytes.len(), source.offset),
            TransferSource::Hud { offset } => remaining(192, *offset),
            TransferSource::Objects { offset } => remaining(546, *offset),
            TransferSource::ActorTilemap { offset } => {
                remaining(self.actor_tiles.tilemap_bytes.len(), *offset)
            }
            TransferSource::Scroll {
                slot,
                half,
                wrapped,
                offset,
            } => {
                let scroll = self.scrolling.get(*slot).ok_or(DisplayError::Source)?;
                let halves = if *wrapped {
                    &scroll.wrapped_bytes
                } else {
                    &scroll.first_bytes
                };
                remaining(
                    halves.get(*half).ok_or(DisplayError::Source)?.len(),
                    *offset,
                )
            }
        }
    }
    pub(crate) fn validate_media(&self, media: &SceneMedia) -> Result<(), DisplayError> {
        self.atmosphere_catalog.validate(media)?;
        for (index, record) in self.objects.produced.iter().enumerate() {
            record.validate(media).map_err(|_| DisplayError::Scene)?;
            self.producer_ids
                .validate_id(record.producer)
                .map_err(|_| DisplayError::Identity)?;
            if self.objects.produced[..index]
                .iter()
                .any(|old| old.object.order == record.object.order)
            {
                return Err(DisplayError::Scene);
            }
        }
        self.validate()?;
        if media.revision() != self.media_revision {
            return Err(DisplayError::Revision);
        }
        let identity = |reference| {
            media
                .asset(reference)
                .map(|_| ())
                .map_err(|_| DisplayError::Identity)
        };
        let transfer = |value: &TileTransfer, active: bool| {
            if let TransferSource::Asset(source) = &value.source {
                source.validate_media(media)?;
            }
            if active && self.source_available(&value.source)? < value.byte_count() {
                return Err(DisplayError::Source);
            }
            Ok(())
        };
        identity(self.hud.media)?;
        for background in &self.backgrounds {
            identity(background.tilemap)?;
            identity(background.atlas)?;
            identity(background.palette)?;
        }
        for half in &self.samus {
            identity(half.descriptor)?;
            transfer(&half.first, half.enabled != 0)?;
            transfer(&half.second, half.enabled != 0 && half.second.count != 0)?;
        }
        for scroll in &self.scrolling {
            for value in &scroll.first {
                transfer(value, scroll.enabled != 0)?;
            }
            for value in &scroll.wrapped {
                transfer(value, scroll.enabled != 0 && scroll.wrapped_count != 0)?;
            }
        }
        for value in &self.uploads {
            transfer(&value.transfer, self.upload_tail != 0)?;
        }
        for value in &self.mode7_uploads {
            if let TransferSource::Asset(source) = &value.source {
                source.validate_media(media)?;
            }
            let count = if value.count == 0 {
                65_536
            } else {
                usize::from(value.count)
            };
            if self.mode7_tail != 0 && self.source_available(&value.source)? < count {
                return Err(DisplayError::Source);
            }
        }
        for read in &self.readbacks {
            identity(read.destination)?;
        }
        self.actor_tiles.source.validate_media(media)?;
        for source in &self.animated_sources {
            source.validate_media(media)?;
        }
        Ok(())
    }
    pub(crate) fn validate_hdma_sources(&self, catalog: &HdmaCatalog) -> Result<(), DisplayError> {
        self.hdma_live.validate()?;
        for (index, channel) in self.hdma.channels.iter().enumerate() {
            SourceBinding::validate_channel(
                &channel.table,
                &channel.source,
                catalog,
                &self.hdma_live,
                &self.hdma,
                index,
            )?;
        }
        for (index, object) in self.hdma.objects.iter().enumerate() {
            object
                .instruction
                .validate_program(catalog, &self.hdma_live, &self.hdma, index)?;
        }
        self.hdma_live.validate_canonical_aliases(&self.hdma)
    }
    pub(crate) fn begin_interval(&mut self, fresh_entry: bool) -> Result<(), DisplayError> {
        if self.phase == DisplayPhase::Failed {
            return Err(DisplayError::Poisoned);
        }
        if self.phase != DisplayPhase::Committed {
            return Err(DisplayError::Phase);
        }
        self.phase = DisplayPhase::Producing;
        if fresh_entry {
            self.objects.produced.clear();
            self.objects.clear_high_table();
            self.objects.cursor = 0;
            self.samus[0].enabled = 0;
            self.samus[1].enabled = 0;
        }
        Ok(())
    }
}
