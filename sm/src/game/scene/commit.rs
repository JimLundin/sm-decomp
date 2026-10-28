//! Original refresh order shared by the native bridge and the owned display.
#![forbid(unsafe_code)]
use std::sync::Arc;

use super::hdma_sources::HdmaCatalog;
use super::records::{
    AtlasBinding, CompiledImage, MapBinding, MediaRef, PaletteIncarnation, Scene, SceneDescription,
    SceneMedia,
};
use super::state::{
    ByteLanes, DisplayError, DisplayPhase, DisplayState, Mode7Target, TileTransfer, TransferSource,
};
use super::{delivery, irq, owned_hdma};

pub(crate) trait RefreshHost {
    fn pending_nmi(&self) -> u8;
    fn palette_and_objects(&mut self) -> Result<(), DisplayError>;
    fn samus(&mut self) -> Result<(), DisplayError>;
    fn animations(&mut self) -> Result<(), DisplayError>;
    fn controls(&mut self) -> Result<(), DisplayError>;
    fn hdma_mask(&self, slot: usize) -> u16;
    fn hdma_bank_slot(&self, slot: usize) -> u8;
    fn hdma_table(&self, slot: usize) -> u16;
    fn bind_hdma(&mut self, bank_slot: u16, table: u16) -> Result<(), DisplayError>;
    fn mode(&self) -> u8;
    fn fake_mode(&self) -> u8;
    fn mode7(&mut self) -> Result<(), DisplayError>;
    fn tilemaps(&mut self) -> Result<(), DisplayError>;
    fn readbacks(&mut self) -> Result<(), DisplayError>;
    fn hdma_enable(&self) -> u8;
    fn publish_hdma(&mut self, value: u8) -> Result<(), DisplayError>;
    fn set_pending_nmi(&mut self, value: u8);
    fn missed(&self) -> u8;
    fn set_missed(&mut self, value: u8);
    fn max_missed(&self) -> u8;
    fn set_max_missed(&mut self, value: u8);
    fn frame_byte(&self) -> u8;
    fn set_frame_byte(&mut self, value: u8);
    fn frame_word(&self) -> u16;
    fn set_frame_word(&mut self, value: u16);
    fn including_lag(&self) -> u16;
    fn set_including_lag(&mut self, value: u16);
}

pub(crate) fn refresh(host: &mut impl RefreshHost) -> Result<(), DisplayError> {
    if host.pending_nmi() != 0 {
        host.palette_and_objects()?;
        host.samus()?;
        host.animations()?;
        host.controls()?;
        for slot in 0..6 {
            if host.hdma_mask(slot) != 0 {
                let bank_slot = u16::from(host.hdma_bank_slot(slot));
                let table = host.hdma_table(slot);
                host.bind_hdma(bank_slot, table)?;
            }
        }
        // Short-circuiting is deliberate: fake mode is only read after actual mode.
        if host.mode() == 7 || host.fake_mode() == 7 {
            host.mode7()?;
        }
        host.tilemaps()?;
        host.readbacks()?;
        let enabled = host.hdma_enable();
        host.publish_hdma(enabled)?;
        host.set_pending_nmi(0);
        host.set_missed(0);
        let next = host.frame_byte().wrapping_add(1);
        host.set_frame_byte(next);
        let next = host.frame_word().wrapping_add(1);
        host.set_frame_word(next);
    } else {
        let missed = host.missed().wrapping_add(1);
        host.set_missed(missed);
        if host.missed() >= host.max_missed() {
            let missed = host.missed();
            host.set_max_missed(missed);
        }
    }
    let next = host.including_lag().wrapping_add(1);
    host.set_including_lag(next);
    Ok(())
}

/// Mutable numeric adaptation media stays private to the one display owner.
/// Public Scene carries authored planes/objects and immutable incarnations.
#[derive(Clone, Debug)]
pub struct ResidentDisplayDescription {
    /// Final resident identities are independent of the last visible Scene line.
    pub image: MediaRef,
    pub palette: MediaRef,
    pub words: Arc<[u16]>,
    pub colors: [u16; 256],
    pub objects: [u8; 544],
}

