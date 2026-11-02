//! Actor publications into the existing display, with explicit allocation origins.
//! Native coordinate declarations remain compatibility inputs; no staging mirror lives here.
#![forbid(unsafe_code)]

use super::{
    objects,
    records::{ObjectContext, ObjectVariants, SceneMedia},
    state::{DisplayError, DisplayPhase, TransferSource, WordOrder},
};
use crate::{
    actors::{
        content::ActorLiveRecord,
        media::{MaterialId, MediaRevision, VisualResourceId},
        presentation_content::{CANNON_CHARACTERS, CannonSourceRole},
        scene::{
            ActorDrawMode, ActorDrawRequest, ActorPaletteMode, ActorTileChunkSource,
            ActorTileDestination, ActorTilemapSource, ActorVisualForm, ActorVisualSelection,
        },
    },
    rom::RomImage,
    room_entry::world::{ResourceId, ResourceKind, WorldBinding},
    session::Session,
};
use std::{collections::BTreeMap, sync::Arc};
mod projectiles;
pub use projectiles::EnemySpriteRequest;

/// Captures are selected at their original observation point. Diagnostic return
/// precedes loaded capture, and can change that selection. Native allocation
/// aliases remain an explicit compatibility dependency, never an owned mirror.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpriteAllocation {
    Loaded,
    Native,
}

pub trait Diagnostics {
    fn invalid_sprite(
        &mut self,
        session: &mut Session,
        source: SpriteAllocation,
        bank: u8,
        address: u16,
    ) -> Result<(), DisplayError>;
    fn loaded_image(&self) -> RomImage;
    fn native_image(&self) -> RomImage;
    fn alias_byte(&self, session: &Session, address: usize) -> Result<u8, DisplayError>;
    fn write_alias_byte(
        &mut self,
        session: &mut Session,
        address: usize,
        value: u8,
    ) -> Result<(), DisplayError>;
    fn object_context(
        &mut self,
        owner: &super::commit::OwnedDisplay,
    ) -> Result<ObjectContext, DisplayError>;
    fn draw_echo_sprite(
        &mut self,
        session: &mut Session,
        observations: &crate::external_inputs::ExternalInputs,
        content: &crate::scene::prepared_samus::PreparedSamusContent,
        echo: usize,
        index: u16,
        position: crate::geometry::Point,
    ) -> Result<(), DisplayError>;
}

/// The exact allocation and linear base admitted by the offline resource owner.
/// Tilemap records start at the captured bank/address.wrapping_add(2) payload.
/// A same-byte allocation is not interchangeable with this registered capture.
#[derive(Clone)]
pub struct Resource {
    pub identity: ResourceId,
    pub kind: ResourceKind,
    pub record: ActorLiveRecord,
    pub native_order: WordOrder,
    pub parts: Arc<[Arc<ObjectVariants>]>,
}

