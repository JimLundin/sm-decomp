//! Read-only consumer of complete committed semantic Scene history.

use crate::rendering as r;
use sm_data::scene::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SceneFrame {
    pub tick: u64,
    pub revision: Revision,
    pub pixels: Vec<r::Bgra>,
    /// Presentation diagnostics. Never written back into G's committed status.
    pub raster_overflow: r::Overflow,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SceneRenderFailure {
    pub error: SceneError,
    pub completed_pixels: Vec<r::Bgra>,
    pub partial_line: Option<Box<[r::Bgra; r::WIDTH]>>,
    pub partial_objects: Option<Box<r::SpriteLine>>,
}

/// Queries only immutable committed records. No gameplay/HDMA/IRQ operation,
/// native object, frame stepping or authoritative status publication occurs.
pub fn render(scene: &Scene) -> Result<SceneFrame, SceneRenderFailure> {
    let mut pixels = Vec::with_capacity(r::WIDTH * scene.lines().len());
    let initial = scene.description().initial_object_overflow;
    let mut overflow = r::Overflow {
        range: initial.range,
        time: initial.time,
    };
    for (line_index, line) in scene.lines().iter().enumerate() {
        let index = match u16::try_from(line_index) {
            Ok(index) => index,
            Err(_) => {
                return Err(SceneRenderFailure {
                    error: SceneError::Geometry,
                    completed_pixels: pixels,
                    partial_line: None,
                    partial_objects: None,
                });
            }
        };
        let row = match render_row(scene.media(), line, index, overflow) {
            Ok(row) => row,
            Err(mut failure) => {
                failure.completed_pixels = pixels;
                return Err(failure);
            }
        };
        pixels.extend_from_slice(row.pixels.as_ref());
        overflow = row.objects.overflow;
    }
    Ok(SceneFrame {
        tick: scene.tick(),
        revision: scene.revision(),
        pixels,
        raster_overflow: overflow,
    })
}

/// The shared raster consumer for one immutable row, also used at settlement.
/// The caller supplies the original zero-based OBJ row and sticky status.
pub struct SceneRow {
    pub pixels: Box<[r::Bgra; r::WIDTH]>,
    pub objects: Box<r::SpriteLine>,
}
pub fn render_row(
    media: &SceneMedia,
    line: &SceneLine,
    index: u16,
    overflow: r::Overflow,
) -> Result<SceneRow, SceneRenderFailure> {
    let failure = |error, partial_line, partial_objects| SceneRenderFailure {
        error,
        completed_pixels: Vec::new(),
        partial_line,
        partial_objects,
    };
    let image = media
        .image(line.media)
        .map_err(|error| failure(error, None, None))?;
    let palette = media
        .palette(line.palette)
        .map_err(|error| failure(error, None, None))?;
    let source = SceneIndexedSource { image, palette };
    line.validate_object_slots(index)
        .map_err(|error| failure(error, None, None))?;
    let sprites = encode_objects(line, image).map_err(|error| failure(error, None, None))?;
    let mut object_line = r::SpriteLine::cleared(overflow);
    if !line.blank {
        match r::evaluate_sprites(
            &source,
            &sprites.borrowed(),
            i32::from(index),
            &mut object_line.pixels,
            overflow,
        ) {
            Ok((has_tiles, flags)) => {
                object_line.has_tiles = has_tiles;
                object_line.overflow = flags;
            }
            Err(error) => return Err(failure(error.into(), None, Some(Box::new(object_line)))),
        }
    }
    let brightness = r::BrightnessTables::for_level(line.color.brightness);
    let config = line_config(line, image, &brightness, object_line.has_tiles)
        .map_err(|error| failure(error, None, Some(Box::new(object_line.clone()))))?;
    let objects = SceneObjectPixels {
        line: &object_line,
        aliases: &line.object_aliases,
    };
    let mut pixels = Box::new([[0; 4]; r::WIDTH]);
    if let Err(error) = r::render_line(
        &source,
        &config,
        &objects,
        i32::from(index) + 1,
        &mut pixels,
    ) {
        return Err(failure(
            error.into(),
            Some(pixels),
            Some(Box::new(object_line)),
        ));
    }
    Ok(SceneRow {
        pixels,
        objects: Box::new(object_line),
    })
}

struct SceneIndexedSource<'a> {
    image: &'a CompiledImage,
    palette: &'a PaletteIncarnation,
}
impl r::IndexedSource for SceneIndexedSource<'_> {
    fn word(&self, index: usize) -> Result<u16, r::RasterFault> {
        self.image
            .words
            .get(index)
            .copied()
            .ok_or(r::RasterFault::Word(index))
    }
    fn palette(&self, index: u8) -> Result<u16, r::RasterFault> {
        Ok(self.palette.colors[usize::from(index)])
    }
}

