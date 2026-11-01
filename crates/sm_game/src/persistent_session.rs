//! The exclusive gameplay owner: Session start descriptions, the persistent
//! owner and its intervals, semantic checkpoints and snapshot payloads.

use crate::actors::snapshot::CheckpointImages;
use std::sync::Arc;

use crate::external_inputs::ExternalInputs;
use crate::session::Session;
use crate::session_owners::*;
use {
    crate::scene::records::SceneMedia,
    crate::scene::{
        component::{DisplayComponent, DisplayStartMetadata},
        samus_body::MarkerPublication,
        samus_content::ContentDescription,
        start::{DisplayStartDescription, StartError, ValidatedDisplayStart},
        state::DisplayError,
    },
};

pub struct SessionStartDescription {
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

/// Tools supplies the complete semantic construction and independent contract.
/// The app transports this value without access to mutable gameplay internals.
pub struct SuppliedSessionStart {
    description: SessionStartDescription,
    media: Arc<SceneMedia>,
    required: SessionStartRequirements,
}
pub struct SessionStartContract(SessionStartRequirements);

impl SuppliedSessionStart {
    pub fn new(
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
    pub fn into_parts(
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
    pub fn new(required: SessionStartRequirements) -> Self {
        Self(required)
    }
    pub fn requirements(&self) -> &SessionStartRequirements {
        &self.0
    }
}

/// The one live gameplay owner. Runtime moves an exclusive lease of this value;
/// it never decodes a second Session while this owner exists.
#[derive(Clone)]
pub struct PersistentSession {
    pub gameplay: Session,
    pub observations: ExternalInputs,
    pub roles: ContentDescription,
    pub required: Arc<SessionStartRequirements>,
}

/// Captured current owners, including final resident and separate visible history.
/// This semantic checkpoint does not run an interval or invoke the native loader.
#[derive(Clone)]
pub struct SessionCheckpoint(pub PersistentSession);

impl PersistentSession {
    pub fn metadata(&self) -> &DisplayStartMetadata {
        self.gameplay
            .control
            .metadata()
            .expect("persistent control owner")
    }
    pub fn metadata_mut(&mut self) -> &mut DisplayStartMetadata {
        self.gameplay
            .control
            .metadata_mut()
            .expect("persistent control owner")
    }
    pub fn begin_interval(&mut self, fresh_entry: bool) -> Result<(), DisplayError> {
        self.gameplay
            .presentation
            .owner_mut()?
            .begin_interval(fresh_entry)
    }
    pub fn checkpoint(&self) -> Result<SessionCheckpoint, DisplayError> {
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
    pub fn poison(&mut self) {
        if let Ok(owner) = self.gameplay.presentation.owner_mut() {
            owner.state.phase = crate::scene::state::DisplayPhase::Failed;
        }
        self.gameplay.actor_identity.retain_poison();
    }
}

impl SessionCheckpoint {
    /// Full owned state with immutable declarations sealed against the supplied
    /// contract. Allocation roles come from the actual caller, never defaults.
    pub fn encode_binary(&self, images: CheckpointImages<'_>) -> Result<Vec<u8>, String> {
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
        let declarations = crate::scene::checkpoint_codec::session_declarations(
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
            state
                .gameplay
                .encode_actor_identity_checkpoint(state.metadata(), images)?,
            crate::session_checkpoint::encode_observations_checkpoint(
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
    pub fn decode_binary(
        bytes: &[u8],
        current: &crate::actors::identity::ActorIdentityState,
        world: Arc<crate::room_entry::world::WorldBinding>,
        media: Arc<SceneMedia>,
        required: SessionStartRequirements,
        images: CheckpointImages<'_>,
    ) -> Result<SuppliedSessionStart, String> {
        let mut read = crate::actors::snapshot::CheckpointReader(bytes);
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
            images,
        )?;
        let observations = crate::session_checkpoint::decode_observations_checkpoint(
            session_binary_component(&mut read)?,
            &metadata,
        )?;
        let display = crate::scene::component::DisplayCheckpoint::decode_checkpoint(
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

    pub fn into_start(self) -> Result<(SessionStartDescription, Arc<SceneMedia>), DisplayError> {
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
    crate::scene::checkpoint_codec::session_declarations(
        &required.roles,
        &required.marker,
        &required.actor_queue_projection,
        &required.samus_publications,
    )
}
fn session_binary_component<'a>(
    read: &mut crate::actors::snapshot::CheckpointReader<'a>,
) -> Result<&'a [u8], String> {
    let length = usize::try_from(read.counter()?)
        .map_err(|_| "Session checkpoint component length overflow")?;
    read.bytes(length)
}

/// One complete semantic branch observation, never an alternative live owner.
#[derive(Clone)]
pub struct SessionSnapshotPayload(pub Option<SessionCheckpoint>);

impl SessionSnapshotPayload {
    pub fn encode_gameplay_checkpoint(&self) -> Result<Vec<u8>, String> {
        let state = self.0.as_ref().ok_or("no supplied Session gameplay")?;
        state
            .0
            .gameplay
            .encode_gameplay_checkpoint(state.0.metadata())
    }
    pub fn decode_gameplay_checkpoint(
        bytes: &[u8],
        metadata: &DisplayStartMetadata,
        world: Arc<crate::room_entry::world::WorldBinding>,
    ) -> Result<crate::session_checkpoint::NumericGameplayCheckpoint, String> {
        Session::decode_gameplay_checkpoint(bytes, metadata, world)
    }
    pub fn is_owned(&self) -> bool {
        self.0.is_some()
    }
    /// Apply only a caller-declared normalization transfer from its actual branch.
    /// Numeric projections carry no permission to manufacture content or history.
    pub fn normalize_from(
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
                        crate::session_encoding::projected_byte(&source.0.gameplay, at)
                        && !crate::session_encoding::write_projected_byte(
                            &mut target.0.gameplay,
                            at,
                            byte,
                        )
                    {
                        return Err(DisplayError::Incomplete);
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
    pub fn final_continuation(&mut self, at: usize, value: u8) -> Result<(), DisplayError> {
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
    pub fn required(&self) -> Option<&Arc<SessionStartRequirements>> {
        self.0.as_ref().map(|state| &state.0.required)
    }
}
