//! Original-allocation capture at the actual supplied Session startup boundary.
#![forbid(unsafe_code)]
use crate::{
    game::scene::records::SceneMedia,
    game_state::{
        Game,
        persistent_session::{self, PreparedSessionStart, SessionStartError, SuppliedSessionStart},
    },
};
use sm_game::{actors::presentation_content::SourceAllocations, scene::state::DisplayError};
use std::sync::Arc;

fn sources(game: &Game) -> Result<SourceAllocations, SessionStartError> {
    Ok(SourceAllocations {
        loaded: game
            .loaded_rom
            .clone()
            .ok_or(SessionStartError::Binding(DisplayError::Source))?,
        native: game
            .capture_native_rom_binding()
            .ok_or(SessionStartError::Binding(DisplayError::Source))?,
    })
}
/// Normal boot/pending-start path consumes independently authored actor manifests.
/// Captures retain the actual registered original allocations, not copied images.
pub(crate) fn prepare_supplied(
    game: &Game,
    mut supplied: SuppliedSessionStart,
) -> Result<PreparedSessionStart, SessionStartError> {
    let request = supplied.take_actor_import();
    let (description, media, required) = supplied.into_parts();
    let actor_start = request.is_some()
        || required.actor_manifest.is_some()
        || game.world().actors().revision().is_some();
    if !actor_start {
        return persistent_session::prepare(game, description, media, &required);
    }
    let mut supplied = SuppliedSessionStart::new(description, media, required);
    if let Some(request) = request {
        supplied = supplied.with_actor_import(request);
    }
    let prepared =
        sm_tools::actor_content::prepare_session(supplied, game.world(), sources(game)?)?;
    let (description, media, required) = prepared.into_parts();
    persistent_session::prepare(game, description, media, &required)
}
/// Reopen/restore uses the retained independent contract and actual current origins.
/// A changed allocation needs fresh explicit import, not matching-byte adoption.
pub(crate) fn validate_retained(
    game: &Game,
    media: &Arc<SceneMedia>,
    actor_content: &Option<
        std::rc::Rc<sm_game::actors::presentation_content::PreparedActorContent>,
    >,
    required: &persistent_session::SessionStartRequirements,
) -> Result<(), SessionStartError> {
    if let Some(content) = actor_content {
        content
            .validate_request(
                required
                    .actor_manifest
                    .as_ref()
                    .ok_or(SessionStartError::MissingActorContent)?,
            )
            .map_err(SessionStartError::Binding)?;
        content
            .check_sources(&sources(game)?)
            .map_err(SessionStartError::Binding)?;
        if content.manifest().world != game.world().revision()
            || content.manifest().scene_media != media.revision()
        {
            return Err(SessionStartError::Binding(DisplayError::Revision));
        }
    } else if game.world().actors().revision().is_some() || required.actor_manifest.is_some() {
        return Err(SessionStartError::MissingActorContent);
    }
    Ok(())
}
