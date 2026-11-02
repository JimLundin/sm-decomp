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
    pub actor_content:
        Option<std::rc::Rc<crate::actors::presentation_content::PreparedActorContent>>,
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
    UninitializedGameLoop,
    UninitializedMenuWindow,
    MissingGameplay,
    MissingObservations,
    MissingRoles,
    MissingBodyMarker,
    MissingActorQueueProjection,
    MissingSamusPublications,
    MissingAudioRequests,
    MissingActorContent,
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
    actor_import: Option<crate::actors::presentation_content::ActorImportRequest>,
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
            actor_import: None,
            description,
            media,
            required,
        }
    }
    /// Content-load boundary supplies two independently authored actor manifests.
    pub fn with_actor_import(
        mut self,
        request: crate::actors::presentation_content::ActorImportRequest,
    ) -> Self {
        self.actor_import = Some(request);
        self
    }
    pub fn take_actor_import(
        &mut self,
    ) -> Option<crate::actors::presentation_content::ActorImportRequest> {
        self.actor_import.take()
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

/// Complete unpublished target admission. Construction consumes supplied state;
/// opening only moves owners already checked against the independent contract.
pub struct ValidatedSessionStart(PersistentSession);
impl ValidatedSessionStart {
    pub fn construct(
        mut supplied: SuppliedSessionStart,
        world: &Arc<crate::room_entry::world::WorldBinding>,
    ) -> Result<Self, SessionStartError> {
        // Tools must consume raw import requests; the engine admits prepared content.
        if supplied.take_actor_import().is_some() {
            return Err(SessionStartError::MissingActorContent);
        }
        let (supplied, media, required) = supplied.into_parts();
        if let Some(content) = &supplied.actor_content {
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
        let required = &required;
        let mut gameplay = supplied
            .gameplay
            .ok_or(SessionStartError::MissingGameplay)?;
        if gameplay.actor_identity.is_poisoned() {
            return Err(SessionStartError::Binding(DisplayError::Poisoned));
        }
        if !gameplay.fx.is_initialized() {
            return Err(SessionStartError::Binding(DisplayError::Incomplete));
        }
        let was_owned = gameplay.presentation.is_owned();
        crate::fx_encoding::validate_control_ownership(&gameplay)
            .map_err(SessionStartError::Binding)?;
        if gameplay.prepared_dialogue.is_some() != required.dialogue.is_some() {
            return Err(SessionStartError::GameplayDisplay);
        }
        if !gameplay.menu_window.is_initialized() || !gameplay.timing.countdown.initialized {
            return Err(SessionStartError::UninitializedMenuWindow);
        }
        if !gameplay.game_loop.is_initialized() {
            return Err(SessionStartError::UninitializedGameLoop);
        }
        if !Arc::ptr_eq(&gameplay.world, world) || gameplay.world.revision() != world.revision() {
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
        if let Some(execution) = &gameplay.prepared_dialogue {
            required
                .dialogue
                .as_ref()
                .ok_or(SessionStartError::GameplayDisplay)?
                .validate(execution, &display.workspace)
                .map_err(|_| SessionStartError::GameplayDisplay)?;
        }
        if *gameplay.presentation.visual() != display.visual
            || *gameplay.presentation.palette() != display.player_palette
            || *gameplay.presentation.atmospheric() != display.atmospheric
        {
            return Err(SessionStartError::GameplayDisplay);
        }
        if let Presentation::Owned {
            display: existing, ..
        } = &gameplay.presentation
            && existing.owner().state != *display
        {
            return Err(SessionStartError::GameplayDisplay);
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
            let low = crate::session_encoding::projected_byte(&gameplay, at)
                .ok_or(SessionStartError::ActorQueues)?;
            let high = crate::session_encoding::projected_byte(&gameplay, at + 1)
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
            for (index, (covered, actor)) in covered[layout.first..end]
                .iter_mut()
                .zip(&display.actor_queues[layer].actors)
                .enumerate()
            {
                if *covered || queue_word(0xe84 + (layout.first + index) * 2)? != *actor {
                    return Err(SessionStartError::ActorQueues);
                }
                *covered = true;
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
        media
            .validate()
            .map_err(|_| SessionStartError::Binding(DisplayError::Scene))?;
        catalog
            .validate(&media)
            .map_err(SessionStartError::Binding)?;
        catalog
            .validate_expected(&required.display.samus_content)
            .map_err(SessionStartError::Binding)?;
        let mut requests = required.roles.requests.iter();
        crate::scene::samus_content::ValidatedContent::construct(
            roles.clone(),
            &required.roles,
            |request| {
                request.validate(
                    requests
                        .next()
                        .ok_or(DisplayError::Incomplete)?
                        .request
                        .pointer,
                    &media,
                )
            },
        )
        .map_err(SessionStartError::Binding)?;
        let opened = DisplayComponent::open(
            ValidatedDisplayStart::construct(supplied.display, media, &required.display)
                .map_err(SessionStartError::Display)?,
        )
        .map_err(SessionStartError::Display)?;
        if crate::controller_state::Word::ALL.iter().any(|word| {
            word.get(&gameplay.control) != word.get_history(&opened.metadata.input_history)
        }) {
            return Err(SessionStartError::GameplayDisplay);
        }
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
        gameplay.presentation = Presentation::Owned {
            display: Box::new(opened.display),
            marker,
            actor_queue_projection: projection,
            samus_publications: publications,
        };
        gameplay.control = Control::Owned(Box::new(CurrentControl {
            metadata: opened.metadata,
            requests: audio_requests,
        }));
        gameplay.fx_sources = None;
        if !was_owned {
            gameplay.hdma = Default::default();
            crate::fx_encoding::adopt_control_ownership(&mut gameplay)
                .map_err(SessionStartError::Binding)?;
        }
        let mut session = PersistentSession {
            actor_content: supplied.actor_content,
            gameplay,
            observations,
            roles,
            required: Arc::new(required.clone()),
        };
        if let Some(content) = session.actor_content.clone() {
            content
                .admit(&mut session.gameplay)
                .map_err(SessionStartError::Binding)?;
        }
        session.checkpoint().map_err(SessionStartError::Binding)?;
        Ok(Self(session))
    }
    pub fn open(self) -> PersistentSession {
        self.0
    }
}

/// The one live gameplay owner. Runtime moves an exclusive lease of this value;
/// it never decodes a second Session while this owner exists.
#[derive(Clone)]
pub struct PersistentSession {
    pub actor_content:
        Option<std::rc::Rc<crate::actors::presentation_content::PreparedActorContent>>,
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
    /// Reopen a complete checkpoint without running or replaying an interval.
    /// External actor handles receive the current owner's next epoch, never rewind.
    pub fn reopen(
        &mut self,
        checkpoint: SessionCheckpoint,
        contract: &SessionStartContract,
    ) -> Result<(), SessionStartError> {
        let epoch = self
            .gameplay
            .actor_identity
            .checked_next_epoch()
            .map_err(|_| SessionStartError::Binding(DisplayError::Identity))?;
        checkpoint
            .0
            .checkpoint()
            .map_err(SessionStartError::Binding)?;
        if session_required_declarations(&checkpoint.0.required)
            .map_err(SessionStartError::Binding)?
            != session_required_declarations(contract.requirements())
                .map_err(SessionStartError::Binding)?
        {
            return Err(SessionStartError::Binding(DisplayError::Identity));
        }
        let (description, media) = checkpoint
            .into_start()
            .map_err(SessionStartError::Binding)?;
        let mut replacement = ValidatedSessionStart::construct(
            SuppliedSessionStart::new(description, media, contract.requirements().clone()),
            &self.gameplay.world,
        )?
        .open();
        let timeline = replacement.gameplay.actor_identity.timeline().clone();
        let residency = replacement
            .gameplay
            .actor_identity
            .palette_residency()
            .clone();
        let mut identity = self.gameplay.actor_identity.clone();
        identity.replace_timeline(epoch, timeline, residency);
        replacement.gameplay.actor_identity = identity;
        *self = replacement;
        Ok(())
    }
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
        if !self.gameplay.game_loop.is_initialized()
            || !self.gameplay.menu_window.is_initialized()
            || !self.gameplay.timing.countdown.initialized
        {
            return Err(DisplayError::Incomplete);
        }
        self.gameplay
            .presentation
            .owner_mut()?
            .begin_interval(fresh_entry)
    }
    pub fn checkpoint(&self) -> Result<SessionCheckpoint, DisplayError> {
        if let Some(content) = &self.actor_content {
            content.validate_request(
                self.required
                    .actor_manifest
                    .as_ref()
                    .ok_or(DisplayError::Incomplete)?,
            )?;
            content.check(&self.gameplay)?;
        } else if self.gameplay.world.actors().revision().is_some()
            || self.required.actor_manifest.is_some()
        {
            return Err(DisplayError::Incomplete);
        }
        crate::fx_encoding::validate_control_ownership(&self.gameplay)?;
        if !self.gameplay.game_loop.is_initialized()
            || !self.gameplay.menu_window.is_initialized()
            || !self.gameplay.timing.countdown.initialized
        {
            return Err(DisplayError::Incomplete);
        }
        self.gameplay.presentation.owner()?.checkpoint_parts()?;
        if self.gameplay.prepared_dialogue.is_some() != self.required.dialogue.is_some() {
            return Err(DisplayError::Incomplete);
        }
        if let Some(execution) = &self.gameplay.prepared_dialogue {
            self.required
                .dialogue
                .as_ref()
                .ok_or(DisplayError::Incomplete)?
                .validate(execution, self.gameplay.presentation.workspace())
                .map_err(|_| DisplayError::Incomplete)?;
        }
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
    /// Commit-time presentation uses the actual live Session queue and workspace.
    /// Input admission/late settlement retains the existing Display phase contract.
    pub fn refresh_display(&mut self) -> Result<(), DisplayError> {
        crate::scene::commit::refresh_session(&mut self.gameplay)
    }
    /// Admit one supplied input boundary with the existing capture and policy.
    pub fn begin_input_interval(
        &mut self,
        record: u64,
        input: crate::scene::start::InputHistory,
        fresh_entry: bool,
    ) -> Result<(), DisplayError> {
        let metadata = self.metadata();
        if record != metadata.identity.next_input_record
            || record != metadata.continuation.next_input_record
            || input.capture != metadata.input_history.capture
            || input.policy != metadata.input_history.policy
        {
            return Err(DisplayError::Identity);
        }
        let next = record.checked_add(1).ok_or(DisplayError::Identity)?;
        self.begin_interval(fresh_entry)?;
        let metadata = self.metadata_mut();
        metadata.input_history = input;
        metadata.identity.next_input_record = next;
        metadata.continuation.next_input_record = next;
        Ok(())
    }
    /// One invocation of the existing suspended runner on the current workspace.
    pub fn resume_dialogue<Frame>(
        &mut self,
        effects: &mut impl crate::prepared_dialogue::Effects,
        render: impl FnMut(sm_data::message::Display<'_>) -> Result<Frame, sm_data::message::Fault>,
    ) -> Result<crate::message_composition::prepared::StepOutput<Frame>, sm_data::message::Fault>
    {
        use sm_data::message::Fault;
        match self
            .gameplay
            .presentation
            .owner()
            .map(|owner| owner.state.phase)
        {
            Ok(crate::scene::state::DisplayPhase::Failed) => return Err(Fault::Poisoned),
            Ok(crate::scene::state::DisplayPhase::Producing) => {}
            _ => return Err(Fault::Interval),
        }
        let content = match self.required.dialogue.as_ref() {
            Some(content) => content.clone(),
            None => {
                self.poison();
                return Err(Fault::MissingDisplay);
            }
        };
        let interval = self.metadata().identity.next_input_record;
        let input = self.metadata().input_history;
        let sample = crate::message_composition::prepared::InputSample {
            new_keys: input.ports[0].pressed,
            held_keys: input.ports[0].sampled,
            bug_fix_counter: input.bug_fix_counter,
        };
        let current = match self.gameplay.prepared_dialogue.as_ref() {
            Some(current) => current,
            None => {
                self.poison();
                return Err(Fault::MissingDisplay);
            }
        };
        let request = if current.continuation == crate::dialogue::Continuation::Start {
            if self.gameplay.messages.queued == 0 {
                None
            } else {
                match content.native_messages.get(&self.gameplay.messages.queued) {
                    Some(id) => Some(*id),
                    None => {
                        self.poison();
                        return Err(Fault::Definition);
                    }
                }
            }
        } else {
            current.request
        };
        let mut execution = self
            .gameplay
            .prepared_dialogue
            .take()
            .ok_or(Fault::MissingDisplay)?;
        execution.request = request;
        execution.delay = self.gameplay.messages.delay;
        let mut workspace = self.gameplay.presentation.workspace_mut().take();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            content.resume(
                &mut execution,
                interval,
                sample,
                &mut SessionDialogue {
                    session: self,
                    content: &content,
                    effects,
                },
                &mut workspace,
                render,
            )
        }));
        if workspace.is_initialized() {
            *self.gameplay.presentation.workspace_mut() = workspace;
        }
        if self.gameplay.prepared_dialogue.is_none() {
            self.gameplay.prepared_dialogue = Some(execution);
        }
        match result {
            Ok(result) => {
                let published = crate::prepared_dialogue::publish(&mut self.gameplay, &content);
                if result.as_ref().is_ok_and(|output| {
                    output.outcome == crate::message_composition::prepared::Outcome::Complete
                }) {
                    self.gameplay.messages.queued = 0;
                }
                if result.is_err() || published.is_err() {
                    self.poison();
                }
                published?;
                result
            }
            Err(payload) => {
                self.poison();
                std::panic::resume_unwind(payload)
            }
        }
    }
    /// Refresh, settle the real owned interval, then publish immutable Scene history.
    pub fn finish_interval(
        &mut self,
        effects: &mut impl crate::scene::commit::SettlementEffects,
        description: crate::scene::records::SceneDescription,
        media: Arc<SceneMedia>,
    ) -> Result<crate::scene::records::Scene, DisplayError> {
        self.finish_interval_with_description(effects, |_, _, _, _| Ok((description, media)))
    }
    /// Extract immutable history after actual row callbacks, before commitment.
    /// Read-only current state/media and prior visible history are explicit inputs.
    pub fn finish_interval_with_description<E: crate::scene::commit::SettlementEffects>(
        &mut self,
        effects: &mut E,
        publish: impl FnOnce(
            &mut E,
            &crate::scene::state::DisplayState,
            &crate::scene::records::Scene,
            &Arc<SceneMedia>,
        ) -> Result<
            (crate::scene::records::SceneDescription, Arc<SceneMedia>),
            DisplayError,
        >,
    ) -> Result<crate::scene::records::Scene, DisplayError> {
        self.refresh_display()?;
        let result = (|| {
            self.gameplay.presentation.owner_mut()?.begin_settlement()?;
            self.gameplay.settle_display(effects)?;
            let owner = self.gameplay.presentation.owner()?;
            let (description, media) =
                publish(effects, &owner.state, owner.committed(), owner.media())?;
            self.gameplay
                .presentation
                .owner_mut()?
                .commit_scene(description, media)
                .cloned()
        })();
        match result {
            Ok(scene) => {
                let metadata = self.metadata_mut();
                metadata.identity.committed_tick = scene.tick();
                metadata.continuation.phase = crate::scene::start::CommittedPhase::SettledInterval;
                Ok(scene)
            }
            Err(error) => {
                self.poison();
                Err(error)
            }
        }
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
        let Presentation::Owned { display, .. } = &state.gameplay.presentation else {
            return Err("no owned Session display checkpoint".into());
        };
        let declarations = session_required_declarations(&state.required)
            .map_err(|error| format!("Session immutable declarations {error:?}"))?;
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
        if session_binary_component(&mut read)?
            != session_required_declarations(&required)
                .map_err(|error| format!("Session immutable declarations {error:?}"))?
        {
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
        crate::fx_encoding::validate_control_ownership(&gameplay)
            .map_err(|error| format!("restored control ownership {error:?}"))?;
        Ok(SuppliedSessionStart::new(
            SessionStartDescription {
                gameplay: Some(gameplay),
                observations: Some(observations),
                display: display_start,
                actor_content: None,
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
            actor_content,
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
                actor_content,
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

fn session_required_declarations(
    required: &SessionStartRequirements,
) -> Result<Vec<u8>, DisplayError> {
    let mut declarations = crate::scene::checkpoint_codec::session_declarations(
        &required.roles,
        &required.marker,
        &required.actor_queue_projection,
        &required.samus_publications,
    );
    declarations.push(u8::from(required.dialogue.is_some()));
    if let Some(content) = &required.dialogue {
        declarations.extend_from_slice(&content.declarations());
    }
    if let Some(request) = &required.actor_manifest {
        let content = request.declarations()?;
        declarations.extend_from_slice(b"SMACTORCONTRACT\x01");
        declarations.extend_from_slice(
            &u64::try_from(content.len())
                .map_err(|_| DisplayError::Source)?
                .to_le_bytes(),
        );
        declarations.extend_from_slice(&content);
    }
    Ok(declarations)
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
                if (range.start < 0x616 && range.end > 0x614)
                    || (range.start <= crate::presentation_workspace::FIRST
                        && range.end
                            >= crate::presentation_workspace::FIRST
                                + crate::presentation_workspace::BYTES)
                {
                    if target
                        .0
                        .required
                        .dialogue
                        .as_ref()
                        .map(|content| content.declarations())
                        != source
                            .0
                            .required
                            .dialogue
                            .as_ref()
                            .map(|content| content.declarations())
                    {
                        return Err(DisplayError::Identity);
                    }
                    target.0.gameplay.prepared_dialogue =
                        source.0.gameplay.prepared_dialogue.clone();
                    if target.0.gameplay.prepared_dialogue.is_some() {
                        *target.0.gameplay.presentation.workspace_mut() =
                            source.0.gameplay.presentation.workspace().clone();
                    }
                }
                if range.start <= 0xd0 && range.end >= 0x2d0 {
                    target.0.gameplay.queued_tilemaps.native_order =
                        source.0.gameplay.queued_tilemaps.native_order;
                } else if range.start < 0x2d0
                    && range.end > 0xd0
                    && target.0.gameplay.queued_tilemaps.native_order
                        != source.0.gameplay.queued_tilemaps.native_order
                {
                    return Err(DisplayError::Identity);
                }
                if range.start
                    < crate::presentation_workspace::FIRST + crate::presentation_workspace::BYTES
                    && range.end > crate::presentation_workspace::FIRST
                {
                    target
                        .0
                        .gameplay
                        .presentation
                        .workspace_mut()
                        .normalize_from(
                            source.0.gameplay.presentation.workspace(),
                            range.clone(),
                        )?;
                }
                if range.start <= 0x945 && range.end >= 0x94c {
                    target.0.gameplay.timing.countdown = source.0.gameplay.timing.countdown;
                } else if range.start < 0x94c
                    && range.end > 0x945
                    && (target.0.gameplay.timing.countdown.native_order
                        != source.0.gameplay.timing.countdown.native_order
                        || target.0.gameplay.timing.countdown.initialized
                            != source.0.gameplay.timing.countdown.initialized)
                {
                    return Err(DisplayError::Identity);
                }
                for at in range.clone() {
                    crate::fx_encoding::note_source_normalization(&mut target.0.gameplay, at)?;
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

struct SessionDialogue<'a, E> {
    session: &'a mut PersistentSession,
    content: &'a crate::prepared_dialogue::Content,
    effects: &'a mut E,
}
impl<E> crate::presentation_workspace::Publication for SessionDialogue<'_, E> {
    fn published(
        &mut self,
        workspace: &crate::presentation_workspace::Workspace,
        first: crate::presentation_workspace::SourcePoint,
        bytes: usize,
    ) {
        let display = &mut self
            .session
            .gameplay
            .presentation
            .owner_mut()
            .expect("current dialogue display")
            .state;
        crate::fx_sources::forget_workspace_range(
            &mut display.hdma_live,
            &display.hdma,
            workspace,
            first,
            bytes,
        );
    }
}
impl<E: crate::prepared_dialogue::Effects> crate::message_composition::prepared::WorkspaceEffects
    for SessionDialogue<'_, E>
{
    fn wait(
        &mut self,
        workspace: &mut crate::presentation_workspace::Workspace,
        published: &mut crate::message_composition::prepared::ExecutionDescription,
    ) -> Result<crate::dialogue::Outcome, sm_data::message::Fault> {
        *self.session.gameplay.presentation.workspace_mut() = workspace.take();
        self.session.gameplay.prepared_dialogue = Some(published.clone());
        let result = crate::prepared_dialogue::publish(&mut self.session.gameplay, self.content)
            .and_then(|()| self.effects.wait(&mut self.session.gameplay));
        *workspace = self.session.gameplay.presentation.workspace_mut().take();
        if let Some(selected) = self.session.gameplay.prepared_dialogue.take() {
            *published = selected;
        } else {
            return Err(sm_data::message::Fault::MissingDisplay);
        }
        result
    }
    fn input(
        &mut self,
        workspace: &mut crate::presentation_workspace::Workspace,
        published: &mut crate::message_composition::prepared::ExecutionDescription,
        sample: crate::message_composition::prepared::InputSample,
    ) -> Result<crate::message_composition::prepared::InputSample, sm_data::message::Fault> {
        *self.session.gameplay.presentation.workspace_mut() = workspace.take();
        self.session.gameplay.prepared_dialogue = Some(published.clone());
        let result = crate::prepared_dialogue::publish(&mut self.session.gameplay, self.content)
            .and_then(|()| self.effects.input_owner(self.session, sample));
        *workspace = self.session.gameplay.presentation.workspace_mut().take();
        if let Some(selected) = self.session.gameplay.prepared_dialogue.take() {
            *published = selected;
        } else {
            return Err(sm_data::message::Fault::MissingDisplay);
        }
        result
    }
    fn effect(
        &mut self,
        effect: crate::message_composition::prepared::Effect,
        workspace: &mut crate::presentation_workspace::Workspace,
        published: &mut crate::message_composition::prepared::ExecutionDescription,
    ) -> Result<(), sm_data::message::Fault> {
        *self.session.gameplay.presentation.workspace_mut() = workspace.take();
        self.session.gameplay.prepared_dialogue = Some(published.clone());
        let result = crate::prepared_dialogue::publish(&mut self.session.gameplay, self.content)
            .and_then(|()| self.effects.apply_owner(effect, self.session));
        *workspace = self.session.gameplay.presentation.workspace_mut().take();
        if let Some(mut selected) = self.session.gameplay.prepared_dialogue.take() {
            selected.delay = self.session.gameplay.messages.delay;
            *published = selected;
        } else {
            return Err(sm_data::message::Fault::MissingDisplay);
        }
        result
    }
}
