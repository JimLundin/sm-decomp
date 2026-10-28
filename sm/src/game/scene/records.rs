//! Immutable committed game-facing image descriptions and resident authored media.
#![forbid(unsafe_code)]

use crate::rendering as r;
use std::{collections::BTreeMap, sync::Arc};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct AssetId(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Revision(pub [u8; 32]);
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct AssetRef {
    pub id: AssetId,
    pub revision: Revision,
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
    pub(crate) fn planar_word(self) -> u16 {
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
    Palette(Palette),
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
    pub(crate) fn atlas(&self, reference: AssetRef) -> Result<&TileAtlas, SceneError> {
        match self.asset(reference)? {
            SceneAsset::Atlas(v) => Ok(v),
            _ => Err(SceneError::AssetKind(reference.id)),
        }
    }
    pub(crate) fn map(&self, reference: AssetRef) -> Result<&Tilemap, SceneError> {
        match self.asset(reference)? {
            SceneAsset::Map(v) => Ok(v),
            _ => Err(SceneError::AssetKind(reference.id)),
        }
    }
    pub(crate) fn palette(&self, reference: MediaRef) -> Result<&PaletteIncarnation, SceneError> {
        if !matches!(self.asset(reference.asset)?, SceneAsset::Palette(_)) {
            return Err(SceneError::AssetKind(reference.asset.id));
        }
        self.palettes
            .get(&reference)
            .ok_or(SceneError::MissingIncarnation(reference))
    }
    pub(crate) fn image(&self, reference: MediaRef) -> Result<&CompiledImage, SceneError> {
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObjectSelection {
    pub atlases: [AssetRef; 2],
    pub size_set: u8,
    pub first_order: Option<u8>,
    pub interlaced: bool,
    pub even_frame: bool,
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
}
impl Scene {
    pub fn new(description: SceneDescription, media: Arc<SceneMedia>) -> Result<Self, SceneError> {
        let scene = Self {
            description: Arc::new(description),
            media,
        };
        scene.validate(&scene.media)?;
        Ok(scene)
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