/// Declared original chunk allocation and actual media resource provenance.
/// Extent belongs to this captured origin, rather than a nominal eight-byte chunk.
pub struct ChunkSource {
    pub media: super::records::AssetRef,
    pub origin: ActorLiveRecord,
    pub extent: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum DataRole {
    BeamPointers,
    EchoPoseHeights,
}
pub struct DataSource {
    pub identity: ResourceId,
    pub record: ActorLiveRecord,
    pub native_order: WordOrder,
}

pub struct Description {
    pub producer_bindings:
        BTreeMap<super::hdma_sources::MajorHdmaRole, super::hdma_sources::SourceBinding>,
    pub world: Arc<WorldBinding>,
    pub actor_media: MediaRevision,
    pub scene_media: super::records::Revision,
    pub resources: BTreeMap<VisualResourceId, Resource>,
    /// Actual shared sprite resources, independently authored from actor appearances.
    pub sprite_sources: BTreeMap<(u8, u16), Resource>,
    /// Native room-sprite captures can differ from loaded captures at the same key.
    pub room_sprite_sources: BTreeMap<(u8, u16), Resource>,
    pub chunk_sources: Vec<ChunkSource>,
    pub data_sources: BTreeMap<DataRole, DataSource>,
    /// Authored trail-frame identities, keyed by the original following program cursor.
    pub trail_parts: BTreeMap<(bool, u16), Arc<ObjectVariants>>,
    /// Independently authored semantic counterparts for complete cannon words.
    pub cannon_parts: BTreeMap<u16, Arc<ObjectVariants>>,
    /// Original lazy table origins on their actual loaded/native allocations.
    pub cannon_sources: BTreeMap<CannonSourceRole, ActorLiveRecord>,
    pub grapple_segment: Arc<ObjectVariants>,
    pub grapple_end: Arc<ObjectVariants>,
    /// Supplied unknown-appearance mappings, independent of the current actor selector.
    pub compatible: BTreeMap<(u8, u16, bool), VisualResourceId>,
    /// Actual packed producer identities, including imported nonactor resources.
    pub queued_sources: BTreeMap<[u8; 3], TransferSource>,
}

/// Immutable resources only. The mutable bytes and queue remain in Session.
pub struct Binding(Description);

struct DrawParameters {
    position: crate::geometry::Point,
    palette: u16,
    tile: u16,
    normal: bool,
    clip: objects::Clip,
}

pub struct SpriteRequest {
    pub bank: u8,
    pub map: u16,
    pub position: crate::geometry::Point,
    pub palette: u16,
    pub base_tile: u16,
    pub clip: objects::Clip,
}

/// A boss spritemap in room coordinates, with the original carry and row policy.
pub struct RoomSpriteRequest {
    pub bank: u8,
    pub map: u16,
    pub position: crate::geometry::Point,
    pub palette: u16,
    pub clip: objects::RoomClip,
}

impl Binding {
    pub fn construct(
        supplied: Description,
        expected: &Description,
        media: &SceneMedia,
    ) -> Result<Self, DisplayError> {
        if !Arc::ptr_eq(&supplied.world, &expected.world)
            || supplied.actor_media != expected.actor_media
            || supplied.scene_media != expected.scene_media
            || supplied.compatible != expected.compatible
            || supplied.queued_sources != expected.queued_sources
            || supplied.producer_bindings != expected.producer_bindings
            || supplied.resources.len() != expected.resources.len()
            || supplied.sprite_sources.len() != expected.sprite_sources.len()
            || supplied.room_sprite_sources.len() != expected.room_sprite_sources.len()
            || supplied.data_sources.len() != expected.data_sources.len()
            || supplied.trail_parts != expected.trail_parts
            || supplied.cannon_parts != expected.cannon_parts
            || supplied.cannon_sources.len() != expected.cannon_sources.len()
            || supplied.grapple_segment != expected.grapple_segment
            || supplied.grapple_end != expected.grapple_end
            || supplied.chunk_sources.len() != expected.chunk_sources.len()
        {
            return Err(DisplayError::Identity);
        }
        if supplied.cannon_parts.len() != CANNON_CHARACTERS.len()
            || supplied.cannon_sources.len() != CannonSourceRole::ALL.len()
        {
            return Err(DisplayError::Incomplete);
        }
        for character in CANNON_CHARACTERS {
            supplied
                .cannon_parts
                .get(&character)
                .ok_or(DisplayError::Incomplete)?
                .validate(media, character & 255)
                .map_err(|_| DisplayError::Scene)?;
        }
        for role in CannonSourceRole::ALL {
            let actual = supplied
                .cannon_sources
                .get(&role)
                .ok_or(DisplayError::Incomplete)?;
            let required = expected
                .cannon_sources
                .get(&role)
                .ok_or(DisplayError::Incomplete)?;
            let capture = role.capture();
            let address = (u32::from(capture.bank) << 16) | u32::from(capture.address);
            let base = usize::try_from(sm_data::cartridge::lorom_offset(address))
                .map_err(|_| DisplayError::Source)?;
            if actual.base != base
                || actual.base != required.base
                || !actual.image.same_allocation(&required.image)
            {
                return Err(DisplayError::Identity);
            }
        }
        for (&id, resource) in &supplied.resources {
            let required = expected.resources.get(&id).ok_or(DisplayError::Identity)?;
            if resource.identity != required.identity
                || resource.kind != required.kind
                || resource.native_order != required.native_order
                || resource.parts != required.parts
                || resource.record.base != required.record.base
                || !resource
                    .record
                    .image
                    .same_allocation(&required.record.image)
            {
                return Err(DisplayError::Identity);
            }
        }
        for (source, required) in supplied.chunk_sources.iter().zip(&expected.chunk_sources) {
            if source.media != required.media
                || source.extent != required.extent
                || source.origin.base != required.origin.base
                || !source.origin.image.same_allocation(&required.origin.image)
            {
                return Err(DisplayError::Identity);
            }
        }
        for (sources, required_sources) in [
            (&supplied.sprite_sources, &expected.sprite_sources),
            (&supplied.room_sprite_sources, &expected.room_sprite_sources),
        ] {
            for (key, source) in sources {
                let required = required_sources.get(key).ok_or(DisplayError::Identity)?;
                if source.identity != required.identity
                    || source.kind != required.kind
                    || source.native_order != required.native_order
                    || source.parts != required.parts
                    || source.record.base != required.record.base
                    || !source.record.image.same_allocation(&required.record.image)
                {
                    return Err(DisplayError::Identity);
                }
                let (bank, map) = *key;
                if supplied
                    .world
                    .definition()
                    .and_then(|world| world.resources.get(&source.identity))
                    != Some(&ResourceKind::ActorSprites)
                    || supplied
                        .world
                        .resource_address(source.identity, ResourceKind::ActorSprites)
                        != ((u32::from(bank) << 16) | u32::from(map))
                {
                    return Err(DisplayError::Identity);
                }
            }
        }
        for role in [DataRole::BeamPointers, DataRole::EchoPoseHeights] {
            let source = supplied
                .data_sources
                .get(&role)
                .ok_or(DisplayError::Identity)?;
            let required = expected
                .data_sources
                .get(&role)
                .ok_or(DisplayError::Identity)?;
            if source.identity != required.identity
                || source.native_order != required.native_order
                || source.record.base != required.record.base
                || !source.record.image.same_allocation(&required.record.image)
            {
                return Err(DisplayError::Identity);
            }
            let address = match role {
                DataRole::BeamPointers => sm_data::menu_lists::BEAM_SPRITEMAPS,
                DataRole::EchoPoseHeights => 0x91b629,
            };
            if supplied
                .world
                .definition()
                .and_then(|world| world.resources.get(&source.identity))
                != Some(&ResourceKind::TileTable)
                || supplied
                    .world
                    .resource_address(source.identity, ResourceKind::TileTable)
                    != address
            {
                return Err(DisplayError::Identity);
            }
        }
        if supplied.world.actor_media().revision() != Some(supplied.actor_media)
            || supplied.scene_media != media.revision()
        {
            return Err(DisplayError::Revision);
        }
        let definitions = supplied
            .world
            .actor_media()
            .definitions()
            .ok_or(DisplayError::Identity)?;
        for (&id, resource) in &supplied.resources {
            let actual = definitions
                .resources
                .get(&id)
                .ok_or(DisplayError::Identity)?
                .binding();
            if actual != (resource.identity, resource.kind) {
                return Err(DisplayError::Identity);
            }
            if resource.record.base > resource.record.image.len() {
                return Err(DisplayError::Source);
            }
        }
        for (&(bank, address, tilemap), &id) in &supplied.compatible {
            let resource = supplied.resources.get(&id).ok_or(DisplayError::Identity)?;
            let kind = if tilemap {
                ResourceKind::ActorTilemap
            } else {
                ResourceKind::ActorSprites
            };
            if resource.kind != kind
                || supplied.world.resource_address(resource.identity, kind)
                    != ((u32::from(bank) << 16) | u32::from(address))
            {
                return Err(DisplayError::Identity);
            }
        }
        // These mutable boss allocations are current owners, never immutable media captures.
        for (token, address) in [
            ([0, 0x20, 0x7e], 0x2000u16),
            ([0, 0x40, 0x7e], 0x4000),
            ([0, 0x50, 0x7e], 0x5000),
        ] {
            let source = supplied
                .queued_sources
                .get(&token)
                .ok_or(DisplayError::Source)?;
            match address {
                0x2000 if !matches!(source,TransferSource::ActorState{address:bound} if *bound==address) =>
                {
                    return Err(DisplayError::Identity);
                }
                0x4000 | 0x5000
                    if source
                        != &(TransferSource::Workspace {
                            point: crate::presentation_workspace::Point::physical(usize::from(
                                address,
                            )),
                        }) =>
                {
                    return Err(DisplayError::Identity);
                }
                _ => {}
            }
        }
        for token in [[0x16, 0xa7, 0xa7], [0, 0xb2, 0x9a]] {
            if !matches!(
                supplied.queued_sources.get(&token),
                Some(TransferSource::Asset(_))
            ) {
                return Err(DisplayError::Identity);
            }
        }
        for (&token, source) in &supplied.queued_sources {
            let address = u16::from_le_bytes([token[0], token[1]]);
            if token[2] == 0x7e
                && ((0x2000..0x3000).contains(&address) || (0x7000..0x9800).contains(&address))
                && !matches!(source, TransferSource::ActorState { address: bound } if *bound == address)
            {
                // These bytes have one current actor/Botwoon/eproj owner, not a media copy.
                return Err(DisplayError::Identity);
            }
            if let TransferSource::Asset(span) = source {
                span.validate_media(media)?;
            }
        }
        for source in &supplied.chunk_sources {
            media
                .asset(source.media)
                .map_err(|_| DisplayError::Identity)?;
            let end = source
                .origin
                .base
                .checked_add(source.extent)
                .ok_or(DisplayError::Source)?;
            if end > source.origin.image.len() {
                return Err(DisplayError::Source);
            }
        }
        Ok(Self(supplied))
    }