struct EncodedObjects {
    words: [u16; 256],
    extension: [u8; 32],
    tile_words: [u16; 2],
    size: u8,
    rotation: Option<u8>,
    interlace: bool,
    even_frame: bool,
}
impl EncodedObjects {
    fn borrowed(&self) -> r::SpriteInput<'_> {
        r::SpriteInput {
            words: &self.words,
            extension: &self.extension,
            tile_words: self.tile_words,
            size: self.size,
            rotation: self.rotation,
            interlace: self.interlace,
            even_frame: self.even_frame,
        }
    }
}
fn atlas_word(image: &CompiledImage, reference: AssetRef) -> Result<u16, SceneError> {
    image
        .atlases
        .iter()
        .find(|binding| binding.atlas == reference)
        .map(|binding| binding.first_word)
        .ok_or(SceneError::MissingAsset(reference.id))
}
fn encode_objects(line: &SceneLine, image: &CompiledImage) -> Result<EncodedObjects, SceneError> {
    let selection = line.object_selection;
    let mut output = EncodedObjects {
        words: [0; 256],
        extension: [0; 32],
        tile_words: [
            atlas_word(image, selection.atlases[0])?,
            atlas_word(image, selection.atlases[1])?,
        ],
        size: selection.size_set,
        rotation: selection.first_order.map(|order| order.wrapping_mul(2)),
        interlace: selection.interlaced,
        even_frame: selection.even_frame,
    };
    // Consume already selected source order. No actor, pose, material or frame
    // selection is repeated, and no asset-ID ordering drives object allocation.
    for object in line.objects.iter() {
        let index = usize::from(object.order);
        if index >= 128 {
            return Err(SceneError::Order);
        }
        let x = object.position.x as u16 & 0x1ff;
        output.words[index * 2] = (x & 255) | ((object.position.y as u16) << 8);
        let second = object.atlas != selection.atlases[0];
        output.words[index * 2 + 1] = object.tile
            | (u16::from(second) << 8)
            | (u16::from(object.palette) << 9)
            | (u16::from(object.priority) << 12)
            | (u16::from(object.flip_horizontal) << 14)
            | (u16::from(object.flip_vertical) << 15);
        let high = ((x >> 8) as u8) | (u8::from(object.size == ObjectSize::Large) << 1);
        output.extension[index / 4] |= high << ((index % 4) * 2);
    }
    for object in line.unauthored_objects.iter() {
        let index = usize::from(object.order);
        if index >= 128 {
            return Err(SceneError::Order);
        }
        let words = object.raster_words();
        output.words[index * 2..index * 2 + 2].copy_from_slice(&words);
        let x = u16::from_le_bytes(object.position.x.to_le_bytes()) & 0x1ff;
        let high =
            u8::try_from(x >> 8).unwrap() | (u8::from(object.size == ObjectSize::Large) << 1);
        output.extension[index / 4] |= high << ((index % 4) * 2);
    }
    Ok(output)
}
struct SceneObjectPixels<'a> {
    line: &'a r::SpriteLine,
    aliases: &'a [SignedObjectPixel],
}
impl r::SpritePixels for SceneObjectPixels<'_> {
    fn pixel(&self, x: i32) -> Result<u16, r::RasterFault> {
        if let Ok(index) = usize::try_from(x)
            && let Some(value) = self.line.pixels.get(index)
        {
            return Ok(*value);
        }
        self.aliases
            .iter()
            .find(|alias| alias.x == x)
            .map(|alias| alias.value)
            .ok_or(r::RasterFault::SpritePixel(x))
    }
}
fn condition(value: WindowCondition) -> u8 {
    match value {
        WindowCondition::Never => 0,
        WindowCondition::Outside => 1,
        WindowCondition::Inside => 2,
        WindowCondition::Always => 3,
    }
}
fn routing(value: &ScreenRouting, masked: bool) -> u8 {
    let bits = if masked {
        &value.masked
    } else {
        &value.visible
    };
    bits.iter().enumerate().fold(0, |value, (index, enabled)| {
        value | (u8::from(*enabled) << index)
    })
}
fn line_config<'a>(
    line: &'a SceneLine,
    image: &CompiledImage,
    brightness: &'a r::BrightnessTables,
    has_objects: bool,
) -> Result<r::LineConfig<'a>, SceneError> {
    let mut backgrounds = [r::Background {
        horizontal_scroll: 0,
        vertical_scroll: 0,
        map_words: 0,
        tile_words: 0,
        wider: false,
        higher: false,
    }; 3];
    for (index, plane) in line.planes.iter().enumerate() {
        let map = image
            .maps
            .iter()
            .find(|binding| binding.map == plane.map)
            .ok_or(SceneError::MissingAsset(plane.map.id))?;
        backgrounds[index] = r::Background {
            horizontal_scroll: plane.scroll.x as u16,
            vertical_scroll: plane.scroll.y as u16,
            map_words: map.first_word,
            tile_words: atlas_word(image, plane.atlas)?,
            wider: plane.width == 64,
            higher: plane.height == 64,
        };
    }
    let mut native_flags = 0;
    let layers = core::array::from_fn(|index| {
        let value = line.windows.layers[index];
        native_flags |= (u32::from(value.first_inverted)
            | (u32::from(value.first_enabled) << 1)
            | (u32::from(value.second_inverted) << 2)
            | (u32::from(value.second_enabled) << 3))
            << (index * 4);
        r::Window {
            first_enabled: value.first_enabled,
            first_inverted: value.first_inverted,
            second_enabled: value.second_enabled,
            second_inverted: value.second_inverted,
            logic: match value.combination {
                WindowCombination::Union => 0,
                WindowCombination::Intersection => 1,
                WindowCombination::Exclusive => 2,
                WindowCombination::Equal => 3,
            },
        }
    });
    let mode = match line.mode {
        SceneMode::TilePlanes => 1,
        SceneMode::AffinePlane { extended: false } => 7,
        SceneMode::AffinePlane { extended: true } => return Err(SceneError::ExtendedBackground),
    };
    let affine = line.affine;
    let color = line.color;
    Ok(r::LineConfig {
        mode,
        forced_blank: line.blank,
        backgrounds,
        windows: r::Windows {
            first: [line.windows.extents[0].first, line.windows.extents[0].last],
            second: [line.windows.extents[1].first, line.windows.extents[1].last],
            native_flags,
            layers,
        },
        routing: r::Routing {
            enabled: [
                routing(&line.routing[0], false),
                routing(&line.routing[1], false),
            ],
            windowed: [
                routing(&line.routing[0], true),
                routing(&line.routing[1], true),
            ],
        },
        affine: r::Affine {
            matrix: [
                affine.matrix[0],
                affine.matrix[1],
                affine.matrix[2],
                affine.matrix[3],
                affine.center.x,
                affine.center.y,
                affine.scroll.x,
                affine.scroll.y,
            ],
            horizontal_flip: affine.flip_horizontal,
            vertical_flip: affine.flip_vertical,
            large_field: affine.bounded_field,
            character_fill: affine.fill_outside,
        },
        mosaic: r::Mosaic {
            native_enabled: line
                .pixelation
                .enabled
                .iter()
                .enumerate()
                .fold(0, |value, (index, enabled)| {
                    value | (u8::from(*enabled) << index)
                }),
            background0_enabled: line.pixelation.enabled[0],
            size: line.pixelation.size,
            start_line: line.pixelation.origin,
            modulo: &line.pixelation.modulo,
            left: 0,
        },
        color: r::ColorMath {
            clip: condition(color.clip),
            prevent: condition(color.prevent_math),
            enabled: color.enabled,
            add_subscreen: color.use_subscreen,
            subtract: color.subtract,
            half: color.half,
            fixed: color.fixed,
            brightness: color.brightness,
        },
        brightness: brightness.borrowed(),
        has_sprites: has_objects,
    })
}
