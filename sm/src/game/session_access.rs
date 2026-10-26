//! Shared access implemented once by the aggregate runtime and composed harness.
#![forbid(unsafe_code)]
use crate::game::{external_inputs::ExternalInputs, session::Session};
pub trait SessionAccess {
    fn session(&self) -> &Session;
    fn session_mut(&mut self) -> &mut Session;
    fn inputs(&self) -> &ExternalInputs;
    fn probe(&self, name: &'static str);
}