    pub fn check(&self, session: &Session) -> Result<(), DisplayError> {
        if session
            .presentation
            .owner()?
            .state
            .hdma_live
            .producer_bindings()
            != &self.0.producer_bindings
        {
            return Err(DisplayError::Source);
        }
        if !Arc::ptr_eq(&self.0.world, &session.world)
            || session.world.actor_media().revision() != Some(self.0.actor_media)
            || session.presentation.owner()?.media().revision() != self.0.scene_media
        {
            return Err(DisplayError::Identity);
        }
        Ok(())
    }

    fn resource(
        &self,
        id: VisualResourceId,
        kind: ResourceKind,
        image: RomImage,
    ) -> Result<Resource, DisplayError> {
        let resource = self.0.resources.get(&id).ok_or(DisplayError::Identity)?;
        if resource.kind != kind || !resource.record.image.same_allocation(&image) {
            return Err(DisplayError::Identity);
        }
        Ok(resource.clone())
    }

    fn compatible(
        &self,
        bank: u8,
        address: u16,
        tilemap: bool,
        image: RomImage,
    ) -> Result<Resource, DisplayError> {
        let id = self
            .0
            .compatible
            .get(&(bank, address, tilemap))
            .ok_or(DisplayError::Identity)?;
        self.resource(
            *id,
            if tilemap {
                ResourceKind::ActorTilemap
            } else {
                ResourceKind::ActorSprites
            },
            image,
        )
    }

