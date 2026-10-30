//! Supplied Session/display preparation and the exclusive presentation owner.
#![forbid(unsafe_code)]

use std::sync::Arc;

use crate::game_state::{Game, PreparedSamusBinding, SamusBindingDescription, SamusExecution};
use sm_game::external_inputs::ExternalInputs;
pub use sm_game::persistent_session::*;
use sm_game::session::Session;
pub(crate) use sm_game::session_owners::*;
use {
    crate::game::scene::records::SceneMedia,
    sm_game::scene::{
        component::{DisplayComponent, DisplayStartMetadata},
        samus_body::MarkerPublication,
        samus_content::ContentDescription,
        start::{StartError, ValidatedDisplayStart},
        state::DisplayError,
    },
};

/// This is an unpublished replacement. Installation must move these owners,
/// close native suspension/adoption, and never retain the supplied duplicate
/// numeric presentation fields as independently writable live state.
pub struct PreparedSessionStart {
    pub(crate) gameplay: Session,
    pub(crate) observations: ExternalInputs,
    pub(crate) display: DisplayComponent,
    pub(crate) metadata: DisplayStartMetadata,
    pub(crate) audio_requests: AudioRequests,
    pub(crate) binding: PreparedSamusBinding,
    pub(crate) marker: MarkerPublication,
    pub(crate) actor_queue_projection: [ActorQueueProjection; 8],
    pub(crate) roles: ContentDescription,
    pub(crate) samus_publications: Arc<SamusPublications>,
    pub(crate) required: Arc<SessionStartRequirements>,
}

