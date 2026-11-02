//! Prepared presentation coordination without a native backend or Game RAM.
#![forbid(unsafe_code)]
use sm_game::{
    persistent_session::PersistentSession,
    scene::{
        commit::SettlementEffects,
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
    effects: &mut impl SettlementEffects,
    description: SceneDescription,
    media: Arc<SceneMedia>,
) -> Result<sm_runtime::scene::SceneFrame, Failure> {
    let scene = session
        .finish_interval(effects, description, media)
        .map_err(Failure::Game)?;
    sm_runtime::scene::render(&scene).map_err(Failure::Render)
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
