//! Supplied Session/display preparation and the exclusive presentation owner.
#![forbid(unsafe_code)]

use std::sync::Arc;

use crate::game::scene::{
    component::{DisplayComponent, DisplayStartMetadata},
    records::SceneMedia,
    samus_body::MarkerPublication,
    samus_content::ContentDescription,
    start::{DisplayStartDescription, StartError, ValidatedDisplayStart},
    state::DisplayError,
};
pub(crate) use crate::game::session_checkpoint::AuxiliaryGameplayCheckpoint;
use crate::game_state::{Game, PreparedSamusBinding, SamusBindingDescription, SamusExecution};
use sm_game::external_inputs::ExternalInputs;
use sm_game::session::Session;
pub(crate) use sm_game::session_owners::*;

pub(crate) struct SessionStartDescription {
    pub gameplay: Option<Session>,
    pub observations: Option<ExternalInputs>,
    pub display: DisplayStartDescription,
    pub roles: Option<ContentDescription>,
    pub marker: Option<MarkerPublication>,
    pub actor_queue_projection: Option<[ActorQueueProjection; 8]>,
    pub samus_publications: Option<Arc<SamusPublications>>,
    pub audio_requests: Option<AudioRequests>,
}

#[derive(Debug)]
pub enum SessionStartError {
    MissingGameplay,
    MissingObservations,
    MissingRoles,
    MissingBodyMarker,
    MissingActorQueueProjection,
    MissingSamusPublications,
    MissingAudioRequests,
    World,
    GameplayDisplay,
    Observations,
    ActorQueues,
    Display(StartError),
    Binding(DisplayError),
    Phase,
}

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

/// Tools supplies the complete semantic construction and independent contract.
/// The app transports this value without access to mutable gameplay internals.
pub struct SuppliedSessionStart {
    description: SessionStartDescription,
    media: Arc<SceneMedia>,
    required: SessionStartRequirements,
}
pub struct SessionStartContract(SessionStartRequirements);