pub(crate) fn prepare(
    game: &Game,
    supplied: SessionStartDescription,
    media: Arc<SceneMedia>,
    required: &SessionStartRequirements,
) -> Result<PreparedSessionStart, SessionStartError> {
    let gameplay = supplied
        .gameplay
        .ok_or(SessionStartError::MissingGameplay)?;
    if !Arc::ptr_eq(&gameplay.world, game.world())
        || gameplay.world.revision() != game.world().revision()
    {
        return Err(SessionStartError::World);
    }
    let observations = supplied
        .observations
        .ok_or(SessionStartError::MissingObservations)?;
    let roles = supplied.roles.ok_or(SessionStartError::MissingRoles)?;
    let marker = supplied
        .marker
        .ok_or(SessionStartError::MissingBodyMarker)?;
    let projection = supplied
        .actor_queue_projection
        .ok_or(SessionStartError::MissingActorQueueProjection)?;
    let publications = supplied
        .samus_publications
        .ok_or(SessionStartError::MissingSamusPublications)?;
    let display = supplied
        .display
        .display
        .as_ref()
        .ok_or(SessionStartError::Display(StartError::MissingDisplay))?;
    if *gameplay.presentation.visual() != display.visual
        || *gameplay.presentation.palette() != display.player_palette
        || *gameplay.presentation.atmospheric() != display.atmospheric
    {
        return Err(SessionStartError::GameplayDisplay);
    }
    if let Presentation::Owned {
        display: existing, ..
    } = &gameplay.presentation
    {
        if existing.owner().state != *display {
            return Err(SessionStartError::GameplayDisplay);
        }
    }
    if observations.transitions.frame_gamma != gameplay.player.transitions.frame_gamma
        || observations.frame != display.nmi.frame_word
    {
        return Err(SessionStartError::Observations);
    }
    if projection != required.actor_queue_projection {
        return Err(SessionStartError::ActorQueues);
    }
    if publications != required.samus_publications
        || !publications
            .object_atlases
            .contains_key(&display.controls.object_layout)
    {
        return Err(SessionStartError::MissingSamusPublications);
    }
    let mut producers = vec![marker.producer, publications.body];
    producers.extend(publications.echoes.iter().flatten().copied());
    for (index, producer) in producers.iter().enumerate() {
        if producers[..index].contains(producer)
            || display.producer_ids.validate_id(*producer).is_err()
        {
            return Err(SessionStartError::MissingSamusPublications);
        }
    }
    for atlases in publications.object_atlases.values() {
        for atlas in atlases {
            media
                .atlas(*atlas)
                .map_err(|_| SessionStartError::MissingSamusPublications)?;
        }
    }
    let queue_word = |at| {
        let low = crate::game::session_encoding::projected_byte(&gameplay, at)
            .ok_or(SessionStartError::ActorQueues)?;
        let high = crate::game::session_encoding::projected_byte(&gameplay, at + 1)
            .ok_or(SessionStartError::ActorQueues)?;
        Ok::<_, SessionStartError>(u16::from_le_bytes([low, high]))
    };
    let mut covered = [false; 114];
    for (layer, layout) in projection.iter().enumerate() {
        let end = layout
            .first
            .checked_add(layout.words)
            .ok_or(SessionStartError::ActorQueues)?;
        if end > covered.len()
            || display.actor_queues[layer].actors.len() != layout.words
            || display.actor_queues[layer].size != queue_word(0xf68 + layer * 2)?
        {
            return Err(SessionStartError::ActorQueues);
        }
        for index in layout.first..end {
            if covered[index]
                || queue_word(0xe84 + index * 2)?
                    != display.actor_queues[layer].actors[index - layout.first]
            {
                return Err(SessionStartError::ActorQueues);
            }
            covered[index] = true;
        }
    }
    if covered.iter().any(|value| !value) {
        return Err(SessionStartError::ActorQueues);
    }
    marker
        .validate(&media)
        .map_err(SessionStartError::Binding)?;
    if marker != required.marker || display.producer_ids.validate_id(marker.producer).is_err() {
        return Err(SessionStartError::MissingBodyMarker);
    }
    let catalog = supplied
        .display
        .samus_content
        .as_ref()
        .ok_or(SessionStartError::Display(StartError::MissingSamusContent))?
        .clone();
    let binding = game
        .prepare_samus_binding(
            SamusBindingDescription {
                world: gameplay.world.clone(),
                roles: roles.clone(),
                spritemaps: catalog.clone(),
                request_media: media.clone(),
                execution: SamusExecution::Prepared,
            },
            &SamusBindingDescription {
                world: game.world().clone(),
                roles: required.roles.clone(),
                spritemaps: required.display.samus_content.clone(),
                request_media: media.clone(),
                execution: SamusExecution::Prepared,
            },
        )
        .map_err(SessionStartError::Binding)?;
    let opened = DisplayComponent::open(
        ValidatedDisplayStart::construct(supplied.display, media, &required.display)
            .map_err(SessionStartError::Display)?,
    )
    .map_err(SessionStartError::Display)?;
    if *gameplay.control.primary() != opened.metadata.continuation.primary
        || *gameplay.control.secondary() != opened.metadata.continuation.secondary
    {
        return Err(SessionStartError::GameplayDisplay);
    }
    if gameplay
        .control
        .metadata()
        .is_ok_and(|existing| *existing != opened.metadata)
    {
        return Err(SessionStartError::GameplayDisplay);
    }
    let audio_requests = supplied
        .audio_requests
        .ok_or(SessionStartError::MissingAudioRequests)?;
    audio_requests
        .validate(&opened.metadata, &required.audio_catalog)
        .map_err(SessionStartError::Binding)?;
    if gameplay
        .control
        .requests()
        .is_ok_and(|existing| *existing != audio_requests)
    {
        return Err(SessionStartError::GameplayDisplay);
    }
    Ok(PreparedSessionStart {
        gameplay,
        observations,
        display: opened.display,
        metadata: opened.metadata,
        audio_requests,
        binding,
        marker,
        actor_queue_projection: projection,
        roles,
        samus_publications: publications,
        required: Arc::new(required.clone()),
    })
}

impl PreparedSessionStart {
    pub(crate) fn into_session(self) -> (PersistentSession, PreparedSamusBinding) {
        let Self {
            mut gameplay,
            observations,
            display,
            metadata,
            audio_requests,
            binding,
            marker,
            actor_queue_projection,
            roles,
            samus_publications,
            required,
        } = self;
        // Replace, rather than retain, the supplied numeric construction copy.
        gameplay.presentation = Presentation::Owned {
            display: Box::new(display),
            marker,
            actor_queue_projection,
            samus_publications,
        };
        gameplay.control = Control::Owned(Box::new(CurrentControl {
            metadata,
            requests: audio_requests,
        }));
        (
            PersistentSession {
                gameplay,
                observations,
                roles,
                required,
            },
            binding,
        )
    }
}

