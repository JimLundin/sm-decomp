//! Compatible resource emission; material rows must already be staged at room preparation.
#![forbid(unsafe_code)]
use super::{
    content::ActorContentError,
    media::{ActorMaterialDefinition, MaterialId, VisualResourceId},
    scene::{
        ActorDrawMode, ActorDrawRequest, ActorPaletteMode, ActorTilemapSource, ActorVisualForm,
        ActorVisualSelection,
    },
};
use crate::game::{
    room_entry::world::{LegacyAddress24, ResourceKey, ResourceKind, WorldBinding},
    session::Point,
    session_runtime::Runtime,
};

pub(super) fn draw(
    runtime: &mut Runtime<'_>,
    request: ActorDrawRequest,
) -> Result<(), ActorContentError> {
    let ActorDrawRequest {
        visual,
        position,
        palette,
        tile,
        palette_mode,
        mode,
    } = request;
    match visual {
        ActorVisualSelection::Compatibility(source)
        | ActorVisualSelection::Unresolved {
            compatibility: source,
            ..
        } => {
            // The effect input retains the requested appearance/reason for semantic
            // observation. The defined compatibility emission itself stays unchanged.
            runtime.actor_draw_compatible(
                source.bank,
                source.address,
                position.x,
                position.y,
                palette,
                tile,
                matches!(source.mode, ActorDrawMode::Base),
                matches!(source.mode, ActorDrawMode::ExtendedOffscreen),
            );
        }
        ActorVisualSelection::Prepared(visual) => match visual.form {
            ActorVisualForm::Sprites(pieces) => {
                let world = runtime.session().world.clone();
                for piece in pieces {
                    if piece.source_origin != Point::default()
                        || piece.source_extent != Point::default()
                        || piece.flip_x
                        || piece.flip_y
                    {
                        return Err(ActorContentError::UnsupportedForm);
                    }
                    let (bank, address) =
                        resource_address(&world, piece.resource, ResourceKind::ActorSprites)?;
                    let image = runtime
                        .loaded_image()
                        .ok_or(ActorContentError::PaletteNotResident(piece.material))?;
                    world.actor_media().validate_material_residency(
                        piece.material,
                        runtime.session().actor_identity.palette_residency(),
                        &image,
                    )?;
                    let material = material(&world, piece.material)?;
                    if !(8..16).contains(&material.cgram_row) {
                        return Err(ActorContentError::UnsupportedMaterialRow(piece.material));
                    }
                    let x = position.x.wrapping_add(piece.offset.x);
                    let y = position.y.wrapping_add(piece.offset.y);
                    let base = matches!(mode, ActorDrawMode::Base);
                    let offscreen = matches!(mode, ActorDrawMode::ExtendedOffscreen);
                    match palette_mode {
                        ActorPaletteMode::Normal => runtime.actor_draw_prepared(
                            bank,
                            address,
                            x,
                            y,
                            u16::from(material.cgram_row - 8) << 9,
                            tile,
                            base,
                            offscreen,
                        ),
                        // Preserve the retained override's OR with embedded palette bits.
                        ActorPaletteMode::Flash | ActorPaletteMode::Frozen => {
                            runtime.actor_draw_compatible(
                                bank, address, x, y, palette, tile, base, offscreen,
                            );
                        }
                    }
                }
            }
            ActorVisualForm::Tilemap(tilemap) => {
                copy(runtime, ActorTilemapSource::Prepared(tilemap))?
            }
        },
    }
    Ok(())
}

pub(super) fn copy(
    runtime: &mut Runtime<'_>,
    source: ActorTilemapSource,
) -> Result<(), ActorContentError> {
    match source {
        ActorTilemapSource::Compatibility { bank, address }
        | ActorTilemapSource::Unresolved { bank, address, .. } => {
            runtime.actor_copy_legacy_tilemap(bank, address);
        }
        ActorTilemapSource::Prepared(tilemap) => {
            if tilemap.extent != Point::default() || tilemap.offset != Point::default() {
                return Err(ActorContentError::UnsupportedForm);
            }
            let world = runtime.session().world.clone();
            let (bank, address) =
                resource_address(&world, tilemap.resource, ResourceKind::ActorTilemap)?;
            let image = runtime
                .loaded_image()
                .ok_or(ActorContentError::PaletteNotResident(tilemap.material))?;
            world.actor_media().validate_material_residency(
                tilemap.material,
                runtime.session().actor_identity.palette_residency(),
                &image,
            )?;
            let material = material(&world, tilemap.material)?;
            if material.cgram_row >= 8 {
                return Err(ActorContentError::UnsupportedMaterialRow(tilemap.material));
            }
            runtime.actor_copy_prepared_tilemap(bank, address, u16::from(material.cgram_row) << 10);
        }
    }
    Ok(())
}

fn resource_address(
    world: &WorldBinding,
    id: VisualResourceId,
    expected: ResourceKind,
) -> Result<(u8, u16), ActorContentError> {
    let media = world
        .actor_media()
        .definitions()
        .ok_or(ActorContentError::MissingResource(id))?;
    let resource = media
        .resources
        .get(&id)
        .ok_or(ActorContentError::MissingResource(id))?;
    let (resource, kind) = resource.binding();
    if kind != expected {
        return Err(ActorContentError::WrongResourceKind(id));
    }
    match world.resource_key(resource) {
        ResourceKey::Long(LegacyAddress24([low, high, bank])) => {
            Ok((bank, u16::from_le_bytes([low, high])))
        }
        ResourceKey::Word { .. } => Err(ActorContentError::WrongResourceKind(id)),
    }
}
fn material(
    world: &WorldBinding,
    id: MaterialId,
) -> Result<&ActorMaterialDefinition, ActorContentError> {
    world
        .actor_media()
        .definitions()
        .and_then(|definitions| definitions.materials.get(&id))
        .ok_or(ActorContentError::MissingMaterial(id))
}