/// The native alias layout is supplied by the current latch owner, including padding.
fn latch_alias_words(state: &DisplayState) -> [u16; 5] {
    let latches = state.latches;
    let order = state.objects.native_order;
    [
        latches.tile_pointer,
        order.decode([
            u8::from(latches.tile_increment_on_high),
            latches.tile_padding[0],
        ]),
        latches.tile_increment,
        order.decode([latches.tile_remap, latches.tile_padding[1]]),
        latches.tile_read_buffer,
    ]
}

fn validate_resident_assets(
    image: MediaRef,
    palette: MediaRef,
    description: &SceneDescription,
) -> Result<(), DisplayError> {
    if !description.resident.contains(&image.asset)
        || !description.resident.contains(&palette.asset)
    {
        return Err(DisplayError::Scene);
    }
    Ok(())
}

impl ResidentDisplayDescription {
    pub(crate) fn validate(&self, media: &SceneMedia) -> Result<(), DisplayError> {
        let image = media.image(self.image).map_err(|_| DisplayError::Scene)?;
        let palette = media
            .palette(self.palette)
            .map_err(|_| DisplayError::Scene)?;
        if image.palette != self.palette
            || image.words.as_ref() != self.words.as_ref()
            || palette.colors != self.colors
        {
            return Err(DisplayError::Scene);
        }
        Ok(())
    }
    pub(crate) fn validate_state(
        &self,
        state: &DisplayState,
        media: &SceneMedia,
    ) -> Result<(), DisplayError> {
        self.validate(media)?;
        if state.revision() != media.revision() {
            return Err(DisplayError::Revision);
        }
        let aliases = self
            .words
            .get(0x8000..0x8005)
            .ok_or(DisplayError::Incomplete)?;
        if aliases != latch_alias_words(state).as_slice() {
            return Err(DisplayError::Scene);
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub(crate) struct DisplayImage {
    pub(crate) image: MediaRef,
    pub(crate) palette: MediaRef,
    pub(crate) words: Vec<u16>,
    pub(crate) colors: [u16; 256],
    pub(crate) objects: [u8; 544],
}
impl DisplayImage {
    fn snapshot_latch_aliases(&mut self, state: &DisplayState) -> Result<(), DisplayError> {
        self.words
            .get_mut(0x8000..0x8005)
            .ok_or(DisplayError::Incomplete)?
            .copy_from_slice(&latch_alias_words(state));
        Ok(())
    }
    fn validate(&self, media: &SceneMedia) -> Result<(), DisplayError> {
        let image = media.image(self.image).map_err(|_| DisplayError::Scene)?;
        let palette = media
            .palette(self.palette)
            .map_err(|_| DisplayError::Scene)?;
        if image.palette != self.palette
            || image.words.as_ref() != self.words.as_slice()
            || palette.colors != self.colors
        {
            return Err(DisplayError::Scene);
        }
        Ok(())
    }
    fn tile_byte(&mut self, word: u16, high: bool, value: u8) -> Result<(), DisplayError> {
        let at = usize::from(word & 0x7fff);
        let destination = self
            .words
            .get_mut(at)
            .ok_or(DisplayError::Alias { offset: at })?;
        if high {
            *destination = *destination & 0x00ff | u16::from(value) << 8;
        } else {
            *destination = *destination & 0xff00 | u16::from(value);
        }
        Ok(())
    }
}

/// Synchronous room work uses the same current display and resident image.
/// Native bridges must export/adopt at their actual boundary; no second owner
/// or invented room result is supplied by the display component.
pub(crate) trait RoomEffects {
    fn follow_transition(
        &mut self,
        state: &mut DisplayState,
        image: &mut DisplayImage,
        media: &mut Arc<SceneMedia>,
    ) -> Result<(), DisplayError>;
}

/// Actual G raster and byte destinations are required integration inputs.
/// Raster updates the real status and immutable row history before HDMA; it
/// may extend current immutable media while earlier line leases remain valid.
pub(crate) trait SettlementEffects: RoomEffects + owned_hdma::Destinations {
    fn raster_row(
        &mut self,
        state: &mut DisplayState,
        image: &mut DisplayImage,
        media: &mut Arc<SceneMedia>,
        row: u16,
    ) -> Result<(), DisplayError>;
}

struct DisplayEffects<'a, E> {
    image: &'a mut DisplayImage,
    media: &'a mut Arc<SceneMedia>,
    content: &'a HdmaCatalog,
    effects: &'a mut E,
}

impl<E: RoomEffects> irq::OwnedIrqEffects for DisplayEffects<'_, E> {
    fn follow_transition(&mut self, state: &mut DisplayState) -> Result<(), DisplayError> {
        self.effects
            .follow_transition(state, self.image, self.media)
    }
}

impl<E: SettlementEffects> delivery::Effects for DisplayEffects<'_, E> {
    fn initialize_hdma(&mut self, state: &mut DisplayState) -> Result<(), DisplayError> {
        owned_hdma::initialize(state, self.image, self.content, self.effects)
    }
    fn step_hdma(&mut self, state: &mut DisplayState) -> Result<(), DisplayError> {
        owned_hdma::step(state, self.image, self.content, self.effects)
    }
    fn raster_row(&mut self, state: &mut DisplayState, row: u16) -> Result<(), DisplayError> {
        self.effects.raster_row(state, self.image, self.media, row)
    }
}