pub(crate) struct PreparedSessionAdoption(pub(crate) Option<PreparedSessionStart>);
pub(crate) fn prepare_snapshot(
    game: &Game,
    payload: &SessionSnapshotPayload,
) -> Result<PreparedSessionAdoption, SessionStartError> {
    let Some(checkpoint) = &payload.0 else {
        return Ok(PreparedSessionAdoption(None));
    };
    let required = checkpoint.0.required.clone();
    let (start, media) = checkpoint
        .clone()
        .into_start()
        .map_err(SessionStartError::Binding)?;
    prepare(game, start, media, &required).map(|start| PreparedSessionAdoption(Some(start)))
}
pub(crate) fn validate_normalized_projection(
    game: &Game,
    prepared: &PreparedSessionStart,
) -> Result<(), DisplayError> {
    let mut matches = true;
    crate::game::session_encoding::encode(&prepared.gameplay, |at, bytes| {
        for (part, byte) in bytes.iter().enumerate() {
            matches &= game.ram().read_byte(at + part) == *byte;
        }
    });
    let continuation = &prepared.metadata.continuation;
    for (at, byte) in [
        (0x611, continuation.outer),
        (0x612, continuation.primary),
        (0x613, continuation.secondary),
        (0x614, continuation.music),
        (0x615, continuation.message),
        (0x616, continuation.auxiliary),
    ] {
        matches &= game.ram().read_byte(at) == byte;
    }
    if matches {
        Ok(())
    } else {
        Err(DisplayError::Identity)
    }
}

use sm_game::scene::samus_content::{PreparedBytes, Registration, Role, RoleLease};
use sm_game::scene::state::{DisplayState, WordOrder};
use sm_game::scene::{samus, samus_body, samus_position_tiles as positions};

struct PreparedTable(RoleLease<PreparedBytes>);
impl positions::Table for PreparedTable {
    type Error = DisplayError;
    fn native_word(&self, offset: usize) -> Result<u16, DisplayError> {
        self.0.native_word(offset)
    }
    fn byte(&self, offset: usize) -> Result<u8, DisplayError> {
        self.0.byte(offset)
    }
}
impl samus_body::Table for PreparedTable {
    type Error = DisplayError;
    fn little_word(&self, offset: usize) -> Result<u16, DisplayError> {
        self.0.little_word(offset)
    }
    fn native_word(&self, offset: usize) -> Result<u16, DisplayError> {
        self.0.native_word(offset)
    }
}