impl SuppliedSessionStart {
    pub(crate) fn new(
        description: SessionStartDescription,
        media: Arc<SceneMedia>,
        required: SessionStartRequirements,
    ) -> Self {
        Self {
            description,
            media,
            required,
        }
    }
    pub(crate) fn into_parts(
        self,
    ) -> (
        SessionStartDescription,
        Arc<SceneMedia>,
        SessionStartRequirements,
    ) {
        (self.description, self.media, self.required)
    }
}
impl SessionStartContract {
    pub(crate) fn new(required: SessionStartRequirements) -> Self {
        Self(required)
    }
    pub(crate) fn requirements(&self) -> &SessionStartRequirements {
        &self.0
    }
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

/// The one live gameplay owner. Runtime moves an exclusive lease of this value;
/// it never decodes a second Session while this owner exists.
#[derive(Clone)]
pub(crate) struct PersistentSession {
    pub(crate) gameplay: Session,
    pub(crate) observations: ExternalInputs,
    pub(crate) roles: ContentDescription,
    pub(crate) required: Arc<SessionStartRequirements>,
}
pub(crate) struct IntervalMetadata {
    pub input: sm_game::scene::start::InputHistory,
    pub continuation: sm_game::scene::start::ContinuationHistory,
    pub audio: sm_game::scene::start::InitialAudioIntent,
}

pub(crate) enum SessionSlot {
    Legacy,
    Ready(Box<PersistentSession>),
    Leased,
    Failed(Option<Box<PersistentSession>>),
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

/// Captured current owners, including final resident and separate visible history.
/// This semantic checkpoint does not run an interval or invoke the native loader.
#[derive(Clone)]
pub struct SessionCheckpoint(PersistentSession);

impl PersistentSession {
    pub(crate) fn metadata(&self) -> &DisplayStartMetadata {
        self.gameplay
            .control
            .metadata()
            .expect("persistent control owner")
    }
    pub(crate) fn metadata_mut(&mut self) -> &mut DisplayStartMetadata {
        self.gameplay
            .control
            .metadata_mut()
            .expect("persistent control owner")
    }
    pub(crate) fn begin_interval(&mut self, fresh_entry: bool) -> Result<(), DisplayError> {
        self.gameplay
            .presentation
            .owner_mut()?
            .begin_interval(fresh_entry)
    }
    pub(crate) fn finish_interval(
        &mut self,
        metadata: IntervalMetadata,
        description: crate::game::scene::records::SceneDescription,
        media: Arc<SceneMedia>,
        effects: &mut impl sm_game::scene::commit::SettlementEffects,
    ) -> Result<(), DisplayError> {
        let current = self.metadata().clone();
        let next_tick = current
            .identity
            .committed_tick
            .checked_add(1)
            .ok_or(DisplayError::Identity)?;
        if description.tick != next_tick
            || metadata.continuation.next_input_record
                != current
                    .identity
                    .next_input_record
                    .checked_add(1)
                    .ok_or(DisplayError::Identity)?
            || metadata.continuation.primary != *self.gameplay.control.primary()
            || metadata.continuation.secondary != *self.gameplay.control.secondary()
            || metadata.continuation.outer != current.continuation.outer
            || metadata.continuation.music != current.continuation.music
            || metadata.continuation.message != current.continuation.message
            || metadata.continuation.auxiliary != current.continuation.auxiliary
            || metadata.continuation.capture != current.identity.capture
            || metadata.continuation.observation_policy != current.reference.observation_policy
            || metadata.continuation.phase != sm_game::scene::start::CommittedPhase::SettledInterval
            || metadata.input.capture != current.identity.capture
            || metadata.input.policy != current.reference.input_policy
            || metadata.input != current.input_history
            || metadata.audio.capture != current.identity.capture
            || metadata.audio.revision != self.required.display.audio_revision
            || metadata.audio.declarations != self.required.display.audio_programs
            || metadata.audio.timing_policy != current.reference.audio_timing_policy
            || metadata.audio != current.audio
        {
            return Err(DisplayError::Identity);
        }
        let requests = self.gameplay.control.requests()?;
        if requests.pending_mutation {
            return Err(DisplayError::Incomplete);
        }
        requests.validate_at_tick(&current, &self.required.audio_catalog, next_tick)?;
        let owner = self.gameplay.presentation.owner_mut()?;
        owner.refresh()?;
        owner.begin_settlement()?;
        owner.settle(effects)?;
        owner.commit_scene(description, media)?;
        self.observations.frame = owner.state.nmi.frame_word;
        let current = self.metadata_mut();
        current.identity.committed_tick = next_tick;
        current.identity.next_input_record = metadata.continuation.next_input_record;
        current.input_history = metadata.input;
        current.continuation = metadata.continuation;
        current.audio = metadata.audio;
        Ok(())
    }
    pub(crate) fn checkpoint(&self) -> Result<SessionCheckpoint, DisplayError> {
        self.gameplay.presentation.owner()?.checkpoint_parts()?;
        self.gameplay
            .control
            .requests()?
            .validate(self.metadata(), &self.required.audio_catalog)?;
        if self.metadata().identity.committed_tick
            != self.gameplay.presentation.owner()?.committed().tick()
            || self.metadata().identity.next_input_record
                != self.metadata().continuation.next_input_record
        {
            return Err(DisplayError::Identity);
        }
        Ok(SessionCheckpoint(self.clone()))
    }
    pub(crate) fn poison(&mut self) {
        if let Ok(owner) = self.gameplay.presentation.owner_mut() {
            owner.state.phase = sm_game::scene::state::DisplayPhase::Failed;
        }
        self.gameplay.actor_identity.retain_poison();
    }
}

impl SessionCheckpoint {
    /// Full owned state with immutable declarations sealed against the supplied
    /// contract. Allocation roles come from the actual caller, never defaults.
    pub(crate) fn encode_binary(
        &self,
        loaded: Option<&crate::rom::RomImage>,
        native: Option<&crate::rom::RomImage>,
        sources: &[(sm_game::room_entry::world::ResourceId, crate::rom::RomImage)],
    ) -> Result<Vec<u8>, String> {
        self.0
            .checkpoint()
            .map_err(|error| format!("Session checkpoint {error:?}"))?;
        let state = &self.0;
        let Presentation::Owned {
            display,
            marker,
            actor_queue_projection,
            samus_publications,
        } = &state.gameplay.presentation
        else {
            return Err("no owned Session display checkpoint".into());
        };
        let declarations = sm_game::scene::checkpoint_codec::session_declarations(
            &state.roles,
            marker,
            actor_queue_projection,
            samus_publications,
        );
        if declarations != session_required_declarations(&state.required) {
            return Err("Session checkpoint immutable bindings differ".into());
        }
        let components = [
            declarations,
            state.gameplay.control.encode_checkpoint(&state.required)?,
            state
                .gameplay
                .encode_gameplay_checkpoint(state.metadata())?,
            state.gameplay.encode_actor_identity_checkpoint(
                state.metadata(),
                loaded,
                native,
                sources,
            )?,
            crate::game::session_checkpoint::encode_observations_checkpoint(
                &state.observations,
                state.metadata(),
            )?,
            display
                .checkpoint()
                .map_err(|error| format!("Session display {error:?}"))?
                .encode_checkpoint(state.metadata(), &state.required.display)?,
        ];
        let mut bytes = Vec::from(b"SMSESSION\x01".as_slice());
        for component in components {
            bytes.extend_from_slice(&(component.len() as u64).to_le_bytes());
            bytes.extend_from_slice(&component);
        }
        Ok(bytes)
    }

