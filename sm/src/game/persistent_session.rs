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

pub(crate) use sm_game::scene::owned_samus::{bottom_visible, draw_body, draw_sprite};

/// Compatibility residency, with one independently revocable inventory owner.
/// The aggregate's current behavior and admission rules remain unchanged.
pub struct SessionSlot {
    pub(crate) aggregate: AggregateSlot,
    pub(crate) menu: std::cell::RefCell<sm_game::menus::state::Domain>,
    pub(crate) inventory: std::cell::RefCell<sm_game::inventory_domain::InventoryDomain>,
    pub(crate) scroll: std::cell::RefCell<sm_game::display_scroll::ScrollDomain>,
}

pub(crate) enum AggregateSlot {
    Legacy,
    Ready(Box<PersistentSession>),
    Leased,
    Failed(Option<Box<PersistentSession>>),
}

impl SessionSlot {
    pub(crate) fn new() -> Self {
        Self {
            aggregate: AggregateSlot::Legacy,
            menu: std::cell::RefCell::new(sm_game::menus::state::Domain::Unowned),
            inventory: std::cell::RefCell::new(sm_game::inventory_domain::InventoryDomain::Unowned),
            scroll: std::cell::RefCell::new(sm_game::display_scroll::ScrollDomain::Unowned),
        }
    }
}