struct BodyInputs<'a> {
    game: &'a mut Game,
    player: &'a mut sm_game::session::Player,
    observations: &'a ExternalInputs,
    publications: Arc<SamusPublications>,
    palette: crate::game::scene::records::MediaRef,
    raster: sm_game::scene::state::RasterState,
}
impl BodyInputs<'_> {
    fn source(
        &self,
        role: Role,
        registration: Registration,
        displacement: usize,
    ) -> Result<PreparedTable, DisplayError> {
        self.game
            .prepared_samus_role(role, registration, displacement)?
            .map(PreparedTable)
            .ok_or(DisplayError::Incomplete)
    }
}
impl samus::Inputs for BodyInputs<'_> {
    fn content(&mut self, _: &DisplayState) -> Result<Arc<samus::SamusCatalog>, DisplayError> {
        self.game
            .samus_spritemaps()?
            .ok_or(DisplayError::Incomplete)
    }
    fn invalid_map(&mut self, _: &mut DisplayState, cursor: u16) -> Result<(), DisplayError> {
        crate::game::session_runtime::native_rom_diagnostic(
            0x920000 | u32::from(cursor),
            self.game,
        );
        Ok(())
    }
    fn object_context(
        &mut self,
        controls: &sm_game::scene::state::DisplayControls,
        _: &sm_game::scene::state::EffectiveDisplayDescription,
    ) -> Result<crate::game::scene::records::ObjectContext, DisplayError> {
        use crate::game::scene::records::{ObjectContext, ObjectSelection};
        let atlases = *self
            .publications
            .object_atlases
            .get(&controls.object_layout)
            .ok_or(DisplayError::Incomplete)?;
        Ok(ObjectContext {
            selection: ObjectSelection {
                atlases,
                size_set: controls.object_layout >> 5,
                first_order: self.publications.object_first_order,
                interlaced: self.raster.interlaced,
                even_frame: self.raster.even_frame,
            },
            palette: self.palette,
        })
    }
}
impl positions::Inputs for BodyInputs<'_> {
    type Table = PreparedTable;
    fn ceres(&mut self) -> Result<u16, DisplayError> {
        Ok(self.observations.ceres)
    }
    fn player_x(&mut self) -> Result<u16, DisplayError> {
        Ok(self.player.x)
    }
    fn player_y(&mut self) -> Result<u16, DisplayError> {
        Ok(self.player.y)
    }
    fn set_player_x(&mut self, _: &mut DisplayState, value: u16) -> Result<(), DisplayError> {
        self.player.x = value;
        Ok(())
    }
    fn set_player_y(&mut self, _: &mut DisplayState, value: u16) -> Result<(), DisplayError> {
        self.player.y = value;
        Ok(())
    }
    fn project(
        &mut self,
        state: &mut DisplayState,
        x: u16,
        y: u16,
    ) -> Result<(u16, u16), DisplayError> {
        let point = sm_game::cinematic::project_cinematic_point_reads(
            sm_game::geometry::Point { x, y },
            |at| state.controls.affine_words[(at - 0x78) / 2],
        );
        Ok((point.x, point.y))
    }
    fn animation_frame(&mut self) -> Result<u16, DisplayError> {
        Ok(self.player.animation_frame())
    }
    fn movement(&mut self) -> Result<u8, DisplayError> {
        Ok(self.player.movement_and_facing.to_le_bytes()[1])
    }
    fn landing_order(&self) -> WordOrder {
        self.source(Role::Landing, Registration::Native, 0)
            .expect("Landing content")
            .0
            .native_order()
    }
    fn landing_offset(&mut self, offset: usize) -> Result<u16, DisplayError> {
        self.source(Role::Landing, Registration::Native, 0)?
            .0
            .native_word(offset)
    }
    fn crouching_offset(&mut self, offset: usize) -> Result<i8, DisplayError> {
        Ok(self
            .source(Role::Crouching, Registration::Native, 0)?
            .0
            .byte(offset)? as i8)
    }
    fn drained_offset(&mut self, offset: usize) -> Result<i8, DisplayError> {
        Ok(self
            .source(Role::Drained, Registration::Native, 0)?
            .0
            .byte(offset)? as i8)
    }
    fn position_source(&mut self) -> Result<PreparedTable, DisplayError> {
        self.source(Role::PositionFallback, Registration::Native, 0)
    }
    fn camera_x(&mut self) -> Result<u16, DisplayError> {
        Ok(self.observations.camera_x)
    }
    fn camera_y(&mut self) -> Result<u16, DisplayError> {
        Ok(self.observations.camera_y)
    }
    fn set_body_x(&mut self, _: &mut DisplayState, value: u16) -> Result<(), DisplayError> {
        self.player.body_position.x = value;
        Ok(())
    }
    fn set_body_y(&mut self, _: &mut DisplayState, value: u16) -> Result<(), DisplayError> {
        self.player.body_position.y = value;
        Ok(())
    }
    fn body_x(&mut self) -> Result<u16, DisplayError> {
        Ok(self.player.body_position.x)
    }
    fn body_y(&mut self) -> Result<u16, DisplayError> {
        Ok(self.player.body_position.y)
    }
    fn pose(&mut self) -> Result<u16, DisplayError> {
        Ok(self.player.pose)
    }
    fn tile_table(&mut self, which: positions::TileTable) -> Result<PreparedTable, DisplayError> {
        let role = match which {
            positions::TileTable::Pose => Role::TilePose,
            positions::TileTable::Top => Role::TileTop,
            positions::TileTable::Bottom => Role::TileBottom,
        };
        self.source(role, Registration::Loaded, 0)
    }
    fn tile_definition(
        &mut self,
        _: &mut DisplayState,
        cursor: u16,
    ) -> Result<PreparedTable, DisplayError> {
        if cursor & 0x8000 == 0 {
            crate::game::session_runtime::native_rom_diagnostic(
                0x920000 | u32::from(cursor),
                self.game,
            );
        }
        let mapped = crate::rtl::lorom_offset(0x920000 | u32::from(cursor)) as usize;
        let base = self
            .game
            .samus_role_base(Role::TileDefinition, Registration::Native)?
            .ok_or(DisplayError::Incomplete)?;
        self.source(
            Role::TileDefinition,
            Registration::Native,
            mapped.checked_sub(base).ok_or(DisplayError::Source)?,
        )
    }
    fn request_word(
        &mut self,
        state: &mut DisplayState,
        half: samus_body::Half,
        pointer: u16,
    ) -> Result<(), DisplayError> {
        let slot = match half {
            samus_body::Half::Top => 0,
            samus_body::Half::Bottom => 1,
        };
        let [low, high] = pointer.to_le_bytes();
        let mut bytes = state.samus[slot].pointer.to_le_bytes();
        bytes[0] = low;
        state.samus[slot].pointer = u16::from_le_bytes(bytes);
        bytes[1] = high;
        state.samus[slot].pointer = u16::from_le_bytes(bytes);
        Ok(())
    }
    fn request_flag(
        &mut self,
        state: &mut DisplayState,
        half: samus_body::Half,
    ) -> Result<(), DisplayError> {
        state.samus[match half {
            samus_body::Half::Top => 0,
            samus_body::Half::Bottom => 1,
        }]
        .enabled = 1;
        Ok(())
    }
    fn tile_request(
        &mut self,
        _: &DisplayState,
        half: samus_body::Half,
        pointer: u16,
    ) -> Result<positions::TileRequest, DisplayError> {
        self.game
            .samus_request(half, pointer)?
            .ok_or(DisplayError::Incomplete)
    }
}
impl samus_body::Inputs for BodyInputs<'_> {
    type BodyTable = PreparedTable;
    fn knockback(&mut self) -> Result<u16, DisplayError> {
        Ok(self.player.knockback_timer)
    }
    fn invincibility(&mut self) -> Result<u16, DisplayError> {
        Ok(self.player.invincibility)
    }
    fn shine(&mut self) -> Result<u16, DisplayError> {
        Ok(self.player.shine_timer)
    }
    fn frame_counter(&mut self) -> Result<u16, DisplayError> {
        Ok(self.observations.frame)
    }
    fn table(&mut self, half: samus_body::Half) -> Result<PreparedTable, DisplayError> {
        self.source(
            match half {
                samus_body::Half::Top => Role::BodyTop,
                samus_body::Half::Bottom => Role::BodyBottom,
            },
            Registration::Loaded,
            0,
        )
    }
    fn set_sprite(&mut self, half: samus_body::Half, index: u16) -> Result<(), DisplayError> {
        match half {
            samus_body::Half::Top => self.player.top_sprite = index,
            samus_body::Half::Bottom => self.player.bottom_sprite = index,
        };
        Ok(())
    }
}

