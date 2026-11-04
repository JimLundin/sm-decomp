//! Animated tiles compose on the shared Session at each legacy entry.
use crate::{game::session_runtime::Runtime, game_state::Game};
pub(crate) use execution::leased::{advance, clear, set_enabled, spawn, upload_pending};
pub(crate) use sm_game::animated_tiles::execution;
#[path = "../../verification/animated_entries.rs"]
pub(crate) mod checks;
