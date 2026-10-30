//! Moved to `sm_game::player_collision`; verifier adapters stay with the compat crate.
#![forbid(unsafe_code)]
pub(crate) use sm_game::player_collision::*;

#[cfg(sm_verify_dll)]
#[path = "../../verification/player_collision_entry_adapters.rs"]
pub(crate) mod verification;