    fn material(
        &self,
        session: &Session,
        id: MaterialId,
        image: &RomImage,
        object: bool,
    ) -> Result<u16, DisplayError> {
        self.0
            .world
            .actor_media()
            .validate_material_residency(id, session.actor_identity.palette_residency(), image)
            .map_err(|_| DisplayError::Identity)?;
        let material = self
            .0
            .world
            .actor_media()
            .definitions()
            .and_then(|definitions| definitions.materials.get(&id))
            .ok_or(DisplayError::Identity)?;
        if object {
            if !(8..16).contains(&material.cgram_row) {
                return Err(DisplayError::Geometry);
            }
            Ok(u16::from(material.cgram_row - 8) << 9)
        } else {
            if material.cgram_row >= 8 {
                return Err(DisplayError::Geometry);
            }
            Ok(u16::from(material.cgram_row) << 10)
        }
    }

    pub fn draw(
        &self,
        session: &mut Session,
        request: ActorDrawRequest,
        diagnostics: &mut dyn Diagnostics,
    ) -> Result<(), DisplayError> {
        self.check(session)?;
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
                if source.address == 0 && source.mode == ActorDrawMode::Base {
                    return Ok(());
                }
                if source.address & 0x8000 == 0 {
                    diagnostics.invalid_sprite(
                        session,
                        SpriteAllocation::Loaded,
                        source.bank,
                        source.address,
                    )?;
                }
                let resource = self.compatible(
                    source.bank,
                    source.address,
                    false,
                    diagnostics.loaded_image(),
                )?;
                self.draw_resource(
                    session,
                    resource,
                    DrawParameters {
                        position,
                        palette,
                        tile,
                        normal: false,
                        clip: actor_clip(source.mode),
                    },
                    diagnostics,
                )
            }
            ActorVisualSelection::Prepared(visual) => match visual.form {
                ActorVisualForm::Tilemap(tilemap) => self.copy_tilemap_data(
                    session,
                    ActorTilemapSource::Prepared(tilemap),
                    diagnostics,
                ),
                ActorVisualForm::Sprites(pieces) => {
                    for piece in pieces {
                        if piece.source_origin != crate::geometry::Point::default()
                            || piece.source_extent != crate::geometry::Point::default()
                            || piece.flip_x
                            || piece.flip_y
                        {
                            return Err(DisplayError::Geometry);
                        }
                        // Resource identity precedes residency; capture follows the warning.
                        let declared = self
                            .0
                            .resources
                            .get(&piece.resource)
                            .ok_or(DisplayError::Identity)?;
                        let address = self
                            .0
                            .world
                            .resource_address(declared.identity, ResourceKind::ActorSprites);
                        let image = diagnostics.loaded_image();
                        let normal = self.material(session, piece.material, &image, true)?;
                        let bytes = address.to_le_bytes();
                        let cursor = u16::from_le_bytes([bytes[0], bytes[1]]);
                        if mode == ActorDrawMode::Base && cursor == 0 {
                            continue;
                        }
                        if cursor & 0x8000 == 0 {
                            diagnostics.invalid_sprite(
                                session,
                                SpriteAllocation::Loaded,
                                bytes[2],
                                cursor,
                            )?;
                        }
                        let resource = self.resource(
                            piece.resource,
                            ResourceKind::ActorSprites,
                            diagnostics.loaded_image(),
                        )?;
                        let normal_palette = palette_mode == ActorPaletteMode::Normal;
                        self.draw_resource(
                            session,
                            resource,
                            DrawParameters {
                                position: crate::geometry::Point {
                                    x: position.x.wrapping_add(piece.offset.x),
                                    y: position.y.wrapping_add(piece.offset.y),
                                },
                                palette: if normal_palette { normal } else { palette },
                                tile,
                                normal: normal_palette,
                                clip: actor_clip(mode),
                            },
                            diagnostics,
                        )?;
                    }
                    Ok(())
                }
            },
        }
    }

    fn draw_resource(
        &self,
        session: &mut Session,
        resource: Resource,
        parameters: DrawParameters,
        diagnostics: &mut dyn Diagnostics,
    ) -> Result<(), DisplayError> {
        let DrawParameters {
            position,
            palette,
            tile,
            normal,
            clip,
        } = parameters;
        let owner = session.presentation.owner_mut()?;
        let media = owner.media().clone();
        super::delivery::in_owner_phase(owner, DisplayPhase::Producing, |owner| {
            let producer = owner
                .state
                .producer_ids
                .allocate()
                .map_err(|_| DisplayError::Identity)?;
            let mut geometry = Geometry(resource.clone());
            let mut host = ActorObjects { owner, diagnostics };
            objects::draw_with_publication(
                &mut host,
                &mut geometry,
                (position.x, position.y),
                clip,
                |word| {
                    Ok(if normal {
                        (word.wrapping_add(tile) & 0xf1ff) | palette
                    } else {
                        word.wrapping_add(tile) | palette
                    })
                },
                |host, index, entry, x, y, word| {
                    let variants = resource.parts.get(index).ok_or(DisplayError::Source)?;
                    let context = host.diagnostics.object_context(host.owner)?;
                    super::samus::publish_object(
                        &mut host.owner.state,
                        &media,
                        super::samus::ObjectPublication {
                            producer,
                            variants: variants.clone(),
                            context,
                            entry,
                            position: (x, y),
                            word,
                        },
                    )
                },
            )
        })
    }

    pub fn submit_sprite(
        &self,
        session: &mut Session,
        request: crate::sprite_effect_state::SpriteDraw,
        diagnostics: &mut dyn Diagnostics,
    ) -> Result<(), DisplayError> {
        self.draw_sprite(
            session,
            SpriteRequest {
                bank: 0xb4,
                map: request.map.0,
                position: request.position,
                palette: request.palette,
                base_tile: request.base_tile,
                clip: objects::Clip::None,
            },
            diagnostics,
        )
    }

    pub fn cannon_tile(
        &self,
        session: &mut Session,
        x: u16,
        y: u16,
        body_y: u16,
        character: u16,
        diagnostics: &mut dyn Diagnostics,
    ) -> Result<(), DisplayError> {
        self.check(session)?;
        let mut host = RoomObjects::new(session, diagnostics)?;
        let saved = objects::Host::read_little_word(&mut host, 544)?;
        let player_x = host.session.player.x;
        let camera_x = u16::from_le_bytes([
            read_byte(host.session, host.diagnostics, 0x911)?,
            read_byte(host.session, host.diagnostics, 0x912)?,
        ]);
        let x = x.wrapping_add(player_x).wrapping_sub(camera_x);
        if x < 256 {
            let entry = usize::from(objects::Host::read_little_word(&mut host, 544)?);
            objects::Host::write_byte(&mut host, entry, x.to_le_bytes()[0])?;
            // The completed X write can alias the freshly selected player/camera.
            let player_y = host.session.player.y;
            let camera_y = u16::from_le_bytes([
                read_byte(host.session, host.diagnostics, 0x915)?,
                read_byte(host.session, host.diagnostics, 0x916)?,
            ]);
            let y = y
                .wrapping_add(player_y)
                .wrapping_sub(body_y)
                .wrapping_sub(camera_y);
            if y < 256 {
                objects::Host::write_byte(&mut host, entry + 1, y.to_le_bytes()[0])?;
                // Runtime::write_slice_native checked extent, without reading bytes.
                if 0x370 + entry + 4 > 0x20000 {
                    return Err(DisplayError::Alias { offset: entry + 2 });
                }
                let encoded = host.order()?.encode(character);
                objects::Host::write_byte(&mut host, entry + 2, encoded[0])?;
                objects::Host::write_byte(&mut host, entry + 3, encoded[1])?;
                let variants = self
                    .0
                    .cannon_parts
                    .get(&character)
                    .ok_or(DisplayError::Source)?
                    .clone();
                let owner = host.session.presentation.owner_mut()?;
                let context = host.diagnostics.object_context(owner)?;
                let producer = owner
                    .state
                    .producer_ids
                    .allocate()
                    .map_err(|_| DisplayError::Identity)?;
                let media = owner.media().clone();
                super::samus::publish_object(
                    &mut owner.state,
                    &media,
                    super::samus::ObjectPublication {
                        producer,
                        variants,
                        context,
                        entry,
                        position: (x, y),
                        word: character,
                    },
                )?;
                for (offset, byte) in saved.wrapping_add(4).to_le_bytes().into_iter().enumerate() {
                    objects::Host::write_byte(&mut host, 544 + offset, byte)?;
                }
            }
        }
        host.armed = false;
        Ok(())
    }

    /// Complete shared sprite emission, including ordinary projectile clip policy.
    pub fn draw_sprite(
        &self,
        session: &mut Session,
        request: SpriteRequest,
        diagnostics: &mut dyn Diagnostics,
    ) -> Result<(), DisplayError> {
        self.check(session)?;
        if request.map == 0 && request.clip == objects::Clip::None {
            return Ok(());
        }
        if request.map & 0x8000 == 0 {
            diagnostics.invalid_sprite(
                session,
                SpriteAllocation::Loaded,
                request.bank,
                request.map,
            )?;
        }
        let image = diagnostics.loaded_image();
        let resource = self
            .0
            .sprite_sources
            .get(&(request.bank, request.map))
            .ok_or(DisplayError::Identity)?;
        if resource.kind != ResourceKind::ActorSprites
            || !resource.record.image.same_allocation(&image)
        {
            return Err(DisplayError::Identity);
        }
        self.draw_resource(
            session,
            resource.clone(),
            DrawParameters {
                position: request.position,
                palette: request.palette,
                tile: request.base_tile,
                normal: false,
                clip: request.clip,
            },
            diagnostics,
        )
    }

    /// Native content acquisition precedes the current camera query, as in the
    /// Mother Brain and Ridley routines. Map zero still reports and reads content.
    pub fn draw_room_sprite(
        &self,
        session: &mut Session,
        request: RoomSpriteRequest,
        diagnostics: &mut dyn Diagnostics,
    ) -> Result<(), DisplayError> {
        self.check(session)?;
        if request.map & 0x8000 == 0 {
            diagnostics.invalid_sprite(
                session,
                SpriteAllocation::Native,
                request.bank,
                request.map,
            )?;
        }
        let image = diagnostics.native_image();
        let resource = self
            .0
            .room_sprite_sources
            .get(&(request.bank, request.map))
            .ok_or(DisplayError::Identity)?;
        if resource.kind != ResourceKind::ActorSprites
            || !resource.record.image.same_allocation(&image)
        {
            return Err(DisplayError::Identity);
        }
        let layer1 = session.room.camera.layer1;
        let mut host = RoomObjects::new(session, diagnostics)?;
        let producer = host
            .session
            .presentation
            .owner_mut()?
            .state
            .producer_ids
            .allocate()
            .map_err(|_| DisplayError::Identity)?;
        let mut geometry = Geometry(resource.clone());
        let result = objects::draw_room_with_publication(
            &mut host,
            &mut geometry,
            (request.position.x, request.position.y),
            (layer1[0], layer1[1]),
            request.palette,
            request.clip,
            |host, record, entry, x, y, word| {
                let variants = resource.parts.get(record).ok_or(DisplayError::Source)?;
                let owner = host.session.presentation.owner_mut()?;
                let context = host.diagnostics.object_context(owner)?;
                let media = owner.media().clone();
                super::samus::publish_object(
                    &mut owner.state,
                    &media,
                    super::samus::ObjectPublication {
                        producer,
                        variants: variants.clone(),
                        context,
                        entry,
                        position: (x, y),
                        word,
                    },
                )
            },
        );
        if result.is_ok() {
            host.armed = false;
        }
        result
    }

    pub fn copy_tilemap(
        &self,
        session: &mut Session,
        source: ActorTilemapSource,
        diagnostics: &mut dyn Diagnostics,
    ) -> Result<(), DisplayError> {
        self.copy_tilemap_data(session, source, diagnostics)?;
        session.actors.drawing.tilemap_pending =
            session.actors.drawing.tilemap_pending.wrapping_add(1);
        Ok(())
    }

    fn copy_tilemap_data(
        &self,
        session: &mut Session,
        source: ActorTilemapSource,
        diagnostics: &mut dyn Diagnostics,
    ) -> Result<(), DisplayError> {
        self.check(session)?;
        let (resource, palette) = match source {
            ActorTilemapSource::Compatibility { bank, address }
            | ActorTilemapSource::Unresolved { bank, address, .. } => (
                self.compatible(bank, address, true, diagnostics.native_image())?,
                None,
            ),
            ActorTilemapSource::Prepared(tilemap) => {
                if tilemap.extent != crate::geometry::Point::default()
                    || tilemap.offset != crate::geometry::Point::default()
                {
                    return Err(DisplayError::Geometry);
                }
                let declared = self
                    .0
                    .resources
                    .get(&tilemap.resource)
                    .ok_or(DisplayError::Identity)?;
                if declared.kind != ResourceKind::ActorTilemap {
                    return Err(DisplayError::Identity);
                }
                let palette = self.material(
                    session,
                    tilemap.material,
                    &diagnostics.loaded_image(),
                    false,
                )?;
                (
                    self.resource(
                        tilemap.resource,
                        ResourceKind::ActorTilemap,
                        diagnostics.native_image(),
                    )?,
                    Some(palette),
                )
            }
        };
        // The supplied payload base already reflects the original wrapped capture.
        let mut record = resource.record.clone();
        loop {
            let destination = usize::from(word(&record, 0, resource.native_order)?);
            if destination == 0xffff {
                return Ok(());
            }
            let count = usize::from(word(&record, 2, resource.native_order)?) * 2;
            record.base = record.base.checked_add(4).ok_or(DisplayError::Source)?;
            let end = destination.checked_add(count).ok_or(DisplayError::Source)?;
            if end > 0x20000 {
                return Err(DisplayError::Alias { offset: end });
            }
            for index in 0..count {
                let mut value = byte(&record, index)?;
                if index & 1 != 0
                    && let Some(bits) = palette
                {
                    value = (value & !0x1c) | bits.to_le_bytes()[1];
                }
                write_byte(session, diagnostics, destination + index, value)?;
            }
            record.base = record.base.checked_add(count).ok_or(DisplayError::Source)?;
        }
    }

    pub fn copy_chunk(
        &self,
        session: &mut Session,
        source: ActorTileChunkSource,
        destination: ActorTileDestination,
        diagnostics: &mut dyn Diagnostics,
    ) -> Result<(), DisplayError> {
        self.check(session)?;
        let declared = self
            .0
            .chunk_sources
            .iter()
            .find(|declared| {
                declared.origin.image.same_allocation(&source.record.image)
                    && source.record.base >= declared.origin.base
                    && source.record.base - declared.origin.base < declared.extent
            })
            .ok_or(DisplayError::Identity)?;
        // Provenance is an admitted resource identity, never inferred by byte equality.
        session
            .presentation
            .owner()?
            .media()
            .asset(declared.media)
            .map_err(|_| DisplayError::Identity)?;
        // All source reads precede the destination slice preflight, as in the original.
        let mut bytes = [0; 8];
        for (index, value) in bytes.iter_mut().enumerate() {
            *value = byte(&source.record, index)?;
        }
        let start = 0x7000 + usize::from(destination.0);
        if start + bytes.len() > 0x20000 {
            return Err(DisplayError::Alias { offset: start });
        }
        for (index, value) in bytes.into_iter().enumerate() {
            write_byte(session, diagnostics, start + index, value)?;
        }
        Ok(())
    }

    pub fn queue_word(
        &self,
        session: &mut Session,
        cursor: u16,
        field: usize,
        value: u16,
        diagnostics: &mut dyn Diagnostics,
    ) -> Result<(), DisplayError> {
        let at = 0xd0 + usize::from(cursor) + field;
        if at + 2 > 0x20000 {
            return Err(DisplayError::Alias { offset: at });
        }
        for (index, value) in value.to_le_bytes().into_iter().enumerate() {
            self.queue_byte(session, at - 0xd0 + index, value, diagnostics)?;
        }
        Ok(())
    }

    /// Explicit numeric publication with a supplied semantic counterpart catalogue.
    /// Collection bank-byte stores use this same seam; spill owners retain their guards.
    pub fn queue_byte(
        &self,
        session: &mut Session,
        offset: usize,
        value: u8,
        diagnostics: &mut dyn Diagnostics,
    ) -> Result<(), DisplayError> {
        let address = 0xd0usize.checked_add(offset).ok_or(DisplayError::Queue)?;
        if offset < crate::display::queued_tilemaps::QUEUE_BYTES {
            session.queued_tilemaps.queue[offset] = value;
            Ok(())
        } else {
            write_byte(session, diagnostics, address, value)
        }
    }

    pub fn palette_word(
        &self,
        session: &mut Session,
        index: isize,
        value: u16,
        diagnostics: &mut dyn Diagnostics,
    ) -> Result<(), DisplayError> {
        let offset = index.checked_mul(2).ok_or(DisplayError::Source)?;
        let address = 0xc000usize
            .checked_add_signed(offset)
            .ok_or(DisplayError::Source)?;
        if address + 2 > 0x20000 {
            return Err(DisplayError::Alias { offset: address });
        }
        for (lane, value) in value.to_le_bytes().into_iter().enumerate() {
            write_byte(session, diagnostics, address + lane, value)?;
        }
        Ok(())
    }

    /// Called only during validated start/reopen admission, before any interval.
    /// Supplies semantic counterparts without querying or draining packed records.
    pub fn admit(&self, session: &mut Session) -> Result<(), DisplayError> {
        session
            .presentation
            .owner_mut()?
            .state
            .hdma_live
            .bind_producers(self.0.producer_bindings.clone())?;
        self.check(session)?;
        let state = &mut session.presentation.owner_mut()?.state;
        for (token, source) in &self.0.queued_sources {
            if let Some(actual) = state.queued_sources.get(token)
                && actual != source
            {
                return Err(DisplayError::Identity);
            }
        }
        state.queued_sources.extend(self.0.queued_sources.clone());
        Ok(())
    }

    /// Source identities are admitted independently; mutable Workspace/ActorState bytes
    /// are read by the current transfer kernel at each transfer byte.
    pub fn boss_graphics(
        &self,
        session: &mut Session,
        token: [u8; 3],
        destination: u16,
        bytes: u16,
    ) -> Result<(), DisplayError> {
        let source = self
            .0
            .queued_sources
            .get(&token)
            .ok_or(DisplayError::Source)?
            .clone();
        super::commit::boss_graphics(session, source, token, destination, bytes)
    }
    /// Called at the original tail publication, after source, bank and destination stores.
    /// Admission already supplied counterparts; there is no extra header read here.
    pub fn publish_queue_tail(
        &self,
        session: &mut Session,
        value: u16,
    ) -> Result<(), DisplayError> {
        session.queued_tilemaps.tail = value;
        Ok(())
    }
}

