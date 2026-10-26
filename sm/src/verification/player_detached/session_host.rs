//! One access implementation for all three actual gameplay Hosts.
use super::{
    Composed,
    game::{external_inputs::ExternalInputs, session::Session, session_access::SessionAccess},
};
impl SessionAccess for Composed {
    fn session(&self) -> &Session {
        &self.world.player
    }
    fn session_mut(&mut self) -> &mut Session {
        &mut self.world.player
    }
    fn inputs(&self) -> &ExternalInputs {
        &self.world.player_inputs
    }
    fn probe(&self, _: &'static str) {}
}