    /// Reconstruct a detached supplied start. Epoch preflight reads the current
    /// identity only; numeric, input, audio and display state come from bytes.
    /// Validate/install without an intervening actor adoption.
    pub(crate) fn decode_binary(
        bytes: &[u8],
        current: &crate::game::actors::identity::ActorIdentityState,
        world: Arc<sm_game::room_entry::world::WorldBinding>,
        media: Arc<SceneMedia>,
        required: SessionStartRequirements,
        loaded: Option<&crate::rom::RomImage>,
        native: Option<&crate::rom::RomImage>,
        sources: &[(sm_game::room_entry::world::ResourceId, crate::rom::RomImage)],
    ) -> Result<SuppliedSessionStart, String> {
        let mut read = sm_game::actors::snapshot::CheckpointReader(bytes);
        if read.bytes(10)? != b"SMSESSION\x01" {
            return Err("unsupported Session checkpoint schema".into());
        }
        if session_binary_component(&mut read)? != session_required_declarations(&required) {
            return Err("Session checkpoint immutable bindings changed".into());
        }
        let control = Control::decode_checkpoint(session_binary_component(&mut read)?, &required)?;
        let metadata = control
            .metadata()
            .map_err(|error| format!("Session control {error:?}"))?
            .clone();
        let numeric = Session::decode_gameplay_checkpoint(
            session_binary_component(&mut read)?,
            &metadata,
            world.clone(),
        )?;
        let identity = numeric.decode_actor_identity(
            session_binary_component(&mut read)?,
            current,
            &metadata,
            loaded,
            native,
            sources,
        )?;
        let observations = crate::game::session_checkpoint::decode_observations_checkpoint(
            session_binary_component(&mut read)?,
            &metadata,
        )?;
        let display = sm_game::scene::component::DisplayCheckpoint::decode_checkpoint(
            session_binary_component(&mut read)?,
            media,
            &metadata,
            &required.display,
        )?;
        if !read.0.is_empty() {
            return Err("trailing Session checkpoint components".into());
        }
        let (opening_start, _) = display.clone().into_start_description(metadata.clone());
        let (display_start, media) = display.into_start_description(metadata);
        let opened = DisplayComponent::open(
            ValidatedDisplayStart::construct(opening_start, media.clone(), &required.display)
                .map_err(|error| format!("Session display start {error:?}"))?,
        )
        .map_err(|error| format!("Session display opening {error:?}"))?;
        let presentation = Presentation::Owned {
            display: Box::new(opened.display),
            marker: required.marker.clone(),
            actor_queue_projection: required.actor_queue_projection,
            samus_publications: required.samus_publications.clone(),
        };
        let audio_requests = control
            .requests()
            .map_err(|error| format!("Session requests {error:?}"))?
            .clone();
        let gameplay = numeric.into_session(identity, control, presentation);
        Ok(SuppliedSessionStart::new(
            SessionStartDescription {
                gameplay: Some(gameplay),
                observations: Some(observations),
                display: display_start,
                roles: Some(required.roles.clone()),
                marker: Some(required.marker.clone()),
                actor_queue_projection: Some(required.actor_queue_projection),
                samus_publications: Some(required.samus_publications.clone()),
                audio_requests: Some(audio_requests),
            },
            media,
            required,
        ))
    }