#[derive(Clone, Copy)]
pub(crate) enum DeliveryStatus {
    Nmi,
    Irq,
    Beam,
    CounterGate,
}

pub(crate) struct OwnedDisplay {
    pub(crate) state: DisplayState,
    image: DisplayImage,
    committed: Scene,
    media: Arc<SceneMedia>,
    hdma_content: Arc<HdmaCatalog>,
    read_results: Vec<(super::records::AssetRef, Vec<u8>)>,
}
impl OwnedDisplay {
    /// Opening adopts supplied committed data and never draws or advances a producer.
    pub(crate) fn open(
        state: DisplayState,
        resident: ResidentDisplayDescription,
        committed: Scene,
        media: Arc<SceneMedia>,
        hdma_content: Arc<HdmaCatalog>,
    ) -> Result<Self, DisplayError> {
        state.validate_media(&media)?;
        if hdma_content.gameplay() != committed.gameplay() {
            return Err(DisplayError::Identity);
        }
        state.validate_hdma_sources(&hdma_content)?;
        if state.revision() != media.revision() || committed.revision() != media.revision() {
            return Err(DisplayError::Revision);
        }
        committed
            .validate(&media)
            .map_err(|_| DisplayError::Scene)?;
        resident.validate_state(&state, &media)?;
        validate_resident_assets(resident.image, resident.palette, committed.description())?;
        let image = DisplayImage {
            image: resident.image,
            palette: resident.palette,
            words: resident.words.to_vec(),
            colors: resident.colors,
            objects: resident.objects,
        };
        Ok(Self {
            state,
            image,
            committed,
            media,
            hdma_content,
            read_results: Vec::new(),
        })
    }
    pub(crate) fn committed(&self) -> &Scene {
        &self.committed
    }
    pub(crate) fn image(&self) -> &DisplayImage {
        &self.image
    }
    fn transfer(&mut self, transfer: &TileTransfer) -> Result<(), DisplayError> {
        if transfer.destination.remap > 3 {
            return Err(DisplayError::Geometry);
        }
        let mut word = transfer.destination.word;
        self.state.latches.tile_pointer = word;
        self.state.latches.tile_increment = transfer.destination.step;
        self.state.latches.tile_remap = transfer.destination.remap;
        self.state.latches.tile_increment_on_high = transfer.destination.increment_on_high;
        for index in 0..transfer.byte_count() {
            // Mutable staging sources are reacquired for every original source byte.
            let value = self.state.source_byte(&transfer.source, index)?;
            let high = match transfer.destination.lanes {
                ByteLanes::Both => index & 1 != 0,
                ByteLanes::Low => false,
                ByteLanes::High => true,
            };
            let mapped = match transfer.destination.remap {
                0 => word,
                1 => word & 0xff00 | (word & 0x00e0) >> 5 | (word & 0x001f) << 3,
                2 => word & 0xfe00 | (word & 0x01c0) >> 6 | (word & 0x003f) << 3,
                _ => word & 0xfc00 | (word & 0x0380) >> 7 | (word & 0x007f) << 3,
            };
            self.image.tile_byte(mapped, high, value)?;
            if high == transfer.destination.increment_on_high {
                word = word.wrapping_add(transfer.destination.step);
                self.state.latches.tile_pointer = word;
            }
        }
        Ok(())
    }
    pub(crate) fn begin_interval(&mut self, fresh_entry: bool) -> Result<(), DisplayError> {
        self.state.begin_interval(fresh_entry)
    }
    pub(crate) fn refresh(&mut self) -> Result<(), DisplayError> {
        if self.state.phase == DisplayPhase::Failed {
            return Err(DisplayError::Poisoned);
        }
        if self.state.phase != DisplayPhase::Producing {
            return Err(DisplayError::Phase);
        }
        match refresh(self) {
            Ok(()) => {
                self.state.phase = DisplayPhase::Refreshed;
                Ok(())
            }
            Err(error) => {
                self.state.phase = DisplayPhase::Failed;
                Err(error)
            }
        }
    }
    pub(crate) fn begin_settlement(&mut self) -> Result<(), DisplayError> {
        if self.state.phase != DisplayPhase::Refreshed {
            return Err(DisplayError::Phase);
        }
        self.state.phase = DisplayPhase::Settling;
        Ok(())
    }
    /// Executes the actual owned beam/IRQ/channel interval. Required G effects
    /// retain their real raster, room and destination behavior at each call.
    pub(crate) fn settle(
        &mut self,
        effects: &mut impl SettlementEffects,
    ) -> Result<(), DisplayError> {
        delivery::settle_owned(
            &mut self.state,
            &mut DisplayEffects {
                image: &mut self.image,
                media: &mut self.media,
                content: &self.hdma_content,
                effects,
            },
        )
    }
    pub(crate) fn disable_irq(&mut self) -> Result<(), DisplayError> {
        if self.state.phase == DisplayPhase::Failed {
            return Err(DisplayError::Poisoned);
        }
        irq::owned_disable(&mut self.state);
        Ok(())
    }
    pub(crate) fn enable_irq(
        &mut self,
        immediate: bool,
        effects: &mut impl RoomEffects,
    ) -> Result<(), DisplayError> {
        if self.state.phase == DisplayPhase::Failed {
            return Err(DisplayError::Poisoned);
        }
        irq::owned_enable(
            &mut self.state,
            immediate,
            &mut DisplayEffects {
                image: &mut self.image,
                media: &mut self.media,
                content: &self.hdma_content,
                effects,
            },
        )
    }
    pub(crate) fn advance_irq(
        &mut self,
        effects: &mut impl RoomEffects,
    ) -> Result<(), DisplayError> {
        if self.state.phase == DisplayPhase::Failed {
            return Err(DisplayError::Poisoned);
        }
        let mut interval = delivery::FaultGuard::new(&mut self.state);
        let result = irq::owned_advance(
            interval.state(),
            &mut DisplayEffects {
                image: &mut self.image,
                media: &mut self.media,
                content: &self.hdma_content,
                effects,
            },
        );
        if result.is_ok() {
            interval.disarm();
        }
        result
    }
    pub(crate) fn read_delivery_status(
        &mut self,
        status: DeliveryStatus,
    ) -> Result<u8, DisplayError> {
        if self.state.phase == DisplayPhase::Failed {
            return Err(DisplayError::Poisoned);
        }
        Ok(match status {
            DeliveryStatus::Nmi => delivery::nmi_status(&self.state),
            DeliveryStatus::Irq => delivery::irq_status(&mut self.state),
            DeliveryStatus::Beam => delivery::beam_status(&self.state),
            DeliveryStatus::CounterGate => delivery::counter_gate_status(&self.state),
        })
    }
    pub(crate) fn write_counter_gate(&mut self, value: u8) -> Result<(), DisplayError> {
        if self.state.phase == DisplayPhase::Failed {
            return Err(DisplayError::Poisoned);
        }
        delivery::counter_gate(&mut self.state, value);
        Ok(())
    }
    /// Full line history is supplied by the actual G late-settlement owner.
    /// Calling this does not execute HDMA, IRQ, room callbacks or a renderer.
    pub(crate) fn commit_scene(
        &mut self,
        description: SceneDescription,
        media: Arc<SceneMedia>,
    ) -> Result<&Scene, DisplayError> {
        if self.state.phase == DisplayPhase::Failed {
            return Err(DisplayError::Poisoned);
        }
        if self.state.phase != DisplayPhase::Settling {
            return Err(DisplayError::Phase);
        }
        if description.tick
            != self
                .committed
                .tick()
                .checked_add(1)
                .ok_or(DisplayError::Phase)?
            || description.gameplay != self.committed.gameplay()
            || description.reference != self.committed.reference()
            || description.revision != self.state.revision()
        {
            self.state.phase = DisplayPhase::Failed;
            return Err(DisplayError::Identity);
        }
        let resident_valid = self
            .image
            .snapshot_latch_aliases(&self.state)
            .and_then(|()| self.image.validate(&media))
            .and_then(|()| {
                validate_resident_assets(self.image.image, self.image.palette, &description)
            });
        if let Err(error) = resident_valid {
            self.state.phase = DisplayPhase::Failed;
            return Err(error);
        }
        if let Err(error) = self.state.validate_hdma_sources(&self.hdma_content) {
            self.state.phase = DisplayPhase::Failed;
            return Err(error);
        }
        let scene = match Scene::new(description, Arc::clone(&media)) {
            Ok(scene) => scene,
            Err(_) => {
                self.state.phase = DisplayPhase::Failed;
                return Err(DisplayError::Scene);
            }
        };
        self.media = media;
        self.committed = scene;
        self.state.phase = DisplayPhase::Committed;
        Ok(&self.committed)
    }
    /// Validation is complete before replacement, preserving the previous owner on failure.
    pub(crate) fn reset(
        &mut self,
        state: DisplayState,
        resident: ResidentDisplayDescription,
        scene: Scene,
        media: Arc<SceneMedia>,
        hdma_content: Arc<HdmaCatalog>,
    ) -> Result<(), DisplayError> {
        let replacement = Self::open(state, resident, scene, media, hdma_content)?;
        *self = replacement;
        Ok(())
    }
    /// Capture committed continuation data without rendering or advancing G.
    pub(crate) fn checkpoint_parts(
        &self,
    ) -> Result<
        (
            DisplayState,
            ResidentDisplayDescription,
            Scene,
            Arc<SceneMedia>,
            Arc<HdmaCatalog>,
        ),
        DisplayError,
    > {
        if self.state.phase != DisplayPhase::Committed {
            return Err(DisplayError::Phase);
        }
        self.state.validate_hdma_sources(&self.hdma_content)?;
        let resident = ResidentDisplayDescription {
            image: self.image.image,
            palette: self.image.palette,
            words: Arc::from(self.image.words.clone()),
            colors: self.image.colors,
            objects: self.image.objects,
        };
        resident.validate_state(&self.state, &self.media)?;
        validate_resident_assets(
            resident.image,
            resident.palette,
            self.committed.description(),
        )?;
        Ok((
            self.state.clone(),
            resident,
            self.committed.clone(),
            Arc::clone(&self.media),
            Arc::clone(&self.hdma_content),
        ))
    }
    pub(crate) fn take_read_results(&mut self) -> Vec<(super::records::AssetRef, Vec<u8>)> {
        std::mem::take(&mut self.read_results)
    }
    /// Publish new immutable media identities. Earlier Scene lines keep their leases.
    pub(crate) fn append_incarnation(
        &mut self,
        image: MediaRef,
        palette: MediaRef,
        atlases: Arc<[AtlasBinding]>,
        maps: Arc<[MapBinding]>,
    ) -> Result<Arc<SceneMedia>, DisplayError> {
        if image.asset != self.image.image.asset || palette.asset != self.image.palette.asset {
            return Err(DisplayError::Identity);
        }
        if self.image.words.len() < 0x801f {
            return Err(DisplayError::Incomplete);
        }
        self.image.snapshot_latch_aliases(&self.state)?;
        self.image.words[0x8005..0x801f].copy_from_slice(&self.image.colors[..26]);
        let mut palettes: Vec<_> = self.media.palette_incarnations().cloned().collect();
        palettes.push(PaletteIncarnation {
            reference: palette,
            colors: self.image.colors,
        });
        let mut images: Vec<_> = self.media.compiled_images().cloned().collect();
        images.push(CompiledImage {
            reference: image,
            palette,
            words: Arc::from(self.image.words.clone()),
            atlases,
            maps,
        });
        let media = SceneMedia::new(
            self.media.revision(),
            self.media.assets().cloned().collect(),
            palettes,
            images,
        )
        .map(Arc::new)
        .map_err(|_| DisplayError::Scene)?;
        self.image.image = image;
        self.image.palette = palette;
        self.media = Arc::clone(&media);
        Ok(media)
    }
}

