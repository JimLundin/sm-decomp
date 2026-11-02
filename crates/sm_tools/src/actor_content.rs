//! Prepare actor presentation from authored manifests and original allocation captures.
//! No current actor selector, OAM bytes, verifier or backend participates in assembly.
#![forbid(unsafe_code)]
use sm_game::{
    actors::{
        content::ActorLiveRecord,
        presentation_content::{
            ActorImportRequest, ActorManifest, AllocationRole, CANNON_CHARACTERS, CannonSourceRole,
            Capture, CaptureOffset, MANIFEST_SCHEMA, PreparedActorContent, QueueDeclaration,
            ResourceDeclaration, SourceAllocations, VariantId,
        },
    },
    persistent_session::{SessionStartError, SuppliedSessionStart},
    presentation_workspace::Point,
    room_entry::{
        prepared_assets::PreparedRoomAssets,
        world::{ResourceKind, WorldBinding},
    },
    scene::{
        actor_presentation::{Binding, ChunkSource, DataSource, Description, Resource},
        hdma_sources::MajorHdmaRole,
        records::{ObjectSize, ObjectVariants, SceneMedia},
        state::{DisplayError, SourceSpan, TransferSource},
    },
};
use std::{collections::BTreeMap, rc::Rc, sync::Arc};

fn record(capture: Capture, sources: &SourceAllocations) -> Result<ActorLiveRecord, DisplayError> {
    let address = match capture.offset {
        CaptureOffset::Linear(_) => capture.address,
        CaptureOffset::Wrapping(displacement) => capture.address.wrapping_add(displacement),
    };
    let mapped = (u32::from(capture.bank) << 16) | u32::from(address);
    let mut base = usize::try_from(sm_data::cartridge::lorom_offset(mapped))
        .map_err(|_| DisplayError::Source)?;
    if let CaptureOffset::Linear(displacement) = capture.offset {
        base = base.checked_add(displacement).ok_or(DisplayError::Source)?;
    }
    // No count, geometry, instruction or table bytes are queried here.
    Ok(ActorLiveRecord {
        image: sources.image(capture.allocation).clone(),
        base,
    })
}
fn variants(
    manifest: &ActorManifest,
    media: &SceneMedia,
) -> Result<BTreeMap<VariantId, Arc<ObjectVariants>>, DisplayError> {
    manifest
        .variants
        .iter()
        .map(|(id, sets)| {
            if id.0 == 0 || sets.is_empty() {
                return Err(DisplayError::Identity);
            }
            for (set, value) in sets {
                if *set > 7 {
                    return Err(DisplayError::Geometry);
                }
                for (binding, size) in [
                    (value.small, ObjectSize::Small),
                    (value.large, ObjectSize::Large),
                ] {
                    let part = binding
                        .definition(media)
                        .map_err(|_| DisplayError::Identity)?;
                    if part.size != size {
                        return Err(DisplayError::Geometry);
                    }
                }
            }
            Ok((
                *id,
                Arc::new(ObjectVariants {
                    size_sets: sets.clone(),
                }),
            ))
        })
        .collect()
}
fn resource(
    value: &ResourceDeclaration,
    world: &WorldBinding,
    sources: &SourceAllocations,
    variants: &BTreeMap<VariantId, Arc<ObjectVariants>>,
) -> Result<Resource, DisplayError> {
    if world
        .definition()
        .and_then(|definition| definition.resources.get(&value.identity))
        != Some(&value.kind)
        || world.resource_address(value.identity, value.kind)
            != ((u32::from(value.capture.bank) << 16) | u32::from(value.capture.address))
    {
        return Err(DisplayError::Identity);
    }
    let expected_offset = if value.kind == ResourceKind::ActorTilemap {
        CaptureOffset::Wrapping(2)
    } else {
        CaptureOffset::Linear(0)
    };
    if value.capture.offset != expected_offset {
        return Err(DisplayError::Source);
    }
    let parts: Result<Vec<_>, _> = value
        .parts
        .iter()
        .map(|id| variants.get(id).cloned().ok_or(DisplayError::Identity))
        .collect();
    Ok(Resource {
        identity: value.identity,
        kind: value.kind,
        record: record(value.capture, sources)?,
        native_order: value.order,
        parts: parts?.into(),
    })
}
fn queue(
    value: &QueueDeclaration,
    sources: &SourceAllocations,
    media: &SceneMedia,
) -> Result<TransferSource, DisplayError> {
    Ok(match value {
        QueueDeclaration::ActorState { address } => {
            TransferSource::ActorState { address: *address }
        }
        QueueDeclaration::WorkspacePhysical { address } => TransferSource::Workspace {
            point: Point::physical(*address),
        },
        QueueDeclaration::WorkspaceRole { role, offset } => TransferSource::Workspace {
            point: Point::new(*role, *offset),
        },
        QueueDeclaration::Asset {
            asset,
            capture,
            length,
            offset,
        } => {
            // Immutable imported media bytes have the exact declared original allocation.
            // Current mutable RAM/Workspace producers are never copied into this variant.
            if capture.allocation != AllocationRole::Loaded {
                return Err(DisplayError::Identity);
            }
            let origin = record(*capture, sources)?;
            let end = origin
                .base
                .checked_add(*length)
                .ok_or(DisplayError::Source)?;
            let bytes = origin
                .image
                .bytes()
                .get(origin.base..end)
                .ok_or(DisplayError::Source)?;
            let span = SourceSpan {
                asset: *asset,
                bytes: Arc::from(bytes),
                offset: *offset,
            };
            span.validate(media.revision())?;
            span.validate_media(media)?;
            TransferSource::Asset(span)
        }
        QueueDeclaration::Hud { offset } => TransferSource::Hud { offset: *offset },
        QueueDeclaration::Objects { offset } => TransferSource::Objects { offset: *offset },
        QueueDeclaration::QueuedScroll { offset } => {
            TransferSource::QueuedScroll { offset: *offset }
        }
        QueueDeclaration::Scroll {
            slot,
            half,
            wrapped,
            offset,
        } => TransferSource::Scroll {
            slot: *slot,
            half: *half,
            wrapped: *wrapped,
            offset: *offset,
        },
    })
}
fn description(
    manifest: &ActorManifest,
    world: &Arc<WorldBinding>,
    media: &SceneMedia,
    sources: &SourceAllocations,
) -> Result<Description, DisplayError> {
    if manifest.schema != MANIFEST_SCHEMA
        || manifest.world != world.revision()
        || world.actors().revision() != Some(manifest.gameplay)
        || world.actor_media().revision() != Some(manifest.actor_media)
        || media.revision() != manifest.scene_media
    {
        return Err(DisplayError::Revision);
    }
    let definitions = world
        .actor_media()
        .definitions()
        .ok_or(DisplayError::Identity)?;
    if manifest.resources.is_empty()
        || manifest.sprites.is_empty()
        || definitions.resources.keys().ne(manifest.resources.keys())
    {
        return Err(DisplayError::Incomplete);
    }
    for role in MajorHdmaRole::ALL {
        if manifest.producer_bindings.get(&role) != Some(&role.declaration()) {
            return Err(DisplayError::Identity);
        }
    }
    if manifest.producer_bindings.len() != MajorHdmaRole::ALL.len() {
        return Err(DisplayError::Identity);
    }
    // Actual actor/Kraid producers are mandatory; their IDs and media come from
    // the independently authored manifest, never from the current transfer queue.
    if manifest.queued_sources.get(&[0x00, 0x20, 0x7e])
        != Some(&QueueDeclaration::ActorState { address: 0x2000 })
    {
        return Err(DisplayError::Incomplete);
    }
    for address in [0x4000u16, 0x5000] {
        let [low, high] = address.to_le_bytes();
        if manifest.queued_sources.get(&[low, high, 0x7e])
            != Some(&QueueDeclaration::WorkspacePhysical {
                address: usize::from(address),
            })
        {
            return Err(DisplayError::Incomplete);
        }
    }
    for token in [[0x16, 0xa7, 0xa7], [0x00, 0xb2, 0x9a]] {
        let Some(QueueDeclaration::Asset { capture, .. }) = manifest.queued_sources.get(&token)
        else {
            return Err(DisplayError::Incomplete);
        };
        if capture.allocation != AllocationRole::Loaded
            || capture.bank != token[2]
            || capture.address != u16::from_le_bytes([token[0], token[1]])
            || capture.offset != CaptureOffset::Linear(0)
        {
            return Err(DisplayError::Identity);
        }
    }
    let variants = variants(manifest, media)?;
    if manifest.cannon_parts.len() != CANNON_CHARACTERS.len()
        || CANNON_CHARACTERS
            .iter()
            .any(|word| !manifest.cannon_parts.contains_key(word))
        || manifest.cannon_sources.len() != CannonSourceRole::ALL.len()
    {
        return Err(DisplayError::Incomplete);
    }
    let cannon_parts = manifest
        .cannon_parts
        .iter()
        .map(|(character, id)| {
            let parts = variants.get(id).cloned().ok_or(DisplayError::Identity)?;
            parts
                .validate(media, character & 255)
                .map_err(|_| DisplayError::Scene)?;
            Ok((*character, parts))
        })
        .collect::<Result<_, DisplayError>>()?;
    let cannon_sources = CannonSourceRole::ALL
        .into_iter()
        .map(|role| {
            let capture = *manifest
                .cannon_sources
                .get(&role)
                .ok_or(DisplayError::Incomplete)?;
            if capture != role.capture() {
                return Err(DisplayError::Identity);
            }
            // Retain the actual loaded/native allocation, without reading future
            // pointers, frames, heights or diagnostic-selected native records.
            Ok((role, record(capture, sources)?))
        })
        .collect::<Result<_, DisplayError>>()?;
    let resources = manifest
        .resources
        .iter()
        .map(|(id, value)| Ok((*id, resource(value, world, sources, &variants)?)))
        .collect::<Result<_, DisplayError>>()?;
    let shared = |declarations: &BTreeMap<(u8, u16), ResourceDeclaration>, allocation| {
        declarations
            .iter()
            .map(|(key, value)| {
                if *key != (value.capture.bank, value.capture.address)
                    || value.capture.allocation != allocation
                    || value.kind != ResourceKind::ActorSprites
                {
                    return Err(DisplayError::Identity);
                }
                Ok((*key, resource(value, world, sources, &variants)?))
            })
            .collect::<Result<_, DisplayError>>()
    };
    let sprite_sources = shared(&manifest.sprites, AllocationRole::Loaded)?;
    let room_sprite_sources = shared(&manifest.room_sprites, AllocationRole::Native)?;
    let chunk_sources = manifest
        .chunks
        .iter()
        .map(|value| {
            media
                .asset(value.media)
                .map_err(|_| DisplayError::Identity)?;
            let origin = record(value.capture, sources)?;
            if origin
                .base
                .checked_add(value.extent)
                .ok_or(DisplayError::Source)?
                > origin.image.len()
            {
                return Err(DisplayError::Source);
            }
            Ok(ChunkSource {
                media: value.media,
                origin,
                extent: value.extent,
            })
        })
        .collect::<Result<_, DisplayError>>()?;
    let data_sources = manifest
        .data
        .iter()
        .map(|(role, value)| {
            if value.capture.allocation != AllocationRole::Loaded
                || value.capture.offset != CaptureOffset::Linear(0)
                || world
                    .definition()
                    .and_then(|definition| definition.resources.get(&value.identity))
                    != Some(&ResourceKind::TileTable)
                || world.resource_address(value.identity, ResourceKind::TileTable)
                    != ((u32::from(value.capture.bank) << 16) | u32::from(value.capture.address))
            {
                return Err(DisplayError::Identity);
            }
            Ok((
                *role,
                DataSource {
                    identity: value.identity,
                    record: record(value.capture, sources)?,
                    native_order: value.order,
                },
            ))
        })
        .collect::<Result<_, DisplayError>>()?;
    let trail_parts = manifest
        .trails
        .iter()
        .map(|(key, id)| {
            Ok((
                *key,
                variants.get(id).cloned().ok_or(DisplayError::Identity)?,
            ))
        })
        .collect::<Result<_, DisplayError>>()?;
    let queued_sources = manifest
        .queued_sources
        .iter()
        .map(|(token, value)| {
            if token[2] == 0x9a {
                let QueueDeclaration::Asset { capture, .. } = value else {
                    return Err(DisplayError::Identity);
                };
                if capture.allocation != AllocationRole::Loaded
                    || capture.bank != token[2]
                    || capture.address != u16::from_le_bytes([token[0], token[1]])
                    || capture.offset != CaptureOffset::Linear(0)
                {
                    return Err(DisplayError::Identity);
                }
            }
            if let QueueDeclaration::ActorState { address } = value {
                if token[2] != 0x7e || *address != u16::from_le_bytes([token[0], token[1]]) {
                    return Err(DisplayError::Identity);
                }
            }
            Ok((*token, queue(value, sources, media)?))
        })
        .collect::<Result<_, DisplayError>>()?;
    Ok(Description {
        world: world.clone(),
        actor_media: manifest.actor_media,
        scene_media: manifest.scene_media,
        resources,
        sprite_sources,
        room_sprite_sources,
        chunk_sources,
        data_sources,
        trail_parts,
        cannon_parts,
        cannon_sources,
        grapple_segment: variants
            .get(&manifest.grapple_segment)
            .cloned()
            .ok_or(DisplayError::Identity)?,
        grapple_end: variants
            .get(&manifest.grapple_end)
            .cloned()
            .ok_or(DisplayError::Identity)?,
        compatible: manifest.compatible.clone(),
        queued_sources,
        producer_bindings: manifest.producer_bindings.clone(),
    })
}
/// Two independently supplied manifests are assembled separately over the exact
/// observed original allocations; expected declarations never come from selectors.
pub fn prepare(
    request: ActorImportRequest,
    world: &Arc<WorldBinding>,
    media: &SceneMedia,
    sources: SourceAllocations,
) -> Result<PreparedActorContent, DisplayError> {
    let expected = request.contract.expected();
    if request.supplied != *expected {
        return Err(DisplayError::Identity);
    }
    if request.supplied.loaded_source == [0; 32]
        || request.supplied.native_source == [0; 32]
        || PreparedRoomAssets::checksum(sources.loaded.bytes()) != expected.loaded_source
        || PreparedRoomAssets::checksum(sources.native.bytes()) != expected.native_source
    {
        return Err(DisplayError::Identity);
    }
    let supplied = description(&request.supplied, world, media, &sources)?;
    let expected = description(expected, world, media, &sources)?;
    let binding = Binding::construct(supplied, &expected, media)?;
    PreparedActorContent::new(binding, request.supplied, sources)
}
/// Actual headless boot/replay content boundary. Consume the raw manifest request
/// before target admission, retaining its checked immutable content in the contract.
pub fn prepare_session(
    mut supplied: SuppliedSessionStart,
    world: &Arc<WorldBinding>,
    sources: SourceAllocations,
) -> Result<SuppliedSessionStart, SessionStartError> {
    let request = supplied.take_actor_import();
    let (mut description, media, mut required) = supplied.into_parts();
    let request = request.or_else(|| {
        required
            .actor_manifest
            .as_ref()
            .map(|original| original.as_ref().clone())
    });
    if let Some(request) = request {
        if required
            .actor_manifest
            .as_ref()
            .is_some_and(|prior| prior.as_ref() != &request)
        {
            return Err(SessionStartError::Binding(DisplayError::Identity));
        }
        let original = request.clone();
        let content =
            prepare(request, world, &media, sources.clone()).map_err(SessionStartError::Binding)?;
        if description
            .actor_content
            .as_ref()
            .is_some_and(|prior| prior.declarations() != content.declarations())
        {
            return Err(SessionStartError::Binding(DisplayError::Identity));
        }
        description.actor_content = Some(Rc::new(content));
        required.actor_manifest = Some(Arc::new(original));
    }
    if let Some(content) = &description.actor_content {
        content
            .check_sources(&sources)
            .map_err(SessionStartError::Binding)?;
        content
            .validate_request(
                required
                    .actor_manifest
                    .as_ref()
                    .ok_or(SessionStartError::MissingActorContent)?,
            )
            .map_err(SessionStartError::Binding)?;
    } else if world.actors().revision().is_some() || required.actor_manifest.is_some() {
        return Err(SessionStartError::MissingActorContent);
    }
    Ok(SuppliedSessionStart::new(description, media, required))
}