fn byte(record: &ActorLiveRecord, offset: usize) -> Result<u8, DisplayError> {
    record
        .base
        .checked_add(offset)
        .and_then(|at| record.image.bytes().get(at).copied())
        .ok_or(DisplayError::Source)
}
fn actor_clip(mode: ActorDrawMode) -> objects::Clip {
    match mode {
        ActorDrawMode::Base => objects::Clip::None,
        ActorDrawMode::Extended => objects::Clip::EnemyInside,
        ActorDrawMode::ExtendedOffscreen => objects::Clip::EnemyOutside,
    }
}
fn word(record: &ActorLiveRecord, offset: usize, order: WordOrder) -> Result<u16, DisplayError> {
    Ok(order.decode([
        byte(record, offset)?,
        byte(record, offset.checked_add(1).ok_or(DisplayError::Source)?)?,
    ]))
}
struct Geometry(Resource);
impl objects::Geometry for Geometry {
    type Error = DisplayError;
    fn count(&mut self) -> Result<u16, DisplayError> {
        word(&self.0.record, 0, self.0.native_order)
    }
    fn x(&mut self, record: usize) -> Result<u16, DisplayError> {
        word(&self.0.record, 2 + 5 * record, self.0.native_order)
    }
    fn y(&mut self, record: usize) -> Result<u8, DisplayError> {
        byte(&self.0.record, 4 + 5 * record)
    }
    fn tile(&mut self, record: usize) -> Result<u16, DisplayError> {
        word(&self.0.record, 5 + 5 * record, self.0.native_order)
    }
}
struct ActorObjects<'a> {
    owner: &'a mut super::commit::OwnedDisplay,
    diagnostics: &'a mut dyn Diagnostics,
}
impl objects::Host for ActorObjects<'_> {
    type Error = DisplayError;
    fn read_native_word(&mut self, at: usize) -> Result<u16, DisplayError> {
        self.owner.state.objects.native_word(at)
    }
    fn read_little_word(&mut self, at: usize) -> Result<u16, DisplayError> {
        objects::Host::read_little_word(&mut self.owner.state.objects, at)
    }
    fn write_byte(&mut self, at: usize, value: u8) -> Result<(), DisplayError> {
        self.owner.state.objects.write_byte(at, value)
    }
    fn write_native_slice(&mut self, at: usize, value: u16) -> Result<(), DisplayError> {
        self.owner.state.objects.write_native_slice(at, value)
    }
    fn write_native_bytes(&mut self, at: usize, value: u16) -> Result<(), DisplayError> {
        objects::Host::write_native_bytes(&mut self.owner.state.objects, at, value)
    }
    fn started(&mut self, clip: objects::Clip) -> Result<(), DisplayError> {
        objects::Host::started(&mut self.owner.state.objects, clip)
    }
}