    pub(crate) fn into_start(
        self,
    ) -> Result<(SessionStartDescription, Arc<SceneMedia>), DisplayError> {
        let PersistentSession {
            gameplay,
            observations,
            roles,
            required: _,
        } = self.0;
        let metadata = gameplay.control.metadata()?.clone();
        let audio_requests = gameplay.control.requests()?.clone();
        let (display, marker, projection, publications) = match &gameplay.presentation {
            Presentation::Owned {
                display,
                marker,
                actor_queue_projection,
                samus_publications,
            } => (
                display.checkpoint()?,
                marker.clone(),
                *actor_queue_projection,
                samus_publications.clone(),
            ),
            Presentation::Legacy(_) => return Err(DisplayError::Phase),
        };
        let (display, media) = display.into_start_description(metadata);
        Ok((
            SessionStartDescription {
                gameplay: Some(gameplay),
                observations: Some(observations),
                display,
                roles: Some(roles),
                marker: Some(marker),
                actor_queue_projection: Some(projection),
                samus_publications: Some(publications),
                audio_requests: Some(audio_requests),
            },
            media,
        ))
    }
}

fn session_required_declarations(required: &SessionStartRequirements) -> Vec<u8> {
    sm_game::scene::checkpoint_codec::session_declarations(
        &required.roles,
        &required.marker,
        &required.actor_queue_projection,
        &required.samus_publications,
    )
}
fn session_binary_component<'a>(
    read: &mut sm_game::actors::snapshot::CheckpointReader<'a>,
) -> Result<&'a [u8], String> {
    let length = usize::try_from(read.counter()?)
        .map_err(|_| "Session checkpoint component length overflow")?;
    read.bytes(length)
}

