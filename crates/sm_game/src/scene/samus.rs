//! Ordered Samus spritemap publication over shared native/owned object staging.
#![forbid(unsafe_code)]

use super::{
    objects,
    records::{
        ObjectContext, ObjectSize, ObjectVariants, ProducedObject, ProducerObjectId, Revision,
        SceneMedia, SceneObject, ScreenPosition,
    },
    state::{DisplayError, DisplayPhase, DisplayState},
};
use std::{collections::BTreeMap, sync::Arc};

pub trait Host: objects::Host {
    type Map: objects::Geometry<Error = Self::Error>;
    /// Separate loaded-content capture for each full, widened table-word query.
    fn table_word(&mut self, index: u16) -> Result<u16, Self::Error>;
    /// Retains warning/failure-flag order before capturing the map allocation.
    fn sprite_map(&mut self, cursor: u16) -> Result<Self::Map, Self::Error>;
    fn published(
        &mut self,
        part: usize,
        entry: usize,
        x: u16,
        y: u16,
        word: u16,
    ) -> Result<(), Self::Error>;
}

pub fn draw<H: Host>(host: &mut H, index: u16, position: (u16, u16)) -> Result<(), H::Error> {
    // Index zero is a real table query; only the first zero table result skips.
    if host.table_word(index)? != 0 {
        let cursor = host.table_word(index)?;
        let mut map = host.sprite_map(cursor)?;
        objects::draw_with_publication(
            host,
            &mut map,
            position,
            objects::Clip::None,
            Ok,
            |host, part, entry, x, y, word| host.published(part, entry, x, y, word),
        )?;
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SamusPart {
    pub x: u16,
    pub y: u8,
    pub native_word: u16,
    pub variants: Arc<ObjectVariants>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SamusMap {
    /// Supplied native count, queried after the original captured OAM cursor.
    pub count: u16,
    pub parts: Arc<[SamusPart]>,
}
impl objects::Geometry for SamusMap {
    type Error = DisplayError;
    fn count(&mut self) -> Result<u16, DisplayError> {
        Ok(self.count)
    }
    fn x(&mut self, record: usize) -> Result<u16, DisplayError> {
        self.parts
            .get(record)
            .map(|part| part.x)
            .ok_or(DisplayError::Source)
    }
    fn y(&mut self, record: usize) -> Result<u8, DisplayError> {
        self.parts
            .get(record)
            .map(|part| part.y)
            .ok_or(DisplayError::Source)
    }
    fn tile(&mut self, record: usize) -> Result<u16, DisplayError> {
        self.parts
            .get(record)
            .map(|part| part.native_word)
            .ok_or(DisplayError::Source)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SamusCatalog {
    pub revision: Revision,
    /// Full word indices; no bank wrapping, inferred entry or zero fallback.
    pub pointers: Arc<[u16]>,
    pub maps: BTreeMap<u16, SamusMap>,
}
impl SamusCatalog {
    pub fn validate(&self, media: &SceneMedia) -> Result<(), DisplayError> {
        if self.revision != media.revision() {
            return Err(DisplayError::Revision);
        }
        for pointer in self
            .pointers
            .iter()
            .copied()
            .filter(|pointer| *pointer != 0)
        {
            if !self.maps.contains_key(&pointer) {
                return Err(DisplayError::Incomplete);
            }
        }
        for map in self.maps.values() {
            if usize::from(map.count) > map.parts.len() {
                return Err(DisplayError::Incomplete);
            }
            for part in map.parts.iter() {
                part.variants
                    .validate(media, part.native_word & 255)
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

/// These queries are supplied by their actual G/content/diagnostic owners.
pub trait Inputs {
    fn content(&mut self, state: &DisplayState) -> Result<Arc<SamusCatalog>, DisplayError>;
    /// Original warning followed by the failure flag only on normal return.
    /// A returning hook may change the selected content before map capture.
    fn invalid_map(&mut self, state: &mut DisplayState, cursor: u16) -> Result<(), DisplayError>;
    fn object_context(
        &mut self,
        controls: &super::state::DisplayControls,
        effective: &super::state::EffectiveDisplayDescription,
    ) -> Result<ObjectContext, DisplayError>;
}

pub fn draw_owned(
    state: &mut DisplayState,
    media: &SceneMedia,
    producer: ProducerObjectId,
    index: u16,
    position: (u16, u16),
    inputs: &mut impl Inputs,
) -> Result<(), DisplayError> {
    if state.phase == DisplayPhase::Failed {
        return Err(DisplayError::Poisoned);
    }
    if state.phase != DisplayPhase::Producing {
        return Err(DisplayError::Phase);
    }
    let mut guard = super::delivery::FaultGuard::new(state);
    let result = draw(
        &mut Owned {
            state: guard.state(),
            media,
            producer,
            inputs,
            active: None,
        },
        index,
        position,
    );
    if result.is_ok() {
        guard.disarm();
    }
    result
}

struct Owned<'a, I> {
    state: &'a mut DisplayState,
    media: &'a SceneMedia,
    producer: ProducerObjectId,
    inputs: &'a mut I,
    active: Option<SamusMap>,
}
impl<I: Inputs> objects::Host for Owned<'_, I> {
    type Error = DisplayError;
    fn read_native_word(&mut self, at: usize) -> Result<u16, DisplayError> {
        objects::Host::read_native_word(&mut self.state.objects, at)
    }
    fn read_little_word(&mut self, at: usize) -> Result<u16, DisplayError> {
        objects::Host::read_little_word(&mut self.state.objects, at)
    }
    fn write_byte(&mut self, at: usize, value: u8) -> Result<(), DisplayError> {
        objects::Host::write_byte(&mut self.state.objects, at, value)
    }
    fn write_native_slice(&mut self, at: usize, value: u16) -> Result<(), DisplayError> {
        objects::Host::write_native_slice(&mut self.state.objects, at, value)
    }
    fn write_native_bytes(&mut self, at: usize, value: u16) -> Result<(), DisplayError> {
        objects::Host::write_native_bytes(&mut self.state.objects, at, value)
    }
    fn started(&mut self, clip: objects::Clip) -> Result<(), DisplayError> {
        objects::Host::started(&mut self.state.objects, clip)
    }
}
impl<I: Inputs> Host for Owned<'_, I> {
    type Map = SamusMap;
    fn table_word(&mut self, index: u16) -> Result<u16, DisplayError> {
        let content = self.inputs.content(self.state)?;
        content
            .pointers
            .get(usize::from(index))
            .copied()
            .ok_or(DisplayError::Source)
    }
    fn sprite_map(&mut self, cursor: u16) -> Result<SamusMap, DisplayError> {
        if cursor & 0x8000 == 0 {
            self.inputs.invalid_map(self.state, cursor)?;
        }
        let content = self.inputs.content(self.state)?;
        let map = content
            .maps
            .get(&cursor)
            .ok_or(DisplayError::Source)?
            .clone();
        self.active = Some(map.clone());
        Ok(map)
    }
    fn published(
        &mut self,
        index: usize,
        entry: usize,
        x: u16,
        y: u16,
        word: u16,
    ) -> Result<(), DisplayError> {
        let part = self
            .active
            .as_ref()
            .and_then(|map| map.parts.get(index))
            .ok_or(DisplayError::Source)?;
        let context = self
            .inputs
            .object_context(&self.state.controls, &self.state.effective)?;
        // Native full/odd aliases complete their original writes before rejection.
        if entry >= 512 || entry & 3 != 0 {
            return Err(DisplayError::Alias { offset: entry });
        }
        let bits = self.state.objects.byte(512 + entry / 16)? >> (2 * (entry / 4 & 3));
        let size = if bits & 2 != 0 {
            ObjectSize::Large
        } else {
            ObjectSize::Small
        };
        let binding = part
            .variants
            .binding(context.selection.size_set, size)
            .map_err(|_| DisplayError::Incomplete)?;
        let definition = binding
            .definition(self.media)
            .map_err(|_| DisplayError::Scene)?;
        let record = ProducedObject {
            producer: self.producer,
            variants: part.variants.clone(),
            context,
            native_word: word,
            object: SceneObject {
                order: (entry / 4) as u16,
                visual: binding.visual,
                frame: binding.frame,
                part: binding.part,
                atlas: definition.atlas,
                material: definition.material,
                tile: word & 255,
                position: ScreenPosition {
                    x: x as i16,
                    y: y as i16,
                },
                size,
                palette: ((word >> 9) & 7) as u8,
                priority: ((word >> 12) & 3) as u8,
                flip_horizontal: word & 0x4000 != 0,
                flip_vertical: word & 0x8000 != 0,
            },
        }
        .for_line(self.media, context, &self.state.objects.bytes)
        .map_err(|_| DisplayError::Scene)?;
        self.state
            .producer_ids
            .validate_id(record.producer)
            .map_err(|_| DisplayError::Identity)?;
        self.state
            .objects
            .produced
            .retain(|old| old.object.order != record.object.order);
        self.state.objects.produced.push(record);
        self.state
            .objects
            .produced
            .sort_by_key(|record| record.object.order);
        Ok(())
    }
}
