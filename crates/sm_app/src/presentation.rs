//! Prepared presentation coordination without a native backend or Game RAM.
#![forbid(unsafe_code)]
use sm_game::{
    persistent_session::PersistentSession,
    scene::{
        commit::{RoomEffects, SettlementContext, SettlementEffects},
        owned_hdma::Destinations,
        records::{SceneDescription, SceneMedia},
        state::DisplayError,
    },
};
use std::sync::Arc;

pub enum Failure {
    Game(DisplayError),
    Render(sm_runtime::scene::SceneRenderFailure),
}
/// One admitted game interval settles before the renderer receives its Scene.
pub fn settle_and_render(
    session: &mut PersistentSession,
    effects: &mut (impl RoomEffects + Destinations),
    declarations: &RasterDeclarations,
    description: SceneDescription,
    media: Arc<SceneMedia>,
) -> Result<sm_runtime::scene::SceneFrame, Failure> {
    let mut raster = RasterEffects {
        external: effects,
        declarations,
        lines: Vec::new(),
        failure: None,
        initial_overflow: None,
        pixels: Vec::new(),
    };
    let scene = session.finish_interval_with_description(
        &mut raster,
        |raster, state, previous, current_media| {
            // The supplied complete media is an admission requirement. The live
            // owner may extend it, but may not silently switch to another revision.
            if media.revision() != current_media.revision() {
                return Err(DisplayError::Revision);
            }
            Ok((
                raster.description(description, state, previous)?,
                Arc::clone(current_media),
            ))
        },
    );
    match scene {
        Ok(scene) if raster.pixels.is_empty() => {
            sm_runtime::scene::render(&scene).map_err(Failure::Render)
        }
        Ok(scene) => Ok(sm_runtime::scene::SceneFrame {
            tick: scene.tick(),
            revision: scene.revision(),
            pixels: raster.pixels,
            raster_overflow: sm_runtime::rendering::Overflow {
                range: scene.description().status.range_overflow,
                time: scene.description().status.time_overflow,
            },
        }),
        Err(error) => match raster.failure {
            Some(failure) => Err(Failure::Render(failure)),
            None => Err(Failure::Game(error)),
        },
    }
}
/// One admitted interval of the persistent message runner and existing renderer.
/// Semantic delivery effects retain their original ordered records.
pub fn resume_dialogue(
    session: &mut PersistentSession,
    effects: &mut impl sm_game::prepared_dialogue::Effects,
) -> Result<
    sm_game::message_composition::prepared::StepOutput<sm_runtime::message::MessageFrame>,
    sm_data::message::Fault,
> {
    session.resume_dialogue(effects, sm_runtime::message::render)
}

/// Admit the complete sealed tools payload with the target's independent world.
pub fn open(
    supplied: sm_game::persistent_session::SuppliedSessionStart,
    world: &Arc<sm_game::room_entry::world::WorldBinding>,
) -> Result<PersistentSession, sm_game::persistent_session::SessionStartError> {
    sm_game::persistent_session::ValidatedSessionStart::construct(supplied, world)
        .map(|start| start.open())
}
/// Move a validated suspended replacement into the sole owner; no interval replay.
pub fn reopen(
    session: &mut PersistentSession,
    checkpoint: sm_game::persistent_session::SessionCheckpoint,
    contract: &sm_game::persistent_session::SessionStartContract,
) -> Result<(), sm_game::persistent_session::SessionStartError> {
    session.reopen(checkpoint, contract)
}

