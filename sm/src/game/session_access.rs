//! Shared access implemented once by the aggregate runtime and composed harness.
#![forbid(unsafe_code)]
use crate::game::{external_inputs::ExternalInputs, session::Session};
pub trait SessionAccess {
    fn session(&self) -> &Session;
    fn session_mut(&mut self) -> &mut Session;
    /// The active host routes the four slots to its single current owner.
    fn atmospheric(&self) -> &crate::game::player_transitions::state::AtmosphericEffects {
        &self.session().atmospheric
    }
    fn atmospheric_mut(
        &mut self,
    ) -> &mut crate::game::player_transitions::state::AtmosphericEffects {
        &mut self.session_mut().atmospheric
    }
    /// Native compatibility has no semantic allocator; owned G overrides this
    /// after the producer's completed stores and preserves failures in its owner.
    fn atmospheric_created(
        &mut self,
        _slot: usize,
    ) -> Result<(), crate::game::scene::state::DisplayError> {
        Ok(())
    }
    fn inputs(&self) -> &ExternalInputs;
    fn probe(&self, name: &'static str);
}