impl RefreshHost for OwnedDisplay {
    fn pending_nmi(&self) -> u8 {
        self.state.nmi.pending
    }
    fn palette_and_objects(&mut self) -> Result<(), DisplayError> {
        self.image
            .objects
            .copy_from_slice(&self.state.objects.bytes);
        self.image.colors.copy_from_slice(&self.state.visual.colors);
        Ok(())
    }
    fn samus(&mut self) -> Result<(), DisplayError> {
        for slot in 0..2 {
            if self.state.samus[slot].enabled != 0 {
                let first = self.state.samus[slot].first.clone();
                self.transfer(&first)?;
                // A zero second count suppresses only the second transfer.
                if self.state.samus[slot].second.count != 0 {
                    let second = self.state.samus[slot].second.clone();
                    self.transfer(&second)?;
                }
            }
        }
        Ok(())
    }
    fn animations(&mut self) -> Result<(), DisplayError> {
        if self.state.visual.animated_tiles.flags & 0x8000 == 0 {
            return Ok(());
        }
        for slot in (0..6).rev() {
            let current = &self.state.visual.animated_tiles.slots[slot];
            if current.definition == 0 || current.source == 0 {
                continue;
            }
            let transfer = TileTransfer {
                source: TransferSource::Asset(self.state.animated_sources[slot].clone()),
                count: current.size,
                destination: super::state::TileDestination {
                    word: current.vram,
                    step: 1,
                    lanes: ByteLanes::Both,
                    remap: 0,
                    increment_on_high: true,
                },
            };
            self.transfer(&transfer)?;
            self.state.visual.animated_tiles.slots[slot].source = 0;
        }
        Ok(())
    }
    fn controls(&mut self) -> Result<(), DisplayError> {
        // Delivered state and gameplay mirrors have distinct source lifetimes.
        let enabled = self.state.controls.nmi_enable;
        delivery::interrupt_enable(&mut self.state, enabled);
        self.state.effective.forced_blank = self.state.controls.brightness & 0x80 != 0;
        let background3 = self.state.controls.screens[2];
        irq::deliver_output(&mut self.state, irq::Output::Background3Map, background3);
        // Native upload publishes gameplay_TM before delivering current TM.
        self.state.controls.previous_main_layers = self.state.controls.main_layers;
        let main = self.state.controls.main_layers;
        irq::deliver_output(&mut self.state, irq::Output::MainLayers, main);
        let selection = self.state.controls.color_selection;
        irq::deliver_output(&mut self.state, irq::Output::ColorWindow, selection);
        let math = self.state.controls.color_math;
        irq::deliver_output(&mut self.state, irq::Output::ColorArithmetic, math);
        self.state.controls.previous_color_selection = self.state.controls.next_color_selection;
        self.state.controls.previous_color_math = self.state.controls.next_color_math;
        // These current SETINI flags latch later at overscan/vblank source points.
        self.state.raster.interlaced = self.state.controls.geometry & 1 != 0;
        self.state.raster.overscan = self.state.controls.geometry & 4 != 0;
        self.state.controls.committed_fake_mode = self.state.controls.fake_mode;
        Ok(())
    }
    fn hdma_mask(&self, slot: usize) -> u16 {
        self.state.hdma.bindings[slot].enable
    }
    fn hdma_bank_slot(&self, slot: usize) -> u8 {
        self.state.hdma.bindings[slot].bank_slot as u8
    }
    fn hdma_table(&self, slot: usize) -> u16 {
        self.state.hdma.bindings[slot].table
    }
    fn bind_hdma(&mut self, bank_slot: u16, table: u16) -> Result<(), DisplayError> {
        // Canonical slot offsets are validated for the portable owner. The native host
        // still performs every full offset and its aliases through the original helper.
        if bank_slot & 15 != 0 || bank_slot > 0x70 {
            return Err(DisplayError::Alias {
                offset: usize::from(bank_slot),
            });
        }
        self.state.hdma.channels[usize::from(bank_slot >> 4)].source_address = table;
        Ok(())
    }
    fn mode(&self) -> u8 {
        self.state.controls.mode
    }
    fn fake_mode(&self) -> u8 {
        self.state.controls.fake_mode
    }
    fn mode7(&mut self) -> Result<(), DisplayError> {
        if self.state.mode7_tail == 0 {
            return Ok(());
        }
        for index in 0..self.state.mode7_uploads.len() {
            let upload = self.state.mode7_uploads[index].clone();
            if upload.transfer_mode != 0 {
                return Err(DisplayError::Incomplete);
            }
            match upload.target {
                Mode7Target::Tiles(destination) => self.transfer(&TileTransfer {
                    source: upload.source,
                    count: upload.count,
                    destination,
                })?,
                Mode7Target::Palette { first_color } => {
                    self.state.latches.palette_pointer = first_color;
                    self.state.latches.palette_second = false;
                    let count = if upload.count == 0 {
                        65_536
                    } else {
                        usize::from(upload.count)
                    };
                    for at in 0..count {
                        let value = self.state.source_byte(&upload.source, at)?;
                        if self.state.latches.palette_second {
                            let pointer = usize::from(self.state.latches.palette_pointer);
                            self.image.colors[pointer] =
                                u16::from(self.state.latches.palette_buffer)
                                    | (u16::from(value & 0x7f) << 8);
                            self.state.latches.palette_pointer =
                                self.state.latches.palette_pointer.wrapping_add(1);
                        } else {
                            self.state.latches.palette_buffer = value;
                        }
                        self.state.latches.palette_second = !self.state.latches.palette_second;
                    }
                }
            }
        }
        self.state.mode7_tail = 0;
        self.state.mode7_uploads.clear();
        Ok(())
    }
    fn tilemaps(&mut self) -> Result<(), DisplayError> {
        if self.state.upload_tail != 0 {
            for index in 0..self.state.uploads.len() {
                let transfer = self.state.uploads[index].transfer.clone();
                self.transfer(&transfer)?;
            }
        }
        self.state.upload_tail = 0;
        self.state.uploads.clear();
        for slot in 0..4 {
            if self.state.scrolling[slot].enabled == 0 {
                continue;
            }
            // Consume the low enable before any data/source reads or transfers.
            self.state.scrolling[slot].enabled = 0;
            for half in 0..2 {
                let transfer = self.state.scrolling[slot].first[half].clone();
                self.transfer(&transfer)?;
            }
            if self.state.scrolling[slot].wrapped_count != 0 {
                for half in 0..2 {
                    let transfer = self.state.scrolling[slot].wrapped[half].clone();
                    self.transfer(&transfer)?;
                }
            }
        }
        Ok(())
    }
    fn readbacks(&mut self) -> Result<(), DisplayError> {
        if self.state.readback_tail as u8 == 0 {
            return Ok(());
        }
        for readback in &self.state.readbacks {
            if readback.first_word == 0 {
                break;
            }
            let mut bytes = Vec::new();
            let count = if readback.count == 0 {
                65_536
            } else {
                usize::from(readback.count)
            };
            for index in 0..count {
                let word =
                    usize::from(readback.first_word.wrapping_add((index >> 1) as u16) & 0x7fff);
                let value = *self
                    .image
                    .words
                    .get(word)
                    .ok_or(DisplayError::Alias { offset: word })?;
                bytes.push(value.to_le_bytes()[index & 1]);
            }
            self.read_results.push((readback.destination, bytes));
        }
        self.state.readback_tail &= 0xff00;
        self.state.readbacks.clear();
        Ok(())
    }
    fn hdma_enable(&self) -> u8 {
        self.state.controls.hdma_enable
    }
    fn publish_hdma(&mut self, value: u8) -> Result<(), DisplayError> {
        for (slot, channel) in self.state.hdma.channels.iter_mut().enumerate() {
            channel.active = value & (1 << slot) != 0;
        }
        Ok(())
    }
    fn set_pending_nmi(&mut self, value: u8) {
        self.state.nmi.pending = value;
    }
    fn missed(&self) -> u8 {
        self.state.nmi.missed
    }
    fn set_missed(&mut self, value: u8) {
        self.state.nmi.missed = value;
    }
    fn max_missed(&self) -> u8 {
        self.state.nmi.max_missed
    }
    fn set_max_missed(&mut self, value: u8) {
        self.state.nmi.max_missed = value;
    }
    fn frame_byte(&self) -> u8 {
        self.state.nmi.frame_byte
    }
    fn set_frame_byte(&mut self, value: u8) {
        self.state.nmi.frame_byte = value;
    }
    fn frame_word(&self) -> u16 {
        self.state.nmi.frame_word
    }
    fn set_frame_word(&mut self, value: u16) {
        self.state.nmi.frame_word = value;
    }
    fn including_lag(&self) -> u16 {
        self.state.nmi.including_lag
    }
    fn set_including_lag(&mut self, value: u16) {
        self.state.nmi.including_lag = value;
    }
}