/// A finite source-backed catalogue of actual native row inputs not represented
/// by the current effective owner: resource identities, OBJSEL/rotation/interlace,
/// EXTBG, window extents/logic/routing, color and mosaic. Supply every reachable
/// (one-based row, full delivered BGMODE byte) combination; absence is an error.
#[derive(Clone)]
pub struct RasterDeclaration {
    pub row: u16,
    pub mode_byte: u8,
    pub line: sm_game::scene::records::SceneLine,
    /// Independently admitted current-row sources for otherwise unbound OAM
    /// entries. These are not copied from the previous committed Scene.
    pub unbound_objects: Arc<[NativeObjectDeclaration]>,
}
#[derive(Clone)]
pub struct NativeObjectDeclaration {
    pub record: sm_game::scene::records::ProducedObject,
    pub bytes: [u8; 4],
    pub extension: u8,
}
/// Admitted authored OAM identity for one current native word/size at a slot.
/// Lifecycle validation still runs against the current producer owner each row.
#[derive(Clone)]
pub struct NativeObjectBinding {
    pub record: sm_game::scene::records::ProducedObject,
    pub word: u16,
    pub large: bool,
}
#[derive(Clone)]
pub struct RasterCatalog {
    pub planes: Arc<[RasterPlaneBinding]>,
    pub object_atlases: Arc<[[sm_game::scene::records::AssetRef; 2]]>,
    pub objects: Arc<[NativeObjectBinding]>,
    /// Explicit native signed-pixel aliases when the admitted path requires them.
    pub object_aliases: Arc<[sm_game::scene::records::SignedObjectPixel]>,
}
#[derive(Clone)]
pub struct RasterPlaneBinding {
    pub mode: u8,
    pub role: sm_game::scene::records::PlaneRole,
    pub map: sm_game::scene::records::AssetRef,
    pub atlas: sm_game::scene::records::AssetRef,
}
pub struct RasterDeclarations {
    rows: Arc<[RasterDeclaration]>,
    catalog: Option<RasterCatalog>,
}
impl RasterDeclarations {
    pub fn new(rows: Arc<[RasterDeclaration]>) -> Result<Self, DisplayError> {
        for (index, row) in rows.iter().enumerate() {
            for (object_index, object) in row.unbound_objects.iter().enumerate() {
                if object.record.object.order >= 128
                    || object.extension > 3
                    || row.unbound_objects[..object_index]
                        .iter()
                        .any(|old| old.record.object.order == object.record.object.order)
                {
                    return Err(DisplayError::Geometry);
                }
            }
            if !(1..=224).contains(&row.row)
                || rows[..index]
                    .iter()
                    .any(|old| old.row == row.row && old.mode_byte == row.mode_byte)
            {
                return Err(DisplayError::Geometry);
            }
        }
        Ok(Self {
            rows,
            catalog: None,
        })
    }
    /// Admit finite authored sources once. Effective line inputs are read at each
    /// actual raster call; callers supply no future-row register description.
    pub fn from_catalog(catalog: RasterCatalog) -> Result<Self, DisplayError> {
        for (index, binding) in catalog.objects.iter().enumerate() {
            if binding.record.object.order >= 128
                || binding.record.native_word != binding.word
                || catalog.objects[..index].iter().any(|old| {
                    old.record.object.order == binding.record.object.order
                        && old.word == binding.word
                        && old.large == binding.large
                })
            {
                return Err(DisplayError::Geometry);
            }
        }
        Ok(Self {
            rows: Arc::from([]),
            catalog: Some(catalog),
        })
    }
    fn row(
        &self,
        row: u16,
        context: &SettlementContext<'_, '_>,
    ) -> Result<RasterDeclaration, DisplayError> {
        if let Some(catalog) = &self.catalog {
            return catalog.row(row, context);
        }
        self.rows
            .iter()
            .find(|entry| {
                entry.row == row && entry.mode_byte == context.state().effective.mode_byte
            })
            .cloned()
            .ok_or(DisplayError::Source)
    }
}
struct RasterEffects<'a, E> {
    external: &'a mut E,
    declarations: &'a RasterDeclarations,
    lines: Vec<sm_game::scene::records::SceneLine>,
    failure: Option<sm_runtime::scene::SceneRenderFailure>,
    initial_overflow: Option<sm_game::scene::records::ObjectOverflow>,
    pixels: Vec<sm_runtime::rendering::Bgra>,
}
impl<E: RoomEffects> RoomEffects for RasterEffects<'_, E> {
    fn follow_transition(
        &mut self,
        context: &mut SettlementContext<'_, '_>,
    ) -> Result<(), DisplayError> {
        self.external.follow_transition(context)
    }
}
impl<E: Destinations> Destinations for RasterEffects<'_, E> {
    fn read_destination(
        &mut self,
        context: &mut SettlementContext<'_, '_>,
        destination: u8,
    ) -> Result<u8, DisplayError> {
        self.external.read_destination(context, destination)
    }
    fn write_destination(
        &mut self,
        context: &mut SettlementContext<'_, '_>,
        destination: u8,
        value: u8,
    ) -> Result<(), DisplayError> {
        if context.write_raster(destination, value) {
            return Ok(());
        }
        self.external.write_destination(context, destination, value)
    }
}
impl<E: RoomEffects + Destinations> SettlementEffects for RasterEffects<'_, E> {
    fn raster_row(
        &mut self,
        context: &mut SettlementContext<'_, '_>,
        row: u16,
    ) -> Result<sm_game::scene::records::ObjectContext, DisplayError> {
        use sm_game::scene::records::{ObjectContext, ObjectOverflow};
        let index = row.checked_sub(1).ok_or(DisplayError::Geometry)?;
        if usize::from(index) != self.lines.len() {
            return Err(DisplayError::Phase);
        }
        let declaration = self.declarations.row(row, context)?;
        let mut line =
            sm_game::scene::hardware::sample_line(context.state(), declaration.line.clone())?;
        line.media = context.image().image;
        line.palette = context.image().palette;
        let object_context = ObjectContext {
            selection: line.object_selection,
            palette: line.palette,
        };
        let media = context.media().ok_or(DisplayError::Incomplete)?;
        let mut sources = context.image().produced.clone();
        // Unbound entries require their own exact native row admission. Compare
        // actual physical bytes before accepting any authored metadata.
        for entry in declaration.unbound_objects.iter() {
            let order = entry.record.object.order;
            if sources.iter().any(|record| record.object.order == order) {
                continue;
            }
            let offset = usize::from(order) * 4;
            let objects = &context.image().objects;
            let extension =
                (objects[512 + usize::from(order) / 4] >> ((usize::from(order) & 3) * 2)) & 3;
            if objects.get(offset..offset + 4) != Some(entry.bytes.as_slice())
                || extension != entry.extension
            {
                return Err(DisplayError::Source);
            }
            sources.push(entry.record.clone());
        }
        sources.sort_by_key(|record| record.object.order);
        let records = sources
            .iter()
            .map(|record| {
                context
                    .state()
                    .producer_ids
                    .validate_id(record.producer)
                    .map_err(|_| DisplayError::Identity)?;
                record
                    .for_line(media, object_context, &context.image().objects)
                    .map_err(|_| DisplayError::Scene)
            })
            .collect::<Result<Vec<_>, _>>()?;
        line.objects = Arc::from(
            records
                .iter()
                .map(|record| record.object.clone())
                .collect::<Vec<_>>(),
        );
        line.capture_unauthored(&context.image().objects)
            .map_err(|_| DisplayError::Scene)?;
        let overflow = sm_runtime::rendering::Overflow {
            range: context.state().raster.range_overflow,
            time: context.state().raster.time_overflow,
        };
        if self.initial_overflow.is_none() {
            self.initial_overflow = Some(ObjectOverflow {
                range: overflow.range,
                time: overflow.time,
            });
        }
        // Successful finite admission binds the current resident entries. The
        // original post-raster history collector now sees these same sources.
        context.image_mut().produced = records;
        let media = context.media().ok_or(DisplayError::Incomplete)?;
        match sm_runtime::scene::render_row(media, &line, index, overflow) {
            Ok(rendered) => {
                let raster = context.raster_mut();
                raster.range_overflow = rendered.objects.overflow.range;
                raster.time_overflow = rendered.objects.overflow.time;
                raster.has_objects = rendered.objects.has_tiles;
                self.pixels.extend_from_slice(rendered.pixels.as_ref());
                self.lines.push(line);
                Ok(object_context)
            }
            Err(mut failure) => {
                failure.completed_pixels = std::mem::take(&mut self.pixels);
                if let Some(objects) = failure.partial_objects.as_ref() {
                    let raster = context.raster_mut();
                    raster.range_overflow = objects.overflow.range;
                    raster.time_overflow = objects.overflow.time;
                    raster.has_objects = objects.has_tiles;
                }
                self.failure = Some(failure);
                Err(DisplayError::Scene)
            }
        }
    }
}
impl<E> RasterEffects<'_, E> {
    fn description(
        &mut self,
        mut description: SceneDescription,
        state: &sm_game::scene::state::DisplayState,
        previous: &sm_game::scene::records::Scene,
    ) -> Result<SceneDescription, DisplayError> {
        if self.lines.is_empty() {
            description.lines = Arc::from(previous.lines().to_vec());
            description.initial_object_overflow = previous.description().initial_object_overflow;
        } else {
            if self.lines.len() != 224 {
                return Err(DisplayError::Incomplete);
            }
            description.lines = Arc::from(std::mem::take(&mut self.lines));
            description.initial_object_overflow =
                self.initial_overflow.ok_or(DisplayError::Incomplete)?;
        }
        let raster = state.raster;
        description.status = sm_game::scene::records::DisplayStatus {
            range_overflow: raster.range_overflow,
            time_overflow: raster.time_overflow,
            has_objects: raster.has_objects,
            even_frame: raster.even_frame,
            interlaced: raster.frame_interlaced,
            overscan: raster.frame_overscan,
        };
        description.geometry.even_frame = raster.even_frame;
        description.geometry.interlaced = raster.frame_interlaced;
        description.geometry.overscan = raster.frame_overscan;
        Ok(description)
    }
}