/// Ordinary OAM bytes stay on the one typed owner. Full/odd native aliases
/// query the current aggregate before the explicit compatibility diagnostic port.
/// No Presentation borrow survives an alias query or effect.
struct RoomObjects<'a> {
    session: &'a mut Session,
    diagnostics: &'a mut dyn Diagnostics,
    armed: bool,
}
impl<'a> RoomObjects<'a> {
    fn new(
        session: &'a mut Session,
        diagnostics: &'a mut dyn Diagnostics,
    ) -> Result<Self, DisplayError> {
        let phase = session.presentation.owner()?.state.phase;
        if phase == DisplayPhase::Failed {
            return Err(DisplayError::Poisoned);
        }
        if phase != DisplayPhase::Producing {
            return Err(DisplayError::Phase);
        }
        Ok(Self {
            session,
            diagnostics,
            armed: true,
        })
    }
    fn byte(&self, offset: usize) -> Result<u8, DisplayError> {
        if offset < 546 {
            self.session
                .presentation
                .owner()?
                .state
                .objects
                .byte(offset)
        } else {
            let address = 0x370usize
                .checked_add(offset)
                .ok_or(DisplayError::Geometry)?;
            read_byte(self.session, self.diagnostics, address)
        }
    }
    fn order(&self) -> Result<WordOrder, DisplayError> {
        Ok(self
            .session
            .presentation
            .owner()?
            .state
            .objects
            .native_order)
    }
}
impl Drop for RoomObjects<'_> {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        if let Ok(owner) = self.session.presentation.owner_mut() {
            owner.state.phase = DisplayPhase::Failed;
        }
    }
}
impl objects::Host for RoomObjects<'_> {
    type Error = DisplayError;
    fn read_native_word(&mut self, offset: usize) -> Result<u16, DisplayError> {
        let first = self.byte(offset)?;
        let second = self.byte(offset.checked_add(1).ok_or(DisplayError::Geometry)?)?;
        Ok(self.order()?.decode([first, second]))
    }
    fn read_little_word(&mut self, offset: usize) -> Result<u16, DisplayError> {
        let first = self.byte(offset)?;
        let second = self.byte(offset.checked_add(1).ok_or(DisplayError::Geometry)?)?;
        Ok(u16::from_le_bytes([first, second]))
    }
    fn write_byte(&mut self, offset: usize, value: u8) -> Result<(), DisplayError> {
        if offset < 546 {
            self.session
                .presentation
                .owner_mut()?
                .state
                .objects
                .write_byte(offset, value)
        } else {
            let address = 0x370usize
                .checked_add(offset)
                .ok_or(DisplayError::Geometry)?;
            write_byte(self.session, self.diagnostics, address, value)
        }
    }
    fn write_native_slice(&mut self, offset: usize, value: u16) -> Result<(), DisplayError> {
        self.byte(offset)?;
        let second = offset.checked_add(1).ok_or(DisplayError::Geometry)?;
        self.byte(second)?;
        let bytes = self.order()?.encode(value);
        self.write_byte(offset, bytes[0])?;
        self.write_byte(second, bytes[1])
    }
    fn write_native_bytes(&mut self, offset: usize, value: u16) -> Result<(), DisplayError> {
        let bytes = self.order()?.encode(value);
        self.write_byte(offset, bytes[0])?;
        self.write_byte(
            offset.checked_add(1).ok_or(DisplayError::Geometry)?,
            bytes[1],
        )
    }
    fn started(&mut self, _clip: objects::Clip) -> Result<(), DisplayError> {
        Ok(())
    }
}

