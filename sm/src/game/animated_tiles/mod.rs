//! Animated tiles compose on the shared Session at each legacy entry.
pub(crate) use sm_game::animated_tiles::execution;
pub(crate) mod native;
use crate::{game::session_runtime::Runtime, game_state::Game};
pub(crate) use execution::leased::{advance, clear, set_enabled, spawn, upload_pending};
#[path = "../../verification/animated_entries.rs"]
pub(crate) mod checks;