pub(crate) fn draw_body(
    owner: &mut sm_game::scene::commit::OwnedDisplay,
    game: &mut Game,
    player: &mut sm_game::session::Player,
    observations: &ExternalInputs,
    publications: Arc<SamusPublications>,
    marker: &MarkerPublication,
    mode: samus_body::Body,
) -> Result<(), DisplayError> {
    let media = owner.media().clone();
    let producer = publications.body;
    let mut inputs = BodyInputs {
        game,
        player,
        observations,
        publications,
        palette: owner.image().palette,
        raster: owner.state.raster,
    };
    samus_body::draw_owned(
        &mut owner.state,
        &media,
        producer,
        marker,
        mode,
        &mut inputs,
    )
}
pub(crate) fn bottom_visible(
    owner: &mut sm_game::scene::commit::OwnedDisplay,
    game: &mut Game,
    player: &mut sm_game::session::Player,
    observations: &ExternalInputs,
    publications: Arc<SamusPublications>,
    marker: &MarkerPublication,
) -> Result<bool, DisplayError> {
    let media = owner.media().clone();
    let producer = publications.body;
    let mut inputs = BodyInputs {
        game,
        player,
        observations,
        publications,
        palette: owner.image().palette,
        raster: owner.state.raster,
    };
    samus_body::bottom_owned(&mut owner.state, &media, producer, marker, &mut inputs)
}
pub(crate) fn draw_sprite(
    owner: &mut sm_game::scene::commit::OwnedDisplay,
    game: &mut Game,
    player: &mut sm_game::session::Player,
    observations: &ExternalInputs,
    publications: Arc<SamusPublications>,
    producer: crate::game::scene::records::ProducerObjectId,
    index: u16,
    position: (u16, u16),
) -> Result<(), DisplayError> {
    let media = owner.media().clone();
    let mut inputs = BodyInputs {
        game,
        player,
        observations,
        publications,
        palette: owner.image().palette,
        raster: owner.state.raster,
    };
    samus::draw_owned(
        &mut owner.state,
        &media,
        producer,
        index,
        position,
        &mut inputs,
    )
}
