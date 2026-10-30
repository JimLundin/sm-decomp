//! The checkpoint codec lives in `sm_game::session_checkpoint`; verification cases stay with the compat crate.
#[cfg(sm_verify_dll)]
use sm_game::{scene::component::DisplayStartMetadata, session::Session};

#[cfg(sm_verify_dll)]
#[path = "../verification/gameplay_checkpoint_cases.rs"]
mod gameplay_checkpoint_cases;
#[cfg(sm_verify_dll)]
pub(crate) use gameplay_checkpoint_cases::assert_gameplay_checkpoint;