impl Binding {
    /// A recognized producer whose immutable source counterparts were admitted
    /// at opening. Every physical alias still publishes at this byte point.
    pub fn mode7_queue_byte(
        &self,
        session: &mut Session,
        address: usize,
        value: u8,
        diagnostics: &mut dyn Diagnostics,
    ) -> Result<(), DisplayError> {
        if crate::session_encoding::write_projected_byte(session, address, value) {
            Ok(())
        } else {
            diagnostics.write_alias_byte(session, address, value)
        }
    }
}

pub fn read_byte(
    session: &Session,
    diagnostics: &dyn Diagnostics,
    address: usize,
) -> Result<u8, DisplayError> {
    match crate::session_encoding::projected_byte(session, address) {
        Some(value) => Ok(value),
        None => diagnostics.alias_byte(session, address),
    }
}
pub fn write_byte(
    session: &mut Session,
    diagnostics: &mut dyn Diagnostics,
    address: usize,
    value: u8,
) -> Result<(), DisplayError> {
    if crate::session_owners::Presentation::requires_semantic_counterpart(address) {
        return Err(DisplayError::Incomplete);
    }
    if crate::session_encoding::write_projected_byte(session, address, value) {
        Ok(())
    } else {
        diagnostics.write_alias_byte(session, address, value)
    }
}
