//! Shared access implemented once by the aggregate runtime and composed harness.
use crate::{external_inputs::ExternalInputs, session::Session};
pub trait SessionAccess {
    fn session(&self) -> &Session;
    fn session_mut(&mut self) -> &mut Session;
    /// The active host routes the four slots to its single current owner.
    fn atmospheric(&self) -> &crate::player_transitions::state::AtmosphericEffects {
        self.session().presentation.atmospheric()
    }
    fn atmospheric_mut(&mut self) -> &mut crate::player_transitions::state::AtmosphericEffects {
        self.session_mut().presentation.atmospheric_mut()
    }
    /// Native compatibility has no semantic allocator; owned G overrides this
    /// after the producer's completed stores and preserves failures in its owner.
    fn atmospheric_created(
        &mut self,
        _slot: usize,
    ) -> Result<(), crate::scene::state::DisplayError> {
        Ok(())
    }
    fn inputs(&self) -> &ExternalInputs;
    /// Refresh the external observations from the Session after stores to
    /// fields they read (the cinematic function, the FX blending, the frozen
    /// time flag, ...); a host that keeps none does nothing.
    fn refresh_inputs(&mut self) {}
    fn probe(&self, name: &'static str);
}