impl RasterCatalog {
    fn row(
        &self,
        row: u16,
        context: &SettlementContext<'_, '_>,
    ) -> Result<RasterDeclaration, DisplayError> {
        use sm_game::scene::records::*;
        let state = context.state();
        let effective = &state.effective;
        let inputs = &effective.raster_inputs;
        let resident = context.image();
        let media = context.media().ok_or(DisplayError::Incomplete)?;
        let image = media
            .image(resident.image)
            .map_err(|_| DisplayError::Source)?;
        // A numeric base selects an admitted placement within its declared role.
        // Multiple authored assets may share a physical base; bytes cannot decide IDs.
        let atlas_at = |reference, word| {
            image
                .atlases
                .iter()
                .any(|binding| binding.atlas == reference && binding.first_word == word)
        };
        let map_at = |reference, word| {
            image
                .maps
                .iter()
                .any(|binding| binding.map == reference && binding.first_word == word)
        };
        let mode = match effective.mode_byte & 7 {
            1 => SceneMode::TilePlanes,
            7 => SceneMode::AffinePlane {
                extended: inputs.extended_background,
            },
            _ => return Err(DisplayError::Scene),
        };
        let plane = |index: usize| -> Result<ScenePlane, DisplayError> {
            let affine = matches!(mode, SceneMode::AffinePlane { .. });
            let word = if affine {
                0
            } else if index == 2 {
                effective.background3_map_words
            } else {
                inputs.map_words[index]
            };
            let role = [PlaneRole::Main, PlaneRole::Secondary, PlaneRole::HudFx][index];
            let tile_word = if affine { 0 } else { inputs.tile_words[index] };
            let mut matches = self.planes.iter().filter(|binding| {
                binding.mode == (effective.mode_byte & 7)
                    && binding.role == role
                    && map_at(binding.map, word)
                    && atlas_at(binding.atlas, tile_word)
            });
            let binding = matches.next().ok_or(DisplayError::Source)?;
            if matches.next().is_some() {
                return Err(DisplayError::Source);
            }
            let reference = binding.map;
            let definition = match media.asset(reference).map_err(|_| DisplayError::Source)? {
                SceneAsset::Map(map) => map,
                _ => return Err(DisplayError::Source),
            };
            let tiles = binding.atlas;
            let wider = if index == 2 {
                effective.background3_map_wider
            } else {
                inputs.map_wider[index]
            };
            let higher = if index == 2 {
                effective.background3_map_higher
            } else {
                inputs.map_higher[index]
            };
            let (width, height) = if affine {
                (128, 128)
            } else {
                (if wider { 64 } else { 32 }, if higher { 64 } else { 32 })
            };
            if definition.atlas != tiles || definition.width != width || definition.height != height
            {
                return Err(DisplayError::Source);
            }
            // Cells are current numeric map contents; only the admitted placement
            // and authored definitions provide their map/atlas identities.
            let mut cells = Vec::with_capacity(usize::from(width) * usize::from(height));
            for y in 0..height {
                for x in 0..width {
                    let offset = if affine {
                        usize::from(y) * 128 + usize::from(x)
                    } else {
                        usize::from(y & 31) * 32
                            + usize::from(x & 31)
                            + usize::from(x / 32) * 0x400
                            + usize::from(y / 32) * if width == 64 { 0x800 } else { 0x400 }
                    };
                    let address = if affine {
                        offset
                    } else {
                        (usize::from(word) + offset - usize::from(x & 31)) & 0x7fff
                    };
                    let address = if affine {
                        address
                    } else {
                        address + usize::from(x & 31)
                    };
                    let raw = *resident.words.get(address).ok_or(DisplayError::Source)?;
                    cells.push(if affine {
                        TileCell {
                            tile: raw & 255,
                            palette: 0,
                            priority: false,
                            flip_horizontal: false,
                            flip_vertical: false,
                        }
                    } else {
                        TileCell {
                            tile: raw & 1023,
                            palette: ((raw >> 10) & 7).to_le_bytes()[0],
                            priority: raw & 0x2000 != 0,
                            flip_horizontal: raw & 0x4000 != 0,
                            flip_vertical: raw & 0x8000 != 0,
                        }
                    });
                }
            }
            Ok(ScenePlane {
                role: [PlaneRole::Main, PlaneRole::Secondary, PlaneRole::HudFx][index],
                map: reference,
                atlas: tiles,
                palette: resident.palette,
                width,
                height,
                cells: Arc::from(cells),
                scroll: ScreenPosition { x: 0, y: 0 },
            })
        };
        let mut matches = self.object_atlases.iter().filter(|pair| {
            atlas_at(pair[0], inputs.object_tile_words[0])
                && atlas_at(pair[1], inputs.object_tile_words[1])
        });
        let object_atlases = *matches.next().ok_or(DisplayError::Source)?;
        if matches.next().is_some() {
            return Err(DisplayError::Source);
        }
        let selection = ObjectSelection {
            atlases: object_atlases,
            size_set: inputs.object_size,
            first_order: inputs
                .object_priority
                .then(|| ((state.latches.object_address & 255) >> 1).to_le_bytes()[0]),
            interlaced: inputs.object_interlaced,
            even_frame: state.raster.even_frame,
        };
        let window = |index: usize| -> Result<LayerWindow, DisplayError> {
            let flags = effective.window_selection[index / 2] >> ((index & 1) * 4);
            Ok(LayerWindow {
                first_enabled: flags & 2 != 0,
                first_inverted: flags & 1 != 0,
                second_enabled: flags & 8 != 0,
                second_inverted: flags & 4 != 0,
                combination: match inputs.window_logic[index] {
                    0 => WindowCombination::Union,
                    1 => WindowCombination::Intersection,
                    2 => WindowCombination::Exclusive,
                    3 => WindowCombination::Equal,
                    _ => return Err(DisplayError::Geometry),
                },
            })
        };
        let affine = effective.affine;
        let condition = |value| match value {
            0 => Ok(WindowCondition::Never),
            1 => Ok(WindowCondition::Outside),
            2 => Ok(WindowCondition::Inside),
            3 => Ok(WindowCondition::Always),
            _ => Err(DisplayError::Geometry),
        };
        let line = SceneLine {
            media: resident.image,
            palette: resident.palette,
            mode,
            blank: effective.forced_blank,
            planes: [plane(0)?, plane(1)?, plane(2)?],
            objects: Arc::from([]),
            unauthored_objects: Arc::from([]),
            object_selection: selection,
            windows: SceneWindows {
                extents: core::array::from_fn(|index| WindowExtent {
                    first: inputs.window_positions[index * 2],
                    last: inputs.window_positions[index * 2 + 1],
                }),
                layers: [
                    window(0)?,
                    window(1)?,
                    window(2)?,
                    window(3)?,
                    window(4)?,
                    window(5)?,
                ],
            },
            routing: core::array::from_fn(|index| {
                let enabled = if index == 0 {
                    effective.main_layers
                } else {
                    inputs.sub_layers
                };
                ScreenRouting {
                    visible: core::array::from_fn(|layer| enabled & (1 << layer) != 0),
                    masked: core::array::from_fn(|layer| {
                        inputs.screen_windowed[index] & (1 << layer) != 0
                    }),
                }
            }),
            affine: SceneAffine {
                matrix: [
                    affine.matrix[0],
                    affine.matrix[1],
                    affine.matrix[2],
                    affine.matrix[3],
                ],
                center: ScreenPosition {
                    x: affine.matrix[4],
                    y: affine.matrix[5],
                },
                scroll: ScreenPosition {
                    x: affine.matrix[6],
                    y: affine.matrix[7],
                },
                flip_horizontal: affine.horizontal_flip,
                flip_vertical: affine.vertical_flip,
                bounded_field: affine.large_field,
                fill_outside: affine.character_fill,
            },
            pixelation: ScenePixelation {
                enabled: core::array::from_fn(|index| inputs.mosaic_enabled[index]),
                size: inputs.mosaic_size,
                origin: state.raster.mosaic_origin,
                modulo: inputs.mosaic_modulo,
            },
            color: SceneColor {
                clip: condition(effective.color_clip)?,
                prevent_math: condition(effective.color_prevent_math)?,
                enabled: effective.color_math_enabled,
                use_subscreen: effective.color_add_subscreen,
                subtract: effective.color_subtract,
                half: effective.color_half,
                fixed: inputs.fixed_color,
                brightness: inputs.brightness,
            },
            object_aliases: Arc::clone(&self.object_aliases),
        };
        let mut objects = Vec::new();
        for order in 0..128u16 {
            if resident
                .produced
                .iter()
                .any(|record| record.object.order == order)
            {
                continue;
            }
            let offset = usize::from(order) * 4;
            let bytes: [u8; 4] = resident.objects[offset..offset + 4]
                .try_into()
                .map_err(|_| DisplayError::Source)?;
            let extension = (resident.objects[512 + usize::from(order) / 4]
                >> ((usize::from(order) & 3) * 2))
                & 3;
            let word = u16::from_le_bytes([bytes[2], bytes[3]]);
            let mut matches = self.objects.iter().filter(|binding| {
                binding.record.object.order == order
                    && binding.word == word
                    && binding.large == (extension & 2 != 0)
            });
            let binding = match matches.next() {
                Some(binding) => binding,
                None => {
                    let geometry =
                        sm_data::scene::UnauthoredObject::from_slot(&resident.objects, order)
                            .map_err(|_| DisplayError::Geometry)?;
                    let index = row.checked_sub(1).ok_or(DisplayError::Geometry)?;
                    if line.blank
                        || geometry
                            .invisible_on_row(selection, index)
                            .map_err(|_| DisplayError::Geometry)?
                    {
                        continue;
                    }
                    return Err(DisplayError::Source);
                }
            };
            if matches.next().is_some() {
                return Err(DisplayError::Source);
            }
            objects.push(NativeObjectDeclaration {
                record: binding.record.clone(),
                bytes,
                extension,
            });
        }
        Ok(RasterDeclaration {
            row,
            mode_byte: effective.mode_byte,
            line,
            unbound_objects: Arc::from(objects),
        })
    }
}