/// One complete semantic branch observation, never an alternative live owner.
#[derive(Clone)]
pub(crate) struct SessionSnapshotPayload(pub(crate) Option<SessionCheckpoint>);
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
impl SessionSnapshotPayload {
    pub(crate) fn encode_gameplay_checkpoint(&self) -> Result<Vec<u8>, String> {
        let state = self.0.as_ref().ok_or("no supplied Session gameplay")?;
        state
            .0
            .gameplay
            .encode_gameplay_checkpoint(state.0.metadata())
    }
    pub(crate) fn decode_gameplay_checkpoint(
        bytes: &[u8],
        metadata: &DisplayStartMetadata,
        world: Arc<sm_game::room_entry::world::WorldBinding>,
    ) -> Result<crate::game::session_checkpoint::NumericGameplayCheckpoint, String> {
        Session::decode_gameplay_checkpoint(bytes, metadata, world)
    }
    pub(crate) fn encode_room_checkpoint(&self) -> Result<Vec<u8>, String> {
        let state = self.0.as_ref().ok_or("no supplied Session room")?;
        state
            .0
            .gameplay
            .room
            .encode_checkpoint(state.0.metadata(), &state.0.gameplay.world)
    }
    pub(crate) fn decode_room_checkpoint(
        bytes: &[u8],
        metadata: &DisplayStartMetadata,
        expected: &sm_game::room_entry::world::WorldBinding,
    ) -> Result<sm_game::session::Room, String> {
        sm_game::session::Room::decode_checkpoint(bytes, metadata, expected)
    }
    pub(crate) fn encode_auxiliary_gameplay_checkpoint(&self) -> Result<Vec<u8>, String> {
        let state = self
            .0
            .as_ref()
            .ok_or("no supplied Session auxiliary gameplay")?;
        state
            .0
            .gameplay
            .encode_auxiliary_checkpoint(state.0.metadata())
    }
    pub(crate) fn decode_auxiliary_gameplay_checkpoint(
        bytes: &[u8],
        metadata: &DisplayStartMetadata,
    ) -> Result<AuxiliaryGameplayCheckpoint, String> {
        sm_game::session::Session::decode_auxiliary_checkpoint(bytes, metadata)
    }
    pub(crate) fn encode_player_checkpoint(&self) -> Result<Vec<u8>, String> {
        let state = self.0.as_ref().ok_or("no supplied Session player")?;
        state
            .0
            .gameplay
            .player
            .encode_checkpoint(state.0.metadata())
    }
    pub(crate) fn decode_player_checkpoint(
        bytes: &[u8],
        metadata: &DisplayStartMetadata,
    ) -> Result<sm_game::session::Player, String> {
        sm_game::session::Player::decode_checkpoint(bytes, metadata)
    }
    pub(crate) fn encode_enemy_projectile_checkpoint(&self) -> Result<Vec<u8>, String> {
        let state = self
            .0
            .as_ref()
            .ok_or("no supplied Session enemy projectiles")?;
        state
            .0
            .gameplay
            .enemy_projectiles
            .encode_checkpoint(state.0.metadata())
    }
    pub(crate) fn decode_enemy_projectile_checkpoint(
        bytes: &[u8],
        metadata: &DisplayStartMetadata,
    ) -> Result<sm_game::session::EnemyProjectiles, String> {
        sm_game::session::EnemyProjectiles::decode_checkpoint(bytes, metadata)
    }
    pub(crate) fn encode_projectile_checkpoint(&self) -> Result<Vec<u8>, String> {
        let state = self.0.as_ref().ok_or("no supplied Session projectiles")?;
        state
            .0
            .gameplay
            .projectiles
            .encode_checkpoint(state.0.metadata())
    }
    pub(crate) fn decode_projectile_checkpoint(
        bytes: &[u8],
        metadata: &DisplayStartMetadata,
    ) -> Result<sm_game::session::Projectiles, String> {
        sm_game::session::Projectiles::decode_checkpoint(bytes, metadata)
    }
    pub(crate) fn encode_observation_checkpoint(&self) -> Result<Vec<u8>, String> {
        let state = self.0.as_ref().ok_or("no supplied Session observations")?;
        crate::game::session_checkpoint::encode_observations_checkpoint(
            &state.0.observations,
            state.0.metadata(),
        )
    }
    pub(crate) fn decode_observation_checkpoint(
        bytes: &[u8],
        metadata: &DisplayStartMetadata,
    ) -> Result<ExternalInputs, String> {
        crate::game::session_checkpoint::decode_observations_checkpoint(bytes, metadata)
    }
    pub(crate) fn encode_control_checkpoint(&self) -> Result<Vec<u8>, String> {
        let state = self.0.as_ref().ok_or("no supplied Session Control owner")?;
        state
            .0
            .gameplay
            .control
            .encode_checkpoint(&state.0.required)
    }
    pub(crate) fn decode_control_checkpoint(
        bytes: &[u8],
        required: &SessionStartRequirements,
    ) -> Result<Control, String> {
        Control::decode_checkpoint(bytes, required)
    }
    /// Detached binary component only; full machine restore remains unsupported.
    pub(crate) fn encode_audio_checkpoint(&self) -> Result<Vec<u8>, String> {
        let state = self.0.as_ref().ok_or("no supplied Session audio owner")?;
        state
            .0
            .gameplay
            .control
            .requests()
            .map_err(|error| format!("{error:?}"))?
            .encode_checkpoint(state.0.metadata(), &state.0.required.audio_catalog)
    }
    pub(crate) fn decode_audio_checkpoint(
        bytes: &[u8],
        metadata: &DisplayStartMetadata,
        required: &SessionStartRequirements,
    ) -> Result<AudioRequests, String> {
        AudioRequests::decode_checkpoint(bytes, metadata, required.audio_catalog.clone())
    }
    pub(crate) fn is_owned(&self) -> bool {
        self.0.is_some()
    }
    /// Apply only a caller-declared normalization transfer from its actual branch.
    /// Numeric projections carry no permission to manufacture content or history.
    pub(crate) fn normalize_from(
        &mut self,
        source: &Self,
        range: std::ops::Range<usize>,
    ) -> Result<(), DisplayError> {
        match (&mut self.0, &source.0) {
            (None, None) => Ok(()),
            (Some(target), Some(source)) => {
                if !Arc::ptr_eq(&target.0.gameplay.world, &source.0.gameplay.world) {
                    return Err(DisplayError::Identity);
                }
                for at in range.clone() {
                    if let Some(byte) =
                        crate::game::session_encoding::projected_byte(&source.0.gameplay, at)
                    {
                        if !crate::game::session_encoding::write_projected_byte(
                            &mut target.0.gameplay,
                            at,
                            byte,
                        ) {
                            return Err(DisplayError::Incomplete);
                        }
                    }
                    match at {
                        0x611 => {
                            target.0.metadata_mut().continuation.outer =
                                source.0.metadata().continuation.outer
                        }
                        0x612 => {
                            target.0.metadata_mut().continuation.primary =
                                source.0.metadata().continuation.primary
                        }
                        0x613 => {
                            target.0.metadata_mut().continuation.secondary =
                                source.0.metadata().continuation.secondary
                        }
                        0x614 => {
                            target.0.metadata_mut().continuation.music =
                                source.0.metadata().continuation.music
                        }
                        0x615 => {
                            target.0.metadata_mut().continuation.message =
                                source.0.metadata().continuation.message
                        }
                        0x616 => {
                            target.0.metadata_mut().continuation.auxiliary =
                                source.0.metadata().continuation.auxiliary
                        }
                        _ => {}
                    }
                }
                match (&mut target.0.gameplay.control, &source.0.gameplay.control) {
                    (Control::Owned(target), Control::Owned(source)) => {
                        target.normalize_audio_from(source, &range)
                    }
                    _ => Err(DisplayError::Incomplete),
                }
            }
            _ => Err(DisplayError::Incomplete),
        }
    }
    pub(crate) fn final_continuation(&mut self, at: usize, value: u8) -> Result<(), DisplayError> {
        let Some(state) = &mut self.0 else {
            return Ok(());
        };
        match at {
            0x611 => state.0.metadata_mut().continuation.outer = value,
            0x612 => {
                state.0.metadata_mut().continuation.primary = value;
            }
            0x613 => {
                state.0.metadata_mut().continuation.secondary = value;
            }
            0x614 => state.0.metadata_mut().continuation.music = value,
            0x615 => state.0.metadata_mut().continuation.message = value,
            0x616 => state.0.metadata_mut().continuation.auxiliary = value,
            _ => return Err(DisplayError::Incomplete),
        }
        Ok(())
    }
    pub(crate) fn required(&self) -> Option<&Arc<SessionStartRequirements>> {
        self.0.as_ref().map(|state| &state.0.required)
    }
}

use crate::game::scene::{samus, samus_body, samus_position_tiles as positions};
use sm_game::scene::samus_content::{PreparedBytes, Registration, Role, RoleLease};
use sm_game::scene::state::{DisplayState, WordOrder};

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
        let point = crate::game::visual_runtime::project_cinematic_point_reads(
            crate::types::Point16U { x, y },
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
