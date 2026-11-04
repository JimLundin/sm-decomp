//! Original refresh order shared by the native bridge and the owned display.
use std::sync::Arc;

use super::hdma_sources::HdmaCatalog;
use super::records::{
    AtlasBinding, MapBinding, MediaRef, ProducedObject, Scene, SceneDescription, SceneMedia,
};
use super::state::{
    ByteLanes, DisplayError, DisplayPhase, DisplayState, Mode7Target, TileTransfer, TransferSource,
};
use super::{delivery, irq, owned_hdma};

pub trait RefreshHost {
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

pub fn refresh(host: &mut impl RefreshHost) -> Result<(), DisplayError> {
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
    pub produced: Arc<[ProducedObject]>,
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
    pub fn validate(&self, media: &SceneMedia) -> Result<(), DisplayError> {
        for (index, record) in self.produced.iter().enumerate() {
            record.validate(media).map_err(|_| DisplayError::Scene)?;
            if self.produced[..index]
                .iter()
                .any(|old| old.object.order == record.object.order)
            {
                return Err(DisplayError::Scene);
            }
        }
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
    pub fn validate_state(
        &self,
        state: &DisplayState,
        media: &SceneMedia,
    ) -> Result<(), DisplayError> {
        self.validate(media)?;
        state
            .producer_ids
            .validate()
            .map_err(|_| DisplayError::Identity)?;
        for record in self.produced.iter() {
            state
                .producer_ids
                .validate_id(record.producer)
                .map_err(|_| DisplayError::Identity)?;
        }
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

#[derive(Clone, Debug, PartialEq)]
pub struct DisplayImage {
    pub image: MediaRef,
    pub palette: MediaRef,
    pub words: Vec<u16>,
    pub colors: [u16; 256],
    pub objects: [u8; 544],
    pub produced: Vec<ProducedObject>,
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
        for (index, record) in self.produced.iter().enumerate() {
            record.validate(media).map_err(|_| DisplayError::Scene)?;
            if self.produced[..index]
                .iter()
                .any(|old| old.object.order == record.object.order)
            {
                return Err(DisplayError::Scene);
            }
        }
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
/// Callback authority for the current settlement operation. Ownership borrows
/// stay private; topology changes use the explicit materializing transition.
pub struct SettlementContext<'a, 'b> {
    state: &'a mut DisplayState,
    image: &'a mut DisplayImage,
    media: Option<&'a mut Arc<SceneMedia>>,
    window: Option<&'a mut crate::menu_window_state::WindowState>,
    controls: Option<&'a mut crate::fx_sources::ExternalControls<'b>>,
}
impl<'a, 'b> SettlementContext<'a, 'b> {
    pub(crate) fn new(
        state: &'a mut DisplayState,
        image: &'a mut DisplayImage,
        media: Option<&'a mut Arc<SceneMedia>>,
        window: Option<&'a mut crate::menu_window_state::WindowState>,
        controls: Option<&'a mut crate::fx_sources::ExternalControls<'b>>,
    ) -> Self {
        Self {
            state,
            image,
            media,
            window,
            controls,
        }
    }
    pub fn state(&self) -> &DisplayState {
        self.state
    }
    pub fn raster_mut(&mut self) -> &mut super::state::RasterState {
        &mut self.state.raster
    }
    pub fn image(&self) -> &DisplayImage {
        self.image
    }
    pub fn media(&self) -> Option<&Arc<SceneMedia>> {
        self.media.as_deref()
    }
    pub fn read_source(
        &mut self,
        owner: super::hdma_sources::LiveOwner,
        offset: usize,
    ) -> Result<u8, DisplayError> {
        use super::hdma_sources::LiveSources;
        if let Some(controls) = self.controls.as_deref_mut() {
            return crate::fx_sources::CurrentSources {
                live: &mut self.state.hdma_live,
                lighting: &mut self.state.visual.fireflea,
                workspace: &mut self.state.workspace,
                controls,
            }
            .read(&mut self.state.hdma, owner, offset);
        }
        crate::fx_sources::DisplaySources {
            live: &mut self.state.hdma_live,
            workspace: &mut self.state.workspace,
            window: self.window.as_deref_mut(),
            lighting: &mut self.state.visual.fireflea,
        }
        .read(&mut self.state.hdma, owner, offset)
    }
    pub fn write_source(
        &mut self,
        owner: super::hdma_sources::LiveOwner,
        offset: usize,
        value: u8,
    ) -> Result<(), DisplayError> {
        use super::hdma_sources::LiveSources;
        if let Some(controls) = self.controls.as_deref_mut() {
            return crate::fx_sources::CurrentSources {
                live: &mut self.state.hdma_live,
                lighting: &mut self.state.visual.fireflea,
                workspace: &mut self.state.workspace,
                controls,
            }
            .write(&mut self.state.hdma, owner, offset, value);
        }
        crate::fx_sources::DisplaySources {
            live: &mut self.state.hdma_live,
            workspace: &mut self.state.workspace,
            window: self.window.as_deref_mut(),
            lighting: &mut self.state.visual.fireflea,
        }
        .write(&mut self.state.hdma, owner, offset, value)
    }
    pub fn read_control_byte(&mut self, address: usize) -> Result<u8, DisplayError> {
        let lane = crate::fx_encoding::control_lane(address).ok_or(DisplayError::Source)?;
        let controls = self
            .controls
            .as_deref_mut()
            .ok_or(DisplayError::Incomplete)?;
        crate::fx_sources::CurrentSources {
            live: &mut self.state.hdma_live,
            lighting: &mut self.state.visual.fireflea,
            workspace: &mut self.state.workspace,
            controls,
        }
        .read_control(&self.state.hdma, lane)
    }
    pub fn write_control_byte(&mut self, address: usize, value: u8) -> Result<(), DisplayError> {
        let lane = crate::fx_encoding::control_lane(address).ok_or(DisplayError::Source)?;
        let controls = self
            .controls
            .as_deref_mut()
            .ok_or(DisplayError::Incomplete)?;
        crate::fx_sources::CurrentSources {
            live: &mut self.state.hdma_live,
            lighting: &mut self.state.visual.fireflea,
            workspace: &mut self.state.workspace,
            controls,
        }
        .write_control(&mut self.state.hdma, lane, value)
    }
    pub fn publish_workspace_byte(
        &mut self,
        point: crate::presentation_workspace::Point,
        value: u8,
    ) -> Result<(), DisplayError> {
        crate::fx_sources::publish_workspace_byte(self.state, point, value)
    }
    pub fn publish_workspace_slice(
        &mut self,
        point: crate::presentation_workspace::Point,
        bytes: &[u8],
    ) -> Result<(), DisplayError> {
        crate::fx_sources::publish_workspace_slice(self.state, point, bytes)
    }
    fn materialize_controls(&mut self) -> Result<crate::fx_encoding::ControlOrigins, DisplayError> {
        let graph = crate::fx_encoding::display_control_graph(self.state)?;
        let origins = crate::fx_encoding::capture_control_origins(self.state, &graph)?;
        let controls = self
            .controls
            .as_deref_mut()
            .ok_or(DisplayError::Incomplete)?;
        let mut values = [0; crate::fx_state::ControlOwnership::LANES];
        let mut live = crate::fx_sources::CurrentSources {
            live: &mut self.state.hdma_live,
            lighting: &mut self.state.visual.fireflea,
            workspace: &mut self.state.workspace,
            controls,
        };
        let mut expected = crate::fx_state::ControlOwnership {
            initialized: true,
            ..Default::default()
        };
        for (lane, &owner) in graph.iter().enumerate() {
            if owner != crate::fx_encoding::ControlOwner::Scalar(lane) {
                expected.select_source(lane);
                if live.scalar_byte(&self.state.hdma.common, lane)? != 0 {
                    return Err(DisplayError::Source);
                }
            }
            values[lane] = live.read_control(&self.state.hdma, lane)?;
        }
        if *live.controls.ownership != expected {
            return Err(DisplayError::Source);
        }
        *live.controls.ownership = Default::default();
        for (lane, value) in values.into_iter().enumerate() {
            live.store_scalar_byte(&mut self.state.hdma.common, lane, value)?;
        }
        Ok(origins)
    }
    fn adopt_controls(&mut self) -> Result<(), DisplayError> {
        let graph = crate::fx_encoding::display_control_graph(self.state)?;
        let controls = self
            .controls
            .as_deref_mut()
            .ok_or(DisplayError::Incomplete)?;
        let edges = crate::fx_encoding::display_control_edges(self.state)?;
        let mut live = crate::fx_sources::CurrentSources {
            live: &mut self.state.hdma_live,
            lighting: &mut self.state.visual.fireflea,
            workspace: &mut self.state.workspace,
            controls,
        };
        let mut ownership = crate::fx_state::ControlOwnership {
            initialized: true,
            ..Default::default()
        };
        for (lane, &owner) in graph.iter().enumerate() {
            if owner != crate::fx_encoding::ControlOwner::Scalar(lane) {
                continue;
            }
            let mut cycle = Vec::new();
            let mut next = lane;
            loop {
                if cycle.contains(&next) {
                    break;
                }
                cycle.push(next);
                let crate::fx_encoding::ControlOwner::Scalar(selected) = edges[next] else {
                    break;
                };
                next = selected;
            }
            if next != lane || cycle.len() <= 1 {
                continue;
            }
            let mut first = None;
            let mut equal = true;
            let mut selected = None;
            let mut conflicting = false;
            for &candidate in &cycle {
                let value = live.scalar_byte(&self.state.hdma.common, candidate)?;
                if let Some(previous) = first {
                    equal &= previous == value;
                } else {
                    first = Some(value);
                }
                let at = crate::fx_encoding::control_address(candidate);
                let bank = 0x7e + u8::try_from(at / 65_536).map_err(|_| DisplayError::Source)?;
                let address = u16::try_from(at % 65_536).map_err(|_| DisplayError::Source)?;
                if let Some((source, offset)) = live.live.physical_coordinate_with_workspace(
                    &self.state.hdma,
                    bank,
                    address,
                    live.workspace,
                )? {
                    let coordinate = live.live.canonical_coordinate(source, offset)?;
                    if let Some(origin) = live.live.native_origins().get(&coordinate) {
                        let origin_lane = crate::fx_encoding::control_lane(origin.address)
                            .filter(|lane| cycle.contains(lane))
                            .ok_or(DisplayError::Source)?;
                        let value = live.scalar_byte(&self.state.hdma.common, origin_lane)?;
                        conflicting |= selected.is_some_and(|previous| previous != value);
                        selected = Some(value);
                    }
                }
            }
            let value = if equal {
                first.ok_or(DisplayError::Incomplete)?
            } else if !conflicting {
                selected.ok_or(DisplayError::Source)?
            } else {
                return Err(DisplayError::Source);
            };
            live.store_scalar_byte(&mut self.state.hdma.common, lane, value)?;
        }
        for (lane, &owner) in graph.iter().enumerate() {
            if owner != crate::fx_encoding::ControlOwner::Scalar(lane) {
                ownership.select_source(lane);
                live.store_scalar_byte(&mut self.state.hdma.common, lane, 0)?;
            }
        }
        *live.controls.ownership = ownership;
        Ok(())
    }
    pub fn replace_workspace_bindings(
        &mut self,
        bindings: crate::presentation_workspace::Bindings,
    ) -> Result<(), DisplayError> {
        let previous = std::mem::replace(&mut self.state.workspace.bindings, bindings);
        let result = self.state.workspace.validate(self.state.revision());
        let bindings = std::mem::replace(&mut self.state.workspace.bindings, previous);
        result?;
        let origins = self.materialize_controls()?;
        self.state.workspace.bindings = bindings;
        crate::fx_encoding::reconcile_control_origins(self.state, &origins)?;
        self.adopt_controls()
    }
}

pub trait RoomEffects {
    fn follow_transition(
        &mut self,
        context: &mut SettlementContext<'_, '_>,
    ) -> Result<(), DisplayError>;
}

/// Actual G raster and byte destinations are required integration inputs.
/// Raster updates the real status and immutable row history before HDMA; it
/// may extend current immutable media while earlier line leases remain valid.
pub trait SettlementEffects: RoomEffects + owned_hdma::Destinations {
    fn raster_row(
        &mut self,
        context: &mut SettlementContext<'_, '_>,
        row: u16,
    ) -> Result<super::records::ObjectContext, DisplayError>;
}

struct DisplayEffects<'a, 'b, E> {
    image: &'a mut DisplayImage,
    media: &'a mut Arc<SceneMedia>,
    content: &'a HdmaCatalog,
    effects: &'a mut E,
    producer_history: &'a mut Vec<Arc<[ProducedObject]>>,
    window: Option<&'a mut crate::menu_window_state::WindowState>,
    controls: Option<&'a mut crate::fx_sources::ExternalControls<'b>>,
}

impl<E: RoomEffects> irq::OwnedIrqEffects for DisplayEffects<'_, '_, E> {
    fn follow_transition(&mut self, state: &mut DisplayState) -> Result<(), DisplayError> {
        let mut context = SettlementContext::new(
            state,
            self.image,
            Some(self.media),
            self.window.as_deref_mut(),
            self.controls.as_deref_mut(),
        );
        self.effects.follow_transition(&mut context)
    }
}

impl<E: SettlementEffects> delivery::Effects for DisplayEffects<'_, '_, E> {
    fn initialize_hdma(&mut self, state: &mut DisplayState) -> Result<(), DisplayError> {
        if let Some(controls) = self.controls.as_deref_mut() {
            return owned_hdma::initialize_with_controls(
                state,
                self.image,
                self.content,
                self.effects,
                controls,
            );
        }
        if let Some(window) = self.window.as_deref_mut() {
            return owned_hdma::initialize_with_window(
                state,
                self.image,
                self.content,
                self.effects,
                window,
            );
        }
        owned_hdma::initialize(state, self.image, self.content, self.effects)
    }
    fn step_hdma(&mut self, state: &mut DisplayState) -> Result<(), DisplayError> {
        if let Some(controls) = self.controls.as_deref_mut() {
            return owned_hdma::step_with_controls(
                state,
                self.image,
                self.content,
                self.effects,
                controls,
            );
        }
        if let Some(window) = self.window.as_deref_mut() {
            return owned_hdma::step_with_window(
                state,
                self.image,
                self.content,
                self.effects,
                window,
            );
        }
        owned_hdma::step(state, self.image, self.content, self.effects)
    }
    fn raster_row(&mut self, state: &mut DisplayState, row: u16) -> Result<(), DisplayError> {
        super::resident_media::raster_row(self, state, row)
    }
}

/// Committed continuation data captured by [`OwnedDisplay::checkpoint_parts`].
pub type CheckpointParts = (
    DisplayState,
    ResidentDisplayDescription,
    Scene,
    Arc<SceneMedia>,
    Arc<HdmaCatalog>,
);

#[derive(Clone, Debug, PartialEq)]
pub struct OwnedDisplay {
    pub state: DisplayState,
    image: DisplayImage,
    committed: Scene,
    media: Arc<SceneMedia>,
    hdma_content: Arc<HdmaCatalog>,
    producer_history: Vec<Arc<[ProducedObject]>>,
}
impl OwnedDisplay {
    /// Opening adopts supplied committed data and never draws or advances a producer.
    pub fn open(
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
        committed
            .validate_producer_ids(&state.producer_ids)
            .map_err(|_| DisplayError::Identity)?;
        resident.validate_state(&state, &media)?;
        validate_resident_assets(resident.image, resident.palette, committed.description())?;
        let image = DisplayImage {
            image: resident.image,
            palette: resident.palette,
            words: resident.words.to_vec(),
            colors: resident.colors,
            objects: resident.objects,
            produced: resident.produced.to_vec(),
        };
        Ok(Self {
            state,
            image,
            committed,
            media,
            hdma_content,
            producer_history: Vec::new(),
        })
    }
    pub fn committed(&self) -> &Scene {
        &self.committed
    }
    pub fn media(&self) -> &Arc<SceneMedia> {
        &self.media
    }
    pub fn image(&self) -> &DisplayImage {
        &self.image
    }
    fn transfer(&mut self, transfer: &TileTransfer) -> Result<(), DisplayError> {
        transfer_into(&mut self.state, &mut self.image, transfer)
    }
    pub fn begin_interval(&mut self, fresh_entry: bool) -> Result<(), DisplayError> {
        self.state.begin_interval(fresh_entry)?;
        self.producer_history.clear();
        Ok(())
    }
    /// Creation has already committed its original fields to the moved G owner.
    pub fn atmospheric_created(&mut self, slot: usize) -> Result<(), DisplayError> {
        if self.state.phase != DisplayPhase::Producing {
            return Err(DisplayError::Phase);
        }
        delivery::poison_on_failure(&mut self.state, |state| {
            match state.atmospheric.slots.get(slot) {
                Some(effect) if effect.frame_and_kind == 0 => {
                    state.atmospheric_producers.slots[slot] = None;
                    Ok(())
                }
                Some(_) => state
                    .atmospheric_producers
                    .created(slot, &mut state.producer_ids),
                None => Err(DisplayError::Geometry),
            }
        })
    }
    pub fn advance_atmosphere(
        &mut self,
        inputs: &mut impl super::atmosphere::Inputs,
    ) -> Result<(), DisplayError> {
        if self.state.phase == DisplayPhase::Failed {
            return Err(DisplayError::Poisoned);
        }
        if self.state.phase != DisplayPhase::Producing {
            return Err(DisplayError::Phase);
        }
        super::atmosphere::advance_owned(&mut self.state, &self.media, inputs)
    }
    pub fn refresh(&mut self) -> Result<(), DisplayError> {
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
    pub fn begin_settlement(&mut self) -> Result<(), DisplayError> {
        if self.state.phase != DisplayPhase::Refreshed {
            return Err(DisplayError::Phase);
        }
        self.state.phase = DisplayPhase::Settling;
        Ok(())
    }
    /// Executes the actual owned beam/IRQ/channel interval. Required G effects
    /// retain their real raster, room and destination behavior at each call.
    pub fn settle(&mut self, effects: &mut impl SettlementEffects) -> Result<(), DisplayError> {
        delivery::settle_owned(
            &mut self.state,
            &mut DisplayEffects {
                image: &mut self.image,
                media: &mut self.media,
                content: &self.hdma_content,
                effects,
                producer_history: &mut self.producer_history,
                window: None,
                controls: None,
            },
        )
    }
    pub(crate) fn settle_with_controls(
        &mut self,
        controls: &mut crate::fx_sources::ExternalControls<'_>,
        effects: &mut impl SettlementEffects,
    ) -> Result<(), DisplayError> {
        delivery::settle_owned(
            &mut self.state,
            &mut DisplayEffects {
                image: &mut self.image,
                media: &mut self.media,
                content: &self.hdma_content,
                effects,
                producer_history: &mut self.producer_history,
                window: None,
                controls: Some(controls),
            },
        )
    }
    /// Full line history is supplied by the actual G late-settlement owner.
    /// Calling this does not execute HDMA, IRQ, room callbacks or a renderer.
    pub fn commit_scene(
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
        if self.image.produced.iter().any(|record| {
            self.state
                .producer_ids
                .validate_id(record.producer)
                .is_err()
        }) {
            self.state.phase = DisplayPhase::Failed;
            return Err(DisplayError::Identity);
        }
        if let Err(error) = resident_valid {
            self.state.phase = DisplayPhase::Failed;
            return Err(error);
        }
        if let Err(error) = self.state.validate_hdma_sources(&self.hdma_content) {
            self.state.phase = DisplayPhase::Failed;
            return Err(error);
        }
        // A no-raster interval retains actual visible history independently of
        // new resident deliveries. Neither opening nor commitment fabricates rows.
        let history = if self.producer_history.is_empty() {
            Arc::from(self.committed.producer_history().to_vec())
        } else {
            Arc::from(self.producer_history.clone())
        };
        let scene = match Scene::new(description, Arc::clone(&media))
            .and_then(|scene| scene.with_producer_history(history))
        {
            Ok(scene) => scene,
            Err(_) => {
                self.state.phase = DisplayPhase::Failed;
                return Err(DisplayError::Scene);
            }
        };
        if scene
            .validate_producer_ids(&self.state.producer_ids)
            .is_err()
        {
            self.state.phase = DisplayPhase::Failed;
            return Err(DisplayError::Identity);
        }
        self.media = media;
        self.committed = scene;
        self.state.phase = DisplayPhase::Committed;
        Ok(&self.committed)
    }
    /// Validation is complete before replacement, preserving the previous owner on failure.
    pub fn reset(
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
    pub fn checkpoint_parts(&self) -> Result<CheckpointParts, DisplayError> {
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
            produced: Arc::from(self.image.produced.clone()),
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
    /// Publish new immutable media identities. Earlier Scene lines keep their leases.
    pub fn append_incarnation(
        &mut self,
        image: MediaRef,
        palette: MediaRef,
        atlases: Arc<[AtlasBinding]>,
        maps: Arc<[MapBinding]>,
    ) -> Result<Arc<SceneMedia>, DisplayError> {
        let media = super::resident_media::append(
            super::resident_media::Resident {
                image: self.image.image,
                palette: self.image.palette,
                words: &mut self.image.words,
                colors: &self.image.colors,
            },
            &self.media,
            super::resident_media::Incarnation {
                image,
                palette,
                atlases,
                maps,
            },
            || latch_alias_words(&self.state),
        )?;
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
        self.image.produced.clone_from(&self.state.objects.produced);
        self.image
            .objects
            .copy_from_slice(&self.state.objects.bytes);
        self.image.colors.copy_from_slice(&self.state.visual.colors);
        Ok(())
    }
    fn samus(&mut self) -> Result<(), DisplayError> {
        delivery::poison_on_failure(&mut self.state, |state| {
            super::samus_tile_upload::upload(&mut OwnedSamusUpload {
                state,
                image: &mut self.image,
                media: &self.media,
            })
        })
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
        self.state.hdma.common.objects[slot].channels
    }
    fn hdma_bank_slot(&self, slot: usize) -> u8 {
        self.state.hdma.common.objects[slot].bank_slot as u8
    }
    fn hdma_table(&self, slot: usize) -> u16 {
        self.state.hdma.common.objects[slot].table
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
        let mut readback_index = 0;
        while let Some(readback) = self.state.readbacks.get(readback_index) {
            let (first_word, destination_asset, count) =
                (readback.first_word, readback.destination, readback.count);
            readback_index += 1;
            if first_word == 0 {
                break;
            }
            let destination = self.state.workspace.source_point(destination_asset, 0).ok();
            let mut bytes = Vec::new();
            let count = if count == 0 {
                65_536
            } else {
                usize::from(count)
            };
            for index in 0..count {
                let word = usize::from(first_word.wrapping_add((index >> 1) as u16) & 0x7fff);
                let value = *self
                    .image
                    .words
                    .get(word)
                    .ok_or(DisplayError::Alias { offset: word })?;
                let byte = value.to_le_bytes()[index & 1];
                if let Some(point) = destination {
                    crate::fx_sources::publish_workspace_byte(
                        &mut self.state,
                        point.advance(index)?,
                        byte,
                    )?;
                } else {
                    bytes.push(byte);
                }
            }
            if destination.is_none() {
                self.state.read_results.push((destination_asset, bytes));
            }
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

fn transfer_into(
    state: &mut DisplayState,
    image: &mut DisplayImage,
    transfer: &TileTransfer,
) -> Result<(), DisplayError> {
    transfer_with_source(state, image, transfer, None, |state, index| {
        state.source_byte(&transfer.source, index)
    })
}
trait TransferHost {
    fn configure(
        &mut self,
        destination: &super::state::TileDestination,
    ) -> Result<(), DisplayError>;
    fn source(&mut self, index: usize) -> Result<u8, DisplayError>;
    fn tile_byte(&mut self, word: u16, high: bool, value: u8) -> Result<(), DisplayError>;
    fn advance_dma(&mut self, channel: Option<usize>) -> Result<(), DisplayError>;
    fn publish_pointer(&mut self, word: u16) -> Result<(), DisplayError>;
}

/// One transfer algorithm. Every current source borrow ends before image/latch writes.
fn transfer_with_host(
    host: &mut impl TransferHost,
    transfer: &TileTransfer,
    dma_channel: Option<usize>,
) -> Result<(), DisplayError> {
    if transfer.destination.remap > 3 {
        return Err(DisplayError::Geometry);
    }
    let mut word = transfer.destination.word;
    host.configure(&transfer.destination)?;
    for index in 0..transfer.byte_count() {
        // Mutable staging sources are reacquired for every original source byte.
        let value = host.source(index)?;
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
        host.tile_byte(mapped, high, value)?;
        host.advance_dma(dma_channel)?;
        if high == transfer.destination.increment_on_high {
            word = word.wrapping_add(transfer.destination.step);
            host.publish_pointer(word)?;
        }
    }
    Ok(())
}

fn configure_transfer(state: &mut DisplayState, destination: &super::state::TileDestination) {
    state.latches.tile_pointer = destination.word;
    state.latches.tile_increment = destination.step;
    state.latches.tile_remap = destination.remap;
    state.latches.tile_increment_on_high = destination.increment_on_high;
}
fn advance_transfer_dma(state: &mut DisplayState, channel: Option<usize>) {
    if let Some(channel) = channel {
        let channel = &mut state.hdma.channels[channel];
        channel.source_address = channel.source_address.wrapping_add(1);
        channel.indirect_cursor = channel.indirect_cursor.wrapping_sub(1);
        channel.transfer_cursor = (channel.transfer_cursor + 1) & 1;
    }
}
struct DirectTransfer<'a, F> {
    state: &'a mut DisplayState,
    image: &'a mut DisplayImage,
    source: &'a mut F,
}
impl<F: FnMut(&DisplayState, usize) -> Result<u8, DisplayError>> TransferHost
    for DirectTransfer<'_, F>
{
    fn configure(
        &mut self,
        destination: &super::state::TileDestination,
    ) -> Result<(), DisplayError> {
        configure_transfer(self.state, destination);
        Ok(())
    }
    fn source(&mut self, index: usize) -> Result<u8, DisplayError> {
        (self.source)(self.state, index)
    }
    fn tile_byte(&mut self, word: u16, high: bool, value: u8) -> Result<(), DisplayError> {
        self.image.tile_byte(word, high, value)
    }
    fn advance_dma(&mut self, channel: Option<usize>) -> Result<(), DisplayError> {
        advance_transfer_dma(self.state, channel);
        Ok(())
    }
    fn publish_pointer(&mut self, word: u16) -> Result<(), DisplayError> {
        self.state.latches.tile_pointer = word;
        Ok(())
    }
}
fn transfer_with_source(
    state: &mut DisplayState,
    image: &mut DisplayImage,
    transfer: &TileTransfer,
    dma_channel: Option<usize>,
    mut source: impl FnMut(&DisplayState, usize) -> Result<u8, DisplayError>,
) -> Result<(), DisplayError> {
    transfer_with_host(
        &mut DirectTransfer {
            state,
            image,
            source: &mut source,
        },
        transfer,
        dma_channel,
    )
}

struct OwnedSamusUpload<'a> {
    state: &'a mut DisplayState,
    image: &'a mut DisplayImage,
    media: &'a SceneMedia,
}
impl OwnedSamusUpload<'_> {
    fn transfer(&self, slot: usize, part: super::samus_tile_upload::Part) -> &TileTransfer {
        match part {
            super::samus_tile_upload::Part::First => &self.state.samus[slot].first,
            super::samus_tile_upload::Part::Second => &self.state.samus[slot].second,
        }
    }
}
impl super::samus_tile_upload::Host for OwnedSamusUpload<'_> {
    type Error = DisplayError;
    type Descriptor = usize;
    type Source = TransferSource;
    fn vmain(&mut self) -> Result<(), DisplayError> {
        self.state.latches.tile_increment = 1;
        self.state.latches.tile_remap = 0;
        self.state.latches.tile_increment_on_high = true;
        Ok(())
    }
    fn enabled(&mut self, half: super::samus_body::Half) -> Result<u8, DisplayError> {
        Ok(self.state.samus[match half {
            super::samus_body::Half::Top => 0,
            super::samus_body::Half::Bottom => 1,
        }]
        .enabled)
    }
    fn descriptor(&mut self, half: super::samus_body::Half) -> Result<usize, DisplayError> {
        let slot = match half {
            super::samus_body::Half::Top => 0,
            super::samus_body::Half::Bottom => 1,
        };
        self.media
            .asset(self.state.samus[slot].descriptor)
            .map_err(|_| DisplayError::Identity)?;
        Ok(slot)
    }
    fn destination(
        &mut self,
        slot: usize,
        part: super::samus_tile_upload::Part,
    ) -> Result<(), DisplayError> {
        if part == super::samus_tile_upload::Part::First
            && self.transfer(slot, part).destination.remap > 3
        {
            return Err(DisplayError::Geometry);
        }
        self.state.latches.tile_pointer = self.transfer(slot, part).destination.word;
        Ok(())
    }
    fn configure_dma(&mut self) -> Result<(), DisplayError> {
        Ok(())
    }
    fn source(&mut self, slot: usize) -> Result<TransferSource, DisplayError> {
        Ok(self.state.samus[slot].first.source.clone())
    }
    fn advance_source(
        &mut self,
        slot: usize,
        _: TransferSource,
        _: u16,
    ) -> Result<TransferSource, DisplayError> {
        Ok(self.state.samus[slot].second.source.clone())
    }
    fn write_source(&mut self, _: &TransferSource) -> Result<(), DisplayError> {
        Ok(())
    }
    fn bank(&mut self, _: usize) -> Result<(), DisplayError> {
        Ok(())
    }
    fn count(
        &mut self,
        slot: usize,
        part: super::samus_tile_upload::Part,
    ) -> Result<u16, DisplayError> {
        Ok(self.transfer(slot, part).count)
    }
    fn write_count(&mut self, _: u16) -> Result<(), DisplayError> {
        Ok(())
    }
    fn dma(
        &mut self,
        slot: usize,
        part: super::samus_tile_upload::Part,
        count: u16,
        source: TransferSource,
    ) -> Result<(), DisplayError> {
        let mut transfer = self.transfer(slot, part).clone();
        transfer.count = count;
        transfer.source = source;
        transfer_into(self.state, self.image, &transfer)
    }
}

impl<E: SettlementEffects> super::resident_media::RasterHost for DisplayEffects<'_, '_, E> {
    fn draw_row(
        &mut self,
        state: &mut DisplayState,
        row: u16,
    ) -> Result<super::records::ObjectContext, DisplayError> {
        let mut context = SettlementContext::new(
            state,
            self.image,
            Some(self.media),
            self.window.as_deref_mut(),
            self.controls.as_deref_mut(),
        );
        self.effects.raster_row(&mut context, row)
    }
    fn media(&self) -> &SceneMedia {
        self.media.as_ref()
    }
    fn produced(&self) -> &[ProducedObject] {
        &self.image.produced
    }
    fn objects(&self) -> &[u8; 544] {
        &self.image.objects
    }
    fn publish_row(&mut self, records: Arc<[ProducedObject]>) {
        self.producer_history.push(records);
    }
}

/// Refresh borrows the current Session. Header aliases read the actual numeric
/// owners; source tokens borrow symbolic allocations at each transfer byte.
struct SessionRefresh<'a> {
    session: &'a mut crate::session::Session,
}
impl RefreshHost for SessionRefresh<'_> {
    fn pending_nmi(&self) -> u8 {
        self.session
            .presentation
            .owner()
            .expect("owned refresh")
            .pending_nmi()
    }
    fn palette_and_objects(&mut self) -> Result<(), DisplayError> {
        self.session
            .presentation
            .owner_mut()
            .expect("owned refresh")
            .palette_and_objects()
    }
    fn samus(&mut self) -> Result<(), DisplayError> {
        self.session
            .presentation
            .owner_mut()
            .expect("owned refresh")
            .samus()
    }
    fn animations(&mut self) -> Result<(), DisplayError> {
        self.session
            .presentation
            .owner_mut()
            .expect("owned refresh")
            .animations()
    }
    fn controls(&mut self) -> Result<(), DisplayError> {
        self.session
            .presentation
            .owner_mut()
            .expect("owned refresh")
            .controls()
    }
    fn hdma_mask(&self, slot: usize) -> u16 {
        self.session
            .presentation
            .owner()
            .expect("owned refresh")
            .hdma_mask(slot)
    }
    fn hdma_bank_slot(&self, slot: usize) -> u8 {
        self.session
            .presentation
            .owner()
            .expect("owned refresh")
            .hdma_bank_slot(slot)
    }
    fn hdma_table(&self, slot: usize) -> u16 {
        self.session
            .presentation
            .owner()
            .expect("owned refresh")
            .hdma_table(slot)
    }
    fn bind_hdma(&mut self, bank_slot: u16, table: u16) -> Result<(), DisplayError> {
        self.session
            .presentation
            .owner_mut()
            .expect("owned refresh")
            .bind_hdma(bank_slot, table)
    }
    fn mode(&self) -> u8 {
        self.session
            .presentation
            .owner()
            .expect("owned refresh")
            .mode()
    }
    fn fake_mode(&self) -> u8 {
        self.session
            .presentation
            .owner()
            .expect("owned refresh")
            .fake_mode()
    }
    fn mode7(&mut self) -> Result<(), DisplayError> {
        self.session
            .presentation
            .owner_mut()
            .expect("owned refresh")
            .mode7()
    }
    fn tilemaps(&mut self) -> Result<(), DisplayError> {
        if self.session.queued_tilemaps.tail != 0 {
            self.drain_queue()?;
            // This semantic list described the same packed queue at import.
            // Draining both would publish every transfer twice.
            let owner = self.session.presentation.owner_mut()?;
            owner.state.uploads.clear();
            owner.state.upload_tail = 0;
        }
        if self.session.presentation.owner()?.state.upload_tail != 0 {
            let count = self.session.presentation.owner()?.state.uploads.len();
            for index in 0..count {
                let transfer = self.session.presentation.owner()?.state.uploads[index]
                    .transfer
                    .clone();
                self.transfer(&transfer)?;
            }
        }
        self.session.presentation.owner_mut()?.state.uploads.clear();
        self.session.presentation.owner_mut()?.state.upload_tail = 0;
        for slot in 0..4 {
            if self.session.presentation.owner()?.state.scrolling[slot].enabled == 0 {
                continue;
            }
            self.session.presentation.owner_mut()?.state.scrolling[slot].enabled = 0;
            for half in 0..2 {
                let transfer =
                    self.session.presentation.owner()?.state.scrolling[slot].first[half].clone();
                self.transfer(&transfer)?;
            }
            if self.session.presentation.owner()?.state.scrolling[slot].wrapped_count != 0 {
                for half in 0..2 {
                    let transfer = self.session.presentation.owner()?.state.scrolling[slot].wrapped
                        [half]
                        .clone();
                    self.transfer(&transfer)?;
                }
            }
        }
        Ok(())
    }
    fn readbacks(&mut self) -> Result<(), DisplayError> {
        self.session
            .presentation
            .owner_mut()
            .expect("owned refresh")
            .readbacks()
    }
    fn hdma_enable(&self) -> u8 {
        self.session
            .presentation
            .owner()
            .expect("owned refresh")
            .hdma_enable()
    }
    fn publish_hdma(&mut self, value: u8) -> Result<(), DisplayError> {
        self.session
            .presentation
            .owner_mut()
            .expect("owned refresh")
            .publish_hdma(value)
    }
    fn set_pending_nmi(&mut self, value: u8) {
        self.session
            .presentation
            .owner_mut()
            .expect("owned refresh")
            .set_pending_nmi(value)
    }
    fn missed(&self) -> u8 {
        self.session
            .presentation
            .owner()
            .expect("owned refresh")
            .missed()
    }
    fn set_missed(&mut self, value: u8) {
        self.session
            .presentation
            .owner_mut()
            .expect("owned refresh")
            .set_missed(value)
    }
    fn max_missed(&self) -> u8 {
        self.session
            .presentation
            .owner()
            .expect("owned refresh")
            .max_missed()
    }
    fn set_max_missed(&mut self, value: u8) {
        self.session
            .presentation
            .owner_mut()
            .expect("owned refresh")
            .set_max_missed(value)
    }
    fn frame_byte(&self) -> u8 {
        self.session
            .presentation
            .owner()
            .expect("owned refresh")
            .frame_byte()
    }
    fn set_frame_byte(&mut self, value: u8) {
        self.session
            .presentation
            .owner_mut()
            .expect("owned refresh")
            .set_frame_byte(value)
    }
    fn frame_word(&self) -> u16 {
        self.session
            .presentation
            .owner()
            .expect("owned refresh")
            .frame_word()
    }
    fn set_frame_word(&mut self, value: u16) {
        self.session
            .presentation
            .owner_mut()
            .expect("owned refresh")
            .set_frame_word(value)
    }
    fn including_lag(&self) -> u16 {
        self.session
            .presentation
            .owner()
            .expect("owned refresh")
            .including_lag()
    }
    fn set_including_lag(&mut self, value: u16) {
        self.session
            .presentation
            .owner_mut()
            .expect("owned refresh")
            .set_including_lag(value)
    }
}
impl SessionRefresh<'_> {
    fn transfer(&mut self, transfer: &TileTransfer) -> Result<(), DisplayError> {
        self.transfer_channel(transfer, None)
    }
    fn transfer_channel(
        &mut self,
        transfer: &TileTransfer,
        channel: Option<usize>,
    ) -> Result<(), DisplayError> {
        let result = transfer_with_host(
            &mut SessionTransfer {
                session: self.session,
                source: &transfer.source,
            },
            transfer,
            channel,
        );
        if result.is_ok()
            && let Some(channel) = channel
        {
            self.session.presentation.owner_mut()?.state.hdma.channels[channel].dma_active = false;
        }
        result
    }
    fn header_byte(&self, offset: usize) -> Result<u8, DisplayError> {
        let at = 0xd0usize.checked_add(offset).ok_or(DisplayError::Queue)?;
        crate::session_encoding::projected_byte(self.session, at)
            .ok_or(DisplayError::Alias { offset: at })
    }
    fn header_word(&self, offset: usize) -> Result<u16, DisplayError> {
        // The original drain's word query is LE even when producers use native stores.
        Ok(u16::from_le_bytes([
            self.header_byte(offset)?,
            self.header_byte(offset.checked_add(1).ok_or(DisplayError::Queue)?)?,
        ]))
    }
    fn drain_queue(&mut self) -> Result<(), DisplayError> {
        // The original writes a slice-word sentinel at the full byte tail before
        // its DMA setup. Tail aliases may change any later header or source.
        let tail = usize::from(self.session.queued_tilemaps.tail);
        let first = 0xd0usize.checked_add(tail).ok_or(DisplayError::Queue)?;
        let second = first.checked_add(1).ok_or(DisplayError::Queue)?;
        crate::session_encoding::projected_byte(self.session, first)
            .ok_or(DisplayError::Alias { offset: first })?;
        crate::session_encoding::projected_byte(self.session, second)
            .ok_or(DisplayError::Alias { offset: second })?;
        if !crate::session_encoding::write_projected_byte(self.session, first, 0)
            || !crate::session_encoding::write_projected_byte(self.session, second, 0)
        {
            return Err(DisplayError::Incomplete);
        }
        // DMAP1/BBAD1 setup precedes all original record queries.
        let channel = &mut self.session.presentation.owner_mut()?.state.hdma.channels[1];
        channel.mode = 1;
        channel.destination = 0x18;
        channel.from_destination = false;
        channel.indirect = false;
        channel.fixed = false;
        channel.decrement = false;
        channel.unused_bit = false;
        let mut entry = 0usize;
        loop {
            let count = self.header_word(entry)?;
            if count == 0 {
                break;
            }
            let destination = self.header_word(entry.checked_add(5).ok_or(DisplayError::Queue)?)?;
            // Preserve the native source-word query before the bank-byte query.
            let address = self.header_word(entry.checked_add(2).ok_or(DisplayError::Queue)?)?;
            let bank = self.header_byte(entry.checked_add(4).ok_or(DisplayError::Queue)?)?;
            let bytes = address.to_le_bytes();
            let token = [bytes[0], bytes[1], bank];
            let channel = &mut self.session.presentation.owner_mut()?.state.hdma.channels[1];
            channel.source_address = address;
            channel.source_bank = bank;
            channel.indirect_cursor = count;
            channel.transfer_cursor = 0;
            channel.dma_active = true;
            let source = self
                .session
                .presentation
                .owner()?
                .state
                .queued_sources
                .get(&token)
                .cloned()
                .ok_or(DisplayError::Incomplete)?;
            self.transfer_channel(
                &TileTransfer {
                    source,
                    count,
                    destination: super::state::TileDestination {
                        word: destination,
                        step: if destination & 0x8000 != 0 { 32 } else { 1 },
                        lanes: ByteLanes::Both,
                        remap: 0,
                        increment_on_high: true,
                    },
                },
                Some(1),
            )?;
            entry = entry.checked_add(7).ok_or(DisplayError::Queue)?;
        }
        self.session.queued_tilemaps.tail = 0;
        Ok(())
    }
}
pub(crate) fn refresh_session(session: &mut crate::session::Session) -> Result<(), DisplayError> {
    let owner = session.presentation.owner_mut()?;
    if owner.state.phase == DisplayPhase::Failed {
        return Err(DisplayError::Poisoned);
    }
    if owner.state.phase != DisplayPhase::Producing {
        return Err(DisplayError::Phase);
    }
    match refresh(&mut SessionRefresh { session }) {
        Ok(()) => {
            session.presentation.owner_mut()?.state.phase = DisplayPhase::Refreshed;
            Ok(())
        }
        Err(error) => {
            session.presentation.owner_mut()?.state.phase = DisplayPhase::Failed;
            Err(error)
        }
    }
}

struct SessionTransfer<'a> {
    session: &'a mut crate::session::Session,
    source: &'a TransferSource,
}
impl TransferHost for SessionTransfer<'_> {
    fn configure(
        &mut self,
        destination: &super::state::TileDestination,
    ) -> Result<(), DisplayError> {
        configure_transfer(
            &mut self.session.presentation.owner_mut()?.state,
            destination,
        );
        Ok(())
    }
    fn source(&mut self, index: usize) -> Result<u8, DisplayError> {
        match self.source {
            TransferSource::ActorState { address } => {
                let displacement = u16::try_from(index).map_err(|_| DisplayError::Source)?;
                let address = usize::from(address.wrapping_add(displacement));
                // This is the current aggregate graph, including actual FX/Window/scalar aliases.
                // No Presentation borrow survives this byte query or the following destination write.
                crate::session_encoding::projected_byte(self.session, address)
                    .ok_or(DisplayError::Alias { offset: address })
            }
            TransferSource::QueuedScroll { offset } => {
                let at = offset.checked_add(index).ok_or(DisplayError::Source)?;
                self.session
                    .queued_tilemaps
                    .scrolling
                    .get(at >> 1)
                    .map(|word| word.to_le_bytes()[at & 1])
                    .ok_or(DisplayError::Alias { offset: at })
            }
            // LEGACY(display-state-schema): tag3 has no current RAM origin.
            // Retained codec/standalone fixtures cannot become a live Session mirror.
            TransferSource::ActorTilemap { .. } => Err(DisplayError::Incomplete),
            source => self
                .session
                .presentation
                .owner()?
                .state
                .source_byte(source, index),
        }
    }
    fn tile_byte(&mut self, word: u16, high: bool, value: u8) -> Result<(), DisplayError> {
        self.session
            .presentation
            .owner_mut()?
            .image
            .tile_byte(word, high, value)
    }
    fn advance_dma(&mut self, channel: Option<usize>) -> Result<(), DisplayError> {
        advance_transfer_dma(&mut self.session.presentation.owner_mut()?.state, channel);
        Ok(())
    }
    fn publish_pointer(&mut self, word: u16) -> Result<(), DisplayError> {
        self.session
            .presentation
            .owner_mut()?
            .state
            .latches
            .tile_pointer = word;
        Ok(())
    }
}
