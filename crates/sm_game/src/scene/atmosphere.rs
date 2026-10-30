//! Complete atmospheric tile output with live queries and ordered staging effects.
#![forbid(unsafe_code)]

use super::records::{
    DisplayProducerIds, ObjectContext, ObjectSize, ObjectVariants, ProducedObject,
    ProducerObjectId, Revision, SceneAsset, SceneError, SceneMedia, SceneObject, ScreenPosition,
};
use super::state::{DisplayError, DisplayState};
use std::{collections::BTreeMap, sync::Arc};

pub use super::atmosphere_protocol::{AdvanceHost, advance};
pub use super::atmosphere_protocol::{AtmosphericEffects, AtmosphericSlot};

/// Independent atmospheric lifecycle slots in the shared display namespace.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AtmosphericProducers {
    pub slots: [Option<ProducerObjectId>; 4],
}
impl AtmosphericProducers {
    pub fn created(
        &mut self,
        slot: usize,
        ids: &mut DisplayProducerIds,
    ) -> Result<(), DisplayError> {
        ids.allocate_with(|id| {
            let at = self.slots.get_mut(slot).ok_or(SceneError::Geometry)?;
            *at = Some(id);
            Ok(())
        })
        .map_err(|error| match error {
            SceneError::Geometry => DisplayError::Geometry,
            _ => DisplayError::Identity,
        })?;
        Ok(())
    }
    pub fn validate(
        &self,
        effects: &AtmosphericEffects,
        ids: &DisplayProducerIds,
    ) -> Result<(), DisplayError> {
        ids.validate().map_err(|_| DisplayError::Identity)?;
        for (slot, identity) in self.slots.iter().enumerate() {
            if effects.slots[slot].frame_and_kind != 0 && identity.is_none() {
                return Err(DisplayError::Incomplete);
            }
            if let Some(id) = identity {
                ids.validate_id(*id).map_err(|_| DisplayError::Identity)?;
                if self.slots[..slot].contains(&Some(*id)) {
                    return Err(DisplayError::Identity);
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AtmosphericTile {
    pub native_word: u16,
    pub variants: Arc<ObjectVariants>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AtmosphericKind {
    pub duration_pointer: u16,
    pub durations: Arc<[u16]>,
    pub frame_count: u16,
    pub tile_pointer: u16,
    pub tiles: Arc<[AtmosphericTile]>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AtmosphericSpritePart {
    pub x: u16,
    pub y: u8,
    pub native_word: u16,
    pub variants: Arc<ObjectVariants>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AtmosphericSprite {
    /// Explicitly supplied null-table entries skip cursor/count publication.
    pub present: bool,
    pub parts: Arc<[AtmosphericSpritePart]>,
}
/// Authored media and source-order numeric definitions are supplied by tools.
/// Operational queries remain lazy; validation never runs in native gameplay.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AtmosphereCatalog {
    pub revision: Revision,
    pub kinds: BTreeMap<u16, AtmosphericKind>,
    pub sprites: BTreeMap<u16, AtmosphericSprite>,
}
impl AtmosphereCatalog {
    fn kind(&self, kind: u16) -> Result<&AtmosphericKind, DisplayError> {
        self.kinds.get(&kind).ok_or(DisplayError::Source)
    }
    pub fn validate(&self, media: &SceneMedia) -> Result<(), DisplayError> {
        if self.revision != media.revision() {
            return Err(DisplayError::Revision);
        }
        for kind in self.kinds.values() {
            for tile in kind.tiles.iter() {
                tile.variants
                    .validate(media, tile.native_word & 0xff)
                    .map_err(|_| DisplayError::Scene)?;
            }
        }
        for sprite in self.sprites.values() {
            if !sprite.present {
                continue;
            }
            for piece in sprite.parts.iter() {
                piece
                    .variants
                    .validate(media, piece.native_word & 0xff)
                    .map_err(|_| DisplayError::Scene)?;
            }
        }
        Ok(())
    }
    pub fn validate_expected(&self, expected: &Self) -> Result<(), DisplayError> {
        if self != expected {
            return Err(DisplayError::Incomplete);
        }
        Ok(())
    }
}

pub trait Host {
    type Error;
    fn frame_and_kind(&mut self, slot: usize) -> Result<u16, Self::Error>;
    fn cursor(&mut self) -> Result<u16, Self::Error>;
    fn x(&mut self, slot: usize) -> Result<u16, Self::Error>;
    fn camera_x(&mut self) -> Result<u16, Self::Error>;
    fn y(&mut self, slot: usize) -> Result<u16, Self::Error>;
    fn camera_y(&mut self) -> Result<u16, Self::Error>;
    /// Entry is the captured full cursor, in bytes relative to staging.
    fn write_x(&mut self, entry: usize, value: u8) -> Result<(), Self::Error>;
    fn write_y(&mut self, entry: usize, value: u8) -> Result<(), Self::Error>;
    /// Capture the loaded table before computing its native kind-word offset.
    fn tile_pointer(&mut self, kind: u16) -> Result<u16, Self::Error>;
    /// Capture the native record lazily, retaining the original diagnostic.
    fn read_tile(&mut self, address: u16) -> Result<u16, Self::Error>;
    /// Preflight the whole native word before either staging byte changes.
    fn write_tile(&mut self, entry: usize, value: u16) -> Result<(), Self::Error>;
    /// Publish little-endian low then high bytes, retaining completed prefixes.
    fn publish_cursor(&mut self, value: u16) -> Result<(), Self::Error>;
    fn written(&mut self) -> Result<(), Self::Error>;
}

pub fn draw_tile<H: Host>(host: &mut H, slot: usize, kind: u16) -> Result<(), H::Error> {
    let frame_offset = 2 * u16::from(host.frame_and_kind(slot)? as u8);
    let saved = host.cursor()?;
    let x = host.x(slot)?.wrapping_sub(host.camera_x()?).wrapping_sub(4);
    if (x as i16) < 0
        || host
            .x(slot)?
            .wrapping_sub(host.camera_x()?)
            .wrapping_sub(260)
            & 0x8000
            == 0
    {
        return Ok(());
    }
    let entry = usize::from(host.cursor()?);
    host.write_x(entry, x as u8)?;
    let y = host.y(slot)?.wrapping_sub(host.camera_y()?).wrapping_sub(4);
    if (y as i16) < 0
        || host
            .y(slot)?
            .wrapping_sub(host.camera_y()?)
            .wrapping_sub(260)
            & 0x8000
            == 0
    {
        return Ok(());
    }
    host.write_y(entry, y as u8)?;
    let pointer = host.tile_pointer(kind)?;
    let tile = host.read_tile(pointer.wrapping_add(frame_offset))?;
    host.write_tile(entry, tile)?;
    host.publish_cursor(saved.wrapping_add(4))?;
    host.written()
}

/// Observations are queried from their actual G owners, never cached in display.
pub trait Inputs {
    fn camera_x(&mut self, state: &DisplayState) -> Result<u16, DisplayError>;
    fn camera_y(&mut self, state: &DisplayState) -> Result<u16, DisplayError>;
    fn fx_y(&mut self, state: &DisplayState) -> Result<u16, DisplayError>;
    fn object_context(
        &mut self,
        controls: &super::state::DisplayControls,
        effective: &super::state::EffectiveDisplayDescription,
    ) -> Result<ObjectContext, DisplayError>;
}

pub fn advance_owned(
    state: &mut DisplayState,
    media: &SceneMedia,
    inputs: &mut impl Inputs,
) -> Result<(), DisplayError> {
    let mut owner = super::delivery::FaultGuard::new(state);
    let result = advance(&mut Owned {
        state: owner.state(),
        media,
        inputs,
        duration_kind: None,
        tile_kind: None,
        active_slot: 0,
        pending: None,
    });
    if result.is_ok() {
        owner.disarm();
    }
    result
}

fn semantic_object(
    media: &SceneMedia,
    producer: ProducerObjectId,
    variants: Arc<ObjectVariants>,
    native: NativeObject,
    native_word: u16,
    context: ObjectContext,
) -> Result<ProducedObject, DisplayError> {
    let NativeObject { entry, x, y, size } = native;
    if producer.0 == 0 {
        return Err(DisplayError::Identity);
    }
    // Odd/wide native aliases remain explicit compatibility responsibilities.
    if entry >= 512 || entry & 3 != 0 {
        return Err(DisplayError::Alias { offset: entry });
    }
    let binding = variants
        .binding(context.selection.size_set, size)
        .map_err(|_| DisplayError::Incomplete)?;
    let definition = match media
        .asset(binding.visual)
        .map_err(|_| DisplayError::Identity)?
    {
        SceneAsset::Visual(definition) => definition,
        _ => return Err(DisplayError::Identity),
    };
    let piece = definition
        .frames
        .get(usize::from(binding.frame))
        .and_then(|frame| frame.parts.get(usize::from(binding.part)))
        .ok_or(DisplayError::Incomplete)?;
    if piece.tile != native_word & 0xff
        || piece.atlas != context.selection.atlases[usize::from(native_word >> 8 & 1)]
        || piece.size != size
    {
        return Err(DisplayError::Scene);
    }
    let object = ProducedObject {
        producer,
        variants,
        context,
        native_word,
        object: SceneObject {
            order: (entry / 4) as u16,
            visual: binding.visual,
            frame: binding.frame,
            part: binding.part,
            atlas: piece.atlas,
            material: piece.material,
            tile: piece.tile,
            position: ScreenPosition {
                x: x as i16,
                y: y as i16,
            },
            size: piece.size,
            palette: ((native_word >> 9) & 7) as u8,
            priority: ((native_word >> 12) & 3) as u8,
            flip_horizontal: native_word & 0x4000 != 0,
            flip_vertical: native_word & 0x8000 != 0,
        },
    };
    object.validate(media).map_err(|_| DisplayError::Scene)?;
    Ok(object)
}

fn record(
    objects: &mut super::state::ObjectStaging,
    ids: &DisplayProducerIds,
    object: ProducedObject,
) -> Result<(), DisplayError> {
    ids.validate_id(object.producer)
        .map_err(|_| DisplayError::Identity)?;
    objects
        .produced
        .retain(|old| old.object.order != object.object.order);
    objects.produced.push(object);
    objects.produced.sort_by_key(|record| record.object.order);
    Ok(())
}
/// A native object entry with its completed staging geometry.
struct NativeObject {
    entry: usize,
    x: u16,
    y: u16,
    size: ObjectSize,
}
fn object_geometry(
    objects: &super::state::ObjectStaging,
    entry: usize,
    x: u16,
    y: u16,
) -> Result<NativeObject, DisplayError> {
    let bits = objects.byte(512 + entry / 16)? >> (2 * ((entry / 4) & 3));
    // Native high-table stores OR their bits, so use the completed staging
    // geometry rather than clearing retained X-high or size bits in Scene.
    let x = (x & !0x01ff) | u16::from(objects.byte(entry)?) | (u16::from(bits & 1) << 8);
    let y = (y & !0x00ff) | u16::from(objects.byte(entry + 1)?);
    let size = if bits & 2 != 0 {
        ObjectSize::Large
    } else {
        ObjectSize::Small
    };
    Ok(NativeObject { entry, x, y, size })
}

struct PendingTile {
    entry: usize,
    x: u16,
    y: u16,
    native_word: u16,
    tile: Option<AtmosphericTile>,
}
struct Owned<'a, I> {
    state: &'a mut DisplayState,
    media: &'a SceneMedia,
    inputs: &'a mut I,
    duration_kind: Option<u16>,
    tile_kind: Option<u16>,
    active_slot: usize,
    pending: Option<PendingTile>,
}
impl<I: Inputs> Host for Owned<'_, I> {
    type Error = DisplayError;
    fn frame_and_kind(&mut self, slot: usize) -> Result<u16, DisplayError> {
        Ok(self.state.atmospheric.slots[slot].frame_and_kind)
    }
    fn cursor(&mut self) -> Result<u16, DisplayError> {
        Ok(u16::from_le_bytes(
            self.state
                .objects
                .native_order
                .encode(self.state.objects.cursor),
        ))
    }
    fn x(&mut self, slot: usize) -> Result<u16, DisplayError> {
        Ok(self.state.atmospheric.slots[slot].x)
    }
    fn y(&mut self, slot: usize) -> Result<u16, DisplayError> {
        Ok(self.state.atmospheric.slots[slot].y)
    }
    fn camera_x(&mut self) -> Result<u16, DisplayError> {
        self.inputs.camera_x(self.state)
    }
    fn camera_y(&mut self) -> Result<u16, DisplayError> {
        self.inputs.camera_y(self.state)
    }
    fn write_x(&mut self, entry: usize, value: u8) -> Result<(), DisplayError> {
        self.state.objects.write_byte(entry, value)?;
        self.pending = Some(PendingTile {
            entry,
            x: u16::from(value),
            y: 0,
            native_word: 0,
            tile: None,
        });
        Ok(())
    }
    fn write_y(&mut self, entry: usize, value: u8) -> Result<(), DisplayError> {
        self.state.objects.write_byte(entry + 1, value)?;
        self.pending.as_mut().ok_or(DisplayError::Phase)?.y = u16::from(value);
        Ok(())
    }
    fn tile_pointer(&mut self, kind: u16) -> Result<u16, DisplayError> {
        let pointer = self.state.atmosphere_catalog.kind(kind)?.tile_pointer;
        self.tile_kind = Some(kind);
        Ok(pointer)
    }
    fn read_tile(&mut self, address: u16) -> Result<u16, DisplayError> {
        let kind = self
            .state
            .atmosphere_catalog
            .kind(self.tile_kind.ok_or(DisplayError::Phase)?)?;
        let displacement = address.wrapping_sub(kind.tile_pointer);
        if displacement & 1 != 0 {
            return Err(DisplayError::Source);
        }
        let tile = kind
            .tiles
            .get(usize::from(displacement >> 1))
            .ok_or(DisplayError::Source)?
            .clone();
        let word = tile.native_word;
        self.pending.as_mut().ok_or(DisplayError::Phase)?.tile = Some(tile);
        Ok(word)
    }
    fn write_tile(&mut self, entry: usize, value: u16) -> Result<(), DisplayError> {
        self.state.objects.write_native_slice(entry + 2, value)?;
        self.pending
            .as_mut()
            .ok_or(DisplayError::Phase)?
            .native_word = value;
        Ok(())
    }
    fn publish_cursor(&mut self, value: u16) -> Result<(), DisplayError> {
        let [low, high] = value.to_le_bytes();
        self.state.objects.write_byte(544, low)?;
        self.state.objects.write_byte(545, high)
    }
    fn written(&mut self) -> Result<(), DisplayError> {
        let pending = self.pending.take().ok_or(DisplayError::Phase)?;
        let tile = pending.tile.ok_or(DisplayError::Source)?;
        let producer = self.state.atmospheric_producers.slots[self.active_slot]
            .ok_or(DisplayError::Incomplete)?;
        let context = self
            .inputs
            .object_context(&self.state.controls, &self.state.effective)?;
        let native = object_geometry(&self.state.objects, pending.entry, pending.x, pending.y)?;
        let object = semantic_object(
            self.media,
            producer,
            tile.variants,
            native,
            pending.native_word,
            context,
        )?;
        record(&mut self.state.objects, &self.state.producer_ids, object)?;
        Ok(())
    }
}
impl<I: Inputs> AdvanceHost for Owned<'_, I> {
    type Error = DisplayError;
    fn frame_and_kind(&mut self, slot: usize) -> Result<u16, DisplayError> {
        Host::frame_and_kind(self, slot)
    }
    fn set_frame_and_kind(&mut self, slot: usize, value: u16) -> Result<(), DisplayError> {
        self.state.atmospheric.slots[slot].frame_and_kind = value;
        if value == 0 {
            self.state.atmospheric_producers.slots[slot] = None;
        }
        Ok(())
    }
    fn remaining(&mut self, slot: usize) -> Result<u16, DisplayError> {
        Ok(self.state.atmospheric.slots[slot].remaining)
    }
    fn set_remaining(&mut self, slot: usize, value: u16) -> Result<(), DisplayError> {
        self.state.atmospheric.slots[slot].remaining = value;
        Ok(())
    }
    fn x(&mut self, slot: usize) -> Result<u16, DisplayError> {
        Host::x(self, slot)
    }
    fn set_x(&mut self, slot: usize, value: u16) -> Result<(), DisplayError> {
        self.state.atmospheric.slots[slot].x = value;
        Ok(())
    }
    fn y(&mut self, slot: usize) -> Result<u16, DisplayError> {
        Host::y(self, slot)
    }
    fn set_y(&mut self, slot: usize, value: u16) -> Result<(), DisplayError> {
        self.state.atmospheric.slots[slot].y = value;
        Ok(())
    }
    fn camera_x(&mut self) -> Result<u16, DisplayError> {
        Host::camera_x(self)
    }
    fn camera_y(&mut self) -> Result<u16, DisplayError> {
        Host::camera_y(self)
    }
    fn fx_y(&mut self) -> Result<u16, DisplayError> {
        self.inputs.fx_y(self.state)
    }
    fn duration_pointer(&mut self, kind: u16) -> Result<u16, DisplayError> {
        let pointer = self.state.atmosphere_catalog.kind(kind)?.duration_pointer;
        self.duration_kind = Some(kind);
        Ok(pointer)
    }
    fn duration(&mut self, address: u16) -> Result<u16, DisplayError> {
        let kind = self
            .state
            .atmosphere_catalog
            .kind(self.duration_kind.ok_or(DisplayError::Phase)?)?;
        let displacement = address.wrapping_sub(kind.duration_pointer);
        if displacement & 1 != 0 {
            return Err(DisplayError::Source);
        }
        kind.durations
            .get(usize::from(displacement >> 1))
            .copied()
            .ok_or(DisplayError::Source)
    }
    fn frame_count(&mut self, kind: u16) -> Result<u16, DisplayError> {
        Ok(self.state.atmosphere_catalog.kind(kind)?.frame_count)
    }
    fn draw_tile(&mut self, slot: usize, kind: u16) -> Result<(), DisplayError> {
        self.active_slot = slot;
        draw_tile(self, slot, kind)
    }
    fn draw_sprite(
        &mut self,
        slot: usize,
        sprite: u16,
        x: u16,
        y: u16,
    ) -> Result<(), DisplayError> {
        let sprite = self
            .state
            .atmosphere_catalog
            .sprites
            .get(&sprite)
            .ok_or(DisplayError::Source)?
            .clone();
        if !sprite.present {
            return Ok(());
        }
        let producers = &self.state.atmospheric_producers;
        let producer_ids = &self.state.producer_ids;
        let media = self.media;
        let inputs = &mut self.inputs;
        let controls = &self.state.controls;
        let effective = &self.state.effective;
        super::objects::draw_with_publication(
            &mut self.state.objects,
            &mut SpriteGeometry(&sprite),
            (x, y),
            super::objects::Clip::None,
            Ok,
            |objects, index, entry, x, y, tile| {
                let piece = sprite.parts.get(index).ok_or(DisplayError::Source)?;
                let producer = producers.slots[slot].ok_or(DisplayError::Incomplete)?;
                let context = inputs.object_context(controls, effective)?;
                let native = object_geometry(objects, entry, x, y)?;
                let object = semantic_object(
                    media,
                    producer,
                    piece.variants.clone(),
                    native,
                    tile,
                    context,
                )?;
                record(objects, producer_ids, object)?;
                Ok(())
            },
        )
    }
    fn probe(&mut self, _: &'static str) {}
}
struct SpriteGeometry<'a>(&'a AtmosphericSprite);
impl super::objects::Geometry for SpriteGeometry<'_> {
    type Error = DisplayError;
    fn count(&mut self) -> Result<u16, DisplayError> {
        u16::try_from(self.0.parts.len()).map_err(|_| DisplayError::Geometry)
    }
    fn x(&mut self, record: usize) -> Result<u16, DisplayError> {
        self.0
            .parts
            .get(record)
            .map(|part| part.x)
            .ok_or(DisplayError::Source)
    }
    fn y(&mut self, record: usize) -> Result<u8, DisplayError> {
        self.0
            .parts
            .get(record)
            .map(|part| part.y)
            .ok_or(DisplayError::Source)
    }
    fn tile(&mut self, record: usize) -> Result<u16, DisplayError> {
        self.0
            .parts
            .get(record)
            .map(|part| part.native_word)
            .ok_or(DisplayError::Source)
    }
}
