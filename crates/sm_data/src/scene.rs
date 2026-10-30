//! Immutable committed game-facing image descriptions and resident authored media.
#![forbid(unsafe_code)]

use crate::raster as r;
use std::{collections::BTreeMap, sync::Arc};

pub use crate::{AssetId, AssetRef, Revision};
/// A live producer's identity, supplied at publication rather than derived from
/// authored media IDs, object order or rendered bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProducerObjectId(pub u64);
/// The one display-wide lifecycle namespace, supplied with the actual state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DisplayProducerIds {
    pub next: ProducerObjectId,
}
impl DisplayProducerIds {
    pub fn validate(&self) -> Result<(), SceneError> {
        if self.next.0 == 0 {
            return Err(SceneError::Identity);
        }
        Ok(())
    }
    pub fn validate_id(&self, id: ProducerObjectId) -> Result<(), SceneError> {
        self.validate()?;
        if id.0 == 0 || id.0 >= self.next.0 {
            return Err(SceneError::Identity);
        }
        Ok(())
    }
    pub fn allocate(&mut self) -> Result<ProducerObjectId, SceneError> {
        self.allocate_with(|_| Ok(()))
    }
    /// Check the next value before the original store; commit it after success.
    pub fn allocate_with(
        &mut self,
        store: impl FnOnce(ProducerObjectId) -> Result<(), SceneError>,
    ) -> Result<ProducerObjectId, SceneError> {
        let next = self.next.0.checked_add(1).ok_or(SceneError::Identity)?;
        self.validate()?;
        let id = self.next;
        store(id)?;
        self.next = ProducerObjectId(next);
        Ok(id)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct MediaRef {
    pub asset: AssetRef,
    pub incarnation: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SceneError {
    Identity,
    Revision,
    MissingAsset(AssetId),
    MissingIncarnation(MediaRef),
    AssetKind(AssetId),
    DuplicateAsset(AssetId),
    DuplicateIncarnation(MediaRef),
    Geometry,
    Unsupported,
    ExtendedBackground,
    EnabledMosaic(usize),
    Incomplete,
    Order,
    Definition(AssetId),
    InconsistentMedia(MediaRef),
    Raster(r::RasterFault),
}
impl From<r::RasterFault> for SceneError {
    fn from(fault: r::RasterFault) -> Self {
        Self::Raster(fault)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScreenPosition {
    pub x: i16,
    pub y: i16,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SceneView {
    pub room: AssetRef,
    pub camera: ScreenPosition,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorInterpretation {
    Bgr15Native,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SceneQuantization {
    Native,
    Reference,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SceneGeometry {
    pub width: u16,
    pub height: u16,
    pub crop_origin: ScreenPosition,
    pub interlaced: bool,
    pub even_frame: bool,
    pub overscan: bool,
    pub color: ColorInterpretation,
    pub quantization: SceneQuantization,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DisplayStatus {
    pub range_overflow: bool,
    pub time_overflow: bool,
    pub has_objects: bool,
    pub even_frame: bool,
    pub interlaced: bool,
    pub overscan: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObjectOverflow {
    pub range: bool,
    pub time: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TileEncoding {
    Planar2,
    Planar4,
    Affine8,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TileAtlas {
    pub reference: AssetRef,
    pub encoding: TileEncoding,
    pub tile_count: u16,
    /// Planar words in tile/plane-pair/row order, or affine 8-bit texels as words.
    pub data: Arc<[u16]>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TileCell {
    pub tile: u16,
    pub palette: u8,
    pub priority: bool,
    pub flip_horizontal: bool,
    pub flip_vertical: bool,
}
impl TileCell {
    pub fn planar_word(self) -> u16 {
        self.tile
            | (u16::from(self.palette) << 10)
            | (u16::from(self.priority) << 13)
            | (u16::from(self.flip_horizontal) << 14)
            | (u16::from(self.flip_vertical) << 15)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tilemap {
    pub reference: AssetRef,
    pub atlas: AssetRef,
    pub palette: AssetRef,
    pub width: u16,
    pub height: u16,
    pub cells: Arc<[TileCell]>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Palette {
    pub reference: AssetRef,
    pub colors: [u16; 256],
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObjectSize {
    Small,
    Large,
}
/// Authored identity supplied for one effective object geometry variant.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObjectPartBinding {
    pub visual: AssetRef,
    pub frame: u16,
    pub part: u16,
}
impl ObjectPartBinding {
    pub fn definition<'a>(&self, media: &'a SceneMedia) -> Result<&'a VisualPart, SceneError> {
        match media.asset(self.visual)? {
            SceneAsset::Visual(visual) => visual
                .frames
                .get(usize::from(self.frame))
                .and_then(|frame| frame.parts.get(usize::from(self.part)))
                .ok_or(SceneError::Definition(self.visual.id)),
            _ => Err(SceneError::AssetKind(self.visual.id)),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObjectSizeVariants {
    pub small: ObjectPartBinding,
    pub large: ObjectPartBinding,
}
/// Every supported size set and both size-bit variants are explicitly supplied.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObjectVariants {
    pub size_sets: BTreeMap<u8, ObjectSizeVariants>,
}
impl ObjectVariants {
    pub fn binding(&self, set: u8, size: ObjectSize) -> Result<ObjectPartBinding, SceneError> {
        let variants = self.size_sets.get(&set).ok_or(SceneError::Incomplete)?;
        Ok(match size {
            ObjectSize::Small => variants.small,
            ObjectSize::Large => variants.large,
        })
    }
    pub fn validate(&self, media: &SceneMedia, tile: u16) -> Result<(), SceneError> {
        if self.size_sets.is_empty() {
            return Err(SceneError::Incomplete);
        }
        for (&set, variants) in &self.size_sets {
            if set > 7 {
                return Err(SceneError::Geometry);
            }
            for (binding, size) in [
                (variants.small, ObjectSize::Small),
                (variants.large, ObjectSize::Large),
            ] {
                let part = binding.definition(media)?;
                if part.tile != tile || part.size != size {
                    return Err(SceneError::Definition(binding.visual.id));
                }
            }
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VisualPart {
    pub atlas: AssetRef,
    pub material: AssetRef,
    pub tile: u16,
    pub offset: ScreenPosition,
    pub size: ObjectSize,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VisualFrame {
    pub parts: Arc<[VisualPart]>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Visual {
    pub reference: AssetRef,
    pub frames: Arc<[VisualFrame]>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Font {
    pub reference: AssetRef,
    pub atlas: AssetRef,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SceneAsset {
    Atlas(TileAtlas),
    Map(Tilemap),
    Palette(Box<Palette>),
    Visual(Visual),
    Font(Font),
    Room(AssetRef),
}
impl SceneAsset {
    pub fn reference(&self) -> AssetRef {
        match self {
            Self::Atlas(v) => v.reference,
            Self::Map(v) => v.reference,
            Self::Palette(v) => v.reference,
            Self::Visual(v) => v.reference,
            Self::Font(v) => v.reference,
            Self::Room(v) => *v,
        }
    }
}

/// G supplies the already committed palette incarnation, including effects from
/// earlier lines. The authored reference survives palette-program changes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaletteIncarnation {
    pub reference: MediaRef,
    pub colors: [u16; 256],
}
/// Content-word placement for the existing numeric core; never a register address.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AtlasBinding {
    pub atlas: AssetRef,
    pub first_word: u16,
    /// Already settled tile words for this image incarnation. The authored atlas
    /// supplies layout and source definitions; uploads may change its resident
    /// pixels without changing the stable authored identity. No initial fallback.
    pub resident_data: Arc<[u16]>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MapBinding {
    pub map: AssetRef,
    pub first_word: u16,
}
/// Compiler/compatibility input. Scene's public planes and objects retain their
/// authored meanings; these word placements are private adaptation metadata.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompiledImage {
    pub reference: MediaRef,
    pub palette: MediaRef,
    /// Explicit indexed words and required native aliases; no fabricated tail.
    pub words: Arc<[u16]>,
    pub atlases: Arc<[AtlasBinding]>,
    pub maps: Arc<[MapBinding]>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SceneMedia {
    revision: Revision,
    assets: BTreeMap<AssetId, SceneAsset>,
    palettes: BTreeMap<MediaRef, PaletteIncarnation>,
    images: BTreeMap<MediaRef, CompiledImage>,
}
impl SceneMedia {
    pub fn new(
        revision: Revision,
        assets: Vec<SceneAsset>,
        palettes: Vec<PaletteIncarnation>,
        images: Vec<CompiledImage>,
    ) -> Result<Self, SceneError> {
        let mut result = Self {
            revision,
            assets: BTreeMap::new(),
            palettes: BTreeMap::new(),
            images: BTreeMap::new(),
        };
        for asset in assets {
            let id = asset.reference().id;
            if result.assets.insert(id, asset).is_some() {
                return Err(SceneError::DuplicateAsset(id));
            }
        }
        for palette in palettes {
            let key = palette.reference;
            if result.palettes.insert(key, palette).is_some() {
                return Err(SceneError::DuplicateIncarnation(key));
            }
        }
        for image in images {
            let key = image.reference;
            if result.images.insert(key, image).is_some() {
                return Err(SceneError::DuplicateIncarnation(key));
            }
        }
        result.validate()?;
        Ok(result)
    }
    pub fn revision(&self) -> Revision {
        self.revision
    }
    pub fn assets(&self) -> impl Iterator<Item = &SceneAsset> {
        self.assets.values()
    }
    pub fn palette_incarnations(&self) -> impl Iterator<Item = &PaletteIncarnation> {
        self.palettes.values()
    }
    pub fn compiled_images(&self) -> impl Iterator<Item = &CompiledImage> {
        self.images.values()
    }
    pub fn asset(&self, reference: AssetRef) -> Result<&SceneAsset, SceneError> {
        self.check_reference(reference)?;
        self.assets
            .get(&reference.id)
            .ok_or(SceneError::MissingAsset(reference.id))
    }
    pub(crate) fn check_reference(&self, reference: AssetRef) -> Result<(), SceneError> {
        if reference.id.0 == 0 {
            return Err(SceneError::Identity);
        }
        if reference.revision != self.revision {
            return Err(SceneError::Revision);
        }
        Ok(())
    }
    pub fn atlas(&self, reference: AssetRef) -> Result<&TileAtlas, SceneError> {
        match self.asset(reference)? {
            SceneAsset::Atlas(v) => Ok(v),
            _ => Err(SceneError::AssetKind(reference.id)),
        }
    }
    pub fn map(&self, reference: AssetRef) -> Result<&Tilemap, SceneError> {
        match self.asset(reference)? {
            SceneAsset::Map(v) => Ok(v),
            _ => Err(SceneError::AssetKind(reference.id)),
        }
    }
    pub fn palette(&self, reference: MediaRef) -> Result<&PaletteIncarnation, SceneError> {
        if !matches!(self.asset(reference.asset)?, SceneAsset::Palette(_)) {
            return Err(SceneError::AssetKind(reference.asset.id));
        }
        self.palettes
            .get(&reference)
            .ok_or(SceneError::MissingIncarnation(reference))
    }
    pub fn image(&self, reference: MediaRef) -> Result<&CompiledImage, SceneError> {
        self.asset(reference.asset)?;
        self.images
            .get(&reference)
            .ok_or(SceneError::MissingIncarnation(reference))
    }
    pub fn validate(&self) -> Result<(), SceneError> {
        if self.revision.0 == [0; 32] {
            return Err(SceneError::Revision);
        }
        for asset in self.assets.values() {
            let reference = asset.reference();
            self.check_reference(reference)?;
            match asset {
                SceneAsset::Atlas(atlas) => {
                    let words = match atlas.encoding {
                        TileEncoding::Planar2 => 8,
                        TileEncoding::Planar4 => 16,
                        TileEncoding::Affine8 => 64,
                    };
                    if atlas.tile_count == 0
                        || atlas.tile_count > 1024
                        || atlas.data.len() != usize::from(atlas.tile_count) * words
                        || atlas.encoding == TileEncoding::Affine8
                            && atlas.data.iter().any(|value| *value > 255)
                    {
                        return Err(SceneError::Definition(reference.id));
                    }
                }
                SceneAsset::Map(map) => {
                    let atlas = self.atlas(map.atlas)?;
                    if !matches!(self.asset(map.palette)?, SceneAsset::Palette(_)) {
                        return Err(SceneError::AssetKind(map.palette.id));
                    }
                    let valid = match atlas.encoding {
                        TileEncoding::Affine8 => map.width == 128 && map.height == 128,
                        _ => {
                            (map.width == 32 || map.width == 64)
                                && (map.height == 32 || map.height == 64)
                        }
                    };
                    if !valid || map.cells.len() != usize::from(map.width) * usize::from(map.height)
                    {
                        return Err(SceneError::Definition(reference.id));
                    }
                    for cell in map.cells.iter() {
                        validate_cell(*cell, atlas)?;
                    }
                }
                SceneAsset::Palette(_) => {}
                SceneAsset::Visual(visual) => {
                    if visual.frames.is_empty() {
                        return Err(SceneError::Definition(reference.id));
                    }
                    for frame in visual.frames.iter() {
                        if frame.parts.is_empty() {
                            return Err(SceneError::Definition(reference.id));
                        }
                        for part in frame.parts.iter() {
                            let atlas = self.atlas(part.atlas)?;
                            if atlas.encoding != TileEncoding::Planar4
                                || part.tile >= atlas.tile_count
                                || !matches!(self.asset(part.material)?, SceneAsset::Palette(_))
                            {
                                return Err(SceneError::Definition(reference.id));
                            }
                        }
                    }
                }
                SceneAsset::Font(font) => {
                    self.atlas(font.atlas)?;
                }
                SceneAsset::Room(_) => {}
            }
        }
        for palette in self.palettes.values() {
            if !matches!(self.asset(palette.reference.asset)?, SceneAsset::Palette(_)) {
                return Err(SceneError::Definition(palette.reference.asset.id));
            }
        }
        for image in self.images.values() {
            self.asset(image.reference.asset)?;
            let palette = self.palette(image.palette)?;
            if image.words.len() != r::MAX_INDEXED_WORDS {
                return Err(SceneError::Incomplete);
            }
            // The explicit native indexed tail includes the first26 palette
            // words. A later palette incarnation must not retain stale aliases.
            if image.words[0x8005..0x801f] != palette.colors[..26] {
                return Err(SceneError::InconsistentMedia(image.reference));
            }
            for (index, binding) in image.atlases.iter().enumerate() {
                let atlas = self.atlas(binding.atlas)?;
                if image.atlases[..index]
                    .iter()
                    .any(|old| old.atlas == binding.atlas)
                {
                    return Err(SceneError::Definition(binding.atlas.id));
                }
                let words_per_tile = match atlas.encoding {
                    TileEncoding::Planar2 => 8,
                    TileEncoding::Planar4 => 16,
                    TileEncoding::Affine8 => 64,
                };
                if binding.resident_data.len() != usize::from(atlas.tile_count) * words_per_tile
                    || atlas.encoding == TileEncoding::Affine8
                        && binding.resident_data.iter().any(|value| *value > 255)
                {
                    return Err(SceneError::Definition(binding.atlas.id));
                }
                match atlas.encoding {
                    TileEncoding::Affine8 => {
                        if binding.first_word != 0 {
                            return Err(SceneError::Unsupported);
                        }
                        for (offset, value) in binding.resident_data.iter().enumerate() {
                            if image.words.get(offset).map(|word| word >> 8) != Some(*value) {
                                return Err(SceneError::InconsistentMedia(image.reference));
                            }
                        }
                    }
                    encoding => {
                        let depth = if encoding == TileEncoding::Planar2 {
                            2
                        } else {
                            4
                        };
                        for tile in 0..usize::from(atlas.tile_count) {
                            for pair in 0..depth / 2 {
                                for row in 0..8 {
                                    let base =
                                        (usize::from(binding.first_word) + tile * 4 * depth + row)
                                            & 0x7fff;
                                    let expected =
                                        binding.resident_data[tile * 4 * depth + pair * 8 + row];
                                    if image.words.get(base + pair * 8).copied() != Some(expected) {
                                        return Err(SceneError::InconsistentMedia(image.reference));
                                    }
                                }
                            }
                        }
                    }
                }
            }
            for (index, binding) in image.maps.iter().enumerate() {
                self.map(binding.map)?;
                if image.maps[..index].iter().any(|old| old.map == binding.map) {
                    return Err(SceneError::Definition(binding.map.id));
                }
            }
        }
        Ok(())
    }
}
fn validate_cell(cell: TileCell, atlas: &TileAtlas) -> Result<(), SceneError> {
    if cell.tile >= atlas.tile_count || cell.tile > 1023 || cell.palette > 7 {
        return Err(SceneError::Definition(atlas.reference.id));
    }
    if atlas.encoding == TileEncoding::Affine8
        && (cell.tile > 255
            || cell.palette != 0
            || cell.priority
            || cell.flip_horizontal
            || cell.flip_vertical)
    {
        return Err(SceneError::Unsupported);
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaneRole {
    Main,
    Secondary,
    HudFx,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScenePlane {
    pub role: PlaneRole,
    pub map: AssetRef,
    pub atlas: AssetRef,
    pub palette: MediaRef,
    pub width: u16,
    pub height: u16,
    pub cells: Arc<[TileCell]>,
    pub scroll: ScreenPosition,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SceneObject {
    pub order: u16,
    pub visual: AssetRef,
    pub frame: u16,
    pub part: u16,
    pub atlas: AssetRef,
    pub material: AssetRef,
    pub tile: u16,
    /// Supplied quantized source position. Native policy consumes low nine X
    /// bits and low eight Y bits; full supplied words are retained in Scene.
    pub position: ScreenPosition,
    pub size: ObjectSize,
    pub palette: u8,
    pub priority: u8,
    pub flip_horizontal: bool,
    pub flip_vertical: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProducedObject {
    pub producer: ProducerObjectId,
    pub object: SceneObject,
    /// Retained through delivery and history for later effective OBSEL changes.
    pub variants: Arc<ObjectVariants>,
    pub context: ObjectContext,
    pub native_word: u16,
}
impl ProducedObject {
    /// Bind declared authored IDs to this line's actual effective geometry.
    /// Numeric OAM position, tile, size-bit and flags are never reconstructed IDs.
    pub fn for_line(
        &self,
        media: &SceneMedia,
        context: ObjectContext,
        objects: &[u8; 544],
    ) -> Result<Self, SceneError> {
        if self.object.order >= 128 {
            return Err(SceneError::Geometry);
        }
        let entry = usize::from(self.object.order) * 4;
        let bits = objects[512 + entry / 16] >> (2 * ((entry / 4) & 3));
        let mut selected = self.clone();
        selected.context = context;
        selected.object.position.x = ((self.object.position.x as u16 & !0x01ff)
            | u16::from(objects[entry])
            | (u16::from(bits & 1) << 8)) as i16;
        selected.object.position.y =
            ((self.object.position.y as u16 & !0x00ff) | u16::from(objects[entry + 1])) as i16;
        selected.object.size = if bits & 2 != 0 {
            ObjectSize::Large
        } else {
            ObjectSize::Small
        };
        selected.native_word = u16::from_le_bytes([objects[entry + 2], objects[entry + 3]]);
        selected.object.tile = selected.native_word & 255;
        selected.object.palette = ((selected.native_word >> 9) & 7) as u8;
        selected.object.priority = ((selected.native_word >> 12) & 3) as u8;
        selected.object.flip_horizontal = selected.native_word & 0x4000 != 0;
        selected.object.flip_vertical = selected.native_word & 0x8000 != 0;
        let binding = self
            .variants
            .binding(context.selection.size_set, selected.object.size)?;
        let part = binding.definition(media)?;
        selected.object.visual = binding.visual;
        selected.object.frame = binding.frame;
        selected.object.part = binding.part;
        selected.object.atlas = part.atlas;
        selected.object.material = part.material;
        selected.validate(media)?;
        Ok(selected)
    }
    pub fn validate(&self, media: &SceneMedia) -> Result<(), SceneError> {
        if self.producer.0 == 0 {
            return Err(SceneError::Identity);
        }
        if self.object.order >= 128 || self.object.palette > 7 || self.object.priority > 3 {
            return Err(SceneError::Geometry);
        }
        let visual = match media.asset(self.object.visual)? {
            SceneAsset::Visual(visual) => visual,
            _ => return Err(SceneError::AssetKind(self.object.visual.id)),
        };
        let part = visual
            .frames
            .get(usize::from(self.object.frame))
            .and_then(|frame| frame.parts.get(usize::from(self.object.part)))
            .ok_or(SceneError::Definition(self.object.visual.id))?;
        if part.atlas != self.object.atlas
            || part.material != self.object.material
            || part.tile != self.object.tile
            || part.size != self.object.size
        {
            return Err(SceneError::Definition(self.object.visual.id));
        }
        let binding = self
            .variants
            .binding(self.context.selection.size_set, self.object.size)?;
        if binding.visual != self.object.visual
            || binding.frame != self.object.frame
            || binding.part != self.object.part
        {
            return Err(SceneError::Definition(self.object.visual.id));
        }
        if self.object.tile != self.native_word & 255
            || self.object.atlas
                != self.context.selection.atlases[usize::from(self.native_word >> 8 & 1)]
            || self.object.palette != ((self.native_word >> 9) & 7) as u8
            || self.object.priority != ((self.native_word >> 12) & 3) as u8
            || self.object.flip_horizontal != (self.native_word & 0x4000 != 0)
            || self.object.flip_vertical != (self.native_word & 0x8000 != 0)
        {
            return Err(SceneError::Definition(self.object.visual.id));
        }
        validate_object_source(&self.object, self.context, media)?;
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObjectSelection {
    pub atlases: [AssetRef; 2],
    pub size_set: u8,
    pub first_order: Option<u8>,
    pub interlaced: bool,
    pub even_frame: bool,
}
/// Actual supplied atlas/geometry and color source at publication or a visible line.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObjectContext {
    pub selection: ObjectSelection,
    pub palette: MediaRef,
}
fn validate_object_source(
    object: &SceneObject,
    context: ObjectContext,
    media: &SceneMedia,
) -> Result<(), SceneError> {
    let selection = context.selection;
    media.palette(context.palette)?;
    let sizes = [
        [8u16, 16],
        [8, 32],
        [8, 64],
        [16, 32],
        [16, 64],
        [32, 64],
        [16, 32],
        [16, 32],
    ];
    let sizes = sizes
        .get(usize::from(selection.size_set))
        .ok_or(SceneError::Geometry)?;
    let size = sizes[usize::from(object.size == ObjectSize::Large)];
    let atlas = media.atlas(object.atlas)?;
    if atlas.encoding != TileEncoding::Planar4
        || !selection.atlases.contains(&object.atlas)
        || object.tile > 255
        || object.material != context.palette.asset
        || !matches!(media.asset(object.material)?, SceneAsset::Palette(_))
    {
        return Err(SceneError::Definition(object.visual.id));
    }
    // Native object rows advance by sixteen tiles; columns wrap within a row.
    for row in 0..size / 8 {
        for column in 0..size / 8 {
            let tile = ((object.tile >> 4) + row) << 4 | ((object.tile + column) & 15);
            if tile >= atlas.tile_count {
                return Err(SceneError::Definition(object.visual.id));
            }
        }
    }
    Ok(())
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowExtent {
    pub first: u8,
    pub last: u8,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowCombination {
    Union,
    Intersection,
    Exclusive,
    Equal,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LayerWindow {
    pub first_enabled: bool,
    pub first_inverted: bool,
    pub second_enabled: bool,
    pub second_inverted: bool,
    pub combination: WindowCombination,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SceneWindows {
    pub extents: [WindowExtent; 2],
    pub layers: [LayerWindow; 6],
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScreenRouting {
    pub visible: [bool; 5],
    pub masked: [bool; 5],
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowCondition {
    Never,
    Inside,
    Outside,
    Always,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SceneColor {
    pub clip: WindowCondition,
    pub prevent_math: WindowCondition,
    pub enabled: [bool; 6],
    pub use_subscreen: bool,
    pub subtract: bool,
    pub half: bool,
    pub fixed: [u8; 3],
    pub brightness: u8,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SceneAffine {
    pub matrix: [i16; 4],
    pub center: ScreenPosition,
    pub scroll: ScreenPosition,
    pub flip_horizontal: bool,
    pub flip_vertical: bool,
    pub bounded_field: bool,
    pub fill_outside: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScenePixelation {
    pub enabled: [bool; 3],
    pub size: u8,
    pub origin: u8,
    pub modulo: [u8; 256],
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SceneMode {
    TilePlanes,
    AffinePlane { extended: bool },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SignedObjectPixel {
    pub x: i32,
    pub value: u16,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SceneLine {
    pub media: MediaRef,
    pub palette: MediaRef,
    pub mode: SceneMode,
    pub blank: bool,
    pub planes: [ScenePlane; 3],
    pub objects: Arc<[SceneObject]>,
    pub object_selection: ObjectSelection,
    pub windows: SceneWindows,
    pub routing: [ScreenRouting; 2],
    pub affine: SceneAffine,
    pub pixelation: ScenePixelation,
    pub color: SceneColor,
    pub object_aliases: Arc<[SignedObjectPixel]>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SceneDescription {
    pub tick: u64,
    pub gameplay: [u8; 32],
    pub reference: [u8; 32],
    pub revision: Revision,
    pub view: SceneView,
    pub geometry: SceneGeometry,
    pub resident: Arc<[AssetRef]>,
    pub lines: Arc<[SceneLine]>,
    pub status: DisplayStatus,
    /// Supplied start-of-history status; opening/redraw never guesses a reset.
    pub initial_object_overflow: ObjectOverflow,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Scene {
    description: Arc<SceneDescription>,
    media: Arc<SceneMedia>,
    producer_history: Arc<[Arc<[ProducedObject]>]>,
}
impl Scene {
    /// Compatibility construction without assigned producer identities.
    pub fn new(description: SceneDescription, media: Arc<SceneMedia>) -> Result<Self, SceneError> {
        let scene = Self {
            description: Arc::new(description),
            media,
            producer_history: Arc::from([]),
        };
        scene.validate(&scene.media)?;
        Ok(scene)
    }
    pub fn with_producer_history(
        mut self,
        history: Arc<[Arc<[ProducedObject]>]>,
    ) -> Result<Self, SceneError> {
        if history.len() != self.lines().len() {
            return Err(SceneError::Incomplete);
        }
        self.producer_history = history;
        self.validate_producer_history()?;
        Ok(self)
    }
    pub fn producer_history(&self) -> &[Arc<[ProducedObject]>] {
        &self.producer_history
    }
    pub fn validate_producer_ids(&self, ids: &DisplayProducerIds) -> Result<(), SceneError> {
        ids.validate()?;
        for records in self.producer_history.iter() {
            for record in records.iter() {
                ids.validate_id(record.producer)?;
            }
        }
        Ok(())
    }
    pub fn tick(&self) -> u64 {
        self.description.tick
    }
    pub fn gameplay(&self) -> [u8; 32] {
        self.description.gameplay
    }
    pub fn reference(&self) -> [u8; 32] {
        self.description.reference
    }
    pub fn revision(&self) -> Revision {
        self.description.revision
    }
    pub fn description(&self) -> &SceneDescription {
        &self.description
    }
    pub fn lines(&self) -> &[SceneLine] {
        &self.description.lines
    }
    pub fn media(&self) -> &Arc<SceneMedia> {
        &self.media
    }
    pub fn validate(&self, media: &SceneMedia) -> Result<(), SceneError> {
        media.validate()?;
        if self.description.gameplay == [0; 32] || self.description.reference == [0; 32] {
            return Err(SceneError::Identity);
        }
        if self.revision() != media.revision() || *self.media != *media {
            return Err(SceneError::Revision);
        }
        let geometry = self.description.geometry;
        if geometry.width != 256
            || geometry.height != 224
            || geometry.crop_origin != (ScreenPosition { x: 0, y: 0 })
            || geometry.overscan
        {
            return Err(SceneError::Geometry);
        }
        if geometry.quantization != SceneQuantization::Native {
            return Err(SceneError::Unsupported);
        }
        if self.lines().len() != usize::from(geometry.height) {
            return Err(SceneError::Incomplete);
        }
        if self.description.status.even_frame != geometry.even_frame
            || self.description.status.interlaced != geometry.interlaced
            || self.description.status.overscan != geometry.overscan
        {
            return Err(SceneError::Geometry);
        }
        if !matches!(
            media.asset(self.description.view.room)?,
            SceneAsset::Room(_)
        ) {
            return Err(SceneError::AssetKind(self.description.view.room.id));
        }
        for (index, reference) in self.description.resident.iter().enumerate() {
            media.asset(*reference)?;
            if self.description.resident[..index].contains(reference) {
                return Err(SceneError::DuplicateAsset(reference.id));
            }
        }
        let resident = |reference: AssetRef| {
            if self.description.resident.contains(&reference) {
                Ok(())
            } else {
                Err(SceneError::MissingAsset(reference.id))
            }
        };
        resident(self.description.view.room)?;
        for line in self.lines() {
            resident(line.media.asset)?;
            resident(line.palette.asset)?;
            for plane in &line.planes {
                resident(plane.map)?;
                resident(plane.atlas)?;
                resident(plane.palette.asset)?;
            }
            for object in line.objects.iter() {
                resident(object.visual)?;
                resident(object.atlas)?;
                resident(object.material)?;
            }
            for atlas in line.object_selection.atlases {
                resident(atlas)?;
            }
            if line.object_selection.even_frame != geometry.even_frame {
                return Err(SceneError::Geometry);
            }
            validate_line(line, media)?;
        }
        self.validate_producer_history()?;
        Ok(())
    }
    fn validate_producer_history(&self) -> Result<(), SceneError> {
        // Only compatibility construction has no supplied line history.
        if self.producer_history.is_empty() {
            return Ok(());
        }
        if self.producer_history.len() != self.lines().len() {
            return Err(SceneError::Incomplete);
        }
        for (line, records) in self.lines().iter().zip(self.producer_history.iter()) {
            for (index, record) in records.iter().enumerate() {
                if record.producer.0 == 0 {
                    return Err(SceneError::Identity);
                }
                if line.objects.get(usize::from(record.object.order)) != Some(&record.object)
                    || record.context.selection != line.object_selection
                    || record.context.palette != line.palette
                    || records[..index]
                        .iter()
                        .any(|old| old.object.order == record.object.order)
                {
                    return Err(SceneError::Order);
                }
                record.validate(&self.media)?;
            }
        }
        Ok(())
    }
}
fn validate_line(line: &SceneLine, media: &SceneMedia) -> Result<(), SceneError> {
    let image = media.image(line.media)?;
    media.palette(line.palette)?;
    if image.palette != line.palette {
        return Err(SceneError::InconsistentMedia(line.media));
    }
    if matches!(line.mode, SceneMode::AffinePlane { extended: true }) {
        return Err(SceneError::ExtendedBackground);
    }
    if line.color.brightness > 15
        || line.color.fixed.iter().any(|v| *v > 31)
        || line.pixelation.size == 0
        || line.pixelation.size > 16
    {
        return Err(SceneError::Unsupported);
    }
    if line.mode == SceneMode::TilePlanes && !line.blank {
        for (index, enabled) in line.pixelation.enabled.iter().enumerate() {
            if *enabled {
                return Err(SceneError::EnabledMosaic(index));
            }
        }
    }
    for (index, plane) in line.planes.iter().enumerate() {
        if plane.role != [PlaneRole::Main, PlaneRole::Secondary, PlaneRole::HudFx][index] {
            return Err(SceneError::Order);
        }
        let map = media.map(plane.map)?;
        let atlas = media.atlas(plane.atlas)?;
        media.palette(plane.palette)?;
        if map.atlas != plane.atlas
            || map.palette != plane.palette.asset
            || map.width != plane.width
            || map.height != plane.height
            || plane.cells.len() != usize::from(plane.width) * usize::from(plane.height)
        {
            return Err(SceneError::Definition(plane.map.id));
        }
        let binding = image
            .maps
            .iter()
            .find(|binding| binding.map == plane.map)
            .ok_or(SceneError::MissingAsset(plane.map.id))?;
        if !image
            .atlases
            .iter()
            .any(|binding| binding.atlas == plane.atlas)
        {
            return Err(SceneError::MissingAsset(plane.atlas.id));
        }
        if line.mode == SceneMode::TilePlanes
            && atlas.encoding
                != if index == 2 {
                    TileEncoding::Planar2
                } else {
                    TileEncoding::Planar4
                }
        {
            return Err(SceneError::Unsupported);
        }
        if matches!(line.mode, SceneMode::AffinePlane { .. })
            && index == 0
            && atlas.encoding != TileEncoding::Affine8
        {
            return Err(SceneError::Unsupported);
        }
        for (cell_index, cell) in plane.cells.iter().enumerate() {
            validate_cell(*cell, atlas)?;
            let column = cell_index % usize::from(plane.width);
            let row = cell_index / usize::from(plane.width);
            let address = if atlas.encoding == TileEncoding::Affine8 {
                if binding.first_word != 0 {
                    return Err(SceneError::Unsupported);
                }
                row * 128 + column
            } else {
                let block = (column / 32) * 0x400
                    + (row / 32) * if plane.width == 64 { 0x800 } else { 0x400 };
                ((usize::from(binding.first_word) + (row % 32) * 32 + block) & 0x7fff) + column % 32
            };
            let actual = *image.words.get(address).ok_or(SceneError::Incomplete)?;
            let expected = if atlas.encoding == TileEncoding::Affine8 {
                cell.tile
            } else {
                cell.planar_word()
            };
            if (if atlas.encoding == TileEncoding::Affine8 {
                actual & 255
            } else {
                actual
            }) != expected
            {
                return Err(SceneError::InconsistentMedia(line.media));
            }
        }
    }
    if line.objects.len() != 128
        || line.object_selection.size_set > 7
        || line.object_selection.first_order.is_some_and(|v| v >= 128)
    {
        return Err(SceneError::Incomplete);
    }
    for atlas in line.object_selection.atlases {
        if media.atlas(atlas)?.encoding != TileEncoding::Planar4
            || !image.atlases.iter().any(|binding| binding.atlas == atlas)
        {
            return Err(SceneError::Definition(atlas.id));
        }
    }
    for (index, object) in line.objects.iter().enumerate() {
        if usize::from(object.order) != index {
            return Err(SceneError::Order);
        }
        let visual = match media.asset(object.visual)? {
            SceneAsset::Visual(v) => v,
            _ => return Err(SceneError::AssetKind(object.visual.id)),
        };
        let frame = visual
            .frames
            .get(usize::from(object.frame))
            .ok_or(SceneError::Definition(object.visual.id))?;
        let part = frame
            .parts
            .get(usize::from(object.part))
            .ok_or(SceneError::Definition(object.visual.id))?;
        if part.atlas != object.atlas
            || part.material != object.material
            || part.tile != object.tile
            || part.size != object.size
            || !line.object_selection.atlases.contains(&object.atlas)
            || object.tile > 255
            || object.palette > 7
            || object.priority > 3
        {
            return Err(SceneError::Definition(object.visual.id));
        }
    }
    for (index, alias) in line.object_aliases.iter().enumerate() {
        if (0..256).contains(&alias.x)
            || line.object_aliases[..index]
                .iter()
                .any(|old| old.x == alias.x)
        {
            return Err(SceneError::Order);
        }
    }
    Ok(())
}
